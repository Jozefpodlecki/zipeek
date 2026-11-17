# Legacy checkpoint acceptance ledger (unverified)

This is a **read-only, verbatim snapshot** of the R521-era task register, preserved during R522 documentation reconciliation. It is NOT a completed-task archive and its unchecked work has NOT been accepted or waived. Some old commands, paths and assertions refer to retired tooling or superseded interfaces and must be reconciled against the current source and test harness before running them. The active triage index is [TASKS.md](../project-management/tasks.md); historical outcomes remain in [completed/](completed/) and [RELEASE_HISTORY.md](RELEASE_HISTORY.md).

Snapshot facts: 644 unchecked and 56 checked checkbox rows in the R521 task register. Historical checked rows are not proof of current R521/R522 runtime acceptance. No item was marked completed by this reconciliation.

---

## R521 WIP: Write Hanzi autocomplete acceptance (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Confirm English (exact gloss, prefix), simplified/traditional Hanzi, tone-marked/numbered/toneless/joined Pinyin and no-results query relevance in Chrome.
- [ ] Verify keyboard Arrow Up/Down, Enter, Escape, focus/Tab, pointer selection, clear/retype, suggestion count, selected word/change, switching Word Sets and stale async requests.
- [ ] Check focus is visible but not doubled, no undesirable nested scrolling, mobile layout, dictionary loading/errors, direct word route and hosted `/zipeek/`.

## R520 WIP: Time Weaver setup acceptance (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Check keyboard activation and focus, toggles, sample tile size/reading, all round sizes, balanced question kinds and correct finish totals.
- [ ] Check preferences survive reload, invalid JSON and unavailable storage safely use defaults, and game speech errors are recoverable.
- [ ] Check desktop/mobile layout, reduced-motion transitions, local `/` and hosted `/zipeek/` paths.

## R519 WIP: Crossword variety acceptance (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome confirm repeated New game and Play again prefer less recently used words, persist across reloads, and preserve the active game on refresh or direct session URL.
- [ ] Check browser localStorage key `zipeek.crosswords.rotation.v1`: maximum 100 unique words and 10 recent games, no duplicate usage within a session, proper LRU eviction, recoverable invalid records and storage failures.
- [ ] Check a constrained HSK pool still produces valid connected puzzles when repeats are unavoidable; review generation latency and both local `/` and hosted `/zipeek/` routes.

## R518 WIP: Word Sets UX acceptance (2026-11-16)

- [ ] In Chrome select Word Set B, add a Dictionary word, reload, and verify B remains the active Add/Remove target; delete B and verify safe fallback without writing to a missing set.

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome verify single page scrollbar, long vocabulary sets, responsive sticky panel, search across Hanzi/pinyin/meaning, search reset on set switch and clear no-matches.
- [ ] Verify set creation, rename disclosure, selected-set delete/confirm, backup/restore under Data tools, empty-set Browse link, audio and per-word removal.
- [ ] Verify all keyboard/focus controls, selected-set no default Practice preference, data persistence and routes at `/` and `/zipeek/`.

## R517 WIP: Listening Choice configuration and UI acceptance (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome test all Never / After answering / Always pinyin and meaning combinations before and after answer, with no answer leaked selectively; confirm dictionary data displays correctly.
- [ ] Verify manual speaker repeat, automatic playback on/off, failed/blocked browser speech, screen-reader labels and keyboard interaction.
- [ ] Verify saved Practice profiles, step configuration restore, wrong-answer feedback, Continue, narrow/mobile layout and no feedback layout jump; routes at `/` and `/zipeek/`.
- [ ] Warn about strict schema changes to saved Practice profiles and sessions.

## R516 WIP: Practice completion UI acceptance (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome verify perfect and partially skipped session headings, progress fill and counts, both actions, narrow and short viewports and reduced-motion behavior.
- [ ] Verify learning-history disabled, recorded, pending and failed/retry states remain accurate and accessible; test local `/` and hosted `/zipeek/`.

## R515 WIP: typed Hanzi progress preview

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on Windows.
- [ ] Verify the active Write Hanzi step still previews dragon completion before Continue, and resets properly on Undo and on the next step.

## R514 WIP: Hanzi progress preview build repair

- [ ] Confirm `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` complete on Windows without E0425 or a newly surfaced error.
- [ ] In Chrome, verify the completed-write preview illuminates the active step before Continue, Undo revokes it and advancing does not carry preview to the next step.

## R513 WIP: Hanzi completion, dragon preview and playback

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome confirm dragon illumination previews on final-character completion, without advancing/saving the step until Continue; Undo revokes preview; next step does not inherit previous preview.
- [ ] Confirm completion area maintains board geometry before/after Continue appears at multiple viewport heights and widths.
- [ ] Check full Watch and single-stroke assistance progress smoothly across straight, hooked, complex and overlapping strokes, no clipped or disconnected masks; reduced-motion and repeat playback.
- [ ] Confirm local `/` and hosted `/zipeek/` exercise routes and keyboard access.

## R512 WIP: automatic Hanzi stroke assistance (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-hanzi-writing -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and E2E compilation.
- [ ] In Chrome verify the first two rejections show normal feedback, third shows a single stroke demonstration then advances; five-mistake and Off configurations behave correctly, including saved session/profile reload.
- [ ] Verify accepted strokes do not count as assisted, Undo/Reset/character switches cancel pending assist, no stale callback advances, and complete-last-stroke displays the right counts and Continue button.
- [ ] Confirm assisted animation uses R511 aligned replay masks, visible completed strokes remain, and keyboard, mobile, high-DPI and reduced-motion playback behave correctly.
- [ ] Test existing stored session/profile recovery after the strict options representation change; warn users to export or finish important state first.

## R511 WIP: Write Hanzi stroke replay alignment (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] In Chrome verify Watch progressively reveals complete outlines without fragmented or vertically displaced masks; repeat on strokes containing curves, hooks and multiple corners.
- [ ] Verify last stroke remains visible until replay completes and multiple Watch activations restart cleanly, at desktop/mobile sizes and under reduced motion.
- [ ] Verify Start cue / Full guide previews, grading, Undo and Reset are unchanged by the playback-only correction.

## R510 WIP: handwriting recognizer and word-prompt acceptance (2026-11-16)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-hanzi-writing -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` in the supported Rust environment.
- [ ] In Chrome verify correct slightly wavy, displaced, early-turn and hooked strokes pass; backwards, missing-turn, wrong-order and long-detour strokes fail with the correct reason.
- [ ] Test all eight combinations of Pinyin / Meaning / Listen in Write Hanzi Configure, including hidden audio, absent dictionary meaning, mobile wrapping, keyboard access, saved profile and resumed Practice session.
- [ ] Check browser TTS availability, accessibility announcements and multi-character words; test root `/` and hosted `/zipeek/` with the appropriate bootstraps.
- [ ] Collect consent-free local calibration fixtures from mouse/touch/stylus and measure false rejection and false acceptance before treating the new matcher as accepted.
- [ ] Warn users that the strict practice-option schema has changed and earlier persisted sessions/profiles may be incompatible without migration.

## Write Hanzi directional hint (2026-11-16, pending acceptance)

- [ ] Run locked Rust unit tests and WASM compilation for the partial-median hint, its rejection counter and SVG markup.
- [ ] Browser-check Start cue escalation (25% / 35% / 50%), automatic reset after accepted strokes, Undo and Reset, and replay/drawing suppression.
- [ ] Check hint contrast and geometry on short, hooked and curved strokes with desktop, touch, zoom and reduced motion.

## R508 WIP: Write Hanzi resize and stable feedback acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` on Windows; verify E0614 is resolved.
- [ ] At several viewport sizes, alternate accepted and rejected strokes in Practice Write Hanzi and confirm the canvas and toolbar geometry stays fixed when the status text appears and disappears.
- [ ] Check loading error and screen-reader announcements; standalone Write Hanzi retains its existing progress UI.
- [ ] Independently address the accepted-stroke drawing/canonical overlay flicker described in the video, with actual Chrome acceptance.

## R507 WIP: Practice recovery and writing modularization (2026-11-16, pending acceptance)

- [ ] Run `cargo fmt --all`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests` on Windows.
- [ ] Verify an incompatible/corrupted stored session shows the safe, actionable recovery card without Rust enum debug text and without destroying other browser storage.
- [ ] Complete a saved session, refresh its removed URL, check the `not-found` data attribute and navigate to setup. With browser storage blocked, confirm Retry appears for a recoverable storage failure.
- [ ] Exercise standalone and embedded Write Hanzi: stroke loads, guide toggle (standalone), None/Start cue/Full guide (Practice), Watch, Reset, Undo, resize, character change during replay, and completion callback.

## R506 WIP: Write Hanzi compile regression (2026-11-16, pending acceptance)

- [ ] Compile R506 on Windows (`cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`); verify E0277 for `on_toggle_guide` is gone and identify any subsequent errors.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome verify the standalone Show/Hide guide button toggles through the reducer, while Practice guidance remains configured only in setup.

## R505 WIP: Write Hanzi learner guidance and canvas acceptance (2026-11-16, pending)

- [ ] Compile and format the new writing settings, profile/session configuration and UI on Windows (`cargo fmt --all`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo check --locked -p zipeek-e2e --tests`).
- [ ] In Chrome verify None, Start cue and Full guide: the next-stroke start dot only appears with guidance, the full outline appears only in Full guide, and no interior animated centerline is present.
- [ ] Verify animation toggle hides Watch when off and shows it when on; Undo removes one accepted stroke and Reset restarts the character. Validate touch and stylus, multi-character progression and session reload.
- [ ] Confirm the standalone Hanzi writing route retains its guide toggle and progress display, while Practice omits counters, visible progress and trace instruction.
- [ ] Review the existing stroke-order playback's visual quality on real reference data, separate from the configuration/guide fix.
- [ ] Before installing, finish or export any active Practice sessions and saved profiles that must be preserved: the options schema changed without a compatibility layer.

## R504 WIP: Practice exercise internal scroll acceptance (2026-11-16, pending)

- [ ] On Windows run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In the actual app at 650px/768px/900px desktop heights, check that the list below Choose/Configure owns the scrollbar, reaches Picture Vocabulary, and the browser/main area does not scroll. Also check the final Configure setting, keyboard PageDown, pointer wheel, and resizing.
- [ ] At mobile width verify document scrolling and no hidden clipped exercise tiles.

## R503 WIP: Practice scroll and service health browser acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, locked Rust tests, WASM check and Rust E2E compilation in Windows.
- [ ] Confirm R503 is loaded (Pronunciation tile, all requested AI tags), then test the inner scroll region below Choose/Configure at desktop heights 600, 768 and 900 pixels. The last tile and last Configure control must be visible at maximum scroll; main and page must not scroll.
- [ ] Test the same flow on a mobile viewport using natural page scrolling and without horizontal overflow.
- [ ] With AI and image services unavailable, observe single-flight health requests at 5/10/15/20/25/30-second spacing, capped thereafter; verify the interval resets on recovery and manual retry works.
- [ ] Decide whether Write Hanzi needs a persisted initial-guide setting, repetition or character-count options; the existing guide toggle works only in the writing exercise.

## R502 WIP: Practice exercise list scrollbar and capability acceptance (2026-11-16, pending)

- [ ] Run locked Rust tests, WASM check and Chrome E2E for the exercise selector and configuration.
- [ ] Verify the scrollbar is visible inside the exercise widget on the right, below Choose/Configure, while the stepper and footer stay in place and the page main area does not scroll.
- [ ] Verify the final tile and last setting are reachable at 768px and shorter viewport heights; mobile remains naturally scrollable.
- [ ] Verify six AI-tagged tiles (including Picture Vocabulary), Picture Vocabulary's Image generation tag, and the Pronunciation title. Labels do not claim live AI integration for currently authored local exercises.

## R501 WIP: Word Sets browser acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` and fix any reported errors.
- [ ] In Chrome confirm the Word Sets custom picker shows the viewed set, supports mouse and keyboard (including Escape), renames/deletes the intended set, and never retains a prior set's delete confirmation after switching.
- [ ] At short desktop heights scroll a 38+ word set to the end inside the vocabulary panel without losing the sidebar; on mobile verify natural page scrolling and responsive word rows.
- [ ] Confirm trash icon accessibility labels, listening, contextual empty-set Dictionary navigation and keyboard focus visibility in light and dark themes.

## R500 WIP: Practice scroll and capability indicators (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome at `/practice` (and `/zipeek/practice`), select the advanced path and scroll to the last tile; verify the final Configure setting is reachable and only one viewport scrollbar is active on desktop, including a short-height viewport.
- [ ] Verify mobile natural page scrolling, keyboard reachability, the AI label on Sentence Translation and AI/Image generation labels on Picture Vocabulary; verify no invented readiness or selected-count announcement.

## R499 WIP: E2E selector contract acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` on Windows; fix any remaining Yew or browser test compile errors.
- [ ] In Chrome check expanded Grammar reveal/choice feedback and disabled Continue; Sentence Transformation Tiles/Typing/Both, tile selection, Undo/Reset and submit; unsupported pattern input; Cloud Match loading/insufficient/ready and Start.
- [ ] Confirm selector stability on root `/` and hosted `/zipeek/` routes, and prefer stable hooks over text- or CSS-dependent locators in additional E2E suites when those are touched.

## R498 WIP: Sentence Transformation parser and reducer acceptance (2026-11-16, pending)

- [ ] On Windows run `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`; confirm the R497 Yew opening-section parser error is eliminated and fix any newly reported errors.
- [ ] Browser-test Sentence Transformation Tiles, Typing and Both with correct/incorrect answers, Undo, Reset, mode changes, answer locking and Continue. In Both, verify inactive method callbacks cannot change state and drafts remain intact until submission.
- [ ] Continue auditing Practice's in-flight generation result lifecycle for session-step changes, retries and out-of-date storage, with direct browser acceptance before claiming it complete.

## R497 WIP: Yew workflow state acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` on Windows. Fix errors before accepting this WIP.
- [ ] Verify Sentence Transformation tiles, typing and Both mode preserve drafts while switching, reject edits after Check, and reset on question/session identity changes; verify the typed accepted alternatives and invalid tile IDs in pure tests.
- [ ] In Practice Configure, type partial/unrecognized patterns and verify the prior pattern remains selected; choose a valid autocomplete entry and All patterns and verify saved options, resume and generated question identity.
- [ ] Open and switch between distinct Practice session IDs, retry history completion and refresh while completion is pending; confirm no stale completion result is shown and durable history remains idempotent.
- [ ] Simulate unavailable browser storage in expanded Grammar and verify the step does not advance on a failed checkpoint save; refresh guided practice and confirm reveal/choice restart is communicated. Verify `/` and `/zipeek/` routes.

## R496 WIP: Grammar duplicate module compile acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-pedagogy`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on Windows and confirm the three E0428 errors no longer occur.
- [ ] Finish R495 Sentence Transformation browser checks for supported pattern autocomplete, tiles / typing / Both selection, grader, profile resume and session persistence after a successful build.

## R494 WIP: Grammar and Practice interaction acceptance (2026-11-16, pending)

