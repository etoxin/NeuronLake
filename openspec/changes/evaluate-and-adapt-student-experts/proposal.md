## Why

After datasets exist, users need a way to evaluate whether small student experts are improving and identify weak domains before investing in adaptation work. Teacher-assisted evaluation and fine-tuning hooks complete the first practical teacher-student loop without making training mandatory for serving.

This change belongs to Milestone 2: Teacher-Student Model.

## What Changes

- Use the teacher model to grade, compare, or critique student expert outputs against domain tasks.
- Track per-expert evaluation results in a human-inspectable format.
- Identify weak domains, missing examples, or routing gaps that need more data.
- Add hooks for fine-tuning or adapter generation where supported by the selected backend/tooling.
- Support importing newly trained or adapted expert artifacts back into the lake.
- Rebuild or prompt rebuilding of NeuronGuard routing artifacts after expert changes.
- Keep normal serving independent from teacher loading and adaptation commands.
- Non-goals: guaranteed automated fine-tuning for every model format, teacher fallback during serving, frontier-model parity claims, and benchmark claims without measured evidence.

## Capabilities

### New Capabilities

- `teacher-student-evaluation-and-adaptation`: Defines teacher-assisted expert evaluation, result tracking, weak-domain reporting, adaptation hooks, trained artifact import, and post-adaptation router rebuild behavior.

### Modified Capabilities

- None.

## Impact

- Affected areas: evaluation commands, teacher runtime use, dataset consumption, result schemas, adaptation hook interfaces, artifact import flow, router rebuild coordination, and documentation.
- Depends on `teacher-dataset-generation` for generated tasks and on `expert-sharing` or registry behavior for importing updated expert artifacts.
- The OpenAI-compatible server remains usable without loading a teacher model.
