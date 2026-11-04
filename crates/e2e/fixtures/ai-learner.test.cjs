const {test} = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');
const {spawn} = require('node:child_process');
const path = require('node:path');

function runLearner(endpoint, tiles) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [
      path.join(__dirname, 'ai-learner.cjs'), endpoint, 'test-model', 'en', '我喜欢吃饭。', ...tiles,
    ]);
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', chunk => { stdout += chunk; });
    child.stderr.on('data', chunk => { stderr += chunk; });
    child.on('error', reject);
    child.on('close', code => resolve({code, stdout, stderr}));
  });
}

test('learner sends only rendered question and visible tile IDs and rejects cheating actions', async t => {
  let proposed = {indices: [2, 0, 1]};
  let modelRequest;
  const server = http.createServer(async (request, response) => {
    let body = '';
    for await (const chunk of request) body += chunk;
    modelRequest = JSON.parse(body);
    response.writeHead(200, {'Content-Type': 'application/json'});
    response.end(JSON.stringify({choices: [{message: {content: JSON.stringify(proposed)}}]}));
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => server.close(resolve)));
  const endpoint = `http://127.0.0.1:${server.address().port}/v1/`;
  const tiles = ['0', 'like', '1', 'eating.', '2', 'I'];
  const accepted = await runLearner(endpoint, tiles);
  assert.equal(accepted.code, 0, accepted.stderr);
  assert.equal(accepted.stdout, '2,0,1');
  assert.equal(modelRequest.model, 'test-model');
  const visible = JSON.parse(modelRequest.messages[1].content);
  assert.deepEqual(visible, {
    source: '我喜欢吃饭。',
    target_language: 'en',
    available_tiles: [{index: 0, text: 'like'}, {index: 1, text: 'eating.'}, {index: 2, text: 'I'}],
  });
  assert.equal('solution' in visible, false);
  assert.equal('reference' in visible, false);
  proposed = {indices: [2, 2, 1]};
  const invalid = await runLearner(endpoint, tiles);
  assert.notEqual(invalid.code, 0);
  assert.match(invalid.stderr, /duplicate tile/);
});
