export async function copyHanzi(text) {
  if (typeof text !== "string" || !text.trim()) throw new Error("No Chinese text to copy");
  if (!globalThis.navigator?.clipboard?.writeText) throw new Error("Clipboard is unavailable. Use a secure browser context (HTTPS or localhost).");
  await globalThis.navigator.clipboard.writeText(text);
}
