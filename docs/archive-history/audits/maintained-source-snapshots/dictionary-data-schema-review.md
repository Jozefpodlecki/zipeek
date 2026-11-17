# Schema, index and search review

**Historical review brief, not current artifact specification.** This document evaluates the former 12,100-row HSK-only schema and DuckDB-WASM 1.32.0. The installed expanded dictionary reports 12,188 rows and browser dependency 1.33.1-dev114.0. For the implemented schema and release gates, read [schema](../../../dictionary-data/schema/schema.md), [runtime](../../../dictionary-data/schema/runtime.md), and [source reconciliation](../SOURCE_RECONCILIATION.md).

Requested 2026-10-29, tracked as A62. The original A62 benchmark/FTS work remains a review brief. The incremental
R01/R02 membership, sense and taxonomy DDL is user-approved source work, but
replacing the generated artifact, changing dependencies or breaking practice
replay is not covered by these checkpoints. The current contract remains in [schema](../../../dictionary-data/schema/schema.md),
[queries](../../../dictionary-data/schema/queries.md) and [runtime](../../../dictionary-data/schema/runtime.md).

## Current repository baseline

- 12,100 reading rows in one `words` table; reading identity differs from a
  headword or individual gloss/sense.
- Primary key `id` and unique `(simplified, pinyin)` constraints, with their
  implicit indexes, plus explicit `idx_words_hsk2` and `idx_words_hsk3` indexes.
- Glosses are pipe-separated and PoS values comma-separated. Practice queries
  currently split PoS and test complete tokens rather than search substrings.
- Browser runtime is pinned npm DuckDB-WASM 1.32.0, engine v1.4.3, opening
  v1.4.0-compatible generated storage read-only. Native generation uses the
  approved neo engine v2.0.0. These are distinct version constraints.
- No category/tag tables, FTS corpus/index or extension packaging exists in
  the current dictionary/search pipeline. Browse UI is still future A13 work.

## Tables to compare, not assume

| Candidate | Reason to evaluate | Open choice |
| --- | --- | --- |
| Tag vocabulary plus word-to-tag relation | Multiple categories and stable tag identifiers without packing more delimiters into a column | Tag dimensions, reading versus sense ownership, unclassified entries, provenance |
| Word-to-PoS relation or typed list column | Cleaner complete-token filters than repeatedly splitting strings | Compare joins/lists with current scans; no speed claim yet |
| Separate gloss/sense records | Per-sense tags, explicit meanings and ambiguity handling | Introduce only if sense-level behavior is required; do not merge distinct readings |
| Generated search/read model | Keep a convenient row-shaped browser facade while improving storage/search internally | View versus materialized/generated data, row uniqueness and export cost |

A category dictionary and many-to-many assignment relation are a useful starting
proposal for A56/A62. Additional tables can improve data meaning and query
clarity; they are not automatically faster. Classification run metadata may
belong in a table or generation reports; do not choose extra tables merely
because AI enrichment is planned.

Practice candidates must remain one row per WordId after filtering. Joins must
not multiply candidates when a word has several tags or PoS values; compare
`EXISTS`/deduplicated membership strategies and retain deterministic ordering.
Respect D44 OR-within/AND-across semantics. Schema/query changes that affect
candidate identities/order also need the existing replay-impact review.

## Do we need more indexes?

