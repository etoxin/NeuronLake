# Product Requirement Document

## Project: NeuronLake

**Author:** etoxin  
**Status:** Draft / Conceptual MVP  
**Primary Test Harness:** OpenCode via an OpenAI-compatible local endpoint  
**Target Runtime:** Local developer workstations with consumer CPU/GPU hardware

---

## 1. Executive Summary

NeuronLake is a local expert-model runtime for terminal-first coding agents.

Instead of asking one large model to answer every coding request, NeuronLake lets a user build a local swarm of small, highly targeted coding experts. Each expert is a compact model, typically around 0.5B parameters, trained or distilled for a narrow domain such as JavaScript, TanStack Router, React, FastAPI, SQL, Tailwind, or a specific internal codebase.

The user defines the available experts in a YAML configuration file. NeuronLake then uses a larger teacher model, initially Gemma 12B, to help distill targeted behavior into the small expert models. Once the user has added and trained the experts, NeuronGuard is trained as the fast routing layer that decides which expert should receive each incoming coding request.

After setup, the user starts the NeuronLake server. NeuronLake exposes a local OpenAI-compatible endpoint that can be used by OpenCode without changes to OpenCode itself.

The goal is a practical local coding runtime:

- Small models for narrow, high-accuracy domains.
- User-owned expert configuration and training.
- Shareable expert models.
- NeuronGuard-powered routing across the expert swarm.
- OpenAI-compatible serving for existing developer agents.

---

## 2. Product Vision

NeuronLake should make local coding agents more useful on ordinary hardware by replacing a single general-purpose local model with a routed set of small specialists.

The intended user flow is:

1. The user creates a `swarm.yaml` file that lists the small expert models they want to use.
2. The user adds local or remote model references for each expert.
3. The user optionally provides domain material, examples, docs, repositories, or prompts for each expert.
4. NeuronLake uses a teacher model such as Gemma 12B to distill targeted training data or behavior into each small expert.
5. The user trains or imports the small expert models.
6. NeuronGuard is trained on the expert set so it can route incoming prompts to the best expert.
7. The user starts the NeuronLake server.
8. OpenCode connects to NeuronLake through a local OpenAI-compatible endpoint.

NeuronLake is not intended to be a monolithic foundation model. It is a local routing and serving layer for user-composed expert models.

---

## 3. Goals

### 3.1 MVP Goals

- Support a user-editable `swarm.yaml` file for defining expert models.
- Allow users to register small local models, initially targeting 0.5B-class GGUF models.
- Support domain-specific expert definitions such as JavaScript, TanStack Router, React, FastAPI, SQL, Tailwind, and project-specific experts.
- Provide a distillation workflow using a larger teacher model such as Gemma 12B.
- Train NeuronGuard as the router over the configured expert set.
- Expose an OpenAI-compatible `/v1/chat/completions` endpoint.
- Support Server-Sent Events streaming for agent compatibility.
- Validate integration using OpenCode as the primary test harness.
- Allow users to import, export, and share expert model definitions and trained artifacts.

### 3.2 Non-Goals For The First MVP

- NeuronLake will not initially train foundation models from scratch.
- NeuronLake will not initially guarantee all models are pinned in memory.
- NeuronLake will not initially target a 1TB expert library.
- NeuronLake will not initially require multi-turn memory or KV-cache persistence.
- NeuronLake will not initially claim frontier-model parity without benchmark evidence.

---

## 4. Core Concepts

### 4.1 Expert Model

An expert model is a small model specialized for a narrow coding domain.

Example experts:

- `javascript-core`
- `typescript-core`
- `react`
- `tanstack-router`
- `tailwind`
- `fastapi`
- `sql`
- `project-local`

Each expert should include:

- A stable expert ID.
- A model path or remote source.
- A target domain.
- Optional training material.
- Optional routing hints.
- Optional metadata for sharing and reuse.

### 4.2 Teacher Model

The teacher model is a larger model used during setup and training, not necessarily during normal serving.

The initial teacher target is Gemma 12B. Its job is to help generate, clean, or distill domain-specific examples that can improve the small experts.

The teacher model may be used for:

- Creating synthetic instruction examples.
- Converting documentation into Q&A or coding tasks.
- Producing high-quality target completions.
- Evaluating whether an expert response matches the intended domain behavior.
- Generating router training examples for NeuronGuard.

### 4.3 NeuronGuard Router

NeuronGuard is the fast routing layer.

After the user's experts are added and trained, NeuronLake trains NeuronGuard to map incoming requests to expert IDs. The router should use features such as:

- Library names.
- Imports.
- File extensions.
- Framework-specific APIs.
- Error messages.
- Code block language tags.
- Package names.
- User-provided routing hints.
- Examples generated during distillation.

