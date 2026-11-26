# Documentation quality and remaining reconciliation

## Organization and authoritative ownership

The twelve top-level documentation sections correspond to the twelve sections in the application Docs reader. Maintained articles use lowercase kebab-case filenames, except conventional README entry points. Topic subfolders are preferred when they separate substantially different subjects; folder depth should not exceed three levels below the repository root for normal articles.

- Product and UX owns application-wide interaction policy, page-level behavior and accessibility.
- Learning and Practice owns exercise behavior, outcomes, pedagogy, course requirements and profile portability.
- Games owns game mechanics and feedback.
- Dictionary and Data owns data generation, storage, vocabulary selection and eligibility.
- Architecture owns module boundaries, state ownership and runtime contracts.
- Engineering owns coding standards and contributor workflows.
- Testing and Quality owns acceptance, browser scenarios and evidence policy.
- Build and Deployment owns CI, packaging, build tasks and optional services.
- Project Management owns active work, technical debt and approved decisions.
- Archive and History owns historical implementation narratives and superseded proposals.
- Issues owns defect reproductions and screenshots.
- Getting Started owns onboarding and first-run navigation.

The [credits](../product-ux/credits.md), [shared Practice foundation](../learning-practice/practice/shared-exercise-foundation.md), [header and dragon](../product-ux/foundation/header-and-dragon.md), and [eligibility counting](../dictionary-data/vocabulary/eligibility-counting.md) are maintained requirements, not implementation evidence.

The [current UX overview](../product-ux/ux.md), [Practice overview](../learning-practice/practice/practice-overview.md), and [browser E2E overview](../testing-quality/e2e/overview.md) are reader entry points. They link to authoritative topic contracts rather than duplicating them. The original dated narrative compilations were retained under [historical audits](../archive-history/audits/).

## Verification boundary

A passing Docs index and link check verifies path consistency, not editorial accuracy, browser rendering, or Rust compilation. Changes to the Docs reader need a Rust/WASM build and browser navigation test before being marked verified.

## Editorial reconciliation status

The former inventory of 55 maintained files was an estimate. The source-specific scan identified 53 files with numbered development references. All 53 have historical source snapshots, and their current-document disposition is tracked in the [editorial reconciliation inventory](editorial-reconciliation.md).

Twelve historical review pages were reduced to references linking the preserved source and current subject owner. The remaining 41 files retain implementation and technical detail with release chronology separated into the source archive. This resolves the identified release-marker cleanup task, not every possible factual disagreement between old documentation and the present code.

## Remaining verification work

- [Source-accuracy audit](source-accuracy-audit.md) has reconciled the workspace, database manifest, generator policy and selected startup boundaries. Continue verifying unsampled technical descriptions and the absent binary artifact in a running browser before asserting runtime completion.
- Inspect the Docs reader in real Chrome after the folder moves, including deep links, search, images and all twelve sections.
- Verify third-party licenses and asset provenance using shipped files, not merely aspirational Credits text.
- Align the E2E coverage matrix with executable Rust scenarios and run strict evidence capture on the supported browser environment.

One link in a frozen completed record still targets a removed source-contract script. Preserve that historical record; treat its warning separately from maintained-document validation.


## Release-review prevention and canonical-document workflow

A release must update the existing subject owner before treating a new design or implementation narrative as current documentation. Root-level R-numbered working notes and R-numbered reviews under `docs/engineering/` are not allowed as ongoing subject documentation. If a genuinely new subject lacks an owner, create a lowercase-kebab-case maintained article under the appropriate existing category and link it from `docs/README.md`.

Release-specific implementation narratives are reconciled into (a) current product/architecture/testing contracts, (b) still-open stable tasks/debt, and (c) a single historical review archive or release history. Date-specific checks and compiler/test claims remain historical and must never be promoted to current verification by relocation. Before removing old files, update maintained links, Docs index and browser-route fixtures. New `R...` Markdown documents under `docs/engineering/` require historical classification instead of creating a parallel authority.

The R610 documentation consolidation placed release-review source text into [one historical collection](../archive-history/audits/release-engineering-review-history.md) and linked the new [knowledge specification](../learning-practice/pedagogy/knowledge-points.md) and [scoring architecture](../architecture/modules/knowledge-scoring.md). This is an editorial ownership change only; no Rust tests or behavioural acceptance follow from it.
