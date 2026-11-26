# R591 complete-source content inventory and technical-debt triage

## Scope and level of assurance

**Input:** `zipeek-R590-WIP-statistics-chart-state-audit-20261119-152512.zip`.

**Coverage:** Every one of the 11,010 input archive file bodies was read. The file ledger includes path, byte size, SHA-256, detected kind, structural findings and content-derived indicators. All Rust and JavaScript source files were read and scanned for relevant code patterns. JSON files were parsed, image files structurally verified, and the bundled Hanzi Writer data was cross-checked at record level. The refreshed scan of the updated checkout is generated separately with `node scripts/source-content-audit.cjs`.

**Assurance boundary:** This is exhaustive **file-content coverage**, not a complete proof of semantic correctness for 152,454 lines of Rust. Automated scans do not establish the correctness of every function or UI interaction. The source archive has no accompanying compiled-build or browser evidence, and the environment has no `cargo`, `rustc`, `rustfmt`, Clippy or browser runner. No file is labeled semantically verified without executable tests and review. A high number of pattern matches is not evidence that each match is a defect.

## Complete-file inventory

| Content group | Input count | Verification |
| --- | ---: | --- |
| Rust source (`.rs`) | 973 | UTF-8 read, content-pattern scan, function/state/panic/unsafe indicators, per-file digest |
| JSON documents (`.json`) | 9,608 | Complete JSON parse and per-file digest |
| Other text/configuration/source | 376 | UTF-8 read, type-sensitive syntax/shape checks, per-file digest |
| Images (`.png`, `.webp`, `.ico`) | 50 | Image decode/structure check and per-file digest |
| UTF-8 dictionary source (`.u8`) | 2 | UTF-8 decode and per-file digest |
| WASM binary | 1 | WASM magic and per-file digest |
| Total | 11,010 | Every input member examined |

The VS Code `.vscode/settings.json` uses JSONC with trailing commas, which is supported by VS Code but rejected by strict JSON parsers. This is a format exception, not corrupt application JSON. All 9,608 other JSON documents parsed successfully. All 50 images passed a Pillow structural integrity check in the audit environment. The 9,574 per-glyph Hanzi Writer JSON records match the 9,574 records inside `all.json`; all stroke and median counts agree. `radStrokes` is absent in 556 records and is not required for the stroke/median contract.

The separate file ledger records content evidence for each path. Keep it as audit evidence, not as a Rust test result. The reproducible scanner lives at `scripts/source-content-audit.cjs`, and a dedicated data consistency check lives at `scripts/hanzi-dataset-audit.cjs`.

## Executed checks

| Check | Result | Meaning |
| --- | --- | --- |
| Full input-file read, hash and classification | 11,010/11,010 processed | Every input file body was inspected programmatically |
| Strict JSON parse | 9,608/9,608 passed | JSON syntax only, not domain semantics |
| Image structural verification | 50/50 passed | Image decoder accepted input images |
| Hanzi combined vs individual records | 9,574/9,574 equal | Source duplication is internally consistent |
| Hanzi stroke/median lengths | 9,574/9,574 valid | Shape contract holds |
| Node offline suites | 16 passed, 0 failed | Only the three offline audit suites |
| Node JavaScript syntax checks | 33/33 passed with module-aware check for worker ES module | Syntax only, not runtime behavior |
| Source asset integrity | Passed with two generated database artifact notices | Source mode, not deploy readiness |
| Docs validator before fixes | 4 active errors | Included broken maintained link, historical links and stale index |
| Docs validator after fixes | 0 active errors, 3 frozen-history warnings | Validates current docs and generated index |
| Rustfmt, Clippy, native/WASM compile, Rust tests, browser E2E | Not run | Required release blockers |

## Concrete corrections in R591

1. `docs/learning-practice/practice/settings-workspace.md`: replace obsolete `services/ai/config.js` reference with the actual `zipeek-browser/src/ai_configuration.rs` module. The API key lives in Rust thread-local browser memory, not a deleted JS config module.
2. `scripts/docs-validate.cjs`: classify paths under the frozen `docs/archive-history/` tree as historical warnings, rather than incorrectly treating some historical source snapshots as maintained-document blockers. Current document links remain strict errors.
3. `crates/zipeek-web/static/docs/index.json`: regenerate the bundled docs index to match maintained source files.
4. `scripts/source-content-audit.cjs`: add reproducible full-source per-file content and digest reporting, with no third-party dependencies.
5. `scripts/hanzi-dataset-audit.cjs`: add reproducible per-glyph and combined dataset consistency checking.

No Rust implementation logic, dependencies, databases, stored user data or public routes were changed. The correctness and architecture debt below remains open.

## Risk-ranked outstanding code findings

These are source-identified review targets, not proven bugs unless explicitly described as such.

