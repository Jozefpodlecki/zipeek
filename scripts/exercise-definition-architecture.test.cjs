const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.resolve(__dirname, '..');
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8');

test('exercise definitions and instances are independent of a central kind enum', () => {
    const definitions = read('crates/zipeek-exercises/src/definition.rs');
    const session = read('crates/zipeek-exercises/src/session.rs');
    assert.match(definitions, /pub struct ExerciseDefinition/);
    assert.match(definitions, /pub fn definition\(id: &str\)/);
    assert.doesNotMatch(definitions, /enum ExerciseKind/);
    assert.match(session, /pub struct ExerciseInstance/);
    assert.match(session, /pub struct QuizSession/);
    assert.match(session, /pub fn learner_context/);
});

test('web session uses independent visible indices, never common matching word IDs', () => {
    const reducer = read('crates/web/src/reducers/session.rs');
    const visible = reducer.split('fn visible_round(content: &RoundContent) -> String {')[1].split('fn record_current_round')[0];
    assert.match(visible, /L\{index\}/);
    assert.match(visible, /R\{index\}/);
    assert.doesNotMatch(visible, /word_id/);
    assert.match(reducer, /domain_session: Option<QuizSession>/);
    assert.match(reducer, /domain_session\.as_mut/);
});

test('AI simulated learner uses shared definition rather than scoring authority', () => {
    const ai = read('crates/zipeek-ai/src/prompts/exercise_simulation.rs');
    assert.match(ai, /instance\.learner_context\(\)/);
    assert.match(ai, /never grade your own response/);
    assert.match(read('crates/web/Trunk.toml'), /\.\.\/zipeek-exercises/);
});
