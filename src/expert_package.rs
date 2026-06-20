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

use crate::lake_config::{
    CompatibilityMetadata, ExpertRegistry, ModelReferenceKind, RegistryExpert,
    RuntimeModelReference, SharingMetadata, TrainingStatus, ValidationErrors,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const EXPERT_PACKAGE_MANIFEST: &str = "expert.yaml";
pub const EXPERT_PACKAGE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpertPackageManifest {
    pub manifest_version: u32,
    pub expert_id: String,
    pub domain: String,
    pub model: PackageModelArtifact,
    #[serde(default)]
    pub routing_hints: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub sharing: Option<SharingMetadata>,
    #[serde(default)]
    pub compatibility: Option<CompatibilityMetadata>,
    #[serde(default)]
    pub training_status: Option<TrainingStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageModelArtifact {
    #[serde(default)]
    pub embedded: Option<String>,
    #[serde(default)]
    pub external_path: Option<String>,
    #[serde(default)]
    pub remote: Option<String>,
    #[serde(default)]
    pub cache_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageExportLayout {
    Folder,
    ManifestArchive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportOptions {
    pub layout: PackageExportLayout,
    pub embed_artifact: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            layout: PackageExportLayout::Folder,
            embed_artifact: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateExpertBehavior {
    Reject,
    Overwrite,
    Rename(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportOptions {
    pub duplicate_behavior: DuplicateExpertBehavior,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            duplicate_behavior: DuplicateExpertBehavior::Reject,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedExpertPackage {
    pub package_root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: ExpertPackageManifest,
    pub warnings: Vec<PackageValidationWarning>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportedExpertPackage {
    pub package_path: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: ExpertPackageManifest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedExpertPackage {
    pub registry: ExpertRegistry,
    pub expert: RegistryExpert,
    pub manifest: ExpertPackageManifest,
    pub warnings: Vec<PackageValidationWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageValidationIssue {
    pub field: String,
    pub message: String,
}

impl PackageValidationIssue {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageValidationWarning {
    pub field: String,
    pub message: String,
}

impl PackageValidationWarning {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug)]
pub enum ExpertPackageError {
    ExpertNotFound {
        expert_id: String,
    },
    DuplicateExpertId {
        expert_id: String,
    },
    InvalidManifest {
        errors: Vec<PackageValidationIssue>,
    },
    EmbeddedArchiveUnsupported,
    MissingModelArtifact {
        expert_id: String,
        path: PathBuf,
    },
    UnsupportedModelSource {
        expert_id: String,
        kind: ModelReferenceKind,
    },
    RegistryValidation(ValidationErrors),
    Io {
        context: String,
        source: std::io::Error,
    },
    Yaml(serde_yaml::Error),
}

impl fmt::Display for ExpertPackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpertNotFound { expert_id } => {
                write!(f, "expert '{expert_id}' is not configured")
            }
            Self::DuplicateExpertId { expert_id } => {
                write!(f, "expert package conflicts with existing expert '{expert_id}'")
            }
            Self::InvalidManifest { errors } => {
                writeln!(
                    f,
                    "expert package manifest is invalid with {} error(s):",
                    errors.len()
                )?;
                for error in errors {
                    writeln!(f, "- {}: {}", error.field, error.message)?;
                }
                Ok(())
            }
            Self::EmbeddedArchiveUnsupported => write!(
                f,
                "embedded artifacts require folder package layout; manifest archives can reference external artifacts"
            ),
            Self::MissingModelArtifact { expert_id, path } => write!(
                f,
                "expert '{expert_id}' model artifact is missing at {}",
                path.display()
            ),
            Self::UnsupportedModelSource { expert_id, kind } => write!(
                f,
                "expert '{expert_id}' uses unsupported model source {kind:?} for this package operation"
            ),
            Self::RegistryValidation(errors) => write!(f, "{errors}"),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::Yaml(error) => write!(f, "expert package YAML error: {error}"),
        }
    }
}

impl std::error::Error for ExpertPackageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RegistryValidation(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            Self::Yaml(error) => Some(error),
            _ => None,
        }
    }
}

pub fn load_expert_package(
    package_path: impl AsRef<Path>,
) -> Result<LoadedExpertPackage, ExpertPackageError> {
    let package_path = package_path.as_ref();
    let metadata = fs::metadata(package_path).map_err(|source| ExpertPackageError::Io {
        context: format!("failed to inspect expert package {}", package_path.display()),
        source,
    })?;
    let (package_root, manifest_path) = if metadata.is_dir() {
        (
            package_path.to_path_buf(),
            package_path.join(EXPERT_PACKAGE_MANIFEST),
        )
    } else {
        (
            package_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
            package_path.to_path_buf(),
        )
    };

    let source = fs::read_to_string(&manifest_path).map_err(|source| ExpertPackageError::Io {
        context: format!("failed to read expert package manifest {}", manifest_path.display()),
        source,
    })?;
    let manifest = serde_yaml::from_str::<ExpertPackageManifest>(&source)
        .map_err(ExpertPackageError::Yaml)?;
    let warnings = validate_expert_package_manifest(&manifest, &package_root)?;

    Ok(LoadedExpertPackage {
        package_root,
        manifest_path,
        manifest,
        warnings,
    })
}

pub fn validate_expert_package_manifest(
    manifest: &ExpertPackageManifest,
    package_root: impl AsRef<Path>,
) -> Result<Vec<PackageValidationWarning>, ExpertPackageError> {
    let package_root = package_root.as_ref();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if manifest.manifest_version != EXPERT_PACKAGE_SCHEMA_VERSION {
        errors.push(PackageValidationIssue::new(
            "manifest_version",
            format!(
                "unsupported manifest version {}; expected {}",
                manifest.manifest_version, EXPERT_PACKAGE_SCHEMA_VERSION
            ),
        ));
    }

    require_non_empty(
        &manifest.expert_id,
        "expert_id",
        "expert ID is required",
        &mut errors,
    );
    require_non_empty(
        &manifest.domain,
        "domain",
        "expert domain is required",
        &mut errors,
    );
    validate_optional_non_empty(
        manifest.version.as_deref(),
        "version",
        "expert version cannot be empty",
        &mut errors,
    );
    validate_compatibility(manifest.compatibility.as_ref(), &mut errors);
    validate_model_artifact(&manifest.model, package_root, &mut errors, &mut warnings);

    if errors.is_empty() {
        Ok(warnings)
    } else {
        Err(ExpertPackageError::InvalidManifest { errors })
    }
}

pub fn export_expert_package(
    registry: &ExpertRegistry,
    expert_id: &str,
    destination: impl AsRef<Path>,
    options: ExportOptions,
) -> Result<ExportedExpertPackage, ExpertPackageError> {
    let expert = registry
        .get(expert_id)
        .ok_or_else(|| ExpertPackageError::ExpertNotFound {
            expert_id: expert_id.to_string(),
        })?;
    let destination = destination.as_ref();

    if options.layout == PackageExportLayout::ManifestArchive && options.embed_artifact {
        return Err(ExpertPackageError::EmbeddedArchiveUnsupported);
    }

    let manifest = manifest_from_expert(expert, options.embed_artifact, destination)?;
    let manifest_path = match options.layout {
        PackageExportLayout::Folder => {
            fs::create_dir_all(destination).map_err(|source| ExpertPackageError::Io {
                context: format!("failed to create expert package {}", destination.display()),
                source,
            })?;

            if options.embed_artifact {
                copy_embedded_artifact(expert, destination, &manifest)?;
            }

            destination.join(EXPERT_PACKAGE_MANIFEST)
        }
        PackageExportLayout::ManifestArchive => {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|source| ExpertPackageError::Io {
                    context: format!("failed to create expert package archive parent {}", parent.display()),
                    source,
                })?;
            }
            destination.to_path_buf()
        }
    };

    let source = serde_yaml::to_string(&manifest).map_err(ExpertPackageError::Yaml)?;
    fs::write(&manifest_path, source).map_err(|source| ExpertPackageError::Io {
        context: format!(
            "failed to write expert package manifest {}",
            manifest_path.display()
        ),
        source,
    })?;

    Ok(ExportedExpertPackage {
        package_path: destination.to_path_buf(),
        manifest_path,
        manifest,
    })
}

pub fn import_expert_package(
    registry: &ExpertRegistry,
    package_path: impl AsRef<Path>,
    options: ImportOptions,
) -> Result<ImportedExpertPackage, ExpertPackageError> {
    let loaded = load_expert_package(package_path)?;
    let target_id = imported_expert_id(registry, &loaded.manifest, &options)?;
    let expert = registry_expert_from_package(&loaded, target_id)?;

    let mut experts = registry.experts().to_vec();
    if matches!(options.duplicate_behavior, DuplicateExpertBehavior::Overwrite) {
        experts.retain(|existing| existing.id != loaded.manifest.expert_id);
    }
    experts.push(expert.clone());

    let registry = registry
        .with_runtime_experts(experts)
        .map_err(ExpertPackageError::RegistryValidation)?;

    Ok(ImportedExpertPackage {
        registry,
        expert,
        manifest: loaded.manifest,
        warnings: loaded.warnings,
    })
}

fn manifest_from_expert(
    expert: &RegistryExpert,
    embed_artifact: bool,
    destination: &Path,
) -> Result<ExpertPackageManifest, ExpertPackageError> {
    let model = if embed_artifact {
        let source_path =
            expert
                .model
                .resolved_path
                .as_ref()
                .ok_or_else(|| ExpertPackageError::UnsupportedModelSource {
                    expert_id: expert.id.clone(),
                    kind: expert.model.kind,
                })?;
        let file_name =
            source_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| ExpertPackageError::MissingModelArtifact {
                    expert_id: expert.id.clone(),
                    path: source_path.clone(),
                })?;
        let embedded_path = Path::new("artifacts").join(file_name);
        PackageModelArtifact {
            embedded: Some(path_to_string(&embedded_path)),
            external_path: None,
            remote: None,
            cache_path: None,
        }
    } else {
        model_reference_from_expert(expert)
    };

    if embed_artifact {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|source| ExpertPackageError::Io {
                context: format!("failed to prepare expert package parent {}", parent.display()),
                source,
            })?;
        }
    }

    Ok(ExpertPackageManifest {
        manifest_version: EXPERT_PACKAGE_SCHEMA_VERSION,
        expert_id: expert.id.clone(),
        domain: expert.domain.clone(),
        model,
        routing_hints: expert.routing_hints.clone(),
        examples: expert.examples.clone(),
        version: expert.version.clone(),
        sharing: expert.sharing.clone(),
        compatibility: expert.compatibility.clone(),
        training_status: expert.training_status.clone(),
    })
}

fn model_reference_from_expert(expert: &RegistryExpert) -> PackageModelArtifact {
    match expert.model.kind {
        ModelReferenceKind::Local | ModelReferenceKind::Imported => PackageModelArtifact {
            embedded: None,
            external_path: Some(
                expert
                    .model
                    .resolved_path
                    .as_ref()
                    .map(path_to_string)
                    .unwrap_or_else(|| expert.model.original.clone()),
            ),
            remote: None,
            cache_path: None,
        },
        ModelReferenceKind::Remote => PackageModelArtifact {
            embedded: None,
            external_path: None,
            remote: Some(expert.model.original.clone()),
            cache_path: expert.model.cache_path.as_ref().map(path_to_string),
        },
    }
}

fn copy_embedded_artifact(
    expert: &RegistryExpert,
    destination: &Path,
    manifest: &ExpertPackageManifest,
) -> Result<(), ExpertPackageError> {
    let source_path =
        expert
            .model
            .resolved_path
            .as_ref()
            .ok_or_else(|| ExpertPackageError::UnsupportedModelSource {
                expert_id: expert.id.clone(),
                kind: expert.model.kind,
            })?;

    match fs::metadata(source_path) {
        Ok(metadata) if metadata.is_file() => {}
        _ => {
            return Err(ExpertPackageError::MissingModelArtifact {
                expert_id: expert.id.clone(),
                path: source_path.clone(),
            });
        }
    }

    let embedded = manifest
        .model
        .embedded
        .as_deref()
        .expect("embedded export manifest has an embedded artifact");
    let target = destination.join(embedded);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| ExpertPackageError::Io {
            context: format!("failed to create expert artifact directory {}", parent.display()),
            source,
        })?;
    }
    fs::copy(source_path, &target).map_err(|source| ExpertPackageError::Io {
        context: format!(
            "failed to copy expert artifact from {} to {}",
            source_path.display(),
            target.display()
        ),
        source,
    })?;
    Ok(())
}

