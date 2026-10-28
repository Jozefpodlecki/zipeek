# CI and target coverage

Current CI state, required gates, and the PR checklist. Part of the
[testing index](TESTING.md). Workflow structure and deployment live in
[CI](../process/CI.md); lockfile policy in
[D27](../process/decisions/process-and-infra.md#d27-workspace-lockfile).

## Current CI and target coverage

A16 configures PR/push/manual CI with Rust 1.97.1, locked npm assets and all five
required gates in a Windows job. Workspace tests run the owned root/Pages Chrome
harness with installed Chrome and managed driver. The Pages reusable workflow is
called only after successful non-PR master validation; it builds on Ubuntu from
web/ with --release --public-url /zipeek/ and uploads web/dist/.

See [CI](../process/CI.md) for triggers, permissions, caching and tool selections.
Local gates and workflow checks are evidence; hosted execution and actual
publication remain unverified until the first GitHub run after push.

## Required gates

For code changes, all five gates must pass:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `cargo test --workspace --locked`
4. `cargo check --target wasm32-unknown-unknown -p zipeek --locked`
5. `trunk build --public-url /zipeek/` (from `web/`)

The wasm check does not replace the Pages build; both are required. UI behaviour
changes also require Chrome e2e smoke coverage once the e2e harness exists. If
any gate is blocked by a missing prerequisite, report it honestly — never mark
it as passed.

For documentation-only changes: Markdown structure, relative links, command
accuracy, and consistency with repository facts. Rust builds and e2e are not
required.

## PR checklist

- Scope and any dependency/schema/decision changes were explicitly approved.
- Applicable checks passed, or failures and missing prerequisites are reported.
- UI changes include Chrome smoke results when the harness is available.
- Pages builds retain `/zipeek/`.
- CURRENT/TASKS, completed outcomes, and affected architecture/product/DB/testing
  documents are updated.
- The PR describes resulting behaviour and validation, with relevant failure output.

## A16 implementation and remaining checks

A16 implements the approved migration. Validate YAML and dependency ordering,
then run the five local gates plus the release build. Check that the built HTML,
workers/modules and dictionary retain /zipeek/ paths and copied notices. Use
fresh locked npm assets before Trunk, matching the workflow sequence.

Under D33, both Picomatch tooling advisories are accepted while patterns remain
developer-controlled build input; do not add overrides or run audit fix for them.
Acceptance is not a package remediation, and the boundary must be reassessed if
application input can reach the dependency. Trunk latest remains the pre-existing unpinned selection.
Hosted action setup, fresh runner behavior, repository Pages publishing/settings
and actual deployment require a GitHub run after commit/push. No such run is
claimed by local validation. No external server or separately spawned driver is
needed by the owned Chrome suite.
