# R570 WIP - Statistics async sections and replaceable projections (2026-11-18)

- Replaced direct fixture reads in the Statistics Overview, Vocabulary, Grammar, Skills, History, Insights and metric cards with typed, owned projections loaded through an asynchronous `StatisticsSource` implementation.
- Added a separate data reducer and scoped context for per-tab loading, immutable successful results, failure/retry, cached successful sections and generation-checked completion and typed projection mismatch rejection. Abandoning a view invalidates its pending request; closing preview resets the cache.
- Retained the existing UI reducer for filtering, sorting, pagination and bulk selection. Vocabulary selection now uses the record identifier supplied by the section projection rather than the rendered Hanzi label. The two-second initial demo preview is unchanged; per-tab source loading does not add artificial waits.
- Updated the Statistics E2E journey to wait for real tab-specific UI markers. Added source/reducer unit tests for all seven projection kinds, stale completions, retry, reset and successful empty sections.
- Updated the normative Statistics architecture and product docs, current work, tasks and release history.

**Boundary:** All projections still originate from fictional offline fixtures; no personal records, database schema, dependencies or production evidence rules changed. Current explorer filtering/pagination remains client-side over bounded loaded projections. A server-side query/page contract is an explicitly outstanding prerequisite for high-volume API integration. No real Practice/Word Set handoff was added.

**Verification:** Static source checks and ZIP integrity only. Rust/WASM compilation, automated unit tests and browser E2E were not run because a Rust toolchain and browser are unavailable here. R570 is an unverified WIP candidate.

# R569 WIP - Statistics History and interaction design (2026-11-18)

- Built a deterministic 72-session History fixture representing all sixteen current registered Practice exercise types and eight representative game activities, with canonical exercise IDs, category, dates, focuses and scored/unscored distinctions. Real personal learning history is not read or changed.
- Replaced the generic repeated History timeline with searchable Practice/Games filters, expandable typed session cards, stable eight-item paging and a recent-activity strip. Practice-only exercise types intentionally have no correctness score.
- Added a reusable, theme-integrated and keyboard-operable Yew selection checkbox for the Statistics vocabulary and grammar tables. Preserved semantic accessible names and state.
- Replaced flat fixture index distribution with date-derived five-week Overview activity counts; Chart.js labels now show the sample week boundaries.
- Updated Statistics product/UI standards and E2E selectors, added regression tests for exercise registry coverage, score eligibility, category filtering and reducer pagination resets.

Archive: `zipeek-R569-WIP-statistics-history-and-ui-controls-20261118-225149.zip` (packaging timestamp recorded in final artifact name).

Source and ZIP checks only. Rust/WASM compilation, actual Chrome visualization, E2E and keyboard acceptance were not run here. No new dependencies, schema changes, generated database edits or actual Practice/Word Set handoffs. This is an unverified WIP, not an accepted release.

# R568 WIP - Statistics preview controls and selection safety

- Added Focused, Activity and Detailed Overview card presets with independently toggleable cards, owned by the existing Statistics reducer.
- Added validated vocabulary page sizes 10, 25, 50 and 100; page changes reset the visible page and retain selected IDs.
- Added explicit select-all-filtered actions for vocabulary and grammar, bounded to 250 matching fixture identities with a clear disabled state for larger matches.
- Limited selection-review previews to twelve labels while preserving the true selection count; added regression tests for preset transitions, invalid page sizes and bounded bulk selection.
- Improved Insights navigation: listening and speaking gap cards now open the corresponding filtered vocabulary; review candidates select review-first sorting. Kept History page height stable across the short last page.
- Updated Statistics product and architecture contracts, current tasks and this canonical handoff note.

No personal evidence or production database changes. No dependency changes. Rust/WASM compilation, browser E2E and theme/mobile acceptance are **not run** in this environment. The complete project ZIP is an unverified WIP.

## R567 WIP - Statistics 2.0 explorers and Chart.js bridge (2026-11-18)

Archive: `zipeek-R567-WIP-statistics-explorers-and-chart-reliability-20261118-220836.zip`

- Added searchable, sortable, paginated Hanzi and Words preview with stable cross-page selection and an explicit temporary selection review. No Word Sets mutation and no selected-ID transfer to Practice yet.
- Added a first-class Grammar tab with 16 illustrative grammar patterns, text filtering, pagination and selection review. These are mock course patterns, not authoritative assessed evidence.
- Added five interactive skill-gap views that navigate to matching sample vocabulary. Added a compact expandable session timeline and actionable demo evidence-gap panels.
- Changed Chart.js configuration serialization to a plain-JavaScript-object-compatible serializer, and added a visible fallback activity visualization while Chart.js is unavailable. Existing pinned vendor library remains.
- Consolidated legacy Statistics sections, extended reducer/context state, added pure selection/filter tests and updated the Statistics E2E contract and owner documentation.
- No new dependencies, database format changes, personal evidence writes or real proficiency assertions. Source/ZIP checks only; Cargo/WASM/Chrome acceptance not executed in the packaging environment.

## R566 WIP - Statistics foundation, Overview and lifecycle (2026-11-18)

- Added a typed Yew feature reducer with guarded preview completion, tab/page state and metric configuration, plus route-local Statistics context. No app-global state or personal records introduced.
- Replaced the blank loading spinner with dashboard-shaped skeletons that honor reduced motion. The Demo retains a simulated two-second load; no delay is imposed on future real data.
- Replaced the default Overview table with a Chart.js activity bar plot (through the existing Rust interop) and an accessible data table plus four short review candidates linking to the full vocabulary tab.
- Kept incomplete pages visually stable in the full vocabulary and history tables; repaired zero-result pagination ranges. No invented table records are added.
- Preserved demo-only fixtures, pinned dependencies, data integrity and no proficiency inference. Work on selection, grammar, full settings and high-volume fixtures remains open.
- ZIP integrity verified. Rust/WASM compilation, Chrome E2E and visual review not run in this environment.

## R565 WIP - Statistics 2.0 documentation-first design (2026-11-18)

- Archive: `zipeek-R565-WIP-statistics-dashboard-design-specification-20261118-212849.zip`. Documentation-only source checkpoint, not an accepted software release.
- Rewrote the maintained Statistics product contract with seven sections, actions, stable pagination, evidence semantics, 5,000+ word scalability, grammar, history, skills, insights, settings presets and dashboard-shaped loading.
- Added the Rust/Yew reducer/context, typed identity, demo-source isolation, Chart.js, selection/Practice handoff and data projection architecture plan.
- Added an explicit Statistics design acceptance matrix with empty/medium/large fixture families, keyboard, theme, E2E, performance and privacy gates.
- Updated docs index/navigation, UI architecture, current work and the task list. No application code, dependencies, schemas or generated databases changed.
- Verification: ZIP integrity passed; the generated Docs index is synchronized, and all new/changed local documentation links resolve. The Node Docs validator still reports three pre-existing obsolete `services/ai/config.js` references (including two archive/audit paths) and one frozen-history warning; those were not altered in this scope. Rust/WASM and browser acceptance were not run for this documentation-only change.

## R564 WIP - Statistics design refinement (2026-11-18)

- Simplified dashboard presentation: one Demo provenance indicator, no repeated assessment warnings or duplicate headings.
- Added a two-second simulated preview loading state with cleanup, preserving the sample-only and zero-persistence boundary.
- Compact Overview with four table rows per page, expanded word table with a separate Writing observation column.
- Added Writing to illustrative Skills coverage, with counts of observed/review/missing records rather than mastery estimates.
- Replaced native Stats settings checkboxes with theme-aware accessible switch buttons.
- Updated Stats E2E contracts and owning Statistics product specification.
- ZIP integrity and static source review only; Rust/WASM compilation and browser acceptance remain unverified.

## R563 WIP - Statistics preview depth and Grammar mosaic navigation (2026-11-18)

- Moved Statistics after Browse with a distinct dashboard icon.
- Expanded strictly in-memory, unassessed fixture data to 30 words and 20 fictional sessions; added paginated evidence and history tables.
- Added tab icons, evidence coverage summaries, sample insights, and a Settings tab for temporary overview card selection. Shifted metric explanations into info controls.
- Removed redundant Grammar course and unfinished-assessment copy. Added previous/next example controls, curated Pinyin underneath Hanzi, and an example counter.
- Preserved the sample-only statistics boundary and did not connect or modify real personal learning records.
- Source review and ZIP integrity only. Rust/WASM compilation and browser acceptance not run in this environment.

# R562 WIP: Province Quest loading icon compile fix

Archive: zipeek-R562-WIP-province-quest-loading-icon-compile-fix-20261118-165500.zip

- Fix the R561 WASM build failure in Province Quest setup: `yew_icons` 0.11 does not provide `IconData::LUCIDE_LOADER_CIRCLE`.
- Use the already established `LUCIDE_REFRESH_CW` loading spinner with reduced-motion support; preserve Start/Loading button behavior and all existing dependencies.
- Source and archive checks passed; Cargo/WASM compile and browser acceptance require verification on Windows.

# R561 WIP: Game controls and Statistics preview

Archive: zipeek-R561-WIP-game-controls-and-stats-preview-20261118-163149.zip

- Provide a visible Exit action on Province Quest setup and gameplay, and pair Start with the Lucide Play icon. The immersive shell remains unchanged.
- Use concise Random word actions: a Dice icon plus Surprise me, and Dictionary rather than Open in Dictionary.
- Introduce a deterministic, explicit sample-data Statistics preview with tabs for Overview, Hanzi and words, Skills, History and Insights, plus a per-word evidence table. Preview data is not persisted, never counted as proficiency and cannot be mistaken for user progress.
- Add a compact expandable data categories table in Learning Data settings; no personal storage format or recording behavior changed.
- Update the Stats E2E contract and add fixture invariants. No new dependencies.
- Static source/archive integrity checked; Rust/WASM compilation and browser visual acceptance not run in the packaging environment.

