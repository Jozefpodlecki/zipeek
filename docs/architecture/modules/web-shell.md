# web-shell module

Startup handoff, static loading shell, shell components, and landscape
composition. Part of the [architecture overview](../ARCHITECTURE.md). Approved
UI direction in [D24](../../process/decisions/web-and-ui.md#d24-web-shell-and-landing-direction),
[D25](../../process/decisions/web-and-ui.md#d25-compact-collapsible-shell), and
[D26](../../process/decisions/web-and-ui.md#d26-persistent-landscape-and-loading-presentation).

## Startup: static loading shell and Yew loading state

The placeholder in `web/index.html` is an **initial loading shell** (also called
a static loading screen or bootstrap loader). It gives the browser useful HTML
to paint while the JavaScript/WebAssembly app downloads and initializes. Resource
prefetching describes fetching resources early; the loading shell describes the
visible placeholder during startup.

HTML owns a persistent decorative backdrop outside `#root`, using the original
embedded image bytes. Theme-specific overlays and translucent shell surfaces
compose the image with both palettes. Bootstrap applies the saved theme or dark
default before rendering. The `#root` bootstrap view and Yew `Loader` share the same
centered spinner/Loading panel styling; reduced-motion preferences disable its
animation. Bootstrap fills the viewport; the route `Loader` uses a bounded inline
variant so it does not add a second viewport height inside the shell.

`main.rs` mounts Yew into `#root` without replacing the backdrop. `DatabaseProvider`
inside `AppShell` owns Loading/Ready/Failed and Retry. Navigation/theme stay usable
while routes wait for the dictionary. Failures are logged with technical detail;
the UI uses a concise unavailable message. The separate `ErrorPage` remains unused.

### Required handoff

1. **Before Yew mounts:** the static HTML shell remains visible while the app
   boots. Keep its essential loading appearance available before Yew runs.
2. **Yew's first render:** initialize app/database state to `Loading` immediately,
   before any asynchronous effect starts. Render the Yew loading view as the
   shell is replaced, preserving a continuous loading experience without a blank
   frame or a flash of the ready UI.
3. **Database initialization:** remain `Loading` while initializing DuckDB-WASM,
   fetching `hanzi.duckdb`, and opening the read-only database for queries.
   Yew mounting or the fetch completing alone does not mean the database is ready.
4. **Ready:** show database-backed content only after initialization succeeds
   and the database is queryable. A valid empty database can be Ready; its Word
   of the Day card uses D05's empty placeholder.
5. **Load failure:** leave the loading state and expose retry rather than spinning
   forever. Apply D05's "No word available" plus Retry behaviour to the card;
   retry re-enters `Loading` until the database is ready or fails again.

This lifecycle is implemented by A06's `DatabaseState`/`DatabaseProvider`. The app
owns database readiness; individual data queries may still have their own loading
states after startup.

## Implemented web shell and landing foundation (A27 / D24)

User-selected direction: neutral slate light/dark with blue accents, desktop
left rail, compact mobile header and a daily-word card followed by three tiles.
The implementation is split into `AppShell`, `Navigation`, `ThemeProvider`,
`ThemeToggle`, `AppFooter`, `DailyWordCard` and `NavigationTile` components, one
per file. `theme.rs` owns preference policy and state; `Home` composes the page.
Legacy `Layout`/`SearchBar`, `SettingsWidget` and `Footer` are not mounted by the
new shell and are preserved.

Navigation preserves the hash router and `/zipeek/` deployment path. Home, Practice
and Browse show the active route; Practice also covers match and Random routes.
The brand returns Home. Skip-to-content focuses main without changing the router
fragment. The desktop repository link remains at the drawer bottom; the theme control
lives independently in the permanent top-right panel. CSS tokens drive both palettes;
focus styles and reduced-motion handling are present. The drawer/inline-navigation
breakpoint is 48rem, and landing tiles stack through 60rem. Viewport policy for
future exercise and browse layouts remains open UX work.

`DailyWordCard` currently renders "No word available" with no fictitious word,
selection query or Retry action. Practice/Browse/Random tiles navigate to the
existing routes, which honestly remain feature placeholders. A06/A12 must connect
real queries, deterministic daily selection and Random practice. A21 still owns
actual database readiness/loading/failure state; visual theme consistency alone
does not implement that pipeline. Static loading markup now respects the same
palette/default, and the saved choice controls its landscape overlay rather than
hiding the image.

Chrome tests cover actual theme changes/persistence/failure, route links and
active indication, keyboard skip, attribution and widths 360/768/1024/1280px.
Desktop dark/light and complete mobile-light screenshots are saved under `target/`.

## Compact shell amendment (A28 / D25)

`AppShell` owns a session-local `expanded` boolean. A top-panel button exposes
`aria-expanded` and `aria-controls`; it hides/shows the named navigation aside.
The top panel and theme switch remain reachable in both states. Collapsing on
desktop removes the drawer grid column, giving main the available width. Brand
navigation and skip focus retain their existing hash-router behaviour. Drawer
state survives route changes but is not persisted across reloads.

Decorative page kickers, brand subtitle, landing tagline/section filler and the
Daily discovery badge were removed. Functional page and card headings remain.
Top-panel height is accounted for in drawer/main sizing; condensed landing
spacing and a 60rem tile breakpoint avoid accidental desktop overflow. The page
uses natural document scrolling, with no global overflow hiding to clip content.
The mobile shell uses a column flex layout so an open inline drawer does not
add its height on top of an independently forced viewport-height main area.

Chrome measures actual document dimensions: no scroll at 1024x768, 1280x720 or
1366x768; expanded/collapsed desktop and mobile navigation work; long mobile
landing content still scrolls. Screenshots under `target/a28-*` were inspected.

## A29 refinements

A29 refines the shell without new behaviour or dependencies: top-panel height uses
`--topbar-height: 3rem` across the grid, sticky drawer and main-height calculation.
Footer padding is reduced, and the theme button has no resting border; hover
and focus-visible feedback remain. `NavigationTile` now renders its functional
icon directly, without a decorative arrow or extra wrapper.

## Persistent landscape and loading composition (A30 / D26)

The existing embedded WebP and `web/public/background.webp` are byte-identical.
Their SHA-256 remains
`64fc1cb7d2b34ff89ff03f5cef4bcfe3df64eae9f27c6ca073f72e8d2e201f93`.
No raster file was changed. A fixed `.app-backdrop` before `#root` owns that same
image and a CSS shade; `#root` is layered above it. Yew mounts and route/theme
changes cannot remove or recreate the backdrop. The legacy `Background` component
is not mounted; do not add a second image inside the app.

`main.css` defines theme-specific shade/surface/card-tint variables. Header, drawer,
footer and cards use translucent surfaces and backdrop blur. `inline.css` contains
critical backdrop/loading geometry and fallbacks, so initial rendering does not
rely on a percentage height or the wasm app creating layout. Static and Yew
loaders share loading-panel/loading-spinner markup, accessible status text and
reduced-motion styling. No artificial progress or loading delay was introduced.
The old flashing square grid and brightness filter are removed from active use.

## Related

- Startup state transitions: [testing/startup.md](../../testing/startup.md).
- Shell behaviour coverage: [testing/shell.md](../../testing/shell.md).
- Database provider lifecycle: [browser-db.md](browser-db.md#provider-lifecycle).
- UI state/motion proposals: [UI_ARCHITECTURE.md](../UI_ARCHITECTURE.md).