# Current state

Live state only: what is in flight, what is blocked, and what is ready next.
Historical validation narrative lives in
[completed/validation-log.md](completed/validation-log.md); per-task handoff
notes for the next session live in [handoff.md](handoff.md). The reviewed A18
scope and amendments were approved on 2026-10-26. D18 is fully approved in
[DB §D18](../architecture/schema/DB.md#d18-source-comparison-and-serialization-approved);
D20 MCQ scope is recorded in [DECISIONS](decisions/index.md),
[PRODUCT](../product/PRODUCT.md), and [UX](../product/UX.md). MCQ scope and D18
are no longer blockers.

## Active

- **A36 / A35 (documentation → implementation):** A36 recorded the user's
  Tailwind-first correction (D31). Tailwind is still built/imported, but recent
  shell/card work drifted into bespoke `main.css` selectors. A35 is the open
  migration task with visual parity acceptance. No CSS/component changes were
  made by A36 itself.

- **A34 (documentation):** Documentation review of the 2026-10-27 architecture
  requests is complete. [testing/e2e.md](../testing/e2e.md) now includes the A15
  modular layout; [UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md) proposes
  scoped providers/reducers, Animato motion policy/adapter and a route host;
  [architecture/modules/target-architecture.md](../architecture/modules/target-architecture.md)
  proposes the A33 crate boundary. D28–D30 remain proposals needing
  implementation review. No code, crate, dependency, or feature changes were
  made for this documentation task.

## Blockers

- **A22:** 11 UX TBD markers remain in [UX](../product/UX.md). D24/D25 resolve
  shell/navigation direction and landing arrangement. Daily-card fields and
  broader responsive policy remain open. UX-MCQ01 covers only feedback timing,
  number-key hints, and answer-button styling, not MCQ scope.

- **A15:** Chrome landing smoke passes with a separately started, ready Trunk
  server. The existing harness still requires an external server on port 1420;
  owned Trunk lifecycle/readiness and the package rename to `e2e` remain open.
  See [testing/e2e.md](../testing/e2e.md) for the proposed split.

- **A16:** CI/deploy select Rust 1.97.0, while policy/manifests require 1.97.1;
  deployment still uses obsolete root paths and omits the Pages public URL.

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

Tailwind drift is an open correction (A35), not a completed styling migration.
Do not extend the bespoke shell/card selector system for new screens. A35
inventories styles/configuration, verifies Tailwind 4 detection of Yew classes,
migrates component utilities, retains shared tokens/minimal bootstrap/keyframe
CSS and removes superseded rules. Preserve the original landscape, current user
branding, compact shell and accessible interaction states. A15 supplies
comparison/test infrastructure; A35 precedes further motion and feature UI
expansion.

The user's architecture review identifies `e2e/tests/smoke.rs` as a dumping
ground; freeze further scenario growth there until A15 establishes
[testing/e2e.md](../testing/e2e.md)'s modular structure. Current tests still
create one Chrome session per case and require external servers. The proposed
single-entry suite runner must preserve D09's one session per binary, cleanup
and isolated named scenarios.

Animato 1.7.3 with path/physics/color was user-added and is already in `web`.
No animation adapter or reducer/provider refactor is implemented. Its Yew CSR
feature is not enabled; do not use that API or add features implicitly. A31 owns
scoped read/command contexts and pure reducers; A32 owns motion after profile
review. Keep frame values local, honor reduced motion, and preserve URL/RNG
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
shows "No word available" and A12 must connect its data/Random. A21 owns
remaining per-route query/state wiring; A06 implements actual DB readiness/retry.
Generated dictionary/reports remain unchanged under `web/static/` and Pages
still copies them to `/zipeek/hanzi.duckdb` with notices. Do not hand-edit or
automatically regenerate database bytes.

**Next ready task: A22** — resolve the 11 UX TBDs before A15 (harness) or
A35 (Tailwind correction) is needed for downstream feature UI. A15 is ready
without A22; A35 depends on A15. A16 depends on A15/A19/A21.

No Git commit has been made. Preserve existing user changes and append-only
history. See [handoff.md](handoff.md) for per-task continuation notes and
[completed/validation-log.md](completed/validation-log.md) for the evidence
behind each completed task.