# Schema, index and search review

Requested 2026-10-29, tracked as A62. This is a review brief, not approved DDL
or permission to replace the generated artifact, change dependencies or break
practice replay. The current contract remains in [schema](schema.md),
[queries](queries.md) and [runtime](runtime.md).

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
