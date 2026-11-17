# Archived source snapshot: product-ux/product.md

This source snapshot preserves historical descriptions, not current requirements. Use the maintained document at its original location for approved behavior.

> **Courses routes (R482 WIP):** The three independent learning tracks have dedicated URLs `/courses/vocabulary`, `/courses/grammar`, and `/courses/pinyin`; `/courses` redirects to Vocabulary. The compact tab navigation shows track names only. See [Courses](../../../learning-practice/courses/courses-mosaic.md).

> **Current approved target behavior:** [Practice outcomes](../../../learning-practice/practice/practice-outcomes.md), [vocabulary selection and exclusions](../../../dictionary-data/vocabulary/vocabulary-selection.md), [profile portability](../../../learning-practice/practice/practice-profile-portability.md), [AI Hanzi recognition](../../../learning-practice/hanzi/ai-hanzi-recognition.md), and [platform policy](../../../product-ux/foundation/interface-and-platform-policy.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


> **Vocabulary curriculum update (2026-11-15, R471 WIP):** The Vocabulary track now has 100 separate domain/theme/word-class regions covering all 11,494 pinned HSK source records, with 132 independently acquirable Word Sets of up to 200 words. Source-entry coverage is audited; semantic classification is glossary-assisted and still needs human review. Acquiring a set is not mastery. See [Courses](../../../learning-practice/courses/courses-mosaic.md) and [coverage report](../../../learning-practice/VOCABULARY_COVERAGE.json).
> **Vocabulary mosaic sizing (R472 WIP):** All 100 Vocabulary-only tiles use deterministic area-proportional geometry based on the count of distinct acquirable HSK readings in each region. Small themed sets occupy small map areas, not equal-sized grid squares. This is unrelated to mastery progress; Grammar and Pinyin geometry do not change. Compiled and visual acceptance are pending.

> **Vocabulary mosaic refinement (R473 WIP):** Reading counts still determine relative region size, but a square-root scale and minimum area replace strict proportionality. The 100 regions are spatially scattered; the panel omits internal region numbering and algorithm descriptions. Verification pending.

> **Courses update (2026-11-15):** The current requirement is **three independent tracks**. Grammar mosaic contains 100 Grammar regions **only**; Vocabulary has its own 100-region mosaic, not pockets within Grammar; Pinyin has a separate six-region mosaic. The R355/R356 descriptions below are historical and their 100+50 combined topology is superseded. Grammar now has a single Start course route and staged lesson workspace without the global application shell. See [current Courses specification](../../../learning-practice/courses/courses-mosaic.md) for verified scope and pending completion evidence.

## R358 Courses third guided grammar lesson

The Courses mosaic now links three authored guided grammar regions. Course 3, **Possession, Existence & Quantities**, teaches 有, 没有, counting with classifiers (本 and 个), 几 and 多少. Five comprehension checks and six guided sentence-building/transformation activities have reference answers and explanations. They do not independently assess mastery or unlock mosaic artwork. The remaining 97 grammar regions and all 50 vocabulary pockets await authored syllabuses and verified proficiency. The existing first two courses and three Pinyin lessons remain available.

## R357 Courses second guided grammar lesson

The Courses mosaic now links two authored guided grammar regions. Course 2, **Time, Place & Everyday Actions**, teaches when and where actions happen: 今天 / 明天 before a verb, 在 + location before an action, combinations of time and place, 去 + destination, and 在哪里 questions. Five knowledge checks and six guided sentence-bank activities provide teaching feedback with answers visible; they are not independent assessments. Course 1 remains available with its four checks and six guided bank activities. The remaining 98 grammar regions have not been authored, and all 150 grammar/vocabulary regions remain covered pending separately verified durable completion and word proficiency.

# Product specification

This document records product intent and earlier approved D-number contracts; **current source and the [source reconciliation](../SOURCE_RECONCILIATION.md) take precedence for implemented behavior when older contracts have been superseded.** For implemented behavior and known gaps consult [CURRENT](../../../project-management/current.md), and for actual route definitions consult `crates/zipeek-web/src/route.rs`.
See [ARCHITECTURE](../../../architecture/architecture.md) for the current implementation
and [DECISIONS](../../../project-management/decisions/index.md) for choices still requiring approval.

## R356 Courses guided grammar practice (no assessed completion)

The Basic Sentence Structure introductory lesson now contains six authored word-bank teaching tasks in addition to its four existing explanation/comprehension checks: three Sentence Builder prompts and three Sentence Transformations. Learners can construct a sentence by choosing individual Hanzi/word tiles, remove individual tiles, undo and reset, see a *visible* reference answer throughout the guided stage, receive pattern-specific feedback and repeat all tasks. This is an instructional flow, not evidence of unassisted grammar mastery. It does not save completion, write pedagogical records or reveal any mosaic artwork. The remaining 99 grammar regions and all 50 vocabulary groups still require authored syllabus mappings and independent assessment evidence.

## Courses and proficiency roadmap (R355 mosaic foundation, assessments pending)

R355 changes Courses from a catalogue to an interactive 100-grammar/50-vocabulary mosaic with an original illustration and a genuine first guided grammar lesson. Existing three Pinyin lessons remain available, with local completion separate from mosaic progression. **This is not yet the assessed grammar-mastery platform**: the other grammar courses are unauthored, all vocabulary group memberships are pending, every region remains concealed, and no completion evidence is inferred from selecting/reading a course. The [approved Courses mosaic specification](../../../learning-practice/courses/courses-mosaic.md) defines 100 large grammar regions and 50 tiny vocabulary regions in one contiguous image, course-specific assessed grammar-point progression, and irreversible completed-course reveals. Vocabulary-region opacity depends on real distinct-word mastery and remains independent of grammar completion; never substitute the current marked-complete stub.

The future [pedagogical history policy](../../../learning-practice/practice/learning-evidence.md) records Practice sessions first and derives per-Hanzi, per-exercise, category, HSK and grammar evidence. Proficiency requires seven distinct active local dates, at least ten qualifying scored encounters across three or more sessions (subject to a per-session cap), and independent successes in recognition, listening, assessed speaking and sentence use. Matching-only repetition cannot establish proficiency. Exact per-skill thresholds, cap and retention policy still require specification; nothing in the existing local completion store provides this automatically.

## Purpose

Query Chinese words by hanzi, pinyin, English, and part of speech, and practise
them through matching and multiple-choice exercises. The app runs client-side
on GitHub Pages under `/zipeek/` and reads a pregenerated DuckDB database.

The original D07 design limited the pool to the union of HSK 2.0 and HSK 3.0. The **current generator** additionally imports valid CC-CEDICT-only readings, and the checked-in manifest reports **12,188 rows**. HSK-filtered exercise pools still use HSK membership, while an unfiltered dictionary pool can include additional entries. See [installed database](../../../dictionary-data/schema/db.md) and [source rules](../../../dictionary-data/schema/sources.md).

## Startup loading experience

Database-owned Loading/Ready/Failed+Retry is implemented. Ready requires a queryable read-only database; Browse, Random, daily-word content and exercises are implemented separately and must still handle their own data/error states.

Show the static loading shell from `crates/zipeek-web/index.html` while the app boots. When
Yew first mounts, its first render must also be the loading state, with a continuous
handoff from the HTML loader. Keep loading visible until DuckDB-WASM is initialized
and the fetched database is open and ready for queries; mounting Yew is not the
end of startup loading. Avoid a blank screen or a flash of empty/ready content.

On a database load failure, show the retry state described for Word of the Day
instead of leaving a permanent spinner. See [ARCHITECTURE](../../../architecture/modules/web-shell.md)
for the startup stages. The shared centered loading presentation and persistent
landscape are implemented in A30; the actual database readiness/retry pipeline
is implemented by A06/A31; A21 adds practice-query loading after database Ready.

## Routes

Paths below are app-relative; deployment retains the `/zipeek/` prefix.

| Route | Intended view |
| --- | --- |
| `/` | Shared collapsible drawer/top-panel theme switch, Word of the Day, Practice/Browse/Random tiles |
| `/practice` | Setup: HSK system/level, practice mode, Fixed count, exclusions reset; PoS picker is future work |
| `/practice/type-pinyin` | Type the pinyin reading of a dictionary word and check it against its tone-marked or numbered reading |
| `/practice/character-builder` | Assemble dictionary-backed Hanzi from selectable character tiles |
| `/practice/word-mosaics` | Clear eight Chinese-to-English vocabulary pairs by selecting matching tiles |
| `/practice/eng-pinyin` | English to pinyin matching |
| `/practice/pinyin-hanzi` | Pinyin to hanzi matching |
| `/practice/hanzi-eng` | Hanzi to English matching |
| `/practice/mcq/hanzi-to-meaning` | MCQ: simplified hanzi prompt, English answers |
| `/practice/mcq/meaning-to-hanzi` | MCQ: English prompt, simplified hanzi answers |
| `/practice/mcq/meaning-to-pinyin` | MCQ: English prompt, display-pinyin answers |
| `/browse` | Paginated word list with a smart search box, filter chips and settings modal |
| `/stats` | Five-section learning dashboard shell; displays no mastery claims until approved consent-gated evidence and personal DuckDB readback are authoritative |
| `/games` | Playable game gallery, including three first-pass curated mini-games |
| `/games/cloud-match` | Source-backed pre-game selection followed by drifting-cloud vocabulary matching; see [CLOUD_MATCH](../../../games/cloud-match.md) |
| `/games/classifier-market` | First-pass curated Mandarin measure-word selection game; currently uses six reviewed-by-code examples, not a dictionary source |
| `/games/listening-navigator` | First-pass browser-spoken Mandarin direction game on a small grid; speech depends on browser voice availability |
| `/games/sentence-relay` | First-pass curated sentence tile-ordering game; authored prompts require linguistic acceptance |

All three matching variants and the D20 MCQ variants are in scope for v1. The overall query requirement
also includes English; the answered Browse controls are specified in [UX](../UX.md#browse).
A37 selects simplified/pinyin/first-English-gloss/HSK/PoS result fields and
the complete q/strategy/f/page/page_size URL contract in UX.
Pinyin browse queries support both display-tone and original numbered forms (D07).
Preserve D16's source transcriptions, including their spacing and conventions.
Both English-bearing variants show the first gloss: left in eng-pinyin, right in
hanzi-eng. Exclude rows without English from those variants; pinyin-hanzi can
still use them. Duplicate detection normalizes labels, not display casing.

## Practice

Practice product contracts and historical constraints now have a dedicated reference. See [Product Practice](../PRODUCT_PRACTICE.md) for the full maintained section.

### HSK filters

Moved to [Product Practice](../PRODUCT_PRACTICE.md).

### Round generation (D02: approved)

Moved to [Product Practice](../PRODUCT_PRACTICE.md).

### Practice modes (D03: approved)

Moved to [Product Practice](../PRODUCT_PRACTICE.md).

### Random shortcut (D04: approved)

Moved to [Product Practice](../PRODUCT_PRACTICE.md).

### Practice URLs and replay (D13: approved)

Moved to [Product Practice](../PRODUCT_PRACTICE.md).

## A22 scope boundary - 2026-10-27

The answered UX requirements in [UX](../UX.md) guide later feature work. Mid-round
exclusion affordance deferred, not part of v1; retain D13/D15 URL/exclusion
lifecycle and setup Reset. A37 now adds secondary traditional text to hanzi-labeled exercise surfaces,
while primary simplified labels and replay comparison keys remain unchanged.

UX-SUM01 was deferred from A22 by explicit user instruction. A10 now applies its
summary/restart/timer requirements under D35; A09 applies matching's first-attempt
pair accounting, clarified under D39. Chart.js 4.6.1 is approved for the ring only,
with esbuild 0.28.2 for the local tree-shaken build and retained licenses.
A22 itself added no dependency or policy row. A37 records the initial route/
drawer motion profile; that separate A32 work does not block practice.

## A37 product reconciliation - 2026-10-28

Traditional display is secondary presentation on hanzi exercise labels; D12
comparison keys and replay bytes stay unchanged. Browse uses the five approved
fields and complete URL/range rules in UX. Home must retain the daily fallback/
Retry and navigation tiles during dictionary failure, implemented with A12.

The earlier product-gap list does not reopen these existing decisions:

- D04 Random selects among the three match variants and always starts Endless;
  adding MCQ to Random is not approved by this pass.
- D13/D15 keep loaded URL exclusions authoritative, saved/pending IDs for generated
  URLs, and explicit setup Reset. Counts reflect the deduplicated valid saved set;
  there is no per-card exclusion/reset control in v1 exercises.
- D15 keeps Fixed progress independent of pool wrapping and resets it on reload.
- D20 meaning-to-pinyin answers use display pinyin; no prompt-variety expansion.
- UX-MATCH02/MCQ01 already define practice progress/streak presentation.
- Theme stays dark by default with saved-choice precedence; automatic OS-theme
  selection is not part of the approved policy.
- Wrong attempts/answers follow the recorded UX. The user's later first-attempt
  metric was deferred from A37 and is now applied to MCQ by A10/D35; matching's
  implementation is now provided by A09, including both-pair disqualification
  on a wrong A/B attempt (D39).

The 150 ms fade-only motion profile is selected; A32 implementation remains
separate. No schema, URL replay format, scoring implementation or dependency
change is made by this documentation pass.

## Site sharing metadata (D34)

Before v1, A40 implements static Open Graph and Twitter/X large-image cards with
one 1200×630 image for every current hash route. Use the existing landscape with
a dark slate overlay, Zipeek wordmark and blue accent; no particular word, round
or score. Approved title: "Zipeek - Chinese hanzi practice". Approved description:
"Look up Chinese words and practise hanzi, pinyin, and meanings." Keep absolute
canonical/image URLs under /zipeek/. This design is approved, not an implemented
asset or a promise that social platforms have fetched it.

## A11 setup delivery

The A48 extension permits a nonempty selection of any of the six types. Start
chooses randomly; each Next chooses a random selected type excluding the previous
one when possible. Mixed matching/MCQ sessions share Fixed round/question counts,
stats and timing. Summary requirements accumulate four pairs per matching round
and one answer per MCQ. D39 disqualifies both pairs on a wrong A/B match attempt.
The optional types URL parameter carries canonical selected tokens, encoded once;
the current route still controls current-round replay. Unknown/duplicate types
are ignored; if the route type is absent from the valid set, use that type alone
without rewriting the loaded link. Single-type URLs remain unchanged. D40 records
the approved session metadata/selection/accounting contract, with no v1 RNG break.

Setup now selects all six variants and starts a new unseeded URL using the
approved HSK/mode/count and validated saved exclusions. Start uses offset zero;
Endless omits rounds, empty exclusions are omitted, and no seed/PoS is invented.
Reset clears storage/local exclusions and updates the next Start URL while
remaining on setup. This does not alter a loaded round. Setup defaults to the
first MCQ activity for the quiz-first priority. The Endless MCQ question loop
is now playable under A10; A11 delivered setup controls/navigation.


## Practice presentation refinement (A49 / D41)

Matching and MCQ use a centered, opaque practice board capped at 56rem inside
the full-width shell. Matching keeps two aligned columns on phones and desktop,
with audio contained in each card and readable settled matched states. Hanzi-to-
English matching and Hanzi-to-meaning MCQ add display pinyin beneath hanzi.
A52 makes matching rows compact with right-edge Listen; MCQ keeps Listen below
the pinyin. Representative desktop matching views fit without vertical scroll. This is presentation only; matching/replay keys are unchanged.

Requested follow-ups A50/A51 are documented in [UX](../UX.md#requested-navigation-and-exercise-help-follow-ups):
narrower icon-labelled drawer navigation and per-exercise info dialogs explaining
the activity. A49 records these requests without implementing those features.


## Recorded future directions (A53)

[Future practice, settings and AI enrichment](../FUTURE.md) owns the 2026-10-29
requests: remove matching success checkmarks; address visually ambiguous
readings such as jing/Jing; local generator-side AI category tags and filtering;
a Settings route with theme and click-to-speak audio preference; two independently toggleable experimental audio exercises (microphone
pronunciation and three-option listening) and AI-assisted sentence translation
using a useful subset of 20 sampled words. These are planned
features with explicit open questions, not implemented routes/schema/exercises
or a change to the existing v1 round contract. A54-A60 track implementation
review and prerequisites in TASKS.

A53 follow-up: sentence translation is experimental too. It is enabled in
Settings with a configured AI base URL, model and API token when the server
requires authentication. Local LM Studio is the first intended connection
target; Pages-to-local browser support is subject to CORS/permission validation.
See [connection requirements](../FUTURE.md#ai-connection-in-settings).

Practice setup also needs richer category and part-of-speech selection (D44 /
A61). Apply OR within each selected group and AND across category, PoS and the
existing HSK filter. Category data depends on A56; existing practice PoS query
support does not mean the setup picker already exists. See
[future filter requirements](../FUTURE.md#practice-setup-category-and-part-of-speech-filters).

## Proposed learning review workflow (R61 discovery; not yet approved as product policy)

A learner can save five HSK 6 verbs from Browse into a personal Word Set. The existing Word Set is a collection, not a mastery claim. A future My Vocabulary view should show collected words and their independent recognition, listening, recall, writing and usage evidence; saved words should remain visible even before any practice. A unified review queue may draw from multiple sets without duplicating learning history for the same word.

Each successful or failed exercise could generate one or more *attributed observations*, with the exercise definition declaring which skill it actually tests. Recognition via MCQ must not be treated as writing; translation from visible tiles must not be treated as unassisted recall. The exercise instance and session retain gameplay authority, while the evidence ledger records learner performance across sessions.

A proposed mastery policy would require repeat successful retrieval across different calendar days over a span of at least one to two weeks, rather than permitting mastery in a day. Exact day count, minimum interval, timezone semantics, treatment of assisted attempts, spaced-review scheduling and downgrade rules remain **open decisions**. R61 introduces only the unscored evidence domain type, without changing the browser's behaviour or persistence.

### R263 authored Practice exercise: Handwriting Recall

Handwriting Recall is selectable from Practice, uses an eight-character example corpus, and runs inside the resumable browser-local UUID session model. The learner sees meaning and pinyin but not Hanzi, draws strokes on the existing canvas, then reveals the target for self-comparison. Undo and Clear are available before reveal. No handwriting recognition, automatic grade or mastery credit is awarded. Ink is not persisted on refresh; the UUID session step is restored for another attempt. Selected vocabulary filters do not yet determine recall prompts.


## Proposed game roadmap - chronology and science (2026-11-12)

These are **future concepts**, not registered routes, authored lesson decks, or acquisition-grade evaluations. They require separate feature approval and implementation planning before becoming playable.

**Time Weaver (时光拼图)**: Playable randomized date, weekday and clock-time slice at `/games/time-weaver`. Configurable 10, 15 or 20 randomized rounds, balanced across five modes, cover assembling a Hanzi date from year/date tiles, choosing a Chinese date, identifying a weekday (星期), assembling a clock time from Hanzi tiles and choosing one of three plausible clock-time readings. Example dates include 21 April 2015 → 二零一五年四月二十一日; example times include 3:20 PM → 下午三点二十分. The pure Rust engine rejects invalid Gregorian dates and clock values, handles leap years/month boundaries, and generates unique valid dates and times from a session seed; browser-started sessions use fresh random seeds, and the same seed reproduces the same questions. It practices conventional Mandarin clock expressions including 一刻, 半, 三刻, 两点 and leading-zero minutes. The standalone setup provides accessible switch cards for Pinyin and speak-on-tap, a three-size segmented tile selector, a 10/15/20 prompt selector and a live tile preview. Choices persist as browser-local `zipeek.time_weaver.options.v1`; invalid records return to safe defaults (15 prompts, large tiles, Pinyin on, audio off). This local preference is not included in personal Word Sets backups. All rounds are game-only scoring, not acquisition evidence. Alternative synonymous expressions, actual microphone speech, free-form text typing, seasons, relative dates, scheduling, lunar-calendar questions, or spoken audio remain unimplemented pending explicit validation policy. No personal schedule data is used.

**Cosmic Lab** (working title): advanced Chinese-language scientific reasoning covering the Solar System, stars, cosmology, and related physics such as neutron stars and nuclear fission. Each authored question and **all three** choices are in Chinese; exactly one answer is scientifically valid under the stated assumptions, while two distractors are plausible but subtly incorrect. Explanations identify the misconception after submission; pre-answer hints must not leak the correct option. Content needs independently reviewed scientific sources, date/version provenance, difficulty and Chinese-language quality review, stable question IDs, deterministic option shuffling, explicit valid-answer validation, and separate science-knowledge vs. language-evidence attribution. AI may assist drafting but cannot publish or score unverified facts. Nuclear fission is nuclear physics, not a planetary topic, so the game is deliberately broader than astronomy.

**Gallery layout amendment:** On page one Province Quest spans a 2×2 four-slot featured card; on page two Sentence Relay now uses that same four-slot treatment. Gallery pagination remains capacity-nine, six game links on page one and five on page two after Time Weaver is introduced. This is presentation only; neither the Sentence Relay question engine nor the game roster has changed.

### R349 first-run setup and developer diagnostics (WIP)

On a first visit with no existing user preferences or work, the landing **Enter Zipeek** action opens a short setup form with one of the six existing themes and a separate, default-off choice to save completed Practice outcomes locally. Both the theme and `learning_history_enabled` consent are stored in `zipeek.preferences.v1`, the single preferences document also read at Practice completion and controlled under Settings → Learning Data → Practice learning history. The retired standalone recording-consent key is not read or migrated; previously opted-in users must opt in again. Saving the choice updates the shell recording indicator immediately and survives reloads. The setting permits local Practice outcome recording only; it does not enable mastery assessment, course unlocking, AI transmission, or the unfinished pedagogical evidence pipeline. Disabling it prevents subsequent outcome recording but does not erase previously saved records. Returning learners retain any consent already stored in `zipeek.preferences.v1`.

In development builds only, the Debug page has Overview, DuckDB and SQL tabs; the SQL tab includes a bounded SELECT terminal for the **read-only packaged dictionary**, an accessible table preview of up to 100 rows and a CSV export of exactly those displayed rows. No personal OPFS database is connected. A future personal-storage admin terminal would require an explicit ownership, permission, export, delete/backup and privacy design; the dictionary-only terminal must not be presented as such a tool.

## R483 Courses mosaic presentation

Grammar and Vocabulary have separate 100-region curricula and region IDs. Their rendering may share clipping and softly curved boundary utilities, but must not mix Grammar and Vocabulary cells. Grammar uses balanced independent regions; Vocabulary retains size cues from HSK word counts. The incomplete-region dim overlay may show the illustration for orientation, but is not completion or mastery evidence. No icons appear inside mosaic pieces. Visual and interaction acceptance is pending Chrome review.

## R485 Grammar curriculum development

As of R488, the 100 Grammar-only mosaic regions all have available teaching courses with **550 authored teaching entries**: the original five courses (24), fifteen courses from R485 (75), and eighty newly populated courses (451). The new lesson data has original pattern labels, short explanations, Chinese examples and English translations and has passed structural/duplicate audits. **This is draft teaching content pending independent Mandarin and HSK review**; a count of entries does not prove 550 distinct linguistic constructions or verified syllabus coverage. Expanded courses use single-question staged guided reviews and browser-local checkpoints, not scored independent mastery assessments. Neither viewing nor practising unlocks mosaic artwork; AI examples remain optional unverified supplementary material.

## November 17 approved next-stage behavior (pending implementation)

AI-requiring practice exercises are disabled when the required service is unavailable, with an actionable explanation. Pedagogical course tiles without proven progress remain visibly locked, with lesson details concealed in the tile. Per-pattern grammar visualization uses ten verified eligible sentence-exercise successes on each of seven qualifying days in a rolling fourteen-day window. New speech-to-text practice must support browser recognition and a configured local recognizer with separate capability/consent and scoring rules. These four decisions are recorded as D53-D56; current source is not evidence of implementation.
