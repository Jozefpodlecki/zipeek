const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');

const code = readFileSync(join(__dirname, '../crates/web/src/services/microphone.js'), 'utf8');
const moduleUrl = `data:text/javascript;base64,${Buffer.from(code).toString('base64')}`;

function setup() {
  const old = {};
  const globals = ['isSecureContext', 'navigator', 'MediaRecorder', 'AudioContext', 'requestAnimationFrame', 'cancelAnimationFrame'];
  for (const name of globals) old[name] = Object.getOwnPropertyDescriptor(globalThis, name);
  const define = (name, value) => Object.defineProperty(globalThis, name, { configurable: true, writable: true, value });
  const tracks = [{ stopped: false, stop() { this.stopped = true; } }];
  let resolvePermission;
  let manualPermission = false;
  const getUserMedia = async () => manualPermission
    ? new Promise((resolve) => { resolvePermission = resolve; })
    : { getTracks: () => tracks };
  define('isSecureContext', true);
  define('navigator', { mediaDevices: { getUserMedia } });
  class Recorder {
    state = 'inactive';
    mimeType = 'audio/webm';
    constructor() { Recorder.latest = this; }
    start() { this.state = 'recording'; }
    stop() {
      this.state = 'inactive';
      this.ondataavailable?.({ data: new Blob(['some audio'], { type: 'audio/webm' }) });
      this.onstop?.();
    }
  }
  define('MediaRecorder', Recorder);
  class Context {
    fftSize = 1024;
    createMediaStreamSource() { return { connect() {} }; }
    createAnalyser() { return { fftSize: 1024, getFloatTimeDomainData(samples) { samples.fill(0.25); } }; }
    async resume() {}
    async close() {}
  }
  define('AudioContext', Context);
  const frames = new Map();
  let frameIndex = 0;
  define('requestAnimationFrame', (callback) => { frames.set(++frameIndex, callback); return frameIndex; });
  define('cancelAnimationFrame', (id) => frames.delete(id));
  const originalCreate = URL.createObjectURL;
  const originalRevoke = URL.revokeObjectURL;
  const revoked = [];
  URL.createObjectURL = () => 'blob:recording';
  URL.revokeObjectURL = (url) => revoked.push(url);
  return {
    tracks, frames, revoked, Recorder,
    deferPermission() { manualPermission = true; },
    allowPermission() { resolvePermission({ getTracks: () => tracks }); },
    restore() {
      for (const name of globals) {
        if (old[name]) Object.defineProperty(globalThis, name, old[name]);
        else delete globalThis[name];
      }
      URL.createObjectURL = originalCreate;
      URL.revokeObjectURL = originalRevoke;
    },
  };
}

test('recording acquires microphone on click, renders levels, stops tracks and exposes playback', async () => {
  const env = setup();
  try {
    const { createMicrophoneRecorder } = await import(moduleUrl);
    const phases = [];
    const levels = [];
    const finished = [];
    const errors = [];
    const recorder = createMicrophoneRecorder((l) => levels.push(l), (p) => phases.push(p), (u) => finished.push(u), (e) => errors.push(e));
    assert.equal(env.tracks[0].stopped, false);
    await recorder.start();
    assert.equal(recorder.phase, 'recording');
    const frame = [...env.frames.values()][0];
    frame(100);
    assert.equal(levels.at(-1).length, 40);
    assert.ok(levels.at(-1).at(-1) > 0);
    recorder.stop();
    assert.equal(env.tracks[0].stopped, true);
    assert.equal(recorder.phase, 'ready');
    assert.deepEqual(finished, ['blob:recording']);
    assert.deepEqual(errors, []);
    recorder.discard();
    assert.deepEqual(env.revoked, ['blob:recording']);
    recorder.dispose();
    assert.ok(phases.includes('processing'));
  } finally { env.restore(); }
});

test('disposed recorder releases late permission grants without starting capture', async () => {
  const env = setup();
  try {
    env.deferPermission();
    const { createMicrophoneRecorder } = await import(moduleUrl);
    const phases = [];
    const recorder = createMicrophoneRecorder(() => {}, (p) => phases.push(p), () => {}, () => {});
    const pending = recorder.start();
    recorder.dispose();
    env.allowPermission();
    await pending;
    assert.equal(env.tracks[0].stopped, true);
    assert.ok(!phases.includes('recording'));
  } finally { env.restore(); }
});

test('permission denied reports a readable error and does not record', async () => {
  const env = setup();
  try {
    const { createMicrophoneRecorder } = await import(moduleUrl);
    Object.defineProperty(globalThis, 'navigator', {
      configurable: true,
      value: { mediaDevices: { async getUserMedia() { throw Object.assign(new Error('denied'), { name: 'NotAllowedError' }); } } },
    });
    const errors = [];
    const recorder = createMicrophoneRecorder(() => {}, () => {}, () => {}, (e) => errors.push(e));
    await recorder.start();
    assert.equal(recorder.phase, 'error');
    assert.match(errors[0], /denied/i);
    recorder.dispose();
  } finally { env.restore(); }
});

test('recording UI is a separate browser adapter with no speech scoring or cloud request', () => {
  assert.doesNotMatch(code, /\bfetch\s*\(|SpeechRecognition|localStorage|sessionStorage/);
  const rust = readFileSync(join(__dirname, '../crates/web/src/components/speaking_recorder.rs'), 'utf8');
  assert.match(rust, /data-component="speaking-recorder"/);
  assert.match(rust, /<AudioWaveform /);
  assert.match(rust, /data-action="stop-recording"/);
  assert.match(rust, /data-component="speaking-playback"/);
  assert.doesNotMatch(rust, /ExerciseResult|PronunciationEvidence|QuizSession/);
});

test('speaking page is reachable from Practice without dictionary initialization', () => {
  const routes = readFileSync(join(__dirname, '../crates/web/src/route.rs'), 'utf8');
  const setup = readFileSync(join(__dirname, '../crates/web/src/pages/practice_setup.rs'), 'utf8');
  assert.match(routes, /#?\[at\("\/practice\/speaking"\)\]/);
  assert.match(routes, /Route::Speaking => html! \{ <Speaking \/> \}/);
  assert.match(routes, /matches!\(routes, Route::NotFound \| Route::Speaking \| Route::Settings \| Route::About\)/);
  assert.match(setup, /data-speaking-setup=""/);
  assert.match(setup, /href="#\/practice\/speaking"/);
});
