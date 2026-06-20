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

use crate::lake_config::{ExpertRegistry, RuntimeServerSettings};
use crate::local_backend::{
    BackendError, BackendMessage, BackendRequest, ExpertBackend, GenerationOptions,
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct ChatServerState {
    model_name: String,
    generator: Arc<dyn ChatGenerator>,
}

impl ChatServerState {
    pub fn new(model_name: impl Into<String>, generator: Arc<dyn ChatGenerator>) -> Self {
        Self {
            model_name: model_name.into(),
            generator,
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }
}

pub trait ChatGenerator: Send + Sync + 'static {
    fn generate(&self, request: &ChatCompletionRequest) -> Result<GeneratedChat, ChatServerError>;

    fn stream(&self, request: &ChatCompletionRequest) -> Result<Vec<String>, ChatServerError> {
        Ok(vec![self.generate(request)?.content])
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlaceholderGenerator;

impl ChatGenerator for PlaceholderGenerator {
    fn generate(&self, request: &ChatCompletionRequest) -> Result<GeneratedChat, ChatServerError> {
        let prompt = request
            .messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .and_then(|message| message.content_text())
            .unwrap_or_else(|| "no user message".to_string());

        Ok(GeneratedChat {
            content: format!(
                "NeuronLake placeholder response for model '{}'. Last user message: {}",
                request.model, prompt
            ),
        })
    }

    fn stream(&self, request: &ChatCompletionRequest) -> Result<Vec<String>, ChatServerError> {
        Ok(split_stream_content(&self.generate(request)?.content))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedChat {
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stop: Option<Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Value,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ChatMessage {
    pub fn content_text(&self) -> Option<String> {
        value_to_text(&self.content)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: AssistantMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssistantMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChunkChoice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionChunkChoice {
    pub index: u32,
    pub delta: ChatCompletionDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ChatCompletionDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiErrorResponse {
    pub error: OpenAiError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug)]
pub enum ChatServerError {
    InvalidModel { requested: String, expected: String },
    InvalidRequest(String),
    Backend(BackendError),
    BindAddress(std::net::AddrParseError),
    Io(std::io::Error),
    Serve(std::io::Error),
}

impl fmt::Display for ChatServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModel {
                requested,
                expected,
            } => write!(
                f,
                "requested model '{requested}' does not match configured model '{expected}'"
            ),
            Self::InvalidRequest(message) => write!(f, "{message}"),
            Self::Backend(error) => write!(f, "{error}"),
            Self::BindAddress(error) => write!(f, "invalid server bind address: {error}"),
            Self::Io(error) => write!(f, "server I/O error: {error}"),
            Self::Serve(error) => write!(f, "server failed: {error}"),
        }
    }
}

impl std::error::Error for ChatServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BindAddress(error) => Some(error),
            Self::Io(error) | Self::Serve(error) => Some(error),
            Self::Backend(error) => Some(error),
            Self::InvalidModel { .. } | Self::InvalidRequest(_) => None,
        }
    }
}

impl From<BackendError> for ChatServerError {
    fn from(error: BackendError) -> Self {
        Self::Backend(error)
    }
}

impl IntoResponse for ChatServerError {
    fn into_response(self) -> Response {
        let (status, error_type, param, code, message) = match self {
            Self::InvalidModel {
                requested,
                expected,
            } => (
                StatusCode::NOT_FOUND,
                "invalid_request_error",
                Some("model".to_string()),
                Some("model_not_found".to_string()),
                format!("Model '{requested}' is not available. Use '{expected}'."),
            ),
            Self::InvalidRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                None,
                Some("invalid_request".to_string()),
                message,
            ),
            Self::Backend(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                None,
                Some("backend_error".to_string()),
                error.to_string(),
            ),
            other => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                None,
                Some("server_error".to_string()),
                other.to_string(),
            ),
        };

        (
            status,
            Json(OpenAiErrorResponse {
                error: OpenAiError {
                    message,
                    error_type: error_type.to_string(),
                    param,
                    code,
                },
            }),
        )
            .into_response()
    }
}

pub fn router_from_registry(registry: &ExpertRegistry) -> Router {
    router_from_settings(registry.server().clone())
}

pub fn router_from_settings(settings: RuntimeServerSettings) -> Router {
    router_with_generator(settings, Arc::new(PlaceholderGenerator))
}

