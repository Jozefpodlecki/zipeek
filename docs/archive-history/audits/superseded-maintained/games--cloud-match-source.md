# Archived source snapshot: games/cloud-match.md

This source snapshot preserves historical descriptions, not current requirements. Use the maintained document at its original location for approved behavior.

# Games and Cloud Match - R115 approved implementation

## Routes and user intent

`#/games` lists learner-facing mini-games; the first playable title is **Cloud Match** (`#/games/cloud-match`). This is a distinct recreational activity, not an exercise in the unified Practice planner, and has no score-ledger integration.

Players must choose **HSK vocabulary** (HSK 2.0 or 3.0, then a level) or **My Word Sets** before starting. The default is HSK 3.0 Level 1, Hanzi ↔ English, Normal difficulty. The selected source is queried through the existing browser read-only DuckDB provider; personal sets use their stored word IDs. Nothing is hardcoded as a substitute for a selected vocabulary source.

Matching formats are Hanzi ↔ Meaning, Hanzi ↔ Pinyin, and Pinyin ↔ Meaning. Calm / Normal / Storm control wave time limits (55 / 38 / 25 seconds). The app checks distinct playable pairs for the **selected format** and requires at least six before enabling Start. It also reports loading, empty or failed vocabulary queries. A source or level change invalidates the prior preflight result.

## Game flow

A game contains at most nine distinct dictionary words, seeded and deterministically ordered, divided into waves of three pairs. Clouds move horizontally toward the centre with a gentle bobbing animation, and the left/right labels appear in different seeded orders. Match by selecting a cloud on one side and the corresponding cloud on the other; tapping the same side changes selection. Correct matches score 100 plus a 20-point per-combo bonus; incorrect pairs cost one of three lives and reset the combo. Unmatched clouds disappear at wave timeout and cost one life; pausing freezes the game clock. Play ends when all waves have finished or the player loses all lives. End screen distinguishes a fully matched game from a failed game and shows scored matches, mistakes, timed-out waves, active time, best combo, and reviewable confused or skipped vocabulary. Each consumed pair cannot be scored twice; cloud positions stay stable after matches.

Use CSS for visual drift, layout and reduced-motion preferences, not for game correctness. The pure `zipeek-exercises::cloud_match` module owns pair eligibility, wave ordering, timers (in whole seconds), scoring, match identity and state transitions. Yew owns DOM events, interval ticks, source-selection state, and rendering. Gameplay is driven by real database results. It is unscored relative to the shared Practice ledger and not a graded proficiency measurement.

## R120 - Focused gameplay presentation

Active gameplay now uses a distraction-free viewport: the application header, drawer, and footer are hidden only while a Cloud Match game state exists. The vocabulary setup continues to use normal navigation. Gameplay has a compact heads-up display with score, combo, lives, remaining time, pause/resume, and a Stop action requiring confirmation before discarding the active run. On exiting the game or changing routes, the ordinary application shell is restored.

Clouds now move continuously using per-wave CSS animations rather than jumping once per second from JavaScript-computed transforms. Each cloud retains its wave-and-word identity, so removing a matched pair does not move the others into different lanes. CSS synchronizes the visual animation with the existing integer-second game clock, pauses animations when the game is paused or finished, and honors reduced-motion preferences. The pure Rust matching/scoring rules and database sources are unchanged.

Verify in a real browser: desktop/mobile viewport fit, pause motion and timer together, stop confirmation, tab navigation while playing, keyboard focus, round changes, cloud overlap with long meanings, reduced-motion preference, and return to setup. Source tests do not establish visual acceptance or successful Rust/WASM compilation.

## R121 - Legible clouds and stable animation timing

The per-second `--cloud-offset` style has been removed. Each wave retains its own keyed cloud elements with a stable animation duration, so routine score and countdown rerenders do not alter animation phase. Pausing continues to suspend CSS animation alongside the Rust game clock; reduced-motion settings still disable visual drift without changing game rules.

Moving labels use a three-line clamp with the full untruncated text available in the native tooltip and accessible label. Side-specific accessible names distinguish selectable Chinese-side and answer-side clouds. The narrow-screen layout increases label capacity while limiting horizontal travel; short-height screens preserve a minimum playable arena and permit vertical scrolling rather than compressing all cloud lanes together. These presentation rules do not truncate the source dictionary data, change matching eligibility, or modify scoring.

