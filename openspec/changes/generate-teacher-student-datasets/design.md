## Context

Teacher-student workflows are Milestone 2 and must remain optional. The first teacher capability should distill training data and router examples from configured sources using a larger teacher model, not require full fine-tuning automation.

Generated artifacts should be inspectable and reusable by routing, evaluation, or external training tools. The immediate product value is improving expert selection and creating training material that can later feed expert adaptation.

## Goals / Non-Goals

**Goals:**

- Add optional teacher configuration to `lake.yaml`.
- Validate teacher model and training sources for dataset commands.
- Generate expert-specific instruction, coding, or domain examples.
- Generate NeuronGuard router training examples with target expert labels.
- Generate boundary and negative examples for ambiguous or near-domain prompts.
- Store datasets and provenance in a human-inspectable format.
- Record teacher identity, generation settings, tags, difficulty, and quality status.
- Keep teacher loading outside normal serving.

**Non-Goals:**

- Requiring a teacher for Milestone 1 serving.
- Automating the full fine-tuning loop.
- Using teacher fallback during OpenCode requests.
- Trusting teacher-generated examples without inspection or later evaluation.

## Decisions

1. Gate teacher behavior behind explicit commands.

   The server should not load the teacher model during normal startup. Dataset generation should run through a separate command or workflow that reads the same `lake.yaml`.

   Alternative considered: load the teacher in the server for opportunistic fallback. That violates the product constraint that The Lake serves without a teacher.

2. Store generated examples as inspectable records.

   Use a line-oriented or simple structured format such as JSONL or YAML records for generated examples. Each record should identify the expert, source, prompt/task, generated response or label, tags, difficulty, teacher model, generation settings, provenance, and quality status.

   Alternative considered: write opaque trainer-specific binary datasets. That blocks user inspection and makes iteration harder.

3. Separate expert datasets from router examples.

   Expert training examples and router classification examples serve different consumers. They should share provenance conventions but be written to distinct outputs.

   Alternative considered: store all generated examples in one mixed file. That complicates downstream validation and training.

4. Generate hard routing examples deliberately.

   Router training must include obvious in-domain prompts, near-domain prompts, ambiguous prompts, and negative examples that should route elsewhere. This is more valuable for NeuronLake than only generating polished expert answers.

   Alternative considered: ask the teacher only for expert-specific Q&A. That improves answer style but does not prove the core routed-lake thesis.

5. Treat Gemma 12B as an initial target, not a hard-coded dependency.

   The config should describe a teacher ID and model reference. Gemma 12B can be documented as the initial target, but the schema should not make it the only possible teacher.

   Alternative considered: hard-code Gemma 12B paths and prompts. That would make local setups brittle.

## Risks / Trade-offs

- Generated data quality varies -> Preserve provenance and later feed evaluation before adaptation.
- Synthetic routing labels can encode teacher bias -> Include inspectable records, negative examples, and quality status so labels can be reviewed before training.
- Teacher runtime is expensive -> Run as an explicit offline workflow and avoid loading it in serving.
- Source material is large -> Start with bounded source discovery and clear limits before optimization.
