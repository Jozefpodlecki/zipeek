const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const source = rel => fs.readFileSync(path.join(root, rel), 'utf8');
test('Browse definitions come from normalized per-sense display data', () => {
  const code = source('crates/web/src/services/database/browse_queries.rs');
  assert.match(code, /string_agg\(d\.definition, '\|' ORDER BY d\.sense_order\)/);
  assert.match(code, /EXISTS \(SELECT 1 FROM sense_display AS present/);
  assert.match(code, /ELSE w\.english END AS english/);
  assert.match(code, /FROM word_senses AS s WHERE s\.word_id = w\.id AND contains\(lower\(s\.gloss\)/);
});
test('Browse cross-reference rows are source-backed, linked and fully named', () => {
  const queries = source('crates/web/src/services/database/queries.rs');
  const card = source('crates/web/src/components/browse/results/word_card.rs');
  assert.match(queries, /FROM sense_relations WHERE word_id = \?/);
  for (const label of ['Abbreviation of', 'Short for', 'Variant of', 'Also written as', 'See also']) assert.ok(card.includes(label));
  assert.match(card, /relation\.source_annotation\.clone\(\)/);
  assert.match(card, /BrowseAction::Search/);
  assert.match(card, /href=\{link\} onclick=\{on_reference\}/);
});