## R560 WIP - Province Quest region component boundaries (2026-11-18)

- Extract per-region SVG presentation, semantic visual-state mapping, accessibility labels, and keyboard/mouse selection callbacks into a focused Yew `MapRegion` component.
- Retain viewport gestures and drag-click suppression in the parent; guard suppression before dispatching pointer selection and leave keyboard selection independent.
- Share the existing `Rc<GeoDataset>` and region index with children instead of cloning complete geometry, and index region identifiers once per map render.
- Use named intermediate `MapPoint` values and guarded `unchecked_into::<Element>()` for known SVG event targets.
- Add pure Rust region visual-state tests; runtime Cargo/WASM/browser checks remain outstanding.

## R559 WIP - Province Quest map interaction refinement

- Account for SVG letterboxing in map panning and wheel-zoom anchoring.
- Normalize mouse-wheel delta units and bound the zoom increment to reduce device-dependent jumps.
- Defer pointer capture until a drag is established so clicking a region remains a native SVG click.
- Preserve cumulative drag distance at threshold crossing; add focused pure-Rust regression coverage.
- Browser acceptance and Rust/WASM compilation remain unverified in the packaging environment.

## R558 WIP - Province Quest map visibility and game controls (2026-11-18)

- Corrected the Hard-mode map regression: render an initial national overview rather than hiding the map until a province is chosen. Selecting a mapped province drills down without scoring an answer.
- Excluded administrative regions without prefecture geometry from the Hard-mode province picker, preventing a blank map when choosing Macau or another region with no mapped divisions.
- Replaced text-only Exit and Settings gameplay controls with accessible Lucide buttons.
- Added regression coverage for the filtered region picker and E2E overview visibility.
- ZIP integrity and source-level checks only; Rust/WASM compilation, unit tests and Chrome acceptance not run.

## R557 WIP - Province Quest feature state and map interaction refactor

- Introduce a feature-level Yew reducer coordinating setup, loading, load failures, active province, and gameplay, while retaining `zipeek-exercises::province_quest::Session` as the scoring authority.
- Guard asynchronous map/deck initialization with request generations, rejecting stale completions after leaving setup or starting a different request.
- Move map view-box math, bounded zoom and pan, and pointer gesture distance tracking into typed Rust structures with focused tests.
- Prevent unrelated pointer events from ending a drag, use cumulative movement to distinguish click from drag, and reset map gestures when datasets or parent regions change.
- Keep the immersive shell, theme-derived map palette, keyboard map selection, and existing geography validation unchanged.
- Offline source and ZIP structure checks only; Cargo, WASM compilation, browser gesture tests, and performance profiling still require Windows execution.

## R556 WIP - Province Quest immersive presentation and map controls

- Use the immersive game shell for Province Quest, removing the regular navigation drawer, top bar, and footer during the game.
- Provide an explicit Exit control in the game header, plus Settings.
- Add mouse wheel zoom, pointer capture for pan, and theme-derived answer colors while retaining keyboard and zoom controls.
- Reduce redundant map copy and relocate geography provenance to Credits.
- Improve map sizing in the viewport and update the choice-round E2E selector.
- Static source and archive checks completed; Rust/WASM build and browser gameplay were not executed in the packaging environment.

## R555 WIP - DB generator bulk insert compile fix

- Fix the Windows `cargo make install` failure in `zipeek-db-gen` caused by attempting `join` on `Vec<PartOfSpeech>`.
- Use one canonical `PartOfSpeech::as_str()` serializer for both the 256-row bulk insert and single-row insert paths, preserving existing comma-separated database values and empty-list handling.
- Add unit coverage for empty, single, and multiple part-of-speech values.
- Verification: source assertions and ZIP integrity checked. Cargo formatting, generator compilation, database generation, and full install were not run because Cargo is unavailable in this environment.
- Next Windows gate: `cargo test --locked -p zipeek-db-gen` followed by `cargo make install`.

## R554 WIP - recognition candidate rendering compile fixes

- Replace unsupported `IconData::LUCIDE_BOOKMARK_CHECK` with existing `LUCIDE_BOOKMARK` for saved Word Set actions. Keep `LUCIDE_BOOKMARK_PLUS` for unsaved entries.
- Compute the Yew candidate-row `key` before moving `RecognitionHit` into its component property, avoiding E0382 without cloning the full hit.
- Reported Windows WASM build failures E0599 and E0382 addressed in source. Archive integrity checked. Cargo/WASM compilation and browser acceptance not run in this environment.

## R553 WIP - full local install task and Province Quest geography

- `cargo make install` now runs the complete explicit local asset preparation sequence, including the dictionary, geography and pedagogy Worker, followed by deployment preflight checks. `cargo make serve` remains unchanged and fast for iterative frontend work.
- Added `geo-fetch`, `geo-audit`, `geo-build` and `geo-install` tasks using the existing pinned AreaCity CLI and local asset destination. Source validation and geography audit precede publishing the map JSON.
- Documented the cold-setup prerequisites and the restriction that AreaCity-derived geometry is local-development-only pending upstream redistribution-rights review.
- Added a no-dependency regression test for task wiring.
- Verification: task-graph and ZIP integrity tests; cargo-make execution, downloading geographic source data, native DB generation and browser acceptance not run in this environment.

## R552 WIP: Recognition response and compact suggestions (20261118 UTC)

- Align the recognition workspace and suggestions as equal-height grid peers on desktop, with internally scrollable candidate rows and normal mobile document flow.
- Replace oversized candidate cards with compact single-row entries and accessible icon actions; keep verified dictionary data and Word Set commands.
- Tighten the vision system prompt to an explicit bounded candidate JSON contract, including the empty-candidate case and rejecting instructions embedded in images.
- Add an optional validated output-token budget to the shared AI client and apply a 1,024-token budget only to recognition; preserve other client request behavior.
- Recognize output truncation in non-streaming completions as well as streaming completions.
- Remove duplicated icon imports and ARIA attributes and add request serialization regression coverage.
- Source/archive verification performed; Rust/WASM compilation and browser acceptance not available in this environment.

## R551 WIP: Recognition icons and loading feedback (20261118-003325 UTC)

- Replace the wordy Upload image button with an image icon and concise Upload label; retain accessible name and focus styles.
- Replace the persistent Draw here text with a subtle decorative pencil icon shown only while the workspace is empty.
- Use a scan icon and Recognize label for explicit AI submission, with an animated and reduced-motion-aware loading indicator during recognition.
- Show an accessible loading indicator in the suggestions header instead of duplicated Recognizing... text.
- Update the recognition E2E submit selector to the stable data-recognition-submit contract.
- Archive integrity checked; Rust/WASM compilation and browser acceptance not run in this environment.

## R550 WIP: Responsive split recognition workspace

Archive: `zipeek-R550-WIP-recognition-split-workspace-and-results-20261118-002406.zip`. Unverified WIP checkpoint.

- Move recognition suggestions beside the writing workspace at desktop widths, retaining the canvas while reviewing candidates. On narrower screens the panels stack, and completed results scroll into view.
- Keep candidate list scrolling bounded on desktop, without nested mobile scrolling or persistent explanatory empty-state copy.
- Preserve typed reducer transitions, theme-aware rendering, explicit AI submission, local dictionary verification and Word Set controls.
- Update recognition E2E selectors to match existing accessible icon-only Clear action and assert the new layout landmarks.
- Update the product specification, task register and current work. No dependencies, schema, AI prompt, image retention or browser interop changed.
- Archive structure and integrity checked. Rust formatting, WASM compilation, Chrome E2E and visual acceptance not run in this environment.

## R549 WIP: Recognition UI copy reduction and interface policy

- Remove the redundant recognition status badge, persistent canvas-help footer, page subtitle, privacy chip, secondary action-panel heading, repeated empty-input instructions and duplicate result disclaimers.
- Use accessible Lucide icon-only controls for Undo and Clear with keyboard focus and explanatory titles, while retaining explicit Upload and Recognize actions in a single workspace panel.
- Keep a minimal empty canvas prompt, model-configuration prerequisite, meaningful errors and active recognition state. Remove the unused feedback/guidance field from the recognition reducer without changing its transition protections.
- Document progressive disclosure and visible-copy standards in `interface-and-platform-policy.md`, including accessibility and timely consent exceptions.
- Update current work and acceptance tasks. No dependency, endpoint, database, image upload or route changes.
- ZIP integrity and source-level consistency checked. Cargo, WASM compilation, E2E and visual browser acceptance remain unverified.

## R548 WIP: Explicit import style for Rust application code

- Document the approved general preference for explicit module-scope imports,
  with qualified paths retained for clarity and disambiguation.
- Import `RecognitionWorkspace`, `RecognitionWorkspaceCallback`, and
  `RecognitionWorkspaceEvent` together at the recognition page module boundary.
- Remove the local `use` in the workspace mount effect and shorten the workspace
  handle annotation and constructor call.
- No runtime, routing, dependency, schema, or static asset changes.
- Verified source substitutions and full ZIP integrity; Cargo formatting,
  WASM compilation and browser acceptance remain unverified.

## R547 WIP: Recognition feature Yew compilation corrections

- Repair the recognition result card's Word Set action by evaluating callback setup in a Rust expression block and returning valid `html!` markup from both branches.
- Expose candidate parsing to its parent recognition feature module with `pub(super)` instead of a private sibling-only function.
- Explicitly type the parsed candidate collection as `Vec<String>` at the async dictionary lookup boundary.
- Address the ten `zipeek-web` diagnostics reported from the R546 WASM build, including cascading unsized string errors. No dependencies, persistent storage, routes, or database assets changed.
- Static source and archive checks completed. Rust/WASM compilation and browser E2E are not available in this environment and remain unverified.

