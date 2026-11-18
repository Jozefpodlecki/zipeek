# Browser vendor asset lifecycle

## Ownership and boundaries

The browser application consumes, but does not generate, local third-party runtime
assets. `zipeek-cli web` prepares these files explicitly. npm remains the only
installer for packages described by `crates/zipeek-web/package-lock.json`.
Trunk copies the prepared `vendor/` directory into the site; its watch/rebuild
pipeline must not install dependencies or regenerate vendor assets.

## Commands (repository root)

| Task | Contract |
| --- | --- |
| `cargo make setup` | Locked npm install followed by preparation and verification |
| `cargo make web-install` | Run `npm ci --no-audit --no-fund` explicitly |
| `cargo make prepare-assets` | Prepare DuckDB/Arrow and Chart.js summary assets if stale |
| `cargo make prepare-duckdb` | Prepare only the pinned DuckDB-WASM/Arrow bundle |
| `cargo make prepare-summary` | Prepare only the pinned esbuild/Chart.js bundle |
| `cargo make verify-assets` | Read-only version, input hash, output hash and file checks |
| `cargo make serve` | Verify once, then run `trunk serve`; no preparation hook |

The `zipeek-cli` implementation is split across `web/inputs.rs` (locked module
inputs), `web/fingerprint.rs` (hashes and verification), `web/prepare.rs`
(staged preparation and publication), and `web/install.rs` (explicit npm install).
The parent `web.rs` remains the CLI-facing API for these operations.

The `zipeek-cli` commands are also available as `cargo run -p zipeek-cli
--locked -- web <install|prepare|verify|prepare-duckdb|prepare-summary>`.

## Reproducibility and failure handling

Preparation checks installed npm package versions against the currently pinned
manifest values. For DuckDB, Rust copies the Base WASM and worker, rewrites all bare
Apache Arrow module imports (the pinned DuckDB build may contain more than one), and copies the approved local Arrow adapter and
license/notice files. For the summary bundle, Rust invokes the pinned esbuild
script only during explicit preparation. No npm install is triggered during
preparation or verification.

Each vendor directory contains a generated `.zipeek-prepared.json` stamp with a
preparation format version, source SHA-256 hashes and generated file hashes.
The stamp is created only after the staged output has been published. A matching
stamp allows a repeated prepare request to skip copying; verification never
updates the stamp. The prepared vendor directories and stamps are generated
output and must not be shipped as source archive inputs.

Generated files are prepared in temporary sibling directories; a failed build
leaves the previous published asset directory intact. Once preparation has
succeeded, the prepared directory replaces the old one. CI and Pages deployment
must explicitly prepare after `npm ci` and before invoking Trunk.

If verification fails after unpacking, run `cargo make setup` from the root.
For a stale input with existing `node_modules`, run `cargo make prepare-assets`.
Do not use `cargo clean` to resolve missing JavaScript assets. Browser runtime
loading, worker lifetime, and dictionary database integrity remain owned by the
WASM database integration, not the vendor build step.

## DuckDB Arrow import regression

The installed `@duckdb/duckdb-wasm@1.33.1-dev114.0` may expose two `from`
references to `apache-arrow`. Asset preparation must rewrite **every** matching
import specifier to `./arrow.mjs` rather than requiring exactly one occurrence.
Zero matching references remain a hard error because that suggests upstream
module output changed incompatibly. This is a vendor preparation check, not a
request to remove imports or reinstall packages.

## Practice progress creatures

The six source WebP illustrations live in `crates/zipeek-web/static/practice/` and have stable creature names: `dragon.webp`, `qilin.webp`, `phoenix.webp`, `white-tiger.webp`, `crane.webp`, and `deer.webp`. They are preserved by the CLI source archive unpacker alongside the authored Courses and brand assets. Trunk copies `static/` into the deployed site. The shared Practice progress component renders the corresponding `/practice/<creature>.webp` URL in root development or `/zipeek/practice/<creature>.webp` on GitHub Pages. Image bytes are never embedded in WASM and no Blob URLs are created.

The existing Animato tween changes the bottom-up reveal; image decoding and asset responses are checked in the focused Chrome scenarios. Unpacker tests verify these authored assets are installed instead of skipped.

## R538 retained Chart.js through a typed Rust boundary

D35 and the pinned Chart.js/esbuild build-only packaging remain in force. `scripts/summary.mjs` now exports only a minimal Chart constructor after importing/registering Chart, DoughnutController and ArcElement. `zipeek-browser::charts` resolves the prepared local module. The Rust `src/services/charts.rs` owns summary/course ring datasets, options and an opaque ChartInstance with idempotent destruction.

The previous `course_charts.js` and `summary.js` wrappers had no active Rust callers and are removed. No live Yew components mount the new chart handle yet. D35 numeric counters, reduced-motion transitions and async unmount lifecycle require an integrated frontend implementation and browser review before being called complete.

## Chart.js WASM interop (R540 WIP)

The pinned Chart.js bundle remains built from `crates/zipeek-web/scripts/summary.mjs` using the existing explicit preparation pipeline. `zipeek-browser::charts` resolves `vendor/summary/summary.mjs` against the document base URI and uses a narrow `wasm-bindgen(inline_js)` dynamic import before constructing Chart.js through its exported factory. A Rust `ChartInstance` newtype owns `destroy` and cleanup. `zipeek-web::services::charts` owns summary/course-specific datasets and presentation options. The old `chart_bridge.js` shim has been removed. Browser acceptance for both `/` and `/zipeek/` remains pending.

## R541 application browser interop

AI configuration, Hanzi clipboard behavior and Recognize Hanzi canvas state are now owned by `zipeek-browser`. The recognition workspace uses typed Rust event handlers, a small `wasm-bindgen(inline_js)` theme observer adapter, and existing Yew DOM events for workspace status. The old `recognition_workspace.js`, AI config shim and unused clipboard shim are removed. The Chart.js vendor entrypoint, esbuild bundle generation, and Node asset tooling remain build-only resources under D35.

The Rust/WASM and browser acceptance gates for these changes are pending.

## Full local installation

`cargo make install` is the explicit all-assets setup entrypoint. It runs `setup-all`, regenerates/compresses/installs the dictionary using its verified native pipeline, fetches/audits/builds the pinned local AreaCity Province Quest geography, prepares the pedagogy Worker, and executes `preflight-deploy` after those dependencies complete. `cargo make geo-install` provides the geography-only sequence. `cargo make serve` remains the fast read-only vendor verification and Trunk startup path.

AreaCity boundary redistribution rights are unresolved. Geography generated by this installation is for local testing only until legal provenance and licensing acceptance is separately recorded. The asset-integrity audit does not certify legal redistribution. Network access and 7z/7zz are required on a cold geography setup; cached validated source data can be reused.
