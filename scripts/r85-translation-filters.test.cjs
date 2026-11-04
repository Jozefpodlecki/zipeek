const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = path => readFileSync(join(__dirname, '..', path), 'utf8');

test('multi-filter source has checked decoding and stable round-trip tests', () => {
  const codec = read('crates/web/src/services/practice_filter_source.rs');
  assert.match(codec, /pub fn encode\(filters: &DictionaryFilters\)/);
  assert.match(codec, /pub fn decode\(source: &str\)/);
  assert.match(codec, /HskBand::new/);
  assert.match(codec, /Duplicate vocabulary filter/);
  assert.match(codec, /round_trip_multiple_dimensions/);
  assert.match(codec, /invalid_or_unbounded_source_fails_closed/);
});

test('Translation uses the parameterized dictionary adapter for custom sources', () => {
  const loader = read('crates/web/src/services/translation_vocabulary.rs');
  const page = read('crates/web/src/pages/sentence_translation.rs');
  const setup = read('crates/web/src/pages/practice_setup.rs');
  assert.match(loader, /practice_filter_source::decode/);
  assert.match(loader, /practice_words_for_filters\(&filters, 128\)/);
  assert.match(page, /source.starts_with\("filters:"\)/);
  assert.match(setup, /practice_filter_source::encode\(filters\)/);
});

test('Practice offers database-derived categories and supports filtered Matching', () => {
  const picker = read('crates/web/src/components/practice_vocabulary_source.rs');
  const setup = read('crates/web/src/pages/practice_setup.rs');
  assert.match(picker, /practice_category_suggestions\(&prefix\)/);
  assert.match(picker, /data-vocabulary-hsk-multi/);
  assert.match(picker, /data-category-chip/);
  assert.match(picker, /suggestions.contains\(&category\)/);
  assert.doesNotMatch(setup, /matching_filters_supported/);
  assert.match(setup, /query.insert\("source".to_owned\(\), crate::services::practice_filter_source::encode\(filters\)\)/);
  assert.match(setup, /disabled=\{!ready \|\| !dictionary_filters_valid \|\| !dictionary_round_ready/);
});
