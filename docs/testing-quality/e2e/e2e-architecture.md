# E2E architecture and evidence contract

## Goals

- One independent Chrome WebDriver session per scenario. The servers and deterministic AI fixture may be shared per suite, not the browser state.
- Capture and preserve a video and named screenshots for every scenario, including application initialization failures.
- Write all runs beneath the repository root `target/e2e`, not `crates/target/e2e`.
- Collect evidence before browser shutdown. Failure details must not be replaced by recorder or cleanup errors.
- Continue executing subsequent scenarios after an individual failure, then report aggregate suite failure.
- Keep existing `HarnessError`, `HarnessResult`, `Harness`, `ScenarioContext`, and Thirtyfour 0.38.1 with `cdp-events`.

## Runtime flow

1. Start the test suite's HTTP and fixture servers and create `target/e2e/<suite>-<pid>-<timestamp>`.
2. For each scenario, start a fresh Chrome session and a unique artifact directory.
3. Connect a CDP event stream and start screencasting **before** initializing browser state, navigating to the application, or awaiting dictionary readiness.
4. Reset the fresh browser and initialize diagnostic instrumentation. Wait for one of the explicit database states. `failed` becomes a named initialization failure; absence of a terminal state is bounded by a 45-second deadline.
5. Capture a ready checkpoint, execute the scenario, then capture the final page and diagnostics, regardless of result. Scenario panics are reported without skipping teardown.
6. Stop screencast, drain frames, encode MP4 with FFmpeg when installed, then close that scenario's browser.
7. Write `scenario.json` with phase, result and evidence presence. Browser cleanup and strict recording errors receive their own failure phases. Run all remaining scenarios despite failures.
8. Write `suite.json` and `summary.txt` at the run root; fail the overall test target if any scenario failed. If server startup fails, still write a suite failure manifest.

CDP frame events must be acknowledged regardless of whether individual frames can be stored. The capture limits are for safety; the recorder must clearly signal incomplete capture if a limit is hit. Variable-duration frame timing is preferred. The recording is diagnostic evidence, not a guaranteed full-frame-rate video.

## Evidence layout

Each run is stored under the repository root `target/e2e` and has a `suite.json`, `summary.txt` and human-readable `review.md`. Each scenario has `scenario.json`, `agent-feedback.md`, a timeline of named checkpoints, and conditional browser, failure and video artifacts.

Chrome CDP JPEG frames stream directly to FFmpeg, rather than being retained as numbered image files. The recorder writes `video/recording.mp4` and `video/status.json`; FFprobe reports validation where available. See [video and feedback evidence](e2e-video-recording.md) for the exact current environment controls, file layout and limitations.

The scenario manifest explicitly distinguishes execution, assertions and recording. An otherwise successful test may still have incomplete video evidence. Strict evidence mode fails a passing scenario if the recorder or video validation fails; it never hides a prior application failure.

## Test tiers

| Tier | Dependency | When to run |
| --- | --- | --- |
| Rust unit and contract tests | None | Every code change |
| Browser fixture smoke | Local app, Chrome, bundled dictionary and deterministic AI fixture | Every release and CI |
| Full browser journey suite | Local app and deterministic fixture | Before acceptance |
| Live-model AI acceptance | Explicit local LM Studio endpoint and loaded model | Opt-in; report model/version |
| Fault injection | Local fixture with delayed/failing responses | CI and recovery changes |
| Visual evidence review | Video, checkpoint images and scenario metadata | After every failed run |

## Route and module regression coverage

The root route matrix executes registered routes in separate Core, Practice, and
other-feature Chrome scenarios, plus a redirect/404 scenario. The same matrix
also tests the GitHub Pages base path. The individual scenarios have separate
browser profiles and evidence, so a failure in Practice routes does not suppress
checks for Settings, Games, or the other groups. The pure Rust matrix tests
verify that every declared route is covered and that each registered route is
assigned to exactly one root scenario group.

The Pages suite registers all 21 existing scenarios directly with the harness.
`Harness::run` accumulates failures while isolating each browser session;
`Harness::finish` reports the aggregate outcome. Do not wrap calls in a
fail-fast result chain. Page-object helpers live in the E2E library
`src/page_objects/`, not under individual integration targets. The root Node
test `scripts/e2e-module-contract.test.cjs` checks that explicit Rust `#[path]`
module references resolve and that test targets do not duplicate shared page
objects. This is a static check, not a replacement for Cargo compilation.

## Stable DOM selector contract

