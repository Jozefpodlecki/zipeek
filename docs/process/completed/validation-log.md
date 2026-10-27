# Validation log

Per-task validation narratives moved out of the live [CURRENT](../CURRENT.md)
state. Each section records what a specific run showed, not a current gate
claim. A failing gate recorded here is not repaired by a later passing run in a
different task.

Preserved verbatim from the former single-file `CURRENT.md`. Do not rewrite.

---

## A18 validation

- Formatting: failed on the baseline core/e2e/page files (repaired in A20/A26).
- Clippy: failed on Timeout and components::* imports (repaired in A20/A26).
- Wasm check: passed with those two warnings.
- Pages build: passed with those two warnings; Tailwind 4.3.3 completed.
- `cargo test --workspace`: failed in `landing_page_loads` after 121.16s:
  `WebDriverError` status 500, `tab crashed` (Chrome 156.0.8078.47). Five
  unit-test targets had zero tests; the one integration smoke failed. No browser
  assertion pass is claimed. A Trunk launch attempt failed because port 1420 was
  occupied; the existing server responded HTTP 200 and was left running.
- Documentation checks passed: 16 Markdown documents, 86 local links/anchors,
  balanced fences, 21 unique open tasks in prerequisite order, and 13 UX TBDs.
- Scope checks passed: BASELINE unchanged; prior COMPLETED prefix preserved;
  only approved package-name/Home code changes; user practice.rs edit preserved;
  dependencies, workflows, app imports, and replay contract unchanged.
- `git diff --check`: passed.

## A23 validation

- Passed: 16 Markdown documents, 88 local links/anchors, balanced fences,
  20 unique open tasks in prerequisite order, and `git diff --check`.
- The existing v1 canonical hash remains `0xa0e531d76df62844`.
- Installed rand 0.10.3 source confirms slice IndexedRandom::sample and
  SliceRandom::shuffle API calls; no dependencies or application code changed.
- Prior COMPLETED history and BASELINE are preserved. No version URL parser
  was added. Rust/Trunk/Chrome gates were not rerun for documentation-only work;
  the A18 results above remain previous-task evidence.

## A24 validation

- Passed: 16 Markdown documents, 89 local links/anchors, balanced fences,
  parseable proposed dependency TOML, 20 unique open tasks in prerequisite order,
  preserved code/manifests/lockfiles/prior history, and `git diff --check`.
- Python hashlib and PowerShell Get-FileHash independently verified all three
  documented source checksums. Fallback gzip/UTF-8 validation and its 125,320-row
  count passed. Exact crate versions were non-yanked when checked; native crate
  archive checksums matched registry metadata. Inputs exist only in audit temp
  storage, not committed generator inputs or application assets.
- No dependencies were installed, no database was created/opened, and no native
  to WASM runtime compatibility test was run. Rust/Trunk/Chrome gates were not
  rerun for this documentation-only audit; earlier failures remain recorded.

## Earlier A25 bundled-build validation (superseded by neo restoration)

- Input/notice hashes, provenance manifest, attributes, scope and documentation
  checks are being finalized; sources are preserved in the working tree.
- `cargo fmt --all -- --check`: fails on existing core/e2e/page formatting;
  the changed db-gen main is formatted.
- `cargo clippy --workspace --all-targets -- -D warnings`: fails on native
  compilation and the existing two app.rs imports.
- `cargo test --workspace` and focused db-gen tests: native compilation fails;
  no tests/engine query pass is claimed and Chrome smoke was not reached.
- Wasm check: passed with two existing import warnings.
- Pages build with /zipeek/: passed with the same warnings and Tailwind 4.3.3.
- No generated dictionary artifact or native-to-WASM test exists. Root lockfile
  resolution changed locally through Cargo; tracking policy remains A19 work.

## Earlier neo restoration and requested compatibility check

- Native cargo check passes. Neo SELECT version() reports v2.0.0.
- Actual WASM 1.32.0 in Node reports engine v1.4.3. Its read-only open rejects
  the default native file (storage version 999, reader supports 64..67).
- An independent file written with STORAGE_VERSION v1.4.0 passes: eight full
  rows, ten columns and both HSK indexes match, including Unicode and NULLs;
  DELETE is rejected in read-only mode. Scratch tools/files/results are under
  target/a25-compat. This is a synthetic fixture, not the generated dictionary.
- Focused native tests: rejection test passes; required-version integration
  test fails because v2.0.0 differs from the unchanged v1.4.3 requirement.
  CLI prints a clear mismatch and exits 1. No successful generation is claimed.
