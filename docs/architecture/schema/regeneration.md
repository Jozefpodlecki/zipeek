# Regeneration

Generation is explicit and manual. Commands below run from the repository root.
No source acquisition or database generation runs during application builds.

## Choose the command

For the complete local workflow, run from the repository root:

```powershell
cargo make dev
```

This generates a fresh candidate, verifies and installs the whole bundle with
a backup, then starts Trunk at http://localhost:1420. It runs once per invocation;
Trunk rebuilds do not regenerate data. Prerequisites are installed cargo-make,
Node, Rust/native generator prerequisites, Trunk and the locked web npm packages
(npm ci --ignore-scripts from web/ when they are absent). Tasks do not install
or upgrade tools/packages. Tested cargo-make version: 0.37.24.

| Task from root | Behavior |
| --- | --- |
| cargo make | List available tasks without changing the database |
| cargo make db-check | Verify/import in memory without writing a database |
| cargo make db-generate | Generate into a fresh target/db-workflow/generated directory |
| cargo make db-install | Install the latest successful candidate; never generate implicitly |
| cargo make serve | Start Trunk using the installed bundle |
| cargo make dev | Generate, install and serve in that order; stop on failure |

Each run prints absolute candidate/database/backup paths. The latest successful
candidate is recorded in target/db-workflow/latest.json. Installation validates
policy, every declared file's size/hash, required reports/notices and the pinned
INPUTS.toml before any replacement. It stages and rechecks the complete bundle,
backs up the previous web/static under target/db-workflow/backups and restores
that backup if publication fails. Candidate/backup directories are retained.
Unsafe paths, symlinks, missing notices and checksum mismatches are refused.
Copy the whole bundle, never just hanzi.duckdb. These operations are explicit
local tasks, not Trunk hooks or application startup behavior.

Direct generator commands remain available:

```powershell
# Show help and examples; no database is written
cargo run -p db-gen --locked -- --help

# Verify sources, native engine/storage and import in memory; write no database
cargo run -p db-gen --locked -- check

# Write a full bundle to a NEW directory
cargo run -p db-gen --locked -- generate --output target/reviewed-dictionary
```

A bare cargo run -p db-gen now displays Clap help, does no import/generation and
returns usage status 2. Previously it imported in memory without writing a file;
that behavior is now the explicit check command. Successful check ends with
"No database file was written" plus the generation command to use next.

Generate prints the absolute output directory before work and the absolute
hanzi.duckdb and hanzi.manifest.json paths after success, even if informational
logging is disabled. Relative --output paths resolve from the current working
directory. Paths with spaces can be quoted. --help and --version exit successfully.

## Default output and existing directories

```powershell
# Uses the repository web/static directory; only works if it does not exist
cargo run -p db-gen --locked -- generate
```

The default is repository-relative web/static, independent of the shell's working
directory. Its normal checked-in existence means this command refuses to overwrite
it. Use an explicit fresh review directory for regeneration. Existing outputs are
always refused; there is no implicit overwrite or --force option.

## Review and replacement

1. Generate into a fresh review directory.
2. Compare hanzi.manifest.json, ordered logical rows, schema, indexes and
   hanzi.diagnostics.json against the current web/static bundle. Database file
   hashes can differ with identical logical content under D22.
3. Preserve a backup of the previous complete bundle. Never patch database bytes
   or replace only the database without its corresponding reports/notices.
4. After approval and verification, replace the complete web/static bundle:
   hanzi.duckdb, manifest, diagnostics, INPUTS.toml, ATTRIBUTION.md and notices.
5. Repeat the applicable native/read-only and pinned browser compatibility checks
   in [browser-db](../../testing/browser-db.md). Old compatibility evidence does
   not automatically validate a new artifact.

The approved sources/schema/dependencies do not change merely by regeneration.
Native generation requires v2.0.0 and v1.4.0 storage compatibility before output
creation. Artifact export checks complete logical equality and read-only reopening.

## Static serving and measured size

Trunk copies web/static contents to the distribution root. The Pages database
URL remains /zipeek/hanzi.duckdb; development uses /hanzi.duckdb. Reports and
notices accompany the database. The recorded A05 snapshot has 12,100 readings,
514 diagnostics and a 3,158,016-byte database; file size is not a performance claim.

## References

- [Sources and notices](sources.md)
- [Import rules](import-rules.md)
- [Artifacts](artifacts.md)
- [Logical reproducibility](reproducibility.md)
- [Generator module](../modules/db-gen.md)
