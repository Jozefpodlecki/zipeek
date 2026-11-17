> **R492 current source ownership:** Grammar teaching tables, guided-check data, region plan and published slugs now live under `crates/zipeek-pedagogy/src/grammar/`. Patterns 21-100 are generated there from `docs/learning-practice/GRAMMAR_CURRICULUM_SOURCE.tsv`. Step/reveal/answer logic lives under `zipeek-exercises::grammar_learning`, with only Yew interaction state in web. All 100 regions are draft-authored; editorial and HSK review remain outstanding. The historical R485 path/status note below is superseded.

# Courses authoring and structural validation

The five published grammar courses are teaching drafts, not verified HSK syllabus coverage or authoritative mastery assessments. This guide explains how to add a course without leaving its registry, teaching activities and assessment formats inconsistent.

## Source ownership

| Source | Authoring responsibility |
| --- | --- |
| `crates/zipeek-pedagogy/src/grammar/curriculum.rs` | Grammar explanations, examples, translations and original five guided three-choice banks |
| `courses/curriculum_registry.rs` | Stable course and grammar-point IDs, local sequence and prerequisites; official HSK status remains unverified |
| `courses/guided_bank.rs` | Guided sentence-builder and transformation banks, accepted tile sequences and feedback |
| `courses/assessment.rs` | Independent draft multiple-choice item for each registered point |
| `courses/construction_assessment.rs` | Independent unassisted sentence-construction draft for each authored course |
| `courses/authoring.rs` | Cross-format structural validation (`validate_published_courses`) and detailed failures |
| `courses/grammar_lesson.rs`, `courses/grammar_expansion_lesson.rs`, `courses/grammar_lesson_scaffold.rs` | Two teaching engines sharing navigation and progress shell; Yew-only course interaction |

## Adding or editing a grammar course

1. Author nonempty titles, explanations, examples and translations. Avoid claims of verified HSK syllabus coverage without an independently checked source.
2. Add a stable curriculum registry entry with the same course identifier and one point ID per authored grammar point; verify prerequisites refer to earlier published courses.
3. Author exactly one guided comprehension check and one independent draft MCQ per point. The independent question should not rely on a visible guided answer.
4. Provide guided tile activities with nonempty metadata, a distinct bank and a full display permutation. Every accepted order must use unique in-bounds tile indices and leave at least one distractor.
5. Add an unassisted construction assessment linked to a point in that same course. Do not repeat a tile or include a reference answer during the attempt.
6. Update `GuidedCourse` region mapping, mosaic region/link and course route. Update the E2E scenario and coverage inventory.
7. Run the structural validator's unit test and the existing Courses tests, then compile natively, compile WASM and run Chrome E2E on the pinned toolchain. A passing Node source-contract test alone is **not** Rust or browser acceptance.

The validator catches missing or duplicated authored identifiers, blank content, invalid question keys, duplicate choices, incomplete point coverage, malformed tile permutations, duplicate activity IDs, and cross-course construction-point mismatches. These are **structural safeguards only**: they do not prove pedagogical fairness, meaningful distractors, translation quality, accessible presentation or a qualified review of the Mandarin.

## Explicitly out of scope

No validation result is a mastery grant. Draft scores remain in memory; all 150 mosaic regions stay concealed. Award rules, verified evidence/provenance, DuckDB writes, completed grammar records, and artwork-reveal predicates require their separately approved implementation and browser acceptance.
