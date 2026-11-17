# Practice setup - historical design

**Update (2026-11-04):** The newly approved top-level four-step session builder and nested Configure tabs are documented in [PRACTICE_SESSION_BUILDER.md](../../learning-practice/practice/practice-session-builder.md). Follow that document if this historical one-page layout conflicts.

Approved direction: 2026-11-04. The reference is the user's two dark-theme
Practice/Dictionary mockups; the mocks illustrate hierarchy, not a promise that
all depicted options already work. This document is the product contract for
the next stages of implementation. R79 implements the Browse-facing first slice and chosen-Word-Set handoff.
R80 implements a pure, typed draft and validation model in `zipeek-exercises`.
R81 introduces the first exercise-first selector and definition-derived settings tabs using that draft.
R82 uses a single, shared source selector on the Practice page: Dictionary/HSK
(single selected band) or a saved personal Word Set. Both the Matching/MCQ and
Translation launch paths consume it; there are no independent vocabulary inputs
in their settings panels. R85 adds multi-HSK and database-suggested category chips to the shared selector and routes custom filters to Translation through a validated URL source; Matching/MCQ still deliberately requires a single HSK band and no other dictionary filters. The unified mixed-session planner, complete combobox keyboard support, and pool-size validation are not yet connected.

## Product mental model

1. **Browse** searches and curates dictionary words. A row can be listened to
   using Chinese speech synthesis and saved into a personal Word Set. It does
   not launch a one-word quiz.
2. **Practice setup** chooses one or more exercise definitions, a vocabulary
   source, relevant exercise-specific options, and session length. The
   configuration is validated before starting.
3. **Session** owns the selected configuration and one shared result ledger;
   exercise UIs own only their interaction state. Abandoned questions are
   terminal without scoring.
4. **Profiles** save configuration for later reuse, not current results,
   microphone recordings, or generated AI answers.

## Screen: Practice setup

The first interaction is **Choose exercise types**. Use accessible multi-select
exercise cards (keyboard toggle, visible selection, and descriptive support
text) for Matching, Multiple Choice, Sentence Translation, Speaking, and
Listening. Speaking is a selectable exercise, **not** a separate top-level
practice route. Recording without validated assessment is permitted only as an
explicitly labelled preview and contributes no proficiency evidence.

Additional configuration surfaces unlock from selected exercise definitions,
not a fixed catch-all set of page tabs:

- **Vocabulary source** appears for any vocabulary-dependent exercise.
- **Session options** (Fixed/Endless, number of finalized questions) appear
  for every quiz type that supports them.
- **Translation options** appear only when Sentence Translation is selected:
  direction, word-bank assistance, and permitted generation backend.
- **Speaking options** appear only when Speaking is selected: reading prompt,
  waveform preference, and a local-only recording disclosure.
- **Listening options** appear only once an actual listening exercise is
  implemented and selected; don't advertise nonfunctional scoring.
- **Profiles** offer Load, Save as, Rename, Delete and explicit overwrite.

The tab strip is conditional: General is always available; specialized tabs
appear only for selected exercise types. Removing a type removes its tab but
must not silently delete saved profile data until explicitly saved. If the
active tab becomes unavailable, focus/selection returns to General. A top
stepper communicates Exercise types → Vocabulary → Options → Review/Start;
only currently available stages are actionable. On mobile the stepper is a
compact progress label, not a horizontal overflow rail.

## Vocabulary source is an exclusive choice

**Personal Word Set:** choose one named, nonempty saved set; use an exact
snapshot of its Word IDs. HSK, category and PoS filters are hidden/disabled
and **never** intersected with that set. Show name, word count and an explicit
empty-set error. A chosen set from Browse can prepopulate this selector.

**Dictionary vocabulary:** use HSK **systems and levels** (multiple selection),
category autocomplete with removable chips, and optional advanced PoS chips.
Never conflate HSK 2.0 and HSK 3.0 levels. Within each filter group selections
are OR; across groups HSK/categories/PoS are AND, consistent with D44. A user
can select multiple categories from suggestions searched as they type. Every
chip has a labelled Remove action, Backspace works when the input is empty,
keyboard arrows navigate suggestions, Enter accepts, Escape closes, and
screen readers receive an announcement of selected count. The control must
support search even with no available categories and never fabricate tags.
Display the estimated eligible word count and reject empty pools before Start.

The initial HSK/category query must be backed by installed database values,
not generated by the model. An AI model may generate a sentence *using* the
selected vocabulary, but cannot choose an unrelated hidden vocabulary pool.
Sense-specific readings and translations must be preserved during sampling.

## Profiles and persistence

Profiles are optional, device-local **versioned configuration snapshots**.
They contain selected exercise definition IDs and versions, vocabulary-source
selector/filters or saved Word Set reference, per-exercise options, session
mode/length, and a human-readable profile name. No scores, hidden answers,
recorded audio, model credentials or generated content. Do not change the
existing Word Set persistence key. Profile storage requires a separate
approved and tested schema; R79 does not introduce it.

Loading a profile atomically replaces the draft setup after validation.
Deleted/missing Word Set, unsupported exercise version, or no supported
exercises blocks Start with a specific recovery message; **never** silently
switch to unrestricted HSK vocabulary. Saving a profile does not start a
session. Start always begins a fresh session with a frozen configuration.

## Launch and acceptance rules

- No session-wide score UI before a meaningful exercise result exists.
- One Start action from setup; AI endpoint/model requirements must be clear
  before an AI-dependent session starts. A missing model never starts an empty
  session that then fails on its first question.
- Fixed count measures **finalized exercises**; an abandoned question does
  not consume a slot or contribute proficiency evidence. Completion shows
  authoritative totals; a sixth question cannot appear in Fixed/5.
