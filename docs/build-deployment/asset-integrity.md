# Generated asset integrity and deployment preflight

## Source checkout versus deployable build

A complete source checkout does not necessarily include the generated dictionary binary, compressed dictionary, Province Quest geography, or pedagogy Worker bindings. A source ZIP passing integrity checks is not evidence that these assets are present or that the website is deployable.

Use `cargo make asset-integrity-source` to check the files actually present in a source checkout. It reads `static/hanzi.manifest.json`, validates included manifest-declared files using their recorded sizes and SHA-256 hashes, validates the dictionary preload path in `index.html`, and checks representative Courses and Practice artwork. Missing generated dictionary, Worker, and geography artifacts are reported as **notices** in source mode. Missing manifest-declared source metadata, such as attribution or licensing files, is an error.

Use `cargo make asset-integrity-deploy` before publishing. Deploy mode treats missing generated dictionary, Worker, and Province Quest geography artifacts as **errors**. A manifest-declared artifact that is present but has changed size or hash is an error in both modes. A partial Worker output is always an error. The audit never generates, installs, deletes, or repairs assets and never changes dependency versions.

`cargo make preflight-source` and `cargo make preflight-deploy` also invoke the corresponding Node checks before the existing Rust preflight. The Rust source preflight permits an entirely absent generated Worker, but deploy preflight requires a complete Worker with a compatible bootstrap.

Run offline regression tests with `cargo make asset-integrity-tests`. For direct execution without Rust: `node scripts/asset-integrity-audit.cjs --source` and `node --test scripts/tests/asset-integrity-audit.test.cjs`.

## Limits and acceptance

The audit validates recorded file hashes and basic headers, not DuckDB table contents, complete WebP/PNG decoding, grammar correctness, or actual browser asset loading. The existing Rust preflight verifies Docs index consistency and additional media properties. A successful deploy preflight does not replace a locked WASM build, the `/zipeek/` navigation check, or real Chrome acceptance. Never mark technical-debt acceptance closed based solely on either preflight.

The current source archive omits generated dictionary binaries, Worker output, and geography files. Source-mode notices reflect that known distinction; deploy mode is expected to fail until the approved generation and installation steps are completed.

The uncompressed `.duckdb` file in the generator manifest is a build input and may be absent from the deployed static directory when the compressed `.duckdb.gz` artifact is present and validated. The deploy gate requires the compressed artifact, not an unnecessary duplicate uncompressed database.
