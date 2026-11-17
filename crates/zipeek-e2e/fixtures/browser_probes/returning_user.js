try {
  const key = 'zipeek.preferences.v1';
  if (location.protocol.startsWith('http') && localStorage.getItem(key) === null) {
    localStorage.setItem(key, '{}');
  }
} catch (_) {
}
