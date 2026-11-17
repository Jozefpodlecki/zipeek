# Themes and Courses - approved product direction

## Scope and status

The user approved two independent navigation destinations, **Themes** and **Courses**, and a six-option appearance gallery plus a course-library design. The described implementation is an early source-level slice, not browser acceptance. Preserve the established light/dark toggle and Chinese font preference.

### Themes (`#/themes`)

- Six explicitly named presets: Lavender Classic, Jade Light, Sunset Warm, Midnight Blue, Ink Minimal, Autumn Maple.
- Each card previews its palette. Selecting a card applies semantic CSS tokens to the application immediately and persists `zipeek.appearance.preset` locally.
- Density control has Grid (baseline), Comfortable (readability-oriented spacing) and Compact (reduced spacing); its independent preference persists under `zipeek.appearance.density`.
- The existing top-bar light/dark toggle is independent. Selecting a preset initially chooses its default contrast mode; users may still toggle afterward. The Chinese font setting remains in Settings.
- The palette preset is distinct from the existing `zipeek-themes` crate's *vocabulary-topic background* `ThemeId`; do not mix the two concepts.
- No custom user-authored palettes, server synchronization or image/background replacement yet. Palette preview is a small CSS representation, not a snapshot of the actual rendered app.

### Courses (`#/courses`)

- Library categories: Pinyin & Pronunciation, Everyday Chinese, Topics & Vocabulary, Dialogues & Scenarios, Grammar Essentials, HSK Preparation.
- Categories beyond Pinyin are intentionally marked **Planned**; the interface does not imply finished lesson content or completion metrics.
- First genuine vertical slice is `#/courses/pinyin` with three authored lessons: Mandarin tones, initials and finals. Individual lesson pages are at `#/courses/pinyin/:lesson`.
- Pinyin lesson pages present instructional content and examples without a manual completion action. No Pinyin completion or mastery status is recorded by these pages.
- Lessons are authored and version-controlled in the web source in the historical source. Content source model, editing/publishing workflow, audio pronunciation references, quizzes, prerequisites, versioned curricula and interoperability with the scored Practice session should be specified before further expansion.
- Content is instructional and does **not** claim pronunciation grading, validated audio playback or official HSK certification.

### Architectural boundary

Courses sequence explanations and learning activities; Practice supplies reusable exercises and assessments. The dictionary supplies classified vocabulary. Adding a course should not require modifying dictionary schema solely to store an authored curriculum. Plan separate course schemas or content assets when the lesson library grows.

### Validation still required

- Windows `cargo test --workspace --locked` and `cargo check -p zipeek --target wasm32-unknown-unknown --locked`.
- Confirm switching among all six presets, persistence after reload, light/dark mode, keyboard and screen-reader selection, 3 density modes, small-screen layout, font independence.
- Confirm Courses/Pinyin/individual lesson navigation, deep links, the absence of manual completion or self-reported mastery, unknown lesson handling and no DuckDB dependency.
- Visual browser contrast/a11y checks, especially preset accent colors and text legibility.

## Approved visual reference

See `docs/product-ux/themes-courses-approved-concept.png` for the original approved side-by-side Themes/Courses mockup. The implementation is a responsive adaptation, not a pixel-perfect reproduction; the current lesson and appearance options are documented above.

## Historical completion implementation (removed)

Earlier releases offered a manually marked Pinyin-completion flag and a corresponding chart. Those features have been removed: reading an instructional lesson cannot establish learning proficiency. Pinyin lessons are navigable in authored sequence, but progression, completion and artwork reveals require future verified learning evidence. The historical release details remain in `docs/archive-history/RELEASE_HISTORY.md`.

## Pinyin lesson discovery (WIP)

The Courses dashboard now renders each of the three authored Pinyin lessons as a keyboard-accessible deep link, as a study link. The prior manual-completion status and Start/Continue/Review state machine have since been removed. This is an incremental catalogue improvement, not the planned Courses mosaic or an assessment-based completion system. Browser and WASM verification are pending.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/learning-practice-courses-themes-and-courses.md). Dated implementation claims in that snapshot are not current build or browser verification.
