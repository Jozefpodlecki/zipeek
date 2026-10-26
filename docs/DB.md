# Database

## Requirements and current state

The planned runtime database is a committed, pregenerated file at
`web/static/hanzi.duckdb`, read-only through DuckDB-WASM in the browser.
The database file is absent. `crates/db-gen` exists as workspace package `db-gen`
with a generation-time native-engine guard. The user restored duckdb-neo at
Git revision 4acbea84794582c4cd962ee971a3d7a5dcb72e39 after the bundled
1.4.3 build failed. Other D21 dependencies are applied and
checksum-identified inputs/notices are preserved in crates/db-gen/data. A26
implements typed manifest loading and verifies every source/notice file
(size/SHA-256) before preparing the guarded in-memory database connection.
A02 creates the approved empty schema and two HSK indexes in memory. Source
parsing/import and artifact generation remain absent.

Database generation is a manual native operation, separate from web builds.
Never hand-edit a DuckDB file. Regenerate it after an approved source or schema
change and include the generated output in that change.

The app is in development: breaking data changes are acceptable and affected
databases are regenerated. There is no application/database versioning or
backward-compatibility requirement, and no obligation to migrate old generated
files. Source provenance, licensing, and dependency version controls still apply.

## Sources (D16 supersedes D06's original input selection)

### Primary: pinned HSK vocabulary snapshot

