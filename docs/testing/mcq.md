# MCQ checks

D20 MCQ coverage plan. Part of the [testing index](TESTING.md). Approved
requirement in [D20](../process/decisions/database-and-artifacts.md).

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

A22 records UX-MCQ01. Required UI assertions when A10 implements it:

- Immediate green/✓ for a correct selection or red/✗ for a wrong selection;
  remaining options dimmed grey. All options lock after the first selection.
- Continue remains visible/in place, disabled until selection, then advances.
- Visible 1/2/3 badges and matching number-key shortcuts; Tab/Enter/Space work.
- Large centred prompt; D32 Listen on hanzi/pinyin surfaces, verified by API spy.
- Full-width stacked answers on narrow screens; adaptive row/stack on wide
  screens based on label length. Implementation chooses exact text thresholds.
- Fixed-only completed/total progress bar without a visible question-number
  label; Endless shows streak/cumulative correct and no progress bar.
- One wrong selection counts as one Fixed mistake; no Endless mistake total.
  Wrong answers reset the in-session streak; flame appears at streak ≥ 3.
- Reduced motion removes slides/fades and retains instantaneous colour/icons.
  All controls have visible keyboard focus. No live practice announcements.

Fixed summary application remains later A09/A10 work. A22 adds no summary
behaviour assertions or Chart.js pin.

## Current status

D20 is approved. A07 provides pure full-round MCQ data for all three variants
via the same round builder. A21 declares the three routes and prepares their
rounds from actual queries; A10 owns playable controls and session progression. These checks describe required
coverage; they are not evidence that MCQ practice is implemented.

## A37 traditional display requirement

Hanzi-labeled prompts/answers show traditional as secondary text. Primary
simplified text and answer-label uniqueness/comparison remain unchanged. Do not
add hanzi to English/pinyin-only surfaces under this rule. Verify retained full
round replay as well as display when A10 implements the controls.
