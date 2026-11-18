## R570 Statistics asynchronous section-source acceptance

- [ ] Run Rust formatting, native `zipeek-web` tests, WASM check and the Statistics browser suite on Windows.
- [ ] Verify all seven tabs load through the section source, with no direct fixture reads in presentation components.
- [ ] Exercise cancellation on navigation and preview close, retry after an injected error, empty responses and cache reuse across tabs.
- [ ] Design and approve canonical query/response identities and server-driven pagination/filtering for 5,000-10,000+ real records before wiring live evidence.
- [ ] Preserve fixture-only isolation and ensure no personal data read, DuckDB schema change or implicit Practice handoff occurs.

## R569 Statistics fixture and interaction acceptance

- [ ] Compile and test the expanded Practice registry-based History fixture and representative game session catalogue in Windows WASM.
- [ ] Confirm unscored Practice exercises never display verified correctness or mastery in History and the exercise registry contract test passes.
- [ ] Verify History Practice/Games filtering, search, timeline expansion, pagination and seven-day activity view in Chrome, both themes and both deployment paths.
- [ ] Confirm theme-integrated selection controls remain keyboard operable and are announced correctly by assistive technology.
- [ ] Complete the 5,000/10,000 record fixture matrix, robust shared select/listbox controls, full tile Settings library, real Practice/Word Sets handoff and actual Chrome chart lifecycle acceptance.

## Statistics 2.0 approved design and phased implementation (R565 documentation)

The [product and interaction contract](../product-ux/pages/learning-statistics-preview.md), [feature architecture](../architecture/modules/statistics-dashboard.md) and [acceptance matrix](../testing-quality/e2e/statistics-dashboard-design-acceptance.md) are approved for documentation and staged UI implementation. R565 changes no application behavior or production data.

- [ ] Stage A: reducer/context, request guards, skeletons, Overview and Chart.js JSON-compatible configuration updated through R567, pending WASM/browser verification. Complete read-only projection source, selection primitives, accessible chart acceptance and paging tests before considering Stage A accepted.
- [ ] Stage B: R567 implements searchable/paginated vocabulary and Grammar previews, temporary multi-selection review, clickable skill gaps, expandable session history and explainable insight links. R568 adds fixture-only card presets, variable page size and bounded filtered selection. Pending: confirmed Word Sets/Practice typed handoffs, canonical grammar mapping, full calendar, a searchable/reorderable 20+ tile library and complete Chrome acceptance.
- [ ] Stage C: 0/1/30/5,000/10,000-record fixture acceptance, responsive/light/dark/system/reduced motion, keyboard/screen-reader and root/hosted Chrome checks, measured performance.
- [ ] Before production data: approve durable ledger acceptance, per-session caps and per-skill validators, Writing assessment role, timezone, data privacy/retention, stable identity and grammar attribution. No mock-to-live promotion without these gates.
- [ ] Record actual Rust/WASM and browser results; documentation validation alone does not satisfy implementation acceptance.

## R564 Statistics design acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Verify Preview dashboard shows an actual two-second loading indicator, then the Demo indicator; leaving the route cancels the timer.
- [ ] Verify Overview fits common desktop heights with three cards and four table rows; full vocabulary and history tables remain paginated.
- [ ] Verify Writing is separate from Sentence use throughout sample evidence tables and Skills coverage.
- [ ] Verify custom switch controls are theme-aware, keyboard operable, accessible and do not persist changes.
- [ ] Verify neither preview activity nor fixture data reaches the personal database or network.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test stats_dashboard -- --nocapture` and visually inspect both themes and narrow viewports.

## R563 Statistics and Grammar preview acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test stats_dashboard -- --nocapture` and representative course mosaic browser tests.
- [ ] Confirm Statistics is directly after Browse, has a dashboard icon, and shows six icon tabs with keyboard access.
- [ ] Verify thirty unique sample words, paginated evidence/history tables, skill coverage and insights with no real data read or write.
- [ ] Verify card configuration is reversible, per-page metric descriptions are accessible, and sample disclaimers remain visible.
- [ ] Verify Grammar example paging stays within selected course, resets on course change, renders available curated Pinyin beneath Hanzi, and remains usable in dark/light and mobile layouts.

