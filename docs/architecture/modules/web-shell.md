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

`main.rs` mounts Yew into `#root` without replacing the backdrop. A31 places
`DatabaseProvider` above the router and shell; `DatabaseBoundary` inside
`AppShell` renders Loading/Ready/Failed and Retry. Navigation/theme stay usable
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

A06 implements this lifecycle; A31 separates its pure reducer, provider and
readiness boundary with scoped read/command hooks. The app
owns database readiness; individual data queries may still have their own loading
states after startup.

## Implemented web shell and landing foundation (A27 / D24)

User-selected direction: neutral slate light/dark with blue accents, desktop
left rail, compact mobile header and a daily-word card followed by three tiles.
The implementation is split into `AppShell`, `Navigation`, `ThemeProvider`,
`ThemeToggle`, `AppFooter`, `DailyWordCard` and `NavigationTile` components, one
per file. `theme.rs` owns preference policy/commands; ThemeProvider owns persistence and
DOM effects around reducers/theme.rs. `Home` composes the page.
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
daily queries, deterministic selection and Random practice. A06/A31 implement
actual database readiness; A21 now connects practice query loading and preparation.
Visual theme consistency alone does not implement gameplay. Static loading markup now respects the same
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

## A22 target shell and responsive requirements — 2026-10-27

The implemented shell described above still has navigation decoration, a drawer
repository link and 48rem/60rem CSS. A22 records requirements only; A35 implements
the shell correction with Chrome comparison coverage. Navigation becomes label-only
with aria-current="page" and no visual active decoration. The existing GitHub
icon moves to the one-row, dot-separated copyright/attribution footer, preserving
its accessible name, external-link attributes and licence notice.

Target Tailwind defaults: sm 40rem (small layout), md 48rem (desktop rail),
lg 64rem (wider content/tiles), xl 80rem (Browse table). These do not describe
current CSS. Minimum supported width is 400px, smaller widths best-effort; no
max-width container. Footer fit at 400px needs review against the no-overflow rule.
The mobile drawer must not trap users: retain an inline/nonmodal arrangement,
or contain focus in an overlay with an operable close action and focus return.
Committed route navigation focuses the new main/heading and updates page title.

## Planned pronunciation boundary (D32)

Keep speech interop in a small web service/effect boundary, outside pure core
and reducers. Construct SpeechSynthesisUtterance(text), set lang='zh-CN', and
invoke window.speechSynthesis.speak. The supplied fallback text is the represented
word's simplified hanzi, independent of the card label. A22 implements no service,
button or new feature. Voice availability and quality vary by OS/browser; tests
stub/spy the API text/language/call rather than listening for sound.
No new assets, DB columns or audio dependency is required by the specification.


## A35 current styling and shell — 2026-10-28

A35 supersedes the earlier bespoke-selector descriptions above. Shell, navigation,
theme controls, daily placeholder, landing tiles, footer and route placeholders
now use Tailwind utilities. main.css contains shared theme tokens/@theme aliases
and minimal foundation rules; unchanged inline.css remains the critical static/
Yew loader and backdrop boundary. @source explicitly scans web/src; no v3 config
or blanket @apply component system remains.

The original brand image/text and backdrop bytes are preserved. Navigation has
labels only and aria-current without active decoration. The repository icon/link
moves to the footer, preserving accessible name, target/rel and simple_icons.
The user approved attribution wrapping inside its item while keeping a single
horizontal copyright/attribution/repository row. Main has no max-width cap and
the mobile top panel fills the viewport. Rail layout begins at md 48rem; three
landing tiles begin at lg 64rem. Future Browse tables still belong to A13.

RoutePlaceholder centralizes the existing placeholder/failure presentation as a
leaf component, not a provider. Light muted slate is #475569 for contrast on
translucent surfaces. Existing 48px top panel, theme persistence, drawer lifetime,
loading/Retry, focus and reduced-motion behaviour remain; tile movement occurs
only under motion-safe. Chrome uses semantic/data hooks rather than styling-class
selectors and captures both root/Pages at 400/640/768/1024/1280/1920px.

## A21 route handoff

Switch composes DatabaseBoundary for each recognized route; 404 bypasses data
readiness while keeping AppShell/theme/navigation mounted. All three match and
three MCQ paths are declared. AppShell owns route-specific document titles and
main-content focus on route changes, including initial deep links and history
navigation. Query-only changes do not steal input focus. The fallback uses the
shared placeholder sizing and a safe Home router link without a navigator unwrap.
Practice now has its own query Loading/Ready/Failed+Retry after dictionary Ready;
its prepared view remains a coming-soon placeholder until gameplay is implemented.
Browse, setup, Random and the daily card retain their separate feature scope.

## Planned social metadata (D34 / A40)

Initial web/index.html metadata will use the approved site-wide title/description
and static preview. The 1200×630 composition reuses existing landscape/branding
with slate overlay and blue accent. Image creation is an explicit asset task,
not database generation. Use absolute /zipeek/ canonical/image URLs and keep
runtime document.title behavior separate. No preview image/tags exist from A37.
