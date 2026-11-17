# Acceptance for approved product behavior

These are acceptance contracts, not recorded test passes. Source implementation must be assessed separately from actual Rust, WASM and Chrome execution.

## Practice outcomes and progression

- [ ] A mixed session displays Correct, Partially correct and Incorrect counts using accessible text and a stacked distribution without a universal percent.
- [ ] An exercise without partial scoring does not invent a partial verdict.
- [ ] Matching retains pair-specific attempts; handwriting retains rejected and assisted strokes without falsely awarding unassisted success.
- [ ] Session progress advances for completed slots; dragon acquisition responds only to persisted qualifying learning evidence.
- [ ] Skip replaces the same exercise type and slot without consuming the configured completion count.
- [ ] Multiple skips cannot spin forever or immediately repeat identical content where alternatives exist.
- [ ] Prior submitted attempts remain accounted for; skipped content produces no new mastery event.
- [ ] Resume restores the current session; new session reuses configuration and selects new eligible content without URL-encoded state.

## Vocabulary policies

- [ ] Pedagogical mode prioritizes rare recently failed words ahead of unseen entries and reduces priority after repeated successful evidence.
- [ ] Pedagogical mode off rotates less recently visited words without creating mastery evidence.
- [ ] Enabled multiple-choice formats are approximately balanced and do not override eligible vocabulary constraints.
- [ ] Excluding a standalone character entry does not exclude longer entries containing it.
- [ ] Global exact-entry exclusions apply across HSK and personal Word Set sources; there is no setup bypass.
- [ ] Word Sets can add, search and restore exclusions; Practice Setup displays the applicable excluded count.
- [ ] Global pool exhaustion preference is reused or finish early, defaults to reuse and is snapshotted per session.
- [ ] Zero eligible vocabulary stops generation with an informative error and never silently expands source filters.

## Profile portability

- [ ] Export JSON contains versioned configuration only, no learner history, service keys or exclusions.
- [ ] Import validates malformed, oversized and unsupported files before saving a new profile.
- [ ] Missing personal Word Set references require explicit resolution, not implicit fallback.

## Hanzi recognition and clipboard

- [ ] Draw, paste and file upload share one canvas on desktop and mobile.
- [ ] Paste works without persistent permission where browser allows and permission is not requested automatically on mount.
- [ ] Focus and paste listeners are cleaned up; old recognition responses cannot overwrite newer content.
- [ ] No clipboard image is uploaded without pressing Recognize.
- [ ] Single-character and multi-character candidates are validated against the dictionary; uncertain matches are clearly labelled.
- [ ] Clearing or navigating discards input and results; no recognition history is written to browser storage.
- [ ] Unsupported or absent vision services disable recognition with actionable explanation.

## Interface and platform

- [ ] System theme follows OS changes until an explicit persistent Light or Dark selection is made.
- [ ] Database initialization failure blocks normal navigation, supports retry and preserves local data.
- [ ] Genuine empty dictionary results remain accessible as empty results.
- [ ] Public metadata includes canonical URLs and appropriate Open Graph fields, without private learner state.
- [ ] Checked-in Docs index, image links and Markdown pass documentation validation, and the Docs reader works at `/` and `/zipeek/`.

## Sentence Relay

- [ ] Setup, explicit Start, preparation with cancel, gameplay and results are visibly distinct; generation does not start implicitly while choosing settings.
- [ ] The configured sentence count controls generation and completion; shortages produce clear feedback rather than a silent count change.
- [ ] Punctuation is not a selectable word-bank token; appropriate Chinese punctuation is rendered and validated.
- [ ] Clicking, dragging, in-answer reordering and equivalent keyboard operations work; selected tokens leave the word bank and return after Undo.
- [ ] Optional pronunciation plays on token selection or drop, reuses shared audio, and does not overlap or fire repeatedly while dragging.
- [ ] Incorrect submitted word order highlights a meaningful misplaced token or phrase while preserving a fully editable answer.
- [ ] For 他 在 吃 饭 正, a reliable validator identifies the displaced 正 rather than incorrectly highlighting all subsequent words.
- [ ] Missing, extra and multiply misplaced tokens receive contextual, progressively helpful feedback; uncertain grammar is never explained with invented certainty.
- [ ] Valid alternate Chinese word orders are not rejected solely for differing from one expected string.
- [ ] Accessible non-color error indicators, announcements, mistake counts per submission and a compact, distinguishable purple completion bar are verified.
- [ ] The setup settings icon exposes an accessible name; leaving an in-progress game warns before discarding progress.

