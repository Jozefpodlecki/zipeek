# UI state architecture and motion proposal

Requested in the user's code review on 2026-10-27. A31 implements the approved
theme/database state boundaries on 2026-10-28. Motion, route transitions and
future feature providers remain proposals for their implementation tasks. Product behaviour remains owned by
[PRODUCT](../product/PRODUCT.md), visual choices by [UX](../product/UX.md), and
implementation status by [CURRENT](../process/CURRENT.md) and [TASKS](../process/TASKS.md).

## Existing implementation and constraints

The user added `animato = { version = "1.7.3", features = ["path", "physics",
"color"] }` to web. Preserve that requirement and those features. The installed
1.7.3 README describes renderer-independent primitives and a separate `yew-csr`
integration feature. That feature is not enabled here. Do not assume Animato Yew
hooks are available, add another animation library, or enable extra features
without naming the change in an approved implementation plan.

ThemeProvider and DatabaseProvider now use pure reducers and separate read and
command contexts. DatabaseProvider owns the session above HashRouter;
DatabaseBoundary renders readiness inside AppShell. AppShell still owns drawer
expansion locally. Route pages are mostly placeholders. There is no practice or
Browse reducer, route-transition host, motion-preference provider or Animato
frame driver yet. Add feature providers with their first concrete consumers.

The existing persistent landscape, compact top panel, top-right theme control,
collapsible drawer and loading presentation remain requirements. Motion must
preserve their readability and accessibility. It must not change canonical URL
state, replay inputs, RNG draws, scoring or Fixed completion.

## Tailwind styling implementation (D31 / A35)

A35 completed the approved correction on 2026-10-28. Component layout, spacing,
typography, borders, responsive rules and hover states now live as complete
Tailwind utility strings in mounted Yew components. main.css retains the shared
light/dark tokens, their @theme inline colour aliases, font aliases and minimal
body/focus/selection defaults. Critical loading/backdrop/error fallback rules and
loading keyframes stay in unchanged inline.css; static HTML and Yew Loader still
share that pre-WASM presentation.

Tailwind 4 source configuration is explicit:

```css
@import "tailwindcss" source(none);
@source "../src";
```

The source path is relative to web/styles/main.css and scans Rust/Yew files.
Conditional classes remain complete literal tokens. The production build contains
both drawer grid variants and responsive/interaction utilities; Chrome checks their
computed behaviour. The unused v3 tailwind.config.js was removed; its Roboto and
Oswald font-family declarations move to @theme font aliases. There is no blanket
@apply component layer or duplicate bespoke shell/card stylesheet.

