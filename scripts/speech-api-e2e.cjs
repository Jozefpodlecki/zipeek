'use strict';

const assert = require('node:assert/strict');
const { spawn, spawnSync } = require('node:child_process');
const fs = require('node:fs');
const net = require('node:net');
const path = require('node:path');
const { setTimeout: delay } = require('node:timers/promises');

const root = path.resolve(__dirname, '..');
const args = new Set(process.argv.slice(2));
const strictTts = args.has('--require-tts');
const strictStt = args.has('--require-stt');
const skipBuild = args.has('--no-build');
const validArgs = new Set(['--require-tts', '--require-stt', '--no-build']);
const referenceWav = path.join(root, 'crates', 'zipeek-tts-stt', 'tests', 'fixtures', 'mandarin_nihao_shijie.wav');
const referenceText = '你好世界';

function voicedWav() {
    const sampleRate = 16000;
    const samples = sampleRate / 2;
    const pcmLength = samples * 2;
    const wav = Buffer.alloc(44 + pcmLength);
    wav.write('RIFF', 0);
    wav.writeUInt32LE(wav.length - 8, 4);
    wav.write('WAVEfmt ', 8);
    wav.writeUInt32LE(16, 16);
    wav.writeUInt16LE(1, 20);
    wav.writeUInt16LE(1, 22);
    wav.writeUInt32LE(sampleRate, 24);
    wav.writeUInt32LE(sampleRate * 2, 28);
    wav.writeUInt16LE(2, 32);
    wav.writeUInt16LE(16, 34);
    wav.write('data', 36);
    wav.writeUInt32LE(pcmLength, 40);
    for (let index = 0; index < samples; index += 1) {
        wav.writeInt16LE(Math.round(6000 * Math.sin(2 * Math.PI * 440 * index / sampleRate)), 44 + index * 2);
    }
    return wav;
}

function assertWav(bytes) {
    assert.ok(bytes.length >= 46, 'TTS did not return a WAV with sample data');
    assert.equal(bytes.toString('ascii', 0, 4), 'RIFF');
    assert.equal(bytes.readUInt32LE(4), bytes.length - 8, 'invalid RIFF size');
    assert.equal(bytes.toString('ascii', 8, 12), 'WAVE');
    assert.equal(bytes.toString('ascii', 12, 16), 'fmt ');
    assert.equal(bytes.readUInt32LE(16), 16, 'unexpected WAV format chunk');
    assert.equal(bytes.readUInt16LE(20), 1, 'TTS must produce PCM audio');
    assert.equal(bytes.readUInt16LE(22), 1, 'TTS must produce mono audio');
    const sampleRate = bytes.readUInt32LE(24);
    assert.ok(sampleRate >= 8000 && sampleRate <= 96000, 'TTS WAV sample rate is invalid');
    assert.equal(bytes.readUInt32LE(28), sampleRate * 2, 'invalid byte rate');
    assert.equal(bytes.readUInt16LE(32), 2, 'invalid block alignment');
    assert.equal(bytes.readUInt16LE(34), 16, 'expected 16-bit PCM');
    assert.equal(bytes.toString('ascii', 36, 40), 'data');
    assert.equal(bytes.readUInt32LE(40), bytes.length - 44, 'invalid data size');
    assert.ok(bytes.length > 44, 'TTS WAV must contain audio data');
    let peak = 0;
    for (let offset = 44; offset < bytes.length; offset += 2) {
        peak = Math.max(peak, Math.abs(bytes.readInt16LE(offset)));
    }
    assert.ok(peak > 100, 'TTS WAV contains only silence or near-silence');
    return { sampleRate, samples: (bytes.length - 44) / 2, peak };
}

async function availablePort() {
    const server = net.createServer();
    await new Promise((resolve, reject) => {
        server.once('error', reject);
        server.listen(0, '127.0.0.1', resolve);
    });
    const port = server.address().port;
    await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
    return port;
}

async function request(base, route, options = {}) {
    const response = await fetch(`${base}${route}`, { signal: AbortSignal.timeout(12000), ...options });
    return { response, bytes: Buffer.from(await response.arrayBuffer()) };
}

async function waitForHealth(base, child, logs) {
    for (let attempt = 0; attempt < 100; attempt += 1) {
        if (child.exitCode !== null || child.signalCode !== null) {
            throw new Error(`speech server exited before readiness: ${logs.join('').slice(-4000)}`);
        }
        try {
            const result = await request(base, '/health');
            if (result.response.status === 200 && result.bytes.toString() === 'ok') return;
        } catch {}
        await delay(100);
    }
    throw new Error(`speech server did not become ready: ${logs.join('').slice(-4000)}`);
}

