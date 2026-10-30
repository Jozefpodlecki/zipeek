# Validation evidence


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


## A11 practice setup — 2026-10-29

All five gates pass after the final setup-surface correction: fmt, workspace/
all-targets Clippy with warnings denied, locked workspace tests (105 Rust tests),
wasm check and the /zipeek/ Trunk build. Three new native tests exercise typed
HSK switching/stale-level rejection, Fixed-count retention/canonical URL omissions,
and saved-ID parsing/deduplication. No core replay or dependency changes.

Root runs fifteen named scenarios and Pages six, plus existing expected fault/
recovery probes. Shared setup checks start all six activities through actual UI
and verify real PreparedPractice plus HSK/mode/count/offset/exclusion query values,
omitted seed/PoS and single-pass serialization. Unknown/duplicate/invalid saved IDs
are handled without load writes; Reset clears storage and the next Start URL.
Held/failing lookup disables Start and reaches Retry; blocked storage removal
preserves count and shows an error; malformed JSON remains untouched. Space/Enter
and 400/768/1280px dark/light screenshots cover setup selection/navigation/layout.

Final log: target/a11/workspace-tests-final.log. Artifacts:
target/e2e/root-19824-1793235095678890800 and
target/e2e/pages-26020-1793235051591505900. Visual review of the first passing
run caught the undefined bg-card utility; the final surface token paints selected
and unselected controls. A computed-background assertion guards that behavior.
The final Pages dark-400 setup image was inspected. The first ID-conversion
compile error was also corrected before the final gates. No unavailable gate is
counted as passed.

Documentation structure/links, command accuracy, consistency and protected-file/
index checks pass; report: target/a11/document-validation.json. Previous outcome
date files stay unchanged; completion is dated 2026-10-29 after the interruption.
The implementation was committed externally; no agent staging/commit occurred.
Storage key is zipeek.excluded_words; only explicit Reset writes/removes it.
Practice URL loads remain independent of saved exclusions. MCQ interaction is
still the next A10 task, not a claim made by setup completion.


## A10 Endless MCQ milestone ? 2026-10-29

All five gates pass: fmt, locked workspace/all-target Clippy with warnings denied,
workspace tests (107 Rust tests), wasm check and /zipeek/ Trunk build from web/.
No missing prerequisites or failed gates. Log: target/a10/workspace-tests.log.
Two added pure reducer tests exercise invalid/stale/duplicate answers, first-answer
locking, Continue gating, expected next-round counter retention and wrong-answer
streak reset. Core deterministic builder/replay remains unchanged.

Root runs seventeen named scenarios and Pages eight, plus the original root
expected fault/recovery probes. Both new shared MCQ scenarios pass against real
rounds for all three Endless variants: wrong/correct feedback and locked/dimmed
answers, persistent Continue, number keys/Space/Enter, editable-field shortcut
exclusion, cumulative correct/streak/flame, no Endless progress bar, canonical
single-encoded exclusions, offset +1, seed clearing, overflow wrap retaining counts,
reload reset and return through setup. Native speech calls are spied for zh-CN
hanzi; audible output is not asserted.

Artifacts: target/e2e/root-46084-1793235660799282700 and
target/e2e/pages-28504-1793235595415036900. Captures cover all three variants at
400/768/1280px in light/dark; horizontal overflow and Continue visibility are
asserted. Reviewed meaning-to-hanzi-dark-400.png and
hanzi-to-meaning-light-1280.png in the Pages responsive scenario. Both summaries
confirm browser quit and owned servers reaped; recorded PIDs have exited.

Documentation structure/local links/commands/facts and protected-file/index checks
pass (target/a10/document-validation.json). Dependency/feature pins, core replay,
database/landscape, bootstrap CSS, earlier outcomes and saved exclusions remain
unchanged. No agent staging/commit. A10 remains open for Fixed gameplay/progress/
completion and summary; this is the first playable Endless milestone.


## A10 Fixed MCQ and summary - 2026-10-29

All five full-regression gates pass for complete A10: fmt, locked all-target
workspace Clippy with warnings denied, locked workspace tests (110 Rust tests),
wasm check and /zipeek/ Trunk build from web/. Locked npm ci --ignore-scripts
and summary packaging also pass. Chart.js 4.6.1 and dev-only esbuild 0.28.2 are
approved under D35. Pre-existing npm records are unchanged; the additions are
Chart.js, its color dependency and esbuild/platform packages. License copies and
local tree-shaken bundle metadata are retained. No runtime bundler/CDN.

