# NeuronLake Router/Proxy Direction

## Thesis

NeuronLake should be the local expert router and self-distillation layer for user-owned models, not a competing model runtime.

Users already have model runners they like: Ollama, llama.cpp, LM Studio, vLLM, or any OpenAI-compatible local server. NeuronLake should sit in front of those runtimes, expose a familiar API, and decide which configured expert should handle each request.

```text
Client -> NeuronLake API -> NeuronGuard router -> user-selected model backend
```

The core bet is that a lake of focused models can outperform one oversized general model for routeable, repeated, domain-specific work. The win comes from expert selection, concise domain behavior, user-owned configuration, and continuous routing improvement.

## Product Shape

NeuronLake should look like an Ollama/OpenAI-compatible API to clients.

Clients point at NeuronLake instead of directly at Ollama or llama.cpp:

```text
OpenCode / Continue / curl / app
  -> http://127.0.0.1:<neuronlake-port>
  -> NeuronLake routes request
  -> Ollama / llama.cpp / OpenAI-compatible backend
```

NeuronLake owns:

- expert registry
- routing metadata
- NeuronGuard router training
- teacher-distilled routing examples
- user corrections
- model/backend health checks
- model/package metadata
- observability for selected expert, confidence, and latency

NeuronLake should not own by default:

- model weights
- model downloads
- GPU scheduling
- low-level inference optimization
- full fine-tuning runtime

It can support local GGUF files as one backend, but that should be one option among several.

## Configuration Direction

The user configures the models they already trust for each domain.

```yaml
name: personal-lake

experts:
  - id: typescript
    domain: TypeScript debugging, refactors, code review, build errors
    routing_hints:
      - typescript
      - tsconfig
      - vite
      - react
      - type error
    backend:
      type: ollama
      url: http://127.0.0.1:11434
      model: qwen2.5-coder:7b

  - id: plants-trees
    domain: plant health, trees, gardening, botany, disease diagnosis
    routing_hints:
      - chlorosis
      - lemon tree
      - leaf yellowing
      - pruning
      - soil
    backend:
      type: ollama
      url: http://127.0.0.1:11434
      model: llama3.1:8b

  - id: sql
    domain: SQL query design, indexing, Postgres performance
    backend:
      type: openai-compatible
      base_url: http://127.0.0.1:8088/v1
      model: sql-expert

teacher:
  id: distiller
  backend:
    type: ollama
    url: http://127.0.0.1:11434
    model: gemma3:12b

server:
  host: 127.0.0.1
  port: 11435
  model_name: neuronlake:auto
```

NeuronLake still needs model metadata, but it should be metadata about user-owned models: backend type, model name, URL/path, context window, tags, tool support, health, and latency notes.

## Runtime Modes

### Auto Router

The client asks for one virtual model:

```text
neuronlake:auto
```

NeuronLake routes to the best expert.

### Explicit Passthrough

The client asks for a configured expert or model directly:

```text
neuronlake/typescript
neuronlake/plants-trees
```

NeuronLake forwards directly while still logging usage and optionally collecting correction data.

### Fallback

If confidence is low, NeuronLake can use a configured fallback expert:

```yaml
fallback:
  expert_id: general
  min_confidence: 0.35
```

The MVP should keep fallback simple and observable.

## Self-Training Loop

NeuronLake trains itself from the user's configuration and usage.

Inputs:

- expert domains
- routing hints
- user-provided examples
- teacher-distilled examples
- successful routing traces
- user corrections
- negative examples

Outputs:

- NeuronGuard router artifact
- router eval set
- confidence thresholds
- per-expert confusion matrix
- suggested missing examples

The teacher model is offline. It should not be required for normal serving.

## Teacher Distillation Role

The teacher can generate:

- expert-specific prompts
- expected expert behavior
- router labels: prompt -> expert ID
- ambiguous routing cases
- negative examples: prompt should not route to expert X
- tags and difficulty levels
- evaluation records

Generated data should be inspectable JSONL with provenance:

```json
{
  "kind": "router_example",
  "target_expert_id": "plants-trees",
  "negative_expert_ids": ["typescript"],
  "prompt": "Why are my lemon tree leaves yellow with green veins?",
  "tags": ["chlorosis", "citrus", "diagnosis"],
  "difficulty": "medium",
  "teacher": {
    "id": "distiller",
    "model": "gemma3:12b"
  },
  "quality_status": "generated",
  "provenance": {
    "source": "synthetic",
    "command": "neuronlake dataset generate"
  }
}
```

Router distillation should come before model distillation. It is cheaper, easier to evaluate, and directly tests the NeuronLake thesis.

## Backend Abstraction

NeuronLake should support multiple backend adapters:

- `ollama`: call local Ollama APIs
- `llama.cpp`: local subprocess now, server mode later
- `openai-compatible`: forward to any `/v1/chat/completions` endpoint
- `mock`: deterministic test backend

The internal backend contract should handle:

- health checks
- model listing where supported
- chat completion
- streaming
- context and output limits
- tool capability metadata
- error normalization

## Why This Is Strong

This direction makes NeuronLake easier to adopt:

- users keep their existing model runners
- users choose the models they trust
- clients only change their base URL
- NeuronLake focuses on routing and learning
- model count can scale from 2 to 1000 without changing client UX
- the lake can improve over time through corrections and teacher-generated examples

It also sharpens the product claim:

> NeuronLake is an Ollama-compatible expert router and self-distillation layer for user-owned local models.

## Near-Term Proof

The next strong proof should route between very different domains:

- TypeScript/code expert
- plants/trees expert
- SQL expert
- general fallback expert

Acceptance checks:

- `GET /api/tags` or `/v1/models` exposes NeuronLake virtual models
- prompt about TypeScript routes to `typescript`
- prompt about lemon tree chlorosis routes to `plants-trees`
- prompt about indexes routes to `sql`
- low-confidence prompt routes to fallback
- response includes debug metadata only when requested
- route eval reports top-1 accuracy, confidence, and confusion matrix

## Non-Goals For The Next Cut

- Do not build a full inference runtime.
- Do not require NeuronLake-managed model downloads.
- Do not require teacher model loading during serving.
- Do not claim frontier-model parity.
- Do not claim huge-model replacement without a measured routing benchmark.

## Open Questions

- Should the primary compatibility target be Ollama's API, OpenAI's API, or both?
- Should explicit expert selection use model names, tags, or `neuronlake/<expert-id>` aliases?
- How should NeuronLake represent tool-capable experts?
- What should happen on low-confidence routing: fallback, ask clarifying question, or multi-expert vote?
- How much routing metadata should be exposed through HTTP headers versus debug endpoints?
