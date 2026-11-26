# Repository technical debt audit, 2026-11-19 (R588 WIP)

## Basis and confidence

Baseline is `zipeek-R587-WIP-shared-record-collections-20261119-144442.zip`. This audit inspected the root engineering, testing and release rules, workspace structure and selected high-risk implementation boundaries. A static inventory counted **970 Rust files** and approximately **152,175 Rust source lines** across 21 crates. This is not a line-by-line semantic or security assessment of every function. No Rust toolchain, WASM target or running Chrome session was available in the audit environment.

The three checked offline Node test suites returned **16 passes, 0 failures**. That result validates only those suites, not Rust code, browser interactions or the changes authored in this checkpoint. ZIP CRC and manifest checks are packaging integrity checks only.

## Fixes in this checkpoint

| Scope | Finding | Resolution | Acceptance evidence |
| --- | --- | --- | --- |
| `zipeek-cli` source unpack | ZIP extraction had path traversal and symlink guards but no bounded extracted size or entry count. A compressed archive could consume unexpectedly large disk space before stage validation. | Enforce at most 40,000 entries, 256 MiB per file and 2 GiB declared expanded data, with bounded copying and tests for count, file size and total size. | Source review; Rust tests authored, not executed. Baseline source ZIP totals ~225 MiB extracted with a largest entry ~52 MiB, inside limits. |
| Statistics Overview chart | Changing the weekly bucket data did not reset `ready` and `error` at the start of a new chart load; missing canvas could leave an unexplained placeholder. | Reset chart load flags on bucket changes and surface the existing failure fallback when the canvas cannot be resolved. | Source review; browser test outstanding. |
| Statistics Overview activity | The breakdown treated every category other than Practice as Games. This could mislabel unknown categories from future sources. | Count Games explicitly and leave unknown categories unclassified, with a focused regression test. | Test authored, not executed. |
| Release workflow | `cargo make release-check` exists, but a standard `cargo make pack` does not require it; previous source handoffs repeatedly bypassed compiler verification. | Added the explicit `cargo make pack-verified` workflow, combining the existing release-check gate with source packaging, without altering the lightweight native pack command. | Task configuration reviewed; cannot execute Cargo here. |
| README | Build instructions referenced `scripts/*.test.cjs` though tests are under `scripts/tests/`. | Corrected the executable Node test command. | Three offline test suites actually run successfully. |

## Open technical debt, prioritized

### Blockers: verification and contracts

- **Rust/WASM release gate:** R587 still has no successful compilation evidence. `cargo fmt --all -- --check`, locked workspace tests, locked WASM check, lint and Chrome acceptance must run in a Rust-equipped environment. Source-level verification cannot substitute for them. This also applies to new R588 Rust edits.
- **Statistics learning transfer:** Selection uses preview string identifiers. Canonical Word Set and Practice transfers remain unimplemented. Do not silently map demo IDs to real records.
- **Statistics dialogs:** Selection review, Grammar details and Overview breakdown still lack a demonstrably shared focus containment and background interaction contract. Test Tab/Shift+Tab, Escape, restoration, and narrow-screen overflow before acceptance.

### Priority architectural debt

- **Large workflow owners:** `crates/zipeek-web/src/pages/practice/local_session.rs` (~799 lines), `crates/zipeek-web/src/pages/system/settings/mod.rs` (~645), `crates/zipeek-exercises/src/cloud_match.rs` (~1,103), `crates/zipeek-web/src/pages/system/stats/query.rs` (~612), `stats/grammar/page.rs` (~401), and `stats/state/mod.rs` (~484) need responsibility/invariant reviews. Their length alone is not a defect; do not mechanically create files to lower the counts.
- **Statistics reducer:** Shared filters, selection, overlay and section navigation state use one reducer. Retain atomic transitions. Prefer typed domain identities at integration boundaries and small pure helpers; do not introduce independent contexts without an actual shared-ownership need.
- **Query and demo scalability:** The current simulator caches shared immutable snapshots, but per-query sorting and lowercase string creation iterate the matching catalogue. Profile with 5,000/10,000 record scenarios before introducing caches or workers.
- **Native and browser trust boundaries:** Archive unpack has additional bounds in this candidate; browser storage and external API conversions still require targeted threat and invalid-input tests. This audit did not prove all such boundaries safe.
- **Safety/HTML policy:** Two Rust source files contain `unsafe {` blocks (`zipeek-db/src/lib.rs`, `zipeek-browser/src/charts.rs`). `zipeek-docs` sanitizes Markdown raw HTML and unsafe URLs before the web component constructs HTML; these remain future focused audit targets, not confirmed vulnerabilities.
- **Lint hygiene:** `zipeek-docs/src/lib.rs` contains crate-wide allow attributes. Review them under a functioning Clippy toolchain before narrowing/removing; do not blindly remove flags in an uncompiled checkpoint.

## Verification performed

- Read release naming and source unpack contracts, root development instructions, CI configuration and selected affected source.
- Reproduced the baseline source ZIP footprint statically and confirmed the new extraction limits exceed the native source archive's advertised size.
- Executed `node --test scripts/tests/e2e-evidence-audit.test.cjs scripts/tests/asset-integrity-audit.test.cjs scripts/tests/installation-workflow.test.cjs`: 16 passed, 0 failed.
- Tool availability: Node installed; Rust/Cargo/rustfmt absent. External Rust downloads were unavailable due to DNS resolution.

## Remaining acceptance commands (Windows, repository root)

```powershell
cargo make release-check
cargo test --workspace --locked
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo make e2e-full
cargo make pack-verified
```

The final packaging command is intentionally separate from compiling a numbered accepted release. A passed build still does not establish browser usability or production readiness. Do not mark this checkpoint accepted until the toolchain and browser gates complete.

## R589 continuation: overlay accessibility

Statistics Selection Review, Grammar detail and Overview breakdown now share a source-level initial-focus, Tab wrap and restoration helper. Forward/reverse boundary tests have been added but not executed. This is not an accessibility certification: Chrome keyboard focus, background inertness, screen readers and mobile behavior still require evaluation. The Rust/WASM gate remains unresolved. The two existing `unsafe` blocks were inspected and are WASM JavaScript import wrappers; they were not modified or classified as confirmed vulnerabilities.
