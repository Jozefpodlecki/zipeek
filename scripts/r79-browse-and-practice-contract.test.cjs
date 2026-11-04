const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = file => readFileSync(join(__dirname, '..', file), 'utf8');

test('Browse shares a five-row page size and lookahead navigation', () => {
  const reducer = read('crates/web/src/reducers/browse.rs');
  const query = read('crates/web/src/services/database/browse_queries.rs');
  const result = read('crates/web/src/components/browse/results/mod.rs');
  assert.match(reducer, /const PAGE_SIZE: u32 = 5;/);
  assert.match(query, /request\.limit \+ 1/);
  assert.match(result, /words\.iter\(\)\.take\(PAGE_SIZE as usize\)/);
  assert.match(result, /words\.len\(\) > PAGE_SIZE as usize/);
});

test('Browse row reuses the existing ListenButton and shows hanzi/pinyin inline', () => {
  const row = read('crates/web/src/components/browse/results/word_card.rs');
  const audio = read('crates/web/src/components/listen_button.rs');
  assert.match(row, /<ListenButton hanzi=\{props\.word\.simplified\.clone\(\)\} compact=\{true\}/);
  assert.match(row, /flex-wrap items-baseline gap-x-2/);
  assert.match(audio, /utterance\.set_lang\("zh-CN"\)/);
  assert.match(row, /data-toggle-saved-word/);
  assert.doesNotMatch(row, /data-practice-word/);
});

test('Browse handoff requires a selected nonempty set and setup reads source', () => {
  const page = read('crates/web/src/pages/browse.rs');
  const panel = read('crates/web/src/components/browse/word_set_panel.rs');
  const practice = read('crates/web/src/pages/practice_setup.rs');
  assert.doesNotMatch(page, /Translate selected vocabulary|data-browse-open-practice/);
  assert.match(panel, /data-practice-selected-set/);
  assert.match(panel, /disabled=\{set\.word_ids\.is_empty\(\)\}/);
  assert.match(panel, /\("source", source\.as_str\(\)\)/);
  assert.match(practice, /query\.get\("source"\)/);
  assert.match(practice, /starts_with\("saved:"\)/);
});

test('Browse pagination and refresh use the approved icon package', () => {
  const pagination = read('crates/web/src/components/browse/results/pagination.rs');
  const form = read('crates/web/src/components/browse/search_form.rs');
  assert.match(pagination, /IconData::LUCIDE_CHEVRON_LEFT/);
  assert.match(pagination, /IconData::LUCIDE_CHEVRON_RIGHT/);
  assert.match(pagination, /aria-label="Next page"/);
  assert.match(form, /IconData::LUCIDE_REFRESH_CW/);
});

test('Approved practice UX explicitly separates shipped and deferred scope', () => {
  const doc = read('docs/product/PRACTICE_SETUP_V2.md');
  assert.match(doc, /Personal Word Set/);
  assert.match(doc, /category autocomplete/);
  assert.match(doc, /versioned configuration snapshots/);
  assert.match(doc, /not yet connected/);
});
