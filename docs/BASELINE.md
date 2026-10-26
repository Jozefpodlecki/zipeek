# Build baseline — A00

Diagnostic snapshot taken on 2026-10-26, before
application repairs. This is evidence, not live task state; use CURRENT and TASKS
for ongoing work. No application source or manifest was edited for this audit.

## Environment

- Installed Rust/Cargo: 1.97.1, default stable Windows MSVC toolchain.
- Approved Rust target: 1.97.1; no root toolchain file currently selects it.
- Installed wasm target: `wasm32-unknown-unknown`.
- Installed Trunk: 0.22.0; its configured Tailwind tool is 4.3.3.
- Cargo resolved wasm-bindgen 0.2.129 from the compatible `0.2.114` requirement;
  this is not evidence of an exact 0.2.114 build.

## Commands and results

| Working directory | Command | Result |
| --- | --- | --- |
| Repository root | `cargo fmt --all -- --check` | Failed, exit 1: formatting differences across scaffold files. No formatting fix applied. |
| Repository root | `cargo clippy --workspace --all-targets -- -D warnings` | Failed, exit 1: 48 errors per library target, including unused imports/variables, dead code, needless borrow, collapsible if, and explicit auto-deref. |
| Repository root | `cargo test --workspace` | Passed, exit 0; zero library, binary, or documentation tests. 44 compiler warnings. No implemented behaviour is covered. |
| Repository root | `cargo check --target wasm32-unknown-unknown -p zipeek` | Passed, exit 0; 44 compiler warnings. |
| `web/` | `trunk build --public-url /zipeek/` | Passed, exit 0; 44 compiler warnings; Tailwind 4.3.3 completed. Development-profile build, as invoked. |

No Chrome e2e was run: the e2e manifest has no harness/source or workspace
membership. A successful build does not establish browser readiness, routing,
loading handoff, or DuckDB integration. The App currently renders test content
and contains a test-panic callback; runtime restoration remains open work.

## Minimal follow-up scope

- A19: align approved toolchain/configuration targets and clarify compatible
  Cargo requirements versus exact pins before altering dependency resolution.
- A20: format existing Rust, remove unused imports/bindings, repair the four
  Clippy expression issues, and address unused scaffold visibility deliberately.
  Do not suppress warnings globally or implement unrelated UI to make Clippy pass.
- A15/A21: add browser lifecycle coverage and restore the actual loading/routes
  integration under their approved scope; they were not tested by A00.
- A16: migrate CI/Pages working directories, Rust selection, build base path,
  quality gates, and artifact path. Existing local success does not validate CI.

The existing staged repository migration was preserved. Builds can create ignored
artifacts and the workspace lockfile; they are not application implementation.
