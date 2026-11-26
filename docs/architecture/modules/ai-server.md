# Local AI server boundary

The native `zipeek-ai-server` executable now starts a real `llama-cpp-2` GGUF inference backend, not the fixture provider. It requires an explicitly selected local model. The executable accepts `--bind 127.0.0.1:1430` and rejects non-loopback binds. The service is managed by the existing Tauri Services manager as the `ai` sidecar.

## Model selection

Set `ZIPEEK_AI_MODEL_PATH` to the GGUF file before starting Tauri, or install exactly one `.gguf` file under `models/ai` relative to the process working directory or executable directory. Multiple GGUF candidates require an explicit environment variable to avoid nondeterministic model selection. Startup fails if the file is missing. `ZIPEEK_AI_GPU_LAYERS` selects GPU offload (default 0), and `ZIPEEK_AI_CONTEXT` selects the context window (default 4096, accepted 512–131072).

The server advertises model ID `local-gguf` and retains one loaded model for the process lifetime. A per-request llama.cpp context is created on a blocking worker. Requests are serialized by the Axum semaphore, tokens are generated with the checkpoint's own embedded chat template, streaming output is bounded, and cancellation is checked during prompt and generation loops. Sampling uses the requested temperature and a fixed seed for repeatable diagnostics. Request cancellation is cooperative and can only interrupt between native decode calls.

The existing Axum OpenAI-compatible endpoints remain `/health`, `/v1/models`, and `/v1/chat/completions`; generation supports text chat with SSE or JSON output. Structured provider-side JSON Schema is not implemented here; the web translation workflow retains its prompt-enforced fallback when structured requests are rejected. This is a text-only backend, not a vision-capable model. The in-memory deterministic `FixtureBackend` remains compiled only in `#[cfg(test)]` so HTTP unit tests do not require a GGUF; no fixture-mode executable is shipped.

## Security and lifecycle

The process only binds to loopback. The server only authorizes known browser origins, including development localhost. The model file and user prompts remain on this computer when Local AI is selected. The model itself is not included in source ZIPs or Tauri bundles. Services settings control process startup; AI settings select the provider and model, without duplicating process actions.

LM Studio uses the configured HTTP-compatible endpoint and model discovered through `/v1/models`. The default proposal is `http://192.168.1.148:1234/v1/`, but the user can edit it. In browser/WebView mode the existing reqwest/WASM transport remains subject to CORS and mixed-content rules; a native Tauri HTTP proxy is not yet integrated. Credentials are not persisted; the client rejects transmitting keys over non-loopback unencrypted HTTP.

## Acceptance still required

The Rust and WASM builds, native DLL linkage, model loading, true GGUF inference and SSE with an installed model have not been run in the packaging environment. On Windows, use `cargo check --locked -p zipeek-ai-server -p zipeek-app`, `cargo test --locked -p zipeek-ai-server -p zipeek-ai-client`, and `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`. Rebuild the sidecar explicitly with `cargo make desktop-sidecars-rebuild`; the development staging task can reuse stale executables. Then start AI from Settings > Services and test both Local AI and LM Studio in Settings > AI. Test the external model list with GET `/v1/models` and a short translation; verify SSE and model failure recovery.