pub fn router_with_generator(
    settings: RuntimeServerSettings,
    generator: Arc<dyn ChatGenerator>,
) -> Router {
    let state = ChatServerState::new(settings.model_name, generator);
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

pub fn router_with_backend(
    registry: ExpertRegistry,
    selected_expert_id: impl Into<String>,
    backend: Arc<dyn ExpertBackend>,
) -> Router {
    let settings = registry.server().clone();
    router_with_generator(
        settings,
        Arc::new(BackendChatGenerator::new(
            registry,
            selected_expert_id,
            backend,
        )),
    )
}

pub async fn serve_from_registry(registry: &ExpertRegistry) -> Result<(), ChatServerError> {
    serve_settings(registry.server().clone()).await
}

pub async fn serve_settings(settings: RuntimeServerSettings) -> Result<(), ChatServerError> {
    let host: IpAddr = settings
        .host
        .parse()
        .map_err(ChatServerError::BindAddress)?;
    let address = SocketAddr::new(host, settings.port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(ChatServerError::Io)?;
    axum::serve(listener, router_from_settings(settings))
        .await
        .map_err(ChatServerError::Serve)
}

pub struct BackendChatGenerator {
    registry: Arc<ExpertRegistry>,
    selected_expert_id: String,
    backend: Arc<dyn ExpertBackend>,
}

impl BackendChatGenerator {
    pub fn new(
        registry: ExpertRegistry,
        selected_expert_id: impl Into<String>,
        backend: Arc<dyn ExpertBackend>,
    ) -> Self {
        Self {
            registry: Arc::new(registry),
            selected_expert_id: selected_expert_id.into(),
            backend,
        }
    }

    fn backend_request(&self, request: &ChatCompletionRequest) -> BackendRequest {
        let ignored_parameters = request.extra.keys().cloned().collect();

        BackendRequest {
            expert_id: self.selected_expert_id.clone(),
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|message| BackendMessage {
                    role: message.role.clone(),
                    content: message.content_text().unwrap_or_default(),
                })
                .collect(),
            options: GenerationOptions {
                temperature: request.temperature,
                top_p: request.top_p,
                max_tokens: request.max_tokens,
                stop: request.stop.clone(),
            },
            ignored_parameters,
        }
    }
}

impl ChatGenerator for BackendChatGenerator {
    fn generate(&self, request: &ChatCompletionRequest) -> Result<GeneratedChat, ChatServerError> {
        let backend_request = self.backend_request(request);
        let prepared = self
            .backend
            .prepare(&self.registry, &self.selected_expert_id)?;
        let completion = self.backend.generate(&prepared, &backend_request)?;
        Ok(GeneratedChat {
            content: completion.content,
        })
    }

    fn stream(&self, request: &ChatCompletionRequest) -> Result<Vec<String>, ChatServerError> {
        let backend_request = self.backend_request(request);
        let prepared = self
            .backend
            .prepare(&self.registry, &self.selected_expert_id)?;
        Ok(self
            .backend
            .stream(&prepared, &backend_request)?
            .into_iter()
            .map(|delta| delta.content)
            .collect())
    }
}

async fn chat_completions(
    State(state): State<ChatServerState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, ChatServerError> {
    validate_request(&state, &request)?;

    if request.stream {
        Ok(streaming_response(&state, &request)?.into_response())
    } else {
        Ok(Json(non_streaming_response(&state, &request)?).into_response())
    }
}

fn validate_request(
    state: &ChatServerState,
    request: &ChatCompletionRequest,
) -> Result<(), ChatServerError> {
    if request.model != state.model_name {
        return Err(ChatServerError::InvalidModel {
            requested: request.model.clone(),
            expected: state.model_name.clone(),
        });
    }

    if request.messages.is_empty() {
        return Err(ChatServerError::InvalidRequest(
            "messages must contain at least one chat message".to_string(),
        ));
    }

    Ok(())
}

fn non_streaming_response(
    state: &ChatServerState,
    request: &ChatCompletionRequest,
) -> Result<ChatCompletionResponse, ChatServerError> {
    let generated = state.generator.generate(request)?;
    let prompt_tokens = estimate_messages_tokens(&request.messages);
    let completion_tokens = estimate_tokens(&generated.content);

    Ok(ChatCompletionResponse {
        id: completion_id(),
        object: "chat.completion",
        created: unix_timestamp(),
        model: state.model_name.clone(),
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: AssistantMessage {
                role: "assistant",
                content: generated.content,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Some(Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }),
    })
}

fn streaming_response(
    state: &ChatServerState,
    request: &ChatCompletionRequest,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, ChatServerError> {
    let id = completion_id();
    let created = unix_timestamp();
    let model = state.model_name.clone();
    let mut events = Vec::new();

    events.push(chunk_event(ChatCompletionChunk {
        id: id.clone(),
        object: "chat.completion.chunk",
        created,
        model: model.clone(),
        choices: vec![ChatCompletionChunkChoice {
            index: 0,
            delta: ChatCompletionDelta {
                role: Some("assistant"),
                content: None,
            },
            finish_reason: None,
        }],
    })?);

    for delta in state.generator.stream(request)? {
        events.push(chunk_event(ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk",
            created,
            model: model.clone(),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionDelta {
                    role: None,
                    content: Some(delta),
                },
                finish_reason: None,
            }],
        })?);
    }

    events.push(chunk_event(ChatCompletionChunk {
        id,
        object: "chat.completion.chunk",
        created,
        model,
        choices: vec![ChatCompletionChunkChoice {
            index: 0,
            delta: ChatCompletionDelta::default(),
            finish_reason: Some("stop".to_string()),
        }],
    })?);
    events.push(Event::default().data("[DONE]"));

    Ok(Sse::new(stream::iter(events.into_iter().map(Ok))))
}

