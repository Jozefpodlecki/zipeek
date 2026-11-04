const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const root = join(__dirname, '..');
const read = (path) => readFileSync(join(root, path), 'utf8');

test('speaking targets live in the exercise domain and validate spoken context', () => {
  const domain = read('crates/zipeek-exercises/src/speaking.rs');
  assert.match(domain, /pub struct SpeakingTarget/);
  assert.match(domain, /pub fn validate\(&self\)/);
  assert.match(domain, /pub fn instance\(&self\)/);
  assert.match(domain, /target\.focus_word/);
  assert.match(domain, /yínháng/);
  assert.match(domain, /yīnyuè/);
  assert.match(read('crates/zipeek-exercises/src/lib.rs'), /pub mod speaking;/);
});

test('changing sentence remounts recorder and renders validated exercise instance', () => {
  const page = read('crates/web/src/pages/speaking.rs');
  assert.match(page, /PREVIEW_TARGETS/);
  assert.match(page, /target\.instance\(\)/);
  assert.match(page, /data-exercise-definition/);
  assert.match(page, /data-action="select-speaking-target"/);
  assert.match(page, /<SpeakingRecorder key=\{target\.id\} \/>/);
  assert.doesNotMatch(page, /ExerciseResult|PracticeEvidence/);
});