Verify in a real browser: long English glosses, mouse and keyboard focus, pause/resume after several seconds, transition to a new wave, narrow mobile viewport and short landscape viewport, and reduced-motion. Node source contracts cannot establish animation smoothness or successful Rust/WASM compilation.

## Limits and follow-up

- R115 is a source checkpoint. Rust/WASM compilation, focus/keyboard, mobile tap layout, real browser timing and animation have **not** been verified in this environment.
- The first version does not persist high scores, last-used game configuration, replay links or game history. The session seed is generated when Play is pressed but is not serialized into a URL.
- HSK query samples at most 400 candidates for preflight and game selection. Saved Word Sets load their selected IDs. Meaning uses the first normalized English definition, and ambiguous duplicates are excluded by mode. Additional sense-aware equivalence and dataset provenance should be evaluated before adding cross-sense synonyms.
- The Words/HSK query requires the current generated database's normalized definitions table; no schema change is introduced.
- Future games may share a gallery, but should not be exposed as playable without their own tested engine and usable controls.

## Verification

`node --test scripts/cloud-gameplay-behavior.test.cjs`
`node --test scripts/*.test.cjs`
`cargo test -p zipeek-exercises --locked`
`cargo check -p zipeek --target wasm32-unknown-unknown --locked`

Browser acceptance: both vocabulary sources, HSK standards and levels, all three matching modes, every difficulty, valid and invalid pool counts, correct/incorrect matches, timeouts, pause/resume, match disappearance, completed/failed results, route highlight, reduced-motion and narrow-screen layout.

## R126 - Full-viewport crossing and refreshed visual treatment

User screenshots showed the previous R120/R121 drift was visually imperceptible: `--cloud-travel` was clamped to a few pixels and the rows resembled static answer buttons. This approval supersedes earlier "move toward the centre" language. Each left-origin cloud now enters from beyond the left viewport boundary and traverses fully to the right, while right-origin clouds cross in the opposite direction. Animations are linearly timed to the existing whole-wave seconds (Calm 55 / Normal 38 / Storm 25), so there is no extra JS animation clock or per-second position rewrite. The cloud buttons remain clickable throughout their path, selected and focused states remain visible, and pause freezes both the game clock and CSS animation. New wave keys restart the travel timeline. CSS disables travel in reduced-motion mode, returning the clouds to their corresponding left/right positions.

The game HUD now carries wave and match count next to score, combo, lives and time. The two persistent bottom status bars were consolidated into a floating contextual instruction message; gameplay uses a richer but still CSS-only sky, softer cloud shapes, larger label targets and higher-contrast focus treatment. Game scoring and dictionary sources are unchanged.

The global scrollbar skin inherits `--accent`, `--border`, `--canvas`, and `--raised` so all six light/dark theme presets affect scrollbar thumb and track. Firefox uses `scrollbar-color` / `scrollbar-width`; Chromium/WebKit receive their prefixed pseudo-elements. This does not remove semantic scroll areas or override platform high-contrast settings.

**Browser acceptance still needed:** confirm clouds actually traverse edge-to-edge at 100% and 125% zoom, align visual travel with each timer, never become unclickable while crossing, stay on separate vertical lanes, pause/resume without teleporting, reset travel on new wave, and remain accessible with reduced-motion. Check dark/light presets and native vertical/horizontal scrollbar appearance, narrow layout, touch and keyboard focus. Rust/WASM compilation and browser rendering could not be run in the source-only environment.

## R286 - Reviewable game outcomes (source checkpoint)

Cloud Match now distinguishes matching every available pair from ending with depleted lives or expired waves. The game engine records incorrect pair attempts, timed-out waves, active ticking seconds, and the highest consecutive-match combo. Incorrect selections add both involved vocabulary entries to a review set; wave expiry adds only unmatched entries from that wave. This set retains the original source-backed labels and is deduplicated by word ID. It is not a proficiency score or durable Practice history.

