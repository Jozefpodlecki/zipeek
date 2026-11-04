const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..');
const sql = fs.readFileSync(path.join(root, 'crates/web/src/services/database/practice_filter_queries.rs'), 'utf8');
const ui = fs.readFileSync(path.join(root, 'crates/web/src/components/practice_vocabulary_source.rs'), 'utf8');

test('count and vocabulary sample call the same validated predicate builder', () => {
  assert.match(sql, /fn practice_filter_predicate\(filters: &DictionaryFilters\)/);
  assert.match(sql, /pub async fn practice_filter_count/);
  assert.match(sql, /let \(predicate, parameters\) = practice_filter_predicate\(filters\)\?/);
  assert.match(sql, /let \(predicate, mut parameters\) = practice_filter_predicate\(filters\)\?/);
  assert.match(sql, /count_and_sample_share_identical_filters/);
});

test('Practice shows count and prevents stale async results overwriting new filters', () => {
  assert.match(ui, /data-vocabulary-pool-count/);
  assert.match(ui, /practice_exact_pool_for_filters\(&filters, &excluded\)/);
  assert.match(ui, /\*generation.borrow\(\) == epoch/);
  assert.match(ui, /No matching words/);
});

test('category suggestions support selecting buttons and keyboard confirmation', () => {
  assert.match(ui, /data-category-suggestions/);
  assert.match(ui, /onkeydown=\{on_category_keydown\}/);
  assert.match(ui, /event.key\(\) != "Enter"/);
  assert.match(ui, /suggestions.contains\(&category\)/);
});