## R546 WIP: Typed recognition workspace events and WASM build corrections

- Replace browser canvas DOM attributes and custom-event roundtrips with a typed `RecognitionWorkspaceEvent` callback passed at mount. Yew receives input state and typed operation failures directly.
- Keep the workspace in `zipeek-browser` and the reducer in `zipeek-web`; use explicit typed state conversion at the crate boundary.
- Correct WASM compiler errors from the R545 recognition workspace: web `File.size()` representation, `JsCast` error conversions, optional clipboard file lists, and deprecated canvas-style setters.
- Surface painting failures through the callback rather than silently suppressing them. Invalidate in-flight recognition when a workspace error occurs, and prevent stale image upload errors after another edit or Clear.
- Ignore secondary pointer-down events while one drawing pointer owns the active stroke.
- No dependency, database, URL, or persisted-data format changes.
- Archive validation only: Rust/WASM compilation, browser acceptance, and full regression tests remain pending on a Rust toolchain.

## R545 WIP: Hanzi recognition feature-state and component refactor

- Replace the monolithic recognition page with reducer-backed state transitions for input changes, request lifecycle, errors, and verified results.
- Split workspace, controls, candidate results, and candidate parsing into focused feature modules.
- Keep workspace browser resources in `zipeek-browser`; feature state and orchestration remain in `zipeek-web`.
- Prevent stale AI responses from replacing results after the canvas changes; add native reducer tests.
- The current canvas-to-Yew event attributes are retained until a separately approved typed event interface can replace them.
- Validation is static only: Rust/WASM compilation and Chrome acceptance were unavailable in the packaging environment.

# R544 WIP: Typed recognition geometry and event-boundary conventions
Archive: zipeek-R544-WIP-recognition-geometry-and-browser-contracts-20261117-235700.zip

- Use a named `CanvasPoint { x, y }` for responsive canvas coordinate transforms and strokes, rather than `Vec<Vec<(f64, f64)>>`. Add coordinate boundary coverage.
- Confirm recognition paste focus and image element creation use `Document` already injected through the existing `AppEnvironment` (introduced in R543). No new global `web_sys::window()` discovery.
- Retain checked `dyn_ref` casts for generic event listeners because the runtime event subtype is not represented by their `Event` callback type; document when unchecked casting is appropriate.
- Extend the Rust/WASM interop contract with browser handle injection, event-casting boundaries, and named domain geometry.
- Verification: static checks and ZIP integrity. Cargo/Rustfmt and browser tests not run in this environment.

# R543 WIP: Recognition workspace error and browser-environment refactor
Archive: zipeek-R543-WIP-recognition-workspace-typed-errors-and-environment-20261117-235120.zip

- Replace ad hoc `JsValue`/`String` errors in the recognition workspace with the domain-specific `RecognitionWorkspaceError` covering unavailable canvas/theme/root, empty input, invalid image input, and browser API failures with operation context.
- Inject the existing `AppEnvironment.window` and `AppEnvironment.document` into the `RecognitionWorkspace::mount` constructor; stop global `web_sys::window()` lookups in drawing, paste focus, and image replacement.
- Pass the injected document root and window into the minimal MutationObserver JS bridge instead of reaching for global `document`/`window` inside the bridge.
- Preserve recognition behavior, privacy and API contracts; report initialization failures through existing `log::error!` and the user-facing error state rather than silently discarding them.
- Verification: static source checks and full archive ZIP integrity only. Rust/WASM compilation and live browser acceptance not run in this environment.

# R542 WIP: Rust/WASM interoperability architecture standards
Archive: zipeek-R542-WIP-rust-wasm-interop-architecture-standards-20261117-234417.zip

- Documentation-only update defining Rust/WASM-first implementation: Rust owns application behavior, async state, validation, resource ownership and cleanup.
- Codifies `zipeek-browser` ownership for reusable browser APIs, `zipeek-db/src/wasm/` for DuckDB-WASM-specific transport, and `zipeek-web` for Yew presentation.
- Documents minimal `wasm-bindgen(inline_js)` shims, opaque Rust newtypes for vendor handles, the approved Chart.js/DuckDB-WASM vendor exception, and build/test/early-bootstrap exceptions.
- Adds cross-links from the architecture overview, coding standards, and docs index. No code, dependencies, approved decisions, runtime behavior, or source archive layout changed.
- Verification: documentation link checks and ZIP integrity only. Cargo and browser tests are not applicable to this documentation-only handoff. R542 is a documentation WIP, not a major release.

# R541 WIP - Rust/WASM browser services migration

Archive: zipeek-R541-WIP-rust-wasm-browser-services-migration-20261117-234235.zip

- Moves AI URL/model settings and tab-memory API key to `zipeek-browser::ai_configuration` using the existing `zipeek.preferences.v1` local storage document. API keys are never persisted.
- Routes the active Hanzi clipboard action through `zipeek-browser::clipboard`. Deletes the unused JS clipboard service.
- Migrates the active Recognize Hanzi workspace into `zipeek-browser::recognition_workspace`. Rust now owns pointer strokes, uploads, clipboard paste, image validation, theme repaint, input events, capture, undo, clear, cancellation of stale image decodes and listener cleanup.
- The single inline JavaScript browser adapter is limited to theme observation with MutationObserver and matchMedia; it has no application data or drawing state.
- Adds native unit coverage for workspace states, supported image limits and scaled canvas coordinates.
- Removes the superseded AI settings and canvas JavaScript modules and canvas-specific Node tests.
- Preserves the pinned Chart.js vendor module, scripts and dependency/decision D35. Adds no crate dependencies; enables necessary existing `web-sys` browser interfaces.
- Updates architecture and current-task documentation.

Verification: static source review and complete archive ZIP integrity check. Cargo/rustfmt, native tests, wasm32 compilation, Trunk build and actual browser behavior could not be executed. R541 remains an unverified WIP.

Required checks: `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-browser`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo make prepare-summary`, `cargo make verify-assets`, and the Hanzi recognition browser journey including image upload, paste, Undo, Clear, theme switching and navigation cleanup.

---

# R540 WIP: Chart.js Rust/WASM browser boundary

Date: 2026-11-17

- Move asynchronous Chart.js module loading and chart instance lifecycle to `zipeek-browser::charts`, following the established browser interop crate boundary.
- Use `wasm-bindgen(inline_js)` solely for JavaScript `import(url)`; call the pinned vendor bundle's exported `createChart` from Rust through `js-sys` and `Reflect`.
- Resolve the vendor URL relative to the document base URI to support root development and `/zipeek/` deployment.
- Keep summary/course progress chart configurations in `zipeek-web`, with the opaque Chart.js newtype and `Drop` cleanup owned by `zipeek-browser`.
- Remove the redundant `chart_bridge.js` file. Preserve the pinned Chart.js/esbuild bundle, licensing, preparation, and D35 decision. No dependency or generated vendor changes.
- Chart integration has no live Yew callers in this source baseline. No claim of runtime chart rendering acceptance.
- Verification: static source inspection and ZIP integrity only. Cargo/rustfmt, WASM build, browser chart import, and root/Pages routing checks not run in this environment.
- Windows acceptance: `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo make verify-assets`, and confirm bundled Chart.js loading under `/` and `/zipeek/` in Chrome.

---

# R535 WIP: Native Mandarin speech output correctness

Date: 2026-11-17

- Correct eSpeak NG stdout WAV placeholder lengths before HTTP delivery, verifying PCM16 mono audio, sample-rate/byte-rate consistency, bounded payload and nonempty samples.
- Reject empty or invalid Whisper transcripts instead of returning HTTP success with no recognized text.
- Apply loopback Host, browser-Origin rejection and no-store responses consistently to health and capability endpoints.
- Add regression tests for real stdout-style placeholder WAV headers, malformed output, invalid transcription text and local-only status endpoints.
- No dependency or engine changes, model additions, database changes or browser-provider integration. The D56 trusted speech bridge remains unapproved.
- Verification: static source review and ZIP integrity only. Cargo/rustfmt, native speech round-trips and performance tests not run in this environment.
- Required Windows checks: `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, `cargo test --locked -p zipeek-tts-stt`; then exercise real local eSpeak NG and Whisper engines.

---

# R534 WIP: Native speech protocol and lifecycle hardening

Date: 2026-11-17

