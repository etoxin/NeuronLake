## ADDED Requirements

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

### Requirement: Offload blocking generation
The system SHALL run blocking backend generation work outside the async HTTP request executor.

#### Scenario: Blocking backend is invoked from chat request
- **WHEN** a valid non-streaming or streaming chat request is handled by a backend that executes a blocking local process
- **THEN** the server awaits the result from an offloaded generation task before formatting the OpenAI-compatible response

### Requirement: Provide constrained OpenCode MVP example
The project SHALL include an OpenCode example that uses the local OpenAI-compatible provider, disables tool use, and documents that the bundled 0.5B model is an integration smoke test rather than a full OpenCode coding-agent model.

#### Scenario: OpenCode example targets local NeuronLake provider
- **WHEN** a user runs the OpenCode example with the NeuronLake MVP server listening on the documented local `/v1` base URL
- **THEN** OpenCode can send chat completion requests using the configured provider and model without NeuronLake-specific client code

#### Scenario: OpenCode example remains bounded
- **WHEN** OpenCode sends a request through the example configuration
- **THEN** the request is constrained by the example's no-tool agent configuration and documented output limit
