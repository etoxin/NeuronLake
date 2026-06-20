## 1. Server Setup

- [x] 1.1 Add the HTTP server module and selected Rust HTTP/SSE dependencies.
- [x] 1.2 Wire server startup to validated `lake.yaml` host, port, and model name.
- [x] 1.3 Define OpenAI-compatible chat request, response, error, and streaming chunk types.

## 2. Chat Completion Handler

- [x] 2.1 Implement `POST /v1/chat/completions` request parsing.
- [x] 2.2 Validate that the request `model` matches the configured lake model name.
- [x] 2.3 Implement deterministic placeholder generation behind the internal generation interface.
- [x] 2.4 Return OpenAI-compatible non-streaming chat completion responses.

## 3. SSE Streaming

- [x] 3.1 Implement OpenAI-style SSE chunk framing for `stream: true`.
- [x] 3.2 Emit ordered `data:` chunks and terminate with `data: [DONE]`.
- [x] 3.3 Add tests for streaming response headers, chunk shape, and done marker.

## 4. OpenCode Compatibility

- [x] 4.1 Add an OpenCode provider configuration example using the local `/v1` base URL.
- [x] 4.2 Add integration tests or fixtures for OpenCode-style non-streaming requests.
- [x] 4.3 Add integration tests or fixtures for OpenCode-style streaming requests.
