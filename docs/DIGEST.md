# Digest

Scratchpad for pending design answers and decisions. This file is written by the
user, consumed by an agent, and wiped after consumption. It is not a permanent
reference and must not be linked from other documents.

## How to use

1. The user writes answers, preferences, and design choices here.
2. An agent reads the whole file at the start of a session, applies those answers
   to the owning documents (`product/UX.md`, `product/PRODUCT.md`,
   `process/decisions/*`, `architecture/*`, `testing/*`, `process/TASKS.md`, and
   so on), records the resulting outcomes in `process/completed/` for the
   consumption date, and then wipes this file.
3. "Wipe" means: delete every entry that was consumed, leaving only the header,
   the "How to use" section, and the "Current entries" heading. If any entry was
   deferred or rejected, it stays with a one-line note explaining why.

## Rules

- **Answers, not questions.** The agent does not answer back here. If something
  is ambiguous, ask in the session and wait; do not guess into a permanent doc.
- **One topic per section.** Prefix a section with the area it affects
  (`UX`, `PRODUCT`, `DB`, `TESTING`, `CI`, `A31`, `A35`, `D29`, and so on) so the
  agent knows which owner document to update.
- **Decisions get a D-number.** If an answer resolves an open choice, the agent
  records a new `DNN` under `process/decisions/<family>.md` and adds an index
  row. If it only refines an existing decision, the agent updates that decision's
  body and notes the amendment date.
- **No tasks here.** Open work belongs in `process/TASKS.md`; this file only
  contains the answers that would otherwise block a task.
- **No implementation notes.** Structural changes belong in `architecture/`;
  what a stage does belongs in `architecture/schema/artifacts.md`; what a test
  checks belongs in `testing/`.
- **Never link to this file.** Anything permanent that references it would break
  after the wipe.
- **Preserve history after the wipe.** The consumption result goes into
  `process/completed/<date>.md` as one row per topic, linking to the files that
  changed and the new or amended decision IDs.
- Treat [docs/DIGEST.md](DIGEST.md) as a consumed scratchpad: read it at the
  start of every session, apply its answers to the owning documents, record the
  result in `docs/process/completed/` for the consumption date, then wipe
  consumed entries. Do not link to it from other documents.

## Current entries

### UX — Fixed summary (UX-SUM01)

Retained: summary amendments and the exact Chart.js pin remain deferred to A09/A10; the later approved first-attempt metric supersedes the old correct-pairs wording, and proposed decision numbers are not reserved.

Appears after the last round of a Fixed session only. Endless mode has no
summary.

**Stats displayed.** Four numbers, as real text (not canvas-only):

1. **Correct pairs** across the whole session: "16 of 20 pairs correct" (or
   "16 / 20"). Match rounds have 4 pairs each, so max is `rounds × 4`.
2. **Mistakes**: total wrong attempts across all rounds. One red flash in match
   or one red MCQ answer = one mistake. Counts every attempt, not just rounds
   with mistakes.
3. **Time taken**: wall-clock from first round render to last Continue/Next
   round click. Captured at completion; does not tick while the summary is
   shown. Never pauses when the tab is hidden.
4. **Highest streak**: the highest in-session streak reached during the
   session. Same definition as UX-MATCH02's streak.

**Visual.** A doughnut accuracy ring showing `correct pairs / total pairs`
filled. Built with Chart.js (see new-dependency section below). The ring sits
above or beside the four stats; the exact placement is left to the
implementer, but the stats must remain as text alongside the canvas for
accessibility.

**Layout.** Title ("Session complete" or similar), doughnut ring, four stats
in a row or grid, then two buttons: **Play again** (primary, filled) and
**Back to setup** (secondary, outlined).

**Play again.** Restart the same Fixed session: same HSK filter, same mode
(Fixed), same round count, but a **fresh random offset** rather than offset 0.
This is an amendment to
[PRODUCT.md](product/PRODUCT.md)'s current "Play again resets the pool
offset to zero" line; the agent must update that sentence to say "fresh random
offset" and note the change in the completed outcome.

