# Database and artifacts

Approved source, schema, generator, runtime and artifact decisions. Each entry
links back to the [index](index.md) and to its owning document. Bodies are the
approved requirement as recorded on the date shown; they do not imply the
feature is implemented.

---

## D06 Sources and licensing

**Status:** Approved 2026-10-26; original source selection superseded by D16.
**Owner:** [DB](../architecture/schema/DB.md), [PRODUCT](../product/PRODUCT.md),
[README](../../../README.md).
**Index row:** [D06](index.md#approved-decisions).

Retain CC-CEDICT attribution and CC BY-SA 4.0 derived-data terms; preserve source
notices; exclude Wiktionary, the noncommercial Anki deck, and any other
unlicensed source.

The original D06 selection (CC-CEDICT + leonsilicon HSK packages) is superseded
by D16. The leonsilicon packages were audited and found to declare no licence;
they are **not used** and must not be fetched or redistributed as substitutes.
Their audit remains in COMPLETED history as evidence, not as an approved source.

Required attribution, present in README and the visible site footer:

> Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed CC BY-SA 4.0.

Generated dictionary data derived from CC-CEDICT is distributed under CC BY-SA 4.0
with transformations documented. Application code retains its own licence.

CC-CEDICT's homepage and download page both currently declare 4.0. A historical
3.0-vs-4.0 discrepancy was reported by the user and is preserved as a note; do
not relabel an older snapshot whose terms differ.

---

## D07 Physical schema and pool

**Status:** Approved 2026-10-26; import rules amended by D16 and D18.
**Owner:** [DB](../architecture/schema/DB.md),
[ARCHITECTURE](../architecture/ARCHITECTURE.md),
[TESTING](../testing/provenance.md).
**Index row:** [D07](index.md#approved-decisions).

One denormalized `words` table containing only the HSK 2.0/3.0 union, supplied
by D16's primary source with empty-gloss fallback from CC-CEDICT.

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

Integer surrogate PK assigned by db-gen, not automatic identity.

simplified is required; traditional is nullable (first selected source form).

pinyin (display) is required; pinyin_numeric preserves the original numeric
form alongside.

english is pipe-separated glosses; NULL when no enrichment is available.

part_of_speech is nullable comma-separated canonical lowercase tokens; NULL
when unavailable.

hsk2_level is NULL or 1-6; hsk3_level is NULL, 1-6, or 7 for the combined
7-9 band.

frequency is a nullable integer rank; lower means more common.

UNIQUE (simplified, pinyin) enforces display-key uniqueness; numeric/display
correspondence is validated during import (D16/D18).

No normalization or separate membership table in v1. Source joining happens in
db-gen, not in runtime queries.

Index policy: DuckDB creates ART indexes for PK and the UNIQUE constraint
automatically. Add exactly two ordinary HSK indexes:

sql
CREATE INDEX idx_words_hsk3 ON words (hsk3_level);
CREATE INDEX idx_words_hsk2 ON words (hsk2_level);
Partial indexes (WHERE ... IS NOT NULL) are rejected by DuckDB and are not used.

D08 DuckDB-WASM version
Status: Approved 2026-10-26.
Owner: AGENTS, DB.
Index row: D08.

Pin the npm package exactly to @duckdb/duckdb-wasm@1.32.0. No range, latest,
next, or development build substitution.

The pin is enforced in the web manifest, lockfile, and the A06 pre-build hook
which verifies the installed version before packaging worker/WASM/Arrow assets.

If integration exposes a genuine wasm toolchain compatibility blocker, stop
and report the exact versions, reproducing commands, and error evidence. Do
not silently switch to another release or tag.

The pinned package reports engine v1.4.3 in the browser. This is the runtime
fact; native generation uses a different engine (D23).

Rationale (historical): the DuckDB team's OPFS article (2026-09-18) recommended
pinning 1.32.0 because latest had an OPFS persistence bug. Zipeek does not use
OPFS persistence; the pin is preserved regardless.

D16 Primary import and IDs
Status: Approved 2026-10-26; further specified by D18.
Owner: DB, TESTING,
PRODUCT.
Index row: D16.

Pin jelleverheyen/hsk-vocabulary complete.json at commit
a66fd30b9580da2c2af7eb19e4b9d8099a29c061 (2025-02-28). CC-CEDICT is consulted
only for empty glosses. The superseded leonsilicon packages are not used.

Source counts (audit): 11,494 entries, 12,655 forms; 12,188 distinct
case-sensitive (simplified, numeric_pinyin) reading keys.

Import key: (simplified, numeric_pinyin); expand distinct readings from
forms[]; retain the first form's traditional value in source-array order,
warning when multiple forms share the key.

Display-key validation: UNIQUE (simplified, pinyin) is enforced; numeric
and display representations are distinct keys. Do not silently normalize or
collapse readings if a future input violates the correspondence.

IDs: ROW_NUMBER() OVER (ORDER BY simplified, numeric_pinyin), cast to
INTEGER, snapshot-local. IDs may shift across regenerations; old shared URLs
or exclusions may identify a different word after regeneration.

Level mapping: old-1..old-6 → hsk2_level; new-1..new-6 →
hsk3_level; both new-7 and new-7+ → hsk3_level = 7; deduplicate
identical tags; conflicting within-system levels pick the lowest with a
warning; unknown tokens logged and skipped.

Exact text: preserve simplified, traditional, pinyin, pinyin_numeric
as supplied; do not rederive display pinyin or normalize stored hanzi.

Glosses: first form's nonempty meanings in source order, trimmed, casing
preserved, literal ASCII | inside a gloss replaced with U+FF5C and logged,
joined with ASCII |. Fallback to CC-CEDICT only when the resulting list is
empty; ignore variant of entries; unresolved → NULL English with a warning.

PoS: map source codes per D18's 42-code case-insensitive table; reject the
whole entry (all forms) on any unknown code before grouping.

Frequency: preserve when provided; NULL otherwise.

Missing labels: drop and log rows with missing required pinyin; missing
English is allowed for pinyin-hanzi but not for English-bearing variants.

First-gloss exercise labels: the first serialized gloss is the label in
both English-bearing variants; right-label uniqueness uses exactly this gloss.

D18 Source comparison and serialization
Status: Approved 2026-10-26.
Owner: DB, TESTING,
PRODUCT.
Index row: D18.

Full contract lives in DB §D18.
Summary:

Comparison-only pinyin normalization is used only for right-label duplicate
checking and HSK-to-CC-CEDICT reading-key matching. Never rewrites stored
source text; never merges distinct exact numeric source keys.

Lowercase, map ü / u: / v → v, ignore whitespace. Convert tone-marked
display input to numeric tones. Preserve numbered tones 1-4. Strip a syllable's
trailing neutral 5 only when the preceding letters contain a vowel; preserve
r5/ng5.

Gloss serialization: trim nonempty meanings, preserve casing/order,
replace literal ASCII | with U+FF5C, join with ASCII |. Do not substitute /.

42-code PoS mapping: exact case-insensitive table; whole-entry unknown-code
rejection; Mg/Rg case aliases; q → classifier, y → modal_particle.

Numeric/display key conflict: fail generation and log both source rows if
two exact numeric keys produce the same display key. Do not silently select one.

Seed eligibility: a nonexcluded seed lacking English is ineligible for
eng-pinyin and hanzi-eng; use D15's deterministic recovery. pinyin-hanzi
does not require English. The same eligibility applies to D20 MCQ variants
with English prompts.

Stored transcriptions remain byte-for-byte source apart from the explicit gloss
delimiter substitution.

D20 MCQ practice
Status: Approved 2026-10-26.
Owner: PRODUCT, UX,
TESTING.
Index row: D20.

Three variants at /practice/mcq/<variant>:

Variant	Prompt	Answer label
hanzi-to-meaning	Simplified hanzi	First English gloss
meaning-to-hanzi	First English gloss	Simplified hanzi
meaning-to-pinyin	First English gloss	Display pinyin
One prompt, exactly three options: one correct answer + two distractors (N=2).

Apply D02/D12 pool widening and visible-answer uniqueness. D18 English
eligibility applies to all three variants.

Endless and Fixed; Fixed counts 5/10/20, default 10.

Offset advances by one per question; Fixed progress is independent of offset.

D19 fixes MCQ prompt selection, two distractor samples, and final option
shuffle on one continuous RNG stream.

UI feedback timing, number-key hints, and button styling are UX-MCQ01 details
owned by A22, not by this decision.

D21 Generator dependencies, sources, and engine guard
Status: Approved 2026-10-26; engine identity superseded by D23.
Owner: DB, TESTING.
Index row: D21.

Approved dependency/removal plan from A24:

Remove: reqwest, serde_plain, tokio, uuid.

Pin: serde =1.0.229 (derive), serde_json =1.0.154, thiserror =2.0.21,
tracing =0.1.44, tracing-subscriber =0.3.23 (env-filter), sha2 =0.10.9,
toml =0.9.8 (std/serde/parse).

Retain local path dependency on zipeek-core.

The bundled duckdb =1.4.3 initial plan was superseded by D23's neo restoration;
this decision remains recorded for the dependency/removal set.

Engine-guard intent: at generator invocation query SELECT version() and log
it, failing clearly on mismatch, before any import/output. D23 updates the
required version from v1.4.3 to v2.0.0 and adds the mandatory v1.4.0 storage
compatibility. This is generation-time validation, not an automatic Cargo
build or generation hook.

Sources preserved with checksums and -text Git attributes:

crates/db-gen/data/hsk-complete-a66fd30.json (primary, SHA-256
fc474be223d2196b13bfc7c13e10054f1793c2f840eb1287dfc9620fa0d507d2).

crates/db-gen/data/cc-cedict-2026-10-26.u8 (fallback, SHA-256
3d8967fe566bc23a74f8ba008d38ac83269a8c7550bd7dc48218fc9bdf284885).

Notices preserved beside inputs; MANIFEST.toml records provenance and hashes.

The official MDBG download URL moves. Subsequent acquisition must reject differing
bytes; reuse the approved preserved snapshot rather than silently updating.

D22 Logical database reproducibility
Status: Approved 2026-10-26.
Owner: DB, TESTING,
CODING_STANDARDS.
Index row: D22.

Identical verified inputs and pinned tooling must reproduce:

Ordered logical rows (every column, ORDER BY id ASC).

IDs.

Schema and index definitions.

Import/drop diagnostics (stable, reproducible).

Record SHA-256 for each final artifact as an integrity check. Byte-identical
.duckdb files across runs are not required. Do not patch or mask physical
database bytes to force equality.

Rationale: DuckDB's file writer can generate random database identifiers for
encrypted or storage-version ≥ 67 files, so physical bytes may differ even
with identical logical content. A05 verifies logical equality across independent
runs; file hashes are per-artifact integrity evidence, not reproducibility proof.

D23 Native neo engine and compatible storage
Status: Approved 2026-10-26; supersedes D21's earlier v1.4.3 native guard.
Owner: DB, TESTING,
AGENTS.
Index row: D23.

Keep restored duckdb-neo at Git revision
4acbea84794582c4cd962ee971a3d7a5dcb72e39. Require:

Actual native engine v2.0.0 at generator invocation.

Storage compatibility v1.4.0 set before output creation.

Generator startup:

Query and log SELECT version().

Fail clearly on anything other than v2.0.0; on query failure, exit non-zero.

Set storage_compatibility_version = v1.4.0.

Query and log the setting; fail on mismatch.

Return the configured in-memory connection for later output work.

No output is created before these checks pass. The earlier native v1.4.3 guard
is superseded.

WASM runtime stays exactly @duckdb/duckdb-wasm@1.32.0 (engine v1.4.3). Default
neo output (storage version 999) is rejected by the pinned WASM; only explicit
v1.4.0 storage is readable. Synthetic fixture evidence passed in Node 24.17.0
before D23 was approved; A06 later validated the actual full dictionary in
browser workers and the /zipeek/ build. Future regenerated artifacts must
repeat compatibility checks.