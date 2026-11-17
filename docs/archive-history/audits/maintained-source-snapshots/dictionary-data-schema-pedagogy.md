# Pedagogy: recognition acquisition foundation (R333 WIP)

## Status and authority

This design records the user's 2026-11-11 decisions for **historical Hanzi recognition acquisition**. The source-only R333 checkpoint introduces `zipeek-pedagogy`, a pure Rust domain crate with deterministic, in-memory projections. It does **not** activate automatic assessment capture, a persistent personal-data authority, a worker, `/stats`, or dragon awards in the UI. Those remain separately gated.

It deliberately does not replace R259 `zipeek-exercises::learning::MasteryPolicy`, which continues to represent its older multi-skill vocabulary projection until an approved migration of domain ownership and reconciliation of its separate definitions. Dragon acquisition and full multi-skill proficiency are not interchangeable.

## Recognition acquisition contract

- Each individual Hanzi independently earns a historical recognition award after **seven consecutive qualifying local dates**.
- Each qualifying day requires **ten correct, eligible, unassisted recognition observations**, across **at least three completed sessions**. At most **four** correct observations from any one session count toward the daily ten.
- The maximum mistakes before daily qualification is **0, 1, or 2**, default 0. The allowance is frozen in the submitted evidence for each item and local date. An inconsistent allowance within one day is an error, not a retroactive adjustment.
- An additional eligible incorrect observation that exceeds the allowance **fails that day immediately** and resets the active acquisition streak. Earlier attempts are not deleted.
- Once a day qualifies it **locks immediately**. Later mistakes that day do not alter that qualification; they remain available for subsequent retention analysis.
- Qualifying days must be **consecutive local calendar dates** with **at least 24 elapsed hours** between the moments each day qualifies. A missed date resets the active streak.
- Later recall failures can change a separate retention status. They do not revoke the timestamp of a historical acquisition award.
- The full HSK 1-9 learning pool is accessible regardless of acquisition progress. Dragon achievements count distinct acquired Hanzi in the selected dictionary-HSK target; words and readings have independent evidence identities and do not automatically award character recognition.

## Evidence boundary

All games may submit scored observations, but game scoring is not itself an acquisition decision. Recognition events have stable attempt/session identities, submission time, local date and zone context, assessment version and evaluator provenance. Invalid dates, inconsistent local dates/UTC offsets, duplicate IDs with conflicting content, and evidence from the future are rejected.

Independent recognition assessments qualify when unassisted and complete. Multiple-choice recognition requires a validated assessment proof from a separate gate: the R333 `ValidatedChoice` type carries a non-empty validation identifier, but **this crate cannot verify its content**. This is a provisional domain API, not permission for production games to fabricate validation IDs. Hinted, incomplete, unvalidated-choice and general exposure events are recorded as supporting-only evidence. No production game is wired to this API yet.

**Outstanding correctness requirements before persistent mastery:** define the authoritative choice-validation gate (prompt/distractor diversity and independent recall), fixed-zone behaviour across travel, trustworthy item identity through dictionary refresh, recovery of clock anomalies, and atomic persistence with one active tab. R333's projection must not be the sole source of durable mastery until those gates pass.

## Crate boundaries

`zipeek-pedagogy` is independent of Yew, JS, DuckDB and all third-party crates. `zipeek-exercises` continues to own game outcomes and the existing R259 learning contract. The future adapter validates outcomes and passes independent evidence to pedagogy. The existing database crate owns authoritative writes after persistence acceptance. A Rust/WASM browser worker is an execution host in existing browser infrastructure, not a reason to put learning rules in `zipeek-worker`.

The new crate includes its own pure calendar-date validation because no dependency direction between the existing learning calendar module and this new domain was approved for R333. Consolidating identical calendar functionality is a follow-up refactoring task before production integration.

## Scheduled stages

1. **R333:** pure evidence and recognition projection plus deterministic tests. WIP until compiled and reviewed.
2. **Later:** validated game-to-evidence adapter, historical policy readback, OPFS DuckDB durability/atomicity, and consent gating. Schema changes require separate approval.
3. **Later:** background WASM worker host, pending-work recovery, single-tab ownership and Settings diagnostics.
4. **Later:** `/stats`, separate skill and retention views, dragon colour progression, HSK-target denominator and accessible UI.
5. **Later:** optional advanced AI insights with external-transfer consent and independently validated generated exercises.

Do not imply the app is collecting pedagogical data, awarding character mastery, or using a running analysis worker until these integrations and tests are completed.

## R334 verified evidence-journal boundary (WIP)

`zipeek-pedagogy::EvidenceJournal` is a synchronous pure-Rust port for reading
recorded recognition events and atomically appending a batch. `append_and_verify`
first validates candidate evidence against the existing journal, requests the
append, then rereads stored records and refuses to return a learning projection
unless the candidate events are present and unchanged. `project_committed`
computes only from journal readback. Replayed identical attempt identifiers are
idempotent; conflicting IDs are rejected before append.

The port contract requires an adapter to provide **atomic, durable** append,
consistent readback, and exclusive single-writer coordination. Those
requirements are **not** implemented by this port itself. Test doubles model
write failures, discarded writes, altered readbacks, and duplicate retries. The
existing Practice session JSON table is not automatically equivalent to a
character-level assessment event journal, and cannot be used without approved
schema and provenance work.

No worker, browser persistence adapter, production capture, award writes, or
`/stats` view was activated in R334. The independent choice-validation gate,
learner timezone authority, consent controls, and authoritative OPFS durability
remain prerequisites to production integration.
