# Zipeek dictionary data

This generated HSK-union database is distributed under CC BY-SA 4.0. Application
code has its own licence. This is a transformed database, not an original export.

Primary data: jelleverheyen/hsk-vocabulary complete.json, pinned commit
`a66fd30b9580da2c2af7eb19e4b9d8099a29c061` (MIT declaration; upstream CC-CEDICT
definitions retain their attribution). Copyright notices supplied with that
snapshot are preserved in `notices/HSK-LICENSE.txt` and
`notices/HSK-SOURCE-README.txt`.

Chinese dictionary data from CC-CEDICT (https://cc-cedict.org/), licensed
CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/).
The fallback snapshot is the MDBG release published 2026-10-26T02:33:39Z;
CC-CEDICT retains the original CEDICT copyright of Paul Andrew Denisowski
(1997, 1998). The complete licence is in `notices/CC-BY-SA-4.0.txt`.

Transformations: reject unknown PoS entries, expand/group exact readings, retain
first-form transcriptions, merge memberships, canonicalize PoS, select primary
glosses in order and replace embedded ASCII pipes with U+FF5C. Apply same-reading
CC-CEDICT fallback only for empty primary glosses, excluding variant-of entries.
Assign snapshot-local integer IDs in simplified/numeric-pinyin order. Preserve
original hanzi and stored pinyin. See `hanzi.diagnostics.json` for this run.

`INPUTS.toml` records source URLs, dates, checksums and licensing provenance.
`hanzi.manifest.json` records actual engine/storage settings, counts, schema,
indexes, logical-content and diagnostics checksums, and each distributed file's
size/SHA-256. Its own checksum is intentionally not self-referential. Different
database file hashes do not imply different logical content.
