# Documentation

Start with [AGENTS.md](../AGENTS.md) for the contributor workflow,
[TASKS](TASKS.md) for open work, and [CURRENT](CURRENT.md) for live state.

The bootstrapped planning state is in [TASKS](TASKS.md), [CURRENT](CURRENT.md),
and [COMPLETED](COMPLETED.md); [UX](UX.md) collects user-facing requirements.
These three task-state files supersede the deleted STATUS.md. Its unique history
was migrated once to COMPLETED; its open work is represented in TASKS. Do not
recreate STATUS or maintain a parallel state log.

| Document | Purpose |
| --- | --- |
| [TASKS](TASKS.md) | Ordered backlog of open work and prerequisites |
| [CURRENT](CURRENT.md) | Active work, blockers, and next ready task |
| [COMPLETED](COMPLETED.md) | Append-only outcomes and migrated documentation history |
| [BASELINE](BASELINE.md) | A00 diagnostic evidence before implementation repairs |
| [UX](UX.md) | User-facing spec and explicit unresolved UX items |
| [PRODUCT](PRODUCT.md) | Intended v1 experience and routes |
| [ROUND_GENERATION](ROUND_GENERATION.md) | D12 candidate bundle, canonical seeds, portable RNG, and duplicate handling |
| [ARCHITECTURE](ARCHITECTURE.md) | Current code, planned structure, and data flow |
| [DB](DB.md) | Database requirements, licensing, schema, and generation |
| [TESTING](TESTING.md) | Validation commands, prerequisites, strategy, and CI gaps |
| [CI](CI.md) | Workflow structure, required gates, caching, and Pages deployment |
| [CODING_STANDARDS](CODING_STANDARDS.md) | Rust/Yew conventions, error handling, tests, and commit/PR format |
| [DECISIONS](DECISIONS.md) | Approved decisions and remaining open choices |

Planned behaviour is labelled explicitly. Proposed defaults remain unapproved
until the user resolves the corresponding decision.
