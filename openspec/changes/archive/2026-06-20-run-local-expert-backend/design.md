## Context

The Lake must serve real local expert models without depending on teacher-student training. This change adds a local inference backend boundary for configured expert artifacts, initially targeting GGUF or llama.cpp-compatible models.

The backend is not the router. It receives a selected expert ID and chat prompt, resolves the expert artifact from the registry, and produces generated text or token events.

## Goals / Non-Goals

**Goals:**

- Define an expert backend interface for non-streaming and streaming generation.
- Load at least one configured local expert model.
- Map supported chat generation parameters into backend options.
- Return actionable runtime errors for missing models or failed generation.
- Keep streaming token handoff compatible with the OpenAI-compatible server.

**Non-Goals:**

- Training models.
- Loading all experts at startup.
- Multi-expert voting or fallback.
- Guaranteeing sub-second startup or benchmark results before measurement.

## Decisions

1. Put GGUF execution behind an `ExpertBackend` boundary.

   The server and router should call a stable internal interface such as `generate(expert_id, request)` and `stream(expert_id, request)`. The concrete GGUF implementation can change without rewriting OpenAI request handling.

   Alternative considered: call a llama.cpp crate or process directly from the server layer. That would make routing and streaming harder to test independently.

2. Start with one loaded expert path, then expand.

   The MVP should prove one configured local expert can generate a response. Multi-expert residency and cache eviction can be added after measurements show the memory and latency profile.

   Alternative considered: load every configured expert at startup. That conflicts with the non-goal of guaranteeing pinned memory and may fail on consumer hardware.

3. Choose a llama.cpp-compatible backend adapter as the first GGUF boundary.

   The implementation should isolate the exact integration choice, such as subprocess wrapper, local server, or Rust binding, behind the backend trait. A subprocess adapter is acceptable for the first pass if it reduces dependency risk; direct bindings can follow if benchmarks justify them.

   Alternative considered: define a custom GGUF runner. That is outside product scope and would delay the lake runtime.

4. Treat generation parameters as best-effort.

   Parameters like temperature, top_p, max tokens, and stop sequences should be accepted by the API layer and passed through only when supported by the backend. Unsupported parameters should be ignored with documented behavior or rejected when they change semantics materially.

   Alternative considered: reject all parameters not supported by the first backend. That would reduce OpenAI client compatibility.

## Risks / Trade-offs

- Backend choice proves too slow or fragile -> Keep the adapter replaceable and add measurement tasks before broad optimization.
- Streaming behavior differs by backend -> Normalize backend token events before they reach the SSE layer.
- Model files are large or missing -> Validate paths before generation and return clear errors instead of panics.
