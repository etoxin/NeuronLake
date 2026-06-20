## Why

NeuronLake's product value comes from routing prompts to small domain experts instead of asking one general model to handle every request. NeuronGuard must be trained from the configured expert set and used as the request-time expert selector.

This change belongs to Milestone 1: The Lake.

## What Changes

- Extract routing features from chat requests, including prompt text, code blocks, imports, file extensions, framework APIs, package names, and error-message terms where available.
- Train a NeuronGuard router from configured expert domains, routing hints, examples, and derived terms.
- Persist a local router artifact tied to the current expert set and rebuild it when relevant `lake.yaml` inputs change.
- Select one expert ID per request for the MVP.
- Return route confidence or scores where practical.
- Provide debug output explaining why an expert was selected.
- Non-goals: multi-expert voting, chained expert calls, teacher-backed fallback, memory pinning, and unmeasured routing performance claims.

## Capabilities

### New Capabilities

- `neuronguard-expert-routing`: Defines router training, feature extraction, artifact rebuild behavior, request-time expert selection, confidence output, and route debugging.

### Modified Capabilities

- None.

## Impact

- Affected areas: router training commands, feature extraction, NeuronGuard integration, server request flow, debug output, artifact invalidation, and tests for routing behavior.
- Depends on `lake-config-and-registry` for expert definitions and should integrate with `local-expert-backend` as the selector for which expert to run.
- OpenCode compatibility is affected because routed requests must still return OpenAI-compatible responses and SSE streams.
