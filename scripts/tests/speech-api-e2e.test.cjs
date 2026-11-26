'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { voicedWav, assertWav } = require('../speech-api-e2e.cjs');

test('speech E2E input is a valid 16 kHz mono PCM16 WAV with signal', () => {
    const wav = voicedWav();
    assertWav(wav);
    assert.equal(wav.readUInt32LE(24), 16000);
    assert.equal(wav.readUInt32LE(40), 16000);
    assert.equal(wav.length, 16044);
    assert.ok(wav.readInt16LE(44 + 100 * 2) !== 0);
});

test('speech E2E WAV guard rejects non-WAV or empty output', () => {
    assert.throws(() => assertWav(Buffer.alloc(44)), /sample data/);
    const wav = voicedWav();
    assert.throws(() => assertWav(wav.subarray(0, 44)), /sample data/);
    const silent = Buffer.from(wav);
    silent.fill(0, 44);
    assert.throws(() => assertWav(silent), /silence/);
    const mismatchedSize = Buffer.from(wav);
    mismatchedSize.writeUInt32LE(0, 4);
    assert.throws(() => assertWav(mismatchedSize), /RIFF size/);
});

test('checked-in Mandarin fixture contains non-silent 16 kHz speech bytes', () => {
    const fs = require('node:fs');
    const path = require('node:path');
    const fixture = path.join(__dirname, '../../crates/zipeek-tts-stt/tests/fixtures/mandarin_nihao_shijie.wav');
    const wav = fs.readFileSync(fixture);
    assertWav(wav);
    assert.equal(wav.readUInt32LE(24), 16000);
});
