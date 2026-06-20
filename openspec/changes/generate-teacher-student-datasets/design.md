## Context

Teacher-student workflows are Milestone 2 and must remain optional. The first teacher capability should generate datasets and router examples from configured sources, not require full fine-tuning automation.

Generated artifacts should be inspectable and reusable by routing, evaluation, or external training tools.

## Goals / Non-Goals

**Goals:**

- Add optional teacher configuration to `lake.yaml`.
- Validate teacher model and training sources for dataset commands.
- Generate expert-specific instruction or coding examples.
- Generate NeuronGuard router training examples.
- Store datasets and provenance in a human-inspectable format.
- Keep teacher loading outside normal serving.

**Non-Goals:**

- Requiring a teacher for Milestone 1 serving.
- Automating the full fine-tuning loop.
- Using teacher fallback during OpenCode requests.
- Claiming dataset quality without evaluation.

## Decisions

1. Gate teacher behavior behind explicit commands.

   The server should not load the teacher model during normal startup. Dataset generation should run through a separate command or workflow that reads the same `lake.yaml`.

   Alternative considered: load the teacher in the server for opportunistic fallback. That violates the product constraint that The Lake serves without a teacher.

2. Store generated examples as inspectable records.

   Use a line-oriented or simple structured format such as JSONL or YAML records for generated examples. Each record should identify the expert, source, prompt/task, generated response or label, and provenance.

   Alternative considered: write opaque trainer-specific binary datasets. That blocks user inspection and makes iteration harder.

3. Separate expert datasets from router examples.

   Expert training examples and router classification examples serve different consumers. They should share provenance conventions but be written to distinct outputs.

   Alternative considered: store all generated examples in one mixed file. That complicates downstream validation and training.

4. Treat Gemma 12B as an initial target, not a hard-coded dependency.

   The config should describe a teacher ID and model reference. Gemma 12B can be documented as the initial target, but the schema should not make it the only possible teacher.

   Alternative considered: hard-code Gemma 12B paths and prompts. That would make local setups brittle.

## Risks / Trade-offs

- Generated data quality varies -> Preserve provenance and later feed evaluation before adaptation.
- Teacher runtime is expensive -> Run as an explicit offline workflow and avoid loading it in serving.
- Source material is large -> Start with bounded source discovery and clear limits before optimization.
