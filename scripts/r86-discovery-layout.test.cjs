const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const read = (path) => fs.readFileSync(path, 'utf8');

test('Random has one visible title and centers the word spotlight', () => {
  const random = read('crates/web/src/pages/random.rs');
  const spotlight = read('crates/web/src/components/word_spotlight.rs');
  assert.doesNotMatch(random, /Discover a random word/);
  assert.match(random, /max-w-5xl/);
  assert.match(spotlight, /max-w-3xl flex-col items-center gap-4 text-center/);
  assert.match(spotlight, /data-word-metadata/);
  assert.match(spotlight, /word\.hsk3_level/);
  assert.match(spotlight, /word\.part_of_speech/);
  assert.match(spotlight, /word_categories/);
  assert.match(spotlight, /data-example-sentence/);
});

test('Spotlight passes the selected hanzi to Browse and Browse restores the query', () => {
  assert.match(read('crates/web/src/components/word_spotlight.rs'), /browse\?query=/);
  assert.match(read('crates/web/src/reducers/browse.rs'), /initial_dictionary_query/);
  assert.match(read('crates/web/src/reducers/browse.rs'), /decode_uri_component/);
});

test('drawer shows current route and stays as an icon rail after collapse', () => {
  const nav = read('crates/web/src/components/navigation.rs');
  const shell = read('crates/web/src/components/app_shell.rs');
  assert.match(nav, /if selected \{"bg-blue-600\/85 text-white shadow-sm"\}/);
  assert.match(nav, /Route::Settings/);
  assert.match(shell, /md:grid-cols-\[4\.5rem_minmax\(0,1fr\)\]/);
  assert.match(shell, /Navigation collapsed=/);
  assert.doesNotMatch(shell, /hidden=\{!\*expanded\}/);
});

test('Settings font choice persists and is applied to the Chinese typography selector', () => {
  assert.match(read('crates/web/src/route.rs'), /\[at\("\/settings"\)\]/);
  assert.match(read('crates/web/src/pages/settings.rs'), /data-hanzi-font-select/);
  assert.match(read('crates/web/index.html'), /zipeek\.chinese_font/);
  assert.match(read('crates/web/styles/main.css'), /\[lang\^="zh"\]/);
});
