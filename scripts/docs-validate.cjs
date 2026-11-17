#!/usr/bin/env node
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const docs = path.join(root, 'docs');
const index = path.join(root, 'crates/zipeek-web/static/docs/index.json');
const failures = [];
const historicalWarnings = [];

function walk(directory) {
  return fs.readdirSync(directory, {withFileTypes: true})
    .flatMap(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : [path.join(directory, entry.name)])
    .sort();
}
function documentEntries() {
  return walk(docs).map(filename => {
    const relative = path.relative(docs, filename).split(path.sep).join('/');
    const markdown = filename.toLowerCase().endsWith('.md');
    const content = markdown ? fs.readFileSync(filename, 'utf8') : '';
    const header = /^#\s+(.+?)\s*$/m.exec(content);
    return {path: relative, title: header ? header[1] : path.basename(filename), markdown, search_text: content};
  });
}
function checkLinks(filename, content) {
  const relative = path.relative(root, filename).split(path.sep).join('/');
  const historic = relative.startsWith('docs/archive-history/completed/');
  const issue = message => (historic ? historicalWarnings : failures).push(`${relative}: ${message}`);
  // Remove fenced snippets and inline code to avoid interpreting code samples as URLs.
  const clean = content.replace(/^```[^\n]*\n[\s\S]*?^```\s*$/gm, '').replace(/`[^`\n]*`/g, '');
  const pattern = /!?\[[^\]\n]*\]\(([^)]+)\)/g;
  for (const match of clean.matchAll(pattern)) {
    const href = match[1].trim().replace(/^<|>$/g, '').split(' "')[0];
    if (!href || href.startsWith('#') || /^(https?:|mailto:|data:)/i.test(href)) continue;
    const raw = href.split('#')[0].split('?')[0];
    let uri;
    try { uri = decodeURIComponent(raw); } catch { issue(`invalid encoded path ${href}`); continue; }
    if (uri.startsWith('/')) { issue(`root-absolute reference ${href}`); continue; }
    const resolved = path.resolve(path.dirname(filename), uri);
    if (!resolved.startsWith(root + path.sep) || !fs.existsSync(resolved)) issue(`missing or outside-repository reference ${href}`);
  }
}
for (const filename of walk(docs)) {
  if (!filename.endsWith('.md')) continue;
  const content = fs.readFileSync(filename, 'utf8');
  if (content.includes('\u2014') || content.includes('\u2013')) failures.push(`${path.relative(root, filename)}: long dash punctuation`);
  checkLinks(filename, content);
}
try {
  const actual = JSON.parse(fs.readFileSync(index, 'utf8'));
  const expected = documentEntries();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push('Docs index differs from source files; regenerate with cargo make docs');
} catch (error) { failures.push(`Docs index unavailable or invalid: ${error.message}`); }

function checkE2eDocsRoutes() {
  const filename = path.join(root, 'crates/zipeek-e2e/tests/docs_reader.rs');
  const source = fs.readFileSync(filename, 'utf8');
  const routes = /"\/docs\/([^"\r\n]*?\.md)(?:\?heading=([a-z0-9-]+))?"/g;
  const referenced = new Set();
  for (const match of source.matchAll(routes)) {
    const relative = match[1].replaceAll('~', '/');
    const document = path.join(docs, relative);
    referenced.add(relative.split('/')[0]);
    if (!fs.existsSync(document)) {
      failures.push(`Docs E2E route targets missing document: ${relative}`);
      continue;
    }
    if (match[2]) {
      const markdown = fs.readFileSync(document, 'utf8');
      const headings = [...markdown.matchAll(/^#{1,6}\s+(.+)$/gm)]
        .map(heading => heading[1].trim().toLowerCase().replace(/[^\p{L}\p{N}\s-]/gu, '').replace(/\s+/g, '-'));
      if (!headings.includes(match[2])) {
        failures.push(`Docs E2E route targets missing heading: ${relative}#${match[2]}`);
      }
    }
  }
  const sectionSamples = /\(\s*"([A-Za-z0-9_./-]+\.md)"\s*,\s*"([^"]+)"\s*\)/g;
  const folders = new Set();
  for (const match of source.matchAll(sectionSamples)) {
    const relative = match[1];
    const folder = relative.split('/')[0];
    folders.add(folder);
    if (!fs.existsSync(path.join(docs, relative))) {
      failures.push(`Docs E2E section sample missing: ${relative}`);
    }
  }
  const required = [
    'getting-started', 'product-ux', 'learning-practice', 'games',
    'dictionary-data', 'architecture', 'engineering', 'testing-quality',
    'build-deployment', 'project-management', 'archive-history', 'issues'
  ];
  for (const folder of required) {
    if (!folders.has(folder)) failures.push(`Docs E2E has no browser section sample: ${folder}`);
  }
  if (referenced.size === 0) failures.push('Docs E2E route validation found no literal article routes');
}
checkE2eDocsRoutes();

for (const message of historicalWarnings) console.warn(`Historical warning: ${message}`);
for (const message of failures) console.error(`Error: ${message}`);
console.log(`Docs validation: ${failures.length} error(s), ${historicalWarnings.length} frozen-history warning(s)`);
if (failures.length) process.exitCode = 1;

function checkCurrentArtifactDocs() {
  const manifestFile = path.join(root, 'crates/zipeek-web/static/hanzi.manifest.json');
  const policyFile = path.join(root, 'crates/zipeek-db-gen/src/policy.rs');
  const currentDbDoc = path.join(docs, 'dictionary-data/schema/db.md');
  const manifest = JSON.parse(fs.readFileSync(manifestFile, 'utf8'));
  const policy = fs.readFileSync(policyFile, 'utf8');
  const doc = fs.readFileSync(currentDbDoc, 'utf8');
  const numeric = n => n.toLocaleString('en-US');
  const checks = [
    ['Dictionary `words` rows', numeric(manifest.rows)],
    ['Identity candidates', numeric(manifest.identity_candidates)],
    ['Source entries', numeric(manifest.source_entries)],
    ['Diagnostic events', numeric(manifest.diagnostics)],
    ['Native generator engine', `\`${manifest.engine}\``],
    ['Storage compatibility', `\`${manifest.storage_compatibility}\``],
  ];
  for (const [label, actual] of checks) {
    if (!doc.includes(`| ${label} | ${actual} |`)) {
      failures.push(`Current database doc has a stale ${label} value (expected ${actual} from installed manifest)`);
    }
  }
  const listed = manifest.files.find(f => f.path.endsWith('.duckdb'));
  if (!listed || !doc.includes(`| Read-only database file | \`${listed.path}\` |`)) {
    failures.push('Current database doc has stale artifact filename');
  }
  for (const [key, value] of [
    ['ENGINE_VERSION', manifest.engine],
    ['STORAGE_VERSION', manifest.storage_compatibility],
    ['WASM_PACKAGE', manifest.wasm_package]
  ]) {
    if (!policy.includes(`const ${key}: &str = "${value}";`)) {
      failures.push(`Generator policy ${key} disagrees with installed manifest`);
    }
  }
  if (!manifest.tables.includes('character_readings')) {
    failures.push('Expected character_readings missing from current manifest');
  }
}
checkCurrentArtifactDocs();
