# Original R524 issue register - preserved snapshot

## Evidence and status

This register records the user's observed behavior and requested changes from the 2026-11-17 screenshots and message. Every issue is OPEN and unverified in source or live browser unless a later test result explicitly closes it. It is not an implementation or acceptance claim. The current project build was previously reported failing, with no authoritative first error supplied. All gameplay changes need separate scope and testing.

## P0 - Broken execution and user facing failures

- [ ] **Translate Sentence**: translation does not complete, with repeated localhost connection-refused requests and an unknown translator action warning in browser console. Capture the failing request, intended backend and first non-network exception. Distinguish service unavailable from application logic.
- [ ] **China Map Challenge**: map asset load reports `Map asset has an invalid schema: expected value at line 1 column 1`. Check the response status, MIME type, actual body and configured asset path under `/` and `/zipeek/`; confirm asset generator output before changing a schema.
- [ ] **Build health**: obtain exact current failing build command, first error and toolchain; do not assume current sources compile.
- [ ] **Onboarding / pedagogical mode**: enabling pedagogy shows an unexpected icon instead of dragon progress in the top bar. Clarify the intended compact progress dimensions and source of progress data.

## P1 - Practice flow and grammar exercises

- [ ] Practice Path: remove or repurpose empty panel; improve muted guidance text and saved-profile empty state; maintain concise actionable hierarchy.
- [ ] Practice Exercise availability: decide whether exercises requiring disconnected services should be disabled, omitted, or selectable with an explicit unavailable state. Require reliable service capability detection.
- [ ] Sentence Transformation setup: remove duplicate title; replace native grammar-pattern suggestion experience with a controlled autocomplete of up to five matches while typing; show human-friendly pattern names with link to Grammar Course; support multiple allowed patterns, but generate a question containing one selected pattern.
- [ ] Sentence Transformation answer method: replace stacked radios with a horizontal, accessible segmented group, responsive on narrow screens.
- [ ] Sentence Transformation session: remove redundant `Select Chinese tiles below` caption.
- [ ] Pronunciation Practice label: shorten to `Pronunciation`. Configure word-only versus sentence reading; sentence generation depends on a known, available AI provider.
- [ ] Speech-to-text verification: design a separate speaking exercise with microphone permission and transcript checking; distinguish recognition confidence, expected text and unscored mode. Decide on local versus service recognition.
- [ ] Audio Dictation: redesign to generate/play a Chinese sentence and reconstruct it from a word bank; define hints, audio replay, source disclosure and whether AI is mandatory.
- [ ] Spot the Mistake: AI-generated flawed sentence, configurable word-bank or typed correction; validate that an unambiguous correction exists.

## P1 - Vocabulary, discovery and navigation

- [ ] Write Hanzi > My Word Sets: list needs scroll containment or pagination with accessible item selection.
- [ ] Word Sets: fix vocabulary-pane overflow / missing vertical scroll; use otherwise-empty right-hand space responsibly; improve grouping and management controls.
- [ ] Word Sets: replace inline `Word Set created` feedback with accessible non-blocking toast / notification.
- [ ] Word Sets Data tools: clarify backup/restore as a separate drawer/dialog rather than multiple nested panels.
- [ ] Browse: deduplicate visible meanings and `Explore meanings` action; define when an expansion is useful.
- [ ] Browse: first page repeatedly displays same words; select five varied database words per refresh or navigation with an explicit repeat/seed policy.
- [ ] Random > Sentences: improve layout; request at least five generated sentences per action, larger Hanzi, per-character or per-word pinyin, playback control, and icon-based regenerate action. Move AI provenance note to an information affordance.

## P1 - Courses, evidence and progression

- [ ] Pedagogical mode: user expects course content gated until acquired; define whether completely hidden or visible as locked placeholders. Avoid claiming unknown abilities are mastered.
- [ ] Grammar Course: generated examples need playable Mandarin audio, compact regenerate icon, no redundant `Correct. You can continue.` label, configurable Hanzi font size.
- [ ] Grammar revelation hypothesis: only evidenced successful Translate Sentence, Sentence Builder and Sentence Transformation exercises may contribute to course tile opacity. Proposed rule: ten successful translations every day over seven days, matched to an identified grammar structure. Clarify consecutive days, scoring integrity, repetitions and partial opacity before implementing.

## P2 - Games and presentation

- [ ] Icon Pairs: configurable on-click audio, show-pinyin and Hanzi font size; remove unnecessary mastery disclaimer while preserving honest evidence semantics.
- [ ] Icon Pairs: HSK 6 reports too few distinct pairs. Measure catalog coverage and define fallback or disabled selection instead of an unexplained error.
- [ ] Classifier Market: redesign progress bar and game feedback, add optional pronunciation, and audit classifier coverage against source data. Avoid unsubstantiated `all classifiers` claims.
- [ ] Radical Forge: improve layout, optional pronunciation, audit covered component/radical inventory; distinguish Kangxi radicals from broader character components.

## Cross-cutting acceptance

- [ ] Screenshots and local observations are user-supplied evidence, not a passing browser test. Verify every fix at desktop/mobile sizes with keyboard focus, accessibility names and network unavailable states.
- [ ] Avoid empty surfaces, duplicated headings, inline low-contrast instructions, inaccessible custom controls and layout shifting when feedback appears.
- [ ] Use only the ordinary hyphen character for dash-style punctuation in maintained documentation and user-visible copy.
- [ ] Preserve separation between study hints, AI-authored examples, trustworthy mastery evidence and visually unlocked course content.

## R524 triage update

The product choices for AI availability, locked course visibility, rolling grammar evidence and dual speech recognition are approved as D53-D56. Their implementations and browser acceptance remain open. The translator click routing and service failure copy have a source-level correction candidate in R524. Province Quest now distinguishes HTML fallback, empty asset and malformed JSON, but the R523 source archive contains no generated Province Quest map JSON: map generation and installation remain required. Do not close the P0 incidents without a live build and browser check.


This is a preserved source snapshot. Track current issue statuses in the individual issue pages.
