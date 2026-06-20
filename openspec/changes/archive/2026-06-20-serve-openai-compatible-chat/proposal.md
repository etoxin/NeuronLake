## Why

NeuronLake needs an OpenAI-compatible HTTP surface so existing terminal-first agents can use it without custom client integration. The first compatibility target is OpenCode through `POST /v1/chat/completions`.

This change belongs to Milestone 1: The Lake.

## What Changes

- Add a local HTTP server that exposes `POST /v1/chat/completions`.
- Accept OpenAI-style chat completion requests with `model`, `messages`, `stream`, and basic generation parameters where supported.
- Return OpenAI-compatible non-streaming response shapes.
- Return OpenAI-style Server-Sent Events when `stream: true`.
- Validate requested model names against the configured lake server model name.
- Provide an OpenCode-compatible configuration example.
- Allow a deterministic or placeholder backend response until real expert execution is added by the backend change.
- Non-goals: production authentication, full OpenAI API coverage, model fine-tuning, memory pinning, and benchmark claims.

## Capabilities

### New Capabilities

- `openai-compatible-chat-server`: Defines the OpenAI-compatible chat endpoint, request/response behavior, SSE streaming contract, and OpenCode compatibility expectations.

### Modified Capabilities

- None.

## Impact

- Affected areas: server runtime, HTTP routing, request/response schemas, SSE framing, model-name validation, integration documentation, and tests for OpenCode-style traffic.
- Depends on `lake-config-and-registry` for host, port, model name, and configuration validation.
- Later changes will replace placeholder generation with routed expert execution without changing the client-facing API contract.
