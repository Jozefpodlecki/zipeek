const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = path => readFileSync(join(__dirname, '..', path), 'utf8');

test('single source UI is shared and old duplicate controls are absent', () => {
  const page = read('crates/web/src/pages/practice_setup.rs');
  const picker = read('crates/web/src/components/practice_vocabulary_source.rs');
  assert.match(page, /<PracticeVocabularySource/);
  assert.doesNotMatch(page, /data-translation-word-set|data-practice-personal-set|<HskPicker/);
  assert.match(picker, /data-vocabulary-dictionary/);
  assert.match(picker, /data-vocabulary-personal/);
  assert.match(picker, /data-vocabulary-hsk-multi/);
  assert.match(picker, /data-vocabulary-set/);
});

test('same vocabulary source is translated to both existing launch formats', () => {
  const page = read('crates/web/src/pages/practice_setup.rs');
  const loader = read('crates/web/src/services/practice_loader.rs');
  assert.match(page, /SetupAction::SelectHsk\(level\)/);
  assert.match(page, /practice_filter_source::encode\(filters\)/);
  assert.match(page, /format!\("saved:\{reference\}"\)/);
  assert.match(page, /word_sets::practice_snapshot\(sets, reference\)/);
  assert.match(page, /query.insert\("set_ids"/);
  assert.match(loader, /database.candidate_pool_for_set\(ids, &query.exclusions\)/);
  assert.match(page, /disabled=\{snapshot_error.is_some\(\) \|\| !dictionary_filters_valid \|\| !dictionary_pool_ready\}/);
});

test('the HSK6 translation fixture selects the unified source control', () => {
  const browser = read('crates/e2e/tests/translation_fixture.rs');
  const wordSets = read('crates/e2e/tests/suites/word_sets.rs');
  assert.match(browser, /data-hsk-band/);
  assert.match(browser, /data-hsk-band=.*3:6/);
  assert.match(browser, /data-translation-setup-count/);
  assert.match(wordSets, /data-vocabulary-personal/);
  assert.match(wordSets, /data-vocabulary-set/);
});