- [ ] Run locked Rust formatting, `zipeek-exercises` tests, `zipeek-web` WASM check and targeted `zipeek-e2e` browser suites on Windows.
- [ ] In Grammar region 100 guided practice, deliberately choose a wrong sentence, then the right one; verify all answer options become disabled afterward and the correct answer remains selected, with playable audio on every choice.
- [ ] At typical 768px and short desktop heights, scroll the Practice Exercise Choose grid to the final option and the Configure panel to its final setting. Verify one independent vertical scroll area rather than competing nested scrollers, including touch/mobile.
- [x] Approve and implement Sentence Transformation pattern selection with authored-pattern autocomplete, plus tiles / typing / Both answer methods. R495 scope covers the existing three authored prompts only.
- [ ] Expand the independent Sentence Transformation bank beyond the current three patterns, with reviewed answer variants, typed grading and browser acceptance.

## R493 WIP: Python removal and Grammar Pinyin acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy -p zipeek-web` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] In Chrome verify tone-marked Pinyin below the Hanzi on original courses 1-5 and expanded courses 6, 21 and 100, including revealed recall and recognition options. Assessments should continue to hide Pinyin.
- [ ] Have a Mandarin editor verify the 550 CC-CEDICT-derived pronunciation strings, especially polyphonic words, tone sandhi and neutral tones.
- [ ] Replace the removed Python vocabulary/grammar generation, membership audits and HSK importer with appropriate Rust-native workflows before changing their source inputs.

## R492 WIP: Grammar architecture acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy -p zipeek-exercises -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] In Chrome verify original courses 1-5 and expanded regions 6, 21 and 100 under root `/` and Pages `/zipeek/`: titles, step counter, progress bar, exit, independent original assessments, recall gate and saved-step refresh.
- [ ] Confirm the HSK index and Grammar regenerated Rust are unchanged versus R491. Review after mobile/focus browser checks; retain unsourced Grammar examples as drafts.
- [ ] Later: unify the two actual exercise/assessment engines and persistence transitions when independently authored question banks and mastery rules are approved. The shared lesson scaffold alone does not unify pedagogical exercise behavior.

## R491 WIP: Grammar guided-practice acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests` on a machine with the Rust toolchain.
- [ ] Verify the expanded Grammar lessons at regions 6, 21, 40, 60, 80 and 100 under both `/` and `/zipeek/`: explanation, hidden-answer recall, answer reveal, meaning recognition, correct-answer gate, previous/next navigation and saved checkpoints.
- [ ] Check direct refresh at a saved guided-practice step, invalid region/answer handling, mobile focus behavior and voice playback of authored Hanzi.
- [ ] Have a qualified Mandarin editor review the 550 draft grammar entries. Author several independently sourced examples, counterexamples, rule usage constraints and independent assessments per point before awarding any mastery or HSK coverage.

## R490 WIP: Grammar HSK mapping editorial acceptance (2026-11-16, pending)

- [ ] Download the 593-row HSK 2025 Grammar TSV in a network-enabled checkout and record the exact file SHA-256 with a future native HSK importer.
- [ ] Check import/audit/report output and the expected level totals (70, 78, 96, 95, 70, 50, 134). Resolve malformed rows against the official syllabus before reviewing.
- [ ] Classify and review all candidate links and unmapped HSK items against authored Grammar lessons. A candidate is not verified coverage.
- [ ] Verify `cargo fmt --all -- --check`, locked Rust/WASM checks and browser journeys under `/` and `/zipeek/`. This checkpoint only changed Python tooling and documentation, so no Rust code was changed.

## R489 WIP: Review HSK 2025 Grammar alignment (2026-11-16, pending)

- [ ] Obtain the 593-record 2025 HSK grammar TSV on a network-enabled machine, review source provenance/license and record SHA-256 through a future native HSK importer.
- [ ] Resolve any malformed TSV rows using the official 2025 grammar syllabus, not a guessed field shift; check the import report and validate with a future native HSK reference audit.
- [ ] Classify draft teaching entries into core constructions, variants, usage/discourse patterns and explicit parent relationships, with Mandarin editor review; verify every HSK item mapping rather than accepting candidate example matches.
- [ ] Audit initial Grammar course 1 (four points) against the 5-6-point planning constraint; change its content only after appropriate linguistic review.
- [ ] Add independent, sufficient assessment questions and verified mastery evidence only after HSK/editorial review; run WASM and E2E gates under `/` and `/zipeek/`.

## R488 WIP: Complete draft Grammar curriculum acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`; fix reported errors in a new numbered handoff.
- [ ] Run `/courses/grammar` selection and `/courses/grammar/topic/{21,40,41,60,61,80,81,100}` browser journeys under both `/` and `/zipeek/`. Verify one-question-per-step review, correct-answer gating, viewport/focus and saved step refresh.
- [ ] Verify source TSV and generated Rust remain synchronized: a Rust-native grammar source synchronization check.
- [ ] Obtain Mandarin educator review for all 550 authored entries, repair unnatural examples or misleading grammatical distinctions, and independently map curriculum to HSK requirements.
- [ ] Write at least five independent assessment questions per grammar point, verify evidence and scoring requirements, and only then connect mastery and mosaic reveals.

## R487 WIP: Grammar route ownership compile verification (2026-11-16, pending)

- [ ] Confirm R486 `Route::GrammarLesson` E0382 is eliminated with `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`; run `cargo fmt --all -- --check` and `cargo test --locked -p zipeek-web`.
- [ ] Test five original Grammar URLs, expanded topic routes, draft assessments and back/forward state isolation under `/` and `/zipeek/`.

## R486 WIP: Grammar routing and compiler verification (2026-11-16, pending)

- [ ] Confirm R485 E0382 is eliminated with `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`; run `cargo fmt --all -- --check` and `cargo test --locked -p zipeek-web`.
- [ ] Run `cargo check --locked -p zipeek-e2e --tests` and browser-test authored `/courses/grammar/{intro,time-place,possession-quantity,descriptions,wants-abilities}` and both `/courses/grammar/topic/:region` and `/courses/grammar/assessment/:course` routes at root and `/zipeek/`.
- [ ] Verify invalid slug handling, page header, exit navigation and course switch reinitialization; inspect saved step resumption and AI example state isolation.

## R485 WIP: Grammar authored expansion and lesson UX acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests`; fix compiler/runtime failures with a new numbered handoff.
- [ ] Browser-test `/courses/grammar/topic/6` through `/courses/grammar/topic/100` and exit links at both `/` and `/zipeek/`; verify checkpoints survive reload and invalid region 101 cannot open lessons.
- [ ] Verify all authored Grammar courses expose expected Chinese teaching examples and guided review; checks must not award verified mastery or reveal mosaics.
- [ ] Test AI example states: model not configured, service checking, unavailable, healthy, generating, malformed response, failure, and rapid point changes. Authored examples must remain visible; no AI-generated material is scored.
- [ ] Test Pinyin module arrows, icon exit, keyboard accessible names, top-of-page navigation after switching modules and Grammar steps, reference disclosure and mobile resizing.
- [ ] Review the 75 new Grammar explanations/examples with Mandarin educators, determine pattern prerequisites/HSK alignment and author at least five independent scored questions per point before verified mastery.
- [ ] Complete independent editorial review of the 80 newly authored draft regions and verify the roughly 550-entry taxonomy for genuine semantic distinction.

## R484 WIP: Pinyin / Grammar learning-flow acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo check --locked -p zipeek-e2e --tests`, and the Courses Chrome suite at both `/` and `/zipeek/`.
- [ ] Verify six Pinyin stages each link to two distinct authored modules; all twelve modules teach, offer browser speech for Hanzi examples, grade two written checks and record pass/retry correctly. Refresh after pass; verify module status and no speech/mastery claims.
- [ ] Verify Grammar lesson step resumes after exit/reload, saved best assessment score survives reload and failed storage writes surface errors. Ensure assisted checks and draft scores do **not** unlock mosaic regions or certify per-point mastery.
- [ ] Validate Pinyin phonetics and grammar instructional examples with Mandarin educators. Browser TTS may vary by voice/OS and is not a controlled audio assessment corpus.
- [ ] Author at least five independently scored questions per Grammar point, define approved completion evidence and verify HSK grammar mapping before implementing permanent mastery-driven reveal. Complete the rest of the 100-region Grammar curriculum under separately reviewed editorial scope.
- [ ] Integrate practice checkpoint export/delete into Settings personal-data controls with explicit user approval; the new key is currently browser-local and not included in current JSON backup.

## R483 WIP: Grammar and Vocabulary mosaic acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Run Courses E2E at `/` and `/zipeek/`: verify 100 independent paths per track, no icons within regions, selection keyboard and pointer behavior, centered first authored Grammar course, and retained Vocabulary weighting/acquisition.
- [ ] In Chrome at 1280x720, 1366x768 and mobile 320/390px, inspect curved shared seams for gaps or overlaps, avoid prominent contour noise, verify Grammar and Vocabulary artwork remains legible while no completion is implied, and ensure preview/details viewport behavior remains stable.

## R482 Courses workspace and navigation acceptance (2026-11-16, pending)

- [ ] Run locked Rust unit tests, WASM check, affected E2E compile and formatter; fix any discovered errors in a new numbered WIP handoff.
- [ ] Verify `/courses` redirects to `/courses/vocabulary` and `/courses/grammar` shows the 100 Grammar-only regions, at both root and Pages `/zipeek/` base paths. Test sidebar and lesson exits.
- [ ] Verify tabs show only Vocabulary, Grammar and Pinyin without count sublabels. Check keyboard navigation, active route and focus styling.
- [ ] Verify identical mosaic/preview/detail height allocation across tracks, fixed 166px Vocabulary Explorer in loading, ready, error and selection changes, Pinyin example preview, Grammar preview and no document-level desktop vertical scroll at 1280×720 and 1366×768. Verify mobile 320/390px natural scrolling.

## R481 Vocabulary loading widgets acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and affected Courses E2E tests.
- [ ] In Chrome, open Vocabulary before dictionary initialization completes: verify the details spinner and four Explorer skeletons appear without misleading error or Retry; verify both transition automatically to real controls and words when Ready.
- [ ] Trigger real initialization and query failures; confirm the skeleton is replaced by the corresponding accessible failure/retry state. Verify no double vertical scroll/height jump at 1280×720 and 1366×768, natural mobile scrolling and reduced-motion preference. Check `/` and `/zipeek/`.

## R480 Vocabulary database readiness acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Browser-check `/` and `/zipeek/`: open Vocabulary before database readiness, verify initializing state without `NotReady` error, automatic selected-region word loading at Ready, pagination, source-backed acquisition and preserved selection.
- [ ] Trigger an actual dictionary initialization failure and test provider Retry; trigger a region query failure and test region Retry; verify rapid selection/reinitialization does not display stale words and does not use cached results from a previous dictionary instance.
- [ ] Verify 1280×720 and 1366×768 desktop layout and 320/390px mobile scrolling, including keyboard/focus and accessible status messages.

## R479 Vocabulary mosaic height acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Verify the mosaic illustration covers its allocated height without horizontal or vertical blank bands and all 100 regions remain selectable in Chrome at 1280x720 and 1366x768. Verify the source-backed roadmap shows per-set acquired/available status, with one acquire action and no duplicate word cards.
- [ ] Confirm no document scrollbar on supported desktop viewports; mobile 320/390 widths may scroll naturally. Test both `/` and `/zipeek/`.

## R478 Vocabulary state, feedback and layout acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Browser-test at `/` and `/zipeek/`: selection/reset, rapid switching, failed dictionary load and retry, bounded Explorer paging, Listen buttons, acquisition feedback, duplicate and invalid imported source IDs.
- [ ] Measure document and panel scrolling at 1280x720 and 1366x768; ensure no clipped SVG or controls. At 320/390px, permit natural scrolling and verify touch access, focus and text readability.
- [ ] Confirm caching does not cross region identity and does not cache personal Word Set state; run course E2E against a freshly generated dictionary.

## R477 Vocabulary Explorer and source validation acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo test --locked -p zipeek-e2e --no-run`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Verify 100 selections, four paginated cards, Listen playback, stale-async avoidance and retry state at `/` and `/zipeek/`.
- [ ] Check no document or right-panel vertical scroll at desktop 1280×720/1366×768, and accessible natural mobile scroll at 320/390px.
- [ ] Confirm renaming/editing/deleting an acquired set and importing an invalid source ID have the correct result.

## R476 source-backed Vocabulary acquisition acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Browser-check course acquisition, rename, edit, deletion, reload and import with source IDs; ensure old unlinked sets are not mistaken for acquired.
- [ ] Verify zero document/panel vertical scroll on a typical desktop viewport, no duplicate bottom dashboard, and natural mobile scrolling at 320/390px.
- [ ] Check route layout at `/` and `/zipeek/`, keyboard focus, storage failures and readability of loading/error states.

## R475 irregular Vocabulary mosaic acceptance (2026-11-16, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on the R475 source.
- [ ] Browser-check 100 irregular pieces, region labels, focus/keyboard behavior and hit targets at `/` and `/zipeek/`; inspect 320, 390, 768, 1280 and 1920 viewport widths.
- [ ] Verify acquire-next-set and per-set actions, acquired progress, persistence across reload, collapsed preview/HSK disclosures, dictionary error/retry and Word Sets storage error states.
- [ ] Confirm SVG polygon boundaries remain aligned on Chrome/Firefox and touch; the polygon model is statically checked but not yet browser-accepted.

## R474 Vocabulary geometry compile-fix acceptance (2026-11-15, pending)

- [ ] Run `cargo fmt --all`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on R474. Confirm the R473 delimiter error is gone and report any further compiler errors.
- [ ] Browser-check 100 mosaic tiles, focus and touch interactions, and Word Set acquisition at both `/` and `/zipeek/` once WASM compilation succeeds.

## R473 scattered Vocabulary mosaic acceptance (2026-11-15, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on the R473 source.
- [ ] Browser-check all 100 areas on `/` and `/zipeek/`, including minimum tile dimensions, scattered large regions, no overlapped/missing regions, click/keyboard focus, and selection-to-Word-Set behavior.
- [ ] At 320, 390, 768, 1280 and 1920 widths, verify touch usability and consider a separately approved zoom/pan view where SVG dimensions scale too small.
- [ ] Confirm no implementation-only region numbering or tile-size explanatory copy appears in the Vocabulary panel; assert theme title, HSK counts and acquisition remain visible.

## R472 Vocabulary proportional mosaic acceptance (2026-11-15, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on the exact R472 source handoff.
- [ ] Run the Courses Chrome E2E suite at both `/` and `/zipeek/`; inspect tile proportions, IDs, click and keyboard selection, active/focus outlines, and coverage of the artwork.
- [ ] At 320, 390, 768, 1280 and 1920 widths, visually inspect and test touch/pointer access to the smallest Vocabulary tiles; keep area proportionality, and request approval for zoom navigation if direct targets prove too small.
- [ ] Verify region ordering/layout after curriculum regeneration and Word Set acquisition unaffected by display geometry.

## R471 Vocabulary editorial and runtime acceptance (2026-11-15, pending)

- [ ] Perform human linguistic review of `docs/learning-practice/VOCABULARY_REVIEW_QUEUE.csv`, especially 5,000+ part-of-speech fallback entries; correct the generated taxonomy and re-audit every pinned HSK source index. Source-level full coverage is implemented, not manually curated semantic accuracy.
- [ ] Confirm all 12,188 pinned (simplified, numeric-pinyin) reading identities resolve against the installed DuckDB bundle, including different readings of the same simplified spelling.
- [ ] Run Rust-native vocabulary generation and source coverage checks after each taxonomy change.
- [ ] Run `cargo fmt --all`, `cargo test --locked -p zipeek-pedagogy`, `cargo test --locked -p zipeek-web`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Browser-check all 100 selectable Vocabulary regions and multiple Word Sets per large region at both `/` and `/zipeek/`, including missing dictionary entries, per-set acquisition, reload persistence, personal storage capacity, keyboard navigation, light/dark and responsive UI.
- [ ] Do not infer mastery or mosaic reveals from selecting or acquiring a Word Set; progression needs separately approved learning evidence.