Not established yet. DuckDB has automatic zonemaps and ART indexes associated
with constraints or explicit index creation. Its 1.4 guidance describes useful
ART scans for selective single-column equality/IN predicates; ordinary ART
indexes are not a blanket acceleration for joins, sorting or aggregation.
They also consume memory. See [DuckDB 1.4 indexing](https://duckdb.org/docs/1.4/guides/performance/indexing).

Retain the two approved HSK indexes for now. Measure whether they are used and
whether retaining, removing or adding indexes helps the actual browser workload.
Do not duplicate constraint indexes or infer that indexing a delimited PoS
string accelerates token membership. Any proposed composite membership key may
enforce uniqueness without providing the lookup performance one expects from
other databases. Review its purpose and plan explicitly.

## Is FTS supported in DuckDB-WASM?

Yes: the official DuckDB 1.4 WASM extension list includes `fts`. WASM extension
loading fetches and dynamically loads a matching WebAssembly extension; its
`INSTALL` behavior differs from native persistent installation. See
[WASM extensions](https://duckdb.org/docs/1.4/clients/wasm/extensions).

DuckDB's FTS extension creates an index schema and BM25 search macros. In 1.4,
defaults include Porter stemming, English stopwords, accent stripping and an
ignore pattern excluding characters outside lowercase ASCII letters. Those
defaults do not preserve our Chinese/pinyin/numeric-tone search semantics.
Evaluate FTS first for English gloss/sentence retrieval; suitable Chinese
segmentation and tone-preserving pinyin behavior require separate design/tests.
See [DuckDB 1.4 FTS](https://duckdb.org/docs/1.4/core_extensions/full_text_search).

BM25 word retrieval is also different from approved Browse substring/prefix and
exact-filter behavior. Do not silently replace that contract or ranking with
FTS. A ranked mode would require a product/URL review.

The documentation establishes platform availability, not tested support in
Zipeek's exact package/build. A62 must probe:

1. Matching extension version/build, asset loading and packaging/notices under
   the existing dependency and local-asset policy; no unapproved CDN/latest path.
2. Read-only queries over an index generated beforehand. Building FTS mutates
   storage, so never create it inside the shipped read-only dictionary at runtime.
3. Native v2.0.0 generator output versus v1.4.3 WASM extension compatibility;
   compatible base storage alone does not prove extension macros/data compatibility.
4. Root and `/zipeek/` browser serving, startup size/latency, missing extension
   behavior and how existing search works without an optional FTS path.
5. English stopwords/inflections, hanzi, pinyin tones/digits/case and rare/proper
   senses, with explicit expected behavior.

Any FTS schema is generated manually with an approved artifact change; no
generation or extension install is added to application builds by this brief.

## Required evidence and deliverable

Compare the current schema/query baseline against minimal candidate designs
using representative HSK-only, PoS-only, category-only and combined filters;
small and large result sets; counts; ordered pagination; exact word lookup;
and approved text-search strategies. Include unknown/multiple tags and PoS.

Record `EXPLAIN ANALYZE`, browser timings (cold/warm), database and extension
bytes, startup/memory cost and correct result/ordering parity. The small current
dictionary makes scans a legitimate baseline, not a proven performance problem.
Use isolated generated scratch artifacts for approved experiments; do not
replace `web/static` while conducting the review.

A62's deliverable is concrete DDL/query/index proposals, benchmark evidence,
FTS compatibility findings, schema/replay/dependency impacts and a recommended
scope for approval. A56 supplies taxonomy/ownership decisions and A61 supplies
the setup-filter workload; schema review itself need not wait for their UI.

## Candidate normalized search model (2026-11-04 review addition)

The current `words` table is a **reading-level projection**, not a normalized
sense dictionary. `part_of_speech` is comma-delimited, and `english` is
pipe-delimited; therefore a query cannot unambiguously associate an individual
sense with a part of speech or a category. A HSK-level filter can use the current
scalar column, but adding systems/editions and combinations complicates DDL.

A candidate layout for review (not approved DDL):

- `readings(reading_id PRIMARY KEY, simplified, traditional, pinyin,
  pinyin_numeric, frequency)`. Preserve existing `WordId` as `reading_id`,
  including the current `(simplified, pinyin)` identity contract.
- `senses(sense_id PRIMARY KEY, reading_id REFERENCES readings, gloss,
  source_order, source_id)`. Store a separate row for each actual source sense;
  **do not** assume splitting the legacy `english` display field reproduces
  independently attested senses or correctly attaches PoS.
- `parts_of_speech(pos_id PRIMARY KEY, code UNIQUE)` and
  `sense_pos(sense_id, pos_id, PRIMARY KEY(sense_id, pos_id))` for genuine
  sense-level labels. If existing input only supplies reading-level PoS,
  instead use `reading_pos(reading_id, pos_id)` until provenance supports the
  more granular model.
- `hsk_membership(reading_id, system, level, PRIMARY KEY(reading_id,
  system))` for system/level filtering. Current D07 semantics only permit one
  level per system per reading; preserving that constraint avoids duplicates.
- `tags(tag_id PRIMARY KEY, slug UNIQUE, label)` with explicit
  `reading_tags(reading_id, tag_id)` or `sense_tags(sense_id, tag_id)` only
  after A56 establishes category ownership and provenance.
- A **generated, stable `words` read projection** preserving the web `WordRow`
  shape during rollout. Build it once during native generation (not a browser
  runtime materialization); queries returning candidates must preserve one row
  per reading and order by `reading_id`.

Candidate index experiments should use selective access patterns, not blanket
indexing: compare `(system, level, reading_id)` on HSK membership, a PoS-to-
reading membership path, exact `(simplified, pinyin)` and `pinyin_numeric`
lookup, and ordered page scans. The PK/unique constraints already create ART
indexes and should not be duplicated. Text search should benchmark normalized
pinyin prefix/equality, Chinese substring queries and English gloss tokens
separately. DuckDB FTS is **not** a drop-in Chinese index; default English
stemming/tokenization is inappropriate for hanzi and tone-marked pinyin.

Acceptance evidence for a schema change: source-level attribution and sense
identity, before/after row parity, no join-induced duplicate candidates,
`EXPLAIN ANALYZE` on native and actual pinned browser-WASM runtime, size and
cold/warm query measurements, regenerated provenance manifests, and explicit
replay impact review. This section is a design proposal only: neither
`words.sql` nor generated DuckDB binaries are changed by the translation MVP.

## Checkpoint R01 - generated normalized memberships (2026-11-04)

The first approved schema implementation adds `word_hsk_membership` and
`word_pos_membership` as distinct, keyed many-to-many relations. Existing
`words` remains the reading identity and browser-facing compatibility table;
its IDs, SQL projection and matching/MCQ replay semantics are unchanged.

HSK rows use `(word_id, system)` as their unique key, with `hsk2` and `hsk3`
and levels 1-7 (7 denotes the consolidated advanced band). PoS membership is
**reading-level source metadata**, not sense-level AI classification; it is
populated from the existing canonical CSV values. No new taxonomy or inferred
per-sense metadata is claimed. Both relations are populated in the same import
transaction, with rollback on failure. The generator's logical snapshot now
includes sorted membership rows for output/reopen comparison.

This checkpoint intentionally does **not** change `words.sql`, browser reads,
indexes, source import formats, `hanzi.duckdb`, or dependency versions. The
compatibility table and its existing two HSK indexes remain because browser
queries depend on them. Adding indexes to membership predicates requires
`EXPLAIN ANALYZE` and WASM benchmarks before indexing decisions.

Next: preserve actual source sense boundaries before introducing `senses`,
controlled tag taxonomy, `sense_pos`/`sense_categories`, classification
provenance and reusable AiMux 0.6.0 enrichment. Then benchmark HSK+PoS+tag
filters and migrate read APIs without duplicating practice candidates.

Validation pending: this environment has no Cargo/Rust toolchain or native
DuckDB engine, so generated-schema tests and native/WASM round trips could not
be executed. R01 is a source checkpoint, not a verified release artifact.

## Checkpoint R02 - source-backed senses and controlled taxonomy (2026-11-04)

Generator source changes introduce `word_senses` keyed by `(word_id, sense_order)` and preserve the original nonempty HSK meanings as individual rows. `source_ref` identifies the one-based HSK source entry and `source_gloss_order` identifies the one-based meaning position; merged duplicate readings retain the first nonempty gloss source, matching the existing compatibility `words.english` selection policy. When primary meanings are missing, CC-CEDICT fallback senses retain the original source line and meaning index. A pipe within an original gloss is preserved as literal content, rather than being interpreted as a sense separator. No inference or silent reassignment of a meaning to another reading occurs.

A constrained `parts_of_speech` catalog uses all 42 canonical `zipeek-core` codes. A version-1 controlled category catalog initially contains travel, transport, food, shopping, family, education, work, health, nature, time, home, and communication. The new `sense_pos` and `sense_categories` relations are **empty by design**: reading-level source PoS cannot safely be assigned to every individual sense. `classification_runs` stores the model, prompt version, taxonomy version and input digest needed for subsequent AI-derived rows; classifier integration and caching are R03 scope. These are schema scaffolds, not claims of classified data.

Generation and staged/read-only verification now compare individual sense rows and category catalog contents. No index is added beyond existing HSK indexes and constraint-backed indexes: selective query benchmarks, shipped artifact growth, and GitHub Pages/WASM load time have not been measured. Current browser-facing `words` queries and deterministic practice URLs remain unchanged.

R02 is a source checkpoint only. The native DuckDB engine/Rust toolchain is not installed here; native schema syntax/constraint support, compilation, generation and browser tests still require validation before a generated artifact is approved.


## R03 - cache-backed sense classification (2026-11-04)

Classification output is strict JSON with allowed `pos` and `categories` codes. Each cached JSONL record references a `(word_id, sense_order)`, source digest, model, prompt version and taxonomy version. `db-gen generate --classification-cache PATH` validates every record before writing anything, applies validated assignments transactionally to both in-memory and staged databases, and verifies all classification rows in logical snapshots. Without the flag the existing offline generator writes no AI classifications. The native AiMux entry point is an async library function, not yet a batch CLI. Compilation, DB reopening and LM Studio integration are unverified; review history is retained in the 2026-11-04 completed outcomes.
