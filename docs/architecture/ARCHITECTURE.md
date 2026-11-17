# Architecture

Workspace architecture overview. Current package membership must be confirmed against the root Cargo manifest. This file describes code boundaries, not older proposals or historical release snapshots. Approved future behavior is documented in [Product](../product-ux/product.md), [UX](../product-ux/ux.md) and [Tasks](../project-management/tasks.md).

## Workspace

The root Cargo workspace contains:

| Package | Role |
| --- | --- |
| `zipeek-web` | Yew CSR application, router, shell, components, browser services and orchestration |
| `zipeek-core` | Shared vocabulary types and deterministic round-domain primitives |
| `zipeek-exercises` | Practice definitions, validated setup drafts and pure exercise/session domains |
| `zipeek-ai-protocol` | Authored prompts and validation of model responses, including sentence analysis and generated exercise contracts |
| `zipeek-ai-core` | New provider-neutral generation request, validation and streamed event contract; no backend connected yet |
| `zipeek-ai-server` | Injectable HTTP chat-completion transport with JSON/SSE and restricted CORS; executable/model runtime still unimplemented |
| `zipeek-ai-client` | OpenAI-compatible HTTP transport for native and WASM clients |
| `zipeek-db` | Native/WASM database infrastructure, typed queries and personal-storage probe |
| `zipeek-db-gen` | Native generator for the read-only DuckDB dictionary and manifest |
| `zipeek-ui`, `zipeek-browser` | UI and browser-specific reusable functionality |
| `zipeek-hanzi-writing` | Pure Hanzi stroke/ink geometry |
| `zipeek-themes` | Theme providers and resolution |
| `zipeek-e2e` | Managed Chrome E2E harness, scenarios and in-process Rust AI fixture |
| `zipeek-image-server` | Optional local image service |
| `zipeek-cli` | Clap command orchestration, asset preparation, source pack/unpack, dictionary publication tooling |
| `zipeek-static-server` | Tokio/Axum server for already-built static Trunk assets, supporting root and Pages prefixes |

`Cargo.toml` is authoritative for current members. Tool/version pins are enforced by manifests and lockfiles, not historical documentation.

## Source audit boundary

The installed dictionary manifest currently declares 120,970 `words` rows, `v2.0.0` engine and storage compatibility, and a generated database file that is **not included in the complete source ZIP**. Older HSK source reading counts are not dictionary row counts. See [database metadata](../dictionary-data/schema/db.md) and [source verification](../project-management/source-accuracy-audit.md). Runtime readiness and browser persistence still require a working application build and evidence.

## Grammar learning ownership

`zipeek-pedagogy::grammar` is the authoritative static Grammar catalogue, slug mapping and region plan for 100 draft courses. `zipeek-exercises::grammar_learning` provides pure step, recall and answer-transition rules without Yew dependencies. `zipeek-web` maps these to a local reducer, route-specific learning components and browser-local progress persistence, while both course engines reuse `GrammarLessonScaffold`. The web layer has no generated Grammar curriculum tables. Two separate pedagogical lesson engines remain; no mastery inference or HSK verification has been added. Rust/WASM/browser acceptance is pending.

## Runtime

`crates/zipeek-web/index.html` supplies the static loading shell and packaged asset references. Trunk builds the Yew/WASM app, which mounts the router and providers. Shared global state includes theme, dictionary readiness, Word Sets and exercise contexts. The app uses the `/#/` hash router for static hosting and `/zipeek/` as the Pages base path.

The database transport loads the generated compressed/verified artifact under `crates/zipeek-web/static/`. DuckDB-WASM opens it read-only and serves typed queries to Browse and Practice. Debug reads typed storage statistics through the database service and, in development builds only, offers an Overview/DuckDB/SQL tabbed view with a 100-row bounded SELECT terminal against the packaged read-only dictionary. Its SQL parser guardrails are not a security sandbox and it never connects to personal OPFS data. The displayed generator storage-compatibility version is manifest metadata, not proof of the WASM engine's default physical file storage version. Native generator and browser engine versions are distinct from manifest storage compatibility metadata.

## Practice and AI data flow

The learner selects Practice options in the four-step setup. `zipeek-exercises::practice_draft` validates choices. Every supported Practice start creates a UUID v7 session in `PracticeSessionStore`, then navigates to `/practice/session/:id`. The only Practice routes are `/practice` and `/practice/session/:id`. The session stores a shared typed configuration map and per-step configuration references; old standalone exercise routes have been intentionally removed. `practice_generated.rs` generates a validated sentence using Settings-owned AI endpoint/model and saved direction; `pages/practice/generated_session.rs` renders its tiles, obtains model evaluation and sends a scored result to the host. `pages/practice/local_session.rs` manages advancement, interruption, restoration and completion cleanup.

