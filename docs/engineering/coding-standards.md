# Coding standards

Conventions for application code. Process rules live in [AGENTS](../../AGENTS.md);
testing strategy in [testing/TESTING.md](../testing-quality/acceptance/testing.md); architecture in
[architecture/ARCHITECTURE.md](../architecture/architecture.md).

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
- **Derive UI affordances from typed state.** Do not hide or enable actions by matching user-facing error-message text; pass explicit props from reducer-backed containers.
- **Keep pagination contracts shared.** Query limits, lookahead checks and displayed results must use the same page-size constant.

## Rust

### Formatting and lints

- `cargo fmt --all` - mandatory, no exceptions.
- Clippy checks deny warnings; use affected packages/targets during routine
  work and workspace/all-targets for full regression under AGENTS §6. Avoid `#[allow(...)]` without an approved technical reason. Do not add code comments. Prefer resolving the lint through code changes.
- Edition 2024, Rust 1.97.1 (see `rust-toolchain.toml`).

### Readability of expressions and match arms

- Keep match arms on separate lines when a match has multiple variants. Do not
  compress whole match expressions or control-flow branches onto a single line,
  even if Rustfmt accepts it.
- Format multi-step closures, error handling and long function calls as readable
  multi-line blocks. Short single-expression closures may stay inline when clear.
- Prefer `const fn` for genuinely compile-time-capable pure methods, such as
  enum-to-static-string mappings; do not mark functions `const` merely to save lines.
- Treat `cargo fmt` as the minimum formatting requirement, not permission to
  squeeze several logical operations onto a single line.

### Imports and qualification

- Prefer explicit `use` declarations at module scope for externally defined types,
  traits, functions and constants used in implementation logic. Group related
  imports clearly and keep application code focused on behavior rather than
  repeated module paths.
- Use a qualified path when it resolves ambiguity or materially improves clarity,
  including naming conflicts and one-off references. Do not use wildcard imports
  except for established preludes or intentionally scoped test imports.
- Keep imports needed by a module at its boundary rather than inside event
  callbacks, effects or render expressions. Preserve a local import only when
  its limited scope serves a concrete purpose.

### Modules and platform API boundaries

- A stateful browser integration such as microphone capture must separate the
  public recorder API, owned state, event/capture wiring, waveform processing,
  and error conversion when those concerns become substantial. Keep the parent
  `mod.rs` limited to exports, shared constants, and module declarations.
- Keep JavaScript-API return types faithful to the pinned `web-sys` signatures.
  In particular, `BlobEvent::data()` is `Option<Blob>`; handle absent chunks
  without panicking, and pass `blob.as_ref()` to `js_sys::Array`.
- Keep tuple fields private by default. A sibling module requiring controlled
  construction should use a `pub(super)` constructor, not a public inner field.
- Keep source tests aligned with extracted module files; never weaken runtime
  assertions just to satisfy a source-layout assertion.
- Preserve single-field tuple structs for opaque ownership wrappers where
  they express the abstraction clearly, such as `MicrophoneRecorder(Rc<RefCell<State>>)`.
- When moving code to submodules, preserve public imports, existing behaviors,
  and test coverage. Source assertions must follow the actual module graph,
  rather than forcing code back into a large parent file.
- A source-level Node pass is not a substitute for a target-specific Rust build;
  WASM/browser APIs require `cargo check --target wasm32-unknown-unknown`
  and browser acceptance before a release can be considered verified.

### Stateful Yew feature components

- Use `#[function_component]` and a typed `Properties` contract for standalone
  UI components; do not expose positional rendering helpers with many unrelated
  arguments (`view(board, feedback, hint, complete, ...)`).
- Keep feature-specific modules inside a named directory (`feature/mod.rs`,
  `feature/board.rs`, `feature/state.rs`) rather than accumulating sibling files
  such as `feature_board.rs` next to route files.
- Keep the route page responsible for URL decoding and data loading. A session
  owner manages interactive state; tiles, controls, progress and results are
  focused components. Use a scoped typed context and reducer when several
  nested components need the same state and actions; do not introduce global
  state for a single game.
- Model transitions as reducer actions with explicit guards. For advancing to
  another round, generate the next board before committing the counter and
  cumulative scores. Never mutate half a session on a failed transition.
