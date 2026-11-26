# Component and state ownership refactor review

## R601 source scope

The user approved the ten targets listed in the R600 corrective candidate review. This pass extracts concrete responsibilities without new dependencies, persistence format changes, route changes or new state stores. This is not a claim of complete remediation of each large controller.

| Target | Change | Remaining debt |
| --- | --- | --- |
| Practice `local_session` | Extracted typed Hanzi completion preview and pure completion arithmetic to `local_session/progress.rs`; extracted completed-session summary and history notice to `local_session/summary.rs` | Async generation and persistence callbacks still share the parent controller |
| Practice `generated_session` | Extracted seeded tile ordering into tested pure `generated_session/order.rs`; empty collections no longer rotate out of bounds | Translation evaluation lifecycle and multi-exercise presentation still share a component |
| Statistics `state` | Moved reducer transition implementation and tests out of state declarations into `state/transitions.rs` and `state/tests.rs` | Reducer action surface and string selection IDs are unchanged |
| Statistics `query` | Extracted pure query selectors and matching-ID enumeration into `query/selectors.rs`, with typed query and page contracts retained in `query/mod.rs` | Query contract/types can be split further after compiler verification; backend adapter tests still required |
| Settings `ai` | Extracted diagnostic status and trace presentation into `ai/diagnostics.rs` | Async connection lifecycle is still owned by the page and requires browser cancellation tests |
| Crossword | Extracted deterministic layout/placement and vocabulary normalization to `crossword/layout.rs` | Runtime gameplay implementation remains in module root; deterministic seeded tests need execution |
| Time Weaver | Extracted date/time language rules and calendar helpers to `time_weaver/calendar.rs` | Question generation and runtime transitions remain combined |
| Exercise session | Extracted public immutable data contracts and validated instance construction to `session/model.rs` | Session transition behavior remains in module root and requires property/failure tests |
| Personal history | Extracted stored-count parsing and UUID validation to `personal_history/validation.rs` | Lock acquisition and read/write/erase remain coupled to the DuckDB adapter |
| Recognition history | Extracted typed provenance wire validation and duplicate-event checks to `recognition_history/validation.rs` | SQL locking/readback retry protocol remains in the adapter |

## Verification and risk

- The same parent module paths and exported feature names are retained. File/directory mixed roots have been replaced by `feature/mod.rs` plus sibling modules.
- New pure generated-session ordering tests cover deterministic permutation output and empty collections. Existing moved tests retain their assertions. The source includes no external dependencies or generated assets.
- 16 offline Node tests passed. Lexical delimiter checks passed across the affected Rust module files. The documentation validator reported 0 active errors and 5 frozen historical warnings.
- Cargo, rustfmt, Clippy, Rust unit tests, locked WASM builds and browser E2E could not be executed in this environment; structural checks are not compiler evidence.
- Changes to module visibility or names in imports may still produce Rust diagnostics. Fix any Windows compiler findings on the exact checkpoint before accepting it.

## Required acceptance checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-exercises -p zipeek-db -p zipeek-web
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo make e2e-full
```

Run Practice generated translation with empty/one/multiple tiles, Grammar and Vocabulary pagination and all-matching selection, Settings AI diagnostic retry/cancellation, deterministic Crossword/Time Weaver and session terminal transitions, and browser storage evidence errors. Preserve `/` and `/zipeek/` behavior.
