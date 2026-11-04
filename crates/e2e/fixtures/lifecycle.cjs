if (process.argv[2] === 'exit') {
    console.error('intentional startup failure');
    process.exit(17);
}
setInterval(() => {}, 1000);
if (process.argv[2] === 'http') {
    let requests = 0;
    require('node:http').createServer((request, response) => {
        const status = ++requests === 1 ? 404 : 200;
        console.log(`HTTP readiness ${status}`);
        response.writeHead(status);
        response.end('readiness fixture');
    }).listen(Number(process.argv[3]), '127.0.0.1');
}
if (process.argv[2] === 'tree') {
    require('node:child_process').spawn(process.execPath,
        ['-e', "require('node:net').createServer(() => {}).listen(Number(process.argv[1]), '127.0.0.1')", process.argv[3]],
        {stdio: 'inherit', windowsHide: true});
}
