const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');

const read = path => fs.readFileSync(path, 'utf8');

test('browser loads the database named by a fresh manifest', () => {
    const bridge = read('crates/web/src/services/database/bridge.js');
    assert.match(bridge, /hanzi\.manifest\.json/);
    assert.match(bridge, /cache: 'no-store'/);
    assert.match(bridge, /new URL\(filename, document\.baseURI\)/);
    assert.match(bridge, /db\.registerFileBuffer\(filename/);
});

test('installer versions published databases and validates the rewritten bundle', () => {
    const script = read('scripts/db-workflow.cjs');
    assert.match(script, /hanzi-\$\{date\.toISOString\(\)/);
    assert.match(script, /fs\.renameSync\(path\.join\(stage, existingDatabase\)/);
    assert.match(script, /validateBundle\(root, stage\)/);
});

test('web compiler regressions use supported syntax without moving borrowed text', () => {
    const random = read('crates/web/src/pages/random.rs');
    const spotlight = read('crates/web/src/components/word_spotlight.rs');
    assert.doesNotMatch(random, /LUCIDE_SPARKLES/);
    assert.match(spotlight, /\{&reference\.target_simplified\}/);
});
