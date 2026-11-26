# Global header and dragon-shaped learning progress

## Design intent

The current visual reference is the **long, detailed horizontal Chinese dragon** approved during the UI review. The dragon itself forms the progress indicator. Do not substitute a generic rectangular bar with an adjacent dragon icon, simple wave, emoji or unsupported percentage badge.

The header contains branding and navigation at the start, the long dragon across available space, and global appearance/settings controls at the end. Remove redundant centered page titles from the global header; each page owns its contextual title inside its main content. Active Practice exercise names are also omitted from their cards.

## Dragon artwork and progress

- Preserve intricate dragon-head details, horns, whiskers, scales, flowing body and tail, and the violet-and-gold visual direction established by the approved mockup.
- The dragon's body itself visualizes **persisted pedagogical evidence**, with brighter completed regions and a subdued unreached region. The source of progress is not merely completed sessions, visited words or independent counters.
- Show **no visible stage name, level, percentage or other labels above the dragon**. Explanatory details appear only on deliberate pointer/keyboard interaction.
- Progress transitions smoothly without distortion, scaling artifacts, stale renders or changing the header's height. The same artwork should align in the completed and incomplete layers.
- A detail panel on click/keyboard activation may show the relevant evidence categories, current developmental stage and next milestone. Do not invent numerical mastery measures until their actual mapping to persisted evidence is defined.
- Artwork should be as faithful to the agreed reference as feasible; an expedient vector approximation is not acceptance. Rendering technique and asset form should be chosen based on fidelity, sharpness and sustainable performance.

## Responsive behavior

The dragon uses remaining horizontal space but must not crowd or overlap the navigation menu, branding, user controls or focus rings. On narrow screens, show a recognizable compact portion and provide progress details on interaction. Mobile collapse must not imply different scoring. Use semantic controls with focus-visible styling and an accurate accessible progress description derived from real data.

## Animation and reliability

Animate only meaningful progress changes and occasional stage transitions, never continuous distracting motion. Respect `prefers-reduced-motion`. Cancel stale animation tasks during route changes, data refresh and unmount; delayed frames must not present a newer learning record with an older fill state. Treat a missing or failed evidence store as an error, not `0% mastery`.

## Verification

Test actual artwork at the target desktop header width, intermediate breakpoints and narrow mobile layouts. Verify detail panel mouse/keyboard/focus behavior, persistent evidence consistency, theme contrast, reduced-motion behavior and loading/error states. Compare the rendered implementation to the approved mockup and actual app screenshots; do not claim visual fidelity from a source-code inspection alone.


## Implemented unassessed state and future scoring

The current header source uses `components/shell/app_shell/header/dragon.rs` and the unassessed artwork at `static/practice/header-dragon-unassessed.webp`. The illustration is shown independently of the Practice recording preference and explicitly reports unavailable/unassessed evidence, not a completion percentage. The diagnostic pedagogy worker and optional Practice outcome records do not authorize a dragon progress projection. Connect only validated, durable, policy-approved R608 scoring after the evidence journal, canonical denominator, access and UI states are established. Visual fidelity, responsive and reduced-motion acceptance remain open.
