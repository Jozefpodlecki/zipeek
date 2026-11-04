const test = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const read = file => readFileSync(file, 'utf8');

test('random spotlight follows the approved centered word discovery mockup', () => {
  const page = read('crates/web/src/pages/random.rs');
  const spotlight = read('crates/web/src/components/word_spotlight.rs');
  assert.match(page, /Word discovery/);
  assert.match(page, /items-center gap-8/);
  assert.match(spotlight, /data-spotlight-actions/);
  assert.match(spotlight, /data-next-random-word/);
  assert.match(spotlight, /LUCIDE_BOOK_OPEN/);
  assert.match(spotlight, /LUCIDE_FILE_TEXT/);
  assert.match(spotlight, /sm:grid-cols-3/);
  assert.doesNotMatch(spotlight, /uppercase tracking-\[\.13em\]/);
});

test('abbreviation notes are separate from meanings and can link to Browse', () => {
  const spotlight = read('crates/web/src/components/word_spotlight.rs');
  assert.match(spotlight, /database\.word_display_definition\(word\.id\)/);
  assert.doesNotMatch(spotlight, /fn display_gloss/);
  assert.match(spotlight, /data-word-cross-reference/);
  assert.match(spotlight, /encode_uri_component\(&reference.target_simplified\)/);
  assert.match(spotlight, /relation_label/);
});

test('About route and navigation contain actual project context', () => {
  const route = read('crates/web/src/route.rs');
  const nav = read('crates/web/src/components/navigation.rs');
  const about = read('crates/web/src/pages/about.rs');
  assert.match(route, /\[at\("\/about"\)\]/);
  assert.match(route, /Route::About => html!/);
  assert.match(nav, /Route::About/);
  assert.match(about, /Why Zipeek exists/);
  assert.match(about, /CC-CEDICT/);
  assert.match(about, /under active development/);
});
