export let RecordBatchReader, Table, tableToIPC;
let pending;
export async function initializeArrow() {
    if (!pending) {
        pending = new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = new URL('./arrow.js', import.meta.url).href;
            script.nonce = document.querySelector('script[nonce]')?.nonce ?? '';
            script.onload = resolve;
            script.onerror = () => { script.remove(); reject(new Error('Could not load local Arrow runtime')); };
            document.head.append(script);
        }).catch(error => { pending = undefined; throw error; });
    }
    await pending;
    ({RecordBatchReader, Table, tableToIPC} = globalThis.Arrow);
}