The router does not replace the expert models. It selects the best expert for a request.

### 4.4 OpenAI-Compatible Server

NeuronLake exposes a local HTTP API compatible with OpenAI-style clients.

The MVP endpoint is:

```text
POST /v1/chat/completions
```

This allows OpenCode to use NeuronLake as if it were an OpenAI-compatible provider.

---

## 5. User Workflow

### 5.1 Configure Experts

The user creates a `swarm.yaml` file in their workspace or NeuronLake config directory.

Example:

```yaml
name: frontend-swarm
teacher:
  id: gemma-12b
  model: ./models/gemma-12b.gguf

experts:
  - id: javascript-core
    domain: JavaScript language and runtime behavior
    model: ./models/javascript-core-0.5b.gguf
    train:
      docs:
        - ./training/javascript/
      prompts:
        - Explain JavaScript async behavior with small runnable examples.
    routing_hints:
      - javascript
      - node
      - async
      - promise

  - id: tanstack-router
    domain: TanStack Router application code
    model: ./models/tanstack-router-0.5b.gguf
    train:
      docs:
        - ./training/tanstack-router/
      repos:
        - ./examples/tanstack-router-app/
    routing_hints:
      - tanstack router
      - createFileRoute
      - routeTree
      - loader

server:
  host: 127.0.0.1
  port: 8080
  model_name: library-lake-v1
```

### 5.2 Distill Experts

The user runs a NeuronLake training or preparation command.

The distillation workflow should:

- Read `swarm.yaml`.
- Validate that configured models and training sources exist.
- Use the teacher model to generate or refine domain-specific training examples.
- Save generated examples in a human-inspectable format.
- Train or fine-tune the small expert models where supported.
- Produce metadata that explains what each expert is intended to handle.

### 5.3 Train NeuronGuard Routing

After expert preparation, NeuronLake trains NeuronGuard to route requests across the configured expert set.

The routing training set can come from:

- User-provided examples.
- Expert routing hints.
- Documentation terms.
- Package names and APIs.
- Teacher-generated examples.
- Evaluation prompts used during distillation.

The output is a local router artifact tied to the current expert set.

### 5.4 Start The Server

The user starts NeuronLake:

```bash
neuronlake serve --config swarm.yaml
```

The server:

- Loads `swarm.yaml`.
- Loads the NeuronGuard router artifact.
- Loads or prepares the configured expert models.
- Starts the OpenAI-compatible HTTP server.
- Streams responses back to the client.

### 5.5 Test With OpenCode

OpenCode is the first target integration and test harness.

Example OpenCode config:

```json
{
  "provider": {
    "neuronlake-swarm": {
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "http://127.0.0.1:8080/v1",
        "apiKey": "neuronlake-handshake"
      },
      "models": {
        "library-lake-v1": {
          "name": "NeuronLake Swarm"
        }
      }
    }
  },
  "model": "neuronlake-swarm/library-lake-v1"
}
```

---

## 6. Functional Requirements

### 6.1 YAML Configuration

NeuronLake must read a user-editable YAML file that defines:

- Swarm name.
- Teacher model.
- Expert models.
- Expert domains.
- Model paths or remote sources.
- Optional training sources.
- Optional routing hints.
- Server configuration.

The YAML file should remain simple enough for users to edit by hand.

### 6.2 Expert Model Registry

NeuronLake must maintain a registry of configured experts at runtime.

The registry should support:

- Local model paths.
- Downloaded model cache paths.
- Imported shared experts.
- Expert metadata.
- Version information.
- Training status.

### 6.3 Distillation Pipeline

NeuronLake must support a workflow where a teacher model helps prepare small experts.

The initial implementation may support one or more of:

- Dataset generation.
- Instruction-response generation.
- Documentation summarization into examples.
- Router example generation.
- Expert evaluation prompts.

The MVP can begin with generated datasets and routing examples before full fine-tuning automation is implemented.

### 6.4 Expert Training Or Import

NeuronLake must allow users to either:

- Train a small expert locally.
- Import an already trained expert.
- Reference an existing local GGUF expert.
- Use a shared expert artifact from another user.

The system should not assume all users will train every expert themselves.

### 6.5 NeuronGuard Router Training

NeuronLake must train NeuronGuard against the configured expert list.

The trained router should:

- Predict the best expert for a request.
- Return confidence or score information where practical.
- Support debugging output explaining why an expert was selected.
- Be rebuildable when `swarm.yaml` changes.

### 6.6 OpenAI-Compatible Chat Endpoint

NeuronLake must expose:

```text
POST /v1/chat/completions
```

