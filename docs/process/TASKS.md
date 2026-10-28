# Open tasks

Bootstrapped on 2026-10-26. This is an ordered backlog of open work only; no task
below is implemented by the existence of this file. The user approved the action
plan on 2026-10-26.

A00/A01/A02/A03/A04/A05/A06/A07/A08/A14/A15/A16/A17/A18/A19/A20/A21/A22/A23/A24/A25/A26/A27/A28/A29/A30/A31/A35/A37/A38/A42
are logged in [completed](completed/).
A34 records the documentation response to the 2026-10-27 architecture review.
A36 records the user's Tailwind correction; A35 completed its implementation
on 2026-10-28 and is recorded in completed outcomes.
A15 is complete under D09/D28, A35 under D31 and A31 implements approved D29
theme/database state boundaries. A21 now implements six-route practice
preparation/loading and routing recovery. A16 implements CI/Pages configuration
with local gates passing; hosted execution remains unverified. A32/A33 remain
implementation proposals with
separate scope and dependency/feature approval requirements. A22's approved answered-UX documentation pass is
complete; A37 now records the remaining product/UX/initial-motion decisions and
approved D34 share-card design. A40 owns its implementation before v1; selective
A38 completes selective cleanup with six deferred/partial/unresolved sections
retained. A41 tracks the remaining contributor-policy questions. The user approved implementation order A15 → A35 → A31 → A21, to
start after A22's outcome was appended. A32/A33 retain separate scope/decision requirements.

Ready means no unfinished prerequisites; specific dependency/schema approvals
are still required where the guide calls for them.

## Source and reconciliation

The on-disk [AGENTS.md §3](../../AGENTS.md#3-repository-map) is a repository map,
not a Current state / NOT implemented table. This backlog uses that map,
[ARCHITECTURE](../architecture/ARCHITECTURE.md)'s current snapshot, the resolved
decisions, and the user's proposed A01-A16 IDs. STATUS.md was migrated into
[completed/](completed/) during A18 and then deleted; its legacy TODOs are fully
represented below and its 15 historical outcomes are preserved in
[completed/](completed/). Additional IDs preserve missing foundation/reconciliation
work instead of silently discarding it.

MCQ is settled: three sub-variants (`hanzi-to-meaning`, `meaning-to-hanzi`,
`meaning-to-pinyin`) under `/practice/mcq/<variant>`, Endless and Fixed modes
with rounds 5/10/20, three options and distractor rules mirroring D02/D12 with
N=2, offset advancing by 1 per question. A10 implements this scope; A22 recorded
MCQ feedback, number-key hints and adaptive styling. A37 records the resolved UX choices; A22 did not change MCQ scope or implement controls.

D02-D18 and D20 are the user-declared resolved specification baseline for this
bootstrap. [Decisions](decisions/index.md) records full D18 approval, resolved
D19 (v1 only), and D20. A23 documented the approved stream/call order; A07
implements the pure builder and native/wasm full-round vectors. Playable MCQ UI
remains A10 work. A24 documents dependency/fallback-source proposals; D21/D22/D23
are approved; A25 applied those choices with native v2.0.0 and verified v1.4.0
storage. A06 now verifies the final dictionary in Node and actual Chrome workers,
including the `/zipeek/` build. The user explicitly prioritized A06 over A15 and
approved using the existing external-server Chrome setup. A22 records the
answered UX requirements. A37 resolves the remaining design choices for feature work; UX-SUM01 application is deferred to A09/A10.

TASKS, [CURRENT](CURRENT.md), and [completed/](completed/) are the only live
task-state sources. A18's one-time STATUS reconciliation is done: all nine legacy
TODOs are covered here, all 15 historical outcomes are preserved in
[completed/](completed/), and STATUS.md was deleted. The guide and documentation
now point at the three state docs. Do not recreate STATUS or maintain parallel
logs. A18's approved reconciliation scope is closed with recorded gate failures;
D21/D22 are now accepted and A25 completed the generator foundation after the A24
audit.

## Ordered backlog

User-directed quiz-first priority, 2026-10-28: A11 → A10 → A09. The first playable
milestone is setup → MCQ question → answer feedback → Continue in Endless mode.
A21 already prepares real deterministic rounds; these tasks replace the remaining
setup/gameplay placeholders. Complete each task's approved scope and validation;
the early milestone does not mark full A10 complete or remove Fixed requirements.

Metadata, animation and database extraction do not block answering quiz questions.
Chart.js pin/summary application remains required for the later Fixed-summary work,
not the first Endless question loop. A40 remains required before v1. Existing task
prerequisites, dependency approvals, replay/UX requirements and validation gates
remain unchanged.

| ID | Title | Goal (one line) | Prerequisites (task IDs) | Status | Ready to start |
| --- | --- | --- | --- | --- | --- |
| A11 | Practice setup screen | Implement HSK context, six-variant activity selection, Endless/Fixed modes, fixed counts, and exclusion reset using approved UX. | A08, A21, A37 | in progress | Yes |
| A10 | MCQ exercise UI | Implement gameplay on A21's declared MCQ routes: Endless/Fixed, three options, N=2 distractors, and offset +1 using UX-MCQ01; review the deferred summary in implementation. | A06, A07, A11, A15, A37, A23 | todo | No |
| A09 | Match exercise UI and Fixed summary | Deliver all three four-pair variants, approved feedback, progression, and in-memory Fixed summary with smoke coverage; apply approved first-attempt metric and confirm exact Chart.js pin during approved implementation. | A07, A11, A15, A37 | todo | No |
| A13 | Browse view | Implement approved query controls, numeric/display-pinyin lookup, result presentation, and pagination with smoke coverage. | A06, A21, A37 | todo | Yes |
| A12 | Landing tiles, Random, and Word of the Day | Connect the A27 landing tiles/card to actual queries, Random practice, deterministic daily selection and empty/failure states. | A06, A09, A13, A21, A37 | todo | No |
| A40 | Static social metadata and preview image | Implement D34 approved share title/copy, site-wide 1200×630 landscape/slate/wordmark/blue image and initial HTML Open Graph/Twitter large-image metadata with /zipeek/ URLs before v1. | A37 | todo | Yes |
| A32 | Animato and route transitions | Implement D29 reduced-motion policy, local frame adapter and interruptible outlet lifecycle using the approved 150 ms fade-only profile; no additional features implicitly approved. | A31, A37, A35 | todo | Yes |
| A33 | Extract browser database adapter | Proposed D30: move transport/typed decoding/private SQL to `zipeek-db-wasm` after approving facade, native target strategy and exact dependencies; retain UI providers/assets in web. | A06, A15 | todo | No |
| A41 | Clarify contributor policy questions | Resolve the unreachable-branch versus sole lint-rationale comment exception and whether D27 needs an additional explicit cargo-update prohibition; no policy change is assumed. | None | awaiting answers | Yes, decision review only |

## Completion and state updates

For approved implementation work, run the actual gates in
[AGENTS.md §6](../../AGENTS.md#6-commands-and-completion) and
[testing/TESTING.md](../testing/TESTING.md); the requested sections 8/9 do not
exist in the current guide. UI behaviour also needs Chrome smoke coverage.
Unavailable or failing gates are reported, not passed. New dependency/version/
schema changes still require the specific approval defined in AGENTS sections
4/5; D17's rand chacha feature choice already has approval.

Keep active work in [CURRENT](CURRENT.md), append finished outcomes to
[completed/](completed/), and remove finished tasks from this open-only backlog
after logging them. Prerequisites may refer to completed IDs in
[completed/](completed/). Update affected UX/architecture/DB/testing docs.
**Never recreate or append to STATUS.**
