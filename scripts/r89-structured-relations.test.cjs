const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const read = file => readFileSync(file, 'utf8');

test('dictionary has source-backed sense relations and keeps original gloss', () => {
  const schema = read('crates/db-gen/src/schema.sql');
  const storage = read('crates/db-gen/src/import/storage.rs');
  assert.match(schema, /CREATE TABLE sense_relations/);
  assert.match(schema, /relation_type IN \('abbreviation_of', 'short_for', 'variant_of', 'also_written_as', 'see_also'\)/);
  assert.match(schema, /FOREIGN KEY \(word_id, sense_order\) REFERENCES word_senses/);
  assert.match(storage, /INSERT INTO sense_relations/);
  assert.match(storage, /INSERT INTO word_senses/);
  assert.match(storage, /lexical_reference\(&sense\.gloss\)/);
});

test('structured extraction is conservative and covered by native fixtures', () => {
  const code = read('crates/db-gen/src/import/relations.rs');
  assert.match(code, /supermarket \(abbr\. for 超级市场\)/);
  assert.match(code, /Gen Z \(abbr\. for 95后 \+ 00后\)/);
  assert.match(code, /target_traditional/);
  assert.match(code, /target_pinyin/);
  assert.match(code, /is_chinese_reference/);
});

test('logical snapshot includes relations so generated artifacts are verified', () => {
  const snapshot = read('crates/db-gen/src/artifact/snapshot.rs');
  assert.match(snapshot, /pub sense_relations: Vec<String>/);
  assert.match(snapshot, /SELECT to_json\(sense_relations\)/);
});

test('discovery uses stored relation metadata rather than guessed link targets', () => {
  const queries = read('crates/web/src/services/database/queries.rs');
  const spotlight = read('crates/web/src/components/word_spotlight.rs');
  assert.match(queries, /SELECT relation_type, target_simplified, source_annotation FROM sense_relations/);
  assert.match(spotlight, /database\.word_relations\(word\.id\)/);
  assert.match(spotlight, /reference\.target_simplified/);
});
