# Archived source snapshot: architecture/ui-architecture.md

This source snapshot preserves historical descriptions, not current requirements. Use the maintained document at its original location for approved behavior.

## R497 Practice and Grammar workflow state boundaries

Sentence Transformation uses a keyed child component for each authored question and a single Yew reducer adapter over pure `zipeek-exercises::sentence_transformation::TransformationState` transitions. Tile order, typed answer, input method and submission share one authoritative state; all edit actions are rejected after Check, and each edit is accepted only while its answer method is active. The pattern autocomplete text is a draft query, not the saved selection: unknown or partial input retains the selected authored pattern, and only an exact accepted suggestion or the All patterns button changes the persisted pattern. Switching session IDs remounts the Practice session host using the route key.

The Practice history-completion effect guards its result with a scoped cancellation cell; expired history responses are ignored by the view. Storage-level completion idempotency remains separate. Expanded Grammar navigation first persists the next checkpoint and advances UI only on success; guided reveal/choice interactions are deliberately ephemeral after refresh, and the UI discloses that replay behavior. No browser-local record format or dependency has changed. The async generation effect is governed by its existing storage attempts and version checks, not by the history-result guard. Locked Rust, WASM and browser validation remain outstanding.

## R492 Grammar domain and lesson UI ownership

`zipeek-pedagogy::grammar` now owns all 100 Grammar course records (550 draft teaching entries), original guided questions, region plan, published slug mapping and region lookups. `zipeek-exercises::grammar_learning` owns pure guided-step sequencing and practice transition rules. In `zipeek-web`, `grammar_learning_flow.rs` adapts the pure practice state to the Yew reducer contract; lesson components own browser-local step/save errors and AI-example presentation. Both original and expanded lesson engines render their exit link, progress bar and title through `GrammarLessonScaffold`, keeping existing E2E data attributes on the outer lesson article. The engines still have different guided activities and independent draft assessment coverage; this is a boundary and shared-chrome consolidation, **not** one unified pedagogical engine. All published routes and localStorage formats remain unchanged. No new provider or dependency is introduced. Locked Rust/WASM and Chrome acceptance remain pending.

## R486 Grammar lesson route ownership

The five original Grammar lessons share one validated `/courses/grammar/:slug` route; the original five public URL spellings remain unchanged. `courses/curriculum.rs` owns the list of published authored lesson slugs and the reversible region lookup; the Grammar detail panel uses this list to build links and the route-backed lesson component resolves only known slugs. Yew component keys isolate lesson-local step, answer and AI-example state across route changes. Expanded `/courses/grammar/topic/:region` and independent assessment `/courses/grammar/assessment/:course` remain separate. No new provider or durable progress format was introduced.


## R485 Grammar expansion boundaries

The original five Grammar courses retain their authored guided banks, draft assessment sessions and registry validation. Regions 6-20 are separate, individually written Grammar teaching courses in `grammar_expansion.rs`, exposed by a typed route and a focused teaching/review page. `grammar_plan.rs` names all 100 regions without claiming that unpublished regions contain teaching material. Browser-local course checkpoint validation recognizes only actual authored IDs and bounded step values. The existing shared AI service handle is consulted by the optional Grammar example component; this widget owns its asynchronous request generation and result state, and does not touch scoring or mastery. No AI credentials or learner progress are included in example prompts. The Word Sets, dictionary and Vocabulary reducer boundaries are unaffected.
> **R484 Courses learning depth:** Pinyin retains six independent mosaic stages but now maps each stage to two modules. Lesson-local Yew state owns multiple-choice selection and feedback; `courses/progress.rs` owns strict browser-local practice checkpoints and validates curriculum slugs/score bounds. Grammar reuses its existing lesson and assessment sessions and writes only last lesson step and best draft score. Neither a scored written Pinyin check nor one draft Grammar question per point grants confirmed speaking/listening competence or course mosaic mastery. No new global provider or dependency.

