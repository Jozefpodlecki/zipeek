const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const read = path => readFileSync(path, 'utf8');

test('original senses remain untouched while cleaned definitions occupy their own table', () => {
  const schema = read('crates/db-gen/src/schema.sql');
  const storage = read('crates/db-gen/src/import/storage.rs');
  assert.match(schema, /CREATE TABLE sense_display/);
  assert.match(schema, /derivation IN \('source_verbatim', 'explicit_annotation_removed', 'relation_only'\)/);
  assert.match(storage, /INSERT INTO word_senses/);
  assert.match(storage, /INSERT INTO sense_display/);
  assert.match(storage, /match reference.as_ref\(\)/);
  assert.match(storage, /source_verbatim/);
});

test('relations support multiple ordered source-backed links without duplicating meanings', () => {
  const schema = read('crates/db-gen/src/schema.sql');
  const storage = read('crates/db-gen/src/import/storage.rs');
  assert.match(schema, /relation_order INTEGER NOT NULL/);
  assert.match(schema, /PRIMARY KEY \(word_id, sense_order, relation_order\)/);
  assert.match(schema, /source_annotation VARCHAR NOT NULL/);
  assert.match(schema, /extraction_method VARCHAR NOT NULL/);
  assert.doesNotMatch(schema.slice(schema.indexOf('CREATE TABLE sense_relations ('), schema.indexOf('CREATE TABLE parts_of_speech')), /definition VARCHAR/);
  assert.match(storage, /explicit_source_annotation/);
});

test('snapshot and runtime both use the normalized schema', () => {
  const snapshot = read('crates/db-gen/src/artifact/snapshot.rs');
  const query = read('crates/web/src/services/database/queries.rs');
  assert.match(snapshot, /pub sense_display: Vec<String>/);
  assert.match(snapshot, /SELECT to_json\(sense_display\)/);
  assert.match(query, /FROM sense_display WHERE word_id = \?/);
  assert.match(query, /ORDER BY sense_order, relation_order/);
});
