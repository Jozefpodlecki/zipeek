# Documentation index

Use [AGENTS](../AGENTS.md) as the sole authoritative agent and contributor guide. Detailed implementation, validation and release rules are maintained by the linked topic documents. **Current behavior** is described by the code and the maintained documents below. Historical release claims and source-diff inventories are not product contracts.

| Topic | Maintained document |
| --- | --- |
| Coding standards and approved agent workflow | [Agent guide](../AGENTS.md), [Coding standards](engineering/coding-standards.md), [Release rules](build-deployment/release-naming.md) |
| Current implementation, proven passes and unverified gates | [Current](project-management/current.md) |
| Active unfinished work and triage | [Tasks](project-management/tasks.md), [legacy unresolved acceptance ledger](archive-history/LEGACY_ACCEPTANCE_LEDGER.md) |
| Prioritized engineering debt and verification gates | [Technical debt](project-management/technical-debt.md) |
| Grammar course authoring and structural validation | [Courses authoring](engineering/courses-authoring.md) |
| Product overview and UX | [Product](product-ux/product.md), [UX](product-ux/ux.md) |
| Future Courses mosaic and grammar assessments | [Courses mosaic spec](learning-practice/courses/courses-mosaic.md) |
| Statistics 2.0 dashboard design and implementation plan | [Statistics product design](product-ux/pages/learning-statistics-preview.md), [Statistics architecture](architecture/modules/statistics-dashboard.md), [Statistics acceptance](testing-quality/e2e/statistics-dashboard-design-acceptance.md) |
| Prospective Knowledge Points scoring policy and unresolved decisions | [Knowledge Points](learning-practice/pedagogy/knowledge-points.md), [scoring engine architecture](architecture/modules/knowledge-scoring.md) |
| Planned session evidence and legacy proficiency requirements | [Learning evidence](learning-practice/practice/learning-evidence.md) |
| Practice setup, storage and persisted translation | [Practice builder](learning-practice/practice/practice-session-builder.md), [Practice session storage](architecture/modules/practice-session-storage.md), [Sentence translation](learning-practice/translation/translate-sentence.md) |
| Word Sets, Browse, writing, games | [Dictionary Browse](dictionary-data/dictionary-browse.md), [Write Hanzi](learning-practice/hanzi/write-hanzi-activity.md), [Cloud Match](games/cloud-match.md) |
| Private data, backup and personal database status | [Personal data](dictionary-data/personal/personal-data-storage.md), [personal schema](dictionary-data/schema/personal-data.md) |
| Historical source-to-documentation evidence | [Archived source reconciliation](archive-history/audits/SOURCE_RECONCILIATION.md), [release engineering review history](archive-history/audits/release-engineering-review-history.md) |
| System architecture and dependencies | [Architecture](architecture/architecture.md), [UI architecture](architecture/ui-architecture.md) |
| Translation and AI responsibilities | [Sentence exercises](architecture/modules/sentence-exercises.md), [AI client](architecture/modules/ai-client.md) |
| Dictionary schema, generator and runtime | [DB index](dictionary-data/schema/db.md), [regeneration](dictionary-data/schema/regeneration.md), [runtime](dictionary-data/schema/runtime.md) |
| Testing strategy and E2E implementation | [Testing](testing-quality/acceptance/testing.md), [Chrome E2E](testing-quality/e2e/overview.md), [E2E architecture](testing-quality/e2e/e2e-architecture.md) |
| Decision provenance | [Decisions](project-management/decisions/index.md) |
| Historical-to-current editorial disposition | [Editorial reconciliation](project-management/editorial-reconciliation.md) |
| Accepted historical outcomes | [Completed](archive-history/completed/), [release history](archive-history/RELEASE_HISTORY.md), [root release notes](../RELEASE_NOTES.md) |
| Current UI visual direction | [Visual and interaction review](product-ux/references/ui-screenshot-review.md), [visual reference status](product-ux/references/reference-mockups.md) |

## Physical documentation sections

