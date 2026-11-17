# Release archive naming and installation

## Numbered source handoffs

The current native `zipeek-cli` unpack discovery accepts two timestamped forms:

```text
zipeek-R<number>-<description>-YYYYMMDD-HHMMSS.zip
zipeek-R<number>-WIP-<description>-YYYYMMDD-HHMMSS.zip
```

A WIP checkpoint is not an accepted major release. Use a real UTC packaging timestamp, a positive decimal release number and a lowercase hyphen-separated description. Check the latest established release before incrementing the number.

Examples:

```text
zipeek-R285-native-ai-server-20261109-221530.zip
zipeek-R285-WIP-ai-transport-20261109-221530.zip
```

`cargo make unpack` searches Downloads for the highest numbered native release or checkpoint. When both a completed release and a WIP checkpoint have the same number, the completed release has priority. An explicit archive path avoids relying on Downloads selection:

```powershell
cargo run -p zipeek-cli --locked -- unpack C:\path\to\archive.zip
```

Before delivering an archive, validate that its filename is recognized by the actual parser in `crates/zipeek-cli/src/command.rs`. Check the ZIP structure for `Cargo.toml`, `Cargo.lock`, `AGENTS.md`, the `crates/`, `docs/` and `scripts/` trees at the ZIP root, with no wrapper directory. The CLI also validates required source files before modifying the checkout. Preserve local edits and backups before unpacking.

The legacy `unpack.ps1` expects an older date-only naming format. The current time-stamped handoff contract is authoritative for `cargo make unpack`; do not assume legacy PowerShell acceptance. Do not silently rename an old archive to a higher release number.

## Native source snapshots

`cargo make pack` produces a different archive under `target/`:

```text
zipeek-source-<workspace-version>-YYYYMMDD-HHMMSS.zip
```

A source snapshot does not use the numbered handoff naming convention. To restore it, supply the path explicitly to the CLI. Review excluded generated assets and source completeness before using any archive as a release handoff.

## Numbered major-release acceptance

The following **major-release-only** gate does not apply to routine tasks, WIP checkpoints, documentation-only changes or native `cargo make pack` snapshots. A numbered major release requires purposeful improvements to at least **20 distinct `.rs` paths**, compared against the agreed baseline. Count each qualifying path once from the final diff; generated code, placeholders, wrappers without meaningful behavior, renames, formatting-only edits, comments and non-Rust files do not qualify.

Do not inflate the count by splitting modules, adding hypothetical features or modifying unrelated code without approval. If the planned work reaches fewer than 20 qualified files, report the shortfall and propose additional valuable changes with risks, validation and expected affected paths **before** expanding scope. If not approved, retain an incomplete or unverified candidate rather than calling the major release accepted.

Before delivering an accepted numbered major release, provide a file-by-file qualifying Rust inventory, separate non-Rust/filtered counts, the actual final diff baseline, applicable test commands and results, unresolved verification, known failures, data-loss implications and archive layout validation. With unavailable build/browser tooling label the output **unverified release candidate**, not a tested release.
