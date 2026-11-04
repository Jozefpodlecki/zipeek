const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const root = join(__dirname, '..');
const domain = readFileSync(join(root, 'crates/zipeek-exercises/src/practice_draft.rs'), 'utf8');
const builder = readFileSync(join(root, 'crates/web/src/components/practice_builder.rs'), 'utf8');
const page = readFileSync(join(root, 'crates/web/src/pages/practice_setup.rs'), 'utf8');

test('Configure includes General plus selected family tabs in a fixed order', () => {
  for (const pair of ['("matching", "matching")', '("multiple-choice", "multiple-choice")', '("sentence-translation", "translation")', '("speaking", "speaking")']) {
    assert.ok(domain.includes(pair), pair);
  }
  assert.match(builder, /data-practice-config-tabs/);
  assert.match(builder, /overflow-x-auto/);
  assert.match(builder, /"multiple-choice" => "Multiple choice"/);
});

test('Matching and MCQ have independent visible variant controls', () => {
  assert.match(page, /data-practice-variant-settings/);
  assert.match(page, /variant\.is_mcq\(\) == \(\*tab == "multiple-choice"\)/);
  assert.match(page, /next\.set_format\(variant, enabled\)/);
  assert.doesNotMatch(page, /<ActivityPicker/);
});

test('General retains existing round controls, Translation retains real launch count, Speaking remains preview', () => {
  assert.match(page, /data-mode-picker/);
  assert.match(page, /data-round-counts/);
  assert.match(page, /data-translation-setup-count/);
  assert.match(page, /data-speaking-setup/);
  assert.match(page, /data-practice-general-settings/);
  assert.match(page, /Unified multi-exercise progression is still being integrated/);
});
