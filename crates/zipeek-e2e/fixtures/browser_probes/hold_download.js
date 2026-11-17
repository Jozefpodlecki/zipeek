const originalFetch = window.fetch.bind(window);
window.fetch = (input, options) => {
    const url = input instanceof Request ? input.url : String(input);
    if (/hanzi(?:-\d{8}-\d{6})?\.duckdb(?:\.gz)?(?:\?.*)?$/.test(url)) {
        return new Promise(resolve => {
            window.releaseDictionary = () => resolve(originalFetch(input, options));
        });
    }
    return originalFetch(input, options);
};
