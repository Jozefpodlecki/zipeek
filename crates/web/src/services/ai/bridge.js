export async function completeLocal(url, model, messages) {
  const endpoint = new URL(url);
  if (!['http:', 'https:'].includes(endpoint.protocol)) {
    throw new Error('AI endpoint must use HTTP or HTTPS');
  }
  const response = await fetch(new URL('chat/completions', endpoint.href.endsWith('/') ? endpoint.href : `${endpoint.href}/`), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model, messages, temperature: 0.3, stream: false }),
  });
  if (!response.ok) {
    throw new Error(`Local AI request failed (HTTP ${response.status})`);
  }
  const result = await response.json();
  const content = result?.choices?.[0]?.message?.content;
  if (typeof content !== 'string' || !content.trim()) {
    throw new Error('Local AI returned no text response');
  }
  return content;
}
