const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = p => readFileSync(join(__dirname, '..', p), 'utf8');
const dir = 'crates/web/src/pages/sentence_translation/';

test('recorder invokes wasm-bindgen instance methods, not unresolved free functions', () => {
  const source = read('crates/web/src/components/speaking_recorder.rs');
  for (const method of ['dispose', 'start', 'stop', 'discard']) {
    assert.match(source, new RegExp(`active\\.recorder\\.${method}\\(\\);`));
    assert.doesNotMatch(source, new RegExp(`(?<!\\.)\\b${method}\\(&active\\.recorder\\)`));
  }
});

test('translation page uses focused modules rather than a giant hook and markup function', () => {
  const module = read('crates/web/src/pages/sentence_translation.rs');
  const settings = read(dir + 'settings.rs');
  const actions = read(dir + 'actions.rs');
  const view = read(dir + 'view.rs');
  const state = read(dir + 'state.rs');
  assert.match(module, /mod actions;/);
  assert.match(module, /mod settings;/);
  assert.match(module, /mod state;/);
  assert.match(module, /mod view;/);
  assert.ok(module.split('\n').length < 110);
  assert.match(settings, /data-word-set/);
  assert.match(actions, /spawn_local\(async move/);
  assert.match(view, /data-component="sentence-translation-page"/);
  assert.match(state, /pub fn reset_configuration/);
});

test('translation async callbacks clone their controller for each request, preserving Fn', () => {
  const actions = read(dir + 'actions.rs');
  const state = read(dir + 'state.rs');
  assert.equal((actions.match(/let task_state = state\.clone\(\);/g) || []).length, 2);
  assert.equal((actions.match(/spawn_local\(async move/g) || []).length, 2);
  assert.match(actions, /if \*task_state\.request_epoch\.borrow\(\) != request_id/);
  assert.match(state, /self\.abandon_pending\(\);/);
  assert.match(state, /self\.invalidate_request\(\);/);
});
