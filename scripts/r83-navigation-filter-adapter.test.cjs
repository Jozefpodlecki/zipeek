const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const root = join(__dirname, '..');
const read = path => readFileSync(join(root, path), 'utf8');

test('drawer icons reuse yew_icons and preserve accessible text and navigation targets', () => {
  const source = read('crates/web/src/components/navigation.rs');
  assert.match(source, /use yew_icons::\{Icon, IconData\}/);
  assert.match(source, /IconData::LUCIDE_HOME/);
  assert.match(source, /IconData::LUCIDE_GRADUATION_CAP/);
  assert.match(source, /IconData::LUCIDE_BOOK_OPEN/);
  assert.match(source, /aria-hidden="true"/);
  assert.match(source, /<span>\{label\}<\/span>/);
  assert.match(source, /aria-current=/);
  assert.match(source, /data-navigation-link/);
});

test('Thirtyfour styling probe now expects navigation icons', () => {
  const source = read('crates/e2e/fixtures/browser_probes/styling_acceptance.js');
  assert.match(source, /decorative icon/);
  assert.doesNotMatch(source, /Label-only navigation|Navigation decoration remains/);
});

test('shared query adapter joins multi HSK and categories and rejects unsafe codes', () => {
  const code = read('crates/web/src/services/database/practice_filter_queries.rs');
  const module = read('crates/web/src/services/database/mod.rs');
  assert.match(module, /mod practice_filter_queries;/);
  assert.match(code, /pub async fn practice_words_for_filters/);
  assert.match(code, /pub async fn practice_category_suggestions/);
  assert.match(code, /FROM word_hsk_membership/);
  assert.match(code, /FROM word_senses AS s JOIN sense_categories AS c/);
  assert.match(code, /FROM word_pos_membership/);
  assert.match(code, /validate_codes\(&filters.categories\)/);
  assert.match(code, /\#\[test\]\s+fn multiple_hsk_bands_are_or_and_filter_groups_are_and/);
  assert.match(code, /\#\[test\]\s+fn category_only_filter_does_not_inject_an_hsk_limit/);
});

test('multi-filter controls are shared by Translation and Matching', () => {
  const picker = read('crates/web/src/components/practice_vocabulary_source.rs');
  const setup = read('crates/web/src/pages/practice_setup.rs');
  assert.match(picker, /data-vocabulary-hsk-multi/);
  assert.match(picker, /data-category-chip/);
  assert.doesNotMatch(setup, /matching_filters_supported/);
  assert.match(setup, /query.insert\("source".to_owned\(\), crate::services::practice_filter_source::encode\(filters\)\)/);
  assert.match(setup, /practice_filter_source::encode/);
});
