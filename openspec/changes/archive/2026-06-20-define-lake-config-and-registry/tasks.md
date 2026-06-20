## 1. Config Model

- [x] 1.1 Add YAML parsing dependencies and typed `lake.yaml` config structs.
- [x] 1.2 Define expert, model reference, server, routing hint, example, sharing metadata, compatibility, and training status fields.
- [x] 1.3 Resolve relative local paths from the directory containing `lake.yaml`.
- [x] 1.4 Add sample valid `lake.yaml` fixtures with at least two experts.

## 2. Validation

- [x] 2.1 Implement validation for required lake name, experts, expert IDs, domains, model references, host, port, and model name.
- [x] 2.2 Collect actionable validation errors for duplicate expert IDs and missing model references.
- [x] 2.3 Add tests for valid config, duplicate IDs, missing model, invalid port, and teacher-free Milestone 1 config.

## 3. Registry

- [x] 3.1 Implement a runtime expert registry built only from validated config.
- [x] 3.2 Preserve original and resolved model references in registry entries.
- [x] 3.3 Preserve imported/shared metadata, version, compatibility, and training status when present.
- [x] 3.4 Add tests that downstream components can query registry entries by expert ID.

## 4. Documentation

- [x] 4.1 Document the Milestone 1 `lake.yaml` fields and path resolution behavior.
- [x] 4.2 Document that teacher configuration is not required for The Lake runtime.