These are the twelve physical first-level sections under `docs/`. Topic subdirectories provide a second level of grouping; individual articles form the third level. The Docs reader uses the same directory hierarchy.

| Directory | Scope |
| --- | --- |
| `getting-started/` | Onboarding and first-run orientation |
| `product-ux/` | Application behavior, navigation, visual design and accessibility |
| `learning-practice/` | Practice, pedagogy, courses and language exercises |
| `games/` | Game-specific rules and interaction contracts |
| `dictionary-data/` | Dictionary, schema, vocabulary and personal data |
| `architecture/` | System architecture, components and state boundaries |
| `engineering/` | Coding standards, authoring and contributor engineering guides |
| `testing-quality/` | Test requirements, acceptance criteria, E2E and evidence |
| `build-deployment/` | CI, asset preparation, packaging and deployment |
| `project-management/` | Active work, decisions and unresolved requirements |
| `archive-history/` | Frozen completed records, audits and release history |
| `issues/` | Issue reports and visual evidence |

The root `docs/README.md` is the documentation index, not an additional category. Moving an article changes its Docs reader URL; old article paths are not retained as compatibility aliases.

## Documentation rules

- The source tree takes precedence for describing implementation; an approved product decision may describe an intended behavior not yet implemented. Label these distinctly.
- Maintain a **single current** description for each area. Do not create a new `R###_*` document for every checkpoint or re-copy outdated test statistics into permanent specs.
- Historical decisions and completed outcomes may be retained for provenance, but are never silent compatibility obligations.
- Report observed Windows/browser results separately from authored Rust test code and Node source-contract checks.
- Avoid stale paths such as `crates/web/` and removed scripts such as `crates/zipeek-e2e/fixtures/ai-server.cjs`.
- Unresolved work belongs in [Tasks](project-management/tasks.md); finalized choices belong in authoritative specifications and [Decisions](project-management/decisions/index.md).

Future learning evidence and seven-day proficiency E2E requirements: [learning evidence](learning-practice/practice/learning-evidence.md) and [E2E contract](testing-quality/e2e/learning-evidence-e2e.md).

## Docs reader navigation

The in-app Docs reader uses the twelve physical documentation directories as its navigation sections. The classification in `crates/zipeek-web/src/pages/system/docs/model.rs` follows those directory paths. The curated sections are Getting started, Product & UX, Learning & Practice, Games, Dictionary & Data, Architecture, Engineering, Testing & Quality, Build & Deployment, Project Management, Archive & History, and Issues. Within each section, related articles appear under a topic. The Markdown path is the article URL identity, and the navigation resolves against the new physical directories.

Historical completed records are displayed under Archive & History and are not current product instructions. The retired pending-decision scratchpad is no longer published or indexed.

New defects and design requests are tracked in the [issue library](issues/README.md), with per-issue screenshots and acceptance checklists. [Open issues](issues/OPEN.md) provides the active list. The old [issue register](project-management/user-reported-issues.md) redirects to the canonical library.

When docs Markdown changes, regenerate the `crates/zipeek-web/static/docs/index.json` using the project's docs publishing command before shipping. Keep that generated manifest consistent with Markdown paths and titles.

## Visual evidence

The Issues section publishes reports from `docs/issues/` and images from `docs/issues/assets/screenshots/`. Keep the corresponding static images under `crates/zipeek-web/static/docs/issues/assets/screenshots/` in sync so screenshot links resolve in the deployed Docs reader. Issue IDs remain stable when an issue changes status.

## Modular reference pages

These topic pages contain the original approved specification text relocated from the broad overview documents. Original section headings remain as links to preserve entry points. The underlying contracts are unchanged.

