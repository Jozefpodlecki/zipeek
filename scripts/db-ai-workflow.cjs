const path = require('node:path');
const crypto = require('node:crypto');
const {spawnSync} = require('node:child_process');
const {generate, installBundle, checkBrowseSchema, validateBundle} = require('./db-workflow.cjs');

const repository = path.resolve(__dirname, '..');
const defaultModel = 'qwen3.5-9b-uncensored-hauhaucs-aggressive';

function positiveInteger(value, label, allowZero = false) {
    if (!/^(0|[1-9]\d*)$/.test(String(value))) throw new Error(`${label} must be an integer`);
    const number = Number(value);
    if (!Number.isSafeInteger(number) || (allowZero ? number < 0 : number < 1)) {
        throw new Error(`${label} must be a ${allowZero ? 'nonnegative' : 'positive'} safe integer`);
    }
    return number;
}

function settings(root, environment = process.env) {
    const model = environment.ZIPEEK_AI_MODEL ?? defaultModel;
    if (!model.trim()) throw new Error('ZIPEEK_AI_MODEL must not be empty');
    const endpoint = environment.ZIPEEK_AI_ENDPOINT ?? 'http://127.0.0.1:1234';
    const url = new URL(endpoint);
    if (!['http:', 'https:'].includes(url.protocol)) throw new Error('ZIPEEK_AI_ENDPOINT must use HTTP(S)');
    const timeout = positiveInteger(environment.ZIPEEK_AI_TIMEOUT_MS ?? '300000', 'ZIPEEK_AI_TIMEOUT_MS');
    const retries = positiveInteger(environment.ZIPEEK_AI_RETRIES ?? '1', 'ZIPEEK_AI_RETRIES', true);
    const modelHash = crypto.createHash('sha256').update(model).digest('hex').slice(0, 16);
    const cache = path.join(root, 'target', 'db-workflow', 'classification-cache', `${modelHash}.jsonl`);
    return {model, endpoint: url.href.replace(/\/$/, ''), timeout, retries, cache};
}

function pendingFromPlan(output) {
    const matches = [...output.matchAll(/^Pending senses:\s*(\d+)\s*$/gm)];
    if (matches.length !== 1) throw new Error('Could not determine pending classifications from db-gen plan output');
    return positiveInteger(matches[0][1], 'Pending senses', true);
}

function runCargo(root, args, runner = spawnSync, capture = false) {
    const result = runner('cargo', ['run', '-p', 'db-gen', '--locked', '--', ...args], {
        cwd: root,
        stdio: capture ? ['ignore', 'pipe', 'inherit'] : 'inherit',
        encoding: capture ? 'utf8' : undefined,
        maxBuffer: 8 * 1024 * 1024,
    });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`db-gen ${args[0]} failed (${result.status ?? result.signal}); installed bundle unchanged`);
    return capture ? result.stdout : '';
}

function checkEnrichmentSchema(report) {
    const tables = Array.isArray(report.tables) ? report.tables : [];
    const required = ['word_senses', 'sense_pos', 'sense_categories', 'classification_runs', 'parts_of_speech', 'categories'];
    const missing = required.filter(table => !tables.includes(table));
    if (missing.length) throw new Error(`Generated dictionary is missing AI enrichment tables: ${missing.join(', ')}`);
}

function full(root, options = {}) {
    const config = options.config ?? settings(root);
    const runner = options.runner ?? spawnSync;
    const planArgs = ['classify-plan', '--model', config.model, '--cache', config.cache];
    const initialPending = pendingFromPlan(runCargo(root, planArgs, runner, true));
    console.log(`Classification model: ${config.model}\nCache: ${config.cache}\nPending senses: ${initialPending}`);
    if (initialPending > 0) {
        runCargo(root, [
            'ai-smoke', '--model', config.model,
            '--endpoint', config.endpoint,
            '--timeout-ms', String(config.timeout),
            '--retries', String(config.retries),
        ], runner);
        runCargo(root, [
            'classify', '--model', config.model, '--cache', config.cache,
            '--endpoint', config.endpoint, '--timeout-ms', String(config.timeout),
            '--retries', String(config.retries), '--limit', String(initialPending),
        ], runner);
    }
    const remaining = pendingFromPlan(runCargo(root, planArgs, runner, true));
    if (remaining !== 0) throw new Error(`${remaining} senses remain unclassified; installed bundle unchanged`);
    const produce = options.generate ?? generate;
    const verify = options.validateBundle ?? validateBundle;
    const check = options.checkBrowseSchema ?? checkBrowseSchema;
    const install = options.installBundle ?? installBundle;
    const candidate = produce(root, {classificationCache: config.cache});
    const report = verify(root, candidate).report;
    check(report);
    checkEnrichmentSchema(report);
    const installed = install(root, candidate);
    console.log(`AI-enriched dictionary installed: ${installed.destination}`);
    return installed;
}

function smoke(root, options = {}) {
    const config = options.config ?? settings(root);
    const runner = options.runner ?? spawnSync;
    runCargo(root, [
        'ai-smoke', '--model', config.model,
        '--endpoint', config.endpoint,
        '--timeout-ms', String(config.timeout),
        '--retries', String(config.retries),
    ], runner);
    console.log(`LM Studio smoke passed for ${config.model}; no database changes.`);
}

if (require.main === module) {
    try {
        if (process.argv[2] === 'smoke') {
            smoke(repository);
        } else if (process.argv.length === 2) {
            full(repository);
        } else {
            throw new Error('Usage: db-ai-workflow.cjs [smoke]');
        }
    } catch (error) {
        console.error(`Full database workflow failed: ${error.message}`);
        process.exitCode = 1;
    }
}

module.exports = {full, smoke, settings, pendingFromPlan, checkEnrichmentSchema};
