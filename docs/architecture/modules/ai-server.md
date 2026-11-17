# Native AI server boundary

`zipeek-ai-core` declares provider-neutral generation requests, streamed events, backend errors and cooperative cancellation. `zipeek-ai-server/src/lib.rs` now implements an injectable Axum transport. The HTTP routes are `/health`, `/v1/models` and `/v1/chat/completions`. The transport validates model identity, supported roles, text-only response format and generation bounds before invoking `GenerationBackend`. It supports JSON and OpenAI-style SSE responses, rejects busy requests with HTTP 429 and limits output size.

The HTTP layer has an origin allowlist for browser calls. By default it permits `http://localhost:1420` and `http://127.0.0.1:1420`. Additional trusted origins must be provided explicitly through `AiServer::with_allowed_origins`; there is no wildcard browser access. The server does not store browser learning records or user input. AI requests necessarily send their supplied prompts to the configured backend.

This is not an operational llama.cpp service. `zipeek-ai-server/src/main.rs` remains a placeholder; copied `llama_cpp` modules are inactive because their `model_runtime` dependency is absent. Model loading, GGUF configuration, queueing and actual cancellation of native inference remain future implementation work. The core backend has to be connected before any model is advertised at runtime. The existing in-process Rust E2E AI fixture remains the deterministic browser test provider.

The transport has authored HTTP regression tests in `zipeek-ai-server/src/http_tests.rs`. Rust compilation, a real model, browser interoperability, Windows startup and CORS behavior must be verified in the target environment. Source inspection alone does not establish these checks.