Domain and workflow roots should expose an existing stable `data-*` marker. Add a
specific marker to a control only when an E2E journey must locate or assert it;
do not instrument decorative wrappers. Prefer stable domain IDs over user-visible
labels, CSS classes, or translated text. Keep domain values separate from action
names and observable state. Native accessibility roles, labels, `disabled`, and
`aria-pressed` remain the authoritative interaction/accessibility semantics.

- Expanded Grammar: `data-grammar-course-id`, `data-grammar-pattern-index`,
  `data-grammar-practice-phase` (`recall`/`recognition`), `data-grammar-reveal`,
  `data-grammar-choice` (index within the current authored question),
  `data-grammar-choice-state`, `data-grammar-practice-feedback` and
  `data-grammar-previous`/`data-grammar-continue`.
- Sentence Transformation: `data-item-id`, `data-transformation-method`,
  `data-transformation-status`, `data-transformation-bank-tile` (source-bank
  index), `data-transformation-selected-tile`, `data-transformation-tile-position`,
  `data-transformation-undo`/`data-transformation-reset`, and
  `data-transformation-feedback`. These indices are question-local, not stable
  syllabus identities.
- Transformation setup: `data-transformation-pattern-search`,
  `data-transformation-pattern-status` (`valid`/`unsupported`),
  `data-transformation-pattern-all`, `data-transformation-answer-mode`,
  and `data-transformation-answer-option`.
- Cloud Match: `data-cloud-match-setup`, `data-cloud-match-eligibility`,
  `data-cloud-match-start` and `data-cloud-match-start-state` with readiness
  values `loading`, `error`, `insufficient`, or `ready`.

These attributes are a public *test selector* contract, not an application state
transport. Do not read domain decisions back from DOM attributes, expose answers
through them, or use an E2E hook to imply a learning result is verified.

## Practice exercise viewport and service selectors

Practice setup uses `[data-practice-exercise-scroll]` as the one desktop vertical scroll owner when choosing or configuring exercise types. The wizard footer and Choose/Configure tabs remain above the list, outside its scrolling region. The enclosing `[data-component="main-content"]` does not scroll on desktop. Browser acceptance must assert that the list has overflow, can reach the last tile, and that the main scroll position remains zero. On mobile, keep natural page scrolling.

Exercise tiles expose `data-external-services` (`none`, `ai`, or `ai,image-generation`) and named `[data-exercise-service]` labels. These describe required external generation services, not their readiness or permission to send private data. The sentence-translation tile requires AI; picture vocabulary requires both AI and image generation. Do not mistake local browser speech/microphone features for external generation services.

## Coverage priorities

P0: initial database loading, explicit error state, retry, stale readiness, server exit and evidence of failures before scenario execution.

P0: deterministic Sentence Translation generated question, tile selection, check, navigation, reload, retry, invalid generation and absence of model configuration. Run the fixture path without access to external model accounts.

P0: session restoration and completion for Matching, Multiple Choice, Listening Choice and Character Hunt; input/keyboard equivalence and no false mastery publication.

P1: authenticated or private storage journeys with isolated browser profiles; OPFS failure and recovery; dictionary checksum/adapter error reporting.

P1: reduced motion, responsive widths, focus, semantics, keyboard navigation, and screen-reader labels across the Practice setup and game views.

P2: optional real AI model responses, image loading/approval and cross-platform Chrome/FFmpeg behavior.

## Acceptance gates

- The first failing database-initialization scenario produces `scenario.json`, `failure.txt`, screenshot when possible, and video or an explicit recording failure record.
- The following scenario still runs with a fresh profile and its own evidence directory.
- A deterministic AI fixture journey appears as a registered browser test, not merely an unused function. Run it with `cargo test -p zipeek-e2e --test smoke deterministic_ai_practice --locked -- --nocapture --test-threads=1`.
- An ordinary multi-scenario run produces separate recordings for each scenario, plus a suite-level summary.
- Artifact locations resolve under the repository root `target/e2e` on Windows and Linux.
- The default upload bundle includes MP4 when available and falls back to raw JPEG frames when encoding was unavailable. Raw AI requests remain opt-in; manually inspect screenshots, videos and text before sharing.
- `cargo check -p zipeek-e2e --tests --locked` and actual Chrome/FFmpeg runs pass before labeling the release verified.

## Limitations

The source-level improvements do not establish a successful Chrome/CDP/FFmpeg execution until the local environment runs them. The current frame recorder may reach its size/frame limits in long scenarios; incomplete video should be explicitly reported and the final failure screenshot retained. A CDP stream can omit unchanged frames, so frame timing is not equivalent to screen capture at fixed FPS.

## Scenario quality and review

