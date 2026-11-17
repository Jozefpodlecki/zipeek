# Managed Chrome and Firefox end-to-end testing

This document describes the E2E code currently present, not a chronological series of checkpoint contracts. Historical release evidence remains in [RELEASE_HISTORY](../RELEASE_HISTORY.md).

## R326 native AI fixture and responsive viewport

The AI fixture is owned by the Rust harness. `FixtureClient` makes bounded loopback HTTP calls to reset scenarios and read request diagnostics, rather than using browser-injected `fetch` during setup. This avoids making Chrome/Firefox networking behaviour a dependency for fixture setup. Actual CORS tests still use the browser. Scenario names are parsed into `FixtureScenario` and validated by the fixture server. Fixture health is checked during harness startup.

Responsive checks should call `ctx.viewport().viewport(width, height, mobile).await?` so the same browser selected for the scenario determines viewport behaviour. Firefox uses WebDriver window sizing and compares the measured content viewport to the requested width and height, adjusting the outer window when necessary. Chrome retains CDP device emulation, including mobile mode. This is not Firefox touch emulation.

Chrome recording remains based on its existing CDP screencast. Firefox collects screenshot/diagnostic evidence but does not produce MP4 recording. Rust compilation and Windows Firefox execution of R326 remain pending.

## Test structure

- `crates/zipeek-e2e/src/harness/` starts owned Trunk/Chrome services, creates isolated scenario state, manages failure artifacts and in-process AI fixtures.
- `crates/zipeek-e2e/src/page_objects/` provides stable selectors and browser operations. Route cases are declared in `routes.rs`; data-dependent vocabulary routes receive valid HSK fixture parameters when navigated by matrices.
- `crates/zipeek-e2e/src/learner/` contains learner-oriented scenario page objects, including the Settings-to-Practice translation journey.
- `crates/zipeek-e2e/tests/` contains the actual Rust integration-test targets. `suites/` contains scenario functions used by those targets.
- The in-process Rust AI HTTP fixture is `src/harness/ai_fixture.rs`; do not reintroduce the retired Node fixture server.

## R324 typed navigation

Use `AppRoute::new("/browse")?` for a validated application route and `.with_query("query", text)?` for percent-encoded query parameters. Call `ctx.open(route).await?` to exercise full browser navigation, initial boot, or a bookmarked deep link. Use `ctx.open_route("/browse?query=%E4%B8%AD")` when a supplied encoded deep link is deliberately under test. Use `ctx.navigate_in_app(route).await?` for a hash-router transition that must preserve the existing document and browser history. Do not build `/#...` URLs with `format!` in new scenarios. Keep direct WebDriver navigation for externally controlled/invalid URLs, explicit redirect assertions, and `about:blank` initialization.

Route serialization handles root and `/zipeek/` origins. Full navigation waits for the application shell, while page objects and scenario assertions must still await their own destination-specific readiness. SPA navigation waits for the expected hash, not a fixed sleep. `AppRoute` owns query encoding; only `from_deep_link` accepts already encoded query strings.

## R307 source-level reliability checks

The R307 candidate hardens DOM probes against absent and empty elements, bounds previously unbounded translation polling, and requires target-specific page-object state after navigation and selection. The local Node suite has 822 checks, 709 passing and the same 113 failing names as R306; these checks do not substitute for actual Chrome execution. The page-object and integration-test Rust changes require compilation on the configured toolchain. See `RELEASE_NOTES.md` for the exact 21-file inventory and targeted browser acceptance commands.

## CSS lookup helpers (R312 WIP)

`WebDriverActions::find_css` and `find_all_css` perform immediate CSS lookups using Thirtyfour `find` and `find_all`; `query_single`, `query_first` and wait variants retain their query-builder semantics. `WebElementQueryActions` provides the same immediate lookups scoped to the originating element. E2E tests should avoid direct `find(By::Css(...))` and `find_all(By::Css(...))` outside `page_objects/driver_ext.rs`. Native XPath, ID and link-text searches remain appropriate. This refactor is source-level only pending Rust and Chrome acceptance.

## Chrome DevTools controls (R322 WIP)

