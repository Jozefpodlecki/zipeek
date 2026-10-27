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
| Rust/application/build/dependency changes | Formatting, Clippy, workspace tests, Pages-ready Trunk build |
| UI behaviour | Code gates plus Chrome e2e smoke covering changed behaviour |
| Test or CI changes | Relevant code gates and exercise affected harness/workflow where possible |

Always update [CURRENT](../process/CURRENT.md) and [TASKS](../process/TASKS.md),
and append finished outcomes to [completed/](../process/completed/). For mixed
changes, apply all relevant checks. Unavailable gates are blocked, not passed;
report prerequisites and failures. Do not silently weaken gates to make the
current scaffold appear complete.

## Commands

Full prerequisites, working directories, and per-area invocation live in the
relevant file below. The five code gates required for any code change are:

```powershell
# Repository root
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --target wasm32-unknown-unknown -p zipeek --locked

powershell
# From web/
npm ci --ignore-scripts
trunk build --public-url /zipeek/
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
e2e.md	Chrome e2e strategy, driver ownership, Trunk lifecycle, A15 modularization proposal
browser-db.md	A06 browser database checks, Pages-prefix test, native/wasm replay
word-of-the-day.md	D05 coverage plan and FNV-1a behaviour checks
practice-url.md	D13/D15 URL, exclusion, offset, and fallback coverage
round-builder.md	D12/D19 round-generation and single-stream vector checks
mcq.md	D20 MCQ coverage plan
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