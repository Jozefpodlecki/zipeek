## R105 follow-up — connect domain coordinator to browser routes

- **P0:** Run `cargo test -p zipeek-exercises --locked`, `cargo test --workspace --locked`, and `cargo check -p zipeek --target wasm32-unknown-unknown --locked` on Windows; resolve any native diagnostics before enabling new UI.
- **P1:** Add an explicit planned-session launch identity to the shared provider. Keep the legacy free-play session distinct; do not infer planned results by exercise type or arrival order alone.
- **P1:** Pass the planned `instance_id` through Matching/MCQ and Translation, publish the final scored result once, acknowledge only after accepting the ledger write, and navigate to the next exercise from the authoritative plan cursor.
- **P1:** Implement finish/abandon and refresh behaviour with a documented recovery policy. Enable unified Start Session in Summary only after multi-exercise browser E2E proves successful completion and replay safety.
- **P2:** Add UI regression tests for stale callbacks, duplicate submissions and navigation away during pending acknowledgement.

## R104 follow-up — Windows validation and unified Practice session

- **P0 Windows build:** Verify corrected Practice Summary delimiters and full Rust/WASM build; send the first diagnostic if any remain.
- **P1 Unified session:** Wire the fixed `SessionPlan` to authoritative result submission/acknowledgement, route progression, and final scored summary before enabling one Start Session action.
- **P1 Practice QA:** Test General Fixed/Endless and 5/10/20 counts against generated URL query and Summary; verify deterministic replay unaffected; test planner preview with selected Matching/MCQ/Translation and non-scored Speaking.
- **P2:** Save profiles and resume policy; translation direction/word-bank implementation; accessibility/focus/scroll testing.
- **Deferred:** Source-only database installation-fixture failures and the earlier approved row-count assertion discrepancy.

## R103 follow-up — authoritative Practice session settings

- **P0 Windows:** Compile/test `zipeek-exercises`, web WASM, then verify Matching/MCQ variants independently; deselect/reselect and confirm settings are retained, Summary matches launched variants, and Translation count 5/10/Endless is carried into its route.
- **P1:** Move the shared Fixed/Endless round mode and count into `PracticeDraft` as launch authority; reconcile existing reducer/URL normalization without changing deterministic legacy replay. Expose and validate Translation direction/word-bank choices only when the route honours them.
- **P1:** Connect the R99 planner to authoritative ledger acceptance and exercise navigation before enabling unified Start Session. Persist profiles only after draft validation and recovery policy.
- **Deferred:** Source-only installation-fixture failures and earlier agreed legacy row-count mismatch.

# Practice session builder — approved four-step navigation

Approved by the user on 2026-11-04 after discussion of the original Practice mockups.
This document supersedes the single long Practice setup layout in `PRACTICE_SETUP_V2.md` where they disagree. It is the UX target, not evidence that every feature has shipped.

## Navigation model

One persistent top-level tab strip, in this order:

1. **Exercise Types** — select any number of available exercise definitions using a responsive grid. The library must scale beyond five: Drawing Hanzi, image-to-Hanzi, and future listening/writing exercises must not need a redesign. Use compact cards, a discoverable expanded library when needed, and disabled states for nonfunctional exercises. Exercise selection never changes the active tab, top-level or nested.
2. **Vocabulary** — exclusive Dictionary/HSK or personal Word Set source shared across the session. For Dictionary, choose **HSK 2.0 / HSK 3.0 / Mixed** first, then show only the corresponding levels. Preserve explicit multiple level selection in Mixed mode; do not conflate HSK 2.0 and 3.0. Show category autocomplete/removable chips, optional PoS and accurate source/exclusion/variant-aware vocabulary preflight. Switching sources must never silently combine filters with a Word Set.
3. **Configure** — second-level tabs: **General** and tabs for selected exercises with configuration. Dynamic and horizontal-scrollable on narrow viewports, never wrap the page into a long set of forms. General covers session-wide options; per-exercise tabs own that exercise's options. Selecting/deselecting an exercise must not automatically switch nested tabs. If the selected nested tab disappears, return to General. Preserve the unsaved draft settings for a deselected exercise until the user explicitly saves/discards the profile.
4. **Summary** — authoritative review of chosen exercises, vocabulary source, exact selected HSK levels and categories, eligibility for each selected exercise, length, and any unsupported/unscored exercises. Show actual verified counts, not invented estimates. Start Session is enabled only after compatible planning, validation, publish/acknowledge/advance, and navigation are all integrated. Save Profile persists the validated complete draft only once its format/recovery policy exists. No fake action buttons.

