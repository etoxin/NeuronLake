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

use crate::lake_config::ExpertRegistry;
use crate::neuron_guard::ThreadBoundedNeuronField;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::fs;
use std::path::Path;

const ROUTER_ARTIFACT_VERSION: u32 = 1;
const DEFAULT_FEATURE_VOCAB_SIZE: usize = 4096;
const TRAINING_AMPLIFY_DELTA: i16 = 16;

pub struct ExpertRouter {
    labels: Vec<RouterLabel>,
    field: ThreadBoundedNeuronField,
    feature_vocab_size: usize,
    artifact: RouterArtifact,
}

impl ExpertRouter {
    pub fn train(registry: &ExpertRegistry) -> Result<Self, RouterError> {
        Self::train_with_vocab_size(registry, DEFAULT_FEATURE_VOCAB_SIZE)
    }

    pub fn train_with_vocab_size(
        registry: &ExpertRegistry,
        feature_vocab_size: usize,
    ) -> Result<Self, RouterError> {
        if registry.experts().is_empty() {
            return Err(RouterError::NoExperts);
        }

        let labels = labels_from_registry(registry);
        let training_examples = training_examples_from_registry(registry);
        let artifact = RouterArtifact {
            schema_version: ROUTER_ARTIFACT_VERSION,
            feature_vocab_size,
            source_fingerprint: routing_fingerprint(registry),
            labels: labels.clone(),
            training_examples,
        };

        Self::from_artifact(artifact)
    }

    pub fn from_artifact(artifact: RouterArtifact) -> Result<Self, RouterError> {
        if artifact.labels.is_empty() {
            return Err(RouterError::NoExperts);
        }

        let field = ThreadBoundedNeuronField::new(artifact.feature_vocab_size);
        let labels = artifact.labels.clone();
        let label_indexes = labels
            .iter()
            .map(|label| (label.expert_id.clone(), label.index))
            .collect::<HashMap<_, _>>();

        for example in &artifact.training_examples {
            let label_index = label_indexes
                .get(&example.expert_id)
                .ok_or_else(|| RouterError::UnknownExpertLabel(example.expert_id.clone()))?;
            train_features(
                &field,
                artifact.feature_vocab_size,
                &example.features,
                *label_index,
            );
        }

        Ok(Self {
            labels,
            field,
            feature_vocab_size: artifact.feature_vocab_size,
            artifact,
        })
    }

    pub fn labels(&self) -> &[RouterLabel] {
        &self.labels
    }

    pub fn artifact(&self) -> &RouterArtifact {
        &self.artifact
    }

    pub fn save_artifact(&self, path: impl AsRef<Path>) -> Result<(), RouterError> {
        self.artifact.save(path)
    }

    pub fn route_text(&self, text: &str, include_debug: bool) -> RouteResult {
        self.route_features(extract_routing_features(text), include_debug)
    }

    pub fn route_messages(&self, messages: &[RoutingMessage], include_debug: bool) -> RouteResult {
        self.route_features(extract_features_from_messages(messages), include_debug)
    }

