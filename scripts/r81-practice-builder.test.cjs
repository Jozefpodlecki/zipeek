const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const read = file => readFileSync(join(__dirname, '..', file), 'utf8');

test('practice builder uses the validated domain draft for selection and tabs', () => {
  const source = read('crates/web/src/pages/practice_setup.rs');
  const builder = read('crates/web/src/components/practice_builder.rs');
  assert.match(source, /next\.toggle_exercise\(id\)/);
  assert.match(source, /next\.configuration_tabs\(\)\.contains/);
  assert.match(builder, /props\.draft\.has_exercise\(id\)/);
  assert.match(builder, /props\.draft\.configuration_tabs\(\)/);
  assert.match(builder, /aria-pressed=\{selected\.to_string\(\)\}/);
  assert.match(builder, /disabled=\{disabled\}/);
  assert.match(builder, /IconData::LUCIDE_GRADUATION_CAP/);
});

test('existing route-specific launch paths remain gated and identified honestly', () => {
  const source = read('crates/web/src/pages/practice_setup.rs');
  const scenario = read('crates/e2e/tests/translation_fixture.rs');
  assert.match(source, /draft\.has_exercise\("sentence-translation"\)/);
  assert.match(source, /draft\.has_exercise\("speaking"\)/);
  assert.match(source, /draft\.has_exercise\("multiple-choice"\)/);
  assert.match(source, /draft\.has_exercise\("matching"\)/);
  assert.match(source, /Start Matching \/ MCQ/);
  assert.match(source, /Start translation only/);
  assert.match(scenario, /translation settings must be conditional/);
});

test('Browse saved set handoff uses matching index and translation source token', () => {
  const source = read('crates/web/src/pages/practice_setup.rs');
  assert.match(source, /preselected_source\.strip_prefix\("saved:"\)/);
  assert.match(source, /draft.vocabulary = VocabularySource::PersonalSet/);
  assert.match(source, /format!\("saved:\{reference\}"\)/);
});