## R560 Province Quest region component acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --locked -p zipeek-web` and `cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture`.
- [ ] Verify map region click, keyboard Enter/Space activation, focus, selected/correct/hint state and drag-click suppression across Easy, Medium, and Hard modes.
- [ ] Verify zoom/pan remains responsive with prefecture geometry on root and `/zipeek/` deployments, including mouse, trackpad and touch interactions.

## R559 Province Quest interaction acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --locked -p zipeek-web` and the Province Quest browser gameplay suite.
- [ ] Verify wheel zoom with a mouse and a trackpad, in letterboxed and wide viewports, including zoom-to-cursor behavior.
- [ ] Verify that click selects a region, drag pans without selecting, and many small drag movements work correctly.
- [ ] Verify all three difficulty modes, keyboard selection and both base paths, including Chrome touch input.

## R558 Province Quest map and controls acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --locked -p zipeek-web` and `cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture`.
- [ ] Verify Hard mode initially shows a national map, province selection zooms into prefecture geometry, and empty provinces are not offered in the selector.
- [ ] Verify accessible Exit and Settings icons with keyboard, focus and both themes.
- [ ] Verify complete Easy/Medium/Hard journeys, mouse wheel zoom, drag-to-pan and mobile layout on `/` and `/zipeek/`.

## R551 recognition visual acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and the recognition E2E browser journey.
- [ ] Visually confirm the Upload/Recognize icons, pencil empty affordance, loading spinner, reduced-motion fallback and icon accessibility in both themes on mobile and desktop.

## R550 recognition layout acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test hanzi_recognition -- --nocapture`.
- [ ] Check canvas/results side-by-side and bounded results scrolling at desktop sizes, including crowded candidates and short viewports.
- [ ] Check that mobile results scroll into view only after a completed recognition, with no nested mobile scrolling.
- [ ] Verify light/dark contrast, accessible icon toolbar, error handling and stale-result invalidation on both root and `/zipeek/` routes.

## R549 recognition UI acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test hanzi_recognition -- --nocapture` with an installed dictionary.
- [ ] Visually review light/dark and mobile/desktop layouts; confirm icon-only Undo and Clear have keyboard focus, tooltips and accessible names.
- [ ] Confirm missing-model and recognition errors remain actionable, the request requires explicit submission, and image data is never logged.

## R546 recognition feature acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` to catch WASM API and component integration errors.
- [ ] Run `cargo test --locked -p zipeek-browser` and `cargo test --locked -p zipeek-web` where supported.
- [ ] Run the recognition E2E journey and accessibility matrix with working generated dictionary assets.
- [ ] Verify direct typed input/error notifications for upload, paste, Undo/Clear and repaint failures; no `data-recognition-*` attribute or `zipeek-recognition-*` event is used for state transfer.
- [ ] Review recording and AI request privacy, stale completion, light/dark rendering, and browser cleanup on `/` and `/zipeek/`.

# Open tasks and verification

## R544 recognition workspace architecture acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Browser-check image upload/paste, Undo/Clear, theme changes, and navigation cleanup in dark and light themes.
- [ ] Verify missing canvas context and unavailable theme report actionable diagnostics without logging image bytes.
- [ ] Verify the named `CanvasPoint` geometry in unit tests and that generic browser event handlers ignore events of an unexpected subtype.

Use the current authoritative product contracts and [acceptance checklist](../testing-quality/acceptance/approved-behavior-acceptance.md). Checkboxes denote unverified work, not historical completion.

## Rust/WASM chart interop verification

