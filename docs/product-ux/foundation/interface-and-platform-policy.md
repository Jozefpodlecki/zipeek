# Interface and platform policy

## Copy, feedback and formatting

Zipeek uses **contextual** language: concise and instructional during exercises, modestly encouraging for genuine achievements. Use sentence case for titles, controls and messages. Give specific, actionable explanations for correct, partially correct and incorrect verdicts where the evaluator can justify them. Do not show unsupported evaluation confidence. Avoid redundant headings and repeated instructions. Use consistent plurals, time formats and numeric presentation.

Use `HSK 7-9` as the advanced-band label. Traditional Hanzi is a secondary display, not a change to dictionary identity or matching keys. Follow existing approved character display rules in their owning feature specifications. Honor keyboard, screen reader, responsive and reduced-motion behavior.

## Appearance

Default theme preference is **System** and follows operating system light/dark changes live. Explicit Light and Dark overrides persist across sessions. Preserve existing saved user selections as feasible while respecting the project's development compatibility policy. Theme colors, including empty and disabled states, must remain legible.

## Startup and database recovery

If essential dictionary initialization or DuckDB loading fails, show a **blocking recovery screen** with a human-readable cause, Retry and optionally expandable diagnostic details. Do not enter the normal application with fabricated empty data. Retry must actually rerun initialization without erasing local settings, Word Sets or learning history. Genuine successful queries returning no matching words are ordinary empty states, not startup failures.

## Public discovery

Zipeek is a publicly indexed application. Maintain descriptive titles and descriptions, canonical URLs, appropriate indexing rules and Open Graph previews. Avoid exposing, indexing or sharing private session state, local learner data or temporary routes. Respect hosted `/zipeek/` paths and any development root-path routing differences.

## Engineering policy

Rust implementation code has **no comments**, including explanatory comments, doc comments, TODO comments and lint-allowance justifications. Prefer clear identifiers, types and module boundaries; document architecture rationale here in maintained documents instead. Dependencies should prefer current stable releases, but **any** addition or update, including transitive updates through `cargo update`, requires explicit user approval. Keep the root lockfile and locked checks. Never perform dependency upgrades incidentally.
