const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8');

test('exported e2e assertion macros resolve their public types', () => {
  const source = read('crates/e2e/src/lib.rs');
  for (const name of ['AssertionFailure', 'AssertionKind']) assert.match(source, new RegExp(`\\b${name}\\b`));
});
test('fixed Translation setup and completion are independently represented', () => {
  const setup = read('crates/web/src/pages/practice_setup.rs');
  const state = read('crates/web/src/pages/sentence_translation/state.rs');
  const actions = read('crates/web/src/pages/sentence_translation/actions.rs');
  const view = read('crates/web/src/pages/sentence_translation/view.rs');
  assert.match(setup, /data-translation-setup-count/);
  assert.match(setup, /"count", if draft\.translation_count == 0/);
  assert.match(state, /round_limit: Option<u32>/);
  assert.match(actions, /stage\.set\(Stage::Complete\)/);
  assert.match(view, /data-translation-complete/);
});
test('thirtyfour runs five HSK6 sentences through setup and requires final completion', () => {
  const scenario = read('crates/e2e/tests/translation_fixture.rs');
  assert.match(scenario, /five_hsk6_translations_from_setup/);
  assert.match(scenario, /filters:3\.6\|\|/);
  assert.match(scenario, /for question in 1\.\.=5/);
  assert.match(scenario, /page\.continue_after_feedback\(\)\.await\?/);
  assert.match(scenario, /\[data-translation-complete\]/);
});