One-off CDP operations in harness and tests use `zipeek_e2e::ChromeDevTools::new(driver)` rather than chaining `driver.cdp().emulation()` or sending raw command names at each call site. `viewport(width, height, mobile)` updates the emulated page viewport, not just the Chrome outer window. `media`, `reduced_motion`, `network_enabled`, `blocked_urls`, `block_patterns`, `init_script` and `remove_init_script` centralize the other supported commands. Each operation remains fallible and errors propagate to the scenario. The visual recorder retains its direct persistent CDP session for screencast events.

When a scenario temporarily changes emulation, restore its expected desktop metrics after the assertion, including failure paths. `writing-tailwind-responsive-layout` does this explicitly. The source-contract tests check that direct one-off CDP usage does not reappear outside the helper and recorder. Rust compilation and live Chrome acceptance for the wrapper are still required.

## Browser bootstrap and state

Most tests begin with a valid, full `zipeek.preferences.v1` JSON fixture from `crates/zipeek-e2e/fixtures/preferences-default.json`, matching application defaults. The harness seeds it only when absent in its isolated browser profile; first-visit tests can remove it deliberately. Avoid an empty `{}` fixture as a proxy for the serialized application document.

The shared route catalog distinguishes canonical route paths from fixture navigation paths. Type Pinyin, Character Builder and Word Mosaics require a vocabulary source, so their matrix navigation includes an HSK selector; otherwise the correct application state would be the source-selection message, not the exercise marker.

Browser-element handles may become stale during Yew rerender. The Browse live query uses one browser action for input dispatch and atomic DOM snapshots for result readiness. Do not hold WebElement references across asynchronous result updates.

## Translation acceptance

1. Save AI configuration through Settings or direct fixture-backed Settings UI.
2. Open Practice setup, select only Sentence Translation, choose a vocabulary source, direction, and five/ten/Endless length.
3. Launch `#/practice/session/<uuid>`, generate a question, assemble the canonical tile sequence and submit for AI evaluation.
4. Verify the scored outcome, navigation and ability to restore the same unfinished question after refresh or route change.
5. Verify finishing removes the temporary Practice storage record and shows the truthful unsaved-results notice.

`translation_fixture.rs` uses a Rust loopback AI server and checks deterministic cases. `ai_learner.rs` is ignored by default, requires `ZIPEEK_E2E_AI_ENDPOINT` and `ZIPEEK_E2E_AI_MODEL`, and exercises a live provider. The tile's `data-bank-tile` is its shuffled index; `data-solution-order` (with first/last flags) identifies one validated canonical solution. This seam verifies UI behavior, **not** independent linguistic accuracy.

Sentence generation requests up to ten distinct sampled vocabulary entries and accepts only a Chinese/English sentence pair; the app validates and constructs tile metadata itself. Pinyin is not requested from the model.

## Commands

```powershell
cargo check --locked -p zipeek-e2e --tests
cargo test --release --locked -p zipeek-e2e --test translation_fixture -- --nocapture
cargo test --release --locked -p zipeek-e2e --test browse_interactions -- --nocapture
cargo test --release --locked -p zipeek-e2e --test accessibility_matrix -- --nocapture
cargo make e2e-full
cargo make pack-e2e
```

The root and GitHub Pages base-path matrices are run by the full suite. Full-system tests can be slow because each suite owns browser and Trunk resources; do not replace scenario diagnostics with silent retries.

## Keeping evidence fresh

`cargo make e2e-full` now cleans `target/e2e/` **after compilation succeeds and before the test invocation**, so its evidence belongs only to that invocation. This is deliberately done once, not per scenario: multiple test binaries must not erase each other's results. `cargo test ... --test matching` does not automatically erase previous evidence. To reset before a targeted invocation, run `cargo make e2e-clean` first.

**Destructive behavior:** `e2e-clean` removes all entries within `target/e2e/`, including earlier recordings and evidence ZIPs stored there. Move any evidence that must be retained before invoking it. The command refuses a symlinked evidence root. It does not touch unrelated `target/` outputs.

`cargo make pack-e2e` packages **all** suite directories from the latest clean invocation. The direct CLI `cargo run --release --locked -p zipeek-cli -- pack-e2e` (without `--all`) still selects the newest completed suite only. That archive preserves each suite directory, preventing collisions between identically named scenarios. `--all` conflicts with `--run`. When tests were launched directly without a preceding cleanup, `--all` may include previous suite folders; run `cargo make e2e-clean` before the test invocation to prevent this. The combined archive records scenario counts and failure counts, but does not replace the per-suite `suite.json` files.

