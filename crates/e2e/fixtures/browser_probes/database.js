const done = arguments[arguments.length - 1];
        (async () => {
            const bridge = await import(new URL('__e2e/bridge.mjs', document.baseURI));
            const handle = await bridge.openDatabase();
            try {
                const rows = await bridge.queryDatabase(handle, 'SELECT * FROM words ORDER BY id ASC', []);
                const selected = await bridge.queryDatabase(handle, 'SELECT id FROM words WHERE id = ?', [17]);
                let readOnly = false;
                try { await bridge.queryDatabase(handle, 'DELETE FROM words WHERE id=1', []); }
                catch (error) { readOnly = /read.only/i.test(error.message); }
                done({ok: rows.length === 12100 && Object.keys(rows[0]).length === 10 && rows.every((row, index) => row.id === index + 1) && selected[0].id === 17 && readOnly});
            } finally { await bridge.closeDatabase(handle); }
        })().catch(error => done({ok: false, error: String(error)}));