fn imported_expert_id(
    registry: &ExpertRegistry,
    manifest: &ExpertPackageManifest,
    options: &ImportOptions,
) -> Result<String, ExpertPackageError> {
    let original_id = manifest.expert_id.trim();
    match &options.duplicate_behavior {
        DuplicateExpertBehavior::Reject => {
            if registry.get(original_id).is_some() {
                Err(ExpertPackageError::DuplicateExpertId {
                    expert_id: original_id.to_string(),
                })
            } else {
                Ok(original_id.to_string())
            }
        }
        DuplicateExpertBehavior::Overwrite => Ok(original_id.to_string()),
        DuplicateExpertBehavior::Rename(new_id) => {
            let new_id = new_id.trim();
            if new_id.is_empty() || registry.get(new_id).is_some() {
                Err(ExpertPackageError::DuplicateExpertId {
                    expert_id: new_id.to_string(),
                })
            } else {
                Ok(new_id.to_string())
            }
        }
    }
}

fn registry_expert_from_package(
    loaded: &LoadedExpertPackage,
    expert_id: String,
) -> Result<RegistryExpert, ExpertPackageError> {
    Ok(RegistryExpert {
        id: expert_id,
        domain: loaded.manifest.domain.trim().to_string(),
        model: runtime_model_reference(&loaded.manifest.model, &loaded.package_root),
        routing_hints: loaded.manifest.routing_hints.clone(),
        examples: loaded.manifest.examples.clone(),
        sharing: loaded.manifest.sharing.clone(),
        version: loaded.manifest.version.clone(),
        compatibility: loaded.manifest.compatibility.clone(),
        training_status: loaded.manifest.training_status.clone(),
    })
}

