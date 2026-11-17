> **R507 saved Practice recovery (2026-11-16):** Missing or invalid saved Practice sessions display a focused recovery view rather than debug enum values. Returning to Practice setup is always available; Retry is shown only for potentially transient failures. Recovery never silently removes browser data. A previously saved session may be incompatible after options-schema changes, without a migration or fallback reader.

> **Current approved target behavior:** [Practice outcomes](../../learning-practice/practice/practice-outcomes.md), [interface policy](../../product-ux/foundation/interface-and-platform-policy.md), [vocabulary selection](../../dictionary-data/vocabulary/vocabulary-selection.md), and [AI Hanzi recognition](../../learning-practice/hanzi/ai-hanzi-recognition.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


> **R504 Practice scroll constraint (2026-11-16):** The desktop Practice Exercise stage requires an explicit viewport-based height; relying solely on a percentage height within auto-sized flex ancestors can prevent the inner list from becoming scrollable. The tile/Configure list is the one right-edge scroll region beneath Choose/Configure, with the stepper, Back/Continue and tabs outside it. The actual browser still requires acceptance at short viewports. Mobile uses natural page scrolling.

> **R518 Word Sets workspace (2026-11-16):** The personal Word Sets manager presents a visible, keyboard-navigable list of named sets and a single, document-scrolling vocabulary workspace. No nested desktop word-list scroller remains. Search filters the current set in-memory by Hanzi, pinyin and English meaning and resets on switching sets. New set keeps a labelled primary action; selected-set rename and deletion are compact, discoverable controls, with explicit two-step deletion. Backup and merge are retained under Data tools. The selection remains an in-memory management choice and does not set a Practice default. No storage or provider schema changes. Rust, WASM and Chrome acceptance pending.

> **R501 Word Sets UX (2026-11-16):** The Word Sets page shows the currently viewed set by name in a custom keyboard-accessible menu, with an explicit distinction from Practice vocabulary selection. The right vocabulary panel owns a bounded desktop scrollbar; mobile continues natural document scrolling. Word removal is an icon button with an accessible name, and Dictionary navigation appears contextually for an empty set. The global focus presentation uses a lighter visible keyboard shadow instead of the former thick outline. Saved Word Sets persist in localStorage, but the viewed selection itself is not a separately persisted preference.

> **R502 Practice setup (2026-11-16):** On desktop the Choose/Configure exercise list has its own right-edge vertical scrollbar below the fixed wizard navigation and stage tabs. The outer Practice main area does not scroll while selecting exercises. On mobile, keep natural page scrolling. AI labels appear on Sentence Translation, Fill in the Blank, Sentence Builder, Sentence Transformation and Spot the Mistake; Picture Vocabulary displays AI plus Image generation. These are capability labels and do not certify that all locally authored exercise implementations use live AI generation. The speaking exercise is titled Pronunciation. The verbose selected/total announcement remains removed.

> **R485 focused course lessons (2026-11-16):** Grammar and Pinyin use icon-only accessible exit/navigation controls and reset their lesson scroll position after advancing. Long Pinyin sound inventories are in a disclosure. Grammar headings are compact; optional AI examples are available only when the actual AI service is reachable and never count as assessment. New Grammar courses 6-20 expose five teaching patterns and a guided matching review. See [Courses](../../learning-practice/courses/courses-mosaic.md).

> **R482 Courses UX (2026-11-16):** Course tabs show names only. `/courses` redirects to Vocabulary, Grammar has `/courses/grammar`, and all three tracks share the same desktop mosaic/preview/details proportions. The Vocabulary Explorer has a stable height across states, with no repeated region summary below the mosaic. Desktop avoids document scrolling; mobile scrolls naturally. See [Courses](../../learning-practice/courses/courses-mosaic.md).


> **Vocabulary course layout (R477 WIP, 2026-11-16):** The mosaic selects a theme, the right panel owns acquisition and region details, and a paginated four-card Vocabulary Explorer occupies the space beneath the mosaic. Explorer cards show Hanzi, pinyin, meaning and an existing Listen action. No repeated totals or acquisition controls appear below the mosaic. Desktop should fit the viewport without extra vertical scrolling; mobile permits normal page scrolling. Build and browser acceptance remain pending.

> **Vocabulary UX update (2026-11-15, R471 WIP):** The independent 100-region Vocabulary mosaic exposes domain/theme titles, HSK band counts, a 30-word preview and one or more Acquire Word Set actions per region. Larger regions are partitioned into 200-word Word Sets. Acquired sets are indicated individually and remain independent of proficiency. See [Courses](../../learning-practice/courses/courses-mosaic.md).

> **Courses experience (2026-11-15):** Grammar, Vocabulary and Pinyin are separate mosaics. Grammar renders Grammar-only regions, with no Vocabulary pockets. The detail panel uses one Start course action. Dedicated course lessons and assessment routes omit the global drawer, header and footer and provide their own exit/navigation. Grammar explanation, checks, guided bank and assessment entry are staged, not presented as one continuous document. See [Courses](../../learning-practice/courses/courses-mosaic.md).

# User experience specification

**Reading convention:** dated A/R/D sections preserve prior UX decisions and context; later “current” sections and the source take precedence for implemented behavior. Older screen designs (especially the R54 translation bank) must not be treated as an active code contract. See [CURRENT](../../project-management/current.md).

This is the user-facing specification transcribed from the agreed decisions and
[PRODUCT](../../product-ux/product.md). It describes intended behaviour, not completed features.
D02-D18 are the user-declared resolved baseline; a TBD below identifies a missing
UX decision, not permission to invent one. D20 settles MCQ scope. A22 records
answered UX requirements on 2026-10-27.
Unresolved choices remain explicit below; these requirements do not imply that
the corresponding UI or tests are implemented.

## Principles

The user's 2026-10-27 review requests animation/page-transition support using the
already-added Animato dependency.
[UI_ARCHITECTURE](../../architecture/ui-architecture.md) proposes the
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
`HSK 3 · 7-9` for the advanced band. Empty/load-failure behaviour remains D05.

## Practice setup

Setup presentation and interaction requirements have been separated for easier maintenance. See [Ux Practice Setup](../../product-ux/practice/ux-practice-setup.md) for the full maintained section.

### Setup layout (UX-SETUP01)

Moved to [Ux Practice Setup](../../product-ux/practice/ux-practice-setup.md).

## Match exercise

Matching exercise requirements have been separated for easier maintenance. See [Ux Match](../../product-ux/practice/ux-match.md) for the full maintained section.

### Match interaction (UX-MATCH01)

Moved to [Ux Match](../../product-ux/practice/ux-match.md).

### Practice header (UX-MATCH02)

Moved to [Ux Match](../../product-ux/practice/ux-match.md).

## MCQ exercise

Multiple choice exercise requirements have been separated for easier maintenance. See [Ux Mcq](../../product-ux/practice/ux-mcq.md) for the full maintained section.

### MCQ interaction (UX-MCQ01)

Moved to [Ux Mcq](../../product-ux/practice/ux-mcq.md).

## Summary screen

Completion summary and score presentation requirements have been separated for easier maintenance. See [Ux Summary](../../product-ux/practice/ux-summary.md) for the full maintained section.

## Browse

Dictionary browsing interactions have been separated for easier maintenance. See [Ux Browse](../../product-ux/pages/ux-browse.md) for the full maintained section.

### Search and filters (UX-BROWSE01)

Moved to [Ux Browse](../../product-ux/pages/ux-browse.md).

### Browse URL state

Moved to [Ux Browse](../../product-ux/pages/ux-browse.md).

### Browse fields (UX-BROWSE01, resolved A37 and amended by D45)

Moved to [Ux Browse](../../product-ux/pages/ux-browse.md).

### Browse states (UX-BROWSE02)

Moved to [Ux Browse](../../product-ux/pages/ux-browse.md).

## Theme and dark mode

Application appearance requirements have been separated for easier maintenance. See [Ux Appearance](../../product-ux/foundation/ux-appearance.md) for the full maintained section.

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
| Exclusion recovery prompt | not enough words after exclusions - reset exclusions? |
| Pool-loop hint | pool exhausted, looping |
| Final round-build failure | Not enough distinct words for this filter. Try a different HSK level or reset exclusions. |

Required footer attribution:

> Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed CC BY-SA 4.0.

Link the licence as specified in [DB](../../dictionary-data/schema/sources.md) and
retain primary-source notices. Use `HSK 7-9` consistently for the combined
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

Accessibility requirements have been separated for easier maintenance. See [Ux Accessibility](../../product-ux/foundation/ux-accessibility.md) for the full maintained section.

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
[web-shell](../../architecture/modules/web-shell.md#planned-pronunciation-boundary-d32).

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
| xl | 80rem / 1280px | Browse results and Word Set panel side by side |

A35 implements the shell md boundary and lg tile layout with Tailwind defaults,
and removes the old 60rem rule. The shell/content stretch to the viewport.
Browse's hybrid result cards, filter chips, URL state, and responsive Word Set
panel are the target layout. This requirement does not assert that all target
interactions have been implemented.

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
MCQ options remain stacked on narrow screens. Browse retains vocabulary cards at all breakpoints, with a separate Word Set
management panel alongside the results on wide screens and stacked or disclosed
below them on smaller screens. Cards show the five approved fields in
UX-BROWSE01.

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
[DB](../../dictionary-data/schema/db.md),
[ROUND_GENERATION](../../games/round-generation.md), [ARCHITECTURE](../../architecture/architecture.md),
[TESTING](../../testing-quality/acceptance/testing.md), and [AGENTS](../../../AGENTS.md). Open-work ordering
and live execution state belong in [TASKS](../../project-management/tasks.md) and
[CURRENT](../../project-management/current.md); completed task outcomes go in
[completed](../completed/).

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
| Share title | Zipeek - Chinese hanzi practice |
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


## Future feedback, settings and exercise directions (A53)

The user requests removing matching success checkmarks, avoiding duplicate simplified hanzi in a matching round, and a Settings view with theme selection plus an option to
hide Listen icons and speak on word/card click. Pronunciation and
three-option listening are separate experimental exercises, independently
toggleable in Settings and subject to eligibility review. Translate sentence
uses a useful subset of 20 random candidate words and AI generation.
See [FUTURE](FUTURE.md) for the source-data example, full requests and open
interaction choices. A54-A60 own future work; current feedback/theme/audio
controls and the six implemented exercise variants are unchanged by these notes.

Sentence translation remains experimental and requires AI connectivity configured in Settings, which owns the endpoint, model and optional in-memory key. The Advanced Practice flow owns the translation direction, vocabulary source and question count. The exercise view contains only learning controls, word-bank tiles, feedback and progress, and links learners to Settings when no model is configured. The Test connection action lives in Settings. The historical proposal is documented under [FUTURE connection details](FUTURE.md#ai-connection-in-settings).


### Future Practice setup filters (D44 / A61)

Add a richer category/PoS filter section alongside HSK controls. Multiple
selected values use OR within their group and AND across category, PoS and HSK
groups; no selection in a group means no restriction from that group. Example:
(medical OR technical) AND (noun OR verb) AND the selected HSK level.
Searchable multi-select controls, active chips, Clear filters and pool count
are proposed for implementation review; the precise layout is not yet chosen.
See [requirements and dependencies](FUTURE.md#practice-setup-category-and-part-of-speech-filters).
Current setup has no category or PoS picker; category data awaits A56.

### Sentence Translation: current generated Practice behavior

The former R54 model-authored word bank/answer contract has been replaced. Settings owns the AI endpoint and model, Practice setup chooses the vocabulary source, direction and question count, and translation-only Advanced Practice uses a persisted UUID run. The model returns **Chinese and English sentences only**; Zipeek derives canonical answer tiles, including occurrence-aware duplicates, and exposes deterministic DOM order attributes for tests. AI evaluation determines the scored outcome. Finite and Endless progress is handled by the Practice record; completion removes the temporary record, and the results view warns that no durable learning history exists. The standalone legacy translation route and the public free-text translator remain distinct. See [Sentence Translation](../../learning-practice/translation/translate-sentence.md), [Practice builder](../../learning-practice/practice/practice-session-builder.md), and [current state](../../project-management/current.md).

## Appearance backgrounds (R245)

Settings > Appearance offers six selectable scenic theme presets with image
previews and three palette swatches: Lavender Classic (wisteria mountain lake),
Jade Light (bamboo valley), Sunset Warm (sunset pavilion), Midnight Blue
(moonlit lake), Ink Minimal (ink-wash mist) and Autumn Maple (golden trees and
stone bridge). Selecting a preset updates the persistent application backdrop
and palette immediately, independently of the Light/Dark display mode. The
first-run preset is Lavender Classic; saved valid selections override it.

The backdrop is a decorative image with a contrast overlay and no focus target.
A compact, preloaded WebP is embedded in HTML for Lavender first paint, while
other optimized WebP files are loaded only when selected. The browser startup
script restores saved preferences before the main application renders. All
assets remain local to the installation and work beneath `/zipeek/`.

## Practice, Browse and appearance interaction references

- Path card selection advances directly to Exercises. Path has no Continue footer.
  Exercises Continue goes directly to Vocabulary; Configure remains optional.
- Supported standalone practice playlists allow multiple selected cards and run
  their distinct routes in deterministic order. A completed finite mode provides
  continuation to the next selected exercise or an explicit completion action.
- Browse word rows place pronunciation beside pinyin, with per-row accessible
  Show all meanings and Show fewer meanings controls. Keep dictionary senses intact;
  do not truncate the stored data. Pagination identifies the current page and
  result count on that page without inventing a total result count.
- Background overlays are preset-dependent in light and dark display modes.
  Preserve visible landscape artwork while retaining contrast on translucent
  surfaces. Verify both display modes at desktop and mobile widths, including
  focus outlines and hover controls.

### Handwriting Recall (R263)

Practice setup offers Handwriting Recall as the fifteenth type. A blank dark drawing board presents an English meaning and pinyin; the hidden Hanzi is not rendered in the UI before reveal. Pointer drawing works through the shared writing surface, with Undo stroke and Clear. Reveal locks drawing and shows the reference beneath the learner's strokes plus a textual answer. Continue advances the browser-local session. The interaction is explicitly self-reviewed and unscored, and refreshing resets in-progress ink and reveal state without discarding the session. Provide manual touch, pen and resize acceptance coverage before considering the feature verified.

## Practice setup visual refinements (user review, 2026-11-08)

- Keep the four-step progress rail, and place Back/Continue consistently in a compact action bar above the step contents. On Summary show `Start` followed by the play icon; omit the redundant review heading. Do not add an `(optional)` suffix to Configure.
- Exercise selection follows the supplied moonlit card/chip mockup direction. The selected count is derived from the draft, with accessible card pressed state and clearly visible selected indicators. All 15 types remain selectable.
- Vocabulary category selection should provide directly selectable categories rather than requiring datalist typing. A text field may narrow the list; accessible focus remains visible without an oversized ring. Selected categories remain removable and appear in Summary.
- Summary must show exercise icons and prominent selected vocabulary filters rather than a bare list of names; descriptions of unscored authored content must remain truthful.
- Loading should be visually centered within its hosting context, larger, and say what is being loaded. The initial loader currently has static detail; future work should use application phase-specific progress. Route/page transitions should use the already approved Animato dependency with reduced-motion fallback once the animation work is approved and implemented.

## Unfinished Practice reminder and Hanzi presentation

Leaving a persisted local UUID Practice session for Settings or another normal
app page keeps its validated saved state. A small fixed panel in the lower-right
corner offers **Resume practice**, **End session**, and dismissal. Resume returns
to the same session ID; End session discards its saved record. The panel is not
shown on the session route, and it is not a modal: the destination remains usable.
Legacy in-memory Matching and MCQ sessions retain their existing leave/reset
semantics; the reminder does not claim to resume those sessions.

The Chinese Characters settings also control Character Builder Hanzi tile size
(Standard, Large, Extra large), stored in the existing preferences JSON. Tiles
share a common user-selected Hanzi font and size, while pinyin labels remain
separately marked. Font options require local installation; their selection
and persistence does not guarantee the font is installed.

## Learning indicator and additional browser tabs (R421 WIP)

During first-visit onboarding, the learner chooses a theme and may separately opt in to saving completed Practice outcomes on this device. The recording checkbox starts unchecked when no previous consent exists. It saves as `learning_history_enabled` in the existing `zipeek.preferences.v1` document, shared with Settings and Practice completion. The old separate consent key is ignored and old opt-in choices are not migrated. The choice can be changed under Settings → Learning Data and does not claim assessed mastery or general pedagogical tracking.

When the user enables recording completed Practice outcomes in Learning Data, an inactive dragon emblem becomes visible in the global top bar. It animates into view subtly, without suggesting mastery; reduced-motion preferences suppress the entrance animation. Disabling recording hides it. The preference is persistent across reloads and navigation.

In browser mode, opening a second Zipeek tab in the same origin displays an unobtrusive informational warning about concurrent learning-data writes. Presence is detected through browser messaging with periodic expiry of stale peers. This warning does not constitute exclusive database-writer ownership. A real exclusive OPFS writer lock is still required before advertising safe concurrent sessions.

Practice shares the standard AppShell header and drawer, including during its setup and sessions.

## R437 Courses track navigation

Courses provides separate Vocabulary, Grammar and Pinyin routes and tabs. Vocabulary has 100 selectable irregular polygon regions generated from deterministically scattered reading-weighted sites. Small regions remain accessible while large groups occupy more space, without rigid rectangular grid lines. The details panel leads with the selected theme and compact acquisition controls; Word Set listings, vocabulary previews and HSK band breakdowns are expandable on demand, rather than constantly consuming panel height. All 100 regions expose named, machine-classified curriculum memberships and acquirable Word Sets; semantic editorial review and runtime acceptance remain outstanding. Selecting or acquiring sets does not count as proficiency evidence. Grammar retains its separate curved 100-region mosaic and Pinyin retains six independent lessons.

## R446 first-run and Home presentation

First-run theme selection previews the background and palette immediately but does not persist until Start learning atomically saves appearance and Practice outcome recording consent. Leaving onboarding with Back or an uncommitted unmount restores the previous preset. Theme cards provide keyboard-visible focus, selected-state feedback and reduced-motion-aware transitions. On Home, the Word of the Day heading and spotlight use one aligned content width, while the meaning navigator stays compact for short definitions and wraps longer definitions. This is a source-level design implementation pending Chrome verification.

## R448 Practice setup and session presentation (2026-11-15)

Practice setup presents unselected exercise types until the learner chooses. Back/Continue and the Choose/Configure tabs remain visible while the grid scrolls independently. Wizard progress uses filled circles for visited steps, a differentiated filled current step, and outlined upcoming steps, without completion checkmarks. Matching settings select 2-7 pairs; four-option MCQ is unaffected. In the persisted Practice session the global app chrome is absent and a minimal Leave session action returns to setup without deleting the temporary record. Matching highlights the active tile, distinguishes wrong pairs and matched pairs, and uses the configured Hanzi font. This current session-only interaction supersedes the older standalone four-pair layout prose above; older direct-route Matching and MCQ descriptions are historical.

## Current session presentation: R457 refinement (2026-11-15)

During a persisted Practice session, the global navigation/header/footer remain absent and the selected theme wallpaper remains behind the content. A natural-proportion illustrated Chinese dragon occupies the left rail on capable viewports, its coloured artwork gradually revealing upward as saved exercises finish. Animato supplies interpolated reveal values; no independent mastery or completion state is inferred. A screen-reader progressbar reports the actual completed steps; reduced-motion users see the authoritative target without a reveal animation. The Leave link is an icon-only arrow with an accessible text label and does not delete the session. MCQ feedback distinguishes correct/incorrect options by readable contrast, not redundant check/X symbols.

## Current Practice session creature progress (R458)

The fullscreen Practice session has one left-side vertical progress rail. It uses the existing saved-session step position, fills its illustration from bottom to top with Animato, and never represents course mastery. Appearance presets select distinct themed creatures: Lavender Classic dragon, Jade Light qilin, Sunset Warm phoenix, Midnight Blue white tiger, Ink Minimal crane and Autumn Maple deer. All share the same layout, accessible progressbar and reduced-motion presentation. Matching and Multiple Choice sessions show the prompt and controls without a redundant visible exercise-type heading; their container retains an accessible label.

## Current Practice creature presentation (R461)

The Practice session owns one theme-selected illustrated progress component. Dragon, Qilin, Phoenix and Crane stand to the left of the exercise, filling bottom to top. The Deer and White Tiger are horizontal beneath the centred exercise, filling left to right. These layouts share the same persisted session progress and Animato interpolation; orientation is presentation-only, not a separate form of mastery. On small viewports the exercise content may scroll internally, but the document and page chrome must not overflow the fullscreen shell. The current and completed progress remain accessible through the semantic progressbar without visible exercise-count labels. Reduced-motion preferences show the current fill immediately.

## Practice creature illumination (R462)

The progress creature is a muted full illustration with a gently illuminated portion derived exclusively from persisted session-step completion. Portrait creatures illuminate from bottom to top; landscape creatures illuminate from left to right. A soft gradient feather replaces hard clipping, and the earlier vertical centre ornament is removed. Animato interpolates progress transitions; a brief finish flourish occurs once at full completion. Reduced-motion presentation uses the final target immediately. No extra progress data, course mastery or new storage key is introduced.

> **Vocabulary view refinement (R476 WIP):** Acquired course Word Sets use explicit `(region_id, set_index)` identities stored in browser-local personal Word Sets, not name equality. The selected-region panel keeps only theme, collection counts, progress and the next-acquire action; redundant Word Sets navigation, expanded sample vocabulary and HSK badges are omitted. The desktop view is viewport-bounded with no repeated dashboard or footer; mobile can scroll as needed.

> **R478 WIP Vocabulary layout/state refinement:** The Vocabulary Explorer remains below the mosaic and does not reproduce the selected-region details or acquisition controls. On desktop, the mosaic SVG fills the allocated height (the background art uses cover cropping and the complete interactive region overlay remains visible) and the Explorer is bounded to four cards per page, while the right panel displays only theme and acquisition information. On smaller viewports, the page scrolls naturally. Region change and retry reset the Explorer to its first page. Loading and acquisition feedback must remain associated with their originating region; cached dictionary content may be reused for up to four regions. Browser verification at supported sizes is pending.

> **R479 WIP Courses layout:** The Vocabulary map covers its allotted height without letterboxing. The right panel uses spare space for a compact, source-backed acquired/available Word Set roadmap below its primary Acquire action; the bottom Vocabulary Explorer remains the only place for word cards, Pinyin, meanings and listening. At narrower desktop widths the four-card Explorer uses a two-column layout to avoid tiny cards. The desktop page must not gain a vertical scrollbar, while mobile can scroll naturally. Browser acceptance is pending.

> **R480 WIP Vocabulary loading state:** Keep the mosaic and region metadata visible while the shared dictionary initializes. Use distinct accessible loading messages for dictionary initialization and selected-region vocabulary lookup. Automatically load the selected region when the database becomes Ready; do not show Retry during normal startup. Only genuine initialization/query failures expose appropriately scoped Retry actions. Preserve selection and prevent stale words or acquisitions during transitions. Browser validation remains pending.

> **R481 WIP Vocabulary loading presentation:** Show an accessible spinner plus a compact acquisition-roadmap skeleton in the region panel, and four noninteractive skeleton cards in the Vocabulary Explorer. Use reduced-motion-safe animation and an accessible phase label, rather than a sentence-only placeholder. Preserve selected-region metadata, never display fabricated progress, and reserve the existing bounded panel space to avoid desktop layout shifts.

## R483 Courses mosaic visual refinement

Vocabulary and Grammar keep distinct 100-region datasets and independent selection state, but share a restrained curved edge treatment. Vocabulary sizes continue to reflect HSK reading counts; Grammar uses a balanced scattered layout rather than bulging warped cells. The underlying illustration remains visible through a consistent dim cover, with highlighted selected outlines. Dimmed artwork is a visual preview, not verified lesson completion. Never add icons inside SVG mosaic regions. Maintain keyboard focus and accessible descriptive names, and verify seams, readability and touch sizes in Chrome.

## R484 Pinyin and Grammar lesson depth (2026-11-16)

Six Pinyin mosaic stages each open two authored modules. The details panel lists both modules with their saved written-check status; the compact lower panel offers browser voice playback without implying audio assessment. Each fullscreen Pinyin module includes reference material where appropriate, playable Hanzi examples, two graded written questions and a clear retry path. Passing the written check persists across reloads without an icon inside the mosaic or a proficiency claim. Grammar lessons reopen at a saved checkpoint and display playable examples; the Grammar overview offers Continue when a checkpoint exists and shows the best independent draft assessment score without granting mastery. The focused lesson shell and the three name-only track tabs remain unchanged.

## R488 Grammar teaching and guided review

All 100 Grammar mosaic regions open authored draft teaching lessons, not placeholders. Expanded courses use focused teaching cards followed by one guided meaning-recognition question per point. The active review question occupies a single panel and the Continue action is disabled until answered correctly. The original five courses' sentence builders conceal the reference answer unless explicitly revealed; checking a submitted answer also displays teaching feedback and the correct example. No guided activity grants course mastery or reveals a mosaic piece. The authored corpus has not yet received independent Mandarin/HSK review; do not imply certification.

## R494 Practice setup scrolling

Superseded by R502: the desktop exercise selection and configuration list itself scrolls inside the widget, with wizard navigation and Choose/Configure tabs outside the scroll region. Only the list owns vertical scrolling while Exercises is active. Mobile remains naturally page-scrollable.

### Practice exercise selector scroll contract (R503 WIP)

On desktop the progress stepper, Back/Continue and Choose/Configure tabs stay visible while the tile grid or exercise settings scroll inside one bounded region below the tabs. The main Practice viewport must not be a competing scroll owner. On mobile, scrolling follows the natural page. When no exercise is selected, Continue is disabled without an additional redundant selection warning.


## R517 WIP: Listening choice hints and audio control

In local Practice sessions, Listening Choice presents Mandarin playback as a large, keyboard-accessible speaker control above three equal-weight word choices. Setup independently configures pinyin and English meaning visibility as Never, After answering (default), or Always; it also offers optional automatic playback (off by default). Disclosure applies uniformly to all three choices, preventing the answer from being identified from a single hint. Correct/incorrect feedback and Continue share a reserved-height area to reduce layout shift. The activity remains practice-only, not scored evidence. If autoplay is blocked by browser speech restrictions, manual replay remains available and an error is announced. The exact settings are preserved in the authored Practice profile and persisted session configuration. The standalone Listening Choice route has separate controls and is unchanged.