## R469 quiet Hanzi Git acquisition acceptance (2026-11-15, pending)

- [ ] On Windows run `cargo test --locked -p zipeek-cli hanzi::tests::git_checkout_uses_repository_relative_path` and `cargo make prepare-hanzi` from a clean upstream checkout, confirming no clone/checkout progress spam.
- [ ] Confirm sparse checkout, asset generation and cached reuse, and that a failing clone preserves Git stderr and nonzero exit status.
- [ ] Run `cargo fmt --all -- --check` and `cargo test --locked -p zipeek-cli` on Windows. No Cargo execution was available for this source handoff.

## R468 Windows Hanzi Git path acceptance (2026-11-15, pending)

- [ ] On Windows run `cargo test --locked -p zipeek-cli hanzi::tests::git_checkout_uses_repository_relative_path` and `cargo make prepare-hanzi` with a clean `external/hanzi-writer-data` directory; confirm `git clone` works from a canonicalized `\\?\C:\...` workspace root.
- [ ] Confirm the Git sparse checkout provides `data/` and `ARPHICPL.TXT`, generated files pass asset checks, and a second `cargo make prepare-hanzi` call reuses the validated bundle.
- [ ] Run `cargo fmt --all -- --check` and `cargo test --locked -p zipeek-cli` on Windows. Linux source-level and local Git command checks do not substitute for this Windows acceptance.

## Courses corrections acceptance (2026-11-15, pending)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-cli`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Update legacy E2E tests referring to `data-grammar-play`; assert one Start course action and focused course shell instead.
- [ ] Browser-check Grammar 100 regions only (no pocket gaps), Vocabulary independent 100 regions, Pinyin six regions and updated artwork at `/` and `/zipeek/`.
- [ ] Check authored `static/courses`, `static/practice`, and `static/brand` pack/unpack with renamed and stale files plus rollback.
- [ ] Verify keyboard and responsive lesson stage navigation, existing independent assessment links, and lack of drawer/top panel/footer.
- [ ] Define and implement verified durable per-point assessment progression and reveal before claiming completion support.

## R465 browser English FTS acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-db-gen`, and locked WASM compilation of `zipeek-web`.
- [ ] Run `cargo make db-generate` then `cargo make db-install` against the regenerated artifact; confirm the native read-only BM25 result count is positive and the block size is 262144.
- [ ] In Browse, select `English relevance (FTS / BM25)`, search for `language`, and verify ranked English results, HSK/category/POS filters and next-page navigation.
- [ ] Test both `/` and `/zipeek/`, online and offline extension loading. Verify a missing `fts` extension produces a visible Browse error without breaking Hanzi/Pinyin search.

## R464 FTS and block-size acceptance (2026-11-15)

- [ ] Compile `zipeek-db-gen` and `zipeek-web` with locked versions; run generator unit/integration tests.
- [ ] Regenerate into a fresh directory with DuckDB 2.0 FTS extension available; verify `pragma_database_size()` reports 262144 and `fts_main_words.match_bm25` returns correctly ranked English words.
- [ ] Validate read-only reopened artifact from native and pinned DuckDB-WASM/Tauri; test extension provisioning, offline behavior, size/memory cost and `/` and `/zipeek/` without external network dependencies.
- [ ] Compare existing Browse exact/substr ranking and HSK filters with unchanged baseline; only consider exposing ranked English FTS UI after acceptance.

## R463 dictionary regeneration and Practice eligibility acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-db-gen`, and locked WASM compilation of `zipeek-web`.
- [ ] Run `cargo run -p zipeek-db-gen --locked -- check` and `cargo run -p zipeek-db-gen --locked -- generate --output target/r463-reviewed-dictionary` with a fresh output directory; inspect the complete regenerated bundle and replace the installed dictionary and manifest together.
- [ ] Confirm 上 shǎng and 丁 zhēng remain in dictionary lookup but are excluded as standalone meaning answers, while 上 shàng remains eligible. Verify filtered, Word Set, matching and MCQ flows, and `/` and `/zipeek/` in Chrome.
- [ ] Confirm regenerated DuckDB contains `sense_display.sense_kind`, no migration, and that `words.english` and dictionary Browse retain the original source glosses.

## R462 feathered Practice creature illumination acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on the complete source.
- [ ] Run `cargo test --locked -p zipeek-e2e --test practice_session_mcq_ui -- --test-threads=1` and the Matching session test in Chrome; verify both mask directions and that no centre ornament or `clip-path` remains.
- [ ] Inspect all six themes at 0%, intermediate steps and 100%, including resumed sessions, reduced motion, short screens, `/` and `/zipeek/` paths, and no document scroll. Confirm the one-time finish flourish and no visible clipping seam.

## R461 mixed-orientation Practice progress acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo test --locked -p zipeek-cli asset_preflight`.
- [ ] Test all six themes in Chrome, including horizontal Deer and White Tiger below the quiz, portrait creatures beside it, decoded images, 0/partial/100% progress, and reduced-motion fill.
- [ ] Confirm `/` and `/zipeek/` assets and no document-level scrolling on desktop, short viewports and mobile.
- [ ] Review the art detail/edge quality of the replacement crane and phoenix against the illustrated dragon and qilin in browser.

## R459 Practice creature image and viewport acceptance (2026-11-15)

- [ ] Run locked wasm32 compilation, formatting, and both focused Practice session E2E targets.
- [ ] Verify each of the six creature images decodes correctly (nonzero `naturalWidth`) in Chrome at `/` and `/zipeek/`, at 0%, partial and complete progress, and while changing appearance presets.
- [ ] Verify Practice has no document-level vertical scrollbar at normal and short desktop sizes; long/zoomed exercise content must remain accessible through its own scroll area.
- [ ] Inspect WASM bundle growth caused by embedded creature images. Reconsider asset delivery once the published Trunk asset root is reliably verified, without reintroducing broken images.

## R458 theme-specific Practice progress acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo test --locked -p zipeek-cli asset_preflight` and the focused `practice_session_mcq_ui` / `practice_session_matching_ui` E2E targets.
- [ ] In Chrome test all six Appearance presets before entering Practice and verify correct creature assets, full-height alignment, asset loading under `/` and `/zipeek/`, Animato fill at 0/20/40/100%, resumed sessions and reduced-motion presentation.
- [ ] Confirm Matching and MCQ show no visible generic exercise-type heading, but retain accessible section labels and functional prompts and feedback.

## R457 illustrated Practice animation acceptance (2026-11-15)

- [ ] Build and format with `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo test --locked -p zipeek-cli asset_preflight`.
- [ ] Run `practice_session_mcq_ui` and `practice_session_matching_ui` in Chrome at root `/` and hosted `/zipeek/` paths. Confirm icon-only Leave with accessible name, MCQ feedback without redundant icons, and the fullscreen shell without global chrome.
- [ ] Visually check detailed dragon artwork and bottom-up Animato reveal at 0/20/40/100%, step changes and resumed sessions, short/tall viewports and all active themes. Reduced-motion mode must reveal the target immediately. Ensure the rail never reserves empty height outside its illustration.

## R456 Practice dragon animation acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` on R456.
- [ ] Verify in Chrome that the dragon fill rises smoothly after completing each exercise and on resumed sessions; with reduced motion enabled, it changes immediately.

## R455 Word of the Day meaning width acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` against R455.
- [ ] Run `cargo test --locked -p zipeek-e2e --test browse_interactions -- --test-threads=1` in Chrome. Inspect short and long definitions in Word of the Day and expanded Browse meanings; short navigation must not span the card, long definitions wrap, arrows and tags remain usable, and no horizontal overflow occurs on 320px screens.

## R454 Practice session presentation acceptance (2026-11-15)

- [ ] Run `cargo fmt --all`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests` on the R454 source.
- [ ] Run `cargo test --locked -p zipeek-e2e --test practice_session_matching_ui -- --test-threads=1` and `cargo test --locked -p zipeek-e2e --test practice_session_mcq_ui -- --test-threads=1` against Chrome.
- [ ] Visually verify theme wallpaper at `/` and `/zipeek/` in all six presets and light/dark modes, with image loading, contrast, keyboard navigation, portrait/mobile breakpoints and reduced-motion settings.
- [ ] Verify bottom-to-top dragon fill at session start, after each completed step, and on completion, with no exercise-position text or horizontal bar. Confirm Leave session retains resumable progress and the shell has no app header, drawer or footer.
- [ ] Confirm MCQ wrong/correct answer tile states and feedback, accessible Hanzi prompt, and setup selected check indicator without Selected pill.
- [ ] Confirm Word of the Day single-meaning presentation has no inert chevrons and the unavailable Sentences action is neutral across theme presets.

## R453 Practice session E2E migration verification (2026-11-15)

- [ ] Compile Rust E2E targets (`cargo check --locked -p zipeek-e2e --tests`) and run `learning_header_tabs` plus the platform route/bootstrap suites in Chrome; confirm setup navigation, dedicated session shell and retired-route Not Found behavior at `/` and `/zipeek/`.
- [ ] Migrate the remaining pre-R444 direct Practice exercise journey suites to sessions created via setup, retaining their answer/feedback assertions. Do not restore old URLs or delete useful coverage merely to obtain green tests.

## R452 Yew macro compilation acceptance (2026-11-15)

- [ ] Run `cargo fmt --all`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo clippy --locked --target wasm32-unknown-unknown -p zipeek-web -- -D warnings` against R452; address subsequent errors if any.
- [ ] Chrome-check Matching selection/feedback and MCQ class states, and Matching 2-7 configuration settings after the build succeeds; run the existing `practice_session_matching_ui` and `visual_workflows` E2E targets.

## R451 whole-app visual and integration acceptance (2026-11-15)

- [ ] Run Rust formatting, locked WASM check, cli asset preflight/unit tests and representative E2E `visual_workflows`, `practice_session_matching_ui`, `courses_tracks_regression`, `course_interactions`, `browse_interactions`, `responsive_matrix`, and platform onboarding tests.
- [ ] At both `/` and `/zipeek/`, verify actual image loads and visual contrast across the three Courses mosaics, correct selection and authored placeholders, natural scrolling and aligned detail-panel heights. Test all six theme presets in light and dark modes and at 320, 390, 768, 1280 and 1920 widths.
- [ ] Verify real session shell marker R451, with no header/sidebar/footer, and Matching correct/incorrect pair flows (2 through 7 pairs), MCQ selected state, keyboard and reduced-motion behavior, session reload/resume and setup scrolling.
- [ ] Review the remaining direct-route E2E suites for obsolete pre-R444 Practice routes; migrate meaningful scenarios to the two-route session model rather than deleting them or adding compatibility aliases.
- [ ] Verify background and vendor provisioning with `cargo make setup`/`cargo make verify-assets`; source handoffs deliberately exclude generated directories. Classify any new compiler failures from user logs and fix before calling the release accepted.

## R450 Practice session and Matching runtime acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo test --locked -p zipeek-e2e --test practice_session_matching_ui -- --test-threads=1`.
- [ ] Restart the dev server, hard-reload Chrome, and confirm the Practice session root has `data-source-checkpoint="R450"`, `data-shell-mode="practice-session"`, with no `[data-component="app-shell"]`, navigation drawer or `[data-app-footer]`.
- [ ] Verify a left Matching tile visibly shows Selected and a highlighted border, a correct answer marks both tiles Matched, wrong answers give visible feedback, and the chosen pair count persists across a session created from setup.
- [ ] Test narrow viewport, keyboard focus, reduced motion and at least two theme presets, then record Chrome evidence before marking the visual regression accepted.

## R449 Practice setup configuration refactor acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo test --locked -p zipeek-web --lib`.
- [ ] Chrome-verify Choose/Configure switching, available settings tabs, Matching/MCQ, Character Hunt, Type Pinyin/Character Builder answer options, Sentence Translation direction, and loaded profile behavior.
- [ ] Confirm saved session configuration and existing data attributes/ARIA controls are unchanged. No extra storage keys, unsupported settings or duplicate registries.

## R448 Practice setup and Matching acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and locked tests for `zipeek-exercises`, `zipeek-web`, and `zipeek-e2e`.
- [ ] Chrome-verify empty initial selections, clear selected/unselected contrast, filled wizard dots, scrolling all exercise cards at small heights, and setup navigation on mobile and desktop.
- [ ] Verify saved 2-7 pair counts restore in session JSON and display exactly that many pairs; repeat with distinct-word shortage, wrong/right interaction, keyboard use, animation reduced motion and all themes.
- [ ] Confirm `/practice/session/:id` has no global header, drawer or footer; Leave session preserves the saved record, and `/zipeek/` hosted routes continue to work.

## R447 Pinyin lesson acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo test --locked -p zipeek-e2e --test course_interactions`.
- [ ] Superseded by R484: confirm all twelve Pinyin modules render within six mosaic stages, Next module navigation works, and written checks persist only their own pass status; no self-awarded speech mastery or reads/writes to the obsolete `zipeek.course.pinyin.completed.v1` key.
- [ ] Check Settings personal-data export/import accepts Word Sets and exclusions while rejecting the retired `course_progress` category. No migration or automatic browser key cleanup.

## R446 onboarding and Home visual acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check`, locked wasm32 build and the platform shell onboarding E2E suite. Confirm preview updates `data-theme-preset` and the background before storage changes; Back restores the original appearance, and successful Start persists preset and consent together.
- [ ] Review all six presets, light and dark modes, reduced-motion settings, keyboard focus and mobile widths at `/` and `/zipeek/`; confirm Home Word of the Day heading/card alignment and short/long meaning wrapping.
- [ ] Verify the six `public/backgrounds/*.webp` images are provisioned by the approved asset workflow: the examined R445 source ZIP omitted the public directory despite references. Do not silently substitute generated assets.

