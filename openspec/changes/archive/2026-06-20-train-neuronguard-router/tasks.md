## 1. Routing Data

- [x] 1.1 Build router labels from the validated expert registry.
- [x] 1.2 Generate training examples from expert domains, routing hints, and examples.
- [x] 1.3 Extract derived signals for package names, imports, file extensions, framework APIs, code block tags, and error-message terms.
- [x] 1.4 Add tests for feature extraction from representative coding prompts.

## 2. NeuronGuard Integration

- [x] 2.1 Implement router training over configured expert IDs using NeuronGuard.
- [x] 2.2 Map NeuronGuard prediction labels back to stable expert IDs.
- [x] 2.3 Add tests that a two-expert lake routes obvious prompts to expected expert IDs.

## 3. Router Artifacts

- [x] 3.1 Persist trained router artifacts with expert-set and routing-input metadata.
- [x] 3.2 Compute a fingerprint from routing-relevant `lake.yaml` fields.
- [x] 3.3 Detect stale router artifacts when expert IDs, domains, hints, or examples change.
- [x] 3.4 Add tests for artifact freshness and stale detection.

## 4. Request-Time Routing

- [x] 4.1 Route each valid chat completion request to one selected expert ID before generation.
- [x] 4.2 Expose route confidence or scores where available.
- [x] 4.3 Implement opt-in route debugging with contributing routing signals.
- [x] 4.4 Verify normal OpenCode-compatible responses do not include routing debug text.
