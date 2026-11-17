# Archived source snapshot: learning-practice/hanzi/write-hanzi-activity.md

This source snapshot preserves historical descriptions, not current requirements. Use the maintained document at its original location for approved behavior.

# Standalone Write Hanzi activity - approved direction (R111 documentation)

User-approved design for **`#/write-hanzi`**. This is a learner-facing route, separate from the current unscored `#/practice/hanzi-writing` prototype. R113 implements the standalone vocabulary picker and reuses the R107 Canvas engine. Stroke-order data and grading remain future work.

## Entry and word selection

Two source modes: **My Word Sets** (choose a saved set, then a word) and **Search English** (query real dictionary English meanings, show Chinese/reading/definition/HSK metadata, explicitly choose a result). When a vocabulary item has multiple characters, show a per-character sequence; users choose a character to write without losing the selected word or its dictionary identity. Preserve user choice and display a useful empty-set state. No hardcoded-only character list for the final activity.

## Workspace and controls

Use the existing `zipeek-hanzi-writing` pure-Rust input and geometry model and the R107 Yew Canvas adapter. Display the currently selected Hanzi, pinyin and meaning, a centered writing canvas, grid/guide visibility, Undo, Clear, and next/previous character. Support pointer, mouse and touch, resize and HiDPI. An optional hide-reference mode provides recall practice. Source-backed stroke-order playback and reference comparisons are deferred until a dataset with acceptable license, mapping, and tests is chosen. **Do not award correctness or display 'stroke order correct' based on geometric diagnostics or the mockup alone.**

## Architecture and acceptance

Vocabulary comes through existing read-only browser database and personal Word Set providers. The writing crate remains free of DOM and DuckDB; adapters translate selected dictionary words and pointer coordinates. No dictionary schema change is needed for this initial route. Browser acceptance needs English search, Word Set selection, character switching, touch/pointer cancellation, undo/clear and resizes. A standalone activity does not publish scored events to the Practice planner until grading is validated.

## Boundary

AI classification stays entirely offline in db-gen and CLI. **No AI Data Processing dashboard or public AI route will be implemented.**

## R113 - standalone activity implementation (2026-11-05)

- Added `#/write-hanzi` to primary navigation and a `DatabaseBoundary` + `WordSetsProvider` route so both saved sets and English dictionary search use the installed database.
- The left pane supports saved-set selection and an explicit English-meaning search (up to 20 results); choosing a result preserves the full dictionary word identity, pinyin and meaning.
- The right pane presents every Hanzi character in the selected expression, with Previous/Next navigation, and reuses the existing pointer-aware drawing component. Canvas state resets when the selected word/character changes.
- `zipeek-hanzi-writing::hanzi_characters` is a browser-free utility that filters non-Hanzi characters and preserves repeated Hanzi and source order; three native Rust unit tests were added.
- Scoring, stroke counts against references, grading and stroke-order playback are **not implemented**. HSK membership badges are displayed when present; end-to-end keyboard focus, mobile layout and browser pointer acceptance still need verification/refinement.
- No schema or dependency changes. Rust/WASM compilation and browser acceptance remain mandatory before declaring the feature fully verified.

## R118 - Direct Word Discovery handoff

Word of the Day and Random Word expose `Write Hanzi` links to `#/write-hanzi?word_id=<positive-id>`. The standalone page validates the integer as a `WordId`, queries the currently loaded dictionary by ID, and opens the selected word at its first writable Chinese character. Unknown IDs and entries without supported Hanzi show an explicit message. The existing Word Set and English-search entry points are unchanged. IDs are scoped to the installed dictionary build and not guaranteed stable across regenerated dictionaries; durable learner data needs a version-aware remapping strategy.

## R124 - Approved Write Hanzi UX refinement (2026-11-05)

User feedback from live screenshots: the two stacked word/character selector widgets were redundant, search required an unnecessary button, search for `water` returned obscure words before `水`, visible focus rings were excessive, and the workspace required unnecessary vertical scrolling. The unscored warning should be behind an accessible info icon, not permanently displayed.

