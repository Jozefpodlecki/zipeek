# Schema

The `words` table, columns, HSK abstraction, and index policy. Part of the
[database documentation](DB.md). Approved schema in
[D07](../../process/decisions/database-and-artifacts.md#d07-physical-schema-and-pool);
HSK abstraction in [D11](../../process/decisions/foundations.md#d11-hsk-abstraction).

## Table (D07: approved; implemented in A02)

Use a **single physical table `words`**, with no normalization or separate
membership table for v1. Source joining happens in db-gen, not in runtime queries.
This supersedes the original generic `hanzi` and `hsk_level` field names with
explicit forms and system-specific columns.

The pool is **only the union of the HSK 2.0 and HSK 3.0 word lists**, provided by
the pinned primary dataset, with CC-CEDICT only for empty-gloss fallback. Do not
import the entire dictionary as extra rows. At least one HSK membership must be
present for every generated row. The approximate 11,000 entries are a planning
estimate; count actual output after deduplication and reading expansion. Word of
the Day and Random's full pool mean this table.

```sql
CREATE TABLE words (
    id              INTEGER PRIMARY KEY,
    simplified      VARCHAR NOT NULL,
    traditional     VARCHAR,
    pinyin          VARCHAR NOT NULL,
    pinyin_numeric  VARCHAR,
    english         VARCHAR,
    part_of_speech  VARCHAR,
    hsk2_level      INTEGER,
    hsk3_level      INTEGER,
    frequency       INTEGER,
    CONSTRAINT uq_word UNIQUE (simplified, pinyin)
);

Column contract
Column	Contract
id	Surrogate integer key assigned by db-gen, not automatic identity generation
simplified	Primary entry's unchanged simplified headword; required
traditional	First selected source form's unchanged traditional text
pinyin	Required source display transcription, preserved exactly
pinyin_numeric	Original primary numeric transcription, preserved alongside display pinyin for numeric browse queries
english	Pipe-separated glosses, e.g. to eat|to have a meal; NULL for missing enrichment
part_of_speech	NULL if unknown; otherwise comma-separated canonical lowercase enum strings, e.g. verb,noun
hsk2_level	NULL if absent from HSK 2.0, otherwise 1-6
hsk3_level	NULL if absent from HSK 3.0, otherwise 1-6 or 7 for the combined 7-9 band
frequency	Nullable integer rank; lower means more common; NULL if no verified source rank
The display-key constraint above remains unchanged. D16 groups imports and orders
ID assignment by (simplified, numeric_pinyin); validate its correspondence to
the display-key constraint. Distinct readings remain separate rows. Preserve both
pinyin forms, source casing, and NULL metadata rather than inventing values.

HSK abstraction (D11)
The HSK abstraction uses the two nullable columns above:

HSK 2.0 and HSK 3.0 are distinct lists, not successive app/database versions.

HSK 2.0 has levels 1-6; HSK 3.0 has levels 1-6 plus advanced tier 7-9.

A word may have memberships in both systems at different levels, or only one.

Preserve the source's membership for each system; do not infer membership or
convert a level from one system to the other.

HSK 3.0's integer value 7 = combined advanced band (7–9); levels 8 and 9 do
not exist as separate stored values. The domain variant is
Hsk3Level::SevenToNine, mapped to 7 in storage and canonical URLs. This defines
the logical value, stored in D07's hsk3_level column. HSK 2.0 permits only 1-6.
Accepting the URL alias 7-9 must normalize to the same value 7.

Represent system and level/tier separately in domain types while mapping to
hsk2_level and hsk3_level in this one table. Query filtering and generation
select the appropriate column for the requested system. D16 resolves conflicting
within-system levels to the lowest value with a warning; identical repeated tags
are deduplicated. Do not add a membership table.

Index policy
DuckDB automatically creates ART indexes for both the primary key and the
UNIQUE (simplified, pinyin) constraint; do not add duplicate indexes for them.
See DuckDB constraints.

Use exactly two additional ordinary indexes, as clarified by the user:

sql
CREATE INDEX idx_words_hsk3 ON words (hsk3_level);
CREATE INDEX idx_words_hsk2 ON words (hsk2_level);
The original proposal included partial-index WHERE ... IS NOT NULL predicates.
DuckDB's official regression tests reject those with
Creating partial indexes is not supported currently, verified in
v1.3.2
and v1.4.4.
The approved ordinary indexes replace that proposal; no runtime version change
is needed to work around it.

Keep PoS and pinyin filtering as scans; avoid further indexes without evidence.
Measure actual row count, database size, browser latency, and relevant query
plans. The 11k estimate and single-table simplicity are not measured speed claims.

What is not in the schema
No normalized membership table.

No gloss table; english is pipe-separated in the row.

No PoS table; part_of_speech is comma-separated canonical tokens.

No CHECK constraints beyond the declared NOT NULL/UNIQUE/PK. HSK ranges,
at-least-one-membership and text policies are enforced by the importer and
typed domain conversions, not the DDL.

No versioning column, migration table, or app version table (D10).

Related
Import rules that produce rows: import-rules.md.

PoS token vocabulary: pos-mapping.md.

Runtime queries that read the table: queries.md.