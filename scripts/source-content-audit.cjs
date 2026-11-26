'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const root = path.resolve(__dirname, '..');
const output = process.argv[2] ? path.resolve(process.argv[2]) : path.join(root, 'target', 'source-content-audit.jsonl');
const textTypes = new Set(['.rs', '.js', '.cjs', '.mjs', '.ts', '.css', '.html', '.sql', '.toml', '.yml', '.md', '.txt', '.csv', '.tsv', '.webmanifest', '.lock', '.json']);
const patterns = {
  panic: /\b(?:panic!|todo!|unimplemented!)\s*\(/g,
  unwrap: /\.(?:unwrap|expect)\s*\(/g,
  unsafe: /\bunsafe\s*(?:\{|extern|fn)/g,
  asynchronous: /\b(?:spawn_local|tokio::spawn)\s*\(/g,
  component: /#\[function_component/g,
  hook: /\buse_(?:state|reducer|mut_ref|effect_with)\s*\(/g,
  storage: /\b(?:local_storage|session_storage|localStorage|sessionStorage)\b/g,
};
function walk(dir) {
  return fs.readdirSync(dir, {withFileTypes: true}).flatMap(entry => {
    if (entry.isDirectory()) {
      if (['target', 'node_modules', '.git', '.unpack-backups'].includes(entry.name)) return [];
      return walk(path.join(dir, entry.name));
    }
    return entry.isFile() ? [path.join(dir, entry.name)] : [];
  }).sort();
}
function classify(filename, bytes) {
  const extension = path.extname(filename).toLowerCase();
  const metrics = {};
  const problems = [];
  let kind = 'binary';
  if (textTypes.has(extension) || extension === '') {
    try {
      const decoded = new TextDecoder('utf-8', {fatal: true}).decode(bytes);
      kind = 'utf8';
      metrics.lines = decoded ? decoded.split('\n').length - Number(decoded.endsWith('\n')) : 0;
      if (extension === '.json' || extension === '.webmanifest') {
        if (path.relative(root, filename).split(path.sep).join('/') === '.vscode/settings.json') {
          kind = 'jsonc';
        } else {
          try { const value = JSON.parse(decoded); kind = 'json'; metrics.rootType = Array.isArray(value) ? 'array' : typeof value; }
          catch (error) { problems.push(`invalid_json:${error.message}`); }
        }
      }
      if (['.rs', '.js', '.cjs', '.mjs', '.ts'].includes(extension)) {
        for (const [name, pattern] of Object.entries(patterns)) {
          const matches = decoded.match(pattern);
          if (matches) metrics[name] = matches.length;
        }
      }
    } catch (error) { problems.push(`invalid_utf8:${error.message}`); }
  } else if (extension === '.wasm') {
    kind = 'wasm';
    if (!bytes.subarray(0, 4).equals(Buffer.from([0, 97, 115, 109]))) problems.push('invalid_wasm_magic');
  } else if (extension === '.png') {
    kind = 'png';
    if (!bytes.subarray(0, 8).equals(Buffer.from('89504e470d0a1a0a', 'hex'))) problems.push('invalid_png_header');
  } else if (extension === '.webp') {
    kind = 'webp';
    if (bytes.toString('ascii', 0, 4) !== 'RIFF' || bytes.toString('ascii', 8, 12) !== 'WEBP') problems.push('invalid_webp_header');
  }
  return {kind, metrics, problems};
}
fs.mkdirSync(path.dirname(output), {recursive: true});
if (output.startsWith(root + path.sep) && !output.includes(path.sep + 'target' + path.sep)) throw Error('Output inside source tree must be under target/');
let count = 0;
let failures = 0;
const handle = fs.openSync(output, 'w');
try {
  for (const filename of walk(root)) {
    if (filename === output) continue;
    const bytes = fs.readFileSync(filename);
    const check = classify(filename, bytes);
    const entry = {
      path: path.relative(root, filename).split(path.sep).join('/'),
      bytes: bytes.length,
      sha256: crypto.createHash('sha256').update(bytes).digest('hex'),
      ...check,
    };
    fs.writeSync(handle, JSON.stringify(entry) + '\n');
    count++;
    failures += check.problems.length;
  }
} finally { fs.closeSync(handle); }
console.log(`Content scan: ${count} files read, ${failures} structural failures. Ledger: ${output}`);
if (failures) process.exitCode = 1;
