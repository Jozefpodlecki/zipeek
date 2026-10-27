# Sources

Primary HSK vocabulary snapshot, CC-CEDICT fallback, licensing, and excluded
sources. Part of the [database documentation](DB.md). Approved source selection
in [D16](../../process/decisions/database-and-artifacts.md#d16-primary-import-and-ids);
licensing policy in [D06](../../process/decisions/database-and-artifacts.md#d06-sources-and-licensing).

## Primary: pinned HSK vocabulary snapshot

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

## CC-CEDICT licensing and attribution

The [CC-CEDICT download page](https://cc-cedict.org/editor/editor.php?handler=Download)
declares [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) and
recommends verified releases from
[MDBG](https://www.mdbg.net/chinese/dictionary?page=cc-cedict). D21 approved the
A24 snapshot and its notices; A25 preserves those input bytes and provenance in
`crates/db-gen/data`. The files are present in the working tree for review, not
yet committed or imported into a generated database.

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
README credits are documented; the visible footer is implemented in `AppFooter`.

## Accepted fallback snapshot (D21)

Downloaded from the official
[MDBG release link](https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.txt.gz)
on 2026-10-26 at 14:47:06 UTC, into temporary audit storage only. The release
page and file header agree on 125,320 entries and publication timestamp
`2026-10-26T02:33:39Z` (header epoch 1792982019), format version 1/subversion 0,
traditional/simplified UTF-8. The header identifies MDBG, CC BY-SA 4.0, and the
original CEDICT copyright of Paul Andrew Denisowski (1997, 1998). The
[official release page](https://www.mdbg.net/chinese/dictionary?page=cc-cedict)
and [CC-CEDICT download page](https://cc-cedict.org/editor/editor.php?handler=Download)
support that provenance; the actual downloaded header was also inspected.

| Input bytes | Size | SHA-256 |
| --- | --- | --- |
| Official gzip archive | 3,982,067 | `bf50c9b8944fb6dce62c1dc3923d189ce11957c97e7ccda8dcb8435bb079b060` |
| Decompressed CC-CEDICT text | 9,868,269 | `3d8967fe566bc23a74f8ba008d38ac83269a8c7550bd7dc48218fc9bdf284885` |
| Existing D16 primary complete.json commit a66fd30 | 9,668,204 | `fc474be223d2196b13bfc7c13e10054f1793c2f840eb1287dfc9620fa0d507d2` |

Python hashlib and PowerShell Get-FileHash independently agreed on all three
hashes. Gzip decompression and strict UTF-8 decoding passed. Counting nonempty,
noncomment dictionary rows gives 125,320, matching the header; this is not a
count of imported fallback rows or generated words. The fallback text has CRLF
line endings and no UTF-8 BOM; preserve those bytes if pinning its checksum.

The official download URL is moving, not immutable. D21/A25 preserve the exact
uncompressed text as `crates/db-gen/data/cc-cedict-2026-10-26.u8` and the pinned
primary JSON as `crates/db-gen/data/hsk-complete-a66fd30.json`, with
path/URL/date/size/SHA-256, notices and transformations recorded in
`crates/db-gen/data/MANIFEST.toml`. Path-specific `.gitattributes` entries with
`-text` prevent Git changing input bytes. The complete fallback header, primary
MIT notice/source README and full CC BY-SA 4.0 licence are preserved beside the
inputs, with `ATTRIBUTION.md` and manifest checksums. These are working-tree
inputs for review, not web assets or a committed/generated DuckDB database. The
Rust preparation pipeline now loads this manifest, validates its policy, and
verifies every source and notice before exposing their bytes for parsing.

The accepted source-preservation choice matters: a checksum identifies bytes
but cannot make a moving URL reproduce them later. Subsequent acquisition must
reject differing bytes; reuse the approved preserved snapshot rather than
silently updating to the newest export. Manual source acquisition/decompression
is separate from the offline generator, which now verifies every declared input
and notice before any source parser can receive the retained bytes. No dataset
is added to app assets automatically; generation stays manual.

## Excluded sources

Do not use Wiktionary, the noncommercial Anki deck, or the superseded leonsilicon
packages. No additional data source or runtime dependency is installed by D16.

## Related

- Import rules applied to these sources: [import-rules.md](import-rules.md).
- PoS mapping applied during import: [pos-mapping.md](pos-mapping.md).
- Dependency pins used with these sources: [generator-deps.md](generator-deps.md).