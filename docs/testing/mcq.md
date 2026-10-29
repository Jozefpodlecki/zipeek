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

Fixed summary application is implemented under D35/A10 and reused by A09.
A22 itself added no summary behavior assertions or Chart.js pin. A48 supplies
shared-session progress/stats across matching and MCQ; see [mixed checks](matching.md).

## Current status

D20 is approved. A07 provides pure full-round MCQ data for all three variants
via the same round builder. A21 declares the three routes and prepares their
rounds from actual queries. The first A10 milestone implements the three Endless
variants and their session progression; the subsequent extension implements
Fixed gameplay and summary for 5/10/20 questions.

## A37 traditional display requirement

Hanzi-labeled prompts/answers show traditional as secondary text. Primary
simplified text and answer-label uniqueness/comparison remain unchanged. Do not
add hanzi to English/pinyin-only surfaces under this rule. Verify retained full
round replay as well as display when A10 implements the controls.

## Endless milestone coverage

Two native reducer tests check first-answer locking, invalid/stale selection,
Continue gating, expected-navigation counter retention and wrong-answer streak
reset. Two shared Chrome scenarios run under root and Pages with real dictionary
rounds: all three variants, three options, disabled Continue before selection,
immediate feedback/locked/dimmed options, number keys and editable-field exclusion,
Space/Enter, cumulative correct and streak/flame, canonical offset/exclusion URLs,
no Endless progress bar, reload reset, seeded Continue clearing seed, u64 overflow
wrapping without losing counters, and returning through setup. Speech API spies
check zh-CN hanzi calls; audible output is not asserted. Each variant has light/
dark screenshots at 400/768/1280px and overflow/Continue visibility assertions.

The adaptive wide-row threshold is 24 Unicode characters per primary label;
longer answer sets stay stacked. Hanzi traditional text is secondary only where
it differs from simplified. No core label/comparison or round RNG changes.
The subsequent Fixed extension below covers the remaining MCQ requirements.

## Fixed and summary coverage (A10 / D35)

Three additional native tests cover core FixedProgress completion independently
of wrapping, first-selection accounting, highest streak, final-click/frozen time,
duplicate completion, fresh restart offsets and whole-minute/second formatting.
Two additional shared Chrome scenarios run on root and Pages with real rounds:

- All three variants across 5/10/20, completed/total progress, last Continue
  reaching the summary without another question, and no early summary.
- Wrong/correct accounting, total-question denominator, mistakes, highest streak,
  a controlled wall clock including visibility changes, and time frozen after
  completion. The test clock is installed in the current document as well as
  future documents because hash navigation alone does not rerun init scripts.
- Fresh-offset replay preserving mode/count/filters/exclusions, counters reset,
  reload and navigate-away/back resetting progress, two fresh restart URLs using
  controlled entropy, and Back to setup restoring defaults.
- A real locally packaged canvas plus four DOM text stats, chart destruction
  after replay/setup, 300 ms normal and instant reduced-motion rendering, live
  preference changes, 400/768/1280px captures in both themes and no overflow.
- Blocked summary-module loading leaves all numeric text and buttons usable.
- A Worker fixture makes the strict view empty while retaining real widened
  candidate rows. Repeated offset-zero Continue completes five questions and
  same-offset replay resets progress, covering unchanged normalized URLs.

Build checks also verify npm ci --ignore-scripts and the local packaging hook,
exact chart.js 4.6.1/esbuild 0.28.2, retained Chart.js/color/esbuild licenses and
unchanged pre-existing npm records. No audio, schema or replay changes.