> **R483 Courses mosaics:** Vocabulary and Grammar reuse a deterministic shared Voronoi clipping/curved-edge utility while retaining independent region layouts. Vocabulary keeps weighted HSK reading-count sites; Grammar uses a balanced 100-site distribution with the authored first course at the center. Curvature is computed symmetrically from each shared edge, with no SVG icon overlays. The Grammar cover is translucent for preview only and does not represent mastery. Tests for geometry and visual acceptance are pending execution.

> **R482 Courses overview:** A shared `CourseWorkspace` Yew component composes three track-owned fragments (mosaic, lower preview, selected-region details) and owns desktop height allocation, not curriculum data or state. Vocabulary retains its reducer/database provider and paginated Explorer; Grammar and Pinyin retain their local selection hooks. `AppShell` applies the viewport-bounded Courses shell to all three overview routes.

# UI state architecture and motion proposal

Requested in the user's code review on 2026-10-27. A31 implements the approved
theme/database state boundaries on 2026-10-28. Motion, route transitions and
future feature providers remain proposals for their implementation tasks. Product behaviour remains owned by
[PRODUCT](../../../product-ux/product.md), visual choices by [UX](../UX.md), and
implementation status by [CURRENT](../../../project-management/current.md) and [TASKS](../../../project-management/tasks.md).

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
expansion locally. Practice and Browse are now implemented at source level. Browse has scoped
search and Word Set feature reducers/providers (see the current section below);
the route-transition host, motion-preference provider and application-wide Animato frame driver
remain unimplemented proposals. R457 uses the approved renderer-independent Animato
`Tween<f32>` locally inside the Practice dragon component, with a scoped timer
and DOM mask update, not a new provider or Yew integration feature.

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
focus, collapse and reduced motion. See [evidence](../../../testing-quality/e2e/evidence.md).

## State ownership

Component ownership, state boundaries and composition rules are maintained separately. See [Ui State Ownership](../../../architecture/ui-state-ownership.md) for the full maintained section.

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
The shared summary applies D35/D39 matching and MCQ accounting, with D40 weighted
requirements for mixed sessions.
A37 selects the initial D29 fade profile below; A32 implementation remains
separate. Do not persist scores, progress or hash state.

use_dictionary, use_database_status, use_database_commands, use_theme and
use_theme_commands are implemented access points.
use_practice_session is now implemented by A09/A48 at Practice's local boundary.
One SessionState owner delegates active-round behavior to pure matching/MCQ
reducers and retains shared FixedProgress, counters and frozen timing across
expected type changes. The earlier separate MCQ hook is replaced by immutable
family projections and named commands. use_motion remains separate A32 work.
Expose read-only snapshots and
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

## A22 practice state inputs  -  2026-10-27

The proposed practice reducer needs in-session current streak, cumulative correct
count and per-round matched-pair count for UX-MATCH02. Reset streak on the first
wrong attempt/answer in a round; show a flame at streak ≥ 3. Fixed progress remains
separate from offsets. These are state requirements for A31/features, not existing
reducer fields. Summary metric application remains A09/A10 work; animation frames
must not determine scoring. Preserve URL/RNG authority and no score persistence.

## A31 implemented state boundary  -  2026-10-28

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

## A21 practice effect boundary  -  2026-10-28

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

The later A09/A48 implementation uses the shared practice session described
above. Its matching/MCQ reducers own the actual interaction fields; session
metadata carries selected types while core replay inputs remain authoritative.

Setup inputs are page-local until Start, not a second global URL authority.
reducers/setup.rs owns typed activity/HSK/mode/count transitions. Children get
values and semantic callbacks; no raw reducer handles or storage access. The
core query_pairs and router serialization format round fields; D40 adds the
optional types metadata without changing canonical round bytes or v1 replay.
No empty context/provider framework was added for a single setup consumer.

use_saved_exclusions keeps Loading/Ready/Failed separate from the selection
reducer. Retry synchronously invalidates old generations before effect cleanup;
cleanup aborts further Rust orchestration and JS statements finish their own
cleanup. Reset is the sole storage mutation, invalidates pending work and keeps
storage/count unchanged if removal fails. There is no mid-round exclusion UI or
saved-ID writer on practice URL loads.

