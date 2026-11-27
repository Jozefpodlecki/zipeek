# Generated asset integrity and deployment preflight

The canonical asset preflight lives in Rust: `zipeek-cli build verify-assets` and `crates/zipeek-cli/src/asset_preflight.rs`. The former root Node audit has been retired. Do not interpret these commands as equivalent to the removed independent SHA-256 manifest audit.

Run `cargo make asset-integrity-source` for the source-mode Rust preflight. It validates the Docs index, required artwork, and pedagogy Worker bootstrap; a completely absent generated Worker is permitted. Run `cargo make asset-integrity-deploy` to require the complete Worker bindings. Both commands may fail on a checkout without the assets they require.

`cargo make asset-integrity-tests` runs `zipeek-cli`'s Rust preflight regression tests. `cargo make preflight-source` and `cargo make preflight-deploy` invoke the same Rust preflight again through their Makefile dependencies and direct commands; there is no Node process.

These checks are not a substitute for validating dictionary manifest hashes, Province Quest geography, the generated DuckDB tables, or browser loading. Regenerate and verify the dictionary through `zipeek-cli db-gen`, install it through `zipeek-cli db-install`, and run native/Browse checks and real WASM/browser acceptance before releasing. Missing independent manifest/hash audit coverage should be restored as Rust functionality in a separate approved milestone.