**Back to setup.** Return to `/practice` with all filters cleared (HSK system,
level, mode, count all default). Not preselected. An explicit change from the
common pattern; agent must note this in [product/UX.md](product/UX.md)'s
Summary section.

**Navigate away.** If the user navigates away (browser Back, navigation link,
browser close) before the summary, the session is lost. No confirmation dialog,
no resume. A new visit to `/practice` starts fresh.

**Animation.** A subtle count-up on the correct-pairs number over ~300 ms,
disabled by `prefers-reduced-motion`. The doughnut ring animates its arc fill
the same way; reduced motion makes both instant.

**Dependency: Chart.js.** New npm dependency, approved for the summary only.

- License: MIT. Required: preserve the license text alongside the packaged
  assets, same pattern as `web/scripts/duckdb-LICENSE.txt`. Add it to the
  library notices folder.
- Import style: **tree-shaken** — `chart.js` with only the needed
  `DoughnutController` and `ArcElement`, not `chart.js/auto`. The chart is one
  doughnut ring; no scales, no tooltips, no legend.
- Integration: `wasm-bindgen` extern declarations (same opaque-handle pattern
  as the DuckDB bridge). A `<canvas>` inside a `position: relative` wrapper;
  create the `Chart` instance in a `use_effect_with` mount, destroy on drop.
- Accessibility: the canvas is opaque to assistive tech. The four numeric
  stats must be real DOM text alongside the canvas, never canvas-drawn.
- Add `chart.js` to [AGENTS.md §4](../AGENTS.md) dependency table with the
  exact pinned version and the "summary ring only" restriction.
- Record a decision amendment in
  [process/decisions/database-and-artifacts.md](process/decisions/database-and-artifacts.md)
  or a new D33 ("Chart.js for Fixed summary ring") noting: new dependency
  approved, tree-shaken, no general charting scope, no persisted chart data.

**Resolves:** UX-SUM01 in [product/UX.md](product/UX.md). Agent must:

- Remove the TBD marker and write the summary section above.
- Update [product/PRODUCT.md](product/PRODUCT.md) — replace "Play again
  resets offset to zero" with "Play again restarts with a fresh random offset",
  and add the four stats, the doughnut ring, and the "no resume on navigate-away"
  rule.
- Add `chart.js` to the approved dependency policy (`AGENTS.md §4`).
- Record the new decision (D33 or an amendment).
- Add e2e coverage in [testing/e2e.md](testing/e2e.md) and
  [testing/mcq.md](testing/mcq.md): summary appears only in Fixed; four
  stats present as text; doughnut canvas rendered; Play again starts with a
  random offset (verify by comparing generated URLs across two Play again
  clicks); Back to setup returns to `/practice` with cleared filters; navigate
  away and return starts fresh; reduced motion disables count-up.
- Add the Chart.js license file under the packaged asset notices.

### UX — Copy and advanced-band label (UX-COPY01)

Retained conservatively: copy is recorded, but summary-label application remains deferred; A37 selected secondary traditional display with unchanged comparison keys, superseding both original conditional branches below.

**Tone: neutral/instructional.** No exclamation marks, no emoji, no "Nice work!"
or "Oops!". Plain, factual language. The streak flame icon from UX-MATCH02 is a
visual exception; its text is still neutral ("5"). Language is English only.

**Advanced-band label.** Setup buttons show `HSK 7–9` for the combined band,
matching the format of `HSK 1` through `HSK 6`. No alternate label elsewhere.

**Sentence case throughout.** Headings, buttons, labels, error messages all use
sentence case. First word capitalised, rest lowercase unless proper noun.

**Number formatting.** Pluralize correctly (`1 mistake` / `3 mistakes`). Time as
`2m 14s`. Correct pairs as `16 / 20`.

**Summary stat labels.** "Correct pairs", "Mistakes", "Time", "Highest streak".

**Copy for open strings:**

