const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const root = join(__dirname, '..');
const read = path => readFileSync(join(root, path), 'utf8');

const translationFiles = ['state.rs', 'settings.rs', 'actions.rs', 'view.rs'];
const translationSource = () => [read('crates/web/src/pages/sentence_translation.rs'), ...translationFiles.map(file => read(`crates/web/src/pages/sentence_translation/${file}`))].join('\n');

test('theme resolution and static provider belong to a standalone crate', () => {
  const workspace = read('Cargo.toml');
  const provider = read('crates/zipeek-themes/src/provider.rs');
  const resolver = read('crates/zipeek-themes/src/resolver.rs');
  assert.match(workspace, /"crates\/zipeek-themes"/);
  assert.match(provider, /pub async fn load/);
  assert.match(provider, /public\/background.webp/);
  assert.match(resolver, /pub fn resolve_theme/);
  assert.doesNotMatch(provider + resolver, /yew|reqwest|AiMux|DuckDB/);
});

test('themed background loads asynchronously and protects stale completions', () => {
  const view = read('crates/web/src/components/themed_background.rs');
  assert.match(view, /use_effect_with/);
  assert.match(view, /spawn_local/);
  assert.match(view, /pending.get\(\)/);
  assert.match(view, /aria-hidden="true"/);
});

test('translation feedback has explicit acknowledge-to-continue phase', () => {
  const page = translationSource();
  const view = read('crates/web/src/components/translation_feedback.rs');
  assert.match(page, /Stage::Feedback/);
  assert.match(page, /<TranslationFeedback/);
  assert.match(page, /sentence.english.clone\(\)/);
  assert.match(page, /sentence.chinese.clone\(\)/);
  assert.match(view, /data-feedback-result/);
  assert.match(view, /data-feedback-reference/);
  assert.match(view, /data-feedback-continue/);
  assert.match(view, /role="status"/);
  assert.match(view, /Not quite right/);
  assert.doesNotMatch(view, /checkbox|on_confirm/);
});

test('classification and source artifacts are unchanged by theme work', () => {
  const classifier = read('crates/db-gen/src/llm/prompts/classification.rs');
  assert.match(classifier, /sense-classification-v3/);
  const manifest = read('crates/web/Cargo.toml');
  assert.match(manifest, /zipeek-themes = \{ path = "\.\.\/zipeek-themes" \}/);
});
