"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const root = path.resolve(__dirname, "../..");
const makefile = fs.readFileSync(path.join(root, "Makefile.toml"), "utf8");

function task(name) {
  const lines = makefile.split(/\r?\n/);
  const start = lines.findIndex(line => line === `[tasks.${name}]`);
  assert.notEqual(start, -1, `Missing task ${name}`);
  let end = start + 1;
  while (end < lines.length && !lines[end].startsWith("[tasks.")) end += 1;
  return lines.slice(start + 1, end).join("\n");
}

function dependencies(name) {
  const match = /^dependencies\s*=\s*\[([^\]]*)\]/m.exec(task(name));
  return match ? Array.from(match[1].matchAll(/"([^"]+)"/g), found => found[1]) : [];
}

test("full installation runs the asset pipelines and final preflight in sequence", () => {
  assert.deepEqual(dependencies("install"), [
    "setup-all", "db-generate", "db-compress", "db-install",
    "geo-install", "pedagogy-worker", "preflight-deploy",
  ]);
});

test("geography preparation audits fetched data before installing JSON", () => {
  assert.deepEqual(dependencies("geo-install"), ["geo-build"]);
  assert.deepEqual(dependencies("geo-build"), ["geo-audit"]);
  assert.deepEqual(dependencies("geo-audit"), ["geo-fetch"]);
  for (const name of ["geo-fetch", "geo-audit", "geo-build"]) {
    assert.match(task(name), /"areacity"/);
  }
  assert.match(task("geo-build"), /"crates\/zipeek-web\/static\/province-quest"/);
});

test("routine serve does not regenerate dictionary or geography", () => {
  assert.deepEqual(dependencies("serve"), ["verify-assets"]);
  assert.ok(!makefile.includes('dependencies = ["install", "serve"]'));
});
