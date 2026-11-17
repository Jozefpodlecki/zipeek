# Hanzi stroke reference and guided tracing -

## Architecture

The original reference viewer remains available. The Hanzi Writing workspace now uses the same packaged glyph data to validate handwriting one stroke at a time. It no longer treats every pointer release as a successful stroke.

- `zipeek-hanzi-writing::matching` samples the reference median and candidate trace by arc length. In addition to uniform samples, original reference and candidate vertices are checked against the opposite polyline at matching arc-length fractions, so short hooks and tight curves cannot hide between samples. It checks starting position, ending position, direction (including folded stroke reversals), stroke length, average geometric deviation, and peak deviation in the same normalized canvas coordinates. It deliberately does not independently normalize the two stroke shapes because that would ignore positional errors.
- `zipeek-hanzi-writing::session` owns accepted stroke count, mistakes, rejection reasons, and completion. A rejected stroke does not advance progress. Undo removes one accepted stroke, and Clear resets the session.
- `hanzi_references.rs` owns the shared, once-per-session aggregate fetch and single-character lookup, while `hanzi_writing/practice/stroke_loading.rs` constructs the validated writing session. Requests from a previous character are invalidated on selection changes and stale results are rejected by character identity in the reducer.
- `hanzi_writing/practice/canvas_input.rs` is a pointer-event adapter: it captures/releases pointers and dispatches typed begin, move, finish and cancellation actions. `practice/controller.rs` is the single reducer owning ink, selected character, guide preference, loading status and graded session. Only its finish transition can submit a trace to the pure Rust session. `practice/board.rs` renders the canvas/overlay; `practice/controls.rs` renders action buttons using callbacks owned by the coordinator. A completed pointer gesture is not accepted by itself.
- `HanziWriting` renders the active pen trace on a canvas. An SVG overlay renders canonical stroke outlines. Accepted strokes snap to the exact sourced outline; the next expected outline and its starting point are highlighted when the guide is on. Missing or malformed reference data disables handwriting acceptance.

This is geometric guided tracing against canonical data, not optical recognition of arbitrary handwritten Chinese characters, recognition of alternative stroke order, or a trained handwriting model. The default matching thresholds are engineering starting values and need calibration with real mouse, touch, and stylus input to determine false acceptance and rejection rates.

## Reference data and licensing

The source-only checkpoint deliberately excludes the licensed upstream Hanzi Writer Data distribution and generated `characters.json` reference bundle from pack/unpack; install those assets separately. To regenerate it from the vendored upstream distribution, run from the repository root:

```text
cargo run --locked -p zipeek-cli -- hanzi prepare-references <upstream-json-directory> crates/zipeek-web/static/hanzi-strokes <path-to-ARPHICPL.TXT>
```

For repeatable local setup, run `cargo make prepare-hanzi`; it reuses an existing valid `characters.json` bundle with its manifest and licence. Otherwise `zipeek-cli hanzi prepare` uses `external/hanzi-writer-data/data` and `external/hanzi-writer-data/ARPHICPL.TXT`, cloning a sparse checkout of Hanzi Writer Data with Git when the source directory is absent. Git must be installed and the initial download requires network access. If an incomplete source directory already exists, the command rejects it rather than overwriting it. The Git subprocess runs with the project root as its working directory and a relative checkout destination to avoid Windows extended-length (`\\?\`) destination paths. Clone and final checkout run quietly, suppressing normal Git progress while preserving stderr diagnostics and failure status; sparse-checkout does not request a quiet option. Use `cargo run --locked -p zipeek-cli -- hanzi prepare --force` to regenerate an existing bundle. Source-only ZIPs exclude the licensed upstream inputs and generated stroke bundles.

The Rust importer validates every glyph before writing the single `characters.json` bundle keyed by lower-case Unicode code point, copies the unmodified licence and a modification notice, and writes a manifest with per-glyph and whole-bundle SHA-256 hashes. On a successful write it removes only legacy hex-named per-character JSON assets. Trunk copies `static/hanzi-strokes` into the site. Both browser consumers use the relative `hanzi-strokes/characters.json` URL, cache one shared in-flight fetch for the browser session, and select individual glyphs in memory. A single initial download is approximately 30.7 MB uncompressed for 9,574 glyphs; enable Brotli/gzip at the hosting layer. The browser does not verify SHA-256 at runtime; the manifest is for package integrity verification.

The runtime does not download character data from a third-party CDN. Without packaged data it shows an unavailable state and does not accept strokes. Successful aggregate loads remain cached, while failed aggregate requests are evicted and may be retried from the handwriting workspace. Concurrent requests still share one in-flight fetch.

## Verification

Run `cargo test --locked -p zipeek-hanzi-writing`, `cargo check --locked -p zipeek-web --target wasm32-unknown-unknown`, and `cargo test --locked -p zipeek-e2e --test writing_interactions -- --nocapture` on Windows, after preparing the relevant assets. Inspect stroke orientation and pointer behavior in Chrome. The new E2E scenario also verifies the unavailable-data state if no assets were packaged.

Pure Rust regression tests cover near-correct curves, reverse direction, translated strokes, truncated strokes, unrelated curvature, stroke order, undo, and reset. Browser and real-device calibration is outstanding.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/architecture-modules-hanzi-stroke-references.md). Dated implementation claims in that snapshot are not current build or browser verification.
