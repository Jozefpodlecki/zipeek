# Personal-data storage: implemented state and proposed database

## Current authority

The application uses `localStorage` for settings, personal Word Sets, Pinyin-course progress, exclusions and temporary UUID Practice records. The keys and backup boundary are documented in [Personal Data](../personal/personal-data-storage.md). `PracticeSessionStore` saves unfinished work under `zipeek.practice.session.v1.<uuid-v7>`; completion removes that temporary record. The the historical source unverified checkpoint introduces a dedicated OPFS DuckDB outcome ledger, gated by a default-off recording preference. This requires browser validation before durability claims.

The Settings Personal Data page implements category-based JSON backup/preview/import/delete. It also exposes isolated OPFS/snapshot diagnostics and a writable DuckDB-WASM persistence **probe**. The probe uses its own database and does not migrate or replace user Word Sets, exclusions, course progress or Practice records. See [`zipeek-db/src/wasm/probe.rs`](../../../crates/zipeek-db/src/wasm/probe.rs) and [`zipeek-web/src/services/personal_data/`](../../../crates/zipeek-web/src/services/personal_data).

## Not yet production-accepted

The inspected source describes an initial `zipeek-personal-learning-v1.duckdb` outcome table, a default-off opt-in and an exclusive Web Lock. There is still no production-accepted pedagogical-history writer, completed migration model, verified cross-tab locking and durable exercise-evidence schema beyond completed Practice outcome records. A successful OPFS probe does not imply any of those capabilities. Attempts/events in [`zipeek-exercises`](../../../crates/zipeek-exercises/src) are domain/test structures, not automatically persisted browser records.

## recognition evidence table (approved WIP, not activated)

The user explicitly approved a dedicated recognition-evidence table and Rust/WASM
adapter on 2026-11-12. The WIP adapter is implemented in
`zipeek-db/src/wasm/recognition_history.rs` and uses the existing personal OPFS
DuckDB file, the same exclusive Web Lock, and the existing DuckDB-WASM bridge.
The table is created lazily on the first explicit append:

```sql
CREATE TABLE IF NOT EXISTS recognition_evidence (
    attempt_id VARCHAR PRIMARY KEY,
    evidence_json VARCHAR NOT NULL
);
```

The versioned JSON payload includes `schema_version: 1`, `attempt_id`, a single
`character`, session `session_id` and `completed`, `occurred_at_utc_ms`,
`timezone_iana`, `timezone_offset_minutes`, `local_date`, `mistake_allowance`
(0, 1 or 2), `evaluator_id`, `assessment_version`, `correct`, `assisted`, and
`format` (tagged object: `independent_recall`, `validated_choice` with
`validation_id`, or `supporting_only`). The adapter rejects inconsistent
identities, invalid fields, oversized records/batches and conflicting duplicate
IDs. It inserts an entire batch in a transaction, checkpoints, reopens the
database and verifies the exact stored JSON by attempt ID. Exact-match retries
are idempotent. `PersonalLearningHistory::erase` now drops both learning
history tables in one transaction.

The API provides `PersonalRecognitionHistory::append_verified` and `read_all`.
The latter is bounded to two million records and is intended for replay and
verification, not repeated per-render dashboard queries. Aggregate SQL and
pagination will be needed before high-volume dashboard adoption.

**Not activated:** This adds neither opt-in collection wiring nor a production
`RecognitionEvent` serializer, browser worker, `/stats` UI or authoritative
acquisition projection. The personal DuckDB migration is additive and lazy;
existing `practice_sessions` records remain untouched until the user explicitly
requests deletion. Browser transaction/OPFS/restore/consent acceptance is
pending; do not treat source-level tests as durability proof.

## Design gates before adoption

1. Implement the approved session-first scored observations in a personal DuckDB with explicit recording consent; finalize opt-in defaults and raw audio/text retention before production collection.
2. Select versioned vocabulary identity keys that remain valid when the read-only dictionary changes; generated integer WordIds alone are not portable.
3. Define append-only attempt semantics, stable exercise/step IDs, schema versions, deletions, backup/import/rollback and a quota/retention policy.
4. Prove write, checkpoint, restart, cross-tab lock, failure recovery, export/import and corruption handling in real browsers for the approved DuckDB-WASM package.
5. Only after durable-write verification, show a success confirmation; when disabled show a privacy notice, and on real write errors show an explicit failure.

Historical exploratory steps and proposed tables are recorded in [completed history](../../archive-history/completed/) and [release history](../../archive-history/RELEASE_HISTORY.md). They must not be read as a deployed learner database specification.

## Approved pedagogical evidence direction (not implemented)

Persist Practice sessions as the evidence authority; derive attempt and skill observations linked to their original session/step, normalized word/Hanzi IDs, exercise kind, HSK/category labels and assessed grammar point. The approved eligibility floor is **seven distinct active local calendar days**, **at least ten qualifying scored encounters over three or more distinct Practice sessions**, a per-session counting cap, and separately successful evidence for recognition, listening, assessed speaking and sentence use. The per-session cap, exact modality thresholds, learner timezone policy and audio privacy/retention remain undecided; never grant mastery automatically on the tenth encounter. Inject a test clock so E2E can advance dates without altering the OS clock. Preserve a distinct irreversible grammar course-completion record once all required grammar-point assessments pass. See [learning evidence](../../learning-practice/practice/learning-evidence.md) and [future Courses spec](../../learning-practice/courses/courses-mosaic.md).

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/dictionary-data-schema-personal-data.md). Dated implementation claims in that snapshot are not current build or browser verification.
