const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const root = join(__dirname, '..');
const page = readFileSync(join(root, 'crates/web/src/pages/practice_setup.rs'), 'utf8');
const reducer = readFileSync(join(root, 'crates/web/src/reducers/setup.rs'), 'utf8');

test('exercise formats are owned by the draft, not the legacy reducer UI state', () => {
  const draft = readFileSync(join(root, 'crates/zipeek-exercises/src/practice_draft.rs'), 'utf8');
  assert.match(draft, /pub matching_formats: Vec<ExerciseVariant>/);
  assert.match(draft, /pub multiple_choice_formats: Vec<ExerciseVariant>/);
  assert.match(draft, /pub fn active_formats/);
  assert.match(draft, /pub fn validate_exercise_settings/);
  assert.match(page, /draft\.active_formats\(\)/);
  assert.match(page, /next\.set_format\(variant, enabled\)/);
  assert.doesNotMatch(page, /SetupAction::Activity\(variant\)/);
});

test('summary exposes per-family formats and existing session options', () => {
  assert.match(page, /data-practice-summary-configuration/);
  for (const family of ['matching', 'multiple-choice', 'translation', 'speaking']) {
    assert.match(page, new RegExp(`data-summary-family="${family}"`));
  }
  assert.match(page, /Fixed · \{\} rounds/);
  assert.match(page, /Choose a Matching question format in Configure/);
  assert.match(page, /Choose a Multiple choice question format in Configure/);
});

test('combined legacy launch refuses a selected family with no configured format', () => {
  assert.match(page, /draft\.has_exercise\("matching"\) && !selected_variants\.iter\(\)\.any\(\|variant\| !variant\.is_mcq\(\)\)/);
  assert.match(page, /draft\.has_exercise\("multiple-choice"\) && !selected_variants\.iter\(\)\.any\(\|variant\| variant\.is_mcq\(\)\)/);
  assert.match(page, /data-practice-start/);
  assert.match(page, /data-translation-setup-start/);
});