The end-of-game overlay shows the matched count and score, these four counters, and a readable list of pairs needing review in the selected matching mode. Play again generates a new game from the current setup and Back to setup restores the existing selection. The completion screen remains keyboard reachable and scrollable at short viewport heights. No database schema, model service, or dependency is added.

Browser acceptance remains open: verify correct and incorrect pairs, timeouts, replay and return, keyboard focus, long labels, small screens and reduced motion. Rust engine tests are authored but cannot be run without Cargo.

## R290 WIP - Game gallery and visual readability

The Games gallery uses one accessible, keyboard-focusable link for each implemented game. It no longer shows redundant Play buttons or Playable badges. Classifier Market, Listening Navigator and Sentence Relay are explicitly noninteractive concept cards, not runnable routes or gameplay implementations.

Cloud Match labels now have a dedicated presentation component with separate Hanzi and meaning text budgets, responsive font-size tiers, two-line bounds and preserved full accessible text. Silhouettes have subtle internal blue-white gradients with distinct per-cloud SVG IDs. Text fitting is currently deterministic rather than canvas-measured; responsive Chrome acceptance must confirm that all labels fit without truncating important identity. The label does not affect word matching or scoring.

Each new wave starts entirely outside the field edges. Its motion model applies a short entry acceleration before settling into the already configured per-cloud difficulty speed. The game retains the single requestAnimationFrame field controller and fixed keyed cloud nodes. A new entry test must verify that the field does not remain empty, that the first wave starts outside the viewport, and that other clouds do not jump on a match.

Five original gradient-based sky scenes rotate on UTC minute boundaries, with crossfaded transitions. They do not copy the watermarked reference images or require network assets. These are procedural backgrounds rather than the five requested possible photographic images. The background does not change gameplay and respects reduced-motion preferences.

This is an uncompiled source-only checkpoint. Gameplay, responsiveness, keyboard focus and browser motion remain subject to Windows Chrome validation. No data migrations, schema changes or new runtime dependencies were introduced.

## R294 WIP - setup ownership and Tailwind presentation

Cloud Match uses `CloudSetupState` for all setup selections and a separate `CloudSession` reducer for active game transitions. HSK standard changes reset the level atomically. `CloudSetupView` renders the configuration form and dispatches setup actions; the page coordinates data loading, browser visibility, sound, navigation, and the running session. No application-wide context is introduced, because this state is local to the game route.

The feature-specific `cloud-match.css` has been removed. Gameplay now renders as a fixed, full-viewport Tailwind-styled surface; cloud silhouettes, labels, HUD, pause dialog and responsive states use Tailwind utilities. The shared requestAnimationFrame controller still owns the moving cloud elements' `transform`, `top`, and `visibility` inline properties. Scenic image URLs are assigned as bounded background-image styles, with Tailwind data variants handling scene crossfades. These imperative properties are animation output, not duplicate Yew gameplay state.

`data-cloud-side` now records the physical movement lane rather than the displayed label language. Mixed Hanzi/meaning sides must be verified in Chrome because the former mismatch could prevent cloud DOM bindings from tracking their positions correctly. Infinity counters use the supported Lucide icon in the HUD and pause overlay. Rust compilation and root/Pages browser checks are still pending.

## R298 WIP - match exit and visual measurement ownership

Successful matching keeps scoring in the existing `CloudGame` reducer. Before dispatch, `CloudField` asks `cloud_exit.rs` to snapshot only the two confirmed matching tiles into a noninteractive overlay attached to `document.body`. The overlay holds their current on-screen rectangles, marks them successful, and fades them away. This makes the success transition independent of wave replacement and Continuous roster changes; `CloudDom` excludes overlay nodes from motion bindings. The overlay does not represent authoritative gameplay state.

Cloud tiles have twelve SVG outlines and two dimension categories: one with the usual per-cloud size variation and one enlarged to support longer visible labels. English definitions continue to use a concise visible phrase with the full definition in the accessible label. `CloudDom` owns measured tile widths and updates them on arena resizing instead of reading offset widths every frame. The remaining `requestAnimationFrame` timeline and `CloudWorld` still own movement. Browser performance and accessibility verification are outstanding.

The Rust/WASM and Chrome tests for this checkpoint have not run here. A successful source contract cannot substitute for visual verification.
