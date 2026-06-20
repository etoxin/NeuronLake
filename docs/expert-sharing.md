# Expert Sharing

NeuronLake expert packages move expert metadata and model artifact references between local lakes. They are local files only; this format does not provide signing, trust, hosting, marketplace discovery, or automated training.

## Package Layout

A folder package contains an `expert.yaml` manifest at the package root:

```text
my-expert/
  expert.yaml
  artifacts/
    my-expert.gguf
```

A manifest-only archive is a single YAML file, typically named with an `.expert.yaml` suffix. Manifest-only archives cannot embed model weights; they reference an external artifact path or remote URI.

## Manifest

The manifest uses `manifest_version: 1` and identifies the expert plus the model artifact needed at runtime:

```yaml
manifest_version: 1
expert_id: sql-shared
domain: SQL query design and optimization
model:
  embedded: artifacts/sql-shared.gguf
routing_hints:
  - sql
  - postgres
examples:
  - Explain why this PostgreSQL query is not using an index.
version: 1.0.0
compatibility:
  backend: llama.cpp
  model_format: gguf
training_status:
  state: trained
  dataset: internal-sql-support
```

`expert_id`, `domain`, and one model artifact reference are required. Routing hints, examples, sharing metadata, version, compatibility metadata, and training status are preserved when exporting and importing packages.

## Model Artifacts

Use `model.embedded` for files stored inside a folder package. The path is resolved relative to the package root and must exist before import succeeds.

Use `model.external_path` for large local weights managed outside the package:

```yaml
model:
  external_path: /models/rust-shared.gguf
```

External paths are preserved and checked during import. If the file is not available, import records a warning instead of failing so users can restore the weight file later.

Use `model.remote` for URL-style references:

```yaml
model:
  remote: hf://example/rust-shared
  cache_path: ./cache/rust-shared.gguf
```

Remote references are metadata only in this milestone. Downloading or hosting model weights is outside the expert package format.

## Import Behavior

Import validates the manifest before adding an expert to the runtime registry. Duplicate expert IDs are rejected by default. Callers can explicitly overwrite the existing expert or rename the imported expert.

A successfully imported expert is exposed through the same registry API as configured experts. Routing receives the imported expert ID, domain, hints, and examples. Serving receives the imported expert model reference; local embedded and external GGUF paths can be prepared by the local backend, while remote references still require a backend that supports them.
