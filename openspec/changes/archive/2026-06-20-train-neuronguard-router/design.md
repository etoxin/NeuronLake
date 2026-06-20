## Context

NeuronGuard is the fast routing engine for NeuronLake. The current repository already contains Rust and Python classification primitives; this change uses that role to select expert IDs from features extracted from incoming coding requests.

The router should be trained from `lake.yaml` expert metadata and rebuilt when the relevant config changes.

## Goals / Non-Goals

**Goals:**

- Extract routing features from chat messages and coding context.
- Train a NeuronGuard router over configured expert IDs.
- Persist a router artifact tied to the current expert set.
- Select one expert per request in the MVP.
- Expose route confidence or scores and debugging information where practical.

**Non-Goals:**

- Replacing expert model generation.
- Multi-expert voting, fallback chains, or teacher fallback.
- Accuracy claims without measured evaluation.
- Making routing depend on a teacher model.

## Decisions

1. Treat expert IDs as router labels.

   Each configured expert maps to a stable motor class in NeuronGuard. The registry should provide the expert ID ordering used during training and prediction.

   Alternative considered: route to domains first, then map domains to experts. That adds indirection before there is evidence it is needed.

2. Build training examples from several local signal types.

   Routing data should include expert domains, routing hints, examples, package names, imports, file extensions, framework APIs, code block tags, and error-message terms. These signals can be generated without teacher-student workflows.

   Alternative considered: require hand-written examples only. That makes the MVP fragile for small configs.

3. Persist router artifacts with a config fingerprint.

   The router artifact should record the expert set and relevant routing inputs used to train it. If `lake.yaml` changes those inputs, the system should rebuild or report that the router is stale.

   Alternative considered: retrain on every server start. That is simple but can hide invalidation bugs and slow startup unnecessarily.

4. Keep debug output explicit but opt-in.

   Routing explanations are useful during setup, but normal OpenCode sessions should not be polluted. Debug output should be available through logs, CLI inspection, or an explicit request/debug option.

   Alternative considered: always inject routing details into chat responses. That would break agent UX and OpenAI-compatible expectations.

## Risks / Trade-offs

- Sparse hints create weak routing -> Surface low confidence and debug terms so users can improve `lake.yaml`.
- Config fingerprint misses a dependency -> Keep the fingerprint scoped to explicit routing inputs first, then expand when new sources are added.
- Router output is overtrusted -> Backend/server should still return clear errors if the selected expert cannot run.