fn runtime_model_reference(model: &PackageModelArtifact, package_root: &Path) -> RuntimeModelReference {
    if let Some(embedded) = model.embedded.as_deref().filter(|value| !value.trim().is_empty()) {
        let resolved = resolve_package_path(package_root, embedded.trim());
        return RuntimeModelReference {
            kind: ModelReferenceKind::Local,
            original: embedded.trim().to_string(),
            resolved_path: Some(resolved),
            cache_path: None,
        };
    }

    if let Some(external_path) = model
        .external_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        let resolved = resolve_package_path(package_root, external_path.trim());
        return RuntimeModelReference {
            kind: ModelReferenceKind::Local,
            original: external_path.trim().to_string(),
            resolved_path: Some(resolved),
            cache_path: None,
        };
    }

    RuntimeModelReference {
        kind: ModelReferenceKind::Remote,
        original: model
            .remote
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .to_string(),
        resolved_path: None,
        cache_path: model
            .cache_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|path| resolve_package_path(package_root, path)),
    }
}

fn validate_model_artifact(
    model: &PackageModelArtifact,
    package_root: &Path,
    errors: &mut Vec<PackageValidationIssue>,
    warnings: &mut Vec<PackageValidationWarning>,
) {
    let active = [
        ("model.embedded", model.embedded.as_deref()),
        ("model.external_path", model.external_path.as_deref()),
        ("model.remote", model.remote.as_deref()),
    ]
    .into_iter()
    .filter_map(|(field, value)| {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| (field, value))
    })
    .collect::<Vec<_>>();

    if active.is_empty() {
        errors.push(PackageValidationIssue::new(
            "model",
            "model artifact reference must define one of embedded, external_path, or remote",
        ));
        return;
    }

    if active.len() > 1 {
        errors.push(PackageValidationIssue::new(
            "model",
            "model artifact reference must define only one of embedded, external_path, or remote",
        ));
        return;
    }

    let (field, value) = active[0];
    match field {
        "model.embedded" => {
            let path = resolve_package_path(package_root, value);
            match fs::metadata(&path) {
                Ok(metadata) if metadata.is_file() => {}
                _ => errors.push(PackageValidationIssue::new(
                    field,
                    format!("embedded model artifact is missing at {}", path.display()),
                )),
            }
        }
        "model.external_path" => {
            let path = resolve_package_path(package_root, value);
            match fs::metadata(&path) {
                Ok(metadata) if metadata.is_file() => {}
                _ => warnings.push(PackageValidationWarning::new(
                    field,
                    format!(
                        "external model artifact is not available at {}; import can continue",
                        path.display()
                    ),
                )),
            }
        }
        "model.remote" => {
            if !(value.contains("://") || value.starts_with("hf:")) {
                warnings.push(PackageValidationWarning::new(
                    field,
                    "remote artifact reference is not a recognized URL-style value",
                ));
            }
        }
        _ => {}
    }

    if let Some(cache_path) = model
        .cache_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let path = resolve_package_path(package_root, cache_path);
        if !path.exists() {
            warnings.push(PackageValidationWarning::new(
                "model.cache_path",
                format!("cache path is not available at {}", path.display()),
            ));
        }
    }
}

