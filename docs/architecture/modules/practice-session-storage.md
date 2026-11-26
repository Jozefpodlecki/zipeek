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
