## R524 WIP - 2026-11-17 - Translation action routing and map asset diagnostics

The source candidate replaces translator delegated dataset dispatch with typed direct callbacks, distinguishes connection failures in translation feedback, and identifies HTML map fallbacks or missing content separately from malformed JSON. It records approved service, course lock, grammar evidence and speech provider decisions D53-D56 without implementing their larger pipelines. Province Quest generated assets were not present in the baseline source archive. Rust/WASM and browser checks remain pending.

## R522 WIP: Documentation and repository hygiene (2026-11-16)

Documentation-only reconciliation on the R521 source. The user reports an unspecified current build failure and unknown feature status; neither is treated as passing acceptance. Replaced the unwieldy active task register with a prioritized index, preserving all 644 unchecked checklist rows and 56 historical checked rows verbatim in `docs/archive-history/LEGACY_ACCEPTANCE_LEDGER.md`. Removed the user-approved legacy `scripts/e2e-evidence.ps1`, updated its script/recording references, and corrected CI guidance to the actual Rust/WASM workflow. Preserved all dated completed records, historical release evidence and unresolved DIGEST entries. No application or dependency changes. Rust/WASM/Chrome not run. See `CURRENT.md` and root release notes.

## R520 WIP: Time Weaver setup polish (2026-11-16)

Reworked the standalone Time Weaver setup with accessible toggle-card switches, a three-size tile selector, 10/15/20 balanced prompt count, and a responsive live reading preview. Persisted validated UI preferences using `AppEnvironment.local_storage` without touching saved Practice or Word Sets data. Added Rust settings/engine tests and updated Chrome test selectors. Source review and archive integrity only; Cargo, WASM and Chrome acceptance are pending. See root `RELEASE_NOTES.md`.

## R519 WIP: Hanzi Crossword word rotation (2026-11-16)

Added a standalone pure crossword rotation model with 100 most-recently-used unique Hanzi answers, ten recent puzzle word signatures, and deterministic candidate scoring. New game and Play again use fresh session-derived seeds and rank up to eight playable candidates against local history. Browser storage uses the established AppEnvironment storage handle, and each session is counted once rather than after every tile placement. Invalid/oversized history falls back to an empty rotation without changing existing saved crossword puzzles. Added history/eviction/determinism/scoring tests and updated crossword and personal-data docs. Source-only inspection and ZIP integrity checks; Rust/WASM and Chrome remain unverified.

## R518 WIP: Word Sets workspace and Dictionary destination (2026-11-16)

Refactored Word Sets layout to use a single document scrollbar, direct set selection, compact management controls and search. Under the clarified Default Word Set requirement, the current set now persists as the Dictionary Add/Remove destination in a separate validated browser-local preference; Practice selection is unchanged. Word Set data and backup remain unchanged. Source/ZIP checks only; Rust/WASM/Chrome pending. See root `RELEASE_NOTES.md`.

## R517 WIP: Listening Choice configuration and audio-centered layout (2026-11-16)

The saved local Listening Choice exercise now provides an accessible speaker-centered layout, three word cards with independent pinyin and meaning disclosure policies, configurable autoplay, and reserved feedback space. Setup preferences persist with both Practice profiles and sessions via a typed ListeningChoice configuration. Added reducer, serialization and browser visibility assertions. The standalone route is unchanged. No dependency changes. Strict saved settings/configuration schema changes may invalidate old profiles or sessions; Rust, WASM and browser verification pending. See root `RELEASE_NOTES.md`.

## R516 WIP: Practice completion UI refinement (2026-11-16)

Redesigned the finite local Practice completion panel with a compact achievement heading, accessible percent progress bar, clearer completed/skipped tiles, subdued history notice and stronger action hierarchy. Preserved session outcomes and saving/retry state semantics; updated the authored-session E2E expectations. No new dependencies, route changes or persisted data changes. Static validation and ZIP verification only; Rust/WASM/Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R515 WIP: typed Hanzi completion-preview state (2026-11-16)

Replaced the local `(String, bool)` Yew preview-state tuple with `HanziCompletionPreview { step_id, ready }`, updating the callback, pure progress helper and regression tests without changing behavior or persisted data. Source and ZIP checks only; locked Rust, WASM and Chrome tests remain pending. See root `RELEASE_NOTES.md`.

## R514 WIP: Practice Hanzi preview missing state hook compile fix (2026-11-16)

Corrected R513 Windows E0425 by declaring the missing `hanzi_ready_preview` Yew state handle in `LocalPracticeSession`. No behavior, dependency or persisted schema change. Static source/archive verification only; Cargo and browser acceptance pending. See root `RELEASE_NOTES.md`.

## R513 WIP: Write Hanzi progress preview and animation presentation (2026-11-16)

Practice dragon progress previews completion of the final Hanzi immediately, without committing or advancing the step until Continue. The writing board reserves its completion-action area to prevent a layout jump. Replay and assistance masks gain per-playback IDs, explicit alpha composition and increased path coverage. Added pure progress-preview regression checks. No dependency or storage format changes. Source/archive validation only; Rust/WASM and browser animation acceptance pending. See root `RELEASE_NOTES.md`.

## R512 WIP: Hanzi assistance after repeated errors (2026-11-16)

Added configurable 3 / 5 / Off auto-assist thresholds to Practice Setup with default 3, single-stroke demonstration before advancing, and separate assisted / handwritten progress in the writing session. Added reducer and domain transition regression tests, stale-timer cancellation, per-character completion counts, and strict option serialization coverage. No dependencies. Old saved Practice options may be incompatible. Source checks only; Cargo/WASM/Chrome tests pending. See root `RELEASE_NOTES.md`.

## R511 WIP: Write Hanzi replay mask coordinate correction (2026-11-16)

Corrected playback masks to use the same authored reference space as the rendered stroke outlines, including the full vertical extent. Added two geometry tests. No matching, guidance, storage, dependencies or feature toggles changed. Source and ZIP checks only; Rust/WASM and browser confirmation pending. See root `RELEASE_NOTES.md`.

## R510 WIP: handwriting recognition and configurable word prompt (2026-11-16)

Refactored stroke matching to use bounded elastic alignment and robust path coverage while retaining structural checks and stroke-order comparison. The Practice exercise header now offers independently configurable dictionary Pinyin, English meaning and browser Mandarin Listen. Added recognizer regression fixtures and setup reducer tests. Source/archive inspection only; Rust, WASM and Chrome acceptance pending. Practice options format changed with no compatibility reader. See root `RELEASE_NOTES.md`.

## R507 WIP: Practice recovery UI and Hanzi writing modules (2026-11-16)

Replaced raw saved Practice session errors with typed recovery guidance and accessible setup navigation, without migrating or deleting invalid local data. Refactored the Write Hanzi module into a thin entry point, one reducer-backed workspace, lifecycle hooks and standalone presentation. Preserved existing writing reducer and public component exports. E2E selectors updated; Rust/WASM/Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R505 WIP: Write Hanzi Practice learner guidance (2026-11-16)

Added typed per-exercise guidance (None / Start cue / Full guide) and animation switch, stored in Practice profile and session settings. Simplified the exercise header and canvas, removed Practice-only progress/mistake/trace chrome, distinguished Undo from Reset, and removed the animated centerline. Standalone writing retains its existing controls. Static source checks only; Rust/WASM and in-app Chrome acceptance not run. See root `RELEASE_NOTES.md`.

## R504 WIP: Practice scroll root height correction (2026-11-16)

Changed the Practice exercise-stage root from inherited desktop `h-full` to an explicit viewport-minus-header/padding height so the inner list has a definite flex size. Reserved the list scrollbar gutter and required the final tile and root to remain within viewport bounds in the E2E assertion. A representative standalone Chromium geometry test passed at desktop 650/768/900px and mobile width; the real app Chrome flow and Cargo checks are not verified. See root `RELEASE_NOTES.md`.

## R502 WIP: Practice exercise panel scrolling and labels (2026-11-16)

Corrected the scroll owner from the outer Practice main area to the bounded exercise widget. Added requested AI labels for four authored exercise types while keeping their current generation behavior unchanged. Renamed the speaking tile and related Practice UI to Pronunciation. Updated browser assertions and UX ownership docs. No dependencies or persisted data changes; Rust/WASM/browser execution pending.

## R500 WIP: Practice scroll and service capability labels (2026-11-16)

The Practice viewport now owns desktop vertical scrolling to the final exercise and configuration controls. Added typed AI/image-generation dependencies in `zipeek-exercises` and visible labels/test attributes only on relevant exercise tiles. Removed the verbose selected/total status announcement; the compact selected count remains. Updated targeted Chrome assertions and current owner documentation. No dependency or persistence changes. Static checks only; Rust/WASM and Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R499 WIP: Stable E2E domain selector contract (2026-11-16)

Added stable root, action, domain item and observable state attributes for expanded Grammar, Sentence Transformation and its setup, and Cloud Match configuration. Migrated targeted Rust E2E journeys from button-text and aria-label queries to stable selectors. No feature, data, dependency or route changes. Static source checks only; Rust/WASM and Chrome acceptance pending. See root release notes.

## R498 WIP: Sentence Transformation parser repair (2026-11-16)

Fixed the missing closure of a nested Yew `html!` expression in the R497 Sentence Transformation view, strengthened pure reducer mode guards and added a regression test. The reported R497 WASM compile error is addressed in source, not verified by a successful build. No dependency or data changes; see root release notes.

## R497 WIP: Practice and Grammar state integrity (2026-11-16)

Corrected Sentence Transformation hook lifetimes and coupled state, prevented unsupported pattern autocomplete from silently switching to All, cancelled stale Practice history UI callbacks, keyed Practice session routes by ID and made expanded Grammar step navigation transactional with saved checkpoints. Added pure transition and selector tests. No dependencies or persisted schema changes. Static source checks and ZIP verification only; Rust/WASM/browser acceptance pending. See root release notes.

## R496 WIP: Grammar Pinyin module compile fix (2026-11-16)

Fixed the repeated `pub mod pinyin;` declaration reported by Windows Rust E0428 in the R495 candidate. Retained a single declaration, with no feature or data changes. Duplicate module source audit passed; Cargo/WASM acceptance pending. See root release notes.

## R494 WIP: Grammar answer lock and Practice interaction fixes (2026-11-16)

Once the correct guided Grammar answer is selected, further option selection is rejected by the pure state reducer and the UI disables choice buttons. Added independent audio buttons to the recognition choices. Removed the Practice exercise panel's nested height-limited vertical scrollbar. No exercise-type, curriculum, data or dependency change. Source/static checks only; Cargo/WASM and browser acceptance pending. See root release notes.

## R492 WIP: Grammar architecture consolidation (2026-11-16)

Moved Grammar catalogue and all teaching tables from web to pedagogy, and isolated pure lesson step/reveal/choice logic in exercises. The two existing course engines share one lesson scaffold, but retain distinct exercise banks and assessments. Updated generators and HSK index tooling to the new source-of-truth paths. Python Grammar/HSK/Vocabulary source checks passed; locked Rust, WASM and Chrome checks not run. See root `RELEASE_NOTES.md`.

## R491 WIP: Grammar guided practice and state integrity (2026-11-16)

Added recall-before-reveal and a follow-up guided meaning check to the expanded Grammar lessons, with a single reducer for reveal/selection invariants and a pure, tested step-flow module. Kept existing 2-step-per-point checkpoints and all course and syllabus data unchanged. Removed duplicated `grammar_completed` module declaration. Grammar/Vocabulary source audits passed; Cargo/WASM/Chrome checks remain outstanding. This is not independently assessed mastery or HSK certification. See root `RELEASE_NOTES.md`.

## R490 WIP: HSK grammar mapping review integrity (2026-11-16)

Added safe, source-hash-bound reimport that preserves editorial decisions, strict source-field and mapping validation, and a per-HSK-level review status report. Six offline tests and pinned Grammar/Vocabulary source audits passed. Full 593-row HSK import and expert mapping, Cargo/WASM and browser acceptance remain pending. See root `RELEASE_NOTES.md`.

## R489 WIP: Grammar syllabus traceability (2026-11-16)

Introduced stable editorial IDs for all 550 authored Grammar entries and a reproducible import/review/audit workflow for the HSK 2025 grammar syllabus. No HSK mappings were certified; full external transcription download and expert review remain pending. Python offline checks passed, Rust/WASM and browser acceptance not run. See root `RELEASE_NOTES.md`.

## R488 WIP (2026-11-16)

All 100 Grammar-only regions now have draft teaching courses, with 550 authored pattern/example entries in total (451 new). Added a checked-in editorial source, deterministic Rust generator, 100-region audit, one-question-at-a-time expanded guided practice, and concealed reference answers in the original guided sentence builder. No mastery credit or HSK certification inferred. Source audits passed; Rust/WASM and browser checks unrun. See root release notes.