- [ ] Compile the R540 `zipeek-browser` Chart.js dynamic import via `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and verify module loading, destroy-on-drop, and root/`/zipeek/` vendor paths in real Chrome. Keep D35 Chart.js packaging unchanged.

## E2E evidence verification

- [ ] Run `cargo check --locked -p zipeek-e2e --tests` and targeted Rust unit tests on a supported toolchain.
- [ ] Run the Word mosaics Chrome browser journey with FFmpeg and FFprobe available, and inspect the actual MP4, timestamped checkpoints, `agent-feedback.md`, `scenario.json`, and suite `review.md`.
- [ ] Run strict evidence capture using `ZIPEEK_E2E_VIDEO_STRICT=1` and `ZIPEEK_E2E_EVIDENCE_STRICT=1`; audit its real MP4, checkpoints and manifests with the offline evidence auditor. Authored offline integrity tests do not substitute for browser verification.
- [ ] Run the larger E2E suite with deterministic fixtures; distinguish browser, assertion, service, and recording failures.
- [ ] Execute the updated Docs reader journeys for all twelve sections, image decoding, search and deep links at `/` and `/zipeek/`. Source checks and route validation are authored but do not constitute a browser pass.
- [ ] Revisit the [E2E requirements matrix](../testing-quality/e2e/e2e-requirements-matrix.md) after each implementation. Do not upgrade a row to verified without actual passing browser evidence.
- [ ] Replace obsolete URL replay and generic feedback assertions after the corresponding approved product behavior is implemented. Never make old assertions pass by inventing application behavior.

## Product features pending implementation or browser acceptance

- [ ] Implement and verify the common Practice exercise component foundation, consistent session navigation, and typed outcome evidence.
- [ ] Verify Practice completion distribution, persistent sessions, Skip and Replace, dragon evidence and profile JSON portability.
- [ ] Implement and test global exact-entry exclusions, vocabulary visitation, and cancellable nonblocking eligibility counting.
- [ ] Implement and verify context-aware Sentence Relay feedback, token dragging, punctuation and setup stages.
- [ ] Implement and verify selected/correct/incorrect Word mosaics tile states, progressive hints and accessible announcements.
- [ ] Add Multiple Choice pinyin and pronunciation controls, including anti-answer-leak rules and browser audio acceptance.
- [ ] Investigate Write Hanzi animation interruptions and stale frame glitches with recorded browser scenarios.
- [ ] Verify the R532 AI Hanzi Recognition route in real Chrome, including vision capability, paste/upload, cancellation, dictionary actions, Word Sets and strict privacy. Source implementation and authored tests are not browser acceptance.
- [ ] Verify original-tab ownership and duplicate-tab full-screen recovery, System theme, database startup blocking and Credits.
- [ ] Review the [open issue library](../issues/OPEN.md), preserving evidence and avoiding unverified closure.

## Engineering and deployment acceptance

- [ ] Verify R530 dictionary-only `db-install` against a real regenerated bundle, including unchanged practice/course/brand assets, old DB backup and restored `white-tiger.webp`.


- [x] Add an offline source/deploy asset integrity gate, covering manifest hashes, generated asset presence and a consistent preload filename, with fixture-based Node tests.
- [ ] Execute the strict deploy preflight against real generated dictionary, Worker and geography assets, then verify Rust/WASM compilation and Chrome deployment paths.

- [ ] Execute `cargo fmt --all -- --check`, `cargo test --workspace --locked`, WASM build and browser fixtures on the actual Windows environment.
- [ ] Verify dictionary and Province Quest asset generation on both local and hosted paths.
- [ ] Verify capability-aware unavailable AI exercises, grammar evidence, course locking and browser/local speech recognition.
- [ ] Review outstanding architecture proposals D28 (modular E2E organization), D29 (scoped state ownership and initial motion profile), and D30 (crate extraction boundaries) against current source. Do not change dependencies or extract crates without explicit approval.

The historical acceptance backlog remains preserved in the [legacy acceptance ledger](../archive-history/LEGACY_ACCEPTANCE_LEDGER.md) and [completed records](../archive-history/completed/). This active list intentionally contains no release-numbered task chronology.

## Documentation and source reconciliation still pending

- [x] Preserve and reconcile historical development-marker references from maintained technical articles. See [editorial disposition and preserved source evidence](editorial-reconciliation.md).
- [x] Audit the architecture, database generator, checked-in manifest and web runtime against maintained documentation. [Source accuracy audit](source-accuracy-audit.md) records corrected facts and open verification gates.
- [ ] Reconcile remaining technical assertions not sampled in the source accuracy audit against implementation and browser results.
- [ ] Verify Credits source-level licensing and generated asset provenance against shipped distributions before release.
- [ ] Compile the Docs reader after the documentation restructuring; check all section headings, deep links and issue images in real Chrome.
- [ ] Implement the approved shared exercise foundation, long detailed dragon and nonblocking vocabulary eligibility service; authored specifications are not implementation evidence.
- [ ] Link the remaining missing E2E scenarios to precise acceptance rows, then run strict video capture on the Windows/Chrome target.

## R529 generator verification follow-up

- [ ] Run the R529 database generator import tests and full regeneration against the pinned inputs; validate output count, logical snapshot, manifest hashes, FTS and read-only reopening.
- [ ] Benchmark the R528 row-wise generator against R529 parameterized 256-row insertion, including generation time, peak memory, file size and browser dictionary lookup latency.
- [ ] Investigate remaining per-row sense display, relation, qualifier and enrichment writes only after baseline measurements; preserve transactional rollback and evidence.

## R531 E2E follow-up

- [ ] Verify R531 preflight with a real installed compressed dictionary and both Root/Pages Trunk outputs.
- [ ] Re-run accessibility_matrix and debug any remaining Province Quest and Docs route failures from browser diagnostics without weakening route assertions.
- [ ] Complete Cargo/Trunk/Chrome gates and review checkpoint artifacts before release acceptance.

## R532 validation gates

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo test --locked -p zipeek-e2e --lib` and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test hanzi_recognition -- --nocapture` and both root and Pages route/accessibility journeys.
- [ ] Check a real image-capable local AI endpoint, including invalid output, unsupported vision model, user cancellation and no uploaded image bytes in diagnostic artifacts.
- [ ] Verify clipboard and touch behavior in Chrome and Firefox, including replacement confirmation, 5 MB image limit, resource cleanup and image quality.
- [ ] Check R532 candidate actions with a real Word Set and actual dictionary pinyin/meaning against both simplified and traditional hits.
- [ ] Reproduce or verify closure evidence for ISS-001, ISS-021 and ISS-028 on Windows; do not close on source-only evidence.

## R533 native speech service follow-up

- [ ] Compile and test the new local Mandarin speech service with pinned `whisper-rs` on the Windows toolchain.
- [ ] Exercise eSpeak Mandarin WAV synthesis and multilingual Whisper Mandarin recognition with licensed local models.
- [ ] Design and obtain approval for a trusted desktop bridge from Yew/browser UI to the loopback-only native service, including origin authentication, microphone permissions, cancellation, and provider selection under D56.
- [ ] Add end-to-end voice recording/assessment tests without interpreting transcript equality as pronunciation mastery.

## R534 native speech protocol hardening

- [ ] Run locked compile and native tests for the speech crate on Windows, including browser-Origin rejection and loopback Host validation.
- [ ] Verify eSpeak overflow/timeout handling and that child processes are reaped on failures using licensed local resources.
- [ ] Keep the trusted desktop bridge pending explicit D56 security approval; do not connect a hosted page directly to loopback HTTP.

## R535 speech output correctness and verification

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-tts-stt`, and `cargo test --locked -p zipeek-tts-stt` on Windows.
- [ ] Generate speech via eSpeak NG and validate the served WAV with a decoder, sample-rate check, and real browser playback; confirm stdout placeholder lengths are repaired.
- [ ] Verify valid Mandarin speech with the pinned multilingual Whisper model and inspect empty recognition outcomes, request cancellation and saturated-service behavior.
- [ ] Keep the trusted desktop bridge pending the explicit D56 origin-authentication and microphone-consent design decision; no web-provider changes in this checkpoint.

