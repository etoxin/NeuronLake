## 1. Backend Boundary

- [ ] 1.1 Define the internal expert backend interface for non-streaming and streaming generation.
- [ ] 1.2 Define backend request, generation option, token delta, completion, and error types.
- [ ] 1.3 Wire the server generation interface to call the backend by selected expert ID.

## 2. GGUF-Compatible Adapter

- [ ] 2.1 Choose the first llama.cpp-compatible integration path behind the backend interface.
- [ ] 2.2 Implement local model artifact preparation for one configured expert.
- [ ] 2.3 Return clear errors for missing, unreadable, or unsupported model artifacts.
- [ ] 2.4 Add fixtures or test doubles for backend behavior without requiring large model files.

## 3. Generation

- [ ] 3.1 Implement non-streaming generation from a prepared local expert.
- [ ] 3.2 Map supported chat generation parameters into backend options.
- [ ] 3.3 Handle unsupported generation parameters according to documented behavior.
- [ ] 3.4 Add tests for successful generation and backend failure paths.

## 4. Streaming

- [ ] 4.1 Implement streaming token or text-delta events from the backend.
- [ ] 4.2 Normalize backend deltas before they reach the SSE layer.
- [ ] 4.3 Add tests that streamed backend deltas become ordered server chunks.
- [ ] 4.4 Add a simple measurement or diagnostic path for startup and generation timing without presenting targets as guarantees.
