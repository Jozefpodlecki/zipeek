const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const page = readFileSync(join(__dirname, '../crates/web/src/pages/practice_setup.rs'), 'utf8');
const builder = readFileSync(join(__dirname, '../crates/web/src/components/practice_builder.rs'), 'utf8');
const spec = readFileSync(join(__dirname, '../docs/product/PRACTICE_SESSION_BUILDER.md'), 'utf8');

test('four independent top-level Practice steps with summary and explicit next/back', () => {
  for (const step of ['exercises', 'vocabulary', 'configure', 'summary']) {
    assert.match(page, new RegExp('data-practice-step=.*|"' + step + '"'));
  }
  assert.match(page, /data-practice-step-tabs/);
  assert.match(page, /data-practice-summary-actions/);
  assert.match(page, /data-practice-step-navigation/);
  assert.match(page, /data-practice-config-tabs|section=\{"configure"/);
});

test('selecting an exercise does not silently switch the configuration tab', () => {
  const onclick = builder.match(/onclick=\{Callback::from\(move \|_\| \{([\s\S]*?)\}\)\}/);
  assert.ok(onclick);
  assert.match(onclick[1], /on_toggle.emit\(id\)/);
  assert.doesNotMatch(onclick[1], /on_tab.emit/);
});

test('design explicitly keeps unsupported unified start out of this checkpoint', () => {
  assert.match(spec, /Exercise Types.*Vocabulary.*Configure.*Summary/s);
  assert.match(page, /Unified multi-exercise progression is still being integrated/);
  assert.match(page, /Start Matching \/ MCQ/);
  assert.match(page, /Start translation only/);
});

test('HSK version first picker restricts visible levels and cleans hidden selections', () => {
  const vocabulary = readFileSync(join(__dirname, '../crates/web/src/components/practice_vocabulary_source.rs'), 'utf8');
  assert.match(vocabulary, /data-vocabulary-hsk-standard/);
  assert.match(vocabulary, /data-hsk-standard=/);
  assert.match(vocabulary, /filters\.hsk\.retain\(\|band\| band\.system == system\)/);
  assert.match(vocabulary, /data-vocabulary-hsk-multi/);
  assert.match(vocabulary, /\.filter\(\|\(system, _\)\|/);
});
