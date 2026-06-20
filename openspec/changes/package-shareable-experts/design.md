## Context

NeuronLake users should be able to reuse experts trained or curated elsewhere. The first sharing format should work locally without a marketplace, account system, or remote registry.

Sharing is metadata and artifact movement. It should integrate with the registry but not require serving, routing, or training to be active.

## Goals / Non-Goals

**Goals:**

- Define a local expert package format.
- Export metadata and artifact references for a configured expert.
- Import and validate packages before adding them to a lake.
- Preserve routing hints and compatibility metadata for downstream router training.
- Support large model artifact references without committing weights to source control.

**Non-Goals:**

- Public marketplace.
- Trust, signing, or remote reputation system.
- Hosting model weights.
- Automated training or benchmark certification.

## Decisions

1. Use a folder/archive package with a manifest.

   A package should contain a manifest file plus optional metadata and local artifacts. The manifest should include expert ID, domain, model reference, routing hints, examples, training metadata, version, and compatibility fields.

   Alternative considered: export raw `lake.yaml` fragments only. That is easy but cannot carry provenance, compatibility, or artifact layout cleanly.

2. Keep model weights optional in the package.

   The package may include a local model file or reference an external path or URL. This avoids forcing large weights into source control or local archives.

   Alternative considered: require packages to be self-contained. That is convenient but impractical for large GGUF artifacts.

3. Import into the registry through validation, not direct file edits.

   Import should validate manifest shape, compatibility metadata, duplicate IDs, and artifact availability before updating or suggesting changes to the lake config.

   Alternative considered: copy files and append YAML directly. That is brittle and hard to roll back.

4. Preserve package metadata even when runtime ignores it.

   Training metadata and compatibility fields may not affect generation immediately, but keeping them allows evaluation and adaptation workflows to reason about imported experts later.

   Alternative considered: discard unknown metadata. That makes sharing less useful across versions.

## Risks / Trade-offs

- Package schema becomes too broad -> Keep required fields small and put optional fields under clearly named metadata sections.
- Artifact references rot -> Validate at import time and store warnings when a referenced artifact is unavailable.
- Duplicate expert IDs create conflicts -> Require explicit rename or overwrite behavior during import.