## Compact E2E evidence archives

`cargo make pack-e2e` excludes generated Trunk `dist/` directories and dependency/build caches, but **includes all available scenario MP4 recordings by default**, alongside manifests, failure evidence, screenshot timelines and known diagnostic files. Output filenames contain an UTC timestamp, for example `target/e2e/zipeek-e2e-all-20261110-123000.zip`, and `manifest.json` records `generatedAtUtc`, scenario counts and recording counts. To omit videos when preparing a smaller upload, run `cargo run --release --locked -p zipeek-cli -- pack-e2e --all --no-videos`. `--diagnostics` includes known browser and fixture JSON logs. `--output` can override the timestamped filename. Archive generation does not delete the original local recordings. The older `--include-videos` argument remains accepted for compatibility but is no longer necessary.

## Evidence and known gaps

The user confirmed 36 Rust E2E library unit tests, the accessibility matrix, all six Browse scenarios and one earlier live LM Studio translation scenario passed on Windows. That translation pass predates the latest UUID-backed Practice integration. The 2026-11-09 Windows run had one Course failure: a document-global alert selector matched two alert elements, including the startup shell alert. The R284 candidate scopes the lookup to main content; the corrected scenario has not yet been rerun in Chrome. No complete current-source Chrome run has been supplied. A library or source-contract check is not a browser pass. The default suite cannot objectively test audible sound, physical microphone access or model linguistic quality.

See [Testing](../../testing-quality/acceptance/testing.md), [E2E architecture](../../testing-quality/e2e/e2e-architecture.md), [recording](../../testing-quality/e2e/e2e-recording.md), and [current acceptance](../../project-management/current.md).

## Owned static server hosting

Root E2E suites run `trunk build` once, then serve its generated `dist/` through
`zipeek_static_server::StaticServer::run` on an ephemeral loopback port. The
harness owns the server handle and awaits `stop()` during suite cleanup; it no
longer invokes `trunk serve` or requires `ZIPEEK_E2E_STATIC_SERVER_BIN`.

Pages suites retain the dedicated Node fixture adapter and the `/zipeek/` Trunk
build prefix so their special route and asset behavior remains unchanged. The
independent static server can be tested without building the web application:

```powershell
cargo test --locked -p zipeek-e2e --test server_dependencies
```

This is an implementation description, not evidence of a successful Chrome run.

## R287 Cloud Match visual acceptance (pending)

The Cloud Match SVG visual checkpoint needs Chrome verification at desktop and mobile sizes. While a wave runs, match one pair and compare a remaining cloud's bounding rectangle before and immediately after the match; it must retain its position and direction without restarting animation. Verify one-way travel, off-screen wrapping, focused and selected blue indicators, Hanzi readability, pause and reduced-motion behavior. Existing source contracts do not replace browser acceptance.

## R288 Cloud Match continuity acceptance (pending)

The R288 rewrite uses one `requestAnimationFrame` controller for the Cloud Field and a single Yew reducer for authoritative game state. Run `cargo test --release --locked -p zipeek-e2e --test game_interactions -- --nocapture` on Windows after building the application. The `cloud-match-motion-continuity` scenario verifies that an unmatched cloud preserves its DOM node and motion phase while a different pair is matched, then checks clipped label layout. The motion must also be inspected in Chrome at short desktop and mobile viewports. The test does not establish visually smooth motion until it has passed against a compiled build; no compiled/browser verification is claimed here.

### R289 Cloud Match edge-entry regression (WIP)

The `cloud-match-motion-continuity` scenario now also checks that newly mounted clouds are hidden until their initial positions have been applied, that active cloud speeds differ within the Normal difficulty band, and that subsequent waves enter at the sides rather than appearing in the middle of the field. Video/browser acceptance is still pending.

### R292 Cloud Match acceptance (pending Windows Chrome execution)

The `game_interactions` suite now additionally verifies that pausing retains an unmatched cloud's motion progress, that the Resume button receives focus, and that the paused dialog closes after resuming. Its separate `cloud-match-continuous-replenishment` scenario selects Continuous mode, matches a pair, and asserts that the replacement appears without changing a surviving cloud's DOM identity, motion progress, or wave ID. The same suite retains its existing wave-entry and label-containment assertions.

