# Technical debt register

Track implementation and verification liabilities here. `TASKS.md` is the active open-task triage index, and `LEGACY_ACCEPTANCE_LEDGER.md` preserves the unresolved historical checkpoint gates; `CURRENT.md` records active checkpoint state. An item is resolved only when its acceptance evidence is recorded; source-contract tests alone do not prove Rust compilation or browser behavior.

| ID | Priority | Area | Evidence and impact | Resolution / acceptance gate | State |
| --- | --- | --- | --- | --- | --- |
| TD-001 | P0 | Rust/WASM and browser acceptance | The user reports that the latest build fails (R522 intake; diagnostic not yet provided), and recent authored Rust/WASM/browser work has no verified full current-tree acceptance. | `cargo fmt --all -- --check`, affected native tests, locked WASM check and Chrome E2E all pass on pinned Windows toolchain; attach diagnostics. | Open |
| TD-002 | P0 (historical) | Retired Node source-contract suite | R360 reported 123 failures in the then-active Node suite. R392 removed all 69 `scripts/**/*.test.cjs` suites and their CI/Cargo Make gates. That historical test-run count no longer measures the current testing baseline. | Historical liability retired with the suite; retain Rust unit and Chrome E2E acceptance requirements in TD-001/TD-008. No claim is made that the old failures correspond to fixed application behavior. | Closed as obsolete test-suite debt in R429; runtime acceptance remains open |
| TD-003 | P1 | Grammar assessment authoring quality | 24 draft MCQs and five construction challenges require qualified Mandarin review; plausible, unambiguous distractors and coverage cannot be proven by automatic syntax checks. | Language review, item revision record, semantic rationale checks, validated item provenance and independent assessment policy before mastery eligibility. | Open |
| TD-004 | P1 | Grammar assessment model | In R360, authored content, validation, and mutable assessment sessions occupied one file. ID-only sessions could still be scored against altered content carrying identical IDs. | Bind scoring to complete immutable authored items, reject orthographically duplicate choices, verify wrong-answer/restart flows, and run native/WASM/Chrome checks. | **Further hardened in R362; native/WASM/Chrome verification open** |
| TD-005 | P1 | Pedagogical evidence and worker | Opt-in UI is not activated; `snapshot_verified` is caller-provided, and application-level verified DuckDB readback / single-tab ownership are absent. | Approved consent/provenance contract, transaction/readback confirmation, single-tab exclusion, restart and browser failure tests. | Open |
| TD-006 | P1 | Docs Markdown | Parser migration and safe Markdown link rendering are implemented; cross-document heading targets previously conflicted with the Yew hash-router syntax. | Verify Rust fixtures, locked WASM build, Chrome navigation (including reload/history) under root and `/zipeek/`, and scroll-to-heading behavior. | R429 adds route-query heading navigation; R431 modularizes Docs and adds cancel-safe document loading. Compilation and Chrome acceptance remain open |
| TD-007 | P1 | Courses mosaic and curriculum | Only 5/100 grammar regions authored; 50 vocabulary pockets unbound to verified proficiency; geometry visual verification and official HSK mappings pending. | Source-backed HSK mapping, content review, geometry screenshots and independently verified reveal predicates. | Open |
| TD-008 | P1 | E2E full journeys | Standard journey and pedagogical seven-day endurance journey have not been run successfully in managed Chrome; enabled variant depends on the unfinished evidence pipeline. | Run reproducible seeded journeys with screenshots, genuine scored interactions and explicit no-pedagogy/Dragon assertions. | Open |
| TD-009 | P2 | Generated static assets | Earlier ZIP hotfixes omitted Docs index or Courses artwork; build products may be stale or missing in archives. R371 adds an asset preflight with source/deploy modes; integration and Rust acceptance remain pending. | Native build asset preflight that checks generated index, bootstrap, WASM and artwork against manifest; ZIP integrity alone is insufficient. | Open |
| TD-010 | P2 (historical) | Duplicate dependency-version guidance | Earlier audit compared `yew_icons` 0.11.0 in manifests against an outdated hardcoded `AGENTS.md` 0.10 reference. R425 removed the duplicated version list; the manifests and lockfile now supply actual configured versions, with approved decisions remaining authoritative for changes. | No conflicting hardcoded agent-guide pin remains. No dependency was changed; cross-target build verification remains TD-001. | Closed as documentation drift in R429 |

