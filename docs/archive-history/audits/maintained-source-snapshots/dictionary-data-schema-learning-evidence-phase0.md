# Personal learning evidence: phase 0 and independent foundations

**Phase-zero design record, not a deployed personal database.** The historical WASM 1.32.0 validation constraint is superseded as an installed-runtime fact by [runtime](../../../dictionary-data/schema/runtime.md); policy approval remains unresolved. See [personal data](../../../dictionary-data/schema/personal-data.md) for current storage authority and migration gates.

Status: design checkpoint, not an active writable DuckDB schema. The current personal learning records remain browser-local as described in `personal-data.md`. No user database was created or reset.

This checkpoint follows `Zipeek_Personal_Learning_Schema_Implementation_Spec(1).md`, but does not activate its DDL. The proposed tables must first be checked against the pinned DuckDB-WASM 1.32.0 runtime, including UUID, TIMESTAMPTZ, foreign-key and uniqueness behavior, OPFS persistence, export/import, locking and recovery.

## Source observations

- `crates/zipeek-exercises/src/evidence.rs` currently exposes six historical evidence categories: recognition, listening, pronunciation, recall, writing and usage. These are **not** equivalent to the spec's four independent vocabulary competency axes and two grammar assessment modes. Do not blend results from the older ledger into verified mastery.
- `crates/zipeek-db-gen/src/schema.sql` identifies dictionary words by generated integer IDs and a `(simplified, pinyin)` unique pair. Neither is sufficient to prove stable, reading- and sense-aware identity across regenerated dictionaries.
- `crates/zipeek-db/src/wasm/probe.rs` exercises a disposable OPFS DuckDB probe. A successful one-session probe does not establish browser-restart durability or suitability as the live personal-record store.
- R258 UUID practice sessions are localStorage records. A session record is not an authoritative, target-specific assessment event log.

## Implemented, not yet wired into persisted user records

The `zipeek-exercises::learning` module contains typed attempts and independent vocabulary/grammar target observations, a conservative validity gate, an in-memory append-once journal, explicit correction events, and raw per-skill observed metrics. It refuses to treat pinyin-only transcription as Hanzi typing, copying as recall, playback or self-report as acoustic pronunciation, and non-obligatory grammar patterns as assessed. It does not award learned/mastered statuses or produce scientifically unverified retention probabilities.

The new code neither replaces the existing `evidence.rs` ledger nor automatically promotes saved practice sessions into skill mastery. Actual assessment integration requires provenance of the stimulus, stable dictionary lexical identity, evaluator/rubric versions and privacy decisions.

## Product-owner decisions required before schema activation

1. Do all four applicable skills define the final `fully_learned` claim, with verified pronunciation unavailable until trustworthy scoring exists? Alternative: show only per-skill states and `receptively_learned` initially.
2. Is Hanzi input through an IME sufficient to assess practical typing, with handwriting treated as optional additional evidence?
3. Does graduation require seven **distinct successful local dates**, a delayed success separated by at least seven days, and an optional 14-day higher tier? Specify what constitutes delayed retention when daily reviews continue uninterrupted.
4. Will pronunciation be objectively scored by a qualified model/human or remain explicitly `self_reported` / `not_assessed` initially?
5. Which authored grammar definitions and evaluator rubrics are the source of truth for obligatorily elicited targets and accepted alternatives?
6. What opt-in, expiry and export/deletion rules apply to recordings, strokes, raw typed text and generated sentence responses?

The user must approve these decisions and the sense-aware lexical identity/reconciliation policy before mastery production logic or a live personal DuckDB begins writing learning records. No backward-compatibility code is proposed. A reset of existing personal data would require a separately approved, explicit user action.

## Next implementation gates

1. Confirm decisions and stable lexical keys, versioned context-suitability and acceptance policies. Add deterministic reconciliation fixtures for homographs and changed dictionary IDs.
2. Implement event schema and transactionally insert an attempt, all observations and any correction with a stable submission key. Test duplicate concurrent submissions and partial failure against actual pinned DuckDB-WASM.
3. Prove writable OPFS durability and cross-tab coordination, export/import and error recovery before making a personal DB authoritative. Keep current user-owned data untouched throughout verification.
4. Derive reproducible projections and scheduling from committed events. Display four independently assessed skill states; never infer verified pronunciation or Hanzi typing from unrelated exercises.