- Formatting and Clippy retain baseline failures. Wasm check and Pages build
  pass with the two existing app.rs import warnings. Workspace tests remain
  blocked by the native required-version test; no Chrome pass is claimed.
- Documentation/scope checks passed: 17 Markdown files, 93 local links/anchors,
  balanced fences, 20 open tasks in prerequisite order, preserved source/notice
  bytes and prior history, actual/documented neo manifest agreement, and
  git diff --check.
- Browser worker/HTTP/full-dictionary integration remains A06 coverage. No
  client/npm app dependency or WASM pin was changed by the isolated probe.

## Final A25 / D23 validation

- Passed: three native generator tests and manual invocation, logging engine
  v2.0.0 and storage compatibility v1.4.0. Wrong-version rejection is covered.
- Passed: wasm check, /zipeek/ Pages build, 17 Markdown files, 93 local links/
  anchors, five source/notice hashes, metadata consistency, and git diff --check.
- Existing failures remain: workspace fmt; Clippy app.rs imports; Chrome smoke
  status 500/tab crashed on Chrome 156.0.8078.47 after 121.26s. The three
  generator tests passed within the workspace run before the browser failure.
- Native-to-WASM synthetic fixture compatibility passed with explicit storage;
  production dictionary and browser/HTTP integration remain future A06 work.

## A26 / A20 validation

- Passed: cargo fmt --all -- --check; cargo clippy --workspace --all-targets
  --offline -- -D warnings; cargo test --workspace --offline (19 generator tests
  and Chrome landing smoke); cargo check --target wasm32-unknown-unknown
  -p zipeek --offline; trunk build --public-url /zipeek/ from web/.
- Chrome passed outside the process sandbox with a temporary root-path Trunk
  server after HTTP readiness; the owned temporary server was stopped afterward.
  This does not implement the A15 harness lifecycle.
- Manual CLI preparation verified two sources, three notices and 19,567,208
  bytes, plus actual engine v2.0.0 and storage compatibility v1.4.0.
- No new dependency, schema, replay or source-byte changes. Earlier validation
  sections remain historical evidence; current formatting/Clippy/Chrome pass.

## A02 validation and next-session handoff

- Passed: fmt, workspace Clippy -D warnings, workspace tests (29 domain/generator
  tests plus Chrome smoke), wasm check for both zipeek and zipeek-core, and Trunk
  build --public-url /zipeek/ from web/. Logs: target/a02-workspace-tests.log.
- Chrome used the existing HTTP-200 port-1420 server outside the process sandbox;
  a new Trunk launch could not bind. Server ownership was not confirmed and no
  existing process was killed. The earlier A26 cleanup assertion was too strong;
  its corrective note is appended in COMPLETED. A15 remains open.
- No new dependencies, source snapshot modifications, schema-policy changes,
  generated dictionary, replay changes, staging or commits.

## A03 validation

- Passed: fmt, workspace Clippy --all-targets --offline -- -D warnings, workspace
  tests (39 domain/generator tests and Chrome landing smoke), wasm check for
  zipeek/zipeek-core, and Trunk build --public-url /zipeek/ from web/.
- Tests compare repeated real-snapshot rows and diagnostics, assert SQL ID order,
  and test rejection/conflict/rollback/NULL/Unicode cases. An independent Python
  audit matched 12,100 accepted keys, 64 rejections and no missing English.
- Manual cargo run -p db-gen --offline succeeds, logs actual v2.0.0/v1.4.0 and
  counts, and writes no artifact. Logs: target/a03-import.log and
  target/a03-workspace-tests.log. An initial concurrent cargo run hit Windows
  executable locking; the serialized run passed.
- Chrome ran outside the process sandbox after a hidden owned Trunk server
  returned HTTP 200. The exact owned PID was stopped; process absence checked.
- No dependency, schema-contract, source-byte, replay, staging or commit changes.

## A04 validation

- Passed: fmt, workspace Clippy --all-targets --offline -- -D warnings, workspace
  tests (49 domain/generator tests plus Chrome smoke), wasm check for both
  zipeek and zipeek-core, and Trunk build --public-url /zipeek/ from web/.
- Seven fallback tests plus three shared normalizer tests pass. Pinned CC-CEDICT
  parses 125,320 entries; the actual primary output stays 12,100 rows with 514
  diagnostics because all primary glosses are present. The synthetic verified
  pipeline fills five, leaves two NULL and skips two variants before SQL insertion.
