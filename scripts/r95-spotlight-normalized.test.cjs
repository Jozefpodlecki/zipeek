const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');
test('Spotlight reads cleaned definitions and all source-backed relationships', () => {
  const queries = read('crates/web/src/services/database/queries.rs');
  const component = read('crates/web/src/components/word_spotlight.rs');
  assert.match(queries, /string_agg\(definition, '\|' ORDER BY sense_order\)/);
  assert.match(queries, /WHEN EXISTS \(SELECT 1 FROM sense_display/);
  assert.match(queries, /definition IS NOT NULL/);
  assert.match(component, /database\.word_display_definition\(word\.id\)/);
  assert.match(component, /database\.word_relations\(word\.id\)/);
  for (const name of ['Abbreviation of', 'Short for', 'Variant of', 'Also written as', 'See also']) {
    assert.ok(component.includes(name));
  }
  assert.match(component, /source_annotation\.clone\(\)/);
  assert.match(component, /data-word-cross-reference/);
  assert.doesNotMatch(component, /display_gloss/);
  assert.doesNotMatch(queries, /pub async fn word_abbreviation/);
});
