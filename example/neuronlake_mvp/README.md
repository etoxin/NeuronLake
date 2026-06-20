# NeuronLake MVP

This example is the first end-to-end NeuronLake MVP check:

- `lake.yaml` defines two local experts.
- `mise.toml` downloads real GGUF model files from Hugging Face.
- The Cargo example trains the NeuronGuard router from `lake.yaml`.
- The server exposes `POST /v1/chat/completions` on `127.0.0.1:8080`.
- Requests are routed to one selected expert before `llama-cli` generates the response.

The example uses the official Qwen GGUF repositories:

- Coding expert: `Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF`, file `qwen2.5-coder-0.5b-instruct-q4_k_m.gguf`
- General expert: `Qwen/Qwen2.5-0.5B-Instruct-GGUF`, file `qwen2.5-0.5b-instruct-q4_k_m.gguf`

The two Q4_K_M files are roughly 1 GB combined.

Downloads use the current Hugging Face `hf download` CLI through `uv`, with `HF_XET_HIGH_PERFORMANCE=1` enabled in `mise.toml`.

## Prerequisites

Install `mise` and `llama.cpp` with `llama-cli` available on PATH. On macOS:

```bash
brew install llama.cpp
```

If your binary is not named `llama-cli`, set `LLAMA_CPP_BIN`:

```bash
export LLAMA_CPP_BIN=/path/to/llama-cli
```

## Run

From this directory:

```bash
mise trust
mise run download:models
mise run doctor
mise run serve
```

In another terminal:

```bash
mise run chat:code
mise run chat:general
```

You can also call the endpoint directly:

```bash
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d @request-code.json
```

## OpenAI-Compatible Shape

Use this provider shape from OpenCode or another OpenAI-compatible client:

```json
{
  "provider": {
    "neuronlake": {
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "http://127.0.0.1:8080/v1",
        "apiKey": "neuronlake-local"
      },
      "models": {
        "neuronlake-mvp": {
          "name": "NeuronLake MVP"
        }
      }
    }
  },
  "model": "neuronlake/neuronlake-mvp"
}
```

## What This Tests

This is intentionally small but real:

- `lake.yaml` validation and model path resolution
- expert metadata and routing hints
- NeuronGuard router training
- routed backend selection per chat request
- local GGUF model preparation
- subprocess generation through `llama-cli`
- OpenAI-compatible JSON responses

Generation starts a fresh `llama-cli` process per request. That is slow but useful for the MVP because it keeps the server simple and exercises the local expert path without a resident model manager.