fn validate_compatibility(
    compatibility: Option<&CompatibilityMetadata>,
    errors: &mut Vec<PackageValidationIssue>,
) {
    let Some(compatibility) = compatibility else {
        return;
    };

    validate_optional_non_empty(
        compatibility.neuronlake.as_deref(),
        "compatibility.neuronlake",
        "compatibility neuronlake constraint cannot be empty",
        errors,
    );
    validate_optional_non_empty(
        compatibility.backend.as_deref(),
        "compatibility.backend",
        "compatibility backend cannot be empty",
        errors,
    );
    validate_optional_non_empty(
        compatibility.model_format.as_deref(),
        "compatibility.model_format",
        "compatibility model_format cannot be empty",
        errors,
    );
    validate_optional_non_empty(
        compatibility.quantization.as_deref(),
        "compatibility.quantization",
        "compatibility quantization cannot be empty",
        errors,
    );
    validate_optional_non_empty(
        compatibility.min_version.as_deref(),
        "compatibility.min_version",
        "compatibility min_version cannot be empty",
        errors,
    );
}

fn validate_optional_non_empty(
    value: Option<&str>,
    field: &str,
    message: &str,
    errors: &mut Vec<PackageValidationIssue>,
) {
    if value.is_some_and(|value| value.trim().is_empty()) {
        errors.push(PackageValidationIssue::new(field, message));
    }
}

