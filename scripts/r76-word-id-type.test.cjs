const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const project = path.resolve(__dirname, '..');
const service = fs.readFileSync(path.join(project, 'crates/web/src/services/translation_vocabulary.rs'), 'utf8');
const word = fs.readFileSync(path.join(project, 'crates/zipeek-core/src/word.rs'), 'utf8');
const sets = fs.readFileSync(path.join(project, 'crates/web/src/services/word_sets.rs'), 'utf8');

test('saved Word Set i32 IDs use the domain i64 conversion path', () => {
    assert.match(sets, /pub word_ids: Vec<i32>/);
    assert.match(word, /impl TryFrom<i64> for WordId/);
    assert.match(service, /WordId::try_from\(i64::from\(id\)\)/);
    assert.doesNotMatch(service, /WordId::try_from\(id\)/);
});

test('translation saved-Word-Set loading retains validated IDs', () => {
    assert.match(service, /\.collect::<Result<BTreeSet<_>, _>>\(\)\?/);
    assert.match(service, /words_by_ids\(&ids\)\.await/);
    assert.match(word, /if !\(1\.\.=i64::from\(i32::MAX\)\)\.contains\(&value\)/);
});
