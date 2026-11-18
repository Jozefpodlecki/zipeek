# Current work

## R570 WIP: Statistics asynchronous data-source architecture

The seven Statistics preview sections now use a replaceable asynchronous `StatisticsSource` and typed, owned section projections. A separate reducer/context manages section loading, completion, failures, retries, stale-request rejection and cache reset. Rendering modules no longer read the fixture constants directly. The route retains its existing Statistics UI reducer for filters, sorting, pagination, selection and tile preferences; the data context owns only fetched projections.

The default source is still offline fictional data. There is no real Practice or Word Set transfer, no personal evidence read/write and no production proficiency inference. Remote high-volume filtering, pagination, canonical evidence IDs and server request cancellation remain outstanding; do not confuse the section source boundary with a completed production API.

Source and archive checks only. Rust/WASM compilation, Chrome E2E and accessibility review must be run on Windows. Verify per-tab loading, retry, cancellation on close/navigation, cache behavior, empty states and root/hosted deployment before acceptance. `cargo fmt --all -- --check`, `cargo test --locked -p zipeek-web`, `cargo check --locked --target wasm32-unknown-unknown -p zipeek-web`, `cargo test --release --locked -p zipeek-e2e --test stats_dashboard -- --nocapture`.
