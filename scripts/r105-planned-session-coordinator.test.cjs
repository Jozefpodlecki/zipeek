const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const source = fs.readFileSync(path.join(root, 'crates/zipeek-exercises/src/planned_session.rs'), 'utf8');
const lib = fs.readFileSync(path.join(root, 'crates/zipeek-exercises/src/lib.rs'), 'utf8');

test('planned session coordinator is exported and preserves the existing result ledger', () => {
  assert.match(lib, /pub mod planned_session;/);
  assert.match(source, /pub struct PlannedSession/);
  assert.match(source, /plan: SessionPlan/);
  assert.match(source, /ledger: QuizSession/);
});

test('result publication checks identity and commits both authorities only after validation', () => {
  assert.match(source, /step\.instance_id != instance_id/);
  assert.match(source, /step\.definition_id != definition_id/);
  assert.match(source, /next\.plan\.submit\(instance_id, result\)/);
  assert.match(source, /next\.ledger\.record_completed\(instance, result\)/);
  assert.match(source, /\*self = next;/);
});

test('advance requires acknowledgement and final ledger completion', () => {
  assert.match(source, /PlanStatus::AwaitingAcknowledgement/);
  assert.match(source, /next\.plan\.acknowledge\(instance_id\)/);
  assert.match(source, /next\.ledger\.complete\(\)/);
  assert.match(source, /fn invalid_messages_leave_both_authorities_unchanged\(/);
  assert.match(source, /fn final_acknowledgement_completes_ledger_and_plan_once\(/);
});
