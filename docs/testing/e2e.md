# Chrome e2e

Chrome end-to-end strategy, driver ownership, server lifecycle, and the A15
modularization proposal. Part of the [testing index](TESTING.md). Approved
requirements in [D09](../process/decisions/process-and-infra.md#d09-e2e-lifecycle);
proposed layout in [D28](../process/decisions/process-and-infra.md#d28-modular-e2e-infrastructure).

## Current state

The existing manifest is named `zipeek-e2e` and specifies `thirtyfour = "0.38.1"`.
It is a workspace member with an executable and a headless integration smoke test.
Both require an external server on port 1420; shared lifecycle/readiness is absent.
The executable has headless mode commented out and suppresses the automation
banner. Two smoke tests cover the landing shell/theme and responsive navigation;
Home retains an `h1` reading Zipeek. A06 later added Chrome cases that hold the
dictionary fetch, exercise failure/retry, read the full 12,100 rows, and check
the `/zipeek/` Pages build. The approved target package name is `e2e`; that
manifest rename has not been made.

Intended command after workspace/harness setup:

```powershell
# Repository root; currently unavailable
cargo test -p e2e -- --test-threads=1

Each test binary owns its own Trunk child and managed WebDriver session. Serial
execution avoids port and browser conflicts. Implement the shared setup/polling
helper once in the e2e crate and reuse it in every test binary.

Chrome and driver ownership
Chrome must be installed locally and on CI. CI may use preinstalled Chrome on
ubuntu-latest or a Chrome setup action such as
browser-actions/setup-chrome. Create the browser session using:

rust
let driver = WebDriver::managed(DesiredCapabilities::chrome()).await?;
Use thirtyfour's managed-driver support (the manager feature). It handles
matching/download of ChromeDriver, a free port, subprocess creation, and teardown
when the final connected WebDriver handle drops. Do not separately install
ChromeDriver, put it on PATH as a prerequisite, or manage its subprocess.
Driver download/cache availability is a prerequisite for the managed setup.
See the thirtyfour managed-driver API.

Trunk lifecycle and readiness
From web/, the harness spawns trunk serve --port <free-port> as a child.
The harness owns a guard that kills and reaps the child on drop, including setup
failure and test failure. Trunk must already be installed.

Poll http://127.0.0.1:<port>/ until any HTTP response is received, or use TCP
connection success to the same port. Use a total timeout of 30 seconds, starting
with 100ms backoff and doubling to a 2s cap. Bound each attempt/wait by the
remaining timeout. Do not use a fixed startup sleep. Report readiness failures
with the child exit status/output where available.

Development base path and Pages validation
E2e starts Trunk without --public-url /zipeek/ and navigates to
http://127.0.0.1:<port> at /. Do not add a serve_base configuration for v1.
The /zipeek/ base path is a Pages deployment concern; validate its build with
trunk build --public-url /zipeek/ separately from root-served e2e tests.

These are approved harness requirements, not existing implemented behaviour.

A15 modularization proposal from the 2026-10-27 review
smoke.rs currently contains all eight test functions (seven standard cases and
one separately invoked Pages case), repeated managed Chrome setup, screenshots,
CDP configuration and long embedded JavaScript probes. Each test currently
creates its own session. It does not implement D09's one session per binary,
owned server lifecycle or reusable isolation. Do not add more feature cases to
that file before A15 establishes these boundaries.

Proposed layout, not implemented:

text
e2e/src/lib.rs
e2e/src/harness/mod.rs
e2e/src/harness/server.rs
e2e/src/harness/browser.rs
e2e/src/harness/scenario.rs
e2e/src/harness/artifacts.rs
e2e/src/main.rs
e2e/tests/smoke.rs
e2e/tests/pages.rs
e2e/tests/suites/{mod,bootstrap,database,routing,shell,pages}.rs
e2e/tests/page_objects/{mod,home,shell}.rs
e2e/fixtures/browser_probes/{database,hold_download}.js
e2e/fixtures/pages-server.cjs
Use tests/page_objects/ as an explicitly declared module directory, not another
automatically discovered test crate. Harness code lives in the e2e library so
the optional executable can reuse it; scenario and page-interaction helpers
remain test modules. Keep assertions in the relevant scenario. Page helpers
centralize semantic interactions/selectors; do not hide the entire assertion
sequence behind a broad assert_page_works helper.

smoke.rs becomes a thin single async test entry point invoking named scenarios
in order on one harness-owned Chrome session. This avoids sharing an async
WebDriver across the separate Tokio runtimes of independent tokio::test cases.
Report every scenario by name; do not misreport scenario count as Rust test count.
The runner should collect scenario failures after cleanup where the session
remains usable, and stop with diagnostics when it cannot recover. Pages has a
separate entry point/session and a build served under /zipeek/; the root suite
always uses /. D09's serial invocation remains the default.

server.rs owns hidden Trunk/static-server children, readiness and kill/reap guards.
Use dynamically assigned ports, per-run output directories and logs; never kill
an unrelated process occupying the old 1420/1421 ports. Root and Pages builds
must not overwrite each other's dist directory. Reuse the Node runtime already
needed by asset packaging for a small owned Pages fixture server; any added
Rust HTTP dependency requires a named approval, not an incidental harness change.

browser.rs owns managed Chrome, headless capabilities, explicit quit and session
setup. scenario.rs resets storage/cookies, returns to a known route and restores
CDP blocked URLs, injected-script identifiers, media and viewport overrides after
each case. A failed case must not poison subsequent cases. Async cleanup is
awaited on success and failure; process guards also cover setup failures and
unwinding. Preserve the original failure if cleanup or artifact capture fails.

artifacts.rs writes uniquely named case/viewport/theme screenshots and useful
console/network diagnostics below target/e2e/<run>/<scenario>/. CDP/download
fixtures belong in browser_probes, not repeated Rust string literals. Product
smoke cases should not discover private wasm-bindgen snippet paths through
performance entries; move the low-level SQL/bridge probe to a dedicated adapter
fixture with an explicit module entry point. Keep observable readiness/retry
checks in the user-facing suite.

Migration acceptance: preserve every existing assertion, including read-only
rejection and /zipeek/ assets; run without manually starting a server; prove
cleanup on failed startup/scenario; record build/session failures with artifacts;
and leave no owned child process behind. Rename the package to e2e and update
its workspace lockfile entry deliberately. No new framework, driver service or
ChromeDriver subprocess is proposed. Existing checks remain passing evidence;
none of this infrastructure is claimed to exist yet.

Planned smoke coverage
Planned smoke coverage includes the landing page, persistent theme switch, browse
queries/pagination, each of the three matching variants, Random, Word of the Day,
at the root-served development URL. The Pages build is checked separately with
/zipeek/. Resolve dependent product decisions first.

When these features are implemented, cover the approved requirements:

HSK memberships can differ across systems or be missing in one; filtering must
not treat equal numeric levels in different systems as equivalent.

Round generation preserves four correct pairs, uses internal candidate reserves,
widens HSK before dropping PoS, enforces visible right-label uniqueness, and
reports insufficient pools. E2e sees exactly four cards per side.

Endless offers Next round without a summary or persisted score. Fixed stops
after N rounds and reports correct pairs, mistakes, and elapsed time with both
exit actions. Test exactly 5/10/20 buttons, default 10, and no custom-N input.

Random uses the shared generator, ignores setup filters, anchors to a seed,
starts Endless, prefers HSK 3.0, and exercises ten invalid-membership retries,
first-eligible fallback, and typed pool failure. Test D12/D13 contracts and
use approved D18 eligibility and the resolved D19 call order for golden assertions.

During development, regenerate affected fixtures/artifacts after approved breaking
changes. Backward-compatibility tests for old generated DB formats are not required.