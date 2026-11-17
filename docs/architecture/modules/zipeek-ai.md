# AI prompts and inference boundaries

The former `crates/zipeek-ai/` prompt owner has been extracted to `crates/zipeek-ai-protocol/`. Prompt generation, structured sentence analysis, classification and exercise-output validation remain domain contracts, not HTTP or model-runtime implementation. `zipeek-web` and `zipeek-db-gen` depend on this protocol crate.

`zipeek-ai-client` owns OpenAI-compatible HTTP transport for native and browser clients. It currently works with a configured provider endpoint, including LM Studio. `zipeek-ai-core` now defines transport-neutral chat messages, a generation request with bounded validated options, generation events, shared cancellation, errors, and an asynchronous `GenerationBackend` streaming callback contract. The core interface does not yet replace the client's established public wire types.

`zipeek-ai-server` is a separate native crate. Its copied `llama_cpp` source references a missing `model_runtime` module; that copied code is intentionally not exposed by the crate entry points and cannot yet be considered an operational inference server. Connecting and testing the backend requires implementing runtime ownership, cancellation, model loading and OpenAI-compatible HTTP endpoints, without weakening deterministic test fixtures.

The deterministic E2E AI fixture remains in `zipeek-e2e/src/harness/ai_fixture.rs`. It is independent of both LM Studio and the unfinished native AI server and must remain the default for reproducible E2E assertions.
