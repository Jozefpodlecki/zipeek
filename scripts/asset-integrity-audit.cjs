#!/usr/bin/env node
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

function audit(root, mode = 'source') {
  if (!['source', 'deploy'].includes(mode)) throw new Error(`Unknown mode: ${mode}`);
  const staticRoot = path.join(root, 'crates/zipeek-web/static');
  const webRoot = path.join(root, 'crates/zipeek-web');
  const findings = [];
  const report = (severity, code, filename, detail) => findings.push({severity, code, file: filename, detail});
  const read = (filename) => {
    const target = path.join(root, filename);
    try {
      const stat = fs.lstatSync(target);
      if (!stat.isFile() || stat.size === 0) {
        report('error', 'INVALID_FILE', filename, 'Missing content or not a regular file');
        return null;
      }
      return fs.readFileSync(target);
    } catch (error) {
      report('error', 'MISSING_SOURCE', filename, error.message);
      return null;
    }
  };
  const manifestPath = 'crates/zipeek-web/static/hanzi.manifest.json';
  const manifestRaw = read(manifestPath);
  let manifest = null;
  if (manifestRaw) {
    try { manifest = JSON.parse(manifestRaw.toString('utf8')); }
    catch (error) { report('error', 'INVALID_MANIFEST', manifestPath, error.message); }
  }
  const htmlPath = 'crates/zipeek-web/index.html';
  const htmlRaw = read(htmlPath);
  const html = htmlRaw?.toString('utf8') || '';
  if (manifest) {
    if (!Array.isArray(manifest.files) || manifest.files.length === 0) {
      report('error', 'INVALID_MANIFEST', manifestPath, 'No manifest files');
    } else {
      const visited = new Set();
      for (const entry of manifest.files) {
        const name = entry?.path;
        if (typeof name !== 'string' || !name || name.includes('\\') || name.includes(':') || name.split('/').some(part => !part || part === '.' || part === '..') || path.posix.isAbsolute(name) || visited.has(name)) {
          report('error', 'INVALID_MANIFEST_PATH', manifestPath, `Invalid or repeated path ${JSON.stringify(name)}`);
          continue;
        }
        visited.add(name);
        const filename = path.posix.join('crates/zipeek-web/static', name);
        const target = path.join(staticRoot, ...name.split('/'));
        let stat;
        try { stat = fs.lstatSync(target); } catch { stat = null; }
        if (!stat) {
          const expectedGenerated = /\.duckdb(?:\.gz)?$/.test(name);
          const onlyBuildInput = name.endsWith('.duckdb');
          const severity = !expectedGenerated || (mode === 'deploy' && !onlyBuildInput) ? 'error' : 'notice';
          report(severity, 'GENERATED_ASSET_ABSENT', filename, 'Generate and install this manifest-declared artifact before deployment');
          continue;
        }
        if (!stat.isFile() || stat.isSymbolicLink()) {
          report('error', 'INVALID_ASSET_TYPE', filename, 'Expected a regular, non-symlink file');
          continue;
        }
        if (!Number.isSafeInteger(entry.size_bytes) || entry.size_bytes <= 0 || stat.size !== entry.size_bytes) {
          report('error', 'ASSET_SIZE_MISMATCH', filename, `Expected ${entry.size_bytes}, observed ${stat.size}`);
          continue;
        }
        if (!/^[a-f0-9]{64}$/.test(entry.sha256 || '')) {
          report('error', 'INVALID_ASSET_HASH', filename, 'Expected SHA-256 is missing or malformed');
          continue;
        }
        const actual = crypto.createHash('sha256').update(fs.readFileSync(target)).digest('hex');
        if (actual !== entry.sha256) report('error', 'ASSET_HASH_MISMATCH', filename, `Expected ${entry.sha256}, observed ${actual}`);
      }
      const preloadNames = [...html.matchAll(/<link\b[^>]*rel=["']preload["'][^>]*>/gi)]
        .map(match => /\bhref=["']([^"']+)["']/.exec(match[0])?.[1]).filter(Boolean);
      const artifact = manifest.files.find(entry => typeof entry.path === 'string' && entry.path.endsWith('.duckdb.gz'));
      if (!artifact) report('error', 'MISSING_DATABASE_ARTIFACT', manifestPath, 'Manifest has no compressed dictionary');
      else if (!preloadNames.includes(artifact.path)) report('error', 'STALE_DATABASE_PRELOAD', htmlPath, `Preload must reference ${artifact.path}`);
      if (!preloadNames.includes('hanzi.manifest.json')) report('error', 'MISSING_MANIFEST_PRELOAD', htmlPath, 'Manifest should be preloaded');
    }
  }
  for (const file of ['crates/zipeek-web/static/courses/grammar-autumn-bridge.png', 'crates/zipeek-web/static/courses/vocabulary-peony-garden.png', 'crates/zipeek-web/static/courses/pinyin-landscape.png']) {
    const bytes = read(file);
    if (bytes && (bytes.length < 24 || !bytes.subarray(0, 8).equals(Buffer.from('89504e470d0a1a0a', 'hex')) || bytes.toString('ascii', 12, 16) !== 'IHDR')) {
      report('error', 'INVALID_PNG', file, 'Expected PNG header and IHDR');
    }
  }
  for (const name of ['dragon', 'qilin', 'phoenix', 'white-tiger', 'crane', 'deer']) {
    const filename = `crates/zipeek-web/static/practice/${name}.webp`;
    const bytes = read(filename);
    if (bytes && (bytes.length < 30 || bytes.toString('ascii', 0, 4) !== 'RIFF' || bytes.toString('ascii', 8, 12) !== 'WEBP')) {
      report('error', 'INVALID_WEBP', filename, 'Expected RIFF WebP header');
    }
  }
  const worker = 'crates/zipeek-web/static/pedagogy-worker';
  const names = ['worker.mjs', 'pedagogy_wasm.js', 'pedagogy_wasm_bg.wasm', 'pedagogy_wasm.d.ts', 'pedagogy_wasm_bg.wasm.d.ts'];
  const exists = names.map(name => fs.existsSync(path.join(webRoot, 'static/pedagogy-worker', name)));
  const bindings = exists.slice(1);
  if (bindings.some(Boolean) && !bindings.every(Boolean)) report('error', 'PARTIAL_WORKER', worker, 'Partial worker output must be rebuilt atomically');
  if (bindings.every(Boolean) && !exists[0]) report('error', 'MISSING_WORKER_BOOTSTRAP', worker, 'Generated bindings require worker.mjs');
  if (!exists.every(Boolean)) report(mode === 'deploy' ? 'error' : 'notice', 'WORKER_NOT_BUILT', worker, 'Generate the complete pedagogy Worker before deployment');
  if (exists.every(Boolean)) {
    const js = fs.readFileSync(path.join(webRoot, 'static/pedagogy-worker/pedagogy_wasm.js'), 'utf8');
    const bootstrap = fs.readFileSync(path.join(webRoot, 'static/pedagogy-worker/worker.mjs'), 'utf8');
    const wasm = fs.readFileSync(path.join(webRoot, 'static/pedagogy-worker/pedagogy_wasm_bg.wasm'));
    if (!js.includes('start_worker') || !bootstrap.includes('start_worker') || !bootstrap.includes('./pedagogy_wasm.js')) report('error', 'INVALID_WORKER_BOOTSTRAP', worker, 'Worker entrypoints disagree');
    if (wasm.length < 8 || !wasm.subarray(0, 4).equals(Buffer.from([0, 97, 115, 109]))) report('error', 'INVALID_WORKER_WASM', worker, 'WASM header is invalid');
  }
  const requiredGeography = ['provinces.json', 'prefectures.json'];
  for (const name of requiredGeography) {
    const filename = `crates/zipeek-web/static/province-quest/${name}`;
    const target = path.join(root, filename);
    if (!fs.existsSync(target)) {
      report(mode === 'deploy' ? 'error' : 'notice', 'GEOGRAPHY_NOT_BUILT', filename, 'Generate Province Quest geography before deployment');
      continue;
    }
    try {
      const value = JSON.parse(fs.readFileSync(target, 'utf8'));
      if (!value || typeof value !== 'object' || Array.isArray(value)) report('error', 'INVALID_GEOGRAPHY', filename, 'Expected JSON object');
    } catch (error) { report('error', 'INVALID_GEOGRAPHY', filename, error.message); }
  }
  return {mode, passed: !findings.some(finding => finding.severity === 'error'), findings};
}

function render(result) {
  const lines = [`Asset integrity: ${result.passed ? 'PASS' : 'FAIL'} (${result.mode})`];
  for (const item of result.findings) lines.push(`${item.severity.toUpperCase()} ${item.code} ${item.file}: ${item.detail}`);
  return lines.join('\n');
}

if (require.main === module) {
  const args = process.argv.slice(2);
  const mode = args.includes('--deploy') ? 'deploy' : 'source';
  const rootArg = args.find(arg => arg.startsWith('--root='));
  const root = rootArg ? path.resolve(rootArg.slice('--root='.length)) : path.resolve(__dirname, '..');
  try {
    const report = audit(root, mode);
    process.stdout.write(`${render(report)}\n`);
    process.exitCode = report.passed ? 0 : 1;
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}

module.exports = {audit, render};
