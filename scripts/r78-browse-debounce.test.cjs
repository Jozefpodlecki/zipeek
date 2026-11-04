const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");

const form = fs.readFileSync(path.join(__dirname, "../crates/web/src/components/browse/search_form.rs"), "utf8");

test("Browse debounce effect returns unit and releases pending Timeout on cleanup", () => {
    assert.match(form, /use_effect_with\(dependencies,/);
    assert.match(form, /Timeout::new\(delay,/);
    assert.match(form, /move \|\| \{\s*drop\(timeout\);\s*\}/);
    assert.doesNotMatch(form, /move \|\| timeout\.cancel\(\)/);
});