- Source/exercise changes during setup don't mutate an existing active
  session. A new Start creates a new session generation.
- Visible exercise payloads and AI learner observations never include hidden
  reference answers. Session scoring remains authoritative in the domain.

## Implementation order

1. R79: compact Browse, row pronunciation, explicit selected-set handoff,
   five-row pagination, remove ambiguous dictionary quiz actions.
2. R80: extract pure typed `PracticeDraft` and source filters; domain tests for
   OR/AND semantics, empty selectors and unsupported assessments. This is
   a validation contract, not a claim that the browser queries support all
   these filters or that dictionary pool sizes are checked yet.
3. R81: Exercise-first multi-select cards with accessible pressed states,
   conditional configuration tabs, and retained existing launch routes.
   The category multi-combobox and shared source/launch controller remain
   follow-up work, not features attributed to R81.
4. Session-aware Start and per-exercise configuration; then versioned local
   profiles with explicit import/export policy.
5. Thirtyfour cross-route acceptance including profile reload and Fixed/5
   HSK6 translation, and responsive accessibility/visual review.

Keep all phases within existing crates. Do not add dependencies, change the
DB schema or change AI classification prompts/cache keys to support this UI.

## R80 typed draft boundary

`zipeek-exercises::practice_draft` now owns a `PracticeDraft` with a set of
registered exercise definition IDs, an exclusive `VocabularySource`, a
`SessionLength`, and translation-specific options. The selected IDs refer to
existing `definition::DEFINITIONS`, not to a parallel central exercise-kind enum.
`configuration_tabs()` derives the optional tab IDs from selected definitions.
`validate_for_scored_session()` distinguishes the current unscored Speaking
preview and not-yet-implemented Listening from scored practice.

`DictionaryFilters::matches()` defines OR within HSK/category/PoS groups
and AND across populated groups, with HSK 2.0 and 3.0 treated as distinct
keys. Category chip insertion removes surrounding whitespace, normalizes to
lowercase, deduplicates and supports removal. The model does not itself
perform a DuckDB query, validate a saved set's contents, resolve category
suggestions, or start a browser session. Those integrations require separate
source adapters, database-backed validation, and explicit UI tests.

Until the exercise-first UI is migrated, existing Practice tabs and route
query semantics remain unchanged. Do not treat the new draft as already
being used by the current Start buttons.

## R81 selector boundary and rollout

`PracticeBuilder` is a Yew presentation component that receives an actual
`PracticeDraft`, invokes domain-validated exercise toggles, and renders
`configuration_tabs()` directly. General is always available, while the
Translation and Speaking settings appear only when those definitions are
selected. Listening is displayed disabled because no playable listening-choice
exercise is implemented. Speaking remains explicitly unscored.

The existing Matching/MCQ variant picker is still responsible for question
formats and the existing routing, and the Translation panel still supports
the established one-HSK-level, saved-set or sample-topic source contract.
Selecting multiple exercise definitions **does not** yet launch a mixed
cross-route session. The UI says this explicitly, and the two launch controls
are labelled for the exercise they actually start. Do not build profiles on
these intermediate route-specific source controls; finish the shared source
and launch controller first.

The Dictionary → Practice handoff uses `saved:<index>` in the URL; the
Matching/MCQ adapter converts this to its established numeric selector,
while Translation keeps its `saved:<index>` source encoding. Empty or deleted
sets are rejected, not silently replaced with unrestricted vocabulary.

## R82 implementation boundary and next work

The source switch is one-of-two, not an intersection. In Word Set mode the
selected set is handed to Matching/MCQ as the exact ID snapshot and to
Translation as `saved:<index>`. In HSK mode one band is passed using the
existing canonical route/query representations. The shared selector is the
only source control; the per-exercise panels retain their own session-length
options until the shared session planner is completed.

The `PracticeDraft` domain already models OR within HSK/category/PoS groups
and AND between groups. To expose those selections safely, extend the
database-backed candidate-pool contract for **both** Matching/MCQ and
Translation first; then add a category autocomplete sourcing valid slugs
from the installed database. Until then, the UI intentionally offers one HSK
band, not decorative chips that are silently ignored on launch.

## R83 parameterized filter adapter

`web/services/database/practice_filter_queries.rs` now provides a bounded,
parameterized read-only dictionary query for the approved filter semantics:
HSK bands match by (system, level) with OR across selections; category and
part-of-speech membership use OR within their own groups; groups intersect.
The category autocomplete data source queries distinct `sense_categories`
slugs, returning at most 24 ordered suggestions for a typed prefix.

This is a **query foundation, not the live practice source picker**. The
current UI still chooses one HSK band. Before displaying additional selected
bands or category chips, both the Matching/MCQ and Translation launch adapters
must consume this same filter contract, handle an insufficient pool explicitly,
and verify that no selected filter is silently ignored. The bounded query's
ordered first rows are not an unbiased sample; subsequent adapter work must
choose a suitable seeded sampling strategy without changing v1 replay
semantics. No profile storage is added here.

## R195 exercise activation amendment

The earlier disabled-listening wording in this document describes pre-R195
behavior. Listening and Picture vocabulary are now selectable from Advanced
Practice, but each opens a standalone exercise page. Their source selection
is currently performed on the destination page using saved Word Sets. The
regular scored-session planner still rejects these IDs until mixed-session
scheduling, feedback, and persistence are explicitly implemented.

## R268 generated exercise Start gate

AI-dependent fixed UUID sessions now require a configured AI model before Start is enabled, as required by the launch contract above. The generated question and image URL are saved as bounded, validated browser-local session content. Sentence translation and picture recognition currently use practice-only feedback, not authoritative mastery scores. Live AI and image-server availability remain runtime dependencies; errors are surfaced with bounded Retry or Skip. Full browser acceptance remains outstanding.
