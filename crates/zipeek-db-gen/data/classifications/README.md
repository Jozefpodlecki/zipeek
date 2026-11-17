# CC-CEDICT gloss-classification export

Source: CC-CEDICT 2026-10-26 (CC BY-SA 4.0), SHA-256 `3d8967fe566bc23a74f8ba008d38ac83269a8c7550bd7dc48218fc9bdf284885`.

- Dictionary entries: 125,320
- Individual gloss records: 199,699
- Records with at least one nonempty automatic label: 75,210
- Records with grammatical label other than unclassified: 47,915
- Records with topical category: 32,314
- Records flagged for review: 151,802

**Important**: These are deterministic pattern-based suggestions, **not classifications manually or individually reviewed by ChatGPT**. Every gloss is present, with explicit unclassified results where evidence is insufficient. A generic subject-word match may still yield a false positive; inspect evidence and review before database publication. The original source-line number and gloss index are stable locators within the source file; they are **not** HSK database word/sense IDs. The production db-gen classification cache has a different schema and source-sense digest, so this export must not be pasted into that cache.

Regenerate from repository root with `node scripts/cedict-classify.cjs`.

## Consumption by `db-full` (R179)

The default `cargo make db-full` now converts this export to database-sense classification cache records, using only verified exact provenance/lexical joins and explicit `unclassified` fallbacks for unmatched senses. It does **not** promote the rule-based labels to human-reviewed annotations. Use `cargo run -p db-gen --locked -- import-cedict-classifications --output <NEW_CACHE.jsonl>` to produce the validated cache without installing a database. See `docs/testing-quality/acceptance/ai-classification.md` for operational details.
