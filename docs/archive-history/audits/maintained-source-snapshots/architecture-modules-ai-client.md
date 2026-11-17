# AI client boundary (R208)

`zipeek-ai-client` is a native-and-browser-WASM OpenAI-compatible HTTP client implemented in Rust with `reqwest =0.13.5`. `lib.rs` only declares modules and re-exports the public API. Internals are split by responsibility:

| File | Responsibility |
| --- | --- |
| `src/client.rs` | `AiClient(ClientConfig)`; HTTP POST, SSE response assembly, configurable progress callback, native five-minute total deadline, and connection diagnostics |
| `src/config.rs` | URL validation, `/v1/chat/completions` construction, credential and insecure-remote-HTTP safeguards |
| `src/protocol.rs` | Serde request and response wire types |
| `src/stream.rs` | Incremental SSE parser handling network chunk and UTF-8 boundaries, completion markers, output limits and malformed streams |
| `src/prompts.rs` | Translation directions, source checks, translation and connection-test messages |
| `src/error.rs` | Status-specific errors, provider JSON parsing, bounded/redacted diagnostic text |

There is **no JavaScript HTTP transport**, wasm-bindgen HTTP binding or platform-specific stub. `reqwest` uses browser Fetch on WASM and the native async HTTP stack on desktop. The already-existing web `services/ai/config.js` remains solely for settings and tab-memory API-key storage; `zipeek-web/src/services/ai/mod.rs` passes the key per request, without storing it in `AiClient`.

`zipeek-ai-protocol` is the pure sentence-analysis contract owner: it builds structured-analysis prompts, parses returned JSON, validates vocabulary and grammar against exact source/translation spans, and builds repair prompts. The web service still orchestrates the structured request, one repair, and plain-translation fallback when a provider rejects structured output with HTTP 400/422. No analysis fields or SQL have moved into the HTTP client.

## Endpoint and browser restrictions

The Rust client rejects non-HTTP(S) URLs, URL-embedded credentials, query/fragment in the base URL, and API-key transmission over non-loopback HTTP. Native builds configure a five-minute overall request deadline on `reqwest::Client` to allow slow local model generation. **The browser reqwest adapter does not expose `ClientBuilder::timeout` on wasm32**: the browser client therefore does not enforce that five-minute deadline. Streaming progress still works, but a separate target-independent timeout/cancellation design is needed before promising bounded browser requests. Native builds use an explicit no-redirect policy. The WASM `reqwest` adapter does not offer the same redirect configuration: browser Fetch may follow redirects; the client rejects responses whose final URL differs from the requested URL, but this is an **after-the-request check**, not a guarantee that the browser did not forward a request. Prefer a direct trusted `/v1/` endpoint. Standard browser CORS, mixed-content, and HTTPS rules still apply.

The default request contract uses `stream: true` and consumes `text/event-stream` using `reqwest::Response::bytes_stream`. A provider that returns a conventional JSON completion despite requesting streaming is also accepted without an extra model call. SSE frames are bounded and must terminate with `[DONE]` or a `finish_reason`; an incomplete response is rejected. The client exposes `complete_streaming(messages, api_key, on_delta)` for progress-aware consumers and retains `complete()` as the assembled-text wrapper. The web translation page and sentence-translation exercise now consume streaming progress. An accessible status widget displays the current phase and observed character count; it does not invent a completion percentage. During structured analysis, partial JSON and private model reasoning are never displayed as final learning content. Models that emit `reasoning_content` can report activity before any answer text appears.

Provider HTTP errors are translated to bounded readable messages. If a provider supplies JSON `error.message` and `error.code`, the message is sanitized, truncated and redacted against the supplied API key and common bearer patterns. Raw HTML and unrecognized error bodies are not shown. The actual configured model may still reject structured analysis because of context length, token budget, or provider capability: a successful plain fallback is not proof that analysis succeeded.

## Deterministic integration fixtures

