# E2E review workflow for developer and Codex

## Evidence goals

Each important browser journey should allow a developer or Codex agent to reconstruct **what happened, what was expected, and where to inspect it**. A passing Rust assertion alone is not sufficient evidence of visual quality. A video artifact alone is not proof that the video was watched or decoded.

## Scenario evidence package

A recorded scenario should retain `recording.mp4`, a scenario result/manifest, timestamped named checkpoints, associated screenshots, browser diagnostics and `agent-feedback.md`. A suite-level `review.md` summarizes which scenarios require attention. Actual output paths and exact JSON keys are owned by the [current harness contract](e2e-architecture.md); this page specifies desired review semantics rather than inventing current files.

Report application execution, assertions and recording integrity **separately**. A scenario can complete its assertions while recording fails; do not collapse both into a single green status. Where FFprobe/FFmpeg are unavailable, state that video validation was not performed. File presence alone is not sufficient playback validation.

## Checkpoint requirements

Capture meaningful transitions instead of constant redundant frames: initial state, active selection, incorrect feedback, correction/hint, accepted answer, completion and any recovery or error. Include a stable step name, a timestamp relative to the recording, the assertion being checked, and the screenshot path where captured.

When a scenario fails, preserve the observed UI and error evidence before cleanup. Never hide a failure by weakening an assertion merely because current UI differs from an approved specification. Mark unimplemented behavior as missing or blocked until a positive browser acceptance run exists.

## Agent review protocol

1. Find the failing scenario in the suite report and inspect the corresponding scenario manifest.
2. Read the expected/observed assertion and the step timeline.
3. Inspect the nearest checkpoint screenshot; use the MP4 to understand transitions and timing where available.
4. Compare observed behavior to the [authoritative acceptance criteria](../acceptance/approved-behavior-acceptance.md), not an obsolete UI snapshot.
5. Form a falsifiable root-cause hypothesis, inspect the relevant implementation, correct it within approved scope and rerun the same scenario.
6. Record what changed and what passed, failed or remained unverified. Do not claim automatic visual understanding merely from generating `agent-feedback.md`.

## Priority reference scenario

Word mosaics should cover first-tile selection, attempted wrong pair and red feedback, automatic deselection, subsequent progressive hint, correct-pair confirmation and matched-count advancement, then full completion. Include keyboard/reduced-motion variants where supported. Existing tests that only check a generic mismatch string are **partial coverage**, not acceptance of the new requirement.

## Review quality gates

Validate that referenced screenshots exist, the timeline relates to the MP4, the video is playable where FFprobe is available, failures retain diagnostics, and strict recording runs fail their evidence gate if capture is unavailable or truncated. Report unavailable Chrome/driver/FFmpeg prerequisites as blocked, not passed.

See [coverage matrix](e2e-requirements-matrix.md), [video contract](e2e-video-recording.md) and [E2E recording guide](e2e-recording.md).

## Offline evidence integrity audit

After a browser run, inspect the concrete `target/e2e/<run-directory>` printed by the harness. From the repository root:

```powershell
node scripts/e2e-evidence-audit.cjs target/e2e/<run-directory> --strict-evidence --decode-video --write-report
```

The audit produces `evidence-audit.md` and `evidence-audit.json` in that run directory, checks that suite and scenario manifests agree, checks checkpoint order and references, validates PNG signatures, and uses FFmpeg to decode the MP4 when `--decode-video` is supplied. It reports artifact integrity separately from the Rust scenario's assertion status. It cannot judge visual correctness by itself.

Use `--strict-evidence` only for evidence-producing runs; it requires a validated recording and checkpoints for every scenario. For diagnostic runs without a video, omit that flag. An evidence audit exit code of 1 signals inconsistent or missing artifacts, 2 signals incorrect invocation or a missing directory. The resulting JSON is suitable for a Codex investigation, but no automatic frame interpretation has occurred.

Run the audit script's offline regression tests without Rust or Chrome:

```powershell
node --test scripts/tests/e2e-evidence-audit.test.cjs
```

For the browser harness, set `ZIPEEK_E2E_EVIDENCE_STRICT=1` to fail scenarios when required video status or checkpoints are missing. Set `ZIPEEK_E2E_VIDEO_STRICT=1` to require FFprobe-validated video capture at the recorder level. `ZIPEEK_E2E_SCREENSHOTS=0` disables optional checkpoint screenshots and should not be set during visual acceptance. Preserve video files, reports, timeline data and `failure.png` together when sharing failures with Codex.

These changes improve evidence completeness, but they do not by themselves make old tests for retired UI behavior correct. The [requirements matrix](e2e-requirements-matrix.md) continues to distinguish authored tests from executed acceptance.
