window.dictionaryDownloads = 0;
window.dictionaryWorkers = 0;
window.Worker = new Proxy(window.Worker, {
    construct(target, args) {
        window.dictionaryWorkers += 1;
        return Reflect.construct(target, args);
    }
});
const originalFetch = window.fetch.bind(window);
window.fetch = (input, options) => {
    if (String(input).endsWith('hanzi.duckdb')) {
        window.dictionaryDownloads += 1;
        return new Promise(resolve => {
            window.releaseDictionary = () => resolve(originalFetch(input, options));
        });
    }
    return originalFetch(input, options);
};
