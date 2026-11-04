const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');

const [distArg, portArg, prefix = '', repoArg] = process.argv.slice(2);
const dist = path.resolve(distArg);
const repo = path.resolve(repoArg);
const types = {'.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.webp': 'image/webp', '.json': 'application/json'};

http.createServer((request, response) => {
    try {
        const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
        if (!pathname.startsWith(`${prefix}/`)) {
            response.writeHead(404).end();
            return;
        }
        const relative = pathname.slice(prefix.length + 1) || 'index.html';
        if (relative === '__e2e/adapter.html') {
            response.writeHead(200, {'Content-Type': 'text/html', 'Cache-Control': 'no-store'});
            response.end(`<html lang="en"><head><base href="${prefix}/"></head><body><h1>Database adapter fixture</h1></body></html>`);
            return;
        }
        const file = relative === '__e2e/bridge.mjs'
            ? path.join(repo, 'web/src/services/database/bridge.js')
            : path.resolve(dist, relative);
        if (relative !== '__e2e/bridge.mjs' && !file.startsWith(`${dist}${path.sep}`)) {
            response.writeHead(403).end();
            return;
        }
        fs.stat(file, (error, stat) => {
            if (error || !stat.isFile()) {
                response.writeHead(404).end();
                return;
            }
            response.writeHead(200, {'Content-Type': types[path.extname(file)] || 'application/octet-stream', 'Cache-Control': 'no-store'});
            fs.createReadStream(file).on('error', () => response.destroy()).pipe(response);
        });
    } catch (error) {
        console.error(error);
        response.writeHead(400).end();
    }
}).listen(Number(portArg), '127.0.0.1', () => console.log(`Serving ${dist} under ${prefix || '/'} on ${portArg}`));