The generation request carries a reproducible sampled vocabulary pool of at most ten entries. Model output is a Chinese and English sentence pair; Zipeek constructs sentence tiles and their canonical answer metadata. Other exercise families retain their dedicated runtime boundaries where applicable. The standalone sentence generation route and the free-text translator are not the same as UUID-backed Practice.

## Storage boundaries

- Generated dictionary: immutable/read-only DuckDB asset, manually regenerated and installed.
- Preferences and non-secret AI settings: `zipeek.preferences.v1` localStorage.
- Personal Word Sets/course progress/exclusions: separate localStorage categories with JSON backup and explicit deletion.
- Unfinished UUID Practice: `zipeek.practice.session.v1.*`, removed after finishing.
- Optional API key: tab-memory only.
- Personal OPFS/DuckDB: diagnostics/snapshot/probe and an unverified opt-in Practice-outcome writer; the latter is not yet browser-verified.

## Testing and deployment

The Rust E2E harness owns test servers, isolated Chrome sessions and a Rust AI fixture. The separate [native AI transport](modules/ai-server.md) is not yet a running model service. Test cases cover concrete routes under local root and Pages base paths. `cargo make e2e-full` compiles WASM and test targets before Chrome execution. See [E2E](../archive-history/audits/e2e.md), [data flow](modules/data-flow.md), [DB](../dictionary-data/schema/db.md) and [current acceptance](../project-management/current.md).

### Practice setup boundaries

`pages/practice/setup/page.rs` coordinates the setup reducer and page panels. Its sibling `validation.rs` computes launch eligibility without UI state changes, `launch.rs` owns route and session launch effects, `profiles.rs` owns the Yew profile controls while delegating persistence to `services/practice_profiles.rs`, and `session_recovery.rs` renders the existing-session recovery screen. The authoritative duplicate-session check remains in `PracticeSessionStore::create()`. This source-level refactor still requires WASM compilation and Chrome journey acceptance.

## Component and module ownership

Cloud Match has a feature entry point at `crates/zipeek-web/src/pages/learning/games/cloud_match/mod.rs`; its private HUD and vocabulary-loading helpers are now `games/cloud_match/game/hud.rs` and `games/cloud_match/vocabulary.rs`. The `games/mod.rs` facade exports `CloudMatchPage`, not those implementation helpers. The vocabulary helper still calls the shared game-pair mapper in `games/pairs.rs`; data loading, reducer ownership, and runtime behavior have not deliberately changed.

Cloud Match E2E probes reside in `crates/zipeek-e2e/tests/cloud_match_scripts.rs` and are used by `game_interactions.rs`. The JavaScript behavior probes have standalone Node regression checks in `scripts/cloud-e2e-probes.test.cjs`. The existing `tests/suites` modules remain shared through multiple integration-test parents and have not been migrated into the library, since that requires Rust target-level compilation. Practice-session schema validation is now implemented in `services/practice_session_store/validation.rs`, with the session model, transitions, and browser storage adapter still in `services/practice_session_store.rs`. The generation transition methods share a finished-step guard. The extraction has source-contract coverage but still requires Rust compilation plus serialized-schema and revision/recovery acceptance before separating persistence.

## pedagogical domain foundation

The newly approved `zipeek-pedagogy` crate houses a source-only, pure Rust recognition acquisition projection for future Dragon Progression. It currently has no dependencies or runtime integrations. Its 10×7 rules are documented in [pedagogy](../dictionary-data/schema/pedagogy.md). The existing the historical source multi-skill learning engine remains in `zipeek-exercises`; no authoritative personal DuckDB, `/stats` route or browser worker has been activated by this change. The domain and browser worker execution host are intentionally separate boundaries.

## pedagogy analysis coordinator (WIP)

`zipeek-pedagogy::worker` now provides a platform-neutral job state machine. Requests carry a stable ID and an assessment as-of UTC timestamp; duplicate IDs with different payloads are rejected. The coordinator reads committed recognition history through the existing `EvidenceJournal` boundary and publishes only successful `RecognitionProjection` calculations. Retry attempts are capped at three, with due-time scheduling and a visible failure state; explicit retry is required after exhaustion. Disabling processing pauses queued work.

This is **not** the browser Web Worker implementation. A separate WASM worker host must initialize the Rust runtime in a worker context, establish typed messaging, coordinate with the existing DuckDB-WASM worker, and recover a durable queue. There is no new database schema, runtime dependency, automatic evidence collection or UI attachment in the source reviewed for this contract.

