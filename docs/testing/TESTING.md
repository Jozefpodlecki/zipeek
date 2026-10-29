# Testing and validation

This file is the index for testing strategy and coverage. Detailed plans,
commands, and evidence live in the files below. Ongoing tasks and current
validation state live in [TASKS](../process/TASKS.md) and
[CURRENT](../process/CURRENT.md); finished outcomes are in
[completed](../process/completed/).

## Checks by change type

| Change | Required checks |
| --- | --- |
| Documentation only | Markdown structure, local links, command accuracy, factual consistency |
| Routine Rust/application fix | Formatting and relevant package checks/tests; a representative affected behavior sample |
| Routine UI change | Focused code checks and a representative Chrome path; avoid the whole browser regression suite |
| Test or CI changes | Exercise the changed test/harness/workflow where possible |
| Very major work or release readiness | Full five-gate regression set, including root/Pages Chrome coverage |

Always update [CURRENT](../process/CURRENT.md) and [TASKS](../process/TASKS.md),
and append finished outcomes to [completed/](../process/completed/). For mixed
changes, apply all relevant checks. Unavailable gates are blocked, not passed;
report prerequisites and failures. State the sample used and checks omitted;
never claim a focused sample as a full regression pass.

## Proportionate local validation (D36)

The user requested on 2026-10-29 that routine changes use obvious representative
samples, with full regression only after very major work. Choose checks based on
the behavior and scope changed. A small fix normally needs its relevant unit test
or a short manual/browser path plus applicable formatting/compilation;
documentation-only work needs documentation gates. Do not run unrelated database,
browser or workspace suites just because a file changed.

Reuse still-valid successful results. After correcting a failure, rerun the
affected test/sample and expand only when failures or new scope justify it.
Very major work includes substantial feature completion and broad architecture,
runtime or data changes; use the full set below at that milestone or release
readiness. Record scope, results and material gaps. Existing automated CI keeps
its configured full gates; this policy changes local iteration, not workflow
execution. A10's complete MCQ/summary feature warranted the full run already
underway when this instruction arrived.

## Commands

Full prerequisites, working directories, and per-area invocation live in the
relevant file below. The five-gate set for very major work/full regression is:

```powershell
# Repository root
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --target wasm32-unknown-unknown -p zipeek --locked
```

```powershell
# From web/
npm ci --ignore-scripts
trunk build --public-url /zipeek/
```

Use cargo fmt --all to apply formatting, then rerun the check. The wasm target
must be installed. The Cargo wasm check is a useful intermediate check, not a
replacement for the deployment build.

D27 tracks the root workspace Cargo.lock; no web-local lockfile is used. The
--locked flags reject unintended dependency re-resolution. Trunk.toml also
sets locked = true and watches the core crate and workspace configuration. Do
not refresh the graph merely to make a check pass.

The configured dev port is 1420, not 8080. Application builds require the
approved Rust toolchain and wasm32-unknown-unknown target. Installations and
version changes are separate from validation; dependency changes still need
approval.

A06's pre-build hook packages exact DuckDB-WASM 1.32.0 and locked Arrow assets
from node_modules into ignored vendor/. It verifies the runtime version,
copies matching workers/WASM and licenses, and does not fetch/install anything
itself.

Files
File	Contents
e2e.md	Chrome e2e strategy, driver ownership, Trunk lifecycle, implemented A15 modular harness
browser-db.md	A06 browser database checks, Pages-prefix test, native/wasm replay
word-of-the-day.md	D05 coverage plan and FNV-1a behaviour checks
practice-url.md	D13/D15 URL, exclusion, offset, and fallback coverage
round-builder.md	D12/D19 round-generation and single-stream vector checks
mcq.md	D20 MCQ coverage and shared summary checks
matching.md	A09 matching and A48 mixed-session native/Chrome checks
startup.md	Static loading shell, handoff, database readiness coverage
shell.md	Theme, sidebar, footer, and shell behaviour coverage
provenance.md	Data source hashing, schema, import, and PoS fixture coverage
ci-targets.md	Current CI state, required gates, PR checklist
evidence.md	Per-task implemented test evidence (A02–A30)
Unit and wasm strategy
Use native unit tests for pure query and exercise logic where it can compile
natively. Browser-specific integration needs wasm checks and behaviour tests;
native workspace tests alone do not prove browser behaviour.

No dedicated wasm browser-test harness has been established. Introduce one only
as an approved task, including approval for any new dependencies. Match tests
to behaviour and risk rather than mirroring implementation details.

Evidence convention
Each implemented task records its test evidence (test counts, logs, screenshots)
in evidence.md and, briefly, in the task's
completed outcome row. Evidence is historical: it
records what a specific run showed, not a current gate claim. A failing gate
recorded in evidence is not repaired by a later passing run in a different task.

PR checklist
Scope and any dependency/schema/decision changes were explicitly approved.

Applicable checks passed, or failures and missing prerequisites are reported.

UI changes include Chrome smoke results when the harness is available.

Pages builds retain /zipeek/.

CURRENT/TASKS, completed outcomes, and affected architecture/product/DB/testing
documents are updated.

The PR describes resulting behaviour and validation, with relevant failure output.
## Owned Chrome harness (A15)

From the repository root, cargo test -p e2e --locked -- --test-threads=1 runs
five harness lifecycle unit tests and four integration entry points: regular root
with nineteen named scenarios, matching root with three, mixed root with two,
and Pages with fifteen, plus the existing expected fault/recovery probes.
Each integration binary owns its single managed headless Chrome session and its
servers/build directory. Workspace tests include both; no external server or
ignored-test opt-in. See [e2e](e2e.md) for prerequisites, cleanup and artifacts.

## Scoped state checks (A31)

Six web-native reducer/gate tests cover stale/duplicate initialization, retry,
unmount invalidation and latest-state theme toggles. The root providers Chrome
scenario checks dictionary ownership through loading/ready navigation and rapid
theme commands; the two integration entry points cover twelve named scenarios.
These checks add no dependency or wasm-browser-test harness.

## Practice route integration checks (A21)

A native request-reducer test covers URL-input and retry generations and duplicate
settlement. Root/Pages Chrome suites exercise all six variant preparations and
replay, normalization against the real dictionary, held/failing queries, stale
result rejection and Retry. Root also checks 404 recovery during dictionary
failure. All five code gates remain required; these checks add no dependencies.

## CI workflow integration (A16)

CI runs the five gates and owned Chrome workspace tests after locked npm install.
Validated non-PR master runs call the reusable release/Pages workflow. See
[CI](../process/CI.md) for working directories, cache boundaries, prerequisites
and the distinction between local evidence and an actual hosted deployment.

## A11 setup checks

Three native setup/parsing tests cover typed HSK transitions, URL omission/count
semantics and saved-ID parsing. The shared Chrome setup suite exercises all six
Start paths, saved-ID validation/reset/error handling, keyboard and responsive
layout at root and /zipeek/. No new dependency or dedicated browser-test framework.
All five gates remain required for the implementation.
