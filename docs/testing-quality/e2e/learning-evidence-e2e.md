# Planned pedagogical-history and proficiency E2E contract

**Status: specification for future Rust tests, not existing passing coverage.** The source baseline documented here has no `/stats` implementation, durable Practice history writer, or production clock injection. This plan deliberately does not assert they exist.

## Testable architecture

- Introduce a `Clock` / `TimeProvider` boundary in the core learning-evidence domain. Reads of *now*, session start/finish, attempt timestamps, calendar-day classification and Stats computation use the same injected service. Production uses a real clock; E2E harness uses a fixed, advancing clock. Prevent direct scattered calls to `SystemTime::now`, `Date.now`, or `new Date()` for evidence timestamps. An explicit learner timezone or deterministic calendar-day rule must be shared by both app and tests.
- Expose the fake clock only in an E2E-authorized build or fixture boundary, never via a publicly writable production endpoint or page attribute. Time travel changes future timestamps, **not previously stored timestamps**. Keep the read-only vocabulary dictionary separate from the personal DuckDB.
- Provide a Rust fixture that can reset an isolated personal database, set the clock, read persisted sessions and derived observations, and assert event counts. Verify the browser's actual OPFS/personal-DB connection; a mocked in-memory ledger cannot substitute for durability tests.
- Use deterministic exercise/assessor fixtures to simulate four genuinely assessed competencies, including a controlled recorded-speech evaluation result. Separately run device-level recording tests; a browser fixture cannot verify microphone or model accuracy.

## Full user journey: seven active days

1. Start with a fresh user profile, enable pedagogical recording when the setting is implemented, and populate a Word Set with ten **distinct, playable Hanzi-bearing entries**. Verify the selected set becomes enabled in Practice setup. Before adding the tenth entry, assert it is disabled and explains how many more are needed.
2. Start Practice from this Word Set. Complete a scored session containing real target-word opportunities and multiple exercise modalities. Verify the session record is committed to the personal DuckDB and the temporary UUID record is cleared only after the durable-write contract has completed.
3. Advance the injected clock to the **next local calendar day**, open another Practice session, and score further independent evidence. Repeat across seven **distinct active dates**, including at least three unique session IDs and a total of at least ten qualifying encounters for one target identity, subject to the configured session cap.
4. Include separately assessed successes for recognition, listening comprehension, spoken production, and correct sentence use. Visit `/stats`; verify the per-skill evidence and totals are sourced from stored events. Only assert an overall *proficient* label once approved per-skill success thresholds exist and all conditions are met. Before that policy is approved, assert **eligibility satisfied / overall proficiency pending**.
5. Reload the browser, navigate away and back to `/stats`, and confirm the same durable outcomes without seeding the totals directly. Verify HSK/category labels and grammar-point references can be traced back to the original scored attempts.

## Negative and boundary scenarios

| Scenario | Required assertion |
| --- | --- |
| Seven encounters on one day, or elapsed seven days with only two active dates | Not eligible: fewer than seven distinct active dates |
| Nine qualifying encounters over seven active days | Not eligible: fewer than ten encounters |
| Ten encounters in one session, even when repeated correctly | Not eligible: fewer than three sessions, with per-session qualification cap enforced |
| Ten correct Matching answers and no other modalities | Recognition evidence only; speaking, listening and sentence use remain unproven |
| Recorded audio without assessed speech result | No speaking success credited |
| Guided Sentence Builder shows correct answer and plays audio | Instructional/assisted event; no unassisted sentence-production credit |
| Incorrect attempt after previous proficiency | Affected dynamic skill moves toward review under the approved failure policy; permanent grammar course completion does not revert |
| Duplicate event, refresh, browser retry, or replayed submission | Idempotent event storage and no extra qualifying encounter |
| Disabled recording or missing consent | No durable pedagogical record is written |
| DuckDB write fails or is blocked | No false saved confirmation; recoverable error state and no silent loss |
| Edited Word Set drops below ten valid entries | Practice source disabled again; launch path rechecks eligibility |
| One word seen ten times, other nine only once | No blanket proficiency award to the other nine |
| Two identical word labels with different stable identities | No accidental aggregation across identities |
| DST transition or timezone change | Distinct-day classification follows the single documented timezone rule |
| Import/export and database reload | Totals reproduce from authoritative session/attempt evidence, not a mutable cached score |

## Separate grammar-course acceptance

Use the approved [Courses spec](../../learning-practice/courses/courses-mosaic.md) and [learning evidence policy](../../learning-practice/practice/learning-evidence.md). For each HSK grammar point, verify explanation, examples, guided practice, independently scored assessment, and retained per-point pass state. Include guided image-to-Hanzi, Hanzi writing, Character Builder, Sentence Builder with a visible/playable correct sentence, and word-bank Sentence Transformation. Confirm visible teaching hints never automatically award independent proficiency. After all required points pass, the grammar region permanently reveals; incorrect later vocabulary answers affect dynamic proficiency only, not historic course completion.

## Integration boundaries

Tests should live in the Rust E2E harness (not a new Node server), reuse the shared page objects, and isolate fixtures per scenario. Do not advance the real machine clock, bypass the actual Practice/storage flow by directly writing mastered flags, or call a simulation-only Stats URL in production. A passing test must verify results *from the personal DuckDB and public UI* and must report whether real microphone/acoustic scoring was exercised or simulated.

## full-run harness checkpoint (2026-11-12)

Two explicit Rust/Thirtyfour suites now exist: `full_run_standard` and `full_run_pedagogical`. Both use a seed from `ZIPEEK_E2E_SEED` (default `20261112`), a hard execution budget from `ZIPEEK_E2E_MAX_MINUTES` (default 60, valid 1-120), a clean landing/onboarding choice, and mandatory PNG screenshots with a JSONL screenshot manifest. The standard run captures the complete route itinerary and five seeded exercise *entry* views; the pedagogical run captures the course/practice/writing/game/Stats itinerary. Both verify persisted opt-in selection. A route visit is never reported as an answered exercise, completed session, or acquired Hanzi.

These suites have `#[ignore]` and are deliberately **non-passing** when invoked explicitly: the standard variant records the missing five scored rounds, word-set editing and AI practice finish gates; the pedagogical variant records the missing independently validated recognition producer, consent-gated personal DuckDB journal, worker verified readback, browser clock injection, and 90%-HSK Golden Dragon award. They emit `full-run-prerequisites.txt` and existing harness failure artifacts. Until these gates are implemented, do not remove `#[ignore]` or assert these journeys passed.

Run on a machine with the pinned Rust toolchain and Chrome E2E prerequisites:

```powershell
$env:ZIPEEK_E2E_SEED = '20261112'
$env:ZIPEEK_E2E_MAX_MINUTES = '60'
cargo make e2e-full-standard
cargo make e2e-full-pedagogical
```

`zipeek-pedagogy::RecognitionClock` abstracts the projection's as-of timestamp; the E2E-only `SimulatedClock` advances monotonically by 86,400,000 ms/day and has unit tests for seven-day spacing. This does **not** yet inject virtual timestamps into the browser's practice and persistence pipeline. Production exposes no simulated-time control or unlock endpoint. Simulated time will be meaningful to the full run only after the same clock is used to timestamp real scored evidence through the approved pipeline.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/testing-quality-e2e-learning-evidence-e2e.md). Dated implementation claims in that snapshot are not current build or browser verification.