## Statistics dashboard boundary (WIP)

The `/stats` route is a dictionary-independent Yew page with Overview, Hanzi, Skills, History and Insights sections. It currently renders an explicit unassessed state and a link to personal storage Settings. It does not read personal DuckDB, initialize the Rust/WASM worker, infer recognition proficiency from game outcomes, or publish acquisition awards. Future authoritative projections require verified committed evidence, host-side consent, single-tab ownership and the existing learning-domain acceptance gates before attaching live data.

## Rust browser worker monitor boundary

`zipeek-pedagogy::monitor` is the platform-neutral read-only health-request state machine; it verifies version, correlation, expiry, health fields and shutdown without using browser APIs. `zipeek-browser::pedagogy_worker` owns the temporary module Worker, event listeners, timeout and termination. Yew Settings calls the Rust browser adapter directly. `static/pedagogy-worker/worker.mjs` is only the WASM loader, buffering requests until initialization and returning correlated startup errors. The separate Rust/WASM worker binary owns the permanent `DedicatedWorkerGlobalScope` message listener, request decoding, single disabled-by-default `WorkerEndpoint`, and response serialization; the browser host remains a read-only diagnostic client. The separate worker is built cross-platform via `zipeek-cli build pedagogy-worker` (exact wasm-bindgen 0.2.130, locked release build); Pages CI runs this before Trunk copies `static/`. The monitor exposes only `get_health` and is not the evidence analysis coordinator. The main application's worker status button does not consent to tracking, connect to OPFS, persist jobs, or award learning progress. Real browser and WASM acceptance remain pending.

## typed worker binary and first-run diagnostics

`zipeek-web` now declares `[[bin]] pedagogy_worker` with an explicit `src/bin/pedagogy_worker/mod.rs` path. `browser.rs` is the only module depending on browser globals; `runtime.rs` validates and dispatches the Rust endpoint; `wire.rs` holds Serde v1 envelopes and stable health/result/error codes; `error.rs` defines typed errors; `evidence.rs` converts recognition DTOs without treating caller-supplied `snapshot_verified` as a proof of durability. The browser JS loader is unchanged. Rust worker still starts disabled. Do not activate analysis based on first-run preferences alone.

The first-run landing form chooses one of the six existing theme presets and stores an inert, default-off pedagogical opt-in preference. Settings permits changing/revoking that preference. Neither UI writes recognition evidence, enables the worker nor alters OPFS ownership. The debug-only dictionary SQL terminal is bounded and read-only, with a tabular result and CSV export of the displayed (at most 100) rows. A fresh WASM database storage-version probe is **not implemented**: the loaded dictionary's manifest compatibility value and engine `version()` are not equivalent to independently reading a newly created database file's physical storage format.

## Vocabulary curriculum ownership

`zipeek-pedagogy::curriculum` now owns the static 100-region Vocabulary catalogue and typed region identity, domain and membership fields. The existing `zipeek-web` course route consumes it through the workspace dependency already present. `zipeek-exercises` continues to own exercises, not region metadata; the dictionary lookup and personal Word Set provider remain in the web application. This is a code ownership change, **not** completed thematic or HSK coverage. A pinned-source audit script reports gaps; the future fully authored curriculum needs entry-level identities and semantic review before coverage claims. No dependencies, stored browser format, or data generator inputs changed.

## exhaustive HSK Vocabulary source membership

`zipeek-pedagogy::curriculum::VOCABULARY_REGIONS` is a deterministic generated Rust catalogue holding 100 distinct region definitions, 11,494 pinned HSK source indices, 12,188 distinct (Hanzi, numeric-pinyin) keys, unique simplified spellings and per-region source HSK counts. The pinned fingerprint and the last recorded assignment audit are retained in `docs/learning-practice/VOCABULARY_COVERAGE.json`; the former Python generators were removed previously and require a native replacement before source changes; machine-classification exceptions are recorded in `docs/learning-practice/VOCABULARY_REVIEW_QUEUE.csv`. Source coverage and semantic editorial acceptance are separate.

`zipeek-web` resolves each region through batched parameterized `Database::words_by_simplified` queries, validates complete unique dictionary IDs and exposes up to 200 words per personal Word Set. The browser Word Sets limit is 250; its serialized structure is unchanged. Dictionary acquisition now matches both simplified spelling and numeric pinyin, so distinct source readings remain independently acquirable. Dictionary runtime matching still needs live verification against the generated bundle. No new dependencies or generated dictionary bundle were introduced.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../archive-history/audits/maintained-source-snapshots/architecture-architecture.md). Dated implementation claims in that snapshot are not current build or browser verification.