- Restrict `/v1/tts` and `/v1/stt` native POST requests to explicit localhost/127.0.0.1 Host values while retaining browser Origin rejection.
- Fail fast if eSpeak exceeds the bounded output stream, and terminate/reap its process when synthesis errors or times out.
- Add HTTP-handler regression tests for valid loopback hosts, invalid/missing hosts, browser origins, invalid WAV/text, and unavailable models.
- Expand strict WAV parser regression coverage for duplicate data, unsupported formats, oversized chunks, and padded ancillary metadata.
- Keep current browser speech provider unchanged. An authenticated desktop speech bridge needs separate design approval under D56.
- Verification in this environment: source edits and ZIP integrity only. Cargo/rustfmt, native eSpeak/Whisper execution, and browser tests were not run.
- Windows checks: `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, `cargo test --locked -p zipeek-tts-stt`.

---

# R533 WIP: Native Mandarin speech service

Date: 2026-11-17
Archive: `zipeek-R533-WIP-local-mandarin-tts-stt-service-20261117-213024.zip`

- Implemented runnable loopback-only `zipeek-tts-stt` server instead of its no-op main and unused draft module.
- Replaced Serbian `sr`/`sr` behavior with Mandarin `cmn` synthesis and `zh` Whisper transcription.
- Added strict bounded WAV PCM16 parsing, 4 KiB synthesis text limit, 30-second transcription limit, per-engine concurrency limits, failure/status handling and native health/capability endpoints.
- Avoided logging/persisting speech payloads and blocked browser Origin requests pending an approved trusted desktop-bridge design.
- Added crate unit tests and authoritative module documentation. No dependencies, models or binaries were added. Existing browser speech remains active.
- **Not verified**: Cargo formatting, locked compilation/tests, real eSpeak/Whisper model execution, browser integration and runtime performance. These require the Windows environment and licensed resources.
- Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, and `cargo test --locked -p zipeek-tts-stt`.

---

# R532 WIP: Hanzi recognition route and privacy-aware E2E fixtures

Date: 2026-11-17

Archive: `zipeek-R532-WIP-hanzi-recognition-and-e2e-integration-20261117-212525.zip`

## Implementation

- Added `/recognize-hanzi` to the Yew route, global navigation and E2E route registry, with an integrated image/drawing canvas. Users can draw with pointer, touch or stylus, Undo/Clear, upload PNG/JPEG/WebP up to 5 MB, or paste a focused clipboard image. Replacing existing content asks permission.
- A deliberate Recognize action sends the composited image through Zipeek's configured OpenAI-compatible chat client. No image upload occurs on paste/upload/mount. Inputs and results are not persisted; page teardown drops image references and invalidates results from stale requests. Network cancellation requires separate client support.
- Parse up to five deduplicated Hanzi candidate strings from bounded JSON. Verify exact dictionary hits locally, including traditional forms. Display pinyin/meaning only from the dictionary; unverified predictions remain identified. Link to Dictionary, Listen, Write Hanzi, and the active Word Set.
- Extended the AI fixture to accept recognition requests, validate that they contain an image, and redact image data from scenario request artifacts. Added a dedicated E2E journey, accessibility route coverage, parser unit test and Node canvas-state tests.
- No new dependencies, no dictionary schema changes, no modifications to persisted learning data. ISS-001, ISS-021 and ISS-028 remain open pending reproduced root causes and actual browser evidence. Geography asset generation and licensing remain separate prerequisites.

## Verification status

- Passed: `node --test scripts/tests/recognition-workspace.test.cjs` (2 tests); JavaScript module import/syntax check; `node scripts/docs-validate.cjs` (zero maintained-document errors; one existing frozen-history warning); ZIP integrity and source content comparison at packaging.
- Not run: Cargo tests, `cargo fmt`, Rust/WASM compile, Trunk build, Chrome E2E, actual vision model and map/translation service acceptance. Cargo/rustc/rustfmt are unavailable in the execution container.
- The selected vision model's image capability cannot be established by the current settings contract. The UI requires a configured model but may receive an actionable provider error if unsupported.

## Required Windows checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-e2e --lib
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
cargo test --release --locked -p zipeek-e2e --test hanzi_recognition -- --nocapture
cargo test --release --locked -p zipeek-e2e --test accessibility_matrix -- --nocapture
```

Also test a real image-capable model using the Settings endpoint and verify `/zipeek/` deployment behavior, images, keyboard/touch input, Word Sets, and generated geography assets before release acceptance.

---

## R531 WIP: E2E dictionary preflight and route readiness