## R487 WIP (2026-11-16)

Corrected the R486 Grammar authored-route `slug` Yew property move by cloning for both key and owned prop. No navigation, state, storage or dependency change. Archive-level verification only; WASM and browser checks pending. See root `RELEASE_NOTES.md`.

## R486 WIP (2026-11-16)

Grammar expanded lesson E0382 ownership repair and five-route consolidation to one validated slug route, with unchanged public paths, scoped lesson state, and added mapping tests. ZIP integrity and source checks only; Windows/WASM and browser acceptance pending. See root `RELEASE_NOTES.md`.

## R485 WIP: Grammar curriculum authoring and lesson UX (2026-11-16)

- 15 new authored Grammar courses, 75 new patterns and a named 100-region scope. 20 authored courses / 99 total patterns; 80 remaining regions unpublished.
- AI readiness-gated three-line supplementary examples; Pinyin/Grammar icon navigation, scroll-to-top and compact reference UI; guided recognition with resumable checkpoints.
- ZIP and source-level validation only; no Rust/WASM/browser acceptance. See root `RELEASE_NOTES.md` for the complete handoff.

## R484 WIP: Pinyin teaching and Grammar practice depth (2026-11-16)

Expanded six Pinyin mosaic stages into twelve authored lesson modules with a systematic initials/finals reference, browser speech examples, 24 scored written-recognition questions and local pass checkpoints. Grammar retains five published courses; their lesson steps and best independent draft scores now persist in browser storage. Crucially, written Pinyin results and one-question-per-point Grammar draft scores are not evidence of speaking/listening mastery and do not unlock mosaics. Focused tests and E2E journey assertions were authored; Rust/WASM/Chrome acceptance remains outstanding. See canonical root `RELEASE_NOTES.md`.

## R483 WIP: Curved independent Courses mosaics (2026-11-16)

Replaced the bulging Grammar grid with a 100-region balanced Voronoi tessellation while retaining the centered authored first region. Vocabulary retained weighted scattered regions but now uses shared gently curved edge rendering. Dimmed Grammar artwork is visible as a visual preview only; no mastery is inferred and no icons were added to shapes. Source geometry and E2E tests added; Rust/WASM/Chrome verification outstanding. See canonical root `RELEASE_NOTES.md`.

## R482 WIP - Coordinated Courses overview UX (2026-11-16)

Replaced large track cards with name-only tabs, dedicated Grammar route (`/courses/grammar`), `/courses` redirect to Vocabulary, and shared mosaic/preview/details viewport allocation. Stabilized the Vocabulary Explorer height across selections and loading states. Added Grammar sentence-pattern and Pinyin pronunciation-example lower previews, updated route/browser assertions and documentation. No new dependencies, data or curriculum changes. Source and archive checks only; Rust/WASM/Chrome acceptance pending.

## R481 WIP: Vocabulary dictionary and Explorer loading widgets (2026-11-16)

Replaced sentence-only waiting/loading copy with dedicated reduced-motion-safe spinner and fixed-size skeletons in the Vocabulary region details and Explorer. No dictionary lifecycle, curriculum, source-identity, dependency or persistence changes. Static checks completed; Rust/WASM and Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R477 WIP: Vocabulary Explorer and acquisition validation (2026-11-16)

Introduced a paginated under-mosaic word explorer with pronunciation, extracted Vocabulary UI/data modules and tightened course source identities against actual curriculum region/chunk indexes. Region-tagged async loading and candidate-first persistence reduce stale selection and mutation hazards. Existing curriculum and independent Grammar/Pinyin remain unchanged. Rust/WASM/Chrome verification pending; see root `RELEASE_NOTES.md`.

## R476 WIP: Source-backed Vocabulary Word Sets and viewport-constrained selection (2026-11-16)

Persisted typed course source identities instead of inferring acquisition from display names. Removed redundant links/long disclosures and provided a desktop viewport-bounded Vocabulary route. Added source identity tests and updated E2E assertions. Cargo/WASM/browser acceptance pending. See root `RELEASE_NOTES.md`.

## R475 WIP: Irregular Vocabulary mosaic and compact collection panel (2026-11-16)

Replaced rectangular Vocabulary cells with scattered, count-weighted Voronoi polygons; retained the existing 100 region identities and acquirable Word Sets. Reduced the panel to theme, counts, acquisition progress and a single next-acquisition action. Moved per-set list, vocabulary sample and HSK breakdown into disclosures. Geometry and curriculum audits were checked independently; Rust/WASM and browser acceptance remain pending. See root `RELEASE_NOTES.md`.

## R474 WIP: Vocabulary geometry parser correction (2026-11-15)

Corrected a malformed parenthesis in the scattered Vocabulary mosaic reading weight closure that blocked the R473 WASM build. No geometry semantics changed. User-reported compiler failure recorded, rerun pending. See root `RELEASE_NOTES.md`.

## R473 WIP: Scattered Vocabulary mosaic with minimum region size (2026-11-15)

Spread all 100 Vocabulary-only regions across the illustration, using softened reading-count weights and minimum tile dimensions rather than tightly clustered exact-proportional rectangles. Removed region-index and geometry explanation copy from the details panel; retained curriculum and acquisition semantics. Independent layout model checked; Rust/WASM/Chrome verification pending. See canonical `RELEASE_NOTES.md`.

## R472 WIP: Reading-weighted Vocabulary mosaic (2026-11-15)

Replaced equal-area Vocabulary tiles with deterministic 100-region treemap layout proportional to distinct Hanzi/Pinyin reading counts. Added geometry unit tests and browser large/small area assertion; kept Grammar and Pinyin unchanged. Pinned-source coverage checks passed; Rust/WASM/browser acceptance outstanding. See root `RELEASE_NOTES.md`.

## R471 WIP: Full pinned-HSK Vocabulary source coverage and multiple Word Sets (2026-11-15)

Assigned all 11,494 pinned HSK source records and 11,470 distinct simplified spellings to 100 independent named Vocabulary regions under `zipeek-pedagogy::curriculum`. Added source fingerprint, deterministic generator, strict per-entry coverage audit and a review queue for unverified automatic semantic grouping. Added batched lookup, HSK counts, region Word Set chunking into 132 acquirable 200-word sets and expanded personal storage capacity. Source audit passed; expert thematic review, dictionary identity checks, Rust/WASM/Chrome acceptance pending. See canonical `RELEASE_NOTES.md`.

## R470 WIP - Vocabulary curriculum crate ownership (2026-11-15)

Relocated the 100-region Vocabulary catalogue from Yew into `zipeek-pedagogy::curriculum` with typed IDs and currently known domain metadata; the web route uses the existing workspace dependency. Added tests and an audit of the pinned HSK vocabulary source, and deduplicated nine repeated words within existing regions. This is not full coverage or complete thematic classification: 96 regions remain frequency-based and 10,281 distinct pinned HSK spellings are not present in the current catalogue. Source audit passed; Rust/WASM/Chrome gates were not run.

## R469 WIP - Quiet Hanzi Git acquisition (2026-11-15)

Added `--quiet` to upstream `git clone` and final `git checkout` in `zipeek-cli hanzi prepare`, leaving sparse-checkout arguments unchanged and retaining command error diagnostics. Updated the local Git regression test for successful quiet clone and a failing quiet clone. No dependency, data format or frontend change. Rust/Windows end-to-end validation pending.

## R468 WIP - Windows Hanzi Git path repair (2026-11-15)

Changed native Hanzi preparation Git commands to run in the project directory with relative checkout paths, avoiding invalid Windows extended-length Git destination arguments. Added a Git fixture regression test without external network requirements and corrected outdated Hanzi preparation documentation. Windows test and real sparse checkout acceptance pending.

## R467 WIP - Courses compilation repair handoff (2026-11-15)

New numbered checkpoint replacing the incorrectly reused R466 filename for the reported Yew `html!` syntax correction. No further Rust source edits in this packaging correction. Source ZIP validation performed, Rust/WASM and Chrome verification outstanding; see canonical `RELEASE_NOTES.md`.

## R466 WIP - Courses UX corrections (2026-11-15)

Grammar-only mosaic topology, focused lessons, authored asset synchronization, Pinyin artwork and documentation updates. Original R466 build failed due to Rust `let` bindings inside Yew `html!` markup; a corrected source was packaged with a reused R466 identifier before R467 fixed the checkpoint numbering. Compilation and browser acceptance remain outstanding.

## R465 WIP - Browser English BM25 integration (2026-11-15)

Source candidate from the user-supplied formatted R464 snapshot. Added explicit WASM FTS loading and Browse's optional BM25-ranked English mode with existing filters/pagination. Strengthened native generator verification to require nonempty BM25 search results. Previous verbose WIP notes have been cleared from root release notes, with history preserved here. Native generation and browser verification remain pending.

## R462 WIP: Practice creature illumination polish (2026-11-15)

Eliminated the artificial centre ornament across portrait themes and the rectangular reveal edge across all six creatures. Existing Animato progress drives a feathered CSS alpha mask in the appropriate direction, with bounded endpoints, restrained completion flourish and reduced-motion support. Source and archive checks performed; Rust/WASM/Chrome acceptance outstanding.

## R461 WIP: Theme-specific creature orientation (2026-11-15)

Deer and White Tiger progress artwork now runs horizontally beneath the exercise with a left-to-right reveal; the other four creatures stay vertical beside it. Replaced four simplistic progress images with transparent illustrations, retained stable WebP names and native unpack support, and added dimension/orientation preflight validation. Source and artwork checks only; WASM and Chrome acceptance remain pending. See canonical `RELEASE_NOTES.md`.

## R459 WIP: Practice image delivery and viewport-height repair (2026-11-15)

Fixed the previously untested creature image delivery dependency by bundling the six source WebPs into the WASM and assigning lifecycle-managed Blob URLs at runtime, keeping the same presets and Animato rail. Replaced nested viewport minimum heights with a bounded session shell and internal exercise scrolling. E2E tests now require decoded image dimensions and no document scroll, rather than a syntactically valid path alone. Source/asset checks only; Rust/WASM/Chrome acceptance remains pending. See canonical `RELEASE_NOTES.md`.

## R458 WIP: Theme-specific illustrated Practice progress (2026-11-15)

Introduced dedicated progress creatures for six appearance presets, all sharing the same persisted-session Animato fill and 1:3 art rail: dragon, qilin, phoenix, white tiger, crane and deer. Removed visible Matching/MCQ type headings while retaining accessible names and prompt-first body. CLI preflight and E2E checks updated. Source/asset verification only; compilation and Chrome visual acceptance pending. See canonical `RELEASE_NOTES.md`.

## R457 WIP: Detailed animated Practice dragon and session UI (2026-11-15)

Replaced the fixed-stroke dragon with a packaged illustrated transparent image and used existing Animato tween primitives to update a bottom-up reveal at 16 ms intervals. The rail retains natural image proportions and shares the session layout; reduced-motion CSS snaps to the authoritative stored progress. MCQ result icons are removed and the Leave link is visually icon-only with accessible text. Asset preflight and focused unit/E2E assertions were updated. Unverified Rust/WASM/Chrome handoff; see canonical `RELEASE_NOTES.md`.

## R455 WIP: Compact shared meaning navigator (2026-11-15)

Corrected the oversized Word of the Day meaning control by removing full-width constraints from the shared navigator. Controls now sit near short definitions; long senses wrap within a capped width. Added a Browse E2E geometry regression. Source-only verification; WASM and Chrome acceptance pending. See canonical `RELEASE_NOTES.md`.

## R454 WIP: Practice dragon progress and theme-aware session (2026-11-15)

Replaced session progress counters/horizontal bars with a vertical bottom-to-top dragon rail driven by saved session position; added appearance-context wallpaper to the standalone session shell, revised the Leave session link, improved MCQ prompt and correct/incorrect response states, simplified exercise selection indicator, and corrected Word of the Day single-meaning and unavailable-Sentences presentation. Added focused browser regressions. Source-only validation; WASM and Chrome acceptance pending. See canonical `RELEASE_NOTES.md`.

## R452 WIP: Yew macro compile remediation (2026-11-15)

Corrected multiple-root Practice setup markup and seven invalid `classes!` literal arguments in Matching/MCQ and setup controls, based on five user-supplied wasm32 compiler errors. Archive/static scan validation only; Rust and Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R448 WIP: Practice setup and Matching presentation (2026-11-15)

