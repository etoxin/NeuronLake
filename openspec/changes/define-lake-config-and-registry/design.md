## Context

The current repository is centered on NeuronGuard classification primitives and Python bindings. NeuronLake needs a new configuration and registry layer around that core before server, backend, routing, sharing, or teacher-student workflows can be implemented safely.

`lake.yaml` is the user-owned source of truth. Runtime code should treat it as input to validate and normalize, not as mutable application state.

## Goals / Non-Goals

**Goals:**

- Define a small, hand-editable `lake.yaml` schema for Milestone 1.
- Normalize expert definitions into a typed runtime registry.
- Validate configuration with actionable errors before server or router startup.
- Keep teacher-student fields optional and outside the Milestone 1 serving requirement.

**Non-Goals:**

- Running model inference.
- Training or fine-tuning experts.
- Pinning all experts in memory.
- Proving performance or accuracy claims.

## Decisions

1. Separate parsed config from runtime registry.

   The config layer should mirror `lake.yaml`; the registry should contain normalized IDs, resolved paths, source metadata, and derived runtime state. This keeps user-facing YAML stable while allowing runtime-specific metadata to evolve.

   Alternative considered: use the parsed YAML object directly everywhere. That couples serving, routing, and sharing to file format details and makes validation harder to centralize.

2. Resolve relative paths from the config file directory.

   A model path like `./models/react.gguf` should mean relative to the `lake.yaml` location, not the process working directory. The registry should retain the original text for display and the resolved path for runtime checks.

   Alternative considered: resolve everything from the current working directory. That breaks when a user starts the server from a different directory.

3. Validate the full config before building the registry.

   Validation should collect multiple errors where practical: duplicate expert IDs, missing models, invalid host or port, empty domains, invalid model names, and malformed remote/import references.

   Alternative considered: fail on the first parse error. That is simpler but creates a poor hand-editing workflow.

4. Keep teacher fields optional and inert in Milestone 1.

   `lake.yaml` may later contain a `teacher` section, but normal serving must not require it. Milestone 1 code should ignore or reject unknown teacher-student fields consistently until the teacher dataset change defines them.

   Alternative considered: include teacher config in the first schema. That increases scope before the lake can serve imported experts.

## Risks / Trade-offs

- Schema grows too quickly -> Keep Milestone 1 fields limited to lake, experts, routing hints, examples, server settings, and sharing metadata.
- Validation blocks useful partial configs -> Separate strict server startup validation from lighter inspection commands if implementation needs both.
- Remote references are underspecified -> Store them as typed metadata first, and defer download/cache behavior to later implementation.