| Context | Copy |
| --- | --- |
| Summary title | "Session complete" |
| Setup heading | "Practice" |
| Browse empty | "No words matched" |
| Browse query failure | "Couldn't load results" + Retry |
| Loading shell | "Loading" |
| 404 | "Page not found" + Back to home |
| Pronunciation button tooltip | "Listen" |

**Existing copy preserved** (no change, listed for completeness): "No word
available", "Retry", "Next round", "Play again", "Back to setup",
"Excluded words: N · Reset", "pool exhausted, looping", "Not enough distinct
words for this filter. Try a different HSK level or reset exclusions.",
"not enough words after exclusions — reset exclusions?".

**Advanced-band badge on the daily card.** Show `HSK 3 · 7–9` when the word is
in the HSK 3.0 advanced band; otherwise `HSK 3 · 4` (system + level). Badge
width varies accordingly.

**Traditional hanzi display.**
<depending on your Q9 answer:
  - if daily-card only: "Traditional shown on the daily card only. Match cards
    and MCQ answers show simplified only. Existing PRODUCT/UX tables remain
    unchanged."
  - if both: "Traditional shown on the daily card and on match cards where a
    hanzi is displayed (`pinyin-hanzi` right column, `hanzi-eng` left column).
    Both appear as `吃饭 · 吃飯` (simplified · traditional). MCQ answer hanzi
    also shows both. This requires updating PRODUCT.md's variant table, UX.md's
    match exercise table, and the D12 right-label comparison for
    `pinyin-hanzi`: the comparison key becomes simplified + separator +
    traditional, not simplified alone."
>

**Recording requirements for the agent:**

- Update [product/UX.md](product/UX.md) → "Copy and tone" section: remove
  the TBD marker and write the tone, advanced-band label, sentence case,
  number formatting, and copy strings above.
- Update [product/PRODUCT.md](product/PRODUCT.md) → "MCQ practice" and
  "Practice" sections: replace any "HSK 7–9 or Advanced (7–9)" with just
  "HSK 7–9".
- If traditional is added to match cards: update the variant table in both
  PRODUCT.md and UX.md, and add an amendment note to
  [process/decisions/rounds-and-replay.md](process/decisions/rounds-and-replay.md)
  under D12: "Right-label comparison for `pinyin-hanzi` compares composite
  simplified · traditional, not simplified alone. Amends D12."
- Update the daily-card badge spec in UX-LAND01 to include the band variant.
- Add copy assertions to Chrome e2e in [testing/shell.md](testing/shell.md)
  and [testing/e2e.md](testing/e2e.md).

### PRODUCT — Open product gaps

Retained conservatively: A37 reconciles the settled items in PRODUCT, but Fixed-summary application remains deferred to A09/A10; this old list does not reopen those settled decisions.

Still open: match/mistake accounting model, Random variant pool, Reset on seeded
rounds, excluded-words count semantics, Fixed on wrap, landing tiles on DB
failure, SEO/social meta, OS theme respect, shared seeded URLs and exclusions,
per-card exclusion UX, MCQ prompt variety (`meaning-to-pinyin` answer is display
pinyin, per D20 — confirm), practice progress display.

### ARCHITECTURE — Proposals awaiting approval

Retained: D28 and D29 state/profile requirements are recorded, but D30 extraction confirmation and its explicit implementation/dependency review remain open under A33.

Still open: D28 modular e2e layout, D29 scoped state ownership details, D29
initial motion profile, D30 crate extraction confirmation.

### PROCESS — CODING_STANDARDS wording conflict

Retained: the unreachable-branch versus sole lint-rationale comment exception is still inconsistent; A41 tracks clarification without changing the rule here.

Still open: pick which of the two "sole exception" readings is intended
(unreachable-branch comments vs `#[allow(...)]` rationale) and update
[process/CODING_STANDARDS.md](process/CODING_STANDARDS.md).

### PROCESS — A19 transitive lockfile policy

Retained: D27 preserves the locked graph and approval controls, but an additional explicit cargo-update prohibition remains unanswered; A41 tracks that choice.

Still open: whether to add a hard "no `cargo update` without approval" rule on
top of D27's tracked root lockfile and `--locked` CI.
