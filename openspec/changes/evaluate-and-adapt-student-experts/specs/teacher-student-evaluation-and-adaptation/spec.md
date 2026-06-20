## ADDED Requirements

### Requirement: Run teacher-assisted student evaluation
The system SHALL use a configured teacher model to grade, compare, or critique student expert outputs for evaluation tasks.

#### Scenario: Teacher grades student output
- **WHEN** an evaluation task has a student expert output and a teacher model is available
- **THEN** the system records the teacher's judgment for that output

### Requirement: Store inspectable evaluation results
The system SHALL store per-expert evaluation results in a human-inspectable format.

#### Scenario: Evaluation result includes context
- **WHEN** evaluation completes for an expert
- **THEN** the result records include expert ID, task identity, student output reference or excerpt, teacher judgment, score when available, failure category when available, and provenance

### Requirement: Report weak domains
The system SHALL identify weak domains, missing examples, or routing gaps based on evaluation results.

#### Scenario: Weak domain report is produced
- **WHEN** evaluation results show repeated failures for a domain or source category
- **THEN** the system reports that domain or category as needing more examples or adaptation work

### Requirement: Provide adaptation hook contract
The system SHALL define a hook contract for supported fine-tuning or adapter generation workflows.

#### Scenario: Supported adaptation hook runs
- **WHEN** an expert, dataset, and supported adaptation hook are configured
- **THEN** the system invokes the hook with documented inputs and records the produced artifact reference

#### Scenario: Unsupported adaptation format is reported
- **WHEN** the user requests adaptation for a model or backend format without a supported hook
- **THEN** the system returns an actionable unsupported-format error

### Requirement: Import adapted expert artifacts
The system SHALL support importing newly trained or adapted expert artifacts back into the lake registry.

#### Scenario: Adapted artifact becomes an expert
- **WHEN** an adaptation hook produces a model artifact with valid metadata
- **THEN** the system imports or updates the corresponding expert registry entry

### Requirement: Coordinate router rebuild after expert changes
The system SHALL rebuild the NeuronGuard router or report that a rebuild is required after imported or adapted expert metadata changes routing-relevant fields.

#### Scenario: Adapted expert changes routing hints
- **WHEN** an adapted expert import changes routing hints or examples
- **THEN** the system rebuilds the router artifact or marks the existing router artifact stale

### Requirement: Keep evaluation outside normal serving
The system SHALL keep teacher-assisted evaluation and adaptation outside the normal OpenAI-compatible serving path.

#### Scenario: Server does not load teacher for evaluation
- **WHEN** the Lake server starts for normal chat completion serving
- **THEN** it does not load the teacher model or run evaluation workflows