async function main() {
    for (const arg of args) assert.ok(validArgs.has(arg), `unknown argument: ${arg}`);
    if (!skipBuild) {
        console.log('Building zipeek-tts-stt with the locked dependency graph...');
        const build = spawnSync('cargo', ['build', '--locked', '-p', 'zipeek-tts-stt'], {
            cwd: root, stdio: 'inherit', shell: process.platform === 'win32'
        });
        if (build.error) throw build.error;
        assert.equal(build.status, 0, 'cargo build failed');
    }
    const executable = path.join(root, 'target', 'debug', process.platform === 'win32' ? 'zipeek-tts-stt.exe' : 'zipeek-tts-stt');
    assert.ok(fs.existsSync(executable), `missing speech server executable: ${executable}`);
    const port = await availablePort();
    const base = `http://127.0.0.1:${port}`;
    const logs = [];
    const child = spawn(executable, [], {
        cwd: root,
        env: { ...process.env, ZIPEEK_SPEECH_PORT: String(port) },
        stdio: ['ignore', 'pipe', 'pipe'],
        windowsHide: true
    });
    child.stdout.on('data', data => logs.push(data.toString()));
    child.stderr.on('data', data => logs.push(data.toString()));
    child.on('error', error => logs.push(String(error)));
    try {
        await waitForHealth(base, child, logs);
        console.log('PASS: server started and GET /health returned 200');
        const capability = await request(base, '/v1/capabilities');
        assert.equal(capability.response.status, 200);
        const fields = Object.fromEntries(capability.bytes.toString().trim().split('\n').map(line => line.split('=')));
        assert.equal(fields.provider, 'local');
        assert.equal(fields.language, 'zh-CN');
        assert.ok(['true', 'false'].includes(fields.synthesis));
        assert.ok(['true', 'false'].includes(fields.recognition));
        console.log(`PASS: capabilities synthesis=${fields.synthesis}, recognition=${fields.recognition}`);

        const badTts = await request(base, '/v1/tts', {
            method: 'POST', headers: { 'Content-Type': 'text/plain; charset=utf-8' }, body: ' '
        });
        assert.equal(badTts.response.status, 400);
        const badStt = await request(base, '/v1/stt', {
            method: 'POST', headers: { 'Content-Type': 'audio/wav' }, body: Buffer.from('invalid WAV')
        });
        assert.equal(badStt.response.status, 400);
        console.log('PASS: both APIs reject malformed payloads');

        const forbidden = await request(base, '/v1/capabilities', { headers: { Origin: 'https://example.org' } });
        assert.equal(forbidden.response.status, 400);
        const preflight = await request(base, '/v1/tts', {
            method: 'OPTIONS', headers: {
                Origin: 'http://localhost:1420',
                'Access-Control-Request-Method': 'POST',
                'Access-Control-Request-Headers': 'content-type'
            }
        });
        assert.equal(preflight.response.status, 204);
        assert.equal(preflight.response.headers.get('access-control-allow-origin'), 'http://localhost:1420');
        console.log('PASS: remote browser origin rejected; approved local preflight allowed');

        const tts = await request(base, '/v1/tts', {
            method: 'POST', headers: { 'Content-Type': 'text/plain; charset=utf-8' }, body: '你好，世界。'
        });
        const audio = voicedWav();
        if (fields.synthesis === 'true') {
            assert.equal(tts.response.status, 200, `TTS reported available but failed: ${tts.bytes.toString('utf8').slice(0, 400)}`);
            assert.equal(tts.response.headers.get('content-type'), 'audio/wav');
            const wavDetails = assertWav(tts.bytes);
            console.log(`PASS: eSpeak TTS generated non-silent WAV (${tts.bytes.length} bytes, ${wavDetails.sampleRate} Hz, peak=${wavDetails.peak})`);
        } else {
            assert.equal(tts.response.status, 503, 'unavailable TTS must return HTTP 503');
            console.log('PASS: unavailable TTS correctly returned 503 (no eSpeak resources)');
            assert.ok(!strictTts, 'TTS is required but eSpeak resources are unavailable');
        }

        if (strictStt) {
            assert.ok(fs.existsSync(referenceWav), 'strict STT needs the checked-in Mandarin WAV fixture');
        }
        const sttAudio = referenceWav ? fs.readFileSync(path.resolve(referenceWav)) : audio;
        const stt = await request(base, '/v1/stt', {
            method: 'POST', headers: { 'Content-Type': 'audio/wav' }, body: sttAudio,
            signal: AbortSignal.timeout(240000)
        });
        if (fields.recognition === 'true') {
            assert.equal(stt.response.status, 200, `STT reported available but failed: ${stt.bytes.toString('utf8').slice(0, 400)}`);
            assert.equal(stt.response.headers.get('content-type'), 'text/plain; charset=utf-8');
            const transcript = stt.bytes.toString('utf8').trim();
            assert.ok(transcript.length > 0, 'STT returned an empty transcript');
            if (referenceText) {
                assert.equal(transcript.replace(/[\s，。！？、,.!?]/g, ''), referenceText.replace(/[\s，。！？、,.!?]/g, ''), 'STT transcript differs from the reference');
            }
            console.log(`PASS: STT returned a nonempty transcript: ${transcript}`);
        } else {
            assert.equal(stt.response.status, 503, `unavailable STT must return 503, got ${stt.response.status}: ${stt.bytes.toString('utf8').slice(0, 400)}`);
            assert.match(stt.bytes.toString('utf8'), /not implemented|unavailable/i);
            console.log('PASS: unavailable STT correctly returned 503 (checkpoint absent)');
            assert.ok(!strictStt, 'real STT transcription is required but recognition is unavailable');
        }
        console.log(`RESULT: HTTP E2E passed; actual speech readiness: TTS=${fields.synthesis}, STT=${fields.recognition}`);
    } finally {
        if (child.exitCode === null && child.signalCode === null) {
            const exited = new Promise(resolve => child.once('exit', resolve));
            child.kill();
            await Promise.race([exited, delay(3000)]);
            if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
        }
    }
}

if (require.main === module) {
    main().catch(error => {
        console.error(`FAIL: ${error.stack || error}`);
        process.exitCode = 1;
    });
}

module.exports = { voicedWav, assertWav };
