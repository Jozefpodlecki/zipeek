# Open tasks

Bootstrapped on 2026-10-26. This is an ordered backlog of open work only; no task
below is implemented by the existence of this file. The user approved the action
plan on 2026-10-26.

A00/A01/A02/A03/A04/A05/A06/A07/A08/A14/A17/A18/A19/A20/A23/A24/A25/A26/A27/A28/A29/A30
are logged in [completed](completed/).
A34 records the documentation response to the 2026-10-27 architecture review.
A36 records the user's Tailwind correction; A35 is the open migration task.
A31/A32/A33 below are implementation proposals, not completed refactors or
approved dependency changes. A22/A15 have no unfinished task prerequisites; their
recorded approval or decision requirements still apply.

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
N=2, offset advancing by 1 per question. A10 implements this scope; A22 resolves
remaining MCQ UI detail (feedback timing, number-key hints, button styling) and
other UX TBDs, not MCQ scope.

D02-D18 and D20 are the user-declared resolved specification baseline for this
bootstrap. [Decisions](decisions/index.md) records full D18 approval, resolved
D19 (v1 only), and D20. A23 documented the approved stream/call order; A07
implements the pure builder and native/wasm full-round vectors. Playable MCQ UI
remains A10 work. A24 documents dependency/fallback-source proposals; D21/D22/D23
are approved; A25 applied those choices with native v2.0.0 and verified v1.4.0
storage. A06 now verifies the final dictionary in Node and actual Chrome workers,
including the `/zipeek/` build. The user explicitly prioritized A06 over A15 and
approved using the existing external-server Chrome setup. A22 resolves the
explicit UX TBDs before dependent UI work.

TASKS, [CURRENT](CURRENT.md), and [completed/](completed/) are the only live
task-state sources. A18's one-time STATUS reconciliation is done: all nine legacy
TODOs are covered here, all 15 historical outcomes are preserved in
[completed/](completed/), and STATUS.md was deleted. The guide and documentation
now point at the three state docs. Do not recreate STATUS or maintain parallel
logs. A18's approved reconciliation scope is closed with recorded gate failures;
D21/D22 are now accepted and A25 completed the generator foundation after the A24
audit.

## Ordered backlog

| ID | Title | Goal (one line) | Prerequisites (task IDs) | Status | Ready to start |
| --- | --- | --- | --- | --- | --- |
| A22 | Resolve UX open items | Resolve the 11 remaining UX TBDs and review proposed motion profiles; established shell/landing choices remain settled. | None | todo | Yes |
| A15 | Modular Chrome e2e infrastructure | Split the smoke dumping ground into harness, suites, page helpers and browser fixtures; implement D09 owned server/session/readiness/cleanup and package rename using [testing/e2e.md](../testing/e2e.md)'s proposal. | A19, A20, A18 | todo | Yes |
| A35 | Restore Tailwind-first component styling | Correct bespoke shell/card CSS drift using Tailwind 4 utilities and shared tokens; preserve current visuals/branding, accessibility and bootstrap exceptions with Chrome comparison coverage. | A15 | todo | No |
| A31 | Scoped UI providers and reducers | Proposed D29: separate stable service access/readiness, typed feature commands and pure reducers; prevent prop drilling and stale async updates per [UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md). | A06, A15 | todo | No |
| A32 | Animato and route transitions | Proposed D29: implement reduced-motion policy, local frame adapter and interruptible outlet lifecycle using approved motion profiles; no additional features implicitly approved. | A31, A22, A35 | todo | No |
| A33 | Extract browser database adapter | Proposed D30: move transport/typed decoding/private SQL to `zipeek-db-wasm` after approving facade, native target strategy and exact dependencies; retain UI providers/assets in web. | A06, A15 | todo | No |
| A21 | Routing and loading-shell integration | Reuse A06 readiness, A31 state boundaries and corrected Tailwind styling; finish routes and per-route query/loading/failure wiring. | A06, A08, A14, A15, A31, A35 | todo | No |
| A11 | Practice setup screen | Implement HSK context, match-variant selection, Endless/Fixed modes, fixed counts, and exclusion reset using approved UX. | A08, A21, A22 | todo | No |
| A09 | Match exercise UI and Fixed summary | Deliver all three four-pair variants, approved feedback, progression, and in-memory Fixed summary with smoke coverage. | A07, A11, A15, A22 | todo | No |
| A10 | MCQ exercise UI | Implement D20 three-variant MCQ routes, Endless/Fixed, three options, N=2 distractors, and offset +1 after A22 resolves UI details. | A06, A07, A11, A15, A22, A23 | todo | No |
| A13 | Browse view | Implement approved query controls, numeric/display-pinyin lookup, result presentation, and pagination with smoke coverage. | A06, A21, A22 | todo | No |
| A12 | Landing tiles, Random, and Word of the Day | Connect the A27 landing tiles/card to actual queries, Random practice, deterministic daily selection and empty/failure states. | A06, A09, A13, A21, A22 | todo | No |
| A16 | CI and Pages deployment | Update web-path workflows, locked npm assets, required gates and `/zipeek/` deployment; review pre-existing Picomatch tooling audit before any separately approved dependency fix. | A15, A19, A21 | todo | No |

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