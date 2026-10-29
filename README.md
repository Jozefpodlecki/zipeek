# Zipeek

![rustc](https://img.shields.io/badge/rustc-1.97.1-blue.svg)
![CI](https://github.com/Jozefpodlecki/zipeek/actions/workflows/ci.yml/badge.svg)
![wasm-bindgen](https://img.shields.io/badge/wasm--bindgen-0.2.114-blue.svg)

A client-side Chinese word query and practice app built with Rust, Yew, and
WebAssembly. The app has a responsive slate/blue navigation shell, saved dark/light
theme and landing tiles. A pregenerated DuckDB dictionary is present; browser
initialization and typed read-only queries work through DuckDB-WASM. Matching
and MCQ are playable in Endless and Fixed modes, including mixed exercise types.
Browse and the Random shortcut remain feature placeholders. Pages builds use `/zipeek/`.

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
Set-Location web
npm ci --ignore-scripts
Set-Location ..
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

### Regenerate the database and start the app

Use this instead of serve when you explicitly want to regenerate and install
the dictionary before starting the server:

```powershell
Set-Location C:\repos\zipeek
cargo make dev
```

It generates a fresh verified bundle, installs the complete bundle in web/static,
backs up the previous bundle, then starts the same server. It does not download
new dictionary sources. Native generator/build prerequisites are also required;
see [generator dependencies](docs/architecture/schema/generator-deps.md).

| Command from the repository root | What it does |
| --- | --- |
| `cargo make` | List available tasks |
| `cargo make serve` | Run the app with the installed database |
| `cargo make dev` | Generate, install the complete bundle, then run the app |
| `cargo make db-check` | Validate inputs/import in memory; write no database |
| `cargo make db-generate` | Generate a fresh candidate; print its absolute location |
| `cargo make db-install` | Verify/install the latest candidate with a backup |

Generated candidates and backups are in target/db-workflow; the installed
database is web/static/hanzi.duckdb. Trunk rebuilds do not regenerate it.
[Detailed workflow and direct generator commands](docs/architecture/schema/regeneration.md).

### If startup fails

- **`cargo make` or `trunk` is not found:** check that cargo-make and Trunk are
  installed and their executables are on PATH. Reopen the terminal after setup.
- **Missing npm packages:** stop the server and repeat npm ci --ignore-scripts
  from web/, then return to the repository root and run cargo make serve.
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

The [generated database](web/static/hanzi.duckdb) contains 12,100 readings.
Its [integrity report](web/static/hanzi.manifest.json) records size, SHA-256,
schema, indexes and verified inputs; [attribution](web/static/ATTRIBUTION.md)
and notices are included. Browser/WASM integration is implemented.

From the repository root, `cargo run -p db-gen --locked -- check` verifies/imports in
memory. `cargo run -p db-gen --locked -- generate --output target/reviewed-dictionary` exports to
a fresh directory and refuses existing output. See [regeneration](docs/architecture/schema/regeneration.md)
for manual review/replacement and reproducibility checks. Builds copy the existing
artifact; they never generate it automatically.

The root Cargo.lock is the workspace lockfile. Use Cargo's --locked flag for
validation; Trunk enforces it automatically. From web/, trunk serve watches both
the web app and shared core sources. Trunk manages the configured Tailwind and
wasm-bindgen tools. Run npm ci --ignore-scripts from web/ before Trunk: A06's
pre-build hook packages pinned DuckDB-WASM/Arrow assets from node_modules for
local serving. No CDN, database generation or package install occurs at build time.

## Credits

- <a href="https://www.flaticon.com/free-icons/asia" title="asia icons">Asia icons created by Freepik - Flaticon</a>
- https://github.com/drkameleon/complete-hsk-vocabulary
- https://cd.hwxnet.com/
