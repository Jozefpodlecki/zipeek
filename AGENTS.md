# Agent guide

Zipeek is a client-side Chinese hanzi query and practice app, built with Rust,
Yew CSR, WebAssembly, and Tailwind. The target database is a pregenerated,
read-only DuckDB file. Hosting is GitHub Pages at
`https://Jozefpodlecki.github.io/zipeek/`; the base path must remain `/zipeek/`.

This file is the source of truth for contributor process. Product details,
implementation notes, and open decisions live in [docs/](docs/README.md).

## 1. Before editing

0. Read [docs/DIGEST.md](docs/DIGEST.md) first. It holds the user's pending
   design answers, is not a permanent reference, and must be consumed and wiped
   by the session that reads it.
1. Read this guide, [CURRENT](docs/process/CURRENT.md),
   [TASKS](docs/process/TASKS.md),
   [ARCHITECTURE](docs/architecture/ARCHITECTURE.md), and
   [PRODUCT](docs/product/PRODUCT.md). Use [docs/README.md](docs/README.md) to
   locate the reference document for a specific task.
2. Read the relevant documents below, including open decisions affecting the task.
3. Follow the user's explicit task. Select the first ready task from TASKS
   only when asked to choose work.
4. Present a plan stating the task, files to change, validation, and any decisions
   or dependency changes requiring approval. **Wait for approval before editing.**
   Approval covers the stated scope; ask again if it materially changes.
5. If an unresolved decision blocks the task, ask the user. Proposed defaults
   are not approved decisions. Continue only independent work already approved.

An approved task does not need a second approval for routine steps within its scope.
Keep changes focused and preserve existing user changes, including staged changes.

## 2. Development phase

The app is in active development. Breaking changes to application code, data, and
generated artifacts are acceptable within approved work. Regenerate affected
artifacts instead of maintaining backward compatibility or migrations for old
outputs. There is no application/database versioning policy at this stage.

D12's explicitly versioned round-seed encoding is a narrow exception: preserve
replay-format compatibility as documented in
[ROUND_GENERATION](docs/product/ROUND_GENERATION.md). Under D19, v1 is the only
format version. A future v2 requires both a canonical prefix bump and a URL
parameter selecting it, plus a documented compatibility path. That is a breaking
change to shared URLs requiring explicit approval; no v2 format,
version-selection parameter, or compatibility parser exists today.

This does not relax dependency pins, approval requirements, source licensing,
COMPLETED history preservation, or the rule against hand-editing DuckDB files.

## 3. Repository map

| Path | Role / current status |
| --- | --- |
| `Cargo.toml` | Workspace manifest; includes `web`, `crates/db-gen`, `crates/zipeek-core`, and `e2e` |
| `web/` | Yew crate, package `zipeek`; Trunk commands run here |
| `web/src/` | App, components, contexts, models, pages, routes, services |
| `web/index.html`, `web/Trunk.toml` | HTML entry point and build configuration |
| `web/styles/`, `web/public/`, `web/icons/` | Current styles and static assets |
| `crates/db-gen/` | Package `db-gen`; verified inputs, engine/storage guards, approved schema and HSK/fallback import, compatible artifact export, reports and native read-only verification |
| `web/static/hanzi.duckdb` | Generated dictionary with reports/notices; A06 opens it read-only through pinned local browser workers |
| `e2e/` | Workspace package `zipeek-e2e`; executable and smoke test exist; lifecycle harness pending |
| `.github/workflows/` | Existing CI and Pages workflows; migration work pending |
| `docs/` | `product/`, `architecture/` (with `modules/` and `schema/`), `testing/`, `process/` (with `decisions/` and `completed/`) — see [docs/README.md](docs/README.md) for the index |

The intended workspace includes the web app, generator, and e2e crate. Do not
add nonexistent crates merely to match this map. The approved shared crate name
is `zipeek-core`; Word/WordId, HSK and canonical PoS types are implemented;
practice URL state and the pure deterministic round builder are implemented;
typed browser database queries are implemented; playable exercise UI remains
future work (D01).

## 4. Version and dependency policy

Preserve the following approved targets. Do not add, move, or upgrade dependencies,
or change these versions, without explicit approval. A task plan must identify
such changes specifically; general task approval does not implicitly approve them.

