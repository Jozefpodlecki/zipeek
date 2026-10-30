# Future practice, settings and AI enrichment

User directions recorded on 2026-10-29 under A53. These features are future
work, not an extension of the implemented v1/replay contract merely because
they appear here. Live status and prerequisites belong in
[TASKS](../process/TASKS.md). Unanswered choices stay open.

## Matching feedback and ambiguous entries

The user considers the green success checkmark unnecessary and requests its
removal from matching cards. Keep an understandable matched/disabled state;
the exact replacement, including whether background feedback alone is enough,
belongs to A54's implementation review. This request does not remove MCQ
feedback icons or change scoring. Current matching still renders the checkmark.

The supplied screenshot shows `井` twice: `jǐng` and `Jǐng`. The pinned primary
dataset has both readings: the lowercase form starts with “a well”; the
capitalized form starts with the Chinese astronomy constellation gloss and
also includes the surname gloss. These are distinct imported reading rows,
not evidence that a dictionary row should be deleted or merged.

Current hanzi-to-English uniqueness compares normalized right-column English
labels, not left-column hanzi. Thus both can appear in one round despite
nearly identical learner-facing prompts. Pinyin capitalization is a weak cue.
The user approved avoiding duplicate simplified hanzi within a matching round
(D42). The four target pairs must not represent different readings sharing the
same simplified hanzi; the two cards belonging to one pair still represent that
one word as usual. Preserve both source readings for other rounds. This does
not authorize sense labels or change MCQ duplicate policy. A55 owns the change.
Do not silently case-fold source reading identity. Any builder selection/RNG
change needs the explicit replay compatibility review required by
[ROUND_GENERATION](ROUND_GENERATION.md) and AGENTS; no replay break is approved
by recording this policy.

## Local AI category enrichment

The user added AiMux/LM Studio scaffolding to `db-gen` to use a local model
for classification. The goal is category tags associated with an entry and
later filtering by those tags. User examples:

- Medicine or medical usage.
- Idioms.
- Technical or specialist usage.
- Poetry/literary usage.
- Very rare usage.

These examples are a starting vocabulary, not an approved exhaustive taxonomy.
“Rare” may be a usage/frequency attribute while “medicine” is a domain and
“idiom” is a form; do not force them into one exclusive class without review.
Preserve specialist entries in the dictionary; classification alone does not
authorize excluding them from all practice.

A56 must refine whether tags apply to a reading row, an individual sense or a
shared headword, allow multiple tags, define stable tag IDs and filter behavior,
and propose the exact storage/query/export changes for schema approval.
Suggested review topics include unclassified entries, manual overrides,
classification provenance/model/prompt identity, caching and reproducibility,
failed responses and confidence/review policy. These are design proposals.
Existing WordIds and source data remain authoritative until explicitly amended.