## Browse feature state (2026-11-04, R41 source refactor)

Browse state and pagination ownership are maintained separately. See [Ui Browse State](../../../architecture/ui-browse-state.md) for the full maintained section.

### Route composition and component imports (R43 - R45)

Moved to [Ui Browse State](../../../architecture/ui-browse-state.md).

### Browse results component boundaries (2026-11-04)

Moved to [Ui Browse State](../../../architecture/ui-browse-state.md).

## Browse error ownership and pagination invariant (R46 - R48)

`BrowseResults` derives whether retry and regeneration guidance may be shown from
`BrowseState.blocked`, then passes explicit boolean props to `BrowseError`.
The presentational error component must not infer control availability from
localized or variable error-message wording. Reducer actions remain responsible
for blocking, canceling and resubmitting queries. The Browse reducer exports
`PAGE_SIZE`; the search form and results container use that one constant for
query limits, visible counts and lookahead navigation.

D45 now approves the hybrid Browse target: URL-authoritative query/filter/page
state and removable filter chips, presented through the modern search-bar,
result-card and responsive Word Set panel layout. The former mandatory desktop
table and smart-combobox requirements are superseded. The current reducer and
provider do **not** yet synchronize Browse URL state or render canonical chips;
implementation and Chrome coverage remain A13 work. Keep URL parsing and
serialization in a tested boundary, reducers pure, and router/history side
effects in the provider rather than presentational components.

## Required provider contracts (R239)

The root `App` mounts `ThemeProvider` and `DatabaseProvider` above the router and
application shell. Their contexts are application composition requirements, not
optional feature flags. The database hooks in `hooks/database.rs` and theme hooks
in `hooks/theme.rs` therefore return concrete context values and fail with an
actionable provider error if mounted outside the required tree. `use_app_environment`
retains the same required contract.

Dictionary readiness remains a separate runtime state in `DatabaseStatus`.
Consumers handle database initialization and failures through that state or
through `Dictionary::database()` errors, rather than masking missing providers.
`DatabaseBoundary` still renders loading, ready and failed states with retry.

This refactor does not create feature-global contexts or modify the database
schema, persistence, routes or appearance settings. Standalone mounts and tests
must include the actual providers. Contexts that are intentionally optional in
other feature scopes remain unchanged.

## Practice setup launch ownership

`PracticeSetupFlow` is the local reducer for the configured exercise path, format selection, vocabulary filters, stage navigation, and launch error flag. `PracticeSetup` coordinates the navigator, required application providers, saved-word-set snapshots, and planned-session handoff. It does not own the dictionary query contract for each destination exercise.

`services/practice_launch.rs` constructs destination-specific vocabulary parameters, including the shared Character Hunt and Picture Vocabulary standalone query. Dictionary selections are encoded as source filters, while saved Word Sets are validated and converted to word IDs. The helper reports invalid references and empty saved sets instead of silently converting them into an empty query. Deep-link selection is validated at the route boundary, before the setup reducer receives it.

The database remains browser-local, and the reducer remains free of navigation and storage side effects. Future changes should test launch preparation independently from route transitions and keep callbacks explicitly owned by their Yew components.

## Browser test contract and display state

Yew components can expose narrowly scoped `data-testid` attributes for stable
E2E identification of controls without exposing puzzle answers or full session
state. Crossword setup, New game, score and replay use those selectors. Tests
should derive visible results through WebDriver and await observable state
transitions. The E2E page objects provide feature-level actions rather than
repeating selector and focus logic across independent scenarios.

## Unified preferences and background assets (R245)

`zipeek-web/src/preferences.rs` defines the single typed browser JSON record
at `zipeek.preferences.v1`. `PreferencesStore` receives a `Storage` reference at
construction and owns reading, initialization and field-preserving updates.
Display preset, light/dark mode, density, font, matching audio, image-server URL,
and AI endpoint/model use the record. The optional AI key remains in tab memory.
Rust/WASM startup restores display preferences before Yew mounts. The HTML
background defaults to Lavender until WASM initialization completes.