Final log: target/a10-fixed/workspace-tests-wrap.log. Artifacts:
target/e2e/root-45940-1793236646515334800 and
target/e2e/pages-40276-1793236540546127000. Nineteen root and ten Pages scenarios
pass, plus original expected fault/recovery probes. New Fixed checks cover
5/10/20 across all variants, completed/total progress, exact final completion,
first-selection accuracy/mistakes/highest streak, controlled frozen wall-clock
time, reload/navigation reset, fresh-offset replay and default setup return.
Chart checks cover actual canvas, four DOM text stats, destroy on replay/leave,
300 ms normal and instant/live reduced motion, blocked-module fallback and
400/768/1280px screenshots in light/dark. Empty-strict-pool fixture retains real
widened rows and verifies unchanged-URL Continue and replay. Both summaries report
clean shutdown and owned PID checks find no remaining processes.

An initial timer fixture missed hash-only navigation; current-document clock
installation corrected it. Review found and fixed a same-URL preparation stall.
Final dark/mobile and light/desktop captures were reviewed across the validation
runs, including final Pages light-1280. Earlier valid evidence was retained.
D36 now reserves routine work for focused samples; this major feature's full
run was already underway when the user requested the policy change.

Documentation structure, local links, commands/facts, prior-outcome prefix,
protected files and preserved external staged-snapshot checks pass:
target/a10-fixed/document-validation.json. DIGEST and core replay/data artifacts
remain unchanged. No agent commit/staging occurred. A10 is complete; next is A09.

External staging was detected during final documentation validation. The agent
made no index edits; target/a10-fixed/index-resumed records the snapshot retained
for the final preservation check. Final documentation updates remain unstaged.


## A46 workflow and A44 regeneration - 2026-10-29

Four focused node --test scripts/db-workflow.test.cjs checks pass: complete bundle
installation with preserved backup, checksum refusal without touching installed
files, path/missing-notice refusal and rollback after simulated publication failure.
Node syntax and cargo-make task discovery/ordered dev plan pass. Existing installed
cargo-make 0.37.24 is used without adding/upgrading tools or project dependencies.

The real cargo make dev sequence passed. It generated a native-verified 12,100-
reading candidate, installed the complete verified bundle with backup, then served
Trunk on 1420. Downloaded hanzi.duckdb bytes exactly matched web/static. One managed
headless Chrome session loaded the installed read-only dictionary, prepared three
MCQ options and read the 12,100-row report. Browser quit succeeded and the owned
cargo-make/Trunk process tree was stopped. Port 1420 is no longer listening.

Evidence: target/a46-workflow/dev.log, browser.log, smoke.json, cleanup.log,
bundle-comparison.json and retained temporary db_workflow_probe.rs. The live
example file was removed after validation. Original complete bundle is retained
at target/db-workflow/backups/1793266078908-32d25ff9-f7b4-4ec6-97c9-a2571cc8fc05;
each backup file hash matches the pre-install snapshot. Logical rows, schema,
indexes, input records and diagnostics are unchanged. New DB SHA-256:
728dca2400ba8f23877043c6216a433e9c0adfad539097f3081cfb200605eb02.

The first helper test caught confusion between generator Cargo-manifest and input-
manifest hashes; validation now compares pinned INPUTS.toml's file record and the
corrected tests pass. No full workspace/root/Pages regression was run for this
workflow change under D36. Docs/links/commands/facts, old outcome prefix, root lock
and staged-index preservation checks pass. No agent staging or commit occurred.


## A09 and A48 matching and mixed sessions - 2026-10-29

Passed: cargo fmt --all -- --check; workspace/all-targets Clippy --locked with
-D warnings; cargo check --target wasm32-unknown-unknown -p zipeek --locked;
trunk build --public-url /zipeek/ from web/. No missing prerequisites.

The initial cargo test --workspace --locked run passed all 53 generator tests,
matching (three scenarios) and mixed (two scenarios), then exited 101 because
Pages late_queries_and_retry_keep_current_route expected AppShell main-content
focus after browser Back. The initial matching heading effect stole that focus.
The fix limits matching heading focus to subsequent rounds. All other fourteen
Pages scenarios passed in that run. Only the failing scenario was rerun with a
temporary Pages harness probe, which passed; its source is retained under
target/a09/a09_focus_probe.rs and removed from the integration-test tree.

