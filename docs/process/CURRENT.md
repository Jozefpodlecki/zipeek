# Current state

Live state only: what is in flight, what is blocked, and what is ready next.
Historical validation narrative lives in
[completed/validation-log.md](completed/validation-log.md); per-task handoff
notes for the next session live in [handoff.md](handoff.md). The reviewed A18
scope and amendments were approved on 2026-10-26. D18 is fully approved in
[DB §D18](../architecture/schema/import-rules.md);
D20 MCQ scope is recorded in [DECISIONS](decisions/index.md),
[PRODUCT](../product/PRODUCT.md), and [UX](../product/UX.md). MCQ scope and D18
are no longer blockers.

## Active

- **A11 active (approved 2026-10-28):** Local pure setup reducer and selection
  controls for all six activities, HSK systems/levels and modes/counts; validated
  saved exclusions with explicit Reset; Start uses existing canonical URL/router
  encoding. No new dependencies or gameplay implementation. Five gates and
  root/Pages Chrome setup coverage pending.

- **Quiz-first priority (user-directed 2026-10-28):** Next implementation is A11
  setup, then A10 MCQ gameplay, then A09 matching/summary. First playable milestone:
  select activity/level → start Endless → answer → feedback → Continue. A21 already
  prepares the rounds; setup and ready gameplay views remain placeholders.
  Metadata/motion/extraction are later priorities, not quiz dependencies. No code
  was changed by this reorder. Required gates and full feature scopes remain.

