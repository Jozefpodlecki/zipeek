const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = name => readFileSync(join(__dirname, '..', name), 'utf8');

test('Word Sets are managed on a dedicated route, not the dictionary page', () => {
  const routes = read('crates/web/src/route.rs');
  const browse = read('crates/web/src/pages/browse.rs');
  const page = read('crates/web/src/pages/word_sets.rs');
  const navigation = read('crates/web/src/components/navigation.rs');
  assert.match(routes, /\#\[at\("\/word-sets"\)\]/);
  assert.match(routes, /<WordSetsPage\s*\/>/);
  assert.match(page, /<WordSetPanel\s*\/>/);
  assert.match(page, /data-component="word-sets-page"/);
  assert.doesNotMatch(browse, /<WordSetPanel\s*\/>/);
  assert.match(navigation, /Route::WordSets/);
});

test('Word Sets provider supports an independent route and keeps browse sync optional', () => {
  const provider = read('crates/web/src/components/word_sets_provider.rs');
  assert.match(provider, /let browse = use_context::<UseReducerHandle<BrowseState>>\(\);/);
  assert.match(provider, /if let Some\(browse\) = &browse/);
  assert.match(provider, /WordSetsAction::Committed/);
});

test('Daily selection is stable by UTC day; Random selection draws fresh entropy', () => {
  const spotlight = read('crates/web/src/components/word_spotlight.rs');
  const home = read('crates/web/src/components/daily_word_card.rs');
  const random = read('crates/web/src/pages/random.rs');
  assert.match(spotlight, /Date::now\(\) \/ 86_400_000\.0/);
  assert.match(spotlight, /Some\(day\) => day % count/);
  assert.match(spotlight, /getrandom::fill\(&mut bytes\)/);
  assert.match(spotlight, /database\.word_at_offset\(offset, &exclusions\)/);
  assert.match(home, /WordSpotlight daily=\{true\}/);
  assert.match(random, /<crate::components::WordSpotlight\s*\/>/);
});
