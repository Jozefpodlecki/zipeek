# Ux Match

## Match exercise

Show four cards on the left and four on the right: four matching pairs and eight
visible labels. Extra generation candidates never render.

| Variant | Left label | Right label |
| --- | --- | --- |
| eng-pinyin | First English gloss | Display pinyin |
| pinyin-hanzi | Display pinyin | Simplified hanzi |
| hanzi-eng | Simplified hanzi | First English gloss |

Right-column labels are unique after the agreed comparison normalization.
Words without English are eligible only for pinyin-hanzi. Insufficient distinct
words produce the approved filter/reset recovery message instead of a panic.

Endless has Next round between rounds and no summary. The required header streak and
cumulative correct count are described below. Fixed stops after the selected number of rounds and shows
its summary. Nominal progress advances four words per round; pool exhaustion
loops with the approved hint. Duplicate-label sliding/fallback may cause overlap.

Mid-round exclusion affordance deferred, not part of v1 (D13 amendment,
2026-10-27). Exclusions remain managed through setup Reset. Preserve the D13/D15
saved/pending and URL-authority lifecycle; deferring the control does not remove
those contracts.

### Match interaction (UX-MATCH01)

Select any card on either side. A different card on the selected side is a
no-op; clicking the selected card again deselects it. Select a card on the
opposite side to attempt a pair. Escape clears the current selection.

A wrong attempt flashes both cards red for about 400 ms with a horizontal shake
of about 200 ms and a visible ✗. Ignore clicks during the flash, then clear
selection and allow retries. A correct pair turns green with a visible ✓, then
settles into a muted, readable state and remains in place. Preserve all card positions. Matching another pair
is allowed while a previous pair fades.

Keep Next round in a persistent bottom action bar, always visible in the same
position. It is dimmed and disabled until all four pairs match, then becomes
the primary enabled action. The round cannot end with an unmatched pair.
Reduced motion disables shake and fade; retain the red/green changes and icons,
with the correct state snapping to the muted, readable final surface.

Cards are Tab-focusable and Enter/Space-selectable with visible focus.
Screen-reader announcements of selection, matches, errors and scores are
intentionally omitted for v1; this is not a WCAG conformance claim.

### Practice header (UX-MATCH02)

Endless shows an in-session consecutive-correct streak and cumulative correct
count in the header, with no round number. The first wrong attempt or wrong
answer in a round resets the streak. Show a flame/accent icon when streak ≥ 3.
The counters are not daily streaks and are never persisted across reloads.

Matching shows `N / 4` near the round, updated after each correct pair.
Fixed also shows `Round N of M` in the header. Summary accounting remains
separate from these live counters and follows the current [Practice outcomes](../../learning-practice/practice/practice-outcomes.md) contract.

A09 implements these matching interactions for all three variants. Correct
cards remain disabled in their original grid positions; a 350 ms local background-color
animation briefly retains green before settling to a muted solid surface. Wrong feedback locks selection
for 400 ms, with a 200 ms shake. Reduced motion removes both animations and settles
matched cards instantly. Listen controls do not select a card. Insufficient
distinct words show the approved filter/reset explanation and Back to setup.

### Local quiz Matching stability

The local quiz uses one stable matching grid and a compact progress footer.
Keep the Continue button mounted in the same footer position throughout the
round. It is disabled until all pairs match and enabled immediately at completion.
Do not insert a `Completed` heading or a new in-flow action when completion
occurs. Matching errors change card styling and the text of a permanently
mounted, visually hidden accessible status node; errors never add a new flow
child or move the board. Preserve the existing Tailwind v4 design system.

Horizontal progress-creature artwork in the Midnight and Maple practice themes
has a finite source-image edge. Feather that horizontal boundary at the existing
progress frame rather than changing page heights or clipping the rail.

### Audio matching preparation feedback

While audio is being prepared, show a compact activity indicator, a completed count
and a bounded progress bar. Do not expose the selected Chinese words during
preparation, since they are the answers to Audio to Hanzi matching. On failure,
show a concise error, the failing word and its position, and the selected words
for that round so the learner can identify problematic vocabulary before retrying.
Keep raw network and acoustic-model diagnostics in development logs rather than
in visible UI. A synthesis request that exhausts acoustic frames without EOS
is an inference failure, not a connectivity or vocabulary-selection failure.
