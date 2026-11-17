# Games - source audit (updated R400 WIP, 2026-11-14)

**Scope:** Rust/Yew source inspection of the twelve routes in `crates/zipeek-web/src/pages/learning/games/gallery.rs`, available `zipeek-exercises` domain implementations, and existing Rust/Chrome test targets. **Not a browser acceptance report.** Cargo, WASM, audio hardware, and Chrome tests have not been run in the authoring environment; no game is certified production-ready by this review.

| Game | Source-level implementation | Remaining acceptance risks |
| --- | --- | --- |
| **China Map Challenge** (formerly Province Quest) | Implemented playable session and region selection using installed province/prefecture assets, difficulty choices, native SVG map with zoom/pan/reset, pronunciation button for disclosed prompt, and independent Rust session reducer | Validate map drag vs. answer click on mouse/touch; scrolling at shorter viewports; map theme contrast and legibility of Hong Kong/Macau; licensing/territorial representation of geographic polygons before public distribution |
| **Icon Pairs** | Shuffled mixed-grid matching game with 48 curated icon-Hanzi semantic associations across 24 icons; supports HSK 2.0/3.0 level selection filtered by actual dictionary records, round-scoped local feedback, and family-neutral `yew_icons::IconData` | Review and grow semantic associations toward the proposed 2,000-icon target; validate HSK filters against installed DuckDB and verify mixed-grid layout on narrow/short viewports; no mastery claims |
| **Time Weaver** | Authored date/weekday question sequence, tile interaction, round feedback and scored completion in pure Rust domain | Full cross-theme mobile, accessible keyboard and browser acceptance pending |
| **Tone Detective** | Playable 12-question tone recognition sequence with browser speech synthesis and scoring | Dependent on available Mandarin voice; verify audio failures, voice quality, and screen-reader behaviour on real hardware |
| **Cloud Match** | Interactive matching game with reducer-owned session and selectable vocabulary; browser pronunciation integration | Confirm animation, timing, selection and responsiveness in Chrome; speech optional |
| **Hanzi Crosswords** | Route and independent crossword implementation present; vocabulary/clue assembly logic is not a visual placeholder | Test real dictionaries and solvability, keyboard focus, responsiveness and restoration |
| **Classifier Market** | Authored classifier question bank with setup, state machine and scoring | Curriculum quality and dictionary filtering need review, plus browser acceptance |
| **Listening Navigator** | Authored direction/route questions, setup and state machine with audio playback | Browser speech availability, spatial prompt clarity, answer accessibility |
| **Sentence Relay** | State-backed word arrangement, undo, clear, checking and AI-assisted generation of rounds | Requires working AI service/model for generated content; validate rejection/retry and unsupported models |
| **Market Bargain** | Price-listening challenge with selected difficulty, state and browser audio | Audio-dependent; handling unavailable speech and numeric discrimination should be browser tested |
| **Radical Forge** | Authored radical/component questions with answer reducer, feedback and restart | Chinese component pedagogy review; character font dependence and keyboard/mobile testing |
| **Dialogue Quest** | Choice-driven dialogue reducer with AI generation and setup | Requires configured AI provider; review generated-language quality, error handling and answer grading |

**Priorities:** P0 verify China Map Challenge interaction and source/attribution before release; P1 run native/WASM and managed Chrome tests for all games; P1 review Mandarin item quality and speech-provider fallbacks; P2 refine visual/presentation consistency and mobile layout. Do not infer mastery or persistent progress from score-only sessions. **Icon Pairs is the newly approved additional game.** The suggested Number Sprint was expressly withdrawn and must not be implemented.

The `scripts/**` JavaScript source-contract test suite was removed at the user's request. Maintain behavioural coverage in Rust (`zipeek-exercises`, `zipeek-cli`) and existing Rust/Chrome E2E tests. CI and Cargo Make no longer invoke deleted Node tests.
