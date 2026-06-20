Here is a comprehensive, production-ready Product Requirement Document (PRD) for **NeuronLake**. It bridges the matrix-free, cache-aligned Rust architecture of NeuronGuard with standard local developer tools, creating a plug-and-play, stateless edge-computing runtime.

---

# Product Requirement Document (PRD)

## Project: NeuronLake

**Author:** etoxin

**Status:** Draft / Conceptual MVP

**Target Architecture:** 16GB RAM Unified Memory Footprint (e.g., Apple Silicon M-Series, Consumer X86 Windows/Linux Edge Workstations)

---

## 1. Executive Summary & Objective

Modern terminal-first AI coding agents (such as OpenCode) are restricted by a harsh local hardware paradox: running massive frontier models locally consumes prohibitive VRAM/RAM resources and causes severe execution latency, while running small models ($0.5\text{B} - 1\text{B}$) results in severe structural hallucination, API syntax cross-contamination, and memory constraints.

**NeuronLake** resolves this by introducing a matrix-free, local **Neuromorphic Routing Swarm**. By combining a lightning-fast, 64-byte cache-aligned Spiking Neural Network (SNN) engine (**NeuronGuard**) with a fluid reservoir of hyper-specialized, library-specific $0.5\text{B}$ GGUF models, NeuronLake enables consumer hardware to achieve domain-specific execution accuracy that rivals monolithic frontier models, operating entirely within a strict $2\text{ GB} - 3\text{ GB}$ memory footprint.

The system exposes a standard local OpenAI-compatible endpoint, allowing external developer agents like **OpenCode** to tap into an extensive, multi-language coding ecosystem with zero modification to their source logic.

---

## 2. System Architecture & High-Level Topology

NeuronLake completely removes the monolithic LLM from the active routing path. It splits memory into two clean, concurrent layers operating on a single consumer machine:

```
                  ┌─────────────────────────────────────┐
                  │       Client Request: OpenCode      │
                  └──────────────────┬──────────────────┘
                                     │ POST /v1/chat/completions
                                     ▼
                  ┌─────────────────────────────────────┐
                  │       NeuronLake HTTP Server        │
                  └──────────────────┬──────────────────┘
                                     │
                        [ Sub-Millisecond SNN Match ]
                                     ▼
                  ┌─────────────────────────────────────┐
                  │      NeuronGuard SNN Router         │
                  │  (Cache-Aligned, Matrix-Free Core)   │
                  └──────────────────┬──────────────────┘
                                     │
           ┌───────────┬─────────────┼─────────────┬───────────┐
           ▼           ▼             ▼             ▼           ▼
       [Model 1]   [Model 2]     [Model 3]     [Model 4]   [Model 5] ... [Models 6 & 7]
       Core HTML   Tailwind      Async JS      FastAPI     SQL/DB        (Docs & Memory)
       (0.5B GGUF) (0.5B GGUF)   (0.5B GGUF)   (0.5B GGUF) (0.5B GGUF)    (0.5B GGUF)
       └─────────────────────────────┬───────────────────────────────┘
                                     │ Concurrent RAM Pinned Buffer (~1.75 GB)
                                     ▼
                        [ 120+ tokens/sec Streaming ]
                                     │
                                     ▼
                  ┌─────────────────────────────────────┐
                  │     Standard OpenAI SSE Output      │
                  └─────────────────────────────────────┘

```

---

## 3. Core Functional Requirements

### 3.1 Framework Architecture & Local Swarm Instantiation

* **Requirement:** The platform must read a flat, user-editable deployment file (`swarm.yaml`) to construct the runtime topology.
* **Specification:** On startup, the engine must parse the layout configuration, match the registered model references, allocate a fixed, unified memory pool, and verify that the target GGUF files are present in the local cache block.

### 3.2 GGUF Integration & Byte-Aligned Metadata Parsing

* **Requirement:** The system must natively ingest standard, unmodified community GGUF formats (e.g., `Q4_K_M` layouts).
* **Specification:** The Rust engine must read the GGUF header file structural layout directly to extract the embedded `tokenizer.ggml.tokens` sheets.
* **Neuromorphic Alignment:** The extracted tokens must be read directly to configure the SNN router’s **Field 0 (Lexical Focus)** topological grid arrays, establishing targeted synaptic path weights for language-specific tokens with zero local training required.