Removed default selections; added full-height exercise scrolling, filled setup progress circles, visible selection feedback, no global chrome in session, configured 2-7 pair matching and responsive Hanzi/match feedback. Source-only handoff; compiler/WASM/Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R445 WIP: R444 route compilation cleanup (2026-11-15)

Corrected the malformed Character Hunt `html!` Link closing tag. Removed the now-dead standalone Practice page, its URL-based session hooks and exclusive Matching/MCQ renderers rather than reintroducing removed direct-route APIs. Preserved the two-route persisted-session model. Rust/WASM/Chrome verification pending.

## R442 WIP: Courses route compilation and service-state consolidation (2026-11-15)

Fixed missing Vocabulary course component import, grouped service monitoring state under a typed Yew reducer, moved the authored Vocabulary catalog out of the page, and consolidated redundant numbered release-note files into the canonical release record. Source/ZIP checks only; WASM and browser acceptance pending.

## R441 WIP: Courses and service E2E regression coverage (2026-11-15)

Six Chrome scenarios added across two standalone test targets, with R440 browser availability semantics reflected in the previous native-controls test. Browser execution and native compilation remain pending.

## R440 WIP: Application-scoped AI and image service health (2026-11-15)

The existing ServiceProvider now performs immediate and 30-second read-only availability checks using the Tauri `service_status` IPC or browser HTTP (`/models` for the configured AI API and `/health` for the image service). Probes are five-second timeout-bounded, in-flight requests do not overlap within a monitor, and old requests are cancelled on configuration changes and unmount. Settings displays service availability for both platforms while retaining native-only Start/Stop controls; the Grammar AI-example button requires both model configuration and a reachable AI endpoint. Source tests cover endpoint validation and status semantics. No package, storage or schema changes. Rust/WASM/Chrome/Tauri execution remains unverified.

## R438 WIP (2026-11-15)

Separate packaged Chinese-themed Courses illustrations for all three tabs, 50 organic Vocabulary regions, coverage masks and asset preflight updates. Grammar and Pinyin track counts remain unchanged. Rust/WASM and Chrome acceptance pending.

## R437 WIP (2026-11-15)

Separated Courses into Vocabulary, Grammar, and Pinyin route-backed tabs. Vocabulary now offers a 50-region mosaic and a right-side panel for four authored starter groups. Dictionary IDs are resolved at runtime and Word Sets are persisted atomically through the existing provider. New unit/E2E assertions authored; target builds and Chrome acceptance pending. Artwork is temporary and no mastery is inferred.

## R435 WIP (2026-11-15)

Onboarding theme selection now includes an explicit default-off local Practice outcome recording preference. Existing dedicated consent storage and Settings control are reused, and change notifications are shared. Two focused browser cases were authored. Static handoff only; Windows/WASM and browser acceptance outstanding.

## R434 WIP (2026-11-15)

Docs TOC now retains the existing `DocumentHeading` DTO instead of anonymous tuples; `DocumentToc` keeps the owning article path with the heading collection, and Yew component properties use descriptive types. Source-only WIP; WASM and browser checks pending.

- R432 WIP: compilation-error remediation, Docs module boundaries, E2E test query and error fixes, canonical worker and test fixture restoration; pending Windows/WASM verification.
## R431 WIP (2026-11-15)

Reorganized Docs into route composition, index/document loading hooks, testable explorer grouping/navigation and separate explorer/article/TOC components. The E0597 borrowed-state dependency was removed, and the common UUID version now comes from the workspace manifest with per-crate features. Rust, WASM, Clippy and Chrome acceptance are outstanding; this is a source-only WIP. See root `RELEASE_NOTES.md`.

## R429 WIP (2026-11-15)

Docs Markdown cross-document heading links now use route queries and post-render scrolling. Old Node-suite TD-002 and duplicate agent-guide pin TD-010 are reclassified as retired documentation/test-baseline liabilities; TD-006 remains awaiting runtime validation. See root `RELEASE_NOTES.md`.

## R428 WIP - Typed tab coordination and CURRENT scope (2026-11-15)

- Replaced the monolithic `other_tab.rs` with `other_tab/mod.rs` and narrow `identity`, `protocol`, `registry`, and `coordinator` modules.
- Added UUID v7 `TabId` with parsed version validation; the existing `uuid 1.29.2` package is now a direct `zipeek-web` dependency with `v7` enabled. Updated the `zipeek-web` entry in `Cargo.lock`; no new resolved package was introduced.
- Isolated typed hello/alive/goodbye messages, bounded peer tracking and testable expiry logic from BroadcastChannel resource ownership. The component only renders the warning, and coordinator drop detaches the listener, cancels its interval and sends goodbye.
- Preserved the browser-only warning and 2-second heartbeat / 7-second timeout. This still is not a single-writer lock. No browser storage or learning-data schema changes.
- Replaced the accumulated checkpoint chronicle in `CURRENT.md` with only R428 work, its verification state, and immediate next actions; updated `AGENTS.md` to keep CURRENT focused on active work in future sessions. History remains in release history and dated completion records.
- Added protocol and registry unit tests. Source checks and ZIP verification only; Rust tooling and Chrome were unavailable. Local locked build and runtime acceptance remain required.

## R427 WIP - Chrono time source and BroadcastChannel callback typing (2026-11-15)

R427 WIP: Explicitly typed the OtherTab BroadcastChannel MessageEvent closure to address reported E0282 and replaced all 12 direct js_sys::Date::now() calls in zipeek-web with chrono::Utc::now().timestamp_millis(). Preserved millisecond heartbeat and session-completion units; time-derived game seeds now use integer epoch milliseconds instead of f64 bit patterns. No dependency or schema changes. Rust formatting, native/WASM builds and browser checks remain unverified.

- R426 WIP: Corrected Courses E2E page-object element-presence check and removed recursive async lesson navigation. Source-only fix for reported E0599; Rust and Chrome execution pending.
## R423 WIP - E2E game and authored-exercise journeys (2026-11-15)

Added deterministic, four-round AI game E2E fixtures and browser scenarios for Sentence Relay and Dialogue Quest; strengthened five authored Practice types from input gates through grading, recovery and continuation. No production behavior, storage or dependencies changed. Compilation and Chrome execution pending.

## R418 WIP - Structured Tauri command errors (2026-11-14)

Source candidate based on R417: typed serializable native service/database errors, thin command modules, shared web IPC rejection decoding and regression tests. No new dependency, data migration or command name changes. Native/WASM compilation and Windows Tauri runtime acceptance pending.

## R327 candidate - Firefox viewport and mini-game integrity (2026-11-11)

The Windows Firefox 390×844 request rendered a ~500px mobile page and failed an exact-match viewport helper; R327 distinguishes exact pixel sizing from measured responsive-breakpoint verification. Also adds native default build members, independent mini-game attempt and grid bounds guards, and a once-per-character completion gate for Write Hanzi Practice. Based on the uploaded 20261111-141641 snapshot: Node contracts 781 pass / 121 inherited fail out of 902, Rust/browser tests pending. See root RELEASE_NOTES.md for the detailed 24-Rust-file change inventory.

## R323 candidate - Vocabulary-driven authored mini-games (2026-11-11)

Unverified source candidate built from the latest R322 CDP checkpoint. Independent Classifier, Navigator and Sentence states now support source-selected decks from real HSK and Word Set dictionary words; backend selection is limited to authored prompts, with empty/error states and reset semantics. Twenty substantive Rust changes plus two supporting registration/copy changes. Node 776/889 passing; 113 inherited failures unchanged. Rust/WASM/Chrome acceptance pending. See root RELEASE_NOTES.md.

## R322 candidate - Tailwind feature CSS consolidation (2026-11-11)

Unverified source candidate based on R321. Eliminated Crossword, Hanzi Writing, Matching and unused Cloud Match feature stylesheets, placing active presentation in owning Yew Tailwind classes and shared keyframes in `styles/main.css`. Browser layout assertions added; 24 Rust source files changed substantively. Node tests 770 passing, 113 failing of 883; Rust/WASM/Trunk and Chrome acceptance pending. See root RELEASE_NOTES.md.

## R321 candidate - Independent game reducers (2026-11-11)

Unverified source candidate based on R320. Three separate domain states and Yew reducers replace MiniGame; Write Hanzi setup mode now handles LaunchKind::WriteHanzi. Node test results: 763 passing, 116 failing of 879. Cargo and browser acceptance pending; full file inventory is in RELEASE_NOTES.md.

## R276 - Yew UI callbacks, quiz integrity and compilation corrections (2026-11-08)

Source candidate based on R275: 20 substantive Rust source changes and one support cleanup cover domain Matching outcomes, MCQ option extraction, typed Hunt controls, shared category selection, and previous visibility/unreachable-pattern repairs. Practice Node 164/164 and full Node 631/642 (same 11 inherited failures). Rust/WASM and Chrome E2E unverified; wider source audit not finished. See root RELEASE_NOTES.md.

## R270 - Practice content and evidence integrity (2026-11-08)

Unverified source candidate based on R269. Authored exercise validity is checked before grading or answer rendering. Learning evidence rejects invalid local review dates before journal insertion, and versioned assessment/lexical identities receive stricter validation. Restored local UUID steps cannot pretend to have generated AI content. Node checks: Practice 164/164; full 626/637, same 11 existing failures. Twenty qualifying Rust files, no schema/dependency change; Rust/WASM/Chrome validation pending. See root `RELEASE_NOTES.md`.

## R266 - Practice setup UX consistency (2026-11-08)

Source candidate based on R265: top-of-step Back/Continue/Start, 15-card selector with derived count and selection checks, always-present category choices with bounded prefix search, summary icon/chip presentation, and explanatory loading status. No new dependencies, no database schema changes, no AI worker or scoring activation. Node: Practice 149/149; full 611/622 with the same 11 baseline failures. Rust/WASM/Chrome remain unverified. See RELEASE_NOTES.md and TASKS.md.

## R265 - Resumable mixed vocabulary hosts (2026-11-08)

A source candidate adds practice-only Matching, Multiple Choice and Listening Choice UUID hosts to supported bounded mixed sessions while preserving single legacy routes. It adds deterministic question preparation, strict score rejection, vocabulary normalization, Chrome source regression and updated launch eligibility. Rust/WASM and browser acceptance remain unverified. The AI worker, Character Hunt host and authoritative mastery publication remain outstanding; see root RELEASE_NOTES.md.

## R264 - Practice session integrity and evidence audit (2026-11-08)

R264 source candidate (not fully accepted): 21 substantive Rust changes cover UUID host readiness, unscored pronunciation, generation lifecycle, lexical identity and learning evidence integrity. Node 602/613; 11 inherited failures. Rust/WASM and managed Chrome not run. Remaining mixed hosts and AI worker are explicitly pending; consult RELEASE_NOTES.md and TASKS.md.

## R263 - Handwriting Recall (2026-11-08)

Added the fifteenth Practice exercise for hidden-character handwriting recall and self-comparison, with explicit practice-only evidence safeguards. Uses the existing ink-capture renderer and UUID session host. Node 597/608 with 11 inherited failures; Rust/WASM/Chrome unverified. See RELEASE_NOTES.md.

## R262 - Spot the Mistake and Sentence Builder build repair (2026-11-08)

Corrected the R261 Sentence Builder Yew closing delimiter and added a fourteenth exercise, Spot the Mistake, with separate phrase-identification and correction feedback, practice-only local sessions and documented browser tests. Node checks 596/607 passed; 11 inherited failures. Rust/WASM and Chrome verification pending. See root RELEASE_NOTES.md.

## R247 - Yew conditional fragment extraction (2026-11-07)

Extracted 33 substantial conditional render branches into named Rust view fragments. Updated structural source-contract tests (561 passed). Rust/WASM and Chrome execution were unavailable and require local verification. See root RELEASE_NOTES.md for the file-by-file qualifying inventory.

### R245 - Background presets and consolidated browser preferences (2026-11-07)

The six Appearance presets now select bundled scenic WebP backgrounds as well as
color palettes, with Lavender Classic as the first-run default. An explicit
`cargo make prepare-backgrounds` task validates bundled optimized assets and updates the inline
first-paint image; `cargo make install` includes this step. Input PNGs are
kept outside the deployed public directory. Startup restores saved colors,
background, display mode, density and Hanzi font before WebAssembly mounts.

Core UI preferences and AI URL/model are stored together under one JSON key,
`zipeek.preferences.v1`. Retired preference keys are removed after a successful
write. No migration from old preference formats is provided. Word Sets,
learning progress, exclusions, crossword sessions and developer logging
remain separate by design. Welcome entry now initializes the preference record;
no independent welcome marker is used. AI keys remain in tab memory only.

Node source-contract tests: 557 passed after the storage refactor. Rust, WASM, Trunk and managed Chrome E2E remain
unverified in the authoring environment. See top-level release notes for the
file inventory and local acceptance commands.

