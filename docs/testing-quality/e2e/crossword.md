# Hanzi Crosswords - testing and acceptance

This is the **current** acceptance checklist for `#/games/crosswords`. It replaces release-specific checklists that described intermediate implementations; historical results remain in [Release history](../../archive-history/RELEASE_HISTORY.md).

## Source and state contracts

- HSK 2.0/3.0 bands are selected in the initial setup. Gameplay and results **do not** offer a Change HSK levels action; Play again retains the original bands.
- Eligible crossword words come from browser-local DuckDB for the selected bands; no 13-word fallback. The generator targets at least six connected words, four crossings, and ten occupied cells. If it cannot produce a valid puzzle, report a generation failure rather than displaying a two-word puzzle or claiming the dictionary lacks words.
- A generated layout uses canonical vocabulary ordering and a seed so that refreshing restores the same board, filled characters, score and selected bands. New game uses a different seed and retains the bands. Generated clue coordinates remain consistent with rendered cropped board geometry.
- Each newly started game records its distinct Hanzi answer words in a browser-local 100-word least-recently-used rotation list and its vocabulary fingerprint in the last 10 game signatures. New game evaluates eight deterministic seed candidates and prefers valid puzzles with less recent vocabulary overlap; this is a preference, not an exclusion, so small pools remain playable. Restoring a saved session does not count the same session twice. This rotation is independent of learning-history consent and Word Sets.
- The `Crossword` domain owns legal placement, intersections, next-unfilled-cell selection, scoring and completion; Yew owns database loading, keyboard/focus and rendering. Setup/loading/errors use the workflow reducer; game actions use the gameplay reducer.

## Browser acceptance

1. Start from a fresh route and choose HSK 3.0 level 1; repeat with HSK 3.0 levels 4-6, HSK 2.0 level 1, and a cross-standard selection. Confirm clues come from the chosen vocabulary. If the generator fails, the error must be accurate and the preceding playable board must not be replaced with an undersized puzzle.
2. At desktop (1366×768, 1440×900), narrow (390×844) and 125% zoom, confirm the desktop viewport is not unnecessarily scrolling, the crossword cells are legible, the tile tray scrolls horizontally, and clue scrolling is independent. No global app bar or duplicate headings should appear in fullscreen gameplay.
3. Clues show gloss and Pinyin only (no numbering/direction prefix or checkmark). Click or keyboard-activate an unfinished clue: highlight its next unanswered cell. Hover must not change selection. Tapping a tile fills the next empty cell; already-filled crossing cells are skipped. Direct square placement and drag/drop remain functional.
4. Complete a clue: it becomes muted and disabled. Stale clicks or keyboard activation must not reactivate it. Correctly filled squares are subdued but legible, including intersections; unfinished clues remain actionable.
5. Type Pinyin without focusing a field: a floating suggestion widget opens. Enter selects a unique reading; number keys 1-5 distinguish tone; multiple characters with the same reading and tone still require explicit choice. Escape closes the widget. Input inside other forms and modified browser shortcuts must not trigger the widget. No permanent Pinyin field or instructional label belongs in the tray.
6. Place a wrong character, test bank depletion, change active clues, refresh halfway through, and verify consistent scoring and persisted selections. Confirm the progress and HUD are updated from one authoritative gameplay state.
7. Start several new games in the same HSK selection, inspect varying word sets where the pool permits, refresh midway and confirm the current puzzle and rotation history persist. Check that small vocabulary pools fall back to playable previously used words, that rotation history is capped, and that a repeated save does not increase usage. Complete the crossword: results replace the board and show points, completed words, mistakes, **Play again** and **All games** only. Play again retains the same HSK selections.
8. Check loading and generation feedback, slow/failed DuckDB queries, unavailable worker, keyboard focus, reduced motion, accessibility labels and browser console diagnostics.

## Validation commands

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo check -p zipeek-web --target wasm32-unknown-unknown --locked
cargo test -p zipeek-exercises --locked crossword
cargo test -p zipeek-web --locked
cargo test --locked -p zipeek-e2e
```

`cargo make release-check` runs the configured gates. Source-text Node tests do not certify Rust compilation, DuckDB integration or rendered Chrome behaviour. See [Testing](../acceptance/testing.md) and [UI architecture](../../architecture/ui-architecture.md).
