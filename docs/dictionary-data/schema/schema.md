# Generated dictionary schema

**Implementation contract:** [`crates/zipeek-db-gen/src/schema.sql`](../../../crates/zipeek-db-gen/src/schema.sql) is the exact DDL. The installed manifest's `tables`, `columns`, `constraints`, `indexes` and `rows` report the generated artifact. This document summarizes the implementation; it does not replace the SQL.

## Compatibility projection: `words`

The ten original display/query columns still exist: `id`, `simplified`, `traditional`, `pinyin`, `pinyin_numeric`, `english`, `part_of_speech`, `hsk2_level`, `hsk3_level`, and `frequency`. `id` is a generated integer PK, `(simplified, pinyin)` is unique, and HSK level indexes are `idx_words_hsk2` and `idx_words_hsk3`. `english` and `part_of_speech` are compatibility/display fields; do not assume they fully model sense provenance or taxonomy.

HSK 2.0 permits levels 1-6; HSK 3.0 stores combined 7-9 as 7. A nullable HSK level is valid for the **extra CC-CEDICT readings**: the old rule that every word must belong to an HSK set is obsolete.

## Normalized source and enrichment relations

The generator DDL defines `dictionary_versions`, `vocabulary_identities`, `characters`, `character_readings`, `word_characters`, `word_hsk_membership`, `word_pos_membership`, `word_senses`, `sense_display`, `sense_relations`, `classification_runs`, `sense_pos`, `sense_categories`, `parts_of_speech` and `categories`. The last two are controlled seed/taxonomy tables. **Installed-manifest evidence:** `character_readings` is present in the top-level table list. This verifies the manifest declaration, not the physical contents of a database binary that is absent from the source archive. Enrichment is not inferred to be complete merely because the tables exist.

- `character_readings` stores source-explicit readings separately from vocabulary-word characters and supports polyphonic characters.
- `vocabulary_identities` records candidate keys/status; it does **not** establish portable learner WordIds across regeneration.
- `word_senses` and relation/category tables distinguish provenance and sense-level metadata from the legacy pipe-separated English projection.
- `sense_display.sense_kind` is the generator-owned, non-null classification (`lexical`, `usage_reference`, `cross_reference`). The source `word_senses.gloss` remains unchanged. A usage-only pronunciation such as 上 shǎng is retained for lookup but is not an ordinary meaning-recognition exercise candidate. This column requires a completely regenerated dictionary; there is no migration.
- `word_hsk_membership` preserves HSK associations independently of the compatibility level columns.

See [`schema.sql`](../../../crates/zipeek-db-gen/src/schema.sql) for each key, foreign key and check constraint. The prior single-table/HSK-only and “no CHECK constraints” declarations are historical plans and must not be applied to the installed database.

## Scope and verification

[`generator.rs`](../../../crates/zipeek-db-gen/src/generator.rs) imports HSK and expands with verified CC-CEDICT. [`artifact/writer.rs`](../../../crates/zipeek-db-gen/src/artifact/writer.rs) copies normalized tables and verifies a reopened snapshot before publication. The installed manifest currently reports **120,970 `words` rows**; see [database index](db.md) for exact installed metadata. Generated artifacts are replaceable read-only data. Personal Word Sets, temporary Practice progress and any future evidence ledger are separate browser-owned data.

Proposals for further FTS, index or learning schema redesign belong in [schema review](review.md) and [personal data](personal-data.md), not here as already-implemented columns.