### R244 - Standalone practice vocabulary exercises (2026-11-07)

Added Type pinyin, Character builder and Word mosaics under Practice using the
existing vocabulary source selections. Separated vocabulary loading and tile
rules with focused unit tests. The source candidate includes 22 distinct Rust
file changes, no dependency/schema changes and no stored-data reset. Node
source-contract tests passed (546/546); Rust, WASM, browser and E2E checks
remain not run. See the top-level `RELEASE_NOTES.md` for the file inventory.

### R243 - Browser test screen contracts (2026-11-07)

Centralized MCQ option selection and post-answer lock assertions, reused the
contract across Endless, Fixed and Mixed test scenarios, moved the Pages
resource-path test into a named browser probe, and added more actionable
assertions for dictionary failures, loading, settings, responsive overflow,
route parsing, and invalid Word Set imports. JavaScript fixture access to
private round identities remains a known test-seam limitation. Native Rust,
WASM and Chrome execution require local validation. No production behaviour
or data schema changed.

### R242 - Unpack safety and browser test contracts (2026-11-07)

R241 already introduced UTC-timestamp release filename discovery. R242 prints
the selected archive and adds complete-source preflight, cleanup for rejected
stages, and attempted rollback of managed directories and replaced files if a
restore fails. Native tests cover stage manifests and recovery. Browser suites
reuse bound fragment navigation, Rust-side fixture inspection, typed saved Word
Sets storage reads and shared overflow checks, removing several interpolated
browser scripts. Node tests passed in the packaging environment; native Rust,
WASM, Clippy and managed Chrome checks were unavailable and remain outstanding.
No database schema or user storage formats changed.

**R239 development candidate (2026-11-07; Rust/WASM/Chrome checks pending):** Verified the root provider tree, made theme/database global access hooks concrete and fail-fast, removed optional-provider fallback branches, and organized the accessors by domain. Updated related theme and appearance markup and added a provider-contract regression. See [UI architecture](../architecture/ui-architecture.md).

## R238  -  Consolidated living UI documentation (2026-11-07)

Removed 13 superseded release-numbered acceptance/audit pages under `docs/testing/` and consolidated the currently applicable checks into [`crossword.md`](../testing-quality/e2e/crossword.md) and [`ui-acceptance.md`](../testing-quality/acceptance/ui-acceptance.md). Retained the R-series history in this file; preserved past test outcomes as historical observations, not current gate claims. No source, data or dependency changes.

## R237  -  Write Hanzi state reducer (2026-11-07)

Consolidated nine local state hooks into one typed workflow reducer with actions for source, searches, selection, and deep links. Node 499/539, including 40 pre-existing source-pattern failures after rustfmt; compiler/Chrome unverified.

## R235  -  Crossword controller refactor and release check (2026-11-07)

Extracted crossword setup, gameplay, results and Pinyin hook/popup into owned modules; centralized setup/loading/error transitions in a workflow reducer; fixed the callback ownership mistake that blocked R234 compilation. Added `cargo make release-check` and Node architecture checks. Node 537/537; Rust/WASM/Chrome **not verified**. See [current Crossword acceptance](../testing-quality/e2e/crossword.md).

## R234  -  Root scripts consolidation (2026-11-07)

Collapsed 54 top-level Node test files into 10 named subsystem suites, keeping 533 prior checks and their assertions intact. Root `scripts/**` now has 18 maintained files. Added one inventory regression test and `scripts/README.md`; Node 534/534. All existing root test commands and the database-only CI gate remain functional. Rust/WASM not rerun. See historical release notes in the dated handoff archive (if retained).

## R233  -  Cloud Match interaction and motion (2026-11-07)

Cloud Match setup/button simplification, score/life HUD feedback, shape-correct selection effects, four cloud silhouette variants and stable individual speeds; clouds remain on-screen while drifting. Browser tab visibility now pauses the active round and retains its state until explicit resume. Node 533/533; Rust/WASM/Chrome acceptance pending. See [current UI acceptance](../testing-quality/acceptance/ui-acceptance.md).

## R229  -  Crossword completed clue state (2026-11-07)

Completed clues are disabled, muted and ignored by reducer activations; solving the active clue clears selection. Filled cells are visually de-emphasized. Node tests 519/519; Rust/WASM/browser verification pending. See [current Crossword acceptance](../testing-quality/e2e/crossword.md).

# R228  -  Crossword clue-first entry (2026-11-07)

Activated clues now support selecting bank characters to fill the next unanswered square directly, with shared placement/error accounting, highlighted next cell, preserved drag/drop and direct placement. 516/516 Node tests passed; Rust/WASM/browser pending. See [current Crossword acceptance](../testing-quality/e2e/crossword.md).

### R226  -  Crossword connectivity search (2026-11-07)

Ranked the HSK-filtered dictionary pool by shared-Hanzi connectivity before retaining 128 entries; generated crossing-position candidates directly, increased the board search area and seed retries, and retained the six-word minimum. Fixed canonical ordering for seeded session replay, widened saved-game validation and bumped development-only crossword storage to v5. Reworded the misleading “choose more levels” generation failure. Node source suite 509/509 passes; Rust/WASM/Chrome acceptance remains pending. See [current Crossword acceptance](../testing-quality/e2e/crossword.md); these notes describe an earlier checkpoint.

### R224  -  HSK Crosswords and completion view (2026-11-07)

Introduced an HSK 2.0/3.0 multi-band setup tied to existing DuckDB practice queries; replaced static crossword candidate words with validated dictionary entries and a seeded bounded placement search. Simplified clues, enlarged bottom tile bank, removed redundant instructions/numbers, and introduced a separate completed-game summary with Play again, Change HSK levels and All games actions. New v4 development session snapshots persist the HSK bands and vocabulary to reproduce generated puzzles. Node source-contract tests: 503/503 passed. Rust/WASM/Chrome require local verification. See [current Crossword acceptance](../testing-quality/e2e/crossword.md).

## R223  -  Full-screen crossword and cropped board geometry (2026-11-07)

The crossword now takes the full desktop viewport independently of shell layout, crops unoccupied search-grid rows/columns, constrains board geometry to the available height and panel width, and centers Hanzi without glyph overlap. Generator semantics and saved cell coordinates are unchanged. Added R223 regressions and revised the legacy square-board assertion. Node tests: 498/498 passing. Native/WASM compilation and Chrome visual acceptance remain unverified; the 13-word generator can still produce sparse layouts.

## R220  -  Lazy vocabulary loading, path alignment and Yew compile fixes (2026-11-07)

Removed the R219 setup-time exclusion-aware exact dictionary scan, debounced timer, six answer calculations and page eligibility guard. Practice Setup now validates only the selected draft, formats, source, and saved Word Set snapshot; each launched exercise performs vocabulary loading and reports inadequate pools through its existing loading/error path. This avoids doing an expensive whole-dictionary pass for every HSK toggle and again at practice startup. Removed the obsolete `PoolPreview` and associated hook, adapted setup tests and preserved runtime HSK/category/PoS filter encoding and exclusions. Summary no longer reports unverified eligibility counts or pending scans; Back and Start remain together. Path cards align icon and heading. Fixed Character Hunt audio `<ListenButton hanzi={word.hanzi}/>` for Yew prop type inference. Added focused regression checks and updated E2E expectations. Node checks passed; Rust compilation and browser execution remain to be verified.

### R219  -  Responsive HSK multi-selection and unified Summary checks (2026-11-07)

Moved the exact dictionary eligibility preflight from the transient Vocabulary tab component into the persistent Practice Setup page, with 420 ms debouncing, timer cancellation, and generation-scoped stale-result protection. The preflight remains exclusion-aware, uses the existing shared eligible-answer computation, and continues when navigating from Vocabulary to Summary. HSK band toggles and standard switching now dispatch typed reducer actions against the latest draft instead of writing captured source snapshots, preventing rapid changes from losing earlier clicks. The legacy single-band round query level stays synchronized with the selected draft level. Summary explicitly distinguishes pending, ready and failed eligibility checks, with Back and Start together in its existing footer (the separate summary controls shown in older screenshots were removed in R218). Added reducer tests, a managed Chrome rapid-toggle-to-Summary journey, and source-contract assertions. No database schema or dependency changes. Rust/WASM and Chrome execution remain unverified; Node suite passed.

### R214  -  Offline Quick Character Hunt, contextual readings and guided gameplay (2026-11-06)

### R218  -  Consistent practice actions and Character Hunt learning panel (2026-11-07)

Consolidated Summary Back / Start practice within one review card, removed the
extra Summary footer, suppressed unselected (zero-count) vocabulary filter
labels, and replaced the duplicated Character Hunt difficulty / grid-size
summary with a single difficulty label. Character Hunt's page now uses the
existing Yew grid icon, concise instructions and a compact round counter.

Moved Hint / Reveal / Exit / Continue into one bottom action bar with a
confirmation-preserving Exit component. Removed the redundant Clear selection
button and its unused UI reducer action; tapping a selected tile already
reverses selection. Moved Next round out of the Progress panel; completed words
now have the shared ListenButton and dictionary meanings. Replaced the native
green progress element with an accessible blue progressbar. Constrained the
practice layout on desktop so the completed-word panel scrolls independently
while the board and actions remain in place. Updated the Chrome solver to
reselect tiles instead of using the removed Clear control.

Node source suite: 486/486 passed. Rust/WASM compilation, Chrome execution,
and visual viewport acceptance remain unverified in this environment.


Moved Character Hunt into Quick Practice as well as Advanced without pretending
it participates in the scored mixed-session planner. Quick mode now switches
explicitly between standalone Character Hunt and Matching/MCQ. Replaced the
in-game “Back to setup” link with a separate Exit practice confirmation, and
kept gameplay controls focused on Clear selection, Hint and Reveal a word.
Sourced per-Hanzi readings from dictionary word context, omitted ambiguous
pronunciations instead of guessing, and extracted a typed HuntTile Yew
component. Added progressive temporary spatial hints after repeated mistakes,
explicit assisted reveals, and domain/reducer/E2E regression cases. No AI,
dependency upgrades or dictionary schema changes. Node suite passed; Rust/WASM
and Chrome validation remain required on Windows.

### R213  -  Typed Character Hunt session and component architecture (2026-11-06)

Replaced the positional `character_hunt_progress::view(board, feedback, ...)`
helper and the sibling `character_hunt_*.rs` files with a conventional
`pages/learning/games/character_hunt/` module. `CharacterHuntPage` only handles
route configuration and asynchronous candidate loading; `HuntSession` owns one
`use_reducer` state machine and provides a scoped `HuntSessionContext` to
actual Yew board, controls, progress and summary components. All UI events
are typed reducer actions. Round advancement now builds the next board
**before** committing the round number or cumulative stats, preserving the
completed board if generation fails. The vocabulary source is a typed enum
(Word Set, HSK, encoded filters) instead of magic strings.

Also split the 407-line pure `zipeek-exercises::character_hunt` module into
`model`, `generation`, `gameplay`, and `tests`, preserving its public API,
deterministic puzzle format, and existing unit tests. Added architectural
regression checks and expanded the Yew coding standards. Node suite: 468/468
passed. Native Rust, WASM compilation, and browser gameplay remain unverified
in the packaging environment; R213 is a development candidate.

### R212  -  Character Hunt compilation fix, solvable generation and Fixed 5 gameplay (2026-11-06)

Corrected the missing `zipeek_exercises::character_hunt::Candidate` import in the Character Hunt page that blocked the R211 WASM build. Extracted board rendering and the progress/summary panel into separate Yew view modules, retaining the approved 2×2 through 5×5 Hanzi font sizes and accessible tile state. The deterministic engine now retries alternative word combinations before rejecting a vocabulary pool, preventing avoidable dead ends; added unit coverage for this case. Added a non-penalizing Clear selection action and cumulative Fixed-session rounds/words/mistakes totals. Extended the existing Chrome journey to attempt all five Easy rounds through the public UI and assert the end summary. Source tests pass; Rust/WASM/Chrome execution still requires the Windows toolchain.

### R211  -  Typed compilation corrections, full Character Hunt filters and clean source unpack (2026-11-06)

Corrected R210 `zipeek-web` compilation errors from forwarding `AiClientError` into `String`-typed error paths and from an omitted `Route::CharacterHunt` title match arm. Extended Character Hunt setup-to-route-to-database vocabulary filtering for multiple HSK bands, categories, and PoS, with one shared parameterized predicate and a 1,024-entry bounded two-/three-character sample. Saved Word Sets continue to resolve exact IDs. Rejected invented filler Hanzi in undersized puzzle inputs. Refined `zipeek-cli unpack` to replace/back up crate `src/`, `js/`, and `tests/` directories, preventing orphaned JS transports and obsolete integration tests. Added database, engine, unpack, E2E and Node regressions. Node: 460 passed; native Rust, WASM and Chrome still require Windows verification.

