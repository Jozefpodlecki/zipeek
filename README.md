# Zipeek

## Complete local installation

Run `cargo make install` from the repository root for the full, explicit local setup. This performs the locked web dependency/vendor preparation, background and Hanzi reference preparation, Docs index generation, dictionary generation/compression/installation, Province Quest geography fetch/audit/build, pedagogy Worker generation, and final deploy asset validation. The command can take a long time, may download sizeable source data, and needs the pinned Rust toolchain, Node.js, cargo-make, the dictionary generator's native DuckDB library, and a working `7z`/`7zz` extractor for AreaCity. A failure stops installation; the final asset verification must pass. A source ZIP alone does not contain all generated artifacts.

The AreaCity geography is for **local development only**. Its upstream boundary-data redistribution rights are unresolved; successful `cargo make install` is not permission to publish those files. The final preflight checks file validity, not redistribution licensing. To rebuild only Province Quest data use `cargo make geo-install`. After a successful installation, use `cargo make serve` for normal development; it does not regenerate the dictionary or download geography.


Zipeek is a browser-based Mandarin Chinese dictionary and practice app built with **Rust 1.97.1**, **Yew 0.23**, **WebAssembly**, **Tailwind CSS** and **DuckDB-WASM**. It runs as a static web application under `/zipeek/` on GitHub Pages. The packaged dictionary is read-only; Word Sets, preferences and unfinished Practice records use browser-local storage.