## R536 local speech and Hanzi recognition acceptance

- [ ] Compile and test `zipeek-tts-stt` with `cargo test --locked -p zipeek-tts-stt`.
- [ ] Compile `zipeek-web` for `wasm32-unknown-unknown` and build with Trunk for `/` and `/zipeek/`.
- [ ] Verify exact-origin CORS and blocked-host behavior with the live local process; test hosted origin denial.
- [ ] Install licensed eSpeak/Whisper resources and confirm local Mandarin audio playback and five-second PCM16 microphone transcription in Chrome.
- [ ] Check microphone permission denial, interruption, cancellation, page navigation, and stale responses.
- [ ] Review Recognize Hanzi in browser with keyboard navigation, mobile layout and model responses (raw/fenced/invalid JSON).

## R537 recognition interface acceptance

- [ ] Verify drawing contrast in Light, Dark and System, including a theme change during an active drawing.
- [ ] Verify page height and vertical scrolling at mobile and desktop sizes, including many candidate results.
- [ ] Run recognition E2E with valid, invalid JSON and unavailable vision model; confirm console diagnostics do not contain image data or raw model text.
- [ ] Run locked WASM check, Node workspace tests and both root/Pages route journeys.

## R538 Rust/WASM interop migration acceptance

- [ ] Run `cargo fmt --all` and `cargo fmt --all -- --check` on the pinned toolchain.
- [ ] Run `cargo test --locked -p zipeek-browser` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Verify local speech Settings, shared Listen buttons and Speaking Recorder through native eSpeak/Whisper in Chrome, including consent denial, cancelled requests, interrupted playback and navigation cleanup.
- [ ] Confirm that local speech is restricted to localhost:1420, preserves browser default, and never persists or logs audio, speech text or recordings.
- [ ] Run `cargo make prepare-summary` and `cargo make verify-assets`; confirm the Chart.js-only thin module loads in root and `/zipeek/` deployments.
- [ ] Mount/verify the new Rust chart handle in real Yew summary/course components. Existing wrappers had no live Rust consumers; numeric text and D35 animation/reduced-motion behavior still require browser acceptance.

