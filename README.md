> **R96 source checkpoint (2026-11-04):** Practice now disables filtered dictionary exercise launch until the matching vocabulary selection has a successful nonzero database count. See `docs/process/CURRENT.md`. This is not the multi-filter Matching/MCQ round-engine adapter; no database regeneration is needed. Native Rust/WASM and browser validation remain pending.

> **R91 source checkpoint:** Schema changed to add `sense_display` and normalize `sense_relations`. If upgrading from R90, run `cargo make db-generate`, `cargo make db-install` and `cargo make db-browse-check` after the Rust checks. The distributable ZIP intentionally does not include the old DuckDB or manifest; it must be installed from the current generator. The three old row-count test failures remain deferred.

> **R89 database rebuild required:** This checkpoint introduces the `sense_relations` table, but the included `crates/web/static/hanzi.duckdb` is still the R88 artifact. On Windows run `cargo test --workspace --locked`, `cargo check -p zipeek --target wasm32-unknown-unknown --locked`, `cargo make db-generate`, then `cargo make db-install` before `cargo make serve`. Hard-reload with browser cache disabled to ensure the new database is loaded. Existing AI classification checkpoints and dependency versions are unchanged.

# Zipeek

![rustc](https://img.shields.io/badge/rustc-1.97.1-blue.svg)
![CI](https://github.com/Jozefpodlecki/zipeek/actions/workflows/ci.yml/badge.svg)
![wasm-bindgen](https://img.shields.io/badge/wasm--bindgen-0.2.300-blue.svg)

A client-side Chinese word query and practice app built with Rust, Yew, and
WebAssembly. The app has a responsive slate/blue navigation shell, saved dark/light
theme and landing tiles. A pregenerated DuckDB dictionary is present; browser
initialization and typed read-only queries work through DuckDB-WASM. Matching
and MCQ are playable in Endless and Fixed modes, including mixed exercise types.
Browse provides debounced automatic search, immediate filter updates, manual refresh, and compact list results (browser validation still required); Home shows a database-backed Word of the Day, Random shows a selectable dictionary word, and personal Word Sets have a dedicated `#/word-sets` management route. Pages builds use `/zipeek/`.

## Practice setup architecture

The approved exercise-first Practice setup is specified in
[`docs/product/PRACTICE_SETUP_V2.md`](docs/product/PRACTICE_SETUP_V2.md).
R80 adds a validated, pure `PracticeDraft` in `zipeek-exercises` for exercise
selection, mutually exclusive vocabulary sources, multi-HSK/category/part-of-
speech filters, and conditional options. The visible Practice setup still uses
the existing route-local controls; the unified builder and saved profiles are
not implemented yet.

## Run the app locally

You need Rust/rustup, Node.js with npm, cargo-make, and Trunk on your PATH.
The repository's rust-toolchain.toml selects Rust 1.97.1 and the WebAssembly
target. A generated dictionary is already included, so you can start the app
without regenerating it.

### First run

Open PowerShell and run these commands in order. Adjust the repository path if
you cloned it somewhere else:

```powershell
Set-Location C:\repos\zipeek
Set-Location crates/web
npm ci --ignore-scripts
Set-Location ../..
cargo make serve
```

Wait for Trunk to finish building, then open **http://localhost:1420** in your
browser. It does not open automatically. To try the quiz, choose **Practice**,
keep the default **Hanzi to meaning** and **Endless**, then select **Start**.
You can select several exercise-type tiles to mix matching and MCQs. Types are
chosen randomly without immediately repeating the previous type when multiple
types are selected.

Leave the terminal running while you use the app. Press **Ctrl+C** in that
terminal to stop the server. Rust/UI changes rebuild automatically.

### Start it again

Once npm dependencies are installed, run from the repository root:

```powershell
Set-Location C:\repos\zipeek
cargo make serve
```

### Verify the local AI model before full classification

With the model loaded in LM Studio and its local server running at
`http://127.0.0.1:1234`, run this from the repository root:

```powershell
cargo make db-ai-smoke
```

This sends **one** real streamed classification request through the same
AiMux 0.6.0 client and strict taxonomy validator used by `db-full`. It does
not import the HSK dataset, create a database, or modify the classification
cache. A pass prints the accepted PoS and categories. A missing final text
response fails with an explicit diagnostic (instead of a JSON EOF error).

`cargo make db-full` also runs this smoke check automatically before starting
new inference when there are pending senses. If all senses are already valid in
the cache, it skips the model request and proceeds to artifact verification.
The smoke check is **live and opt-in** when run independently: ordinary
`cargo test --workspace --locked` never requires LM Studio. The automated mock
HTTP tests cover streamed JSON aggregation and empty responses offline.

AI settings (`ZIPEEK_AI_MODEL`, `ZIPEEK_AI_ENDPOINT`,
`ZIPEEK_AI_TIMEOUT_MS`, `ZIPEEK_AI_RETRIES`) are shared with `db-full`.

### Build the complete AI-enriched dictionary

With the chosen model loaded and the LM Studio local server running, execute **one command** from the repository root:

```powershell
cargo make db-full
```

This resumes sense classification from a model-specific JSONL cache, completes all
pending classifications, generates a new normalized DuckDB bundle **with the AI
labels applied**, validates its checksums and Browse/enrichment tables, and installs
the bundle with backup and rollback. If LM Studio fails or any sense remains
unclassified, the installed dictionary is **not replaced**. A complete first run
requires model inference for all dictionary senses and can take substantial time.
Subsequent runs reuse valid cached classifications. The previous dictionary stays
available while classification is running.

The classifier makes up to three attempts per sense when AI output is empty,
malformed JSON or violates the approved taxonomy. These are separate from
`ZIPEEK_AI_RETRIES`, which applies to AiMux transport requests. Exhausted
attempts stop generation and preserve completed JSONL checkpoints. R55 keeps
`sense-classification-v3` cache compatibility.


To launch Trunk after the full workflow, use `cargo make db-full-dev` instead.
To change the model or endpoint, set environment variables before invoking the task:

```powershell
$env:ZIPEEK_AI_MODEL = "qwen3.5-9b-uncensored-hauhaucs-aggressive"
$env:ZIPEEK_AI_ENDPOINT = "http://127.0.0.1:1234"
$env:ZIPEEK_AI_TIMEOUT_MS = "300000"
$env:ZIPEEK_AI_RETRIES = "1"
cargo make db-full
```

The cache lives in `target/db-workflow/classification-cache/`, keyed by model.
It is not included in source ZIPs; retain the local `target` directory to resume
inference after applying later releases. This workflow does not install or launch
LM Studio, download models, or change the pinned HSK/CC-CEDICT source data.

### Regenerate the database and start the app

Use this instead of serve when you explicitly want to regenerate and install
the dictionary before starting the server:

```powershell
Set-Location C:\repos\zipeek
cargo make dev
```

It generates a fresh verified bundle, installs the complete bundle in crates/web/static,
backs up the previous bundle, then starts the same server. It does not download
new dictionary sources. Native generator/build prerequisites are also required;
see [generator dependencies](docs/architecture/schema/generator-deps.md).

| Command from the repository root | What it does |
| --- | --- |
| `cargo make` | List available tasks |
| `cargo make serve` | Run the app with the installed database |
| `cargo make dev` | Fast offline regeneration, install, then run the app (no AI inference) |
| `cargo make db-ai-smoke` | Test one real LM Studio streamed classification without database/cache writes |
| `cargo make db-full` | Classify all senses with local AI, generate an enriched bundle, validate, and install |
| `cargo make db-full-dev` | Run `db-full`, then start Trunk |
| `cargo make db-check` | Validate inputs/import in memory; write no database |
| `cargo make db-generate` | Generate a fresh candidate; print its absolute location |
| `cargo make db-install` | Verify/install the latest candidate with a backup |

Generated candidates and backups are in target/db-workflow; the installed
database is crates/web/static/hanzi.duckdb. Trunk rebuilds do not regenerate it.
[Detailed workflow and direct generator commands](docs/architecture/schema/regeneration.md).

### If startup fails

- **`cargo make` or `trunk` is not found:** check that cargo-make and Trunk are
  installed and their executables are on PATH. Reopen the terminal after setup.
- **Missing npm packages:** stop the server and repeat npm ci --ignore-scripts
  from crates/web/, then return to the repository root and run cargo make serve.
- **Port 1420 is already in use:** stop the previous server with Ctrl+C before
  starting another one.
- **Generator says the output directory already exists:** use cargo make dev
  or db-generate, which choose fresh directories. Direct generator output must
  point to a new directory; it never overwrites an existing one.

## Dictionary data

The generated dictionary is loaded read-only in the browser; see
[data sources, licensing and runtime verification](docs/architecture/schema/DB.md).
The primary input is [jelleverheyen/hsk-vocabulary complete.json](https://github.com/jelleverheyen/hsk-vocabulary/blob/a66fd30b9580da2c2af7eb19e4b9d8099a29c061/complete.json)
at commit `a66fd30b9580da2c2af7eb19e4b9d8099a29c061`, declared MIT. Preserve its
copyright/permission notice and upstream attribution. CC-CEDICT is an empty-gloss
fallback; the primary dataset also credits CC-CEDICT for definitions.

Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

Generated dictionary data derived from CC-CEDICT is distributed under
CC BY-SA 4.0, with its transformations documented. Application code retains its
own licence.

The [generated database](crates/web/static/hanzi.duckdb) contains 12,100 readings.
Its [integrity report](crates/web/static/hanzi.manifest.json) records size, SHA-256,
schema, indexes and verified inputs; [attribution](crates/web/static/ATTRIBUTION.md)
and notices are included. Browser/WASM integration is implemented.

From the repository root, `cargo run -p db-gen --locked -- check` verifies/imports in
memory. `cargo run -p db-gen --locked -- generate --output target/reviewed-dictionary` exports to
a fresh directory and refuses existing output. See [regeneration](docs/architecture/schema/regeneration.md)
for manual review/replacement and reproducibility checks. Builds copy the existing
artifact; they never generate it automatically.

The root Cargo.lock is the workspace lockfile. Use Cargo's --locked flag for
validation; Trunk enforces it automatically. From crates/web/, trunk serve watches both
the web app and shared core sources. Trunk manages the configured Tailwind and
wasm-bindgen tools. Run npm ci --ignore-scripts from crates/web/ before Trunk: A06's
pre-build hook packages pinned DuckDB-WASM/Arrow assets from node_modules for
local serving. No CDN, database generation or package install occurs at build time.

## Credits

- <a href="https://www.flaticon.com/free-icons/asia" title="asia icons">Asia icons created by Freepik - Flaticon</a>
- https://github.com/drkameleon/complete-hsk-vocabulary
- https://cd.hwxnet.com/


## Experimental sentence translation (work in progress)

Practice has tabbed setup for Matching/MCQ, Sentence Translation, and Speaking.
The Sentence Translation exercise at `#/practice/translation` can draw vocabulary
from an HSK 2.0/3.0 level, a locally saved Word Set, or one of three sample topics
(Travel, Everyday life, Shopping). It uses up to 24 distinct dictionary words per
request, with validation that the generated Chinese sentence contains at least
one selected word. In LM Studio, start the OpenAI-compatible local server,
load a model, and enter its model identifier in the exercise configuration. The
initial API URL is `http://127.0.0.1:1234/v1/`. CORS must permit the origin of
the web app. Hosted HTTPS pages may additionally be affected by browser
local-network or mixed-content restrictions; a successful connection in the
local development server does not establish GitHub Pages compatibility.

This is a provisional independent exercise: it does not yet participate in the
legacy deterministic Matching/MCQ mixed-session URL or scoring. Model responses
are advisory. No generated database schema or assets were modified.

### Local classification preview and bounded run

```powershell
cargo run -p db-gen --locked -- classify-plan --model YOUR_MODEL --cache target/classifications.jsonl
cargo run -p db-gen --locked -- classify --model YOUR_MODEL --cache target/classifications.jsonl --limit 10
```

Requires an independently running LM Studio server with the selected model available. The user confirmed one successfully streamed local classification with LM Studio; model quality and full-batch results still require review.


## Personal Word Sets and practice

Browse and the dedicated Word Sets page support personal Word Sets backed by localStorage, with add/remove,
rename/delete and validated JSON export/import. Practice Setup can use a selected
set in Matching/MCQ; shareable URLs include up to 128 sorted word IDs in
`set_ids`, so a recipient need not have the original localStorage. Shared URLs
require the same generated dictionary. Invalid ID snapshots do not silently
widen to unrestricted practice. Runtime browser validation of this workflow is
still outstanding. Historical implementation notes are preserved in
[completed outcomes](docs/process/completed/2026-11-04.md).

## Verify a source update

From the repository root, run `cargo fmt --all -- --check`,
`cargo test --workspace --locked` and
`cargo check --target wasm32-unknown-unknown -p zipeek --locked`.
Install web assets with `Set-Location crates/web; npm ci --ignore-scripts`
before running browser tests or Trunk. Return to the root with `Set-Location ../..`.
To validate installed-dictionary rollback without generating a database, run
`node --test scripts/db-workflow.test.cjs` from the root.

Local `.unpack-backups/`, `target/`, `crates/web/node_modules/` and
`crates/web/vendor/` are not source inputs and should be excluded from source
ZIPs. No `CHECKPOINT-Rxx.md` files are needed; maintained status is in
`docs/process/CURRENT.md`, open work in `TASKS.md` and recorded outcomes in
`docs/process/completed/`.

### Browse database readiness

The included `crates/web/static/hanzi.duckdb` is a legacy single-table artifact; normalized Browse search needs a freshly generated dictionary. From the repository root, run `cargo make db-browse-check` to inspect the installed manifest. If the missing-table diagnostic appears, run `cargo make db-generate` then `cargo make db-install` using the pinned native toolchain, and rerun the check. The preflight confirms table names only; validate the resulting search and personal-set practice flow in Chrome. Do not manually edit generated `.duckdb` files.

### Experimental sentence translation (R54)

Sentence Translation now uses a shuffled English word bank generated with the Chinese sentence. Select tokens in order or drag them to the answer lane; START and END anchors distinguish its boundaries. Drag selected tokens to reorder or back into the bank, or click to remove them. All tokens must be used before Check translation can submit the answer to the existing AI evaluator. This is experimental; answers and evaluation are model-generated and may be linguistically incorrect. The new word-bank schema requires a compatible local model and is not backward compatible with old two-field sentence output.

For offline contract tests use `cargo test -p zipeek-ai --locked`. For WebAssembly typing use `cargo check --target wasm32-unknown-unknown -p zipeek --locked`, then test actual drag/drop, keyboard and touch in Chrome. The existing Rust/Thirtyfour E2E harness now includes an isolated AI fixture and an ignored, opt-in live Translation simulated learner (`crates/e2e/tests/ai_learner.rs`). It uses learner-visible word-bank tiles only. See [E2E documentation](docs/testing/e2e.md) for the local LM Studio command and deterministic test coverage.

Native db-gen now emits timestamps and per-sense completion/ETA data in **future runs**; a process already running an earlier version keeps its existing logging configuration. The sense classifier remains on the identical `sense-classification-v3` contract so an in-progress classification cache is not invalidated by this release.

### Deferred AI classifications (R56)

`cargo make db-full` continues past individual empty/invalid model outputs after
bounded retries. Deferred senses are attempted once more at the end of the
batch. Check the per-model `target/db-workflow/classification-cache/*.failures.jsonl`
report for unresolved senses. The installed database is **not replaced** while
`classify-plan` reports pending senses. Re-running the command reuses the valid
cache and retries unresolved senses. R56 retains the v3 prompt identity, so
previous successful R55 classifications remain reusable.

### Sentence translation input and in-flight requests

The experimental word-bank accepts click/tap selection, native drag/drop, and keyboard-accessible left/right reordering buttons. Changing the Word Set, LM Studio model or endpoint while a request is pending discards its eventual stale response. This does not cancel an already transmitted network request, but prevents the old result from replacing the current exercise.

### Exercise domain (R60)

`crates/zipeek-exercises` now owns versioned exercise definitions (learner-visible description, input/output contract, scoring authority), concrete exercise instances and a quiz-session ledger. Matching/MCQ are bridged into the shared ledger; Sentence Translation follows later. This foundation does not change the active AI classification workflow or its cache. Run `cargo test -p zipeek-exercises --locked` for the new pure domain tests.

### Experimental bidirectional sentence translation (R62)

The Sentence Translation page supports Chinese→English, English→Chinese, or **Mixed**. Mixed randomly selects the initial direction then alternates after each accepted generated sentence. A language-specific word bank supplies every required tile and is independently shuffled in the browser; learners can click, drag or use accessible arrow controls to arrange it. The AI returns a bilingual pair, and evaluation checks the learner's translated meaning in the requested direction. This feature still uses the local LM Studio model and is not yet part of the shared scored quiz session or persistent proficiency evidence. Source-only sentence generation and evaluation prompts changed to v3; offline dictionary classification uses unchanged sense-classification-v3.

### Visual themes and translation feedback (R63)

`crates/zipeek-themes` resolves topic hints into a session visual theme and
exposes an asynchronous bundled-background provider. Sentence Translation uses
this provider for its decorative background; all topics currently share the
existing generic `public/background.webp` (no image generation API yet).
Translation feedback appears only after evaluation; correct answers get a green
panel, incorrect answers a red panel with the reference sentence, and Continue
advances either result after acknowledgement.

To verify on a Rust-equipped machine:

```powershell
cargo fmt --all -- --check
cargo test -p zipeek-themes --locked
cargo test --workspace --locked
cargo check -p zipeek --target wasm32-unknown-unknown --locked
```

### Experimental Sentence Translation sessions (R64)

The standalone Sentence Translation screen now registers each successfully
AI-generated exercise in a page-local `zipeek-exercises::QuizSession` and records
one result after a successful AI evaluation. It shows completed/correct/mistake
totals. Changing the Word Set, model, endpoint or direction starts a new local
session. No session history or per-word proficiency evidence is persisted yet.

Speaking currently has a browser-only, unscored recording preview and three
context-aware sentence targets. A live SVG waveform displays microphone amplitude;
recordings are not uploaded or evaluated. See `docs/architecture/audio-recording.md`
for capture, permissions and teardown requirements.

### Experimental speaking recorder (R65)

Open Practice → **Speaking recorder (preview)** or `#/practice/speaking`.
Choose one of three context-aware sample sentences, then press Record to grant microphone permission, speak the chosen sentence,
then Stop and replay the clip or Discard it. Changing the sample discards the previous audio. The waveform visualizes microphone
amplitude but does **not** assess Mandarin pronunciation. Microphone capture
requires a secure context (`localhost` or HTTPS); recordings stay in memory
until discarded, replaced or the page closes, and are never uploaded. Stop is
automatic after 30 seconds. To check browser capture, use `cargo make serve`
and test with a microphone in Chrome. The Rust/WASM build and microphone
behaviour still require local validation.

### Shared quiz sessions (R68)

Completed Matching, MCQ and Sentence Translation results now contribute to one
in-memory session while navigating the application. The session bar appears only within practice routes after a completed attempt,
displaying aggregate scores and offering Finish session or New session (which returns to
Practice setup). Unfinished questions and microphone recordings do not create
results. A reload clears the session; a cross-route exercise scheduler and
per-skill review history are not yet implemented.

### First web run after an interrupted AI dictionary build

`cargo make db-full` checkpoints successful classifications, but does not install
a partially enriched dictionary. Press **Ctrl+C** once to stop the active
classifier; valid existing checkpoints remain available for a later resume
with the same model/prompt/source. To launch the browser using the **already
installed** DuckDB artifact, run `cargo make serve` from the repository root.
Avoid `cargo make db-full-dev` for this test: it resumes AI classification
before serving. If no compatible DuckDB artifact has yet been installed,
`serve` cannot manufacture one from the unfinished classification cache; use
the separately documented offline generation/install workflow deliberately.

Before opening the app on Windows, run `cargo fmt --all -- --check`,
`cargo test --workspace --locked` and
`cargo check -p zipeek --target wasm32-unknown-unknown --locked`.
Check the Speaking microphone preview (permission/record/replay/discard),
Matching and MCQ navigation, bidirectional Translation with LM Studio, and
shared-session totals while leaving unfinished exercises. Browser rendering
and microphone support require a secure context (localhost satisfies this).

### R71 — Translation controller refactor and Rust integration fixes

The Sentence Translation page now lives in `crates/web/src/pages/sentence_translation/`:
`sentence_translation.rs` owns hooks, `state.rs` owns request epoch/abandonment/reset rules,
`settings.rs` owns configuration controls, `actions.rs` owns async requests and
word-bank actions, and `view.rs` owns rendering. The shared session API and
existing browser selectors are unchanged. R71 also fixes `wasm-bindgen` recorder
method calls in `components/speaking_recorder.rs` and ensures async translation
closures clone their controller instead of moving a Yew `Fn` callback's state.
All Node regression checks pass. Rust and wasm32 compilation must still be
verified on the user's Rust-equipped workstation; the working database
classifier and its checkpoint identity are not modified.

### R72 — Importer test expectation update

R72 updates two `db-gen --test hsk_import` assertions that predated HSK
subtype normalization. `QV` and `QT` map to Classifier, `MQ` to Numeral,
and `CC` to Conjunction; these supported tags must not generate unknown-PoS
warnings. Tests now check the canonical memberships and meaningful diagnostic
invariants instead of requiring an obsolete warning count. Importer code,
classification prompts/caches and the installed DuckDB are unchanged.

On Windows, run:

```powershell
cargo test -p db-gen --test hsk_import --locked
cargo test --workspace --locked
cargo check -p zipeek --target wasm32-unknown-unknown --locked
cargo make serve
```

### R75: fixed Sentence Translation and E2E coverage

Practice → Sentence translation now offers five, ten, or unlimited questions.
Choose HSK 2.0/3.0 level or a saved Word Set and Start translation. On the
Translation page, configure the local AI endpoint/model and Generate sentence;
a fixed session shows a completion view after the final Continue. The app-level
shared session continues to accumulate completed exercise results independently.

The deterministic Thirtyfour suite now covers five translations from HSK 3.0
level 6 through the setup tab using a local AI fixture. Run
`cargo test -p e2e --test translation_fixture --locked -- --test-threads=1 --nocapture`
from the repository root. It does not require LM Studio, but does require
installed Chrome, Trunk and the Windows Rust/WASM prerequisites.

### R76 Windows fixture build prerequisite

If running the R75 HSK 6 Translation E2E scenario, first check the browser target with `cargo check -p zipeek --target wasm32-unknown-unknown --locked`. R76 fixes the saved Word Set `i32`-to-`i64` domain ID conversion that previously prevented Trunk from launching the test browser. Then use `cargo test -p e2e --test translation_fixture --locked -- --test-threads=1 --nocapture`.

### R78 Browse debounce compile fix

R78 corrects the Yew effect teardown in the auto-search form: a pending timeout is dropped (and cancelled) when the dependency tuple changes or Browse unmounts. Verify on Windows using `cargo check -p zipeek --target wasm32-unknown-unknown --locked` before `cargo make serve`. No database rebuild is needed.

### Exercise-first setup (R81 preview)

Practice setup now shows exercise-definition selectors first and derives
conditional General, Sentence Translation and Speaking settings from the
`zipeek-exercises` draft. Existing Matching/MCQ and Translation start actions
remain separate while mixed-session scheduling, multi-category dictionary
filters and saved setup profiles are under development. Listening is not yet
playable, and the microphone preview does not produce pronunciation scores.

### Navigation and dictionary-filter infrastructure (R83)

The main drawer now shows icon-and-text links using the existing `yew_icons`
Lucide feature set. Accessible link names and route targets are unchanged.
The browser database adapter has a tested parameterized foundation for
multiple HSK bands, category/PoS filters, and category slug suggestions.
**These combined filters are not exposed in the Practice UI yet**: the
Matching/MCQ and Translation launch paths still take a single HSK band.

### Practice source selection (R82)

Practice setup presents one exclusive source selector for the selected exercise
types: a single HSK 2.0/3.0 band or a saved personal Word Set. Matching/MCQ and
Sentence Translation consume the same selection, while retaining separate
launch routes. Multi-HSK/category autocomplete, profiles, and combined
scheduling remain future work; see `docs/product/PRACTICE_SETUP_V2.md`.
