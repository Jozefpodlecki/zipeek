const SETTINGS_KEY = 'zipeek.preferences.v1';
const DEFAULT_URL = 'http://127.0.0.1:1234/v1/';

function readSettings() {
  const value = JSON.parse(localStorage.getItem(SETTINGS_KEY) || '{}');
  if (!value || Array.isArray(value) || typeof value !== 'object') throw new Error('Invalid settings document');
  return value;
}
let tabKey = '';

export function readAiConfiguration() {
  try {
    return {
      url: readSettings().ai_url || DEFAULT_URL,
      model: readSettings().ai_model || '',
      keyPresent: tabKey.length > 0,
    };
  } catch {
    return { url: DEFAULT_URL, model: '', keyPresent: tabKey.length > 0 };
  }
}

export function saveAiConfiguration(url, model, key) {
  const parsed = new URL(url);
  if (!['http:', 'https:'].includes(parsed.protocol)) throw new Error('Only HTTP(S) AI endpoints are supported');
  if (parsed.username || parsed.password) throw new Error('Do not place credentials in the endpoint URL');
  if (!model.trim()) throw new Error('A model identifier is required');
  try {
    const settings = readSettings();
    settings.ai_url = parsed.href;
    settings.ai_model = model.trim();
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch { throw new Error('Browser storage unavailable; settings could not be saved'); }
  if (key.trim()) tabKey = key.trim();
}

export function apiKeyForRequest() { return tabKey; }
export function clearAiKey() { tabKey = ''; }