## R541 Rust/WASM browser service migration acceptance

- [ ] Run locked native `zipeek-browser` unit tests and locked `wasm32-unknown-unknown` check for `zipeek-web`.
- [ ] Confirm AI configuration can save URL/model, preserves other preferences, rejects credentials and invalid endpoints, and does not persist API keys; reload must clear the API key.
- [ ] Run Hanzi Recognition browser journeys with mouse, stylus, touch, paste, upload, undo, clear, light/dark/system appearance, stale image decode and navigation cleanup.
- [ ] Verify restricted image MIME, byte size and dimensions; confirm no image bytes in console logs or persisted settings.
- [ ] Run root `/` and hosted `/zipeek/` navigation, production asset preparation and pinned Chart.js bundle integrity checks.

- [ ] R545 verification: run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and recognition browser E2E to validate the reducer-backed feature split.

- [ ] Verify R552 recognition request-token compatibility against configured vision providers and visually test compact desktop/mobile suggestions with keyboard and screen reader.

- [ ] Run `cargo make install` on Windows with DuckDB v2 native libraries and 7z/7zz available; confirm dictionary, Worker and Province Quest assets all pass `preflight-deploy`.
- [ ] Validate Province Quest root and `/zipeek/` asset URLs, including both JSON files and playable map geometry, after unified install.
- [ ] Review the redistribution rights of upstream AreaCity boundary data before shipping generated Province Quest assets publicly.

## R557 follow-up verification

- [ ] Verify typed Province Quest lifecycle and gesture changes with WASM compilation and Chrome E2E.
- [ ] Cover wheel zoom, pan, pointer cancellation and click suppression in real browser sessions.
- [ ] Profile Hard mode SVG rendering before introducing optimization caches.
