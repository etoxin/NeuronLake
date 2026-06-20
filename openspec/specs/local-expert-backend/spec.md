# local-expert-backend Specification

## Purpose
TBD - created by archiving change run-local-expert-backend. Update Purpose after archive.
## Requirements
### Requirement: Define local expert backend interface
The system SHALL provide an internal backend interface that can generate responses for a selected expert ID without exposing backend-specific details to the OpenAI-compatible server.

#### Scenario: Server calls backend by expert ID
- **WHEN** the server receives a valid chat request and an expert ID has been selected
- **THEN** it can request generation through the backend interface using that expert ID

### Requirement: Load configured local expert
The system SHALL load or prepare a configured local expert model from the expert registry before generation.

#### Scenario: Existing model file can be prepared
- **WHEN** the selected expert references an existing local GGUF-compatible model file
- **THEN** the backend prepares that expert for generation

### Requirement: Report missing model artifacts
The system SHALL return an actionable runtime error when a selected expert references a missing or unreadable local model artifact.

#### Scenario: Missing model file fails clearly
- **WHEN** generation is requested for an expert whose local model file does not exist
- **THEN** the backend returns an error that identifies the expert ID and missing artifact

### Requirement: Generate non-streaming output
The system SHALL generate a complete assistant response from the selected local expert for non-streaming requests.

#### Scenario: Local expert returns a response
- **WHEN** a valid non-streaming chat request is handled by a prepared expert backend
- **THEN** the backend returns generated assistant text to the server response layer

### Requirement: Stream generated token events
The system SHALL expose streaming token or text-delta events from the selected local expert as they become available.

#### Scenario: Backend streams deltas
- **WHEN** a valid streaming chat request is handled by a prepared expert backend
- **THEN** the backend emits ordered generation deltas that the server can frame as SSE chunks

### Requirement: Map supported generation parameters
The system SHALL map supported chat completion generation parameters into backend options and handle unsupported parameters predictably.

#### Scenario: Supported temperature is passed through
- **WHEN** a request includes a supported `temperature` parameter
- **THEN** the backend receives the corresponding generation option

#### Scenario: Unsupported parameter is handled predictably
- **WHEN** a request includes a generation parameter unsupported by the selected backend
- **THEN** the system either ignores it according to documented behavior or returns an OpenAI-compatible validation error

### Requirement: Clamp generated output length
The local expert backend SHALL apply a configurable positive default output-token limit as both the fallback for omitted `max_tokens` and the upper bound for oversized client `max_tokens`.

#### Scenario: Missing max tokens uses backend default
- **WHEN** a backend generation request omits `max_tokens`
- **THEN** the backend invokes the local generation command with the configured default output-token limit

#### Scenario: Oversized max tokens is capped
- **WHEN** a backend generation request asks for more tokens than the configured backend output-token limit
- **THEN** the backend invokes the local generation command with the configured limit instead of the oversized client value

### Requirement: Keep backend performance measurable
The system SHALL avoid claiming specific generation latency, startup latency, or resident expert counts without measurements captured by tests or benchmarks.

#### Scenario: Backend reports measured startup data
- **WHEN** backend performance information is displayed or documented
- **THEN** it is identified as measured data, a target, or an implementation note rather than an unverified guarantee

### Requirement: Identify subprocess backend boundary
The MVP llama.cpp backend SHALL execute generation through a local subprocess per request and report diagnostics as measured subprocess execution information rather than benchmark guarantees.

#### Scenario: Subprocess generation reports measured diagnostics
- **WHEN** the llama.cpp subprocess backend completes a generation request
- **THEN** the backend diagnostics identify the expert, model artifact path, backend name, and measured subprocess generation timing

#### Scenario: Documentation avoids resident-cache claims
- **WHEN** documentation describes the MVP backend behavior
- **THEN** it states that generation starts a fresh local process per request and does not claim resident expert caching or warm-swap latency
