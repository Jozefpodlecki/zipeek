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
- Treat [docs/DIGEST.md](docs/DIGEST.md) as a consumed scratchpad: read it at the
  start of every session, apply its answers to the owning documents, record the
  result in `docs/process/completed/` for the consumption date, then wipe
  consumed entries. Do not link to it from other documents.

## Current entries

### UX — Shell cleanup: remove navigation decoration, move repo link to footer

Three changes to the app shell, none requiring a new D-number.

**Remove navigation numbers.** Each navigation link in the drawer currently
renders a `navigation-number` span (`01`, `02`, `03`) before the label. Remove
the spans from markup and their CSS selectors from stylesheets. Navigation links
contain only the label text.

**Remove the active-route indicator dot.** Each link renders a
`navigation-indicator` span at the right, shown when the link is `is-active`.
Remove the span and its CSS. **No replacement visual cue is added.** The active
state remains only via `aria-current="page"` for assistive technology. Sighted
users rely on the page content to know which route they are on; the navigation
links have no visual distinction between active and inactive. Do not add bold
text, underline, colour, left border, or any other active-state decoration.

**Move the GitHub repository link to the footer.** The repository link currently
lives in the drawer's `sidebar-tools` block. Move it to the footer alongside the
copyright and CC-CEDICT attribution. Keep the accessible name
("Zipeek repository on GitHub"), the `target="_blank"` and
`rel="noopener noreferrer"` attributes, and the `simple_icons` GitHub icon with
its existing license comment.

**Footer layout: one line.** Copyright, attribution, and GitHub icon all sit on
a single row, dot-separated:

**No replacement decoration.** Do not add other decorative chrome to navigation
(numbering, dots, underlines, borders) unless a future UX decision asks for it.

```text
© 2026 Jozef Podlecki · Chinese dictionary data from CC-CEDICT (cc-cedict.org), licensed CC BY-SA 4.0. · [GitHub icon]
```

### UX — Word of the Day card fields (UX-LAND01)

Display, in this order inside the card:

- Top row of small badges: HSK level, part-of-speech. Both as muted rounded pills.
- Center: simplified hanzi in very large type (the focal point).
- Below: traditional hanzi, smaller and muted.
- Below: pinyin, medium size, accent-colored.
- Bottom: English gloss (first stored gloss).
- A volume icon button near the pinyin or hanzi to trigger pronunciation via the
  Web Speech API (see UX entry below).

Empty and load-failure states follow D05: "No word available" plus Retry on load
failure.

**Resolves:** UX-LAND01 in [product/UX.md](../product/UX.md). Agent must remove
the TBD marker and record the field list there.

### UX — Setup screen layout (UX-SETUP01)

Single-column, scrollable, no decorative filler. Top to bottom:

