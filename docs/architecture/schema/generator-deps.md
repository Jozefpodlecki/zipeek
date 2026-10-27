# Generator dependencies

Approved dependency pins, neo restoration, bundled-build blocker, and the
WASM compatibility probe. Part of the [database documentation](DB.md). Approved
dependencies in [D21](../../process/decisions/database-and-artifacts.md#d21-generator-dependencies-sources-and-engine-guard)
and [D23](../../process/decisions/database-and-artifacts.md#d23-native-neo-engine-and-compatible-storage).

## Current dependency block

D21 pins plus the user-requested neo restoration, pinned to its previous resolved
revision:

```toml
[dependencies]
serde = { version = "=1.0.229", features = ["derive"] }
serde_json = "=1.0.154"
thiserror = "=2.0.21"
tracing = "=0.1.44"
tracing-subscriber = { version = "=0.3.23", features = ["env-filter"] }
duckdb-neo = { git = "https://github.com/duckdb/duckdb-rs.git", rev = "4acbea84794582c4cd962ee971a3d7a5dcb72e39", features = ["chrono", "r2d2", "uuid"] }
sha2 = "=0.10.9"
toml = { version = "=0.9.8", default-features = false, features = ["std", "serde", "parse"] }
zipeek-core = { path = "../zipeek-core" }
```

All eight registry versions in the proposal were checked against the public
crates.io version API: none was yanked at audit.
The retained versions matched the audited local resolution; these exact pins
and feature changes now have explicit D21 approval. The tagged native wrapper
declares Rust minimum 1.84.1,
below this repository's 1.97.1. Registry MSRVs for serde/serde_json/thiserror,
tracing/subscriber, and toml are also below 1.97.1; sha2's metadata omits an MSRV,
so the audit itself made no build-success claim; A25 validation is tracked in
CURRENT. No new direct flate2 dependency is needed:
the generator will consume the verified uncompressed snapshot. Transitive build
dependencies may still include reqwest/flate2; removing direct application
dependencies does not remove every occurrence from the build graph.

A24 audit — accepted and rejected direct dependencies
At audit time, the ignored root Cargo.lock resolved duckdb-neo 0.20000.0 and
libduckdb-sys 1.20000.0 from Git revision
4acbea84794582c4cd962ee971a3d7a5dcb72e39. This identifies one local resolution;
the old manifest had no revision pin, and an ignored lockfile is not a reproducible
repository dependency contract. Root lockfile tracking needs separate approval
in A19/A25; exact direct requirements alone do not lock transitive dependencies.

Dependency at audit	Action	Reason
reqwest 0.13 (json, no defaults)	Remove	Generator consumes local verified files; source acquisition is separate
serde 1.0.171 (derive)	Pin =1.0.229, retain derive	Deserialize primary JSON and input manifest; exact version already resolved locally
serde_plain 1.0.2	Remove	Fixed PoS/HSK mappings can use explicit matches
serde_json 1.0.99	Pin =1.0.154	Parse source JSON with the currently resolved release
tokio 1.54.1 (full)	Remove	Manual local-file generation is synchronous
uuid 1.29 (v4)	Remove	Approved row-number WordIds are deterministic integers
thiserror 2.0.21	Pin =2.0.21	Typed generation/input errors
tracing 0.1	Pin =0.1.44	Structured entry/form rejection, conflict, transformation diagnostics
tracing-subscriber 0.3 (env-filter)	Pin =0.3.23, retain env-filter	Native command logging and RUST_LOG filtering
duckdb-neo (moving Git)	Restore at pinned rev 4acbea8… with chrono/r2d2/uuid	D23; bundled duckdb 1.4.3 proposal was blocked
zipeek-core (path)	Retain	Approved shared domain types (D01)
—	Add sha2 =0.10.9 and toml =0.9.8	Verify SHA-256 and parse MANIFEST.toml
Earlier bundled 1.4.3 proposal — evidence and limits
The fixed npm version 1.32.0 reports Git head
a20b8290e79e37a7b914e62621aa412d69ba95fc in its
registry metadata.
Its tagged README
identifies engine 1.4.3; its DuckDB submodule is
d1dc88f950d456d72493df452dabdcd13aa413dd, recorded by the
upstream contents API.
Do not interpret the npm package version as the native engine version.

The accepted crates.io duckdb 1.4.3 package depends exactly on libduckdb-sys
1.4.3. Both inspected crate archives record upstream revision
65e8496fc57098bc896d9fb2ec993b6f0de4e399; the sys archive contains
duckdb.tar.gz, used by the bundled build. Their archive SHA-256 values match
registry checksums:

Package archive	SHA-256
duckdb 1.4.3	d7eeb487dde618b9f6ab26a451775ad5fac3fabe1ca2b64cbbe90b105f264ccd
libduckdb-sys 1.4.3	c8c60c2d269e63ae5197e4fe9075efffed35dfda0095a5ac8b41f3c765b18456
See the exact wrapper archive
and native sys archive.
Bundled mode avoids accidentally linking a different system DuckDB; it requires
a working C++ compiler/toolchain and builds the packaged native source. The
wrapper still has transitive Arrow dependencies even without optional features.
No JSON/Parquet/chrono/connection-pool/UUID DuckDB features are proposed.

Matching engine releases and the documented
storage compatibility rules
were evidence for the earlier choice, not a successful cross-client test. D23
now requires the generator to query SELECT version() on its restored neo
connection, log the raw result and reject anything except v2.0.0 before
import/output creation. Storage compatibility must be set and verified as
v1.4.0; query failures also fail the invocation. A25 adds guard/storage tests
and does not add a Cargo build.rs or automatic application-build generation
step. The guard opens only an in-memory connection. A05 exports the dictionary
through that guarded connection.

Earlier A25 bundled native build blocker (2026-10-26)
The earlier bundled guard/tests initially could not run. With Rust/Cargo 1.97.1,
bundled libduckdb-sys 1.4.3 compilation selected MSVC 19.51.36248
(14.51.36231/bin/HostX64/x64/cl.exe) and failed in vendored fmt:

text
error: failed to run custom build command for `libduckdb-sys v1.4.3`
.../duckdb/third_party/fmt/include/fmt/format.h(326): error C2653: 'stdext': is not a class or namespace name
.../duckdb/third_party/fmt/include/fmt/format.h(326): error C2061: syntax error: identifier 'checked_array_iterator'
Reproduced with cargo test -p db-gen and cargo test --workspace; Clippy also
hits the native build failure and existing app.rs unused imports. No dependency
substitution, compiler-flag workaround, or source/header patch was applied. The
user subsequently restored neo; its native build now passes. D23 then approved
native v2.0.0 and mandatory compatible storage, resolving that blocker. This is
not the deferred A06 native-to-WASM read-only test; that remains an open question
and is not an A25 prerequisite.

Neo-to-WASM compatibility probe (2026-10-26)
The user requested restoring duckdb-neo and then asked for a direct file-read
check before changing the native-engine guard. Neo is restored at the prior
resolved Git revision 4acbea84794582c4cd962ee971a3d7a5dcb72e39, retaining its
previous chrono/r2d2/uuid features and prebuilt-library defaults. Other D21 pins,
inputs and D22 policy are unchanged. Pinning the revision avoids advancing the
Git source while restoring it. The native query reports v2.0.0.

A temporary native neo executable created two independent eight-row fixtures
using the exact ten-column words DDL, PK/display-key uniqueness and two ordinary
HSK indexes. It checkpointed, detached and closed both files. Neither fixture
is the final dictionary artifact; they contain synthetic Chinese/pinyin/English
rows with NULL metadata and different readings of the same hanzi. Files and
probe tools are under ignored target/a25-compat, not app assets.

Exactly @duckdb/duckdb-wasm 1.32.0 was installed only in that isolated directory.
Its node-blocking bindings instantiated the actual packaged WASM (engine
v1.4.3) and attempted read-only opens:

Neo 2.0 writer configuration	Pinned WASM outcome
Default storage settings	Rejected: file version 999; reader accepts 64 through 67
ATTACH with STORAGE_VERSION 'v1.4.0'	Passed: all 8 complete rows, 10 columns and both named HSK indexes matched; DELETE was rejected as read-only
The passing test compares every column, including original Unicode/pinyin,
pipe-separated glosses and NULL values. This is direct WASM-engine evidence in
Node 24.17.0, not a browser-worker/HTTP deployment test or full-dictionary test.
A06 retains those integration checks. Default neo output cannot be assumed
readable by this pinned WASM; the explicit older storage setting matters.

Reproduce the preserved scratch probe from the repository root:

powershell
cargo run --manifest-path target/a25-compat/native/Cargo.toml --target-dir target --offline
node target/a25-compat/read-wasm.cjs
The native scratch producer refuses to overwrite existing fixture files; use a
fresh scratch location for regeneration. Results are saved in
target/a25-compat/results.json. After reviewing this evidence the user approved
D23: require native v2.0.0 and v1.4.0 storage compatibility. The earlier native
v1.4.3 guard is superseded.

Related
Engine/storage facts: runtime.md.

Artifact export verification: artifacts.md.

Source bytes the generator consumes: sources.md.