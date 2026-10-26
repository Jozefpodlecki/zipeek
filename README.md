# Zipeek

![rustc](https://img.shields.io/badge/rustc-1.97.1-blue.svg)
![CI](https://github.com/Jozefpodlecki/zipeek/actions/workflows/ci.yml/badge.svg)
![wasm-bindgen](https://img.shields.io/badge/wasm--bindgen-0.2.114-blue.svg)

A client-side Chinese word query and practice app built with Rust, Yew, and
WebAssembly. A pregenerated read-only DuckDB dictionary is planned; the current
app contains route scaffolds. GitHub Pages builds use `/zipeek/`.

### Dictionary data

Dictionary integration is planned; see [data sources and licensing](docs/DB.md).
The primary input is [jelleverheyen/hsk-vocabulary complete.json](https://github.com/jelleverheyen/hsk-vocabulary/blob/a66fd30b9580da2c2af7eb19e4b9d8099a29c061/complete.json)
at commit `a66fd30b9580da2c2af7eb19e4b9d8099a29c061`, declared MIT. Preserve its
copyright/permission notice and upstream attribution. CC-CEDICT is an empty-gloss
fallback; the primary dataset also credits CC-CEDICT for definitions.

Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

Generated dictionary data derived from CC-CEDICT will be distributed under
CC BY-SA 4.0, with its transformations documented. Application code retains its
own licence.

### Credits  

- <a href="https://www.flaticon.com/free-icons/asia" title="asia icons">Asia icons created by Freepik - Flaticon</a>
- https://github.com/drkameleon/complete-hsk-vocabulary
- https://cd.hwxnet.com/