This follows Tailwind's [source detection](https://tailwindcss.com/docs/detecting-classes-in-source-files)
and [theme-variable](https://tailwindcss.com/docs/theme) configuration. No package,
feature or version changes were required.

Shell changes apply the answered UX: label-only navigation with aria-current,
repository icon in a dot-separated footer row, and full-width content without a
max-width cap. The user approved attribution wrapping inside its footer item on
2026-10-28; the row itself does not wrap. The top panel stretches across mobile
viewports; md 48rem is the rail boundary and lg 64rem starts three landing tiles.
The existing branding/image, 48px panel, themes, drawer persistence, keyboard focus
and natural scrolling remain. Light muted text uses slate #475569 for conservative
contrast over translucent surfaces; the other palette tokens remain unchanged.

RoutePlaceholder is a small presentational leaf for the existing placeholder and
failure views. It does not introduce feature state or implement A31/A21.
Reduced-motion tile movement is guarded with motion-safe:hover, avoiding hover
specificity overriding a reduced-motion reset. Semantic/data e2e hooks replace
removed styling-class selectors. Before/after screenshots and metrics cover
400/640/768/1024/1280/1920px in both themes, root and Pages, plus loading/failure,
focus, collapse and reduced motion. See [evidence](../testing/evidence.md).

## State ownership

| State | Authority and lifetime | Access and status |
| --- | --- | --- |
| Theme | App lifetime; existing approved saved preference | Implemented ThemeProvider, read/command hooks and persistence effect |
| Motion preference | App lifetime; live prefers-reduced-motion value | MotionProvider publishes a small preference snapshot; no new saved setting proposed |
| Database client | One app-owned runtime session | Implemented stable Dictionary context; typed read-only Database facade |
| Database readiness | App lifetime, independent of routes | Implemented status context, typed Retry command and pure reducer |
| HSK, PoS, mode, count, seed, offset and URL exclusions | Current URL, normalized by A08 | Route adapter produces immutable typed inputs; no second writable global filter store |
| Saved and pending exclusions | Explicit user actions, approved D13 lifecycle | A11 local setup hook validates saved IDs and exposes Retry/Reset; generation only, never union into loaded URLs. Shared feature commands can follow their first consumers. |
| Browse request/results/pagination | Browse feature scope | Feature reducer plus cancellable query effect; URL authority follows approved UX |
| Practice round, guesses, mistakes and progress | Practice session scope | Feature reducer; A07 builds round data, core FixedProgress tracks completion |
| Drawer expansion | AppShell; current reload-reset behaviour | Keep local unless another component actually needs shared commands |
| Hover, temporary focus and individual animated values | Owning component/controller | Local state/refs; never broadcast every frame through app context |

Do not create one application-wide reducer containing every result list, input
and animation tick. A provider belongs at the nearest shared ownership boundary.
Use props for leaf inputs such as a card label, selected state or click callback.
Components that need a shared service or feature state consume its hook directly;
intermediate layout components must not relay those values through several levels.

## Proposed composition

```text
App
  ThemeProvider
    MotionProvider
      DatabaseProvider
        Router
          AppShell
            DatabaseBoundary
              RouteTransitionHost
                FeatureProvider, where shared feature state is needed
                  Page container
                    Presentational components
```

This is a target ownership tree, not an instruction to wrap every page in every
provider. DatabaseProvider owns the session above route changes; DatabaseBoundary
selects Loading, Failed/Retry or the outlet. AppShell and the original backdrop
stay mounted, so a navigation animation cannot reopen DuckDB or interrupt theme
controls. The provider/boundary split also separates service ownership from UI.

Page containers read normalized route inputs and initiate effects. Feature hooks
expose snapshots and named commands. Presentational components render immutable
view models and emit semantic actions; they do not query SQL or access storage.
Suggested web directories are contexts/, providers/, hooks/, features/,
motion/ and components/. Introduce a directory only when its first concrete
module is needed; do not create empty layers or a generic framework.

Reducers, effects and commands
Reducers are synchronous, pure state transitions. They never fetch, access the
DOM/localStorage, read a clock or advance an RNG. Effects/controllers execute
commands and dispatch typed results. Pass measured time into an action where
the approved score/timing behaviour needs it; animation frames cannot determine
practice progress or round completion.

Database readiness starts Loading at generation zero. Retry(request_id),
Opened(request_id, metadata) and Failed(request_id, error) are internal reducer
actions. Only the current Loading request can settle; stale and duplicate
completions are ignored. The provider owns the handle separately. Retry
synchronously advances its request gate and releases the previous handle before
dispatch, closing the gap before effect cleanup. Cleanup invalidates only its
own generation, so an old cleanup cannot cancel a newer attempt.

The JS open API has no cancellation operation. Keep its Rust future alive until
it settles; dropping that future early could abandon the returned JS session.
A stale success is dropped without publishing it, invoking existing last-handle
async connection/worker cleanup. This is logical cancellation and eventual
disposal, not awaited shutdown or transport-level cancellation. A33 retains the
separate shutdown/backend review. Dictionary access is published independently
from readiness and Retry; metadata state contains no JS handles.

For a feature query, model Idle/Loading/Ready/Failed explicitly, with a request
identity derived from normalized inputs plus an attempt generation. A new query
invalidates the old generation. Use cancellation where supported and always
discard stale completions. Keep previous content during loading only when the
approved UX allows it; do not invent that interaction as an implementation detail.

Practice owns one session reducer, not separate counters in cards, summaries and
page components. Core round content and progress helpers remain authoritative.
Reducers receive resolved rounds and user actions; the effect boundary calls the
builder once for those normalized inputs. A22 records match/MCQ feedback and
controls in UX; implement those approved interactions in the feature tasks.
Summary application is deferred to A09/A10, while remaining product choices and
A37 selects the initial D29 fade profile below; A32 implementation remains
separate. Do not persist scores, progress or hash state.

use_dictionary, use_database_status, use_database_commands, use_theme and
use_theme_commands are implemented access points. use_motion and
use_practice_session remain proposed. Expose read-only snapshots and
named commands rather than raw UseStateHandle setters or unrestricted dispatch.
Handle a missing required provider deliberately; do not unwrap context in render.
Use small context values and stable Rc handles. Split contexts when their update
frequency and consumer sets differ; do not add an unapproved selector library.

Motion ownership and Animato integration
MotionProvider owns preferences and shared policy, not per-frame values. A small
web/motion adapter should drive the existing Animato primitives from the browser
frame clock, apply values to the owning component, and cancel callbacks on drop.
Verify the browser features needed by that adapter before implementation; any
dependency/feature additions need explicit approval. No Yew-specific Animato API
or additional feature is assumed by this proposal.

Prefer browser CSS for simple hover/focus changes. Use Animato where coordinated
or interruptible value animation benefits the interaction: route presence, drawer
geometry or feature feedback after its UX is approved. Path, physics and color
features are available tools, not a requirement to animate every surface.
Do not animate the landscape, delay readiness, invent loading progress or make
the theme switch move around the shell.

Each active controller owns its frame subscription and values. Stop requesting
frames when idle. Keep those updates local; avoid app-wide reducer/context
dispatch on every frame. Browser DOM writes remain in the adapter/effect layer.
Prefer transform/opacity where suitable; height/layout or color interpolation
needs measurement and a concrete reason. Do not create simultaneous CSS and
Animato writers for the same property.

Route-transition lifecycle
Propose an outlet host inside the stable shell, with explicit Idle, Exiting and
Entering phases and a navigation generation. The router/URL remains navigation
authority. The transition host tracks only displayed/pending views; it must not
create a second history stack or rewrite query values.

Use one active feature page at a time initially: exit, unmount, then mount/enter
the latest destination. Keep the old page noninteractive during exit and cancel
its queries; do not keep two live practice providers running for a crossfade.
Late animation or query completions must match the current generation. Rapid
A-to-B-to-C navigation commits C, never a late B completion. Route queries and
pagination changes need feature-specific treatment rather than automatically
replaying the entire page transition. Those visual choices need review.

Focus transfer follows committed navigation, independently of animation events.
Acceptance requires focus to move to the new main content heading (or main)
and document.title to distinguish the new route.
Maintain the skip link, keyboard controls and meaningful heading focus. An
outgoing page cannot remain keyboard-accessible or exposed as a second active
page to assistive technology. Back/Forward and direct URLs use the same host.

Reduced motion commits the destination directly and uses stable final values;
it cannot depend on an animationend callback that never fires. Respond to live
preference changes. Finish/cancel active transitions when the document becomes
hidden so a paused frame clock cannot trap navigation or keep resources alive.
Essential semantic changes happen even when animation is disabled or fails.

Proposed motion policy and acceptance
Use shared named motion purposes rather than durations scattered across pages:
control feedback, drawer movement, route exit/enter and practice feedback.
A37 selects 150 ms fade-only route and drawer transitions, without slide or
scale and without whole-route animation for query/pagination changes. Preserve
existing practice feedback timings. A32 reviews easing/phase allocation within
that profile and implements interruption, focus and live reduced-motion behavior.
A22 did not select this profile. Bootstrap keeps its CSS reduced-motion treatment.

Acceptance for A31/A32:

Native reducer tests cover stale results, retry and navigation generations;
existing canonical round golden vectors remain unchanged.

Context consumers cannot mutate another feature's state through raw setters;
unrelated feature changes do not recreate the database client.

Chrome covers rapid navigation, Back/Forward, failed queries, reduced-motion
changes, focus and drawer/theme controls during transitions.

No duplicate interactive page, hidden focused control, leaked frame callback
or second database initialization appears during navigation.

Screenshot setup can settle motion through its preference/idle boundary;
tests wait on observable states rather than a fixed animation sleep.

A31 establishes the current provider/reducer boundaries. A32 can add motion
through the adapter after its profile and implementation scope are approved. Neither task changes database schema, routes,
replay format or dependency features without its specific approval.

## A22 practice state inputs — 2026-10-27

The proposed practice reducer needs in-session current streak, cumulative correct
count and per-round matched-pair count for UX-MATCH02. Reset streak on the first
wrong attempt/answer in a round; show a flame at streak ≥ 3. Fixed progress remains
separate from offsets. These are state requirements for A31/features, not existing
reducer fields. Summary metric application remains A09/A10 work; animation frames
must not determine scoring. Preserve URL/RNG authority and no score persistence.

## A31 implemented state boundary — 2026-10-28

A31 composition was ThemeProvider → DatabaseProvider → HashRouter → AppShell
→ DatabaseBoundary → Switch. A21 places Switch before the boundary so recognized
routes wait for data while 404 renders independently of database readiness. The proposed MotionProvider/RouteTransitionHost
above are not mounted. Provider ownership and readiness presentation are separate.

contexts/database.rs publishes immutable DatabaseStatus, a pointer-stable
Dictionary reader and DatabaseCommands::retry. Dictionary::database returns a
cloned typed query facade or DbError::NotReady; it exposes no slot setter, raw
SQL or worker. A consumer that needs readiness subscribes to the status hook;
the stable dictionary context alone does not signal readiness changes. Retained
query handles may finish existing work; feature tasks must reject stale results.

reducers/database.rs and reducers/theme.rs hold synchronous pure transitions.
ThemeCommands::toggle dispatches against current reducer state, including rapid
successive commands. ThemeProvider loads preference once and owns DOM/storage
effects. Initial loading does not write a default preference; a user toggle marks
state for persistence. Existing dark default and storage failure handling remain.
No context exposes a reducer handle or arbitrary dispatch. Missing providers are
handled explicitly by mounted consumers and logged; hooks return Option.

Six native tests cover superseded/duplicate initialization, retry after failure,
unmount invalidation/unavailable client access and rapid theme commands. Chrome
holds the dictionary download across navigation and theme changes, then checks
Back/Forward and rapid route changes without another download or worker. Existing
failure/Retry, theme persistence/storage failure and root/Pages checks remain.
Practice streak/cumulative-correct/progress and Browse query state remain with
A09/A10/A13; URL inputs, exclusions and build_round stay authoritative.

## A21 practice effect boundary — 2026-10-28

use_prepared_practice consumes the stable dictionary hook directly. Request
identity contains variant, parsed URL query and retry generation. The reducer
accepts one completion for the current Loading request. Render masks an older
snapshot as Loading as soon as inputs change; an active-request ref also rejects
late completion before effect cleanup. Retry invalidates that ref synchronously.
Effect cleanup aborts the Rust query orchestration and invalidates its generation.
An already-issued JS query can finish and close its statement; this is not a
DuckDB query-cancellation API and never aborts Database::open.

The service validates referenced IDs in one bound ordered lookup, calls core
normalization, queries the appropriate seed-derived or URL-derived candidate
views, and invokes build_round once. Recovery preloads ordered HSK-union rows
and SQL candidate views for their distinct HSK/PoS contexts; only the core builder
chooses recovery and consumes RNG. This rare path can issue many view queries;
cancellation stops scheduling further queries. Original seed/filter canonical
inputs remain unchanged, and the loaded URL is never rewritten. Fixed skips
seed lookup unless that ID independently occurs in exclusions. Loaded URLs never
read or write saved exclusions.

PreparedPractice retains normalized inputs and the complete core Round for later
feature consumers; no scoring/session counters are invented here. The page shows
an explicit coming-soon state after preparation, not an indefinite loading view.
Debug builds publish card ID/label diagnostics as data-practice-round for Chrome
replay assertions; release builds omit that attribute. Product copy does not
expose canonical inputs, SQL or worker details. A09/A10 own playable controls and
summary; A11 owns setup/reset; A13 and A12 own Browse/daily/Random queries.

## A37 presentation boundaries

Traditional secondary text must not enter core Card.label or answer comparison
keys. Feature containers resolve its Word metadata for the represented WordId,
then pass immutable secondary display text to the appropriate hanzi leaf. No
builder draw/order/canonical input changes are needed. A09/A10 own implementation.

Browse's A13 state codec follows UX's two-layer f encoding, one-based page and
page_size contract. Keep raw input validation separate from the normalized request;
loaded malformed URLs do not rewrite themselves. Result cards/table use the same
five display fields. The new URL grammar does not change A08 practice codecs.

Home's A12 failure layout needs a page-specific readiness boundary: show the
daily fallback/Retry alongside navigation tiles while data destinations keep
handled failures. The A21 whole-content boundary is current implementation,
not evidence that this newly approved Home behavior exists.

## A11 setup boundary

Setup inputs are page-local until Start, not a second global URL authority.
reducers/setup.rs owns typed activity/HSK/mode/count transitions. Children get
values and semantic callbacks; no raw reducer handles or storage access. The
existing core query_pairs and router serialization remain the sole URL format.
No empty context/provider framework was added for a single setup consumer.

use_saved_exclusions keeps Loading/Ready/Failed separate from the selection
reducer. Retry synchronously invalidates old generations before effect cleanup;
cleanup aborts further Rust orchestration and JS statements finish their own
cleanup. Reset is the sole storage mutation, invalidates pending work and keeps
storage/count unchanged if removal fails. There is no mid-round exclusion UI or
saved-ID writer on practice URL loads.
