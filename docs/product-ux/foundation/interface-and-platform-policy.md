# Interface and platform policy

## Copy, feedback and formatting

Zipeek uses **contextual** language: concise and instructional during exercises, modestly encouraging for genuine achievements. Use sentence case for titles, controls and messages. Give specific, actionable explanations for correct, partially correct and incorrect verdicts where the evaluator can justify them. Do not show unsupported evaluation confidence. Avoid redundant headings and repeated instructions. Use consistent plurals, time formats and numeric presentation.

Use `HSK 7-9` as the advanced-band label. Traditional Hanzi is a secondary display, not a change to dictionary identity or matching keys. Follow existing approved character display rules in their owning feature specifications. Honor keyboard, screen reader, responsive and reduced-motion behavior.

## Progressive disclosure and useful interface copy

Show text when it helps a person make a decision or recover from a problem, not merely to describe the current implementation state. A visible label must earn its place by explaining an unfamiliar action, identifying data, or communicating important information that cannot be inferred from the interface. Prefer discoverable, theme-integrated controls, clear grouping and progressive disclosure to permanent instructional paragraphs.

- Do not expose internal state names such as `Empty`, `Drawing`, or `Image added` as badges when the current content already communicates that state. Keep such state in typed application logic.
- Do not repeat a page title as a card heading or repeat the same instruction in the page subtitle, action panel and footer. A simple drawing workspace needs the canvas, meaningful actions and results, not prose explaining each step.
- Use disabled affordances for unavailable actions when the cause is obvious, rather than permanently showing messages like `Add input before submitting`. If an unavailable action needs a non-obvious explanation, place it contextually near that action.
- Prefer concise icon-only controls for familiar secondary actions such as Undo and Clear when supported by an understandable icon. Give them accessible names, visible keyboard focus, descriptive tooltips and appropriately sized touch targets. Do not rely on icon shape alone for unfamiliar or high-impact actions.
- Use visible loading feedback during actual waits, a clear empty state only when it helps orientation, and actionable error or recovery text only when a failure occurs. Do not render status chatter or success narration after routine edits.
- Present recognition uncertainty where it changes a decision, such as an unverified candidate; do not surround every result with repeated disclaimers. Keep verified dictionary facts distinguishable from AI suggestions.
- Privacy and consent information must remain accessible and timely. Preserve necessary disclosures at the point of consequential sharing or in a linked contextual explanation, rather than repeating long privacy paragraphs throughout the workspace. Explicit user-triggered uploads and existing privacy protections remain mandatory.
- Audit copy in desktop, mobile, light and dark layouts. Remove redundant text before shrinking fonts or tightening spacing; reduced reading burden is a design goal, not merely a vertical-height optimization.

These rules do not remove accessible names, validation messages, meaningful empty states, setup prerequisites, security disclosures, or error diagnostics. Screen-reader labels and programmatic state remain available even when redundant visible copy is removed.

## Form controls and shared interaction design

Zipeek must not expose unstyled browser-default selects, checkboxes, radios or pagination that clash with the active theme. Use one consistent, reusable control vocabulary for sizing, color tokens, disabled/hover/focus states and pointer/touch targets. Preserve native HTML semantics whenever possible while styling their presentation; a custom-painted surface is not permission to lose keyboard, label, focus, screen-reader or form behavior.

- Prefer semantic native input elements with theme-integrated appearance where their behavior is sufficient. For application-specific selection patterns, reusable Yew button-based checkbox or switch controls may be used only with complete accessible state and keyboard operation.
- Put reusable controls in the relevant shared UI layer when multiple features need them; avoid independently inventing one-off toggles and dropdowns per route. For prototypes that do not yet have consumers outside a feature, a focused feature-local reusable component is acceptable.
- For custom listboxes or select menus, support Escape, arrow keys, Enter/Space, focus restoration, selection announcement, click-away behavior and mobile touch before replacing a native select. Prefer a styled native select if that full interaction contract is not justified.
- Keep filter values, pagination selection and checkbox state in typed reducer state when they affect multiple panels. Users should never need to infer a persistent selection from hover color alone.
- Test in Light, Dark, System, reduced motion, keyboard-only and narrow viewport states. Use design tokens rather than hard-coded colors.

## Appearance

Default theme preference is **System** and follows operating system light/dark changes live. Explicit Light and Dark overrides persist across sessions. Preserve existing saved user selections as feasible while respecting the project's development compatibility policy. Theme colors, including empty and disabled states, must remain legible.

## Startup and database recovery

If essential dictionary initialization or DuckDB loading fails, show a **blocking recovery screen** with a human-readable cause, Retry and optionally expandable diagnostic details. Do not enter the normal application with fabricated empty data. Retry must actually rerun initialization without erasing local settings, Word Sets or learning history. Genuine successful queries returning no matching words are ordinary empty states, not startup failures.

## Public discovery

Zipeek is a publicly indexed application. Maintain descriptive titles and descriptions, canonical URLs, appropriate indexing rules and Open Graph previews. Avoid exposing, indexing or sharing private session state, local learner data or temporary routes. Respect hosted `/zipeek/` paths and any development root-path routing differences.

## Engineering policy

Rust implementation code has **no comments**, including explanatory comments, doc comments, TODO comments and lint-allowance justifications. Prefer clear identifiers, types and module boundaries; document architecture rationale here in maintained documents instead. Dependencies should prefer current stable releases, but **any** addition or update, including transitive updates through `cargo update`, requires explicit user approval. Keep the root lockfile and locked checks. Never perform dependency upgrades incidentally.