- The word header now renders **each Hanzi as an interactive character button**, retaining pinyin, audio, definition and character count. This **replaces** the separate previous/next controls and separate character row. Selecting a character resets only its writing canvas through the existing keyed component.
- English search is **debounced 300 ms** after typing, with a busy indicator, no Search button, stale-response cancellation and no forced deselection while typing.
- The dedicated parameterized dictionary query ranks **exact gloss**, then gloss prefix, then substring, with HSK membership and shorter Hanzi lengths as tie-breakers. `water` should prioritize `水` when its source senses contain the exact gloss. This is query relevance, not a modification to dictionary data. Actual results depend on the generated dictionary and require browser acceptance.
- The header's accessible info tooltip explains the unscored status. A subtle focus-visible ring remains for keyboard accessibility; the selected result border is quieter. The vocabulary panel has a bounded internal scroll, the canvas is width-bounded, and optional stroke references are collapsed by default.
- No schema, dependencies, scoring or personal data changes. Cargo/WASM compilation and responsive browser checks still required; source tests cannot substitute for these.

## R125 - viewport workspace and top-bar title (2026-11-05)

The desktop Write Hanzi route is now a **viewport workspace**, not a long document: the title and accessible unscored-state info tooltip move to the existing application top bar. The page-level title/subtitle are removed, the application footer is hidden on this route, and the shell content occupies the remaining viewport height. Vocabulary results scroll inside their own panel, while the writing workspace independently scrolls only when necessary. The Canvas is bounded by available viewport height, and definitions in picker rows are limited to two lines. Below 1024px width or 600px height, retain ordinary page scrolling to protect touch accessibility and controls.

English dictionary search preserves exact/prefix/substring gloss priority and now uses existing nullable `words.frequency` (lower = more frequent) before HSK/length tie-breaks. Browse searches also use frequency when a query is supplied; English-specific Browse places exact/prefix meanings first. No schema migration or new data source is necessary. Search result ranking and layout require browser acceptance with the real installed DuckDB: use `water`, `wa`, long definitions, Word Sets, resize, zoom, keyboard and touch. The new title/help controls must remain discoverable and usable by keyboard.

Navigation icon decision: Courses `LUCIDE_GRADUATION_CAP`, Games `LUCIDE_GAMEPAD`, Practice `LUCIDE_DUMBBELL`, preventing duplicated graduation-cap use. These symbols must be checked against pinned yew_icons during Windows compilation.

## R134 - Compact paginated vocabulary finder

The finder now displays at most five records per visible page, with Previous/Next controls and a page counter. Search remains debounced, frequency-aware, and independent of writing selection. The current query fetches up to 30 ranked English entries and pages them in memory; deeper dictionary-backed pagination and total matches are pending. A changed query, vocabulary mode or Word Set resets the page to one. The title and writing-info tooltip move from the right edge to a shared **centered** top-bar title. No writing-scoring changes.

## Advanced Practice exercise integration (R319 WIP)

The `write-hanzi` exercise replaces the retired self-assessed Handwriting Recall activity. Both Quick and Advanced Practice can select it, the local UUID session host schedules it, and its browser component loads a word from the session's selected vocabulary source. The learner completes each supported Hanzi character via the shared canonical writing reducer before proceeding. Missing geometry prevents validated completion and exposes the existing reference retry. This exercise reports `Completed` but deliberately does not publish scored proficiency evidence pending handwriting calibration and real-device acceptance. Generated geometry remains excluded from pack/unpack.

## R505 WIP - Practice writing guidance and focused workspace (2026-11-16)

The Write Hanzi exercise's Configure panel now owns two persisted session options: guidance (`none`, `start_cue`, `full_guide`) and stroke-order animation (`true` or `false`). Start cue previews the first 25% of the next reference median (35% after one rejection, 50% after two or more); Full guide shows the reference stroke outline and the same short directional preview; None hides all uncompleted stroke hints. Completed strokes remain visible in every mode. The preview follows arc length across reference vertices, resets on successful strokes, Undo, Reset and character changes, and disappears while drawing or replaying. The animated median/direction line through the highlighted stroke remains removed from the static guidance overlay; the separate optional Watch playback remains available when enabled.

The Practice exercise header displays `Write <selected word>` rather than a separate character index. The exercise view omits the visible progress bar, stroke count, mistake count and routine trace instruction, while retaining screen-reader status updates, reference-load errors, and the Complete/Continue control. Undo removes the most recently accepted stroke; Reset restarts the current character and clears mistakes; Watch is conditional on the saved animation setting. The standalone writing routes retain their previous guide toggle and progress UI. Stroke validation and the unscored session evidence contract are unchanged.