The five original Cloud Match sky assets are stored under `crates/zipeek-web/static/cloud-match-skies/` and copied by the existing Trunk `static` copy-dir entry. Test the asset URLs in both root development and the `/zipeek/` deployed base path. A passing Node source contract does not establish that the images loaded in Chrome or that the continuous roster is visually smooth.

### R298 Cloud Match exit and label visual acceptance

`game_interactions` checks that two correct-match visual clones appear, turn into noninteractive successful exit tiles, fade within the expected short interval, and leave the underlying matched clouds inactive. The cloud label and unpositioned-spawn checks now select `[data-cloud-tile]` and `[data-cloud-label]`, not class names from the removed feature stylesheet. Inspect the actual MP4 recording for success animation and screen-size-related overlaps; source contracts are not sufficient evidence. Continuous mode must keep the surviving nodes and progress unchanged. Run `cargo test --release --locked -p zipeek-e2e --test game_interactions -- --nocapture` and package recordings with `cargo make pack-e2e`.

## R306 E2E script ownership (source candidate, 2026-11-10)

Browser-executable JavaScript invoked through Thirtyfour is owned by `crates/zipeek-e2e/src/js_scripts/`, which exposes feature-oriented `pub const` script bodies via `zipeek_e2e::js_scripts`. Call sites pass dynamic values through `execute`/`execute_async` argument arrays rather than formatting JavaScript source. Existing `harness::browser_scripts` and browser-probe fixtures remain available for shared harness semantics and fixture injection.

Shared integration-test scenarios live under `tests/suites/practice/`, `tests/suites/platform/`, and `tests/suites/learning/`. Integration-test entrypoints keep their existing test target names and explicitly refer to these modules; `tests/suites/mod.rs` is the smoke orchestration entrypoint.

The Node source-contract suite includes `scripts/e2e-extracted-scripts-contract.test.cjs`, which checks extracted JavaScript syntax, call sites, module paths, responsive Matching failures and Cloud Match argument passing. These checks do **not** replace pinned Rust compilation and managed Chrome E2E acceptance. The R306 candidate has not been compiled or browser-verified.

## Stable feature selectors (R314 WIP)

Shell brand coverage locates the actual home link through `[data-brand-link] a`, without coupling to its location inside a `header` or to its URL string. The browser test still checks both its visible name and actual `href="#/"` routing contract. The `data-brand-link` marker is on an inert wrapper because the Yew Router `Link<Route>` remains responsible for navigation.

Crossword E2E selectors use `data-crossword-setup`, `data-crossword-band="system-level"` and `data-crossword-tile` for element identity. `aria-label`, `aria-checked`, and `aria-pressed` remain on their controls as accessibility and state contracts; existing styling classes remain untouched. When changing these features, update the source component and its associated page-object constants together.

The corresponding executable Node source-contract tests verify selector ownership and accessible markup. They do not establish that a browser scenario or WASM build passes.

## Static Server native integration coverage (R318 WIP)

`crates/zipeek-static-server/tests/http_integration.rs` exercises the server
without Trunk, Chrome, or compiled WASM assets. It starts real listeners on
loopback ephemeral ports, checks root and Pages-prefix HTTP behavior, verifies
missing assets do not return the SPA index, tests shutdown and port reuse, and
launches the `zipeek-static-server` executable. Temporary fixture directories
are isolated per test.

Run `cargo test --locked -p zipeek-static-server --all-targets` on a machine
with the pinned Rust toolchain. This does not replace the full E2E harness or
a real Trunk asset build.

## R318 image and audio exercise acceptance (source checkpoint)

The E2E Rust HTTP fixture now recognizes the image-scene message contract in
`zipeek-ai-protocol::image_scene`, builds a deterministic English scene
containing the required subject names, and records the request as `image-scene`.
The Picture Vocabulary browser scenario configures both the owned placeholder
Image Server and the AI fixture, waits for image approval, selects an option,
checks grading, and requires approval again after regeneration. No AI model
files or GPU are needed for this scenario.

Managed Chrome uses its fake audio input device for successful recorder
scenarios. A script probe counts `getUserMedia` requests, media tracks
acquired/stopped, and blob URL revocations; the test checks stop, playback,
discard, and retry through the real `zipeek-browser::MicrophoneRecorder`.
The denied-permission scenario still explicitly overrides `getUserMedia` to
reject with `NotAllowedError`. Listening practice checks a valid Mandarin
speech-service invocation and repeated playback of the same content.

