# Coding standards

Conventions for application code. Process rules live in [AGENTS](../AGENTS.md);
testing strategy in [TESTING](TESTING.md); architecture in [ARCHITECTURE](ARCHITECTURE.md).

## General principles

- **Prefer clarity over cleverness.** A reader should understand the code without
  tracing types across files.
- **Fail loudly in development, gracefully in production.** `unwrap` / `expect` is
  acceptable in tests and unreachable branches; never in request handlers, DB
  queries, or user input parsing.
- **No silent fallbacks for correctness bugs.** If a query returns unexpected
  data, log it. If a computation can fail, return `Result`, don't `Option`-swallow
  it.
- **Preserve intent when editing.** Do not refactor adjacent code to "clean up"
  while fixing a bug; keep diffs focused.

## Rust

### Formatting and lints

- `cargo fmt --all` — mandatory, no exceptions.
- `cargo clippy --workspace --all-targets -- -D warnings` — mandatory, no
  `#[allow(...)]` without a comment explaining why.
- Edition 2024, Rust 1.97.1 (see `rust-toolchain.toml`).

### Naming

- Types, traits, enums: `PascalCase`.
- Functions, variables, modules: `snake_case`.
- Constants and statics: `SCREAMING_SNAKE_CASE`.
- Acronyms: `Url` not `URL`, `Api` not `API`, per Rust API guidelines.
- Domain types: `Word`, `CandidatePool`, `RoundFilter`, `HskSystem`, `Hsk2Level`,
  `Hsk3Level` — no `Data`, `Info`, `Item` suffixes.
- Error types: `BuildError`, `DbError`, `LoadError` — not `Error` or generic names.

### Errors

- Public functions returning fallible operations use `Result<T, E>` with a
  domain-specific error enum.
- Use `thiserror` for error enums (already a dependency). Do not use `anyhow`
  in library code; it's permitted only in `db-gen`'s `main.rs` if needed for
  user-facing errors.
- Errors must be actionable: include the offending input where possible.
  - Bad: `Err(BuildError::InsufficientPool)`
  - Good: `Err(BuildError::InsufficientPool { view: View::Strict, found: 5, needed: 8 })`

### `unwrap` / `expect` / `panic!`

- **Allowed:** in unit tests, in `const fn` when the failure is a compile-time
  error, and in genuinely unreachable branches (documented with a comment).
- **Not allowed:** on `Result`/`Option` from user input, DB queries, network
  calls, or parsing external data.
- Never `panic!` in a Yew component render or an event handler — the app dies.
  Use a Result-returning path and render an error state.

### Modules and file organization

- One module per file. `web/src/components/foo.rs` not `web/src/components.rs`.
- Public API goes through `mod.rs` re-exports where it improves ergonomics
  (e.g., `pub use round::{build_round, CandidatePool, RoundFilter};`).
- Pure logic lives in `zipeek-core` when shared between `web` and `db-gen`;
  otherwise local to its crate.
- Wasm-specific code (DOM, `js_sys`, `web_sys`) stays in `web/`; `zipeek-core`
  must compile natively.

## Yew

### Components

- One component per file, function components only (no classes).
- Props are `#[derive(Properties, PartialEq)]` structs.
- Use `#[function_component]` with a clear component name matching the file
  (`WordCard` in `word_card.rs`).
- Avoid prop drilling past 2 levels; use contexts.

### Hooks

- `use_state` for local UI state; `use_reducer` when state changes are complex
  or need to be shared.
- `use_effect_with` for side effects, with explicit dependency lists — do not
  use empty dependency lists unless the effect is genuinely once-only.
- `use_memo` for derived state that's expensive to compute; do not memoize
  cheap operations.
- `Callback` for event handlers passed to children; `use_callback` when the
  identity matters (prevents unnecessary re-renders).

### Events and handlers

- Never `unwrap` on `event.target_dyn_into::<HtmlInputElement>()` — the DOM can
  change. Use `if let Some(el) = ...`.
- Every interactive element has a keyboard equivalent (Enter/Space for buttons,
  arrow keys for navigation). See UX.md's accessibility section.

### Async

- Use `wasm_bindgen_futures::spawn_local` for fire-and-forget effects, or
  `use_async` patterns for stateful async work.
- Do not block the render loop. Long queries go through a state machine
  (Idle → Loading → Ready / Error), never a synchronous await in render.

## Shared crate (`zipeek-core`)

- Must compile on both `wasm32-unknown-unknown` and native.
- No `web_sys`, `js_sys`, `wasm_bindgen` imports.
- No I/O — it holds types and pure logic only. Queries take data as arguments,
  not a DB handle.
- Public API is versioned by the repo, not semver; breaking changes to
  `zipeek-core` are fine during development (AGENTS.md §2).

## Database generator (`crates/db-gen`)

- Native binary; runs manually, never in an app build.
- Reads pinned inputs only — no network calls. Source fetching is a separate
  shell script (see DB.md).
- Verifies SHA-256 of every input file against `MANIFEST.toml` before parsing;
  aborts on mismatch.
- Logs every dropped word, unknown PoS code, level conflict, and gloss-pipe
  replacement.
- D22 requires deterministic logical output: identical verified inputs and
  pinned tooling produce the same ordered rows, IDs, schema/index definitions,
  and import/drop diagnostics. Record SHA-256 for each final `hanzi.duckdb`
  artifact as an integrity check; different runs need not have identical file
  bytes. Do not patch or mask database headers to force a matching checksum.
- Before generation, query `SELECT version()` against the native
  engine, log the returned version, and fail unless it is exactly `v2.0.0`
  (D23 supersedes D21's earlier v1.4.3 condition). Configure and verify
  `storage_compatibility_version = v1.4.0` before creating output; default neo
  files are not readable by the pinned WASM runtime. Future artifact creation
  must use this configured connection and explicitly compatible storage.
  This is a generator invocation check, not a Cargo build script or an
  automatic application-build generation step.

## Tests

- Unit tests in the same file as the code (`#[cfg(test)] mod tests { ... }`)
  for pure logic.
- Integration tests in `tests/` for cross-module behaviour.
- Test names describe the scenario, not the function:
  - Bad: `test_build_round`
  - Good: `build_round_returns_insufficient_pool_when_strict_view_is_small`
- Fixtures live in `tests/fixtures/` as data, not inline structs, when larger
  than ~20 lines. See TESTING.md's fixture patterns.
- Golden tests (round builder, seed encoding) assert both the input string and
  the expected output hash — see ROUND_GENERATION.md for the contract.

## Documentation

Code carries no doc comments and no explanatory inline comments. The code
should be self-explanatory through naming and structure; if a reader needs a
comment to follow it, the fix is clearer names, smaller functions, or an entry
in the relevant `docs/` file.

- No `///` doc comments on functions, types, or modules.
- No `//` explanatory comments above or inside code.
- `// TODO:` and `// FIXME:` comments are also forbidden. If work is needed,
  add a task to `TASKS.md` and reference it in the commit message or PR body.
- Rationale and design decisions live in `docs/ARCHITECTURE.md`,
  `docs/ROUND_GENERATION.md`, `docs/DB.md`, or the relevant decision record —
  not in the source tree.
- Only exception: `#[allow(...)]` attributes must be accompanied by a comment
  explaining *why the lint is suppressed*, since Clippy runs with `-D warnings`
  and reviewers need to see the justification. This is the sole permitted
  comment form.