### 3.3 Sub-Millisecond Neuromorphic Stream Routing

* **Requirement:** Stream routing must execute via NeuronGuard's matrix-free spatiotemporal ensemble system.
* **Specification:**
* **Field 0 (Lexical Focus):** Evaluates exact library keywords, macros, and namespaces.
* **Field 1 (Structural Context):** Audits code geometry, syntax indentation profiles, and block scopes at line-rate.
* **Winner-Take-All (WTA) Pool:** The moment an incoming token block crosses a specific accumulator threshold, the router must instantly pass execution handles to that specific expert. Total routing computation latency must remain under **1 millisecond**.



### 3.4 OpenCode Target Integration via OpenAI Reverse Proxy

* **Requirement:** Expose an OpenAI-compliant HTTP endpoint (`/v1/chat/completions`) supporting Server-Sent Events (SSE) streaming.
* **Specification:** The backend must interface natively with any local config layout. A developer must be able to drops a standard `opencode.json` configuration block into a local workspace root, pointing OpenCode to NeuronLake seamlessly:

```json
{
  "provider": {
    "neuronlake-swarm": {
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "http://127.0.0.1:8080/v1",
        "apiKey": "neuronlake-handshake"
      },
      "models": {
        "library-lake-v1": { "name": "NeuronLake Swarm" }
      }
    }
  },
  "model": "neuronlake-swarm/library-lake-v1"
}

```

### 3.5 Stateless Context Execution with Background Memory Offloading

* **Requirement:** The first pass execution architecture will explicitly handle queries statelessly to eliminate runtime KV-cache syncing overhead.
* **Specification:** Every chat payload is treated as a clean, self-contained prompt.
* **The Background Memory Agent:** To preserve cross-session insights, the runtime must route transaction footprints asynchronously to a dedicated **0.5B Memory Expert**. This background specialist captures key configurations, language preferences, and API schemas, writing them straight to a local, human-readable markdown file, ensuring the active coding specialists remain unburdened by history.

---

## 4. Hardware & Performance Targets

Target metrics are mapped explicitly to a base **16GB MacBook Pro** running under a strict 7-model configuration pool:

* **Active Memory Budget:** 7 models $\times$ $\sim 250\text{ MB}$ per INT4 model = **~1.75 GB total active footprint**. The runtime must pin this entire model block concurrently in RAM/VRAM to eliminate disk I/O thrashing.
* **Switching Latency:** Moving execution pointers between hot resident models in the unified memory buffer must execute in **$\sim 0$ milliseconds**.
* **Generation Throughput:** Token generation utilizing embedded `llama.cpp` bindings must match or exceed **120 tokens/second** on standard M-Series Apple Silicon graphics pipelines.

---

## 5. Developer Lifecycle & Workflow

The entire user interaction path is optimized for zero friction, stripping away complex deep learning toolchains:

```
[ Step 1: Clone Repository ]
            │
            ▼
[ Step 2: Define Experts in swarm.yaml ]
            │
            ▼
[ Step 3: Run 'python start_lake.py' ] ───> Downloads GGUF files & mounts SNN weights
            │
            ▼
[ Step 4: Drop config into opencode.json ]
            │
            ▼
[ Step 5: Execute 'opencode' in Terminal ] ───> Instant, stateless local execution loop

```

---

## 6. Future Expansion Roadmap

* **Phase 2 (Dynamic Paging Scale):** Transition from a 7-model memory-pinned cache to a full **1-Terabyte NVMe library swarm** (4,000+ experts), utilizing NeuronGuard's Field 2 (Sequential Delta) array to run predictive, asynchronous background pre-fetching threads in under 20ms over PCIe Gen 4/5 data lines.
* **Phase 3 (Flash State Resuming):** Implement flat, pointerless serialization structures to save and swap small internal KV-cache binary blocks directly, enabling ultra-fast multi-turn continuity for active experts without re-parsing text strings.