# Docs Markdown renderer

The `zipeek-docs` crate uses `pulldown-cmark =0.13.0` with the `html` feature and no default CLI features. The existing Yew application calls it through Rust/WASM directly; there is no separate browser JS parser or WASM module.

## Syntax and security

CommonMark nested lists, fenced code, links and references, GFM tables, task lists, strikethrough and footnotes are supported. Rust emits deterministic unique heading IDs and exposes the same headings to the Docs TOC. Raw HTML becomes escaped text. Links are checked against a safe external-scheme and docs-root policy, including reference-style destinations. Invalid image targets render a placeholder.

All files in `docs/` remain published. `#/docs/` links and `/zipeek/` asset base behavior are preserved. Links to headings in another Markdown document use `#/docs/<encoded-path>?heading=<heading-id>` so the hash router receives one valid route. The Docs page scrolls to the heading after rendering, including direct links and reloads. Same-document `#heading-id` anchors remain local to the article.

## Acceptance

```powershell
cargo test --locked -p zipeek-docs
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo run --locked -p zipeek-cli -- build docs
cargo make serve
```

In Chrome, verify nested Markdown, footnotes, checkboxes, tables, raw/source toggle, duplicate headings, CJK heading IDs, internal links, relative images and browser security under `/zipeek/`. These checks have not run here. TD-006 remains open until this R429 route/scroll behavior and prior renderer behavior pass locked WASM and real-browser acceptance.

## Static publication

Trunk copies `../../docs` directly into the published `docs/` directory via the
`copy-dir` entry in `crates/zipeek-web/index.html`. No duplicate Markdown tree
is maintained in `static/docs`. `cargo make docs` generates only
`crates/zipeek-web/static/docs/index.json`, which the Docs explorer uses for
navigation and search; run it after editing documentation or during first-time
setup. `cargo make setup-all` includes this preparation. `cargo make serve`
does not regenerate the index on every invocation. CI and deployment continue
to generate the index before building. GitHub Pages uses the same relative
`docs/` assets under `/zipeek/`.
