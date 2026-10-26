# Open tasks

Bootstrapped on 2026-10-26. This is an ordered backlog of open work only; no task
below is implemented by the existence of this file. The user approved the action
plan on 2026-10-26. A00/A01/A02/A17/A18/A20/A23/A24/A25/A26 are logged in COMPLETED; A03 is active.
A22/A19/A03/A07/A08 have no unfinished task prerequisites; their recorded
approval or decision requirements still apply.
Ready means no unfinished prerequisites; specific dependency/schema approvals
are still required where the guide calls for them.

## Source and reconciliation

The on-disk AGENTS.md section 3 is a repository map, not a Current state / NOT
implemented table. This backlog uses that map, ARCHITECTURE's current snapshot,
the resolved decisions, and the user's proposed A01-A16 IDs. STATUS.md was
migrated into COMPLETED.md during A18 and then deleted; its legacy TODOs are
fully represented below and its 15 historical outcomes are preserved in
COMPLETED.md.
Additional IDs preserve missing foundation/reconciliation work instead of silently
discarding it. MCQ is settled: three sub-variants (hanzi-to-meaning,
meaning-to-hanzi, meaning-to-pinyin) under `/practice/mcq/<variant>`, Endless and
Fixed modes with rounds 5/10/20, three options and distractor rules mirroring
D02/D12 with N=2,
offset advancing by 1 per question. A10 implements this scope; A22 resolves
remaining MCQ UI detail (feedback timing, number-key hints, button styling) and
other UX TBDs, not MCQ scope.

D02-D18 and D20 are the user-declared resolved specification baseline for this
bootstrap. DECISIONS records full D18 approval, resolved D19 (v1 only), and D20.
A23 documented the approved stream/call order; full-round golden vectors remain
A07/A10 implementation work. A24 documents dependency/fallback-source proposals;
D21/D22/D23 are approved; A25 applied those choices with native v2.0.0 and
verified v1.4.0 storage. Full-dictionary/browser compatibility remains A06 work.
A22 resolves the explicit UX TBDs before dependent
UI work.

TASKS, CURRENT, and COMPLETED are the only live task-state sources. A18's one-time
STATUS reconciliation is done: all nine legacy TODOs are covered here, all 15
historical outcomes are preserved in COMPLETED, and STATUS.md was deleted.
The guide and documentation now point at the three state docs. Do not recreate
STATUS or maintain parallel logs. A18's approved reconciliation
scope is closed with recorded gate failures; D21/D22 are now accepted and
A25 completed the generator foundation after the A24 audit.

## Ordered backlog

| ID | Title | Goal (one line) | Prerequisites (task IDs) | Status | Ready to start |
| --- | --- | --- | --- | --- | --- |
| A22 | Resolve UX open items | Resolve UX.md UI-detail TBDs within approved D20 MCQ scope, without inventing interactions, copy, or layouts. | None | todo | Yes |
| A19 | Align pinned toolchain and build configuration | Verify already-aligned local Rust/npm/wasm pins and record remaining policy/config gaps; workflow migration belongs to A16. | A00 | todo | Yes |
| A03 | Import the pinned HSK source | Import complete.json using D16/D18 grouping, levels, exact source text, frequency, rejection logs, and snapshot-local IDs. | A02, A18, A24, A25 | doing | No |
| A04 | CC-CEDICT gloss fallback | Use a recorded fallback snapshot only for empty primary meanings, skipping variant-of entries and preserving notices. | A03, A18, A24, A25 | todo | No |
| A05 | Generate the read-only DuckDB artifact | Manually generate the words table and two ordinary HSK indexes, recording output counts, size, provenance, and notices. | A04 | todo | No |
| A07 | Pure deterministic round builder | Implement build_round and CandidatePool using D12/D17 canonical inputs and portable rand ChaCha with fixture/golden tests. | A02, A18, A23 | todo | Yes |
| A08 | Practice URL parsing and state normalization | Implement D13/D15 filters, exclusions, defaults, seed fallbacks/recovery, and canonical round inputs. | A02, A18, A23 | todo | Yes |
| A15 | Chrome e2e lifecycle harness | Rename the existing workspace e2e crate and implement managed Chrome plus shared Trunk readiness and teardown helpers. | A19, A20, A18 | todo | No |
| A06 | DuckDB-WASM query layer | Integrate the exact wasm pin and test read-only compatibility with the native v2.0.0 artifact with v1.4.0 storage; add typed queries and observable readiness/failure state. | A02, A05, A15, A20 | todo | No |
| A14 | Theme, left sidebar, and footer | Implement saved dark/light choice with dark default, repository icon placement, copyright, and data attribution. | A20, A15, A22 | todo | No |
| A21 | Routing and loading-shell integration | Extend existing match/browse route scaffolds and connect shared layout with continuous static-to-Yew loading through database readiness and retry. | A06, A08, A14, A15 | todo | No |
| A11 | Practice setup screen | Implement HSK context, match-variant selection, Endless/Fixed modes, fixed counts, and exclusion reset using approved UX. | A08, A21, A22 | todo | No |
| A09 | Match exercise UI and Fixed summary | Deliver all three four-pair variants, approved feedback, progression, and in-memory Fixed summary with smoke coverage. | A07, A11, A15, A22 | todo | No |
| A10 | MCQ exercise UI | Implement D20 three-variant MCQ routes, Endless/Fixed, three options, N=2 distractors, and offset +1 after A22 resolves UI details. | A06, A07, A11, A15, A22, A23 | todo | No |
| A13 | Browse view | Implement approved query controls, numeric/display-pinyin lookup, result presentation, and pagination with smoke coverage. | A06, A21, A22 | todo | No |
| A12 | Landing tiles, Random, and Word of the Day | Deliver the three navigation tiles, Random shortcut, deterministic daily card, and empty/failure states. | A06, A09, A13, A21, A22 | todo | No |
| A16 | CI and Pages deployment | Update web-path workflows, required quality/Chrome gates, and Pages artifact deployment under /zipeek/. | A15, A19, A21 | todo | No |

## Completion and state updates

For approved implementation work, run the actual gates in AGENTS.md section 6
and TESTING.md; the requested sections 8/9 do not exist in the current guide.
UI behaviour also needs Chrome smoke coverage. Unavailable or failing gates are
reported, not passed. New dependency/version/schema changes still require the
specific approval defined in AGENTS sections 4/5; D17's rand chacha feature choice
already has approval.

Keep active work in CURRENT, append finished outcomes to COMPLETED, and remove
finished tasks from this open-only backlog after logging them. Prerequisites may
refer to completed IDs in COMPLETED. Update affected UX/architecture/DB/testing
docs. **Never recreate or append to STATUS.**