Run targeted acceptance after compiling on Windows:

```powershell
cargo check --locked -p zipeek-e2e --all-targets
cargo test --locked -p zipeek-e2e --test exercise_interactions -- --nocapture
cargo test --locked -p zipeek-e2e --test speaking_interactions -- --nocapture
cargo test --locked -p zipeek-e2e --test practice_answer_options -- --nocapture
```

Fake-device recording establishes browser service integration, not acoustic
quality, real microphone permission behavior, or native audio playback quality.
The browser translation scenarios still use the Rust HTTP fixture adapter;
migration of its scenario controls/request capture to the owned `AiServer`
remains open. Rust compilation and Chrome acceptance were not run in this
source-only checkpoint.

## Firefox smoke coverage

The managed harness defaults to Chrome. To test Firefox from PowerShell, install Firefox and run:

```powershell
$env:ZIPEEK_E2E_BROWSER = 'firefox'
cargo test --release --locked -p zipeek-e2e --test firefox_smoke -- --nocapture
```

The `BrowserKind` selection is read at suite startup. The test records its browser in `browser.txt`, `scenario.json`, and `suite.json`. Firefox uses geckodriver through `WebDriver::managed()` and a separate initialization path that persists test preferences and silent speech in browser storage before reloading the app. Smoke coverage includes browser identity, core routes, history transitions, and responsive writing layout. Chrome continues to use CDP for video, pre-navigation scripts, and network controls; Firefox does not produce CDP video. The broad Chrome suites remain Chrome-specific where they depend on interception, CDP emulation, or pre-navigation script injection. Such dependencies must not be silently treated as supported on Firefox.

## Structured meanings browser acceptance (R329)

Browse interactions now cover lazy expansion of an individual word into ordered meanings with stable `data-browse-sense-panel` and `data-meaning-counter` selectors. Word Discovery covers previous/next and word-switch selection reset. Run the suite against a freshly regenerated DuckDB bundle and verify that the displayed per-sense region and usage labels never leak to unrelated meanings. Rust/WASM and Chrome/Firefox execution of these new scenarios is pending.

### Province Quest R332 WIP acceptance

`province_quest_gameplay.rs` now declares separate Easy, Medium and Hard
managed-browser scenarios. Each completes all ten rounds, alternating region
selection and three-Hanzi identification; wrong Hanzi answers must enter
review and lock choices. Hard selects a valid parent province (Guangdong) before
map rounds. Reading support is exercised with explicit reveal help. The
highlighted region must expose a descriptive accessibility label without
exposing the Hanzi name until review.

Run `cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture`
with Chrome, then repeat using the harness's Firefox selection. These tests are
source-only additions in R332; neither browser suite has been run in the
packaging environment.

## R353 standard full journey, source-implementation checkpoint

`crates/zipeek-e2e/src/journeys/standard.rs` supplies actual UI actions for the no-pedagogy full-run stage. It reuses existing selector/page-object support, drives a fixed five-round combination of MCQ and four-pair matching, and requires an exact `14 / 14` first-attempt summary with zero mistakes. Subsequent steps create a browser-persisted Word Set, search the dictionary and writer, open Province Quest and a Pinyin lesson, configure local AI/image fixtures, and score an image-vocabulary response and a sentence-translation response.

Fixture-backed results verify application/provider contracts only, not a real external image or translation service. R353 does not yet claim that the Settings controls turn on the two practice modes. No verified DuckDB OPFS no-evidence readback exists in the test; the standard test remains explicitly ignored by default and fails with `full-run-prerequisites.txt` rather than reporting a false pass. The pedagogical variant remains unchanged and cannot grant Golden Dragon.

To compile and invoke it on Windows, run:

```powershell
cargo check --locked -p zipeek-e2e --tests
cargo test --release --locked -p zipeek-e2e --test full_run_standard -- --ignored --nocapture
```

The second command intentionally returns a failing prerequisite status until remaining acceptance gates are addressed. Inspect `full-run-screenshots/manifest.jsonl` and the captured PNGs to diagnose earlier failures. Rust/browser execution has not been performed in this source-only checkpoint.
