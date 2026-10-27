# Zipeek

![rustc](https://img.shields.io/badge/rustc-1.97.1-blue.svg)
![CI](https://github.com/Jozefpodlecki/zipeek/actions/workflows/ci.yml/badge.svg)
![wasm-bindgen](https://img.shields.io/badge/wasm--bindgen-0.2.114-blue.svg)

A client-side Chinese word query and practice app built with Rust, Yew, and
WebAssembly. The app has a responsive slate/blue navigation shell, saved dark/light
theme and landing tiles. A pregenerated DuckDB dictionary is present; browser
initialization and typed read-only queries work through DuckDB-WASM. Browse and
exercise interfaces remain placeholders. Pages builds use `/zipeek/`.

### Dictionary data

The generated dictionary is loaded read-only in the browser; see
[data sources, licensing and runtime verification](docs/DB.md).
The primary input is [jelleverheyen/hsk-vocabulary complete.json](https://github.com/jelleverheyen/hsk-vocabulary/blob/a66fd30b9580da2c2af7eb19e4b9d8099a29c061/complete.json)
at commit `a66fd30b9580da2c2af7eb19e4b9d8099a29c061`, declared MIT. Preserve its
copyright/permission notice and upstream attribution. CC-CEDICT is an empty-gloss
fallback; the primary dataset also credits CC-CEDICT for definitions.

Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

Generated dictionary data derived from CC-CEDICT is distributed under
CC BY-SA 4.0, with its transformations documented. Application code retains its
own licence.

The [generated database](web/static/hanzi.duckdb) contains 12,100 readings.
Its [integrity report](web/static/hanzi.manifest.json) records size, SHA-256,
schema, indexes and verified inputs; [attribution](web/static/ATTRIBUTION.md)
and notices are included. Browser/WASM integration remains A06.

From the repository root, `cargo run -p db-gen -- --check` verifies/imports in
memory. `cargo run -p db-gen -- generate target/reviewed-dictionary` exports to
a fresh directory and refuses existing output. See [regeneration](docs/DB.md#regeneration-and-sizing)
for manual review/replacement and reproducibility checks. Builds copy the existing
artifact; they never generate it automatically.

The root Cargo.lock is the workspace lockfile. Use Cargo's --locked flag for
validation; Trunk enforces it automatically. From web/, trunk serve watches both
the web app and shared core sources. Trunk manages the configured Tailwind and
wasm-bindgen tools. Run npm ci --ignore-scripts from web/ before Trunk: A06's
pre-build hook packages pinned DuckDB-WASM/Arrow assets from node_modules for
local serving. No CDN, database generation or package install occurs at build time.

### Credits  

- <a href="https://www.flaticon.com/free-icons/asia" title="asia icons">Asia icons created by Freepik - Flaticon</a>
- https://github.com/drkameleon/complete-hsk-vocabulary
- https://cd.hwxnet.com/
