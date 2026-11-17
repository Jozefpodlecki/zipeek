const resources = performance.getEntriesByType('resource');
const local = resources
    .map(resource => new URL(resource.name))
    .filter(url => url.origin === location.origin);
const dictionary = local.some(url => url.pathname === '/zipeek/hanzi.duckdb');
const vendor = local.some(url => url.pathname.startsWith('/zipeek/vendor/duckdb/'));
const paths = local.every(url => url.pathname.startsWith('/zipeek/'));
return { dictionary, vendor, paths, local: local.map(url => url.pathname) };
