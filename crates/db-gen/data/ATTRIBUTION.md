# Dictionary input provenance

The approved D21 inputs are preserved byte-for-byte as identified in
[MANIFEST.toml](MANIFEST.toml). They are generator inputs, not web assets or a
generated database. Generator invocation remains manual/offline. The current binary loads the
manifest, verifies all declared source/notice sizes and hashes, retains those
verified bytes, and prepares the native engine with compatible storage. It does
not parse/import dictionary entries or create a dictionary artifact.

The primary HSK source is jelleverheyen/hsk-vocabulary complete.json at commit
`a66fd30b9580da2c2af7eb19e4b9d8099a29c061`. Retain the complete
[MIT notice](HSK-LICENSE.txt) and [upstream README](HSK-SOURCE-README.txt).
The licence identifies Copyright (c) 2023 Yanis Zafirópulos. The README includes
its own 2025 notice and credits upstream dictionary sources; that provenance is
preserved alongside the repository's MIT declaration.

Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)),
licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
The complete [licence text](CC-BY-SA-4.0.txt) is preserved here. The fallback
retains its original MDBG publication header, licence URL and CEDICT copyright
notice for Paul Andrew Denisowski (1997, 1998).

The fallback snapshot was published at 2026-10-26T02:33:39Z with 125,320 entries.
Only gzip decompression was performed; the original text/header, UTF-8 and CRLF
bytes are retained. The moving MDBG URL must not silently replace this snapshot.
Use the preserved files and reject a different checksum during future input
verification. Path-specific Git attributes prevent line-ending conversion.

The approved distribution policy for generated CC-CEDICT-derived dictionary
data remains CC BY-SA 4.0, with transformations recorded. Application code
retains its own licence. No dictionary rows have been imported or transformed
by preserving these input files.