- Validate the compressed DuckDB entry from the installed dictionary manifest before launching Trunk. Reject missing, malformed, empty or size-mismatched assets with actionable setup instructions. After Trunk, verify that the distribution includes the expected compressed file.
- Make E2E deep-link navigation wait for the route-specific readiness marker rather than requiring the app shell on standalone game, lesson and error screens. Retain the shell requirement for unregistered routes.
- Use the Time Weaver setup marker for its initial route, retaining the gameplay marker for gameplay-specific tests.
- Add preflight regression coverage for missing, corrupt and unsafe asset paths.
- The uploaded 2026-11-17 browser evidence still reports Province Quest readiness failure; browser-level root-cause verification remains outstanding.
- Not verified in this environment: Rust compilation, Trunk builds, live Chrome E2E, database provisioning and asset hashes. Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-e2e --lib`, and `cargo test --release --locked -p zipeek-e2e --test accessibility_matrix -- --nocapture` after installing a generated dictionary.

# R530 WIP: dictionary-only installation

Archive: `zipeek-R530-WIP-db-install-preserves-static-assets-20261117-202702.zip`

- Corrected `cargo make db-install` to preserve the existing `crates/zipeek-web/static` tree and replace only verified dictionary-owned files. Unrelated practice, Courses, brand, Docs and user-added assets remain untouched.
- Superseded timestamped `.duckdb` and `.duckdb.gz` files are moved into timestamped backups, not deleted permanently. Updated manifest, diagnostics, attribution, inputs and dictionary notices are installed individually; the HTML dictionary preload is updated after the files are in place.
- Kept existing file and HTML backups and added best-effort rollback if file publication fails. Refused unexpected dictionary manifest paths and symlinked replacement targets.
- Added regression tests checking `practice/white-tiger.webp`, course/brand art, unrelated files, old database backup and rejection of an injected practice asset.
- Static review and ZIP integrity verified. Rust compilation, formatting and tests could not run here because Cargo/rustfmt are unavailable. This is an unverified WIP handoff.

---

# R529 WIP: parameterized dictionary bulk insertion

- Replaced row-at-a-time main-table insertion with 256-row parameterized multi-value inserts for dictionary words, source senses, and character readings, without a dependency upgrade.
- Retained the established transaction, rollback, enrichment, and artifact logical verification pipeline. Small word inserts still use individual statements for exact error attribution.
- Added a regression test inserting 513 character readings across three batch boundaries.
- Kept `BLOCK_SIZE 262144` (256 KiB), the maximum DuckDB permits; 512 KiB or 1 MiB would be invalid. This request was reviewed against current upstream documentation.
- No local Rust compiler available. The SQL generation, native import tests, regenerated database equivalence and performance gains require local verification. This is an unverified WIP handoff.

---

# R528 WIP: dictionary import diagnostics

- Downgraded expected `GlossPipeReplacement` diagnostics from WARN to DEBUG while retaining each record in the structured diagnostics artifact.
- Added an explicit `gloss_pipe_replacements` count to the existing HSK import summary and kept unexpected diagnostic kinds at WARN.
- Expanded the diagnostic retention unit test to cover the normalized-gloss case.
- Reviewed generator performance: the output already uses `BLOCK_SIZE 262144`; per-row prepared statements and profiling of import, enrichment, FTS, and verification are the next performance investigation candidates. No database format, batching, or dependency changes were made.
- Verification: static source review and ZIP integrity only; Rust checks and performance benchmarks were not run because Cargo is unavailable here.

---

# Asset-integrity technical-debt reduction

This unverified source handoff reduces TD-009, without closing its acceptance gate.

- Added `scripts/asset-integrity-audit.cjs` with source and deploy policies for checked-in manifest files, SHA-256 hashes, dictionary preloads, Courses/Practice artwork, complete pedagogy Worker output and Province Quest geography.
- Added eight Node fixture tests at `scripts/tests/asset-integrity-audit.test.cjs` and Cargo Make entry points for source, deploy and regression-test checks.
- Corrected the Rust generated-asset preflight so an untouched source checkout does not require absent generated Worker output; deploy mode still requires complete bindings. Added two Rust regression tests for this boundary.
- Updated the technical-debt ledger, current work, open tasks, deployment guidance and generated Docs search index.

Verification performed here: 13 Node tests passed across the new asset audit and existing E2E evidence tests. Source integrity audit passed with expected missing-generated-asset notices. Deploy integrity audit intentionally failed because the source archive lacks compressed dictionary, Worker output and Province Quest geography. Docs validator passed with zero errors and one existing historical warning. Rust, WASM and Chrome were not run because Cargo is unavailable in this environment. No dependency versions were changed.

Windows verification: `cargo make asset-integrity-tests`, `cargo make preflight-source`, generate and install approved assets, `cargo make preflight-deploy`, `cargo test --locked -p zipeek-cli asset_preflight`, `cargo check --workspace --locked`, WASM checks and recorded Chrome E2E.

---

## Documentation source accuracy audit

- Compared authoritative architecture, data-generation, dictionary manifest, browser startup and cross-tab descriptions against the checked-in Rust source, package manifests and generated metadata.
- Corrected outdated database counts, storage policy and artifact filenames in maintained architecture, data, engineering, CI and E2E documentation. The source manifest declares 120,970 words, storage v2.0.0 and the latest timestamped dictionary artifact; this source ZIP does not include the generated database binary.
- Documented existing web-shell behavior distinctly from approved but unimplemented whole-app database blocking, original-tab ownership and dragon header requirements.
- Added `docs/project-management/source-accuracy-audit.md` with referenced source evidence and explicit runtime verification gates.
- Extended `scripts/docs-validate.cjs` to compare the database summary against the checked-in manifest and generator policy.
- Regenerated the Docs index and ran the Node documentation validator: zero errors and one existing frozen-history link warning.
- No Cargo or WASM compilation, native database reopen, Chrome E2E test or actual dictionary hash verification was performed. No dependencies or Rust application code were changed.

# Docs reader regression and source verification

- Corrected the Docs browser deep-link test so it targets the current tasks article and an existing heading, rather than a deleted route.
- Added browser journeys covering all twelve physical Docs sections, an issue screenshot decoding check, and documentation search-to-article navigation.
- Fixed Docs reader curated article ordering after the move to lowercase kebab-case filenames and added a focused Rust regression assertion.
- Updated the Markdown renderer's route tests to use actual maintained document paths.
- Extended the Node Docs validator to check Docs E2E article routes, heading targets and twelve-section test sample paths. Updated the requirements-to-tests matrix and current task status.
- Source validation executed: `node scripts/docs-validate.cjs` passed with zero errors and one pre-existing frozen-history warning. Chromium is installed but Cargo/Rust and a compiled local Zipeek app are absent, so neither Rust compilation nor genuine browser E2E was run. This source archive is unverified for runtime behavior.
- No dependencies changed and no user data or application learning behavior was modified.

---

# Documentation reconciliation handoff

This documentation-focused source handoff consolidates the authoritative Product, UI architecture, Practice Setup, Write Hanzi, Courses mosaic, Cloud Match, Multiple Choice and appearance pages. Prior implementation narratives are preserved under historical source snapshots with rebased links. New maintained pages specify shared exercise component ownership, the long dragon-shaped evidence indicator, cancellable nonblocking vocabulary eligibility calculations, credits and license review, and a developer/Codex E2E evidence review workflow. The documentation index and acceptance criteria were updated.

This changes documentation and generated index content only. It does not implement the newly specified application behavior or alter dependencies. Rust/WASM compilation, browser acceptance, strict MP4 playback and human visual review remain unverified. Retained historical documents may contain obsolete behavior and are not authoritative.

---

# Release notes

## Documentation cleanup: retired scratchpad and visual concepts

- Removed the pending-decision DIGEST scratchpad. Remaining D28-D30 architecture review items are retained in active tasks.
- Removed three obsolete UI concept PNGs: course mosaic, Practice session dashboard, and Practice setup. Current specifications and screenshot-driven review take precedence.
- Reconciled the documentation index, visual references, source-reconciliation notes, contributor guide, handoff guide, and Docs reader metadata/tests.
- Rebuilt the Docs search manifest and validated Markdown links and typography: zero maintained-document errors, one preserved historical broken-link warning.
- No dependency updates or application behavior changes. Rust compilation, WASM and Chrome browser verification were not run.

## E2E evidence and documentation reconciliation

- Added per-scenario `agent-feedback.md` with observed failure phase, assertion result, recording state, linked checkpoint evidence and diagnostic locations.
- Added suite `review.md`; extended `scenario.json` with independent assertion, execution and video validation status.
- Enabled named checkpoint screenshots by default, retaining explicit opt-out; added significant Word mosaics checkpoints without claiming new gameplay assertions pass.
- Added FFprobe video-stream validation and strict-mode failure for invalid, absent, empty, truncated or unverified recordings.
- Added focused Rust unit tests for reporting and missing video validation; no new crates or dependency upgrades.
- Reconciled maintained E2E recording documents with actual bounded JPEG-to-FFmpeg MP4 streaming; added a source-based requirements-to-E2E coverage matrix.
- Removed release-numbered active task narratives from CURRENT and TASKS; preserved historical ledgers and frozen completed records.
- Applied the previously requested Province Quest `Option::ok()` compile fix without changing its asset-loading behavior.
- Verified in this environment: Node documentation index and link validator, archive integrity and static files. Rust toolchain and Chrome E2E are unavailable here, so native/WASM builds, unit tests and playable browser recording remain unverified.


## Documentation decision reconciliation

- Incorporated final approved product decisions into maintained specifications without changing application runtime code or dependencies.
- Added Practice outcomes, vocabulary selection, profile JSON portability, AI Hanzi recognition, platform policy, visitation boundaries and corresponding acceptance criteria.
- Trimmed the active DIGEST to unresolved architecture proposals and reconciled the Rust no-comments rule and dependency update approval policy.
- Historic decision records and visual issue evidence remain intact.
- This archive is a documentation handoff only. Source behavior, Windows builds, WASM and Chrome acceptance are not verified by these edits.

# R526 WIP: Documentation modularization and validation (2026-11-17)

Archive: `zipeek-R526-WIP-docs-modularization-and-validation-20261117-003721.zip`

- Moved ten coherent specification sections out of broad UX, Product and UI Architecture documents into topic-specific Markdown pages. All moved text is retained, and the original section headings link to the new owners to preserve existing entry points.
- Updated the documentation index and added links from `docs/README.md` to the maintained topic pages.
- Added `scripts/docs-validate.cjs` with no new package dependency and `cargo make docs-validate` to check relative links, inline screenshot references, long dash typography and Docs manifest freshness.
- Corrected two stale source-link targets in `SOURCE_RECONCILIATION.md` without inventing new current-module claims. One pre-existing link inside a frozen completed record remains a warning.
- Source and data behavior are unchanged. All 28 issue reports and 27 screenshots are retained with open status. The current 12 Docs UI categories are unchanged.
- Verified: validator passes with zero current errors and one historical warning; simulated stale-index failure detected; ZIP CRC and archive root checked.
- Not verified: Rust tests, WASM compilation, Chrome navigation and hosted `/zipeek/` Docs rendering. The user-reported failing build remains an unresolved P0.

# R525 WIP - Issue library and visual evidence (2026-11-17)

Archive: `zipeek-R525-WIP-issue-library-and-visual-evidence-20261117-003500.zip`

- Created an authoritative `docs/issues/` library with 28 uniquely identified open reports, issue indexes, original observations, acceptance criteria and linked screenshots.
- Preserved the R524 issue register as an immutable source snapshot in `docs/issues/ORIGINAL_REPORT_2026-11-17.md`.
- Added Issues as the twelfth Docs navigation section, with category topics and targeted classification tests.
- Included issue screenshots under documentation and served static documentation paths, and regenerated the Docs index.
- Updated current work, task tracking and documentation entry points. No application gameplay, user data or dependencies changed.
- Validation: ZIP CRC and static references checked. Cargo formatting, Rust/WASM tests and Chrome checks not run; previous build failure remains open.

## R524 WIP: Translation actions, map diagnostics and product decisions (2026-11-17)

Archive: `zipeek-R524-WIP-translation-map-diagnostics-and-decisions-20261117-002724.zip`

- Fixed Translate Sentence controls to call the clear/submit callbacks directly rather than dispatching based on the target element's DOM dataset. This removes the observed unknown-action code path.
- Show actionable AI service connection guidance for common browser network errors; translation still requires a configured, reachable model.
- Added Province Quest response classification for HTML fallbacks, empty map assets and invalid JSON. Domain validation remains intact. Generated map JSON was not supplied in the R523 source snapshot and must be installed before this game can run.
- Recorded approved D53-D56 decisions: required AI availability, visibly locked pedagogical course tiles, rolling per-pattern grammar evidence, and dual browser/local speech recognition. These larger behaviors are NOT implemented by this patch.
- Added focused Rust regression assertions for error classification. No third-party dependencies or persisted schema changes.
- Static inspection and ZIP integrity only. Rust formatting, locked tests, WASM build, Chrome acceptance and live AI/map service verification are pending. This is an unverified WIP candidate, not an accepted release.

## R523 WIP: Documentation navigation and issue triage (2026-11-17)

- Reclassified Docs reader entries into 11 deliberate navigation sections using typed metadata, preserving every existing Markdown path and deep link.
- Added curated section and article order, title-first result ranking, and marked internal decision-inbox entries as excluded from normal Docs navigation.
- Added a detailed, explicitly unverified issue register for the user's screenshots and written product observations. No reported gameplay, service, map or onboarding defect is claimed fixed.
- Normalized long dash punctuation to ASCII hyphens in Markdown documentation as requested.
- Regenerated Docs static search manifest to include current Markdown files and new issue register.
- ZIP and source static checks only. No Rust/WASM compiler or live browser testing executed; do not treat this as an accepted release.

# R521 WIP: Write Hanzi unified vocabulary autocomplete

Archive: `zipeek-R521-WIP-write-hanzi-vocabulary-autocomplete-20261116-223928.zip`

## Changes

- One debounced search input supports simplified/traditional Hanzi, English definitions, tone-marked and numbered Pinyin, and joined or spaced Pinyin.
- Added parameterized unified dictionary candidate lookup and deterministic exact/prefix/partial ranking with stable tie-breakers. Word Set filtering retains the current source selection.
- Replaced paged search results with seven compact suggestions initially, expanding by seven on demand; Arrow Up/Down, Enter, Escape and pointer selection supported. After choosing, a compact selected-word summary replaces the list.
- Removed the redundant search input focus ring while retaining the accessible focus border on its enclosing control. Global accessibility styles remain otherwise unchanged.
- Guards against stale asynchronous responses from older search queries or set/source selections. Updated rank and state tests, and the existing E2E search label expectation.
- No new dependencies, no personal data changes and no dictionary generator modifications.

## Verification

Static source inspection and ZIP integrity checks only. Cargo/rustfmt, locked unit tests, WASM compilation and Chrome acceptance have not been run in this environment; R521 is an unverified WIP source candidate.

## Windows checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

In Chrome test English, Hanzi and Pinyin queries, keyboard selection, stale search changes, both word sources, focused input/Tab, empty results, and desktop/mobile at `/` and `/zipeek/`.

---

# R520 WIP: Time Weaver setup controls and preferences

Archive: `zipeek-R520-WIP-time-weaver-setup-and-preferences-20261116-220737.zip`

## Changes

- Redesigned the Time Weaver setup with whole-row accessible Pinyin/audio switches, three visual tile sizes (Compact, Standard, Large), a live sample preview and a compact pre-start summary.
- Added 10, 15 and 20-question round lengths. The deterministic seeded engine still balances the five exercise types evenly; the original seed-only API retains its 15-question default.
- Persisted validated setup options in `zipeek.time_weaver.options.v1` using the existing `AppEnvironment.local_storage` access pattern. Invalid/unavailable storage uses the existing defaults; failed writes are nonfatal.
- Kept current game interactions, score/review flow, speech service, word assets and routes intact. Added domain, serialization and browser selector/persistence assertions. No dependency changes.

## Storage and limitations

Browser-local setup preferences do not contain learning history and are not included in Word Set backups. Clearing site data resets preferences. Existing saved personal data and Practice sessions are unchanged. New configuration may not be readable by older application builds; no backwards-compatibility layer was introduced.

## Verification

Static source/wiring inspection and ZIP integrity only. Cargo formatting, Rust tests, WASM compilation and Chrome acceptance were not available in this environment. R520 is an unverified WIP candidate.

## Required local checks

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo test --locked -p zipeek-exercises -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Verify selected 10/15/20 lengths finish with the corresponding total, each exercise type appears equally often, settings survive reload, invalid/blocked storage is safe, tile preview matches game display, speech failure remains recoverable, and `/` plus `/zipeek/` routes display correctly at mobile/desktop widths.

---

# R519 WIP: Hanzi Crossword variety and browser-local rotation

Archive: `zipeek-R519-WIP-crossword-variety-and-local-history-20261116-220208.zip`

## Changes

- New games keep their fresh session-derived random seed and now compare eight valid deterministic puzzle candidates, preferring unused and older vocabulary without sacrificing the minimum crossword quality floor.
- Added browser-local history capped at 100 distinct Hanzi answer words in least-recently-used order and 10 recent puzzle word-list fingerprints. This is weighted rotation, not a hard exclusion: small pools continue to work.
- History is recorded when a game is started, independently of game completion and learning-history consent, and is not recorded again for a restored or edited session. Existing gameplay, saved crossword session snapshots, Word Sets, and crossword board rendering stay unchanged.
- Reused the existing `AppEnvironment.local_storage` access pattern and checked serialized history for validity and size; malformed/unavailable storage falls back gracefully. No dependencies or server-side data were added.
- Added regression tests for 100-word LRU eviction, 10-game cap, session idempotence, deterministic candidate ranking and invalid data.

## Browser storage and limitations

`zipeek.crosswords.rotation.v1` is a new small site-local key outside existing Word Set backups. Clearing browser site data removes this history. Existing crossword session snapshots retain their current schema. Puzzle diversity depends on the chosen HSK pool and constraints; some words may recur when alternatives cannot form a connected puzzle.

## Verification

Static integration and archive inspection completed. Rust formatting, locked Rust unit tests, WASM compilation and actual browser acceptance were not run in this environment. This is an unverified WIP source candidate.

## Required local checks

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Test repeated New game, results Play again, mid-game refresh, narrow HSK pools, browser storage eviction/error handling and `/zipeek/` routing.

---

# R518 WIP: Word Sets workspace and scrolling

Archive: `zipeek-R518-WIP-word-sets-workspace-and-dictionary-default-20261116-215127.zip`

## Changes

- Replaced the bounded desktop vocabulary scroll container with a natural document-scrolling workspace to eliminate the two-scrollbar problem.
- Exposed saved Word Sets as an immediately visible, keyboard-accessible list with word counts and selection highlighting.
- Added a search field for simplified Hanzi, pinyin and English meanings; search clears when switching Word Sets.
- Simplified set management: New set retains an explicit primary label, rename uses a pencil disclosure, and deletion uses a trash icon with a second confirmation step.
- Moved backup/restore under a labelled Data tools disclosure while retaining the existing underlying import/export behavior.
- Preserved Practice selection, audio, per-word removal, empty-state Browse navigation, Word Set data and ID formats. No dependency changes.
- Made the current set the remembered **Dictionary save destination**. The selected Word Set persists in a separate browser-local preference; reload resolves it against available sets and falls back to the first set if the saved one was deleted. No Practice source auto-selection.
- Updated the E2E Word Set journey for direct-set selection and rename disclosure.

## Default Word Set

The user clarified that Default Word Set means the Word Set that receives Dictionary Add/Remove actions. R518 implements this as the remembered current selection, not as a Practice source preference. The existing Word Sets backup format is unchanged; this preference is local browser UI state.

## Verification

Static source review and ZIP structure/integrity checks only. Rust compilation, WASM builds, unit tests, E2E and live Chrome review could not be executed in this environment.

## Windows follow-up

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Verify Word Sets at long-set and mobile sizes, selection/search, rename/deletion, default Dictionary destination across reloads and deleted-set fallback, the backup panel, Practice integration and both local `/` and hosted `/zipeek/` base paths.

---

# R517 WIP: Listening Choice audio and configurable hints

Date: 2026-11-16 (UTC)

Archive: `zipeek-R517-WIP-listening-choice-audio-and-hints-20261116-214323.zip`

## Changes

- Rebuilt the persisted Practice Listening Choice screen around a clear, circular speaker button with existing Mandarin browser speech and manual replay. Preserved the three-option answer and practice-only outcome semantics.
- Added independent Practice Setup controls for pinyin and meaning: Never, After answering (default), and Always. Each policy applies uniformly to every answer choice, and post-answer feedback respects the chosen policy. Added optional autoplay (off by default) with announced browser-speech failures; the manual speaker remains available.
- Loaded dictionary readings and meanings for the prepared candidate words and displayed them alongside the corresponding Hanzi. Reserved feedback/Continue height to avoid layout shift, improved answer states and focus styles.
- Persisted settings in Practice profiles and typed per-session Listening Choice configurations. Added unit tests for disclosure, reducer isolation and serialized-session validation, and adapted the browser session journey to assert pre/post-answer hints.
- The standalone Listening Choice route is unchanged. No new dependencies, server calls or scoring changes.

## Data and compatibility

The strict persisted Practice options/session configuration structure changed. Earlier saved profiles and in-progress sessions may fail validation. Finish or export important local state before installing; project policy does not use migrations in development.

## Verification

Static source/integration checks and full archive CRC validation only. Rust formatting, unit-test execution, WASM compilation, E2E compilation and Chrome browser acceptance were not available and remain pending. This is an unverified WIP candidate.

## Required local checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

In Chrome check Never / After answering / Always combinations, audio autoplay and replay (including blocked synthesis), incorrect answer, keyboard/focus, responsiveness, restore and saved profiles, and local `/` versus deployed `/zipeek/`.

---

# R516 WIP: Practice completion-screen redesign

Date: 2026-11-16 (UTC)

Archive: `zipeek-R516-WIP-practice-completion-ui-20261116-214000.zip`

## Changes

- Redesigned the finite Practice completion panel with a compact trophy heading, context-sensitive "Excellent work!" / "Session finished!" message, and visible exercise completion count. Added a pure percentage helper with empty and partial-session regression cases.
- Added a percent-based accessible progress bar and quieter completed/skipped statistics without changing how exercise outcomes are counted.
- Simplified the learning-history status display while preserving finalization, recorded, not-recorded, error and Retry behavior.
- Kept Practice again as the primary navigation action and Explore vocabulary as the secondary route; improved responsive spacing and focus styles.
- Updated authored-session browser assertions for the new heading and notice. No dependencies, saved data format, session progression or route changes.

## Verification

Static source checks and complete-source ZIP CRC validation only. Rust formatting, tests, WASM compilation and live Chrome behavior were not available here. This is an unverified WIP candidate.

## Required local checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Check desktop/mobile layout, reduced motion, fully completed and skipped sessions, and all learning-history finalization states.

---

# R515 WIP: Typed Hanzi completion-preview state

Date: 2026-11-16 (UTC)

Archive: `zipeek-R515-WIP-hanzi-typed-completion-preview-20261116-213216.zip`

## Change

- Replaced the anonymous `(String, bool)` state tuple with the private named `HanziCompletionPreview { step_id: String, ready: bool }` struct in `local_session.rs`.
- Updated the Yew state hook, ready callback, dragon preview calculation, and targeted regression test to use named fields.
- Preserved the step-ID guard, preview timing, Continue behavior, Undo behavior and all existing stroke features.
- No dependency, storage format, API, or user-facing behavior changes.

## Verification

- Verified all former preview tuple references were replaced and the complete-source ZIP structure and CRCs are valid.
- Rust formatting, unit tests, WASM compilation and Chrome acceptance were not run because Rust tooling and a live browser are unavailable here. R515 remains an unverified WIP candidate.

## Required local checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
```

