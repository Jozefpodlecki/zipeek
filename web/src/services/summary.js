export async function createSummary(canvas, count, correct, total) {
    const module = await import(new URL('vendor/summary/summary.mjs', document.baseURI).href);
    return module.createSummary(canvas, count, correct, total);
}

export function destroySummary(handle) {handle.destroy();}
