const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const read = name => fs.readFileSync(path.join(__dirname, '..', name), 'utf8');

test('filtered Matching route carries the serialized source across sessions', () => {
  const setup = read('crates/web/src/pages/practice_setup.rs');
  const query = read('crates/zipeek-core/src/practice/normalize.rs');
  assert.match(setup, /query.insert\("source"\.to_owned\(\), crate::services::practice_filter_source::encode\(filters\)\)/);
  assert.match(query, /pairs.push\(\("source"\.to_owned\(\), source.clone\(\)\)\)/);
  assert.match(query, /Some\(source\) => format!\("\{legacy\}\|\{source\}"\)/);
});

test('filtered pool uses the same dictionary predicate without LIMIT and never widens', () => {
  const loader = read('crates/web/src/services/practice_loader.rs');
  const db = read('crates/web/src/services/database/practice_filter_queries.rs');
  assert.match(db, /practice_exact_pool_for_filters/);
  assert.match(db, /practice_filter_predicate\(filters\)\?/);
  assert.match(db, /AND w\.id NOT IN/);
  assert.match(db, /ORDER BY w\.id ASC/);
  assert.match(loader, /hsk_widened: words.clone\(\), hsk_widened_any_pos: words/);
  assert.match(loader, /build_query.word_set = Some\(initial.strict.iter\(\).map\(\|word\| word.id\).collect\(\)\)/);
  assert.match(loader, /&build_query/);
});

test('malformed and mixed-source filtered URLs fail closed', () => {
  const parser = read('crates/zipeek-core/src/practice/query.rs');
  const loader = read('crates/web/src/services/practice_loader.rs');
  assert.match(parser, /query\.dictionary_source = Some/);
  assert.match(parser, /!fields\.contains_key\("seed"\)/);
  assert.match(parser, /!fields\.contains_key\("set_ids"\)/);
  assert.match(loader, /warning\.field == "source"/);
  assert.match(loader, /practice_filter_source::decode/);
});
