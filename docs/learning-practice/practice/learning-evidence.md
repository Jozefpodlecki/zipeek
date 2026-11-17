# Learning evidence, proficiency, and history

**Status:** approved product requirements, refined 2026-11-09; **not yet implemented in the production learner database, Settings, or Stats route**. This is the normative future behavior, not a description of today's Practice scoring. See [personal-data architecture](../../dictionary-data/schema/personal-data.md), [Practice setup](practice-session-builder.md), and [deterministic E2E plan](../../testing-quality/e2e/learning-evidence-e2e.md).

## Purpose and authority

Collect **Practice records first**, then derive learning observations with traceable provenance. The personal DuckDB database is the planned durable authority, distinct from the read-only dictionary DuckDB and the temporary UUID Practice records in localStorage. Do not describe a probe database or a deleted temporary Practice record as saved pedagogical history. On completion, a durable session write must be acknowledged before clearing the last recoverable evidence and showing a saved confirmation. On write failure the learner must receive a truthful error and an explicit recovery path; silently losing results is unacceptable.

Record session ID, creation/start/end timestamps, configured vocabulary source, stable item IDs, exercise configuration, category and HSK membership where available, and a distinct attempt ID for each scored response. Derive per-Hanzi, per-word, per-grammar-point, and per-skill evidence from the scored attempts; preserve session and attempt references, item version, outcome, validity, and assessor. Distinguish a character within a word from a tested word identity; do not claim separate Hanzi recognition from an unrelated whole-sentence score without a defensible attribution rule. Duplicate delivery/replay of one attempt must be idempotent.

Raw microphone recordings and free-form learner text are more sensitive than score metadata. An assessed spoken result needs evidence of pronunciation/meaning correctness, **not merely permission to record or the presence of audio bytes**. Consent, audio retention, export, deletion, and local-only versus external AI processing remain to be specified before activation.

## Approved proficiency eligibility

All of the following apply to **the same stable Hanzi or word identity**:

1. **Seven distinct active local calendar days:** at least one qualifying, scored observation on each of seven different days. Seven encounters on one date, or a seven-day elapsed interval containing only two active dates, are insufficient. A missing day is not a failure. Dates must be evaluated consistently using a documented learner timezone policy; daylight-saving transitions must not change the day count arbitrarily.
2. **At least ten qualifying scored encounters**, distributed across **at least three different completed Practice sessions**. Repeated attempts in one session can contribute but **must be capped per item/session** so a learner cannot manufacture proficiency by endlessly repeating one item. The exact per-session cap and whether remediation retries count remain policy parameters to finalize, not numbers invented here. Record all attempts for diagnostics even when some do not contribute to eligibility.
3. **Successful evidence in four independently assessed competencies:** visual recognition, listening comprehension, assessed spoken production, and appropriate use in a sentence. Selecting Matching ten times can build recognition evidence, **not** certify speaking, listening, or sentence use. A competency with no qualifying successes remains unproven. The precise number and proportion of successful observations per skill are not yet approved. Ten encounters is an overall minimum, **not ten in each skill** unless a later decision changes the policy.
4. **Dynamic performance:** a later qualified failure can lower the affected skill rating or return the item to needs-review. A historical mastery award is not an immutable course-completion record. Repeated failures must not be masked by accumulated easy successes. The decay/recovery thresholds are still open.

The first three requirements are **necessary, not automatically sufficient**. Until per-skill acceptance criteria and validators are implemented, Stats may display the observed counts and pending competencies, but must **not** label the learner proficient simply on day seven or encounter ten.

### Classifying evidence by what an exercise actually tests

| Experience | Permissible evidence when scored and validated | Not automatically evidenced |
| --- | --- | --- |
| Matching / glyph-choice / image-to-Hanzi | Recognition of the tested identity | Listening, speech, productive sentence use |
| Listening comprehension (audio to meaning/selection) | Listening, where audio playback and answer are genuinely assessed | Speaking from merely hearing audio |
| Recorded spoken response | Speaking **only after a reliable assessor checks it** | Pronunciation competence from recording existence alone |
| Sentence creation, transformation, or context use | Sentence use if meaning and grammar are assessed and the target identity is attributable | All component words mastered, or free production from a fully revealed answer |
| Grammar-course guided examples and hints | Instructional activity; possibly separate assisted evidence | Unassisted mastery or passed grammar assessment |

Track `assistance_level` and `assessment_kind` so a word-bank Sentence Builder displaying the correct solution and playable audio remains a useful **teaching experience** without improperly awarding unassisted production credit. Learning activities and scored assessments are distinct events.

## Word Set eligibility in Practice setup

**New requirement, not currently enforced across the shipped setup:** a saved Word Set is selectable as a Practice vocabulary source only if it contains **at least ten distinct valid Hanzi-bearing vocabulary entries**. The intended user example is ten words, not ten total Unicode Hanzi code points. Deduplicate by stable vocabulary identity; ten duplicates or invalid/stale entries do not pass. If fewer than ten are playable for the chosen exercise, the option is **disabled/greyed out** with an accessible explanation and count (for example, `7 of 10 words available; add 3`). The same eligibility rule must be revalidated when launching, including after a set is edited or dictionary entries become unavailable; disabling only the visual button is insufficient.