fn chunk_event(chunk: ChatCompletionChunk) -> Result<Event, ChatServerError> {
    serde_json::to_string(&chunk)
        .map(|data| Event::default().data(data))
        .map_err(|error| ChatServerError::InvalidRequest(error.to_string()))
}

fn split_stream_content(content: &str) -> Vec<String> {
    let mut chunks: Vec<String> = content
        .split_inclusive(' ')
        .map(str::to_string)
        .filter(|chunk| !chunk.is_empty())
        .collect();

    if chunks.is_empty() {
        chunks.push(String::new());
    }

    chunks
}

fn value_to_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(parts) => {
            let text = parts
                .iter()
                .filter_map(|part| match part {
                    Value::String(text) => Some(text.as_str()),
                    Value::Object(object) => object.get("text").and_then(Value::as_str),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            (!text.is_empty()).then_some(text)
        }
        Value::Object(object) => object
            .get("text")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn estimate_messages_tokens(messages: &[ChatMessage]) -> u32 {
    messages
        .iter()
        .filter_map(ChatMessage::content_text)
        .map(|content| estimate_tokens(&content))
        .sum()
}

fn estimate_tokens(text: &str) -> u32 {
    text.split_whitespace().count().max(1) as u32
}

fn completion_id() -> String {
    format!("chatcmpl-{}", unix_timestamp())
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

pub fn opencode_provider_example(base_url: &str, model_name: &str) -> Value {
    let mut models = Map::new();
    models.insert(
        model_name.to_string(),
        json!({
            "name": "NeuronLake"
        }),
    );

    json!({
        "provider": {
            "neuronlake": {
                "npm": "@ai-sdk/openai-compatible",
                "options": {
                    "baseURL": base_url,
                    "apiKey": "neuronlake-handshake"
                },
                "models": models
            }
        },
        "model": format!("neuronlake/{model_name}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lake_config::load_expert_registry;
    use crate::local_backend::{
        BackendCompletion, BackendDelta, BackendDiagnostics, BackendTiming, PreparedExpert,
    };
    use axum::body::to_bytes;
    use axum::http::header::CONTENT_TYPE;
    use axum::http::{Request, StatusCode};
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tower::ServiceExt;

    fn test_settings() -> RuntimeServerSettings {
        RuntimeServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
            model_name: "library-lake-v1".to_string(),
        }
    }

    fn chat_body(stream: bool) -> String {
        if stream {
            include_str!("../tests/fixtures/opencode/chat_completion_stream_request.json")
        } else {
            include_str!("../tests/fixtures/opencode/chat_completion_request.json")
        }
        .to_string()
    }

    fn fixture_registry() -> ExpertRegistry {
        load_expert_registry(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/lake/valid_two_experts/lake.yaml"),
        )
        .unwrap()
    }

    struct RecordingBackend {
        requests: Mutex<Vec<BackendRequest>>,
        deltas: Vec<String>,
    }

    impl RecordingBackend {
        fn new(deltas: impl IntoIterator<Item = impl Into<String>>) -> Self {
            Self {
                requests: Mutex::new(Vec::new()),
                deltas: deltas.into_iter().map(Into::into).collect(),
            }
        }

        fn request_count(&self) -> usize {
            self.requests.lock().unwrap().len()
        }

        fn last_request(&self) -> BackendRequest {
            self.requests.lock().unwrap().last().unwrap().clone()
        }
    }

    impl ExpertBackend for RecordingBackend {
        fn prepare(
            &self,
            registry: &ExpertRegistry,
            expert_id: &str,
        ) -> Result<PreparedExpert, BackendError> {
            let expert = registry
                .get(expert_id)
                .ok_or_else(|| BackendError::ExpertNotFound {
                    expert_id: expert_id.to_string(),
                })?;
            Ok(PreparedExpert {
                expert_id: expert.id.clone(),
                domain: expert.domain.clone(),
                model_path: PathBuf::from("/tmp/test-model.gguf"),
                backend_name: "recording-backend".to_string(),
                preparation: BackendTiming { elapsed_micros: 1 },
            })
        }

        fn generate(
            &self,
            prepared: &PreparedExpert,
            request: &BackendRequest,
        ) -> Result<BackendCompletion, BackendError> {
            self.requests.lock().unwrap().push(request.clone());
            Ok(BackendCompletion {
                content: format!("backend response from {}", prepared.expert_id),
                diagnostics: BackendDiagnostics {
                    backend_name: prepared.backend_name.clone(),
                    expert_id: prepared.expert_id.clone(),
                    model_path: prepared.model_path.clone(),
                    preparation: prepared.preparation,
                    generation: BackendTiming { elapsed_micros: 2 },
                    note: Some("test double timing is not a benchmark target".to_string()),
                },
            })
        }

        fn stream(
            &self,
            prepared: &PreparedExpert,
            request: &BackendRequest,
        ) -> Result<Vec<BackendDelta>, BackendError> {
            self.requests.lock().unwrap().push(request.clone());
            Ok(self
                .deltas
                .iter()
                .enumerate()
                .map(|(index, content)| BackendDelta {
                    content: format!("{}:{content}", prepared.expert_id),
                    index,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn non_streaming_chat_completion_shape_matches_openai_style() {
        let app = router_from_settings(test_settings());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(CONTENT_TYPE, "application/json")
                    .body(chat_body(false))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(value["object"], "chat.completion");
        assert_eq!(value["model"], "library-lake-v1");
        assert_eq!(value["choices"][0]["message"]["role"], "assistant");
        assert!(value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap()
            .contains("Fix this OpenCode request path."));
        assert!(value["usage"]["total_tokens"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn unknown_model_returns_openai_style_error() {
        let app = router_from_settings(test_settings());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(CONTENT_TYPE, "application/json")
                    .body(
                        json!({
                            "model": "wrong-model",
                            "messages": [{"role": "user", "content": "Hello"}]
                        })
                        .to_string(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["error"]["type"], "invalid_request_error");
        assert_eq!(value["error"]["param"], "model");
        assert_eq!(value["error"]["code"], "model_not_found");
    }

    #[tokio::test]
    async fn streaming_chat_completion_uses_sse_chunks_and_done_marker() {
        let app = router_from_settings(test_settings());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(CONTENT_TYPE, "application/json")
                    .body(chat_body(true))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("text/event-stream")
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("data: {\"id\":\"chatcmpl-"));
        assert!(text.contains("\"object\":\"chat.completion.chunk\""));
        assert!(text.contains("\"delta\":{\"role\":\"assistant\"}"));
        assert!(text.contains("Stream "));
        assert!(text.contains("this "));
        assert!(text.contains("OpenCode "));
        assert!(text.contains("response."));
        assert!(text.contains("data: [DONE]"));
    }

    #[tokio::test]
    async fn backend_generator_calls_selected_expert_with_generation_options() {
        let backend = Arc::new(RecordingBackend::new(["unused"]));
        let app = router_with_backend(fixture_registry(), "javascript-core", backend.clone());
        let body = json!({
            "model": "library-lake-v1",
            "messages": [{"role": "user", "content": "Use the backend"}],
            "temperature": 0.3,
            "top_p": 0.8,
            "max_tokens": 12,
            "presence_penalty": 0.1
        })
        .to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value["choices"][0]["message"]["content"],
            "backend response from javascript-core"
        );

        assert_eq!(backend.request_count(), 1);
        let request = backend.last_request();
        assert_eq!(request.expert_id, "javascript-core");
        assert_eq!(request.options.temperature, Some(0.3));
        assert_eq!(request.options.top_p, Some(0.8));
        assert_eq!(request.options.max_tokens, Some(12));
        assert_eq!(request.ignored_parameters, vec!["presence_penalty"]);
    }

    #[tokio::test]
    async fn backend_stream_deltas_become_ordered_sse_chunks() {
        let backend = Arc::new(RecordingBackend::new(["first ", "second"]));
        let app = router_with_backend(fixture_registry(), "javascript-core", backend.clone());
        let body = json!({
            "model": "library-lake-v1",
            "messages": [{"role": "user", "content": "Stream through backend"}],
            "stream": true
        })
        .to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        let first = text.find("javascript-core:first ").unwrap();
        let second = text.find("javascript-core:second").unwrap();
        assert!(first < second);
        assert!(text.contains("data: [DONE]"));
        assert_eq!(backend.request_count(), 1);
    }

    #[test]
    fn opencode_provider_example_uses_local_v1_base_url() {
        let value = opencode_provider_example("http://127.0.0.1:8080/v1", "library-lake-v1");
        assert_eq!(
            value["provider"]["neuronlake"]["options"]["baseURL"],
            "http://127.0.0.1:8080/v1"
        );
        assert_eq!(value["model"], "neuronlake/library-lake-v1");
    }
}