## R445 compiler remediation acceptance (2026-11-15)

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` on Windows; confirm Character Hunt summary HTML and removed `Route::practice` references compile, then report remaining diagnostics.
- [ ] Run the revised Practice setup-to-session flow and Character Hunt game in Chrome; older direct-route E2E assertions still need rewriting for the two-route model.

## R444 session-only Practice acceptance (2026-11-15)

- [ ] Compile `zipeek-web` with locked WASM target and execute session-store unit tests; verify UUID v7, serialized ISO timestamps, config references, revision-safe progress and generated exercise recovery.
- [ ] Chrome-verify creating and resuming fixed and translation-only Endless sessions from setup, switching tabs, completion evidence, optional recording and cleanup.
- [ ] Retire or rewrite historical E2E scenarios that intentionally navigate to removed direct Practice exercise URLs. Validate old URLs return Not Found, not a compatibility alias.
- [ ] Confirm existing records under the v1 prefix are not silently deleted, and document deliberate clearing when old-format data fails validation.

## R443 game-route acceptance (2026-11-15)

- [ ] Verify all twelve game routes and their gallery links through a locked WASM build and Chrome route-matrix suite at `/` and `/zipeek/`.
- [ ] Verify an unknown `/games/:slug` renders Not Found and that Cloud Match/Crosswords preserve their app-shell layout.

## R441 E2E verification (2026-11-15)

- [ ] Compile and execute `courses_tracks_regression` and `service_monitor_regression` on Windows/Chrome using the R441 source; review the recorded scenario evidence.
- [ ] Run the existing platform shell consent and browser native-service tests; confirm `/zipeek/` hosted route behavior in the appropriate deployment harness.

## R439 Grammar lesson player acceptance (2026-11-15)

- [ ] Compile the R439 Grammar inline lesson player for wasm32 and run the Courses browser suite; verify Play, Previous, Next, close and switching among the five authored courses in both root and `/zipeek/` deployments.
- [ ] Verify optional AI generation against a configured provider, including missing model, error response, malformed two-line output and stale-response isolation when changing grammar points. Generated examples are not accepted mastery evidence.
- [ ] Check right-panel scrolling and focus behavior on desktop and mobile; existing full lessons and independent assessments must remain reachable.

## R437 Courses three-track acceptance (2026-11-15)

- [ ] Compile R437 for wasm32 and the Courses E2E suite against the exact source archive, then verify Vocabulary/Grammar/Pinyin routing on both hosted base paths.
- [ ] Verify four authored Vocabulary collections resolve real dictionary entries and can be saved atomically to personal Word Sets; test duplicate names, storage failures, reload and keyboard access.
- [ ] Author the remaining 46 Vocabulary course memberships and connect mastery-derived reveals only after verified learner evidence is available.
- [ ] R438: Verify all three packaged artworks and the curved Vocabulary regions on desktop/mobile in light/dark themes at `/` and `/zipeek/`; confirm keyboard selection, persistent Word Set behavior, local Pinyin completion opacity, and no unverified mastery reveal. Run the locked WASM build and asset preflight.

- [ ] R435: Verify first-run theme and Practice-history choice on Chrome: default off, explicit opt-in, persistence after reload, immediate dragon indicator, Settings → Learning Data updates, disabled recording suppression; check under `/` and `/zipeek/`. Run locked WASM web compilation and the existing platform shell E2E suite.
- [ ] R431: Compile and validate Docs refactor (`cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo test --locked -p zipeek-web --lib`, Clippy), then Chrome test index/article loading, stale request suppression, source toggle, same/cross-document anchors, direct route queries, reload/history, mobile navigation and `/zipeek/` base. Confirm the workspace UUID feature selection builds both consumers.
- [ ] R430: Verify UUID v7 WASM randomness feature with `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and the two-tab warning flow in Chrome.
- [ ] Validate R428 UUID v7 cross-tab coordination and module refactor on Windows/WASM/Chrome: formatting, locked WASM build, native unit tests, Clippy; two-tab hello/heartbeat/bye/timeout, malformed payloads, repeated mount/unmount cleanup, and Tauri-mode exclusion. Warning only; no writer lock is claimed.
- [ ] Verify R427 on Windows: run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, and `cargo clippy --locked -p zipeek-web --target wasm32-unknown-unknown -- -D warnings`; test other-tab presence/expiry, Practice completion timestamps and time-seeded game flows in Chrome.
- [ ] Verify R426 Courses E2E compile repair on Windows with `cargo check --locked -p zipeek-e2e --tests`, then execute `cargo make e2e-exercise-types-and-courses` and confirm six Pinyin lesson journeys.
- [ ] Run R424 authored Practice full-session Chrome suite with `cargo make e2e-authored-session-completion`; verify five exercise types reach the local summary, history cleanup prevents stale session replay, and the audio fixture works in every dictation round. Run `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Run R423 focused Chrome E2E: `cargo make e2e-ai-games-and-exercise-completion`; verify all four AI rounds, wrong-answer correction, restart, failure presentation and authored Practice grading, including silent-TTS dictation. Run `cargo test --locked -p zipeek-e2e --lib harness::ai_fixture` and `cargo check --locked -p zipeek-e2e --tests`.
- [ ] Run R422 focused Chrome E2E: `cargo make e2e-exercise-types-and-courses`; review all six Pinyin completion/reload assertions, all sixteen exercise configuration tabs, and five authored exercise journeys. Expand audio, stroke tracing and AI exercise completion when deterministic fixtures support them.
- [ ] On Windows validate R419 isolated native DuckDB connections: run `cargo test --locked -p zipeek-app --lib database::tests`, `cargo check --locked -p zipeek-app`, open the desktop dictionary, commit/read/erase Practice history, verify SQL Debug cannot select personal tables, and confirm persistence after restarting Tauri.
- [ ] On Windows verify R418 typed Tauri IPC errors: cargo check/test for zipeek-app, WASM check for zipeek-web, start/stop missing-binary and external-process error surfaces, and native DuckDB query rejection serialization.
- [ ] Verify R417 Windows compilation and formatting (`cargo fmt --all --check`, `cargo check --locked -p zipeek-core`, `cargo check --locked -p zipeek-app`) and run the Tauri desktop smoke test.
- [ ] Validate R415 service context/hooks and R411 Tauri-managed services on Windows: compile `zipeek-app`, `zipeek-web` WASM, place built server binaries beside the app or configure absolute-path overrides, start/stop both, verify healthy/starting/exited/unowned states, close app to terminate only owned processes, and test web-only Settings has no native controls. Production AI backend and Tauri bundle sidecars remain separate work.
- [ ] Browser-verify automatic Debug DuckDB fresh WASM and OPFS probe after selecting DuckDB tab, including storage tags, failure diagnostics and repeated navigation; check expanded POS SQL example and WASM compilation.
- [ ] Verify R396 Practice generation loading state is centered, displays completed-exercise progress accurately (including endless sessions), and handles failures/retries in Chrome; run native SDXL regression tests and a long-prompt CUDA job to verify the 77-token truncation behavior.
- [ ] Chrome-verify R395 compact warning width, two-track Courses navigation across mobile and desktop, focus/hover/active states, mosaic/details alignment and no avoidable desktop scrolling.
- [ ] Chrome-verify R394 Grammar/Pinyin opacity (80% at rest, 100% on hover/focus), theme contrast, keyboard selection and alignment of the Grammar details with the artwork at desktop and mobile widths.
- [ ] Review Icon Pairs semantics with a Mandarin speaker, verify IconData variants compile, browser/keyboard/mobile acceptance, and plan an expanded, reviewed icon-Hanzi mapping catalogue before claiming thousands of pairs.
- [ ] On Windows run `cargo make prepare-hanzi` from a fresh checkout with Git installed; verify the sparse checkout contains only `data/` and `ARPHICPL.TXT`, the generated bundle passes preflight, and a second run reuses it without network access. Run `cargo test --locked -p zipeek-cli`.
- [ ] Browser-test China Map Challenge controls, map dragging and region selection at narrow viewports in all themes; run native Rust and managed Chrome E2E suites. No additional game is approved yet.
- [ ] Browser-verify compact Home layout, accessibility of disabled Sentences explanation, meaning navigation, and dedicated sentence generation against an available and unavailable AI model. No background model polling.
- [ ] Chrome verify R387 Practice exercise-list internal scrolling at 900-1200px widths, non-overflowing Image Generation controls, working Docs brand icon, and Save/Test/Reset actions without `None` warnings.
- [ ] Verify R385 Practice setup viewport sizing in Chrome for short and tall displays, mobile overflow, keyboard scrolling in exercise cards, and contrast of selected/hover/rest states in light and dark themes.
- [ ] Chrome-verify the R384 clean Grammar/Vocabulary and companion Pinyin artwork in both themes and at desktop/mobile widths; inspect the absence of compositing seams and confirm correct cache-busted static asset requests.
- [ ] Verify the softer Courses region highlight in Chrome and confirm the selected grammar region no longer obscures the bridge artwork.
- [ ] Validate the expanded six-module Pinyin mosaic and lesson flow, including ordering, completion persistence and mobile layout.
- [ ] Run the R382 `zipeek-cli` unpacker Rust tests and a real R382 source unpack on Windows. Confirm `static/courses/` assets are restored, generated dictionary assets preserved, and rollback still works on failure.
- [ ] Verify both mosaics in Chrome after unpack: artwork loads from the new PNGs, no horizontal letterboxing, full region interaction, desktop no-scroll where practical, and narrow-screen scrolling.
## R378 - Courses visual and route acceptance (2026-11-13)

- [x] Put Pinyin & Pronunciation in the Courses collection navigation rather than a floating button.
- [x] Remove the redundant Courses introduction and the extra preview footer; keep one concise unavailable-progress notice.
- [x] Replace the blurred placeholder artwork, update source asset preflight and add focused tests.
- [x] R379: Generate a new self-contained lantern-bridge art asset and replace straight-looking mosaic edges with deterministic shared cubic curves.
- [ ] Historical course mosaic concept parity request (concept image retired; not a current visual acceptance requirement): unmasked richly detailed autumn-night artwork, organically shaped contiguous grammar regions, tiny vocabulary pockets, and perfectly aligned interactive paths. Do not treat the current illustration as the approved final art.
- [ ] Verify the new versioned Courses artwork assets (`autumn-lantern-bridge-r381.png`, `pinyin-pronunciation-r381.png`) load correctly in Chrome without stale-cache blur on Grammar and Pinyin mosaics.
- [ ] Validate no unwanted scrolling at desktop viewport sizes, no distorted artwork at mobile sizes, correct focus/tap interaction, and both themes in Chrome.
- [ ] Run locked Rust/WASM build, asset preflight and Trunk production build; report actual results.

## R376 - Stats and Docs UI acceptance (2026-11-13)

- [x] Remove nonfunctional statistics tabs while pedagogical learning evidence is unavailable.
- [x] Support direct routing to Settings sections through validated `section` query parameters.
- [x] Remove duplicate Docs counters, improve folder labels/search/actions and article navigation.
- [ ] Verify source code with `cargo fmt --all -- --check`, locked WASM check, Trunk Pages build and Chrome on desktop/mobile and light/dark themes.
- [ ] Rebuild and verify the published Docs mirror/index using `cargo make docs`; the snapshot already fails the Docs publication equality check.
- [ ] Confirm the hash-router Settings deep links survive refresh and browser back/forward navigation.

## R375 Courses mosaic browser acceptance

- [ ] On Windows run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `trunk build --public-url /zipeek/` from `crates/zipeek-web/` after asset preparation.
- [ ] In Chrome verify the preview art loads, all 150 regions remain navigable at desktop and mobile widths, vocabulary replaces grammar in the same details panel, the missing-data banner is clear, and Pinyin remains accessible.
- [ ] Verify narrow viewport scrolling, both light/dark themes and alternate theme presets, focus and zoom/pan navigation, and existing grammar lesson routing. Do not describe preview selection as mastery or persisted progress.

## R374 - Trunk selects the intended web binary (2026-11-13)

- [x] Select `zipeek-web` explicitly in Trunk HTML while retaining the standalone pedagogy worker binary.
- [x] Add a focused source-contract test for explicit main-binary selection.
- [ ] Confirm Trunk `serve`, Pages build and browser startup on Windows with installed toolchain.
- [ ] Audit the reported dead-code warnings and decide whether unused service APIs are obsolete before modifying them.

## R373 - Remove unnecessary page `#[path]` overrides (2026-11-13)

- [x] Organize `pages` into conventional `system`, `explore`, `home` and `learning` modules without moving source files or changing route-facing page names.
- [x] Update source-contract tests and run focused module checks; compare broader failures against the previous source baseline.
- [ ] Validate `cargo fmt --all -- --check`, locked wasm32 compilation, and Trunk/Chrome behavior in the supported development environment.

## R371 - Generated static-asset preflight (2026-11-13)

- [x] Introduce Rust source/deployment asset verification for the full Docs source mirror, search index, Courses bridge artwork, worker bootstrap and WASM bindings.
- [x] Add fail-closed checks for stale Docs, missing/partial outputs, unsafe index paths and invalid WASM signatures; add negative unit tests.
- [x] Run strict verification in Pages before Trunk; provide source/deploy Cargo Make tasks.
- [ ] Run native Cargo tests and actual Pages/Windows worker builds; close TD-009 only after acceptance.

## R370 Docs - Link safety and browser acceptance (2026-11-13)

- [x] Prevent unsafe Markdown links from creating clickable placeholder anchors; preserve text content.
- [x] Intercept same-article anchor clicks without overwriting the Docs hash route.
- [x] Add Rust safety fixtures, focused Node contracts and a Chrome Docs reader scenario.
- [ ] Run locked Rust/wasm32 builds and the new Chrome scenario; attach diagnostics (TD-001/TD-006).
- [ ] Verify R429 cross-document heading query routes and post-render scrolling in Chrome, including reload, browser history, CJK headings, duplicate heading IDs, root and `/zipeek/` deployment. Rust and WASM acceptance remain pending.

## R369 Docs - Markdown rendering (2026-11-13)

- [x] Migrate handwritten Docs parser to pinned Rust `pulldown-cmark`; retain full source publication and Yew/WASM direct calls.
- [x] Handle nested Markdown/GFM, reference-style links, safe destinations and HTML escaping; share TOC/heading anchors.
- [x] Add Rust fixtures and Node source-contract tests.
- [ ] Run locked Cargo native and wasm32 checks; confirm dependency resolution.
- [ ] Run Chrome tests under `/zipeek/`, inspect heading navigation and Markdown styles before closing TD-006.

## R368 Courses - Fifth authored course and maintainability (2026-11-13)

- [x] Add course 5 with five authored grammar points, guided questions, six word-bank exercises, one draft MCQ per point and an independent construction challenge.
- [x] Wire the stable registry, route and mosaic region 5; make region 6 the next unauthored region; keep all 150 regions concealed.
- [x] Centralize mosaic accessible labels through authored course metadata rather than repeated titles; extend browser scenario and source tests.
- [ ] Run pinned Rust/WASM compilation and managed Chrome Courses journey on the exact R368 source and fix diagnostics (TD-001).
- [ ] Arrange qualified Mandarin review for all 24 draft MCQs and five construction tasks, especially modal semantics and context (TD-003).


## R367 Courses - Shared authoring validation and maintenance (2026-11-12)

- [x] Add a single structural validation gate across registered grammar points, guided questions, word banks, draft MCQs and unassisted construction items for all four published courses.
- [x] Add negative Rust tests for missing text, question coverage, invalid choice keys, duplicate answers/IDs and invalid tile permutations; retain the existing separate assessment validators.
- [x] Remove duplicated guided-course region numbering from the lesson UI and describe the authoring workflow in Docs.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, locked wasm32 build and Chrome Courses E2E on Windows; fix real compiler/browser issues and record evidence (TD-001).
- [ ] Obtain qualified Mandarin-language and HSK syllabus review before treating any draft score as authoritative or revealing artwork (TD-003/TD-007).

## R366 Courses - Descriptions & Comparisons (2026-11-12)

- [x] Author five explanation/example points, five guided checks, six guided word-bank interactions, five independent draft MCQs and one unassisted sentence construction for a fourth course.
- [x] Register stable `g04-` grammar-point IDs, recommended prerequisites and pending syllabus status; route mosaic region 4 to the shared grammar lesson.
- [x] Keep all grammar and vocabulary covers concealed and scores in memory; add source and native test coverage.
- [ ] On pinned Windows toolchain run Rust format/tests, wasm32 check and Chrome `course_interactions`; fix any actual errors and verify the fourth course with keyboard and screen reader.
- [ ] Independently review Mandarin wording (especially degree 很 and comparisons) and authoritative HSK mapping; no persistence or progression without separate approval.

## R365 assessment accessibility and acceptance (2026-11-12)

