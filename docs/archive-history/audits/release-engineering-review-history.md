# Release engineering review history (R310-R609)

This is historical source-review material consolidated from standalone R-numbered files. It records source claims and test status at the time, **not current product authority or present build verification**. Use the [documentation index](../../README.md) and topic owners for current contracts.

### r592-ten-file-remediation - R592 selected ten-file technical-debt remediation

Baseline: R591 WIP full-source content audit. Scope: the ten targets approved by the user, plus one shared Statistics request hook. The results below are source-level findings, not compiler or browser confirmation.

| Target file | Confirmed source observation | Applied correction | Remaining verification |
| --- | --- | --- | --- |
| `stats/grammar/page.rs` | Duplicate async bulk-selection generation, active, query and loading/error state | Uses a shared, typed `use_bulk_selection` hook with request ticket, query equality and unmount guard; action remains specific to Grammar | WASM compilation, stale-response integration test |
| `stats/explorer/page.rs` | Same duplicated request lifecycle | Uses the same hook for Vocabulary and preserves `SelectMatchingWords` transition | WASM compilation, query-switch browser test |
| `stats/state/mod.rs` | Empty identifiers could enter selected sets; empty review could be opened through direct action | Filters blank bulk IDs and prevents opening empty selection review; focused reducer test authored | Rust unit tests, full action invariant audit |
| `practice/local_session.rs` | Async exercise generation could commit state after component unmount | Adds mounted guard to generation continuation and bounds displayed completion percentage; unit test extended | Browser interruption/resume tests, persistence fault injection |
| `system/settings/mod.rs` | UI font state showed saved selection after failed persistence | Returns without updating local selected value on preference write failure | Browser storage failure test; broader workspace decomposition still open |
| `services/personal_data/backup.rs` | Excluded word list accepted duplicate IDs | Rejects duplicates with size-bound validation and adds test | Rust tests and backup round trip |
| `services/personal_data/storage.rs` | Pretty-printed export could exceed import's 2 MB limit | Enforces shared import limit on export, with isolated boundary test | Browser storage quota/recovery E2E |
| `zipeek-cli/src/archive/unpack.rs` | ZIP paths differing only by case could collide on Windows | Rejects casefolded duplicate paths before writing; adds regression fixture | Windows Rust extraction/rollback suite |
| `stats/overview/lifecycle.rs` | Chart effect watched data buckets but not appearance theme | Watches appearance preset as well, reinstantiating chart on theme change | Browser theme-switch & cleanup test |
| `zipeek-exercises/src/cloud_match.rs` | Rejected candidate with a colliding label could reserve an otherwise usable ID | Defers ID insertion until both labels have passed eligibility; adds deterministic test | Rust engine suite and replay verification |

Supporting module: `stats/bulk_selection.rs`, a reusable typed request lifecycle. It owns request tickets, cancellation, query identity, loading/error presentation and mounted guard. Reducer state remains the selection authority. No dependencies or generated data files changed.

### Evidence and limitations

- 16 offline Node tests passed.
- Hanzi dataset: 9,574 glyph entries consistent.
- Maintained documentation validator: zero errors, three frozen-history warnings.
- Each source change was inspected, but no Cargo/rustfmt/Clippy/locked WASM or browser test could execute because the corresponding toolchain is unavailable.
- Tests in this checkpoint are authored, **not executed**. A future compilation pass may uncover type errors; the archive is not an accepted release.
- Broader semantic audit findings from R591 remain open. This checkpoint covers only the ten approved source targets, not the entire workspace.

