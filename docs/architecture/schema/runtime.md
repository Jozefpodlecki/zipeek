# Runtime

DuckDB-WASM version pin, engine/storage facts, and the OPFS rationale. Part of
the [database documentation](DB.md). Approved pin in
[D08](../../process/decisions/database-and-artifacts.md#d08-duckdb-wasm-version);
engine/storage in [D23](../../process/decisions/database-and-artifacts.md#d23-native-neo-engine-and-compatible-storage).

## Runtime version (D08: approved 2026-10-26)

Pin the npm package exactly to `@duckdb/duckdb-wasm@1.32.0`; do not use a range,
`latest`, `next`, or a development build. A06 installs this exact npm version
and packages matching local worker/WASM assets; see the A06 verification in
[artifacts.md](artifacts.md#a06-final-dictionarybrowser-integration).

The [DuckDB team's OPFS article](https://duckdb.org/2026/09/18/opfs-wasm), published
2026-09-18, recommends pinning 1.32.0 as an option because the then-current
`latest` build had an OPFS persistence bug. This is the historical rationale for
the approved pin; it does not introduce OPFS persistence into Zipeek's read-only
runtime requirements.

## Engine and storage facts

The pinned package and its compatibility with the generated artifact:

| Fact | Value |
| --- | --- |
| npm package | `@duckdb/duckdb-wasm@1.32.0` (exact) |
| WASM engine | `v1.4.3` |
| Native generator engine | `duckdb-neo` v2.0.0 at rev `4acbea84794582c4cd962ee971a3d7a5dcb72e39` (D23) |
| Storage compatibility | `v1.4.0` (explicit, required by D23) |
| Access mode | `READ_ONLY`; full fetched buffer |
| OPFS | not used |
| Cross-origin isolation | not required for the `/zipeek/` build |
| Artifact | 12,100 rows, 3,158,016 bytes |
| Current SHA-256 (A44/A46, 2026-10-29) | `728dca2400ba8f23877043c6216a433e9c0adfad539097f3081cfb200605eb02` |

Default neo output (storage version 999) is rejected by this pinned WASM; only
explicit v1.4.0 storage is readable. Future regenerated artifacts must repeat
the compatibility checks; no default neo storage compatibility is implied.

The engine version and storage version are distinct facts:

- **Engine version** is what `SELECT version()` reports on the native connection
  and the version the WASM runtime reports for its built-in DuckDB.
- **Storage version** is the file-level format the writer produces. Even a
  v2.0.0 engine can write a file in v1.4.0 format when `storage_compatibility_version`
  is set explicitly before output creation.

D23 requires native `v2.0.0` and storage `v1.4.0`. D08 requires exact WASM 1.32.0.
Together they are the pinned compatibility triple for this snapshot.

## Compatibility blocker protocol

When integrating, verify the pinned runtime against the wasm toolchain and native
generator's database format. If a genuine compatibility blocker appears, stop and
report the exact versions, reproducing commands, and error evidence. Do not
silently switch to another release or tag.

Loading strategy, native generator compatibility, and deployed assets are
implemented (A05/A06); any change to the runtime pin requires explicit approval
and re-verification.

## Related

- Generator dependency pins: [generator-deps.md](generator-deps.md).
- Artifact export and browser verification: [artifacts.md](artifacts.md).
- Browser query implementation: [../modules/browser-db.md](../modules/browser-db.md).
- Testing commands: [../../testing/browser-db.md](../../testing/browser-db.md).
