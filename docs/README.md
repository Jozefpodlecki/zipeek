# Documentation

Start with [AGENTS.md](../AGENTS.md) for the contributor workflow,
[TASKS](process/TASKS.md) for open work, and [CURRENT](process/CURRENT.md) for
live state.

The bootstrapped planning state is in [TASKS](process/TASKS.md),
[CURRENT](process/CURRENT.md), and [completed/](process/completed/);
[UX](product/UX.md) collects user-facing requirements. These three task-state
sources supersede the deleted STATUS.md. Its unique history was migrated once to
[completed/archive.md](process/completed/archive.md); its open work is
represented in [TASKS](process/TASKS.md). Do not recreate STATUS or maintain a
parallel state log.

## Where to look

| If you want to know… | Read |
| --- | --- |
| What v1 does and which routes exist | [product/PRODUCT.md](product/PRODUCT.md) |
| What the user sees and open UX items | [product/UX.md](product/UX.md) |
| Future AI tags, Settings, speaking/sentence exercises and ambiguous prompts | [product/FUTURE.md](product/FUTURE.md) |
| How rounds are built and replayed | [product/ROUND_GENERATION.md](product/ROUND_GENERATION.md) |
| What code exists and how modules fit | [architecture/ARCHITECTURE.md](architecture/ARCHITECTURE.md) |
| Which generator module owns what | [architecture/modules/db-gen.md](architecture/modules/db-gen.md) |
| What shared types and pure logic live where | [architecture/modules/zipeek-core.md](architecture/modules/zipeek-core.md) |
| How the browser queries the database | [architecture/modules/browser-db.md](architecture/modules/browser-db.md) |
| How the shell, loading, and backdrop work | [architecture/modules/web-shell.md](architecture/modules/web-shell.md) |
| How data moves from source to browser | [architecture/modules/data-flow.md](architecture/modules/data-flow.md) |
| The target crate graph and A33 proposal | [architecture/modules/target-architecture.md](architecture/modules/target-architecture.md) |
| Implemented scoped state and proposed motion | [architecture/UI_ARCHITECTURE.md](architecture/UI_ARCHITECTURE.md) |
| Sources, licensing, and the pinned snapshot | [architecture/schema/sources.md](architecture/schema/sources.md) |
| The `words` table and indexes | [architecture/schema/schema.md](architecture/schema/schema.md) |
| Requested schema/index/FTS review and benchmark criteria | [architecture/schema/REVIEW.md](architecture/schema/REVIEW.md) |
| How source rows become `words` rows | [architecture/schema/import-rules.md](architecture/schema/import-rules.md) |
| The 42-code PoS mapping | [architecture/schema/pos-mapping.md](architecture/schema/pos-mapping.md) |
| Word of the Day and practice SQL contracts | [architecture/schema/queries.md](architecture/schema/queries.md) |
| The DuckDB-WASM pin and engine facts | [architecture/schema/runtime.md](architecture/schema/runtime.md) |
| Generator dependency pins and neo restoration | [architecture/schema/generator-deps.md](architecture/schema/generator-deps.md) |
| What "reproducible" means for artifacts | [architecture/schema/reproducibility.md](architecture/schema/reproducibility.md) |
| What each generator stage actually does | [architecture/schema/artifacts.md](architecture/schema/artifacts.md) |
| How to regenerate the dictionary | [architecture/schema/regeneration.md](architecture/schema/regeneration.md) |
| Every approved decision D01–D40 | [process/decisions/index.md](process/decisions/index.md) |
| Testing strategy and commands | [testing/TESTING.md](testing/TESTING.md) |
| Chrome e2e and the A15 split | [testing/e2e.md](testing/e2e.md) |
| Matching and mixed-session checks | [testing/matching.md](testing/matching.md) |
| Browser DB checks and the wasm replay runner | [testing/browser-db.md](testing/browser-db.md) |
| CI structure and required gates | [process/CI.md](process/CI.md) |
| Rust/Yew conventions | [process/CODING_STANDARDS.md](process/CODING_STANDARDS.md) |
| Historical build evidence | [process/BASELINE.md](process/BASELINE.md) |

## Directory layout

```text
docs/
  README.md                     (this file)

  product/
    PRODUCT.md                  v1 behaviour and routes
    UX.md                       user-facing spec and TBDs
    ROUND_GENERATION.md         D12/D19 algorithm and replay contract

  architecture/
    ARCHITECTURE.md             overview and module map
    UI_ARCHITECTURE.md          scoped state and motion proposal
    modules/                    per-module implementation notes
      db-gen.md
      zipeek-core.md
      browser-db.md
      web-shell.md
      data-flow.md
      target-architecture.md
    schema/                     database documentation
      DB.md                     index
      sources.md
      schema.md
      import-rules.md
      pos-mapping.md
      queries.md
      runtime.md
      generator-deps.md
      reproducibility.md
      artifacts.md
      regeneration.md

  testing/
    TESTING.md                  index and strategy
    e2e.md
    browser-db.md
    word-of-the-day.md
    practice-url.md
    round-builder.md
    mcq.md
    startup.md
    shell.md
    provenance.md
    ci-targets.md
    evidence.md

  process/
    BASELINE.md                 A00 diagnostic evidence (historical)
    CI.md                       workflow structure and deployment
    CODING_STANDARDS.md         Rust/Yew conventions
    TASKS.md                    open backlog
    CURRENT.md                  live state
    handoff.md                  per-task continuation notes
    completed/                  append-only outcomes by date
      README.md
      2026-10-26.md
      2026-10-27.md
      archive.md
      validation-log.md
    decisions/                  approved and proposed decisions
      index.md
      foundations.md
      rounds-and-replay.md
      database-and-artifacts.md
      web-and-ui.md
      process-and-infra.md
```

Conventions
Planned behaviour is labelled explicitly. Proposed defaults remain
unapproved until the user resolves the corresponding decision. Do not present
intended routes, tools, tests, or database integration as if they exist.

Task state lives in exactly three places: TASKS (open
work), CURRENT (live state), and
completed/ (append-only outcomes). Never recreate STATUS.

Decisions are grouped, not scattered. Each DNN has one home file under
process/decisions/ and one index row in
index.md. New decisions append to the group file
for their family and add an index row.

The schema split mirrors the approved decisions.
architecture/schema/DB.md is the index; each
sub-file is the section of the original database document it now owns.

No-comments rule for source code, with a single exception for
#[allow(...)] rationales — see
CODING_STANDARDS.

Do not rewrite history. Completed outcomes and archived entries are
preserved verbatim; only relative link paths were adjusted when files moved.
