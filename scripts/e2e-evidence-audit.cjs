#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

function inspectRun(root, options = {}) {
  const problems = [];
  const scenarios = [];
  const add = (severity, location, message) => problems.push({ severity, location, message });
  const readJson = (file, required = true) => {
    try { return JSON.parse(fs.readFileSync(file, 'utf8')); }
    catch (error) {
      if (required || error.code !== 'ENOENT') add('error', path.relative(root, file) || file, `Unreadable JSON: ${error.message}`);
      return null;
    }
  };
  const suite = readJson(path.join(root, 'suite.json'));
  if (!suite) return { root, scenarios, problems, valid: false };
  if (!Array.isArray(suite.scenarios)) add('error', 'suite.json', 'Missing scenarios array');
  const actual = fs.readdirSync(root, { withFileTypes: true })
    .filter(entry => entry.isDirectory() && fs.existsSync(path.join(root, entry.name, 'scenario.json')))
    .map(entry => entry.name).sort();
  const reported = new Set();
  for (const name of actual) {
    const directory = path.join(root, name);
    const result = readJson(path.join(directory, 'scenario.json'));
    if (!result) continue;
    if (result.scenario !== name) add('error', name, `Scenario ID mismatch: ${result.scenario}`);
    if (!['passed', 'failed'].includes(result.status)) add('error', name, 'Invalid scenario status');
    if (!['passed', 'failed', 'not_run'].includes(result.assertion_status)) add('error', name, 'Invalid assertion status');
    if (!['validated', 'failed', 'unverified', 'truncated', 'unavailable'].includes(result.recording_status)) add('error', name, 'Invalid recording status');
    if (!fs.existsSync(path.join(directory, 'agent-feedback.md'))) add('error', name, 'Missing Codex feedback report');
    if (result.failure_screenshot && !fs.existsSync(path.join(directory, 'failure.png'))) add('error', name, 'Manifest refers to missing failure screenshot');
    const video = path.join(directory, 'video', 'recording.mp4');
    if (result.recording_status === 'validated' && (!fs.existsSync(video) || fs.statSync(video).size === 0)) add('error', name, 'Validated recording is missing or empty');
    if (options.decodeVideo && fs.existsSync(video)) {
      const decoded = spawnSync('ffmpeg', ['-hide_banner', '-v', 'error', '-i', video, '-f', 'null', '-'], { encoding: 'utf8', timeout: 120000, maxBuffer: 1024 * 1024 });
      if (decoded.error || decoded.status !== 0) add('error', name, `MP4 decode failed: ${decoded.error ? decoded.error.message : (decoded.stderr || 'ffmpeg failed').trim()}`);
    }
    const videoStatus = readJson(path.join(directory, 'video', 'status.json'), false);
    if (videoStatus) {
      const reportedStatus = videoStatus.validation && videoStatus.validation.status;
      if (result.recording_status === 'validated' && reportedStatus !== 'valid') add('error', name, 'Manifest claims validated recording but FFprobe validation disagrees');
      if (videoStatus.truncated && result.recording_status !== 'truncated' && result.recording_status !== 'failed') add('error', name, 'Truncated video is not reported as truncated');
      if (result.recording_status === 'validated' && !(videoStatus.captured > 0)) add('error', name, 'Validated recording has no captured frames');
    }
    if (result.status === 'passed' && result.execution_status !== 'passed') add('error', name, 'Conflicting execution status');
    if (result.status === 'failed' && !result.error) add('error', name, 'Failed scenario has no recorded error');
    const timeline = path.join(directory, 'timeline', 'steps.jsonl');
    let checkpoints = 0;
    if (fs.existsSync(timeline)) {
      let previousIndex = -1;
      let previousTime = -1;
      for (const [offset, line] of fs.readFileSync(timeline, 'utf8').split(/\r?\n/).entries()) {
        if (!line.trim()) continue;
        let step;
        try { step = JSON.parse(line); }
        catch (error) { add('error', name, `Malformed timeline line ${offset + 1}: ${error.message}`); continue; }
        checkpoints++;
        if (!Number.isSafeInteger(step.index) || step.index !== previousIndex + 1) add('error', name, `Out-of-order checkpoint index on line ${offset + 1}`);
        if (!Number.isSafeInteger(step.unix_ms) || step.unix_ms < previousTime) add('error', name, `Invalid checkpoint timestamp on line ${offset + 1}`);
        if (typeof step.label !== 'string' || !/^[\w-]{1,80}$/.test(step.label)) add('error', name, `Invalid checkpoint label on line ${offset + 1}`);
        if (step.screenshot != null) {
          if (typeof step.screenshot !== 'string' || !/^\d{3}-[\w-]+\.png$/.test(step.screenshot)) add('error', name, `Unsafe screenshot reference on line ${offset + 1}`);
          else {
            const image = path.join(directory, 'timeline', step.screenshot);
            if (!fs.existsSync(image)) add('error', name, `Missing checkpoint screenshot: ${step.screenshot}`);
            else {
              const sig = Buffer.alloc(8);
              const fd = fs.openSync(image, 'r');
              try { fs.readSync(fd, sig, 0, 8, 0); } finally { fs.closeSync(fd); }
              if (!sig.equals(Buffer.from('89504e470d0a1a0a', 'hex'))) add('error', name, `Invalid PNG signature: ${step.screenshot}`);
            }
          }
        }
        previousIndex = step.index;
        previousTime = step.unix_ms;
      }
    } else if (options.requireEvidence) add('error', name, 'No checkpoint timeline');
    if (options.requireEvidence && result.recording_status !== 'validated') add('error', name, `Evidence mode requires validated video, got ${result.recording_status}`);
    if (options.requireEvidence && checkpoints === 0) add('error', name, 'Evidence mode requires checkpoints');
    scenarios.push({ name, status: result.status, assertion: result.assertion_status, video: result.recording_status, checkpoints });
  }
  if (Array.isArray(suite.scenarios)) {
    for (const entry of suite.scenarios) {
      if (typeof entry.scenario !== 'string') { add('error', 'suite.json', 'Scenario without name'); continue; }
      if (reported.has(entry.scenario)) add('error', 'suite.json', `Duplicate scenario ${entry.scenario}`);
      reported.add(entry.scenario);
      if (!actual.includes(entry.scenario)) add('error', 'suite.json', `Missing scenario directory ${entry.scenario}`);
    }
    for (const name of actual) if (!reported.has(name)) add('error', 'suite.json', `Scenario omitted from suite: ${name}`);
    if (suite.passed !== suite.scenarios.filter(entry => entry.status === 'passed').length || suite.failed !== suite.scenarios.filter(entry => entry.status === 'failed').length) add('error', 'suite.json', 'Suite pass/fail totals disagree with scenario manifest');
  }
  if (!fs.existsSync(path.join(root, 'review.md'))) add('warning', 'review.md', 'Missing human-readable suite report');
  return { root, scenarios, problems, valid: !problems.some(problem => problem.severity === 'error') };
}

