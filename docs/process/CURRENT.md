# Current development checkpoint — R105 (2026-11-04)

**Status:** source checkpoint; Windows Rust, WASM, and browser validation outstanding.

## R105 changes — authoritative domain result coordination

- Added `zipeek-exercises::planned_session::PlannedSession`, which couples the deterministic `SessionPlan` and scored `QuizSession` ledger into one transactional domain state machine. It validates the planned instance ID and exercise type, publishes a scored result into both authorities, and rejects duplicate, stale, misrouted, or invalid submissions without changing either authority.
- Explicit acknowledgement is required before advancing to the next planned exercise. The final acknowledgement completes the ledger and planner together; totals remain available from the ledger.
- Added four native Rust tests covering acknowledgment gating, rejection atomicity, final completion and unsupported drafts, plus three focused Node source-contract tests.
- **Not yet implemented:** cross-route planned-instance handoff, UI Start Session, rehydration after refresh, and real result delivery/acknowledgment from Matching/MCQ or Translation. Existing exercise routes and replay paths are deliberately unaffected.
- Node suite: 142/146 passing; the same four source-only DB workflow fixture failures remain. Cargo/WASM compilation unavailable in this environment.

## Latest changes

- Repaired two missing Yew `html!` closing delimiters in Practice Summary that caused E0765-style parsing failure during the user's R103 wasm build (compiler reported an unclosed delimiter).
- Consolidated General Fixed/Endless mode and round count into the `PracticeDraft.length` field; the General controls, Summary, and legacy Matching/MCQ launch query now use that one authority. The existing URL format and `PracticeMode` values remain unchanged. Unsupported fixed counts are rejected by conversion rather than silently mapped to Endless.
- Added a read-only `SessionPlan::from_draft` preview to Summary for valid, fixed, scored selections. This is **not** a unified launch; result publication/acknowledgement and cross-route orchestration are still not wired.
- Added a source delimiter regression test and mode-contract tests. Node suite: 139/143 passing; four known installation tests need generated manifest fixtures absent from this source-only archive. Added Rust unit tests for draft-to-legacy mode conversion; they have not run.

## Next blockers

1. **P0:** Run `cargo test --workspace --locked` and `cargo check -p zipeek --target wasm32-unknown-unknown --locked` on Windows. Fix any additional native/wasm diagnostics before further feature work.
2. **P1:** Link `SessionPlan` to shared result ledger publish/acknowledge/advance and cross-route navigation. Enable unified Start Session only after a complete end-to-end scored flow exists.
3. **P1:** Validate the four-step Practice UI and browser navigation, and retain draft values across steps.
4. **P2:** Profile persistence and Translation direction/word-bank settings require explicit data/recovery semantics and functional route support.

No new database schema, dependencies, or generated DuckDB artifacts in R104.
