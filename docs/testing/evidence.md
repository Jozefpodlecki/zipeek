

## A15 modular Chrome harness — 2026-10-28

All five required gates pass: formatting, workspace Clippy with warnings denied,
locked workspace tests, wasm check and the /zipeek/ Trunk build. Workspace tests
pass 94 Rust tests: 88 existing domain/generator/query tests, four new lifecycle
tests and two integration entry points. The root entry runs eight named product/
adapter scenarios; Pages runs one. Two additional expected browser faults (error
and panic) prove same-session recovery and are not product failures/test counts.
Log: target/a15/workspace-tests.log.

Final Chrome artifact runs: target/e2e/root-7396-1793180771694633200 and
target/e2e/pages-30952-1793180767003955600. Each summary records successful browser
quit and reaped owned servers. Scenario screenshots and console/resource JSON,
build/server logs, expected-failure screenshots and clean-recovery markers are
retained. A process inventory after validation found no live Node/Trunk PID from
the recorded harness runs.

Lifecycle tests prove bounded readiness timeout without killing an unrelated
listener, early-exit status/output, guard Drop during assertion unwinding and
termination of a descendant TCP listener. The first sandbox run denied Windows
tree termination; that failure is not a pass. Its two leftover simple Node
fixtures were identified and stopped; final lifecycle/workspace/Chrome validation
ran with permission to manage owned process trees. Direct-child cleanup fallback
reports a failed tree termination honestly.

The first Chrome run exposed a pre-existing obsolete Home h1 lookup. Existing
Home markup has Zipeek in the header Home brand link. The helper now targets that
actual link and preserves the same text assertion; application markup/branding
was not changed. All 46 original explicit assertions remain across the modular
suites, including 12,100 rows, ten columns, bound ID lookup, read-only rejection,
loading/retry, URL replay boundary, themes, keyboard, responsive and Pages prefix.

The workspace package rename is the only lockfile graph change. Exact dependency
records otherwise match the pre-task graph. The AGENTS version/dependency policy
table, source scratchpad, dictionary and background hashes remain unchanged.
No new dependency, schema, runtime API, replay or product UI was introduced.


## A35 Tailwind component migration — 2026-10-28

All five gates pass after the final focus-layer correction: formatting, workspace
Clippy with warnings denied, locked workspace tests (94 Rust tests), wasm check
and /zipeek/ Trunk build. Both Chrome entry points pass: nine root scenarios and
two Pages scenarios, plus the existing expected-fault/recovery probes. Log:
target/a35/workspace-tests.log; gate summary: target/a35/gates.json.

Before screenshots/metrics: target/e2e/root-30556-1793181623041320200/
shell_and_card_styling. Final root/Pages runs:
target/e2e/root-20944-1793198301539203200 and
target/e2e/pages-42024-1793198288786632300. Dark/light captures cover 400/640/768/
1024/1280/1920px in each build. The final root run also captures mounted Loading
and Failed at 400/1280px in both themes, and the existing bootstrap checks remain.
The scrolled 400px footer, desktop/ultrawide cards and mobile failure view were
visually inspected. Twelve before/after metric comparisons preserve card/tile
font, rounding, blur and gradient; intentional geometry changes are the md/lg
boundaries, full-width panel/content and approved navigation/footer cleanup.
Report: target/a35/comparison-report.json.

Acceptance verifies no horizontal overflow at supported widths, md rail and lg
three-tile layout, no max-width cap, label-only navigation with unchanged visual
appearance for active/inactive links and retained aria-current, footer repository
attributes/24px target, single row with attribution wrapping inside its item,
visible keyboard focus with top-panel offset, borderless resting theme control,
backdrop ownership and reduced-motion hover without translation/transitions.

Conservative contrast composition uses black/white image bounds, theme shade and
surface/card opacity. Minimum muted text is 5.39:1 dark and 5.43:1 light; focus/
accent against surface is 8.75:1 and 5.50:1. This is not a full rendered-pixel or
WCAG audit. Light muted text is slate #475569, one shade darker than before.
The focus default belongs to the base CSS layer so component offset utilities
can override it. Hover translation is motion-safe-only: an intermediate Chrome
check caught hover specificity defeating a reduced-motion reset, then passed
after that correction. An intermediate removed-class selector and Yew classes
macro token error were also corrected before the final green run.

Production CSS includes complete literal conditional drawer, breakpoint and
interaction utilities from explicit web/src scanning. No blanket @apply or
bespoke component selector layer remains; the unused v3 config was removed.
Shared tokens and unchanged critical inline CSS preserve loading/backdrop bytes.
Dependency manifests/lockfiles, source scratchpad, dictionary, landscape/HTML and
prior completed history are preserved. The user staged work during the resumed
session; the agent performed no staging and preserved that index. Process
inventory found no live owned Trunk/Node PIDs after final validation.


## A31 scoped state boundaries — 2026-10-28