function renderReport(report) {
  const lines = ['# E2E evidence audit', '', `Run: ${report.root}`, '', `Evidence integrity: ${report.valid ? 'PASS' : 'FAIL'}`, '', '| Scenario | Execution | Assertion | Video | Checkpoints |', '| --- | --- | --- | --- | ---: |'];
  for (const caseResult of report.scenarios) lines.push(`| ${caseResult.name} | ${caseResult.status} | ${caseResult.assertion} | ${caseResult.video} | ${caseResult.checkpoints} |`);
  lines.push('', '## Findings', '');
  if (!report.problems.length) lines.push('No evidence integrity issues found. This does not establish that the application behavior is correct.');
  else for (const problem of report.problems) lines.push(`- ${problem.severity.toUpperCase()} ${problem.location}: ${problem.message}`);
  lines.push('', 'This report checks artifact consistency, not the semantic correctness of E2E assertions or the visual content of the video.', '');
  return lines.join('\n');
}

if (require.main === module) {
  const args = process.argv.slice(2);
  if (args.includes('--help') || !args.length || args.filter(arg => !arg.startsWith('--')).length !== 1) {
    process.stdout.write('Usage: node scripts/e2e-evidence-audit.cjs <target/e2e/run-directory> [--strict-evidence] [--decode-video] [--write-report]\n');
    process.exit(args.includes('--help') ? 0 : 2);
  }
  const root = path.resolve(args.find(arg => !arg.startsWith('--')));
  if (!fs.existsSync(root) || !fs.statSync(root).isDirectory()) {
    process.stderr.write(`E2E run directory does not exist: ${root}\n`);
    process.exit(2);
  }
  const report = inspectRun(root, { requireEvidence: args.includes('--strict-evidence'), decodeVideo: args.includes('--decode-video') });
  const markdown = renderReport(report);
  process.stdout.write(markdown);
  if (args.includes('--write-report')) {
    fs.writeFileSync(path.join(root, 'evidence-audit.md'), markdown);
    fs.writeFileSync(path.join(root, 'evidence-audit.json'), JSON.stringify(report, null, 2) + '\n');
  }
  process.exit(report.valid ? 0 : 1);
}

module.exports = { inspectRun, renderReport };