| Component | Target |
| --- | --- |
| Rust | `1.97.1`, edition `2024` |
| Yew | `0.23`, CSR |
| yew-router | `0.20` |
| yew_icons | `0.10`, `lucide` and `simple_icons` |
| Tailwind | `4.3.3` |
| wasm-bindgen | `0.2.114` |
| DuckDB-WASM | Exact npm pin `@duckdb/duckdb-wasm@1.32.0` (D08) |
| Native generator | User-restored `duckdb-neo` at Git revision `4acbea84794582c4cd962ee971a3d7a5dcb72e39`; query/log and require native `v2.0.0`; require `v1.4.0` storage compatibility before output creation (D23). |
| e2e | `thirtyfour` `0.38.1` + installed Chrome; managed ChromeDriver (D09) |
| Build | Trunk; no exact version selected yet |
| Round RNG | `rand` `0.10.3` with approved `chacha` feature; use `rand::rngs::ChaCha12Rng` (D17), no separate rand_chacha |
| Animato | User-added `1.7.3` with `path`, `physics`, `color`; additional integration features still require approval |

These are policy targets, not a claim that all manifests and workflows agree.
Cargo version requirements are not necessarily exact pins. Record discrepancies
in CURRENT/TASKS and obtain approval before changing them.

Under D27, track the single workspace Cargo.lock at the repository root. Do not
restore a web-local lockfile. Use --locked for Cargo validation and preserve the
resolved graph unless an approved dependency change requires updating it.
Trunk builds enforce locked resolution through web/Trunk.toml.

## 5. Hard rules

- Follow [CODING_STANDARDS](docs/process/CODING_STANDARDS.md), including its
  no-comments rule and sole lint-suppression rationale exception.
- Never hand-edit `*.duckdb`. Regenerate with the approved native generator;
  generation is manual, never an automatic application build step. Commit the
  generated database with an approved data change.
- Obtain explicit approval for DB schema changes and open-decision resolutions.
  Document source licensing before implementing the generator.
- Preserve `/zipeek/` in Pages builds, asset URLs, and routing. E2e development
  serving uses `/` without `--public-url`; see
  [testing/e2e.md](docs/testing/e2e.md) for the deliberate distinction.
- TASKS, CURRENT, and COMPLETED are the only live task-state sources. Keep TASKS
  open-only and ordered by prerequisites; record active work/blockers in CURRENT;
  append outcomes to COMPLETED before removing finished tasks from TASKS. The
  COMPLETED log is split by outcome date under `docs/process/completed/`; append
  new outcomes to the file for the outcome date, and never rewrite historical
  date files. `docs/process/completed/archive.md` and
  `docs/process/completed/validation-log.md` are frozen historical records.
  STATUS.md is retired; do not recreate or append to it. Its one-time history
  migration is preserved in `archive.md`.
- Treat [docs/DIGEST.md](docs/DIGEST.md) as a consumed scratchpad. Read it at
  the start of every session, apply its answers to the owning documents, record
  the result in `docs/process/completed/` for the consumption date, then wipe
  consumed entries (see the file for the wipe rule). Do not link to it from
  other documents, and do not treat it as a permanent reference.
- Do not claim intended routes, tools, tests, or database integration already exist.
- Use the approved 64-bit FNV-1a for Word of the Day and reproducible hashes
  (D14). `DefaultHasher` is for ephemeral in-session use only, never Word of the
  Day or reproducible values. Do not persist hash state. Word exclusions store
  WordIds, not hash outputs.
- Use exactly one `ChaCha12Rng` per `build_round`, seeded once from
  `fnv1a64(canonical_seed_input)`. Seed recovery consumes draws from this
  same stream; generation continues from the resulting state. No reset, no
  second RNG, no re-seed. Recovery-failure fallback (first eligible word by
  `id`) consumes no RNG. Changing recovery attempt count, draw order, or
  sampler is a hard replay break — bump the canonical seed prefix.
  D19 keeps v1; any future break also requires the URL-selection and explicit
  approval steps in section 2. This bullet is the authoritative single-stream
  contract if ROUND_GENERATION conflicts with it.

## 6. Commands and completion

D08 pins `@duckdb/duckdb-wasm@1.32.0` exactly. Do not substitute `latest`, `next`,
or development builds. If integration exposes a genuine wasm toolchain
compatibility blocker, stop and report commands, versions, and error evidence.
See [architecture/schema/DB.md](docs/architecture/schema/DB.md) and
[architecture/schema/runtime.md](docs/architecture/schema/runtime.md) for the pin
rationale and remaining integration work.