---

# R514 WIP: Practice Hanzi progress preview compile fix

Date: 2026-11-16 (UTC)

Archive: `zipeek-R514-WIP-hanzi-progress-preview-compile-fix-20261116-213010.zip`

## Correction

- Fixed the Windows `E0425` compilation failure reported for `local_session.rs` at the progress-preview callback and the dragon progress calculation. R513 referenced `hanzi_ready_preview` at both sites without defining the hook in the `LocalPracticeSession` component.
- Added the missing `use_state(|| None::<(String, bool)>)` hook alongside the session state, within the component's unconditional hook order. The existing callback and `preview_progress` helper remain unchanged, including their active-step guard.
- No storage, dependency or user-visible behavior changes.

## Verification

- Reproduced the missing declaration by inspecting the R513 source archive and verified the fix is present in the updated source. Full Rust compilation and browser tests are not available in this environment. This remains a WIP candidate.
- On Windows, run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`. Recheck dragon progress, Undo, and Continue in Chrome.

---

# R513 WIP: Hanzi completion preview, stable layout and stroke playback

Date: 2026-11-16 (UTC)

Archive: `zipeek-R513-WIP-hanzi-progress-animation-and-layout-20261116-211201.zip`

## Changes

- The Practice dragon now previews the completed Write Hanzi exercise as soon as the final character is finished. It does not record the completion or move to the next exercise until the learner clicks Continue. The preview is tied to the active step ID and is revoked by Undo.
- The writing board always reserves the space for its completion summary and Continue button. Finishing the character no longer injects height below the drawing area.
- The Watch and automatic-assistance SVG masks use explicit alpha compositing, distinct IDs for each replay and wider centerline coverage for thick reference outlines. The authored stroke coordinates, durations, matching and hints are unchanged.
- Added a focused regression test to verify preview progress cannot leak across steps. No dependencies or persisted data formats changed.

## Verification and limitations

Static source and archive checks were performed. Rust formatting/tests, WASM compilation and browser motion verification were not available in this environment. The SVG changes are an unverified correction candidate, not evidence that animation is visually fixed. Verify repeated Watch, automatic stroke assistance, complex hooks, and reduced-motion behavior in actual Chrome.

## Required local checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

---

# R512 WIP: Automatic Hanzi stroke assistance

Date: 2026-11-16 (UTC)

Archive: `zipeek-R512-WIP-hanzi-assisted-stroke-progression-20261116-210708.zip`

## Changes

- Practice Setup > Write Hanzi now offers independent mistake-assistance policies: after three failed attempts (default), after five, or Off. The selected policy is persisted in the current strict Practice profile/session options.
- At the threshold, the current expected stroke is demonstrated via the aligned R511 outline/mask replay for 750 ms, then completed after 850 ms. The learner cannot draw during assistance. The mechanism operates even if the optional full-character Watch button is hidden.
- The writing domain distinguishes accepted handwritten strokes with match scores from assisted completion slots, retaining cumulative mistakes. Completion reports handwritten and assisted stroke totals separately; assisted strokes do not become independent proficiency evidence.
- Pending assistance can be cancelled by Undo, Reset or switching/loading a character. Stale timer completions are rejected by an epoch guard. Per-stroke failures reset after correct handwriting or assisted completion. Standalone Hanzi writing uses the default three-mistake policy.
- Added focused domain, reducer, setup-policy and persisted-options tests. No dependency or dictionary changes.

## Data impact

The required `assistance` value changes the strict current Write Hanzi options representation. Older stored Practice sessions/profiles may not deserialize. This project explicitly requires no backward-compatibility migration during development. Finish or export important sessions before installing the candidate.

## Verification

- Source-level consistency and ZIP CRC checks only. Rust/Cargo tooling and real app Chrome testing were unavailable. Tests were authored but **not executed**; R512 is an unverified WIP candidate.
- R510 handwriting calibration and R511 replay browser acceptance remain pending.

## Required local verification

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-hanzi-writing -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

In Chrome validate 3 / 5 / Off, keyboard/mouse/touch, full and reduced motion, standalone and embedded Practice, first and final assisted strokes, Undo/Reset and character changes during playback, status layout, completed-stroke persistence, and saved Practice profile/session restart. Check both local `/` and hosted `/zipeek/` where applicable.

---

# R511 WIP: Write Hanzi stroke replay alignment

Date: 2026-11-16

Archive: `zipeek-R511-WIP-hanzi-replay-mask-alignment-20261116-210325.zip`

## Changes

- Fixed the Watch playback mask coordinate mismatch: reference medians were normalized to screen coordinates, then rendered under the authored-outline Y inversion, causing fragmented stroke reveal.
- Added a dedicated replay median path conversion that returns normalized points to the outline's raw coordinate system. The existing static direction hint deliberately stays in screen coordinates.
- Expanded SVG mask bounds to include the raw stroke vertical extent from -124 to 900, preventing clipping at the edge.
- Added two Rust geometry regression tests for representative multi-segment median alignment and both extremes of the reference vertical range.
- Left handwriting grading, Practice settings, word audio, styling, animation timing, dictionary assets, dependencies and saved data unchanged.

## Verification

- Static source and archive integrity checks performed. No Rust compiler, Cargo/WASM build or live app Chrome test was available in this environment.
- R511 remains an unverified WIP until locked Rust tests, WASM compilation and browser visual acceptance pass.

## Required acceptance

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

In Chrome, press Watch on characters with straight, curved and hooked strokes; confirm complete progressive outlines, no disconnected fragments, correct stroke persistence, replay restart and reduced-motion behavior. Verify both local `/` and deployed `/zipeek/` layouts where applicable.

---

# R510 WIP: Handwriting-aware recognition and configurable word prompts

Date: 2026-11-16 (UTC)

Archive: `zipeek-R510-WIP-hanzi-recognition-and-word-audio-20261116-205909.zip`

## Changes

- Replaced rigid uniformly indexed stroke-shape correspondence and worst-vertex veto with bounded dynamic time warping over arc-length-resampled reference and candidate strokes. Limited translation adjustment and symmetric robust point-to-segment coverage retain shape discrimination while accepting ordinary hand variation.
- Maintained absolute start/end checks, overall drawing direction, plausible stroke length, reverse-direction detection and discrimination against later canonical strokes. Retained canonical order for indistinguishable overlapping strokes. The writer now shows specific start, end, direction, order, length and shape rejection messages.
- Added regression tests for realistic wavy/shifted/early-turn strokes, reversed strokes, missing hooks/corners, detours, wrong order, invalid geometry, and scoring/session transitions. Fixed the elastic alignment's accumulated insertion/deletion penalty.
- Added **Pinyin**, **Meaning**, and **Listen** independent switches to Practice Configure > Write Hanzi. All default to enabled. The word header always displays the Hanzi and conditionally shows the dictionary-sourced Pinyin, definition and the existing accessible browser Mandarin Listen button. Missing meanings are explicitly identified; audio failures use the existing notice.
- Persisted prompt options in the current Practice exercise options and covered them with reducer and session serialization tests. Standalone Hanzi writing stays separate. No new dependencies, dictionary regeneration, external audio processing or changes to personal learning telemetry.

## Data impact

`WriteHanziOptions` gained three required boolean fields. Strict existing session/profile deserialization may refuse older saved representations. Under the development-phase no-backward-compatibility policy, no old-format reader or migration was added. Finish or export valuable pending sessions and profiles before installing this candidate.

## Validation and limitations

- Reviewed the source paths and checked the ZIP name/required root layout and full archive CRC integrity.
- Rust formatting, unit tests, locked WASM build, E2E test compilation and real browser/touch/stylus acceptance were **not run**; Rust toolchain and browser runtime are unavailable in this environment. Authored tests are not passed tests.
- Matcher numerical tolerances remain provisional. Collect representative user handwriting and measure false rejection/acceptance before grading or production acceptance. R508/R509 browser acceptance is still pending.

## Windows checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-hanzi-writing -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Also verify all combinations of the three prompt switches, TTS with/without Mandarin voice support, saved Practice profile/session reload, multi-character words, keyboard and mobile wrapping, real pointer traces, `/` and `/zipeek/` routes.

---

# R509 WIP: Progressive Hanzi stroke hint

Date: 2026-11-16

Archive: `zipeek-R509-WIP-hanzi-progressive-stroke-hint-20261116-204838.zip`

## Changes

- Replaced the Practice Start cue dot with an arc-length-aware, tapered directional preview of the upcoming reference stroke. The preview covers 25% initially, 35% after one rejection, and 50% after subsequent rejections.
- Full guide retains its reference outline and shows the same partial-stroke preview. None continues to hide unfinished-stroke guidance.
- Reset the per-stroke hint escalation after successful acceptance, Undo, Reset, and character changes. Suppress the hint while drawing or playing reference animation.
- Updated writing configuration copy, relevant Rust unit tests, the Write Hanzi feature specification, and the Hanzi writing architecture documentation. No dependencies, scoring thresholds, saved-data formats, or routes were changed.

## Validation and limitations

- Compared the changed-file list with the supplied source snapshot and checked full-source archive member integrity, required unpack paths and filename shape against the CLI parser rules.
- Rust formatting, locked tests, WASM compilation and actual Chrome/browser checks: **not run**. This is an uncompiled WIP source candidate, not an accepted release.
- The older R508 layout acceptance and accepted-stroke canvas-to-SVG visual transition remain pending browser verification.

## Windows checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

In Chrome, verify all three guidance modes, 25/35/50% progression, acceptance/Undo/Reset/character-change resets, replay and drawing suppression, short hooks and curved strokes, touch/stylus inputs, and no board-layout movement during feedback.

---

# R508 WIP: Hanzi writing resize compile fix and stable feedback layout

Date: 2026-11-16

Archive: `zipeek-R508-WIP-hanzi-resize-and-feedback-layout-20261116-202031.zip`

## Changes

- Fixed the E0614 compilation error in `practice/workspace.rs`: `use_resize_epoch` already returns a `u64`; the `use_effect_with` dependency now uses `resize_epoch` directly rather than dereferencing it.
- In embedded Practice Write Hanzi, reserved a persistent `min-h-5` training-status slot. Rejected-stroke feedback now replaces the slot's content instead of inserting a new layout row and shifting the board or toolbar. The existing `role=status`, `aria-live` and loading-error `role=alert` semantics remain.
- Standalone writing status/progress display, grading, strokes, replay and saved options are unchanged. No new dependencies or storage changes.

## Validation and limitations

- Checked source against the exact R507 baseline and verified the expected resize helper return type, updated Yew dependency expression, and stable status wrapper. ZIP CRC and all packaged source contents were verified.
- Rust formatting, locked tests, WASM compilation and actual Chrome E2E: **not run** (Cargo/Chrome app runtime unavailable). This is an uncompiled WIP candidate; other build errors may remain.
- The distinct accepted-stroke canvas-to-canonical SVG visual transition noted in the preceding video **is not corrected in this focused release**. It requires dedicated browser acceptance and rendering work.

## Windows checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

In Practice Write Hanzi, alternate accepted/rejected strokes and verify that the board and toolbar do not move when `Stroke not accepted. Try again.` appears; verify E2E selectors and standalone status behavior separately.

---

# R507 WIP: Practice recovery UX and Hanzi writing module boundaries

Date: 2026-11-16

Archive: `zipeek-R507-WIP-session-recovery-hanzi-modules-20261116-201210.zip`

## Changes

- Replaced the raw `Session unavailable` / `InvalidStoredSession` page with a focused, accessible recovery card. Missing sessions, invalid saved data, browser storage failures and out-of-date sessions have distinct explanations. Back to Practice setup is the primary action; Retry appears only when recovery can reasonably succeed. Neither action deletes stored data automatically.
- Preserved strict current-format session validation. This release does **not** migrate older R505-incompatible practice data or alter session persistence. A session that fails validation may still require starting fresh.
- Split the Hanzi writing practice module into a thin `mod.rs`, a `workspace.rs` session orchestrator, `lifecycle.rs` hooks for resize and replay timer ownership, and `standalone.rs` for standalone-only progress, target picker, header and reference summary. The existing pure `controller.rs` reducer, board, controls, pointer handlers and guarded async reference loading remain the state owners. No context provider was introduced because writing state is used by a single feature instance rather than a shared subtree.
- Updated the authored-session E2E assertion to verify stable recovery component/error markers and the new back-to-setup action rather than implementation-specific error text.
- Added recovery-message unit checks for incompatible data and temporary storage failure. No dependency, URL or storage format changes.

## Verification and limitations

- Static source and E2E selector checks, absence of duplicate Rust module declarations, full archive CRC and source-vs-archive comparison: run for this handoff.
- Rust formatting, locked tests, WASM compilation and actual browser E2E: **not run** (Rust toolchain and running Zipeek unavailable in this container). Source edits are uncompiled WIP; compiler errors remain possible.
- Before concluding that R507 is accepted, verify restored, missing, incompatible and temporary storage-error sessions in Chromium; check writing-guide changes, stroke undo/reset/replay and resize at both standalone and Practice routes.

## Windows checks

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

---

# R506 WIP: Write Hanzi guide callback compile correction

Date: 2026-11-16

Archive: `zipeek-R506-WIP-write-hanzi-guide-callback-compile-fix-20261116-200521.zip`

## Fix

- Restored the missing typed `on_toggle_guide` callback in `HanziWriting` and connected it to the existing `WorkspaceAction::ToggleGuide` reducer dispatch. The standalone guide-toggle control retains its existing behavior; the Practice guide control remains hidden and guidance is configured before the session.
- Inspected the writing feature boundaries. `controller.rs` already owns stroke/feedback/replay transitions, with guarded asynchronous reference loads in `mod.rs`; `board.rs` and `controls.rs` are presentation components. No behavior or persistence schemas changed.
- Remaining design debt: `practice/mod.rs` still combines session orchestration and page presentation; consider separating a feature hook/controller only when refactoring is backed by browser acceptance. The current reducer is suitable for the coupled writing state, so no replacement framework or speculative modules were introduced.

## Verification

- Confirmed the original R505 source referenced `on_toggle_guide` without defining it; R506 defines it with `Callback<MouseEvent>` through `action_callback` and keeps the `WritingControlsProps` contract intact.
- Archive member comparison and ZIP CRC: performed for this handoff.
- Rust formatting, `cargo test`, locked WASM build, actual Chrome E2E: **not run** (Cargo unavailable in this environment). The specific compiler diagnostic should be resolved but a complete build has not been demonstrated.

## Windows checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

Confirm the standalone writing route's Show/Hide guide button, and the absence of that control in Practice mode. R505 changed stored Practice options; the same compatibility limitation still applies.

---

# R505 WIP: Write Hanzi Practice configuration and focused canvas

Date: 2026-11-16

Archive: `zipeek-R505-WIP-write-hanzi-guidance-and-controls-20261116-200019.zip`

## Changes

- Replaced the description-only Write Hanzi Configure view with three guidance modes: None, Start cue, and Full guide, plus an optional stroke-order animation switch.
- Persisted these typed settings in Practice profiles and in the saved session's per-exercise configuration; loaded them into the running Write Hanzi component.
- Changed the Practice exercise heading from a generic title and character counter to `Write <selected Chinese word>`.
- Removed the visible stroke counts, mistakes, progress bar, and repetitive trace instruction from Practice Write Hanzi, retaining nonvisual status announcements and error handling.
- Made Undo and Reset distinct: Undo removes the latest accepted stroke, while Reset restarts the entire current character. Watch is visible only when animation is enabled. Guidance controls no longer appear inside Practice writing.
- Replaced the animated centerline of the highlighted stroke with a small starting marker. Full guide continues to show the stroke's authored outline; Start cue shows the marker alone; None hides both.
- Preserved the separate standalone Write Hanzi editor's existing guide toggle, progress UI, and playback controls.
- Updated relevant E2E selector identity and added reducer, serialization and session configuration tests. No new Rust dependencies or Python scripts.

## Validation and limitations

- Source inspections, wiring/consistency checks and ZIP integrity: run for this handoff.
- Rust formatting, locked Cargo tests, WASM build and actual Zipeek Chrome E2E: not run because the Rust toolchain and app runtime are unavailable here.
- Existing Practice profiles and pending sessions may be invalidated by the new current settings schema. No legacy reader or migration was introduced. Back up or finish them before installing.
- Animation playback itself retains the existing authored-stroke mask/reveal implementation; independent visual acceptance and animation polish remain necessary.

## Required Windows checks

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
```

