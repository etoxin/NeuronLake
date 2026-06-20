## Why

The Lake should support experts that are imported, manually trained, or shared by another user. A practical package format allows expert definitions and metadata to move between projects without requiring an external marketplace.

This change belongs to Milestone 1: The Lake.

## What Changes

- Define a shareable expert package format using local folders or archives.
- Include expert ID, domain description, model artifact reference, routing hints, training metadata, version information, and compatibility metadata.
- Add import behavior that validates packages before adding them to a lake.
- Add export behavior that writes a package from a configured expert and available metadata.
- Support references to large model weights without requiring them to live in source control.
- Preserve imported experts as first-class registry entries.
- Non-goals: public marketplace, remote trust system, automated model hosting, fine-tuning automation, and benchmark claims.

## Capabilities

### New Capabilities

- `expert-sharing`: Defines expert package metadata, import validation, export behavior, artifact references, and registry integration for shared experts.

### Modified Capabilities

- None.

## Impact

- Affected areas: expert metadata schema, import/export CLI behavior, package validation, registry integration, compatibility checks, documentation, and fixtures.
- Depends on `lake-config-and-registry` for registry shape and validation rules.
- Routing and serving changes can consume imported experts like any other configured expert.