    pub fn route_features(&self, features: Vec<String>, include_debug: bool) -> RouteResult {
        let mut scores = vec![0i32; self.labels.len()];
        let mut contributions = Vec::new();

        for feature in &features {
            let token_id = feature_token(feature, self.feature_vocab_size);
            unsafe {
                let neuron = self.field.get_neuron(token_id);
                for i in 0..neuron.active_connections as usize {
                    let expert_index = neuron.target_neuron_ids[i] as usize;
                    if expert_index < scores.len() {
                        let weight = neuron.weight_modifiers[i] as i32;
                        scores[expert_index] += weight;
                        contributions.push(FeatureContribution {
                            feature: feature.clone(),
                            expert_id: self.labels[expert_index].expert_id.clone(),
                            weight,
                        });
                    }
                }
            }
        }

        let selected_index = scores
            .iter()
            .enumerate()
            .max_by_key(|(index, score)| (**score, std::cmp::Reverse(*index)))
            .map(|(index, _)| index)
            .unwrap_or(0);
        let selected_score = scores.get(selected_index).copied().unwrap_or_default();
        let total_positive = scores.iter().filter(|score| **score > 0).sum::<i32>();
        let confidence = if total_positive > 0 {
            selected_score.max(0) as f32 / total_positive as f32
        } else {
            0.0
        };

        let route_scores = self
            .labels
            .iter()
            .zip(scores)
            .map(|(label, score)| RouteScore {
                expert_id: label.expert_id.clone(),
                score,
            })
            .collect::<Vec<_>>();

        let selected_expert_id = self.labels[selected_index].expert_id.clone();
        RouteResult {
            expert_id: selected_expert_id.clone(),
            expert_index: selected_index,
            confidence,
            scores: route_scores,
            debug: include_debug.then(|| RouteDebug {
                features,
                contributions: contributions
                    .into_iter()
                    .filter(|contribution| contribution.expert_id == selected_expert_id)
                    .collect(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouterArtifact {
    pub schema_version: u32,
    pub feature_vocab_size: usize,
    pub source_fingerprint: String,
    pub labels: Vec<RouterLabel>,
    pub training_examples: Vec<RoutingTrainingExample>,
}

impl RouterArtifact {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), RouterError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| RouterError::Io {
                context: format!(
                    "failed to create router artifact directory {}",
                    parent.display()
                ),
                source,
            })?;
        }
        let source = serde_yaml::to_string(self).map_err(RouterError::Yaml)?;
        fs::write(path, source).map_err(|source| RouterError::Io {
            context: format!("failed to write router artifact {}", path.display()),
            source,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, RouterError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| RouterError::Io {
            context: format!("failed to read router artifact {}", path.display()),
            source,
        })?;
        serde_yaml::from_str(&source).map_err(RouterError::Yaml)
    }

    pub fn is_current_for(&self, registry: &ExpertRegistry) -> bool {
        self.source_fingerprint == routing_fingerprint(registry)
    }

    pub fn is_stale_for(&self, registry: &ExpertRegistry) -> bool {
        !self.is_current_for(registry)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouterLabel {
    pub expert_id: String,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingTrainingExample {
    pub expert_id: String,
    pub source: TrainingExampleSource,
    pub text: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrainingExampleSource {
    Domain,
    RoutingHint,
    ExamplePrompt,
    DerivedSignal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteResult {
    pub expert_id: String,
    pub expert_index: usize,
    pub confidence: f32,
    pub scores: Vec<RouteScore>,
    pub debug: Option<RouteDebug>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteScore {
    pub expert_id: String,
    pub score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDebug {
    pub features: Vec<String>,
    pub contributions: Vec<FeatureContribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureContribution {
    pub feature: String,
    pub expert_id: String,
    pub weight: i32,
}

#[derive(Debug)]
pub enum RouterError {
    NoExperts,
    UnknownExpertLabel(String),
    Io {
        context: String,
        source: std::io::Error,
    },
    Yaml(serde_yaml::Error),
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoExperts => write!(f, "cannot train router without configured experts"),
            Self::UnknownExpertLabel(expert_id) => {
                write!(f, "router artifact references unknown expert '{expert_id}'")
            }
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::Yaml(error) => write!(f, "router artifact YAML error: {error}"),
        }
    }
}

impl std::error::Error for RouterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Yaml(error) => Some(error),
            _ => None,
        }
    }
}

pub fn labels_from_registry(registry: &ExpertRegistry) -> Vec<RouterLabel> {
    registry
        .experts()
        .iter()
        .enumerate()
        .map(|(index, expert)| RouterLabel {
            expert_id: expert.id.clone(),
            index,
        })
        .collect()
}

pub fn training_examples_from_registry(registry: &ExpertRegistry) -> Vec<RoutingTrainingExample> {
    let mut training_examples = Vec::new();

    for expert in registry.experts() {
        push_example(
            &mut training_examples,
            &expert.id,
            TrainingExampleSource::Domain,
            &expert.domain,
        );

        for hint in &expert.routing_hints {
            push_example(
                &mut training_examples,
                &expert.id,
                TrainingExampleSource::RoutingHint,
                hint,
            );
        }

        for example in &expert.examples {
            push_example(
                &mut training_examples,
                &expert.id,
                TrainingExampleSource::ExamplePrompt,
                example,
            );
        }

        let derived = derive_training_signals(
            std::iter::once(expert.domain.as_str())
                .chain(expert.routing_hints.iter().map(String::as_str))
                .chain(expert.examples.iter().map(String::as_str)),
        );
        for signal in derived {
            push_example(
                &mut training_examples,
                &expert.id,
                TrainingExampleSource::DerivedSignal,
                &signal,
            );
        }
    }

    training_examples
}

pub fn extract_features_from_messages(messages: &[RoutingMessage]) -> Vec<String> {
    let mut text = String::new();
    for message in messages {
        text.push_str(&message.role);
        text.push('\n');
        text.push_str(&message.content);
        text.push('\n');
    }
    extract_routing_features(&text)
}

pub fn extract_routing_features(text: &str) -> Vec<String> {
    let mut features = BTreeSet::new();
    let lower = text.to_lowercase();

    for token in text_tokens(&lower) {
        features.insert(format!("tok:{token}"));
    }

    extract_code_block_languages(text, &mut features);
    extract_imports_and_packages(text, &mut features);
    extract_file_extensions(text, &mut features);
    extract_framework_apis(text, &mut features);
    extract_error_terms(&lower, &mut features);
    extract_phrases(&lower, &mut features);

    features.into_iter().collect()
}

pub fn routing_fingerprint(registry: &ExpertRegistry) -> String {
    let mut hash = Fnv1a64::new();
    hash.write_str(registry.lake_name());
    for expert in registry.experts() {
        hash.write_str(&expert.id);
        hash.write_str(&expert.domain);
        for hint in &expert.routing_hints {
            hash.write_str(hint);
        }
        for example in &expert.examples {
            hash.write_str(example);
        }
    }
    format!("{:016x}", hash.finish())
}

fn push_example(
    training_examples: &mut Vec<RoutingTrainingExample>,
    expert_id: &str,
    source: TrainingExampleSource,
    text: &str,
) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    training_examples.push(RoutingTrainingExample {
        expert_id: expert_id.to_string(),
        source,
        text: trimmed.to_string(),
        features: extract_routing_features(trimmed),
    });
}

fn train_features(
    field: &ThreadBoundedNeuronField,
    feature_vocab_size: usize,
    features: &[String],
    label_index: usize,
) {
    for feature in features {
        let token_id = feature_token(feature, feature_vocab_size);
        unsafe {
            let neuron = field.get_neuron(token_id);
            neuron.token_id = token_id as u32;
            neuron.update_or_add_connection(label_index as u32, TRAINING_AMPLIFY_DELTA);
        }
    }
}

fn feature_token(feature: &str, feature_vocab_size: usize) -> usize {
    let mut hash = Fnv1a64::new();
    hash.write_str(feature);
    (hash.finish() as usize) % feature_vocab_size
}

fn derive_training_signals<'a>(texts: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut signals = BTreeSet::new();
    for text in texts {
        for feature in extract_routing_features(text) {
            if let Some(signal) = feature.strip_prefix("tok:") {
                if signal.len() > 2 {
                    signals.insert(signal.to_string());
                }
            }
            if let Some(signal) = feature.strip_prefix("api:") {
                signals.insert(signal.to_string());
            }
            if let Some(signal) = feature.strip_prefix("package:") {
                signals.insert(signal.to_string());
            }
        }
    }
    signals.into_iter().collect()
}

fn text_tokens(text: &str) -> Vec<String> {
    text.chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| token.len() > 1)
        .map(str::to_string)
        .collect()
}

fn extract_code_block_languages(text: &str, features: &mut BTreeSet<String>) {
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(language) = trimmed.strip_prefix("```") {
            let language = language
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim()
                .to_lowercase();
            if !language.is_empty() {
                features.insert(format!("code_lang:{language}"));
            }
        }
    }
}

fn extract_imports_and_packages(text: &str, features: &mut BTreeSet<String>) {
    for raw_line in text.lines() {
        let line = raw_line.trim();
        let lower = line.to_lowercase();

        if lower.starts_with("import ") || lower.starts_with("export ") {
            if let Some(package) = quoted_package(line) {
                features.insert(format!("import:{}", package.to_lowercase()));
                features.insert(format!("package:{}", package.to_lowercase()));
            }
        }

        if lower.contains("require(") {
            if let Some(package) = quoted_package(line) {
                features.insert(format!("require:{}", package.to_lowercase()));
                features.insert(format!("package:{}", package.to_lowercase()));
            }
        }

        if lower.starts_with("from ") && lower.contains(" import ") {
            if let Some(package) = line.split_whitespace().nth(1) {
                let package = package.trim_matches(|c: char| !is_package_char(c));
                if !package.is_empty() {
                    features.insert(format!("import:{}", package.to_lowercase()));
                    features.insert(format!("package:{}", package.to_lowercase()));
                }
            }
        }

        for token in line.split_whitespace() {
            let token = token.trim_matches(|c: char| !is_package_char(c));
            if is_package_like(token) {
                features.insert(format!("package:{}", token.to_lowercase()));
            }
        }
    }
}

fn extract_file_extensions(text: &str, features: &mut BTreeSet<String>) {
    for token in text.split_whitespace() {
        let token = token.trim_matches(|c: char| {
            !(c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '/')
        });
        if let Some(extension) = token.rsplit_once('.').map(|(_, extension)| extension) {
            let extension = extension
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if matches!(
                extension.as_str(),
                "js" | "jsx" | "ts" | "tsx" | "py" | "sql" | "css" | "html" | "rs"
            ) {
                features.insert(format!("ext:{extension}"));
            }
        }
    }
}

fn extract_framework_apis(text: &str, features: &mut BTreeSet<String>) {
    let apis = [
        "createFileRoute",
        "routeTree",
        "loader",
        "beforeLoad",
        "useLoaderData",
        "useNavigate",
        "useState",
        "useEffect",
        "jsx",
        "tsx",
        "FastAPI",
        "APIRouter",
        "Depends",
        "SELECT",
        "JOIN",
        "WHERE",
        "tailwind",
        "className",
    ];
    let lower = text.to_lowercase();
    for api in apis {
        if lower.contains(&api.to_lowercase()) {
            features.insert(format!("api:{}", api.to_lowercase()));
        }
    }
}

fn extract_error_terms(lower: &str, features: &mut BTreeSet<String>) {
    let terms = [
        "typeerror",
        "referenceerror",
        "syntaxerror",
        "traceback",
        "sqlstate",
        "undefined",
        "not assignable",
        "does not exist",
        "cannot read",
    ];
    for term in terms {
        if lower.contains(term) {
            features.insert(format!("error:{}", term.replace(' ', "_")));
        }
    }
}

fn extract_phrases(lower: &str, features: &mut BTreeSet<String>) {
    let phrases = [
        "tanstack router",
        "react",
        "javascript",
        "typescript",
        "fastapi",
        "tailwind",
        "sql",
        "node",
        "promise",
    ];
    for phrase in phrases {
        if lower.contains(phrase) {
            features.insert(format!("phrase:{}", phrase.replace(' ', "_")));
        }
    }
}

fn quoted_package(line: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let mut parts = line.split(quote);
        parts.next();
        if let Some(candidate) = parts.next() {
            if is_package_like(candidate) {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

fn is_package_like(value: &str) -> bool {
    if value.is_empty() || value.starts_with('.') {
        return false;
    }
    value.contains('/')
        || value.starts_with('@')
        || matches!(
            value.to_lowercase().as_str(),
            "react" | "fastapi" | "sqlalchemy" | "tailwindcss" | "typescript"
        )
}

fn is_package_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '@' | '/' | '-' | '_' | '.' | ':')
}

struct Fnv1a64 {
    state: u64,
}

impl Fnv1a64 {
    fn new() -> Self {
        Self {
            state: 0xcbf29ce484222325,
        }
    }

    fn write_str(&mut self, value: &str) {
        for byte in value.as_bytes() {
            self.state ^= *byte as u64;
            self.state = self.state.wrapping_mul(0x100000001b3);
        }
        self.state ^= 0xff;
        self.state = self.state.wrapping_mul(0x100000001b3);
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lake_config::{load_expert_registry, LakeConfig};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture_registry() -> ExpertRegistry {
        load_expert_registry(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/lake/valid_two_experts/lake.yaml"),
        )
        .unwrap()
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("neuronlake-router-{name}-{nonce}.yaml"))
    }

    #[test]
    fn extracts_representative_coding_features() {
        let features = extract_routing_features(
            r#"
```tsx
import { createFileRoute } from '@tanstack/react-router'
export const Route = createFileRoute('/posts/$id')({
  loader: ({ params }) => params.id
})
```
Fix routes/posts.$id.tsx TypeError: loader is undefined
"#,
        );

        assert!(features.contains(&"code_lang:tsx".to_string()));
        assert!(features.contains(&"import:@tanstack/react-router".to_string()));
        assert!(features.contains(&"package:@tanstack/react-router".to_string()));
        assert!(features.contains(&"api:createfileroute".to_string()));
        assert!(features.contains(&"api:loader".to_string()));
        assert!(features.contains(&"ext:tsx".to_string()));
        assert!(features.contains(&"error:typeerror".to_string()));
        assert!(features.contains(&"error:undefined".to_string()));
    }

    #[test]
    fn creates_labels_and_training_examples_from_registry() {
        let registry = fixture_registry();
        let labels = labels_from_registry(&registry);
        let examples = training_examples_from_registry(&registry);

        assert_eq!(labels[0].expert_id, "javascript-core");
        assert_eq!(labels[1].expert_id, "tanstack-router");
        assert!(examples
            .iter()
            .any(|example| example.expert_id == "javascript-core"
                && example.source == TrainingExampleSource::RoutingHint
                && example.text == "promise"));
        assert!(examples
            .iter()
            .any(|example| example.expert_id == "tanstack-router"
                && example
                    .features
                    .contains(&"api:createfileroute".to_string())));
    }

    #[test]
    fn trains_and_routes_obvious_prompts_to_expected_experts() {
        let registry = fixture_registry();
        let router = ExpertRouter::train(&registry).unwrap();

        let js = router.route_text("Explain JavaScript promise async behavior in Node", false);
        let tanstack = router.route_text(
            "Fix a TanStack Router createFileRoute loader in routes/posts.$id.tsx",
            true,
        );

        assert_eq!(js.expert_id, "javascript-core");
        assert!(js.confidence > 0.0);
        assert_eq!(tanstack.expert_id, "tanstack-router");
        assert!(tanstack.confidence > 0.0);
        assert!(tanstack
            .debug
            .as_ref()
            .unwrap()
            .contributions
            .iter()
            .any(|contribution| contribution.feature == "api:createfileroute"));
    }

    #[test]
    fn persists_artifact_and_detects_stale_registry_inputs() {
        let registry = fixture_registry();
        let router = ExpertRouter::train(&registry).unwrap();
        let path = unique_temp_path("artifact");

        router.save_artifact(&path).unwrap();
        let artifact = RouterArtifact::load(&path).unwrap();
        let reloaded = ExpertRouter::from_artifact(artifact.clone()).unwrap();

        assert!(artifact.is_current_for(&registry));
        assert_eq!(reloaded.labels()[1].expert_id, "tanstack-router");

        let changed_yaml = r#"
name: frontend-lake
experts:
  - id: javascript-core
    domain: JavaScript language and runtime behavior
    model: ./models/javascript-core-0.5b.gguf
    routing_hints:
      - javascript
      - promise
      - esm
  - id: tanstack-router
    domain: TanStack Router application code
    model: ./models/tanstack-router-0.5b.gguf
    routing_hints:
      - tanstack router
      - createFileRoute
server:
  host: 127.0.0.1
  port: 8080
  model_name: library-lake-v1
"#;
        let config: LakeConfig = serde_yaml::from_str(changed_yaml).unwrap();
        let changed_registry = ExpertRegistry::from_config(&config, "/tmp/lake.yaml").unwrap();
        assert!(artifact.is_stale_for(&changed_registry));

        fs::remove_file(path).unwrap();
    }
}
