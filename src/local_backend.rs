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

use crate::lake_config::{ExpertRegistry, ModelReferenceKind};
use serde_json::Value;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub trait ExpertBackend: Send + Sync + 'static {
    fn prepare(
        &self,
        registry: &ExpertRegistry,
        expert_id: &str,
    ) -> Result<PreparedExpert, BackendError>;

    fn generate(
        &self,
        prepared: &PreparedExpert,
        request: &BackendRequest,
    ) -> Result<BackendCompletion, BackendError>;

    fn stream(
        &self,
        prepared: &PreparedExpert,
        request: &BackendRequest,
    ) -> Result<Vec<BackendDelta>, BackendError> {
        let completion = self.generate(prepared, request)?;
        Ok(split_deltas(&completion.content))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackendRequest {
    pub expert_id: String,
    pub model: String,
    pub messages: Vec<BackendMessage>,
    pub options: GenerationOptions,
    pub ignored_parameters: Vec<String>,
}

impl BackendRequest {
    pub fn prompt_text(&self) -> String {
        self.messages
            .iter()
            .map(|message| format!("{}: {}", message.role, message.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenerationOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedExpert {
    pub expert_id: String,
    pub domain: String,
    pub model_path: PathBuf,
    pub backend_name: String,
    pub preparation: BackendTiming,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCompletion {
    pub content: String,
    pub diagnostics: BackendDiagnostics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendDelta {
    pub content: String,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendDiagnostics {
    pub backend_name: String,
    pub expert_id: String,
    pub model_path: PathBuf,
    pub preparation: BackendTiming,
    pub generation: BackendTiming,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendTiming {
    pub elapsed_micros: u128,
}

impl BackendTiming {
    pub fn measured(elapsed: Duration) -> Self {
        Self {
            elapsed_micros: elapsed.as_micros(),
        }
    }
}

#[derive(Debug)]
pub enum BackendError {
    ExpertNotFound {
        expert_id: String,
    },
    UnsupportedModelSource {
        expert_id: String,
        kind: ModelReferenceKind,
    },
    MissingModelArtifact {
        expert_id: String,
        path: PathBuf,
    },
    UnreadableModelArtifact {
        expert_id: String,
        path: PathBuf,
        source: std::io::Error,
    },
    UnsupportedModelArtifact {
        expert_id: String,
        path: PathBuf,
        message: String,
    },
    CommandFailed {
        executable: PathBuf,
        status: Option<i32>,
        stderr: String,
    },
    Io {
        context: String,
        source: std::io::Error,
    },
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpertNotFound { expert_id } => {
                write!(f, "expert '{expert_id}' is not configured")
            }
            Self::UnsupportedModelSource { expert_id, kind } => write!(
                f,
                "expert '{expert_id}' uses unsupported model source {kind:?}; local GGUF artifacts are required"
            ),
            Self::MissingModelArtifact { expert_id, path } => write!(
                f,
                "expert '{expert_id}' model artifact is missing at {}",
                path.display()
            ),
            Self::UnreadableModelArtifact {
                expert_id,
                path,
                source,
            } => write!(
                f,
                "expert '{expert_id}' model artifact at {} is unreadable: {source}",
                path.display()
            ),
            Self::UnsupportedModelArtifact {
                expert_id,
                path,
                message,
            } => write!(
                f,
                "expert '{expert_id}' model artifact at {} is unsupported: {message}",
                path.display()
            ),
            Self::CommandFailed {
                executable,
                status,
                stderr,
            } => write!(
                f,
                "backend command {} failed with status {:?}: {}",
                executable.display(),
                status,
                stderr
            ),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
        }
    }
}

impl std::error::Error for BackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnreadableModelArtifact { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlamaCppSubprocessBackend {
    executable: PathBuf,
    extra_args: Vec<String>,
}

impl LlamaCppSubprocessBackend {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            extra_args: Vec::new(),
        }
    }

    pub fn with_extra_args(
        mut self,
        extra_args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.extra_args = extra_args.into_iter().map(Into::into).collect();
        self
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }
}

impl ExpertBackend for LlamaCppSubprocessBackend {
    fn prepare(
        &self,
        registry: &ExpertRegistry,
        expert_id: &str,
    ) -> Result<PreparedExpert, BackendError> {
        let started = Instant::now();
        let expert = registry
            .get(expert_id)
            .ok_or_else(|| BackendError::ExpertNotFound {
                expert_id: expert_id.to_string(),
            })?;

        if expert.model.kind != ModelReferenceKind::Local {
            return Err(BackendError::UnsupportedModelSource {
                expert_id: expert.id.clone(),
                kind: expert.model.kind,
            });
        }

        let path = expert.model.resolved_path.clone().ok_or_else(|| {
            BackendError::UnsupportedModelArtifact {
                expert_id: expert.id.clone(),
                path: PathBuf::new(),
                message: "local model reference did not resolve to a path".to_string(),
            }
        })?;

        match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => {
                return Err(BackendError::UnsupportedModelArtifact {
                    expert_id: expert.id.clone(),
                    path,
                    message: "model artifact must be a file".to_string(),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(BackendError::MissingModelArtifact {
                    expert_id: expert.id.clone(),
                    path,
                });
            }
            Err(error) => {
                return Err(BackendError::UnreadableModelArtifact {
                    expert_id: expert.id.clone(),
                    path,
                    source: error,
                });
            }
        }

        if !is_gguf_path(&path) {
            return Err(BackendError::UnsupportedModelArtifact {
                expert_id: expert.id.clone(),
                path,
                message: "expected a .gguf model artifact".to_string(),
            });
        }

        Ok(PreparedExpert {
            expert_id: expert.id.clone(),
            domain: expert.domain.clone(),
            model_path: path,
            backend_name: "llama.cpp-subprocess".to_string(),
            preparation: BackendTiming::measured(started.elapsed()),
        })
    }

    fn generate(
        &self,
        prepared: &PreparedExpert,
        request: &BackendRequest,
    ) -> Result<BackendCompletion, BackendError> {
        let started = Instant::now();
        let mut command = Command::new(&self.executable);
        command.stdin(Stdio::null());
        command.args([
            "--no-conversation",
            "--no-display-prompt",
            "--simple-io",
            "--verbosity",
            "1",
        ]);
        command.args(&self.extra_args);
        command.arg("--model").arg(&prepared.model_path);
        command.arg("--prompt").arg(request.prompt_text());

        if let Some(max_tokens) = request.options.max_tokens {
            command.arg("--n-predict").arg(max_tokens.to_string());
        }
        if let Some(temperature) = request.options.temperature {
            command.arg("--temp").arg(temperature.to_string());
        }
        if let Some(top_p) = request.options.top_p {
            command.arg("--top-p").arg(top_p.to_string());
        }

        let output = command.output().map_err(|source| BackendError::Io {
            context: format!(
                "failed to execute backend command {}",
                self.executable.display()
            ),
            source,
        })?;

        if !output.status.success() {
            return Err(BackendError::CommandFailed {
                executable: self.executable.clone(),
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        Ok(BackendCompletion {
            content: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            diagnostics: BackendDiagnostics {
                backend_name: prepared.backend_name.clone(),
                expert_id: prepared.expert_id.clone(),
                model_path: prepared.model_path.clone(),
                preparation: prepared.preparation,
                generation: BackendTiming::measured(started.elapsed()),
                note: Some(
                    "Measured subprocess execution time; not a benchmark target".to_string(),
                ),
            },
        })
    }
}

pub fn split_deltas(content: &str) -> Vec<BackendDelta> {
    let mut deltas = content
        .split_inclusive(' ')
        .filter(|chunk| !chunk.is_empty())
        .enumerate()
        .map(|(index, content)| BackendDelta {
            content: content.to_string(),
            index,
        })
        .collect::<Vec<_>>();

    if deltas.is_empty() {
        deltas.push(BackendDelta {
            content: String::new(),
            index: 0,
        });
    }

    deltas
}

fn is_gguf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("gguf"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lake_config::{LakeConfig, ModelReferenceKind};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("neuronlake-{name}-{nonce}"))
    }

    fn registry_with_model(model_path: &Path) -> ExpertRegistry {
        let yaml = format!(
            r#"
name: backend-lake
experts:
  - id: local-js
    domain: JavaScript
    model: {}
server:
  host: 127.0.0.1
  port: 8080
  model_name: backend-lake-v1
"#,
            model_path.display()
        );
        let config: LakeConfig = serde_yaml::from_str(&yaml).unwrap();
        ExpertRegistry::from_config(&config, "/tmp/lake.yaml").unwrap()
    }

    fn request() -> BackendRequest {
        BackendRequest {
            expert_id: "local-js".to_string(),
            model: "backend-lake-v1".to_string(),
            messages: vec![BackendMessage {
                role: "user".to_string(),
                content: "Write a promise example".to_string(),
            }],
            options: GenerationOptions {
                temperature: Some(0.2),
                top_p: Some(0.9),
                max_tokens: Some(16),
                stop: None,
            },
            ignored_parameters: vec!["presence_penalty".to_string()],
        }
    }

    #[test]
    fn prepares_existing_local_gguf_expert() {
        let dir = unique_temp_dir("prepare");
        fs::create_dir_all(&dir).unwrap();
        let model_path = dir.join("local-js.gguf");
        fs::write(&model_path, b"fake gguf").unwrap();
        let registry = registry_with_model(&model_path);
        let backend = LlamaCppSubprocessBackend::new("/bin/echo");

        let prepared = backend.prepare(&registry, "local-js").unwrap();

        assert_eq!(prepared.expert_id, "local-js");
        assert_eq!(prepared.model_path, model_path);
        assert_eq!(prepared.backend_name, "llama.cpp-subprocess");
        assert!(prepared.preparation.elapsed_micros <= u128::MAX);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_model_artifact_identifies_expert_and_path() {
        let model_path = unique_temp_dir("missing").join("missing.gguf");
        let registry = registry_with_model(&model_path);
        let backend = LlamaCppSubprocessBackend::new("/bin/echo");

        let error = backend.prepare(&registry, "local-js").unwrap_err();

        match error {
            BackendError::MissingModelArtifact { expert_id, path } => {
                assert_eq!(expert_id, "local-js");
                assert_eq!(path, model_path);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn unsupported_model_format_is_reported() {
        let dir = unique_temp_dir("unsupported");
        fs::create_dir_all(&dir).unwrap();
        let model_path = dir.join("local-js.bin");
        fs::write(&model_path, b"not gguf").unwrap();
        let registry = registry_with_model(&model_path);
        let backend = LlamaCppSubprocessBackend::new("/bin/echo");

        let error = backend.prepare(&registry, "local-js").unwrap_err();

        assert!(matches!(
            error,
            BackendError::UnsupportedModelArtifact { .. }
        ));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn remote_model_source_is_rejected_by_local_backend() {
        let yaml = r#"
name: backend-lake
experts:
  - id: remote-js
    domain: JavaScript
    model:
      remote: hf://example/js
server:
  host: 127.0.0.1
  port: 8080
  model_name: backend-lake-v1
"#;
        let config: LakeConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = ExpertRegistry::from_config(&config, "/tmp/lake.yaml").unwrap();
        let backend = LlamaCppSubprocessBackend::new("/bin/echo");

        let error = backend.prepare(&registry, "remote-js").unwrap_err();

        assert!(matches!(
            error,
            BackendError::UnsupportedModelSource {
                kind: ModelReferenceKind::Remote,
                ..
            }
        ));
    }

    #[test]
    fn subprocess_backend_generates_and_streams_deltas() {
        let dir = unique_temp_dir("generate");
        fs::create_dir_all(&dir).unwrap();
        let model_path = dir.join("local-js.gguf");
        fs::write(&model_path, b"fake gguf").unwrap();
        let registry = registry_with_model(&model_path);
        let backend = LlamaCppSubprocessBackend::new("/bin/echo");
        let prepared = backend.prepare(&registry, "local-js").unwrap();
        let request = request();

        let completion = backend.generate(&prepared, &request).unwrap();
        let deltas = backend.stream(&prepared, &request).unwrap();

        assert!(completion.content.contains("--model"));
        assert!(completion.content.contains("Write a promise example"));
        assert_eq!(completion.diagnostics.expert_id, "local-js");
        assert_eq!(completion.diagnostics.model_path, model_path);
        assert!(completion
            .diagnostics
            .note
            .as_deref()
            .unwrap()
            .contains("not a benchmark target"));
        assert!(deltas.iter().any(|delta| delta.content.contains("--model")));
        fs::remove_dir_all(dir).unwrap();
    }
}
