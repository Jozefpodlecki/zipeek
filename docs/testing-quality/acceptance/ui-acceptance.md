# UI feature acceptance - Cloud Match, CSS, Write Hanzi

This document consolidates the current non-crossword UI checks. Previous release-specific checklists are superseded; versioned evidence remains in [Release history](../../archive-history/RELEASE_HISTORY.md).

## Cloud Match

1. Setup has a shared Back/Cloud Match title row and a compact **Start** action with a play icon. Verify both HSK and Word Set sources, three matching formats and all difficulty settings.
2. During a wave, clouds have varied silhouettes and individual movement speeds, remain visible/selectable throughout their bounded motion, and do not shift lanes when a match is removed.
3. Selection highlights the cloud silhouette, not a detached oval. Correct/incorrect feedback updates the score/lives HUD, not a bottom notification. Announcements remain accessible.
4. Match, miss, time out, pause, resume and stop: assert the pure engine score/lives/wave transitions still agree with the HUD and no pair can score twice.
5. Hide the browser tab during a wave; return to a **paused** round with its score, wave and remaining time intact. Resume explicitly. Test keyboard and touch, small screens, zoom, and `prefers-reduced-motion`.

## Styles and shared shell

1. `crates/zipeek-web/styles/main.css` stays a Tailwind entry point, not a feature-style dumping ground. Tokens, base, accessibility, shared layout and feature-specific CSS retain their separate ownership.
2. Run asset preparation and the normal Trunk build to confirm Tailwind expands all local imports and scans Rust `@source` declarations.
3. Check light/dark and preset themes, button/action classes, focus/contrast, scrollbar styling and Chinese fonts. Verify shell fullscreen routes do not inherit an unwanted sidebar grid.
4. Visually compare Crossword, Cloud Match, Write Hanzi and landing layouts at desktop, mobile and high zoom. Source checks cannot establish pixel equivalence.

## Write Hanzi

1. Change between My Word Sets and English search; changing source or set resets selection and pagination as intended.
2. Search with fast successive edits and verify the latest submitted query wins. Page through more than five results and select a different word.
3. Navigate via a deep-linked word ID, including missing/unsupported entries; confirm explicit statuses and no stale selection.
4. Change characters inside a chosen word; prevent out-of-range indices. Verify the drawing workspace, stroke interactions and responsive sizing are unchanged.
5. Shared `Dictionary` and `WordSetsContext` remain providers. Route-local query/source/loading/selection/character state belongs in one typed workflow reducer; presentation children receive typed props and callbacks.

## Verification

Run `cargo make release-check` and the focused Rust/web tests appropriate to the change. Check behavior in Chrome with real browser data. Node source-contract successes are not build proof. Historical test counts and known missing gates are documented in [Release history](../../archive-history/RELEASE_HISTORY.md).

## Practice setupacceptance)

1. At `/practice`, traverse Path → Exercises → Vocabulary → Summary and back. Verify the single top action bar does not jump below the panels, including on narrow screens and with a large font; Start is followed by the play icon.
2. Toggle ten then zero then all fifteen exercise cards. The selected-count badge must always match the draft, including when moving to Configure and back; test keyboard and screen reader announcements.
3. In Categories & filters, confirm suggestions load before typing, filter immediately, prevent stale response overwrite, and allow selection/removal. With dictionary unavailable, no fabricated category options should appear.
4. Review the Summary icons, HSK and category chips, Word Set alternative, chosen session mode, and truthful practice-only limitations. Test zero and multiple selected filters.
5. Verify initial and per-session loading statuses, screen-reader announcements, light/dark contrast and `prefers-reduced-motion`; confirm no Animato transition is being claimed for this release.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/testing-quality-acceptance-ui-acceptance.md). Dated implementation claims in that snapshot are not current build or browser verification.
