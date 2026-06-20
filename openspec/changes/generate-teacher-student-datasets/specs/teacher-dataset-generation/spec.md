## ADDED Requirements

### Requirement: Configure optional teacher model
The system SHALL support optional teacher model configuration in `lake.yaml` for teacher-student dataset workflows.

#### Scenario: Teacher config is present for dataset command
- **WHEN** `lake.yaml` includes a teacher ID and model reference and the user runs a dataset generation command
- **THEN** the system validates the teacher configuration for that command

### Requirement: Keep serving independent from teacher config
The system SHALL keep normal OpenAI-compatible serving usable without loading or requiring a teacher model.

#### Scenario: Server starts without teacher
- **WHEN** `lake.yaml` contains experts and server settings but no teacher section
- **THEN** the Lake server can start without teacher validation or teacher loading

### Requirement: Validate training sources
The system SHALL validate configured training sources for dataset generation, including docs, repositories, prompts, and examples where present.

#### Scenario: Missing docs source is reported
- **WHEN** an expert training source references a missing docs directory
- **THEN** dataset generation fails with an error that identifies the missing source

### Requirement: Generate expert-specific datasets
The system SHALL use the configured teacher model to generate expert-specific training examples from configured source material.

#### Scenario: Expert dataset is generated
- **WHEN** an expert has configured training sources and the teacher runtime is available
- **THEN** the system writes generated examples associated with that expert ID, including prompt/task text and expected expert behavior

### Requirement: Generate router training examples
The system SHALL use teacher-assisted workflows to generate routing examples that map prompts or code context to configured expert IDs.

#### Scenario: Router examples include expert labels
- **WHEN** router example generation completes
- **THEN** each generated router example identifies the target expert ID

#### Scenario: Router examples include boundary cases
- **WHEN** router example generation is requested for multiple configured experts
- **THEN** the generated router dataset includes in-domain, ambiguous, and near-domain examples for distinguishing those experts

#### Scenario: Router examples include negative labels
- **WHEN** a generated prompt should not route to a candidate expert
- **THEN** the router training record identifies the correct target expert or records the negative candidate relationship in an inspectable field

### Requirement: Store inspectable generated artifacts
The system SHALL store generated datasets and router examples in a human-inspectable format.

#### Scenario: User inspects generated records
- **WHEN** dataset generation completes
- **THEN** the generated output can be opened as structured text records without a custom binary reader

#### Scenario: JSONL output is inspectable
- **WHEN** dataset generation writes JSONL output
- **THEN** each line is a complete generated record that can be reviewed independently

### Requirement: Preserve provenance
The system SHALL record provenance for generated examples, including source material, expert ID, generation command, and teacher model identity where available.

#### Scenario: Generated example records source
- **WHEN** an example is generated from configured documentation
- **THEN** the output record includes provenance linking it to the source documentation and expert ID

### Requirement: Record teacher-generation metadata
The system SHALL record teacher identity, generation settings, tags, difficulty, and quality status for generated examples.

#### Scenario: Generated record includes teacher metadata
- **WHEN** the teacher model generates an expert or router dataset record
- **THEN** the output record includes teacher model identity, generation settings where available, tags, difficulty, and a quality status field suitable for later review
