# User experience specification

This is the user-facing specification transcribed from the agreed decisions and
[PRODUCT](PRODUCT.md). It describes intended behaviour, not completed features.
D02-D18 are the user-declared resolved baseline; a TBD below identifies a missing
UX decision, not permission to invent one. D20 settles MCQ scope. A22 records
answered UX requirements on 2026-10-27.
Unresolved choices remain explicit below; these requirements do not imply that
the corresponding UI or tests are implemented.

## Principles

The user's 2026-10-27 review requests animation/page-transition support using the
already-added Animato dependency.
[UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md) proposes the
state/lifecycle/accessibility architecture. A37 selects 150 ms fade-only route
and drawer transitions; A32 owns implementation details and validation. This
does not change the approved practice feedback timings or add controls. The existing reduced-motion loading behaviour remains required.

- Help users query and practise Chinese words through hanzi, pinyin, and English.
- Keep loading continuous from the initial static loading shell through Yew's
  first render until the dictionary is ready. Handle loading failure with retry.
- Prefer dark appearance while respecting the user's saved dark/light choice.
- A shared practice URL reproduces its round for the same database snapshot and
  effective inputs. Regenerated data can invalidate old word-ID links.
- Fixed progress and scores are session-local; Endless has no score persistence.

## Routes and navigation

These paths are app-relative; Pages hosting retains the /zipeek/ prefix.

| Path | Intended destination |
| --- | --- |
| / | Landing with Word of the Day and Practice, Browse, Random tiles |
| /practice | Practice setup |
| /practice/eng-pinyin | English-to-pinyin match |
| /practice/pinyin-hanzi | Pinyin-to-hanzi match |
| /practice/hanzi-eng | Hanzi-to-English match |
| /practice/mcq/hanzi-to-meaning | Hanzi-to-meaning MCQ |
| /practice/mcq/meaning-to-hanzi | Meaning-to-hanzi MCQ |
| /practice/mcq/meaning-to-pinyin | Meaning-to-pinyin MCQ |
| /browse | Paginated word query |

Random is a one-click shortcut into a practice variant, always Endless. It ignores
setup HSK/PoS filters; the chosen seed supplies its effective HSK/PoS context.
Seeded Next round clears the anchor and enters unseeded practice with the same
filter. Invalid query values use the agreed safe defaults, including level 1.

Home, Practice and Browse navigation is shared across routes (D24/A27, amended
2026-10-27 by A22). Links contain only their label: remove navigation numbers
and the active-route dot. Keep `aria-current="page"` as the sole active-route
indication; do not add bold, underline, colour, borders or replacement decoration.
Practice links back to setup, including
from practice-related routes. The brand links Home; keyboard skip-to-content
keeps the route unchanged.

## Landing

- Show a Word of the Day card and exactly three tiles: Practice, Browse, Random.
- Word of the Day is selected deterministically for the browser's local date
  from the full imported word pool, independently of practice filters.
- Empty data shows "No word available". A database load failure adds Retry.
- Use the shared left sidebar and footer described below.

Approved landing arrangement (D24/A27): a prominent Word of the Day card
followed by exactly three navigation tiles. The shell currently displays the
agreed unavailable-word copy; actual daily selection remains A06/A12.

### Daily card fields (UX-LAND01)

Display these fields in order:

1. Muted rounded HSK and part-of-speech badges.
2. Simplified hanzi in very large type as the focal point.
3. Traditional hanzi, smaller and muted.
4. Pinyin, medium size and accent-coloured.
5. The first stored English gloss.

Place a volume button near the hanzi or pinyin; see Pronunciation below.
The HSK badge includes system and level, for example `HSK 3 · 4` or
`HSK 3 · 7–9` for the advanced band. Empty/load-failure behaviour remains D05.

## Practice setup

- Offer HSK 2.0 levels 1-6 and HSK 3.0 levels 1-6 plus one combined 7-9 band.
  A word's membership can differ between systems. Do not show separate levels 8/9.
- Use the existing normalization defaults: Endless, HSK 3.0, level 1. A malformed
  level never selects all levels.
