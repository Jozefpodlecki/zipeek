const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..');
const setup = fs.readFileSync(path.join(root, 'crates/web/src/pages/practice_setup.rs'), 'utf8');
const picker = fs.readFileSync(path.join(root, 'crates/web/src/components/practice_vocabulary_source.rs'), 'utf8');

test('dictionary counts propagate source identity to setup and reset on vocabulary changes', () => {
  assert.match(picker, /on_pool_count: Callback<\(VocabularySource, Option<BTreeSet<WordId>>, Option<Result<PoolPreview, String>>\)>/);
  assert.match(picker, /on_pool_count\.emit\(\(source\.clone\(\), exclusions\.clone\(\), None\)\)/);
  assert.match(picker, /on_pool_count\.emit\(\(selected_source, selected_exclusions, Some\(count\)\)\)/);
  assert.match(setup, /checked_pool\.set\(None\)/);
  assert.match(setup, /checked_source == source && checked_exclusions == &current_exclusions && preview.matching_rows > 0/);
});

test('both launch actions block zero, pending and failed dictionary availability checks', () => {
  assert.match(setup, /data-practice-pool-gate/);
  assert.match(setup, /disabled=\{snapshot_error\.is_some\(\) \|\| !dictionary_filters_valid \|\| !dictionary_pool_ready\}/);
  assert.match(setup, /disabled=\{!ready \|\| !dictionary_filters_valid \|\| !dictionary_round_ready/);
  assert.match(setup, /if !dictionary_pool_ready \{/);
  assert.match(setup, /VocabularySource::PersonalSet \{ \.\. \} => true/);
});