- [x] Add live, readable assessment progress and review navigation to original scored questions without using hash fragments (Yew hash-router safety).
- [x] Announce independent construction selected-tile progress; adapt E2E selectors for richer labels.
- [x] Remove hardcoded response-choice size from the assessment session and add a regression test for invalid-choice preservation.
- [ ] Run Rust unit tests, wasm32 compilation and Chrome Courses E2E; verify screen-reader output and keyboard focus, then attach results under TD-001.
- [ ] Obtain qualified Mandarin item review and approve independent assessment provenance/completion policy before any awards or artwork reveals (TD-003).

## R364 independent sentence construction and acceptance (2026-11-12)

- [x] Add one authored independent sentence-construction challenge per currently published grammar course, using the existing stable grammar-point IDs; retain all 14 independent MCQs.
- [x] Reject malformed tile banks and duplicate choices; keep draft answers unscored until locked submission, provide undo/remove/reset and a new attempt, and reveal the correct construction only in post-submission review.
- [x] Add native Rust unit scenarios, browser E2E interactions and focused Node contracts. Ensure artwork remains concealed and no award/persistence authority is introduced.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, locked WASM check, and `course_interactions` in managed Chrome. Fix reported actual compiler/browser failures.
- [ ] Obtain qualified Mandarin-language item review and decide approved assessment provenance, retries, persistence and independently evidenced completion before any mosaic unlock (TD-003/TD-007).

## R363 grammar assessment diagnostics and content review (2026-11-12)

- [x] Add a read-only, per-grammar-point review that is derived exclusively from a submitted and matching authored assessment snapshot.
- [x] Fail closed on mismatched content and reset an assessment's in-memory session when navigating between courses.
- [x] Inventory all 14 authored assessment questions and the unresolved linguistic review requirements; add focused Rust and Node contracts and Chrome assertions.
- [ ] Run the pinned Rust/WASM toolchain and managed Chrome; assess language/content quality independently before any mastery evidence or mosaic reveal integration.

## R362 Assessment stabilization and debt hardening (2026-11-12)

- [x] Bind draft assessment scoring to the complete immutable authored question snapshot, not just matching IDs/order; reject changed answer keys, distractors and rationales.
- [x] Reject whitespace/punctuation-only distractor variants using orthographic normalization; continue to require qualified review for semantic ambiguity.
- [x] Extend the managed Chrome scenario to check retry clears responses and that a genuinely incorrect answer scores lower only after submission.
- [ ] Run pinned native Rust/WASM compilation and the managed Chrome Courses scenario, investigate any actual failures, and record acceptance evidence under TD-001/TD-004.
- [ ] Perform qualified Mandarin assessment review and decide separate provenance/completion rules before enabling awards or mosaic reveals (TD-003).

## R361 Assessment debt reduction and validation (2026-11-12)

- [x] Extract stateful draft assessment sessions from authored content, bind scored attempts to ordered question IDs, and add quality guards for structurally equivalent answer options.
- [x] Add the ongoing [technical debt register](../project-management/technical-debt.md) and link it from the documentation index. Preserve all 14 authored grammar-point mappings and no-award behavior.
- [ ] Run pinned Rust formatting, native unit tests and WASM compilation, plus the managed Chrome Courses assessment scenario; resolve actual failures.
- [ ] Conduct expert Mandarin item/distractor review and approve assessment provenance/completion policy before durable awards or mosaic reveals.

## R360 Independent grammar assessment draft (2026-11-12)

- [x] Author a separate draft assessment item for each of the 14 stable grammar-point IDs across the three published Courses; no lesson answer hints during the attempt.
- [x] Provide an in-memory, answer-locked Rust session with complete-submission validation and score only after submitting all questions; add a separate Courses assessment route and UI.
- [x] Add focused Node contracts, native Rust unit tests, and managed Chrome scenario checkpoints; preserve mosaic concealment and existing Pinyin lessons.
- [ ] Run pinned native Rust/WASM tests and managed Chrome browser scenario, review all draft prompts and distractors with a qualified Mandarin reviewer, and verify keyboard/mobile accessibility.
- [ ] **Separate approval gate:** define authoritative grammar-point assessment quality, versioned provenance, consent, durable completion, retries/cooling-off, and permanent reveal predicates before any score is persisted or grants mastery.

## R359 Courses curriculum registry (2026-11-12)

- [x] Register the three authored courses with 14 stable grammar-point IDs, local teaching stages, recommended prerequisites and explicit unverified-syllabus status.
- [x] Validate uniqueness, authored-content/question counts and prerequisite direction using native Rust tests; add focused Node source contracts and in-page authoring metadata.
- [ ] Run Rust unit tests, WASM compilation and the managed Chrome Courses E2E on Windows; resolve any compiler or runtime diagnostics.
- [ ] Independently obtain and audit the authoritative HSK grammar syllabus edition, attach source-level item references, review linguistic wording and map all remaining courses and assessments.
- [ ] Keep prerequisite recommendations separate from access locks, personal tracking, awards and reveal state until independent assessment and consent authority are approved.

## R358 Courses third authored grammar course (2026-11-12)

- [ ] On Windows run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests`; correct any diagnostics.
- [ ] Run Chrome `course_interactions`; verify all five possession/quantity comprehension checks and six guided word-bank exercises, including incorrect choices, undo/reset, review and replay, and unchanged fully concealed grammar-1/2/3 regions.
- [ ] Review classifier/quantity wording with Chinese-language curriculum expertise and map independently validated HSK grammar objectives before calling the authored lessons comprehensive or assessed.

## R357 Courses second authored grammar course (2026-11-12)

- [x] Author Course 2 on time, place, destination, and location questions with five explanations, five guided comprehension checks, and six word-bank teaching tasks.
- [x] Add mosaic-region selection and Course 2 routing while retaining Course 1 and existing Pinyin lessons; no guided practice awards mastery or reveals art.
- [x] Reuse the typed guided session and Yew lesson UI for both authored courses, extend Rust unit tests and Courses E2E, and add focused Node contracts.
- [ ] Run Rust formatting, native tests, pinned WASM checks, and managed Chrome E2E; examine linguistic nuances with a qualified Mandarin reviewer before using authored distractors in independent assessment.
- [ ] Map courses to the authoritative HSK syllabus and implement separately approved independent assessed completion and durable reveal authority.

## R356 Courses guided word-bank teaching and acceptance (2026-11-12)

- [x] Author six first-course word-bank teaching activities, including Sentence Builder and word-bank Sentence Transformation, with visible references and pattern-specific explanations. Preserve unassessed mosaic state and existing Pinyin completion.
- [x] Add deterministic pure Rust answer/transition tests, focused Node source contracts, and extend the managed Chrome Courses flow through all six activities.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and `cargo check --locked -p zipeek-e2e --tests` on the pinned toolchain; fix any diagnostics.
- [ ] Run `course_interactions` in Chrome, test wrong answers, tile removal, undo/reset, replay and keyboard/screen-reader controls, and confirm no artwork unlock or recognition evidence is persisted.
- [ ] Continue actual HSK syllabus-to-course mapping and design independent, persisted course-point assessment before implementing authoritative reveals; no nominal course IDs imply HSK coverage.

## R355 Courses mosaic acceptance and curriculum expansion (2026-11-12)

- [ ] On pinned Rust run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`; correct all compiler and lint errors.
- [ ] In Chrome validate all 150 selectable map regions, no gaps/overlaps at joins, tiny vocabulary pocket area ratios, selected-region outline only, zoom/pan, mobile/keyboard interaction and `/zipeek/` asset loading. Run `course_interactions` E2E.
- [ ] Independently validate mosaic geometry using rendered bounds and adjacency/area checks; source topology/unit tests do not replace visual overlap inspection. Refine artistic region shape if needed to meet the approved organic irregular boundary target.
- [ ] Map the **actual HSK grammar syllabus** to authored courses and publish the remaining 99+ courses only as their grammar points, worked examples, Sentence Transformation and independent assessments are complete. Nominal geometry IDs are not curriculum mappings.
- [ ] Design and validate 50 vocabulary-group memberships and trustworthy distinct-word mastery readback; wire up differential opacity without allowing arbitrary UI state or Pinyin checkmarks to grant proficiency.
- [ ] Connect independent assessed grammar completion to durable, irreversible region reveals only after the separately approved personal evidence pipeline; keep all 150 regions concealed meanwhile.

## R352 Full-run E2E implementation and acceptance (2026-11-12)

- [x] Define two separate E2E entry points with seeded themes, opt-in/off onboarding, full route itinerary, mandatory PNG/JSONL artifacts, configurable 1-120-minute budget, and explicit failure status while finish criteria are incomplete.
- [x] Introduce injectable pure Rust recognition projection clock and test-only monotonic simulated clock with seven-day/24-hour spacing tests. Do not expose fake time through production routes.
- [ ] Execute `cargo test --locked -p zipeek-pedagogy`, `cargo test --locked -p zipeek-e2e --tests --no-run`, and both explicitly invoked ignored full runs on Windows with managed Chrome. Fix diagnostics.
- [ ] Complete standard journey browser acceptance: R354 now drives five graded mixed rounds, Word Set creation, dictionary/writer searches, ten-round scored Province Quest, Settings Save + reload persistence, course lesson, fixture-backed image/translation grading, and a fail-closed OPFS-empty check. Run managed Chrome from a fresh browser profile; investigate any OPFS files instead of silently considering them absent, and verify other possible recognition stores separately if introduced. A source-only implementation is not a passing browser run.
- [ ] Complete pedagogical journey: validate game/practice evidence producers, consent-gated personal OPFS DuckDB journal, verified readback, single-tab ownership, worker restart recovery, and HSK acquisition-to-Dragon integration. Test clock injection must reach actual evidence timestamps, without bypassing the 10×7 rules or award 90% threshold.
- [ ] Confirm screenshot milestones (landing, onboarding, dashboard, practice, word sets, dictionary, writer, games, settings, courses, and finish), root and `/zipeek/` hosting, reproducible failures, budget timeout behavior, and no fake success after a blocked prerequisite.

## R351 Docs reader usability and link resolution (2026-11-12)

- [ ] Run `cargo test --locked -p zipeek-docs` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, then verify Windows/Chrome interactions and GitHub Pages `/zipeek/` deployment.
- [ ] Browser-check previous/next article links, subject expand/collapse, markdown-to-markdown and local image links, blocked docs-root escapes and fenced code examples.
- [ ] Adopt an approved pinned CommonMark/GFM parser for complex Markdown; the dependency-free subset still cannot render every document faithfully.
- [ ] Validate R429 Docs deep links in live Chrome with route queries, post-render scroll and browser back/forward navigation.

## R350 Docs reader acceptance (2026-11-12)

- [ ] Compile `cargo test --locked -p zipeek-docs -p zipeek-cli`, `cargo fmt --all -- --check`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`; fix any compiler diagnostics.
- [ ] Run `cargo run --locked -p zipeek-cli -- build docs` and `trunk build --public-url /zipeek/`; verify all source `docs/` paths and bytes are present and `index.json` count matches, including binary/non-Markdown assets and process history.
- [ ] Browser-test the dedicated Docs shell (no app drawer), domain/topic/document expansion, search, breadcrumbs, deep links, markdown tables, link navigation, code blocks, TOC and mobile keyboard use on `/` and `/zipeek/`.
- [ ] Complete CommonMark/GFM renderer using an explicitly pinned/approved parser dependency; initial safe subset does not support all Markdown constructs and internal relative links/anchors still require end-to-end acceptance.
- [ ] Perform publish-all content/license/credential audit. Do not silently exclude docs files. The read-only UI is for reviewing/refining content; editing requires a separately designed workflow.

## R349 typed worker, onboarding and debug SQL acceptance (2026-11-12)

- [ ] On pinned Rust 1.97.1 run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web --bin pedagogy_worker`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web --bin pedagogy_worker`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` (including release profile). Correct all formatting/compile diagnostics before accepting R349.
- [ ] Run `cargo run --locked -p zipeek-cli -- build pedagogy-worker` and verify the generated `start_worker` export and Settings → Pedagogy health check under both `/` and `/zipeek/`; expect disabled, correlated health, proper errors and shutdown.
- [ ] Browser-test landing → theme/opt-in setup → start with storage persisted. Verify opt-in defaults off, revisiting skips setup, settings can withdraw the preference and **no recognition event or OPFS write occurs** even when opted in.
- [ ] Browser-test Debug tabs, SQL table and CSV download on read-only dictionary; verify invalid/multiple/writing SQL is refused, huge result previews cap at 100, UTF-8 CSV quote/formula behavior, loading/failure states, and production builds hide Debug.
- [ ] **Separate open design:** provide a developer-only, independently created/reopened *temporary file-backed* DuckDB-WASM database storage-version probe. Distinguish default engine storage format from packaged dictionary manifest storage compatibility; do not claim the current engine/version/manifest diagnostic already performs this check.
- [ ] **Separate open design:** personal OPFS SQL and export UI requires explicit consent, Web Lock ownership, read-only session isolation, sensitive-data export warnings, query timeouts, bounded results, and no account secrets. The R349 terminal only queries the immutable dictionary.
- [ ] Complete remaining source-contract regression triage and real-browser E2E; scripts alone do not prove WASM execution.

- [ ] R348 acceptance: run `cargo test --locked -p zipeek-cli`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web --bin pedagogy_worker`, and `cargo run --locked -p zipeek-cli -- build pedagogy-worker`; confirm generated bindgen exports `start_worker` and no WASM toolchain errors. The WIP source ZIP intentionally excludes outdated R347-generated worker bindings; generation is required.
- [ ] In Chrome, open Settings → Pedagogy and check worker availability both under `/` and `/zipeek/`; verify startup races, missing bindings, request correlation, worker termination, no evidence writes, and disabled-by-default state. Check worker browser console for errors.
## R347 bootstrap recovery acceptance (Windows)

- [ ] Run `cargo test --locked -p zipeek-cli` and `cargo run --locked -p zipeek-cli -- build pedagogy-worker` on Windows with `worker.mjs` initially missing; confirm restoration and completed bindgen build.
- [ ] Run the deployed Settings → Pedagogy health check in Chrome under `/zipeek/`; confirm worker startup, read-only disabled status, and clean shutdown.

## R347 CLI worker build acceptance

- [ ] On Windows Rust 1.97.1 run `cargo test --locked -p zipeek-cli`, `cargo fmt --all -- --check` and `cargo run --locked -p zipeek-cli -- build pedagogy-worker`; verify the pinned bindgen version and target checks, expected generated JS/WASM sizes, and removal of stale outputs after an induced failed build.
- [ ] Verify GitHub Pages CI builds both worker outputs before Trunk copies static files, and that deployed `/zipeek/pedagogy-worker/` contains `worker.mjs`, `pedagogy_wasm.js` and `pedagogy_wasm_bg.wasm`.
- [ ] Exercise Settings read-only probe in a real browser; no collection/mastery activation.

## R346 Rust worker-monitor migration acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy monitor`, and `cargo test --locked -p zipeek-pedagogy` on pinned Rust 1.97.1; reconcile any compiler or rustfmt diagnostics.
- [ ] Run `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo check --locked -p zipeek-e2e --tests`, `cargo run --locked -p zipeek-cli -- build pedagogy-worker`, and the full Trunk/Pages build. Confirm generated Rust/WASM worker JS and WASM assets are actually included in deployed output.
- [ ] Browser-test Settings → Pedagogy → Check worker on both root and `/zipeek/` hosting: successful disabled health, absent WASM asset, wrong protocol, invalid health, worker error, timeout, and cleanup/termination. Include concurrent out-of-order replies using a deterministic browser Worker fixture; native Rust tests alone cannot prove actual DOM/event integration.
- [ ] Confirm only `get_health` crosses the UI boundary. Do not activate evidence collection or mastery until approved consent, provenance, exclusive-tab/OPFS readback and durable job recovery gates pass.

