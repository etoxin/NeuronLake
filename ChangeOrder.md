  1. openspec/changes/define-lake-config-and-registry/proposal.md
  2. openspec/changes/serve-openai-compatible-chat/proposal.md
  3. openspec/changes/run-local-expert-backend/proposal.md
  4. openspec/changes/train-neuronguard-router/proposal.md
  5. openspec/changes/package-shareable-experts/proposal.md
  6. openspec/changes/generate-teacher-student-datasets/proposal.md
  7. openspec/changes/evaluate-and-adapt-student-experts/proposal.md


  ```
  Recommended Changes

   Change ID           Milestone    Scope
  ━━━━━━━━━━━━━━━━━━  ━━━━━━━━━━━  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   define-lake-               M1    lake.yaml schema, parsing, validation, server config, expert metadata, local paths, imported/shared expert references, registry preparation.
   config-and-
   registry
  ──────────────────  ───────────  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   serve-openai-              M1    HTTP server, POST /v1/chat/completions, OpenAI-style request/response shape, model naming, basic OpenCode compatibility, SSE contract. Can start with a
   compatible-chat                  deterministic/mock backend.
  ──────────────────  ───────────  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   run-local-                 M1    GGUF / llama.cpp-compatible execution boundary, load one configured expert, generation parameters, non-streaming and streaming token handoff. Backend
   expert-backend                   choice belongs here.
  ──────────────────  ───────────  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   train-                     M1    Feature extraction, router training from hints/examples/imports/extensions/APIs, route confidence/debug output, rebuild when lake.yaml changes, per-
   neuronguard-                     request expert selection.
   router
  ──────────────────  ───────────  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   package-                   M1    Expert package format, import/export commands, compatibility/version metadata, training metadata, local folder/archive sharing.
   shareable-
   experts
  ──────────────────  ───────────  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   generate-                  M2    Optional teacher config, teacher loading outside serving path, generated expert datasets, router examples, provenance, human-inspectable output.
   teacher-student-
   datasets
  ──────────────────  ───────────  ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   evaluate-and-              M2    Teacher grading/comparison, per-expert eval results, weak-domain reporting, fine-tuning/adaptation hooks, importing newly trained artifacts, router
   adapt-student-                   rebuild after expert changes.
   experts

  I’d keep SSE in the API change as a contract, then make the backend change prove real streamed tokens can flow through it. That avoids making the first server change depend on a working
  GGUF backend.

  I have not created OpenSpec artifact files yet because this was an explore pass. Next practical step is generating proposal/design/spec/tasks for these change IDs.
  ```