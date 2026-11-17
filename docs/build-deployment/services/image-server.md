# Image service scaffold

The standalone native crate `crates/zipeek-image-server` is **not used by the Yew frontend yet**. It runs an Axum server bound to `127.0.0.1:1431` and uses a deterministic **SVG placeholder** backend, not Candle or a diffusion model. A trait in `src/inference` is the future inference integration point. Generated placeholders are **not valid image questions** and must not be used as learner-facing assessment content.

## Build and launch

On first install, run `cargo check -p zipeek-image-server` **without** `--locked` to update Cargo.lock for the new dependencies. Commit the generated lockfile, then use `--locked` in CI. This release could not regenerate Cargo.lock because no Rust toolchain is available in its packaging environment. Do not run `cargo generate-lockfile` blindly if preserving existing pinned dependency resolutions matters.

```powershell
cargo check -p zipeek-image-server
cargo test -p zipeek-image-server
cargo make image-server
```

### API

- `GET http://127.0.0.1:1431/health` returns the backend identifier.
- `POST /v1/images` with `{ "prompt": "red apple on a table", "width": 512, "height": 512 }` returns HTTP 202 and an in-memory job ID.
- `GET /v1/jobs/{uuid}` returns `queued`, `running`, `completed` or `failed`, and `image_url` on completion.
- `GET /v1/images/{uuid}` serves the persisted SVG.

Job statuses are process-local; image bytes and their metadata survive restart in `target/zipeek-images`. A restart does not restore old job statuses. Queue admission is capped at four pending/running tasks, with one active generator. IDs are parsed as UUIDs; request bodies, dimensions and prompt length are bounded. There is no public-bind or authentication support, and no configured cross-origin access.

This is an architecture scaffold. Production work must add a real inference implementation, GPU feature-gating, output image validation, review/approval, durable queue state, retention/cleanup, authentication if remote exposure is needed, and image-exercise planning and scoring in `zipeek-exercises`.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/build-deployment-services-image-server.md). Dated implementation claims in that snapshot are not current build or browser verification.