- Offer Endless and Fixed. Fixed has three count buttons: 5, 10, 20; default 10.
  No custom round-count input in v1.
- Show "Excluded words: N · Reset" only when N > 0. Reset clears
  saved/pending exclusions and generates a fresh practice URL without exclusions.
- Initial setup filters are HSK system + level. The URL supports PoS sets;
  a setup PoS picker is future work.

### Setup layout (UX-SETUP01)

Use a single scrollable column without decorative filler, in this order:

1. Activity: six variant tiles in a grid, three match and three MCQ. Each has
   an icon, label and subtitle: "Match four pairs" or "Pick the right answer".
   Select one or more with independent toggle buttons; keep at least one
   selected. The default set contains Hanzi to meaning only. Start chooses
   randomly from the selected set. Each subsequent round/question chooses a
   random selected type other than the previous type when more than one is
   selected; a single selected type repeats normally (D40).
2. HSK level: `HSK 2.0` / `HSK 3.0` segmented control, then conditional level
   buttons (1–6 or 1–6 plus the combined `HSK 7–9` band). Default level 1;
   no all-levels button.
3. Mode: `Endless` / `Fixed` segmented control. Fixed reveals count pills
   `5`, `10`, `20`, with `10` preselected; no custom input.
4. Start: one full-width primary button, with the exclusions/reset line above
   or beside it, shown only when the excluded count is greater than zero.

Do not add audio, example sentences or stroke order to setup.

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
separate from these live counters and is deferred under UX-SUM01.

A09 implements these matching interactions for all three variants. Correct
cards remain disabled in their original grid positions; a 350 ms local background-color
animation briefly retains green before settling to a muted solid surface. Wrong feedback locks selection
for 400 ms, with a 200 ms shake. Reduced motion removes both animations and settles
matched cards instantly. Listen controls do not select a card. Insufficient
distinct words show the approved filter/reset explanation and Back to setup.

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

## Summary screen

After the last Fixed Continue/Next round, show Session complete, a doughnut
accuracy ring and four stats as real DOM text: First-attempt, Mistakes, Time,
Highest streak. Endless never shows a summary. D35 records application of
UX-SUM01 during A10; matching applies the same requirements in A09.

For MCQ, First-attempt is correct first selections / total questions (5/10/20).
Each question has one locked selection; a wrong selection contributes one
mistake and no correct answer. For matching, the expanded metric label is
First-attempt correct pairs (or First-attempt in a compact row). A pair matched
on its first attempt counts; a pair needing any retry does not. The matching
denominator is total required pairs (rounds × 4). Every wrong matching attempt
contributes one mistake. These are distinct from eventually matched pairs.
Under the A09 clarification (D39), a wrong attempt combining words A and B
disqualifies both pairs from the first-attempt numerator. Both eventual matches
still contribute to live correct/streak counts. Matching positions/WordIds and
comparison keys remain unchanged.

Mixed sessions (D40) use one shared Fixed count of 5/10/20 rounds/questions.
Each matching round contributes four required pairs; each MCQ contributes one
required answer. The First-attempt numerator adds eligible matching pairs and
correct first MCQ selections. Its denominator is the sum of those requirements
for the actual sequence played, not a fixed rounds × 4 formula for mixed types.
Live correct/streak, highest streak, mistakes, timing and progress continue
across type changes. Play again retains the selected types and starts a fresh
session; with multiple types it also excludes the last type when choosing its
starting type. Back to setup restores the single-type default. Reload/external
navigation loses the session counters while honoring the current round URL.

Time is wall-clock from the first round render until the final Continue/Next
round click, never pauses when hidden, and freezes at completion. Display whole
minutes/seconds, such as 2m 14s. Highest streak is the session's maximum
consecutive-correct streak. Stats and session progress are memory-only. Leaving
loses the session without confirmation/resume; reload resets progress and honors
the URL offset. Pool wrapping leaves the session counter unchanged.

