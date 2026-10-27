# UI state and motion architecture proposal

Requested in the user's code review on 2026-10-27. This is a concrete proposal
for A31/A32, not a claim that the providers, reducers or transitions below exist.
Implementation plans still require approval. Product behaviour remains owned by
[PRODUCT](../product/PRODUCT.md), visual choices by [UX](../product/UX.md), and
implementation status by [CURRENT](../process/CURRENT.md) and [TASKS](../process/TASKS.md).

## Existing implementation and constraints

The user added `animato = { version = "1.7.3", features = ["path", "physics",
"color"] }` to web. Preserve that requirement and those features. The installed
1.7.3 README describes renderer-independent primitives and a separate `yew-csr`
integration feature. That feature is not enabled here. Do not assume Animato Yew
hooks are available, add another animation library, or enable extra features
without naming the change in an approved implementation plan.

ThemeProvider currently uses local state and context. DatabaseProvider uses
separate state handles for status and retry attempts; DatabaseContext exposes
the writable status handle. AppShell owns drawer expansion locally. Route pages
are mostly placeholders. There is no route-transition host, feature reducer,
motion-preference provider or Animato frame driver yet.

The existing persistent landscape, compact top panel, top-right theme control,
collapsible drawer and loading presentation remain requirements. Motion must
preserve their readability and accessibility. It must not change canonical URL
state, replay inputs, RNG draws, scoring or Fixed completion.

## Tailwind styling correction (D31)

On 2026-10-27 the user explicitly identified the drift away from Tailwind and
required correction. Tailwind is still installed/imported and built by Trunk;
the gap is how components are authored. main.css currently implements shell,
sidebar/top panel, cards, controls and responsive rules with bespoke selectors.
That is not the intended styling architecture and must not become the pattern
for new feature screens.

A35 migrates ordinary component styling into Tailwind utilities in Yew markup.
Keep shared light/dark tokens and consume them consistently; do not replace the
approved palette or translucent landscape surfaces. Retain only focused custom
CSS such as critical bootstrap rules and keyframes, and remove superseded rules
instead of maintaining two competing style systems. Avoid a blanket @apply
rewrite that merely conceals the same stylesheet-driven component architecture.

First inventory main.css, inline.css, component class strings and the legacy
tailwind.config.js. Verify the actual Tailwind 4 source/configuration path and
Rust/Yew class detection, then migrate one component group at a time. Use
complete literal utility variants so production output retains conditional
classes. Shared presentation belongs in small Yew components, not an unrelated
new CSS framework or an animation provider.

Acceptance preserves current desktop/mobile appearance, saved themes, original
background bytes, user's current branding edits, the 48px top panel, top-right
borderless theme control, drawer collapse, keyboard focus, reduced motion and
natural scrolling. Compare before/after screenshots and behaviour, including
bootstrap and DB Loading/Failed/Ready states at / and the /zipeek/ build. Update
e2e selectors to stable semantic/data hooks when a styling class is removed;
do not let test selectors dictate the stylesheet architecture. Run all five
code gates and Chrome coverage. No dependency upgrade, redesign or implementation
is performed by this documentation update.

## State ownership

| State | Authority and lifetime | Proposed access |
| --- | --- | --- |
| Theme | App lifetime; existing approved saved preference | ThemeProvider and a theme hook; persistence in its effect boundary |
| Motion preference | App lifetime; live prefers-reduced-motion value | MotionProvider publishes a small preference snapshot; no new saved setting proposed |
| Database client | One app-owned runtime session | Stable client context; consumers receive typed query operations |
| Database readiness | App lifetime, independent of routes | Status context and typed Retry command; reducer owns transitions |
| HSK, PoS, mode, count, seed, offset and URL exclusions | Current URL, normalized by A08 | Route adapter produces immutable typed inputs; no second writable global filter store |
| Saved and pending exclusions | Explicit user actions, approved D13 lifecycle | Exclusion provider and commands; merge only when generating a URL |
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

For database readiness, propose actions equivalent to Initialize, Retry,
Opened(request_id, client_metadata) and Failed(request_id, error). A reducer
rejects completions from superseded requests. The provider owns the actual
client handle and cancels/disposes stale initialization; UI state does not clone
or serialize JS handles. Publish readiness separately from the stable query
client so status or feature changes do not invalidate every consumer.

For a feature query, model Idle/Loading/Ready/Failed explicitly, with a request
identity derived from normalized inputs plus an attempt generation. A new query
invalidates the old generation. Use cancellation where supported and always
discard stale completions. Keep previous content during loading only when the
approved UX allows it; do not invent that interaction as an implementation detail.

Practice owns one session reducer, not separate counters in cards, summaries and
page components. Core round content and progress helpers remain authoritative.
Reducers receive resolved rounds and user actions; the effect boundary calls the
builder once for those normalized inputs. Feedback timing, answer controls and
summary details still depend on A22. Do not persist scores, progress or hash state.

Hooks such as use_dictionary, use_database_status, use_motion and
use_practice_session are proposed access points. Expose read-only snapshots and
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
Exact durations, easing/springs, displacement and color interpolation are not
selected here. A22 reviews that profile before A32 implementation. Bootstrap
continues to use its existing CSS reduced-motion treatment before Yew mounts.

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

A31 should establish provider/reducer boundaries first. A32 then adds the approved
motion profile through the adapter. Neither task changes database schema, routes,
replay format or dependency features without its specific approval.