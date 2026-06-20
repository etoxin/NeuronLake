## 1. Spec Alignment Audit

- [x] 1.1 Confirm `src/chat_server.rs` satisfies the `/v1/models`, provider-prefixed model alias, blocking-generation offload, non-streaming, and SSE requirements.
- [x] 1.2 Confirm `src/local_backend.rs` satisfies local GGUF preparation, missing artifact errors, supported generation options, subprocess diagnostics, and configurable output-token clamping.
- [x] 1.3 Confirm `src/expert_router.rs` and routed chat serving select one expert before backend generation and do not inject route debug text into assistant responses.
- [x] 1.4 Confirm `src/lake_config.rs` preserves imported/manual GGUF expert metadata, server model identity, sharing metadata, compatibility metadata, and training status without teacher configuration.
- [x] 1.5 Confirm `example/neuronlake_mvp/` and `example/opencode_neuronlake/` document the MVP boundary, external model downloads, output limits, and constrained no-tool OpenCode smoke test.

## 2. Close Any Runtime Gaps

- [x] 2.1 Add or update focused Rust tests only if the audit finds an implemented behavior is missing coverage for a new spec requirement.
- [x] 2.2 Add or update example documentation only if the audit finds a user-facing MVP behavior is undocumented or contradicted.
- [x] 2.3 Avoid adding teacher-student training, model distillation, resident model caching, or tool-calling behavior as part of this alignment change.

## 3. Verification

- [x] 3.1 Run `cargo test --no-default-features` and confirm all Rust tests pass.
- [x] 3.2 Run the NeuronLake MVP example doctor task or equivalent checks for model-path, llama.cpp, and example build prerequisites.
- [x] 3.3 With the MVP server running, verify `GET /v1/models` returns `neuronlake-mvp`.
- [x] 3.4 With the MVP server running, verify a non-streaming chat request succeeds through `/v1/chat/completions`.
- [x] 3.5 With the MVP server running, verify the OpenCode example non-interactive smoke task exits and returns the bounded review-only answer.

## 4. OpenSpec Finalization

- [x] 4.1 Run `openspec status --change align-mvp-implementation-with-specs` and confirm the change is apply-ready.
- [x] 4.2 After verification succeeds, confirm the change is ready to archive so the canonical specs can include the MVP-alignment requirements.