[CI](https://github.com/Jozefpodlecki/zipeek/actions/workflows/ci.yml) | [Documentation](docs/README.md) | [Current implementation](docs/project-management/current.md) | [Open tasks](docs/project-management/tasks.md)

## Run locally

Install [Rustup](https://rustup.rs/), [Node.js](https://nodejs.org/), [cargo-make](https://github.com/sagiegurari/cargo-make), and [Trunk](https://trunkrs.dev/). The repository pins Rust and the WASM target through `rust-toolchain.toml` and frontend dependencies through their lockfiles.

```powershell
git clone https://github.com/Jozefpodlecki/zipeek.git
cd zipeek
cargo make setup
cargo make serve
```

Open `http://localhost:1420` after Trunk starts. Stop the local server with **Ctrl+C**. The first setup installs/builds the required frontend vendor assets. Subsequent sessions generally need only `cargo make serve`. Use `cargo make verify-assets` to validate vendor assets without rebuilding them. Database regeneration is **not required** to run the existing packaged dictionary.

## What is in the application

- **Dictionary and Word Sets:** browse/search Chinese vocabulary and save personal selections.
- **Practice:** matching, multiple choice, vocabulary and generated exercises. Advanced translation-only Practice uses a persisted UUID record that restores unfinished generated questions, progress and scores.
- **Sentence Translation:** choose a vocabulary source and direction in Practice; AI endpoint/model belong in Settings. The model returns Chinese and English only, and Zipeek builds the answer tiles. The separate free-text translator is available at `#/translate-sentence`.
- **Courses, games and writing:** guided Pinyin lessons, Cloud Match, crosswords and Hanzi activities.
- **Settings and Debug:** appearance and Hanzi font/size preferences, AI configuration and DuckDB runtime diagnostics.

Completed Practice records are removed from temporary localStorage after the configured evidence workflow. An R283/R284 opt-in personal DuckDB Practice-outcome writer exists in source, but durable history remains unverified in Chrome; the personal database functionality is not yet a verified learning-results writer. See [personal storage](docs/dictionary-data/personal/personal-data-storage.md).

## Serve an existing build without Trunk

`zipeek-static-server` serves a prebuilt Trunk directory. It does not compile WASM, watch files, or prepare DuckDB assets. Build with Trunk first, then run from the workspace root:

```powershell
cargo run --locked -p zipeek-static-server -- --root crates/zipeek-web/dist --host 127.0.0.1 --port 1420
```

A Pages build created with `trunk build --public-url /zipeek/` can be served with `--base-path /zipeek/`; use `http://127.0.0.1:1420/zipeek/`. Startup fails if `index.html` is absent. Hash routing requires no server-side SPA fallback; unknown asset paths correctly return 404. The standalone AI server is not ready to replace LM Studio yet.

## Build and test

Run from the workspace root. Plain `cargo build --locked` and `cargo test --locked` use native default members; the Yew web crate remains a workspace member and requires the separate WASM-target check. `cargo build --workspace` is not a native web build.

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo check --locked -p zipeek-e2e --tests
node --test scripts/tests/*.test.cjs
cargo make e2e-full
```

### Firefox browser smoke tests

Install Firefox locally, then select it for the managed Rust E2E harness:

```powershell
$env:ZIPEEK_E2E_BROWSER = 'firefox'
cargo test --release --locked -p zipeek-e2e --test firefox_smoke -- --nocapture
Remove-Item Env:ZIPEEK_E2E_BROWSER
```

`thirtyfour` manages the corresponding geckodriver, including downloading it when necessary. The harness keeps Chrome as the default. Firefox uses standard WebDriver for navigation, input, screenshots, and viewport changes; its smoke suite covers startup, routes, browser history, and responsive Hanzi Writing. The current video recorder uses Chrome CDP and runs only for Chrome. Firefox evidence still includes screenshots, scenario results, and diagnostics. Chrome-specific CDP network interception and pre-navigation script injection remain explicitly unsupported in Firefox, so the entire Chrome suite is not yet portable. Set `ZIPEEK_E2E_VISIBLE=1` to watch the browser instead of running headless. Set `ZIPEEK_E2E_BROWSER=chrome` to select Chrome explicitly. If `ZIPEEK_E2E_VIDEO_STRICT=1` is enabled, Firefox reports video unavailability instead of claiming a recording was captured.

The last command runs the managed Chrome suites, including root and GitHub Pages route matrices, accessibility, Browse, Practice and translation scenarios. `cargo make pack-e2e` packages the test evidence. The optional live AI test needs a local compatible provider:

```powershell
$env:ZIPEEK_E2E_AI_ENDPOINT = 'http://127.0.0.1:1234/v1/'
$env:ZIPEEK_E2E_AI_MODEL = 'your-loaded-model-id'
cargo test --release --locked -p zipeek-e2e --test ai_learner -- --ignored --nocapture
```

The standard suites use a Rust in-process AI fixture and do not need LM Studio. **Tests authored in source are not automatically verified**; see [current evidence](docs/project-management/current.md) and [E2E guidance](docs/archive-history/audits/e2e.md).

## Database and packaging

The generated dictionary, manifest and notices are under `crates/zipeek-web/static/`. To rebuild it deliberately, consult the [database regeneration guide](docs/dictionary-data/schema/regeneration.md) and use the supported native `db-generate`, `db-compress` and `db-install` tasks. Do not hand-edit the DuckDB file or regenerate it during normal Trunk builds.

`cargo make pack` creates a lightweight source-review ZIP for code handoff, excluding repository-root `assets/`, `crates/zipeek-web/public/`, `crates/zipeek-web/static/hanzi-strokes/`, generated DuckDB payloads and build outputs. It is not a standalone build or deploy package. Keep local application assets when using `cargo make unpack`; the unpacker neither restores nor removes `hanzi-strokes/` (including when reading older archives), and does not remove the other excluded asset directories. The codebase is organized into `crates/zipeek-*` packages; see the [architecture map](docs/architecture/architecture.md).

## Troubleshooting

If the web build cannot find WASM or vendor files, check `rustup target add wasm32-unknown-unknown` and rerun `cargo make setup` or `cargo make verify-assets`. If E2E fails, include the first compiler/Chrome error and the scenario evidence under `target/e2e`. For an AI test, confirm the endpoint is running, accepts cross-origin requests and has the exact model ID loaded. No local API key is persisted by Zipeek.

## License

The application is MIT licensed ([LICENSE](LICENSE)). The dictionary and third-party assets retain their own source attribution and notices; see the generated artifact bundle and [source provenance](docs/dictionary-data/schema/sources.md).

### R285 E2E evidence hygiene (WIP)

`cargo make e2e-full` compiles the managed tests, empties `target/e2e` through the CLI, then runs all managed browser suites. For one targeted `cargo test` invocation, use `cargo make e2e-clean` beforehand if you need a clean evidence directory. **Cleanup deletes prior screenshots, videos, and ZIPs in `target/e2e`; back up anything you want to retain first.**

After the run, `cargo make pack-e2e` exports **all** current suite folders to `target/e2e/zipeek-e2e-all-YYYYMMDD-HHMMSS.zip`. You can still use the direct `zipeek-cli pack-e2e` command to export the newest single suite, or `zipeek-cli pack-e2e --run <suite-path>` for a particular suite. The combined export only represents one invocation if the folder was cleaned before that invocation. `pack-e2e` excludes copied Trunk builds but includes MP4 recordings by default. For a smaller upload, use `zipeek-cli pack-e2e --all --no-videos`; timestamps and recording counts are included in the archive manifest.

### Pedagogical worker build

After installing the pinned Rust toolchain, WASM target, and `wasm-bindgen-cli 0.2.130`, build the isolated worker from the repository root:

```sh
cargo run --locked -p zipeek-cli -- build pedagogy-worker
```

Or use `cargo make pedagogy-worker`. The CLI generates `pedagogy_wasm.js` and `pedagogy_wasm_bg.wasm` alongside `worker.mjs` under `crates/zipeek-web/static/pedagogy-worker/`. This builds the transport only; it does not enable pedagogical tracking.
