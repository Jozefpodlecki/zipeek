# Hanzi writing domain crate - R106

`crates/zipeek-hanzi-writing` is the domain-only foundation for a future Hanzi Drawing exercise. It has no third-party dependencies and does not reference Yew, `web_sys`, HTML canvas, browser events, fonts, or stroke-dataset formats.

## Boundary

- `geometry`: validated two-dimensional points, ordered strokes, extents, CSS-client-to-normalized canvas coordinates, normalized-to-backing coordinates (including high-DPI canvases), deterministic arc-length resampling and translation/uniform-scale normalization.
- `input`: pointer-specific pen capture, duplicate sample suppression, completion, cancellation, undo and clear. It deliberately does not capture pointer events or assign pressure/touch semantics.
- `comparison`: a deterministic order-sensitive geometric distance diagnostic for equal stroke counts. Results are **not** correctness scores, HSK evidence, stroke-order certification, or a basis for grading without calibrated reference data and evaluated thresholds.

## Browser adapter (not implemented)

A Yew component should use a canvas element and pointer events. Read `getBoundingClientRect`, CSS dimensions, and drawing buffer dimensions, then create `CanvasTransform`. Call `client_to_normalized` for samples, record them using `InkCapture`, and on redraw map the normalized values back to backing pixels. Apply pointer capture and handle `pointercancel`, `lostpointercapture`, resize, and device pixel ratio changes. Prevent unwanted touch scrolling only on the drawing surface. Canvas rendering and hit-testing must not become core crate dependencies.

## Future test strategy

Rust unit tests cover pointer lifecycle and coordinate geometry using numeric fixtures, including invalid dimensions, NaN input, duplicate samples, wrong pointers, cancel, resampling, and reversible navigation state. Browser E2E should later verify pointer capture, touch-action, resize/DPR scaling, rendering and undo using controlled pointer events. Real stroke-order assessment needs a separately approved, licensed dataset and measured false-positive/false-negative rates. No DB schema changes are needed for the domain crate.

## R107 - Browser drawing workspace

- Added `/practice/hanzi-writing` as a separate, explicitly **unscored** preview accessible from Practice; the route does not require DuckDB. Character samples are temporary UI fixtures, **not** licensed reference stroke trajectories.
- `crates/zipeek-web/src/pages/hanzi_writing.rs` owns canvas drawing only. It uses `CanvasTransform` to map pointer client coordinates into normalized positions, and `InkCapture` for pointer ownership, duplicate suppression, completion, cancellation, undo, and clear. All completed and active strokes are redrawn from the same model.
- Rendering keeps the 2D backing resolution proportional to device pixel ratio (capped at 2048 pixels) and redraws on window resize, model changes, guide changes, or character selection. Pointer capture keeps a stroke connected when the pointer exits the drawing area. `touch-none` applies only to the writing surface.
- Known follow-ups: browser E2E for pointer capture, lost capture, DPI/resize, and touch; live dictionary vocabulary target selection; sourced stroke datasets, stroke-order feedback and independently validated assessment. Changing the target clears prior ink. No grades, scores, or session ledger writes are produced.

## R319 - Current guided handwriting display

The guided-writing route has a Yew reducer for the active character, in-progress ink, and sequential grading session. The `WritingBoard` owns the rendering surface, while a visual-only `DirectionGuide` previews 25% of the next canonical median using arc-length interpolation, growing to 35% and 50% after successive rejections of the same stroke. A static fading amber stroke replaces the former start dot, and a small start anchor remains. The reducer counts per-stroke rejections and resets the count after acceptance, Undo, Clear or character loading. This overlay is suspended during drawing and playback, does not modify matching thresholds, and does not animate. Browser acceptance and touch/stylus calibration remain pending.

## R510 WIP - Recognition and prompt boundary

`matching.rs` validates endpoint placement, net direction and plausible length before calling `elastic.rs` for constrained dynamic-time-warp alignment and symmetric robust path coverage. The latter relaxes timing/vertex alignment, not stroke identity. `session.rs` checks candidate scores against later reference strokes to give stroke-order feedback; indistinguishable overlaps advance in supplied canonical order. The web reducer exposes `StrokeRejection::message()` without leaking matcher internals into views. No handwriting input leaves the browser.

`WriteHanziOptions` in `zipeek-web` owns three independently persisted boolean prompt flags. `practice_exercise_settings/writing.rs` configures them; `write_hanzi_exercise.rs` renders dictionary-sourced word title metadata and reuses `ListenButton` / `SpeechService` for user-initiated `zh-CN` playback. The `HanziWriting` canvas and stroke loading remain isolated from word metadata. The new strict options format intentionally has no old-format reader.
