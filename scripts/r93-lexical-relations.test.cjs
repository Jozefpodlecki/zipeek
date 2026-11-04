const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const read = file => readFileSync(file, 'utf8');

test('relationship types are constrained and original source sense is preserved', () => {
  const schema = read('crates/db-gen/src/schema.sql');
  const storage = read('crates/db-gen/src/import/storage.rs');
  for (const type of ['abbreviation_of', 'short_for', 'variant_of', 'also_written_as', 'see_also'])
    assert.match(schema, new RegExp(`'${type}'`));
  assert.match(storage, /INSERT INTO word_senses/);
  assert.match(storage, /reference\.relation_type/);
  assert.match(storage, /reference\.source_annotation/);
});

test('annotation-only senses are not stored as definitions', () => {
  const schema = read('crates/db-gen/src/schema.sql');
  const storage = read('crates/db-gen/src/import/storage.rs');
  assert.match(schema, /derivation = 'relation_only' AND definition IS NULL/);
  assert.match(storage, /Some\(value\) if value\.definition\.is_empty\(\) => \(None, "relation_only"\)/);
});

test('source parser rejects ambiguous cross-references and validates targets', () => {
  const parser = read('crates/db-gen/src/import/relations.rs');
  assert.match(parser, /contains\(" or "\)/);
  assert.match(parser, /contains\(" \+ "\)/);
  assert.match(parser, /is_chinese_reference\(simplified\)/);
  assert.match(parser, /extracts_reading_and_traditional_form/);
  assert.match(parser, /supports_explicit_variants_short_forms_and_cross_references/);
});
