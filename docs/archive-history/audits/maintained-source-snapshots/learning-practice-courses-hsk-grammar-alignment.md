# Mapping Zipeek Grammar to the 2025 HSK syllabus

## Evidence status

- Zipeek R488 has 100 draft Grammar courses and 550 teaching entries; this is not a certified inventory of independent grammatical rules.
- The 2025 HSK syllabus is a separate reference. A community transcription reports **593 items**, covering parts of speech, phrase structures, sentence patterns, pragmatic constructions and related categories. These items are not necessarily one independent rule each.
- **No HSK mappings have been verified in R490.** A matching Chinese example is only an editorial suggestion, never evidence of semantic equivalence.
- The third-party TSV was verified as publicly accessible through web search, but this runtime cannot download it into the source tree. The full HSK alignment has not been performed or fabricated.

## Sources and provenance

Official syllabus landing page: <https://www.chinesetest.cn/syllabus>.

Official PDF publisher: Center for Language Education and Cooperation, November 2025 (effective July 2026); grammar occupies pages 385-406. Reference download: <https://hsk.cn-bj.ufileos.com/3.0/%E6%96%B0%E7%89%88HSK%E8%80%83%E8%AF%95%E5%A4%A7%E7%BA%B21219.pdf>.

Community TSV transcription: <https://github.com/chelsea6502/hsk-2025-data/blob/master/grammar.tsv>. It contains `examLevelId`, `content`, `grammarType`, `categoryType`, `grammarDetail`, and `cases`; the repository advertises 593 source records. It is not itself an official certification. Keep its provenance and calculated SHA-256 with every imported mapping. Review malformed input rows rather than silently shifting field positions.

## Workflow

The former Python index/import/audit/report commands were removed in R493 at the user's request. The checked-in `GRAMMAR_TEACHING_INDEX.tsv` and the historical editorial mapping files remain for review. The full third-party HSK TSV still needs to be obtained and checked against the official PDF. Automated reimport, source-hash comparison, and mapping reports are **not currently available** until a Rust-native replacement is implemented. Do not assume the historical report can be refreshed or that a mapping has been verified.

`GRAMMAR_TEACHING_INDEX.tsv` was generated from the original 20 Rust courses and the 451-row editorial TSV for regions 21-100. Each entry receives an `gNNN-pNN` teaching ID, a region and a point ordinal, with `kind=unclassified` and `review_status=pending_linguistic_review` until an editor explicitly classifies it. The lesson runtime is not changed by this editorial index.

The HSK mapping queue has one row per source item. An exact shared Chinese example can suggest a teaching-entry ID. The status remains `unreviewed`; items with malformed source fields are marked `needs_source_review`. A reviewer must inspect the official syllabus and the lesson, decide whether a mapping is `candidate`, `verified`, or `unmapped`, and provide a rationale for `verified` and `unmapped` entries. Candidate and verified statuses require valid teaching IDs. Source metadata is checked on audit, and malformed source rows cannot advance beyond `needs_source_review` without correcting the reference. The former report command wrote a per-HSK-level count of unreviewed, candidate, verified, unmapped and source-review entries plus the number of unique verified teaching IDs. A summary of 593 items is not evidence of 593 verified mappings; the report sets `hsk_coverage_verified` only when all 593 are verified and no parse errors exist. Multiple HSK items may map to one teaching point; one HSK item may require multiple teaching points. A parent/variant relationship must be reviewed separately, not guessed from English labels.

## Acceptance gates

1. Confirm the source edition, legal use and all 593 source records (including levels 7-9). Resolve any malformed transcription rows against the official PDF.
2. Review each item's category, required competence, correct course placement and construction/variant relationships. No unreviewed item may count toward syllabus coverage.
3. Add explicit editorial review, natural Mandarin examples, tone-marked Pinyin and varied, independent point assessments; correct mistaken teaching entries without adding duplicate filler to reach an arbitrary total.
4. Only after independently scored evidence meets approved thresholds may a Grammar region be awarded mastery or reveal its artwork. Importing or mapping syllabus records never changes user progress.
5. Run locked Rust/WASM tests and Chrome journeys under both `/` and `/zipeek/` before accepting a release.

## Known limitation

The first Grammar region contains four points, while the intended curriculum structure is generally five to six points per course. This was preserved in R489 to avoid inventing an additional construction without expert review. Its four-point count should not be silently represented as full compliance with the updated authoring target.

## R490 verification status

The grammar reference is discoverable online, but downloading the 593-row TSV into the local project remains blocked by the environment network restriction. The tooling changes are tested with synthetic samples, not with a full 593-row import. The published level distribution for the 2025 transcription is HSK 1: 70, HSK 2: 78, HSK 3: 96, HSK 4: 95, HSK 5: 70, HSK 6: 50 and HSK 7-9: 134, totaling 593. Those figures are external-source metadata, not Zipeek verified teaching coverage.