Use the working directories and prerequisites in
[testing/TESTING.md](docs/testing/TESTING.md). Common commands:

```powershell
# Repository root: code quality gates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --target wasm32-unknown-unknown -p zipeek --locked
```

```
# From web/: Pages-ready build (dev server uses port 1420)
npm ci --ignore-scripts
trunk serve
trunk build --public-url /zipeek/
```

```
# Repository root: approved target invocation after e2e package/harness setup
cargo test -p e2e -- --test-threads=1
```

The current manifest is still named zipeek-e2e; renaming it to e2e and
implementing the owned Trunk lifecycle remain follow-up work (A15). It is
already a workspace member with an executable and tests/smoke.rs. The smoke
test currently expects an externally running server on port 1420; the
executable currently opens visible Chrome, while the test enables headless
Chrome. D09 uses WebDriver::managed(DesiredCapabilities::chrome()), a
harness-owned Trunk child, and shared readiness polling. Do not install or
spawn ChromeDriver separately. See testing/e2e.md for
the approved layout and
process/decisions/process-and-infra.md
for the binding decision.

For code changes, all five gates must pass:

cargo fmt --all -- --check

cargo clippy --workspace --all-targets --locked -- -D warnings

cargo test --workspace --locked

cargo check --target wasm32-unknown-unknown -p zipeek --locked

trunk build --public-url /zipeek/ (from web/)

The wasm check does not replace the Pages build; both are required. UI
behaviour changes also require Chrome e2e smoke coverage once the e2e
harness exists. If any gate is blocked by a missing prerequisite, report
it honestly — never mark it as passed.

A06 requires npm dependencies installed in web/ before Trunk. Its pre-build hook
packages the exact pinned DuckDB-WASM worker/module assets and Arrow dependency
locally under ignored web/vendor/; it never installs packages or generates a DB.
Retain the packaged library licenses/notices. Avoid CDN/latest substitutions.

For documentation-only changes, check Markdown structure, relative links, command
accuracy, and consistency with repository facts. Rust builds and e2e are not required.

Before declaring completion:

Run the applicable checks in TESTING, including Chrome e2e for UI changes.
Report failures or unavailable prerequisites
honestly; never mark an unavailable gate as passed.

Update CURRENT and TASKS for every task; append finished outcomes to COMPLETED.
Update the matching documents when product,
architecture, DB, testing, or decision details change.

Report what changed, which checks passed or were blocked, and the next TODO.
Keep the report concise; include relevant failure output rather than full successful logs.

If preparing a PR, describe the concrete change and its validation.

7. Documentation ownership
Document	Contents / update trigger
product/PRODUCT.md	v1 behaviour and routes; product changes
product/UX.md	User-facing spec and explicit unresolved UX items
product/ROUND_GENERATION.md	Pure builder, candidate views, deterministic replay, RNG, label uniqueness
architecture/ARCHITECTURE.md	Module map, routing, state, data flow; structural changes
architecture/UI_ARCHITECTURE.md	Proposed scoped state/providers, reducers, Animato adapter and page-transition lifecycle
architecture/modules/	Per-module implementation notes (db-gen, zipeek-core, browser-db, web-shell, data-flow, target-architecture)
architecture/schema/DB.md	Database overview and section index
architecture/schema/	Sources, schema, import rules, PoS mapping, queries, runtime pin, generator deps, reproducibility, artifacts, regeneration
testing/TESTING.md	Commands, unit/wasm/e2e strategy, PR checks, evidence convention
testing/	Per-area test plans (e2e, browser-db, word-of-the-day, practice-url, round-builder, mcq, startup, shell, provenance, ci-targets, evidence)
process/BASELINE.md	A00 diagnostic evidence (historical)
process/CI.md	Workflow structure, gates, caching, deployment; CI changes
process/CODING_STANDARDS.md	Rust/Yew conventions and no-comments policy; coding convention changes
process/decisions/index.md	Index of D01–D31; each decision has one home file grouped by family
process/TASKS.md	Ordered open backlog, prerequisites, and ready markers
process/CURRENT.md	Active work, blockers, and next task; update every task
process/handoff.md	Per-task continuation notes for the next session; reference only
process/completed/	Append-only outcomes split by outcome date; never rewrite
Keep process instructions here instead of duplicating them in a kickoff prompt.