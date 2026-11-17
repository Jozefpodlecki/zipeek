# Queries

Word of the Day query contract, practice pool queries, right-label
normalization, and round builder tuning. Part of the
[database documentation](db.md). Approved contracts in
[D05](../../project-management/decisions/foundations.md#d05-word-of-the-day),
[D12](../../project-management/decisions/rounds-and-replay.md#d12-deterministic-round-builder),
and [D13](../../project-management/decisions/rounds-and-replay.md#d13-counts-urls-exclusions-offsets).

## Word of the Day query contract (D05: approved)

Use the full word pool, not practice filters. Count using `COUNT(*)`; when zero,
return an empty result for the neutral placeholder without modulo or offset work.
Otherwise use D14's 64-bit FNV-1a local-date hash modulo that count as the offset
in an ordered query. Use `u64` for the hash/modulo arithmetic. Count and selection
must refer to the same pool; never substitute `DefaultHasher`.

With D07's synthetic IDs, use `ORDER BY simplified, pinyin` for Word of the Day;
the unique natural key makes that a total order. This maps D05's generic `hanzi`
ordering to the selected physical column. `ORDER BY id ASC` is also deterministic
for a fixed snapshot, but must not be confused with cross-snapshot ID stability.
`LIMIT 1 OFFSET n` without an explicit deterministic `ORDER BY` is forbidden.
Regeneration can change the selected word; preserving a prior generated pool's
Word of the Day is not a backward-compatibility requirement.

Surface load failure separately from a successfully loaded empty pool so the UI
can add Retry only to the load-failure placeholder. See
[PRODUCT](../../product-ux/product.md).

## Practice pool queries (D13: approved)

Practice meaning-based rounds select the first `sense_display` definition classified `lexical` for the exact `word_id`/reading, ordered by `sense_order`. All strict, widened, filtered and Word Set pools exclude readings without such a definition. The saved `words.english` projection is not used as an exercise-sense selector; Browse continues to show legitimate usage references. The same selected definition is used to build and display a round. The generator classifies source glosses during regeneration, currently excluding standalone `used in ...` references and relation-only cross-references; it does not discard or reorder dictionary data. This is a conservative vocabulary eligibility classification, not a universal guarantee that every lexical sense makes an unambiguous question.

Apply the active HSK system/level and PoS context, remove excluded `WordId`s before
selection, and reapply exclusions during D02 widening. Unseeded rounds start with
ordered slices of four at the URL offset, then apply D12's bounded duplicate-label
slide; seeded rounds use the seed plus its HSK context and do not use offset.
Under D12 the caller issues three queries for the strict, widened-HSK, and
widened-HSK/any-PoS views. Every query applies exclusions in SQL and includes
`ORDER BY id ASC`. Views are nested supersets, using clamped levels from the same
HSK system. The pure builder makes no further DB calls.

The URL's parsed exclusions are authoritative for a loaded round; saved browser
exclusions must not alter its queries. URL generation unions saved and current
round sets first. Unknown integer IDs naturally match no rows and are ignored;
noninteger entries are dropped before query construction, with a debug warning.

Use the same eligible ordered pool for its count, offset bounds, and selection.
If offset plus four exceeds pool size, wrap to zero. Exclusions leaving fewer
than eight candidates after widening require the reset-exclusions state. Neither
old generated pools nor old ID-based bookmarks are guaranteed unchanged after DB
regeneration; replay assumes the same database snapshot.

## Right-column label normalization (D12)

Use only the actual right-column label, never the left label, to form duplicate
keys.

| Variant | Right label | Comparison key |
| --- | --- | --- |
| `eng-pinyin` | Pinyin | Canonical numeric tone form, no whitespace, lowercase; e.g. `chīfàn` becomes `chi1fan4` |
| `pinyin-hanzi` | Simplified hanzi | Exact bytes; no case folding or whitespace stripping |
| `hanzi-eng` | Displayed English gloss | Trim, lowercase, collapse internal whitespace |

Compare normalized keys byte-for-byte. Homophones collide in `eng-pinyin`; 行 with
different readings collides in `pinyin-hanzi`; equal normalized English labels
collide in `hanzi-eng`. English `to eat` and `eat` remain different. Earlier
examples checking the left column were corrected and are not part of the rule.

English `to eat` and `eat` are distinct keys. Do not silently remove words or
punctuation as a synonym heuristic. The comparison key is derived from the exact
label the UI displays, not another gloss or the entire pipe-separated storage
field. D16 selects the first gloss for both English-bearing variants; rendering
and comparison must use that same gloss. D18 specifies pinyin comparison details;
no new normalization dependency is approved.

## Round builder tuning

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

See [ROUND_GENERATION](../../games/round-generation.md) for the complete
algorithm and the distinction between URL offset, internal slide start, and
zero-based widened views.

## Related

- Builder algorithm: [ROUND_GENERATION](../../games/round-generation.md).
- Browser query implementation: [../modules/browser-db.md](../../architecture/modules/browser-db.md).
- Schema read by these queries: [schema.md](schema.md).

## Future schema/query review (A62)

Compare the existing candidate/search workload with normalized tag/PoS and
optional English FTS proposals. Ensure membership joins do not duplicate WordIds
or change deterministic order; preserve approved Browse search semantics.
Measure plans/browser latency/size before selecting more indexes. See
[review brief](review.md) for official support and the pinned-WASM/read-only probe.

## Implemented normalized Browse search

`crates/zipeek-web/src/services/database/browse_queries.rs` introduces a typed, parameterized
read-only search endpoint over the regenerated schema. Search uses the original
`words` projection only for rendering and stable `ORDER BY w.id ASC`; individual
English glosses are matched through `word_senses`. HSK, source PoS and AI-assigned
category filters are applied with correlated `EXISTS` subqueries so multiple senses
and labels cannot duplicate the same result. Filters are OR within a code list and
AND across dimensions; paging is limited to 100 results per request.

The query is now connected to Browse through `BrowseSearchForm` and `BrowseResults`. The installed manifest declares normalized membership, sense and category tables, and the user reported a successful Windows run of Browse interactions. This is not a claim that all filter combinations or the latest UUID Practice changes have been browser-tested.

Text matching currently uses `contains(...)` (case-folded for pinyin/English),
which is a substring scan rather than an indexed full-text search. It deliberately
does not pretend that an index on an HSK column will speed up arbitrary substrings.
Benchmark real generated data with `EXPLAIN ANALYZE` for exact hanzi, pinyin,
English substring, HSK-only, category-only and combined filters using both sparse
and common categories. Compare latency, plans and file size before approving
new secondary indexes or FTS features in DuckDB-WASM.

PoS filters use `word_pos_membership` (source reading-level PoS), while category
filters use `sense_categories` (AI-derived sense-level labels). An entry with no
AI classification does not match a category filter; an unfiltered search still
returns it. Future sense-level PoS filtering can be added explicitly once the
product rules for source vs AI classification are decided.
