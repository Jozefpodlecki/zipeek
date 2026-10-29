const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const {spawnSync} = require('node:child_process');

const repository = path.resolve(__dirname, '..');
const manifestName = 'hanzi.manifest.json';
const required = ['hanzi.duckdb', 'hanzi.diagnostics.json', 'INPUTS.toml', 'ATTRIBUTION.md', 'notices/HSK-LICENSE.txt', 'notices/HSK-SOURCE-README.txt', 'notices/CC-BY-SA-4.0.txt'];
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const unique = () => `${Date.now()}-${crypto.randomUUID()}`;

function inside(root, target) {
    const resolved = path.resolve(target);
    const relative = path.relative(path.resolve(root), resolved);
    if (!relative || relative.startsWith(`..${path.sep}`) || relative === '..' || path.isAbsolute(relative)) throw new Error(`Path must stay inside ${root}: ${resolved}`);
    return resolved;
}

function regular(root, relative) {
    if (typeof relative !== 'string' || !relative || relative.includes('\\') || relative.includes(':') || relative.split('/').some(part => !part || part === '.' || part === '..')) throw new Error(`Invalid bundle path: ${relative}`);
    const file = inside(root, path.join(root, relative));
    let cursor = root;
    for (const part of relative.split('/')) {
        cursor = path.join(cursor, part);
        if (fs.lstatSync(cursor).isSymbolicLink()) throw new Error(`Bundle symlink refused: ${cursor}`);
    }
    if (!fs.statSync(file).isFile()) throw new Error(`Expected bundle file: ${file}`);
    return file;
}

function secureDirectory(root, directory) {
    const resolved = inside(root, directory);
    let cursor = root;
    for (const part of path.relative(root, resolved).split(path.sep)) {
        cursor = path.join(cursor, part);
        if (!fs.existsSync(cursor)) fs.mkdirSync(cursor);
        if (fs.lstatSync(cursor).isSymbolicLink() || fs.realpathSync(cursor) !== cursor) throw new Error(`Directory must not use symlinks: ${cursor}`);
    }
    return resolved;
}

function validateBundle(root, directory) {
    const bundle = inside(root, directory);
    if (fs.lstatSync(bundle).isSymbolicLink() || fs.realpathSync(bundle) !== bundle) throw new Error(`Bundle must use its real path: ${bundle}`);
    const report = JSON.parse(fs.readFileSync(regular(bundle, manifestName), 'utf8'));
    if (report.engine !== 'v2.0.0' || report.storage_compatibility !== 'v1.4.0' || report.wasm_package !== '@duckdb/duckdb-wasm@1.32.0') throw new Error('Bundle engine/storage/WASM policy mismatch');
    if (!Number.isSafeInteger(report.rows) || report.rows < 0 || !Array.isArray(report.files)) throw new Error('Malformed bundle report');
    const names = new Set();
    for (const file of report.files) {
        if (names.has(file.path) || file.path === manifestName) throw new Error(`Duplicate/self-referencing bundle entry: ${file.path}`);
        names.add(file.path);
        const bytes = fs.readFileSync(regular(bundle, file.path));
        if (bytes.length !== file.size_bytes || hash(bytes) !== file.sha256) throw new Error(`Bundle checksum mismatch: ${file.path}`);
    }
    for (const file of required) if (!names.has(file)) throw new Error(`Required bundle file missing: ${file}`);
    const input = report.files.find(file => file.path === 'INPUTS.toml');
    if (input.sha256 !== hash(fs.readFileSync(path.join(root, 'crates/db-gen/data/MANIFEST.toml')))) throw new Error('Bundle does not match the current pinned input manifest');
    return {bundle, report, files: [...names, manifestName]};
}

function workflow(root) {
    const directory = inside(root, path.join(root, 'target/db-workflow'));
    secureDirectory(root, directory);
    if (fs.realpathSync(directory) !== directory) throw new Error('Workflow directory must not use symlinks');
    return directory;
}

function generate(root) {
    const directory = workflow(root);
    secureDirectory(root, path.join(directory, 'generated'));
    const output = inside(directory, path.join(directory, 'generated', unique()));
    const args = ['run', '-p', 'db-gen', '--locked', '--', 'generate', '--output', output];
    const result = spawnSync('cargo', args, {cwd: root, stdio: 'inherit'});
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`Generator failed (${result.status ?? result.signal}); installed bundle unchanged`);
    const checked = validateBundle(root, output);
    const pointer = path.join(directory, 'latest.json');
    const temporary = path.join(directory, `latest-${unique()}.json`);
    fs.writeFileSync(temporary, JSON.stringify({directory: path.relative(root, checked.bundle)}, null, 2), {flag: 'wx'});
    fs.renameSync(temporary, pointer);
    console.log(`Candidate bundle: ${checked.bundle}\nReadings: ${checked.report.rows}\nInstall it with: cargo make db-install`);
    return checked.bundle;
}

function installBundle(root, source) {
    const checked = validateBundle(root, source);
    const directory = workflow(root);
    const destination = inside(root, path.join(root, 'web/static'));
    const stage = inside(directory, path.join(directory, 'staging', unique()));
    const backup = inside(directory, path.join(directory, 'backups', unique()));
    if (checked.bundle === destination) throw new Error('Candidate is already the installed bundle');
    if (fs.realpathSync(path.dirname(destination)) !== path.dirname(destination)) throw new Error('Install parent must not use symlinks');
    if (fs.existsSync(destination) && fs.lstatSync(destination).isSymbolicLink()) throw new Error('Installed bundle symlink refused');
    secureDirectory(root, stage);
    for (const name of checked.files) {
        const output = inside(stage, path.join(stage, name));
        fs.mkdirSync(path.dirname(output), {recursive: true});
        fs.copyFileSync(regular(checked.bundle, name), output, fs.constants.COPYFILE_EXCL);
    }
    validateBundle(root, stage);
    secureDirectory(root, path.dirname(backup));
    let moved = false;
    try {
        if (fs.existsSync(destination)) {
            fs.renameSync(destination, backup);
            moved = true;
        }
        fs.renameSync(stage, destination);
    } catch (error) {
        if (moved) {
            try {fs.renameSync(backup, destination);} catch (rollback) {throw new Error(`Install failed: ${error.message}. Restore backup ${backup}: ${rollback.message}`);}
        }
        throw error;
    }
    console.log(`Installed database: ${path.join(destination, 'hanzi.duckdb')}\nReadings: ${checked.report.rows}\nPrevious bundle: ${moved ? backup : 'none'}`);
    return {destination, backup: moved ? backup : null};
}

function main() {
    const command = process.argv[2];
    if (command === 'generate') return generate(repository);
    if (command === 'install') {
        const directory = workflow(repository);
        const pointer = path.join(directory, 'latest.json');
        if (!fs.existsSync(pointer)) throw new Error('No generated candidate. Run cargo make db-generate first.');
        const latest = JSON.parse(fs.readFileSync(pointer, 'utf8'));
        return installBundle(repository, inside(directory, path.resolve(repository, latest.directory)));
    }
    throw new Error('Usage: node scripts/db-workflow.cjs <generate|install>');
}

if (require.main === module) {
    try {main();} catch (error) {console.error(`Database workflow failed: ${error.message}`); process.exitCode = 1;}
}

module.exports = {validateBundle, installBundle};
