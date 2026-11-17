# Practice route and replay validation

There are **two different state models**. Classic Matching/MCQ rounds use typed URL queries with a deterministic round seed; translation-only Advanced Practice uses a browser-local UUID record and should not embed per-question configuration into the URL. Tests must not conflate them or treat previous release encodings as permanent compatibility requirements.

## Classic URL rounds

`zipeek-core/src/practice/` owns query parsing and normalization. Tests cover HSK system/level, Fixed/Endless selection, exclusions, seed, offset, saved Word Set snapshots and safe failure for invalid inputs. Reproducing a shared round requires the same dictionary snapshot and effective inputs. Reloading a Fixed round does not imply restoring its prior page-local score or progress. See [round generation](../../games/round-generation.md), [matching](../../testing-quality/e2e/matching.md) and [MCQ](../../testing-quality/e2e/mcq.md).

## UUID-backed generated Practice

`#/practice/session/<id>` resolves a record created by Practice setup. The persisted record contains selected vocabulary, translation direction, schedule seed, generated question and outcomes. Missing/corrupt IDs must show a recovery error rather than silently launching an unrelated question. Navigation/reload of unfinished work restores that record; completion deletes the temporary record. See [sentence exercises](../../architecture/modules/sentence-exercises.md) and [Practice setup](../../learning-practice/practice/practice-session-builder.md).

## Actual acceptance

`crates/zipeek-e2e/tests/translation_fixture.rs` covers the UUID translation journey with the Rust fixture. Other integration test targets cover classic URL replay and browser interactions. The latest UUID translation implementation still requires execution on Windows. Do not preserve obsolete URL encodings merely to satisfy old snapshots: the active Rust types and explicit user decisions govern the contract.
