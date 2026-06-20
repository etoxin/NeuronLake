# OpenCode With NeuronLake

This folder is an OpenCode-focused workspace that points OpenCode at the local NeuronLake MVP server.

It uses a project-level `opencode.json` with a custom OpenAI-compatible provider:

- provider ID: `neuronlake`
- model: `neuronlake/neuronlake-mvp`
- base URL: `http://127.0.0.1:8080/v1`

OpenCode loads project config from `opencode.json` in the project root. The provider shape follows the current OpenCode custom/local provider docs: `npm: "@ai-sdk/openai-compatible"`, `options.baseURL`, and a `models` map.

## Prerequisites

Install OpenCode, `mise`, and llama.cpp:

```bash
brew install llama.cpp
```

The Homebrew llama.cpp package provides `llama-completion`, which this example uses for one-shot local generation.

## Run

Terminal 1:

```bash
cd example/opencode_neuronlake
mise trust
mise run download:models
mise run doctor
mise run serve
```

Terminal 2:

```bash
cd example/opencode_neuronlake
mise run api:models
mise run api:chat
mise run
```

Inside OpenCode, ask:

```text
Inspect sample/buggy-counter.ts and suggest the smallest immutable fix. Do not edit files.
```

You can also run a non-interactive OpenCode prompt:

```bash
mise run opencode:ask
```

## Files

- `opencode.json` configures OpenCode to use NeuronLake.
- `mise.toml` starts NeuronLake, checks prerequisites, and launches OpenCode.
- `request-opencode.json` is a direct API smoke test with an OpenCode-style system message.
- `sample/buggy-counter.ts` gives OpenCode a small local file to inspect.

## Notes

This example reuses the sibling `example/neuronlake_mvp/lake.yaml` and model files so the same downloaded experts power both the curl MVP and OpenCode.

The `mise` OpenCode tasks use isolated OpenCode state under `.opencode-state/`. This keeps the example reproducible and avoids failures from an existing global OpenCode SQLite state. The directory is ignored by Git.

OpenCode can omit `max_tokens`; this example sets `NEURONLAKE_DEFAULT_MAX_TOKENS=192` so llama.cpp does not fall back to its infinite generation default.

If `llama-completion` fails while initializing Metal, force CPU execution for the server:

```bash
LLAMA_CPP_ARGS="--device none --no-op-offload --no-kv-offload" mise run serve
```
