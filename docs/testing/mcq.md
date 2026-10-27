# MCQ checks

D20 MCQ coverage plan. Part of the [testing index](TESTING.md). Approved
requirement in [D20](../process/decisions/database-and-artifacts.md#d20-mcq-practice).

## Coverage plan

Cover all three `/practice/mcq/<variant>` routes:

- `hanzi-to-meaning`
- `meaning-to-hanzi`
- `meaning-to-pinyin`

For each variant, verify:

- One correct option and two distinct-answer distractors (N=2).
- D02/D12 widening rules apply: HSK ±1 within the same system, then drop PoS,
  and visible-answer uniqueness holds.
- Exclusions are applied before selection and after each widening step.
- D18 eligibility: all three variants require English, so rows without English
  are ineligible.
- Endless and Fixed modes.
- Fixed counts exactly 5/10/20, default 10; no custom-N input.
- Offset advances by one per question, independently of Fixed progress.
- D19 single-stream draw order: prompt selection, two accepted distractor draws,
  then one shuffle of `[correct, distractor_1, distractor_2]`. No reseeding,
  cloned-stream replay, or caller-owned second RNG.

UI-detail assertions (feedback timing, number-key hints, button styling) wait
for [A22](../process/TASKS.md) to resolve UX-MCQ01.

## Current status

D20 is approved. A07 provides pure full-round MCQ data for all three variants
via the same round builder; A10 owns playable MCQ routes and UI, and those routes
are not yet declared in `web/src/route.rs`. These checks describe required
coverage; they are not evidence that MCQ practice is implemented.