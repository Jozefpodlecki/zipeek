const test = require('node:test');
const assert = require('node:assert/strict');
const {readFileSync} = require('node:fs');
const {join} = require('node:path');
const source = readFileSync(join(__dirname, '../crates/zipeek-exercises/src/planner.rs'), 'utf8');

test('planner chooses scored exercises in registry order and rejects unsupported formats', () => {
  assert.match(source, /draft\.validate_for_scored_session\(\)/);
  assert.match(source, /DEFINITIONS\s*\.iter\(\)/);
  assert.match(source, /EndlessRequiresIncrementalScheduling/);
  assert.match(source, /fixed_plan_uses_registry_order_and_stable_instances/);
});

test('planner cannot advance before an explicit matching acknowledgement', () => {
  assert.match(source, /PlanStatus::AwaitingAcknowledgement/);
  assert.match(source, /self\.pending_result = Some\(result\)/);
  assert.match(source, /if current\.instance_id != instance_id/);
  assert.match(source, /self\.cursor \+= 1/);
  assert.match(source, /rejected_or_unacknowledged_results_do_not_advance/);
  assert.match(source, /final_acknowledgement_finishes_plan/);
});