- Retain stable `data-*` hooks, `aria-pressed`, keyboard-accessible buttons,
  and difficulty-dependent Hanzi font sizes through refactors.
- Preserve the domain engine in a pure Rust crate without `web-sys`, Yew or
  JavaScript dependencies. Split domain model, generation, gameplay and tests
  when they have separate reasons to change.

### Naming

- Types, traits, enums: `PascalCase`.
- Functions, variables, modules: `snake_case`.
- Constants and statics: `SCREAMING_SNAKE_CASE`.
- Acronyms: `Url` not `URL`, `Api` not `API`, per Rust API guidelines.
- Domain types: `Word`, `CandidatePool`, `RoundFilter`, `HskSystem`, `Hsk2Level`,
  `Hsk3Level` - no `Data`, `Info`, `Item` suffixes.
- Error types: `BuildError`, `DbError`, `LoadError` - not `Error` or generic names.

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
  error, and in demonstrably unreachable branches with self-explanatory types or control flow.
- **Not allowed:** on `Result`/`Option` from user input, DB queries, network
  calls, or parsing external data.
- Required provider contexts are component-tree invariants: `use_context::<RequiredContext>().unwrap()` is allowed when the parent route always mounts that provider. A missing provider is a programming defect, not a user-facing recoverable condition.
- Never panic on user input, database operations, localStorage, network failures, or other recoverable conditions in a Yew component or handler; render an error state or return a typed error instead.

### Modules and file organization

- One module per file. `web/src/components/foo.rs` not `web/src/components.rs`.
- Public API goes through `mod.rs` re-exports where it improves ergonomics
  (e.g., `pub use round::{build_round, CandidatePool, RoundFilter};`).
- Pure logic lives in `zipeek-core` when shared between `web` and `db-gen`;
  otherwise local to its crate.
- WASM-specific browser access (DOM, `js_sys`, `web_sys`) belongs in
  `zipeek-browser` or the `zipeek-db/wasm` adapter when it is not Yew-specific.
  Yew rendering stays in `zipeek-web`. `zipeek-core` must compile natively.
- **Rust/WASM-first browser implementation:** Implement Zipeek-owned application behavior,
  state, validation, async orchestration, resource lifetime and cleanup in Rust.
  Prefer typed `web-sys`/`js-sys` APIs. If a browser feature or third-party
  library needs JavaScript, use the smallest `wasm-bindgen` interop boundary,
  normally `inline_js`, with typed Rust-facing functions and opaque newtypes
  for external objects. Do not add handwritten `.js` service layers merely to
  call another JavaScript module or to hold application state.
- **Exceptions are intentional:** Pinned third-party JavaScript/WASM assets
  (including D35 Chart.js and DuckDB-WASM), build scripts, test probes, and
  minimal pre-WASM bootstrap remain supported. Preserve their packaging and
  licenses. Distinguish a permitted vendor module from application logic.
  See [Rust/WASM browser interoperability](../architecture/modules/rust-wasm-interop.md)
  for ownership, newtype, dynamic-import, lifecycle and verification rules.

## Yew

### Tailwind styling (D31)

Tailwind CSS 4.3.4 remains the required component styling approach. A35 corrected
the shell/card stylesheet drift on 2026-10-28: mounted components use utilities,
main.css holds theme aliases and minimal foundation rules, and inline.css retains
critical bootstrap/loading styling. Rust source detection is explicit; the
unused v3 configuration and bespoke shell/card selectors are removed.

- Use Tailwind utilities in Yew markup for layout, spacing, sizing, typography,
  borders, responsive rules and interaction states. Keep conditional variants as
  complete class strings that Tailwind can detect; do not construct class names
  from fragments at runtime.
- Keep shared theme/design tokens in CSS where appropriate and consume them
  through utilities. Preserve the slate/blue theme and landscape composition.
- Allow focused custom CSS for critical pre-WASM bootstrap styling, shared
  tokens, keyframes or a concrete effect that benefits from it. A reason for one
  exception does not justify moving ordinary component styling into stylesheets.
- Do not hide the same bespoke component system behind blanket @apply rules.
  Factor repeated UI into Yew components and small explicit utility variants.
- Verify Tailwind 4 source detection for Rust/Yew files and remove obsolete
  conflicting rules as components migrate. Do not assume legacy v3 configuration
  is automatically active or upgrade packages during the correction.

