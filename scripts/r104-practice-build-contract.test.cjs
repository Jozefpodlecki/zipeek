const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const root = join(__dirname, '..');
const page = readFileSync(join(root, 'crates/web/src/pages/practice_setup.rs'), 'utf8');

function balancedRustDelimiters(source) {
  const stack = [];
  let quoted = false;
  let escaped = false;
  const opens = '([{';
  const closes = ')]}';
  for (const token of source) {
    if (quoted) {
      if (escaped) escaped = false;
      else if (token === '\\') escaped = true;
      else if (token === '"') quoted = false;
      continue;
    }
    if (token === '"') { quoted = true; continue; }
    if (opens.includes(token)) stack.push(token);
    if (closes.includes(token)) {
      const prev = stack.pop();
      assert.equal(opens.indexOf(prev), closes.indexOf(token), 'Unbalanced markup/Rust delimiter');
    }
  }
  assert.equal(quoted, false, 'Unterminated Rust string');
  assert.deepEqual(stack, [], 'Unclosed markup/Rust delimiters');
}

test('R103 Summary nested HTML branches have balanced macro delimiters', () => {
  balancedRustDelimiters(page);
  assert.match(page, /Choose a Matching question format in Configure\."\}<\/p>\}\}\}/);
  assert.match(page, /Choose a Multiple choice question format in Configure\."\}<\/p>\}\}\}/);
});

test('draft owns shared Fixed/Endless mode and UI/launch use it consistently', () => {
  assert.match(page, /legacy_round_mode\(&draft\.length\)/);
  assert.match(page, /next\.length = if fixed/);
  assert.match(page, /next\.length = SessionLength::Fixed\(u16::from\(rounds\.count\(\)\)\)/);
  assert.match(page, /match &draft\.length \{ SessionLength::Fixed\(count\)/);
  assert.doesNotMatch(page, /SetupAction::Fixed\(/);
  assert.doesNotMatch(page, /SetupAction::Rounds\(/);
  assert.match(page, /SessionLength::Fixed\(_\) => None/);
});

test('Summary previews a real fixed scored plan without claiming unified launch is working', () => {
  assert.match(page, /SessionPlan::from_draft\(&draft\)/);
  assert.match(page, /data-practice-plan-preview/);
  assert.match(page, /data-practice-planned-steps/);
  assert.match(page, /preview only/);
  assert.match(page, /Unified multi-exercise progression is still being integrated/);
});
