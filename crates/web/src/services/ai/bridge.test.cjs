const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');

async function main() {
  const source = readFileSync(require('node:path').join(__dirname, 'bridge.js'), 'utf8');
  const { completeLocal } = await import(`data:text/javascript,${encodeURIComponent(source)}`);
  let observed;
  global.fetch = async (url, options) => {
    observed = { url: String(url), options };
    return {
      ok: true,
      json: async () => ({ choices: [{ message: { content: '{"chinese":"你好","target_words":["你好"]}' } }] }),
    };
  };
  const response = await completeLocal('http://127.0.0.1:1234/v1/', 'example', [{ role: 'user', content: 'hi' }]);
  assert.equal(JSON.parse(response).chinese, '你好');
  assert.equal(observed.url, 'http://127.0.0.1:1234/v1/chat/completions');
  assert.deepEqual(JSON.parse(observed.options.body).messages, [{ role: 'user', content: 'hi' }]);
  await assert.rejects(completeLocal('file:///tmp/', 'example', []), /HTTP or HTTPS/);
  global.fetch = async () => ({ ok: false, status: 503 });
  await assert.rejects(completeLocal('http://127.0.0.1:1234/v1/', 'example', []), /503/);
  console.log('AI bridge tests passed');
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