## R345 Pedagogy Settings diagnostics acceptance

- [ ] Compile the Settings Pedagogy panel with Rust 1.97.1 and the locked WASM target. Check `cargo fmt --all -- --check` and `cargo make build`.
- [ ] Build the isolated pedagogical worker (`cargo run --locked -p zipeek-cli -- build pedagogy-worker`) and browser-test its one-shot Settings health probe under `/` and `/zipeek/`, including missing bundle, timeout and initialization failure. Verify no worker remains running after probe.
- [ ] Maintain the approved consent/provenance/OPFS-readback gates before enabling pedagogy data collection, analysis, mastery, or dragon progression.

## R344 Statistics dashboard verification

- [ ] Compile the new `/stats` Yew route and navigation using pinned Rust 1.97.1 and run `cargo fmt --all -- --check` plus locked WASM build.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test stats_dashboard -- --nocapture` and verify dashboard tabs, keyboard navigation and Settings link in real browsers under root and `/zipeek/` deployment paths.
- [ ] Connect only approved and verified consent-gated personal evidence readback in a later milestone; until then do not show mastery values, HSK acquisition percentages, retention confidence or worker-running claims.

## R343 Time Weaver clock-time expansion acceptance

- [ ] Compile with pinned Rust: `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises time_weaver`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`. Resolve diagnostics before release acceptance.
- [ ] Run the 15-round `time_weaver_gameplay` managed browser E2E under Chrome and Firefox at `/` and `/zipeek/`; verify randomized date assembly, weekdays, clock-time assembly, three Chinese choice variants, review lock, completion and restart.
- [ ] Check small-screen, keyboard and screen-reader interactions on both clock and date tile types; no microphone, personal recording or acquisition evidence should start from game interaction.
- [ ] Further Time Weaver expansions (relative dates, seasons, 日/号 and 星期/周/礼拜 synonyms, listening/speech) need an explicitly approved answer-equivalence policy before coding.

## R342 Time Weaver date and weekday game acceptance

- [ ] On pinned Rust 1.97.1 run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises time_weaver`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`. Correct compiler errors before release acceptance.
- [ ] Run managed Chrome `cargo test --release --locked -p zipeek-e2e --test time_weaver_gameplay -- --nocapture`, then Firefox; verify full fifteen-round completion, no leaked date answer, disabled post-submission controls, keyboard and touch accessibility, and routing at `/` and `/zipeek/`.
- [ ] Clock-time slice added in R343; remaining seasons, relative dates, synonyms (日/号, 星期/周/礼拜) and later listening/speaking require explicit answer-equivalence policy; avoid unverified acquisition evidence.
- [ ] Investigate the inherited Cloud Match `data-sky-scene` source contract failure independently rather than weakening tests during unrelated game work.

## R341 gallery layout verification and future game discovery

- [ ] Run `cargo test --locked --target wasm32-unknown-unknown -p zipeek-web` and browser game-gallery scenarios; confirm Sentence Relay is a 2×2 four-slot card on page two at desktop width, with five links after Time Weaver and working keyboard/touch navigation. Check narrow-screen layout and `/zipeek/` routing.
- [ ] Continue the newly implemented Time Weaver beyond its initial date and weekday slice after verifying the present route, authored dates and acceptance scenarios. Confirm clock-time curriculum and answer-equivalence policies first.
- [ ] Confirm proposed Cosmic Lab content scope and scientific editorial workflow before implementing a route. Require validated Chinese-only three-choice questions, two plausible misconceptions, and source review independent of AI generation.
- [ ] Continue independently approved worker/DuckDB acceptance gates from R340; do not activate learner evidence capture without consent/provenance and verified OPFS readback.

## R340 Courses navigation and worker-health acceptance

- [ ] Compile the Yew Courses dashboard and managed Courses E2E target with pinned Rust; run `cargo test --locked -p zipeek-e2e --test course_interactions -- --nocapture` against managed Chrome and verify all three direct Pinyin lesson links, keyboard focus, completion display, and root/`/zipeek/` navigation.
- [ ] Run `node --test scripts/course-dashboard-navigation.test.cjs scripts/pedagogy-worker-client.test.cjs scripts/pedagogy-worker.test.cjs scripts/province-quest/assets.test.cjs` and verify worker health response schema against a real built Rust/WASM worker in both deployment bases.
- [ ] Flesh out the full approved Courses 100+50 contiguous mosaic only after verified geometry, authored HSK grammar mapping, independent assessments and durable mastery predicates exist. Do not derive vocabulary overlay opacity from locally marked Pinyin lesson completion.

## R339 read-only browser worker monitor acceptance

- [ ] Run `node --test scripts/pedagogy-worker-client.test.cjs scripts/pedagogy-worker.test.cjs` and actual browser startup checks under `/` and `/zipeek/` with the generated WASM bundle.
- [ ] Wire the read-only health monitor into a consent-aware Settings view only after validating the asset URL and real browser Worker lifetime; it does not yet enable analysis or read DuckDB.
- [ ] Retain the pending evidence-v1 mapping, verified OPFS readback, ownership lease, and durable queue approval gates before activating analysis.

## R338 isolated WASM worker acceptance

- [ ] On Windows Rust 1.97.1 run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web --bin pedagogy_worker`, then `cargo run --locked -p zipeek-cli -- build pedagogy-worker` using pinned `wasm-bindgen-cli 0.2.130`.
- [ ] Verify generated `pedagogy_wasm.js` and `.wasm` are copied into `static/pedagogy-worker/` and deployed under both root `/` and GitHub Pages `/zipeek/`, without absolute asset URLs. Run `node --test scripts/pedagogy-worker.test.cjs` and actual browser Worker startup and message/restart E2E.
- [ ] Before Yew integration, design and approve the evidence-v1 to domain mapping, independently verified choice IDs, host-enforced consent, one-tab ownership and genuine readback from `zipeek-db`. Do not trust the worker input's `snapshot_verified` flag as proof of database durability.
- [ ] Implement approved durable pending-job recovery, cross-tab coordination and worker diagnostics before enabling background processing, `/stats` awards, dragon progress or acquisition writes.

## R337 typed worker endpoint acceptance

- [ ] On pinned Rust 1.97.1 run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy`, and the WASM check; fix any failures before accepting R337.
- [ ] Define and approve the actual serializable browser-worker message transport, bundling/initialization of a separate Rust/WASM entry point and consent-gated read-only evidence adapter. The typed endpoint alone is not a running Web Worker.
- [ ] Run worker browser E2E for protocol mismatch, duplicate enqueue, disabled tracking, verified OPFS projections, bounded retries, failures and restart; retain the R336 durable queue and single-tab design approval gates.

## R336 background coordination acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy`, `cargo test --workspace --locked` and WASM check on the pinned Rust toolchain. Resolve compile errors before accepting the checkpoint.
- [ ] Implement and browser-test an actual dedicated Rust/WASM Worker host with a versioned message protocol, typed request validation, no DOM dependencies and observable lifecycle. The new coordinator is a domain scheduler only, not a browser background thread.
- [ ] Design and explicitly approve durable pending-job storage/recovery and single-active-tab ownership before activating automatic analysis. The current in-memory queue does not survive tab termination.
- [ ] Connect the host through consent-gated, verified personal DuckDB reads and enforce single-writer semantics; test disabled tracking, restart, crash, offline, OPFS errors and repeated retries.
- [ ] Do not emit `/stats` mastery, dragon awards or game evidence until the R333-R335 acceptance prerequisites and evidence-provenance rules pass.

## R335 personal recognition DuckDB adapter acceptance

- [ ] On pinned Rust toolchain run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-db --features wasm --target wasm32-unknown-unknown` where supported, `cargo test --locked -p zipeek-pedagogy`, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Browser-test transaction failure/rollback, duplicate exact replay, conflicting IDs, checkpoint and reopen persistence, restart recovery, >100k observations, quota exhaustion, and OPFS errors under the existing Web Lock.
- [ ] Confirm Settings personal-data deletion erases both `practice_sessions` and `recognition_evidence`, including restart/readback and failure recovery.
- [ ] Before attaching games, explicitly approve and implement a canonical `RecognitionEvent` -> wire-v1 conversion, consent gate, and validated-choice provenance. No mastery award may be made before committed readback.
- [ ] Keep `/stats`, AI, worker orchestration, and the dragon out of this storage milestone; paginate/aggregate large histories before using full readback in a dashboard.

## R334 Pedagogy evidence-journal boundary acceptance

- [ ] Compile and test the no-dependency `zipeek-pedagogy::EvidenceJournal` readback boundary with pinned Rust 1.97.1: `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy`, then `cargo test --workspace --locked`.
- [ ] Validate persisted evidence adapters against atomic batch commit, read-after-write equality, duplicate retries, conflicting IDs, interrupted writes and restart recovery before considering journal readback authoritative.
- [ ] Obtain separate approval for any personal DuckDB schema change, game-event capture contract, choice-validation policy, or recording-consent activation. The R334 port is not an activated database adapter.
- [ ] Do not start `/stats` awards, the worker, or Dragon progress from unverified observations.

## R333 Pedagogy foundation acceptance

- [ ] On pinned Rust 1.97.1 run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-pedagogy`, `cargo test --locked -p zipeek-exercises` and `cargo test --workspace --locked`; repair any compiler or test issues before acceptance.
- [ ] Validate the pure 10×7 replay rules against reference sessions and calendar boundaries, including missed dates, failure locking, 0/1/2 allowance snapshots, 24-hour spacing, duplicate attempts and multi-character isolation.
- [ ] Before production capture, approve and implement independently validated choice evidence, assessment-timezone policy and stable Hanzi identity mapping. No game may directly self-award acquisition by fabricating a validation identifier.
- [ ] Complete authoritative personal DuckDB commit/readback, single active tab, interrupted write and OPFS recovery gates before wiring awards, worker or `/stats` to saved learner data.
- [ ] Preserve the independent R259 vocabulary/full-skill projection until its domain ownership is explicitly reconciled; do not silently replace it with the Hanzi dragon rules.

## R332 Province Quest compiler and browser acceptance

- [ ] On approved Rust 1.97.1 run `cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and `cargo check --locked -p zipeek-e2e --tests`; review any new compiler diagnostics before release acceptance.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture` with managed Chrome, then `ZIPEEK_E2E_BROWSER=firefox` (per e2e harness environment instructions) to verify full Easy/Medium/Hard ten-round flows, deliberately wrong Hanzi choices, map keyboard/pointer selection, review feedback, and completion.
- [ ] Inspect accessible highlighted map controls, mobile viewport hit targets, and Pages `/zipeek/` root-relative asset fetching. Geographic boundary licensing remains a separate publication blocker.

## R331 Province Quest silhouette privacy and verification

- [ ] Verify Rust unit tests, locked WASM compilation and Pages Trunk build for R331 on the approved Rust 1.97.1 toolchain.
- [ ] Verify Guided, Reading and Challenge silhouette questions hide the name and Pinyin until explicit help or answer review, in Chrome and Firefox.
- [ ] Complete the remaining R330 Province Quest browser, accessibility and geography redistribution acceptance items below.

## R330 Province Quest gameplay verification

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises province_quest` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`; correct any compile or test failures.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture` in Chrome and Firefox; test maps and three-choice Hanzi questions for Easy, Medium and Hard, including wrong answers, reveal help, review and completion.
- [ ] Check map visibility, click targets, focus order and screen reader labels at mobile and desktop widths on root and `/zipeek/` Pages builds. Verify no answer labels leak before review.
- [ ] Review the upstream geographic boundary redistribution conditions before deploying the supplied assets publicly.

## R330 AreaCity geography pipeline acceptance

- [ ] On Rust 1.97.1, run `cargo fmt --all`, `cargo check --locked -p zipeek-cli` and `cargo test --locked -p zipeek-geo-assets`; correct any compiler/test issues.
- [ ] Install `7z` and fetch pinned AreaCity release; inspect recorded SHA-256 checksums against independently obtained upstream files, then run `geo audit --provider areacity` and adjudicate synthetic prefectures and missing province/prefecture polygons.
- [ ] Resolve underlying boundary-data redistribution rights (including Amap, Tencent and National Geographical Names Database) before publishing generated assets.
- [ ] Build assets, validate complete 34-province parent coverage and province-capital labels, run `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` and Province Quest E2E in Chrome and Firefox.

## R328 Market Bargain reading support acceptance

- [ ] Compile the updated Market Bargain Yew components and Rust domain tests.
- [ ] Run the Market Bargain browser scenario in both Chrome and Firefox, including Guided, Reading and Listening modes, Reveal Help, answer review, and the Continue transition.
- [ ] Review generated Mandarin price pronunciation and tone sandhi with a fluent Mandarin speaker before marking the learning aid pedagogically accepted.
- [ ] Check layout and keyboard accessibility at mobile and desktop widths; verify the review and help controls have clear semantics.

## R328 Market Bargain acceptance

- [ ] Run Rust unit tests for procedural price generation and state transitions (`cargo test --locked -p zipeek-exercises`).
- [ ] Compile the WASM frontend and run Chrome/Firefox `game_interactions` to verify eight generated Hard prices, progress, replay, difficulty selection, and Settings navigation.
- [ ] Review the constructed Mandarin currency phrasing with a native speaker across yuan/jiao/fen, including zero-jiao amounts.

## R328 Cloud Match module refactor acceptance

- [ ] Run `cargo fmt --all -- --check` and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web` after setup/game module relocation.
- [ ] Run Cloud Match E2E in Chrome and Firefox; verify HSK and Word Set setup, Start, waves, continuous, pause, and replay.
- [ ] Review setup controls on desktop/mobile and confirm no changes to gameplay state or matching behaviour.

## R328 pending acceptance

- [ ] Run Cargo formatting, zipeek-exercises unit tests, zipeek-web WASM check, zipeek-e2e test compilation and full game_interactions/route_matrix browser suites.
- [ ] Test both AI-backed games against a controlled fixture that produces valid and invalid JSON, malformed permutations, duplicate dialogue answers, and provider/model errors.
- [ ] Test switching HSK and Word Set sources during generation; confirm a stale response cannot replace the newly configured game.
- [ ] Review AI-produced Chinese sentences and dialogue choices with a qualified Mandarin speaker; schema validation alone does not establish pedagogical correctness.
- [ ] Verify all nine gallery routes and setup screens on Chrome and Firefox, including mobile sizes and keyboard navigation.
- [ ] Review the limited authored price and decomposition banks before expanding difficulty and HSK content.

## R327 pending acceptance

- [ ] On Windows, run `cargo fmt --all -- --check`, `cargo build --locked`, `cargo test --locked -p zipeek-exercises`, `cargo test --locked -p zipeek-web --target wasm32-unknown-unknown` where supported, and `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`. Default host build excludes the WASM-only web crate; explicit native `-p zipeek-web` is not supported.
- [ ] Run Firefox `firefox_smoke` with visible browser and confirm the measured mobile breakpoint passes. Record the measured inner viewport; a 500px Firefox view does not substitute for a 390px exact-width test.
- [ ] Rerun Chrome `writing_interactions`, `game_interactions` and `responsive_matrix` plus Firefox smoke after this change. Confirm game mistake de-duplication, valid Navigator selected decks, and Write Hanzi Continue protection.
- [ ] Add deterministic installed-reference Hanzi acceptance: correct and reversed strokes, full-character completion, multi-character Practice resume, pointer cancellation, and animation in a real browser. Current source proves the feature is wired, not that every gesture is accepted correctly.
- [ ] Review six Classifier questions, six Navigator steps and four Sentence prompts with a Mandarin speaker and expand the limited authored bank before claiming complete vocabulary-driven gameplay.
- [ ] Triage the 121 existing failing Node contracts from the uploaded source independently of R327's passing source-level checks.

