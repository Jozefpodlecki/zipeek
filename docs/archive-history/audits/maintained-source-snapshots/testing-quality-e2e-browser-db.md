# Browser dictionary verification

## Current test targets

The dictionary is served from [`zipeek-web/static`](../../../../crates/zipeek-web/static) and loaded through [`zipeek-db` WASM](../../../../crates/zipeek-db/src/wasm). The installed manifest currently declares **12,188** words, normalized relations and browser package `1.33.1-dev114.0`. The original 12,100-row tests and 1.32.0 browser engine described in old A06 notes were evidence for a **different, earlier artifact** and are not release criteria for the current one.

From the workspace root on a configured Windows development machine:

```powershell
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo test --release --locked -p zipeek-e2e --test browse_interactions -- --nocapture
cargo test --release --locked -p zipeek-e2e --test settings_diagnostics -- --nocapture
cargo test --release --locked -p zipeek-e2e --test route_matrix -- --nocapture
```

The user reported all six Browse scenarios passing (Word Sets, pronunciation accessibility, empty search, live search/clear, pagination) and the accessibility matrix passing on an earlier R282 source. The latest UUID translation changes have not undergone the full browser matrix. The tests must check actual runtime readiness and query behavior rather than accepting the presence of a manifest or an element as a pass.

## Invariants to verify

- Manifest checksums and source metadata refer to the installed compressed database; the browser resolves local worker/Arrow assets for both root and `/zipeek/` base.
- The connection opens dictionary data **read-only**; no Practice/personal-data mutation occurs there.
- A typed read must succeed before the app reports Ready; failure and Retry remain navigable.
- The current manifest metadata, browser engine and runtime disk statistics are separate sources of facts, displayed separately in Debug.
- Browse filtered search must not duplicate IDs; pages must be ordered and next/previous transitions reproducible for the same source snapshot.
- Confirm generator `character_readings` presence against the actual installed database before claiming character-reading query coverage: the installed manifest table list omits it although the generator SQL defines it.

See [current validation](../../../project-management/current.md), [browser DB architecture](../../../architecture/modules/browser-db.md) and [source reconciliation](../SOURCE_RECONCILIATION.md). Historical output-specific hashes and old Chrome runs remain in [release history](../../RELEASE_HISTORY.md).