Classification belongs to an explicit generator/enrichment workflow, not a
Trunk rebuild or an implicit runtime dependency. Current checked-in data has
no category-tag schema or classifier output. See
[db-gen notes](../architecture/modules/db-gen.md#user-added-local-llm-scaffold).

## Practice setup category and part-of-speech filters

User request and refinement recorded 2026-10-29 (D44 / A61): Practice setup
needs a richer filter section so a session can use only words from chosen
categories and/or parts of speech, alongside the existing HSK selection.

Approved combination rule: **OR within each group, AND across groups**. For
example, (medical OR technical) AND (noun OR verb) AND the selected HSK level.
An empty category or PoS group adds no restriction for that group. Each selected
exercise still applies its own existing eligibility requirements. The selected
session filters should apply throughout a mixed-type session.

Proposed UI for the implementation review: searchable multi-select category
and PoS controls, removable chips for active selections, Clear filters, and an
eligible-word count to help avoid unusably small pools. Exact layout, mobile
interaction, count semantics and insufficient-pool handling remain open.
The word "fancy" expresses a request for a richer usable filter experience,
not approval of a particular widget/library or new dependency.

Current practice URL/core query types already support PoS, but setup currently
constructs an empty PoS selection and exposes no PoS picker. PoS UI can therefore
be scoped independently. Category filters depend on A56's approved taxonomy,
tag data and schema/query design; current artifacts contain no such tags.
Practice filtering should use generated tags rather than classify words with
a live AI call whenever a session starts.

A61 must define how filters survive Start, mixed transitions, Play again and
Reset, plus any URL encoding/sharing changes. Do not invent a category URL
parameter or modify v1 canonical replay inputs in this documentation task.
Document schema/replay impacts and obtain their required approval before code.

## Settings route and audio interaction

The user requests a future Settings route with theme selection and an audio
preference. `/settings` is the proposed app-relative path, preserving the Pages
base `/zipeek/`; the current router has no Settings route.

Requested audio mode: hide the audio icon and automatically play pronunciation
when the user clicks a word/card. This is user-triggered speech, not automatic
speech on route load. Explicit icon mode remains the current behavior.
A57 must define default mode, persistence, which surfaces/clicks speak, replay
and cancellation behavior, keyboard equivalence, unsupported-speech fallback,
and whether the existing topbar theme control remains available too.
Card selection must still perform its exercise action; the preference must
not create a second scoring action or nested interactive controls.

## Experimental pronunciation and listening exercises

The user clarified that these are two distinct future exercise types (D43):

- **Pronounce a word:** the learner speaks into a microphone. Recognition,
  acceptable pronunciation, grading/feedback and microphone failure behavior
  remain to be defined under A58.
- **Listen and choose:** the app speaks a word and the learner identifies it
  from exactly three options. Option representation (hanzi, meaning or another
  label), retries/feedback and progression remain to be defined under A60.

Both are experimental and must be independently toggleable in Settings. They
should only be offered in setup/mixed-session selection when their Settings
option is enabled. Default enablement, preference persistence and behavior if
a previously enabled type is disabled while selected remain open. Current
six-type setup/URL behavior is unchanged by this future requirement.

The user reports that speech synthesis does not pronounce the Chinese word
for "2" (二, èr) reliably. Record this as a reported speech-quality issue, not
as an inherent inability to pronounce that character in every voice/browser.
Speech synthesis and microphone speech recognition are separate capabilities.
Qualification for one does not prove the other works.

Eligibility is required for the audio exercises, but its source is undecided:
a curated allowlist, voice/browser qualification, per-reading pronunciation
metadata, or another reviewed policy. Evaluate both exercises separately, and
include the reported 二 / èr case in future synthesis evaluation. Do not exclude
entries from unrelated exercises as a consequence.

## Translate sentence exercise

The user proposes querying 20 random words and asking AI to make a sentence
for a translation exercise. They approved using a useful subset of that
20-word candidate pool; the model does not have to use all 20. This exercise
is experimental and must be enabled in Settings. Enabling it exposes AI
connection configuration; it is usable only with a working configured endpoint.
A59 must define:

- Translation direction and how the learner submits an answer.
- Difficulty/HSK/category filters, grammar constraints and sense selection.
- Expected translation, acceptable alternatives, grading and feedback.
- Browser connection to the user-configured AI endpoint, initially targeting
  local LM Studio. GitHub Pages/browser compatibility still needs a practical
  spike; generator-side AiMux does not run inside the static browser app.
- How to handle unavailable AI, malformed/unsuitable sentences, latency,
  caching and word/sentence provenance.

No specific model, hosted provider/service, new route, exercise URL/replay
format or dependency is selected here. Current v1 round determinism remains intact.


### AI connection in Settings

User refinement recorded 2026-10-29: sentence translation joins the two audio
exercises as an experimental Settings-gated feature. Settings needs an AI
connection section. Proposed fields: API base URL, model identifier, an API
key/token when required by the chosen server, and Test connection. Exact layout,
validation and endpoint/token persistence are still implementation choices.

For the first local-development target, LM Studio supports an OpenAI-compatible
API base such as http://localhost:1234/v1 and model discovery. Its authentication
is optional by default; if Require Authentication is enabled, requests need
a bearer API token. Browser calls from Zipeek's localhost:1420 to port 1234
are cross-origin, so LM Studio must permit CORS. These are documented platform
capabilities, not a claim that a Zipeek connection has been tested. See
[LM Studio compatibility](https://lmstudio.ai/docs/developer/openai-compat),
[server settings](https://lmstudio.ai/docs/developer/core/server/settings) and
[authentication](https://lmstudio.ai/docs/developer/core/authentication).

GitHub Pages is static hosting; the browser would contact the user's endpoint
directly. A localhost endpoint means the visitor's own machine. Pages does not
run LM Studio/db-gen or provide a secret-hiding backend. A public HTTPS page
may reach a local server in supporting browsers with the relevant local/loopback
permission and CORS, but browser support/policy and connection failure behavior
must be tested before claiming hosted support. Local development is the first
validation target; unsupported hosted connections need a clear unavailable
state rather than a guaranteed fallback. See [GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages)
and [Chrome local network access](https://developer.chrome.com/docs/modern-web-guidance/guides/security/local-network-access).

Use a user-supplied token when needed; never embed a shared application secret
in the static bundle, route URL or generated dictionary. Session-only versus
remembered credential storage remains open. No endpoint call, token collection,
new dependency or remote service is introduced by these notes.


## Schema and query review prerequisite

The user requests a schema review covering indexes, additional tables and
full-text search support in WASM. A62 evaluates those choices against the real
word/filter/search workload, including A56 tags and A61 setup filters. See
[the review brief](../architecture/schema/REVIEW.md). Official WASM support for
the fts extension is documented; Zipeek's pinned version/read-only artifact
compatibility is still untested. No extra table/index or ranked-search change
is selected merely by recording this request.
