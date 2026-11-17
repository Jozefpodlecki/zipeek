# Multiple Choice interaction

## Question variants

Multiple Choice supports Hanzi to meaning, meaning to Hanzi and meaning to Pinyin, selected approximately evenly across enabled formats during a session. Word selection follows the approved vocabulary eligibility, visitation and exclusion policies. The same format should not repeat unnecessarily when alternatives are eligible. The available number of options and distractor semantics must be validated against the real question bank rather than treated as universal across every exercise host.

## Pinyin and pronunciation

**Show pinyin: Always** is the approved default, as long as it does not expose an answer before submission. When pinyin is the answer itself (for example, meaning to Pinyin), conceal the correct reading in unsubmitted context; display it after submission or only where the answer choices themselves are the challenge. Do not interpret `Always` as permission to leak the correct choice.

**Automatic question audio: On** is the approved default whenever the question presents a Chinese pronunciation target. A manual replay control remains available regardless of the autoplay setting for an available Chinese target, including after submission. Audio is not offered for an English-only prompt with no Chinese target to pronounce before the answer. Respect browser autoplay policy, availability/error states and saved learner preferences. The UI must remain usable if automatic playback is blocked.

## Selection and feedback

Each option has an obvious focus, selected, correct and incorrect state with accessible text/icon equivalents. The learner must know exactly which answer was selected. Explain correct/incorrect feedback where supported by the underlying dictionary or validator. Prevent repeated taps from duplicate-scoring one submission. Use the shared Practice exercise frame for progress, feedback and actions, without a redundant exercise-name header or second exit control.

Options must remain accessible by pointer, touch and keyboard, with appropriately announced feedback. Consistent segmented and radio controls belong in Configure, not ad hoc layouts unique to MCQ.

## Acceptance

Browser tests must cover all enabled variants, the balanced format sequence, default Pinyin and audio settings, user overrides and persistence, audio availability/autoplay failure, manual replay, correct and incorrect answer states, no answer leak, and both desktop and narrow layouts. See [shared exercise foundation](../../learning-practice/practice/shared-exercise-foundation.md) and [E2E matrix](../../testing-quality/e2e/e2e-requirements-matrix.md).

Historical MCQ implementation assumptions are preserved in [the archived source](../../archive-history/audits/superseded-maintained/product-ux--practice--ux-mcq-source.md); they are not current behavior contracts.