### Windows validation gate

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-exercises -p zipeek-cli -p zipeek-web
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo make e2e-full
```

Browser acceptance should include Grammar/Vocabulary query switching during bulk-selection requests, component unmount during generation, theme changes in Overview, failed localStorage writes and imported backup round trips.

---

### r593-ten-file-remediation - R593 ten-file correctness and trust-boundary remediation

Baseline: R592 WIP. Status: source candidate, not compiler-verified. This batch targets ten additional source files distinct from R592. No dependency, stored-data schema or external service contract was changed intentionally.

| Primary file | Source finding | Correction | Verification still required |
| --- | --- | --- | --- |
| `crates/zipeek-web/src/pages/system/stats/query.rs` | A malformed internal `QueryPage` with a zero limit panicked in `page_count`, and range arithmetic could overflow. | Guard page-count denominator, saturate display-range arithmetic and reject invalid page contracts. Added edge tests. | Rust/WASM test and query-source integration |
| `crates/zipeek-web/src/services/planned_handoff.rs` | Planned ticket instance values were accepted with leading/trailing whitespace or arbitrary length. | Reject whitespace-trim mismatches and IDs longer than 256 bytes; add malformed-ticket tests. | Planned handoff browser flow |
| `crates/zipeek-web/src/services/practice_launch.rs` | Session creation checked Sentence Transformation options even when that exercise was not selected. | Validate those options only for sessions containing the relevant exercise configuration. | Native/WASM tests and Practice launch |
| `crates/zipeek-web/src/services/word_sets.rs` | Saved Word Set labels could include embedded control characters; edit and import checks were inconsistent with an explicit text-field contract. | Centralize name validation and reject control characters in create, rename and stored records. Added tests. | Word Set import/export and UI |
| `crates/zipeek-web/src/services/personal_data/opfs.rs` | OPFS snapshot writing did not validate backup content, and snapshot reads loaded full files before checking the backup size limit. | Validate content on save, inspect file size before snapshot text reads, retain existing probe-file behavior. | Real-browser OPFS fault injection |
| `crates/zipeek-web/src/services/personal_data/files.rs` | Failed registration of the second picker event listener left the first one attached; receiver failure skipped cleanup. | Roll back partial listener setup and perform listener cleanup before propagating receiver errors. | Browser cancel and failure behavior |
| `crates/zipeek-web/src/services/practice_generated.rs` | Generated picture URL validation allowed backslashes and an empty authority host before a port. | Reject ambiguous backslashes and hostless authority forms, with regression cases. | Real browser URL normalization and image loading |
| `crates/zipeek-web/src/services/practice_session_store/persistence.rs` | If browser storage changed while an inventory was enumerated, a missing listed key/value was silently skipped and might permit an incorrect new-session decision. | Fail closed with `OutOfDate` on a missing enumerated entry. | Cross-tab mutation tests |
| `crates/zipeek-web/src/services/practice_session_store/validation.rs` | Validation did not reject a session completion earlier than start, or step finish earlier than session creation. | Enforce chronological bounds for stored session records. Added focused regression test in the session-test module. | Locked Rust tests and real timestamp transitions |
| `crates/zipeek-web/src/services/exclusions.rs` | Unbounded local-storage exclusions JSON was parsed without a raw-size guard. | Reject values larger than 2 MiB before parsing, matching the personal-backup boundary. Added limit test. | Browser storage failure and large-data test |

Additional regression-only file: `crates/zipeek-web/src/services/practice_session_store/tests.rs`.

### Evidence and limits

- `node --test scripts/tests/*.test.cjs`: 16 tests passed.
- `node scripts/hanzi-dataset-audit.cjs`: 9,574 records consistent.
- `node scripts/docs-validate.cjs`: must pass after this document is added and the documentation index is regenerated.
- Rust compilation, rustfmt, Clippy, WASM and browser acceptance were not run; Cargo is not available in this environment. Authored Rust tests are unexecuted. These changes may need compiler corrections.
- All previous R591 and R592 open acceptance work remains open. This batch does not certify a complete codebase-wide semantic audit or production readiness.

### Required acceptance

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo make e2e-full
```

Browser scenarios: incomplete file-picker registration, cancellation and OPFS size rejection; Word Set import invalid labels; session inventory mutation in another tab; Practice launch without Sentence Transformation; generated image URL rejection; History and catalogue paging.

---

### r594-ten-file-remediation - R594 ten-file source review and local speech client

Baseline: R593 WIP source archive. The ten approved primary targets were inspected in full and received changes. This document separates observed defects, defensive work and unresolved tests. The speech client is additional approved scope.

| Source file | Source finding and correction | Confidence and gate |
| --- | --- | --- |
| `crates/zipeek-ai-client/src/client.rs` | Non-SSE JSON completion used unbounded `response.json()`; now checks a 4 MiB response limit before parsing. | Source-level size control; native/WASM integration needed |
| `crates/zipeek-ai-client/src/stream.rs` | A large transport chunk containing many *small* SSE events was rejected before parsing, while multiple short `data:` lines in one event escaped the aggregate event bound. Parse complete lines incrementally and bound total event data. | Focused Rust tests authored, not run |
| `crates/zipeek-web/src/pages/practice/generated_session.rs` | Clearing a translation answer did not invalidate its generation ticket or clear prior evaluation. Reset now invalidates outstanding evaluation state. | Browser async cancellation test needed |
| `crates/zipeek-web/src/services/practice_session_store/transitions.rs` | Revision increments could overflow in debug or wrap in release. Saturating increments prevent unchecked arithmetic (long-term revision rollover policy remains open). | Unit testing needed |
| `crates/zipeek-web/src/services/image_generation/client.rs` | Server job error bodies were shown without bounds or control-character filtering. Errors now have a 300-character display bound. | Browser job-response injection needed |
| `crates/zipeek-exercises/src/planned_session.rs` | Instance validation occurred after cloning and mutating the candidate plan. It is now evaluated before cloning. Coupled plan/ledger publication remains atomic. Mistake counts remain independently meaningful and are not equated with missed points. | Existing atomicity tests need run |
| `crates/zipeek-web/src/pages/learning/games/cloud_match/game/session.rs` | Failed restart could retain an old playable game, and returning to setup preserved a stale setup error. Reset the old game and feedback on failure, clear error on setup return. | Reducer regression test authored |
| `crates/zipeek-web/src/pages/system/settings/ai_state.rs` | A diagnostic result arriving after editing or saving configuration could incorrectly reinstate feedback. Ignore completion events when no test is active. A new test beginning before a prior test finishes still requires generation IDs end-to-end. | Reducer regression test authored; cross-generation race open |
| `crates/zipeek-web/src/components/audio/speaking_recorder/state.rs` | External level arrays could include NaN, infinity or out-of-range amplitudes. Bound array length and clamp finite values; failures explicitly set an error phase. | Reducer regression test authored; browser acceptance open |
| `crates/zipeek-docs/src/lib.rs` | Safe path validation rejected NUL but admitted other control characters. Reject all control characters, with tests. Existing crate-wide lint allowances remain to be reviewed separately. | Tests authored, not run |

### Speech API

`zipeek-tts-stt-client` adds a native loopback-only typed API without external dependencies. The crate includes unit tests for HTTP framing, capability parsing, invalid inputs, and a mock TCP listener. The existing speech server and browser bridge remain separate; service audio/model runtime correctness is not claimed verified.

### Verification and unresolved risks

The 16 existing offline Node tests, Hanzi dataset consistency audit, and maintained documentation validator can execute in the authoring environment. Rustfmt, Cargo, Clippy, WASM and Chrome are not installed. The R594 ZIP is a **source-only candidate**, not a compiler-verified release. Rust tests authored during this batch are not counted as passing.

The R591 outstanding semantic risk register, R592/R593 compiler gates, production AI streaming size/timeout policies, speech-service model licensing, dedicated desktop bridge permission semantics, async-generation cancellation, and full browser acceptance remain open. No new external dependency, personal storage schema change, or hosted service endpoint was introduced.

---

### r595-speech-client-protocol - R595 native speech client protocol review

Baseline: R594 WIP native client source. This focused review covers the no-dependency loopback client, not the whole speech engine, browser recorder or model acceptance.

### Confirmed findings and source corrections

- The client previously accepted successful HTTP payloads without checking media type. It now requires an endpoint-specific Content-Type and rejects duplicate or missing type headers.
- TTS responses and STT uploads previously checked only WAV magic bytes and minimum size. They now validate RIFF length, PCM16 format, mono channels, byte rate, sample alignment, nonempty data chunks and chunk framing. STT additionally requires 16 kHz and at most 30 seconds of samples. The server independently validates STT WAV.
- A chunked HTTP frame formerly evaluated `length + 2` without checked arithmetic. It now rejects overflow.
- Returned HTTP service error text is bounded and has control characters removed before it reaches client displays.

### Ownership and limits

`SpeechClient` remains a synchronous native interface over 127.0.0.1. No new network hosts, dependencies, provider state, browser privileges, persisted recordings or models were introduced. The existing WASM browser bridge continues to own browser permissions and capture.

### Verification evidence and open work

Protocol and WAV regression tests were authored in the client crate. Rust and end-to-end audio tests still require the locked toolchain and licensed model resources. Source-level review does not establish Rust compilation. Exercise failure handling, native/desktop integration and cross-platform lifecycle testing remain open; this checkpoint does not mark them complete.

---

### r597-header-dragon-implementation - R597 header dragon implementation and evidence boundary

The former header icon was conditional on `learning_history_enabled` and visually suggested proficiency merely because Practice session recording was enabled. Recording consent is not assessment. In R597, the header artwork is present regardless of that preference and its semantic state is explicitly `unavailable`.

### Current presentation

`components/shell/app_shell/header/dragon.rs` renders one unassessed illustration and an ordinary keyboard-operable details/summary disclosure. The source asset is `static/practice/header-dragon-unassessed.webp`. There is no invented percentage, fill, stage, evidence counter or mastery score. The artwork is an unverified candidate against the approved visual reference and may require further work for desktop fidelity.

`header.rs` no longer displays a centered route title. Document titles remain set by `AppShell`; contextual page headings are no longer suppressed by global layout CSS. Header route and dragon E2E expectations were updated accordingly.

### Outstanding blocking integration

`zipeek-pedagogy` projects `RecognitionEvent` into verified acquisition history from trustworthy events. The current personal Practice history JSON is a session-outcome record, not eligible character-level recognition attempts. The worker is diagnostic/read-only; no approved durable event journal, identity mapping, validated-choice gate, single-tab writer, timezone policy or target HSK denominator is wired to the shell. The only truthful live progress state is therefore unavailable, not 0%.

Before a genuine dragon fill is introduced, accept the evidence schema and provenance rules, persist and re-read verified records atomically with consent, define target/denominator and failure semantics, and browser-test the artwork layers for exact registration, focus, theme, responsiveness and reduced motion.

### R598 worker state correction

`zipeek-pedagogy::WorkerCoordinator` now clears cached projections when disabled or upon failed journal analysis, and `latest_projection()` refuses to expose a projection during a disabled/error state. Previously successful diagnostic timestamps remain historical only, not proof of fresh assessed progress. This protects future consumers but does not turn a diagnostic worker into an authoritative store. Tests are authored but not compiled in this environment.

---

### r602-session-selection-debt-review - R602 session and selection invariant review

### Reviewed scope

R602 continues the R601 ownership pass with narrow correctness changes. The reviewed files are `crates/zipeek-exercises/src/session/{mod,model}.rs` and the existing exercise-session tests, together with `crates/zipeek-web/src/pages/system/stats/state/{mod,transitions,tests}.rs`. Existing APIs and persisted formats remain unchanged.

### Confirmed findings and corrections

| Finding | Before | Correction | Validation status |
| --- | --- | --- | --- |
| Session terminal transition | Calling `QuizSession::complete` after successful completion returned `Ok(())` unlike other terminal operations | `complete` now rejects repeated completion with `AlreadyCompleted` before reading exercise entries | Regression test authored; Rust tests unrun |
| Exercise session root responsibility | `session/mod.rs` combined domain re-exports, 137 lines of session transitions, and 260+ lines of tests | Moved transitions into `session/transitions.rs`, tests into `session/tests.rs`; `mod.rs` contains declarations and exports | Module paths inspected; Rust compilation unrun |
| Blank Statistics identifiers | Single-item toggles, visible-page selection, and Grammar detail activation could accept empty or whitespace-only IDs | Reject blank IDs at reducer action boundaries; filter invalid visible-page entries before testing selection state | Focused reducer regression test authored; Rust tests unrun |

### Explicitly open technical debt

- Practice local and generated controllers retain asynchronous orchestration, resource cancellation and substantial UI composition. The existing unmount guard does not by itself prove route-change race freedom.
- Statistics still stores selection IDs as `String`; canonical selection transfer and typed domain identities are separate, unresolved contracts.
- AI Settings retains its asynchronous connection-test lifecycle in the component; cancellation requires browser verification.
- Browser storage and DuckDB transaction coordination, exercise generator determinism, and native speech runtime need their outstanding acceptance work.
- This checkpoint is not evidence of a successful Rust build. Run the locked Rust/WASM and browser gates before accepting it.

### Validation

- Executed offline Node regression suites: 16 passed.
- Hanzi Writer consistency audit: 9,574 records, zero contract failures.
- Source and module paths inspected, complete archive CRC checked.
- Cargo, rustfmt, Clippy, Rust unit tests and browser tests unavailable here.

---

### r603-practice-route-session-invariant-review - R603 Practice route and exercise-session invariants

### Source findings

1. `LocalPracticeSession` tracked unmount but not the current route session identity. A pending asynchronous generation task could finish after the same Yew component received a different `id` prop, then publish an old session snapshot to the new route. A route-ID reference now rejects that completion before storage reload or UI dispatch. This does not cancel the external generation work and requires real browser navigation/race testing.
2. `ExerciseInstance` has public fields. Its constructor validates an instance ID and definition, but callers could construct invalid instances directly and pass them into `QuizSession::push` or `record_completed`. Both insertion paths now validate the value before mutation. Regression tests cover whitespace identifiers and unknown definitions with unchanged session snapshots.

### Verification evidence and exclusions

- Source changes and tests: authored. Full crate compilation, tests and WASM/browser acceptance are not verified here because Rust tooling is unavailable.
- No storage schema, dependencies, generated assets, scoring semantics or URL formats changed.
- Remaining technical debt: route-level session restore races, cancelled external generation, concurrent storage edits, canonical Statistics IDs, AI diagnostics and DuckDB transaction recovery. Avoid treating this checkpoint as a complete workflow verification.

### Windows checks

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-exercises
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo make e2e-full
```

---

### r604-practice-local-session-refactor - R604 Practice local-session ownership review

### Scope

Reviewed the R603 Practice local-session controller and the existing PracticeSessionStore persistence contract. Extracted route composition, restoration, generation reconciliation, revision-aware progress commands, history finalization, and exercise presentation from the controller without introducing a second domain-state store.

### Code ownership

- `local_session/mod.rs`: exports `LocalPracticeSession` and route-keyed child page. The key resets local hooks and invalidates mounted async work when the route session ID changes.
- `page.rs`: Yew integration, local UI callbacks, async generation start and page layout.
- `restore.rs`: loads and recovers stored sessions, begins a ready step, writes each revision using existing store guarantees.
- `generation.rs`: reconciles generated output against the latest stored session, generation attempt, and existing stale-response rules; reloads on revision conflict.
- `persistence.rs`: handles retry, completion, and endless-session finalization; reloads authoritative saved state if a revision conflict occurs.
- `history.rs`: owns history finalization effect, stale task guard and retry input.
- `exercise.rs`: renders the appropriate exercise from a validated Practice step and typed callbacks. It does not touch storage.
- `progress.rs`, `summary.rs`, `tests.rs`: progress calculations, completion view and existing progress tests. Fixed invalid visibility qualifier on `preview_progress` parameter.

### Behavior and limitations

- PracticeSession remains the authoritative domain model. No duplicate reducer is introduced.
- Stored revision checks remain in PracticeSessionStore; this change does not make cross-tab localStorage compare-and-swap atomic.
- Generated requests remain in flight after navigation; result publication is guarded by the unmounted and active-session checks and the attempt/revision reconciliation.
- History writes retain the prior finalization service and its idempotency contract; stale UI results are suppressed.
- The API and saved record shape remain unchanged.
- R604 does not claim comprehensive cancellation of external AI/network tasks or atomic cross-tab persistence. Those require separate service-level design and verification.

### Validation

- `node --test scripts/tests/*.test.cjs`: 16 passed, 0 failed, using the complete source's scripts and root Makefile.toml.
- Inspected module exports, dependency paths, and ZIP replacement byte-for-byte integrity.
- Rustfmt, Cargo compilation, WASM build, unit tests, Clippy and browser interaction tests: **not run; no Rust toolchain installed in this environment**.

### Required Windows tests

```powershell
cargo fmt --all -- --check
cargo test --locked -p zipeek-web
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo clippy --locked -p zipeek-web --all-targets -- -D warnings
```

Browser verification must cover route changes while generation and history finalization are active; restoration/retry; repeated completion; endless-session finishing; and revision conflict recovery. Do not accept R604 based on ZIP integrity alone.

---

### r605-generated-practice-state-ownership - R605 Generated Practice state ownership review

### Source scope

Based on R604, `crates/zipeek-web/src/pages/practice/generated_session/mod.rs` previously decoded both generated exercise kinds, owned selection, translation evaluation, and image-loading state, and rendered both interfaces in one component. This made state from unrelated variants coexist and required a broad reset effect.

### Implemented ownership

- `generated_session/mod.rs`: validates the saved generated payload, chooses a variant-specific keyed child, and exposes the existing `GeneratedPractice` interface.
- `generated_session/translation.rs`: owns translation tile selection, an evaluation request generation counter, evaluation result, and evaluation-in-progress state. It invalidates outstanding evaluations on unmount and Clear. Completion eligibility is now derived from a successful evaluation rather than stored in a second Boolean.
- `generated_session/image.rs`: owns image choice selection, answer check, and one `Loading | Ready | Failed` image lifecycle state. The former `loaded`/`failed` Boolean pair could represent conflicting states.
- `generated_session/order.rs`: retains deterministic tile-order generation, unchanged.

Variant children receive decoded and validated data as typed properties. A changed raw instance remounts its child and therefore resets its local state. The external props and completion callbacks are unchanged. No additional Context, reducer, dependency, persisted schema, seed algorithm or route was introduced.

### Open risks

- The AI evaluation transport is not cancelled; unmounted/cleared requests are invalidated at the UI publication boundary. A future service-level abort contract needs separate design.
- Accessibility and browser visual behavior must be checked on translation and image variants, including rapid route/instance changes, image load failure, answer Clear, and delayed AI evaluation.
- The generated instance key contains the validated raw payload. The current limit is 262,144 bytes. A future stable, typed instance identity could replace the content key if an approved identity contract is provided.
- Cross-tab Practice persistence and broader R604 tests remain unresolved.

### Evidence

- `node --test scripts/tests/*.test.cjs`: 16 passed, 0 failed.
- Static source/module review only; Cargo, rustfmt, locked Rust/WASM tests, Clippy and Chrome E2E were not available in this environment.

---

### r606-practice-store-review - R606 Practice session-store refactor review

### Source boundaries

The former `services/practice_session_store.rs` is now `services/practice_session_store/mod.rs`, with `codec.rs` owning validated JSON decoding, `inventory.rs` owning inventory admission and completed-removal predicates, `persistence.rs` retaining browser `Storage` access, and the existing model, transitions, generation, validation and tests remaining separate.

### Confirmed risk and correction

`recoverable_sessions()` previously read keys by live numeric indexes while `localStorage` could change. The revised implementation captures an ordered key snapshot, rejects a missing or duplicate enumerated key, reads records from that snapshot, and compares the key snapshot with a second enumeration. A key-set change detected during the scan produces `OutOfDate` instead of returning an inventory based on a shifting index sequence.

### Boundaries not solved

The snapshot comparison is **not** an atomic transaction. A cross-tab write may change the value of an existing key without changing the key set, and `load`-then-`set_item` remains a non-atomic optimistic revision check. Browser `localStorage` is not a suitable compare-and-swap database. Do not claim cross-tab write serializability or reliable collision prevention from these checks. Resolving that requires a separately approved transactional storage design and explicit data-loss/recovery analysis.

### Verification

- Focused Rust regression assertions were added for matching and changed key snapshots.
- The existing Practice session tests were retained, including completion timestamp tests.
- Rustfmt, Rust unit tests, locked WASM compilation and browser cross-tab tests were unavailable in this environment and remain blocking.
- Offline Node, documentation and archive verification results are recorded in the canonical release notes.

### Browser acceptance to perform

Test create, save, recoverable-session inventory, invalid-session cleanup, completion and discard in one tab; then test simultaneous mutation from two tabs, key insertion/removal during inventory, changed record values under the same key, quota errors and storage access denial. Confirm application UI exposes recovery paths and no saved session disappears silently.

---

### r607-ai-settings-request-ownership - R607 AI Settings diagnostic ownership review

### Scope and evidence

Baseline: R606 full source. Reviewed Settings AI page, reducer, status/trace presentation and connection-test call sites. No AI transport API, saved configuration format, browser permission or dependency change.

### Confirmed findings

1. The old AI Settings component combined configuration editing, persistence, async diagnostics and presentation in one Yew function. It stored a mutable request generation outside the reducer while reducer actions used an uncorrelated `test_busy` flag. A rapid second test before a render could start more than one request, while the generation guard would drop the older completion and the reducer had no request identity.
2. The old state allowed a completion action from any request to end whichever test was active, unless the caller remembered to gate it. The reducer now requires matching ticket identity.
3. Test cancellation is a UI-level dismissal, not a transport abort. Old requests may still reach the endpoint and incur cost. The UI calls this **Dismiss test** and states that the request may still finish.

### Implementation

- `settings/ai/mod.rs` declares modules and exports the page.
- `settings/ai/page.rs` composes the form and typed controller callbacks without owning the async task.
- `settings/ai/controller.rs` manages configuration changes, persistence notifications and a request gate. The gate synchronously prevents rapid duplicate test launches and invalidates stale completions on edit, save, clear, dismissal or unmount.
- `settings/ai/state.rs` owns reducer actions with explicit request tickets; successful and failed responses with stale tickets cannot mutate the current diagnostic state. The old `test_busy` bool is replaced by the `active_test: Option<u64>` lifecycle identity.
- `settings/ai/diagnostics.rs` continues to present status and redacted trace details.
- Focused Rust tests cover duplicate starts, stale completion after cancellation/restart and unrelated edit/save transitions.

### Remaining risks and acceptance

- Native Rust/Cargo/rustfmt, WASM compilation and browser execution were not available. Tests were authored, not run. Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and browser Settings AI tests.
- Dismissal does not abort network transport. A separate, approved service-level cancellation contract is required before claiming it does.
- Verify editing settings during an in-flight request, rapid double click, navigation away and back, failure/success trace, password key management, and both hosted `/zipeek/` and development `/` paths.

---

### r609-knowledge-engine-review - R609 Knowledge Points pure-domain engine: source candidate

### Scope

R609 introduces `zipeek_pedagogy::knowledge` as a separate pure-Rust candidate scoring module. It does not replace existing recognition or learning projections, and no web UI, AI evaluator, storage schema, Statistics or dragon is connected to it. See [R608 specification](../../learning-practice/pedagogy/knowledge-points.md).

The implementation covers entity/dimension identity, policy validation, integer scoring chunks, day-local distinct-context counting, attempt-ID idempotency, daily disqualification, debt-eligibility threshold, bounded linear quota, one ordinary debt event per day, debt repayment after qualification, single-chunk relapse, and debt-first relapse recovery.

There are no new dependencies. The public constructor requires a caller-supplied scoring policy: R609 deliberately does not establish final numerical debt defaults.

### Domain boundaries

- `model.rs`: canonical identity shape, dimensions, day and entity state, relapse phase.
- `policy.rs`: validated, explicit parameters; fixed dimension weight is encoded by projection denominator `5 * required_days`.
- `engine.rs`: synchronous validated event transitions, independent of WASM/Yew/AI/DB.
- `projection.rs`: rational chunk representation and aggregation; floating-point conversion is display-only.
- `tests.rs`: deterministic scenario regressions.

`context_key` is an **upstream-certified equivalence-family identifier**, not an arbitrary prompt ID. The engine can reject duplicate context keys per day but cannot prove two different keys are semantically distinct. No caller should feed this engine raw AI-generated IDs without an approved independent context validator.

The engine currently processes events in **accepted submission order** on each day and rejects earlier-day inserts. A pending evaluator event must therefore be fully resolved before a later event is committed for the same entity/dimension, or be excluded under the approved bounded-retry failure policy. This is an integration precondition, not implemented retry infrastructure.

### Explicit unresolved policy details

1. `successes_per_debt_unit`, `maximum_daily_successes`, `maximum_debt_units` and `relapse_debt_units` remain proposed calibration inputs, not approved constants.
2. As a candidate engine convention, each policy day snapshots its quota on its first assessment, so failures affect the next day. The source does not implement a separate persistent policy-day boundary or timezone selection.
3. As a candidate engine convention, qualifying a day repays one existing debt unit even if an attributed failure arrives later that day. A later attributable failure adds up to one new unit; final net debt follows event order. This needs explicit product review before integration.
4. As a candidate engine convention, severe relapse sets debt to `max(current_debt, configured_relapse_debt_units)` rather than adding debt on top. This needs approval when handling full credit with pre-existing ordinary debt.
5. A qualifying day while fully credited, or during relapse recovery, can repay debt without awarding a second chunk. Relapse chunk restoration occurs only at zero debt.
6. The exact policy-day boundary, distinct-context certification, grammar scoring, catalogue denominator and epoch-reset transaction are outside R609.
7. `seen_attempts` and per-day contexts are held in an in-memory state. Storage, replay, retention bounds and cross-tab serialization are unimplemented; do not feed untrusted unbounded journals without storage-layer protections.
8. The implementation does not yet provide a canonical full five-dimension catalogue projection or persisted event schema. The entity score aggregation is guarded for exactly five distinct dimensions.

### Verification

The Rust unit tests are authored but must be run with `cargo test --locked -p zipeek-pedagogy`. Rust compilation, rustfmt, Clippy, WASM and Chrome are not verified in the agent environment. The new module is not wired into live learning and does not make any current user's dragon or Statistics assessed.

Recommended acceptance commands:

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
```

The R608 product specification remains design authority; disagreements with the listed candidate engine conventions must be resolved before the module becomes live scoring authority.

---

### R310-E2E-DRIVER-REFINEMENT-WIP - E2E driver ergonomics - WIP after R309

The browser selector convenience API is implemented as `zipeek_e2e::page_objects::WebDriverActions`. Import the trait at the call site to use `driver.click(css).await?` or `driver.element(css).await?`. Both operations retain normal Thirtyfour `WebDriverResult` errors and `single()` lookup semantics.

Shared selectors are in `zipeek_e2e::page_objects::selectors::{practice,image_review}`. Recorder buttons assert their required event dispatch metadata; Image Review adds stable `data-image-review` attributes on its container, status, error and approval elements.

**Validation:** Node source contracts: 842 total, 729 passed and the same 113 pre-existing failures as R309. Four new focused source contracts passed. Rust compilation, rustfmt and browser E2E not executed because Cargo and Chrome were not available.

**Release status:** WIP only. Seven Rust paths have purposeful changes (including two new reusable E2E files); this does not satisfy the 20-substantive-Rust-file major release requirement. The existing R309 release notes are preserved unchanged.

---

### R596-LEARNING-DATA-REVIEW - R596 Learning Data settings: specification and implementation review

### Source and screenshot findings

- The screenshot combines Practice history recording, stored outcome diagnostics, portable backup, restore, deletion, data categories and worker diagnostics into one long form. The user cannot tell if the pedagogical engine is active.
- `learning_history_enabled` is the saved preference for recording completed Practice outcomes to a personal DuckDB/OPFS store, not a proficiency engine switch. It is default-off. A saved opt-in is neither proof of successful outcome commits nor of pedagogy activation.
- `zipeek-pedagogy` provides a foundation and a separate diagnostic worker, but the worker is not connected to authoritative stored evidence or to the Statistics UI. Worker health is not a mastery status.
- The header renders a small conditional SVG only when the recording preference is on, plus a centered route title. This violates `docs/product-ux/foundation/header-and-dragon.md`: the approved header requires long detailed dragon artwork as the progress shape based on verified persisted evidence, with no progress labels. There is no authoritative progress projection to render in the current source.

### R596 source changes

- Settings now clearly distinguishes Practice saving preference (On/Off/Unavailable) from unconnected pedagogical assessment and says what enabling the former does not do.
- Recording status reads the persisted preference and reacts to same-tab `zipeek-recording-change` and cross-tab `storage` events, consistent with the AppShell pattern.
- Storage operations are grouped into Practice records, collapsed Backup/Restore and Delete sections; history inspection and destructive history deletion are separately collapsed.
- One-shot worker probe is under Technical diagnostics and is explicitly not an opt-in or mastery operation.
- No storage format, recording preference behavior, persisted data, network dependency or worker execution behavior was changed.

### Open P0: header dragon and pedagogical data

Do not claim this requirement complete. The full dragon must use approved long-form artwork and verified persisted evidence. No fake percentage, static completion bar, proxy count or invented proficiency. The header's centered title and small conditionally shown SVG remain technical/UX debt. Required work: approve/render appropriate art asset, wire committed and validated recognition projections from personal storage through a typed browser data service, distinguish disabled/unavailable/loading/progress states, then implement accessible responsive rendering and test in Chrome under root and /zipeek/ base paths with reduced motion.

### Unverified gates

The Rust toolchain and browser tests were unavailable. Run `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, and the relevant settings and global header browser E2E journeys. Test recording preference on/off/error, same-tab update, cross-tab update, backup/restore accordion, destructive confirmation, and technical worker check.

---

### R600-REFACTOR-CANDIDATES - R600 corrective audit: next ten refactor candidates

Basis: the R591 codebase audit (risk items A03-A07, A11) and direct inspection of the R600 source contents. This is a **candidate list**, not a list of ten proven bugs or ten completed fixes. Priority depends on invariants and test coverage, not file size alone.

| # | Priority | Path | Observed coupling / review objective |
|---|---|---|---|
| 1 | P0 | `crates/zipeek-web/src/pages/practice/local_session.rs` | 810 lines; combines stateful session UI, async generation, completion/progress projections and callbacks. Extract typed coordinator/hook and pure progress projection; test stale generation and session termination. |
| 2 | P0 | `crates/zipeek-web/src/pages/practice/generated_session.rs` | 394 lines with multiple local state hooks and async tasks. Separate generation request lifecycle and presentation; test cancellation, retries and unmount. |
| 3 | P0 | `crates/zipeek-web/src/pages/system/stats/state/mod.rs` | 498-line shared reducer with selection, filters, paging, overlays and tiles. Audit transition invariants and typed IDs; isolate reducer subdomains without introducing parallel authoritative state. |
| 4 | P0 | `crates/zipeek-web/src/pages/system/stats/query.rs` | 644 lines containing query contracts, pagination, projections and filtering. Split domain query definitions, query validation and pure selectors; keep typed page contract. |
| 5 | P1 | `crates/zipeek-web/src/pages/system/settings/ai.rs` | 312-line UI handles reducer, storage and two async requests. Move diagnostics lifecycle to a hook and isolate config persistence from presentation; maintain credential safety. |
| 6 | P1 | `crates/zipeek-exercises/src/crossword.rs` | 845-line exercise implementation intertwines puzzle domain state, layout generation, scoring and helpers. Separate deterministic generator from runtime transitions; add seeded/failure tests. |
| 7 | P1 | `crates/zipeek-exercises/src/time_weaver.rs` | 857-line date/time language representation and gameplay state machine. Extract date/time formatting and generator from runtime; add boundary/date tests. |
| 8 | P1 | `crates/zipeek-exercises/src/session.rs` | 479-line session aggregate, event recording, validation and terminal transitions. Audit atomicity/idempotency; separate immutable domain contracts from transition implementation where valuable. |
| 9 | P1 | `crates/zipeek-db/src/wasm/personal_history.rs` | 463-line storage adapter combines parsing, locking, aggregation, transactional writes and erase. Separate validation and transaction coordination; test quota/failure and partial commit. |
| 10 | P1 | `crates/zipeek-db/src/wasm/recognition_history.rs` | 408-line verified-evidence adapter handles validation, SQL read/write, retry conflict and readback. Extract pure validation/invariants from storage operations; verify duplicate and retry behavior. |

### Gating criteria

For each candidate, record a source-level finding and a justified code correction or explicit no-change decision. Maintain module-root consistency (`feature/mod.rs`), narrow typed props, single ownership for async state, cancellation and persistence, and tests for error/terminal paths. Do not treat file splitting as debt resolution. This R600 corrective checkpoint does **not** implement these ten refactors.

### Verification outstanding

`cargo fmt --all -- --check`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, locked tests, Clippy and browser acceptance must be run in a Rust-equipped environment. This checkpoint fixes only the reported `SettingsPage` visibility chain.
