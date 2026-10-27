# Regeneration

Commands, manual review/replacement procedure, and measured sizing. Part of the
[database documentation](DB.md). Source rules in [sources.md](sources.md);
reproducibility in [reproducibility.md](reproducibility.md); artifact structure
in [artifacts.md](artifacts.md).

## Commands

Generation remains explicit and manual.

In-memory checking:

```powershell
# Repository root
cargo run -p db-gen -- --check
```

Initial generation, only when web/static/ does not exist:

powershell
cargo run -p db-gen -- generate
Regeneration into a fresh review directory:

powershell
cargo run -p db-gen -- generate target/reviewed-dictionary
The generator refuses an existing output directory and never overwrites a
previous bundle.

Review and replacement procedure
Run the review-directory command above.

Compare the new integrity report (hanzi.manifest.json), ordered logical
rows/schema, indexes and diagnostics (hanzi.diagnostics.json) against the
current web/static/ bundle.

Preserve a backup of the previous bundle during review. Never patch database
bytes or replace only the file without its matching report.

After verification, replace the complete web/static/ bundle in an approved
data change. Include the regenerated artifact, manifest, diagnostics, inputs
manifest, attribution and notices together.

Re-run ../testing/browser-db.md compatibility
checks against the new file; the previous compatibility is not implied for a
new snapshot.

No source acquisition, generation, overwrite or migration runs during app builds.

Static and Pages serving
web/index.html uses copy-dir with data-target-path set to a dot, copying
static contents unchanged to the distribution root, following the
Trunk asset declaration.
The Pages path is /zipeek/hanzi.duckdb; the native development path is
/hanzi.duckdb. Report, INPUTS.toml, attribution and notices accompany it.

Measured sizing
Snapshot	Rows	Diagnostics	DB bytes
A05 working tree	12,100	514	3,158,016
A05 independent repeat	12,100	514	3,158,016
Native read-only reopening and exact HTTP byte serving pass. Browser
query/download/memory impact is measured via A06's Chrome and Pages tests; see
../modules/browser-db.md for current facts. This size
is not a browser performance claim beyond A06's verified load.

Related
Input sources and their pinned hashes: sources.md.

Import rules applied on regeneration: import-rules.md.

Artifact structure and verification: artifacts.md.

Logical reproducibility rule: reproducibility.md.

Browser compatibility check: ../../testing/browser-db.md.