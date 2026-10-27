# Data flow

Runtime data flow, artifact publication, and Pages serving. Part of the
[architecture overview](../ARCHITECTURE.md). Sources and licensing in
[D06](../../process/decisions/database-and-artifacts.md#d06-sources-and-licensing)
and [D16](../../process/decisions/database-and-artifacts.md#d16-primary-import-and-ids);
artifact reproducibility in [D22](../../process/decisions/database-and-artifacts.md#d22-logical-database-reproducibility).

## Runtime data flow

The static web app uses Rust/Yew compiled to WebAssembly, `yew-router` for the
[product routes](../../product/PRODUCT.md), and Tailwind for styling. It runs
under `/zipeek/`.

Intended data flow:

1. A developer manually runs the native `crates/db-gen` binary against approved,
   licensed sources.
2. The generator produces `web/static/hanzi.duckdb`, which is committed and
   deployed with the static app.
3. DuckDB-WASM opens that file read-only in the browser.
4. Query results drive browse pagination, Word of the Day, and practice rounds.

D16 supersedes D06's initial source plan: use pinned
`jelleverheyen/hsk-vocabulary` `complete.json` for the HSK union, both
hanzi/pinyin forms, memberships, PoS, rank, and meanings. CC-CEDICT is consulted
only for an empty primary gloss list. The selected repository declares MIT;
retain its notice and upstream data attribution. The superseded unlicensed
leonsilicon inputs are not used.

During db-gen import, D18 lowercases source PoS codes before the fixed 42-code
lookup; any unrecognized code rejects the entire source entry/all forms with a
warning before reading-key grouping and ID assignment. Stored transcriptions
remain untouched. Primary import and CC-CEDICT fallback implement these rules.
See [schema/DB.md](../schema/DB.md) for the immutable pin, audited formats, import
rules, and notices. Visible footer attribution is implemented in `AppFooter`;
database-backed page content remains future work.

The approved D07 storage contract is one physical `words` table containing only
the HSK 2.0/3.0 union from the primary dataset, with optional CC-CEDICT gloss
fallback. No normalized membership/gloss/PoS tables are used in v1. db-gen groups
readings by simplified/numeric pinyin and validates correspondence to D07's
display-pinyin `UNIQUE` constraint. It preserves source display/numeric pinyin,
pipe-separated glosses, nullable comma-separated PoS, separate nullable HSK
columns, and nullable frequency. Two ordinary HSK indexes supplement implicit
constraint indexes. IDs use row numbers ordered by simplified/numeric pinyin;
they reproduce unchanged input but can shift after regeneration. Word of the Day
and Random use the full HSK-union table, not all of CC-CEDICT.

Generator schema/import/export, static-copy setup, browser loading and typed
queries are implemented. DuckDB-WASM uses exactly npm 1.32.0 (D08); the engine
reports v1.4.3 and reads the native v2.0.0 artifact with v1.4.0 storage. See
[schema/DB.md](../schema/DB.md) for the pin and compatibility facts.

## Artifact publication (A05)

The CLI defaults to in-memory checking; explicit `generate` optionally takes an
output directory and defaults to `web/static/`. Existing output directories are
refused. Export builds a new sibling staging directory, attaches a database with
explicit `STORAGE_VERSION v1.4.0`, creates the exact approved schema/indexes, and
inserts imported words. It compares complete rows and schema metadata against
the in-memory source, checkpoints/detaches, reopens read-only and compares again.
Only then does it hash the closed file, emit provenance/notices/diagnostics and
rename the staging directory to its final name. Failures do not publish a partial
bundle; cleanup is restricted to the directory created for that run.

Reports contain stable logical/diagnostics checksums separately from each file's
integrity checksum. The report does not checksum itself. Input manifest bytes
are retained at verification time, and notices are copied from verified buffers.
Two full source runs produced identical logical content with different physical
hashes, as allowed by D22.

## Pages serving

Trunk copies `web/static/` contents to the distribution root via `copy-dir` with
`data-target-path` set to a dot. The Pages artifact serves:

- `/zipeek/hanzi.duckdb` — the database file.
- `/zipeek/hanzi.manifest.json` — integrity report.
- `/zipeek/hanzi.diagnostics.json` — import diagnostics.
- `/zipeek/INPUTS.toml` — retained input manifest.
- `/zipeek/ATTRIBUTION.md` — attribution.
- `/zipeek/notices/` — byte-identical source notices.

Development serves the same files at `/`. Neither build nor app startup runs the
generator. Native read-only tests and HTTP byte checks pass; A06 verifies actual
browser-worker queries against this full artifact at `/` and `/zipeek/`.

## Assets and document base

`web/index.html` copies `web/public/` assets and the generated static bundle.
DuckDB-WASM worker/WASM/Arrow assets are packaged by the A06 pre-build hook into
ignored `web/vendor/duckdb` and served from there. No CDN is used.

`document.baseURI` resolves all runtime assets and the dictionary for `/` and
`/zipeek/`. Paths are not hard-coded to the deployment prefix.

## Background image

`web/public/background.webp` and the embedded bootstrap image are byte-identical
to the original. Their SHA-256 is
`64fc1cb7d2b34ff89ff03f5cef4bcfe3df64eae9f27c6ca073f72e8d2e201f93`.
No raster file is generated or changed at build time. The HTML loads it directly
and the A30 backdrop uses the same bytes.

## Regeneration

Commands, manual review, and replacement procedure live at
[schema/regeneration.md](../schema/regeneration.md). Generation is explicit and
manual; never an automatic application build step. Do not hand-edit a DuckDB
file. Regenerate it after an approved source or schema change and include the
generated output in that change.

## Related

- Schema details: [schema/DB.md](../schema/DB.md).
- Generator internals: [db-gen.md](db-gen.md).
- Browser queries: [browser-db.md](browser-db.md).
- Web shell and backdrop: [web-shell.md](web-shell.md).