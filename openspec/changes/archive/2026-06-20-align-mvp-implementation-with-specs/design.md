## Context

The Milestone 1 MVP has moved from placeholder-only serving to a real local runtime path: `lake.yaml` is loaded into an expert registry, NeuronGuard trains an in-memory router, chat requests are routed to one expert, and the llama.cpp subprocess backend generates OpenAI-compatible chat responses. The examples also now include a curl-focused MVP and a constrained OpenCode smoke test that uses a no-tool agent because the bundled 0.5B model is not capable of driving OpenCode's full tool-using agent loop.

The archived specs describe the broad architecture but miss several concrete behaviors added during implementation and debugging. This change treats the current code and examples as the Milestone 1 baseline and updates specs to match that baseline.

## Goals / Non-Goals

**Goals:**

- Align OpenSpec requirements with implemented MVP behaviors for serving, routing, backend execution, model naming, and examples.
- Keep Milestone 1 independent from teacher-student training and distillation.
- Preserve OpenCode compatibility as a concrete integration target through a local OpenAI-compatible `/v1` API.
- Make performance language precise: subprocess timing can be measured, but no resident-cache or latency guarantee is claimed.

**Non-Goals:**

- No new runtime feature work unless verification finds a mismatch between tests and specs.
- No tool-calling protocol support for OpenCode in this change.
- No model distillation, teacher dataset generation, fine-tuning, or base-model training.
- No resident model manager, hot/cold expert cache, memory pinning, eviction policy, or benchmark target.

## Decisions

1. Treat this as a spec-alignment change, not an implementation milestone.

   Rationale: the current MVP already demonstrates the important runtime path. The immediate risk is drift between specs and code, not missing runtime code. Alternative considered: roll this into a Milestone 2 architecture proposal. That would blur the boundary between the current MVP and future model-cache/base-set work.

2. Add narrow delta requirements rather than rewriting entire archived specs.

   Rationale: existing specs still describe the core capabilities correctly. The missing pieces are specific acceptance criteria such as `/v1/models`, provider-prefixed model IDs, token clamping, and the constrained OpenCode example. Alternative considered: replace the archived specs wholesale. That would create more churn than value and make archive review harder.

3. Keep OpenCode support honest and bounded.

   Rationale: the current 0.5B local model can prove API compatibility but cannot reliably operate as a full OpenCode coding agent with tool schemas and long system prompts. The spec should require a reproducible no-tool smoke test, not imply production agent quality. Alternative considered: require full tool-calling compatibility now. That belongs in a future base-set/tool-caller milestone.

4. Specify the llama.cpp subprocess backend as the MVP backend boundary.

   Rationale: the current backend starts a local command per request, prepares local GGUF paths, maps supported generation options, and records measured subprocess diagnostics. This is simple and testable for Milestone 1. Alternative considered: specify a resident server/pool now. That is a later performance architecture and would invalidate the current MVP unnecessarily.

## Risks / Trade-offs

- Small-model output quality can look weak in OpenCode -> Mitigation: document the OpenCode example as a constrained integration smoke test and keep quality claims tied to the selected expert.
- Specs may overfit to the current subprocess backend -> Mitigation: describe it as the current MVP backend boundary, not the final inference architecture.
- Provider-prefixed model aliases may be interpreted too broadly -> Mitigation: require only aliases whose final path segment matches the configured model name.
- Token clamping can surprise clients that request larger outputs -> Mitigation: document the server/backend cap in examples and keep it configurable through environment-driven backend setup.
- Future Milestone 2 work may need stronger base experts and tool-calling -> Mitigation: leave tool-calling and distillation explicitly out of this change so they can be proposed cleanly.
