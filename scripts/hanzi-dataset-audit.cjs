'use strict';
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..', 'external', 'hanzi-writer-data', 'data');
const combined = JSON.parse(fs.readFileSync(path.join(root, 'all.json'), 'utf8'));
const files = fs.readdirSync(root).filter(file => file.endsWith('.json') && file !== 'all.json');
const names = new Set(files.map(file => file.slice(0, -5)));
const failures = [];
let optionalRadicals = 0;
for (const file of files) {
  const name = file.slice(0, -5);
  const entry = JSON.parse(fs.readFileSync(path.join(root, file), 'utf8'));
  if (!Array.isArray(entry.strokes) || !Array.isArray(entry.medians) || entry.strokes.length !== entry.medians.length) failures.push(`${file}: invalid stroke/median contract`);
  if (!Object.hasOwn(entry, 'radStrokes')) optionalRadicals++;
  if (!Object.hasOwn(combined, name) || JSON.stringify(entry) !== JSON.stringify(combined[name])) failures.push(`${file}: differs from all.json`);
}
for (const name of Object.keys(combined)) if (!names.has(name)) failures.push(`${name}: missing individual file`);
console.log(`Hanzi dataset: ${files.length} individual entries, ${Object.keys(combined).length} combined entries; ${optionalRadicals} without optional radStrokes; ${failures.length} contract failures`);
for (const failure of failures.slice(0, 20)) console.error(failure);
if (failures.length) process.exitCode = 1;
