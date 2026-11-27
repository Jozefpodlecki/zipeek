# Practice session persistence and lifecycle boundaries

**Current source boundaries:** `crates/zipeek-web/src/services/practice_session_store/`, `pages/practice/local_session/` and `pages/practice/generated_session/`. These modules do not yet constitute a verified end-to-end browser acceptance. This architecture concerns temporary Practice session state, **not** authoritative R608 pedagogical evidence.

## Local session ownership

`local_session/mod.rs` exports the route-keyed component; `page.rs` composes Yew state and effects; `restore.rs` recovers stored sessions; `generation.rs` reconciles generation attempts; `persistence.rs` writes retries, progress and completion; `history.rs` owns finalization; `exercise.rs` renders validated steps. Route-keyed mounting and stale-result checks prevent some old-session result publication; they do **not** cancel external generation requests. A persisted `PracticeSession` remains authoritative.

Generated Practice isolates translation in `generated_session/translation.rs`, image vocabulary in `image.rs`, and deterministic tile ordering in `order.rs`. Separate child lifecycle state does not provide transport-level cancellation.

## Persistence and concurrency

`practice_session_store/mod.rs` exposes the store contract. `codec.rs` validates persisted JSON, `inventory.rs` validates recoverable record membership and cleanup, and `persistence.rs` owns browser storage and revisions; other modules own model, transitions, generation and invariants.

The recoverable-session inventory snapshots and sorts storage keys, reads the named records and checks a second key snapshot. This detects **some** concurrent key-set changes. It is **not** atomic cross-tab compare-and-swap: a value can change while the key set remains the same, and load-then-save remains optimistic. Never claim write serializability or collision freedom.

A completed practice outcome may be finalized into optional personal history. Session-history recording and R608 assessed-knowledge evidence are distinct contracts; one cannot substitute for the other.

## Acceptance still required

Run locked native/WASM tests and Chrome E2E for rapid route navigation, stale generation, restart/restore, completion/history idempotency, quota errors, invalid saved records, two-tab writes and inventory changes. Failed storage commits must not silently discard the last recoverable record.

Historical R602-R606 source-review detail is preserved in [release engineering history](../../archive-history/audits/release-engineering-review-history.md).

## R701 reviewed-answer progress preview

Character Builder reports reviewed correctness through an optional typed callback. The local resumable session uses its existing step-identified preview state so the theme creature animates on a correct reviewed answer without marking the persisted step complete. Continue remains the sole completion action. Wrong/retried answers do not grant preview progress. The page hides its Leave control after session completion, and the existing round transition honours reduced-motion CSS.

## R704 saved-session recovery controls

The setup recovery panel and compact session reminder provide direct one-click deletion of a saved session with no confirmation step. Deleting a valid session can lose results not yet committed to personal history. Recovery or resumption is the non-destructive alternative. Invalid records can be deleted separately; the storage service rechecks each record before removal and must not remove a record that has become decodable. Corrupt session-key suffixes are handled by the invalid-record cleanup path without relaxing the validation of loadable session IDs. This is browser-storage cleanup, not a migration or restoration of data.

## R704 immediate observation recording and Endless setup

Practice setup now exposes Endless for any selection of registered exercises with a resumable host, not only translation. Translation-only setup and mixed setup share `SessionLength` as the launch authority; the translation-only count picker updates that same state. The existing deterministic scheduler and three-step lookahead remain in place. The hard 1,000-step limit is still enforced; genuinely unlimited practice needs a bounded active-session window plus lossless chunked history persistence and updated readers.

When the user has enabled Practice learning-history recording, `practice_observations` records each committed exercise outcome under an immutable, step-identified local-storage key. This happens after successful session progress persistence, and restoration replays any missing observations from the saved session. Duplicate writes with identical evidence are idempotent; conflicting evidence is rejected. Erase Practice history also removes these observations; app reset clears local storage. This separate observation journal does not assert knowledge-entity attribution, validated mastery, or progression of the pedagogical engine. Qualified `KnowledgeLedger` updates still require independently verifiable exercise-level target manifests and a durable authoritative knowledge journal, and cannot be inferred from session outcomes alone.

Generated Sentence Translation and retry-resolved Sentence Builder now use the shared `ExerciseReview` for feedback plus Continue. Sentence Builder's retry branch remains distinct and does not emit Continue until its existing retry policy allows it.
