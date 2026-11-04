export async function openDatabase() {
    const base = new URL('vendor/duckdb/', document.baseURI);
    const arrow = await import(new URL('arrow.mjs', base).href);
    await arrow.initializeArrow();
    const duckdb = await import(new URL('duckdb-browser.mjs', base).href);
    const bundle = await duckdb.selectBundle({
        mvp: {mainModule: new URL('duckdb-mvp.wasm', base).href, mainWorker: new URL('duckdb-browser-mvp.worker.js', base).href},
        eh: {mainModule: new URL('duckdb-eh.wasm', base).href, mainWorker: new URL('duckdb-browser-eh.worker.js', base).href}
    });
    const worker = new Worker(bundle.mainWorker);
    const db = new duckdb.AsyncDuckDB(new duckdb.VoidLogger(), worker);
    let connection;
    let timer;
    try {
        const initialize = async () => {
            await db.instantiate(bundle.mainModule);
            const manifestResponse = await fetch(new URL('hanzi.manifest.json', document.baseURI), {cache: 'no-store', signal: AbortSignal.timeout(30000)});
            if (!manifestResponse.ok) throw new Error(`Dictionary manifest download failed (HTTP ${manifestResponse.status})`);
            const manifest = await manifestResponse.json();
            const filenames = (manifest.files ?? []).map(file => file.path).filter(name => /^hanzi(?:-\d{8}-\d{6})?\.duckdb$/.test(name));
            if (filenames.length !== 1) throw new Error('Dictionary manifest must identify exactly one database file');
            const filename = filenames[0];
            const response = await fetch(new URL(filename, document.baseURI), {signal: AbortSignal.timeout(30000)});
            if (!response.ok) throw new Error(`Dictionary download failed (HTTP ${response.status})`);
            await db.registerFileBuffer(filename, new Uint8Array(await response.arrayBuffer()));
            await db.open({path: filename, accessMode: duckdb.DuckDBAccessMode.READ_ONLY});
            connection = await db.connect();
            const info = (await connection.query('SELECT version() AS engine, CAST(count(*) AS INTEGER) AS row_count FROM words')).toArray()[0].toJSON();
            return {db, connection, info};
        };
        return await Promise.race([initialize(), new Promise((_, reject) => {
            timer = setTimeout(() => reject(new Error('Dictionary initialization timed out; retry the download')), 45000);
        })]);
    } catch (error) {
        worker.terminate();
        throw error;
    } finally { clearTimeout(timer); }
}

export function databaseInfo(handle) { return handle.info; }

export async function queryDatabase(handle, sql, parameters) {
    const statement = await handle.connection.prepare(sql);
    try {
        const result = await statement.query(...parameters);
        return result.toArray().map(row => row.toJSON());
    } finally { await statement.close(); }
}

export async function closeDatabase(handle) {
    try { await handle.connection.close(); }
    finally { await handle.db.terminate(); }
}
