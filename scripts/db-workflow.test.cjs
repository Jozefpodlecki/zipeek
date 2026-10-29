const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {validateBundle, installBundle} = require('./db-workflow.cjs');

const repository = path.resolve(__dirname, '..');
const base = path.join(repository, 'target/a46-workflow');
fs.mkdirSync(base, {recursive: true});

function fixture() {
    const root = fs.mkdtempSync(path.join(base, 'fixture-'));
    fs.mkdirSync(path.join(root, 'crates/db-gen/data'), {recursive: true});
    fs.copyFileSync(path.join(repository, 'crates/db-gen/data/MANIFEST.toml'), path.join(root, 'crates/db-gen/data/MANIFEST.toml'));
    fs.mkdirSync(path.join(root, 'web'));
    const source = path.join(root, 'candidate');
    fs.cpSync(path.join(repository, 'web/static'), source, {recursive: true});
    fs.cpSync(source, path.join(root, 'web/static'), {recursive: true});
    fs.writeFileSync(path.join(root, 'web/static/previous-only.txt'), 'preserve me');
    return {root, source, destination: path.join(root, 'web/static')};
}

test('complete installation retains matching notices and preserves the previous directory', () => {
    const {root, source, destination} = fixture();
    const result = installBundle(root, source);
    assert.equal(validateBundle(root, destination).report.rows, 12100);
    assert.equal(fs.readFileSync(path.join(result.backup, 'previous-only.txt'), 'utf8'), 'preserve me');
    assert.equal(fs.existsSync(path.join(destination, 'previous-only.txt')), false);
    assert.ok(fs.existsSync(path.join(destination, 'notices/CC-BY-SA-4.0.txt')));
});

test('checksum failure leaves the installed bundle untouched', () => {
    const {root, source, destination} = fixture();
    const file = path.join(source, 'hanzi.manifest.json');
    const report = JSON.parse(fs.readFileSync(file, 'utf8'));
    report.files[0].sha256 = '0'.repeat(64);
    fs.writeFileSync(file, JSON.stringify(report));
    assert.throws(() => installBundle(root, source), /checksum mismatch/);
    assert.equal(fs.readFileSync(path.join(destination, 'previous-only.txt'), 'utf8'), 'preserve me');
    assert.equal(validateBundle(root, destination).report.rows, 12100);
});

test('manifest traversal and missing notices are refused before replacement', () => {
    const {root, source} = fixture();
    const file = path.join(source, 'hanzi.manifest.json');
    const report = JSON.parse(fs.readFileSync(file, 'utf8'));
    const original = report.files[0].path;
    report.files[0].path = '../outside';
    fs.writeFileSync(file, JSON.stringify(report));
    assert.throws(() => validateBundle(root, source), /Invalid bundle path/);
    report.files[0].path = original;
    report.files = report.files.filter(file => file.path !== 'notices/HSK-LICENSE.txt');
    fs.writeFileSync(file, JSON.stringify(report));
    assert.throws(() => installBundle(root, source), /Required bundle file missing/);
});

test('publication failure restores the previous complete bundle', () => {
    const {root, source, destination} = fixture();
    const rename = fs.renameSync;
    fs.renameSync = (from, to) => {
        if (String(from).includes(`${path.sep}staging${path.sep}`) && to === destination) throw new Error('simulated publication failure');
        return rename(from, to);
    };
    try {
        assert.throws(() => installBundle(root, source), /simulated publication failure/);
    } finally {
        fs.renameSync = rename;
    }
    assert.equal(fs.readFileSync(path.join(destination, 'previous-only.txt'), 'utf8'), 'preserve me');
    assert.equal(validateBundle(root, destination).report.rows, 12100);
});