The remaining root binary passed all nineteen scenarios and existing expected
error/panic recovery probes. cargo test -p zipeek -p zipeek-core --locked passed
the remaining 65 native tests. Combined coverage is 118 native tests and 39
distinct Chrome scenarios (19 root, 3 matching root, 2 mixed root, 15 Pages).
This is retained successful coverage plus targeted recovery under D36, not a
claim that the original workspace command exited successfully. No full rerun
was performed after the focused fix. Final fmt/Clippy/wasm/Pages build passed.

Logs: target/a09/workspace-tests.log, focus-rerun.log, root-tests.log and
native-remaining.log. Focused development logs: matching-focused.log and
mixed-focused.log. Final preservation/document report: target/a09/validation.json.
Chrome artifact runs under target/e2e/: matching root-31616-1793284867114747800;
mixed root-4864-1793284926624856000; Pages pages-35916-1793284952246064600;
corrected focus pages-37024-1793285168716333400; root root-37340-1793285185707760400.

Inspected matching mobile/desktop light/dark captures and mixed-summary layout.
The controlled Date.now clock in mixed tests freezes Chart.js animation in its
screenshot; reduced-motion summary captures exercise the final ring. Tests cover
all variants, 5/10/20 fixed counts, both-pair retry accounting, 11/14 mixed accuracy,
wrong/timer/stale-event handling, keyboard/Listen, restart, no-repeat transitions,
seed/reload/exclusion URLs, insufficient pools, wrapping and responsive layouts.
No dependency, installed dictionary, input/source or v1 replay changes.

Final documentation checks cover 32 Markdown files and 381 relative links. New
feature links/anchors pass, command examples match package/test names, and
completion history retains its exact original byte prefix. Nine pre-existing
links in database module/schema notes target missing D08/D16/D18/D21/D23 sections
or an old A06 heading; these unrelated documentation gaps remain recorded in
target/a09/validation.json. Installed artifacts, dependency policy/pins, retained
scratchpad and frozen history are unchanged. The user updated the index during
work; no agent staging occurred. All thirteen recorded owned server/build PIDs
are absent after the runs. git diff --check passes.


## A49 practice presentation - 2026-10-29

Passed: cargo fmt --all -- --check; cargo clippy -p zipeek -p e2e --all-targets
--locked -- -D warnings; cargo test -p zipeek --lib --locked (27 tests);
cargo check --target wasm32-unknown-unknown -p zipeek --locked;
trunk build --public-url /zipeek/ from web/.

Focused Chrome cargo test -p e2e --test matching --locked passed its three
interaction/layout, Fixed summary/restart and recovery/seed/same-URL scenarios.
After tightening padding/type spacing and strengthening card border contrast,
only the affected interaction/layout scenario was rerun via an archived temporary
probe, passing both times. The probe is retained at target/a49/a49_layout_probe.rs
and removed from e2e/tests. No full regression under D36; reducer/round/database
behavior is unchanged.

Logs: target/a49/matching.log, layout-final.log, layout-contrast-final.log.
Initial run: target/e2e/root-8744-1793286262301659900. Refined layout run:
root-32572-1793286348954224500. Final contrast/layout run:
root-21556-1793286425029938100. Fresh cards captured at 400/1280px in both themes;
1920px dark verifies the 896px cap. Existing matched captures also cover 768px.
Checks assert equal row heights, contained audio, opaque board, no overflow and
pinyin placement only on approved surfaces. Existing keyboard/audio, feedback,
Next gating and URL checks pass. MCQ pinyin captures cover mobile light/desktop
dark. Final matching light/dark, fresh/matched and MCQ screenshots were inspected.

Enabled card borders use slate-500 against white or slate-900, exceeding 3:1.
Matched text remains opaque with slate-600 on slate-100 or slate-300 on slate-800.
A49 documents A50/A51 without claiming drawer/help-dialog implementation.
Preservation/link/process results are recorded in target/a49/validation.json.


## A52 compact matching layout - 2026-10-29

