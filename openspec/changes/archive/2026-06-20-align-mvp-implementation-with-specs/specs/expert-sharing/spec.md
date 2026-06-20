## ADDED Requirements

### Requirement: Preserve external source metadata for MVP experts
The system SHALL preserve source and compatibility metadata for shareable experts that reference external GGUF artifacts such as Hugging Face model files.

#### Scenario: Hugging Face GGUF source metadata is preserved
- **WHEN** an expert definition references a local GGUF artifact downloaded from an external source and includes sharing source, license, compatibility, and training-status metadata
- **THEN** the registry and sharing metadata preserve those fields without requiring the large model artifact to be embedded in source control

### Requirement: Keep package metadata useful before training workflows
The system SHALL allow imported or manually configured expert metadata to be exported, inspected, or routed before any teacher-student training workflow exists.

#### Scenario: Imported expert is usable before distillation
- **WHEN** an expert has imported metadata and a compatible local model artifact
- **THEN** the expert remains available for registry inspection, routing, serving, and future export without requiring distillation state
