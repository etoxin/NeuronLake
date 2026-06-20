## ADDED Requirements

### Requirement: Clamp generated output length
The local expert backend SHALL apply a configurable positive default output-token limit as both the fallback for omitted `max_tokens` and the upper bound for oversized client `max_tokens`.

#### Scenario: Missing max tokens uses backend default
- **WHEN** a backend generation request omits `max_tokens`
- **THEN** the backend invokes the local generation command with the configured default output-token limit

#### Scenario: Oversized max tokens is capped
- **WHEN** a backend generation request asks for more tokens than the configured backend output-token limit
- **THEN** the backend invokes the local generation command with the configured limit instead of the oversized client value

### Requirement: Identify subprocess backend boundary
The MVP llama.cpp backend SHALL execute generation through a local subprocess per request and report diagnostics as measured subprocess execution information rather than benchmark guarantees.

#### Scenario: Subprocess generation reports measured diagnostics
- **WHEN** the llama.cpp subprocess backend completes a generation request
- **THEN** the backend diagnostics identify the expert, model artifact path, backend name, and measured subprocess generation timing

#### Scenario: Documentation avoids resident-cache claims
- **WHEN** documentation describes the MVP backend behavior
- **THEN** it states that generation starts a fresh local process per request and does not claim resident expert caching or warm-swap latency
