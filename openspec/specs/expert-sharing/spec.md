# expert-sharing Specification

## Purpose
TBD - created by archiving change package-shareable-experts. Update Purpose after archive.
## Requirements
### Requirement: Define expert package manifest
The system SHALL define a shareable expert package manifest containing expert ID, domain description, model artifact reference, routing hints, training metadata, version information, and compatibility metadata.

#### Scenario: Manifest contains required fields
- **WHEN** an expert package is inspected
- **THEN** its manifest identifies the expert ID, domain, and model artifact reference

### Requirement: Export configured expert package
The system SHALL export a configured expert and available metadata into a local folder or archive package.

#### Scenario: Expert export writes package
- **WHEN** the user exports an expert from a valid lake registry
- **THEN** the system writes a package manifest and included metadata for that expert

### Requirement: Support external model artifact references
The system SHALL allow a package manifest to reference large model artifacts without requiring the weight file to be embedded in the package.

#### Scenario: Package references external weights
- **WHEN** an expert package uses an external model artifact reference
- **THEN** package validation records the reference without requiring the weight file to be stored in source control

### Requirement: Preserve external source metadata for MVP experts
The system SHALL preserve source and compatibility metadata for shareable experts that reference external GGUF artifacts such as Hugging Face model files.

#### Scenario: Hugging Face GGUF source metadata is preserved
- **WHEN** an expert definition references a local GGUF artifact downloaded from an external source and includes sharing source, license, compatibility, and training-status metadata
- **THEN** the registry and sharing metadata preserve those fields without requiring the large model artifact to be embedded in source control

### Requirement: Validate package before import
The system SHALL validate package manifest shape, compatibility metadata, duplicate expert IDs, and artifact availability before importing an expert into a lake.

#### Scenario: Duplicate imported expert ID is rejected
- **WHEN** a package contains an expert ID already present in the target lake and overwrite is not explicitly requested
- **THEN** import fails with an error that identifies the conflicting expert ID

### Requirement: Import shared expert into registry
The system SHALL make a successfully imported expert available as a first-class registry entry for serving, routing, and future export.

#### Scenario: Imported expert is available downstream
- **WHEN** a valid expert package is imported into a lake
- **THEN** the registry exposes its ID, domain, model reference, routing hints, and compatibility metadata to downstream components

### Requirement: Preserve training and compatibility metadata
The system SHALL preserve package training metadata and compatibility metadata even when the current runtime does not use all fields.

#### Scenario: Unknown optional metadata is preserved
- **WHEN** a package contains optional training metadata not used by the current serving runtime
- **THEN** the metadata remains available for future inspection or teacher-student workflows

### Requirement: Keep package metadata useful before training workflows
The system SHALL allow imported or manually configured expert metadata to be exported, inspected, or routed before any teacher-student training workflow exists.

#### Scenario: Imported expert is usable before distillation
- **WHEN** an expert has imported metadata and a compatible local model artifact
- **THEN** the expert remains available for registry inspection, routing, serving, and future export without requiring distillation state
