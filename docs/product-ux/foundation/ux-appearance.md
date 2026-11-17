# Appearance and theme behavior

## Theme preference

The default is **System**, which follows operating-system light and dark appearance changes without a reload. A user-selected Light or Dark preference overrides System and persists. Preserve a previously saved explicit preference where feasible without requiring backwards-compatibility readers that conflict with the active-development policy.

The loading shell, database error recovery screen, route content, cards, header and footer must use a coherent theme; no avoidable flashing of the opposite theme. Contrast, selected-state clarity and focus-visible affordances remain usable in both themes. The landscape art can be retained with translucent overlays when text remains legible.

## Global header and footer

The top panel owns navigation, Zipeek branding, the [long evidence-based dragon](header-and-dragon.md) and global controls. It does not repeat page titles. The footer is centered within the main content region, excludes sidebar width and wraps accessibly on narrow viewports without overlapping the Practice reminder. Preserve mandatory legal attribution, Credits link and the repository link with appropriate accessible names and safe external-link behavior.

## Motion

Reduced-motion preference disables nonessential progress shimmers, tile movement and transition effects without removing state feedback. Themes, layout and status must be accessible even when motion is suppressed.

See [platform policy](interface-and-platform-policy.md), [global UX](../ux.md) and [acceptance](../../testing-quality/acceptance/approved-behavior-acceptance.md). A prior dark-default theme contract is preserved in [historical source](../../archive-history/audits/superseded-maintained/product-ux--foundation--ux-appearance-source.md) only.