Play again keeps the same mode/count/filter/exclusions and starts fresh with a
random in-pool offset, replacing the old reset-to-zero behavior. Back to setup
returns to default controls, with no preselected session filters. Render both
actions as native buttons. The summary is a ring plus a responsive row/grid of
text stats; no meaning depends on canvas alone. If the chart fails, text stats
and actions remain usable. Count and arc animate for 300 ms; reduced motion
makes both instant, including when the preference changes during animation.

Chart.js is exactly pinned to 4.6.1, with esbuild 0.28.2 as a build-only tool.
Use only the tree-shaken DoughnutController/ArcElement, no scales/tooltips/legend,
with local assets and retained licenses. Create an opaque chart handle in an
effect and destroy it on unmount, including late async creation. No chart data
is persisted. This is summary-only scope, not general charting infrastructure.

## Browse

### Search and filters (UX-BROWSE01)

Browse is a paginated word list with a smart search box, removable filter chips
and a settings modal. These are intended A13 requirements, not existing controls.

The input is a combobox. A recognized token shows a filter suggestion first
(e.g. "+ Filter by PoS: verb") and a text-search row (`Search for "verb"`).
The text-search row is always present. Selecting a filter creates a chip and
consumes that token; selecting text search retains the free-text query. Typing
without choosing a filter row never implicitly applies a filter.

| Token | Filter |
| --- | --- |
| Any of the 42 canonical PoS tokens, e.g. `noun`, `verb` | Complete-token PoS match |
| `hsk2`, `hsk3` | HSK system membership |
| `hsk2:1` … `hsk2:6` | HSK 2.0 level |
| `hsk3:1` … `hsk3:7`, `hsk3:7-9` | HSK 3.0 level; 7 and 7-9 mean the advanced band |
| `hsk2:1-4`, `hsk3:2-7` | Inclusive level range within the chosen system |
| `freq:100`, `freq:100-500` | Exact frequency rank or inclusive range; no artificial cap |
| `id:42` | One WordId per chip; comma-separated IDs do not parse |
| `simp:吃`, `trad:吃` | Exact simplified/traditional hanzi |
| `pinyin:chi` | Exact pinyin match |

Filter chips always use exact matching independently of the global strategy.
Combine multiple chips with AND. Each chip displays its token and a visible,
keyboard-operable ×; removal reruns the query immediately.

HSK range bounds are clamped to 1–6 or 1–7 for their respective systems.
Invalid single levels such as `hsk2:0` and `hsk3:8` create no chip and show a
small inline valid-range message. Reversed ranges, missing endpoints,
non-integers and negative values create no chip and show an inline error. Do not
swap reversed bounds. Keep numeric HSK range clamping; frequency ranks must be
positive integers, without an artificial product cap.

The settings icon opens a modal with global free-text match strategy radio
buttons: Exact, Prefix, Substring (default). Selection closes the modal and
updates the URL immediately; there is no Save button or per-field override.

Free text returns the union of matches across:

- Pinyin: global strategy against a tone-stripped, whitespace-free, case-folded
  comparison value. Support source display and numbered forms without rewriting
  stored pinyin; this search key is distinct from D18's tone-preserving replay key.
- English: case-insensitive substring, regardless of global strategy.
- Simplified hanzi: prefix, including exact full-sequence matches, regardless
  of global strategy.

Order by pinyin ascending, with simplified-hanzi bytes as the tie-break.
Page sizes are 5, 10, 15, 20, default 5. Show numbered page links with previous/
next arrows and ellipses for gaps, e.g. `← 1 2 [3] 4 5 … 100 →`.

### Browse URL state

All Browse state belongs in the URL. The approved A37 contract is:

| Parameter | Meaning / default |
| --- | --- |
| `q` | Free text; empty when omitted |
| `strategy` | `exact`, `prefix`, `substring`; default `substring` |
| `f` | Ordered comma-separated encoded filter tokens; empty when omitted |
| `page` | One-based page number; default `1` |
| `page_size` | `5`, `10`, `15`, `20`; default `5` |

