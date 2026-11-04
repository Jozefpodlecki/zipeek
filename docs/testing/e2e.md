# Chrome e2e

A15 implements the approved [D09](../process/decisions/process-and-infra.md#d09-e2e-lifecycle)
and [D28](../process/decisions/process-and-infra.md#d28-modular-e2e-infrastructure)
lifecycle and modular layout. Product coverage below remains planned until its
feature is implemented; the harness itself is implemented.

## Run commands and prerequisites

Install Chrome, Node and Trunk, and the repository's Rust/wasm toolchain. Before
Trunk, install the locked web dependencies from crates/web/ with npm ci --ignore-scripts.
The packaging hook verifies exact DuckDB-WASM 1.32.0 and retains worker/Arrow
licences. Neither tests nor builds generate the checked-in dictionary.

```powershell
# From crates/web/, when npm dependencies are absent
npm ci --ignore-scripts
```

```powershell
# Repository root: all four Chrome binaries and lifecycle checks
cargo test -p e2e --locked -- --test-threads=1

# Focused invocations; no separately started server required
cargo test -p e2e --test smoke --locked -- --test-threads=1
cargo test -p e2e --test pages --locked -- --test-threads=1
cargo test -p e2e --lib --locked

# Optional headless executable; reuses the root harness
cargo run -p e2e --locked
```

Use thirtyfour 0.38.1 WebDriver::managed with Chrome capabilities. It manages its
matching driver download/cache and subprocess. Do not separately install/spawn
ChromeDriver or require it on PATH. Network/cache access for that managed setup
is a prerequisite. The e2e package is now named e2e; the root lockfile changes
only that workspace package identity, preserving resolved dependency versions.

## Implemented layout and ownership

```text
crates/e2e/src/lib.rs
crates/e2e/src/harness/{mod,server,browser,scenario,artifacts,error}.rs
crates/e2e/src/main.rs
crates/e2e/tests/smoke.rs
crates/e2e/tests/pages.rs
crates/e2e/tests/matching.rs
crates/e2e/tests/mixed.rs
crates/e2e/tests/suites/{mod,bootstrap,database,providers,practice,setup,mcq,fixed_mcq,matching,mixed,routing,shell,pages,styling}.rs
crates/e2e/tests/page_objects/{mod,home,shell}.rs
crates/e2e/fixtures/browser_probes/{database,hold_download,release_download,provider_lifetime,practice_queries,diagnostics,styling,styling_acceptance}.js
crates/e2e/fixtures/pages-server.cjs
crates/e2e/fixtures/lifecycle.cjs
```

The library owns process/browser lifecycle and ScenarioContext. Integration-test
modules own product assertions. Page helpers contain reusable queries only;
suites retain assertions. A15 supplied eight root scenarios and one Pages
scenario, preserving the original cases and separating the adapter probe from
retry. A35 adds styling, A31 adds provider lifetime and A21 adds practice/route
coverage. Regular root runs nineteen named scenarios, matching root three, mixed
root two and Pages fifteen, each binary on its own single managed session. Four
Rust integration tests cover thirty-nine named
scenarios, plus the existing root expected error/panic recovery probes.

Root scenarios cover held dictionary Loading, bootstrap landscape/themes,
failure/Retry, real adapter row/binding/read-only checks, practice URL parsing,
landing theme/storage, responsive navigation/skip and drawer/scroll behaviour.
Two shared A10 milestone scenarios exercise the three Endless MCQ variants,
answer locking, keyboard/feedback/counters, pronunciation API calls, canonical
Continue URLs, seeded Next, overflow wrapping, reload/reset and responsive layout.
Two further shared scenarios implement Fixed 5/10/20, progress/accounting, frozen
time, restart/reset, chart cleanup, reduced motion and responsive summary coverage.
The prior Home h1 lookup was stale against the existing user branding markup;
A15 preserves the Zipeek text assertion using the actual header Home brand link.
No app markup or styling changes accompany that selector correction.

## Server lifecycle, readiness and base paths

Each run writes under target/crates/e2e/<suite>-<pid>-<timestamp>/. Root first builds into
its own dist directory, then owns a hidden trunk serve child with a dynamic port,
loopback address and no Pages public URL. A separate small owned Node fixture
server serves that same build for the explicit adapter fixture. Pages owns a
trunk build --public-url /zipeek/ child, then its own Node static server under
/zipeek/. Root and Pages never share dist directories or overwrite crates/web/dist.
The standalone Pages build gate still runs from crates/web/:

```powershell
trunk build --public-url /zipeek/
```

server.rs owns child guards for build, Trunk and static fixture processes. Build
completion has a five-minute bound. Server readiness uses a shared 30-second poll with 100ms exponential
backoff capped at 2 seconds. Fixture/listener tests use TCP; A16 adds an HTTP 200
probe for Trunk so its initial rebuild cannot expose a temporary 404 to Chrome.
Connect/read/write waits are bounded. Browser readiness then waits on the app's
observable database Ready state. There is no fixed startup sleep. Failures include exit status and
captured output. Each process has log and PID artifacts.

Explicit teardown quits Chrome, kills owned server trees and waits/reaps their
parents; Drop also covers setup failure and unwinding. Windows children are
hidden; taskkill targets only the owned live PID/tree. Unix children have a
separate process group. No process is killed merely for occupying 1420/1421.
If Windows tree cleanup is denied, the direct child is killed/reaped and the
suite reports the tree-cleanup failure rather than claiming success.

## Scenario isolation and diagnostic artifacts

Before and after every scenario, move off the old page, remove registered init
scripts, clear blocked URLs/cache overrides, cookies and both origins' storage,
restore 1280×1000 desktop metrics and no-preference reduced-motion baseline,
install console diagnostics, and return to database Ready at the suite Home URL.
Page reload discards prototype mutations and page-local globals. Scripts added
through ScenarioContext are tracked by their CDP identifiers and removed.

Scenario tasks catch assertion panics via Tokio join results, capture failure
screenshots and browser console/resource diagnostics, and await cleanup on both
success and failure. Collect named failures while reset can recover the session;
stop if it cannot. Diagnostic/cleanup failures do not overwrite the primary
scenario failure. Per-scenario screenshots, browser.json, passed/failure text,
server logs and a run summary stay below the run directory.

The root binary also runs two explicitly expected fault probes (returned error
and assertion panic), poisoning storage/cookies, an init script, blocked URLs,
viewport and reduced motion. Each must fail, then recover to Ready on the same
session with all poison absent. Their failure artifacts are expected evidence,
not product failures or extra Chrome sessions.

The read-only adapter probe navigates a dedicated fixture page and imports the
production bridge through the fixture server's explicit __crates/e2e/bridge.mjs mapping.
It uses the same built local DuckDB assets and unchanged dictionary, without
private wasm-bindgen snippet discovery or a production testing API. Raw SQL stays
in the adapter fixture; product suites assert visible readiness and Retry.

## Lifecycle acceptance and evidence

Four library tests cover bounded readiness failure while preserving an unrelated
listener, early exit status/output, Drop cleanup during assertion unwinding and
shutdown of a descendant TCP listener. These tests use small owned Node fixtures,
not a driver service. See [evidence](evidence.md#a15-modular-chrome-harness--2026-10-28)
for the actual run results. Windows process-tree/Chrome validation requires a
process environment that permits managing the owned children.

## Planned feature coverage

As features land, cover Browse controls/pagination, all match/MCQ variants,
Random and Word of the Day according to their owner documents. Preserve HSK
system distinctions, four visible cards per match side, widening/uniqueness,
URL-authoritative exclusions, offset/session separation and the v1 single RNG
stream. Keep root development and Pages prefix checks separate. Resolve relevant
A37 choices and the deferred Fixed summary specification before asserting them.

## A22 answered UX acceptance — planned coverage

These are requirements for A35/A09/A10/A11/A12/A13 as those features land.
A22 adds no executable tests and makes no new passing-test claims. Use A15's
modular suites; do not add more scenarios to the existing smoke dumping ground.

- Shell: navigation has labels only, no numbering or active visual decoration;
  aria-current="page" remains. GitHub is in the single dot-separated footer
  row with copyright/attribution, its accessible name and external-link attributes.
- Daily card: HSK/PoS pills, large simplified hanzi, muted traditional, accent
  pinyin, first gloss and Listen. Verify advanced-band badge and D05 states.
- Setup: six activity tiles with distinct match/MCQ subtitles, conditional HSK
  levels, Endless/Fixed, 5/10/20 with 10 selected, one Start action and exclusions
  line only when count > 0. No all-levels/custom-count/audio control.
- Match: selection from either side, same-side alternative is a no-op, selected
  card toggles off, Escape clears. Wrong attempt gives red/✗ with click lock;
  correct gives green/✓ then dimming without layout movement. Other matches
  remain operable during a correct fade. Next round stays in place and disabled
  until the fourth pair. Reduced motion disables shake/fade and retains feedback.
- Header: Endless streak/cumulative correct, flame at ≥ 3, wrong-action reset,
  no Endless round number; match N / 4; Fixed Round N of M. Reload clears counters.
- MCQ: immediate tint/icons, single-selection lock, Continue gating, 1/2/3
  badges/shortcuts, stacked short-screen/adaptive wide-screen answers, Fixed
  progress bar without question text and no bar in Endless. See [mcq](mcq.md).
- Browse: filter/text autocomplete rows, no implicit token application, AND
  chips, chip removal, exact filter semantics independent of strategy, free-text
  union, ordering, default 5 and 5/10/15/20 sizes, numbered pagination, debounce,
  Enter commit and replaceState roundtrip, zero-result clear and failure Retry.
  Cover combobox keys and delayed (>200ms) spinner with controlled query delay.
  Complete URL grammar/result fields must be resolved before those assertions.
- Pronunciation: stub/spy speechSynthesis.speak and inspect utterance text and
  zh-CN language for the represented word's simplified hanzi under D32's supplied
  fallback. Assert API invocation, never audibility.
- Accessibility: keyboard operation and visible focus for every control, route
  focus to main/heading, distinct document titles, drawer close/focus behaviour
  without an inescapable trap or access to obscured content, feedback icons with
  colour and practical zh-CN content labels. No formal WCAG conformance claim.
- Responsive: check 400, 640, 768, 1024, 1280, 1920px. No horizontal overflow at
  400; stacked match below 768 and side-by-side at 768+; Browse cards below 1280
  and table at 1280+; desktop rail at 768+; summary readable at 1920 when built.
  Verify both orientations and natural scrolling; smaller widths are best-effort.
- Spot-check contrast in light/dark: text ≥4.5:1, controls/focus/icons ≥3:1,
  including muted hanzi, dimmed MCQ options, footer, theme icon and feedback.
  No new axe/Lighthouse dependency is approved.

Fixed summary specification/application is deferred to A09/A10. A22 does not
add acceptance assertions for unreviewed summary behaviour or pin Chart.js.


## A35 styling acceptance — implemented

The styling scenario captures computed metrics and screenshots in dark/light at
400/640/768/1024/1280/1920px, both root and /zipeek/. Assertions check no horizontal
overflow, full-width 48px top panel and uncapped main, md rail/lg landing-grid
boundaries, label-only navigation with aria-current and identical active/inactive
appearance, footer repository placement/attributes/target size and one-row layout,
daily placeholder/backdrop ownership, visible keyboard focus and motion-free
reduced-motion hover. Loading/Failed root boundary screenshots cover dark/light
at 400/1280px; existing bootstrap/loading/retry/Ready checks remain.

The user approved attribution text wrapping inside its item while the footer
items remain in one horizontal row. Capture the scrolled 400px footer to inspect
all attribution/notice links and GitHub. Source utilities replace stylesheet
component selectors; tests use semantic/data hooks instead of those classes.

Contrast checks use conservative black/white image backings composited through
the theme shade and surface/card alpha values. The report records minimum muted
text and focus/icon contrast for those surfaces; this is a bounded composition
check, not a complete rendered-pixel accessibility audit or WCAG claim. Light
muted slate was darkened one shade to preserve contrast. Default focus styling
lives in the base layer so the top-panel focus-offset utility can take effect.
See [evidence](evidence.md#a35-tailwind-component-migration--2026-10-28).

## A31 provider lifecycle coverage — 2026-10-28

The providers suite holds the dictionary download while navigating Home/Practice/
Browse and issuing two theme toggles in the same browser script. It checks the
dark default is not persisted before interaction and two toggles return to dark
with the saved choice intact. After release, real Ready metadata is retained
through Back/Forward and rapid route changes. The download count stays one and
worker construction count does not increase. This proves ownership across route
changes without a production test API; it does not prove transport cancellation.
The existing failure/Retry scenario still reaches the actual read-only dictionary.
Native reducer/gate tests cover superseded success/failure, duplicate completion,
retry after failure and unmount invalidation. Motion/feature-query cases remain
requirements for A32 and the relevant feature tasks; A21 query coverage follows.

## A21 route/query coverage

The practice suite covers all six declared variant routes, full card ID/label
replay across reload, Fixed+seed preserving offset, retained seed zeroing offset,
unknown/excluded seed fallback, unknown exclusion removal and saved-exclusion
independence. It asserts entered URLs remain unchanged. Both root and Pages run
this suite and the late-query/failure/Retry scenario. The latter wraps the pinned
worker postMessage protocol in the browser fixture: hold one prepared query,
navigate, then release its failure after the new route is Ready. A separate
one-shot malformed SQL request produces a handled query failure and successful
Retry without mutating the dictionary. No production failure switch is added.

Root additionally checks unknown-route recovery while dictionary fetch fails.
Route titles and main focus are checked on deep links/history navigation. The
bootstrap suite now waits for the prepared practice placeholder instead of
requiring its former permanent Loader; controlled query holding verifies the
intermediate Loader separately. No gameplay assertions are claimed.

## A16 CI startup readiness

The initial local A16 run exposed a listening Trunk server returning HTTP 404
before its first served build was ready. server.rs now uses bounded HTTP GET /
readiness for Trunk, accepting HTTP/1.0 or HTTP/1.1 status 200, while preserving
TCP readiness for fixture/listener checks. A fifth lifecycle unit test starts an
HTTP fixture that returns 404 before 200 and verifies both probes occurred.
This fixes startup readiness without increasing browser timeouts or hiding
scenario failures. The workflow runs the same owned suite; hosted results remain
separate from local evidence.

## A37 approved feature acceptance requirements

Future A09/A10 tests must verify secondary traditional hanzi on hanzi surfaces
while first-label comparison keys, canonical inputs and complete replay layouts
remain unchanged. Pinyin/English-only labels must not reveal a new hanzi answer.
A13 tests cover the five Browse fields in cards/table, both available HSK systems,
first gloss, complete URL roundtrips, encoded Unicode/comma/percent values,
malformed escapes/ranges, safe loaded defaults without rewrite, page reset and
out-of-range clamping. A12 verifies Home daily fallback/Retry plus all three tiles
during dictionary failure. A32 verifies 150 ms fade-only route/drawer behavior,
no full-route animation on query/page changes and instant reduced-motion final
states. These are approved test requirements, not implemented assertions.

## Planned static metadata checks (D34 / A40)

Verify approved title/description and Open Graph/Twitter large-image tags in the
initial HTML without executing Yew or loading the database. Check the 1200×630
preview dimensions, deployed /zipeek/ asset URL and availability. Visually inspect
the landscape/slate/wordmark/blue composition and absence of word/score content.
Keep route-title/root/Pages checks green. A local artifact check does not prove a
social platform has fetched/refreshed its preview. These are future A40 checks.

## A11 setup coverage

The shared setup suite runs at root and Pages. It starts all six activities with
Fixed HSK 2.0 level 3 / 20 rounds and verifies real preparation plus decoded URL
fields, offset zero, omitted seed/PoS and single-pass comma serialization.
It checks MCQ/default HSK/Endless selection, conditional count pills, retained
count, advanced-band display/system fallback and keyboard Space/Enter operation.
Saved exclusions include duplicate, invalid and unknown IDs: the count reflects
only existing IDs, no load write occurs, and successful Reset clears storage and
the next Start link. A held/failing validation query disables Start and reaches
Retry; blocked storage removal preserves count and displays an error. Malformed
JSON is tolerated without being rewritten. Screenshots cover 400/768/1280px in
both themes with no horizontal overflow. These are setup/preparation checks;
playable answer/Continue behavior is still A10.

## Component selector contract (R28–R30)

Prefer `[data-component="browse-page"]` for the mounted Browse route,
`[data-component="practice-setup-page"]` for Practice Setup,
`[data-component="matching-exercise"]` and
`[data-component="mcq-exercise"]` for live exercise content, and
`[data-component="practice-summary"]` for the completion view.
Shell components use `app-shell`, `app-header`, `main-navigation`,
`navigation-drawer`, `main-content`, and `theme-toggle` markers.

A component marker identifies a stable DOM surface, not its loading or result
state. Continue to use specific role/`data-*` assertions for state, and check
that the expected control is interactable. Browse navigation tests must not
look for `[data-route-placeholder] h1`, because Browse is implemented. The
Random and 404 placeholder assertions are still applicable.

Run `node --test scripts/component-selectors.test.cjs` for a quick source-level
selector contract check. It does not replace browser E2E tests. The two smoke
cases fixed in this cycle are `navigation_and_rapid_theme_commands_preserve_dictionary`
and `responsive_shell_preserves_navigation_theme_and_keyboard_access`.

## R31–R33 personal Word Set smoke coverage

The root smoke suite includes
`personal_sets_persist_across_navigation_and_backup_rejects_invalid_data`.
It exercises the real Browse UI and browser localStorage: create a named set,
confirm that Practice Setup offers it, refresh Browse, export it as JSON,
then reject an invalid import while preserving the saved collection. The
scenario uses the component/interaction `data-*` contract and does not
require classification data. It must be run with Chrome locally; its presence
in source is not a passing browser result.

The separate `word_sets::browse_selection_rejects_missing_sets_instead_of_dropping_the_filter`
unit test confirms that malformed or deleted saved-set references fail closed.
Valid empty sets deliberately produce an empty candidate list rather than
unrestricted Browse results. To validate actual hanzi/pinyin/gloss/HSK/PoS/
category search results, first run `cargo make db-generate`,
`cargo make db-install`, and `cargo make db-browse-check` with the approved
native generator; the currently bundled database does not pass this preflight.

## Browse retry safety regression (R34–R36)

The root smoke harness includes `browse_retry_keeps_invalid_personal_set_filter_blocked`.
It creates a personal set, simulates a stale selection, submits a search, then
clicks Retry. The missing-set error must remain visible and no unrestricted
Browse results may appear. The test uses `data-component` and existing stable
`data-browse-*` attributes. The regression is distinct from validating live
normalized search results, which requires a newly generated dictionary.

The user confirmed a full `cargo test --workspace --locked` pass and separate
Chrome smoke pass on the R33 baseline (2026-11-04), before this new scenario.
To exercise the new case, rerun:

```powershell
cargo test -p e2e --test smoke --locked -- --nocapture
```


## Browse saved-set mutation coverage (R37–R39)

`deleting_personal_set_requires_explicit_new_search` in the root Smoke suite creates and selects an empty personal set, confirms that the selected-set query remains empty, then deletes the set and checks that previous Browse results are invalidated. There is deliberately no Retry button in this state; a new Search is required to display unrestricted results. The scenario also exercises successful backup export/import and verifies that it blocks stale results until another explicit Search. This is distinct from the earlier invalid-backup and malformed-selection checks.

The pure Browse unit test `saved_set_mutation_resets_page_without_losing_scope` checks that modifying a selected set resets its query offset to zero while retaining the search field and an explicit saved-ID filter (including an empty set). Run `cargo test -p zipeek --locked`, followed by `cargo test -p e2e --test smoke --locked -- --nocapture` on a machine with the approved Chrome/Trunk toolchain. The bundled single-table dictionary can exercise mutation guards, but real normalized text/category/HSK Browse acceptance still requires `cargo make db-generate`, `db-install`, and `db-browse-check`.

## R42 Browse result-view composition

`BrowseResults` now renders `BrowseLoading`, `BrowseError`, `BrowseEmpty`, `BrowseWordCard`, and `BrowsePagination` from `components/browse/results/`. Source selector regression coverage checks their `data-component` identifiers and existing `data-browse-*` hooks. The unit/browser smoke suite must be rerun after this structural change; the Node source checks are not a Rust compiler or Chrome substitute.

## R74 — deterministic AI fixtures and opt-in browser learner

The existing Rust/Thirtyfour harness remains authoritative. R74 adds a locally
owned, dependency-free Node `chat/completions` fixture under
`crates/e2e/fixtures/ai-server.cjs`. The same owned-process lifecycle and
failure artifacts apply to this server, which runs on a dynamically allocated
loopback port. Every scenario resets the fixture before execution. Fixture
modes are `success`, `malformed-generation`, `server-error`, and
`slow-evaluation`. Successful generation uses an intentionally synthetic
bilingual sentence derived from **visible selected vocabulary**, not a
language-quality reference. Evaluation compares the submitted answer to its
reference for deterministic acceptance tests.

Run the fixture contract tests independently of Rust and Chrome:

```powershell
node --test crates/e2e/fixtures/ai-server.test.cjs
```

Run deterministic root browser coverage, which includes selecting visible
Translation tiles, handling malformed AI responses, discarding a delayed
AI evaluation after navigation, and desktop/mobile screenshots for Browse and
Practice:

```powershell
cargo test -p e2e --test translation_fixture --locked -- --test-threads=1 --nocapture
```

The `learner::TranslationPage` page object observes only rendered source text,
target language and available tile IDs/text, then uses Thirtyfour to click
those tiles. It never reads the hidden solution from script globals, the
`data-practice-round` debug attribute, a database answer key or an evaluation
prompt. This is intentionally separate from the existing legacy practice
scenarios that inspect round internals to verify deterministic generation.

The live simulated learner is **ignored by default** and requires a running
LM Studio API accessible both to the browser and to Node. It uses the same
Thirtyfour actions but asks a real model to order the visible tiles. A learner
mistake is not a test failure; failure means no valid model action, broken UI
progression or incorrect shared-session accounting. PowerShell invocation:

```powershell
$env:ZIPEEK_E2E_AI_ENDPOINT = "http://127.0.0.1:1234/v1/"
$env:ZIPEEK_E2E_AI_MODEL = "your-loaded-model-id"
cargo test -p e2e --test ai_learner --locked -- --ignored --nocapture --test-threads=1
```

Live learner artifacts capture its visible observation and selected tile IDs,
plus the existing browser logs, screenshots, process logs and scenario
summary. The test does **not** grade Mandarin pronunciation, run every
exercise type, or establish reliable AI answer accuracy. It is the first
live Translation path; Matching/MCQ learner observation and a multi-mode
planner remain follow-up work.

For layout testing the harness saves `practice-mobile.png`,
`practice-desktop.png`, `browse-mobile.png`, and `browse-desktop.png` into
scenario artifacts and checks for horizontal overflow. These are **captures,
not approved pixel baselines**. Dynamic backgrounds, fonts and motion must be
stabilized before image-difference assertions are introduced.

## R75 deterministic fixed Translation acceptance

`crates/e2e/tests/translation_fixture.rs` now includes
`five_hsk6_translations_from_setup` in the normal Thirtyfour suite.
The browser scenario opens Practice setup, chooses **Sentence translation**,
selects **HSK 3.0 level 6**, sets **5 exercises**, enters the isolated AI
fixture settings, generates the first sentence, and completes five full
word-bank/evaluation/Continue cycles. It checks the progress counter, shared
ledger, final completion screen, and absence of a sixth question. The test
never reads the reference solution from the app/database; its deterministic
fixture exposes an easily solvable learner-visible sentence instead.

The setup Start button currently navigates to the Translation configuration
page; that page still requires a model identifier and a separate Generate
sentence click. This extra step is covered explicitly, not represented as
an automatic start. HSK 2.0 level 6 is a separate vocabulary source from
HSK 3.0 level 6 and needs its own case when desired.

```powershell
cargo test -p e2e --lib --locked
cargo test -p e2e --test translation_fixture --locked -- --test-threads=1 --nocapture
```

The earlier R74 test compilation failure arose from exported macros expanding
`$crate::AssertionKind` and `$crate::AssertionFailure` without corresponding
root re-exports; both are now re-exported from `e2e::lib`.
The deterministic fixture does not require LM Studio; Chrome/Trunk are managed
by the existing harness. Full browser execution remains to be verified on
Windows with the Rust toolchain.

## R77 Browse automatic-search regression

Thirtyfour Browse/Word Set tests must no longer click `[data-browse-search]`. The query field `[data-browse-query]` performs debounced search; each filter change issues a fresh request. `[data-browse-refresh]` manually repeats the current selection, and `[data-browse-clear]` clears filters. The list is marked `[data-component="browse-results-list"]`, with stable `[data-component="browse-result"]` rows. Test stale generations, invalid saved-set IDs, deletion auto-refresh, narrow viewport overflow, and repeated refresh.

## R83 navigation icon and filter-query acceptance

The root styling probe asserts that the three drawer navigation links contain
an `aria-hidden` SVG icon and retain readable Home/Practice/Browse labels,
`aria-current`, and existing responsive geometry. Run the existing root shell
suite after the WASM build; source-level Node checks cannot verify SVG output.

The new `database::practice_filter_queries` native unit tests cover multiple
HSK memberships, OR-within/category/PoS group intersections, parameter binding,
category-only filters, empty filters, invalid category codes and bounded sample
sizes. These tests must pass before wiring the filters to live exercise starts.
The current browser scenarios still configure a single HSK band by design.