fn require_non_empty(
    value: &str,
    field: &str,
    message: &str,
    errors: &mut Vec<PackageValidationIssue>,
) {
    if value.trim().is_empty() {
        errors.push(PackageValidationIssue::new(field, message));
    }
}

fn resolve_package_path(package_root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        package_root.join(path)
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

fn path_to_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lake_config::LakeConfig;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("neuronlake-package-{name}-{nonce}"))
    }

    fn fixture_package(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/expert_packages/{name}"
        ))
    }

    fn registry_with_model(model_path: &Path) -> ExpertRegistry {
        let yaml = format!(
            r#"
name: package-lake
experts:
  - id: javascript-core
    domain: JavaScript language support
    model: {}
    routing_hints:
      - javascript
    examples:
      - Explain promise resolution order.
    version: 0.1.0
    sharing:
      package: javascript-core
      authors:
        - Web Team
    compatibility:
      backend: llama.cpp
      model_format: gguf
    training_status:
      state: trained
      dataset: js-docs
server:
  host: 127.0.0.1
  port: 8080
  model_name: package-lake-v1
"#,
            model_path.display()
        );
        let config: LakeConfig = serde_yaml::from_str(&yaml).unwrap();
        ExpertRegistry::from_config(&config, "/tmp/lake.yaml").unwrap()
    }

    fn empty_target_registry() -> ExpertRegistry {
        let yaml = r#"
name: target-lake
experts:
  - id: existing
    domain: Existing expert
    model: /tmp/existing.gguf
server:
  host: 127.0.0.1
  port: 8080
  model_name: target-lake-v1
"#;
        let config: LakeConfig = serde_yaml::from_str(yaml).unwrap();
        ExpertRegistry::from_config(&config, "/tmp/lake.yaml").unwrap()
    }

    #[test]
    fn loads_embedded_package_manifest_with_required_fields() {
        let package = load_expert_package(fixture_package("embedded")).unwrap();

        assert_eq!(package.manifest.manifest_version, 1);
        assert_eq!(package.manifest.expert_id, "sql-shared");
        assert_eq!(package.manifest.domain, "SQL query design and optimization");
        assert_eq!(
            package.manifest.model.embedded.as_deref(),
            Some("artifacts/sql-shared.gguf")
        );
        assert!(package.warnings.is_empty());
    }

    #[test]
    fn external_package_records_missing_artifact_warning() {
        let package = load_expert_package(fixture_package("external")).unwrap();

        assert_eq!(package.manifest.expert_id, "rust-shared");
        assert!(package
            .warnings
            .iter()
            .any(|warning| warning.field == "model.external_path"));
    }

    #[test]
    fn export_folder_manifest_preserves_expert_metadata() {
        let dir = unique_temp_dir("export-folder");
        fs::create_dir_all(&dir).unwrap();
        let model_path = dir.join("javascript-core.gguf");
        fs::write(&model_path, b"fake gguf").unwrap();
        let registry = registry_with_model(&model_path);
        let target = dir.join("package");

        let exported = export_expert_package(
            &registry,
            "javascript-core",
            &target,
            ExportOptions::default(),
        )
        .unwrap();
        let loaded = load_expert_package(&target).unwrap();

        assert_eq!(exported.manifest_path, target.join(EXPERT_PACKAGE_MANIFEST));
        assert_eq!(loaded.manifest.expert_id, "javascript-core");
        assert_eq!(loaded.manifest.routing_hints, vec!["javascript"]);
        assert_eq!(
            loaded
                .manifest
                .sharing
                .as_ref()
                .and_then(|sharing| sharing.package.as_deref()),
            Some("javascript-core")
        );
        assert_eq!(
            loaded
                .manifest
                .compatibility
                .as_ref()
                .and_then(|compatibility| compatibility.backend.as_deref()),
            Some("llama.cpp")
        );
        assert_eq!(
            loaded
                .manifest
                .training_status
                .as_ref()
                .and_then(|status| status.dataset.as_deref()),
            Some("js-docs")
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn export_embedded_package_copies_model_artifact() {
        let dir = unique_temp_dir("export-embedded");
        fs::create_dir_all(&dir).unwrap();
        let model_path = dir.join("javascript-core.gguf");
        fs::write(&model_path, b"fake gguf").unwrap();
        let registry = registry_with_model(&model_path);
        let target = dir.join("embedded-package");

        let exported = export_expert_package(
            &registry,
            "javascript-core",
            &target,
            ExportOptions {
                layout: PackageExportLayout::Folder,
                embed_artifact: true,
            },
        )
        .unwrap();

        assert_eq!(
            exported.manifest.model.embedded.as_deref(),
            Some("artifacts/javascript-core.gguf")
        );
        assert!(target.join("artifacts/javascript-core.gguf").is_file());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn export_manifest_archive_references_external_artifact() {
        let dir = unique_temp_dir("export-archive");
        fs::create_dir_all(&dir).unwrap();
        let model_path = dir.join("javascript-core.gguf");
        fs::write(&model_path, b"fake gguf").unwrap();
        let registry = registry_with_model(&model_path);
        let target = dir.join("javascript-core.expert.yaml");

        let exported = export_expert_package(
            &registry,
            "javascript-core",
            &target,
            ExportOptions {
                layout: PackageExportLayout::ManifestArchive,
                embed_artifact: false,
            },
        )
        .unwrap();
        let loaded = load_expert_package(&target).unwrap();

        assert_eq!(exported.manifest_path, target);
        assert_eq!(
            loaded.manifest.model.external_path.as_deref(),
            Some(model_path.to_string_lossy().as_ref())
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn import_embedded_package_adds_first_class_registry_entry() {
        let registry = empty_target_registry();

        let imported = import_expert_package(
            &registry,
            fixture_package("embedded"),
            ImportOptions::default(),
        )
        .unwrap();
        let expert = imported
            .registry
            .get("sql-shared")
            .expect("imported expert should exist");

        assert_eq!(expert.domain, "SQL query design and optimization");
        assert_eq!(expert.model.kind, ModelReferenceKind::Local);
        assert_eq!(
            expert.model.resolved_path.as_deref(),
            Some(
                fixture_package("embedded")
                    .join("artifacts/sql-shared.gguf")
                    .as_path()
            )
        );
        assert_eq!(expert.routing_hints, vec!["sql", "postgres", "query planner"]);
        assert_eq!(
            expert
                .compatibility
                .as_ref()
                .and_then(|compatibility| compatibility.quantization.as_deref()),
            Some("q4_k_m")
        );
    }

    #[test]
    fn import_rejects_duplicate_id_by_default() {
        let registry = import_expert_package(
            &empty_target_registry(),
            fixture_package("embedded"),
            ImportOptions::default(),
        )
        .unwrap()
        .registry;

        let error = import_expert_package(
            &registry,
            fixture_package("embedded"),
            ImportOptions::default(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            ExpertPackageError::DuplicateExpertId { expert_id } if expert_id == "sql-shared"
        ));
    }

    #[test]
    fn import_can_rename_duplicate_and_preserve_metadata() {
        let registry = import_expert_package(
            &empty_target_registry(),
            fixture_package("embedded"),
            ImportOptions::default(),
        )
        .unwrap()
        .registry;

        let imported = import_expert_package(
            &registry,
            fixture_package("embedded"),
            ImportOptions {
                duplicate_behavior: DuplicateExpertBehavior::Rename("sql-shared-copy".to_string()),
            },
        )
        .unwrap();
        let expert = imported
            .registry
            .get("sql-shared-copy")
            .expect("renamed expert should exist");

        assert_eq!(expert.version.as_deref(), Some("1.0.0"));
        assert_eq!(
            expert
                .training_status
                .as_ref()
                .and_then(|status| status.state.as_deref()),
            Some("trained")
        );
        assert!(expert.examples.iter().any(|example| example.contains("PostgreSQL")));
    }
}
