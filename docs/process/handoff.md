# Next-session handoff

Per-task continuation notes moved out of the live [CURRENT](CURRENT.md) state.
These are reference prose for the next session or a new contributor picking up a
task; they are not live task state. Live work is in [CURRENT](CURRENT.md) and
[TASKS](TASKS.md); completed outcomes in [completed/](completed/).

Preserved verbatim from the former single-file `CURRENT.md`. Do not rewrite.

---

## A34 handoff

Implement A15 first using [testing/e2e.md](../testing/e2e.md)'s explicit
directory/ownership and cleanup plan; then review A31's scoped state plan before
expanding feature UI. A32 motion profile and A33 crate extraction remain separate
proposed scopes. Preserve current source files, staged changes and A06 test
evidence; doc-only validation did not rerun Rust/Trunk/Chrome. No implementation
completion is claimed.

## A19 handoff

Root `Cargo.lock` is authoritative and staged; only its new index entry was added.
Existing staged files were preserved. The stale `web/Cargo.lock` is deleted,
`.gitignore` no longer ignores the root lock, and `Trunk.toml` sets `locked=true`
to prevent silent resolution changes. Watch paths include `web`, shared core and
root `Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`. Shared-core watch proof
passed without changing bytes; original timestamp restored after owned-server
shutdown. All five gates pass with locked resolution; log
`target/a19-workspace-tests.log`.

Tailwind 4.3.3 npm packages are absent locally, but Trunk's managed CLI works;
global wasm-bindgen is absent from PATH, but its Trunk pin is 0.2.114 and build
passes. Do not install/upgrade tooling to correct these observations. Local
Trunk 0.22.0 is tested, not an approved exact pin. A16 still owns CI Rust 1.97.0
and obsolete root-path/Pages workflow fixes.

A19 scope checks: working diff and newly staged `Cargo.lock` pass whitespace
checks. The full pre-existing staged diff reports whitespace in fallback
`NOTICE.txt` and `hsk.json`, and static `CC-BY-SA-4.0.txt`/`HSK-SOURCE-README.txt`
notices. These protected fixture/source bytes were preserved; no normalization
was attempted. Details: `target/a19-existing-staged-whitespace.log`. This does
not fail a Rust/build gate.

## A06 handoff

Run `npm ci --ignore-scripts` from `web/` before Trunk. The `pre_build` hook
verifies package 1.32.0 and copies matching local MVP/EH workers/WASM, Arrow
bundle/adapter and licenses to ignored `web/vendor/duckdb`. HTML copies it; watch
ignores vendor to avoid loops. No CDN, automatic `npm install` or DB generation.

`bridge.js` resolves assets/dictionary via `document.baseURI` for `/` and
`/zipeek/`, opens a full fetched buffer `READ_ONLY`, prepares/binds statements
and closes them. `runtime.rs` owns async Rust handles/cleanup; `row.rs` validates
`WordId`s/levels/PoS without changing source display text; `queries.rs` provides
lookup, full-pool count/ordered offset/rows, pagination and the three clamped
HSK/PoS views.

`DatabaseProvider` inside `AppShell` starts Loading before effects, keeps
navigation usable, renders routes only after a typed probe, and exposes concise
failure plus Retry. `contexts/database.rs` supplies `DatabaseContext`/
`DatabaseState` for future route queries. The existing practice `Loader` is still
a feature placeholder; do not interpret it as DB readiness. No Browse/daily/
exercise UI was implemented.

Tests/logs: `target/a06-workspace-tests.log` (88 domain tests + seven Chrome
tests; one Pages test ignored there), `target/a06-pages-chrome.log` (that Pages
test passed separately). Node probe `target/a06-compat.cjs` also passed. Both
owned servers stop after validation. Dictionary and background bytes, Cargo graph
and replay remain unchanged. No commit/staging changes were made in A06.

`npm audit` reports existing tooling picomatch 2.3.1 with a high-severity
advisory (`target/a06-npm-audit.json`). It was already in the previous npm
lockfile; existing package records were preserved and Picomatch is not copied
into browser assets. A16 tracks review; dependency upgrades require specific
approval. Do not run `npm audit fix` as an unapproved follow-up to this task.

## A07 handoff