## Application-shell and screenshot-reviewed UX

- [ ] Footer attribution is centered in the main content area and does not overlap floating reminders at narrow widths.
- [ ] Practice reminder shows concise icon actions, omits duplicate progress, preserves sessions on dismiss and synchronizes across local updates and storage changes.
- [ ] A second tab has a full-screen, non-dismissible blocking state while the original retains exclusive write ownership.
- [ ] A closed or crashed original tab cannot leave permanent stale ownership; at most one waiting tab acquires ownership.
- [ ] Word Sets removes redundant explanatory copy and offers a concise Practice this set action that preselects the vocabulary in Practice Setup.
- [ ] Write Hanzi suggestion-source labels truthfully distinguish a selected empty set from broader dictionary suggestions.
- [ ] Third-party Credits identify verified resources, applicable licenses and separate inspiration acknowledgements without implying endorsement.

## Word mosaics feedback

- [ ] A first tile selection remains visibly identifiable and screen-reader detectable until the second selection or cancellation; keyboard and touch selection behave consistently.
- [ ] The two attempted tiles remain identifiable during evaluation, including when the pair is incorrect.
- [ ] Correct pairs receive visible green and textual confirmation, then are removed or locked, with matched-pair progress incremented exactly once.
- [ ] Incorrect pairs receive visible red and textual feedback, then automatically deselect without removing tiles; each submitted wrong pair increments mistakes once.
- [ ] Selection, correct and incorrect states remain understandable without color, with reduced motion enabled and with a screen reader.
- [ ] Repeated wrong matches progressively reveal truthful semantic hints and, when reliable, candidate emphasis; the first error does not reveal the full solution.
- [ ] Earlier mistakes and assistance remain in the outcome evidence even if a pair is subsequently completed; no incorrect attempt is overwritten as independent mastery.
- [ ] Rapid clicks, interrupted feedback, navigation and session resume do not create stale selections, duplicate scoring or stuck feedback.
- [ ] Active exercise header does not repeat the exercise name; no redundant Exit practice action appears inside the Word mosaics card.
- [ ] Browser validation checks the accessible, responsive match grid and verifies matched-pair counts at completion.

## Shared Practice layout and header

- [ ] Every active exercise type uses a common session-position and feedback hierarchy without repeating its exercise-type name as a card header.
- [ ] Leaving uses one top-right control that preserves resumable state; no duplicate bottom-card Exit practice button appears.
- [ ] Session progress, exercise-specific progress and dragon acquisition evidence are independently understandable, persistent where required, and never conflated.
- [ ] Shared actions, focus handling, announcement behavior and error states remain consistent across Matching, Multiple Choice, Listening, Character builder, Word mosaics and Write Hanzi.
- [ ] The global header has no redundant centered page title; the long detailed dragon itself is the progress shape with no labels above it. Visual comparison, keyboard detail panel, evidence-backed fill and reduced-motion behavior pass on desktop and mobile.

## Practice vocabulary counting and setup

- [ ] Exercise selection offers accessible Select all and Clear all; only available exercises may be selected; configuration persists on temporary deselection.
- [ ] Short exclusive options, descriptive radio options and independent checkboxes follow shared patterns. An exercise without extra configuration has a truthful no-settings state.
- [ ] Eligibility calculations never monopolize the main browser thread; show measured or indeterminate progress and keep controls responsive.
- [ ] Filter changes invalidate older results; cancelling, navigating away or changing exclusions cannot cause stale counts or stuck loading UI.
- [ ] Zero eligible, failed count and exhausted-but-valid pool are different states, and only a validated nonempty pool can start a session.

## Multiple Choice pronunciation

- [ ] Pinyin defaults to Always for non-revealing displays; meaning-to-Pinyin questions do not leak the answer.
- [ ] Automatic question audio is enabled by default only where a Chinese target exists; blocked autoplay remains recoverable, and manual replay is available.
- [ ] Pinyin and audio settings survive saved profile and session round trips and preserve keyboard accessibility.

## Credits and legal notices

- [ ] The Credits page distinguishes bundled resources and applicable notices from design inspirations, without implying endorsement.
- [ ] Referenced license files and attribution statements match what is actually shipped.
