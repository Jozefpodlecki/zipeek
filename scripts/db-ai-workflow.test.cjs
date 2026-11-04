const {test} = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const {full, smoke, settings, pendingFromPlan, checkEnrichmentSchema} = require('./db-ai-workflow.cjs');

const root = path.resolve(__dirname, '..');
const config = {
    model: 'local-test-model', endpoint: 'http://127.0.0.1:1234',
    cache: path.join(root, 'target/db-workflow/classification-cache/test.jsonl'),
    timeout: 300000, retries: 1,
};

function harness(pending) {
    const calls = [];
    let planIndex = 0;
    const runner = (_command, args, options) => {
        calls.push({kind: 'cargo', args, options});
        const action = args[5];
        if (action === 'classify-plan') return {status: 0, stdout: `Model: local-test-model\nPending senses: ${pending[Math.min(planIndex++, pending.length - 1)]}\n`};
        if (action === 'classify' || action === 'ai-smoke') return {status: 0};
        throw new Error(`Unexpected command: ${action}`);
    };
    const candidate = path.join(root, 'target/db-workflow/generated/test');
    const options = {
        config, runner,
        generate: (location, settings) => {calls.push({kind: 'generate', location, settings}); return candidate;},
        validateBundle: () => {calls.push({kind: 'validate'}); return {report: {tables: ['words', 'word_senses', 'sense_categories', 'word_pos_membership', 'word_hsk_membership', 'sense_pos', 'classification_runs', 'parts_of_speech', 'categories']}};},
        checkBrowseSchema: report => {calls.push({kind: 'schema', report});},
        installBundle: (_location, source) => {calls.push({kind: 'install', source}); return {destination: 'installed'};},
    };
    return {calls, options};
}

test('full workflow classifies pending senses before generating, validating and installing', () => {
    const {calls, options} = harness([3, 0]);
    full(root, options);
    assert.deepEqual(calls.map(item => item.kind), ['cargo', 'cargo', 'cargo', 'cargo', 'generate', 'validate', 'schema', 'install']);
    assert.equal(calls[1].args[5], 'ai-smoke');
    assert.equal(calls[2].args[5], 'classify');
    assert.equal(calls[2].args[calls[2].args.indexOf('--limit') + 1], '3');
    assert.equal(calls[4].settings.classificationCache, config.cache);
});

test('full workflow with complete cache skips the inference and still publishes an enriched artifact', () => {
    const {calls, options} = harness([0, 0]);
    full(root, options);
    assert.equal(calls.filter(call => call.args?.[6] === 'classify').length, 0);
    assert.equal(calls.at(-1).kind, 'install');
});

test('incomplete AI classification never publishes or installs', () => {
    const {calls, options} = harness([2, 1]);
    assert.throws(() => full(root, options), /1 senses remain unclassified/);
    assert.equal(calls.some(call => call.kind === 'generate' || call.kind === 'install'), false);
});

test('failed model call preserves installed bundle and reusable cache', () => {
    const {calls, options} = harness([2]);
    options.runner = (command, args, params) => {
        const response = harnessRun(command, args, params);
        return response;
    };
    function harnessRun(_command, args) {
        calls.push({kind: 'cargo', args});
        return args[5] === 'classify' ? {status: 1} : {status: 0, stdout: 'Pending senses: 2\n'};
    }
    assert.throws(() => full(root, options), /installed bundle unchanged/);
    assert.equal(calls.some(call => call.kind === 'generate' || call.kind === 'install'), false);
});

test('invalid plan output is rejected before writing anything', () => {
    assert.throws(() => pendingFromPlan('No counts available'), /Could not determine/);
    assert.throws(() => pendingFromPlan('Pending senses: 1\nPending senses: 0\n'), /Could not determine/);
});

test('settings reject invalid numeric values, empty models, and non-http endpoints', () => {
    assert.throws(() => settings(root, {ZIPEEK_AI_MODEL: ' '}), /must not be empty/);
    assert.throws(() => settings(root, {ZIPEEK_AI_RETRIES: '-1'}), /integer/);
    assert.throws(() => settings(root, {ZIPEEK_AI_TIMEOUT_MS: '0'}), /positive/);
    assert.throws(() => settings(root, {ZIPEEK_AI_ENDPOINT: 'file:///tmp'}), /HTTP/);
    const resolved = settings(root, {});
    assert.match(resolved.cache, /classification-cache/);
    assert.equal(resolved.model, 'qwen3.5-9b-uncensored-hauhaucs-aggressive');
});

test('enrichment schema is required before installation', () => {
    assert.throws(() => checkEnrichmentSchema({tables: ['word_senses', 'sense_categories']}), /missing AI enrichment tables/);
    const {calls, options} = harness([0, 0]);
    options.validateBundle = () => ({report: {tables: ['words', 'word_senses', 'sense_categories']}});
    assert.throws(() => full(root, options), /missing AI enrichment tables/);
    assert.equal(calls.some(call => call.kind === 'install'), false);
});


test('live smoke task uses configured model and endpoint without cache or database writes', () => {
    const calls = [];
    smoke(root, {config, runner: (_command, args) => {
        calls.push(args);
        return {status: 0};
    }});
    assert.equal(calls.length, 1);
    assert.equal(calls[0][5], 'ai-smoke');
    assert.equal(calls[0][calls[0].indexOf('--model') + 1], config.model);
    assert.equal(calls[0][calls[0].indexOf('--endpoint') + 1], config.endpoint);
});

test('failed live smoke stops full workflow before classification or installation', () => {
    const {calls, options} = harness([3]);
    options.runner = (_command, args) => {
        calls.push({kind: 'cargo', args});
        if (args[5] === 'classify-plan') return {status: 0, stdout: 'Pending senses: 3\n'};
        if (args[5] === 'ai-smoke') return {status: 1};
        throw new Error(`Unexpected action ${args[5]}`);
    };
    assert.throws(() => full(root, options), /ai-smoke failed/);
    assert.deepEqual(calls.map(call => call.args?.[5]), ['classify-plan', 'ai-smoke']);
});
