# Technical debt register

Track implementation and verification liabilities here. [Tasks](tasks.md) is the active open-task triage index, and [historical acceptance ledger](../archive-history/LEGACY_ACCEPTANCE_LEDGER.md) preserves the unresolved historical checkpoint gates; [Current](current.md) records active checkpoint state. An item is resolved only when its acceptance evidence is recorded; source-contract tests alone do not prove Rust compilation or browser behavior.

| ID | Priority | Area | Evidence and impact | Resolution / acceptance gate | State |
| --- | --- | --- | --- | --- | --- |
| TD-001 | P0 | Rust/WASM and browser acceptance | A complete current-tree native/WASM build and managed Chrome acceptance have not been established in this archive. | `cargo fmt --all -- --check`, affected native tests, locked WASM check and Chrome E2E all pass on pinned Windows toolchain; attach diagnostics. | Open |
| TD-003 | P1 | Grammar assessment authoring quality | 24 draft MCQs and five construction challenges require qualified Mandarin review; plausible, unambiguous distractors and coverage cannot be proven by automatic syntax checks. | Language review, item revision record, semantic rationale checks, validated item provenance and independent assessment policy before mastery eligibility. | Open |
| TD-004 | P1 | Grammar assessment model | Authored content integrity and stable scoring snapshots require verification against the current implementation. | Bind scoring to complete immutable authored items, reject orthographically duplicate choices, verify wrong-answer/restart flows, and run native/WASM/Chrome checks. | Source hardening reported historically; native/WASM/Chrome verification open |
| TD-005 | P1 | Pedagogical evidence and worker | Opt-in UI is not activated; `snapshot_verified` is caller-provided, and application-level verified DuckDB readback / single-tab ownership are absent. | Approved consent/provenance contract, transaction/readback confirmation, single-tab exclusion, restart and browser failure tests. | Open |
| TD-006 | P1 | Docs Markdown | Parser migration and safe Markdown link rendering are implemented; cross-document heading targets previously conflicted with the Yew hash-router syntax. | Verify Rust fixtures, locked WASM build, Chrome navigation (including reload/history) under root and `/zipeek/`, and scroll-to-heading behavior. | Source-level fixes are documented; Rust/WASM and Chrome acceptance remain open. Compilation and Chrome acceptance remain open |
| TD-007 | P1 | Courses mosaic and curriculum | Grammar and vocabulary curriculum completeness, qualification rules, geometry and official HSK mappings require current source and browser verification. | Source-backed HSK mapping, content review, geometry screenshots and independently verified reveal predicates. | Open |
| TD-008 | P1 | E2E full journeys | Standard journey and pedagogical seven-day endurance journey have not been run successfully in managed Chrome; enabled variant depends on the unfinished evidence pipeline. | Run reproducible seeded journeys with screenshots, genuine scored interactions and explicit no-pedagogy/Dragon assertions. | Open |
| TD-009 | P2 | Generated static assets | Source archives require generated-asset preflight and cross-target verification; ZIP integrity alone does not establish deployability. Node source/deploy preflight and offline tests now exist, and Rust source preflight permits a not-yet-generated Worker. | Native build asset preflight that checks generated index, bootstrap, WASM and artwork against manifest; ZIP integrity alone is insufficient. | Partially addressed: static integrity gate authored and Node tests executed; real deploy, Rust, WASM and browser acceptance open |


## Review procedure

When modifying a feature, inspect its relevant debt IDs and update source/tests together. Report newly discovered debt, addressed debt, and remaining acceptance evidence in [Current](current.md) and release notes. Do not close a debt entry solely because a draft test was added.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../archive-history/audits/maintained-source-snapshots/project-management-technical-debt.md). Dated implementation claims in that snapshot are not current build or browser verification.
