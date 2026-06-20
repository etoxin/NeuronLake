## Why

The MVP implementation now includes real local GGUF generation, routed expert selection, `/v1/models`, OpenCode-specific provider behavior, output token clamping, and reproducible examples that are only partially described by the archived specs. This Milestone 1 change aligns the source-of-truth specs with the implemented MVP contract so future changes build on the actual runtime shape instead of stale placeholder requirements.

## What Changes

- Update the OpenAI-compatible chat server spec to cover `/v1/models`, provider-prefixed model aliases such as `neuronlake/neuronlake-mvp`, blocking backend offload, and the constrained OpenCode MVP example.
- Update the local expert backend spec to cover configurable fallback/upper-bound `max_tokens` behavior and the current subprocess-per-request performance boundary.
- Update the routing spec to cover routed backend selection in normal chat serving and the requirement that routing metadata/debug output stays out of assistant content unless explicitly requested.
- Update the lake config and registry spec to clarify that the MVP example can be built from imported/manual GGUF experts and model metadata without teacher configuration.
- Update the expert sharing spec to clarify that the MVP example preserves sharing and compatibility metadata for imported Hugging Face GGUF artifacts without embedding large model files.
- Keep this change as spec alignment only; no runtime code edits are required unless validation reveals an implemented behavior is not covered by tests.

Non-goals:

- No teacher-student distillation, fine-tuning, or model training workflow.
- No resident model manager, hot expert cache, memory pinning, or eviction policy.
- No benchmark claims beyond measured diagnostics and smoke-test timings.
- No full OpenCode tool-calling support; the current OpenCode example remains a no-tool review smoke test for a 0.5B local model.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `openai-compatible-chat-server`: Document implemented OpenCode compatibility details, model listing, provider-prefixed model aliases, output behavior, and bounded no-tool OpenCode example expectations.
- `local-expert-backend`: Document configurable default/max token clamping and the subprocess-per-request backend boundary.
- `neuronguard-expert-routing`: Document routed chat serving behavior and the separation between route selection/debug data and assistant output.
- `lake-config-and-registry`: Clarify MVP imported/manual GGUF expert metadata and server model naming behavior.
- `expert-sharing`: Clarify preservation of external model artifact metadata for shareable/imported expert definitions used by the MVP examples.

## Impact

- Affected specs: `openspec/specs/openai-compatible-chat-server/spec.md`, `openspec/specs/local-expert-backend/spec.md`, `openspec/specs/neuronguard-expert-routing/spec.md`, `openspec/specs/lake-config-and-registry/spec.md`, and `openspec/specs/expert-sharing/spec.md`.
- Affected implementation areas for verification only: `src/chat_server.rs`, `src/local_backend.rs`, `src/expert_router.rs`, `src/lake_config.rs`, `examples/neuronlake_mvp.rs`, `example/neuronlake_mvp/`, and `example/opencode_neuronlake/`.
- External compatibility: OpenCode remains the primary client compatibility target through an OpenAI-compatible local `/v1` API.