The implemented styling boundary and visual checks are in
[UI_ARCHITECTURE](../architecture/ui-architecture.md#tailwind-styling-implementation-d31--a35).

### Components

- One component per file, function components only (no classes).
- Props are `#[derive(Properties, PartialEq)]` structs.
- Use `#[function_component]` with a clear component name matching the file
  (`WordCard` in `word_card.rs`).
- Avoid prop drilling past 2 levels; use contexts.
  The 2026-10-27 review's concrete provider/reducer proposal is in
  [UI_ARCHITECTURE](../architecture/ui-architecture.md). Shared services and
  feature state belong behind scoped hooks/providers; simple leaf inputs still
  use props. Do not expose raw state setters as another feature's public command
  interface.

### Yew presentation composition

- Import components explicitly at the module boundary (`use crate::components::{BrowseProvider, WordSetsProvider};`) rather than repeating qualified paths in `html!` markup. Wildcard imports are not required for component composition.
- Write nested providers and other multi-element Yew trees across multiple lines, with each element's closing tag aligned to its opener; do not compress provider nesting into one line.
- Keep page-specific callback conditions and view-building chains readable with normal Rust multiline blocks, even where the formatter would tolerate a dense expression.
- For nontrivial status-specific markup, compute `let status_component = match &state.status { ... };` before the parent `html!` and insert `{status_component}` in the shallow outer view.
- Extract meaningful loading/error/empty, result-card, and pagination views into focused components. Prefer keeping short related status components in one module over one file per line of markup.
- Container components access reducers/providers and wire callbacks; presentational components take focused `Properties` and do not reach into unrelated feature contexts.
- Keep stable `data-component` and existing `data-*` E2E hooks during refactors.

### Hooks

- `use_state` for temporary form input local to the owning component; use a pure `use_reducer` plus a scoped provider for shared feature state and interdependent transitions. Keep I/O in the provider/service, never inside `Reducible::reduce`.
- `use_effect_with` for side effects, with explicit dependency lists - do not
  use empty dependency lists unless the effect is genuinely once-only.
- `use_memo` for derived state that's expensive to compute; do not memoize
  cheap operations.
- `Callback` for event handlers passed to children; `use_callback` when the
  identity matters (prevents unnecessary re-renders).

### State ownership and refactor discipline

- A page with multiple interdependent request/output/error/loading fields should use
  a typed `Reducible` state machine. Independent controls such as a font preference
  may remain `use_state`; hook count is a review signal, not an automatic failure.
- Reducers must be pure. Network requests, browser storage and DOM operations stay
  in effects, callbacks or services; render state transitions are dispatched
  atomically, with stale asynchronous results rejected using request epochs.
- Place substantial reducer implementations in a feature-local `state.rs` module.
  Do not replace independently managed hooks with a large mutable "controller"
  that merely hides the same coupling.
- Split oversized presentation trees into focused feature components or view
  functions. Prefer short, named sections to nested 100-line `html!` expressions.
  Keep event types explicit (`FocusEvent` for focus, `MouseEvent` for clicks).
- A module declared as `mod foo;` may use either `foo.rs` or `foo/mod.rs`, never
  both. Parent modules named `foo.rs` may own child files `foo/bar.rs` without
  a second `foo/mod.rs`; do not use `#[path]` to paper over moves.
- A source-only Node suite does **not** validate Rust syntax or type checking.
  Before declaring a refactor compiled, run `cargo fmt --all -- --check`,
  `cargo check --workspace --locked`, `cargo test --workspace --locked`, Clippy,
  and the WASM target. Report unexecuted checks explicitly.

### Events and handlers

- Never `unwrap` on `event.target_dyn_into::<HtmlInputElement>()` - the DOM can
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
- No I/O - it holds types and pure logic only. Queries take data as arguments,
  not a DB handle.
- Public API is versioned by the repo, not semver; breaking changes to
  `zipeek-core` are fine during development (AGENTS.md §2).

## Database generator (`crates/zipeek-db-gen`)

- Native binary; runs manually, never in an app build.
- Reads pinned inputs only - no network calls. Source fetching is a separate
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
  `storage_compatibility_version = v2.0.0` before creating output; check the actual installed runtime and artifact format rather than relying on an older compatibility claim. Future artifact creation
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
  the expected output hash - see ROUND_GENERATION.md for the contract.

## E2E component selectors

- Give significant mounted Yew page and feature roots a unique, stable
  `data-component="kebab-case-name"` value. Prefer identifying the component
  (for example, `browse-page`) rather than its current heading or CSS layout.
- Use `data-action` for controls where action identity improves testing, and
  `data-state` or an existing dedicated `data-*` attribute for observable state.
  Retain semantic roles, accessible labels, and native keyboard behaviour.
- E2E selectors must not infer a route from an incidental tag structure or
  placeholder once the route renders a real component. Placeholders may still
  be asserted for genuine unimplemented or not-found routes.
- Preserve existing test-specific attributes during gradual migration; no
  dependency or round-URL format changes are required.

## Documentation

Code carries no doc comments and no explanatory inline comments. The code
should be self-explanatory through naming and structure; if a reader needs a
comment to follow it, the fix is clearer names, smaller functions, or an entry
in the relevant `docs/` file.

- No `///` doc comments on functions, types, or modules.
- No `//` explanatory comments above or inside code.
- `// TODO:` and `// FIXME:` comments are also forbidden. If work is needed,
  add a task to `TASKS.md` and reference it in the commit message or PR body.
- Rationale and design decisions live in `docs/architecture/architecture.md`,
  `docs/games/round-generation.md`, `docs/dictionary-data/schema/db.md`, or the
  relevant decision record - not in the source tree.
- No comment exceptions apply to lint suppressions or unreachable branches. When a lint allowance is necessary, record rationale in maintained technical documentation and obtain approval for the change.

## Feature module organization

- Put feature-specific Yew components under `components/<feature>/`; Practice has
  `setup/`, `exercises/` and `session/` subdivisions. Generic primitives remain at
  the component root. Pages live under `pages/<feature>/`, keeping stable route
  module names through explicit Rust `#[path]` declarations where appropriate.
- Preserve existing public reexports while moving implementation files. Avoid
  adding icon abstractions: use `yew_icons::{Icon, IconData}` directly in views.
- Only use icon constants present in the pinned `yew_icons` inventory.
  Verify icon identifiers against the currently pinned `yew_icons` crate and compile affected Rust views. Historical icon fixture details are preserved in the linked source snapshot.
- When moving files, update the owning `mod.rs`, import paths, source-contract
  tests and living documentation in the same change. Do not move files merely
  to shorten individual names or add wrappers without an actual dependency need.

## Additional implementation review contracts

- **Required contexts:** Trace the actual provider tree, routes, tests and callers before changing `Option<T>` hooks to required accessors. Required providers should have an explicit accessor contract and actionable absence diagnostics; intentionally optional providers remain optional. A provider's runtime loading/failure state is not its absence.
- **Workflow state:** Identify authoritative, derived, transient, request and persisted state before major refactors. Verify coordinated resets, invalid states, overlapping requests, stale callbacks and lifecycle cleanup; use a reducer only when it clarifies the invariant.
- **Markup and events:** Keep `html!` markup expanded and reviewable, name callbacks before markup rather than constructing them in attributes, and prefer typed callbacks over avoidable DOM parsing. If DOM `data-*` routing is justified, validate values, choose `current_target` when appropriate and test nested clicks and keyboard activation.
- **UX and accessibility:** Preserve shared shell, tokens, icons, semantic controls, focus, touch and keyboard behavior, and explicit loading/failure/recovery states. Inspect rendered UI when available; compilation alone does not verify usability.
- **Async performance:** Review browser work, cancellation, stale results and cleanup before adding timers or workers. Use measurable evidence before claiming performance improvements; deferring synchronous computation does not make it nonblocking.
- **Source audits:** For material findings cite file/line evidence, observed behavior, concrete impact, minimal remedy, priority and remaining uncertainty. Avoid judgments based only on line count, hook count or stylistic preference.
- **No legacy compatibility:** The development-phase policy in [AGENTS](../../AGENTS.md) applies to all historical formats. Deterministic current behavior, input validation and intentional handling of local data loss remain required.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../archive-history/audits/maintained-source-snapshots/engineering-coding-standards.md). Dated implementation claims in that snapshot are not current build or browser verification.