`core/round` exports `build_round`, `RoundCandidates`, `CandidatePool`,
`ContextPool`, `Round`/`RoundContent`, `Card`, `BuildTrace` and typed
`BuildError`. Modules separate orchestration, pool validation, labels, recovery,
selection and final layout. Pass normalized A08 state, ordered views and original
seed metadata. For Ready seeds, initial views use the seed's effective context.
For recovery, provide ordered exclusion-clean rows and `ContextPool` entries for
possible recovered contexts; missing views fail explicitly. Canonical input
retains the original URL seed/filter; never recover with a second caller-owned
RNG. A06 now supplies SQL ordering/exclusion binding and DB readiness; A21 must
wire those APIs to route normalization and the builder.

Matching reserves stay in trace only; visible columns have four cards, MCQ has
one prompt and three options. English cards use the first pipe-separated gloss.
No gameplay/score controls are connected to the placeholder pages yet.

`tests/fixtures/round-golden.txt` and the shared `replay_vectors` example lock 18
complete rounds; `tests/check-round-vectors.mjs` runs the wasm export with host
calls trapped. See [testing/round-builder.md](../testing/round-builder.md) for
exact build/run commands and why web participates in wasm feature unification.
All five gates and wasm replay pass. Latest log:
`target/a07-workspace-tests.log`; the owned server is stopped. No Git commit made.

## A08 handoff

`crates/zipeek-core/src/practice` separates types, query parsing, normalization,
canonical encoding and session state. `ParsedPracticeQuery` takes already-decoded
pairs. `normalize` takes an explicit `Word` lookup, drops unknown exclusion IDs
and returns `Unseeded`, `Ready` or `RecoveryRequired`. The latter keeps the
original bad seed/filter at offset zero for canonical hashing; A07 owns all RNG
initialization, recovery, selection and shuffling. Do not resolve seeds against
an empty placeholder database. The web adapter only parses today; A21 must
provide rows using A06 APIs and call normalization. `query_pairs` returns decoded
values for one normal router serialization; no additional percent-encoding.

Saved/pending exclusions are unioned only for explicit URL generation. Offset
helpers advance match by four and MCQ by one; `FixedProgress` is separate from
offset and resets in memory. Match pages still render `Loader`, not a game.
The published canonical golden remains `0xa0e531d76df62844`. Checks/logs are
recorded in [completed/](completed/) and `target/a08-workspace-tests.log`; no
owned server remains running.

## A06 asset packaging notes

`npm ci --ignore-scripts` from `web/` runs the pre-build hook that packages exact
DuckDB-WASM 1.32.0 and locked Arrow assets from `node_modules` into ignored
`vendor/`. It verifies the runtime version, copies matching workers/WASM and
licenses and does not fetch/install anything itself.

The npm tarball lacks DuckDB's MIT license file; the pinned upstream MIT text is
included alongside Arrow's shipped license/NOTICE. Do not substitute `latest`,
`next`, or development builds. If integration exposes a genuine wasm toolchain
compatibility blocker, stop and report commands, versions, and error evidence.
See [architecture/schema/DB.md](../architecture/schema/DB.md) and
[architecture/schema/runtime.md](../architecture/schema/runtime.md) for the pin
rationale.

## A15 handoff — 2026-10-28

