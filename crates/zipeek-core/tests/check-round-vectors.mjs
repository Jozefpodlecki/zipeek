import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const bytes = readFileSync(resolve(root, 'target/wasm32-unknown-unknown/debug/examples/replay_vectors.wasm'));
const module = new WebAssembly.Module(bytes);
const imports = {};
for (const entry of WebAssembly.Module.imports(module)) {
    assert.equal(entry.kind, 'function');
    (imports[entry.module] ??= {})[entry.name] = () => {
        throw new Error(`Pure replay vectors called host import ${entry.module}.${entry.name}`);
    };
}
const instance = new WebAssembly.Instance(module, imports);
const actual = BigInt.asUintN(64, instance.exports.round_golden_digest());
const fixture = readFileSync(new URL('./fixtures/round-golden.txt', import.meta.url), 'utf8').replaceAll('\r\n', '\n');
let expected = 0xcbf29ce484222325n;
for (const byte of Buffer.from(fixture)) {
    expected = BigInt.asUintN(64, (expected ^ BigInt(byte)) * 0x100000001b3n);
}
assert.equal(actual, expected);
console.log(`wasm32 replay vectors match the native fixture: ${actual.toString(16).padStart(16, '0')}`);
