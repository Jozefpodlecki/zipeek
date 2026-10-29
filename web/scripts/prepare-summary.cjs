const fs = require('node:fs');
const path = require('node:path');
const esbuild = require('esbuild');
const root = path.resolve(__dirname, '..');
const chartRoot = path.dirname(path.dirname(require.resolve('chart.js')));
const esbuildRoot = path.dirname(path.dirname(require.resolve('esbuild')));
const colorRoot = path.dirname(path.dirname(require.resolve('@kurkle/color')));
for (const [source, version] of [[chartRoot, '4.6.1'], [esbuildRoot, '0.28.2']]) {
    if (JSON.parse(fs.readFileSync(path.join(source, 'package.json'), 'utf8')).version !== version) throw new Error(`Required summary dependency version ${version} missing`);
}
const output = path.join(root, 'vendor', 'summary');
fs.mkdirSync(output, {recursive: true});
const result = esbuild.buildSync({
    entryPoints: [path.join(__dirname, 'summary.mjs')],
    outfile: path.join(output, 'summary.mjs'),
    bundle: true,
    format: 'esm',
    platform: 'browser',
    target: 'es2022',
    treeShaking: true,
    minify: true,
    legalComments: 'none',
    metafile: true
});
fs.writeFileSync(path.join(output, 'bundle-meta.json'), JSON.stringify(result.metafile));
for (const [source, name] of [[chartRoot, 'CHARTJS'], [colorRoot, 'COLOR'], [esbuildRoot, 'ESBUILD']]) {
    fs.copyFileSync(path.join(source, 'LICENSE.md'), path.join(output, `${name}-LICENSE.txt`));
}
