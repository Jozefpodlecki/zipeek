const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const root = path.resolve(__dirname, '..');
const executableSuffix = process.platform === 'win32' ? '.exe' : '';
const servicePackages = ['zipeek-ai-server', 'zipeek-image-server', 'zipeek-tts-stt'];

function executable(profile, name) {
  return path.join(root, 'target', profile, `${name}${executableSuffix}`);
}

function run(program, args) {
  const result = spawnSync(program, args, { cwd: root, stdio: 'inherit', shell: false });
  if (result.error) {
    process.stderr.write(`${program}: ${result.error.message}\n`);
    process.exit(1);
  }
  if (result.signal) process.exit(1);
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function ensureBinary(profile, name) {
  if (fs.existsSync(executable(profile, name))) return;
  const args = ['build', '--locked'];
  if (profile === 'release') args.push('--release');
  args.push('-p', name);
  run('cargo', args);
  if (!fs.existsSync(executable(profile, name))) {
    process.stderr.write(`Build did not produce ${executable(profile, name)}\n`);
    process.exit(1);
  }
}

function main(args) {
  const profile = args[0] === '--release' ? 'release' : 'debug';
  const cliArgs = profile === 'release' ? args.slice(1) : args;
  if (!cliArgs.length) {
    process.stderr.write('Usage: node scripts/run-existing-cli.cjs [--release] <cli-command> [args...]\n');
    process.exit(2);
  }
  if (cliArgs[0] === 'desktop-sidecars') {
    for (const service of servicePackages) ensureBinary(profile, service);
  }
  ensureBinary(profile, 'zipeek-cli');
  run(executable(profile, 'zipeek-cli'), cliArgs);
}

if (require.main === module) main(process.argv.slice(2));

module.exports = { executable, servicePackages };