All five gates pass: formatting, workspace/all-targets Clippy with warnings denied,
locked workspace tests (100 Rust tests), wasm32 zipeek check and standalone
/zipeek/ Trunk build. The six new native tests cover stale success/failure,
duplicate settlement, retry after failure, synchronous request invalidation before
cleanup, unmount/unavailable dictionary access and latest-state rapid toggles.
The original canonical replay vectors remain green. Initial callback dependency
and needless-borrow compile/lint errors were corrected before final validation.

Two Chrome integration entries pass: ten named root scenarios, two Pages
scenarios and the existing expected error/panic isolation probes. The new
providers scenario holds the dictionary fetch, changes routes/theme while Loading,
then checks real 12,100-row Ready metadata, Back/Forward and rapid navigation.
Download count remains one and worker construction count stays unchanged. Two
clicks in one browser script return theme to dark with persisted choice; initial
default loading does not write storage. Existing failed-download Retry, storage
failure, responsive styling and root/Pages asset checks remain green.

Log: target/a31/workspace-tests.log. Artifacts:
target/e2e/root-42252-1793199115180855600 and
target/e2e/pages-19764-1793199099312534400. Both summary.txt files report success
and reaped servers; all five recorded build/server PIDs have exited. Markdown
structure/links, command accuracy and repository consistency checks pass;
report: target/a31/document-validation.json. Protected-file hashes and index
comparison preserve dependency pins, scratchpad, dictionary/background, critical
CSS, previous completed history and the user-staged snapshot.

Cancellation coverage is logical request invalidation. The JS open promise
settles before stale handles are dropped into existing async cleanup; this does
not prove transport abortion or awaited shutdown. Feature query cancellation,
practice/Browse reducers and route motion remain their separate implementation
tasks. No dependency, feature, DB/schema, URL or scoring changes were made.


## A21 routing and practice preparation — 2026-10-28

All five gates pass: formatting, workspace/all-targets Clippy with warnings denied,
locked workspace tests (101 Rust tests), wasm32 zipeek check and standalone
/zipeek/ Trunk build. One new native request-reducer test covers route-input/retry
generations, stale failure and duplicate settlement. Existing pure normalization,
recovery and full-round golden tests remain green; no core algorithm was changed.

Root passes thirteen named scenarios; Pages passes four. Existing expected
error/panic recovery probes pass. New practice scenarios run all six variants at
both base paths, comparing complete debug card ID/label snapshots after reload.
They verify Fixed ignores seed and preserves offset, valid seed zeroes offset,
URL-excluded/unknown seed falls back without offset loss, unknown exclusion IDs
are removed, loaded links remain unchanged and saved exclusions do not affect
shared rounds or get overwritten. A held worker query remains Loading; navigation
to another variant can become Ready before releasing the old failure. A one-shot
bad prepared query reaches Failed+Retry and then succeeds at the same URL/offset.
History navigation preserves route titles/main focus and no old page survives.
Root also proves 404 can render with the dictionary download blocked and Home
can recover through ordinary database Retry. Theme/drawer/style coverage remains.

Final log: target/a21/workspace-tests-final.log. Artifacts:
target/e2e/root-41536-1793200168552371100 and
target/e2e/pages-28464-1793200139986899000. The first root run failed only because
the 404 test used same-document hash navigation with an already-ready client;
adding an explicit refresh exercised the intended blocked download and the full
rerun passed. Initial compile/lint issues were corrected before these final gates.
Both final summaries report reaped servers; all five recorded build/server PIDs
have exited. Documentation validation report: target/a21/document-validation.json.

Markdown structure, relative links, command accuracy and factual consistency pass.
A pre-existing link to the replay section exposed its missing Markdown heading;
the heading now matches that existing fragment. Protected hashes/index checks
preserve dependency manifests/lockfiles, source scratchpad, dictionary/background,
critical CSS and previous completed history. No staging/commit was performed.

Ready practice is still a placeholder: no playable controls, scoring, Fixed
summary, reset action or speech UI is claimed. Recovery view preloading may issue
many queries on the rare ineligible-seed path; orchestration cancellation stops
future scheduling while issued JS queries finish/close normally. Transport-level
cancellation and awaited adapter shutdown remain separate. Debug card diagnostics
are absent in release builds. No new dependency, feature, schema or RNG stream.


## A16 CI, Pages and HTTP readiness — 2026-10-28

All five local gates pass after the readiness fix: formatting, workspace/all-targets
Clippy with warnings denied, locked workspace tests (102 Rust tests), wasm check
and /zipeek/ Trunk build. The separate release /zipeek/ build also passes. Locked
npm ci --ignore-scripts succeeds after the sandbox registry/cache denial is
resolved through the approved execution path. Manifests/lockfiles are unchanged;
npm still reports one high-severity Picomatch tooling dependency advisory.
A39 records separately approved repair, not a change applied by A16.

