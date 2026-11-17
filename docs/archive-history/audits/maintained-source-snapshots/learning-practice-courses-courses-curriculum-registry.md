> **Current R489 state (2026-11-16):** All 100 Grammar-only regions contain draft lessons with 550 teaching entries. The historical R359/R366 registry below describes the first five courses only, not the complete curriculum. The separate teaching index (`GRAMMAR_TEACHING_INDEX.tsv`) now assigns stable IDs to all 550 entries. Official HSK 2025 grammar alignment is **not verified**; a reproducible external-source import and review process is documented in `HSK_GRAMMAR_ALIGNMENT.md`. The original five courses still have separately authored draft assessments; the other 95 have guided recognition only.

# Courses curriculum registry

R359 introduces a stable, local authoring registry for the four currently published grammar courses. It does **not** establish that the content is a verified mapping to the official HSK syllabus.

## Authoring registry

| Mosaic region | Stable course ID | Topic | Point IDs | Recommended prior course |
| --- | --- | --- | --- | --- |
| 1 | `grammar-1` | Basic Sentence Structure | `g01-subject-predicate`, `g01-shi-identity`, `g01-bu-negation`, `g01-ma-question` | None |
| 2 | `grammar-2` | Time, Place & Everyday Actions | `g02-time-placement`, `g02-zai-location`, `g02-time-location`, `g02-qu-destination`, `g02-nali-question` | `grammar-1` |
| 3 | `grammar-3` | Possession, Existence & Quantities | `g03-you-possession`, `g03-meiyou-negation`, `g03-classifiers`, `g03-ji-count`, `g03-duoshao-quantity` | `grammar-1` |

| 4 | `grammar-4` | Descriptions & Comparisons | `g04-adjective-predicate`, `g04-hen-degree`, `g04-bu-adjective`, `g04-tai-le`, `g04-bi-comparison` | `grammar-1` |

The recommended prior-course relationship is teaching guidance, **not** an access restriction or assessed prerequisite. Existing authored lesson routes remain available directly.

The registry lives in `crates/zipeek-web/src/pages/learning/courses/curriculum_registry.rs` and enforces unique course regions, course IDs and grammar point IDs, matching authored lesson/check counts, and backward-only valid prerequisite references. Validation runs in Rust unit tests, not as a startup blocker. IDs are intended to remain stable once published and must not be treated as acquired grammar-point evidence.

## Syllabus provenance and limitations

All published registry entries currently use `PendingIndependentVerification`. Teaching stages are local editorial sequencing, not official HSK levels. No claim of 19 official HSK syllabus objectives, comprehensive HSK coverage, or independent assessment validity is made. Before publishing a verified syllabus mapping, obtain and independently review the authoritative grammar-standard edition and item identifiers, associate every item with authored content and an independent assessment, then add explicit verified references with validation tests. That is separate from implementing evidence-backed completion and irreversible mosaic reveals.

Future courses should receive new stable IDs only when their content exists, and should leave un-authored mosaic regions marked unavailable. The final grammar curriculum may exceed 100 regions to cover the approved syllabus.

## Independent grammar assessment draft (R360)

The first four published courses now expose `#/courses/grammar/assessment/grammar-1`, `grammar-2`, `grammar-3`, and `grammar-4`, with one separate three-choice item per registered grammar point (4 + 5 + 5 + 5). The items are authored in `assessment.rs` using stable point IDs and independent prompts, rather than reusing the guided lesson answer array. A distinct `AssessmentSession` records one answer per item, permits revisions *before* submission, disallows submission of incomplete attempts, locks answers afterwards and calculates a score only after full submission. The Yew page withholds rationales and correct answers until the review screen.

**No authoritative completion claim:** This remains an editorial and interaction draft. It does not implement HSK syllabus verification, qualified Mandarin-language assessment review, durable personal evidence, consent-backed recording, independently validated distractors, passing thresholds, retries/cooldowns, or artwork reveal. No score is stored or reported to the pedagogy engine. The mosaic stays concealed even if all draft items are correct. Assessment-time answer correctness is not the same as the acquisition engine's separate Hanzi 10×7 rule.

## R366 extension - Descriptions and Comparisons

Course 4 introduces adjective predicates, degree words 很, adjectival negation 不,
太…了, and comparisons with 比. It contains five explanatory teaching points,
five immediate-feedback guided questions, six authored word-bank activities,
five separately authored draft multiple-choice assessment items, and a separate
no-hints construction item. All item IDs use the new `g04-` registry namespace.
The ordinary lesson and draft assessment have different feedback rules.
All drafted answers need independent Mandarin linguistic review, particularly
nuances around 很, 太…了 and comparison interpretation. No achievements or mosaic
regions can be revealed based on these drafts.
