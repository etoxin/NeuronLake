## 1. Server Setup

- [ ] 1.1 Add the HTTP server module and selected Rust HTTP/SSE dependencies.
- [ ] 1.2 Wire server startup to validated `lake.yaml` host, port, and model name.
- [ ] 1.3 Define OpenAI-compatible chat request, response, error, and streaming chunk types.

## 2. Chat Completion Handler

- [ ] 2.1 Implement `POST /v1/chat/completions` request parsing.
- [ ] 2.2 Validate that the request `model` matches the configured lake model name.
- [ ] 2.3 Implement deterministic placeholder generation behind the internal generation interface.
- [ ] 2.4 Return OpenAI-compatible non-streaming chat completion responses.

## 3. SSE Streaming

- [ ] 3.1 Implement OpenAI-style SSE chunk framing for `stream: true`.
- [ ] 3.2 Emit ordered `data:` chunks and terminate with `data: [DONE]`.
- [ ] 3.3 Add tests for streaming response headers, chunk shape, and done marker.

## 4. OpenCode Compatibility

- [ ] 4.1 Add an OpenCode provider configuration example using the local `/v1` base URL.
- [ ] 4.2 Add integration tests or fixtures for OpenCode-style non-streaming requests.
- [ ] 4.3 Add integration tests or fixtures for OpenCode-style streaming requests.
