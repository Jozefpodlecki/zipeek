# Chrome E2E recording and AI test evidence

Zipeek uses Rust/Thirtyfour with managed Chrome for repeatable browser tests. The harness records a compact `steps.jsonl` event timeline and streams CDP frames straight to FFmpeg to produce an MP4, without storing individual frame JPEGs. Failure screenshots remain automatic; named checkpoint screenshots are enabled by default; use `ZIPEEK_E2E_SCREENSHOTS=0` to disable them. Chrome DevTools Recorder JSON remains a separate option for manually reproducing interactions. Video capture and checkpoints are enabled by default, including during application initialization; set `ZIPEEK_E2E_VIDEO=0` or `ZIPEEK_E2E_RECORD=0` to disable them.

## Record a deterministic automated run (PowerShell)

From the repository root after `cargo make unpack`:

```powershell
$env:ZIPEEK_E2E_RECORD = '1'
cargo test -p zipeek-e2e --test smoke --locked -- --nocapture
```

`ZIPEEK_E2E_VISIBLE=1` shows the managed Chrome window. Leave it unset for headless execution. Each scenario saves `target/e2e/root-*/<scenario>/timeline/steps.jsonl`, with PNG screenshots unless `ZIPEEK_E2E_SCREENSHOTS=0`. The harness also writes `agent-feedback.md`, `scenario.json`, and either `passed.txt` or `failure.txt`; failure screenshots are conditional. Even if the Rust test fails, the evidence directory is retained. The new `ai_sentence_uuid_generates_restores_and_continues` scenario uses the deterministic local fixture and the browser's real Practice setup and UUID session. The harness runs its own fixture server, so it does not require LM Studio or a cloud provider.

For an **actual configured model** (optional; must be running and accessible from Chrome):

```powershell
$env:ZIPEEK_E2E_RECORD = '1'
$env:ZIPEEK_E2E_AI_ENDPOINT = 'http://127.0.0.1:1234/v1/'
$env:ZIPEEK_E2E_AI_MODEL = 'your-loaded-model-id'
cargo test -p zipeek-e2e --test ai_learner --locked -- --ignored --nocapture
```

The live-model test is opt-in. Its outcome is not deterministic and must not be reported as CI fixture success. No API credentials are included in the source archive.

## Record a manual web-app journey with Chrome DevTools Recorder

1. Open Zipeek in Chrome, then press `Ctrl+Shift+P` in DevTools and select **Show Recorder panel**.
2. Create a recording, start recording, and reproduce the setup/AI/refresh/answer sequence.
3. Stop and replay the recording to verify the steps.
4. Select **Export → JSON** and save the user flow (for example `zipeek-practice-flow.json`). This is a replayable interaction specification, not a video. You can also export a Puppeteer script using Chrome's own option.
5. Upload the JSON here, optionally with screenshots or the automated bundle. Remove any text fields containing secrets, private words, URLs with tokens, or sensitive prompts before sharing.

Reference: [Chrome DevTools Recorder](https://developer.chrome.com/docs/devtools/recorder/reference).

## One-command evidence packaging

After any E2E run, from the repository root on Windows:

```powershell
cargo make pack-e2e
```

This picks the newest run in `target/e2e/root-*` and creates `target/e2e/zipeek-e2e-root-<run-id>.zip`. It includes scenario MP4 recordings, assertions, timeline records, available failure screenshots and the suite summary. Checkpoint screenshots are included by default unless disabled. It does not run tests and does not include raw AI requests or browser diagnostics by default. If no run exists, the command fails with instructions to run E2E first. Review all visual artifacts before uploading.

For evidence packaging, use the maintained `cargo make pack-e2e` task. The former PowerShell evidence helper is not supported. For an older run, use the supported native CLI/task options described in current `cargo make --list` output and the CLI help rather than relying on a removed script. Review output, screenshots, Recorder JSON and diagnostic content for private data before sharing; evidence packaging is not proof of a passing test.

## Continuous video and deterministic AI Practice

CDP video capture is on by default; use `ZIPEEK_E2E_VIDEO=0` to disable it. Set `ZIPEEK_E2E_VIDEO=1` explicitly to request CDP screencast capture for each Thirtyfour scenario, separately from `ZIPEEK_E2E_RECORD=1` checkpoint timelines (set `ZIPEEK_E2E_SCREENSHOTS=0` to disable PNG screenshots). Each scenario streams bounded JPEG frames into FFmpeg's stdin and retains `video/recording.mp4` plus `video/status.json` without individual frame files. Set `ZIPEEK_E2E_VIDEO_STRICT=1` only when a missing video must fail an otherwise successful scenario. The PowerShell exporter includes MP4 and compact recording status, but raw AI requests remain opt-in. This workflow is not yet verified against local Chrome or Windows PowerShell.

## Focused AI fixture acceptance

From the repository root, with no LM Studio instance required:

```powershell
cargo test -p zipeek-e2e --test smoke deterministic_ai_practice --locked -- --nocapture --test-threads=1
```

This runs two independently recorded scenarios: the missing-model launch guard and a fixture-generated Sentence Translation flow with reload, answer feedback and continuation. Each scenario gets a fresh browser; the local fixture server is shared per suite. Inspect `target/e2e/root-*/suite.json` and each scenario's `scenario.json` for success/failure, then use `cargo make pack-e2e` to prepare an uploadable ZIP. The test binary is not a substitute for the optional live-model `ai_learner` target.

If browser startup or dictionary initialization fails, look for `browser-failure.txt`, `failure.txt`, `failure.png`, `video-failure.txt`, and `scenario.json` in the scenario directory. A failed app server writes `setup-failure.txt`, `suite.json` and `summary.txt` at the run root.

For the current output schema and independent recording quality status, see [video evidence](e2e-video-recording.md) and the [requirements coverage matrix](e2e-requirements-matrix.md).