In Chrome, verify all three guidance modes, Watch hidden when disabled, Undo/Reset behavior, multi-character progression, reload/persisted profile, and that standalone Write Hanzi remains unchanged.

## E2E evidence stabilization source candidate

- Added an offline E2E evidence auditor and five Node unit tests. The auditor checks suite/individual manifest agreement, checkpoint timelines and screenshots, required reports, recording validation claims, and optional FFmpeg video decoding. It produces Markdown and JSON reports in the actual run directory.
- Exposed checkpoint-capture failures in scenario manifests and Codex feedback rather than silently dropping them. Added `ZIPEEK_E2E_EVIDENCE_STRICT=1` to fail evidence-focused runs when video or checkpoints are missing or invalid.
- Added a `cargo make e2e-evidence-audit-tests` task and updated E2E workflow documentation, coverage notes, and the current verification tracker.
- No crates or dependencies were changed. No product gameplay or learner data was modified.
- Verified: five Node artifact-integrity tests pass; documentation validator reports zero maintained-document errors and one existing frozen-history warning. Rust/Cargo and live browser E2E could not be executed here and remain unverified. The next verification gate is the Windows locked E2E compilation and a real Chrome recording reviewed with the new auditor.

## R536 WIP - trusted local speech bridge and recognition UI (2026-11-17)