### R210  -  Image question reliability and cross-target AI client fix (2026-11-06)

Corrected Picture Vocabulary's three broken domain tests: their `object N` glosses were filtered by the actual image-eligibility rules. Migrated question implementation to `image_questions/{model,eligibility,generation,tests}.rs`, strengthened deterministic/distractor/duplicate validation, and improved on-screen failure diagnostics. Removed the unsupported WASM `reqwest::ClientBuilder::timeout` call without weakening the five-minute native timeout; CI now checks `zipeek-ai-client` explicitly on wasm32. The browser request deadline remains an open item. Node suite passes; native Cargo/WASM/Chrome tests must still be run.

**R207 (2026-11-06, development candidate):** Reqwest SSE streaming for local AI translation and structured analysis, five-minute request limit, typed callback for incremental content, fragment-safe streaming parser, live test progress and network fixtures. Rust/WASM and live-model acceptance pending.

### R206  -  Explicit web target boundary for native workspace checks (2026-11-06)

Addressed the native compile errors `E0432/E0433` caused by importing
`zipeek-browser::microphone` and `zipeek-db::wasm` from the `zipeek-web` library
when compiling for the host. The Yew application library and browser entry
point now compile for `wasm32`; a small host-only entry point makes workspace
checks possible without exposing browser APIs on native targets. Added
source-regression tests for both entry points and the existing target gates.
No microphone, DuckDB, AI request or application runtime behavior changed.
Rust native and WASM compilation remain unverified in this authoring environment.

### R205  -  Native/WASM reqwest AI transport and complex-sentence fixtures (2026-11-06)

Replaced the R204 browser-only `fetchChat` bridge with `reqwest 0.12.28` in `zipeek-ai-client`. The crate now has small modules for typed wire payloads, prompt contracts, URL validation, error diagnostics and HTTP operations. The browser still maintains URL/model preferences and tab-memory keys; it does not construct or execute HTTP requests in JavaScript. Native uses a no-redirect policy; WASM browser Fetch cannot configure redirects equivalently, so a final-URL check rejects changed responses after the fact.

Added real native-loopback HTTP integration tests using complex Chinese/English sentence fixtures, including schema-validated grammar/vocabulary observations, actual serialized requests, credential handling, error diagnostics and a structured-400/plain-translation recovery case. Added ignored opt-in tests for real-model translation and analysis in both directions. All 442 Node source regression checks pass; native Rust/WASM, the Rust integration tests, and Chrome were unavailable here and remain **unverified**.

## R201  -  Microphone visibility and structural refactors (2026-11-06)

Corrected `E0423` in microphone capture using a module-visible constructor rather
than a public tuple field. Extracted generator classification response parsing,
LLM batch planning/chunk validation, their tests, CLI vendor tests, Cloud Match
cloud tiles, and the Appearance settings panel into dedicated modules. The
original public interfaces and runtime policy are intentionally unchanged.
Updated source assertions to follow the new module boundaries. Node tests:
435/435 passed; Rust/WASM, Trunk and browser acceptance pending.

## R193  -  Native Downloads release selection (2026-11-06)

`cargo make unpack` now finds the highest-numbered `zipeek-R<number>-description-YYYYMMDD.zip`
in the user's Downloads directory, using date and modification time as tie-breakers.
The native CLI still accepts an explicit archive path for `target/` snapshots or other ZIPs.
`cargo make pack` continues to generate `target/zipeek-source-<version>-<timestamp>.zip`.
Existing extraction security checks and managed-source backups are unchanged.
Rust/Windows execution remains pending validation in the release environment.

## R192  -  Build-pipeline, source snapshots and static preloads (2026-11-06)

Renamed active workspace tooling to the `zipeek-*` layout. Source archives now
use the `zipeek-source-<version>-<UTC-YYYYMMDD-HHMMSS>.zip` convention; native
unpack selects the latest matching archive. `zipeek-cli db-install` verifies
bundle checksums and pinned generator inputs, stages an installation, updates
manifest artifact names, publishes the static dictionary and keeps the HTML
preload filename synchronized, with backups and rollback on publication
failure. The HTML now contains literal manifest and gzip fetch preloads, with
runtime JS insertion removed; Yew continues to verify and open the dictionary.
Node source tests 407/407 passed in the review environment. Rust, WASM, Trunk,
Windows native installation and browser acceptance remain unverified.

## Landing-page source release (2026-11-06)

Removed the welcome tagline and footer, changed the Enter Zipeek button to use the Yew Lucide play icon, and replaced decorative Hanzi art with a static 16:9 product preview for Practice, Browse and Word Sets. Existing welcome persistence and deep-link entry behavior are unchanged. Updated the landing contract and focused source tests. Node source suite: 401 passed. Rust/WASM/Trunk and browser validation were not available in the packaging environment.

## R70 web acceptance  -  first local run

This is a manual acceptance checklist, not a claim that Windows, WASM, Chrome,

## R144  -  Cross-cutting quality and E2E coverage (2026-11-05)

`scripts/db-workflow.test.cjs` now constructs a temporary **synthetic** validated bundle with the pinned engine/storage/WASM values, a copy of the real input manifest, checksums and the required attribution files. Synthetic `.duckdb` bytes are intentionally **not** a runnable DuckDB: these tests verify manifest validation, staging, replacement and rollback, not SQL semantics. All filesystem fixtures are isolated in unique OS temporary directories and removed by Node test cleanup.

## R145  -  Practice launch state ownership and E2E regression (2026-11-05)

Extracted a pure `services/practice_launch.rs` launch-policy boundary from `pages/practice_setup.rs`. The same `apply_vocabulary` function is now used by both ordinary Matching/MCQ launch and finite planned-session launch. It translates the selected `VocabularySource` into an immutable query snapshot. Word Set selection is validated before modifying any URL fields. Invalid, empty, missing, or inaccessible sets reject launch **atomically**, without leaving an incomplete `set_ids` query. Existing dictionary filters have their encoded source prese

## R146  -  Compiler recovery and full-suite CI gate (2026-11-05)

Windows compiler diagnostics for R145 were provided by the user, not reproduced in this environment. Node source tests were executed on the R146 working tree; see release handover for the exact result.

## R147  -  Practice flow refactor (2026-11-05)

**Quick Practice:** Matching and Multiple choice, individually or together, without an AI backend. The default is both selected and Endless mode. Finite 5/10/20 uses the existing guarded planned-session runtime when a valid plan is available. Endless uses the established URL-encoded free-play route and retains round variant selections. **Advanced Practice:** Sentence translation (requires a local AI backend) or the unscored Speaking recorder. This checkpoint allows one advanced activity at a time because no cross-type scored/AI session engine e

## R148  -  Feature-based module layout and icon inventory

Fix the invalid `IconData::LUCIDE_SPARKLES` in the R147 Practice selector by Group Practice components under `components/practice/{setup,exercises,session}`.

## R149  -  Classification prompt consolidation (2026-11-05)

Classification system and user prompts now live in `crates/zipeek-ai/src/prompts/classification.rs`. The same module owns the supported PoS/category codes and their taxonomy version. `db-gen` retains dictionary types, request dispatch, response parsing, validation, source hashing, checkpointing, and database writes. A borrowed `ClassificationInput` maps each `(Word, SourceSense)` into a portable prompt input.

## R150  -  Translation module compilation recovery (2026-11-05)

R149 reported E0583 for the four child modules of `pages/translation/sentence_translation.rs` after the R148 page relocation. The sources existed under `pages/translation/sentence_translation/`, but Rust attempted to find them directly under `pages/translation/`. The parent now uses explicit `#[path]` directives for `actions`, `settings`, `state`, and `view`, retaining the existing public `pages::sentence_translation::SentenceTranslation` contract.

## R151  -  Translation module and state refactor (2026-11-05)

Replace the R150 four `#[path]` overrides with standard Rust nested modules: Keep public `pages::sentence_translation::SentenceTranslation` and adjacent translation page imports available via the existing `pages` re-exports.

## R152  -  Practice UI alignment and translation module naming

Rename the standalone Translate tool module `pages/translation/sentence_translator.rs`, with component `SentenceTranslatorPage`. Rename the AI-powered Practice exercise to the conventional `pages/translation/sentence_practice/mod.rs` module, with component `SentencePracticePage` and regular `mod actions`, `mod settings`, `mod state`, and `mod view` child declarations.

## R153  -  Practice typed steps and panel extraction

Fix E0425 in `pages/practice/setup.rs`: `has_advanced_exercise` is computed before the `unified_eligible` gate uses it. Keep one definition. Replace the wizard's stringly-typed `path`, `exercises`, `vocabulary`, and `summary` state with `PracticeStep`. Navigation callbacks now use `Callback<PracticeStep>`; previous/next derive from the same ordered enum.

## R154  -  Practice setup state and launch policy

Native Rust, wasm32, Clippy, Trunk and browser E2E: **not run** in the

## R155  -  Crossword workspace and persistent local sessions

Crossword gameplay uses a focused workspace; the standard navigation drawer and footer are not shown for this route. Header navigation remains available. Board, clues, and tile bank are separate components in `components/games/crossword/`.

## R156  -  Crossword module layout compilation correction

Move `pages/learning/crossword.rs` to `pages/learning/crossword/mod.rs`. Declare `pub mod crossword;` in `pages/learning/mod.rs` and expose the previous public name via `pub use learning::crossword;` in `pages/mod.rs`.

## R157: Crossword build recovery

Corrected `CrosswordClues` event handler types. Mouse enter/leave callbacks accept `MouseEvent`; focus/blur callbacks accept `FocusEvent`. This resolves the E0277 errors caused by reusing mouse callbacks for focus listeners. The shipped module layout uses only `pages/learning/crossword/mod.rs` with conventional child `state.rs` and `storage.rs` modules. `pages/learning/crossword.rs` must **not** coexist with that directory's `mod.rs`.

## R158  -  Workspace Rust module integrity and stale source cleanup

Retain `pages/learning/crossword/mod.rs`, `state.rs`, and `storage.rs`; remove obsolete `pages/learning/crossword.rs`. Retain `pages/translation/sentence_practice/mod.rs` and `sentence_translator.rs`; remove obsolete `pages/translation/sentence_translation.rs`, its retired directory, and `translate_sentence.rs`.

## R159  -  Theme action colours and daily-word identification

Keep `--accent`/`--accent-soft` for non-solid accents and selection surfaces. Expose `--brand-action` and `--brand-hover` as Tailwind semantic colours.

## R160  -  AI model default and Practice mode order

Updated the **db generation workflow default** model to `qwen3-14b-claude-4.5-opus-high-reasoning-distill` in `scripts/db-ai-workflow.cjs`, its tests and current regeneration instructions. Left browser AI Settings model storage untouched: it remains a user-configured string in `localStorage` and must not be silently overwritten by a release.

## R161  -  Yew codebase-wide quality sweep (2026-11-05)

R161 adds new source contracts for reducer ownership, pure state transitions, Four native Rust reducer unit tests were added but **not executed**.

## R162  -  Practice Session Completion and Matching UX

Correct Settings module resolution by making `pages/system/settings/mod.rs` the conventional parent of `ai_state.rs`; remove obsolete `settings.rs`. Automatically acknowledge recorded planned exercise results through the existing `PlannedRuntime` transition. Do not expose `AwaitingAcknowledgement` as a learner-facing screen. Preserve generation, instance and definition guards and reject duplicate publication.

## R163  -  Connected Crossword Workspace

Removed the repetitive `2 characters` suffix from crossword clue labels. Clues show their number, direction, English gloss and pinyin. Replaced the prior two disconnected word chains with **one 13-entry connected crossword**. `法师` connects `想法` to `老师`, with an `老人` → `人生` → `生日` → `日记` continuation. Shared Hanzi occupy **one cell** and update all intersecting clues.

## R164  -  Environment-scoped browser storage and Crossword build recovery

Fix `CrosswordState::Reset` by initializing `last_crossing: None` (E0063). Remove unused `HtmlElement` import from Matching and unused root `PracticeVocabularySource` re-export.

## R165  -  Practice compile recovery, environment access, Browse filter UX

Fix the Practice setup `personal_sets` initializer after R164 moved `word_sets::read` to an injected-storage API. Initialize using `use_app_environment()` and read `Result<Vec<UserWordSet>, String>` through that storage handle. Pass the `Result` value, not the Yew state handle, to `practice_launch::apply_vocabulary`.