- Owned hidden Trunk served HTTP 200 before Chrome ran outside the sandbox.
  Its PID was stopped and absence confirmed after termination. Workspace log:
  target/a04-workspace-tests.log. No dependency/schema/replay/source-byte changes,
  artifact file, staging or commits. Input notices and hashes remain intact.

## A05 validation

- Passed: fmt, workspace Clippy --all-targets --offline -- -D warnings, workspace
  tests (55 domain/generator tests and Chrome smoke), wasm check for both zipeek
  and zipeek-core, and Trunk build --public-url /zipeek/ from web/.
- Generated web/static/hanzi.duckdb: 12,100 rows, 514 diagnostics, 3,158,016 bytes.
  SHA-256: 491fae6f84a96b5825c860163701f3e138cf72e5b50763848fde0e96ba03c546.
- Independent target/a05-repeat export has identical complete logical content,
  IDs, schema/indexes and diagnostics; a different physical checksum is allowed.
  Logical SHA-256: bf2865b52fd3e4343979cf2952cb42c01314042df489129fd22287d8d4c38f25.
- Pages output contains byte-identical database, metadata, diagnostics and notices
  at its root. Development GET /hanzi.duckdb returned the exact expected hash.
  Chrome used the ready owned hidden Trunk server outside the sandbox; its PID
  was stopped and absence confirmed. Logs: target/a05-generation.log,
  target/a05-repeat.log, target/a05-workspace-tests.log and target/a05-trunk.*.
- Native read-only DELETE fails without changing rows. Six artifact tests cover
  independent runs, all checksums, empty output, quoting, refusal/cleanup and CLI
  errors. No dependency/schema/source-byte/replay changes or commits/staging.

## A27 / A14 validation

- Passed: fmt, workspace Clippy --all-targets --offline -- -D warnings, workspace
  tests (55 domain/generator tests and two Chrome tests), wasm check for zipeek,
  and Trunk build --public-url /zipeek/ from web/.
- Chrome verifies default/saved/invalid themes, keyboard switch, storage-write
  errors, footer, three tile destinations, active links, brand Home and skip focus
  without hash changes. Typed CDP covers 360/768/1024/1280px with no overflow.
- Desktop dark/light and full mobile-light screenshots were captured/inspected:
  target/a27-desktop-dark.png, target/a27-desktop-light.png,
  target/a27-mobile-light.png. Logs: target/a27-final-workspace-tests.log and
  target/a27-responsive-final.log. The owned hidden ready Trunk server was stopped
  and process absence confirmed. A15 still owns automated harness lifecycle.
- No dependencies, source/artifact bytes, schema, replay, staging or commits
  changed. D24/UX, product, architecture and test docs record the implemented scope.

## A28 validation

- Passed: fmt, workspace Clippy --all-targets --offline -- -D warnings, workspace
  tests (55 domain/generator tests plus three Chrome tests), wasm check and
  Trunk build --public-url /zipeek/ from web/.
- Chrome verifies theme top-right position, removed decorative copy, actual
  document dimensions at 1024x768/1280x720/1366x768, keyboard collapse, hidden
  drawer, full-width content, re-expansion and mobile navigation. Long mobile
  landing content still scrolls, rather than being clipped or hidden.
- Screenshots: target/a28-desktop-dark.png, target/a28-desktop-light.png,
  target/a28-mobile-light.png, target/a28-collapsed.png. Desktop/collapsed views
  inspected. Log: target/a28-workspace-tests.log. The owned hidden ready Trunk
  server was stopped and process absence confirmed after validation.
- No dependency, schema, dictionary/artifact-byte or replay changes, staging or
  commit. Prior completed outcomes and BASELINE remain intact.

## A30 validation

- Passed: fmt, workspace Clippy, workspace tests (55 domain/generator tests and
  four Chrome tests), wasm check and /zipeek/ Trunk build. Log/captures: target/a30-*.
- Chrome blocks the real wasm resource to inspect actual bootstrap centering,
  clipping, no overflow, decoded background and saved dark/light themes; checks
  1280x800 and 360x800 plus reduced motion. Live app retains its backdrop during
  theme changes, and the route Loader shares accessible Loading status.
- Original image and embedded bytes match SHA-256
  64fc1cb7d2b34ff89ff03f5cef4bcfe3df64eae9f27c6ca073f72e8d2e201f93.
  Image files are untouched. Dark/light loading and app screenshots inspected.
- No dependency, schema, dictionary/artifact, replay, staging or commit changes.
  Owned hidden ready Trunk was stopped and process absence confirmed.