- Added exact-origin, local-development-only browser access to the loopback native Mandarin speech service. Browser-origin requests outside the local Zipeek development origin remain rejected.
- Added explicit runtime-only local provider opt-in in Speech Settings, with capability checks, WAV TTS playback and five-second microphone capture converted to 16 kHz PCM16 WAV for Whisper.
- Wired normal Mandarin Listen/Practice speech requests to the selected local service; browser speech remains the default.
- Reduced Recognize Hanzi canvas height and removed unrelated Word Set selection from the initial view. Word Set actions remain available for verified results.
- Accepted strictly fenced candidate JSON without accepting invalid Hanzi candidates.
- Added offline JS WAV/bridge tests. Rust compilation, browser speech acceptance and deployed path checks remain unverified.
- Existing Speaking Recorder now offers an explicit local transcription action for completed in-memory recordings, with no grading or history.

## R537 WIP - recognition theme, scroll and safe diagnostics (2026-11-17)

- Corrected the recognition canvas's hard-coded white fill and dark stroke color: appearance now follows the active application surface and ink tokens, with theme-change repaint support.
- Reduced initial page spacing for smaller viewports; result content remains vertically scrollable rather than clipping.
- Malformed vision model output now provides JSON error location and structural classification, with console WARN diagnostics that never include raw model text, submitted image data, character candidates or credentials.
- Added offline canvas theme regression coverage and JSON failure assertions.
- Rust compilation, WASM build, root/Pages browser acceptance and live recognition are not verified in this environment.

## R538 WIP - Rust/WASM local speech and pinned Chart.js bridge (2026-11-17)

Archive: `zipeek-R538-WIP-rust-wasm-speech-and-chart-interop-20261117-232322.zip`

- Replaced the active JavaScript local speech module with Rust/WASM browser transport, explicit provider state, guarded loopback fetch, WAV playback, microphone capture, PCM16 audio conversion and cancellation. All existing direct Rust/Yew speech consumers now use the Rust implementation.
- Added native Rust WAV encoding tests and origin tests; removed the superseded JavaScript implementation and its Node-specific test.
- Retained the D35 Chart.js/esbuild pins, vendor packaging and notices. Chart.js construction uses a thin JavaScript module; Rust owns chart options and an opaque destructible handle. Unused JavaScript chart wrappers were removed.
- No dependency or schema changes, no new native service exposure, no persistent audio. Browser speech remains the default.
- Source and JavaScript syntax checks only. Rust formatting, native/WASM builds, real local speech/Chrome tests, Chart.js mounting/reduced-motion lifecycle and deployment-path acceptance were not run. This is an unverified WIP source candidate.

## R539 WIP - Recognition experience redesign

Reworked `/recognize-hanzi` into a responsive writing-and-actions workspace with theme-aware drawing surface, contextual empty and input states, explicit image submission, visible error recovery, concise privacy copy, and results displayed as dictionary-verified cards. Removed redundant controls in the empty state, kept Word Set actions contextual to verified hits, corrected the E2E's Recognize with AI label, and made Undo/Clear reflect actual editable content. Only the existing JavaScript canvas interop was changed; no dependency or storage changes.

Verification: JavaScript syntax and ZIP integrity checked. Rust, WASM, and real browser visual acceptance not run in this environment.
