# Archived source snapshot: product-ux/practice/ux-mcq.md

Archived observations may be superseded by current approved requirements.

# Ux Mcq

## MCQ exercise

D20 approves three variants at `/practice/mcq/<variant>`: `hanzi-to-meaning`,
`meaning-to-hanzi`, and `meaning-to-pinyin`. Use simplified hanzi, the first
English gloss, and display pinyin for the corresponding prompt/answer labels.
Each question offers three options: one correct answer and two distractors
(N=2). Apply D02/D12 pool widening and distinct visible-answer rules.

Both Endless and Fixed are in scope. Fixed offers 5/10/20 questions, default
10. Advance offset by one per question; keep Fixed progress in memory separately
from offset wrapping. Routes and both gameplay modes, progress and the MCQ
summary are implemented by A10.

### MCQ interaction (UX-MCQ01)

Feedback is immediate on the first selection. A correct selected answer is
green with ✓; a wrong selected answer is red with ✗. The other two options
are dimmed grey. All options lock after that click: no retry or switching.
One wrong selection is one mistake in Fixed (0..N); Endless has no mistake
total. A wrong answer resets the Endless streak.

Continue occupies a persistent bottom action bar, always visible and dimmed
until a selection, then enabled to advance. Each option shows a `1`, `2` or `3`
badge; those keys select the corresponding option. Tab and Enter/Space remain
supported. No screen-reader practice feedback announcements are required.

The prompt is large and centred. Hanzi prompts have the pronunciation control.
Answers stack full-width on narrow screens. On wide screens, use a horizontal
row when all three are short, otherwise stack; exact text-length thresholds
are implementation choices. Fixed has a horizontal `completed / total`
progress bar above the prompt without a visible "Question 3 of 10" label.
Endless has no progress bar, only streak and cumulative-correct header counts.
Reduced motion keeps colour changes instantaneous and removes slides/fades.

The first A10 milestone implements these interactions for all three Endless
variants. Options form a wide-screen row only when every primary label has at
most 24 Unicode characters; otherwise they stack. New questions focus the prompt.
Listen remains independent of answer selection, including after answers lock.
The subsequent A10 extension implements Fixed progress and the summary below.
