# CI and deployment

Local commands and prerequisites live in [testing/TESTING.md](../testing/TESTING.md).
A16 implements the workflow migration on 2026-10-28. Configuration and local
validation are recorded here; GitHub-hosted execution is not claimed by a local run.

A10 adds exact chart.js 4.6.1 and dev-only esbuild 0.28.2 under D35. The same
locked npm ci --ignore-scripts installs them; Trunk's additional pre-build hook
verifies the pins and emits a local tree-shaken summary bundle plus licenses.
The hook neither installs packages nor generates a database. Both root/Pages
Chrome builds and the standalone Pages gate exercise this packaging locally.

## Entry points and validation

.github/workflows/ci.yml handles pushes to master, pull requests and manual runs.
Its Windows validate job selects Rust 1.97.1 with rustfmt, Clippy and the wasm32
target, Node 22 and the existing Trunk installation action. It verifies installed
Chrome, prints tool versions and installs web assets from web/ using
npm ci --ignore-scripts before any build or browser test.

The automated job runs all five full-regression gates in order. D36 permits
focused local validation for routine work; it does not change this workflow:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --target wasm32-unknown-unknown -p zipeek --locked
```

```powershell
# From web/
trunk build --public-url /zipeek/
```

Workspace tests include the owned root and Pages Chrome suites. Each harness
build uses its own directory and dynamic server ports. Trunk must serve HTTP
200 before Chrome starts; a regression fixture covers initial 404 responses.
No external server or
separate ChromeDriver setup is used. Thirtyfour manages its matching driver.
The GitHub Windows image supplies installed Chrome; the prerequisite check fails
explicitly if it is absent. See the official
[runner image inventory](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-Readme.md).
Network/cache access is needed for managed driver and pinned native neo prebuilt
library downloads, in addition to Rust/npm/Trunk tool setup. The neo revision's
own .duckdb-release selects native v2.0.0; do not substitute a runner DuckDB.
Existing native tests enforce that engine and v1.4.0 storage compatibility.

## Validation-gated Pages

Only a non-PR master run with successful validate calls the local reusable
.github/workflows/deploy.yml. Other-branch manual runs validate without deploying.
Manual deployment starts from the CI workflow on master. deploy.yml has only a
workflow_call trigger, preventing an independent push/manual path from bypassing
validation or testing a different commit. Local reusable calls use the caller's
checkout context; see [GitHub reusable workflows](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows).

The reusable Ubuntu build installs Rust 1.97.1/wasm32, matching wasm-bindgen-cli
0.2.114, the existing Trunk selection and Node 22. It installs locked npm assets
from web/, then runs:

```sh
trunk build --release --public-url /zipeek/
```

That command runs from web/. Trunk handles the pinned Tailwind tool and local
DuckDB asset packaging hook; no root npm install, separate npx stylesheet command
or DB generation runs. actions/upload-pages-artifact@v3 uploads web/dist/.
The dependent deploy job uses actions/deploy-pages@v4 and the github-pages
environment, publishing its page_url output. Existing repository Pages publishing
must be configured for GitHub Actions; this change does not modify remote settings.
A separate configure-pages step is unnecessary for the explicitly configured
Trunk public URL and existing Pages site. Deployment permissions, environment and
build dependency follow [GitHub's custom workflow requirements](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).

PR validation has contents: read only. The caller Pages job grants contents: read,
pages: write and id-token: write to the reusable call; its deployment job requests
only Pages/OIDC writes. A PR cannot enter that job. Validate has a 45-minute bound,
release build 30 minutes and deploy 10 minutes. Superseded PR validation can cancel;
master validation and an active Pages deployment are not cancelled by a newer run.
The Pages concurrency group serializes active deployments using GitHub's normal
pending-run handling.

## Caches and tool selections

Cargo caches registry/Git sources and target/, keyed by OS, Rust 1.97.1, the
root Cargo.lock, workspace manifests, rust-toolchain.toml, web/Trunk.toml and web
package lock. Pages release uses a separate key. Cargo executables are not cached:
restoring old binaries must not replace selected tools. setup-node caches npm's
package cache using web/package-lock.json; node_modules and generated vendor/dist
outputs are rebuilt by the locked install/hook/build.

Existing action versions are retained: checkout v6, CI cache v5, Pages cache v4,
setup-node v4, trunk-action v0.5.0, upload-pages-artifact v3 and deploy-pages v4.
No new package, Cargo feature or action version is introduced. Rust is corrected
to the already-approved 1.97.1. Trunk remains latest as in the previous deployment
workflow: no exact version is approved by A16, and this remains an explicit
reproducibility limitation. Local evidence uses installed Trunk 0.22.0.

## Tooling advisory review

The preserved lockfile contains picomatch 2.3.1 through micromatch. The original
A06 audit and the A16 locked npm install report one high-severity dependency.
The reviewed upstream advisories cover
[extglob ReDoS](https://github.com/advisories/GHSA-c2c7-rcm5-vvqj) and
[POSIX character-class method injection](https://github.com/advisories/GHSA-3v7f-55p6-f55p),
with 2.3.2 as the patched 2.x release. These concern processing attacker-controlled
glob patterns; here the dependency belongs to build tooling, not browser search
input. PR source is already executable build input. Under [D33](decisions/process-and-infra.md#d33-accept-build-tool-picomatch-advisories),
the user accepts both advisories under that build-tool-only input condition.
Do not add an override, run npm audit fix or refresh the lockfile for them.
Reassess if untrusted application patterns reach this dependency. Acceptance is
not remediation; audit output may persist. An independently approved future
Trunk/Tailwind upgrade may bring a fixed version naturally. A39 is retired.

## Validation limits

Local checks cover YAML structure, triggers/permissions/needs, cache/input paths,
gate commands, all five actual code gates and the release artifact contents.
Root/Pages Chrome run locally. The Actions service, fresh hosted Windows/Ubuntu
images, remote permissions/environment approval and actual Pages publication
cannot be exercised by this workspace. No push, dispatch, remote setting change
or deployment was performed. Confirm the first hosted run after these changes
are committed/pushed; do not describe local success as a hosted deployment.
