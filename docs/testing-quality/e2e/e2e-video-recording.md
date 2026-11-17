# E2E video and feedback evidence

## Purpose

Each Chrome scenario supplies reviewable evidence for the developer and Codex. An MP4 shows visual activity, while named screenshots and timeline records isolate relevant transitions. A recording is not proof that assertions passed, and a passing assertion is not proof that video capture succeeded.

## Capture contract

- `zipeek-e2e` uses Thirtyfour with Chrome CDP `Page.screencastFrame` events, base64-decoded in memory, and a bounded FFmpeg input channel.
- Frames are streamed to an H.264/yuv420p `video/recording.mp4` and are **not** retained as JPEG files or a frame manifest.
- Frame count, byte total and dropped/truncated flags are written to `video/status.json` after the encoder finishes.
- FFprobe checks for a readable video stream when available. The manifest distinguishes `validated`, `unverified`, `failed` and `unavailable`. FFprobe stream probing is not a guarantee that every video frame will decode.
- Named `timeline/steps.jsonl` checkpoints contain an event label, wall-clock Unix milliseconds and a screenshot path. Screenshots are enabled by default when checkpoints are enabled. They can be disabled with `ZIPEEK_E2E_SCREENSHOTS=0`.
- Browser diagnostics and a failure screenshot are collected before stopping the recorder and quitting the browser.
- Normal scenarios retain their application and assertion result even if recording fails. `ZIPEEK_E2E_VIDEO_STRICT=1` fails a scenario that otherwise passed if capture or validation fails; it does not override a primary application failure.

## Environment controls

| Variable | Behavior |
| --- | --- |
| `ZIPEEK_E2E_VIDEO=0` | Disable MP4 recording |
| `ZIPEEK_E2E_VIDEO_STRICT=1` | Require successful recording and probe for otherwise passing scenarios |
| `ZIPEEK_E2E_RECORD=0` | Disable timeline checkpoints |
| `ZIPEEK_E2E_SCREENSHOTS=0` | Skip checkpoint images while retaining event metadata |
| `ZIPEEK_E2E_CAPTURE_AI_REQUESTS=1` | Opt in to sensitive fixture request diagnostics; review before sharing |

FFmpeg is required for MP4 creation and FFprobe is required to mark a recording validated. Chrome is required for CDP capture. Unsupported browsers do not produce CDP recordings.

## Artifacts

```text
target/e2e/<suite>/
  suite.json
  summary.txt
  review.md
  <scenario>/
    scenario.json
    agent-feedback.md
    timeline/
      steps.jsonl
      000-scenario-ready.png
      ...
    video/
      recording.mp4
      status.json
    browser.json
    failure.png
    failure.txt
```

Some files are conditional on browser availability and scenario outcome. `review.md` summarizes all recorded scenarios. `agent-feedback.md` records execution status, assertion status, video state, failure message and available evidence checkpoints. It is generated from observed metadata, not from automatic video interpretation. Times in `steps.jsonl` are wall-clock timestamps; `video/status.json` includes the recorder start time to help correlate events. Recording timing is event-driven, not frame-accurate.

## Recommended verification

1. Run the Chrome browser scenario with FFmpeg and FFprobe installed.
2. Open the MP4 in a real media player and inspect the initial frame, the failure/feedback transition and the final frame.
3. Compare named screenshots and timeline labels against `scenario.json` and `agent-feedback.md`.
4. Check that application failures, assertion failures and recording failures remain distinguishable.
5. Treat any unavailable, truncated or unverified video as incomplete evidence, even if the test assertions pass.

Do not upload browser recordings or diagnostics containing private learning records or credentials without review.