Passed the existing matching_interaction_and_layout scenario alone through a
temporary owned Root harness entry. It samples all three matching variants,
400px mobile light/dark, desktop 1280?720 light/dark and 1920?720 dark, verifies
no desktop page vertical scroll, contained right-edge 44px audio, pinyin beneath
hanzi, selection/keyboard/audio independence, wrong/correct feedback and Next.
Log: target/a52/layout.log. Artifacts:
target/e2e/root-33448-1793286774690346000/matching_interaction_and_layout.
Desktop dark and mobile light captures were visually inspected.

Passed fmt --all -- --check, affected zipeek/e2e all-target Clippy --locked
-D warnings, zipeek wasm32 check --locked and Trunk Pages build /zipeek/.
No full regression or unchanged native reducer tests repeated under D36 for
this focused layout iteration. Probe source archived to target/a52 and removed
from e2e/tests. Preservation/docs results: target/a52/validation.json.


## A53 future requirements documentation - 2026-10-29

Documentation-only checks cover Markdown structure, relative links/anchors,
command accuracy and facts against source. Pinned input inspection confirms
the same simplified hanzi has lowercase jing3 and uppercase Jing3 forms with
well versus constellation/surname glosses. Current ROUND_GENERATION uniqueness
checks right labels only. AiMux 0.5.0 ranges and LM Studio client scaffold are
present in user edits; core/providers and their AiMux helpers resolve to 0.5.0
in the root lock. The scaffold is unwired from generator CLI/import. No compile or runtime success is claimed.

User clarified same-hanzi avoidance, two distinct experimental audio exercises
with independent Settings toggles, and sentence generation using a useful subset
of 20 words. D42/D43 and owning docs record these, retaining open schema/tag,
API/eligibility/scoring, settings default and runtime-generation questions.
No Rust/Trunk/Chrome or AI request was run for the doc-only task. Source/DB/
manifest/lock hashes, staged index and exact prior completed prefix are checked
in target/a53/validation.json. Retained scratchpad entries are untouched.

Final A53 report covers 18 Markdown files, 305 existing relative-link targets
and new anchors. New-change structure/link checks pass. Twelve inherited
database-document link gaps remain (six missing anchors and six incorrect
relative decision paths already present in the staged baseline); these are
listed separately in validation.json, not claimed fixed. All 107 protected
source/data/manifest/lock files match their starting hashes, staged index is
unchanged and previous completed bytes are an exact prefix. git diff --check
passes. Root lock inspection confirms all four AiMux packages at 0.5.0.


## A53 experimental AI connection amendment - 2026-10-29

Official LM Studio compatibility/settings/authentication docs verify local /v1,
CORS enablement and optional-by-default API-token authentication. Official
Chrome local-network guidance and GitHub Pages hosting docs support the
distinction between localhost development and public-site browser permissions.
References are linked beside the claims in product/FUTURE.md. These are source
checks, not successful endpoint/model integration evidence. No Rust/Trunk/Chrome
or local-server/model requests were run for the documentation-only amendment.
Structure/links/commands/facts and preserved source/index/completed-prefix checks
are recorded in target/a53-connection/validation.json.


## A53 Practice filter amendment - 2026-10-29

Docs-only structure/link/command/fact checks. Source inspection confirms setup
currently creates PracticeFilter with an empty part_of_speech Vec, while core
practice URL/query support for PoS exists. No setup picker/category schema was
implemented. D44 records the clarified Boolean semantics; A61 owns follow-up
review, with category data dependent on A56. Source/data/index/completed-prefix
preservation and doc results: target/a53-filters/validation.json. No Rust/Trunk/
Chrome checks or model calls for this documentation-only amendment.


## A53 schema/index/FTS review brief - 2026-10-29

Read generator schema DDL and current browser query predicates; confirmed the
ordinary HSK index pair and PoS string_split/list_contains scan. Official
DuckDB 1.4 WASM extensions/FTS/indexing docs establish platform availability
and limitations, linked in schema/REVIEW.md. No native/WASM FTS loading, browser
benchmark or generated-index query was performed; exact pinned compatibility
is explicitly future A62 work, not a passed gate. Documentation-only validation
and protected source/data/index/completed prefix results are recorded in
target/a53-schema/validation.json. Existing replay/dependency/schema approvals
remain in force; no Rust/Trunk/Chrome or model calls for this doc update.
