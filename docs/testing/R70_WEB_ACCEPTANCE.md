# R70 web acceptance — first local run

This is a manual acceptance checklist, not a claim that Windows, WASM, Chrome,
or LM Studio were executed in the source-only build environment.

## Stop classification without losing valid checkpoints

1. In the terminal running `cargo make db-full`, press **Ctrl+C** once and let
   the process exit. Do not delete `target/db-workflow/` or change the model ID
   or classification prompt before resuming.
2. The `db-full` workflow installs a newly enriched bundle only after all senses
   have been classified and the artifact validates. At 25% it has *not* installed
   a partially enriched DB. The checked-in/statically installed DuckDB file can
   still be used for exercising the web interface.
3. To test the existing installed artifact, use `cargo make serve` from the
   repository root. **Do not use `cargo make db-full-dev`** when the goal is to
   test the web app, because it resumes classification before serving.
4. Keep a backup copy of the checkpoint directory before any manual cleanup.
   Interruption alone does not require cache repair.

## Compile and launch (PowerShell from the repository root)

```powershell
cargo fmt --all -- --check
cargo test -p zipeek-exercises --locked
cargo test --workspace --locked
cargo check -p zipeek --target wasm32-unknown-unknown --locked
cargo make serve
```

The static assets and database must remain available at the URLs expected by
Trunk. The production base path remains `/zipeek/`; the local development server
serves at `/`. Do not regenerate the database just to test the UI.

## In-browser checklist

1. Verify the Home, Browse, Practice setup, and Sentence Translation routes
   load without a console exception. Check that Browse queries the installed
   DuckDB bundle and that Practice can load a round.
2. Finish one Matching round; note the shared completed-exercise count. Move
   to MCQ and complete one question; then open Translation. The previous totals
   should remain visible. Matching pairs and MCQ questions are different
   scoring units: do not interpret combined `earned/possible` as a uniform
   question accuracy score.
3. Click Continue twice rapidly on a completed Matching or MCQ question. The
   shared result should not be counted twice. A route/navigation error should
   be reported; complete publication acknowledgement is still future work.
4. Start an exercise but leave before submitting. Return to the exercise type.
   A fresh question should be created and the abandoned attempt should not
   increment mistakes or completed totals.
5. In Sentence Translation, enter the local LM Studio model and test both
   Chinese-to-English and English-to-Chinese. Start a question, change model or
   direction, and verify the old word bank disappears. Generate a new question
   and confirm it is accepted rather than blocked by an abandoned ID.
6. Submit a translation and navigate away while AI evaluation is pending.
   The delayed reply must not create an additional shared result. Watch the
   browser console for any rejection or resource-lifecycle errors.
7. Open Speaking practice; request microphone access with Record, check the
   animated vertical waveform, Stop, replay and Discard. Deny permission once
   and verify a recoverable error. No pronunciation score is awarded.
8. Click Finish session, then New session. Completed totals reset; stale
   callbacks from the previous session must not append results to the new one.

Capture the exact first compiler error or Chrome console output if any step
fails, along with the URL/route and the action that triggered it. The embedded
read-only dictionary predates the incomplete AI classification run; absence
of new AI-enriched sense classifications is expected.
