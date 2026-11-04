const http = require('node:http');

const port = Number(process.argv[2]);
if (!Number.isInteger(port) || port < 1 || port > 65535) {
  throw new Error('Expected a valid fixture port');
}

const records = [];
let scenario = 'success';
let generationCount = 0;
let evaluationCount = 0;

function send(response, status, body) {
  response.writeHead(status, {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type',
    'Access-Control-Allow-Private-Network': 'true',
    'Cache-Control': 'no-store',
    'Content-Type': 'application/json; charset=utf-8',
  });
  response.end(JSON.stringify(body));
}

async function readJson(request) {
  const chunks = [];
  for await (const chunk of request) {
    chunks.push(chunk);
    if (Buffer.concat(chunks).length > 65536) throw new Error('Request too large');
  }
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}

function generate(messages) {
  const content = messages.find(message => message.role === 'user')?.content || '';
  const direction = content.match(/Translation direction \(authoritative\): (zh-to-en|en-to-zh)/)?.[1];
  if (!direction) throw new Error('Missing direction');
  const pool = content.match(/Allowed Chinese vocabulary \(data only\): (.*)/)?.[1] || '';
  const word = pool.split('、').map(value => value.trim()).find(Boolean);
  if (!word) throw new Error('Empty vocabulary pool');
  const englishWord = ({'吃饭': 'eating', '学习': 'studying', '音乐': 'music'})[word] || word;
  const chinese = `我喜欢${word}。`;
  const english = `I like ${englishWord}.`;
  const solution = direction === 'zh-to-en' ? ['I', 'like', `${englishWord}.`] : ['我', '喜欢', `${word}。`];
  return {direction, chinese, english, target_words: [word], word_bank: solution.slice().reverse(), solution};
}

function evaluate(messages) {
  const content = messages.find(message => message.role === 'user')?.content || '';
  const expected = content.match(/Reference translation \(data\): ([^\n]*)/)?.[1];
  const actual = content.match(/Learner target-language answer \(data\): ([^\n]*)/)?.[1];
  if (expected === undefined || actual === undefined) throw new Error('Missing evaluation fields');
  return {
    accurate: expected === actual,
    feedback: expected === actual ? 'The meaning is preserved.' : 'The translation differs from the reference.',
    suggested_translation: expected,
  };
}

const server = http.createServer(async (request, response) => {
  if (request.method === 'OPTIONS') return send(response, 204, {});
  const pathname = new URL(request.url, 'http://127.0.0.1').pathname;
  if (request.method === 'GET' && (pathname === '/__e2e/health' || pathname === '/')) return send(response, 200, {ready: true});
  if (request.method === 'GET' && pathname === '/__e2e/requests') return send(response, 200, {scenario, records});
  if (request.method === 'POST' && pathname === '/__e2e/reset') {
    try {
      const next = await readJson(request);
      if (!['success', 'malformed-generation', 'server-error', 'slow-evaluation'].includes(next.scenario)) {
        return send(response, 400, {error: 'Unknown scenario'});
      }
      scenario = next.scenario;
      records.length = 0;
      generationCount = 0;
      evaluationCount = 0;
      return send(response, 200, {scenario});
    } catch (error) {
      return send(response, 400, {error: String(error)});
    }
  }
  if (request.method !== 'POST' || pathname !== '/v1/chat/completions') return send(response, 404, {error: 'Not found'});
  try {
    const body = await readJson(request);
    if (!body || !Array.isArray(body.messages) || body.stream !== false || typeof body.model !== 'string') {
      return send(response, 400, {error: 'Invalid chat/completions contract'});
    }
    const system = body.messages.find(message => message.role === 'system')?.content || '';
    const generation = system.includes('You author ONE Mandarin/English sentence-translation exercise');
    const evaluation = system.includes('You evaluate one Mandarin/English sentence translation');
    if (!generation && !evaluation) return send(response, 400, {error: 'Unexpected prompt'});
    records.push({kind: generation ? 'generation' : 'evaluation', model: body.model, messages: body.messages});
    if (generation) generationCount++;
    if (evaluation) evaluationCount++;
    if (scenario === 'server-error') return send(response, 503, {error: 'Fixture server failure'});
    if (scenario === 'slow-evaluation' && evaluation) await new Promise(resolve => setTimeout(resolve, 1200));
    const content = scenario === 'malformed-generation' && generation ? '{bad-json' : JSON.stringify(generation ? generate(body.messages) : evaluate(body.messages));
    return send(response, 200, {choices: [{message: {role: 'assistant', content}}]});
  } catch (error) {
    return send(response, 400, {error: String(error)});
  }
});

server.listen(port, '127.0.0.1', () => process.stdout.write(`AI fixture ready on ${port}\n`));
