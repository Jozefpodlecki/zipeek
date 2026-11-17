# R451 UI and integration source audit

This is a source-level review against user-supplied Chrome screenshots, not a passed visual acceptance report. The inspected baseline was the full R450 working tree. Source evidence describes implementation, while the screenshots establish the user-observed defects.

| Area | Confirmed baseline finding | R451 action | Remaining validation |
| --- | --- | --- | --- |
| Grammar mosaic | Source illustration was a screenshot of an earlier Zipeek UI, visible through gaps | Replaced with detailed original autumn painting; versioned asset path and unpack/preflight checks | Verify screenshot contains no UI at all, real browser loading |
| Vocabulary mosaic | 50 regions had 98.5% cover, leaving rich garden invisible; only four have authored curricula | Use subdued theme-aware cover, mark authored regions, clarify unpublished selection | Visual test at 0% and responsiveness |
| Pinyin mosaic | Six enormous solid-blue regions covered faint artwork | Reduce cover, preserve all six lessons, improve page layout | Art is still low-detail; a replacement illustration must be approved/visually validated before adoption |
| Course layout | Full-viewport constraints in AppShell and track components stretched/clipped details | Remove Courses viewport lock and independent panel stretch; use natural page scrolling | Browser heights 600, 768, 900 and mobile |
| Grammar details | Repetitive messages about unavailable mastery dominated panels | Reduce noise; keep the real lesson player and explicit unpublished states | Verify interactive player |
| Practice setup | Exercise list scroll relied on desktop flex sizing; selection contrast under theme backgrounds was weak | Explicit max-height scroll area, strong selected card/path/format styling and theme-aware action tones | Actual pointer/keyboard interaction across presets |
| Practice session | Dedicated shell exists in baseline but user screenshot showed global chrome and stale Matching copy | Preserve dedicated branch, make it self-contained, add R451 marker; add in-session step progress | Establish which bundle Chrome serves; run shell E2E |
| Matching | Selected and matched states had UI labels but failed user-visible acceptance; MCQ lacked a selected style | Larger standalone quiz card, clearer selected MCQ controls, consistent state feedback and progress | Verify actual pairs by clicks and keyboard; partial answers remain in local component state |
| Browse | Headword undersized, flexible row columns produced poor alignment, meaning controls overly spaced | Prominent Hanzi and compact two-column rows with responsive fallback; compact meaning control | Long meanings and mixed scripts in Chrome |
| Home/onboarding | Theme selection preview and card spacing were implemented but different screens used fixed blue actions | Theme-specific primary and hover palette; consistent spotlight controls | Confirm preview rollback, storage and wallpaper provisioning |
| Pinyin lesson | Example presentation compact and mixed script label assigned to entire mixed-language string | Better card typography and responsive spacing | Pronunciation pedagogy and audio assessment not in this UI batch |
| Testing | Existing browser tests still inspect obsolete direct Practice URLs; previous course opacity assertions stale | Updated focused tests and authored visual workflows; retained legacy scenarios in task register | Large migration still required for pre-R444 test suites |

## Source and build blockers

The full source contains `static/courses` illustrations, but `crates/zipeek-web/public` and `vendor` are not included. These are explicitly prepared by repository-owned commands and remain out of a source archive; do not introduce generated/vendor binaries into the source by hand. A fresh environment requires explicit `cargo make setup` and asset verification before Trunk.

The provided environment lacks `cargo`, `rustfmt`, the WASM target, a running frontend and a Chrome harness. No successful compilation, dynamic testing, screenshots or browser acceptance can be claimed. Archive integrity only establishes that the source is distributable, not that the changes compile or behave correctly.

The app retains no fake course mastery from region selection, no manual Pinyin completion, only two Practice routes, and no backward compatibility aliases. Full per-exercise answer-state persistence and verified pedagogical evidence remain separate future work.