## Per-checkpoint debt movement

- **R429:** TD-002 and TD-010 are closed solely as stale testing/documentation liabilities, not as proven application bug fixes. TD-006 gains cross-document heading navigation in source, but requires Rust/WASM and Chrome acceptance before closure.

- **R370:** Invalid or escaping Markdown links no longer produce clickable placeholder anchors; heading-only article links are intercepted before Yew hash routing. Added Rust fixtures, source guards and a Chrome Docs reader scenario. TD-006 remains open pending native/WASM/browser acceptance and cross-document fragment navigation.


- **R369:** Replaced handwritten Docs parser with pinned Rust pulldown-cmark, unified TOC anchors and event-level link safety. TD-006 is not closed until locked native/WASM and real-browser checks pass.


- **R368:** Course 5 extends authored coverage to 5/100 grammar regions, 24 draft MCQs and five construction items (TD-007/TD-003). Accessible mosaic region titles now use authored curriculum metadata, eliminating a duplicate maintenance list. All new Chinese assessment content and modal distinctions remain subject to independent review; Rust/WASM/Chrome gates (TD-001) are unverified.


- **R367:** Added a native-testable cross-format Courses authoring validator and negative fixtures; centralized guided-course region mapping and wrote an authoring reference. This reduces TD-007's *content consistency* risk and makes future course edits cheaper, but it does not close HSK syllabus verification, qualified Mandarin review (TD-003), or native/WASM/Chrome acceptance (TD-001). No assessment authority or reveal policy changed.


- **R366:** Added Course 4 with five new registry points, five draft MCQs, six guided word banks, and a separate construction item. TD-007 moves to 4/100 authored grammar regions; TD-003 increases to 19 draft MCQs requiring independent Mandarin review. Native/WASM and Chrome acceptance remain outstanding; all 150 mosaic covers remain concealed.


- **R365:** Assessment review now uses keyboard-operable jump controls, screen-reader announcements and actual authored choice counts. Fixed the related browser E2E tile selector. This reduces TD-004 UI/model debt but does not close TD-001/TD-003; neither native/wasm compilation nor assistive-technology testing is verified.


- **R364:** Added an unassisted sentence-construction format for the three authored grammar courses and pure Rust validation of tile banks, unique indices, item snapshots and locked submission. TD-003 remains **open**: only three draft construction prompts are authored, linguistic quality is unreviewed, and no authoritative provenance/retry/award policy exists. TD-001 Rust/WASM/Chrome acceptance also remains open.


- **R363:** Added the 14-item review inventory in `docs/testing-quality/acceptance/grammar-assessment-content-review.md` and a locked, per-grammar-point diagnostic report. TD-003 remains open pending qualified reviewer and approved assessment policy; TD-004 remains open pending native/WASM/Chrome acceptance. No durable grading or mosaic unlocks.


- **R362:** TD-004 now binds draft scoring to a copy of each complete authored assessment item (ID, point ID, prompt, choices, answer key, rationale), not only ID and ordering. Quality validation rejects punctuation/whitespace-only distractors; unit tests cover altered keys, choices, and rationales. The managed Chrome test now covers answer reset and an incorrect result. This is orthographic validation, **not** an approved pedagogical assessment or Mandarin linguistic review; TD-003 and native/WASM/Chrome acceptance remain open.


- **R361:** TD-004 was partially addressed in source by separating assessment sessions and content-quality rules from the 14 authored items. Scoring checks ordered question identity; duplicate options are checked after whitespace normalization. Native compilation and Mandarin expert review remain required. No new dependency, persistence or award semantics were introduced.

## Review procedure

When modifying a feature, inspect its relevant debt IDs and update source/tests together. Report newly discovered debt, addressed debt, and remaining acceptance evidence in `CURRENT.md` and release notes. Do not close a debt entry solely because a draft test was added.