`tests/fixtures/complex_zh_en.json` and `complex_en_zh.json` include complex concessive/conditional sentences with expected translation, two grammar explanations and three vocabulary items each. These are **hand-authored fixture expectations, not model evaluations**. `tests/http_integration.rs` launches a local TCP HTTP server and checks real reqwest request serialization, authorization headers, HTTP status handling, translation responses and `zipeek_ai_protocol::sentence_analysis::parse` validation for both directions. It exercises a structured HTTP 400 followed by a successful plain translation. These tests run without a live AI model.

`tests/live_model.rs` is ignored by default and invokes a real configured model for both fixture sentences. The detailed analysis uses streaming and prints incremental deltas to stdout with `--nocapture`, so lengthy generation shows progress before completion. It asserts nonempty plain translation and complete schema-valid analysis; it does not claim semantic equivalence to the hand-authored reference translation. A failure here may indicate the model cannot comply with the structured contract even if plain translation succeeds.

### Windows validation

```powershell
cargo test -p zipeek-ai-client --locked
cargo test -p zipeek-ai-client --test http_integration --locked
cargo check -p zipeek-ai-client --target wasm32-unknown-unknown --locked
cargo check -p zipeek-web --target wasm32-unknown-unknown --locked
cargo fmt --all -- --check
```

To exercise your local OpenAI-compatible model (example LM Studio endpoint), from PowerShell:

```powershell
$env:ZIPEEK_AI_ENDPOINT = "http://127.0.0.1:1234/v1/"
$env:ZIPEEK_AI_MODEL = "your-loaded-model-id"
# Optional: $env:ZIPEEK_AI_KEY = "your-key"
cargo test -p zipeek-ai-client --test live_model --locked -- --ignored --nocapture
```

Afterward run the existing managed Chrome Translate Sentence fixture scenarios from `zipeek-e2e`, and manually check the screenshot's structured-analysis rejection against the actual model. Node source tests do not validate Rust compilation or live browser behavior.

## Streaming browser acceptance

The managed E2E AI fixture now accepts both `stream: true` SSE and legacy JSON completions. `translator-streaming` intentionally sends a reasoning delta before delivering a fragmented structured-analysis response, giving the Chrome scenario time to assert the accessible `data-ai-stream-progress` status before the final result. Generation and evaluation fixtures also return SSE to match the real Rust HTTP contract. The fixture's private reasoning text is never displayed by the UI; only its length is surfaced. These Chrome scenarios still require actual Chrome/WebDriver execution.

## R208 typed protocol and structured-analysis contract

`zipeek-ai-client/src/message.rs` defines serializable/deserializable `ChatMessage` and `ChatRole` for ordinary translation and connection tests. The HTTP request and non-streaming response are typed; the SSE decoder deserializes typed per-choice deltas, including opaque reasoning activity counts. The public generic completion entry points continue accepting existing serializable message collections while callers migrate away from `serde_json::Value`.

`zipeek-ai-protocol/src/sentence_analysis/prompt.rs` owns the deliberately explicit translation and linguistic-evidence prompt; `schema.rs` owns the strict provider JSON Schema, and `sentence_analysis.rs` owns the `SentenceAnalysis` domain types and semantic evidence checks. All ten top-level fields are required, including empty enrichment arrays; unknown properties and ungrounded vocabulary/grammar fragments are rejected. The schema requires nested item objects to contain only their declared fields. `estimated_hsk` remains an estimate and may be null.

The Translate Sentence workflow requests OpenAI-compatible `response_format: {type: "json_schema", json_schema: {name: "sentence_analysis", strict: true, schema: ...}}` with streaming. If the provider rejects that format with HTTP 400/422, Zipeek retries the existing pedantic prompt without provider-side schema enforcement, then retains its plain-translation fallback. Partial structured JSON remains hidden until `SentenceAnalysis::parse` succeeds, and a single bounded repair attempt is still supported.

**Limits:** Neither JSON Schema nor substring matching proves correct pinyin or linguistic interpretation. The live Qwen test provided by the user passed both fixture directions in 182.64 seconds on an earlier version; R208's stricter contract and schema negotiation need a fresh native/WASM and live-model run. Dictionary-backed reading verification is a separate future enhancement. Browser provider support for strict JSON Schema varies; schema rejection is not the same as translation failure.
