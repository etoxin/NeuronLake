## Why

Milestone 1 needs a user-owned source of truth before serving, routing, backend execution, or sharing can be reliable. `lake.yaml` and an in-memory expert registry establish the contract for what experts exist, where their artifacts live, and how the rest of NeuronLake should interpret them.

This change belongs to Milestone 1: The Lake.

## What Changes

- Add a hand-editable `lake.yaml` configuration format for lake metadata, expert definitions, routing hints, examples, model references, server settings, and optional sharing metadata.
- Add validation for required fields, unique expert IDs, model source shape, server settings, and unsupported or ambiguous configuration.
- Add a runtime expert registry built from validated configuration.
- Track local paths, downloaded cache paths, imported shared experts, version information, compatibility metadata, and training status where present.
- Keep teacher-student configuration out of the required Milestone 1 serving path.
- Non-goals: automated fine-tuning, memory pinning of all experts, benchmark claims, and model execution.

## Capabilities

### New Capabilities

- `lake-config-and-registry`: Defines `lake.yaml` behavior, validation, and runtime expert registry preparation.

### Modified Capabilities

- None.

## Impact

- Affected areas: configuration loading, validation errors, expert metadata types, CLI/server startup preparation, test fixtures, and documentation examples.
- Downstream changes depend on this registry to select expert IDs, load model artifacts, serve configured model names, route requests, and package shareable experts.
- This change does not require OpenCode integration by itself, but it provides the server configuration consumed by OpenAI-compatible serving.