| ID | Priority | Scope and evidence | Unresolved action | Confidence |
| --- | --- | --- | --- | --- |
| A01 | Blocker | No Rust/WASM compile or browser evidence for successive Statistics architecture checkpoints | Run locked workspace tests, formatting, Clippy, WASM and root/Pages E2E on exact candidate | Verified evidence gap |
| A02 | High | `crates/zipeek-web/src/pages/system/stats/grammar/page.rs` and `stats/explorer/page.rs` duplicate generation/query/cancellation state for bulk selection | Extract and test shared typed request lifecycle; prove query changes and unmount cannot commit stale selections | Source-confirmed architecture debt |
| A03 | High | `stats/state/mod.rs` holds coupled reducer fields including selected preview string IDs | Audit each reducer action invariant; introduce canonical typed identities before Practice and Word Set integration | Source-confirmed architecture debt |
| A04 | High | `crates/zipeek-web/src/pages/practice/local_session.rs` combines setup, generation, scoring and session UI lifecycle | Function-by-function ownership review with reducer-transition and failure tests | Review required |
| A05 | High | `crates/zipeek-web/src/pages/system/settings/mod.rs` owns a large Settings workspace | Review component and preference ownership and split only independent responsibilities | Review required |
| A06 | High | `crates/zipeek-web/src/services/personal_data/{backup,storage,opfs}.rs` and Practice session store parse and persist browser-local learner data | Fault-inject invalid/truncated backups, quota failures, repeated writes and cancellation; verify readback and deletion | Risk boundary, not confirmed flaw |
| A07 | High | AI endpoints, response parsing, streaming, and model payloads in `zipeek-ai-client`, `zipeek-ai-protocol`, `zipeek-web/src/services/ai` | Validate external-input budgets, redirect and URL behavior on native/WASM and malformed responses | Risk boundary, not confirmed flaw |
| A08 | High | CLI archive unpacker at `crates/zipeek-cli/src/archive/unpack.rs` now has entry/size/path/symlink controls | Execute the Rust archive-limit and restore/rollback tests; test file collisions and partial failure on Windows | Existing mitigations source-confirmed, not tested here |
| A09 | Medium | Chart hook at `stats/overview/lifecycle.rs` derives chart accent at initialization but its effect key follows weekly buckets | Verify theme changes do not leave stale chart color; select explicit theme dependency if required | Potential UI defect |
| A10 | Medium | Dialog focus helper supplies Tab wrapping but background inertness and nested dialog behavior are not browser-verified | Run keyboard, screen-reader and mobile browser acceptance, both base paths | Verification gap |
| A11 | Medium | `crates/zipeek-exercises/src/cloud_match.rs`, `crossword.rs` and `time_weaver.rs` are large domain gameplay modules | Check deterministic state transitions, failed generation and replay handling with seeded tests | Maintainability and correctness review |
| A12 | Medium | `crates/zipeek-db/src/lib.rs` declares crate-level `allow(dead_code)`, `allow(unused)`, `allow(private_interfaces)` | Check which lint suppressions are actually needed, narrow them once compiling | Source-confirmed lint debt |
| A13 | Medium | `crates/zipeek-pedagogy/src/curriculum/vocabulary.rs` is a 5,658-line curriculum source | Verify editorial data constraints and split source data only when useful, not for arbitrary line counts | Editorial maintainability review |
| A14 | Medium | Browser `unsafe` extern import wrappers at `zipeek-browser/src/charts.rs` and `zipeek-db/src/lib.rs` | Confirm JS/WASM signatures and import error paths with actual WASM build | Reviewed wrapper pattern; not assumed vulnerability |
| A15 | Medium | Historical docs contain three broken source links, preserved as frozen history | Keep warnings discoverable; no retrospective rewriting of source evidence | Historical warnings acknowledged |

## Cross-crate review register

This table records which areas were inventoried, which contained concrete risk indicators, and where semantic verification remains necessary. **Content scanned** means every file body was read by the audit; it does not mean every function passed a human or compiler review.

| Crate | Rust files | Rust lines | Primary next review |
| --- | ---: | ---: | --- |
| zipeek-ai-client | 11 | 1,376 | HTTP/stream limits and error handling |
| zipeek-ai-core | 1 | 247 | Pure AI domain contracts |
| zipeek-ai-protocol | 14 | 2,037 | Parsing/validation of external model responses |
| zipeek-ai-server | 10 | 1,046 | Process lifecycle and resource limits |
| zipeek-app | 9 | 1,336 | Desktop service ownership and sidecars |
| zipeek-browser | 17 | 2,167 | Browser bindings and JS import lifecycle |
| zipeek-cli | 22 | 5,479 | Source restore safety and rollback |
| zipeek-core | 26 | 3,591 | Practice domain invariants |
| zipeek-db | 19 | 2,992 | Native/WASM facade and lint suppressions |
| zipeek-db-gen | 62 | 8,976 | Data ingestion, generators and deterministic outputs |
| zipeek-docs | 3 | 445 | Sanitization of Markdown/HTML |
| zipeek-e2e | 135 | 24,777 | Coverage and reproducibility |
| zipeek-exercises | 71 | 14,744 | Seeded engines and transitions |
| zipeek-geo-assets | 9 | 1,530 | External geometry validation |
| zipeek-hanzi-writing | 9 | 1,507 | Stroke/recognition correctness |
| zipeek-image-server | 11 | 1,296 | Inference resource management |
| zipeek-pedagogy | 20 | 12,772 | Curriculum contracts and generated worker data |
| zipeek-static-server | 3 | 622 | HTTP file/path boundaries |
| zipeek-themes | 4 | 157 | Theme provider state |
| zipeek-tts-stt | 4 | 704 | Microphone/TTS external boundaries |
| zipeek-ui | 1 | 3 | Placeholder crate necessity |
| zipeek-web | 512 | 64,650 | Yew controllers, state, storage and async effects |

## Reproducibility and interpretation

```powershell
node scripts/source-content-audit.cjs target/source-content-audit.jsonl
node scripts/hanzi-dataset-audit.cjs
node --test scripts/tests/*.test.cjs
node scripts/asset-integrity-audit.cjs
node scripts/docs-validate.cjs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --target wasm32-unknown-unknown -p zipeek-web --locked
```

The first command emits a path-by-path evidence ledger, including source patterns, hash and verification exceptions. Node source checks can detect broken files and flag risky constructs, but cannot certify Rust ownership, lifetime, concurrency, sanitization, UX or runtime behavior. No clean Rust build or application-wide debt-free claim is supported by the present evidence. Future fixes require compiler and behavior gates before acceptance.
