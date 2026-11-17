# Current work

## Technical-debt reduction: generated asset integrity

- Added source and deployment modes for an offline Node asset-integrity audit with file size and SHA-256 validation against the dictionary manifest, HTML preload verification, Worker completeness checks, and generated geography checks.
- Added eight offline regression tests and integrated the checks into the existing Cargo Make source and deploy preflight entry points.
- Updated Rust preflight so a source checkout without any generated Worker is accepted in source mode, while deploy mode still requires complete Worker output. Added Rust regression tests for both cases.
- The source archive currently lacks generated dictionary binaries, Worker bindings and Province Quest geography assets. They must be generated and verified before deploy acceptance; the audit intentionally identifies them.
- This work reduces the exposure under TD-009 but does not close it. Rust compilation, live deploy preflight and Chrome acceptance remain unverified.
- Relevant contracts: [asset integrity](../build-deployment/asset-integrity.md), [technical debt](technical-debt.md), [tasks](tasks.md).

# Current work

## Documentation cleanup and acceptance verification

- **Scope:** Consolidate authoritative product, architecture, Practice and E2E requirements after the twelve-section editorial restructuring. Preserve source checkpoint narratives in historical audits while keeping direct maintained links stable.
- **Completed source edits:** Rewrote the maintained Product, UI architecture, Practice setup, Write Hanzi, Courses mosaic, Cloud Match, Multiple Choice and appearance contracts. Archived their former implementation narratives as separate, non-authoritative source snapshots. Added current contracts for shared Practice exercise components, the long dragon header, nonblocking vocabulary eligibility calculation, Credits and developer/Codex E2E review. Updated owning indexes, acceptance criteria and task tracking.
- **Validation:** Docs index and local-link checks are required on each change. The historical broken-link warning remains separate from maintained-document results.
- **Not verified:** Rust/WASM compilation, real Docs reader navigation, Chrome E2E execution, playable video generation, visual dragon fidelity, accessibility testing and gameplay implementation.
- **Editorial cleanup:** The source-specific review of 53 numbered-development references has been recorded in [editorial reconciliation](editorial-reconciliation.md). Historical originals are preserved separately; old review status is not current acceptance.
- **Next:** Verify current implementation contracts against code, then complete Docs reader and browser/build validation on the configured development toolchain. Product requirements are not evidence of implementation.
- **References:** [Documentation quality](documentation-quality.md), [tasks](tasks.md), [acceptance](../testing-quality/acceptance/approved-behavior-acceptance.md), [E2E matrix](../testing-quality/e2e/e2e-requirements-matrix.md).

## Docs reader source verification progress

- Reconciled the stale Docs deep-link browser scenario with the existing tasks article and its current E2E heading.
- Authored separate section-navigation and search journeys, including a decoded issue screenshot check.
- Corrected the Docs reader curated filename ordering for the maintained lowercase filenames.
- Added validation of browser-test document references and twelve section samples to the Node Docs validator.
- Documentation validation passes locally, with the existing one frozen-history warning. Rust compilation and live Chrome execution remain unverified because Cargo and a runnable Zipeek build are unavailable in this environment.

## Source-to-documentation verification

- Inspected the current workspace manifests, database generator policy and artifact writer, installed dictionary manifest, startup route boundaries and cross-tab coordinator.
- Corrected current database row totals, storage versions, artifact filenames, and the false missing-table claim in the maintained architecture/data/testing documentation.
- Replaced stale web-shell implementation statements with source-evidenced status and recorded mismatches with approved startup/tab behavior.
- [Source accuracy audit](source-accuracy-audit.md) distinguishes source declarations from runtime verification and identifies follow-up gates. No Rust/WASM or Chrome build has been executed.

## E2E evidence stabilization handoff

- Added an offline Node evidence-integrity auditor with five self-contained regression tests, optional full MP4 decode, and human/JSON reports for Codex review.
- Browser harness now records checkpoint capture failures in each scenario manifest and feedback report, and supports a strict evidence gate that rejects missing video status, invalid recording status, or missing checkpoint timeline.
- Offline checks run locally: Node E2E evidence unit tests and Docs validation. Rust checks and live Chrome E2E have not run in this environment; compile and verify on Windows before accepting the source candidate.
- Next verification: `cargo check --locked -p zipeek-e2e --tests`, a targeted recording-enabled Chrome E2E scenario, then the offline auditor on its actual run directory. Preserve failed artifacts for Codex review.