- Treat E2E tests as production code. Use small domain-focused scenarios with observable acceptance assertions, intentional setup and independent browser state. Avoid giant catch-all tests or repeated raw selector chains.
- Reuse existing typed harness APIs, stable semantic selectors or minimal `data-testid` hooks. Do not expose private answer keys or serialized internal round state in the DOM solely for tests.
- Prefer explicit bounded waits on observable conditions to arbitrary sleeps; ensure independent fixtures and cleanup. Isolate any necessary injected JavaScript behind short typed helpers and validate dynamic script inputs.
- Verify success, failure, keyboard, focus, navigation and responsive cases when relevant. Keep screenshots, videos and diagnostics, but do not equate their creation with visual review.
- Report exact suites, environment, artifacts, failures, skipped checks and pre-existing failures. Do not weaken meaningful assertions to obtain a passing result.

## exercise-type and Pinyin course scenarios

`cargo make e2e-exercise-types-and-courses` runs the focused exercise and
course suites. `exercise_type_journeys.rs` checks that all sixteen authored
Practice exercise definitions are selectable in Advanced mode and have an
accessible, individually selectable configuration tab. It also verifies the
exact eight-exercise Quick subset and runs isolated authored interactions for
Fill in the Blank, Audio Dictation, Sentence Builder, Sentence Transformation,
and Spot the Mistake. These journeys assert input gates, answer feedback and
selection recovery; they do not constitute verified mastery evidence or
full writing-stroke/microphone acceptance. Other established exercise targets
continue to cover Matching, Multiple Choice, Character Hunt, Type Pinyin,
Character Builder, Word Mosaics, Speaking, Picture Vocabulary, and translation.

`course_interactions.rs` validates the six Pinyin lesson identities, links,
sequential completion and persisted overview state. Grammar courses retain
their assessment, guided-tile, incorrect-answer and review assertions.
The Chrome suites remain pending execution until the supported toolchain and
browser fixtures are available.

## deterministic game completion and authored feedback

`cargo make e2e-ai-games-and-exercise-completion` runs the fixture-backed
`ai_game_journeys.rs` and expanded `exercise_type_journeys.rs` suites. The
shared Rust fixture recognizes the actual Sentence Relay and Dialogue Quest
completion prompts and returns four stable, validated game rounds. The browser
journeys test wrong-answer feedback, corrected answers, all four completions,
restart, and a visible server failure. AI requests remain bound to the local
loopback fixture and contain no real learner records.

Authored Practice checks now include traditional-character cloze answers,
incorrect-answer feedback and continuation, a full silent-TTS dictation
playback/check/continue flow, tile reconstruction and review for Sentence
Builder and Sentence Transformation, and detection-versus-repair feedback
for Spot the Mistake. These tests do not grant mastery or claim coverage of
Hanzi stroke tracing, physical microphones, or real AI inference.

All these tests are source-authored pending an actual locked Rust build and
Chrome execution; fixture unit checks do not replace browser acceptance.

## authored Practice session completion

`cargo make e2e-authored-session-completion` runs five isolated end-to-end journeys through the saved UUID Practice session lifecycle for Fill in the Blank, Audio Dictation, Sentence Builder, Sentence Transformation and Spot the Mistake. The test checks the actual number of scheduled rounds, accepted answers for four formats, intentional rejected dictation input and the playback prerequisite on each round. It then validates the final local-session completion count, skipped count, completion percentage and learning-history notice. After the history operation removes the temporary session, reloading its URL must show `[data-component="practice-session-unavailable"]` with a `not-found` error state and provide navigation back to Practice setup. It must not resurrect finished steps. This differs deliberately from standalone legacy MCQ summary/restart behavior.

These journeys do not provide evidence for Mandarin microphone transcription accuracy, handwriting gesture matching or browser audio audibility. The existing speaking and writing suites cover those boundaries to the extent that test fixtures permit. Rust compilation and Chrome runs are required before these scenarios count as passing.

## Dictionary preflight and deep-link readiness (R531 WIP)

The E2E startup checks the installed `static/hanzi.manifest.json` and verifies that its single compressed DuckDB file exists with the declared positive byte size, before launching Trunk. Once Trunk completes, it checks the corresponding output file in `dist`. This is a fast presence/size gate, not a replacement for the separately maintained SHA-256 asset-integrity audit or browser dictionary initialization. Generated dictionary binaries remain excluded from source handoffs.

Registered deep links wait for their route-specific marker; unregistered deep links retain the app-shell readiness fallback. Full-screen sessions and lessons need not render a global app shell. The Time Weaver entry route initially presents its setup screen, while gameplay tests separately assert the game marker.
