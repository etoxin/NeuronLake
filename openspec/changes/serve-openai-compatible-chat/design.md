## Context

OpenCode should be able to call NeuronLake as an OpenAI-compatible provider. This change introduces the HTTP boundary before real model execution is mandatory, allowing client compatibility and streaming behavior to be tested early.

The server depends on `lake.yaml` for host, port, and exposed model name. It must preserve the client-facing API as backend and routing layers mature.

## Goals / Non-Goals

**Goals:**

- Serve `POST /v1/chat/completions` locally.
- Accept OpenAI-style chat requests with `model`, `messages`, `stream`, and basic generation parameters.
- Return compatible non-streaming JSON and SSE streaming events.
- Provide a minimal OpenCode configuration example.
- Support a deterministic placeholder generator until the local backend is implemented.

**Non-Goals:**

- Full OpenAI API coverage.
- Production authentication or multi-tenant security.
- Real model execution.
- Automated fine-tuning or benchmark claims.

## Decisions

1. Implement a thin OpenAI-compatible adapter over an internal generation interface.

   Request validation, response shaping, and SSE framing should live in the server layer. Actual generation should be delegated to an internal interface so placeholder, backend, and routed execution can use the same API contract.

   Alternative considered: implement server responses directly in backend code. That would force backend changes to understand OpenAI response details.

2. Use the configured lake model name as the exposed model.

   The client should request `server.model_name` from `lake.yaml`, not individual expert IDs. NeuronLake owns routing behind that public model name.

   Alternative considered: expose every expert as a separate OpenAI model. That can be useful later, but it weakens the routed-lake abstraction for the MVP.

3. Use OpenAI-style SSE framing for `stream: true`.

   The server should emit chat completion chunks as `data: ...` events and terminate with `data: [DONE]`. This keeps OpenCode compatibility central to the API contract.

   Alternative considered: stream raw text. Raw text is easier but is not OpenAI-compatible.

4. Prefer a Rust HTTP stack for the MVP server.

   The repository already has Rust core code. A Rust server using a mainstream async HTTP stack such as Axum/Tokio keeps the runtime local and avoids a second long-running language boundary for serving.

   Alternative considered: build the server in Python first. Python would be quick, but the long-term NeuronLake runtime is likely to need Rust ownership of config, routing, and backend process control.

## Risks / Trade-offs

- OpenAI compatibility drift -> Add focused request/response and SSE fixtures that mirror OpenCode traffic.
- Placeholder backend hides integration gaps -> Keep placeholder behavior explicit and replace it behind the generation interface in the backend change.
- Async dependencies increase crate surface -> Keep server code isolated from NeuronGuard internals.