## R325 Firefox cross-browser acceptance

- [ ] Install Firefox locally and execute `cargo check --locked -p zipeek-e2e --tests`, the harness unit tests and `firefox_smoke` with `ZIPEEK_E2E_BROWSER=firefox`; check route startup, history, responsive layout, and driver lifecycle.
- [ ] Compare Firefox screenshots and diagnostics with the existing Chrome E2E evidence; investigate CSS, fonts, SVG, WASM and local browser storage differences rather than silently skipping assertions.
- [ ] Re-run `route_resilience` with default Chrome and confirm the CDP MP4 recorder is unchanged. Longer-term Firefox media/network test controls require an explicit separate design decision.

## R323 vocabulary-driven game acceptance

- [ ] Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-exercises`, `cargo test --locked -p zipeek-web`, and WASM checks on the pinned toolchain; repair any compiler failures before accepting R323.
- [ ] Run `game_interactions` in managed Chrome, including invalid source, saved Word Set, and HSK selection. Verify empty-deck messages, typed session reset, speech, and navigation-grid bounds on desktop and narrow layouts.
- [ ] Expand the linguistically authored challenge bank only after reviewing curriculum coverage. R323 intentionally filters existing authored content and does not fabricate measure-word or sentence answers.

## R322 Tailwind consolidation acceptance

- [ ] Compile pinned Rust/WASM and Trunk Tailwind v4 output. Verify custom media variants, dynamic color-mix utilities, SVG direction/replay animation and no missing classes.
- [ ] Run `writing_interactions` and `game_interactions` under managed Chrome, including the new desktop/mobile computed-style checks, and compare screenshots at narrow/short desktop viewports.
- [ ] Check Crossword board/clue scrolling, highlighted cell states and tile rejection; Writing reference overlays with reduced motion; Matching compact card/footer height; Cloud Match high-contrast focus.
- [ ] Triage remaining 113 Node contract failures separately; do not treat the current source-only result as compiled or browser acceptance.

## R321 acceptance and game architecture

- [ ] Compile `zipeek-web` for wasm32 with the pinned toolchain; confirm the Write Hanzi Practice setup match is exhaustive and the three game reducers compile.
- [ ] Execute domain and Yew Rust unit tests; verify actions after completion, restart, bad selections and stale notices.
- [ ] Run managed Chrome `game_interactions` and inspect the new independent-game-state scenario and restart assertions. Verify root and `/zipeek/` route behaviour.
- [ ] Run the Hanzi `writing_interactions` suite to exclude regressions from the Practice setup correction.

## R320 release-candidate verification

- [ ] Run Cargo fmt, native Rust tests and WASM checks for the R320 game and Hanzi source changes; fix any compiler or Clippy issues before accepting the release.
- [ ] Run managed Chrome `game_interactions`, `writing_interactions` and `route_matrix` suites on Windows, both root and Pages where supported; upload packed evidence, especially successful character-reference replay and 3-game progression.
- [ ] Review the curated Classifier Market, Listening Navigator and Sentence Relay prompts with a Mandarin speaker before asserting pedagogical correctness.
- [ ] Verify Write Hanzi local-session reload, retry and character-index reset when the selected vocabulary source changes.

## R319 games and writing follow-up

- [ ] Compile Rust workspace and wasm web with the new `write-hanzi` exercise, and run `writing_interactions` and `game_interactions` under managed Chrome.
- [ ] Browser-test local Practice selection, mixed-session resume, successful completion of multi-character words, and missing-reference handling for `write-hanzi`.
- [ ] Review and approve curated classifier, listening-direction and sentence prompts with a Mandarin speaker. Verify browser speech availability and reduced-motion/keyboard interactions.
- [ ] Decide whether the three new games should load personal Word Sets or dictionary vocabulary; existing playable starter rounds intentionally use curated prompts.
- [ ] Replace the remaining Crossword and Hanzi feature CSS only after validating equivalent responsive layout, SVG animation and reduced-motion behaviour with Tailwind utilities.

## R319 guided Hanzi writing acceptance

- [ ] Run the native Rust importer to reproduce the packaged 9,574-character `characters.json`, verify manifest hashes and root/`/zipeek/` HTTP paths in Chrome, and measure compressed transfer size. The upstream Hanzi Writer Data and original licence are already included in this candidate.
- [ ] Compile and run `zipeek-hanzi-writing` unit tests, WASM check, and `zipeek-e2e --test writing_interactions` in Chrome. Exercise correct/reversed/wrong-order/truncated traces with real pointer input and review the canonical path alignment on Windows and touch devices.
- [ ] Calibrate match thresholds against representative mouse, stylus, and touch traces; review dot and curved-stroke false accepts/rejects. Request approval for additional R319 scope to meet the 20 meaningful Rust-file release threshold.

## R308 acceptance and remaining E2E coverage

- [ ] Run Cargo formatting/check/tests and managed Chrome acceptance against the R308 candidate. Prioritize new Sentence Transformation, Audio Dictation and Picture Vocabulary mode assertions; resolve Rust compiler failures before calling R308 accepted.
- [ ] Complete Image Vocabulary UI image approval/answer/scoring E2E, successful Speaking recording/stop/discard with deterministic media fixture, and opt-in real `zipeek-ai-server` process/browser tests. R318 now checks image server generation through browser CORS and recorder acquisition attempts; neither is full exercise acceptance.
- [ ] Triage 113 inherited Node source-contract failures against actual modules and remove outdated assertions only after checking intended behavior.

- [ ] R300 Cloud Match browser acceptance: rerun `game_interactions` after replacing the 22 px continuity threshold with elapsed-time- and velocity-bounded travel. Verify the remaining cloud keeps its DOM identity, has no backward/overspeed jumps, match-fade completes, and pause/resume does not advance motion. Windows run 2026-11-10 13:11 passed six scenarios but failed continuity at 27.796 px versus the stale 22 px cap.
- [ ] R293 Cloud Match acceptance: confirm both sides contain mixed Hanzi/meaning labels in Waves and Continuous modes; ensure audio follows Hanzi regardless of side, optional highlight reveals only the selected pair's partner, and existing cloud animation progress survives selection/matching. Confirm the adjusted wave deadline permits visible exit on common viewport sizes and check narrow-screen layout.
- [ ] R291 Character Builder acceptance: compile WASM and run `practice_answer_options` in Chrome; confirm automatic retry has one transient feedback message, leaves the panel height stable, and cancels pending timers on navigation. Verify the Continue fade and next-round entrance with reduced motion enabled and disabled.
- [ ] Plan a unified local-session contract for Endless Character Builder, Type Pinyin and other standalone hosts. The current UUID host requires a fixed resumable round count, so standalone query routes remain an intentional fallback. Avoid claiming a complete migration until the endless lifecycle is implemented and tested.
- [ ] Review dictionary gloss normalization for practice prompts. CC-CEDICT may encode synonymous English alternatives within a single sense (e.g. `not only; not just`); do not create fabricated lexical senses or change the source dictionary merely to shorten a UI label.
- [ ] R289 Practice acceptance: compile WASM; run `cargo test --locked -p zipeek-web` and `cargo test --release --locked -p zipeek-e2e --test practice_answer_options -- --nocapture`. Verify Type Pinyin hint toggling and refresh, Character Builder automatic feedback and retry, profile round-trip, and same-tab Practice setup refresh immediately after discarding the saved session. Inspect keyboard focus and alignment in Chrome.
- [ ] R289 follow-up: audit configuration support for all 16 registered exercise types. Only Character Builder and Type Pinyin currently use the shared `PracticeExerciseOptions` struct; other exercise configuration resides in draft, per-session fields or exercise-specific state and does not yet have an equivalent comprehensive persisted-options contract.
- [ ] R289 Cloud Match acceptance: compile WASM and run `game_interactions` in Chrome. Confirm clouds appear at screen edges on every wave (not center), speeds vary within difficulty bands, matched clouds do not teleport survivors, labels stay within silhouettes, and reduced-motion presentation remains playable. Review any failures using recorded video and bounding-box evidence.
- [ ] R286 games/Practice browser acceptance: verify Cloud Match one-way travel at desktop/mobile sizes, stable positions after a matched pair, unlimited lives, click-to-pronounce, red setup-error isolation, and replay/setup; verify Character Builder drag/click controls, feedback, responsive layout, and Pinyin legibility. Rust/WASM compilation required first.
# Open work

Keep only actionable unfinished work here. Shipped source changes and historical checkpoint summaries belong in [CURRENT](../project-management/current.md), [completed](completed/), and [release history](RELEASE_HISTORY.md). A checked item here should be removed or moved to completed history rather than repeated in later checkpoint sections.

## R326 pending acceptance

- [ ] Compile R326 Rust E2E tests and run fixture-client health/reset unit regressions on Windows.
- [ ] Rerun Firefox smoke and Chrome route/writing tests, including actual Firefox content viewport sizing and fixture reset isolation.
- [ ] Compare Firefox screenshot and scenario timeline artifacts against Chrome; do not treat source-contract passes as browser verification.

## R324 pending acceptance

- [ ] Compile the typed E2E navigation API with pinned Rust; verify `cargo fmt`, `cargo check`, library route tests, and application root/Pages hash routing.
- [ ] Run Chrome route resilience, route matrix, responsive, game, Practice, and Browse tests after R324; inspect artifact evidence for routing redirects, query encoding and SPA back/forward transitions.
- [ ] Audit remaining deliberate raw navigation and fixed sleeps after browser acceptance; do not replace malformed-URL coverage with a validated route builder.

## Approved 2026-11-10 structure and E2E work

- [ ] R305 acceptance: compile and run pinned Rust 1.97.1 checks, `cargo test --locked -p zipeek-web`, Chrome Practice lifecycle E2E (profile CRUD, authored/listening step switches, quiz formats, back navigation), and verify source-contract failures are unchanged. The archive is only an unverified release candidate.

- [ ] R305 acceptance: compile and run pinned Rust 1.97.1 checks, `cargo test --locked -p zipeek-web`, Chrome Practice lifecycle E2E (profile CRUD, authored/listening step switches, quiz formats, back navigation), and verify source-contract failures are unchanged. The archive is only an unverified release candidate.

- [ ] Run `cargo check --locked -p zipeek-web`, `cargo check --locked -p zipeek-e2e --tests`, wasm check, and focused managed Chrome `game_interactions`/`route_resilience`/`matching` against the modified source. Node script-probe tests alone do not establish Rust/browser acceptance.
- [ ] Compile and test the initial `practice_session_store/validation.rs` boundary and generation-step guard consolidation; verify stored-session serialization/revision/recovery compatibility in Rust and Chrome. Before further persistence extraction, add explicit event-dispatch and storage concurrency regression tests.
- [ ] Evaluate a library-owned shared E2E-suites API against each integration test binary's feature imports and harness lifecycle before moving test files. Current `tests/suites` multi-root inclusion remains a coupling concern.
- [ ] Continue separating Cloud Match runtime assertions into named probes and focused scenarios while preserving timing and DOM identity expectations.

## R284 native services and integration

- [ ] Compile and run the new `zipeek-static-server` on Windows; verify root/Pages asset paths, WASM MIME types and optional root E2E serving through `ZIPEEK_E2E_STATIC_SERVER_BIN`.
- [ ] Rerun Course E2E after scoping the missing-lesson alert selector; validate the recovery navigation in Chrome.
- [ ] Implement `zipeek-ai-server` runtime ownership and the missing `model_runtime` layer for the copied llama.cpp backend, with model-load errors, cancellation and bounded concurrency. Do not activate copied inference modules until they compile.
- [ ] Expose the required OpenAI-compatible chat completion and streaming endpoints, prove `zipeek-ai-client` interoperation and keep the deterministic E2E AI fixture as the default.
- [ ] Reconcile the relocated `zipeek-ai-protocol` code with legacy source-contract tests, retaining actual product behavior assertions.
- [ ] Audit `zipeek-hanzi-writing` newly added dependencies and `zipeek-cli` typed error boundaries, respecting dependency approval rules.

## Documentation and dependency reconciliation

- [ ] Confirm that the installed native database contains `character_readings`; generator SQL and installed manifest include it. Validate the physical binary directly before treating its query surface as verified.
- [ ] Review the deferred UX-DIGEST entries as separate pending decisions; do not silently convert historic drafts into active code contracts.

## Release acceptance

- [ ] Compile current R284 candidate web/WASM and all Rust E2E targets on the Windows toolchain.
- [ ] Run deterministic `translation_fixture` against the UUID-backed Practice lifecycle and inspect the browser evidence for all scenarios.
- [ ] Run the optional live LM Studio test against the same Settings-to-Practice workflow, verifying request, canonical tile assembly, AI evaluation, progress and stored outcome.
- [ ] Run `cargo make e2e-full` for root and `/zipeek/` scenarios; triage any failures against the source in the tested archive.
- [ ] Verify the completion notice, automatically cleared temporary Practice record, and reload/resume semantics in Chrome.
- [ ] Resolve the currently failing Node source-contract tests by reviewing each against present source behavior; do not describe them as newly passing.

## Pedagogical evidence and storage

- [ ] Browser-verify and harden the R283 opt-in personal DuckDB Practice-outcome writer, disabled/saved/failed feedback and retry, plus cross-tab and quota behavior. Add full learning-evidence provenance and derived attempt/character/word/exercise/HSK/category/grammar-point observations. The source checkpoint is not yet release-accepted. See [learning evidence](../learning-practice/practice/learning-evidence.md).
- [ ] Implement the approved proficiency eligibility rules: seven **distinct active dates**, at least ten **qualifying scored encounters** across at least **three different sessions**, a per-session anti-inflation cap, and independently assessed recognition/listening/speaking/sentence-use evidence. Finalize the cap, per-skill acceptance thresholds, recovery/decay and microphone grading; do not award all four skills from Matching alone.
- [ ] Add deterministic injected time authority shared by evidence writes and `/stats`; test seven-day E2E time travel with actual personal DuckDB persistence, and reject timestamp manipulation or duplicate encounters. See [planned E2E](../testing-quality/e2e/learning-evidence-e2e.md).
- [ ] Browser-verify the implemented ten-distinct-Word-ID guard in Practice selection and launch, and confirm playable Hanzi vocabulary identity against dictionary data. Shared snapshot and selector guards have source and Rust unit tests but are not WASM/browser verified.
- [ ] Build a source-backed `/stats` route showing day/session/encounter counts and per-modality evidence without fabricated proficiency.
- [ ] Implement and test the [Courses mosaic specification](../learning-practice/courses/courses-mosaic.md) as a separate future milestone: contiguous data-driven 100+50 geometry, authored grammar curriculum, point assessments, word-bank Sentence Transformation and verified permanent completion reveal. Do not infer progress from marked-complete stubs or fabricated vocabulary percentages.
- [ ] Verify disabled-recording, successful durable save and failed-write retry feedback in Chrome; confirm no premature local cleanup.
- [ ] Verify personal-data export/import, OPFS storage authority, quota, cross-tab, reload and corruption handling before promoting the personal DB probe to an authoritative learning store.

## Practice and AI quality

- [ ] Compile and browser-test the new Word-bank Sentence Transformation Practice exercise: setup selection, token input/undo/reset, alternate valid answers, distractors, feedback, refresh/resume and completion. Conduct Chinese linguistic review of all authored prompts before promotion to scored grammar assessment.

- [ ] Audit remaining exercise combinations for lifecycle parity rather than assuming all legacy routes use UUID persistence.
- [ ] Evaluate Chinese segmentation and ambiguous pinyin with real sentence fixtures; canonical tile ordering alone does not validate language correctness.
- [ ] Test invalid bilingual model output, absent model, provider errors, cancellations, slow evaluation and retry in both deterministic and optional live suites.
- [ ] Confirm completion and resumption of mixed and unlimited Practice exercises under browser navigation and refresh.

## Product and accessibility acceptance

- [ ] Complete root/Pages responsive and keyboard/assistive-technology audits beyond the current DOM-level route matrices.
- [ ] Check generated dictionary/manifest provenance and storage diagnostics on the installed artifact before any database regeneration or publication.
- [ ] Validate voice/microphone behavior with a supported device; DOM automation does not prove audio output or recording fidelity.

- [ ] R284 Matching Chrome acceptance: rerun `cargo test --release --locked -p zipeek-e2e --test matching -- --nocapture` on Windows and upload `cargo make pack-e2e` artifacts. Verify 1280×720 no-scroll Matching layout and all three recovery scenarios; inspect the actual overflow screenshot before further CSS adjustment.

### R285 follow-ups

- Compile and run `zipeek-cli e2e-clean` and `pack-e2e --all` on Windows; check multi-suite ZIP contents and restore no unwanted evidence deletion.
- Rerun `cargo make e2e-full` against the supplied source and verify the latest Matching viewport and Practice recovery behavior.
- Design an invocation manifest that groups all suite outputs and makes stale-run discrimination independent of manual cleanup, including interrupted and parallel test execution.
- Continue approved AI-server, CLI, Practice-history, and contract-failure work before claiming a major R285 release. Enforce the 20 substantive Rust-file minimum.

- [ ] Compile and review R285 `zipeek-ai-server` transport. Wire the native runtime/executable to `zipeek-ai-core::GenerationBackend` only after implementing the missing model runtime and testing bounded concurrency, browser CORS policy and streaming cancellation. Confirm HTTP tests and `zipeek-ai-client` interoperability on Windows.

## R286 games acceptance

- [ ] Compile the Cloud Match outcome and review changes in `zipeek-exercises` and `zipeek-web`; run its Rust domain tests and browser E2E for wrong pairs, skipped waves, finished versus failed outcomes and both replay actions.
- [ ] Browser-check game result focus/keyboard access, narrow viewport overflow, high-contrast/reduced-motion and full-length dictionary labels; confirm no Practice proficiency evidence is inferred from recreational game scores.
- [ ] Identify and approve additional substantive Rust game improvements before an accepted R286 release; the Cloud Match slice alone does not meet the 20 distinct Rust-file rule.

- [ ] R286 E2E compact archive acceptance: run `cargo test --locked -p zipeek-cli`, verify `cargo make pack-e2e` omits build output and videos while preserving failure evidence, and verify explicit `--include-videos` includes recordings.
- [ ] R286 Cloud Match review acceptance: compile and browser-test distinct incorrect/expired review reasons, pause/finish keyboard focus, and narrow viewport results.

### R286 Practice durability acceptance

- [ ] Compile and browser-test `SessionReminder` after a completed UUID Practice outcome cannot be committed to personal DuckDB: navigate away, recover through the reminder, retry commit, confirm the temporary record is cleared only after acknowledgment.
- [ ] Exercise corrupt local Practice data and disabled/unavailable localStorage: verify the reminder reports the issue without deleting records, then retry discovery.
- [ ] Test concurrent tabs, quota failures, all-steps-completed session refresh, and consent toggling. Verify saved outcomes remain idempotent and that unscored activities never award unsupported proficiency.

### R286 Practice lifecycle and E2E acceptance (open)

- Verify the single-active UUID Practice gate against Windows WASM and browser E2E; test completed-session history retry and invalid-storage recovery.
- Complete the 16-exercise E2E matrix in `docs/testing-quality/e2e/practice-exercise-matrix.md`, prioritizing Audio Dictation, Sentence Transformation, Word Mosaics and Fill Blank.
- Decide and implement a cross-tab atomic session-creation mechanism if strict simultaneous-tab exclusion is required; the current localStorage inventory check is not atomic across tabs.
- Continue per-attempt provenance, learning-history recording, schema and consent verification before claiming Practice product compliance.

### R286 saved Practice profiles acceptance

- [ ] Compile the standalone Character Builder and Type Pinyin launch-query option handling; verify Retry and Audio choices both before and after browser refresh, while checking UUID-session options remain authoritative.
- [ ] Run the new Practice profile save/load/delete E2E journey and verify name validation, same-name replacement, storage errors, deleted personal Word Sets, browser refresh, restored exercise options, and multi-tab behavior.
- [ ] Decide whether Quick Practice should continue preselecting Matching and Multiple Choice or start with zero selected exercise types; current source retains the existing default pending a UX decision.

### R286 Practice setup refactor acceptance

- [ ] Compile the extracted Practice setup launch, validation, profile hook and recovery component on Windows with `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`.
- [ ] Run Practice profile save/load/remove, existing-session recovery, Character Builder options, Quick Practice launch and invalid-vocabulary Chrome journeys after the refactor.
- [ ] Run `cargo test --locked -p zipeek-cli archive::paths` and confirm `cargo make pack` excludes both `.duckdb` and `.duckdb.gz` while preserving source regeneration inputs.
- [ ] Treat standalone launch error reporting and session inventory refresh as ongoing runtime acceptance; source tests are not browser verification.

### R286 Tone Detective and Practice selection acceptance

- [ ] Compile `zipeek-exercises` Tone Detective and `zipeek-web` route on Windows; inspect both dark and light themes, 1280x720 and mobile layouts.
- [ ] Run `game_interactions` for all 12 tone answers, missed-tone review and replay; verify Mandarin voice availability and intelligibility on the target browser.
- [ ] Run `practice_answer_options` including the new first-click selection scenario and verify path switching and empty selection visually.
- [ ] Review the authored Mandarin tone prompt inventory with a Chinese-language reviewer before treating recognition results as pedagogical evidence.

### R287 Cloud Match options and visual acceptance

- [ ] Compile `zipeek-exercises`, `zipeek-web` WASM, and the E2E tests on Windows; run the new game options tests and browser configuration scenario.
- [ ] Visually review all eight SVG silhouette variants, the layered twilight background, contrast, text truncation, motion, and reduced-motion fallback at desktop and mobile resolutions.
- [ ] Validate 10/20/30/endless pair modes and 1/2/3/endless lives, including low-vocabulary error states, partial-wave completion, game exit, replay, and eventual repeated endless waves.
- [ ] Record before/after unmatched-cloud positions through other pair matches to confirm the remaining-cloud animation never restarts or teleports during a wave.

### R292 Cloud Match follow-up

- [ ] Compile R292 Rust/WASM source and run the new Continuous engine unit tests.
- [ ] Execute Chrome E2E for paused-game focus and progress, then Continuous pair replenishment without survivor replacement or teleportation.
- [ ] Inspect the five scenic backgrounds and ensure asset loading works at both `/` and `/zipeek/` deployment bases.
- [ ] Verify Continuous games with 10, 20, 30 and Endless targets, each with 1, 2, 3 and Endless lives, at desktop and mobile sizes.
- [ ] Measure image contrast and text-fit behavior at 100%, 125% and 150% zoom, including long English definitions and multi-character Hanzi.

## R294 Cloud Match acceptance

- [ ] Compile the refactored Yew setup reducer and Tailwind presentation on Windows with the existing locked toolchain. Check both root and Pages base-path image loading.
- [ ] Run managed Chrome Cloud Match scenarios for mixed-label sides, cloud DOM identity/motion after matches, scene changes, paused focus and reduced-motion navigation.
- [ ] Review legacy source-contract tests that depend on the removed feature CSS; replace each with a behavior-oriented assertion where still relevant and preserve true failures.

## R296 Cloud Match browser acceptance

- [ ] Compile and run Cloud Match after the R296 SVG feedback and HUD changes; verify click transitions, exactly two red wrong-pair tiles, automatic feedback expiry, no animation-position reset, keyboard and reduced-motion behavior.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test game_interactions -- --nocapture` and inspect the `cloud-match-temporary-wrong-pair-feedback` recording for 850 ms feedback and matching continuity.