| Domain | Topic pages |
| --- | --- |
| Practice presentation | [Setup](product-ux/practice/ux-practice-setup.md), [Matching](product-ux/practice/ux-match.md), [Multiple choice](product-ux/practice/ux-mcq.md), [Summary](product-ux/practice/ux-summary.md) |
| Dictionary and accessibility | [Browse](product-ux/pages/ux-browse.md), [Accessibility](product-ux/foundation/ux-accessibility.md) |
| Appearance | [Theme and dark mode](product-ux/foundation/ux-appearance.md) |
| Practice domain behavior | [Current Practice overview](learning-practice/practice/practice-overview.md) |
| UI state architecture | [State ownership](architecture/ui-state-ownership.md), [Browse state](architecture/ui-browse-state.md) |
| Rust/WASM browser interoperability | [Browser boundary and thin wrappers](architecture/modules/rust-wasm-interop.md) |

Use `cargo make docs-validate` (Rust asset preflight) to check source links, image references, navigation index freshness and prohibited typography before packaging.

## Current approved product requirements

The following are authoritative **target-behavior** specifications. They do not imply successful builds, implemented features, or completed browser acceptance. Historical narrative in older documents remains available for provenance, but must not override these current requirements.

| Responsibility | Authoritative specification |
| --- | --- |
| Shared exercise frame and typed outcomes | [Practice exercise foundation](learning-practice/practice/shared-exercise-foundation.md) |
| Header and long dragon progress | [Header and dragon](product-ux/foundation/header-and-dragon.md) |
| Nonblocking vocabulary count | [Eligibility counting](dictionary-data/vocabulary/eligibility-counting.md) |
| Credits and acknowledgements | [Credits](product-ux/credits.md) |
| E2E evidence review workflow | [Developer and Codex review](testing-quality/e2e/review-workflow.md) |
| Practice verdicts, summary and Skip and Replace | [Practice outcomes](learning-practice/practice/practice-outcomes.md) |
| Pedagogical selection, visitation and global exclusions | [Vocabulary selection](dictionary-data/vocabulary/vocabulary-selection.md) |
| Portable saved Practice profiles | [Profile JSON portability](learning-practice/practice/practice-profile-portability.md) |
| Unified vision recognition canvas and privacy | [AI Hanzi recognition](learning-practice/hanzi/ai-hanzi-recognition.md) |
| Copy, theme, startup recovery, SEO and engineering policy | [Interface and platform policy](product-ux/foundation/interface-and-platform-policy.md) |
| Shared local visitation architecture | [Visitation boundaries](dictionary-data/vocabulary/vocabulary-visitation.md) |
| Sentence Relay stages, interaction and contextual correction | [Sentence Relay](games/sentence-relay.md) |
| Application screenshot-driven UI requirements | [Visual and interaction review](product-ux/references/ui-screenshot-review.md) |
| Testable requirements | [Acceptance criteria](testing-quality/acceptance/approved-behavior-acceptance.md) |

Unresolved decisions and implementation gaps remain in [tasks](project-management/tasks.md) and [issues](issues/OPEN.md). Historical release records are not current product requirements.

The [E2E requirements-to-test matrix](testing-quality/e2e/e2e-requirements-matrix.md) distinguishes authored browser scenarios from executed, verified evidence.

## Editorial ownership and filenames

Maintained article filenames use lowercase kebab-case, except conventional `README.md` entry points. Article titles should identify a specific responsibility, and similarly named files must have distinct ownership. Topic subfolders are allowed; do not introduce deeper nesting without a demonstrable navigation need. Historical and issue filenames retain identifiers when necessary for provenance. Avoid release identifiers in maintained titles and body text.

Overlapping contracts: the current [UX overview](product-ux/ux.md) and [Practice overview](learning-practice/practice/practice-overview.md) are navigation entry points, not duplicate authorities. E2E [recording workflow](testing-quality/e2e/e2e-recording.md) is operational guidance; the [video evidence contract](testing-quality/e2e/e2e-video-recording.md) governs recording metadata and validity. Historical implementation chronicles are stored in archive-history/audits.

For ongoing editorial and implementation verification gaps, see [documentation quality](project-management/documentation-quality.md).

- [Native Candle streaming ASR implementation status](architecture/native-streaming-asr.md)
