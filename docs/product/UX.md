# User experience specification

This is the user-facing specification transcribed from the agreed decisions and
[PRODUCT](PRODUCT.md). It describes intended behaviour, not completed features.
D02-D18 are the user-declared resolved baseline; a TBD below identifies a missing
UX decision, not permission to invent one. D20 settles MCQ scope; its UI details
remain open.

## Principles

The user's 2026-10-27 review requests animation/page-transition support using the
already-added Animato dependency.
[UI_ARCHITECTURE](../architecture/UI_ARCHITECTURE.md) proposes the
state/lifecycle/accessibility architecture. Exact motion profiles are still for
review; this does not select durations, easing, movement, new controls or feedback
timing. The existing reduced-motion loading behaviour remains required.

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

Approved in D24/A27: Home, Practice and Browse navigation is shared across
routes, with an active-route indicator. Practice links back to setup, including
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

<!-- TBD: UX-LAND01 - Choose the word fields the daily card displays once data is connected. -->

## Practice setup

- Offer HSK 2.0 levels 1-6 and HSK 3.0 levels 1-6 plus one combined 7-9 band.
  A word's membership can differ between systems. Do not show separate levels 8/9.
- Use the existing normalization defaults: Endless, HSK 3.0, level 1. A malformed
  level never selects all levels.
- Offer Endless and Fixed. Fixed has three count buttons: 5, 10, 20; default 10.
  No custom round-count input in v1.
- Show "Excluded words: N · Reset". Reset clears saved/pending exclusions and
  generates a fresh practice URL without exclusions.
- Initial setup filters are HSK system + level. The URL supports PoS sets;
  a setup PoS picker is future work.

<!-- TBD: UX-SETUP01 — Define selection of the three match variants and the arrangement of setup controls. -->

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

Endless has Next round between rounds and no summary. A running header tally or
streak is permitted. Fixed stops after the selected number of rounds and shows
its summary. Nominal progress advances four words per round; pool exhaustion
loops with the approved hint. Duplicate-label sliding/fallback may cause overlap.

Marking "don't show again" saves a pending exclusion for the next generated URL;
it does not replace current cards or mutate the current URL mid-round. A loaded
URL's exclusions are authoritative even when the browser has other saved exclusions.

<!-- TBD: UX-MATCH01 — Define card selection/pairing interaction, selected/correct/incorrect feedback, retry or locking behaviour, and when Next round is enabled. -->

<!-- TBD: UX-MATCH02 — Decide whether to show the permitted Endless header tally/streak and define the per-round progress display. -->

## MCQ exercise

D20 approves three variants at `/practice/mcq/<variant>`: `hanzi-to-meaning`,
`meaning-to-hanzi`, and `meaning-to-pinyin`. Use simplified hanzi, the first
English gloss, and display pinyin for the corresponding prompt/answer labels.
Each question offers three options: one correct answer and two distractors
(N=2). Apply D02/D12 pool widening and distinct visible-answer rules.

Both Endless and Fixed are in scope. Fixed offers 5/10/20 questions, default
10. Advance offset by one per question; keep Fixed progress in memory separately
from offset wrapping. These are approved requirements, not implemented routes.

<!-- TBD: UX-MCQ01 — Define answer feedback timing, number-key hints, and answer-button styling within the approved D20 scope. -->

## Summary screen

After Fixed round N, show correct pairs, total mistakes, and time taken, with
Play again and Back to setup. Play again resets the pool offset to zero.
Endless does not show a summary.

Fixed round progress is in memory only. Reload resets progress while honoring
the URL's pool offset; wrapping that offset does not reset the session counter.

<!-- TBD: UX-SUM01 — Define correct-pair and mistake accounting, timer start/end/pause rules, and summary presentation. -->

## Browse

Provide paginated word query with the agreed hanzi, pinyin, English, and
part-of-speech query purpose. Pinyin input supports both display and numeric forms.
Canonical PoS tokens come from the agreed mapping. Unknown query values follow
the safe parsing policy rather than crashing the page.

<!-- TBD: UX-BROWSE01 — Define result fields, query controls and matching behaviour, HSK filter exposure, result ordering, page size, and pagination controls. -->

<!-- TBD: UX-BROWSE02 — Define browse-specific loading, empty-result and query-failure presentation, copy, and retry behaviour. -->

## Theme and dark mode

- Dark is the preferred default when there is no saved choice. Offer a dark/light
  toggle in the top panel at the top-right (D25), and remember the selection across reloads.
- A saved light choice overrides the dark default. The loading shell, Yew loading
  view, and ready app use the same choice without a light flash.
- Anchor a GitHub icon at the bottom of the left sidebar, linked to
  https://github.com/Jozefpodlecki/zipeek, with an accessible repository name.
- Show Jozef Podlecki's copyright with the current year in the footer, alongside
  the required dictionary attribution. Example: "© 2026 Jozef Podlecki".

Approved direction (D24/A27): neutral slate dark/light with blue accents.
D25 amends the placement: a permanent top panel contains brand, drawer toggle
and theme control at the top-right. Home/Practice/Browse and the bottom repository
link remain in an expandable/collapsible drawer. Narrow layouts show the expanded
navigation below the top panel; the collapsed drawer is hidden and content uses
the available width. Copyright/attribution remain visible in both themes.
Native buttons/links support keyboard use; the theme switch exposes its state.
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
retain primary-source notices. The advanced band may be labelled "HSK 7–9" or
"Advanced (7–9)" under the prior decision.

<!-- TBD: UX-COPY01 — Define the overall copy/tone, remaining help/status/error text, and the final advanced-band display label. -->

## Accessibility

The repository icon link must have an accessible name identifying the repository.
Other interaction/accessibility details have not been specified.

<!-- TBD: UX-A11Y01 — Choose an accessibility target and define keyboard/focus behaviour, screen-reader labels and announcements, and non-colour correctness/error cues for the app. -->

## Responsive breakpoints

D24/A27 approves the desktop-rail/mobile-header direction. The shell implementation
uses a compact top panel and optional inline navigation through 48rem, a left
drawer above it, stacked landing tiles through 60rem and three columns above it. Chrome checks 360, 768, 1024 and 1280px
widths without horizontal overflow. These are shell implementation/test sizes,
not a complete supported-device or exercise-layout policy.

<!-- TBD: UX-RESP01 - Define the full supported-device/viewport policy and narrow-screen adaptations for the future practice and browse content. -->

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