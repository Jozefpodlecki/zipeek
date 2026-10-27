# Shell checks

Theme, sidebar, footer, and shell behaviour coverage. Part of the
[testing index](TESTING.md). Approved requirements in
[D24](../process/decisions/web-and-ui.md#d24-web-shell-and-landing-direction),
[D25](../process/decisions/web-and-ui.md#d25-compact-collapsible-shell), and
[D26](../process/decisions/web-and-ui.md#d26-persistent-landscape-and-loading-presentation).

## Theme, sidebar, and footer coverage

- With no saved selection, verify dark is the default even if the OS prefers light.
  Toggle to light and back to dark, reload, and navigate routes to verify
  persistence. A saved light choice must override the preferred dark default.
- Check the same theme during static HTML loading, Yew loading, and ready content;
  theme initialization must avoid a light flash during the startup handoff.
- Verify the shared left sidebar has the theme toggle and a bottom-anchored GitHub
  icon whose link is `https://github.com/Jozefpodlecki/zipeek`, with an accessible
  name.
- Verify the footer shows Jozef Podlecki, the copyright symbol and current year,
  and the required CC-CEDICT attribution when dictionary data is served.

## Compact shell coverage (D25 / A28)

- Verify actual document dimensions at 1024x768, 1280x720, 1366x768: no
  unintended vertical overflow.
- Verify top-right theme placement; removed decorative copy; drawer visibility
  and state; keyboard collapse and re-expansion; `aria-expanded`; hidden
  navigation when collapsed; full-width main content when collapsed; mobile
  collapse/navigation.
- Verify long mobile content remains scrollable rather than being clipped or
  hidden.
- Verify theme containment is measured against actual top-panel bounds
  (`--topbar-height`), not a hardcoded pixel count.

## Landscape and loading coverage (D26 / A30)

- Block the real wasm resource via CDP to inspect the actual bootstrap DOM, not
  injected preview markup.
- Verify the loading panel is centered and unclipped at 1280x800 and 360x800.
- Verify the original image is decoded and outside `#root`, in both saved themes.
- Verify no vertical overflow on the bootstrap viewport.
- Verify reduced-motion emulation disables spinner animation.
- After unblocking and reloading, verify the live app keeps the same backdrop
  when toggling theme, and that the route `Loader` exposes the same Loading status.

## A27 / A28 / A29 / A30 evidence

A27 added two Chrome tests: landing shell/theme (copyright/attribution, dark
default, keyboard switch, persisted light after reload, malformed saved
preference recovery, continued UI switching when storage writes throw) and
responsive navigation (three real tiles, repository URL, active route, brand
Home, skip-to-content without hash change). Typed CDP viewport emulation checks
360, 768, 1024, 1280 px widths with no horizontal overflow. Screenshots under
`target/a27-*`.

A28 added one Chrome test for actual document dimensions at 1024x768, 1280x720,
1366x768; top-right theme placement; drawer visibility/state/keyboard
interaction; mobile navigation/scrolling. Screenshots under `target/a28-*`.

A29 reuses the same Chrome tests to check refined shell: 48px top panel via
shared `--topbar-height`; containment measured against actual panel bounds;
arrow-free cards; borderless theme control with hover/focus-visible feedback.

A30 added one Chrome test for landscape and loading: CDP-blocked wasm to inspect
bootstrap DOM; centered loading panel at 1280x800 and 360x800; decoded image
outside `#root`; both themes; no bootstrap overflow; reduced-motion spinner
disabled; live-app backdrop retained across theme toggle; route `Loader` shares
accessible Loading status. Screenshots under `target/a30-*`.

Workspace test counts by task: A27 = 55 domain + 2 Chrome; A28 = 55 + 3 Chrome;
A29 = 55 + 3 Chrome; A30 = 55 + 4 Chrome. All five gates pass for each task.
Logs: `target/a27-final-workspace-tests.log`, `target/a28-workspace-tests.log`,
`target/a29-workspace-tests.log`, `target/a30-workspace-tests.log`.

## Current status

D24/D25/D26 are implemented (A27/A28/A29/A30). All shell behaviour covered above
passes in Chrome. Database-backed content (DailyWordCard, Browse, practice
exercises) is not tested by these shell tests — see [browser-db.md](browser-db.md)
and the per-feature coverage files. Theme bootstrapping during startup is covered
above and by [startup.md](startup.md); they overlap intentionally because the
failure modes differ (blank frame vs. wrong colour flash).