Use [jelleverheyen/hsk-vocabulary](https://github.com/jelleverheyen/hsk-vocabulary)
`complete.json` as the primary source for the HSK-union word pool, forms, levels,
PoS, frequency, pinyin, and English. CC-CEDICT is only a gloss fallback; do not
build the pool from it or import the previously selected leonsilicon packages.

Pinned commit: **`a66fd30b9580da2c2af7eb19e4b9d8099a29c061`** (2025-02-28).
Use the [immutable JSON URL](https://raw.githubusercontent.com/jelleverheyen/hsk-vocabulary/a66fd30b9580da2c2af7eb19e4b9d8099a29c061/complete.json),
not a moving branch URL. Audit on 2026-10-26 found 11,494 entries and 12,655 forms,
with no missing display/numeric pinyin at the form level. These are source counts,
not an already-generated database. Case-sensitive reading-key grouping yields
12,188 distinct `(simplified, numeric_pinyin)` keys; 394 keys have repeated forms.

The pinned [LICENSE](https://github.com/jelleverheyen/hsk-vocabulary/blob/a66fd30b9580da2c2af7eb19e4b9d8099a29c061/LICENSE)
declares MIT with a 2023 Yanis Zafirópulos notice; its README carries a 2025 notice.
Retain the supplied copyright and permission notices when distributing data.
The [source README](https://github.com/jelleverheyen/hsk-vocabulary/blob/a66fd30b9580da2c2af7eb19e4b9d8099a29c061/README.md)
also attributes dictionary definitions to CC-CEDICT, so the repository's MIT
declaration does not erase upstream data provenance. Preserve CC-CEDICT attribution
and applicable upstream notices alongside the dataset's MIT notice. Record actual
input checksums and transformations when implementing generation.

The earlier missing-license audit of leonsilicon HSK packages remains in COMPLETED
history. Those packages are no longer selected, so that package-specific blocker
does not apply to this import plan. Do not fetch or redistribute them as substitutes.

### Import identity, forms, and deterministic IDs

The logical import key is **`(simplified, numeric_pinyin)`**, where staging
`numeric_pinyin` maps to the persisted `pinyin_numeric` column. Expand distinct
readings from `forms[]`; do not add a separate word merely for a traditional
variant sharing the same reading key. Retain the first form's traditional value
in source-array order and log a warning when multiple forms share that key.

D07 still enforces `UNIQUE (simplified, pinyin)` on display pinyin. These are
different representations, not universally interchangeable keys. The audited
snapshot has no numeric-key group with conflicting display pinyin and no
display-key group with distinct numeric pinyin (case-sensitive checks). Assert
that relationship when generating; do not silently normalize or collapse readings
if a future input violates it.

After reading-key grouping, assign:

```sql
ROW_NUMBER() OVER (ORDER BY simplified, numeric_pinyin)
```

Here `numeric_pinyin` is the staging name; ordering the final column would use
`pinyin_numeric`. Persist the assigned value as the INTEGER `id`, checking its
range. IDs are snapshot-local. Changing data between runs may shift IDs; saved
exclusions and shared `seed=<word_id>` URLs can break or identify another word.
Committing the DB with its consumer does not make old URLs portable across DB
regenerations. No cross-snapshot ID guarantee or migration is required.

### Level mapping

| Source token | Stored membership |
| --- | --- |
| `old-1` through `old-6` | Corresponding `hsk2_level`, else NULL |
| `new-1` through `new-6` | Corresponding `hsk3_level`, else NULL |
| `new-7` or `new-7+` | `hsk3_level = 7`, the combined 7-9 band |

The pinned JSON actually uses `new-7+`, while its README illustrates `new-7`.
Accept both exact spellings as known tag-format variance; do not accept arbitrary
suffixes, separate levels 8/9, or other unrecognized tokens. Log and skip unknown
level tokens. Deduplicate repeated identical tags. If different levels occur
within one system, choose the lowest and warn; retain the independent value in
the other system. No conflicting distinct levels were found in the audited input.

### Exact text preservation and field coverage

| Target | Source / import rule |
| --- | --- |
| `simplified` | Entry's simplified text, unchanged |
| `traditional` | Selected form's traditional text, unchanged |
| `pinyin` | Selected form's `transcriptions.pinyin`, unchanged; do not derive from numeric |
| `pinyin_numeric` | Selected form's `transcriptions.numeric`, unchanged |
| `english` | Nonempty trimmed meanings in source order, pipe-separated; preserve display casing |
| `part_of_speech` | Map source `pos` codes to canonical lowercase enum tokens; nullable if unavailable |
| `frequency` | Source integer ranking when provided; lower means more common; otherwise NULL |

Preserve spacing, case, and hanzi codepoints exactly, without extra Unicode
normalization of stored hanzi/pinyin. Missing or blank pinyin makes a word unusable:
drop it and log it rather than synthesizing a reading. The audited input has none,
but the generator must handle bad future input.

**Verified format differences:** the pinned data is not uniformly CC-CEDICT's
`u:`/neutral-`5` convention. It includes `ban4 lü3`, unnumbered neutral syllables
such as `zhen1 de`, spaced pinyin, and display text such as `bàn lu:3`.
Preserve those originals per D16. Lookup and round-comparison normalization is
separate from storage; do not claim raw numeric strings are interchangeable with
CC-CEDICT without the comparison-only D18 policy below.

## D18: Source comparison and serialization (approved)

Fully approved on 2026-10-26: the comparison-only pinyin rules, gloss-pipe
serialization, numeric/display-key conflict handling, seed eligibility below,
and the [canonical PoS mapping](#canonical-part-of-speech-mapping-d18-pos-rules-approved).
These are requirements; the generator and recovery implementation remain absent.
D19 fixes the single recovery/build RNG stream and sole v1 format in
[ROUND_GENERATION](ROUND_GENERATION.md#single-stream-rng-and-draw-order-d19-approved);
implementation and golden vectors remain A07/A10 work.

### Comparison-only pinyin normalization (D18)

`normalize_for_comparison()` is used only for right-label duplicate checking and
HSK-to-CC-CEDICT reading-key matching. Never use it to rewrite stored source text
or merge distinct exact numeric source keys. Lowercase, map ü / `u:` / `v` to
`v`, and ignore whitespace. Convert tone-marked display input to numeric tones
for comparison. Preserve numbered tones 1–4.

For neutral tones, split into syllables at letter/digit boundaries before
discarding separators. Strip each syllable's trailing `5` only if the preceding
letters contain a vowel (`a e i o u v`). Preserve consonant-only or ambiguous
syllables, including `r5` and `ng5`. A final digit is not removed globally.

| Input | Comparison key |
| --- | --- |
| `ma5ma5` | `mama` |
| `mai5` | `mai` |
| `r5` | `r5` |
| `ng5` | `ng5` |
| `chi1fan4` | `chi1fan4` |
| `ma5ma` | `mama` |

Omitted neutral tone and a removable neutral `5` compare equally. Stored numeric
and display forms remain byte-for-byte source transcriptions.

Source PoS is coded (for example `n`, `v`, `a`, `q`), not the app's enum strings.
D18's complete mapping below lowercases source codes before lookup, accepts
`Mg`/`Rg` case aliases, and rejects entire entries containing unknown codes such
as `cc`, `mq`, `qt`, and `qv`, with warnings.
The new source supplies PoS/frequency; the prior sources' broad absence of those
fields is no longer the active coverage model. Individual missing fields remain NULL.

### Gloss selection and CC-CEDICT fallback

For each reading, trim meanings and remove blank strings. Use the first form's
nonempty meanings when available; inspect remaining same-reading forms if needed.
Keep source order and original case. Replace literal ASCII `|` inside each gloss
with fullwidth `｜` (U+FF5C) before joining glosses with ASCII `|`. Log both the
original and replaced string for each occurrence. This is deliberately lossy;
never substitute `/`, which already carries meaning in glosses such as `he/she`.
Do not reorder glosses merely to choose a different match label.

Consult CC-CEDICT **if and only if** the resulting primary gloss list is empty.
Match simplified hanzi and disambiguate by numeric reading as needed. Ignore
fallback entries whose gloss begins with `variant of`. If no usable fallback
exists, store `english = NULL` and warn; do not drop an otherwise usable reading.

The first serialized gloss is the English label in both English-bearing variants:
the right column in `hanzi-eng`, the left column in `eng-pinyin`. The right
column of `eng-pinyin` shows the source's diacritic/display pinyin. A row with
NULL English remains eligible for `pinyin-hanzi`, but is ineligible for either
English-bearing variant. Filter missing required labels before candidate indexing.

The exact trigger is an empty list, not a heuristic quality score or a preference
for another dictionary. Use D18's comparison-only reading keys for fallback lookup.

### Numeric/display key conflicts and seed eligibility (D18)

Different exact numeric reading keys that produce the same `(simplified, pinyin)`
display key violate the approved UNIQUE constraint. Fail generation and log both
source rows verbatim; never silently select one. Conversely, forms with the same
`(simplified, numeric_pinyin)` and differing display renderings are one reading:
keep the first source form and warn, per D16. Do not fail for display drift alone.

A nonexcluded seed lacking English is ineligible for `eng-pinyin` and `hanzi-eng`.
Use D15's deterministic URL recovery: up to ten ChaCha draws from the ordered,
exclusion-clean pool, then the first eligible word by id. `pinyin-hanzi` does not
require English. The same eligibility condition applies to D20's English-bearing
MCQ variants; D20 separately approves their routes and scope.

### CC-CEDICT licensing and attribution

The [CC-CEDICT download page](https://cc-cedict.org/editor/editor.php?handler=Download)
declares [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) and recommends
verified releases from [MDBG](https://www.mdbg.net/chinese/dictionary?page=cc-cedict).
D21 approved the A24 snapshot and its notices; A25 preserves those input bytes
and provenance in crates/db-gen/data. The files are present in the working tree
for review, not yet committed or imported into a generated database.

The user reported an older homepage/download discrepancy of 3.0 versus 4.0; on
2026-10-26 the [homepage](https://cc-cedict.org/wiki/) also declared 4.0. Preserve
the historical note and do not relabel an older snapshot whose terms differ.

Required attribution in README and the visible site footer remains:

> Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed CC BY-SA 4.0.

Retain the primary dataset's MIT notice as well as applicable CC-CEDICT notices.
The approved derived-data distribution policy remains CC BY-SA 4.0 for the
generated database containing CC-CEDICT-derived content, with transformations
documented. Application code retains its own licence. Source selection does not
replace checking the actual notices and upstream provenance of acquired inputs.
README credits are documented; the visible footer remains implementation work.

### Excluded sources

Do not use Wiktionary, the noncommercial Anki deck, or the superseded leonsilicon
packages. No additional data source or runtime dependency is installed by D16.

## Schema (D07: approved; implemented in A02)

Use a **single physical table `words`**, with no normalization or separate
membership table for v1. Source joining happens in db-gen, not in runtime queries.
This supersedes the original generic `hanzi` and `hsk_level` field names with
explicit forms and system-specific columns.

The pool is **only the union of the HSK 2.0 and HSK 3.0 word lists**, provided by the pinned primary dataset, with CC-CEDICT only for empty-gloss
fallback. Do not import the entire dictionary as extra rows. At least one
HSK membership must be present for every generated row. The approximate 11,000
entries are a planning estimate; count actual output after deduplication and
reading expansion. Word of the Day and Random's full pool mean this table.

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
```

| Column | Contract |
| --- | --- |
| `id` | Surrogate integer key assigned by db-gen, not automatic identity generation |
| `simplified` | Primary entry's unchanged simplified headword; required |
| `traditional` | First selected source form's unchanged traditional text |
| `pinyin` | Required source display transcription, preserved exactly |
| `pinyin_numeric` | Original primary numeric transcription, preserved alongside display pinyin for numeric browse queries |
| `english` | Pipe-separated glosses, e.g. `to eat\|to have a meal`; NULL for missing enrichment |
| `part_of_speech` | NULL if unknown; otherwise comma-separated canonical lowercase enum strings, e.g. `verb,noun` |
| `hsk2_level` | NULL if absent from HSK 2.0, otherwise 1-6 |
| `hsk3_level` | NULL if absent from HSK 3.0, otherwise 1-6 or 7 for the combined 7-9 band |
| `frequency` | Nullable integer rank; lower means more common; NULL if no verified source rank |

The display-key constraint above remains unchanged. D16 groups imports and orders
ID assignment by `(simplified, numeric_pinyin)`; validate its correspondence to
the display-key constraint. Distinct readings remain separate rows. Preserve both
pinyin forms, source casing, and NULL metadata rather than inventing values.
See the D16 rules above for first-form handling, lower-level conflict resolution,
empty-only gloss fallback, and snapshot-local IDs.

The HSK abstraction (D11) uses the two nullable columns above:

- HSK 2.0 and HSK 3.0 are distinct lists, not successive app/database versions.
- HSK 2.0 has levels 1-6; HSK 3.0 has levels 1-6 plus advanced tier 7-9.
- A word may have memberships in both systems at different levels, or only one.
- Preserve the source's membership for each system; do not infer membership or
  convert a level from one system to the other.

HSK 3.0's integer value **7 = combined advanced band (7–9)**; levels 8 and 9
do not exist as separate stored values. The domain variant is
`Hsk3Level::SevenToNine`, mapped to 7 in storage and canonical URLs. This defines
the logical value, stored in D07's `hsk3_level` column. HSK 2.0 permits
only 1-6. Accepting the URL alias `7-9` must normalize to the same value 7.

Represent system and level/tier separately in domain types while mapping to
`hsk2_level` and `hsk3_level` in this one table. Query filtering and generation
select the appropriate column for the requested system. D16 resolves conflicting
within-system levels to the lowest value with a warning; identical repeated tags
are deduplicated. Do not add a membership table.

### Index policy

DuckDB automatically creates ART indexes for **both** the primary key and the
`UNIQUE (simplified, pinyin)` constraint; do not add duplicate indexes for them.
See [DuckDB constraints](https://www.duckdb.org/docs/1.4/sql/constraints).

Use exactly two additional ordinary indexes, as clarified by the user:

```sql
CREATE INDEX idx_words_hsk3 ON words (hsk3_level);
CREATE INDEX idx_words_hsk2 ON words (hsk2_level);
```

The original proposal included partial-index `WHERE ... IS NOT NULL` predicates.
DuckDB's official regression tests reject those with
`Creating partial indexes is not supported currently`, verified in
[v1.3.2](https://github.com/duckdb/duckdb/blob/v1.3.2/test/fuzzer/pedro/create_index_error.test)
and [v1.4.4](https://github.com/duckdb/duckdb/blob/v1.4.4/test/fuzzer/pedro/create_index_error.test).
The approved ordinary indexes replace that proposal; no runtime version change
is needed to work around it.

Keep PoS and pinyin filtering as scans; avoid further indexes without evidence.
Measure actual row count, database size, browser latency, and relevant query
plans. The 11k estimate and single-table simplicity are not measured speed claims.

### Canonical part-of-speech mapping (D18: PoS rules approved)

The shared PartOfSpeech mapping and fixture tests are implemented in A02.
The importer must still log and reject entries on mapping failure in A03. Pinyin and hanzi remain stored exactly as supplied under D16; the derived
PoS column intentionally stores mapped canonical values.

Lowercase every source code **before lookup**. The source codes are
case-insensitive: `Mg` -> `mg` -> `numeric_morpheme`; `Rg` -> `rg` ->
`pronoun_morpheme`. Implement locale-independent lowercasing for these ASCII
codes, preserving source provenance rather than rewriting input data.

| Source code after lowercasing | Canonical enum string |
| --- | --- |
| `a` | `adjective` |
| `ad` | `adjective_adverbial` |
| `ag` | `adjective_morpheme` |
| `an` | `adjective_nominal` |
| `b` | `non_predicate_adjective` |
| `c` | `conjunction` |
| `d` | `adverb` |
| `dg` | `adverb_morpheme` |
| `e` | `interjection` |
| `f` | `directional_locality` |
| `g` | `morpheme` |
| `h` | `prefix` |
| `i` | `idiom` |
| `j` | `abbreviation` |
| `k` | `suffix` |
| `l` | `fixed_expression` |
| `m` | `numeral` |
| `mg` | `numeric_morpheme` |
| `n` | `noun` |
| `ng` | `noun_morpheme` |
| `nr` | `personal_name` |
| `ns` | `place_name` |
| `nt` | `organization_name` |
| `nx` | `nominal_string` |
| `nz` | `other_proper_noun` |
| `o` | `onomatopoeia` |
| `p` | `preposition` |
| `q` | `classifier` |
| `r` | `pronoun` |
| `rg` | `pronoun_morpheme` |
| `s` | `space_word` |
| `t` | `time_word` |
| `tg` | `time_word_morpheme` |
| `u` | `auxiliary` |
| `v` | `verb` |
| `vd` | `verb_adverbial` |
| `vg` | `verb_morpheme` |
| `vn` | `verb_nominal` |
| `w` | `symbol_punctuation` |
| `x` | `unclassified` |
| `y` | `modal_particle` |
| `z` | `descriptive` |

If **any** code in a source entry is unrecognized after lowercasing, skip the
**entire word entry and all its forms**, and log a warning identifying the word
and unknown code(s). Do not guess a mapping, silently drop only the bad tag, or
keep the entry with NULL PoS. Apply the skip before reading-key merging and
row-number ID assignment, so counts and IDs reflect the accepted input.

A missing/empty PoS list is different from a present unknown code: retain the
word with `part_of_speech = NULL` under D07. Recognized multi-PoS values use the
existing comma-separated canonical-string column; ensure deterministic output
and no duplicate canonical tokens. Query by complete tokens, never substrings.

This full table supersedes the earlier illustrative vocabulary:
`q` maps to `classifier`, not `measure_word`; `y` maps to `modal_particle`,
not generic `particle`. The canonical enum, DB values, query filters, and RNG
seed tokens must agree. URL `pos` still follows D13's different input policy:
ignore unknown query tokens while keeping recognized tokens; empty/omitted or
all-unknown means no filter. Source import's whole-word rejection does not apply
to malformed URL filters.

Audit against pinned `a66fd30b9580da2c2af7eb19e4b9d8099a29c061`, using the
case-insensitive table above:

| Observation | Count / consequence |
| --- | --- |
| Mapping entries | 42 |
| Source entries containing mixed-case codes | 6; case alone does not reject them |
| Unknown lowercase codes | `cc`, `mq`, `qt`, `qv`; do not invent extensions |
| Source entries rejected by the unknown-code rule | 64 out of 11,494 |
| Forms belonging to those rejected entries | 105 out of 12,655 |

Counts describe source entries/forms before other import rules and reading-key
grouping, not final generated DB rows. Some entries have more than one unknown
code. Log/report these exclusions during generation and warn on newly observed
codes rather than silently changing the approved table.

D18's PoS, comparison, delimiter, key-conflict, and missing-label-seed policies
are resolved. Required fixture tests remain implementation work; stored source
transcriptions must not be altered except the explicit gloss delimiter substitution.

## Word of the Day query contract (D05: approved)

Use the full word pool, not practice filters. Count using `COUNT(*)`; when zero,
return an empty result for the neutral placeholder without modulo or offset work.
Otherwise use D14's 64-bit FNV-1a local-date hash modulo that count as the offset
in an ordered query. Use `u64` for the hash/modulo arithmetic. Count and selection
must refer to the same pool; never substitute DefaultHasher.

With D07's synthetic IDs, use `ORDER BY simplified, pinyin` for Word of the Day;
the unique natural key makes that a total order. This maps D05's generic `hanzi`
ordering to the selected physical column. `ORDER BY id ASC` is also deterministic
for a fixed snapshot, but must not be confused with cross-snapshot ID stability.
`LIMIT 1 OFFSET n` without an explicit deterministic `ORDER BY` is forbidden.
Regeneration can change the selected word; preserving a prior generated pool's
Word of the Day is not a backward-compatibility requirement.

Surface load failure separately from a successfully loaded empty pool so the UI
can add Retry only to the load-failure placeholder. See [PRODUCT](PRODUCT.md).

## Practice pool queries (D13: approved)

Apply the active HSK system/level and PoS context, remove excluded WordIds before
selection, and reapply exclusions during D02 widening. Unseeded rounds start with
ordered slices of four at the URL offset, then apply D12's bounded duplicate-label
slide; seeded rounds use the seed plus its HSK
context and do not use offset. Under D12 the caller issues three queries for the
strict, widened-HSK, and widened-HSK/any-PoS views. Every query applies exclusions
in SQL and includes `ORDER BY id ASC`. Views are nested supersets, using clamped
levels from the same HSK system. The pure builder makes no further DB calls.

The URL's parsed exclusions are authoritative for a loaded round; saved browser
exclusions must not alter its queries. URL generation unions saved and current
round sets first. Unknown integer IDs naturally match no rows and are ignored;
noninteger entries are dropped before query construction, with a debug warning.

Use the same eligible ordered pool for its count, offset bounds, and selection.
If offset plus four exceeds pool size, wrap to zero. Exclusions leaving fewer
than eight candidates after widening require the reset-exclusions state. Neither
old generated pools nor old ID-based bookmarks are guaranteed unchanged after DB
regeneration; replay assumes the same database snapshot.

### Right-column label normalization (D12)

Use only the actual right-column label, never the left label, to form duplicate
keys. For `eng-pinyin`, canonicalize pinyin to numeric tone form, remove whitespace,
and lowercase (`chīfàn` -> `chi1fan4`). For `pinyin-hanzi`, compare simplified hanzi
as exact bytes with no trimming or case transformation. For `hanzi-eng`, trim and
lowercase the displayed English gloss and collapse internal whitespace.

English `to eat` and `eat` are distinct keys. Do not silently remove words or
punctuation as a synonym heuristic. The comparison key is derived from the exact
label the UI displays, not another gloss or the entire pipe-separated storage
field. D16 selects the first gloss for both English-bearing variants; rendering
and comparison must use that same gloss. D18 specifies pinyin comparison details;
no new normalization dependency is approved.

### Round builder tuning

Initial `const MAX_SLIDE: usize = 16`; each view tries at most
`min(pool.len(), MAX_SLIDE)` four-word windows before falling through. This is a
tunable performance guard, not a fixed user-facing round count. Most pools are
expected to have near-unique right labels, so sixteen windows provide headroom;
verify that assumption with real pool data. Seeded rounds do not slide.

Always pick the first clean window and give each view its own cap. Changing the
cap leaves windows found before both limits unchanged, but can change a successful
widened fallback into a successful strict result (or the reverse), changing that
URL's round. Do not claim all successful replay is unaffected by cap tuning.
D19 requires explicit approval, a new prefix and URL selector, and a documented
compatibility path for replay-changing tuning. Test first-clean-window/fallthrough
behaviour rather than asserting the constant equals 16.

See [ROUND_GENERATION](ROUND_GENERATION.md) for the complete algorithm and the
distinction between URL offset, internal slide start, and zero-based widened views.

## Runtime version (D08: approved 2026-10-26)

Pin the npm package exactly to `@duckdb/duckdb-wasm@1.32.0`; do not use a range,
`latest`, `next`, or a development build. Implementation and matching worker/wasm
asset setup are still pending; this documentation update does not install it.

The [DuckDB team's OPFS article](https://duckdb.org/2026/09/18/opfs-wasm), published
2026-09-18, recommends pinning 1.32.0 as an option because the then-current
`latest` build had an OPFS persistence bug. This is the historical rationale for
the approved pin; it does not introduce OPFS persistence into Zipeek's read-only
runtime requirements.

When integrating, verify the pinned runtime against the wasm toolchain and
native generator's database format. If a genuine compatibility blocker appears,
stop and report the exact versions, reproducing commands, and error evidence.
Do not silently switch to another release or tag. Loading strategy, native
generator compatibility, and deployed assets still need implementation.

## Regeneration and sizing

Generator invocation (currently contains only the engine guard; native build is
blocked as recorded below, and no dictionary database is produced):

```powershell
# Repository root
cargo run -p db-gen
```

Intended output: `web/static/hanzi.duckdb`. Trunk must be configured to deploy it
at the correct base path; current HTML copies `public/`, not `static/`.

After implementation, document exact input retrieval and generation steps, source
checksums or versions, row counts, output size, and browser download/memory impact.
No database size or performance measurements are available yet.


## A24 generator dependency/source audit (2026-10-26)

A24 recorded the following audit/proposal. D21 and D22 were subsequently
approved on 2026-10-26, and A25 applies the dependency block and source
preservation below. No dictionary import or generated artifact exists yet.

### Audited manifest and accepted direct dependencies (D21)

At audit time, the ignored root Cargo.lock resolved duckdb-neo 0.20000.0 and
libduckdb-sys 1.20000.0 from Git revision
`4acbea84794582c4cd962ee971a3d7a5dcb72e39`. This identifies one local resolution;
the old manifest had no revision pin, and an ignored lockfile is not a reproducible
repository dependency contract. Root lockfile tracking needs separate approval
in A19/A25; exact direct requirements alone do not lock transitive dependencies.

| Current direct dependency | Proposed action | Reason |
| --- | --- | --- |
| reqwest 0.13, json, no defaults | Remove | Generator consumes local verified files; source acquisition is separate. |
| serde 1.0.171, derive | Pin =1.0.229, retain derive | Deserialize primary JSON and input manifest; exact version already resolves locally. |
| serde_plain 1.0.2 | Remove | Fixed PoS/HSK mappings can use explicit matches; no generic enum-string conversion is needed. |
| serde_json 1.0.99 | Pin =1.0.154 | Parse source JSON with the currently resolved release; preserve original string values. |
| tokio 1.54.1, full | Remove | Manual local-file generation is synchronous; no runtime or full async feature set is required. |
| uuid 1.29, v4 | Remove | Approved row-number WordIds are deterministic integers, not UUIDs. |
| thiserror 2.0.21 | Pin =2.0.21 | Typed generation/input errors, consistent with coding standards. |
| tracing 0.1 | Pin =0.1.44 | Structured entry/form rejection, conflict, and transformation diagnostics. |
| tracing-subscriber 0.3, env-filter | Pin =0.3.23, retain env-filter | Native command logging and RUST_LOG filtering. |
| duckdb-neo, moving Git, chrono/r2d2/uuid | Replace with duckdb =1.4.3, bundled only | Match the fixed WASM engine release; single local connection and integer/text schema need none of those optional features. |
| zipeek-core, ../zipeek-core | Retain path dependency | Approved shared domain types; its implementation remains A02. |
| Not direct dependencies today | Add sha2 =0.10.9 and toml =0.9.8 | Verify SHA-256 and parse the coding-standard MANIFEST.toml; no handwritten checksum or TOML parser. |

Current generator dependency block: D21 pins plus the user-requested neo
restoration, pinned to its previous resolved revision:

```toml
[dependencies]
serde = { version = "=1.0.229", features = ["derive"] }
serde_json = "=1.0.154"
thiserror = "=2.0.21"
tracing = "=0.1.44"
tracing-subscriber = { version = "=0.3.23", features = ["env-filter"] }
duckdb-neo = { git = "https://github.com/duckdb/duckdb-rs.git", rev = "4acbea84794582c4cd962ee971a3d7a5dcb72e39", features = ["chrono", "r2d2", "uuid"] }
sha2 = "=0.10.9"
toml = { version = "=0.9.8", default-features = false, features = ["std", "serde", "parse"] }
zipeek-core = { path = "../zipeek-core" }
```

All eight registry versions in the proposal were checked against the public
[crates.io version API](https://crates.io/data-access): none was yanked at audit.
The retained versions matched the audited local resolution; these exact pins
and feature changes now have explicit D21 approval. The tagged native wrapper declares
[Rust minimum 1.84.1](https://raw.githubusercontent.com/duckdb/duckdb-rs/v1.4.3/Cargo.toml),
below this repository's 1.97.1. Registry MSRVs for serde/serde_json/thiserror,
tracing/subscriber, and toml are also below 1.97.1; sha2's metadata omits an MSRV,
so the audit itself made no build-success claim; A25 validation is tracked in
CURRENT. No new direct flate2 dependency is needed: the generator will consume the
verified uncompressed snapshot. Transitive build dependencies may still include
reqwest/flate2; removing direct application dependencies does not remove every
occurrence from the build graph.

### Earlier bundled 1.4.3 proposal: evidence and limits

The fixed npm version 1.32.0 reports Git head
`a20b8290e79e37a7b914e62621aa412d69ba95fc` in its
[registry metadata](https://registry.npmjs.org/@duckdb/duckdb-wasm/1.32.0).
Its [tagged README](https://raw.githubusercontent.com/duckdb/duckdb-wasm/v1.32.0/README.md)
identifies engine 1.4.3; its DuckDB submodule is
`d1dc88f950d456d72493df452dabdcd13aa413dd`, recorded by the
[upstream contents API](https://api.github.com/repos/duckdb/duckdb-wasm/contents/submodules/duckdb?ref=v1.32.0).
Do not interpret the npm package version as the native engine version.

The accepted crates.io duckdb 1.4.3 package depends exactly on libduckdb-sys
1.4.3. Both inspected crate archives record upstream revision
`65e8496fc57098bc896d9fb2ec993b6f0de4e399`; the sys archive contains
`duckdb.tar.gz`, used by the bundled build. Their archive SHA-256 values match
registry checksums:

| Package archive | SHA-256 |
| --- | --- |
| duckdb 1.4.3 | d7eeb487dde618b9f6ab26a451775ad5fac3fabe1ca2b64cbbe90b105f264ccd |
| libduckdb-sys 1.4.3 | c8c60c2d269e63ae5197e4fe9075efffed35dfda0095a5ac8b41f3c765b18456 |

See the exact [wrapper archive](https://static.crates.io/crates/duckdb/duckdb-1.4.3.crate)
and [native sys archive](https://static.crates.io/crates/libduckdb-sys/libduckdb-sys-1.4.3.crate).
Bundled mode avoids accidentally linking a different system DuckDB; it requires
a working C++ compiler/toolchain and builds the packaged native source. The
wrapper still has transitive Arrow dependencies even without optional features.
No JSON/Parquet/chrono/connection-pool/UUID DuckDB features are proposed.

Matching engine releases and the documented
[storage compatibility rules](https://duckdb.org/docs/1.4/internals/storage)
were evidence for the earlier choice, not a successful cross-client test. D23
now requires the generator to query `SELECT version()` on its restored neo
connection, log the raw result and reject anything except `v2.0.0` before
import/output creation. Storage compatibility must be set and verified as
`v1.4.0`; query failures also fail the invocation. A25 adds guard/storage tests
and does not add a Cargo build.rs or automatic
application-build generation step. The current guard opens only an in-memory
connection and produces no dictionary artifact.

**A06 status:** the later neo 2.0 fixture test below establishes read-only WASM
engine compatibility when storage version v1.4.0 is explicit. The final generated
dictionary and browser worker/asset integration remain untested. These remain
A06 work and do not block the restored A25 native build. A02/A05 validate native schema, IDs,
indexes, storage settings and clean checkpoint/close; A06 must test the same
artifact with the pinned worker/WASM assets and report observable failures.
The fixed WASM pin remains unchanged.

### Accepted source snapshot (D21)

Downloaded from the official
[MDBG release link](https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.txt.gz)
on 2026-10-26 at 14:47:06 UTC, into temporary audit storage only. The release
page and file header agree on 125,320 entries and publication timestamp
`2026-10-26T02:33:39Z` (header epoch 1792982019), format version 1/subversion 0,
traditional/simplified UTF-8. The header identifies MDBG, CC BY-SA 4.0, and the
original CEDICT copyright of Paul Andrew Denisowski (1997, 1998).
The [official release page](https://www.mdbg.net/chinese/dictionary?page=cc-cedict)
and [CC-CEDICT download page](https://cc-cedict.org/editor/editor.php?handler=Download)
support that provenance; the actual downloaded header was also inspected.

| Input bytes | Size | SHA-256 |
| --- | --- | --- |
| Official gzip archive | 3,982,067 | bf50c9b8944fb6dce62c1dc3923d189ce11957c97e7ccda8dcb8435bb079b060 |
| Decompressed CC-CEDICT text | 9,868,269 | 3d8967fe566bc23a74f8ba008d38ac83269a8c7550bd7dc48218fc9bdf284885 |
| Existing D16 primary complete.json commit a66fd30 | 9,668,204 | fc474be223d2196b13bfc7c13e10054f1793c2f840eb1287dfc9620fa0d507d2 |

Python hashlib and PowerShell Get-FileHash independently agreed on all three
hashes. Gzip decompression and strict UTF-8 decoding passed. Counting nonempty,
noncomment dictionary rows gives 125,320, matching the header; this is not a
count of imported fallback rows or generated words. The fallback text has CRLF
line endings and no UTF-8 BOM; preserve those bytes if pinning its checksum.

The official download URL is moving, not immutable. D21/A25 preserve the exact
uncompressed text as
`crates/db-gen/data/cc-cedict-2026-10-26.u8` and the pinned primary JSON as
`crates/db-gen/data/hsk-complete-a66fd30.json`, with path/URL/date/size/SHA-256,
notices and transformations recorded in `crates/db-gen/data/MANIFEST.toml`.
Path-specific `.gitattributes` entries with `-text` prevent Git changing input
bytes. The complete fallback header, primary MIT notice/source README and full
CC BY-SA 4.0 licence are preserved beside the inputs, with ATTRIBUTION.md and
manifest checksums. These are working-tree inputs for review, not web assets
or a committed/generated DuckDB database. The Rust preparation pipeline now loads this manifest, validates its policy,
and verifies every source and notice before exposing their bytes for parsing.

The accepted source-preservation choice matters: a checksum identifies bytes
but cannot make a moving URL reproduce them later. Subsequent acquisition must
reject differing bytes; reuse the approved preserved snapshot rather than
silently updating to the newest export. Manual source acquisition/decompression
is separate from the offline generator, which now verifies every declared input
and notice before any source parser can receive the retained bytes. No dataset is added to app assets automatically; generation stays manual.

### Approved logical reproducibility (D22)

D22 replaces the former byte-identical-output requirement. The inspected
[DuckDB 1.4.3 file writer](https://raw.githubusercontent.com/duckdb/duckdb/v1.4.3/src/storage/single_file_block_manager.cpp)
contains GenerateDBIdentifier using RandomEngine; CreateNewDatabase calls it
when encryption is enabled or the selected file storage version is at least 67.
The identifier is serialized into the main header. This is conditional: it does
not prove every unencrypted older-format file is nondeterministic. Exact storage
settings have not been selected or exercised, and byte identity is not established.

Approved D22 rule: require identical ordered logical
rows, IDs, schema/index definitions, and import/drop diagnostics for identical
verified inputs and pinned tooling; record SHA-256 of each final artifact for
integrity without requiring different builds to have the same file hash.
CODING_STANDARDS and TESTING now use that logical-content rule. Do not patch,
mask, or hand-edit database headers to force checksums to match. A05 must
compare independent runs at the logical level and record each artifact checksum;
physical checksum equality is not the reproducibility acceptance criterion.


## Earlier A25 bundled native build blocker (2026-10-26)

The earlier bundled guard/tests initially could not run. With Rust/Cargo
1.97.1, bundled
libduckdb-sys 1.4.3 compilation selected MSVC 19.51.36248
(`14.51.36231/bin/HostX64/x64/cl.exe`) and failed in vendored fmt:

```text
error: failed to run custom build command for `libduckdb-sys v1.4.3`
.../duckdb/third_party/fmt/include/fmt/format.h(326): error C2653: 'stdext': is not a class or namespace name
.../duckdb/third_party/fmt/include/fmt/format.h(326): error C2061: syntax error: identifier 'checked_array_iterator'
```

Reproduced with `cargo test -p db-gen` and `cargo test --workspace`; Clippy also
hits the native build failure and existing app.rs unused imports. No dependency
substitution, compiler-flag workaround, or source/header patch was applied.
The user subsequently restored neo; its native build now passes. D23 then
approved native v2.0.0 and mandatory compatible storage, resolving that blocker.
This is not the deferred A06 native-to-WASM read-only test; that remains an open
question and is not an A25 prerequisite. Input hashes and documentation can be
checked independently; current native guard/storage tests now pass as below.


## Neo-to-WASM compatibility probe (2026-10-26)

The user requested restoring duckdb-neo and then asked for a direct file-read
check before changing the native-engine guard. Neo is restored at the prior
resolved Git revision `4acbea84794582c4cd962ee971a3d7a5dcb72e39`, retaining its
previous chrono/r2d2/uuid features and prebuilt-library defaults. Other D21 pins,
inputs and D22 policy are unchanged. Pinning the revision avoids advancing the
Git source while restoring it. The native query reports `v2.0.0`.

A temporary native neo executable created two independent eight-row fixtures
using the exact ten-column words DDL, PK/display-key uniqueness and two ordinary
HSK indexes. It checkpointed, detached and closed both files. Neither fixture
is the final dictionary artifact; they contain synthetic Chinese/pinyin/English
rows with NULL metadata and different readings of the same hanzi. Files and
probe tools are under ignored target/a25-compat, not app assets.

Exactly @duckdb/duckdb-wasm 1.32.0 was installed only in that isolated directory.
Its node-blocking bindings instantiated the actual packaged WASM (engine
`v1.4.3`) and attempted read-only opens:

| Neo 2.0 writer configuration | Pinned WASM outcome |
| --- | --- |
| Default storage settings | Rejected: file version 999; reader accepts 64 through 67. |
| ATTACH with STORAGE_VERSION 'v1.4.0' | Passed: all 8 complete rows, 10 columns and both named HSK indexes matched; DELETE was rejected as read-only. |

The passing test compares every column, including original Unicode/pinyin,
pipe-separated glosses and NULL values. This is direct WASM-engine evidence in
Node 24.17.0, not a browser-worker/HTTP deployment test or full-dictionary test.
A06 retains those integration checks. Default neo output cannot be assumed
readable by this pinned WASM; the explicit older storage setting matters.

Reproduce the preserved scratch probe from the repository root:

```powershell
cargo run --manifest-path target/a25-compat/native/Cargo.toml --target-dir target --offline
node target/a25-compat/read-wasm.cjs
```

The native scratch producer refuses to overwrite existing fixture files; use a
fresh scratch location for regeneration. Results are saved in
`target/a25-compat/results.json`. After reviewing this evidence the user approved
D23: require native v2.0.0 and v1.4.0 storage compatibility. The earlier native
v1.4.3 guard is superseded. The generator queries/logs SELECT version(), checks
v2.0.0, sets storage_compatibility_version to v1.4.0, queries/logs and verifies
that setting, then returns its configured in-memory connection for future output
work. Mismatch, missing query values, or database errors fail clearly. No output
is created before these checks; no production dictionary exists yet. Future A05
file creation must use that configured connection and explicit v1.4.0 storage.

Focused native tests now pass: actual engine version, incorrect-version
rejection, and configured compatible storage. The command logs:

```text
DuckDB engine: v2.0.0
DuckDB storage compatibility: v1.4.0
```


## Implemented input preparation (A26)

`cargo run -p db-gen` now performs real offline preparation. The three-line
binary delegates to CLI handling; library modules own manifest loading, file
verification, orchestration and database setup. Set RUST_LOG=info to see the
verification summary; an invalid explicit filter fails rather than hiding it.

The manifest's engine, storage, WASM package and reproducibility fields must
match approved policy. Source names must be unique, notice references declared,
and each input/notice path must be a nonempty relative path resolving inside
the manifest directory. Duplicate resolved files, malformed hashes, missing
files, incorrect source sizes and SHA-256 mismatches return typed errors with
paths and expected/actual values. No database connection is prepared until all
files pass. Verified source and notice buffers are retained in memory; future
importers parse those bytes instead of reopening paths after verification.

The current preserved manifest verifies two sources and three notices, totaling
19,567,208 bytes. A02 now also creates the empty words schema in memory.
A03/A04 parsing/import and A05 output remain separate work.


## Implemented schema and domain boundary (A02)

schema/words.sql is the exact approved D07 ten-column table; schema.rs adds
only idx_words_hsk3 and idx_words_hsk2. Preparation creates it after input and
engine/storage checks. No CHECK constraints or schema changes were introduced:
HSK ranges, at least one membership and import-level text policies remain the
responsibility of typed conversions and the upcoming importer. Native fixtures
assert column types/nullability, primary/display-key enforcement, exact source
text, independent memberships and absent metadata. No file artifact is written.

The shared types preserve both pinyin strings and nullable traditional, numeric
pinyin, English and frequency fields. PartOfSpeech accepts exactly the 42 source
codes above, case-insensitively, with strict canonical strings for DB/URL use.
Source lists are canonical-token sorted and deduplicated; empty maps to NULL
when inserted. WordId permits 1 through i32::MAX. A03 must assign IDs after
accepted-entry grouping in the documented simplified/numeric-pinyin order.