This is a Practice **source eligibility requirement**, not a minimum size for creating or storing Word Sets, and not a claim that all ten words must appear in every question. A source with ten distinct entries allows the learner to practise; it does **not** guarantee ten qualifying observations for any one word. Other vocabulary sources and exercise-specific minimums remain separate unless explicitly revised.

## Future /stats experience

The proposed `/stats` route reports per-word/Hanzi evidence transparently: qualifying encounters, distinct sessions, distinct active days, per-skill measured/needs-evidence/needs-review status, category and HSK association, and relevant grammar points. Expose why an item is not proficient; do not show only an unexplained percentage. A character's observed skill state and a word's observed skill state must remain distinguishable. Current does not implement this route or its durable backing ledger.

## Grammar learning and irreversible completion

The future [Courses mosaic](../courses/courses-mosaic.md) is built against the **HSK grammar syllabus**, with an explicit mapping from each authoritative grammar point to taught course, questions, assessments and stable IDs. Do not assert complete official coverage before item-by-item verification. Courses use authored explanations, worked examples, guided exercises (image-to-Hanzi, Hanzi writing, Character Builder, Sentence Builder and word-bank Sentence Transformation), and individually scored grammar-point assessments.

Instruction may intentionally reveal an answer, e.g. a Sentence Builder's correct-sentence label with playable audio below the builder. Such scaffolding is **not equivalent to a correct independent assessment**. Per-point assessment requires at least five scored questions and 80% in one attempt by default, as specified in the Courses contract. Passing every required point makes **course completion permanent** and reveals that course's large mosaic region. Vocabulary proficiency remains dynamic and separately determines small vocabulary-region opacity; grammar-point completion never implicitly marks associated words known.

## Storage/privacy gates

Use the personal DuckDB for authoritative session and derived evidence records once durability is verified; implement a clear Settings recording/retention control before collection. A final decision on opt-in default, raw audio/text retention, local/export/erase behavior, and handling of model-provider transfer is still required. The existing temporary Practice record must not be silently treated as a history database. Plan versioned schema and migrations, dictionary-identity continuity, idempotent commits, cross-tab coordination, quota/corruption recovery, export/import and user deletion.

## Verification, including time travel

Use an injected clock interface for both domain computation and timestamp generation. Production uses real time; tests use a deterministic fake clock with **explicit date/time and timezone**, advanced between sessions. Never change the operating-system clock, monkey-patch browser `Date` globally, or trust mutable DOM timestamps for proficiency. The Rust domain clock and browser persistence boundary must receive the **same simulated time**. Record timestamps immutably when attempts are scored; altering the test clock cannot rewrite history. See [E2E scenarios](../../testing-quality/e2e/learning-evidence-e2e.md).

Tests must cover seven distinct dates, ten capped scored encounters over at least three sessions, all four skill categories, session replay, duplicates, failure-driven regression, disabled recording, successful durable saves, failed writes, cross-session reload, and Stats reading the same persisted facts. Test both the boundary just before eligibility and the first genuinely eligible state. When per-skill success thresholds remain undecided, test *eligibility evidence* and a clearly pending overall status, not a fabricated proficiency pass.

## Open decisions (only these remain policy-dependent)

- The maximum qualifying encounters for one identity in one session; counting of immediate retries and partial answers.
- Per-skill success sample sizes/thresholds, remediation, and reliable listening, speaking and sentence-use assessment rules. Selecting only Matching must never satisfy all four modalities.
- Learner timezone policy when travelling, recording opt-in default, and retention/export/deletion of raw responses and microphone audio.
- Stable vocabulary/character identity across dictionary updates and the distinction between a word's compositional evidence and its constituent characters.

Do not convert these unresolved details into undocumented hard-coded defaults.

## Hanzi recognition award versus vocabulary proficiency

The separate [Hanzi recognition acquisition contract](../../dictionary-data/schema/pedagogy.md) is approved for future Dragon Progression: ten valid correct recognitions in at least three completed sessions per qualifying day, at most four credits per session, over seven consecutive dates separated by at least 24 elapsed hours; a per-day 0/1/2 mistake allowance defaults to zero. A failed day resets that acquisition streak, and a successful day locks when it qualifies. Earlier specification passages above describe the pre-existing the historical source vocabulary and full multi-skill proficiency design; they must not be used as the Hanzi Dragon awarding algorithm. Recognition acquisition is a distinct historical milestone, not proof of listening, speech, writing or sentence-use proficiency. The the historical source engine is in-memory only and is not integrated into production event capture or `/stats`.

## Approved 2026-11-17 grammar observation window

For grammar visualization, the selected target rule is seven qualifying local days within a rolling fourteen-day window, per canonical grammar-pattern ID. On each qualifying day, ten independent correct and verified attempts must use that pattern in Translate Sentence, Sentence Builder or Sentence Transformation. Merely generating an AI sentence, revealing a solution or matching a transcript without a validated assessment is insufficient. This is not implemented or verified and does not revise the D52 Hanzi recognition streak or the existing per-point course assessment contract without separate reconciliation.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/learning-practice-practice-learning-evidence.md). Dated implementation claims in that snapshot are not current build or browser verification.