The current configuration is stored in the existing Practice profile and session data structures as a new `WriteHanzi` exercise configuration. Existing saved profiles and pending sessions authored before this schema change have not been migration-tested; no compatibility reader was added. Verify settings round-trip, route isolation, touch input, pointer reset, both animation states and standalone behavior in Chrome before acceptance.

## R510 WIP: Flexible stroke recognition and optional word prompt

The writing matcher now uses bounded dynamic time warping over resampled points, soft centroid alignment, robust symmetric point-to-segment coverage and explicit absolute endpoints, stroke direction and length limits. The active expected stroke is compared with later reference strokes to detect out-of-order input without penalizing identical overlapping strokes. Rejections present distinct start, end, direction, order, length, shape and geometry messages. These thresholds remain provisional until real handwriting calibration, and neither scores nor TTS constitute certified learning evidence.

Practice Configure offers three independent Write Hanzi word-prompt options: show dictionary Pinyin, show dictionary meaning and show a manual Mandarin Listen button. All are enabled by default; turning them off hides only the corresponding aid. The word's Hanzi remains visible in its title. A missing definition is labelled unavailable when meanings are enabled, and an unavailable speech engine reports an explicit notice. Audio uses existing local browser speech synthesis; no external service receives the learner's handwriting. This is a current Practice settings format change with no legacy migration under the project's development-phase policy. The standalone writing workspace retains its own presentation.

## R511 WIP: Stroke-order playback mask alignment

Watch replay reveals the authored stroke outlines through median-shaped SVG masks. Authored outlines and replay masks now share raw Hanzi reference coordinates before applying the existing outline transform. Static guidance previews continue using top-left screen coordinates. The mask bounds include the complete source reference Y range (`-124` to `900`), including lower extremities. This corrects the source-level vertical inversion that made stroke playback appear fragmented; real browser acceptance, repeated playback, reduced-motion timing and varied-character testing are outstanding.

## R512 WIP: assisted progression after repeated mistakes (2026-11-16)

Practice Configure > Write Hanzi offers **Help after mistakes** with `after_three` (default), `after_five` and `off`. Each rejected stroke counts once toward the current stroke's threshold, then an aligned, 750 ms canonical-outline reveal demonstrates only that expected stroke. A cancellable 850 ms completion timer records the stroke as assisted, not correctly written, and proceeds to the following stroke. The progress model retains accepted scores separately from assisted slots and cumulative mistake count. A normally accepted stroke resets the per-stroke failure count, as do Undo and Reset. Practice completion shows handwritten and assisted counts before Continue. Standalone writing uses the default three-mistake assistance policy without a new standalone configuration surface. The Practice option is stored in the current strict session/profile representation; old saved configurations may be incompatible. No personal learning proficiency credit is emitted and browser acceptance remains pending.

## R513 WIP - Completion layout and Practice progress preview

The embedded writing board always reserves the completion-summary and Continue area so finishing a character does not resize or vertically recenter the canvas. On the final character of the selected word, a temporary ready signal previews the completed exercise on the Practice creature before Continue. Continue remains the only action that commits the completed exercise to the saved session and starts the next activity. Undo returns the preview to its previous state. The preview is bound to the active step identity so it cannot apply to another exercise. Replay and assisted-stroke SVG masks use explicit alpha compositing, unique replay IDs and wider reveal centerlines to improve coverage of authored outlines. Actual browser animation verification remains pending.

## R521 WIP: Unified vocabulary autocomplete (2026-11-16)

The standalone Write Hanzi vocabulary finder offers My Word Sets and Search vocabulary sources. The same input supports simplified/traditional Hanzi, English glosses, and tone-marked, numbered, spaced or joined Pinyin. The dictionary search uses parameterized SQL and locally ranks exact words/readings before prefixes and partial meanings. The 300 ms query debounce is retained.

The compact suggestions list displays Hanzi, reading, gloss and HSK membership, supports Arrow Up/Down, Enter and Escape, and shows a selected-word summary after selection. The active source and chosen word remain explicit. The finder eliminates its redundant focus ring while keeping a visible keyboard focus affordance on the surrounding field. The spelling/reading source is the installed dictionary; no personal stored data or dictionary generation is changed.

Candidate SQL, ranking and keyboard behavior require locked Rust/WASM compilation and real Chrome testing before accepting R521. Search is capped at 60 retrieved candidates, with seven visible initially and Show more for additional results.