Canonical filter tokens are `pos:<canonical-pos>`, `hsk2`, `hsk3`,
`hsk2:<level-or-range>`, `hsk3:<level-or-range>`, `freq:<rank-or-range>`,
`id:<id>`, `simp:<hanzi>`, `trad:<hanzi>` and `pinyin:<text>`. The input's bare
PoS suggestion emits `pos:<token>` on the wire. HSK 3.0 advanced `7-9` single-tier
alias canonicalizes to `7`; ordinary inclusive ranges use their normalized bounds.

Encode each complete filter token individually, join tokens with literal commas
in insertion order, then serialize that entire `f` value once in the outer URL
query. Decode the outer query once, split its `f` value at literal commas, then
decode each token once. This intentional two-layer codec keeps commas/percent
signs inside token values distinct from filter separators; it is specific to
Browse and changes no practice URL encoding rule. Example semantic filters:
`pos:verb`, `hsk3:2`, `freq:100-500`.

Query, chip/filter and page-size changes reset page to 1. A requested page beyond
the available result pages uses the last available page; zero results use the
initial page 1 with the approved empty view. Malformed loaded values use safe
defaults/validated filters and never trigger an implicit URL rewrite. Explicit
new state generation uses the canonical grammar. Full codec tests must cover
Unicode, commas, percent signs, malformed escapes and reload roundtrips.
Debounce live search by 300 ms. Enter forces an immediate query and commits the
URL. Typing alone does not update it. Chip changes, strategy changes and page
navigation query and update immediately. All Browse state updates use
`replaceState`, never `pushState`; Back returns to the page before Browse.

Tab focuses the input/chips. Up/Down selects autocomplete rows. Enter chooses
the focused row, or commits free text if none is focused. Escape closes the
suggestions. Chip removal supports Enter/Space.

### Browse fields (UX-BROWSE01, resolved A37)

Use the same five fields in cards and desktop table columns, in this order:
simplified hanzi, pinyin, English, HSK and PoS. English uses the first stored
gloss. HSK shows every available system and its level/band; do not collapse
HSK 2.0 and 3.0 into one inferred level. Missing metadata is not invented.
Do not add traditional-hanzi or frequency columns. Their exact filter tokens
remain supported independently of the displayed field set.

### Browse states (UX-BROWSE02)

With no query/chips, show all words on page 1 and "Type to search or add a filter".
Zero results shows "No words matched" and Clear search, which clears query,
chips and page number and returns to the initial view.

Show no spinner for normal queries; after a query takes more than 200 ms,
show a subtle results-area spinner. This is a performance guard, not a measured
latency promise. Query failure shows "Couldn't load results" and Retry inline;
retry reruns the last query without changing the URL.

## Theme and dark mode

- Dark is the preferred default when there is no saved choice. Offer a dark/light
  toggle in the top panel at the top-right (D25), and remember the selection across reloads.
- A saved light choice overrides the dark default. The loading shell, Yew loading
  view, and ready app use the same choice without a light flash.
- Place the GitHub repository icon in the footer alongside copyright and
  attribution, linked to https://github.com/Jozefpodlecki/zipeek.
- Show Jozef Podlecki's copyright with the current year in the footer, alongside
  the required dictionary attribution. Example: "© 2026 Jozef Podlecki".

Approved direction (D24/A27): neutral slate dark/light with blue accents.
D25 amends the placement: a permanent top panel contains brand, drawer toggle
and theme control at the top-right. Home/Practice/Browse remain in an
expandable/collapsible drawer; A22 moves the repository link into the footer.
Narrow layouts show the expanded
navigation below the top panel; the collapsed drawer is hidden and content uses
the available width. Copyright/attribution remain visible in both themes.
Native buttons/links support keyboard use; the theme switch exposes its state.
The footer is one dot-separated row: copyright, CC-CEDICT attribution, GitHub
icon. Keep the repository link's accessible name "Zipeek repository on GitHub",
`target="_blank"`, `rel="noopener noreferrer"`, existing `simple_icons` icon and
licence notice. On 2026-10-28 the user approved attribution text wrapping
inside its item while copyright, separators and repository icon remain in one
horizontal row. A35 implements that layout without horizontal overflow at 400px.
A29 refines the same user-selected shell: no decorative tile arrows, a 48px top
panel, less footer padding and no outer border on the theme button. Hover and
keyboard-focus feedback remain.
D26/A30 reuses the original landscape in both themes, through loading and app
rendering. Translucent surfaces and theme overlays keep text readable. Loading
uses a small centered spinner/label panel, with animation disabled for reduced
motion; the landscape itself is unchanged and is not hidden in light mode.