- **A38 complete (2026-10-28):** Consumed 11 fully recorded scratchpad topics;
  retained six deferred/partial/unresolved sections with reason notes and original
  payloads. Repaired retained relative links. Documentation and preservation
  checks pass; no code, dependency, product/policy decision or index changes.
  [Outcome](completed/2026-10-28.md#a38-selective-scratchpad-cleanup).
  A41 tracks the unresolved contributor-policy questions. Next ready task: A11 under the later user-directed quiz-first order.

- **A37 complete (2026-10-28):** Traditional secondary exercise display,
  Browse fields/URL/ranges, Home failure layout and initial 150 ms fade-only
  profile are recorded. Final share-card design/copy is approved as D34; A40
  owns its implementation before v1. D33 accepts both build-tool advisories and
  retires A39. Documentation gates pass, with no code/dependency/scratchpad edits.
  [Outcome](completed/2026-10-28.md#a37-remaining-product-ux-and-initial-motion-decisions).
  A32/A11/A13 prerequisites are satisfied; A38 selective cleanup is complete; retained sections remain pending.

- **A16 complete (2026-10-28):** PR/push/manual validation now gates reusable
  master Pages publication, using Rust 1.97.1, locked web assets, all five gates
  and owned Chrome. Correct web paths, /zipeek/ release output, permissions and
  cache boundaries are implemented. Trunk HTTP readiness fixes the observed
  initial-404 race. Local gates pass: 102 Rust tests, thirteen root and four Pages
  scenarios, plus release/artifact and documentation checks. Hosted execution
  and actual publication remain unverified; no push/dispatch/deployment occurred.
  [Outcome](completed/2026-10-28.md#a16-ci-and-pages-workflow-migration),
  [evidence](../testing/evidence.md#a16-ci-pages-and-http-readiness--2026-10-28).
  A37 decisions are recorded; D33 retires the advisory repair proposal.

- **A21 complete (2026-10-28):** Six practice routes normalize dictionary IDs,
  fetch candidate views and prepare deterministic core rounds behind guarded
  Loading/Ready/Failed+Retry. Route titles/main focus and offline-independent 404
  recovery are implemented. Gameplay/Browse/Random/daily remain feature work.
  All five gates pass: 101 Rust tests, thirteen root and four Pages scenarios.
  Documentation/preservation checks pass; staged snapshot remains unchanged.
  [Outcome](completed/2026-10-28.md#a21-routing-and-practice-loading-integration),
  [evidence](../testing/evidence.md#a21-routing-and-practice-preparation--2026-10-28).
  A16 configuration is complete; A37 decisions are now recorded.

- **A31 complete (2026-10-28):** Approved theme/database state boundaries are
  implemented: pure reducers, separate read/command hooks, stable dictionary
  access and provider/boundary separation above route changes. Stale/duplicate
  results and old cleanup are guarded. All five gates pass with 100 Rust tests,
  including six new reducer/gate tests and twelve named root/Pages scenarios.
  Documentation gates pass; previous outcomes and staged snapshot are preserved.
  [Outcome](completed/2026-10-28.md#a31-scoped-ui-providers-and-reducers),
  [evidence](../testing/evidence.md#a31-scoped-state-boundaries--2026-10-28).
  A21 routing/loading-shell integration is now complete; A16 configuration is now complete.

- **A35 complete (2026-10-28):** Component utilities, explicit source detection,
  shell cleanup and responsive defaults are delivered. Attribution wraps inside
  its item in the approved single-row footer. All five gates pass after the
  final focus-layer correction: 94 Rust tests, root/Pages styling coverage at six
  widths in both themes, and before/after metrics/screenshots. Dependency files,
  dictionary/landscape/HTML, critical inline CSS and source scratchpad unchanged.
  [Outcome](completed/2026-10-28.md#a35-tailwind-first-component-styling),
  [evidence](../testing/evidence.md#a35-tailwind-component-migration--2026-10-28).
  A31/A21 are now complete; A16 configuration is now complete. The user-staged snapshot
  is preserved.

- **A15 complete (2026-10-28):** Approved modular harness and package rename
  delivered. All five gates pass; 94 workspace Rust tests include four lifecycle
  tests and two Chrome integration entry points with nine named product/adapter
  scenarios, plus two expected fault/recovery probes. No live owned Trunk/Node
  PIDs remain. [Outcome](completed/2026-10-28.md), [e2e guide](../testing/e2e.md).
  A35/A31/A21 are now complete; A16 configuration is now complete.

- **A22 scoped documentation pass complete:** Answered UX requirements are
  recorded, documentation gates pass and the [outcome](completed/2026-10-27.md)
  is appended. A37 records later choices; A38 has completed separate selective cleanup. No code, dependency or scratchpad changes were made.
  A15/A35/A31/A21 are now complete.

- **A36 / A35:** A36 recorded the Tailwind-first correction; A35 now implements
  it. main.css holds shared tokens/foundation rules and inline.css is the unchanged
  critical loading/backdrop boundary. Do not reintroduce bespoke component CSS.

- **A34 (documentation):** Documentation review of the 2026-10-27 architecture
  requests is complete. [testing/e2e.md](../testing/e2e.md) now includes the A15
  modular layout; [UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md) proposes
  scoped providers/reducers, Animato motion policy/adapter and a route host;
  [architecture/modules/target-architecture.md](../architecture/modules/target-architecture.md)
  proposes the A33 crate boundary. D28 is implemented by A15; D29/D30
  retain separate scope: A31 state is now implemented; A32 motion and D30
  extraction still require review. A34 itself changed documentation only.

## Blockers

- **A40:** Approved social-card image and metadata must be implemented before v1.
  A37 is complete; no image or HTML metadata has been implemented by that task.
  UX-SUM01 remains A09/A10 implementation work with the approved first-attempt
  metric; confirm the latest stable Chart.js pin in its implementation plan.

- **D33:** Both Picomatch advisories are accepted while glob patterns remain
  developer-controlled build input. No override or audit fix. A39's repair
  proposal is retired; this corrects the missed prior answer rather than claiming
  the dependency was patched. Reassess if the boundary changes.

## What is not a blocker

- **A06 is complete.** Exact npm DuckDB-WASM 1.32.0, local matching
  workers/WASM/Arrow assets, modular typed queries and app-owned
  Loading/Ready/Failed+Retry. Actual 12,100-row dictionary passes Node and
  Chrome read-only checks with engine v1.4.3 and unchanged checksum. All five
  gates pass: 88 domain/query/generator tests, seven standard Chrome tests and
  one separately run Pages-prefix test. See
  [architecture/modules/browser-db.md](../architecture/modules/browser-db.md)
  and [testing/browser-db.md](../testing/browser-db.md).

- **A19 is complete under D27.** Root `Cargo.lock` is staged for tracking, its
  ignore entry and obsolete web lockfile are removed, and Trunk enforces locked
  builds while watching shared core/workspace configuration. The graph is
  unchanged. All five gates pass and touching core triggered a rebuild.

- **A07 is complete.** Modular pure round builder for all six variants, with
  validated pools, single-stream recovery/selection/shuffling and first-gloss
  labels. The approved `rand` 0.10.3/chacha dependency moved from `web` to
  `zipeek-core`. 84 domain/generator tests and five Chrome tests pass. Eighteen
  full-round native/wasm vectors match digest `a94efad5496532a0`.

- **A08 is complete.** Modular dependency-free parsing, typed practice state,
  seed validation/recovery intent, canonical v1 encoding and offset/session
  helpers. Web uses existing router decoding without rewriting URLs. Actual
  seed recovery draws remain in A07's single-stream builder; no dependency
  changes.

- **A19 / D27** tracks the root `Cargo.lock`. Direct semver ranges remain ranges;
  approved dependency changes must update the graph deliberately. No exact
  Trunk version is selected; local 0.22.0 is tested.

## Other recorded follow-up

A35 corrected Tailwind drift. Future features use literal utilities and shared
@theme aliases; keep the critical bootstrap/loading exception focused. A35
preserves branding and landscape while implementing approved navigation/footer
cleanup and md/lg shell/landing rules. A31/A21 build on this baseline; A16 configuration is now complete. The
user-approved attribution wrapping resolves narrow footer fit; Browse columns
and future exercise layouts retain their separate UX/feature scope.

A15 replaced the smoke dumping ground with modular suites and page helpers.
Each root/Pages binary now owns one managed session and its server lifecycle.
Add future feature assertions to their named suite modules, not the thin test
entry point. Diagnostic artifacts live under target/e2e/<run>/<scenario>/.

Animato 1.7.3 with path/physics/color was user-added and is already in `web`.
A31 implements scoped theme/database read/command contexts and pure reducers.
No animation adapter is implemented. Animato Yew CSR is not enabled; do not use
that API or add features implicitly. A32 owns motion after profile review. Keep frame values local, honor reduced motion, and preserve URL/RNG
authority and the static landscape. See
[UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md) for lifecycle details.

A33 proposes `zipeek-db-wasm` for runtime/JS bridge, typed row conversion/private
SQL and errors. Keep core pure and generator native; keep Yew/provider state and
npm/Trunk asset deployment in `web`. It is not a new existing workspace member.
Approve name, exact manifest moves, shutdown/cloning and native-backend strategy
before extracting; native compilation and `/zipeek/` assets must stay verified.
See [decisions/process-and-infra.md](decisions/process-and-infra.md#d30-extract-browser-database-adapter-proposed).

[CODING_STANDARDS](CODING_STANDARDS.md) permits unreachable-branch comments and
declares lint-suppression rationales its sole comment exception. Resolve that
wording before adding exception comments; A26 introduced no Rust comments.
Local Rust/npm/wasm configuration already matches the declared targets; A19
verifies remaining configuration/pin-policy gaps without assuming dependency
approval. [BASELINE](BASELINE.md) remains unchanged historical evidence, not
current results.

## Next up

The user's D25 request supersedes D24's theme placement: dark/light belongs
at the top-right of a permanent top panel; the drawer is collapsible. Decorative
headers like "Chinese vocabulary" / "A fresh starting point" and landing
marketing filler have been removed. Do not restore them. Slate/blue palettes
remain. AppShell owns expanded state, initially true, and preserves it across
routes; reload resets it. Toggle uses `aria-controls`/`aria-expanded` and hidden
navigation. Top panel and theme remain accessible while collapsed. Desktop main
expands into the freed column; mobile uses inline expandable navigation under
the top panel. Natural scrolling is required when content genuinely exceeds the
viewport.

Latest D26/A30 requirement: keep the original landscape visible through loading
and app rendering in both themes. The fixed `.app-backdrop` is outside `#root`;
do not move it back inside Yew, hide it in light mode or mount the legacy
`Background` component on top. `main.css` shade/translucent surface variables
and `inline.css` critical layout compose it; image bytes are unchanged. Bootstrap
and route `Loader` share a small centered spinner/status panel. Reduced motion
stops animation. This does not implement A06/A21 database readiness/retry; do not
claim it does.

A29 refinements still apply: no decorative card arrows; 48px top panel via the
shared `--topbar-height` variable; reduced footer padding; no resting theme-button
border. Retain hover/focus feedback. Do not reintroduce removed decorative
elements.

A27/A14/A28 UI work is complete. Shared shell/theme components remain modular.
A06 opens the dictionary and supplies typed queries; `DailyWordCard` still
shows "No word available" and A12 must connect its data/Random. A21 implements
practice query/state wiring; A06/A31 implement DB readiness/retry. Remaining
query surfaces belong with their feature tasks.
Generated dictionary/reports remain unchanged under `web/static/` and Pages
still copies them to `/zipeek/hanzi.duckdb` with notices. Do not hand-edit or
automatically regenerate database bytes.

**A37 decision review is complete.** A32 motion, A11 setup and A13 Browse now
have their A37 prerequisite satisfied. A40 owns approved D34 social metadata/image
implementation before v1, after the quiz-first tasks in the updated backlog. A38 selective
cleanup is complete; A41 tracks contributor-policy clarification. A33 extraction
retains its explicit scope/feature/dependency review. Feature work
uses the recorded requirements without changing replay or dependency targets.

CI configuration passes local gates and structural checks, but the first hosted
run must still confirm fresh runner setup, remote permissions/environment and
actual Pages publication after commit/push. No such run was performed. Trunk
latest remains the existing unpinned selection. See [CI](CI.md) for exact scope.

A22 records answered UX requirements only: daily fields, setup, match/header,
MCQ, Browse semantics/states, copy except traditional display, accessibility,
responsive policy and shell cleanup; D32 records pronunciation with the supplied
simplified-hanzi fallback assumption. No code or dependency policy changed.
A22 left the source scratchpad untouched. A38 later performed separately approved
selective cleanup after validation, retaining six deferred/partial/unresolved
sections. Never clear those retained entries without applying their answers.

Later A09/A10 summary handoff (approved by the user during A22, applied only
at implementation): label the metric "First-attempt correct pairs", optionally
"First-attempt" in a compact stat row. A pair that needed any retry contributes
nothing to that numerator; the denominator is total required pairs (rounds × 4
in match). Reconcile MCQ summary presentation during A10. A22 does not apply
UX-SUM01's restart/timer/chart/stat amendments. Confirm and exactly pin the latest
stable Chart.js when implementation is approved; the suggested 4.4.x/4.4.6
candidate is unconfirmed. Leave AGENTS §4 and package manifests unchanged.

No Git commit has been made. Preserve existing user changes and append-only
history. See [handoff.md](handoff.md) for per-task continuation notes and
[completed/validation-log.md](completed/validation-log.md) for the evidence
behind each completed task.
