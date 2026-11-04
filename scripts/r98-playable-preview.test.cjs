const test = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = path => readFileSync(join(__dirname, '..', path), 'utf8');

test('playable answer preview shares normalization and eligibility with the round builder', () => {
  const labels = read('crates/zipeek-core/src/round/labels.rs');
  const selection = read('crates/zipeek-core/src/round/selection.rs');
  assert.match(labels, /pub fn distinct_playable_answers/);
  assert.match(labels, /filter\(\|word\| eligible\(word, variant\)\)/);
  assert.match(labels, /answer_key\(word, variant\)/);
  assert.match(selection, /let needed = if variant.is_mcq\(\) \{ 3 \} else \{ 8 \}/);
  assert.match(labels, /distinct_answers_are_not_raw_row_counts/);
});

test('preview is exclusion-aware and rejects stale results for other excluded sets', () => {
  const picker = read('crates/web/src/components/practice_vocabulary_source.rs');
  const setup = read('crates/web/src/pages/practice_setup.rs');
  assert.match(picker, /practice_exact_pool_for_filters\(&filters, &excluded\)/);
  assert.match(picker, /exclusions: Option<BTreeSet<WordId>>/);
  assert.match(picker, /\*generation.borrow\(\) == epoch/);
  assert.match(setup, /checked_exclusions == &current_exclusions/);
  assert.match(setup, /selected_variants.iter\(\).all\(\|variant\| preview.supports\(\*variant\)\)/);
  assert.match(setup, /!dictionary_round_ready/);
  assert.match(setup, /data-practice-playable-gate/);
});
