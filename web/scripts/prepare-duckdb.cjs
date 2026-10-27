const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const packageRoot = path.dirname(path.dirname(require.resolve('@duckdb/duckdb-wasm')));
const arrowRoot = path.dirname(require.resolve('apache-arrow'));
const version = JSON.parse(fs.readFileSync(path.join(packageRoot, 'package.json'), 'utf8')).version;
if (version !== '1.32.0') throw new Error(`DuckDB-WASM must be 1.32.0, found ${version}`);
const output = path.join(root, 'vendor', 'duckdb');
fs.mkdirSync(output, {recursive: true});
for (const file of ['duckdb-browser-mvp.worker.js', 'duckdb-browser-eh.worker.js', 'duckdb-mvp.wasm', 'duckdb-eh.wasm']) {
    fs.copyFileSync(path.join(packageRoot, 'dist', file), path.join(output, file));
}
const moduleSource = fs.readFileSync(path.join(packageRoot, 'dist', 'duckdb-browser.mjs'), 'utf8');
if (!moduleSource.includes('from"apache-arrow"')) throw new Error('Pinned DuckDB Arrow import was not found');
fs.writeFileSync(path.join(output, 'duckdb-browser.mjs'), moduleSource.replaceAll('from"apache-arrow"', 'from"./arrow.mjs"'));
fs.copyFileSync(path.join(arrowRoot, 'Arrow.es2015.min.js'), path.join(output, 'arrow.js'));
fs.copyFileSync(path.join(root, 'scripts', 'arrow.mjs'), path.join(output, 'arrow.mjs'));
for (const [source, name] of [[packageRoot, 'DUCKDB'], [arrowRoot, 'ARROW']]) {
    const license = fs.readdirSync(source).find(file => /^LICENSE(?:\.|$)/i.test(file));
    if (!license && name !== 'DUCKDB') throw new Error(`Missing ${name} license`);
    fs.copyFileSync(license ? path.join(source, license) : path.join(__dirname, 'duckdb-LICENSE.txt'), path.join(output, `${name}-LICENSE.txt`));
}
fs.copyFileSync(path.join(arrowRoot, 'NOTICE.txt'), path.join(output, 'ARROW-NOTICE.txt'));
