## 1. Evaluation Inputs

- [ ] 1.1 Define evaluation task inputs from generated datasets or user-provided evaluation records.
- [ ] 1.2 Define per-expert evaluation result schema with task identity, output reference, teacher judgment, score, failure category, and provenance.
- [ ] 1.3 Add fixtures for passing, failing, and ambiguous student outputs.

## 2. Teacher-Assisted Evaluation

- [ ] 2.1 Implement the offline evaluation command or workflow.
- [ ] 2.2 Run selected student experts on evaluation tasks.
- [ ] 2.3 Ask the configured teacher model to grade, compare, or critique student outputs.
- [ ] 2.4 Write inspectable per-expert evaluation results.
- [ ] 2.5 Add tests using a teacher test double for deterministic judgments.

## 3. Weak-Domain Reporting

- [ ] 3.1 Aggregate evaluation results by expert, domain, source category, and failure category.
- [ ] 3.2 Report weak domains, missing examples, and routing gaps.
- [ ] 3.3 Add tests for weak-domain detection from repeated failures.

## 4. Adaptation Hooks

- [ ] 4.1 Define the adaptation hook input and output contract.
- [ ] 4.2 Implement unsupported-format errors for models without configured hooks.
- [ ] 4.3 Invoke supported hooks with documented dataset and expert metadata inputs.
- [ ] 4.4 Record produced adapted artifact references.

## 5. Artifact Import And Router Coordination

- [ ] 5.1 Import adapted artifacts back into the expert registry with metadata.
- [ ] 5.2 Detect routing-relevant metadata changes after import.
- [ ] 5.3 Rebuild the NeuronGuard router or mark it stale after adapted expert changes.
- [ ] 5.4 Verify normal OpenAI-compatible serving does not load the teacher or run evaluation.
