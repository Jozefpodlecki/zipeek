# CI and validation targets

The authoritative CI workflows are `.github/workflows/ci.yml` and `deploy.yml`; the authoritative local build tasks are in `Makefile.toml`. Historical release commands referring to `crates/web`, a package named `zipeek`, or `web/dist` should not be copied into new verification instructions.

## Local gates

From the workspace root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
cargo make e2e-full
```

For an explicit Pages artifact, run Trunk with `/zipeek/` as its public URL from `crates/zipeek-web/` after preparing vendor assets. Native/WASM compilation, Chrome interaction checks, and deployment-path verification are required; the legacy Node source-contract suite was removed in R392. For documentation-only tasks, validate local links and consistency with source, without claiming an unexecuted Rust gate.

## Evidence

The user reported successful E2E harness unit, accessibility and Browse suites on Windows on an earlier R282 tree, as well as one earlier live model translation browser pass. The newest UUID translation integration and current full suite have not yet been accepted on that evidence. Read [CURRENT](../../../project-management/current.md) and [TESTING](../../../testing-quality/acceptance/testing.md) rather than reusing historical pass counts.
