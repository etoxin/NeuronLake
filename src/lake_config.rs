// Copyright 2026 Adam Lusted
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LakeConfig {
    pub name: Option<String>,
    #[serde(default)]
    pub experts: Vec<ExpertConfig>,
    #[serde(default)]
    pub server: Option<ServerConfig>,
    #[serde(default)]
    pub teacher: Option<serde_yaml::Value>,
}

impl LakeConfig {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let errors = self.validation_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors::new(errors))
        }
    }

    fn validation_errors(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        require_non_empty(
            self.name.as_deref(),
            "name",
            "lake name is required",
            &mut errors,
        );

        if self.experts.is_empty() {
            errors.push(ValidationError::new(
                "experts",
                "at least one expert must be configured",
            ));
        }

        let mut seen_ids = HashSet::new();
        for (index, expert) in self.experts.iter().enumerate() {
            let id_field = format!("experts[{index}].id");
            if let Some(id) = require_non_empty(
                expert.id.as_deref(),
                &id_field,
                "expert id is required",
                &mut errors,
            ) {
                if !seen_ids.insert(id.to_string()) {
                    errors.push(ValidationError::new(
                        id_field,
                        format!("duplicate expert id '{id}'"),
                    ));
                }
            }

            require_non_empty(
                expert.domain.as_deref(),
                &format!("experts[{index}].domain"),
                "expert domain is required",
                &mut errors,
            );

            match &expert.model {
                Some(model) => {
                    if let Err(error) =
                        validate_model_reference(model, &format!("experts[{index}].model"))
                    {
                        errors.push(error);
                    }
                }
                None => errors.push(ValidationError::new(
                    format!("experts[{index}].model"),
                    "expert model reference is required",
                )),
            }
        }

        match &self.server {
            Some(server) => {
                require_non_empty(
                    server.host.as_deref(),
                    "server.host",
                    "server host is required",
                    &mut errors,
                );
                require_non_empty(
                    server.model_name.as_deref(),
                    "server.model_name",
                    "server model_name is required",
                    &mut errors,
                );
                match server.port {
                    Some(1..=65535) => {}
                    Some(port) => errors.push(ValidationError::new(
                        "server.port",
                        format!("server port {port} is outside the valid range 1-65535"),
                    )),
                    None => errors.push(ValidationError::new(
                        "server.port",
                        "server port is required",
                    )),
                }
            }
            None => errors.push(ValidationError::new(
                "server",
                "server settings are required",
            )),
        }

        errors
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExpertConfig {
    pub id: Option<String>,
    pub domain: Option<String>,
    pub model: Option<ModelReferenceConfig>,
    #[serde(default)]
    pub routing_hints: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub sharing: Option<SharingMetadata>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub compatibility: Option<CompatibilityMetadata>,
    #[serde(default)]
    pub training_status: Option<TrainingStatus>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub host: Option<String>,
    pub port: Option<u32>,
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ModelReferenceConfig {
    Shorthand(String),
    Detailed(DetailedModelReference),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DetailedModelReference {
    #[serde(default, alias = "local")]
    pub path: Option<String>,
    #[serde(default)]
    pub remote: Option<String>,
    #[serde(default)]
    pub imported: Option<String>,
    #[serde(default)]
    pub cache_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SharingMetadata {
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CompatibilityMetadata {
    #[serde(default)]
    pub neuronlake: Option<String>,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub model_format: Option<String>,
    #[serde(default)]
    pub quantization: Option<String>,
    #[serde(default)]
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TrainingStatus {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub dataset: Option<String>,
    #[serde(default)]
    pub last_trained_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeServerSettings {
    pub host: String,
    pub port: u16,
    pub model_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpertRegistry {
    lake_name: String,
    server: RuntimeServerSettings,
    experts: Vec<RegistryExpert>,
    experts_by_id: HashMap<String, usize>,
}

impl ExpertRegistry {
    pub fn from_config(
        config: &LakeConfig,
        config_path: impl AsRef<Path>,
    ) -> Result<Self, ValidationErrors> {
        config.validate()?;

        let config_dir = config_path
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let mut experts = Vec::with_capacity(config.experts.len());
        let mut experts_by_id = HashMap::with_capacity(config.experts.len());

        for expert in &config.experts {
            let id = trimmed_required(expert.id.as_deref());
            let index = experts.len();
            let model = resolve_model_reference(
                expert.model.as_ref().expect("validated expert model"),
                config_dir,
            )
            .expect("validated model reference");

            experts_by_id.insert(id.to_string(), index);
            experts.push(RegistryExpert {
                id: id.to_string(),
                domain: trimmed_required(expert.domain.as_deref()).to_string(),
                model,
                routing_hints: expert.routing_hints.clone(),
                examples: expert.examples.clone(),
                sharing: expert.sharing.clone(),
                version: expert.version.clone(),
                compatibility: expert.compatibility.clone(),
                training_status: expert.training_status.clone(),
            });
        }

        let server = config.server.as_ref().expect("validated server settings");

        Ok(Self {
            lake_name: trimmed_required(config.name.as_deref()).to_string(),
            server: RuntimeServerSettings {
                host: trimmed_required(server.host.as_deref()).to_string(),
                port: server.port.expect("validated server port") as u16,
                model_name: trimmed_required(server.model_name.as_deref()).to_string(),
            },
            experts,
            experts_by_id,
        })
    }

    pub fn lake_name(&self) -> &str {
        &self.lake_name
    }

    pub fn server(&self) -> &RuntimeServerSettings {
        &self.server
    }

    pub fn experts(&self) -> &[RegistryExpert] {
        &self.experts
    }

    pub fn get(&self, id: &str) -> Option<&RegistryExpert> {
        self.experts_by_id
            .get(id)
            .and_then(|index| self.experts.get(*index))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryExpert {
    pub id: String,
    pub domain: String,
    pub model: RuntimeModelReference,
    pub routing_hints: Vec<String>,
    pub examples: Vec<String>,
    pub sharing: Option<SharingMetadata>,
    pub version: Option<String>,
    pub compatibility: Option<CompatibilityMetadata>,
    pub training_status: Option<TrainingStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeModelReference {
    pub kind: ModelReferenceKind,
    pub original: String,
    pub resolved_path: Option<PathBuf>,
    pub cache_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelReferenceKind {
    Local,
    Remote,
    Imported,
}

#[derive(Debug)]
pub enum LakeConfigLoadError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    Validation(ValidationErrors),
}

impl fmt::Display for LakeConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "failed to read lake config: {error}"),
            Self::Yaml(error) => write!(f, "failed to parse lake config YAML: {error}"),
            Self::Validation(errors) => write!(f, "{errors}"),
        }
    }
}

impl std::error::Error for LakeConfigLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Yaml(error) => Some(error),
            Self::Validation(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for LakeConfigLoadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_yaml::Error> for LakeConfigLoadError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Yaml(error)
    }
}

impl From<ValidationErrors> for LakeConfigLoadError {
    fn from(error: ValidationErrors) -> Self {
        Self::Validation(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }

    pub fn contains_field(&self, field: &str) -> bool {
        self.errors.iter().any(|error| error.field == field)
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "lake config validation failed with {} error(s):",
            self.errors.len()
        )?;
        for error in &self.errors {
            writeln!(f, "- {}: {}", error.field, error.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

pub fn load_lake_config(path: impl AsRef<Path>) -> Result<LakeConfig, LakeConfigLoadError> {
    let source = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&source)?)
}

pub fn load_expert_registry(path: impl AsRef<Path>) -> Result<ExpertRegistry, LakeConfigLoadError> {
    let path = path.as_ref();
    let config = load_lake_config(path)?;
    Ok(ExpertRegistry::from_config(&config, path)?)
}

fn validate_model_reference(
    model: &ModelReferenceConfig,
    field: &str,
) -> Result<(), ValidationError> {
    match model {
        ModelReferenceConfig::Shorthand(value) => {
            if value.trim().is_empty() {
                Err(ValidationError::new(
                    field,
                    "model reference cannot be empty",
                ))
            } else {
                Ok(())
            }
        }
        ModelReferenceConfig::Detailed(details) => {
            let active_sources = [
                details.path.as_deref(),
                details.remote.as_deref(),
                details.imported.as_deref(),
            ]
            .into_iter()
            .flatten()
            .filter(|value| !value.trim().is_empty())
            .count();

            if active_sources == 0 {
                return Err(ValidationError::new(
                    field,
                    "model reference must define one of path, remote, or imported",
                ));
            }

            if active_sources > 1 {
                return Err(ValidationError::new(
                    field,
                    "model reference must define only one of path, remote, or imported",
                ));
            }

            if details
                .cache_path
                .as_deref()
                .is_some_and(|cache_path| cache_path.trim().is_empty())
            {
                return Err(ValidationError::new(
                    format!("{field}.cache_path"),
                    "cache_path cannot be empty",
                ));
            }

            Ok(())
        }
    }
}

fn resolve_model_reference(
    model: &ModelReferenceConfig,
    config_dir: &Path,
) -> Result<RuntimeModelReference, ValidationError> {
    match model {
        ModelReferenceConfig::Shorthand(value) => {
            let value = value.trim();
            let kind = if looks_remote(value) {
                ModelReferenceKind::Remote
            } else {
                ModelReferenceKind::Local
            };
            Ok(RuntimeModelReference {
                kind,
                original: value.to_string(),
                resolved_path: (kind == ModelReferenceKind::Local)
                    .then(|| resolve_local_path(config_dir, value)),
                cache_path: None,
            })
        }
        ModelReferenceConfig::Detailed(details) => {
            if let Some(path) = details
                .path
                .as_deref()
                .filter(|path| !path.trim().is_empty())
            {
                return Ok(RuntimeModelReference {
                    kind: ModelReferenceKind::Local,
                    original: path.trim().to_string(),
                    resolved_path: Some(resolve_local_path(config_dir, path.trim())),
                    cache_path: resolve_optional_path(config_dir, details.cache_path.as_deref()),
                });
            }

            if let Some(remote) = details
                .remote
                .as_deref()
                .filter(|remote| !remote.trim().is_empty())
            {
                return Ok(RuntimeModelReference {
                    kind: ModelReferenceKind::Remote,
                    original: remote.trim().to_string(),
                    resolved_path: None,
                    cache_path: resolve_optional_path(config_dir, details.cache_path.as_deref()),
                });
            }

            if let Some(imported) = details
                .imported
                .as_deref()
                .filter(|imported| !imported.trim().is_empty())
            {
                return Ok(RuntimeModelReference {
                    kind: ModelReferenceKind::Imported,
                    original: imported.trim().to_string(),
                    resolved_path: Some(resolve_local_path(config_dir, imported.trim())),
                    cache_path: resolve_optional_path(config_dir, details.cache_path.as_deref()),
                });
            }

            Err(ValidationError::new(
                "model",
                "model reference must define one of path, remote, or imported",
            ))
        }
    }
}

fn resolve_optional_path(config_dir: &Path, value: Option<&str>) -> Option<PathBuf> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| resolve_local_path(config_dir, value))
}

fn resolve_local_path(config_dir: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    };
    normalize_path(joined)
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(value) => normalized.push(value),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

fn looks_remote(value: &str) -> bool {
    value.contains("://") || value.starts_with("hf:")
}

fn require_non_empty<'a>(
    value: Option<&'a str>,
    field: &str,
    message: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<&'a str> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => Some(value),
        None => {
            errors.push(ValidationError::new(field, message));
            None
        }
    }
}

fn trimmed_required(value: Option<&str>) -> &str {
    value.expect("validated required field").trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/lake/valid_two_experts/lake.yaml")
    }

    fn parse_config(source: &str) -> LakeConfig {
        serde_yaml::from_str(source).expect("test config should parse")
    }

    #[test]
    fn loads_valid_two_expert_fixture() {
        let path = fixture_path();
        let config = load_lake_config(&path).expect("fixture config should load");
        let registry =
            ExpertRegistry::from_config(&config, &path).expect("fixture should validate");

        assert_eq!(registry.lake_name(), "frontend-lake");
        assert_eq!(registry.server().host, "127.0.0.1");
        assert_eq!(registry.server().port, 8080);
        assert_eq!(registry.server().model_name, "library-lake-v1");
        assert_eq!(registry.experts().len(), 2);
        assert!(config.teacher.is_none());
    }

    #[test]
    fn resolves_relative_local_paths_from_config_directory() {
        let path = fixture_path();
        let registry = load_expert_registry(&path).expect("registry should load");
        let config_dir = path.parent().expect("fixture has parent");
        let expert = registry
            .get("javascript-core")
            .expect("javascript expert should exist");

        assert_eq!(expert.model.kind, ModelReferenceKind::Local);
        assert_eq!(expert.model.original, "./models/javascript-core-0.5b.gguf");
        assert_eq!(
            expert.model.resolved_path.as_deref(),
            Some(
                config_dir
                    .join("models/javascript-core-0.5b.gguf")
                    .as_path()
            )
        );
    }

    #[test]
    fn preserves_remote_cache_and_sharing_metadata() {
        let path = fixture_path();
        let registry = load_expert_registry(&path).expect("registry should load");
        let config_dir = path.parent().expect("fixture has parent");
        let expert = registry
            .get("tanstack-router")
            .expect("tanstack expert should exist");

        assert_eq!(expert.model.kind, ModelReferenceKind::Remote);
        assert_eq!(expert.model.original, "hf://example/tanstack-router-0.5b");
        assert_eq!(expert.model.resolved_path, None);
        assert_eq!(
            expert.model.cache_path.as_deref(),
            Some(config_dir.join("cache/tanstack-router-0.5b.gguf").as_path())
        );
        assert_eq!(
            expert
                .sharing
                .as_ref()
                .and_then(|sharing| sharing.package.as_deref()),
            Some("tanstack-router-expert")
        );
    }

    #[test]
    fn rejects_duplicate_expert_ids() {
        let config = parse_config(
            r#"
name: duplicate-lake
experts:
  - id: javascript-core
    domain: JavaScript
    model: ./models/js.gguf
  - id: javascript-core
    domain: TypeScript
    model: ./models/ts.gguf
server:
  host: 127.0.0.1
  port: 8080
  model_name: duplicate-lake-v1
"#,
        );

        let error = ExpertRegistry::from_config(&config, "/workspace/lake.yaml")
            .expect_err("duplicate expert IDs should fail validation");

        assert!(error.contains_field("experts[1].id"));
        assert!(error.errors.iter().any(|error| error
            .message
            .contains("duplicate expert id 'javascript-core'")));
    }

    #[test]
    fn rejects_missing_expert_model() {
        let config = parse_config(
            r#"
name: missing-model-lake
experts:
  - id: javascript-core
    domain: JavaScript
server:
  host: 127.0.0.1
  port: 8080
  model_name: missing-model-lake-v1
"#,
        );

        let error = ExpertRegistry::from_config(&config, "/workspace/lake.yaml")
            .expect_err("missing expert model should fail validation");

        assert!(error.contains_field("experts[0].model"));
    }

    #[test]
    fn rejects_invalid_server_port() {
        let config = parse_config(
            r#"
name: invalid-port-lake
experts:
  - id: javascript-core
    domain: JavaScript
    model: ./models/js.gguf
server:
  host: 127.0.0.1
  port: 70000
  model_name: invalid-port-lake-v1
"#,
        );

        let error = ExpertRegistry::from_config(&config, "/workspace/lake.yaml")
            .expect_err("invalid port should fail validation");

        assert!(error.contains_field("server.port"));
    }

    #[test]
    fn preserves_imported_expert_metadata() {
        let config = parse_config(
            r#"
name: shared-lake
experts:
  - id: sql
    domain: SQL query design
    model:
      imported: ./packages/sql-expert
    routing_hints:
      - sql
    version: 1.2.3
    sharing:
      package: sql-expert
      authors:
        - Data Team
    compatibility:
      backend: llama.cpp
      model_format: gguf
    training_status:
      state: imported
      last_trained_at: "2026-06-01"
server:
  host: 127.0.0.1
  port: 8080
  model_name: shared-lake-v1
"#,
        );

        let registry = ExpertRegistry::from_config(&config, "/workspace/lake.yaml")
            .expect("imported expert config should validate");
        let expert = registry.get("sql").expect("sql expert should exist");

        assert_eq!(expert.model.kind, ModelReferenceKind::Imported);
        assert_eq!(
            expert.model.resolved_path.as_deref(),
            Some(Path::new("/workspace/packages/sql-expert"))
        );
        assert_eq!(expert.version.as_deref(), Some("1.2.3"));
        assert_eq!(
            expert
                .compatibility
                .as_ref()
                .and_then(|compat| compat.backend.as_deref()),
            Some("llama.cpp")
        );
        assert_eq!(
            expert
                .training_status
                .as_ref()
                .and_then(|status| status.state.as_deref()),
            Some("imported")
        );
    }

    #[test]
    fn registry_queries_experts_by_id() {
        let registry = load_expert_registry(fixture_path()).expect("registry should load");

        assert!(registry.get("javascript-core").is_some());
        assert!(registry.get("tanstack-router").is_some());
        assert!(registry.get("missing-expert").is_none());
    }
}
