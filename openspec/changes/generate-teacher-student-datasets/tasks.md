## 1. Teacher Configuration

- [ ] 1.1 Extend `lake.yaml` parsing with optional teacher configuration for dataset workflows.
- [ ] 1.2 Validate teacher ID and model reference only when teacher-student commands require them.
- [ ] 1.3 Add tests proving normal server startup does not require teacher configuration.

## 2. Training Sources

- [ ] 2.1 Add config structures for expert training docs, repos, prompts, and examples.
- [ ] 2.2 Validate configured training source paths for dataset generation commands.
- [ ] 2.3 Add tests for missing docs, missing repo, and valid prompt-only sources.

## 3. Dataset Generation

- [ ] 3.1 Define generated expert dataset record schema.
- [ ] 3.2 Define generated router example record schema.
- [ ] 3.3 Implement the teacher runtime boundary used by dataset generation.
- [ ] 3.4 Generate expert-specific examples from configured source material.
- [ ] 3.5 Generate router examples labeled with target expert IDs.

## 4. Outputs And Provenance

- [ ] 4.1 Write generated records in a human-inspectable structured text format.
- [ ] 4.2 Record provenance for source material, expert ID, command, and teacher model identity.
- [ ] 4.3 Add tests for output file shape and provenance fields.
- [ ] 4.4 Document that dataset generation is optional and outside normal serving.