Final log: target/a16/workspace-tests-final.log. Root/Pages runs:
target/e2e/root-17280-1793216706947297200 and
target/e2e/pages-13028-1793216675837749200. Thirteen root/four Pages scenarios and
expected error/panic recovery probes pass. Five lifecycle tests pass, including
a new HTTP server fixture whose first request returns 404 and later 200. The
initial A16 root setup failure's screenshot showed the browser's HTTP 404 page;
TCP acceptance preceded Trunk's initial served build. Bounded HTTP readiness
before Chrome fixes that observed race. No fixed browser sleep or timeout increase
was added. Final summaries report reaped servers; all five build/server PIDs exited.

Workflow YAML was parsed and checked for events, needs, permissions, gate commands,
working directories, cache inputs and Pages upload path. ci.yml is the sole entry
point; only successful non-PR master validation calls deploy.yml. PRs cannot enter
the write-permission Pages job. Existing action versions and Trunk latest remain.
Report: target/a16/workflow-validation.json. This is a structural/command review,
not a GitHub Actions service run or actionlint claim.

Release artifact report: target/a16/release-artifact.json. Prefixed HTML references
resolve locally; JS/WASM, bridge, worker/Arrow assets and library licences are
present. Dictionary/manifests/input notices/attribution are byte-identical to
source. The check uses actual Trunk output, which has prefixed references without
an HTML base element; document.baseURI derives from the served /zipeek/ location.
The initial artifact script incorrectly required a base element and was corrected.

Markdown structure, links, command accuracy and consistency pass. Dependency graph,
source scratchpad, generated database, background/critical CSS and previous
completed history are preserved. The work was externally staged during the
interruption; target/a16/index-before and index-resumed record that transition.
The agent performed no staging or commit and preserved the resumed snapshot.

Hosted Windows/Ubuntu setup, actual Actions permissions/environment and Pages
publication remain unverified until commit/push and a GitHub run. No workflow
was dispatched, remote setting changed or deployment performed by this session.
Local Trunk is 0.22.0; latest remains an unpinned hosted-tool limitation.


## A37 answered decisions and D33 correction — 2026-10-28

Recorded the user's answered traditional display, five Browse fields, complete
Browse URL/range behavior, Home failure layout and initial 150 ms fade profile
in owning product/architecture/testing documents. These are requirements for
future feature tasks, not implemented behavior. Existing replay comparison keys,
core RNG, summary-definition handoff and practice URL serialization remain.
An illustrative ignored codec check roundtrips Unicode, comma and percent values
through the approved per-filter/outer-query encoding layers; it is a documentation
review, not an implemented browser codec test. Report: target/a37/codec-review.json.

Recovered an explicit user answer missed during A16: accepted build-tool-only
ReDoS advisory. The user confirmed the same condition for the POSIX advisory.
D33 records both acceptances; A39 repair is retired with an appended correction.
No override, audit fix, manifest or lockfile change was made. Existing historical
A16 outcomes retain their original wording.

Documentation structure/relative links and command/repository consistency checks
pass; protected files, prior completed rows and staged index are preserved.
Report: target/a37/document-validation.json. No Rust/Trunk/Chrome gates were run
for this documentation-only pass. A37 overall remains open for the pending final
social-card design/copy answer; social cards and a preview image are required
before v1. No image/metadata implementation or scratchpad cleanup was performed.


## A37 final share-card answer — 2026-10-28

The user approved the proposed static site-wide 1200×630 share-card design and
exact title/description. D34 records the answer in UX/PRODUCT and the decision
index; A40 tracks image/metadata implementation before v1. This supersedes the
pending-answer state in the prior A37 evidence entry, without rewriting it.
A37's completed outcome is appended, its open task removed, and prerequisite-ready
markers updated. Markdown structure/links, command accuracy, consistency and
protected-file/history/index checks pass. Report:
target/a37-final/document-validation.json. No code/image/HTML/dependency or
scratchpad changes; no Rust/Trunk/Chrome runs for this documentation-only pass.


## A38 selective scratchpad cleanup — 2026-10-28

Audit: 17 source topics, 11 fully recorded topics consumed and six retained.
Permanent owners and later superseding answers were checked before removal.
Retained bodies/header compare to the original except added reason notes and
relative-link repairs; deferred summary/chart, old partial lists and unresolved
policy/extraction material remain. Topic inventory and payload checks are recorded
in target/a38/preservation.json; dispositions in target/a38/cleanup-audit.json.
The appended completed table has one row per topic and precedes removal of A38
from the open backlog. A41 tracks the two unresolved policy questions.

Markdown structure, relative links, command accuracy and consistency pass; report:
target/a38/document-validation.json. Earlier completed history/current-date prefix,
application/test/workflow sources, dependency/data files and staged index are
preserved. No product/policy decision, Rust/Trunk/Chrome run, staging or commit
was made for this documentation-only cleanup. A40 remains the next ready item.