## Copy and tone

D25 removes decorative kickers/taglines from page and brand headings. Retain
functional page/card/action labels. Desktop content should fit without accidental
overflow at ordinary viewport heights; genuine narrow-screen overflow can scroll.

Existing agreed labels/copy:

| Context | Copy |
| --- | --- |
| Landing tiles | Practice; Browse; Random |
| Practice modes | Endless; Fixed |
| Round progression | Next round |
| Summary actions | Play again; Back to setup |
| Empty daily word | No word available |
| Failed-load recovery | Retry |
| Setup exclusions | Excluded words: N · Reset |
| Exclusion recovery prompt | not enough words after exclusions — reset exclusions? |
| Pool-loop hint | pool exhausted, looping |
| Final round-build failure | Not enough distinct words for this filter. Try a different HSK level or reset exclusions. |

Required footer attribution:

> Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed CC BY-SA 4.0.

Link the licence as specified in [DB](../architecture/schema/sources.md) and
retain primary-source notices. Use `HSK 7–9` consistently for the combined
advanced-band level control, matching `HSK 1` through `HSK 6`.

### Copy rules (answered part of UX-COPY01)

Use neutral, instructional English and sentence case. No exclamation marks,
emoji, "Nice work!" or "Oops!". The streak flame is a visual exception.
Pluralize correctly (`1 mistake`, `3 mistakes`); time uses `2m 14s` and ratios
use `16 / 20` as formatting examples, not summary-accounting definitions.

| Context | Copy |
| --- | --- |
| Summary title | Session complete |
| Setup heading | Practice |
| Browse empty | No words matched |
| Browse failure | Couldn't load results; Retry |
| Loading shell | Loading |
| 404 | Page not found; Back to home |
| Pronunciation tooltip | Listen |
| Other summary stats | Mistakes; Time; Highest streak |

Existing agreed strings above remain unchanged. D35 applies First-attempt for
MCQ; matching uses First-attempt correct pairs or its compact First-attempt label.

Traditional hanzi appears as secondary text on match/MCQ surfaces whose primary
label is hanzi. Keep simplified hanzi as that primary label. Do not reveal hanzi
on a pinyin/English-only card as part of this display rule. Existing D12 label
comparison/uniqueness keys, canonical inputs, seeded layout and RNG behavior are
unchanged; traditional text is presentation metadata, never a composite answer
key. The daily card retains its approved two-form field order.

## Accessibility

Pragmatic accessibility (UX-A11Y01): implement the rules below without claiming
formal WCAG conformance. Practice feedback announcements are deliberately
omitted; a future summary canvas must have its numeric stats in adjacent DOM text.

- All actions support keyboard use: Tab, Enter/Space, plus documented shortcuts.
  This includes cards, answers, navigation, theme/drawer toggles, pronunciation,
  GitHub and progression actions. No mouse-only elements.
- Every focusable element has a visible focus ring/outline with at least 3:1
  contrast against its surroundings; never remove outlines without replacement.
- Prefer native buttons, links, nav, main and aside. Avoid redundant roles,
  unnecessary ARIA or tabindex; use tabindex only for a specific focus target.
- Correct/wrong feedback includes visible ✓/✗ alongside green/red, independent
  of hover. This applies to match cards and MCQ answers.
- Text contrast is at least 4.5:1; UI boundaries, focus rings and icon strokes
  at least 3:1. Check muted traditional hanzi, dimmed answers, theme control,
  footer and feedback states in both themes.
