# R259 - dictionary identity and learning evidence foundations

Status: **source candidate; Rust compilation, generator regeneration, and browser validation pending.**

## Schema proposal assessment

| Proposal | R259 decision | Rationale |
| --- | --- | --- |
| `dictionary_versions` | Adopt snapshot fingerprint metadata | The fingerprint describes the generated lexical content, not a backward-compatibility migration version. |
| `vocabulary_identities` | Adopt **unresolved candidate** records only | Same surface form and reading do not establish same lexical sense across dictionary revisions. No verified stable keys are generated. |
| `characters` | Adopt Unicode scalar inventory | This records characters occurring in simplified word forms; readings and radical properties require a verified separate source. |
| `word_characters` | Adopt ordered character occurrences | Preserves repeats and position, even where words include non-Hanzi characters. |
| `example_sentences` / `sentence_vocabulary` | Defer | No approved licensed sentence dataset with sense-level provenance. AI-generated examples cannot silently become catalog evidence. |
| `word_relations` | Defer generic table | `sense_relations` already models explicit source annotations. Synonym/antonym edges need additional verified sources. |
| `character_components` | Defer | Requires a licensed structural dataset and accepted decomposition policy. |
| `classifiers` / `word_classifiers` | Defer | Current PoS membership identifies classifier words, not reliable noun-to-classifier links. |
| `word_frequency_sources` | Defer | Existing scalar frequency has no verified corpus-by-corpus provenance for a multi-source table. |

## Identity safety

The generator now populates `dictionary_versions` using a deterministic content digest and inserts a Unicode character index. The `vocabulary_identities` records have `identity_status = 'unresolved'`. Their candidate keys use a length-bearing spelling and numeric-reading representation, **not** a stable sense ID. They are not eligible to join personal mastery data across dictionary refreshes.

The pure learning domain supplies `LexicalIdentity` with explicit source-lexeme, reading, and sense keys; `reconcile()` classifies resolved, missing, and ambiguous references without falling back to word indexes. Only externally verified identities may be registered for assessment policies. A future generator change must trace stable source identifiers and sense reconciliation before emitting `verified` records.

## Learning evidence and mastery

The R258 checkpoint's in-memory attempt journal is retained and extended. Identical submission retries are no-ops; conflicting retries are rejected. Corrections are append-only and a single observation cannot be corrected twice without a new audited event design.

R259 adds an independent `mastery-v1` projection with **seven distinct successful local dates** plus at least one pair of consecutive successful review dates separated by **at least seven days**. Ordinary consecutive daily practice does not satisfy this delayed test. Qualified failure following the last success changes that **skill only** to `needs_review`; missed attendance is not a failed review. A conservative `scheduler-v1` suggests review intervals independently; its interval table is a configurable product heuristic, not calibrated learning science.

The four vocabulary competencies are separate. `receptively_learned` requires recognition and listening; `fully_learned` requires verified success across all applicable recognition, listening, pronunciation and typing. Inapplicability requires a versioned content policy, not an inferred pass. No spoken self-report or pinyin-only transcription establishes verified pronunciation or Hanzi typing.

The event journal, evidence gate and projections are still **pure Rust / in-memory**. R259 does not enable automatic production event capture, a writable personal DuckDB, mastery badges in the UI, or a new data authority.

## No migration or silent reset

This project permits replacing the **generated dictionary artifact** outright. No legacy-table compatibility reader or data migration is introduced. However the existing installed `hanzi-*.duckdb` files are retained until the user manually regenerates and installs a validated artifact. The browser-local Word Sets, preferences and practice session records are not deleted. Do not reset learner-owned storage as part of dictionary regeneration.

After local Rust checks, run `cargo make db-generate` and review the new artifact before `cargo make db-install`. Then run `cargo make serve`, exercise Browse and Practice, and verify the generated schema in the installed dictionary. The full user-owned DuckDB requires separate OPFS close/reopen, restart, cross-tab, export/import, quota and crash-recovery acceptance before becoming authoritative.

## Release gates

1. `cargo fmt --all -- --check`
2. `cargo test -p zipeek-exercises --locked`
3. `cargo test -p zipeek-db-gen --locked`
4. `cargo test --workspace --locked`
5. `cargo check -p zipeek-web --target wasm32-unknown-unknown --locked`
6. `cargo make db-generate`, `cargo make db-install`, `cargo make serve`
7. Native/WASM generator equivalence, identity ambiguity fixtures, Chrome browsing, and personal-storage retention

No Rust compiler, native DuckDB generator, or managed Chrome was available in the packaging environment; these gates remain unverified.

## R270 input integrity follow-up (2026-11-08)

Invalid ISO local review dates are now rejected at the evidence qualification and journal append boundaries rather than counted as observations and discarded only when raw projections run. Verified lexical identities reject leading/trailing whitespace and NUL-bearing keys; the policy catalog rejects whitespace-only assessment version strings. This does not activate a persisted evidence journal or mastery publication from the UUID Practice runtime. The existing browser-local schema and model remain unchanged.