This JSON does not contain learning records. Word Sets, progress, exclusions,
and saved sessions retain their own storage. Welcome entry initializes the
preferences document; there is no separate welcome marker. No migration,
legacy key cleanup or compatibility reader is performed.

`assets/background-sources/` is the non-deployed source directory. A task in
`cargo make install` invokes the Rust web-asset preparation pipeline,
validates the bundled optimized WebPs under `zipeek-web/public/backgrounds/`, and injects the
Lavender WebP data URL in `zipeek-web/index.html`. A static early script applies
saved preset and display-mode attributes before the WASM UI initializes.

## R256 saved-vocabulary identity ownership

Word Sets are stored in browser-local JSON as `UserWordSet { id, name, word_ids }`. `WordSetId` is an opaque typed identifier with validated lowercase hexadecimal encoding. `WordSetsProvider` owns changes, validates before persisting and reconciles Browse state. The reducer retains selected IDs across reorder and import, rather than storing a positional array index. Browse, Practice and Write Hanzi consume snapshots from this same identity contract. Old positional URLs and old saved data are intentionally unsupported under the development-phase reset policy.

## R256 DuckDB cancellation ownership

The main browser database worker is created under a temporary guard until it is transferred to a live `DbSession`. Dropping an unfinished initialization terminates its worker immediately and attempts best-effort asynchronous termination of the DuckDB instance. Prepared statements close on success and failure, with both errors preserved when a query and its cleanup fail. These invariants require browser-level failure-injection validation and do not imply guaranteed asynchronous cancellation completion.

## R458 Practice creature asset selection

`CreatureProgress` consumes the existing `Appearance` context directly. A pure mapping from `Preset` to a packaged WebP path and creature identifier selects artwork without introducing a new provider or persisted preference. The local `animato::Tween` updates the single bottom-up mask on saved-session progress changes; both subdued and illuminated images use the same theme-selected asset. CSS reduced-motion rules display the authoritative target immediately. Asset preflight verifies every source image.

## R461 creature orientation and asset contract

`CreatureProgress` maps the existing appearance preset to a stable progress asset and portrait or landscape orientation. The session page uses the same orientation to place a portrait rail beside the exercise or a landscape rail below the exercise. Both share the existing Animato tween, semantic progressbar, and saved completion index; CSS changes only reveal direction. `static/practice` is an authored source folder protected by native unpack and copied by Trunk. Preflight requires VP8X WebPs with alpha and appropriate aspect ratios. No new user settings, persistence fields, or parallel animation provider are introduced.

## Vocabulary course page boundaries (R477 WIP)

`courses/vocabulary.rs` coordinates region selection, asynchronous dictionary loading and the Word Sets command callback. Its `vocabulary/data.rs` helper resolves pinned `(Hanzi, numeric pinyin)` reading identities from batched dictionary results. `mosaic.rs` owns the SVG selection UI and leaves geometry generation in `vocabulary_geometry.rs`. `region_details.rs` reads the shared Word Sets provider for acquisition state and renders region metadata. `word_explorer.rs` displays paginated vocabulary cards from the same loaded result, without a second dictionary query. Source index validation and candidate-first acquisition commit remain in `services/word_sets.rs`. The shared Word Sets provider and reducer continue to own browser-local mutation and persistence.

## R478 Vocabulary page state

`pages/learning/courses/vocabulary/state.rs` owns a feature-scoped Yew reducer for region selection, retry generation, dictionary load status and Explorer pagination. `vocabulary.rs` owns dictionary effects with cancellation and a four-region bounded in-memory word cache; its presentational children receive typed loading state and callbacks. `word_explorer.rs` does not issue dictionary queries. `region_details.rs` obtains acquisition counts from the shared Word Sets provider and displays only source-scoped acquisition feedback. The Word Sets provider continues to own local persistence and writes. No new crate, dependency or personal-storage format is introduced.
