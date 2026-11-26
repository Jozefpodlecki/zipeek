# UI architecture and state ownership

## Ownership principles

The root application owns durable cross-route services and readiness boundaries. A feature owns its transient interaction state unless it must be shared across routes or persisted. Use pure domain transitions and focused reducers where multiple fields must change consistently. Do not introduce a global provider for each local control.

- Root state includes theme preference, dictionary/database readiness and shared session-level services where source code confirms them.
- Route and feature state includes browse queries, Word Set selection, wizard drafts, lesson navigation and exercise-local drafts.
- Persistence services own browser data writes; UI components should not independently derive mastery counters or duplicate final outcomes.
- Async requests, animation timers and event listeners are scoped to their owners, cancelled or invalidated on navigation, and protected from stale completions.

Read [state ownership](ui-state-ownership.md), [Browse state](ui-browse-state.md), [system architecture](architecture.md) and [shared Practice exercise foundation](../learning-practice/practice/shared-exercise-foundation.md) before changing providers.

## Practice composition

A session host owns session identity, persistence, scheduling, navigation and aggregate progress. A shared exercise frame owns common presentation, accessibility, actions and feedback. Each exercise owns its domain interaction rules and emits typed attempts/outcomes to the host. Do not make an oversized component that switches on every exercise type. Skip and Replace does not count as an exercise verdict. See [Practice outcomes](../learning-practice/practice/practice-outcomes.md).

## Layout and motion

Use the current Tailwind/theme infrastructure and shared accessible control styles. The global header should not repeat page or exercise titles. The long dragon-shaped header visual reflects persisted pedagogical evidence, not completion or a floating percentage. Motion must preserve accessibility and honor reduced motion. Avoid implicit animation ownership shared between unrelated components; stop frame tasks on component unmount or content change.

## Verification and source boundaries

Implementation should be checked against the actual Yew provider tree, `crates/zipeek-web/src/pages/`, and the responsible domain crates before refactoring. Names in this document are conceptual boundaries, not claims that every shared component already exists. Validate both root and hosted routing, keyboard access, browser storage and in-flight work cancellation.

Older checkpoint observations, including source descriptions of Tailwind, Grammar and course adapters, are preserved in [the archived UI architecture snapshot](../archive-history/audits/superseded-maintained/architecture--ui-architecture-source.md). They are not evidence of current compilation or acceptance.

## Statistics dashboard architecture

The approved Statistics 2.0 UI uses a feature reducer and scoped context only for genuinely shared dashboard state. Heavy read-only projections are separate from UI selection state; typed IDs, stable paging, explicit provenance and Rust-owned Chart.js lifecycle are planned in the [Statistics architecture specification](modules/statistics-dashboard.md). This remains a design target, not the implemented R564 state-management structure.

## Record feature ownership (R670 WIP)

The Record reference and replay panels read `currentTime / duration` from the active local browser audio element to render a progressing PCM waveform and offer accessible seeking; the reference text highlights progressively by character index. This highlight approximates timing and does not represent forced speech alignment. Playback progress is transient component state, not the reducer or persistent learning data.

The `/record` route is composed within `crates/zipeek-web/src/pages/record/`. `mod.rs` mounts the feature controller and typed child views. `state.rs` owns target and workflow transitions, with a `RecordingId` identifying the current captured utterance. `audio.rs` owns the transient microphone WAV and its PCM-derived waveform. `controller.rs` owns local TTS/STT callbacks, cancellation, stale-request invalidation and the audio resource lifecycle. Target selection, reference, microphone, waveform and transcription panels render from typed properties and emit callbacks; they do not independently own speech requests or WAV buffers. The recorded payload is never in a Yew render-state value, and no audio persists beyond the route lifecycle. This is an authored, not yet WASM/browser-verified source boundary.