## R298 Cloud Match and test consolidation acceptance

- [ ] Compile the new Cloud Match exit overlay with `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and run Cloud Match domain and E2E suites on Windows.
- [ ] Verify the mint 350 ms fade for correct matches in both Waves and Continuous modes. Confirm the next pair enters normally, survivor DOM nodes retain identity, and reduced-motion playback has no unwanted transition.
- [ ] Visually check twelve silhouettes and adaptive cloud dimensions at desktop/mobile resolutions and 100%, 125%, and 150% zoom. Check long definitions, Hanzi, keyboard focus, and background contrast.
- [ ] Inspect and triage the pre-existing Node source-contract failures after the release-specific consolidation; prefer behavioral tests over checks tied to moved CSS or source layout.

## R299 Cloud Match verification and technical debt (2026-11-10)

- [ ] Compile the R298-to-R299 Cloud Match correction on Windows with `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`; local review does not substitute for compilation.
- [ ] Run `cargo test --release --locked -p zipeek-e2e --test game_interactions -- --nocapture` after successful WASM compilation. Confirm both matched-cloud ghost fades and continued motion of unmatched clouds; inspect actual recordings.
- [ ] Audit remaining complex inline `classes!` expressions in Cloud Match and other Yew pages; use named local class variables for multiline or conditional class sets. Avoid brittle space-separated string literals within `classes!`.
- [ ] Repair obsolete Node source contracts without deleting meaningful behavioral assertions. Track the exact failing cases and only mark them passing after execution.
- [ ] Resolve outstanding browser acceptance for cloud text fit, ghost cleanup, reduced motion, dynamic scenery, HUD interactions, and Continuous-mode replacement.
- [ ] Verify and document Practice UUID lifecycle coverage for every exercise, including Endless, cross-tab discard handling, and profile option persistence.
- [ ] Finish the E2E scenario matrix and durable learning-evidence verification; older source-only assertions are not release acceptance.
- [ ] Audit the R298 script consolidation against the original registered test names and run the entire Node suite; verify no tests were inadvertently dropped.

## R302 silent-speech forward integration (2026-11-10)

- [ ] Validate Rust/WASM compilation and managed E2E on Windows against the user-supplied 13:41 source baseline; R301 archive was based on an older source and must not be applied as a replacement.
- [ ] Verify ordinary manual gameplay retains audible Mandarin playback and E2E mode is silent across Cloud Match, Tone Detective, matching and listening exercises.
- [ ] Confirm E2E speech event recording and tab-scoped session settings across redirects/reloads, with production builds remaining immune to storage-only activation.
- [ ] Triage remaining pre-existing Node route catalog contract (`66 !== 64`) and full suite independently of speech integration.

## R303 WIP follow-up (2026-11-10)

- [ ] Run `cargo fmt --all`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, and WASM build on the exact R303 source; fix any discovered compiler/lint errors.
- [ ] Run affected Matching/MCQ speech tests under managed Chrome at `/` and `/zipeek/`, then the full E2E suite.
- [ ] Triage and repair the 113 baseline Node source-contract failures by checking each assertion against current behavior.
- [ ] Audit remaining inline WebDriver scripts and move only substantial, reused or opaque blocks to named JS assertions.
- [ ] Audit remaining game functionality and UX against approved requirements; verify behavior in the browser before marking completed.


### R306 E2E release-candidate acceptance

- [ ] On the pinned Rust toolchain, run `cargo fmt --all -- --check`, `cargo check --locked -p zipeek-e2e --tests` and `cargo test --locked -p zipeek-e2e --tests`; repair any compiler or formatting issues caused by feature grouping or named scripts.
- [ ] Run the managed Chrome E2E suite, with emphasis on Cloud Match parameterized selectors, Matching responsive layout, translation AI fixture polling, and shared smoke/page test suite imports. Examine artifacts before claiming browser acceptance.
- [ ] Separately triage the 113 legacy Node source-contract failures against current source behavior instead of changing tests merely to force a green total.


- [ ] Validate R377 Courses/Docs/Practice UI in Chrome at desktop and mobile sizes, keyboard navigation, light/dark themes, and production `/zipeek/` base path; check mosaic artwork fit and Docs favicon asset path.

## R380 WIP acceptance - Courses layouts

- [ ] Run `cargo fmt --all -- --check`, `cargo check --target wasm32-unknown-unknown -p zipeek-web --locked`, and `trunk build --public-url /zipeek/` after preparing assets. Resolve any Rust/Yew errors.
- [ ] In Chrome, check Grammar and Pinyin mosaics at desktop widths and 720px-height screens. Confirm the main page does not scroll vertically, the selected details remain visible, and no clipping or horizontal overflow occurs.
- [ ] Check compact/tablet/mobile layouts where ordinary vertical scrolling should remain available; verify Pinyin SVG, all three click/keyboard targets, focus, and screen-reader names.
- [ ] Complete Pinyin lessons, navigate back, and verify the selected-region completion status uses existing local storage; confirm no fabricated mastery data.
- [ ] Compare both artwork assets against the approved visual reference and refine the remaining grammar geometry/artwork mismatch only as a separately scoped task.

## R420 verification follow-ups

- Run wasm32 compilation and Chrome/Edge smoke tests for Speech Settings, including TTS failure reporting and user-initiated microphone recognition.
- Verify browser and Tauri Learning Data layout and the analysis-worker technical disclosure; check the existing recording, backup and deletion workflows.
- Implement native speech backends before displaying selectable native TTS or STT providers.

## R421 browser presence follow-up

- [ ] Verify Dragon header visibility, animation and reduced-motion behavior in Chrome after toggling Learning Data, navigation and reload.
- [ ] Run `cargo make e2e-learning-header-tabs` with two browser tabs and verify notice clears after graceful close and tab crash.
- [ ] Add an exclusive browser learner-data writer lock before allowing parallel tabs to write OPFS; BroadcastChannel presence detection is advisory, not a concurrency guarantee.

- [ ] Verify R432 WIP fixes on Windows and WASM: compile Docs, E2E and CLI; stage genuine Tauri sidecars before native app check; run tests and root/Pages Docs acceptance.

## R436 consolidated Practice recording consent acceptance

- [ ] Verify default-off first launch, onboarding opt-in, reload, Settings toggle, dragon indicator sync, and Practice outcome suppression after opt-out with Chrome at `/` and `/zipeek/`.
- [ ] Run locked WASM and E2E compilation on Windows and validate preferences JSON persistence, including old consent-key presence not re-enabling recording.

## R440 service monitoring acceptance

- [ ] Compile `zipeek-web` for wasm32 and verify all supported Rust tests against the R440 source.
- [ ] In Chrome, check immediate and 30-second AI/image status transitions, Settings saves, manual refresh, unreachable/CORS failures and recovery at `/` and `/zipeek/`.
- [ ] In Tauri, verify native ownership-sensitive Start/Stop controls and that externally managed services report availability without claiming ownership.
- [ ] Confirm Grammar AI generation remains disabled while the endpoint is unavailable and authored content remains accessible.
