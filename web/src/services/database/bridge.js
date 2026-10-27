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
            const response = await fetch(new URL('hanzi.duckdb', document.baseURI), {signal: AbortSignal.timeout(30000)});
            if (!response.ok) throw new Error(`Dictionary download failed (HTTP ${response.status})`);
            await db.registerFileBuffer('hanzi.duckdb', new Uint8Array(await response.arrayBuffer()));
            await db.open({path: 'hanzi.duckdb', accessMode: duckdb.DuckDBAccessMode.READ_ONLY});
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
