## Why

Milestone 2 needs a teacher-assisted preparation workflow that can improve small experts and produce router examples, while keeping normal serving independent from the teacher model. Dataset generation is the first useful teacher-student capability before full fine-tuning automation.

This change belongs to Milestone 2: Teacher-Student Model.

## What Changes

- Add optional teacher configuration to `lake.yaml`, initially targeting Gemma 12B or another local teacher model.
- Validate teacher model paths and training sources without making teacher configuration required for serving.
- Generate expert-specific datasets from docs, repos, prompts, examples, or configured training material.
- Generate router training examples for NeuronGuard.
- Store generated datasets in a human-inspectable format.
- Preserve provenance that links generated examples back to their configured source material.
- Keep teacher model loading outside the normal OpenAI-compatible serving path.
- Non-goals: mandatory teacher-backed serving, full fine-tuning automation, frontier-model parity claims, and benchmark claims.

## Capabilities

### New Capabilities

- `teacher-dataset-generation`: Defines optional teacher configuration, source validation, expert dataset generation, router example generation, provenance, and inspectable output formats.

### Modified Capabilities

- None.

## Impact

- Affected areas: optional config validation, teacher runtime boundary, training-source discovery, generated artifact layout, dataset schemas, provenance metadata, and dataset generation commands.
- Depends on `lake-config-and-registry` for expert and optional teacher configuration.
- Feeds later router training and expert evaluation, but The Lake server remains runnable without this change.
