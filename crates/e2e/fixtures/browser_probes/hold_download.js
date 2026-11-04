const originalFetch = window.fetch.bind(window);
window.fetch = (input, options) => {
    if (String(input).endsWith('hanzi.duckdb')) {
        return new Promise(resolve => {
            window.releaseDictionary = () => resolve(originalFetch(input, options));
        });
    }
    return originalFetch(input, options);
};