## R166  -  Standalone Translate Sentence workspace

Direction is a two-tab control in this order: English → 中文 (default), 中文 → English. Exactly one editable source textarea. Its language is conveyed by placeholder and accessible label; no visible “English sentence” heading.

## R167  -  Audio caller and format label corrections

`ListenButton` obtains the `Window` handle through `use_app_environment()` and passes it to `practice_audio::speak`, fixing E0061 after the explicit browser-capability refactor. User-facing setup, summary, and vocabulary eligibility labels consistently call the Chinese-character-to-definition direction `Hanzi → Meaning` in both Matching and Multiple Choice. Internal `ExerciseVariant::HanziEng` identifiers and existing routes are unchanged for compatibility.

## R168  -  Practice domain and router ownership

`ExerciseVariant` variants now identify **both** the mechanic and question direction (e.g., `MatchingHanziMeaning` versus `MultipleChoiceHanziMeaning`), avoiding the previous ambiguous `HanziEng`/`HanziToMeaning` identifiers. `ExerciseMechanic` and `QuestionFormat` are independent typed enums. Both Hanzi-to-definition activities resolve to `QuestionFormat::HanziToMeaning`.

## R169  -  Practice variant compilation fix

Native compilation is not verified in this environment.

## R170  -  Script consolidation and Rust AI workflow

`scripts/` reduced from 109 paths (105 tests, three executable scripts, one fixture) to **15 paths** (12 test suites, two executable scripts, one fixture). The old `db-ai-workflow.cjs` is retired. Test cases were preserved and grouped by domain (database, Practice, translation, games, UI, platform and legacy coverage), with file-local scopes so originally duplicated imports and helper names do not collide. No additional test runner or dependency was introduced.

## R171  -  Translator analysis repair and retired script cleanup

Preserve the original translation-and-analysis request through the configured AI model. If structured output is invalid, send **one** correction request containing the parser's diagnostic and a bounded excerpt of the previous response.

## R172  -  Translator Controls Component Split

`direction_tabs.rs` owns the two translation direction tabs. `sentence_input.rs` owns the sentence textarea, character count and Clear action.

## R140 product workspace notes

Historical design checkpoint; details remain in Git history.

## R174  -  Expanded classification taxonomy, prompt contracts, rejected-response evidence (2026-11-05)

Expanded the category taxonomy to 62 precise semantic subjects (taxonomy version 2), aligned the DuckDB category seed table, and revised sense-level prompts with explicit grammatical/topic boundaries and strict JSON-only output requirements. Cache identity is now `sense-classification-v6` / `-v6.jsonl`, intentionally invalidating earlier classifications. Tightened the sentence-generation, translation-evaluation and simulated-learner output requirements. Rejected raw classification responses are appended to a local JSONL diagnostic file, separate from the classification cache. Native Rust/LM Studio verification is required.

## R175  -  Aimux output schema and UI feature boundaries (2026-11-05)

Classification requests now specify an Aimux output schema and retain raw rejection diagnostics. Ten large UI features were split by responsibility; Rust build and live model acceptance remained outstanding.

## R176  -  Correct module paths and extract ten further UI responsibilities (2026-11-05)

Corrected two E0583 failures by converting the Write Hanzi and Personal Storage parents to standard nested `mod.rs` modules, preserving their existing child files. Removed the two unused bindings reported by Windows Cargo. Extracted Browse filtering and word relations, Practice step metadata and exercise selection, microphone recorder bindings, app-shell header, sentence available tiles, Word Set backup, and the Matching and MCQ keyboard effects into independently owned child modules. No new providers or state stores were introduced. Node source regression tests and normal Rust module-resolution checks were run; Cargo/WASM compilation remains required on Windows.

## R177  -  Large batch smoke and narrow JSON suffix recovery (2026-11-05)

`cargo make db-ai-smoke` now sends one single-sense request followed by a **100-sense batch** sampled deterministically from verified imported dictionary data. It does not write a classification cache, database or published bundle. `db-full` retains a small preflight matched to its configured classification batch size (default 2), so the production workflow is not unnecessarily forced to use 100-sense requests. Classifier output token allowance scales with requested batch size, capped at 16,384 tokens. Aimux's structured output remains the primary parser; a fallback accepts only otherwise-valid JSON with one or two trailing full stops, followed by normal strict validation. Responses containing reasoning prose, incomplete JSON, or unknown labels remain rejected and logged. Native Cargo and live LM Studio acceptance remain outstanding.

## R178  -  CC-CEDICT gloss classification dataset (2026-11-05)

Exported all 125,320 CC-CEDICT 2026-10-26 entries as 199,699 individually addressable gloss records in `crates/db-gen/data/classifications/cc-cedict-2026-10-26-classified.json`. This **is not a ChatGPT-reviewed or LLM-generated classification corpus**: `scripts/cedict-classify.cjs` applies conservative reproducible English-gloss patterns and marks uncertain cases for review. Preserve original source line, gloss index, traditional/simplified headwords, pinyin, English gloss, suggested PoS and controlled taxonomy categories. The JSON array is accompanied by checksum/quality metadata; do not import it directly as db-gen HSK classification checkpoints, since IDs and digests differ.

## R179 (2026-11-05)  -  CC-CEDICT rules to database import

`db-full` defaults to offline import of R178 rule-derived CC-CEDICT labels, validated exact joins to HSK senses, explicit unclassified placeholders and the existing atomic bundle publisher. Retains optional `ZIPEEK_DB_CLASSIFIER=lmstudio` mode and separate AI smoke. Classification quality remains unreviewed.

## R180  -  Native image service scaffold (2026-11-05)

Added opt-in `zipeek-image-server` using Axum 0.8.9, Tokio 1.54.1 and a typed asynchronous job API with deterministic SVG placeholders and disk storage. No image model is bundled, no frontend integration is claimed. The new crate needs Cargo.lock resolution and Rust compilation on the developer machine. See [Image server](../build-deployment/services/image-server.md).

## R181  -  independently staged database classification (2026-11-05)

Added `cargo make db-classify`: offline CC-CEDICT cache import/validation and
creation of a new enriched candidate without installation. Manual sequence is
`db-generate` → `db-classify` → `db-install`. `db-full` continues to perform
classification, candidate generation and atomic installation together.
The initial candidate is retained, and the latest-candidate pointer advances
only after verified generation. New Rust tests cover classify-only sequencing,
missing coverage, and no premature installation. Native Cargo validation is
still required.

### R182  -  Independent database stages and DuckDB DLL safety (2026-11-05)

Fix Windows `Access is denied` during `cargo make db-classify`: the previous `db-gen` coordinator launched `node generate`, which launched another `cargo run -p db-gen` while the first process still held DuckDB. Classification now validates and selects a cache only, exits, and the Makefile launches generation as a separate task. `db-classify` chains cache preparation and enriched generation; `db-full` chains classification and verified installation. A versioned marker under `target/db-workflow` selects only a validated cache. `db-generate` remains a base-only candidate operation.

### R185  -  Image vocabulary exercise and five web refactors (2026-11-06)

- Added pure `zipeek-exercises::image_questions` planner: reproducible seeded selection, one-of-three and select-all grading, vocabulary-defined answer keys.
- Added experimental `#/practice/image-vocabulary` route, reachable from advanced Practice setup: uses saved Word Sets, the configured Axum API, asynchronous job status, image approval, and answer feedback. This page is **not yet scheduled in mixed sessions**.
- Extracted focused UI modules from sentence translation progress, Word Spotlight example feedback, Matching statistics, MCQ statistics, and Hanzi Writing default targets.
- Source-contract test suite: 388 passing. **Rust, WASM, live image inference, and browser E2E not run in the packaging environment.**
- Physical DuckDB compaction remains future benchmark work, not shipped in this release.

## R190  -  DuckDB-WASM Base assets and Yew refactors (2026-11-06)

Updated the 1.33.1-dev114.0 asset preparation from legacy MVP/EH filenames to `duckdb-browser-base.worker.js` and `duckdb-base.wasm`. Both the main database bridge and personal OPFS probe now configure the Base bundle; experimental threaded execution is deliberately not enabled. Recorded the two explicitly approved npm install scripts. Extracted Settings navigation, Cloud Match HUD, Practice Setup translation source, Hanzi Writing database search, and Image Vocabulary review panel into conventional adjacent modules. Node regression suite: 398/398 passing. Rust/WASM compilation and live browser database opening were not executed in the authoring environment.

### R195  -  Native database extraction and standalone exercise activation (2026-11-06)

Moved the existing native DuckDB environment, version gates and query error
handling to `zipeek-db/native` with a compatibility re-export in the generator.
Fixed leftover `zipseek_db_gen` import names. Added pure deterministic listening
question preparation and grading, with a Yew standalone listening page using
browser Mandarin speech synthesis and saved Word Sets. Promoted existing
Picture vocabulary to a selectable Advanced Practice exercise while retaining
its image-server/approval workflow. No mixed-session scheduling, persistent
listening scores, or database schema changes. Browser/Rust acceptance pending.

### R197  -  Typed browser/database boundaries (2026-11-06)

Migrated the microphone recording lifecycle into `zipeek-browser` using Rust `web-sys`. Introduced typed `BaseBundle`, `Arrow(JsValue)` and `DuckDb(Rc<Handle>)` in `zipeek-db/wasm`, with the pinned DuckDB module and worker interop in `zipeek-db/js/`. Moved the isolated personal OPFS probe's SQL, checkpoint and reopen/verification sequence into Rust; a small JS shim retains vendor calls and Web Locks callback bridging. Removed the old web-owned microphone, dictionary bridge and probe JavaScript modules. Updated Node contract and transport tests (422/422 pass). Native/WASM compiler, browser microphone and OPFS round-trip verification are still required before release acceptance.

R197 unpack cleanup also marks `crates/zipeek-web/scripts` as a managed restore directory (backed up before replacement), so obsolete `prepare-duckdb.cjs` is not left behind after installing source releases. Other private or generated paths remain excluded.

### R198  -  Icon-first exercise selection cards (2026-11-06)

Changed Advanced Practice exercise cards to use the exercise icon within the
left selection square instead of a separate checkmark. Selected cards receive
an accent border, subtle ring and stronger surface; unselected cards remain muted.
The entire card continues to be a toggle button with `aria-pressed`, existing
exercise draft state and launch behavior. Added visual-contract source regression
tests. Rust/Trunk and visual browser acceptance remain required.

### R199  -  Exercise tile extraction and microphone callback fix (2026-11-06)

Fixed the `BlobEvent` closure type inference failure in `zipeek-browser`. Extracted the icon-first selectable exercise card into `exercise_tile.rs`, preserving `PracticeDraft` selection, the whole-tile button, accessibility, and existing data hooks. Added a `data-selected` inspection hook and updated source regression tests. Rust/Trunk compilation and browser interaction still require local validation.

### R200  -  Microphone API correction and Rust module boundaries (2026-11-06)

- Adapted microphone `BlobEvent::data()` to its optional Blob contract, ignoring
  null/empty chunks rather than panicking, and pushing valid `Blob` references
  to the JS buffer array.
- Extracted microphone recorder, state, capture, waveform, and browser error
  classification into dedicated Rust modules; retained the public tuple wrapper,
  generation protection, callbacks, and cleanup behavior; recording phases are
  now a typed Rust enum and only converted to strings at the Yew callback boundary.
- Extracted vendor preparation into Rust input validation, fingerprints,
  preparation/publication, and explicit npm installation modules.
- Split practice-draft models, methods, tests, and Practice session reducer
  key/stats, round handling, transitions, and tests into feature modules.
- Updated source regression mapping to traverse the new module boundaries.
- Node source suite: 426/426 pass. Cargo, WASM, and live browser acceptance
  were not executable in the packaging environment and remain mandatory.

### R202  -  Practice progress and exercise stages (2026-11-06)

Replaced large step tabs with an accessible progress rail (earlier visited circles clickable, future circles locked), separated Choose and Configure exercises with icon-labelled subtabs and sequential footer navigation, defaulted to Fixed 5, and added navigation source/reducer regression tests.

### R203  -  Chrome learner journeys and practice setup syntax correction (2026-11-06)

Fixed the unclosed `html!` / conditional delimiter in `practice_setup_panels.rs` introduced by R202's Choose/Configure split. Added six named WebDriver scenarios under `zipeek-e2e/tests/suites/learner_journeys.rs`: interactive practice progress and configuration, landing-page tile navigation, secondary page routing, Word Set renaming/switching/deletion, Browse filter+audio click+save, and Quick Summary Fixed 5 default. Registered each in the managed Chrome root suite and refreshed the stale Home tile/Random page assertions in the responsive shell scenario. Recorded the complete 13-journey acceptance matrix and distinguished existing tests from pending fixture work (notably a true five-round advanced AI run and a separately persisted default Word Set). The existing CDP network-failure technique remains the bootstrap failure injection method; no production query-string failure flag was added. Node regression suite: 441/441 passed. Rust/WASM compilation and Chrome execution were unavailable at packaging time.