A15 implements the package e2e, owned lifecycle, modular suites and explicit
adapter fixture. Use [testing/e2e.md](../testing/e2e.md) for current commands;
earlier handoff paragraphs about external servers are historical. The root suite
has eight named scenarios plus two expected-failure recovery probes; Pages has
one scenario. Each binary owns one managed Chrome session and an isolated build.
Four lifecycle unit tests and all five gates pass. Evidence and artifacts are
listed in [testing/evidence.md](../testing/evidence.md#a15-modular-chrome-harness--2026-10-28).

Next is A35: preserve current user branding, landscape and accessibility, apply
the recorded shell cleanup/responsive changes and migrate component styling to
Tailwind utilities. Use the new run/scenario directories for visual comparisons.
Keep the source scratchpad unchanged pending separately approved A38 cleanup;
remaining UX decisions stay in A37. D29 motion and D30 extraction remain separate.


## A35 handoff — 2026-10-28

A35 implements D31 utilities, explicit Rust source detection and theme aliases;
there is no legacy v3 config or bespoke shell/card stylesheet. Critical inline
loading/backdrop CSS, source scratchpad, dependency graph and dictionary/image
bytes remain unchanged. The approved shell cleanup is applied: label-only
navigation with aria-current, footer repository icon, full-width mobile panel/
main and md/lg boundaries. The user approved attribution wrapping inside a single
footer row. Light muted text was darkened for contrast; focus defaults use the
base layer and tile motion is guarded by motion-safe.

All five gates and root/Pages Chrome pass after the final focus-offset fix.
See [A35 evidence](../testing/evidence.md#a35-tailwind-component-migration--2026-10-28)
for before/after artifacts and contrast scope. A31 is next in the approved order:
review its concrete scoped read/command context and reducer plan before editing.
A21 follows. D29 motion profiles/A32, D30 extraction/A33, A37 remaining UX choices
and separately proposed A38 cleanup remain outside the completed styling task.
Do not clear the source scratchpad implicitly. No commit or staging was performed
by the agent; preserve the user's current staged snapshot.


## A31 handoff — 2026-10-28

A31 is complete. ThemeProvider → DatabaseProvider → HashRouter → AppShell →
DatabaseBoundary → Switch is the mounted tree. Use use_dictionary for stable
typed query access, use_database_status for readiness notifications and
use_database_commands for Retry. Theme uses use_theme/use_theme_commands. No
consumer receives raw reducer handles/setters. reducers/ holds pure metadata/theme
transitions; provider effects own DOM/storage/initialization and a synchronous
request gate rejects old results before cleanup. Pending JS opens cannot be
aborted: stale successful handles close after settlement. A33 shutdown review
remains separate. See the A31 architecture/evidence entries for details.

All five gates, 100 workspace tests and twelve named root/Pages Chrome scenarios
pass. The user-staged index, dependencies, scratchpad and previous outcomes are
preserved; no staging/commit was performed. A21 routing/loading-shell integration
is next in the approved order. Add concrete feature providers with their first
consumers; leave Browse/practice product choices, motion profiles/A32 and adapter
extraction/A33 to their own tasks. Never clear the scratchpad implicitly.


## A21 handoff — 2026-10-28

All six practice variant routes now use services/practice_loader.rs through
hooks/practice.rs and a scoped pure request reducer. The loader validates seed/
exclusion IDs, normalizes with core, obtains SQL-ordered candidate views and calls
build_round once. It never reads saved exclusions or rewrites loaded URLs.
Recovery preloads all possible context views; the core builder alone owns RNG.
Ready retains PreparedPractice/normalized query/Round but renders coming-soon
until A09/A10. Debug-only data-practice-round exposes card IDs/labels for replay
checks; release omits it. Failures log detail and expose safe Retry copy.
Cleanup aborts Rust orchestration, not the already-issued JS query; generation
checks and render masking prevent stale results. No Database::open abortion.

The app now mounts Switch inside the stable shell, with DatabaseBoundary per
recognized route and independent 404 recovery. Titles/main focus follow route
changes, including history; query-only changes preserve focus. MCQ variants share
Practice navigation selection. Five gates pass with 101 Rust tests and seventeen
named Chrome scenarios. See A21 evidence for paths and the corrected 404 fixture.

A16 CI/Pages migration is next proposed implementation; A37 decision review is
still open. A09/A10/A11/A13/A12 retain their feature behavior, and A32/A33 motion/
extraction need separate review. Preserve the user-staged index, append-only
history and untouched scratchpad. No staging or commit was performed.


## A16 handoff — 2026-10-28

CI now owns PR/master push/manual validation with the approved Rust 1.97.1,
locked web install, all five gates and workspace Chrome suites. Successful
non-PR master validation calls reusable deploy.yml for Ubuntu release build and
web/dist/ Pages upload/deploy. Manual publication starts from CI on master;
deploy.yml has only workflow_call. Existing action versions and Trunk latest
remain; no package/feature/tool pin was selected. See process/CI.md for permissions,
cache boundaries, source links and native/browser prerequisites.

Validation exposed an actual harness startup race: TCP could accept while Trunk
served 404. Bounded HTTP 200 readiness and a 404-then-200 fixture fix it; five
lifecycle tests now pass. All local gates pass with 102 Rust tests and seventeen
named Chrome scenarios, plus the release build and artifact/preservation checks.
See A16 evidence for final paths. Hosted execution and actual publication still
need confirmation after user commit/push; no remote action was performed here.

A37 remaining decisions is next proposed work. A39 tracks the reviewed Picomatch
advisory requiring separate dependency approval. A38 scratchpad cleanup and A32/
A33 motion/extraction retain their own scopes. Preserve the source scratchpad and
append-only history. The user staged work externally during interruption; use the
resumed snapshot as the preservation reference. No staging/commit was performed
by the agent.

## A37 continuation — 2026-10-28

Approved and recorded: secondary traditional on hanzi exercise labels without
comparison-key changes; five Browse fields; q/strategy/f/page/page_size grammar
with individually encoded filter tokens then outer query serialization; malformed
range rejection with existing HSK clamping; Home fallback/Retry plus tiles; 150 ms
fade-only route/drawer profile, preserving practice timings/reduced motion.
Existing D04/D13/D15/D20/theme rules settle most old product-gap entries.

The user requires social cards and a preview image before v1. The final proposed
share-card layout/copy question is pending; do not infer acceptance. No card/image
or metadata implementation has started. D33 accepts both tooling advisories under
the build-tool-only input condition; A39 repair was retired, with an appended
correction preserving A16 history. No override/audit fix or scratchpad edits.


## A37 final answer and closure — 2026-10-28

The share-card layout/copy is approved as D34, superseding the pending-design
state above. A37 is complete and its outcome appended; A40 now owns image/metadata
implementation before v1. Use UX's exact share copy and static site-wide 1200×630
landscape/slate/wordmark/blue composition, /zipeek/ URLs and no round word/score.
A32/A11/A13 have their A37 prerequisite satisfied. Scope/dependency approvals for
implementation still apply. Documentation gates pass; no image, HTML, code,
dependency, scratchpad or staged-index changes were made by this closing pass.


## A38 selective cleanup — 2026-10-28

The separately approved cleanup is complete: 11 fully recorded topics consumed,
six sections retained (summary/Chart.js, conservative copy/product remnants,
D30 extraction and two contributor-policy questions). Retained payloads remain
with one reason note each and repaired relative links. The traditional composite-
label branches are superseded by A37 secondary display with unchanged keys;
summary application/exact Chart.js pin remain A09/A10. Do not treat the old retained
question lists or proposed D-numbers as newly approved or reserved choices.

The completed outcome records each topic's disposition and permanent owner.
A41 tracks comment-exception/transitive-policy clarification without deciding it.
Documentation/preservation checks pass; no code/dependency or staged-index change,
Rust/Trunk/Chrome run, staging or commit. A40 is the next ready backlog item for
approved D34 social metadata/preview implementation; it still needs its concrete
implementation plan under the normal process.


## User-directed quiz-first order — 2026-10-28

This supersedes earlier next-A40 suggestions. Next implementation is A11 setup,
then A10 MCQ gameplay, then A09 matching/summary. First playable milestone is the
Endless MCQ question/feedback/Continue loop using A21 prepared rounds. Full task
scopes and validation remain; do not claim full A10 completion from that milestone.
Metadata/motion/extraction are later priorities, and A40 is still required before
v1. No code/dependency or scratchpad change was made by this reorder.


## A11 setup handoff — 2026-10-29

A11 is complete. Page-local SetupState reducer plus ActivityPicker/HskPicker/
SetupChoice implement all six activity paths, HSK levels and modes/counts. Initial
activity is Hanzi to meaning for quiz-first priority, HSK 3.0 level 1/Endless;
Fixed defaults to 10 and preserves selected count across toggles. Start uses core
query_pairs and existing router serialization, never a hand-built query string.

use_saved_exclusions validates JSON-array WordIds via words_by_ids, counts known
unique IDs, and exposes Retry/Reset without raw handles. Load never writes storage;
explicit Reset removes zipeek.excluded_words and updates the next Start URL while
remaining on setup. Failed removal keeps count and shows an error. Generation
checks and orchestration cancellation reject stale work. Shared practice URLs
continue to ignore saved exclusions. No shared exclusion provider is needed yet.

All five gates pass with 105 Rust tests and twenty-one named Chrome scenarios.
Surface correction uses bg-surface; do not reintroduce undefined bg-card. See
A11 evidence for final screenshots/logs. Completion is in 2026-10-29.md; earlier
date files are frozen. Work was committed externally during interruption; preserve
that commit and existing index. No agent staging/commit was performed.

Next quiz-first task: A10 question/answer/feedback/Continue, starting with the
Endless MCQ loop using A21's PreparedPractice. Full Fixed requirements remain;
Chart.js/summary pin work must not block the first playable Endless milestone.


## A10 Endless milestone handoff ? 2026-10-29

The quiz is playable in all three Endless MCQ variants from setup. A10 remains
open for Fixed gameplay/progress/completion and summary. Keep this milestone
separate from full A10 completion; do not remove its TASKS row yet.

McqPractice stays mounted through query loading/error/retry. use_mcq_session wraps
one pure McqState reducer; no handles/setters escape. Resolve retains counters
only for the expected normalized Continue query. Select accepts the first valid
answer and ignores stale/double commands. Continue uses advance_offset plus core
query_pairs/router serialization, clears seed, retains filters/exclusions and
preserves counters on wrap. External query/variant changes, leaving or reload
reset scores. No session persistence, per-card exclusions or core RNG changes.

PreparedPractice now includes selected MCQ Word rows and the next strict pool
size; seed context/filter differences fetch the next unseeded pool. ListenButton
uses existing web-sys speech bindings and simplified hanzi, independent of visible
pinyin/English. Hanzi traditional is secondary only; comparison labels stay intact.
The wide-row threshold is 24 primary-label characters; long answers stack.

All five gates pass: 107 Rust tests and 25 named Chrome scenarios. Root and Pages
artifacts and inspected screenshots are in the A10 evidence section. No dependency,
feature, data/schema or scratchpad changes. The six retained scratchpad sections
remain deferred. Confirm/approve an exact stable Chart.js version when the summary
implementation is undertaken; this milestone adds no package or AGENTS row.


## A10 completion and A43 validation policy - 2026-10-29

A10 is fully complete: Endless/Fixed MCQ 5/10/20, progress, accounting, timing,
shared summary and restart/setup actions. D35 pins chart.js 4.6.1 and build-only
esbuild 0.28.2; no Cargo feature/dependency changes. prepare-summary.cjs packages
only Chart/DoughnutController/ArcElement locally with licenses. Chart effects
clean up normal and late handles; text/actions survive chart failure. MCQ's
approved metric is First-attempt / total questions; matching will track first-
attempt correct pairs / required pairs. Use the same ring rather than repinning.

The hook explicitly reloads preparation when Continue/replay leaves normalized
query unchanged (empty strict view with usable widened rows). Zero-offset repeat
must still increment FixedProgress and restart must reset it. Tests use a Worker
fixture for this case. Ordinary replay RNG/canonical inputs remain unchanged.

Full feature validation passes: 110 Rust tests, 19 root and 10 Pages scenarios,
plus fmt/Clippy/wasm/Pages gates. See A10 Fixed evidence for final artifacts.
Completion is appended in 2026-10-29.md; earlier rows/date files are preserved.
No agent staging or commit. Scratchpad entries remain unchanged pending separately
scoped cleanup after owning documents/implementations are fully applied.

A43/D36 applies the user's testing instruction: routine work uses focused checks
and obvious samples, not full regression each time. Reuse valid evidence and
rerun affected checks after fixes. Full regression only for very major work or
release readiness; existing automated CI is unchanged. Next task: A09 matching
with shared-summary integration. Present its focused scope before starting.


## Database-first priority - 2026-10-29

User now prioritizes A44 generation before A09 matching. Existing generator,
pinned inputs and 12,100-reading artifact are implemented. Both source hashes/
sizes and all three notice hashes pass read-only preflight; report:
target/a44-priority/preflight.json. Do not assume the user requested new sources,
schema or dependencies. Proposed command after scope approval, from root:

```powershell
cargo run -p db-gen --locked -- generate --output target/db-generation-review-2026-10-29
```

The review directory does not exist at proposal time. Preserve the current bundle
and compare rows, logical/diagnostic hashes, schema/indexes and notices. Verify
native v2.0.0/storage v1.4.0 and focused pinned-WASM read-only loading before
replacing the complete web/static bundle within approved scope. Database bytes
may differ with logically equal content. No generation/replacement has run yet.
Use focused generator/compatibility validation under D36; do not rerun unrelated
MCQ or full browser regression for the priority edit.


## A45 CLI handoff - 2026-10-29

Clap 4.6.7 is approved/implemented for native db-gen with defaults plus derive.
Bare cargo run now shows help (usage exit 2); --help exits 0. Use check for an
in-memory import and generate --output NEW_DIRECTORY for writing. Absolute output
and final database/manifest paths print independently of log filtering. Default
remains repository web/static and existing directories always fail, without force.

The focused CLI smoke generated target/a45-cli/review bundle/hanzi.duckdb with
12,100 readings and matching current logical/diagnostic hashes. Native export
verification passes. It is not installed in web/static; A44 can reuse it for
further browser compatibility/review, subject to that task's scope approval.
Logs are under target/a45-cli; checked-in artifacts and sources remain untouched.
Five selected native tests, package Clippy and fmt pass. No full app/browser
regression for this CLI task under D36. Existing Cargo versions/index preserved.


## A46/A44 workflow and database handoff - 2026-10-29

Both tasks are complete under the user's explicit workflow approval. From root,
cargo make dev generates into a fresh target/db-workflow/generated directory,
checks the complete bundle, backs up/replaces web/static and starts Trunk.
Separate db-check/db-generate/db-install/serve tasks exist; bare cargo make lists
commands. Installed cargo-make 0.37.24 was used, with existing Node/Rust/Trunk and
no dependency change. Web npm dependencies must already be installed as documented.

The actual pipeline installed the new database and matching manifest. All other
bundle files were verified/copied and are byte-identical. Logical rows/schema/
indexes/inputs/diagnostics match the original. New DB hash is
728dca2400ba8f23877043c6216a433e9c0adfad539097f3081cfb200605eb02. Complete original
bundle is at target/db-workflow/backups/1793266078908-32d25ff9-f7b4-4ec6-97c9-a2571cc8fc05.

Focused verification passed: four Node installer/failure tests, task plan/syntax,
real generation/install/Trunk serving, matching HTTP bytes and one managed Chrome
dictionary/MCQ load. Logs and a temporary probe source are in target/a46-workflow.
Temporary e2e/example source was removed after validation. Browser/server stopped;
no full application/Pages regression was run under D36. Existing index and prior
completed history were preserved. Next proposed feature is A09 matching.


## A09/A48 handoff - 2026-10-29

Both approved scopes are complete. Practice supports all six variants, multi-type
selection and shared mixed-session summaries. The single session hook delegates
to pure matching/MCQ reducers. D39/D40 own retry accounting and URL/type behavior;
core round generation and dependencies are unchanged.

Combined validation covered 118 native tests and 39 Chrome scenarios. One Pages
browser-Back focus failure in the initial workspace run was fixed by preserving
shell focus on the first matching round, then directly rerun successfully. Root
and remaining native binaries ran separately; no second full suite was needed.
See testing/evidence.md for precise logs and artifact runs. The temporary probe
was archived to target/a09/a09_focus_probe.rs and removed from e2e/tests.

Next proposed work is A13 Browse; present its files/validation plan before editing.
Use cargo make serve for the existing dictionary; cargo make dev explicitly
regenerates/install-backs-up the bundle before serving. Preserve user staging,
prior completed history, pinned packages and untouched retained scratchpad.


## A49 presentation handoff - 2026-10-29

The approved practice redesign is complete under D41: bounded opaque board,
consistent card rows/audio, readable matched states and pinyin on the two
hanzi-to-meaning surfaces. Existing reducers, URL/replay and DB are unchanged.
The matching interaction/layout suite now checks pinyin, board cap and audio
containment, and captures fresh as well as matched states plus MCQ prompts.
Focused matching checks passed; only layout was rerun after visual refinements.
See testing/evidence.md for logs. The temporary layout probe is archived under
target/a49/a49_layout_probe.rs and removed from e2e/tests.

User also requested documentation of drawer icons/reduced width and exercise
info dialogs with explanations/examples. These are recorded in UX and TASKS as
A50/A51, not implemented. Final drawer width remains for that implementation plan.
A13 is still first ready in backlog order. Preserve staging and retained digest.


## A52 compact matching handoff - 2026-10-29

The user requested smaller cards and audio on the right after noticing vertical
scroll. Matching now uses an 88px minimum with no audio footer, compact board
spacing and independent right-edge audio. Pinyin remains beneath hanzi. D41
is amended; MCQ presentation is unchanged. The existing interaction/layout
scenario now checks no document vertical scroll at 1280?720 and 1920?720, plus
mobile and audio containment. Focused run passed; no full regression repeated.
Temporary entry archived in target/a52/a52_layout_probe.rs, removed from tests.
A50/A51 remain documentation-only requests pending their implementation plans.


## A53 future product directions - 2026-10-29

Documentation-only task complete. product/FUTURE.md owns the full requests and
open choices; A54-A60 own live follow-ups. D42 approves avoiding duplicate
simplified hanzi among a matching round's four target pairs, while retaining
separate source readings. This still requires explicit replay review before
builder changes. D43 records future checkmark removal, Settings/audio mode,
independent experimental pronunciation/listening toggles, and sentence AI
using a useful subset of 20 sampled words.

AiMux/LM Studio additions are user-owned scaffold changes. Their 0.5.0 manifest
ranges and root lock resolutions are observed at 0.5.0, not new exact policy
pins. Preserve manifest/lock/llm changes and obtain the
complete dependency/configuration/storage plan before classification work.
No code was fixed or model called; doc-only validation leaves source intact.
Next first ready task is A54 feedback removal; present its implementation plan.