- Touch targets are at least 24×24 CSS px or sufficiently spaced for a 24px
  circle not to overlap another target. Aim for 44×44px on mobile controls.
  The footer's icon-only GitHub link must meet the 24×24px minimum.
- Committed route navigation moves focus to the new main or heading and updates
  `document.title` so routes are distinguishable, e.g. "Practice — Zipeek".
- A mobile overlay drawer contains focus with a close action returning it to
  the toggle, or uses an inline/nonmodal arrangement without moving focus.
  Never strand focus, expose obscured content through Tab, or trap the user.
- Retain the existing skip link: focus main without changing the route.
- Keep `<html lang="en">`; standalone hanzi uses `lang="zh-CN"` where practical.
- Search/future inputs need a visible label or aria-label, not only a placeholder.
- Reduced motion disables nonessential motion and keeps semantic colour changes.
  Practice selection, correctness and score updates have no live announcements.

## Pronunciation (D32)

Provide a lucide volume button on the daily card and match cards, and on MCQ
prompts/answers displaying hanzi or pinyin. Tooltip: "Listen". Use native browser
speech synthesis with `zh-CN`; voice quality and availability depend on OS/browser.
No audio files, assets or database fields are added.

Use the explicitly supplied fallback rule: read the simplified hanzi of the word
the card represents, regardless of its displayed label. This is the recorded
assumption pending any later per-surface refinement. A distractor control speaks
its own word rather than the correct answer. Do not promise audible output;
coverage spies/stubs API invocation. See the planned JS interop boundary in
[web-shell](../architecture/modules/web-shell.md#planned-pronunciation-boundary-d32).

## Responsive breakpoints (UX-RESP01)

Supported minimum width is 400px; smaller screens are best-effort without a
refusal screen. There is no maximum content width or centred max-width wrapper:
shell and content fill the viewport, including ultrawide screens. Support
portrait, landscape, pointer, touch and keyboard everywhere.

| Tailwind breakpoint | Width | Role |
| --- | --- | --- |
| sm | 40rem / 640px | Small layouts; drawer begins collapsing |
| md | 48rem / 768px | Mobile/desktop line; desktop rail |
| lg | 64rem / 1024px | Wider content and landing tiles |
| xl | 80rem / 1280px | Browse table and large-desktop layout |

A35 implements the shell md boundary and lg tile layout with Tailwind defaults,
and removes the old 60rem rule. The shell/content stretch to the viewport.
Browse table/card and exercise layouts remain future feature work using the
requirements above; this does not claim those views exist.

A49/D41 supersedes the earlier full-width exercise/stacked-column proposal.
Matching uses two equal columns at all supported widths, preserving shuffled
positions, with equal height per row and space for long labels. The practice
board is centered at max-width 56rem (896px) inside the full-width shell, with
an opaque white/slate background. Cards have solid theme-aware surfaces, larger
hanzi, secondary traditional text and a separate Listen button at their right edge. Match buttons and Listen remain sibling native buttons.

Hanzi-to-English matching cards show display pinyin under hanzi/traditional,
with Listen alongside at the right. Hanzi-to-meaning MCQ prompts retain pinyin
beneath hanzi/traditional and above Listen. This does not add
pinyin hints to pinyin-to-hanzi answers or alter comparison keys, RNG or URLs.
A52 tightens matching cards to an 88px minimum row height with compact padding
and no audio footer. Representative 1280?720 and 1920?720 desktop layouts fit
the full board, action and site footer without vertical scroll. Long labels
may grow rows naturally; narrow/short viewports retain normal page scrolling
without clipping content. The independent Listen target stays at least 44px.
Matched cards briefly flash green, then use a muted solid surface and green
checkmark; preserve text contrast rather than fading the whole card/background.
Reduced motion snaps to that final state. The persistent Next/Continue bar uses
a full-width button on phones and a compact right-aligned button from sm.
MCQ options remain stacked on narrow screens. Browse is a card list below xl
and a table at xl and above, with the five approved fields in UX-BROWSE01.

### Requested navigation and exercise help follow-ups

Recorded 2026-10-29; implementation is tracked separately as A50/A51.

- Drawer navigation items should pair a nearby icon with their existing text.
  Reduce the desktop drawer width from its current 15rem; the final width is
  selected during the implementation plan and checked with labels/focus at
  desktop and narrow sizes. Retain accessible names, active state and collapse.
- Each setup exercise type should have an adjacent info button opening a
  ?How it works? overlay/dialog with a short explanation and representative
  matching/MCQ example for that variant. The info button must not toggle the
  exercise selection, start a session or sit inside another button.
- Give each info button a variant-specific accessible name. Dialogs need a
  visible title and Close action, modal semantics, focus containment, Escape
  dismissal and focus returned to the invoking info button. Examples explain
  the exercise without changing live session data or requiring a new dependency.


## What UX.md does not cover

Database DDL, source licensing mechanics, import algorithms, SQL queries, hash/RNG
internals, seed-byte encoding, dependencies/toolchain versions, build/deployment
steps, and e2e process lifecycle belong in
[DB](../architecture/schema/DB.md),
[ROUND_GENERATION](ROUND_GENERATION.md), [ARCHITECTURE](../architecture/ARCHITECTURE.md),
[TESTING](../testing/TESTING.md), and [AGENTS](../../AGENTS.md). Open-work ordering
and live execution state belong in [TASKS](../process/TASKS.md) and
[CURRENT](../process/CURRENT.md); completed task outcomes go in
[completed](../process/completed/).

## Home dictionary failure (A37)

Keep the daily card's "No word available" plus Retry and all three navigation
tiles visible when the dictionary fails. Data-dependent destinations retain their
own unavailable/Retry view. This is an approved A12 requirement; the current
whole-content dictionary boundary must be adjusted during that implementation.
Do not claim this layout is already implemented.

## Initial motion profile (A37 / D29)

Use 150 ms fade-only route and drawer transitions. Do not add slide or scale.
Query/pagination updates do not animate the whole route. Preserve existing match/
MCQ feedback timing; reduced motion commits movement/fades instantly while
retaining semantic/color feedback. A32 owns implementation and reviews phase
allocation/easing within this profile, live preference changes, interruption and
focus handling. No additional Animato feature or dependency is approved.

## Social metadata and preview image (A37 / D34)

Define and implement social cards and the preview image before v1. Approved
2026-10-28:

| Element | Approved value |
| --- | --- |
| Share title | Zipeek — Chinese hanzi practice |
| Share description | Look up Chinese words and practise hanzi, pinyin, and meanings. |
| Preview canvas | 1200×630 pixels |
| Preview design | Existing landscape, dark slate overlay, Zipeek wordmark and blue accent |
| Scope | One static site-wide image shared by all current hash routes |
| Social formats | Open Graph and Twitter/X large-image cards |

No round-specific word or score appears on the preview. Keep route-specific
browser titles separate from the approved static share copy. Canonical home and
absolute image URLs retain /zipeek/. Metadata belongs in the initial HTML head,
without requiring Yew or database startup; see the [Open Graph protocol](https://ogp.me/).
A40 owns image creation, tags and asset validation. This is an approved design,
not an implemented image or metadata change.

## A11 setup implementation status

The six-tile setup, HSK controls, Endless/Fixed counts and Start action are now
implemented. Initial activity is Hanzi to meaning under the quiz-first priority;
HSK 3.0 level 1 and Endless retain the approved defaults. Fixed starts at 10 and
remembers a chosen count while toggling modes. Changing systems preserves valid
numeric level selections; the advanced band resets to level 1 for HSK 2.0.

Excluded count uses deduplicated saved IDs that exist in the dictionary. Loading
and failed validation disable Start until success; failure offers Retry. Reset
removes saved IDs explicitly, updates the next generated Start URL to omit exclude,
and leaves the user on setup. Failed removal retains the count and displays an
inline error. Loading never rewrites malformed/unknown saved data. Storage read
failure uses an empty local set and logs the failure. No custom count, all-levels,
PoS picker, setup audio or gameplay has been added by A11.
