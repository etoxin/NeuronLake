## ADDED Requirements

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