1. **Activity**: six variant tiles in a grid (three match variants, three MCQ
   variants). Each tile has an icon, label, and short subtitle ("Match four
   pairs" / "Pick the right answer") so the match/MCQ distinction is visible.
2. **HSK level**: system segmented control (`HSK 2.0` / `HSK 3.0`), then level
   buttons conditional on system (1–6, or 1–6 plus `7–9`). Default level 1.
   No "all levels" button.
3. **Mode**: `Endless` / `Fixed` segmented control. When Fixed is selected, three
   count pills `5` `10` `20` appear with `10` preselected. No custom count input.
4. **Start**: single full-width primary button. Above or beside it, the
   `Excluded words: N · Reset` line, shown only when count > 0.

Do not add audio, example sentences, or stroke order to setup. Those are not in
the data model and are not approved.

**Resolves:** UX-SETUP01 in [product/UX.md](../product/UX.md). Agent must remove
the TBD marker and record the layout there.

### UX — Pronunciation (new; affects PRODUCT/UX/ARCHITECTURE)

Add a pronunciation button (volume icon) on the Word of the Day card, on match
cards, and on MCQ prompts/answers where a hanzi or pinyin label is shown.

Implementation: browser-native **Web Speech API**, specifically
`window.speechSynthesis.speak(new SpeechSynthesisUtterance(text))` with
`utterance.lang = 'zh-CN'`. No audio files, no new assets, no new DB columns.
Voice availability and quality depend on the user's OS/browser; do not promise
consistent audio.

Icon: `yew_icons` / lucide `volume`.

Recording requirements: agent must add this to
[product/UX.md](../product/UX.md) as a new UX section, to
[product/PRODUCT.md](../product/PRODUCT.md) under the landing/practice sections,
and to [architecture/modules/web-shell.md](../architecture/modules/web-shell.md)
(or a new small module note) as the JS interop boundary. Record a new decision
in [process/decisions/web-and-ui.md](../process/decisions/web-and-ui.md) —
proposed ID **D32 (Web Speech API for pronunciation)** — and add the index row.
Note that testing verifies the button calls the API (spy/stub), not that audio
was audible.

**Open side-question for agent to ask, not to answer:** which exact label does
the button read aloud for each surface?
- Word of the Day: simplified hanzi (default; `zh-CN`).
- Match round: the hanzi on the card (both left and right cards may offer it; if
  the label is pinyin or English, the button reads the underlying hanzi, not the
  label).
- MCQ: the prompt if it is hanzi, otherwise the correct answer's hanzi.

If the user does not confirm the per-surface rule before consumption, default to
"read the simplified hanzi of the word the card represents, regardless of which
label the card shows" and note that as the assumption in the completed outcome.

### UX — Match interaction (UX-MATCH01)

Duolingo-style matching interaction. A round shows four cards on the left and
four on the right; the user pairs them one at a time.

**Selection.** Click any card to select it. Selection is one side only: after
selecting a left card, clicking another left card is a no-op; the user must
click a right card (or click the selected card again to deselect). The same
applies in reverse. Clicking the already-selected card toggles it off.

**Wrong attempt.** Both cards briefly flash red (~400 ms) with a ~200 ms
horizontal shake, then selection clears. The user can retry immediately. Clicks
are ignored during the flash so no second attempt can stack. Reduced motion
disables the shake and any fade; the red colour flash stays.

**Correct match.** Both cards turn green, then dim and stay visible in place.
Layout is preserved — the round stays visually stable, no cards collapse or
disappear. Clicks during the green fade are allowed; a fast user can match
another pair while the previous one is still animating. Reduced motion disables
the fade; cards snap to their green-then-dimmed state, keeping the colour change.

**Next round button.** A persistent footer/bottom bar action, always present
but dimmed and disabled until all four pairs are matched. When the fourth pair
matches, it becomes the primary action. Do not hide it or shift its position —
only its visual state and clickability change. This follows the Duolingo
"Check" / "Continue" pattern.

**Round end.** When all four pairs are matched, the round is complete and the
Next round button enables. In Fixed mode, the header shows the completed round
count; in Endless, the streak and cumulative-correct counters update.

**Keyboard.** Required and fully operable without a mouse. Method is up to the
implementer; minimum expected: Tab to move focus across cards, Enter/Space to
select, Escape to clear the current selection. The focus indicator must always
be visible. Implementer may add arrow-key navigation within or across columns.

**Screen reader announcements.** Not required for this round. Selection,
match, and wrong-answer feedback are visual only. This is a deliberate exclusion;
if WCAG AA is later targeted for this specific interaction, it needs revisiting.

**Don't show again.** Deferred for v1. D13's mid-round exclusion affordance is
not implemented. Exclusions remain managed via setup Reset only. The agent must
record this as a deferred clause in D13's decision body (one line: "mid-round
exclusion affordance deferred, not part of v1") and in
[product/UX.md](../product/UX.md) under Practice → Match. Do not remove D13's
exclusion lifecycle rules; only the mid-round affordance is deferred.

**Resolves:** UX-MATCH01 in [product/UX.md](../product/UX.md). Agent must remove
the TBD marker, write the interaction above, and add the reduced-motion,
click-lock, and Next-round-bar rules there. Also add these behaviours to
[testing/e2e.md](../testing/e2e.md) as e2e coverage requirements: selection by
either side, same-side click is a no-op, toggle deselect, red flash + click
lock, green-then-dimmed layout stability, Next round disabled-until-fourth-match,
reduced-motion disabling shake and fade. Update the "match/mistake accounting"
entry under "PRODUCT — Open product gaps" once the user answers UX-SUM01.

### UX — Endless tally and streak (UX-MATCH02)

Endless mode shows an **in-session streak** in the header: a consecutive-correct
count that resets on the first wrong attempt or wrong answer in a round. This is
an in-session counter, not a daily streak and not persisted across reloads;
localStorage is not involved. Include a visual flame/accent icon when the streak
is ≥ 3.

Also show cumulative correct count alongside the streak, so the user sees total
progress even after a reset.

Per-round progress: show `N / 4` as text near the round, updating on each correct
pair. Do not show the current round number in the Endless header — only the
streak and cumulative-correct count.

Fixed mode: same per-round `N / 4`, and the header additionally shows
`Round N of M`.

**Resolves:** UX-MATCH02 in [product/UX.md](../product/UX.md). Agent must remove
the TBD marker, add a "Practice header" section describing the streak +
cumulative-correct display and the `N / 4` per-round progress, and record the
streak state shape in the practice feature reducer note when A31 is written.

### UX — MCQ feedback (UX-MCQ01)

A round shows one prompt and three answer options. Click, feedback, Continue.
Modelled on Duolingo's quiz pattern.

**Feedback on click.** The clicked button immediately shows its state:

- Correct answer → green.
- Wrong selected answer → red.
- The other two options → dimmed grey.

The user sees the full state at once; nothing is hidden.

**Locked after one click.** All three answer buttons become non-clickable after
the first selection. No retrying, no switching. One selection per question.

**Continue to advance.** A persistent bottom bar action, always visible and
dimmed until an answer is chosen, then enabled. Mirrors the match round's Next
round button placement (UX-MATCH01). Clicking Continue loads the next question.

**Mistake counting.** One wrong selection per question = one mistake. Because
the answer locks after the first click, this is unambiguous. Fixed-mode summary
shows the count (0..N). Endless does not count.

**Number keys.** Each answer button shows a `1`, `2`, or `3` badge in its
corner. Pressing `1`/`2`/`3` selects that answer, same as clicking it. This is
a keyboard shortcut, not a full keyboard-navigation scheme; the answer buttons
are still Tab-focusable and Enter/Space-selectable per the shell's existing
accessibility rules.

**Streak resets on wrong answer.** Same rule as UX-MATCH02: a wrong MCQ answer
resets the in-session streak counter. Fixed mode still tracks the round count
separately.

**Reduced motion.** With `prefers-reduced-motion`, colour changes stay
instantaneous; no slide or fade transitions. Same rule as UX-MATCH01.

**Prompt.** Large, centred. When the prompt is hanzi
(`hanzi-to-meaning`), include a pronunciation button (volume icon) that reads
the hanzi via the Web Speech API (see UX pronunciation entry).

**Answers — adaptive layout.** On narrow screens, always stacked vertically
full-width. On wide screens, use a horizontal row if all three answers are
short; stack vertically if any is long. The implementation picks the exact
character-count and breakpoint thresholds; the shell's existing `48rem` mobile
breakpoint is the natural starting point.

**Progress bar.** A horizontal progress bar above the prompt shows
`completed / total` questions in Fixed mode. No text label ("Question 3 of 10"
is not shown). Endless mode has no progress bar; only the streak and cumulative
correct counters in the header.

**Resolves:** UX-MCQ01 in [product/UX.md](../product/UX.md). Agent must remove
the TBD marker and write the interaction above there. Add these behaviours to
[testing/mcq.md](../testing/mcq.md) as coverage requirements: colour tint on
click, locked buttons, Continue gating, number-key selection, adaptive answer
layout, progress bar absent in Endless, and reduced-motion colour-only
behaviour.

### UX — Fixed summary (UX-SUM01)

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
[PRODUCT.md](../product/PRODUCT.md)'s current "Play again resets the pool
offset to zero" line; the agent must update that sentence to say "fresh random
offset" and note the change in the completed outcome.

**Back to setup.** Return to `/practice` with all filters cleared (HSK system,
level, mode, count all default). Not preselected. An explicit change from the
common pattern; agent must note this in [product/UX.md](../product/UX.md)'s
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
- Add `chart.js` to [AGENTS.md §4](../../AGENTS.md) dependency table with the
  exact pinned version and the "summary ring only" restriction.
- Record a decision amendment in
  [process/decisions/database-and-artifacts.md](../process/decisions/database-and-artifacts.md)
  or a new D33 ("Chart.js for Fixed summary ring") noting: new dependency
  approved, tree-shaken, no general charting scope, no persisted chart data.

**Resolves:** UX-SUM01 in [product/UX.md](../product/UX.md). Agent must:

- Remove the TBD marker and write the summary section above.
- Update [product/PRODUCT.md](../product/PRODUCT.md) — replace "Play again
  resets offset to zero" with "Play again restarts with a fresh random offset",
  and add the four stats, the doughnut ring, and the "no resume on navigate-away"
  rule.
- Add `chart.js` to the approved dependency policy (`AGENTS.md §4`).
- Record the new decision (D33 or an amendment).
- Add e2e coverage in [testing/e2e.md](../testing/e2e.md) and
  [testing/mcq.md](../testing/mcq.md): summary appears only in Fixed; four
  stats present as text; doughnut canvas rendered; Play again starts with a
  random offset (verify by comparing generated URLs across two Play again
  clicks); Back to setup returns to `/practice` with cleared filters; navigate
  away and return starts fresh; reduced motion disables count-up.
- Add the Chart.js license file under the packaged asset notices.

### UX — Browse semantics and states (UX-BROWSE01, UX-BROWSE02)

Browse is a paginated word list with a smart search box, filter chips, and a
settings modal. URL state, live results, and a distinct filter-token grammar.

**Search box.** A single combobox input at the top. Typing shows an autocomplete
dropdown with:

- **Filter row** (shown first when the token is recognized): `+ Filter by PoS:
  verb`, `+ Filter by HSK: 3.0 level 2`, `+ Filter by frequency: 100–500`, etc.
- **Text search row** (always present): `Search for "verb"`.

Both rows always appear for any recognized filter token. Clicking a filter row
converts the typed text into a chip below the input. Clicking a text-search row
leaves the text in the input as the free-text query. If the user keeps typing
without picking a row, no filter is applied and the text remains free-text.

**Filter token grammar.** Recognized tokens produce filter rows:

| Typed | Meaning |
| --- | --- |
| `noun`, `verb`, `adjective`, … (all 42 D18 canonical PoS tokens) | Filter by part of speech |
| `hsk2`, `hsk3` | Filter by HSK system |
| `hsk2:1` … `hsk2:6` | HSK 2.0 level |
| `hsk3:1` … `hsk3:7` or `hsk3:7-9` | HSK 3.0 level (7 and 7-9 both mean the advanced band) |
| `hsk2:1-4`, `hsk3:2-7` | HSK level range (clamped to system bounds) |
| `freq:100` | Exact frequency value |
| `freq:100-500` | Frequency range (inclusive, no artificial cap) |
| `id:42` | Single word ID |
| `simp:吃` | Exact-match simplified hanzi |
| `trad:吃` | Exact-match traditional hanzi |
| `pinyin:chi` | Exact-match pinyin |

Range tokens clamp to valid bounds: `hsk2` levels 1–6, `hsk3` levels 1–7. An
invalid range (`hsk2:0`, `hsk3:8`) is a parse error — no chip is created, and a
small inline message says what the valid range is.

`id:` accepts **one ID per chip**. Comma-separated values (`id:42,43,44`) do
not parse; the user adds multiple chips instead.

Filter tokens are always **exact-match** regardless of the global strategy
setting. The strategy affects only the free-text box.

**Filter chips.** Applied filters appear as removable chips below the search
input. Each chip shows its token and a visible `×` for removal. Multiple chips
combine with AND: `noun` + `hsk3:2` means "HSK 3.0 level 2 nouns". Removing a
chip re-runs the query.

**Settings modal.** A settings icon next to the search box opens a modal with:

- **Global match strategy** for free-text: radio buttons `Exact` / `Prefix` /
  `Substring`. No per-field override.

Defaults: `Substring`.

The setting is encoded in the URL (`strategy=`). The modal does not have a Save
button; selecting a strategy closes the modal and updates the URL immediately
(on the same `replaceState` rule as below).

**Free-text matching fields.** With no field-specific chips, free-text matches
against all three fields:

- **Pinyin** — applies the global strategy (exact/prefix/substring) against
  normalized pinyin (tone-stripped, whitespace removed, case-folded).
- **English** — always substring, case-insensitive.
- **Simplified hanzi** — prefix only (exact if the user typed the whole
  character sequence; prefix if shorter). Ignores the global strategy;
  substring on hanzi produces nonsense matches.

Results are the union of matches across fields.

**Result ordering.** Pinyin alphabetically, ascending. Tie-break by simplified
hanzi bytes.

**Page size.** User-selectable from `5`, `10`, `15`, `20`. Default **5**.

**Pagination.** Numbered page links with previous/next arrows. Example:
`← 1 2 [3] 4 5 … 100 →`. Ellipsis for gaps. Clicking a page navigates to that
page.

**URL state.** All browse state is in the URL:

```text
#/browse?q=chi&strategy=prefix&f=pos:verb,hsk:3.0:2,freq:100-500
```
q= — free-text query (URL-encoded).

strategy= — exact / prefix / substring. Omitted means default
(substring).

f= — comma-separated applied filter tokens. Each token is URL-encoded; the
comma is a separator. Order is insertion order.

URL updates on Enter, not on every keystroke. As the user types in the
search box, the visible results may update live, but the URL is only updated
when the user presses Enter (or clicks a chip, or changes the strategy, or
changes the page — those update the URL immediately). This avoids history spam
and avoids replaceState thrashing during typing.

History behaviour. URL updates use replaceState, never pushState. Back
never walks through individual keystrokes. Browser Back returns to the page
before Browse.

Empty query, initial load. With no q= and no chips, show all words on
page 1 with a small hint above the list: "Type to search or add a filter".

Zero results. "No words matched" plus a Clear search link that clears
q=, all chips, and the page number, and returns to the initial all-words view.

Loading state. No visible spinner for normal queries — DuckDB-WASM
completes in <100 ms against 12,100 rows. If a query takes >200 ms, show a
subtle spinner in the results area. This threshold is a perf guard, not a
normal state.

Query failure. Show inline in the results area: "Couldn't load results" plus
a Retry button that re-runs the last query. The URL remains unchanged so a
retry is possible without retyping.

Search trigger. As-you-type with 300 ms debounce. Enter forces an immediate
re-run and commits the URL. Clicking a chip or a page link triggers an immediate
re-run.

Keyboard behaviour. Standard combobox pattern: Tab to focus the search
input; Up/Down to move between autocomplete rows; Enter to select the focused
row (or to commit free-text if no row is focused); Escape to close the
dropdown. Chips are Tab-focusable; each × is activated by Enter/Space.

Recording requirements for the agent:

Update product/UX.md → "Browse" section: remove both TBD
markers (UX-BROWSE01 and UX-BROWSE02) and write the above.

Update product/PRODUCT.md → "Routes" table entry for
/browse: replace "Paginated results with part-of-speech, pinyin, and
hanzi-character filters" with a short summary of the actual model ("Paginated
word list with a smart search box, filter chips, and settings modal").

Update architecture/modules/browser-db.md
→ "Browser query modules" section: add a new query method
browse(query, filters, strategy, page, page_size) with the SQL shape implied
by the filter grammar. Note the ordering (pinyin ASC, simplified hanzi tie-break)
and the count query alongside the row query.

Add e2e coverage in testing/e2e.md for the browse page:
autocomplete suggestion rows, chip creation/removal, strategy change, URL
state roundtrip on Enter, page navigation, zero-results clear, query failure
retry, keyboard navigation of the combobox.

Note in process/decisions/foundations.md
or a new D-number that browse has a filter-token grammar and URL state — a
small decision entry if you want it recorded, or skip if you consider it
covered by the existing D13 URL-state precedent.

### UX — Copy and advanced-band label (UX-COPY01)

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

- Update [product/UX.md](../product/UX.md) → "Copy and tone" section: remove
  the TBD marker and write the tone, advanced-band label, sentence case,
  number formatting, and copy strings above.
- Update [product/PRODUCT.md](../product/PRODUCT.md) → "MCQ practice" and
  "Practice" sections: replace any "HSK 7–9 or Advanced (7–9)" with just
  "HSK 7–9".
- If traditional is added to match cards: update the variant table in both
  PRODUCT.md and UX.md, and add an amendment note to
  [process/decisions/rounds-and-replay.md](../process/decisions/rounds-and-replay.md)
  under D12: "Right-label comparison for `pinyin-hanzi` compares composite
  simplified · traditional, not simplified alone. Amends D12."
- Update the daily-card badge spec in UX-LAND01 to include the band variant.
- Add copy assertions to Chrome e2e in [testing/shell.md](../testing/shell.md)
  and [testing/e2e.md](../testing/e2e.md).

### UX — Accessibility target (UX-A11Y01)

**Pragmatic accessibility. No formal WCAG conformance claim.** The app
implements the specific rules below; it does not claim WCAG A or AA, because
practice interactions deliberately omit screen-reader announcements (per
UX-MATCH01 and UX-MCQ01) and the summary chart is canvas-only (per UX-SUM01).

**Keyboard: everything must work without a mouse.** Tab moves focus, Enter/Space
activates. Every interactive element — cards, answer buttons, navigation links,
theme toggle, drawer toggle, volume buttons, GitHub link, Next round / Continue
buttons — is reachable and operable by keyboard. No element may be mouse-only.

**Visible focus.** Every focusable element has a clearly visible focus indicator
(a ring or outline with at least 3:1 contrast against its surroundings). Never
`outline: none` without a replacement. Focus styles from the shell already
exist; extend them to any new interactive element added later.

**Semantic HTML before ARIA.** Use `<button>` for actions, `<a>` for navigation,
`<nav>` for navigation regions, `<main>` for page content, `<aside>` for
supplementary panels. Do not add `role` attributes to elements that already
have the correct implicit role. Do not add `tabindex` unless the element needs
to be programmatically focusable for a specific reason (skip link, focus
management target).

**Colour is not the sole indicator.** Red/green for wrong/correct in MCQ and
match must be accompanied by a visible icon or text: a `✓` for correct, `✗` for
wrong, so colourblind users can tell the difference. Applies to:

- Match cards after a wrong attempt (red flash + ✗ icon).
- Match cards after a correct match (green + ✓ icon).
- MCQ answer buttons after selection (red/green + ✗/✓).

The icon can be small and in the corner; it must be visible when the colour is
visible, and must not depend on hover.

**Contrast.** Text: minimum 4.5:1 against its background. UI components (button
borders, focus rings, icon strokes): minimum 3:1. Check the following during
implementation: muted traditional hanzi on the daily card, dimmed grey MCQ
answers, the borderless theme toggle icon (A29), the attribution footer text,
and the red/green feedback states against both light and dark themes.

**Touch targets.** Interactive elements are at least 24×24 CSS px, or spaced so
a 24 px circle centred on one target does not overlap an adjacent target. Aim
for 44×44 px on mobile where the element size allows (volume buttons, theme
toggle, drawer toggle). The GitHub link in the footer, if icon-only, must be at
least 24×24 px.

**Focus management on route change.** When navigating between routes (Home →
Practice, Practice → Browse, etc.), focus moves to the new page's `<main>` or
its `<h1>`. It must not remain on the clicked navigation link or on a removed
element. This is the SPA equivalent of a page load. Applies to all route
transitions in the shell.

**Focus trap for the mobile drawer.** When the drawer is open on mobile and
covers the main content, Tab does not escape it into the hidden content behind.
Either the drawer is a focus-contained region with a close button that returns
focus to the toggle, or the drawer does not move focus at all and remains
reachable without trapping. A keyboard trap that cannot be escaped is not
acceptable.

**Skip link.** The existing skip-to-content link is retained. It must move focus
to `<main>` without changing the route (the current implementation does this).

**`lang` attributes.** `<html lang="en">` on the document. Hanzi elements that
appear as standalone content carry `lang="zh-CN"` where practical (daily card
hanzi, match card hanzi on the `pinyin-hanzi` and `hanzi-eng` variants, MCQ
answer hanzi). This also drives Web Speech API pronunciation language, so it
serves both accessibility and the pronunciation feature.

**Page titles.** Each route updates `document.title` (e.g. "Practice — Zipeek",
"Browse — Zipeek"). The exact format is up to the implementer; the requirement
is that routes are distinguishable by title.

**Form labels if inputs are added.** Browse will have a search input; any future
input carries a visible `<label>` or an `aria-label`. Placeholder text alone is
not a label.

**Reduced motion.** Already required by UX-MATCH01, UX-MCQ01, UX-SUM01. Applies
to any future animation too: `prefers-reduced-motion` disables non-essential
motion, keeps colour changes.

**Screen-reader feedback for practice: deliberately none.** Selection, match,
wrong/correct, and score updates are visual only. This is recorded as a
deliberate omission; if a future decision reverses it, this entry and the
practice interaction entries (UX-MATCH01, UX-MCQ01) must be revisited together.

**No ARIA "just in case".** Native elements provide the correct semantics. Do
not add `role="list"` to a `<ul>`, `aria-label` to a button that has visible
text, or `aria-live` to regions that do not need announcements.

**Recording requirements for the agent:**

- Update [product/UX.md](../product/UX.md) → "Accessibility" section: remove the
  TBD marker and write the rules above.
- Update [architecture/UI_ARCHITECTURE.md](../architecture/UI_ARCHITECTURE.md)
  → route-transition lifecycle section: add "focus moves to main content heading
  on committed navigation" as an explicit acceptance criterion.
- Add a note to [architecture/modules/web-shell.md](../architecture/modules/web-shell.md)
  → shell section: the mobile drawer must not trap focus.
- Update [testing/shell.md](../testing/shell.md) and
  [testing/e2e.md](../testing/e2e.md): coverage for keyboard operability of all
  interactive elements, visible focus, focus-to-main on route change, no focus
  trap in the mobile drawer, ✓/✗ icons alongside red/green feedback, page title
  changes per route.
- Add a contrast spot-check step to the Chrome e2e run or the implementation
  checklist; do not add an automated axe/Lighthouse check in v1 (that would be a
  new tooling dependency requiring approval).
- Do not claim WCAG conformance anywhere in the docs.

### UX — Device policy (UX-RESP01)

**Supported width.** Minimum 400 px. Screens below 400 are best-effort only —
no refusal screen, no "not supported" message, but no guarantee of usability
either. The 400 px floor is a promise, not an enforcement.

**Maximum width.** None. Shell and content both stretch to fill the viewport at
any width. No centered max-width container; on ultrawide monitors, cards,
lists, and the summary stretch edge to edge.

**Orientation.** Portrait and landscape both. No rotate-blocking.

**Breakpoints.** Tailwind default breakpoints replace the shell's current
48/60 rem values:

| Token | Width | Role |
| --- | --- | --- |
| `sm` | 40rem (640px) | Small layouts; drawer begins collapsing |
| `md` | 48rem (768px) | Mobile/desktop line; desktop rail layout |
| `lg` | 64rem (1024px) | Wider content layouts |
| `xl` | 80rem (1280px) | Browse table appears; large-desktop layouts |

Agent must update `main.css` and any Tailwind config to use these breakpoints
and remove the old 48rem/60rem hard-coded values. Re-run the Chrome responsive
test at 400, 640, 768, 1024, 1280, 1920 px widths.

**Match exercise on narrow screens.** Below the `md` breakpoint (or wherever the
shell switches to mobile), stack the two card columns vertically: left column
above right column, each wrapped in a small visual container (a bordered or
subtly-tinted box) so the user knows which column is which. Scrolling is
accepted; the target is casual practice, not competitive speed. On `md` and
above, the two columns sit side by side as now.

**MCQ on narrow screens.** Answers stay full-width stacked (handled by
UX-MCQ01's adaptive rule). No change needed.

**Browse layout.** Card list below `xl`, table at `xl` (1280px) and above.
<columns line pending your answer: simplified / pinyin / english / HSK / PoS,
or the set you specify>.

**Input methods.** Pointer, touch, and keyboard all supported everywhere.
Consistent with UX-MATCH01's keyboard requirement. No desktop-only or
mobile-only features.

**"Supported" means best-effort.** No "not supported" screen. If the app
renders, it's usable enough at any width. The 400 px floor is documented policy,
not enforcement.

**Recording requirements for the agent:**

- Update [product/UX.md](../product/UX.md) → "Responsive breakpoints" section:
  remove the TBD marker and write the policy above.
- Update [architecture/modules/web-shell.md](../architecture/modules/web-shell.md)
  → "Implemented web shell and landing foundation" section: the sentence about
  "48rem" and "60rem" breakpoints becomes "Tailwind defaults: sm 40 / md 48 /
  lg 64 / xl 80", with the roles as above.
- Change the current `48rem` and `60rem` values in `main.css` (or wherever the
  shell breakpoints live) to Tailwind's `md` (48rem for the mobile/desktop line)
  and `xl` (80rem for browse table). The `60rem` tile-stacking breakpoint
  becomes `lg` (64rem).
- Add a Chrome responsive test at 400, 640, 768, 1024, 1280, 1920 px widths in
  [testing/shell.md](../testing/shell.md), asserting: no horizontal overflow at
  400; two-column match at 768+; card list browse below 1280; browse table at
  1280+; sidebar behaviour at 768+; summary readable at 1920.
- Update [testing/e2e.md](../testing/e2e.md) responsive section to reference
  these widths.

**Does not need a new D-number** — this fills an explicit UX TBD
(UX-RESP01).

### PRODUCT — Open product gaps

Still open: match/mistake accounting model, Random variant pool, Reset on seeded
rounds, excluded-words count semantics, Fixed on wrap, landing tiles on DB
failure, SEO/social meta, OS theme respect, shared seeded URLs and exclusions,
per-card exclusion UX, MCQ prompt variety (`meaning-to-pinyin` answer is display
pinyin, per D20 — confirm), practice progress display.

### ARCHITECTURE — Proposals awaiting approval

Still open: D28 modular e2e layout, D29 scoped state ownership details, D29
initial motion profile, D30 crate extraction confirmation.

### PROCESS — CODING_STANDARDS wording conflict

Still open: pick which of the two "sole exception" readings is intended
(unreachable-branch comments vs `#[allow(...)]` rationale) and update
[process/CODING_STANDARDS.md](../process/CODING_STANDARDS.md).

### PROCESS — A19 transitive lockfile policy

Still open: whether to add a hard "no `cargo update` without approval" rule on
top of D27's tracked root lockfile and `--locked` CI.

### PROCESS — Picomatch advisory (CVE-2026-33671)

**Decision: accept.** The advisory (ReDoS via crafted extglob patterns) only
affects applications that pass untrusted, user-supplied glob patterns to
picomatch. Zipeek's picomatch is a transitive build-time dependency (Trunk/
Tailwind tooling); it is not shipped to browsers and receives only
developer-controlled patterns. No user input reaches it.

The affected version (2.3.1) and the fixed versions (2.3.2 / 4.0.4) are
recorded for future reference. Do not add an override or run `npm audit fix`
for this advisory. If a future Trunk or Tailwind upgrade pulls picomatch to a
fixed version on its own, that's fine; no action required.

**Recording requirements for the agent:**

- Update [process/CURRENT.md](../process/CURRENT.md) → the "Other recorded
  follow-up" section, or the picomatch note if it still exists there: replace
  "A16 tracks review; dependency upgrades require specific approval" with
  "Accepted as build-time-only, no user input reaches it; see
  completed/2026-10-27.md for the rationale."
- Add a row to [process/completed/2026-10-27.md](../process/completed/2026-10-27.md)
  (or the consumption date) recording the acceptance, the CVE, and the
  reasoning.
- Do not modify `package.json`, `package-lock.json`, or `AGENTS.md §4`.