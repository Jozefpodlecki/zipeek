const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const test = require('node:test');
const { executable, servicePackages } = require('../run-existing-cli.cjs');

test('service package list covers all three desktop sidecars', () => {
  assert.deepEqual(servicePackages, ['zipeek-ai-server', 'zipeek-image-server', 'zipeek-tts-stt']);
});

test('platform executable paths respect the build profile', () => {
  const suffix = process.platform === 'win32' ? '.exe' : '';
  assert.ok(executable('debug', 'zipeek-cli').endsWith(path.join('target', 'debug', `zipeek-cli${suffix}`)));
  assert.ok(executable('release', 'zipeek-cli').endsWith(path.join('target', 'release', `zipeek-cli${suffix}`)));
});

test('pack and unpack reuse a present CLI without calling Cargo', { skip: process.platform === 'win32' }, () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'zipeek-cli-bootstrap-'));
  try {
    fs.mkdirSync(path.join(root, 'scripts'));
    fs.mkdirSync(path.join(root, 'target', 'release'), { recursive: true });
    fs.copyFileSync(path.join(__dirname, '..', 'run-existing-cli.cjs'), path.join(root, 'scripts', 'run-existing-cli.cjs'));
    const cli = path.join(root, 'target', 'release', 'zipeek-cli');
    fs.writeFileSync(cli, '#!/bin/sh\nprintf "%s\\n" "$@"\n', { mode: 0o755 });
    for (const action of ['pack', 'unpack']) {
      const output = execFileSync(process.execPath, [path.join(root, 'scripts', 'run-existing-cli.cjs'), '--release', action], {
        encoding: 'utf8',
        env: { ...process.env, PATH: '/nonexistent' },
      });
      assert.equal(output.trim(), action);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});


test('desktop staging reuses present debug sidecars and CLI without calling Cargo', { skip: process.platform === 'win32' }, () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'zipeek-sidecar-bootstrap-'));
  try {
    fs.mkdirSync(path.join(root, 'scripts'));
    fs.mkdirSync(path.join(root, 'target', 'debug'), { recursive: true });
    fs.copyFileSync(path.join(__dirname, '..', 'run-existing-cli.cjs'), path.join(root, 'scripts', 'run-existing-cli.cjs'));
    for (const name of ['zipeek-cli', ...servicePackages]) {
      fs.writeFileSync(path.join(root, 'target', 'debug', name), '#!/bin/sh\nprintf "%s\\n" "$@"\n', { mode: 0o755 });
    }
    const output = execFileSync(process.execPath, [path.join(root, 'scripts', 'run-existing-cli.cjs'), 'desktop-sidecars'], {
      encoding: 'utf8',
      env: { ...process.env, PATH: '/nonexistent' },
    });
    assert.equal(output.trim(), 'desktop-sidecars');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
