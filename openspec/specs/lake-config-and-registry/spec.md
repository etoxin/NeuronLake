# lake-config-and-registry Specification

## Purpose
TBD - created by archiving change define-lake-config-and-registry. Update Purpose after archive.
## Requirements
### Requirement: Load lake configuration
The system SHALL load a user-provided `lake.yaml` file containing lake metadata, expert definitions, and server settings.

#### Scenario: Valid lake configuration loads
- **WHEN** the user provides a syntactically valid `lake.yaml` with a lake name, at least one expert, and server settings
- **THEN** the system loads the configuration without requiring any teacher model configuration

### Requirement: Validate expert definitions
The system SHALL validate that each configured expert has a stable unique ID, a non-empty domain, and a local, remote, or imported model reference.

#### Scenario: Duplicate expert IDs are rejected
- **WHEN** `lake.yaml` defines two experts with the same `id`
- **THEN** validation fails with an error that identifies the duplicate expert ID

#### Scenario: Missing expert model is rejected
- **WHEN** an expert definition omits its model reference
- **THEN** validation fails with an error that identifies the affected expert

### Requirement: Resolve local paths relative to configuration
The system SHALL resolve relative local paths in `lake.yaml` against the directory containing the configuration file.

#### Scenario: Relative model path resolves from config directory
- **WHEN** `lake.yaml` at `/workspace/lake.yaml` references `./models/react.gguf`
- **THEN** the runtime registry records the resolved model path as `/workspace/models/react.gguf`

### Requirement: Validate server settings
The system SHALL validate configured server host, port, and exposed model name before server startup uses them.

#### Scenario: Invalid port is rejected
- **WHEN** `lake.yaml` configures a server port outside the valid TCP port range
- **THEN** validation fails with an error that identifies the invalid port

### Requirement: Preserve configured server model identity
The lake registry SHALL expose the configured `server.model_name` as the public model identity used by OpenAI-compatible endpoints and examples.

#### Scenario: Server model name becomes public API model
- **WHEN** `lake.yaml` configures `server.model_name`
- **THEN** the runtime registry exposes that model name for `/v1/models`, chat completion validation, and OpenCode provider configuration

### Requirement: Build expert registry
The system SHALL build a runtime expert registry from validated configuration.

#### Scenario: Registry contains configured expert metadata
- **WHEN** validation succeeds for an expert with ID, domain, model reference, routing hints, examples, and sharing metadata
- **THEN** the registry exposes those fields for downstream server, router, backend, and sharing components

### Requirement: Track expert runtime metadata
The system SHALL support registry metadata for local model paths, downloaded cache paths, imported shared experts, version information, compatibility information, and training status when those fields are configured or discovered.

#### Scenario: Imported expert metadata is preserved
- **WHEN** an expert is configured with imported package metadata and compatibility information
- **THEN** the registry preserves that metadata without requiring the model to be trained inside NeuronLake

### Requirement: Support imported local GGUF experts in MVP examples
The lake registry SHALL support manually imported local GGUF experts whose metadata records source, sharing, compatibility, version, and training status without requiring a teacher model.

#### Scenario: Imported GGUF metadata loads without teacher configuration
- **WHEN** `lake.yaml` defines local GGUF experts with sharing, compatibility, version, and imported training-status metadata but no teacher section
- **THEN** validation and registry construction succeed and preserve that metadata for routing, serving, and sharing components

### Requirement: Keep Milestone 1 independent from teacher configuration
The system SHALL allow Milestone 1 validation and registry construction to succeed without any teacher model configuration.

#### Scenario: Lake runtime config has no teacher section
- **WHEN** `lake.yaml` contains experts and server settings but no `teacher` section
- **THEN** validation and registry construction succeed for Milestone 1 runtime use
