const [endpoint, model, direction, source, ...rawTiles] = process.argv.slice(2);
if (!endpoint || !model || !direction || !source || !rawTiles.length || rawTiles.length % 2 !== 0) {
  process.stderr.write('Expected endpoint, model, direction, source and visible tile labels\n');
  process.exit(2);
}

async function main() {
  const tiles = [];
  for (let index = 0; index < rawTiles.length; index += 2) {
    tiles.push({index: Number(rawTiles[index]), text: rawTiles[index + 1]});
  }
  if (!Array.isArray(tiles) || !tiles.length || tiles.some(tile => !Number.isInteger(tile.index) || typeof tile.text !== 'string')) {
    throw new Error('Invalid visible tiles');
  }
  const input = {
    source,
    target_language: direction,
    available_tiles: tiles,
  };
  const url = new URL('chat/completions', endpoint.endsWith('/') ? endpoint : `${endpoint}/`);
  if (!['http:', 'https:'].includes(url.protocol)) throw new Error('Invalid local model endpoint');
  const response = await fetch(url, {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({
      model,
      stream: false,
      temperature: 0,
      messages: [
        {role: 'system', content: 'You are a Chinese language learner completing a sentence translation task. You may use ONLY the visible prompt and visible word-bank tiles. Respond with exactly one JSON object {"indices":[integer,...]} ordering all supplied tile indices. Do not invent indices or use hidden information.'},
        {role: 'user', content: JSON.stringify(input)},
      ],
    }),
    signal: AbortSignal.timeout(180000),
  });
  if (!response.ok) throw new Error(`Learner model returned HTTP ${response.status}`);
  const data = await response.json();
  const answer = data?.choices?.[0]?.message?.content;
  if (typeof answer !== 'string') throw new Error('Learner returned no answer');
  const trimmed = answer.trim().replace(/^```(?:json)?\s*/, '').replace(/\s*```$/, '');
  const parsed = JSON.parse(trimmed);
  const indices = parsed.indices;
  if (!Array.isArray(indices) || indices.length !== tiles.length || indices.some(value => !Number.isInteger(value))) {
    throw new Error('Learner action must use all visible tiles exactly once');
  }
  const available = tiles.map(tile => tile.index);
  if (new Set(indices).size !== indices.length || indices.some(value => !available.includes(value))) {
    throw new Error('Learner referenced absent or duplicate tile');
  }
  process.stdout.write(indices.join(','));
}

main().catch(error => {
  process.stderr.write(`${error.message}\n`);
  process.exitCode = 1;
});
