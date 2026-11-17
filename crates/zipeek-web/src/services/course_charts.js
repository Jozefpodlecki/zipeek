export async function createCourseProgress(canvas, completed, total) {
    const module = await import(new URL('vendor/summary/summary.mjs', document.baseURI).href);
    return module.createCourseProgress(canvas, completed, total);
}

export function destroyCourseProgress(handle) {
    handle.destroy();
}
