## Context

Dataset generation creates candidate material, but users still need to know whether student experts are improving. This change adds teacher-assisted evaluation, weak-domain reporting, and adaptation hooks while keeping serving independent from teacher loading.

The implementation should accept that fine-tuning support may vary by model format and backend.

## Goals / Non-Goals

**Goals:**

- Run teacher-assisted grading or comparison for student expert outputs.
- Store per-expert evaluation results in inspectable files.
- Identify weak domains, missing examples, and routing gaps.
- Define hooks for fine-tuning or adapter generation where supported.
- Import newly trained artifacts back into the lake and coordinate router rebuilds.

**Non-Goals:**

- Guaranteeing fine-tuning support for every GGUF or model format.
- Loading the teacher during normal OpenAI-compatible serving.
- Serving teacher fallback responses.
- Claiming benchmark or frontier parity without measured evidence.

## Decisions

1. Make evaluation a separate offline workflow.

   Evaluation should read datasets, run student experts, ask the teacher to grade or compare outputs, and write results. It should not run inside the normal chat endpoint.

   Alternative considered: evaluate every live response. That is too expensive and changes agent latency.

2. Store evaluation results with enough context to audit them.

   Results should include expert ID, task ID, student output reference or excerpt, teacher judgment, score if present, failure category, and provenance. This lets users inspect whether the teacher is judging the right behavior.

   Alternative considered: store only aggregate scores. Aggregates hide failure modes and are not enough to improve experts.

3. Define adaptation as hooks, not a universal trainer.

   The workflow should be able to call supported external or internal fine-tuning/adaptation commands, then import produced artifacts. The hook contract should be explicit about inputs, outputs, and unsupported formats.

   Alternative considered: build a complete fine-tuning system in the first teacher-student pass. That is too broad and would block useful evaluation.

4. Rebuild routing after expert artifact changes.

   When an adapted expert is imported or its metadata changes, NeuronGuard artifacts may become stale. The workflow should rebuild the router or clearly report that a rebuild is required.

   Alternative considered: leave routing unchanged after adaptation. That can send prompts to experts with outdated hints or metadata.

## Risks / Trade-offs

- Teacher grades are inconsistent -> Store examples and rationales where available, and compare trends instead of treating one score as absolute truth.
- Adaptation hooks are too generic -> Start with a small command contract and expand only when a backend needs it.
- Evaluation is slow on local hardware -> Make it batchable, resumable where practical, and separate from serving.