Navigation is available by clicking top-level tabs and by Back/Continue controls; all steps share one draft and preserve values across visits. Hide unrelated configuration surfaces so ordinary sessions do not require excessive vertical scrolling. Use keyboard-accessible tab controls and visible focus states. Small screens stack or scroll content appropriately without clipping actions.

## Phased implementation

- **R100 first slice:** render the four top-level tabs, isolate existing exercise selection and source picker, put existing conditional settings under Configure and truthful separate launch actions in Summary. Remove implicit settings-tab changes when clicking exercise cards. Add HSK 2.0/3.0/Mixed version-first level grouping; selecting one standard drops the other standard's selected levels and defaults to its level 1 when empty. Retain existing launch/replay contracts. No schema or dependencies changed.
- **R101:** Configure now renders General plus selected Matching, Multiple Choice, Translation and Speaking tabs in a horizontally scrollable row. Matching and MCQ display only their respective selectable question variants; General retains the existing mode, round-count and exclusion settings. Translation retains its existing sentence-count setting and separate launch; Speaking remains an unscored preview. Deselection of the active exercise falls back to General without clearing other draft settings. Shared session scheduling, per-exercise independent format persistence, and full browser accessibility validation remain open.
- **Next:** complete the per-exercise configuration models and summary eligibility messaging. Improve keyboard focus management and verify responsive height/scroll behaviour.
- **Following:** connect R99 planner to authoritative result ledger, safe acknowledgement and cross-route navigation, then activate a single Start Session; implement saved profiles only with explicit recovery policy. Do not claim these are shipped before tests.

## Exercise expansion

Drawing Hanzi requires stroke datasets and handwriting assessment; image recognition requires source/licensing and sense-level image mappings. Neither is enabled merely by adding an exercise card. Do not introduce unverified scoring or image provenance claims.

### R102 implementation note — selected family readiness

Enabling Matching or Multiple Choice must initialize that family's question-format selection if empty, without replacing the other family's choices. Deselecting and reselecting an exercise retains its configuration. Summary must expose each selected family's actual formats and warn if a selected family has none. The combined legacy Matching/MCQ launch requires every selected family to have at least one configured format; another family's format is not a substitute. This is not yet a single unified multi-exercise session.

### R103 implementation note — authoritative exercise options

The typed `PracticeDraft` now owns Matching and Multiple Choice format selections separately, plus the Sentence Translation count (5, 10, or 0 for Endless). The selected formats used by Summary, playable-pool preflight, and the legacy Matching/MCQ launch are derived from the draft, not a second configuration reducer. Selections persist through deselection and re-selection of an exercise; a selected family with no formats fails validation, while an unselected family's retained formats do not launch. Translation count is read from the draft for both Summary and Translation route parameters. The setup reducer no longer stores exercise-format selections.

**Not yet consolidated:** shared Fixed/Endless Matching/MCQ session mode and round count, which remain in the setup reducer; the draft's existing `length` field is not yet the launch authority. Translation direction and word-bank options are typed in the draft but not exposed as functional controls in the current Translate route. Keep these differences visible until they are reconciled and validated in Windows. Saved profiles and the unified Start Session remain gated on a complete authoritative model and planner/ledger integration.

## R103 — Practice exercise settings owned by draft (Windows verification pending)

- `PracticeDraft` now owns Matching and MCQ question-format selections independently and the Translation count (5/10/Endless). Its validation rejects selected families without formats, cross-family format contamination and unsupported Translation counts.
- Configure, Summary, pool preflight, Matching/MCQ launch query, and Translation route parameters read these settings from the same draft. The setup reducer no longer stores exercise formats; legacy session mode/round count and HSK level still live there pending a subsequent consolidation. Existing legacy URL/replay encoding and round builders are unchanged.
- Node tests 136/140 passed; four database workflow installation tests require generated bundle fixtures intentionally omitted from source-only archive. Rust/WASM and browser checks remain pending. No schema or dependency changes.
