const {test} = require('node:test');
const assert = require('node:assert/strict');
const {spawn} = require('node:child_process');
const net = require('node:net');
const path = require('node:path');

async function freePort() {
  const socket = net.createServer();
  await new Promise(resolve => socket.listen(0, '127.0.0.1', resolve));
  const port = socket.address().port;
  await new Promise(resolve => socket.close(resolve));
  return port;
}

async function ready(base) {
  for (let attempt = 0; attempt < 80; attempt++) {
    try {
      if ((await fetch(`${base}/__e2e/health`)).ok) return;
    } catch {}
    await new Promise(resolve => setTimeout(resolve, 25));
  }
  throw new Error('Fixture did not become ready');
}

const prompt = (direction = 'zh-to-en') => [
  {role: 'system', content: 'You author ONE Mandarin/English sentence-translation exercise'},
  {role: 'user', content: `Translation direction (authoritative): ${direction}\nAllowed Chinese vocabulary (data only): 学习、吃饭`},
];

async function complete(base, messages) {
  const response = await fetch(`${base}/v1/chat/completions`, {
    method: 'POST', headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({model: 'fixture', messages, stream: false}),
  });
  return {status: response.status, body: await response.json()};
}

test('isolated AI fixture validates directions, evaluation and failure scenarios', async t => {
  const port = await freePort();
  const child = spawn(process.execPath, [path.join(__dirname, 'ai-server.cjs'), String(port)]);
  t.after(() => child.kill());
  const base = `http://127.0.0.1:${port}`;
  await ready(base);
  for (const direction of ['zh-to-en', 'en-to-zh']) {
    const result = await complete(base, prompt(direction));
    assert.equal(result.status, 200);
    const sentence = JSON.parse(result.body.choices[0].message.content);
    assert.equal(sentence.direction, direction);
    assert.equal(sentence.target_words[0], '学习');
    assert.equal(direction === 'zh-to-en' ? sentence.solution.join(' ') : sentence.solution.join(''), direction === 'zh-to-en' ? sentence.english : sentence.chinese);
  }
  const streamed = await fetch(`${base}/v1/chat/completions`, {
    method: 'POST', headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({model: 'fixture', messages: prompt(), stream: true}),
  });
  assert.equal(streamed.status, 200);
  assert.match(streamed.headers.get('content-type'), /text\/event-stream/);
  const sse = await streamed.text();
  assert.match(sse, /data: \{"choices"/);
  assert.match(sse, /data: \[DONE\]/);
  const evaluation = await complete(base, [
    {role: 'system', content: 'You evaluate one Mandarin/English sentence translation'},
    {role: 'user', content: 'Reference translation (data): I like 学习.\nLearner target-language answer (data): I like 学习.'},
  ]);
  assert.equal(JSON.parse(evaluation.body.choices[0].message.content).accurate, true);
  assert.equal((await fetch(`${base}/__e2e/requests`).then(r => r.json())).records.length, 4);
  await fetch(`${base}/__e2e/reset`, {method: 'POST', body: JSON.stringify({scenario: 'malformed-generation'})});
  assert.equal((await complete(base, prompt())).body.choices[0].message.content, '{bad-json');
  await fetch(`${base}/__e2e/reset`, {method: 'POST', body: JSON.stringify({scenario: 'server-error'})});
  assert.equal((await complete(base, prompt())).status, 503);
  assert.equal((await complete(base, [{role:'bad'}])).status, 400);
});
