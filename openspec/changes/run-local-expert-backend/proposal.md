## Why

The Lake must become useful with imported or externally trained local expert models before teacher-student workflows exist. A bounded local inference backend lets NeuronLake load a configured expert and produce real responses behind the OpenAI-compatible server.

This change belongs to Milestone 1: The Lake.

## What Changes

- Define the backend boundary for GGUF or llama.cpp-compatible local model execution.
- Load one configured expert model from the expert registry.
- Generate non-streaming responses from the selected expert.
- Stream generated tokens to the server SSE layer as soon as they are available.
- Map supported generation parameters from chat requests into backend options.
- Return clear runtime errors for missing model files, unsupported backends, or failed generation.
- Keep backend selection measurable and explicit instead of assuming performance.
- Non-goals: automated fine-tuning, resident loading of every expert, multi-expert voting, fallback orchestration, and benchmark claims.

## Capabilities

### New Capabilities

- `local-expert-backend`: Defines local expert model loading, generation, streaming token handoff, backend errors, and the GGUF-compatible inference boundary.

### Modified Capabilities

- None.

## Impact

- Affected areas: backend abstraction, model artifact resolution, server generation path, streaming token pipeline, runtime errors, dependency selection, and backend tests.
- Depends on `lake-config-and-registry` for model references and on `openai-compatible-chat-server` for request handling and response streaming.
- OpenCode compatibility is affected indirectly because server responses should become real model outputs without changing OpenCode configuration.
