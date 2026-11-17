# Open tasks and verification

Use the current authoritative product contracts and [acceptance checklist](../testing-quality/acceptance/approved-behavior-acceptance.md). Checkboxes denote unverified work, not historical completion.

## E2E evidence verification

- [ ] Run `cargo check --locked -p zipeek-e2e --tests` and targeted Rust unit tests on a supported toolchain.
- [ ] Run the Word mosaics Chrome browser journey with FFmpeg and FFprobe available, and inspect the actual MP4, timestamped checkpoints, `agent-feedback.md`, `scenario.json`, and suite `review.md`.
- [ ] Run strict evidence capture using `ZIPEEK_E2E_VIDEO_STRICT=1` and `ZIPEEK_E2E_EVIDENCE_STRICT=1`; audit its real MP4, checkpoints and manifests with the offline evidence auditor. Authored offline integrity tests do not substitute for browser verification.
- [ ] Run the larger E2E suite with deterministic fixtures; distinguish browser, assertion, service, and recording failures.
- [ ] Execute the updated Docs reader journeys for all twelve sections, image decoding, search and deep links at `/` and `/zipeek/`. Source checks and route validation are authored but do not constitute a browser pass.
- [ ] Revisit the [E2E requirements matrix](../testing-quality/e2e/e2e-requirements-matrix.md) after each implementation. Do not upgrade a row to verified without actual passing browser evidence.
- [ ] Replace obsolete URL replay and generic feedback assertions after the corresponding approved product behavior is implemented. Never make old assertions pass by inventing application behavior.

## Product features pending implementation or browser acceptance

- [ ] Implement and verify the common Practice exercise component foundation, consistent session navigation, and typed outcome evidence.
- [ ] Verify Practice completion distribution, persistent sessions, Skip and Replace, dragon evidence and profile JSON portability.
- [ ] Implement and test global exact-entry exclusions, vocabulary visitation, and cancellable nonblocking eligibility counting.
- [ ] Implement and verify context-aware Sentence Relay feedback, token dragging, punctuation and setup stages.
- [ ] Implement and verify selected/correct/incorrect Word mosaics tile states, progressive hints and accessible announcements.
- [ ] Add Multiple Choice pinyin and pronunciation controls, including anti-answer-leak rules and browser audio acceptance.
- [ ] Investigate Write Hanzi animation interruptions and stale frame glitches with recorded browser scenarios.
- [ ] Implement AI Hanzi Recognition and validate permissions, cancellation, dictionary linking and privacy behavior.
- [ ] Verify original-tab ownership and duplicate-tab full-screen recovery, System theme, database startup blocking and Credits.
- [ ] Review the [open issue library](../issues/OPEN.md), preserving evidence and avoiding unverified closure.

## Engineering and deployment acceptance

- [x] Add an offline source/deploy asset integrity gate, covering manifest hashes, generated asset presence and a consistent preload filename, with fixture-based Node tests.
- [ ] Execute the strict deploy preflight against real generated dictionary, Worker and geography assets, then verify Rust/WASM compilation and Chrome deployment paths.

- [ ] Execute `cargo fmt --all -- --check`, `cargo test --workspace --locked`, WASM build and browser fixtures on the actual Windows environment.
- [ ] Verify dictionary and Province Quest asset generation on both local and hosted paths.
- [ ] Verify capability-aware unavailable AI exercises, grammar evidence, course locking and browser/local speech recognition.
- [ ] Review outstanding architecture proposals D28 (modular E2E organization), D29 (scoped state ownership and initial motion profile), and D30 (crate extraction boundaries) against current source. Do not change dependencies or extract crates without explicit approval.

The historical acceptance backlog remains preserved in the [legacy acceptance ledger](../archive-history/LEGACY_ACCEPTANCE_LEDGER.md) and [completed records](../archive-history/completed/). This active list intentionally contains no release-numbered task chronology.

## Documentation and source reconciliation still pending

- [x] Preserve and reconcile historical development-marker references from maintained technical articles. See [editorial disposition and preserved source evidence](editorial-reconciliation.md).
- [x] Audit the architecture, database generator, checked-in manifest and web runtime against maintained documentation. [Source accuracy audit](source-accuracy-audit.md) records corrected facts and open verification gates.
- [ ] Reconcile remaining technical assertions not sampled in the source accuracy audit against implementation and browser results.
- [ ] Verify Credits source-level licensing and generated asset provenance against shipped distributions before release.
- [ ] Compile the Docs reader after the documentation restructuring; check all section headings, deep links and issue images in real Chrome.
- [ ] Implement the approved shared exercise foundation, long detailed dragon and nonblocking vocabulary eligibility service; authored specifications are not implementation evidence.
- [ ] Link the remaining missing E2E scenarios to precise acceptance rows, then run strict video capture on the Windows/Chrome target.
