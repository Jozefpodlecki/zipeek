const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const root = join(__dirname, '..');
const page = readFileSync(join(root, 'crates/web/src/pages/practice_setup.rs'), 'utf8');
const draft = readFileSync(join(root, 'crates/zipeek-exercises/src/practice_draft.rs'), 'utf8');
test('translation count is stored in draft and read by the launch and summary', () => {
  assert.match(draft, /pub translation_count: u16/);
  assert.match(page, /next.translation_count = count/);
  assert.match(page, /draft.translation_count == 0/);
  assert.doesNotMatch(page, /let translation_count = use_state/);
});
test('selected formats drive preflight, summary, and launch from the same draft', () => {
  assert.match(page, /let selected_variants = draft.active_formats\(\)/);
  assert.match(page, /let types = draft.active_formats\(\)/);
  assert.match(page, /&draft.active_formats\(\)/);
  assert.match(draft, /fn exercise_settings_are_independent_and_retained_on_deselection/);
  assert.match(draft, /fn invalid_family_formats_and_translation_counts_rejected/);
});
