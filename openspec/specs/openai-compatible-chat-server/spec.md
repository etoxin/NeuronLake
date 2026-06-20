# openai-compatible-chat-server Specification

## Purpose
TBD - created by archiving change serve-openai-compatible-chat. Update Purpose after archive.
## Requirements
### Requirement: Start local chat server
The system SHALL start a local HTTP server using the host, port, and exposed model name from validated `lake.yaml` server settings.

#### Scenario: Server starts from validated settings
- **WHEN** the user starts NeuronLake with a valid `lake.yaml`
- **THEN** the server listens on the configured host and port and exposes the configured model name

### Requirement: Accept chat completion requests
The system SHALL expose `POST /v1/chat/completions` and accept OpenAI-style chat completion requests containing `model`, `messages`, `stream`, and supported generation parameters.

#### Scenario: OpenCode-style request is accepted
- **WHEN** OpenCode sends a chat completion request to `/v1/chat/completions` with the configured model name and chat messages
- **THEN** the server accepts the request for generation

### Requirement: Validate requested model name
The system SHALL reject chat completion requests whose `model` does not match the configured lake model name.

#### Scenario: Unknown model is rejected
- **WHEN** a request specifies a model name that is not the configured lake model name
- **THEN** the server returns an OpenAI-compatible error response instead of generating text

### Requirement: List configured models
The system SHALL expose `GET /v1/models` with an OpenAI-compatible model list containing the configured lake model name.

#### Scenario: Model list includes configured lake model
- **WHEN** a client sends `GET /v1/models` to a running NeuronLake server
- **THEN** the response includes a list object with the configured lake model name and NeuronLake ownership metadata

### Requirement: Accept provider-prefixed model aliases
The system SHALL accept OpenCode-style provider-prefixed model IDs whose final path segment matches the configured lake model name.

#### Scenario: Provider-prefixed model alias is accepted
- **WHEN** a chat completion request uses model `neuronlake/<configured-model-name>`
- **THEN** the server treats the request as targeting `<configured-model-name>` and proceeds with normal request handling

#### Scenario: Non-matching provider-prefixed model alias is rejected
- **WHEN** a chat completion request uses a provider-prefixed model whose final path segment does not match the configured lake model name
- **THEN** the server returns an OpenAI-compatible model error instead of generating text

### Requirement: Return non-streaming chat response
The system SHALL return an OpenAI-compatible chat completion JSON response when `stream` is absent or false.

#### Scenario: Non-streaming response shape is compatible
- **WHEN** a valid request sets `stream` to false
- **THEN** the response includes an ID, object type, created timestamp, model name, choices with assistant message content, and usage information when available

### Requirement: Stream Server-Sent Events
The system SHALL return OpenAI-style Server-Sent Events when a valid request sets `stream` to true.

#### Scenario: Streaming response emits chunks and done marker
- **WHEN** a valid request sets `stream` to true
- **THEN** the server emits `data:` events containing chat completion chunks and terminates the stream with `data: [DONE]`

### Requirement: Offload blocking generation
The system SHALL run blocking backend generation work outside the async HTTP request executor.

#### Scenario: Blocking backend is invoked from chat request
- **WHEN** a valid non-streaming or streaming chat request is handled by a backend that executes a blocking local process
- **THEN** the server awaits the result from an offloaded generation task before formatting the OpenAI-compatible response

### Requirement: Preserve API contract with placeholder generation
The system SHALL support deterministic placeholder generation before real local expert execution is implemented.

#### Scenario: Placeholder backend produces compatible output
- **WHEN** the chat server is running without a real expert backend
- **THEN** valid non-streaming and streaming requests still receive OpenAI-compatible response shapes

### Requirement: Provide OpenCode configuration example
The project SHALL document an OpenCode provider configuration that points to the local `/v1` base URL and configured lake model name.

#### Scenario: User follows OpenCode example
- **WHEN** a user copies the documented OpenCode provider configuration and starts NeuronLake on the matching host, port, and model name
- **THEN** OpenCode can send chat completion requests without NeuronLake-specific client code

### Requirement: Provide constrained OpenCode MVP example
The project SHALL include an OpenCode example that uses the local OpenAI-compatible provider, disables tool use, and documents that the bundled 0.5B model is an integration smoke test rather than a full OpenCode coding-agent model.

#### Scenario: OpenCode example targets local NeuronLake provider
- **WHEN** a user runs the OpenCode example with the NeuronLake MVP server listening on the documented local `/v1` base URL
- **THEN** OpenCode can send chat completion requests using the configured provider and model without NeuronLake-specific client code

#### Scenario: OpenCode example remains bounded
- **WHEN** OpenCode sends a request through the example configuration
- **THEN** the request is constrained by the example's no-tool agent configuration and documented output limit
