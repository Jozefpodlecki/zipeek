# Web and UI

Approved shell, motion, and styling decisions. Each entry links back to the
[index](index.md) and to its owning document. Bodies are the approved
requirement as recorded on the date shown; they do not imply the feature is
implemented.

---

## D24 Web shell and landing direction

**Status:** Approved 2026-10-27.
**Owner:** [UX](../product/UX.md), [PRODUCT](../product/PRODUCT.md),
[ARCHITECTURE](../architecture/ARCHITECTURE.md).
**Index row:** [D24](index.md#approved-decisions).

The user selected the shell/landing direction on 2026-10-27:

- **Palette:** neutral slate dark/light with blue accents.
- **Layout:** desktop left rail; compact mobile header.
- **Navigation:** shared Home/Practice/Browse links with active-route indication;
  brand links Home. Repository link, theme control, keyboard skip-to-content.
- **Landing:** prominent daily-word area followed by exactly three tiles —
  Practice, Browse, Random.
- **Attribution:** copyright footer plus required CC-CEDICT attribution.

Navigation/shell work comes first; dictionary-backed content remains A06/A12
and no daily word is fabricated. Daily-card fields and unrelated UX decisions
remain open.

Implementation status: A27 delivered the shell. Current UI uses agreed unavailable
copy for the daily word; actual selection is A12.

Superseded in part by D25's theme placement change.

---

## D25 Compact collapsible shell

**Status:** Approved 2026-10-27 by direct user request; supersedes D24's
theme-in-sidebar placement.
**Owner:** [UX](../product/UX.md), [PRODUCT](../product/PRODUCT.md),
[ARCHITECTURE](../architecture/ARCHITECTURE.md).
**Index row:** [D25](index.md#approved-decisions).

Amendments to D24:

- Remove decorative headers such as "Chinese vocabulary" and "A fresh starting
  point" and landing marketing filler. Do not restore them.
- Move theme to the top-right of a **permanent top panel**.
- Make the drawer collapsible via an accessible keyboard-operable toggle
  exposing `aria-expanded` and `aria-controls`; hidden navigation state is
  respected by assistive technology.
- Remove unnecessary vertical scrolling; use natural document scrolling when
  content genuinely exceeds the viewport. No global overflow hiding.
- Preserve functional page/card/action labels and access to drawer/theme controls
  in both expanded and collapsed states.

Implementation: A28 owns the amendment. A29 refines:
- no decorative tile arrows;
- `--topbar-height: 3rem` (48px) shared across grid/drawer/main;
- reduced footer padding;
- borderless resting theme control, with hover and focus-visible feedback preserved.

Drawer expansion state is AppShell-local, initially true, preserved across
routes, reset on reload.

---

## D26 Persistent landscape and loading presentation

**Status:** Approved 2026-10-27 by direct user request.
**Owner:** [UX](../product/UX.md), [PRODUCT](../product/PRODUCT.md),
[ARCHITECTURE](../architecture/ARCHITECTURE.md).
**Index row:** [D26](index.md#approved-decisions).

Reuse the existing unmodified landscape image as the shared loading and app
backdrop; compose light/dark surfaces with it; replace the clipped loading
treatment.

Requirements:

- The image lives **outside** Yew's `#root` mount point (fixed `.app-backdrop`
  before `#root`). Yew mounting, route changes, and theme toggles cannot remove
  or recreate it.
- Image bytes unchanged. Current SHA-256 for both the embedded WebP and
  `web/public/background.webp` is
  `64fc1cb7d2b34ff89ff03f5cef4bcfe3df64eae9f27c6ca073f72e8d2e201f93`.
- `main.css` provides theme-specific shade/surface/card-tint variables; surfaces
  are translucent with backdrop blur. `inline.css` holds critical backdrop/
  loading geometry so initial render does not depend on percentage height or
  wasm.
- Static HTML and Yew loaders share a small centered spinner/Loading panel with
  accessible status text and reduced-motion support. No artificial progress or
  loading delay.
- Reduced-motion preferences disable spinner animation; the landscape itself
  remains visible in both themes.

This is presentation work only. A06/A21 still own database initialization,
readiness, and retry. A30 implements D26; the actual DB readiness pipeline is
A06/A21.

---

## D29 Scoped UI state and motion (proposed)

**Status:** Proposed 2026-10-27 by user review. Existence of the need is
approved; concrete plan requires implementation review.
**Owner:** [UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md), [UX](../product/UX.md).
**Index row:** [D29](index.md#approved-decisions).

The user's 2026-10-27 code review requested modular e2e, scoped UI state, and
motion support. This decision records the *requirement*; the detailed plan in
[UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md) is a proposal, not approval
to add crates or change dependency features.

Approved requirements:

- Scoped read/command contexts and reducers; persistent database/theme providers;
  motion preference provider, small Animato adapter, interruptible route host.
- Preserve the user-added Animato 1.7.3 with `path`, `physics`, and `color`
  features. Do not enable `yew-csr` or add another animation library without
  explicit approval.
- Do not expose raw state setters as another feature's public command interface.
- Motion must not change canonical URL state, replay inputs, RNG draws, scoring,
  or Fixed completion.
- Honor reduced-motion preferences; essential semantic changes happen even when
  animation is disabled.
- No app-wide reducer containing every result list, input, and animation tick.
  Providers belong at the nearest shared ownership boundary.

Implementation: A31 owns scoped providers/reducers; A32 owns motion profile and
route transitions. Both are open in TASKS. Choose motion profiles before
enabling Animato features.

---

## D31 Tailwind-first component styling

**Status:** Required by the user on 2026-10-27. Implementation not started.
**Owner:** [CODING_STANDARDS](../CODING_STANDARDS.md),
[UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md).
**Index row:** [D31](index.md#approved-decisions).

Correct the drift from Tailwind to bespoke component CSS. Tailwind 4.3.3
remains installed/imported and built by Trunk; the drift is in *how components
are authored* — recent shell/card work uses bespoke main.css selectors instead
of utilities.

Requirements:

- Tailwind utilities are the default for layout, spacing, sizing, typography,
  borders, responsive rules and interaction states.
- Keep shared theme/design tokens in CSS where appropriate and consume them
  through utilities. Preserve the slate/blue palette and translucent landscape
  surfaces.
- Allow focused custom CSS for critical pre-WASM bootstrap styling, shared
  tokens, keyframes, or a concrete effect that benefits from it. A reason for
  one exception does not justify moving ordinary component styling into
  stylesheets.
- Do not hide the same bespoke component system behind blanket `@apply` rules.
  Factor repeated UI into Yew components and small explicit utility variants.
- Keep conditional variants as complete class strings Tailwind can detect; do
  not construct class names from fragments at runtime.
- Verify Tailwind 4 source detection for Rust/Yew files and remove obsolete
  conflicting rules as components migrate.

A35 owns migration with visual/behaviour parity: before/after screenshots at
`/` and the `/zipeek/` build, all five gates and Chrome coverage. Update e2e
selectors to stable semantic/data hooks when a styling class is removed.

This direction does not approve dependency upgrades, a redesign, or a new
styling framework. The migration itself has not started.