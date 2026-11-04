const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.resolve(__dirname, '..');
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8');

const components = new Map([
  ['crates/web/src/pages/home.rs', ['home-page']],
  ['crates/web/src/pages/browse.rs', ['browse-page']],
  ['crates/web/src/components/browse/search_form.rs', ['browse-search-form']],
  ['crates/web/src/components/browse/results/mod.rs', ['browse-results']],
  ['crates/web/src/components/browse/results/status.rs', ['browse-loading', 'browse-error', 'browse-empty']],
  ['crates/web/src/components/browse/results/word_card.rs', ['browse-result']],
  ['crates/web/src/components/browse/results/pagination.rs', ['browse-pagination']],
  ['crates/web/src/components/browse/word_set_panel.rs', ['word-set-panel', 'word-set-manager', 'word-set-backup']],
  ['crates/web/src/pages/practice_setup.rs', ['practice-setup-page']],
  ['crates/web/src/pages/practice.rs', ['practice-page']],
  ['crates/web/src/pages/sentence_translation/view.rs', ['sentence-translation-page']],
  ['crates/web/src/components/app_shell.rs', ['app-shell', 'app-header', 'navigation-drawer', 'main-content']],
  ['crates/web/src/components/navigation.rs', ['main-navigation']],
  ['crates/web/src/components/theme_toggle.rs', ['theme-toggle']],
  ['crates/web/src/components/matching_practice.rs', ['matching-exercise']],
  ['crates/web/src/components/mcq_practice.rs', ['mcq-exercise']],
  ['crates/web/src/components/practice_summary.rs', ['practice-summary']],
  ['crates/web/src/components/route_placeholder.rs', ['route-placeholder']],
]);

test('component markers exist and use unique, stable identities', () => {
  const seen = new Set();
  for (const [file, values] of components) {
    const source = read(file);
    for (const name of values) {
      assert.match(name, /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/);
      assert.ok(!seen.has(name), `duplicate component marker: ${name}`);
      seen.add(name);
      assert.ok(source.includes(`data-component="${name}"`), `${file} missing ${name}`);
    }
  }
});

test('Browse smoke assertions use its real page, not a route placeholder', () => {
  const providers = read('crates/e2e/tests/suites/providers.rs');
  const shell = read('crates/e2e/tests/suites/shell.rs');
  assert.match(providers, /data-component='browse-page'/);
  assert.match(shell, /data-component='browse-page'/);
  assert.doesNotMatch(providers, /data-route-placeholder/);
  assert.equal((shell.match(/data-route-placeholder/g) ?? []).length, 1);
  assert.match(shell, /"Random"/);
  assert.match(read('crates/e2e/tests/suites/routing.rs'), /data-route-placeholder/);
});

test('Browse page composes feature components without owning persistence or search effects', () => {
  const page = read('crates/web/src/pages/browse.rs');
  const search = read('crates/web/src/components/browse/search_form.rs');
  const results = read('crates/web/src/components/browse/results/mod.rs');
  const pagination = read('crates/web/src/components/browse/results/pagination.rs');
  const status = read('crates/web/src/components/browse/results/status.rs');
  const panel = read('crates/web/src/components/browse/word_set_panel.rs');
  const provider = read('crates/web/src/components/word_sets_provider.rs');
  assert.match(page, /<BrowseSearchForm\s*\/>/);
  assert.match(page, /<BrowseResults\s*\/>/);
  assert.doesNotMatch(page, /<WordSetPanel\s*\/>/);
  assert.match(read('crates/web/src/pages/word_sets.rs'), /<WordSetPanel\s*\/>/);
  assert.doesNotMatch(page, /use_state\(|word_sets::write|browse_search\(/);
  assert.match(search, /data-browse-refresh/);
  assert.match(search, /use_effect_with/);
  assert.doesNotMatch(search, /data-browse-search=/);
  assert.match(results, /let status_component = match/);
  assert.match(results, /\{status_component\}/);
  assert.match(results, /<BrowseWordCard/);
  assert.match(results, /<BrowsePagination/);
  assert.match(pagination, /data-browse-next/);
  assert.match(status, /data-browse-error/);
  assert.match(panel, /data-word-set-backup-json/);
  assert.match(provider, /word_sets::write\(&next\)/);
});


test('Browse route uses explicit provider imports and readable nested markup', () => {
  const route = read('crates/web/src/route.rs');
  assert.match(route, /components::\{BrowseProvider, DatabaseBoundary, WordSetsProvider\}/);
  assert.match(route, /Route::Browse => html! \{\s*<BrowseProvider>\s*<WordSetsProvider>\s*<Browse \/>\s*<\/WordSetsProvider>\s*<\/BrowseProvider>\s*\},/);
  assert.match(route, /<DatabaseBoundary>\s*\{page\}\s*<\/DatabaseBoundary>/);
  assert.doesNotMatch(route, /<crate::components::/);
});


test('Browse error actions depend on reducer state, not message wording', () => {
  const results = read('crates/web/src/components/browse/results/mod.rs');
  const errorView = read('crates/web/src/components/browse/results/status.rs');
  assert.match(results, /can_retry=\{state\.blocked\.is_none\(\)\}/);
  assert.match(results, /show_regeneration_tip=\{state\.blocked\.is_none\(\)\}/);
  assert.match(errorView, /if !props\.can_retry/);
  assert.match(errorView, /if !props\.show_regeneration_tip/);
  assert.doesNotMatch(errorView, /message\.starts_with|message\.contains/);
});


test('Browse search, result preview, and reducer share one page-size contract', () => {
  const reducer = read('crates/web/src/reducers/browse.rs');
  const search = read('crates/web/src/components/browse/search_form.rs');
  const results = read('crates/web/src/components/browse/results/mod.rs');
  assert.match(reducer, /pub\(crate\) const PAGE_SIZE: u32 = 5;/);
  assert.match(search, /limit: PAGE_SIZE/);
  assert.match(results, /words\.len\(\) > PAGE_SIZE as usize/);
  assert.match(results, /words\.iter\(\)\.take\(PAGE_SIZE as usize\)/);
});

test('Browse automatically searches with a debounced query, immediate filters, and manual refresh', () => {
  const search = read('crates/web/src/components/browse/search_form.rs');
  const reducer = read('crates/web/src/reducers/browse.rs');
  const rows = read('crates/web/src/components/browse/results/mod.rs');
  assert.match(search, /use_effect_with\(dependencies/);
  assert.match(search, /Timeout::new\(delay/);
  assert.match(search, /320/);
  assert.match(search, /submit_search\(&state, &sets_snapshot, true\)/);
  assert.match(search, /data-browse-refresh/);
  assert.match(search, /data-browse-clear/);
  assert.doesNotMatch(search, /data-browse-search=/);
  assert.match(reducer, /BrowseAction::ClearFilters/);
  assert.match(rows, /data-component="browse-results-list"/);
  assert.doesNotMatch(rows, /grid gap-3 sm:grid-cols-2/);
});