The endpoint should accept standard chat messages and return OpenAI-compatible responses.

The MVP must support:

- `model`
- `messages`
- `stream`
- Basic generation parameters where supported by the backend.

### 6.7 SSE Streaming

When `stream: true` is provided, NeuronLake must stream OpenAI-style Server-Sent Events.

This is required for agent UX and OpenCode compatibility.

### 6.8 Model Sharing

NeuronLake should allow users to share expert models and metadata.

A shareable expert package should include:

- Expert ID.
- Domain description.
- Model artifact reference.
- Routing hints.
- Training metadata.
- Compatibility information.

The first version can use simple local folders or archives before any registry or marketplace exists.

---

## 7. Runtime Request Flow

```text
OpenCode
  |
  | POST /v1/chat/completions
  v
NeuronLake server
  |
  | extract prompt/code/context features
  v
NeuronGuard router
  |
  | select expert ID
  v
Expert model backend
  |
  | generate response
  v
OpenAI-compatible response / SSE stream
```

For the MVP, one expert should handle each request. Later versions may support multi-expert voting, fallback, or chained expert calls.

---

## 8. Architecture

### 8.1 Components

```text
NeuronLake
├── config
│   └── swarm.yaml parser and validator
├── registry
│   └── expert model metadata and artifact tracking
├── distillation
│   └── teacher-model assisted dataset and routing example generation
├── router
│   └── NeuronGuard expert selection
├── backend
│   └── GGUF / llama.cpp-compatible model execution
├── server
│   └── OpenAI-compatible HTTP and SSE API
└── sharing
    └── import/export format for trained experts
```

### 8.2 Existing NeuronGuard Role

The existing NeuronGuard codebase provides the starting point for:

- Cache-aligned routing structures.
- Fast token/feature classification.
- Trainable associative routing.
- Python and Rust integration patterns.

NeuronLake should preserve NeuronGuard as the routing engine while adding the model-serving and expert-management layers around it.

---

## 9. MVP Implementation Phases

### Phase 1: Configuration And Server Skeleton

- Add `swarm.yaml` schema.
- Parse and validate expert definitions.
- Add OpenAI-compatible HTTP server.
- Add basic `/v1/chat/completions` response shape.
- Add OpenCode configuration example.

### Phase 2: Local Expert Execution

- Integrate a GGUF-compatible inference backend.
- Load one configured expert.
- Generate non-streaming responses.
- Add streaming SSE support.

### Phase 3: NeuronGuard Routing

- Extract prompt features for routing.
- Train NeuronGuard from routing hints and examples.
- Select an expert per request.
- Add route debugging output.

### Phase 4: Distillation Workflow

- Add Gemma 12B teacher configuration.
- Generate expert-specific training examples.
- Generate router training examples.
- Store generated datasets for inspection.
- Add hooks for fine-tuning or importing trained experts.

### Phase 5: Expert Sharing

- Define an expert package format.
- Add import/export commands.
- Track version and compatibility metadata.
- Allow users to reuse community or team experts.

---

## 10. Performance Targets

Initial performance targets should be measured, not assumed.

MVP targets:

- Router selection should be fast enough to be invisible compared with model generation.
- Server overhead should be low enough for interactive coding use.
- Streaming should begin as soon as the selected expert starts producing tokens.
- The runtime should support at least one local 0.5B-class expert model.
- The runtime should be designed to grow toward several resident experts as backend support allows.

Future targets:

- Multiple hot-loaded experts.
- Sub-millisecond NeuronGuard routing on normal prompts.
- Concurrent expert availability without disk thrashing.
- Optional fallback to a larger local model when routing confidence is low.

---

## 11. Open Questions

- Which GGUF inference backend should be used first: direct llama.cpp bindings, a Rust crate, or a subprocess wrapper?
- What is the first supported format for training or fine-tuning 0.5B experts?
- Should NeuronLake generate datasets only, or automate the full fine-tuning loop in the MVP?
- How should shared expert packages reference model weights that may be too large for normal source control?
- Should low-confidence routing fall back to the teacher model, a default expert, or an error response?
- How much of the router explanation should be exposed to users during normal OpenCode sessions?

---

## 12. Success Criteria

The MVP is successful when:

- A user can define at least two small experts in `swarm.yaml`.
- NeuronLake can validate the config and prepare the expert registry.
- NeuronGuard can be trained to route between those experts.
- NeuronLake can start a local OpenAI-compatible server.
- OpenCode can send chat requests to NeuronLake.
- NeuronLake can route a request to the selected expert.
- NeuronLake can stream an OpenAI-compatible response back to OpenCode.
- Expert configuration and trained artifacts can be exported or reused by another user.