### R204  -  Typed AI client, translation diagnostics and E2E migration (2026-11-06)

Added `crates/zipeek-ai-client` with a single-field typed client wrapper,
translation direction/validation/prompt construction, and connectivity
contracts. Migrated `zipeek-web`'s AI calls to the Rust client and removed
`services/ai/bridge.js` in favor of a narrow fetch/credential/timeout adapter.
Structured-analysis HTTP 400/422 fallback still supplies a plain translation,
but reports the provider's bounded JSON error message/code where available;
non-JSON error bodies are suppressed and common key patterns redacted. The
screenshot's successful translation alongside HTTP 400 corresponds to this
fallback, not a successful structured analysis.

Added deterministic AI fixture modes and Chrome scenarios for both supported
structured analysis and reject/fallback translation. Migrated old Practice
setup E2E cases to the R202 interactive progress rail and exercise subtabs,
plus relevant translation fixture selectors. Updated source regressions and
AI architecture documentation. Node suite: 445/445 passed, dedicated
transport checks passed. Rust/WASM compilation and WebDriver tests were not
available in the packaging environment; **R204 is not a certified build**.

### R207  -  Incremental AI completions and accessible progress
The reqwest client requests SSE and assembles fragmented UTF-8 chunks with bounded buffering. Reasoning-model deltas report activity without exposing reasoning text. Translate Sentence shows phase changes and character counts for analysis, repair and fallback. Sentence Translation exercises show progress during generation and evaluation. The 30-second client deadline was replaced by an overall five-minute bound. The managed browser AI fixture serves SSE for translation, generation and evaluation; a Chrome scenario checks progress before the final result. Node validation passes; Rust/WASM and browser execution remain unverified.

### R208  -  Typed AI protocol, comprehensive language contract and structured JSON schema (2026-11-06)

Introduced serde-backed `ChatMessage`/`ChatRole` and structured `ResponseFormat` request models in `zipeek-ai-client`, plus typed SSE choice/delta decoding. Kept existing serializable-message caller compatibility. Moved the comprehensive bilingual translation/linguistic instructions and provider JSON Schema into focused `zipeek-ai/sentence_analysis/` modules; all ten JSON analysis fields now require explicit presence, even when arrays are empty. Translate Sentence attempts strict structured JSON streaming and retries prompt-enforced structured analysis when a local provider rejects the response-format parameter; the existing repair/plain translation paths remain available. Added native integration fixtures for complex sentences, schema/validation rejection cases and HTTP response_format serialization. Node regression checks: 449/449 passed. **Cargo/WASM/native HTTP/live-model/Chrome tests not executed in the packaging environment.**

### R209  -  Character Hunt, responsive Hanzi and vocabulary layout (2026-11-06)

Added pure Rust `CharacterHunt` board generation and verdicts, a typed difficulty in PracticeDraft (Easy 2×2, Medium 3×3, Difficult 4×4, Hardcore 5×5), Yew Configure controls, a dedicated multi-round practice route and setup launch using an HSK band or saved Word Set. Board generation is deterministic, marks completed cells gray, includes bounded hints and fails explicitly for inadequate word pools. Larger Hanzi on easier grids are deliberate. Mixed-engine round scheduling remains unsupported. Vocabulary now has Source & HSK / Categories & filters subtabs and shows eligibility information in Summary instead of an editor-sized count panel. Added pure Rust unit tests, a browser launch journey, and source regression tests; Node suite 454/454 passed. Cargo, WASM, real Chrome and full session completion remain unverified.

### R215  -  Character Hunt completed-word meanings and unobtrusive hints (2026-11-06)

Carries dictionary glosses through the Character Hunt candidate and answer models, and presents completed words as Hanzi with dictionary-aligned word-level pinyin (when available) and English meanings. Handles multiple source definitions without inventing translations; reports missing definitions explicitly. Matching feedback identifies the completed word. Replaced recurring pulsing hints with one-off third/fifth-miss spatial highlights that dismiss instantly when the learner taps a tile; manual Hint cannot interfere with a partially entered answer. Reduced live announcements to the feedback text. Added domain/reducer/source tests and browser assertions. Node checks: 473/473 passed. Rust/WASM compilation and Chrome E2E remain unverified.

### R216  -  Crossword board, Browse pagination and AI-first Picture Vocabulary (2026-11-06)

Corrected the crossword page's zero-height flex/grid sizing risk and removed
desktop overflow clipping; updated the browser crossword acceptance check to
assert visible board dimensions, use tile accessibility labels and click the
renamed New game control. Browse moved the result range into the pagination
navigation alongside page number. Picture Vocabulary now starts automatically
when launched: typed HSK/filter or Word Set source, deterministic answer-key
preparation, streamed AI generation of a strict JSON scene sentence grounded in
the selected visible subjects, validated image prompt, asynchronous server
image generation, and mandatory user approval before grading. The AI scene
contract and native Rust tests live in `zipeek-ai`, while source decoding,
orchestration and presentation remain in focused web modules. Updated route
handoff and source contracts; obsolete Word Set-only loader removed.
Node checks 474/474 passed; Rust/native/WASM, image server, local model and
Chrome remain unverified in the packaging environment.

### R217  -  Practice exercise usability and setup simplification (development candidate)

The global shared-session banner and redundant Exercises explanation are removed so Practice Setup has one progress navigator and exercise pages have one coherent action footer. A reusable Yew round-actions component owns Continue and an explicit Exit confirmation; leaving a standalone round resets the shared quiz ledger. Multiple choice now shows the correct answer immediately after any response, highlights the actual correct choice, distinguishes the incorrect selection and presents a compact responsive two-column question/answer layout. Matching defaults to Hanzi → Meaning, calls English glosses "Meaning", and displays proper left/right-arrow icons for each format. A typed MatchingColumnOrder setting switches the rendered left/right columns and is handed through both direct and planned session URLs without altering word IDs or matching correctness. **The number of pairs remains four:** it is defined in the v1 deterministic round generator, matching reducer and scoring contracts; adding a 3/4/5/6 selector without migrating the engine would misrepresent the setting. R217 intentionally does not introduce a nonfunctional selector or break deterministic v1 replay. Node source regressions and browser assertions were updated; Rust/WASM/Chrome remain unverified in this environment.

### R227  -  Games and Crossword interaction polish (2026-11-07)

Crossword HSK setup replaces native checkbox styling with accessible custom checked buttons, uses a compact Start control with the existing Yew play icon, and changes clue selection from hover-driven state updates to stable click/keyboard activation. The selected, completed, hover and focus treatments are visually separated. The Games gallery now uses typed `Link<Route>` navigation and Yew Lucide icons rather than literal arrow glyphs and manual hash links. R227 regression checks added and superseded hover-contract tests revised. Node checks 513/513 passed; Rust/WASM/Chrome remain unverified.

### R230  -  Crossword results, loading and tone-aware Pinyin entry (2026-11-07)

Redesigned crossword completion as a focused results view with only Play again and All games. Removed Change HSK levels from game and results (initial setup remains the only place to select levels). Added remaining-bank Pinyin matching with tone-mark and tone-number normalization, Enter for unambiguous readings, number keys 1 - 5 for tone discrimination, and accessible candidate suggestions. Added a visible generating state and a browser-frame yield before starting synchronous layout computation. R230 source regressions: 523/523 passed, with Rust unit tests added. Native/WASM compilation and browser interaction unverified in the packaging environment.

### R232  -  Floating Pinyin suggestion overlay (2026-11-07)

Replaced the always-visible Pinyin form embedded in the crossword character bank with an on-demand fixed-position popover activated by keyboard typing. Kept Enter/tone-number disambiguation, added keyboard-input exclusions and removed redundant instruction text. Node regressions 529/529 passed; Rust/WASM and browser visuals unverified.

### R236  -  Write Hanzi presentation split (2026-11-07)

Extracted typed results/pagination and writing workspace Yew views from the page controller. The formatted baseline has 40 failing source-contract Node tests; R236 has the same 40, and Rust/WASM validation remains outstanding. See [current UI acceptance](../testing-quality/acceptance/ui-acceptance.md).

### R240 - Formatted-source tests and Practice Setup launch cleanup (2026-11-07)

The formatted-source Node contract suite was corrected to check Rustfmt-independent invariants. Practice Setup now shares a standalone vocabulary query builder across Character Hunt and Picture Vocabulary, rejects invalid saved-set route references, and keeps presentation callbacks named and typed in the touched components. Rust unit tests cover the standalone query's dictionary, saved-set, unavailable, and empty cases. Node validation passed 540/540 on the authoring snapshot. Rustfmt, Cargo/WASM, Clippy, and browser testing remain unavailable in the authoring environment, so this is an unverified source candidate. No schema or storage changes.

### R241 - Write Hanzi build fix, timestamped archive discovery and E2E contracts (2026-11-07)

Fixed the `VocabularyMode` Yew key error and unused import in Write Hanzi,
including callback ownership in the modified page. Updated `zipeek-cli` to
validate UTC release timestamps and select the latest numbered archive, with
Rust tests for valid and invalid names. The E2E suites now share typed helpers
for deterministic round fixture inspection, browser storage, focus and
Crossword controls; the standalone integration test roots import those helpers.
Stable test selectors were added to Crossword setup, replay and score controls.
Node source-contract checks passed in the packaging environment; Rust/WASM,
Clippy and Chrome execution are pending local validation. No database changes.


## R246 Yew markup and callback cleanup (2026-11-07)

The R245 baseline received a source-wide Yew readability cleanup. Named callbacks
replace inline handler construction in 34 qualifying Rust files; 87 additional
Rust files received formatting-only changes. Source-contract suite 559/559;
Rust/WASM, formatting and Chrome E2E remain unverified. See RELEASE_NOTES.md.


## R248 - Conditional Yew rendering follow-up and Crossword scope correction (2026-11-07)

Restored CrosswordBoard cell rendering to the row/column iterator after user-supplied E0425 errors from R247. Extracted 22 additional scoped Yew conditional render fragments in separate Rust files. Node source contracts passed 563/563; Rust/WASM and Chrome E2E were not available in the packaging environment and remain required for acceptance.


## R267 - Character Hunt UUID hosting and deterministic readiness (2026-11-08)

Added a resumable, practice-only Character Hunt host with dictionary-loaded candidates, persisted difficulty, seed-stable boards, valid completion callback and explicit errors when a pool cannot generate a puzzle. Strengthened session input validation and candidate selection rules. 22 substantive Rust files, plus three registration-only Rust edits. Focused Node suite 152/152; full 616/627 with 11 inherited failures. Rust/WASM/Chrome tests remain unrun and local acceptance is pending.

## R268 - AI preparation, resumable generated exercises and E0382 fixes (2026-11-08)

Connected the UUID generation queue to validated Sentence Translation and Picture Vocabulary hosts and a model-aware setup gate. Added bounded retry, stale-response protection, persisted typed payload validation and practice-only result handling. Corrected three moved Yew state handles from R267. 26 qualifying Rust files plus three registration-only edits. Node Practice 156/156; full 620/631 with 11 inherited failures. Cargo/WASM/Trunk/Chrome not run; source candidate awaiting local acceptance.

## R269 - Unified fixed-session entry and generation-state resilience (2026-11-08)

Single supported Fixed Practice exercises now enter the same UUID host as mixed fixed sessions, without imposing historical compatibility obligations on alternate runtime code. Interrupted generation no longer double-counts retries, saved state integrity is tighter, image approval requires a browser load event and content/input validation has additional guards and tests. 22 qualifying Rust source files. Practice source checks 160/160; full Node 622/633 with the same 11 inherited failures. Rust/WASM/Chrome acceptance pending. See root `RELEASE_NOTES.md`.


## R271 - Fixed UUID launch and Matching/Multiple Choice format parity (2026-11-08)

Removed the unused planned fixed-session launch branch, persisted quiz-format and Matching orientation per UUID step, and used the same format contract to present six quiz variants deterministically. Added honest Endless-host eligibility and updated setup, summary, domain and browser regression tests. Twenty substantive Rust files. Practice source tests 164/164, full Node 626/637 with 11 inherited failures; Rust/WASM/Chrome not run. Release remains an unverified source candidate.

## R273 - Practice evidence guardrails (2026-11-08)

