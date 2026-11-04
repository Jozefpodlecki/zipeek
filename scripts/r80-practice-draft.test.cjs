const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..');
const source = fs.readFileSync(path.join(root, 'crates/zipeek-exercises/src/practice_draft.rs'), 'utf8');
const lib = fs.readFileSync(path.join(root, 'crates/zipeek-exercises/src/lib.rs'), 'utf8');
const spec = fs.readFileSync(path.join(root, 'docs/product/PRACTICE_SETUP_V2.md'), 'utf8');

test('practice draft is owned by the exercise domain without new kind enum', () => {
  assert.match(lib, /pub mod practice_draft;/);
  assert.match(source, /pub struct PracticeDraft/);
  assert.match(source, /definition\(id\)/);
  assert.doesNotMatch(source, /enum ExerciseKind/);
  assert.doesNotMatch(source, /use yew|web_sys|duckdb/);
});

test('exclusive vocabulary variants and typed multi-filter semantics are covered', () => {
  assert.match(source, /pub enum VocabularySource\s*\{\s*PersonalSet/);
  assert.match(source, /Dictionary\(DictionaryFilters\)/);
  assert.match(source, /multiple_values_are_or_within_group_and_and_across_groups/);
  assert.match(source, /category_chips_are_normalized_deduplicated_and_removable/);
  assert.match(source, /hsk_systems_do_not_conflate_advanced_levels/);
});

test('unsupported scored modalities stay explicit and UI integration is not claimed', () => {
  assert.match(source, /validate_for_scored_session/);
  assert.match(source, /UnsupportedAssessment/);
  assert.match(spec, /R81 introduces the first exercise-first selector/);
  assert.match(spec, /multi-category dictionary\n  filters and saved setup profiles are under development|category multi-combobox.*remain/);
});
