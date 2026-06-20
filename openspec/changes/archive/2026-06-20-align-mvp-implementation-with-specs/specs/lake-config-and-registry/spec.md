## ADDED Requirements

### Requirement: Support imported local GGUF experts in MVP examples
The lake registry SHALL support manually imported local GGUF experts whose metadata records source, sharing, compatibility, version, and training status without requiring a teacher model.

#### Scenario: Imported GGUF metadata loads without teacher configuration
- **WHEN** `lake.yaml` defines local GGUF experts with sharing, compatibility, version, and imported training-status metadata but no teacher section
- **THEN** validation and registry construction succeed and preserve that metadata for routing, serving, and sharing components

### Requirement: Preserve configured server model identity
The lake registry SHALL expose the configured `server.model_name` as the public model identity used by OpenAI-compatible endpoints and examples.

#### Scenario: Server model name becomes public API model
- **WHEN** `lake.yaml` configures `server.model_name`
- **THEN** the runtime registry exposes that model name for `/v1/models`, chat completion validation, and OpenCode provider configuration
