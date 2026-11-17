'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { inspectRun, renderReport } = require('../e2e-evidence-audit.cjs');

function fixture(callback) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'zipeek-evidence-audit-'));
  try {
    const dir = path.join(root, 'mosaics');
    fs.mkdirSync(path.join(dir, 'timeline'), { recursive: true });
    const scenario = { scenario: 'mosaics', status: 'passed', execution_status: 'passed', assertion_status: 'passed', recording_status: 'unavailable', failure_screenshot: false };
    fs.writeFileSync(path.join(dir, 'scenario.json'), JSON.stringify(scenario));
    fs.writeFileSync(path.join(dir, 'agent-feedback.md'), '# Evidence\n');
    fs.writeFileSync(path.join(dir, 'timeline', 'steps.jsonl'), JSON.stringify({ index: 0, unix_ms: 1000, label: 'ready', screenshot: null }) + '\n');
    fs.writeFileSync(path.join(root, 'suite.json'), JSON.stringify({ passed: 1, failed: 0, scenarios: [scenario] }));
    fs.writeFileSync(path.join(root, 'review.md'), '# Review\n');
    callback(root, dir, scenario);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
}

test('valid non-video run is consistent without strict evidence', () => fixture(root => {
  const report = inspectRun(root);
  assert.equal(report.valid, true);
  assert.equal(report.scenarios[0].checkpoints, 1);
  assert.match(renderReport(report), /Evidence integrity: PASS/);
}));

test('strict evidence rejects missing recordings', () => fixture(root => {
  assert.equal(inspectRun(root, { requireEvidence: true }).valid, false);
}));

test('detects missing screenshot and inconsistent suite totals', () => fixture((root, dir) => {
  fs.writeFileSync(path.join(dir, 'timeline', 'steps.jsonl'), JSON.stringify({ index: 0, unix_ms: 1000, label: 'ready', screenshot: '000-ready.png' }) + '\n');
  fs.writeFileSync(path.join(root, 'suite.json'), JSON.stringify({ passed: 0, failed: 0, scenarios: [{ scenario: 'mosaics', status: 'passed' }] }));
  const report = inspectRun(root);
  assert.equal(report.valid, false);
  assert.match(JSON.stringify(report.problems), /Missing checkpoint screenshot/);
  assert.match(JSON.stringify(report.problems), /totals disagree/);
}));

test('detects invalid validated video and corrupted PNG', () => fixture((root, dir, scenario) => {
  fs.mkdirSync(path.join(dir, 'video'));
  fs.writeFileSync(path.join(dir, 'video', 'recording.mp4'), 'bad');
  fs.writeFileSync(path.join(dir, 'video', 'status.json'), JSON.stringify({ validation: { status: 'invalid' }, captured: 1 }));
  fs.writeFileSync(path.join(dir, 'timeline', '000-ready.png'), 'bad png');
  fs.writeFileSync(path.join(dir, 'timeline', 'steps.jsonl'), JSON.stringify({ index: 0, unix_ms: 1000, label: 'ready', screenshot: '000-ready.png' }) + '\n');
  scenario.recording_status = 'validated';
  fs.writeFileSync(path.join(dir, 'scenario.json'), JSON.stringify(scenario));
  const report = inspectRun(root);
  assert.equal(report.valid, false);
  assert.match(JSON.stringify(report.problems), /validation disagrees/);
  assert.match(JSON.stringify(report.problems), /Invalid PNG signature/);
}));

test('rejects unsafe relative screenshot reference', () => fixture((root, dir) => {
  fs.writeFileSync(path.join(dir, 'timeline', 'steps.jsonl'), JSON.stringify({ index: 0, unix_ms: 1000, label: 'ready', screenshot: '../outside.png' }) + '\n');
  assert.match(JSON.stringify(inspectRun(root).problems), /Unsafe screenshot reference/);
}));
