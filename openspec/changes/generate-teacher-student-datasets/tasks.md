## 1. Teacher Configuration

- [ ] 1.1 Extend `lake.yaml` parsing with optional teacher configuration for dataset workflows.
- [ ] 1.2 Validate teacher ID and model reference only when teacher-student commands require them.
- [ ] 1.3 Add tests proving normal server startup does not require teacher configuration.

## 2. Training Sources

- [ ] 2.1 Add config structures for expert training docs, repos, prompts, and examples.
- [ ] 2.2 Validate configured training source paths for dataset generation commands.
- [ ] 2.3 Add tests for missing docs, missing repo, and valid prompt-only sources.

## 3. Dataset Generation

- [ ] 3.1 Define generated expert dataset record schema with expert ID, prompt/task, expected behavior, tags, difficulty, teacher identity, generation settings, quality status, and provenance.
- [ ] 3.2 Define generated router example record schema with target expert ID, optional negative candidate expert IDs, prompt/context, tags, difficulty, teacher identity, generation settings, quality status, and provenance.
- [ ] 3.3 Implement the teacher runtime boundary used by dataset generation.
- [ ] 3.4 Generate expert-specific examples from configured source material.
- [ ] 3.5 Generate router examples labeled with target expert IDs.
- [ ] 3.6 Generate boundary and negative router examples for ambiguous or near-domain prompts.

## 4. Outputs And Provenance

- [ ] 4.1 Write generated records in a human-inspectable structured text format such as JSONL.
- [ ] 4.2 Record provenance for source material, expert ID, command, teacher model identity, and generation settings.
- [ ] 4.3 Add tests for output file shape, metadata fields, quality status, and provenance fields.
- [ ] 4.4 Document that dataset generation is optional, offline, and outside normal serving.