Strengthened same-target correction integrity, objective score requirements, practice-only domain scoring prevention, assisted-learning aggregates and lexical/policy identifiers. Added restored UUID revision and generated-picture payload validation. Twenty Rust files changed with targeted regression coverage. Node Practice 164/164 and full Node 626/637, the same 11 inherited failures under identical R272/R273 test invocations. Rust/WASM/Chrome unverified. Endless unification and authoritative mastery publication remain open.

## R274 (2026-11-08) - CDP E2E video and Rust browser operations

Complete source candidate with Rust-owned Gzip/OPFS lock orchestration, initial continuous Chrome CDP video recording and evidence export, plus corrected lexical comparison and quiz return lifetimes. Twenty qualifying Rust files against R273; 21 changed including registration-only. Rust/WASM/Chrome/PowerShell acceptance remains open; see root RELEASE_NOTES.md.

## R279 (2026-11-08) - E2E evidence packaging and scenario verification

Numbered source candidate. Adds `cargo make pack-e2e`, PowerShell evidence export, E0425 correction, scenario manifest/readiness guards, AI tile validations and further screenshot-backed browser assertions. Node 639/652 with 13 inherited failures; Rust and Chrome acceptance pending.

## R281/R282 documentation consolidation (2026-11-09)

The temporary `docs/testing/R281_*` and `R282_*` coverage, translation and source-audit files were merged into the maintained current-state documents: `docs/testing-quality/acceptance/testing.md`, `docs/archive-history/audits/e2e.md`, `docs/architecture/modules/sentence-exercises.md`, `docs/learning-practice/translation/translate-sentence.md`, `docs/learning-practice/practice/practice-session-builder.md`, and `docs/project-management/current.md`. Their chronological change inventories remain available in `RELEASE_NOTES.md` and the previously distributed source archives, but are not current product or compatibility contracts. No Rust behavior was changed by the documentation consolidation.

## R420 WIP (2026-11-14)

Learning Data Settings consolidation; browser-only Speech TTS/STT test controls; removed inert pedagogical consent from first-run flow. Source checks only; no executable build or browser confirmation.

## R425 WIP - Agent release handoff and ChatGPT archive hygiene (2026-11-15)

- Updated `AGENTS.md` to require cleanup of superseded agent-generated project/release ZIPs within an accessible ChatGPT session, only after replacement artifact validation. User uploads, unknown files, local backups, external Downloads, and evidence archives are protected.
- Made Codex versus ChatGPT execution scope explicit: agents must not assume one session can access another session's files.
- Clarified that requested numbered WIP/major-release handoffs include the complete installable source archive, separately accessible changed primary files, and release notes; standalone file delivery alone is insufficient.
- Preserved manifest-first dependency version policy and existing dependency change approval requirements.
- Documentation-only WIP handoff. No Rust source, dependencies, storage formats or runtime behavior changed. No numbered major-release claim; native Cargo and Chrome testing not applicable to the documentation edit and were not run. ZIP integrity and archive-root layout checked.


## R430 WIP - UUID v7 WASM randomness feature (2026-11-15)

- Corrected `zipeek-web` UUID feature selection from `["v7"]` to `["v7", "js"]`, addressing the user-reported UUID `compile_error!` for `wasm32-unknown-unknown`.
- Kept the existing `uuid 1.29.2` package, `Uuid::now_v7()` identity behavior and `getrandom` configuration. The root lockfile package inventory is unchanged.
- Restricted `CURRENT.md` to the active work and added one targeted verification task.
- Static manifest and archive checks passed. Rust/WASM compilation, formatting and browser acceptance were not executed in this environment. This checkpoint is not an accepted release.


- R433 WIP: Docs Yew property visibility, E2E element type import, bootstrap writer compilation test; Windows and WASM acceptance pending.

## R436 WIP: Single-key Practice consent (2026-11-15)

Merged Practice outcome recording consent into `zipeek.preferences.v1`, using `learning_history_enabled` with a default of false. Updated onboarding, Settings, completion gating, tests, and owner documentation. The legacy standalone consent value is ignored; users previously opted in there need to opt in again. No learner-history records are deleted. Static checks only; WASM and Chrome acceptance pending.

## R439 WIP - In-panel Grammar lesson player (2026-11-15)

Added an in-panel step-by-step player for published Grammar courses, optional unverified AI examples through the existing configured client, and focused source/browser assertions. Did not change curriculum membership, scoring, unlock rules, preferences or dependencies. Full acceptance remains unverified.

- **R443 WIP, 2026-11-15:** Condensed game routing to a single parameterized route with typed slug handling; preserved game URLs and component providers, with source tests and pending WASM/Chrome acceptance.

## R444 WIP - Session-only Practice routes (2026-11-15)

Replaced standalone Practice routing with `/practice` and `/practice/session/:id`, UUID v7 temporary session IDs and the existing v1 session-storage prefix with the new JSON schema. Added shared typed exercise configurations referenced from individual scheduled steps, ISO session/step timestamps, and revision-safe step start transitions. No backwards-compatibility aliases or localStorage migration; recording-dependent learning history remains separate. Source changes and test updates are unverified pending Windows/WASM/Chrome acceptance.

- **R446 WIP, 2026-11-15:** Onboarding preview-only theme selection with rollback and atomic save, improved cards and controls, aligned Home spotlight/heading, compact meaning navigation, and added E2E assertions. Static checks only, Chrome/WASM pending.

### R449 WIP (2026-11-15): Practice setup configuration boundaries

The callback-heavy Exercises panel was reduced to a reducer-handle boundary and exercise-specific settings were split into focused components. Existing typed reducer actions, dictionary-backed definition tabs, persistence and public UI selectors remain. Reducer transition tests were added. Rust/WASM and Chrome verification remained outstanding at source handoff.

- **R450 WIP (2026-11-15):** Practice session shell runtime marker and Matching selection/match feedback regression coverage; source integrity only, Rust/WASM and Chrome acceptance pending.

- **R451 WIP (2026-11-15):** Cross-page visual remediation for Courses, Practice session/Matching, Practice setup, Browse, Home and Pinyin lessons; replaced an invalid Grammar screenshot asset with original landscape artwork and updated web/preflight/unpack tests. Added browser assertions. Source and ZIP inspected; Rust/WASM/Chrome execution pending.

## R453 WIP: Session-only Practice E2E route migration (2026-11-15)

Replaced three obsolete direct-route browser assertions with current setup/session shell and retired-URL Not Found checks. Existing exercise behavior suites remain for a separate full migration. Static source review only; Rust and Chrome checks not run.

## R479 WIP - Vocabulary full-height artwork and source-backed collection roadmap (2026-11-16)

Corrected desktop letterboxing by filling the allocated SVG viewport, preserving image proportions using cover cropping, and keeping the full 100-region interactive overlay. Added browser height/viewport assertions. Build and Chrome acceptance pending. See canonical `RELEASE_NOTES.md`.


R479 also expands the right collection overview with a compact per-set acquired/available roadmap, without moving dictionary word cards from the under-mosaic Explorer. The primary Acquire action stays visible above the roadmap. Explorer cards use two columns at narrower desktop widths and four on extra-wide screens. Runtime acceptance pending.

## R478 WIP: Vocabulary page state and viewport consolidation (2026-11-16)

Replaced independent selection/loading/paging hooks with a feature-scoped reducer, reused resolved region words from a bounded four-region cache, tagged asynchronous responses with region/request generation, and scoped acquisition feedback to originating course Word Set identity. Kept the four-card Explorer below the fitted mosaic without duplicating right-panel acquisition controls. Added unit and browser regression assertions. Pinned source coverage checked; Rust/WASM/Chrome acceptance outstanding. See root release notes.

## R480 WIP: Vocabulary dictionary readiness (2026-11-16)

Linked the Vocabulary region-loading effect to the existing shared database provider's readiness status, added explicit wait/failure states, automatic resolution upon Ready, provider-scoped retry and cache invalidation after database reinitialization. Kept the region reducer, stale-response guards and personal Word Sets semantics. Rust/WASM/browser acceptance pending. See root release notes.

## R493 WIP (2026-11-16): Grammar Pinyin and Python tooling removal

Removed six Python utility scripts by request; added 550 CC-CEDICT-assisted draft Pinyin lines to a static Rust inventory for the original and expanded Grammar lessons. Independent assessment questions remain unchanged. Native regeneration/HSK tools and Mandarin editorial validation are pending; Rust/WASM and browser acceptance not run here.

## R495 WIP (2026-11-16): Sentence Transformation configuration

Added selectable authored-pattern autocomplete and tile/typing/Both modes to Advanced Practice Sentence Transformation. Connected typed configuration through setup, profiles, stored sessions and exercise execution; added curated typed-answer grading. Three authored patterns only, no new linguistic corpus. Static and ZIP checks only; Cargo/WASM/Chrome acceptance outstanding. See root `RELEASE_NOTES.md`.

## R501 WIP: Word Sets workspace and selection (2026-11-16)

Refined the Word Sets UI and set selection contract, scoped desktop scrolling to the word list, added accessible icon actions, adjusted global focus presentation and updated E2E selectors. Personal Word Sets storage unchanged; selected viewed set remains an in-memory choice. Static and archive checks only; Rust/WASM/Chrome acceptance pending. See root RELEASE_NOTES.md.

## R503 WIP: Practice scroll build correction and service retries (2026-11-16)

Repaired R502 Yew `classes!` misuse, bounded the internal exercise-scroll workspace, removed redundant selection messaging, added capped linear retry spacing to service probes, and refreshed relevant checks. Build/browser acceptance pending. See root RELEASE_NOTES.md.

## R506 WIP: Write Hanzi missing callback correction (2026-11-16)

Restored `on_toggle_guide` action callback in the Yew writing page, fixing the R505 E0277 prop contract error. Confirmed reducer and component responsibilities without unrelated restructuring. Static source/ZIP verification only; locked Rust/WASM/Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R508 WIP: Write Hanzi resize and feedback stability (2026-11-16)

Corrected `use_resize_epoch()`'s `u64` dependency use in the writing canvas effect and reserved an always-present status slot in Practice so rejected-stroke feedback does not shift the board. No engine/storage changes. Source and archive checks only; locked Rust/WASM and browser checks pending. See root `RELEASE_NOTES.md`.

## R509 WIP: Progressive Hanzi stroke hint (2026-11-16)

Replaced the Write Hanzi Start cue point with a static fading partial-median preview that increases from 25% to 35% to 50% after rejected attempts. Preserved reference guide modes, stroke grading, settings formats and existing R508 board layout. Authored tests and documentation updated; locked Cargo/WASM and browser checks not run. See root `RELEASE_NOTES.md`.

## R518 WIP: Word Sets workspace redesign (2026-11-16)

Changed Word Sets to use document-level scrolling with a directly visible personal-set list and searchable vocabulary table. Compact rename/delete controls preserve explicit confirmation; backup and restore remain available under Data tools. Existing selection remains local management state, not a default Practice set. Updated E2E journey selectors and project docs. Only static and ZIP checks performed; Rust/WASM/Chrome acceptance pending. See root `RELEASE_NOTES.md`.

## R521 WIP: Write Hanzi vocabulary autocomplete (2026-11-16)

Introduced unified Hanzi/Pinyin/English vocabulary lookup with candidate ranking, debounced suggestions, accessible keyboard selection and reduced redundant focus decoration. Retained Word Set filtering and direct-entry behavior, updated source tests and E2E wording. Static and ZIP checks only; Rust/WASM/Chrome acceptance outstanding. See root `RELEASE_NOTES.md`.

## R523 WIP (2026-11-17)

Documentation navigation classification, search ordering and issue register updated; no gameplay changes. ZIP validated, Rust/WASM/browser acceptance not executed.

## R525 WIP - Issues and visual evidence (2026-11-17)

Documentation issue migration, visual assets and Docs navigation. Archive/link static checks only; Rust/WASM and Chrome unverified. No product defects closed.

## R526 WIP - Documentation modularization and validation (2026-11-17)

Extracted ten focused specification sections, kept their original entry point headings, created a Node-only local link and index freshness validator, and regenerated the Docs manifest. Current source Markdown checks passed; a frozen historical dead reference was reported as a warning. Rust/WASM and browser verification remain pending; no product issues were closed.

## Documentation decision reconciliation - 2026-11-17

- Consolidated current approved product decisions into maintained Practice, vocabulary, portability, recognition and platform requirements.
- Updated no-code-comments and dependency-approval policy, preserving historical decision provenance.
- Added executable acceptance checklists as future verification gates, not statements of implementation.
- Retained open technical architecture questions separately from consumed product decisions.
- The documentation validator was run on the assembled documentation overlay. Rust, WASM and browser testing were not performed.
