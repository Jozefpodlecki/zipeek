# R139 - Dictionary verification for AI sentence vocabulary

## Implemented

After structured AI translation, Zipeek attempts one read-only DuckDB query for the entire bounded vocabulary list (at most 24 candidates). The query uses parameter placeholders for exact simplified/traditional matches, with no fuzzy semantic matching. The result includes dictionary ID, Chinese forms, pinyin, source English glossary, HSK 2.0 and 3.0 levels, and frequency rank when available.

The reconciliation keeps the AI observation separate from the dictionary record. When several entries share a Chinese spelling, exact normalized pinyin is preferred, then known/lower frequency rank and stable word ID. Alternative-entry count is disclosed; selecting a likely row is not a disambiguated sense match. Original dictionary definitions and AI glosses are shown with separate provenance labels. A missing dictionary match is not presented as verified.

The source-only response remains functional when the dictionary is unavailable or lookup fails; the page reports the failed verification without discarding successful translation or AI analysis. Generation checks prevent an old database response from appearing after source or direction edits.

## Known limitations

- Dictionary verification establishes **word metadata**, not whole-sentence HSK difficulty; grammar and sentence level remain AI estimates.
- Dictionary glosses are word-level and may not describe the sense used in context. No word-sense alignment or Chinese segmentation correction is claimed.
- Frequency is a rank, not a probability; missing frequency is allowed.
- Pinyin matching strips whitespace and is case-insensitive, but does not transliterate numeric versus tone-mark forms. Ambiguous spellings require a future explicit sense selector.
- Dynamic queries rely on generated dictionary availability and browser DuckDB; Windows WASM build, live AI + database E2E, Traditional Chinese matching and HSK/frequency data correctness require verification.
- No new schema, dependency, AI configuration, or local learner data migration.

## Windows acceptance

1. `cargo fmt --all --check`
2. `cargo test --workspace --locked`
3. `cargo check -p zipeek --target wasm32-unknown-unknown --locked`
4. Run `cargo make serve`, configure LM Studio, translate Chinese → English and English → Chinese, and check verified HSK/frequency labels.
5. Test a phrase such as 一本 (missing exact entry), a homograph such as 行 (ambiguity) and a dictionary-unavailable scenario. Translation and AI analysis should remain readable.
