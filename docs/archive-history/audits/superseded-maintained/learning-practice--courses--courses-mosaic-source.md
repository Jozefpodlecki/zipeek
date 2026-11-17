# Archived source snapshot: learning-practice/courses/courses-mosaic.md

This source snapshot preserves historical descriptions, not current requirements. Use the maintained document at its original location for approved behavior.

# Courses: independent learning tracks and mosaics

**Current direction (2026-11-15).** This document supersedes earlier combined Grammar/Vocabulary geometry and the R439 inline Grammar player. Older dated release and decision records are historical evidence only; they do not override an explicit current user requirement. Implemented versus pending behavior is distinguished below.

## Track boundaries

Courses has three distinct tracks selected from the course overview: **Vocabulary**, **Grammar**, and **Pinyin**. Each has its own illustration, its own region geometry, its own authored course content and its own progression semantics.

- **Grammar:** exactly 100 grammar-only selectable regions in the present implementation. There are **no vocabulary regions, vocabulary pockets, vocabulary cutouts or vocabulary overlays** in the Grammar mosaic. The first authored course is selected by default and occupies the central region. R488 has authored teaching material in all 100 regions, totaling 550 pattern/example entries (99 existing and 451 newly authored). All new material is draft and requires Mandarin-language editorial assessment and separately verified HSK mapping. The entry count does not demonstrate independent grammar mastery or claim exactly 550 distinct linguistic constructions.
- **Vocabulary:** 100 vocabulary-only regions in its own mosaic. The source catalogue is owned by `zipeek-pedagogy::curriculum` and rendered by `zipeek-web`. All 100 regions are assigned named domains, themes or language-function categories. The deterministic glossary-assisted catalogue covers all 11,494 pinned HSK source records, 11,470 simplified spellings and 12,188 distinct readings, including old-only and HSK 3.0 7-9 entries. Automatic classification is not equivalent to human semantic review. The machine-generated review queue explicitly identifies uncertain memberships. No vocabulary mastery is inferred from mere selection or Word Set acquisition.
- **Pinyin:** six independently selectable mosaic stages containing twelve authored modules in total. Pinyin regions do not include Grammar or Vocabulary cells. Each stage exposes two modules and their locally saved written-check state; completing written checks does not certify spoken pronunciation.

Visual consistency does not imply shared topology. The tracks may reuse primitives such as SVG selection, region borders, keyboard handling and accessible controls, but their region data, coverage and learning progress stay separate.

## Vocabulary curriculum ownership, acquisition and review

`zipeek-pedagogy::curriculum` owns the generated 100-region catalogue. The removed historical vocabulary generator previously consumed the pinned `hsk-complete-a66fd30.json` and emits the checked-in Rust curriculum, `docs/learning-practice/VOCABULARY_COVERAGE.json` and `docs/learning-practice/VOCABULARY_REVIEW_QUEUE.csv`. The pinned HSK source SHA-256 is attached to the Rust catalogue and is subject to Rust catalogue invariants and editorial review. Every individual pinned source index has exactly one region membership; 24 extra source records share a simplified spelling with another source record. The catalogue groups those records together rather than silently discarding them.

Each region has a domain and a titled theme or word-class/function grouping, a deduplicated simplified-word membership, HSK band counts and stable pinned source-entry indices. Classification currently uses English source gloss keyword matching, then a documented part-of-speech fallback. **Source coverage is complete, but lexical-semantic review is not:** the review queue lists assignments that could not be matched confidently to domain keywords. Do not call the taxonomy manually curated or claim perfect meaning-level homograph resolution. Dictionary acquisition now uses the explicit (simplified, numeric-pinyin) keys for all 12,188 distinct source readings, rather than collapsing same-spelling variants. Runtime verification against the installed dictionary and expert semantic review remain open.

`zipeek-web` loads region words using chunked, parameterized dictionary queries and keeps the complete resolution strict. A missing dictionary reading (Hanzi plus numeric pinyin) renders a retryable error rather than saving a partial Word Set. The details panel shows selected-region metadata and acquisition, while a separate paginated Vocabulary Explorer displays a bounded selection of resolved words below the mosaic. Each region offers one or more independently acquirable Word Sets, grouped into batches of at most 200 dictionary words. The 100 regions currently correspond to 132 Word Sets. Acquisition snapshots dictionary IDs in browser-local storage and remains distinct from verified proficiency. The browser storage limit now allows 250 Word Sets, leaving space for personal sets as well as the curriculum, with no storage-format change.

`zipeek-exercises` retains exercise construction and scoring. `zipeek-db` resolves installed dictionary words and HSK fields, and neither authored curriculum nor personal Word Sets are moved into a service. The previous Python source generators and audits were removed at user request in R493. Future taxonomy edits require a Rust-native replacement for regeneration and source membership auditing before acceptance. Source coverage tests do not replace dictionary generation, Word Set runtime verification or expert linguistic review.

## Vocabulary mosaic sizing

Vocabulary uses 100 independently selectable, scattered, irregular Voronoi regions, with stable region IDs and restrained quadratic curves along shared edges. The sites originate from a deterministic dispersed weighted partition using each region's HSK Hanzi-plus-Pinyin reading count. Square-root weighting and a minimum-area floor soften extremes; a final nearest-site polygon tessellation replaces rectangular borders with natural irregular boundaries. For the current pinned catalogue, all resulting polygons have a bounding-box width and height of at least 36 SVG units, and their combined area covers the full illustration. Geometry is cached between selections. Region size is a visual cue, not a precise quantitative chart.

The details panel leads with the theme title and description, reading and Word Set counts, acquisition progress and a single acquire-next-set action. The selected-region panel contains one primary acquire-next action and source-backed acquisition progress. It does not duplicate full vocabulary lists or HSK breakdowns below the map. The desktop Vocabulary view targets a viewport-bounded layout without an extra footer or a second vertical scroll, subject to live browser acceptance; on shorter mobile screens the page may scroll for accessibility. It must not display region index headings or implementation explanations such as how tile sizes are calculated. Region identity and counts remain accessible through path labels. Touch-target usability on narrow viewports is still subject to browser acceptance, since an SVG minimum size does not guarantee 44 CSS pixels at mobile widths. Mosaic size represents curriculum membership, not acquisition, proficiency or reveal progress. Grammar and Pinyin use their own independent geometry.

## Region appearance and interaction

- Regions tessellate a continuous underlying illustration. Grammar uses its own 100 balanced irregular Voronoi regions with gently curved, shared boundaries, **without gaps, vocabulary cells or small pockets**. Avoid a rigid or visibly fragmented look.
- Incomplete Grammar regions retain a translucent dark cover so learners can recognize the illustration. The cover is a purely visual preview, not progress or mastery. Selecting a region changes its outline and preview opacity, not its completion state.
- Revealing unobstructed Grammar artwork requires independently verified completion evidence. Until that evidence is connected to the viewer, no region is presented as completed. Do not manufacture completion values.
- Selection is accessible by pointer, Enter and Space. Use a distinct visible selection stroke, focus styling, descriptive labels and reduced-motion-safe transitions.
- Grammar details show one meaningful primary action, **Start course**, leading to the full dedicated lesson. Do not embed a second Play lesson player, duplicate lesson text, or add an additional full-lesson action.
- Invalid/out-of-range region identifiers must not open lesson content. The current 100 regions have teaching content; unavailable content must not be represented as published.

Neither the Vocabulary nor Grammar SVG region paths contain icons. Selection is identified through outlines, without icon overlays. Both tracks use shared geometric helpers, but preserve independent region membership, arrangement and semantics.

## Grammar architecture (R492)

Teaching content for all 100 regions now resides in `zipeek-pedagogy::grammar`, generated from the unchanged editorial TSV where applicable. The pure exercise progression and answer gate reside in `zipeek-exercises::grammar_learning`. Original and expanded lesson paths use a shared lesson scaffold for navigation, progress and title, but their practice/assessment activity formats remain distinct and unverified. Course IDs, URLs and browser checkpoint values are unchanged. No guided activity grants mastery or reveals the mosaic.

## Grammar guided-practice presentation (R491)

The 95 expanded-topic courses (regions 6-100) present the same authored material in two stages per pattern: teaching and guided practice. Guided practice starts with the English meaning while the authored Chinese sentence is hidden; the learner first tries to recall the sentence, then explicitly reveals the author-written example, and only then answers a meaning-recognition choice drawn from that course's examples. The reveal action does not score the learner. Recognition remains assisted practice because the answer has already appeared in teaching and in the revealed example. It does not generate independent mastery evidence or reveal the mosaic.

The reveal and answer are managed in a single local reducer, which rejects recognition selections before reveal and rejects out-of-range options. Stage count remains `2 × pattern count` and saved browser checkpoint data is unchanged. This improvement is **not a replacement for additional Mandarin-reviewed usage restrictions, tone-marked Pinyin, counterexamples, fresh unseen assessment items or syllabus mapping**. The first five authored courses retain their existing guided exercise flows.

## Grammar lesson URL structure (R488)

`/courses/grammar` is the Grammar overview. The five original authored courses use `/courses/grammar/:slug` with the established `intro`, `time-place`, `possession-quantity`, `descriptions` and `wants-abilities` paths. The additional authored regions 6-100 retain `/courses/grammar/topic/:region` and scored draft assessments use `/courses/grammar/assessment/:course`. Unknown lesson slugs do not produce lesson content. Published route IDs are defined once in the curriculum module. The route and component key isolate lesson-local state when switching courses.

## Focused learning workspace

Dedicated Grammar teaching and assessments and Pinyin lesson routes do not render the global application drawer, top header or footer. Use a focused layout with an explicit exit link and in-lesson progress/navigation. The Courses overview and its track mosaics retain the regular application shell.

Grammar teaching presents one stage at a time instead of a long scrolling list: authored grammar-point explanations, guided comprehension, guided sentence construction/transformation, then the existing independent assessment. Previous and Continue must let learners revisit authored content; answered guided questions provide rationales and require a response before proceeding. Guided feedback or simply reaching the last stage is **not** evidence of mastery.

The staged UI introduced here holds its step in component state: **resumable progress and persistent per-point mastery are not yet implemented or verified**. These require an independently reviewed pedagogical data contract, assessment evidence and browser-local storage lifecycle. Do not claim course completion or mosaic reveal based on teaching-stage navigation.

## Art direction and assets

Keep detailed Chinese-themed artwork distinct per track. The Pinyin mosaic now uses a brighter landscape illustration with recognizable structures and multiple local features to remain legible beneath its covers. Authored assets use stable names, not release-number suffixes: `grammar-autumn-bridge.png`, `vocabulary-peony-garden.png`, and `pinyin-landscape.png` in `crates/zipeek-web/static/courses/`. Practice creature images in `static/practice/` retain their existing descriptive `.webp` names without release suffixes.

## Packaging contract

Both `static/courses/` and `static/practice/` are authored, managed directories in the source archive. Pack includes their files; unpack restores the archive's complete authored contents and removes stale renamed images, rather than overlaying individual files indefinitely. `static/brand/` is also managed. Generated dictionary, Hanzi-stroke and vendor assets remain protected from source archive replacement.

## Verification and remaining acceptance

The source edits are not equivalent to a compiled WASM build or browser acceptance. Run locked CLI unpack tests, locked WASM compilation, the impacted course interaction E2E suites and full browser visual checks at `/` and `/zipeek/`. The E2E expectations for the retired inline player must be updated to the single entry workflow. Test all three mosaics, responsive and keyboard states, complete asset loading, short and multi-set Vocabulary regions, persistence of acquisition, and 0%/partial/verified-complete progress once the corresponding evidence model exists. The current Grammar viewer does not implement durable evidence-driven reveals; that remains an explicit open implementation gap.

## Course Word Set source identity

A personal Word Set acquired from a Vocabulary region stores an optional, typed `source` object containing the one-based `region_id` (1-100) and `set_index` (1-based within the region). Source identities must be unique among saved Word Sets. Display name changes and edits to saved word contents do not remove course acquisition identity; deleting a set removes its acquisition. Personal sets created without a course source remain valid. Older personal sets without source metadata are not inferred to be acquired by matching a possibly edited title. A learner may need to re-acquire a course set after updating from an older build, or rename an older, name-colliding personal set before acquisition. No proficiency or mastery evidence is inferred.

The selected-region card is the only place for region metadata and acquisition controls. The mosaic area has a paginated, read-only Vocabulary Explorer beneath it showing dictionary words, pinyin, meaning and pronunciation without duplicating the selected-region acquisition data, and the standard footer is omitted on the Vocabulary route at desktop viewport sizes. Keep the compact view readable at 320-390px mobile widths by permitting normal document scrolling instead of clipping essential actions.

## Vocabulary Explorer and acquisition integrity (R477 WIP)

The left column combines a 100-region mosaic with a compact Vocabulary Explorer. The Explorer uses the already resolved dictionary words for the current region, renders at most four cards per page and provides Previous/Next controls and existing zh-CN Listen buttons. It does not repeat the right panel's theme, HSK totals, Word Set acquisition progress or buttons. Desktop reserves the available height for the two left surfaces and right detail panel without a document-level vertical scroll; narrow viewports retain natural scrolling. Live browser layout and accessibility acceptance are still pending.

The page is composed from separate mosaic, details, explorer and data-resolution modules. Loaded dictionary results carry the originating region index so previous async responses cannot be mistaken for the newly selected region; the effect retains cancellation. Course Word Set source IDs are checked against the current curriculum's real region IDs and valid per-region chunk indexes. Course creation validates on a candidate collection and commits only after success. Edited personal Word Set membership is intentionally not treated as proof of original curriculum membership; acquisition is source-associated, not mastery. Import source identity is metadata rather than cryptographic provenance.

## R478 Vocabulary feature ownership and state transitions

The Vocabulary page owns a reducer for selected region, request generation, loading/error/ready state and Explorer page. A region change or retry clears displayed results and resets pagination; an asynchronous response is accepted only when both its region and request generation match. The page keeps a bounded cache of up to four resolved regions; cached words are shared by the selected-region acquisition panel and the four-card Explorer, without duplicate dictionary queries. The cached results do not contain personal Word Set state.

The right panel is the only acquisition surface. Its success/error feedback is associated with the acquired `(region_id, set_index)` source so messages from unrelated regions are not shown there. The Explorer contains Hanzi, Pinyin, meaning, pronunciation and page controls only. Desktop fits both left surfaces and the details panel into the available course viewport; the illustration is contained rather than cropped when the mosaic is height-limited. Narrow viewports retain natural page scrolling. Desktop and mobile layout requirements still require browser acceptance.

Acquired Word Sets retain source identities after renaming or editing, but source metadata is provenance, not proof that a user-edited or imported list retains the original curriculum membership. Do not equate acquisition with proficiency.

## R479 Vocabulary course viewport composition

The Vocabulary illustration fills its allocated mosaic viewport with cover cropping rather than leaving vertical bands. The selected-region card remains the only acquisition surface and now includes a compact per-set acquisition roadmap, derived from stable curriculum source identities and the user's saved set identities. Acquired and available states are not treated as mastery. The next-set acquisition control appears before the roadmap so it is not pushed below the available space. The under-mosaic Vocabulary Explorer provides four paginated dictionary word cards and pronunciation controls. On medium-size desktop screens it uses two columns for more legible Pinyin and definitions; on extra-wide screens it may use four columns. The right panel must not duplicate the Explorer's dictionary cards, and no extra page scrollbar should be introduced on the supported desktop viewport. Chrome verification remains pending.

## Vocabulary dictionary readiness (R480 WIP)

Vocabulary curriculum metadata and the interactive mosaic remain available while the shared dictionary initializes. The selected region and four-card Explorer show an initializing state without treating `DbError::NotReady` as a permanent region failure. On the existing database provider's Ready transition, the current selected region automatically resolves its complete HSK reading membership. Region selection or database retry invalidates pending async results, and a provider reinitialization clears the bounded resolved-word cache. Distinguish provider initialization failures (retry the provider) from selected-region dictionary query failures (retry only the region). Acquisition remains unavailable until successful complete resolution. The provider, feature reducer and Word Sets store retain their existing ownership boundaries. Runtime acceptance remains pending.

## Vocabulary loading presentation (R481 WIP)

While the shared dictionary initializes or a selected region resolves, display a compact accessible loading widget in the region details area and four bounded skeleton word cards in the Explorer. Keep the mosaic and curriculum-derived region metadata visible. The widget distinguishes dictionary preparation from region word lookup, includes a non-text spinner and inert placeholders, and honors reduced-motion preferences. It must not invent percentage progress, show an acquisition action before resolution, add a document scrollbar, or make a normal readiness transition require manual Retry. Genuine failures retain the R480 retry paths.

## R482 Courses track workspace and route redesign (2026-11-16)

The course navigation is a compact, accessible three-tab strip showing **Vocabulary**, **Grammar**, and **Pinyin** without count sublabels. Each track has a dedicated route: `/courses/vocabulary`, `/courses/grammar`, and `/courses/pinyin`. `/courses` redirects to Vocabulary. Grammar lesson and assessment exits return to the dedicated Grammar route. No legacy route fallback is introduced.

All three overview tracks use the same viewport-bounded two-column workspace on desktop: a full-height mosaic above a 166px track-specific preview on the left, and a matching-height selected-region panel on the right. On mobile, the mosaic retains a 5:3 area, and content scrolls normally. The Vocabulary Explorer has a constant 166px card height across dictionary waiting, loading, error, empty, and ready states, so changing region selection cannot move the grid. Grammar previews one published sentence pattern, or a neutral unpublished state. Pinyin previews up to three instructional pronunciation examples. These lower panels do not repeat the right-side details/actions.

The region geometry, course membership, Word Set acquisition, dictionary provider and reducer ownership, Grammar lessons and Pinyin lesson contents remain unchanged. No dependency or personal-data change. Rust/WASM build and browser visual acceptance are required and have not run in the source-authoring environment. Check 1280×720 and 1366×768 viewports, mobile 320/390px, both `/` and `/zipeek/`, keyboard selection, resize and route navigation.


## R484 Pinyin learning modules and local practice records (2026-11-16)

The six Pinyin mosaic stages now group twelve learning modules with authored explanations, relevant Mandarin examples, a reference inventory for consonant initials and common finals, browser speech playback for Hanzi-bearing examples, and two independent written-recognition questions per module. A perfect two-question result records only a *written Pinyin check passed* for that module. These tiny checks are not psychometrically validated, do not assess speech production or audio discrimination, and never award proficiency or unlock mosaic artwork. No microphone capture, new data service or third-party audio dependency has been introduced. A separate validated listening curriculum, human-reviewed audio and production assessment remain outstanding.

Grammar retains five published courses containing 24 authored points; the remaining Grammar-only mosaic regions are not falsely presented as completed courses. A focused Grammar lesson can now resume its last saved step. An independent grammar assessment retains its best valid attempt in local browser storage, with the draft score explicitly separate from verified point mastery. Existing assisted sentence-building and one-question-per-point draft checks do not meet the separate learning-evidence requirement of at least five independently scored questions per point at 80% in one attempt. For that reason **no grammar mastery or completion-driven mosaic reveal is awarded**. Grade persistence and curriculum completion are deliberately separate concepts.

The current browser-only `zipeek.courses.progress.v1` record is a strict, versioned course practice checkpoint containing passed Pinyin module slugs, Grammar step indexes and best Grammar practice scores. It is not the authoritative multi-skill evidence ledger; it is not included in the current Word Set/exclusions personal-data export. Browser storage may be unavailable, and the UI exposes save failures. A future integrated personal-data export, verified listening and speaking assessment, complete 100-region Grammar syllabus and approved proficiency rules remain separately tracked work.

## R485 Grammar lesson authoring and AI example contract

Regions 6-20 contain five authored patterns each, accessible via `/courses/grammar/topic/:region`, with local step checkpoints and a guided sentence-to-meaning review. The original five courses retain their separate authored guided checks and draft assessments. All 100 regions have named subject planning entries, but regions 21-100 cannot be entered as published courses. Neither completion of a page nor guided review implies mastery.

Lessons use a compact header and keyboard-accessible icon navigation; route or step changes scroll to the top. Pinyin reference inventories are collapsible. A supplementary Grammar AI example widget uses the shared AI service's actual availability state; unavailable/failed services do not enable requests. Generated output must contain three lines (Chinese sentence, tone-marked Pinyin and English), is explicitly labeled unverified, and does not contribute to independent assessment or mosaic reveal. No AI credentials or personal progress are sent in the prompt; authored grammar point metadata/examples are supplied as teaching context.

## Complete draft Grammar teaching inventory (R488)

The first five authored courses retain their original teaching and assessment flow. Regions 6-100 use the expanded lesson interface. The complete draft course inventory has 550 authored pattern/example entries across 100 regions, with no duplicated Chinese example or pattern label in the checked-in static inventory. Regions 21-100 are maintained in `docs/learning-practice/GRAMMAR_CURRICULUM_SOURCE.tsv`; the four checked-in Rust modules are the runtime source, and Rust catalogue tests check count and region invariants. The removed Python generator must be replaced with a native source-sync check before editing the TSV. Each expanded course offers one guided recognition question per pattern, with one question per step and explicit correction feedback. A correct guided answer is required to advance but does not establish mastery.

**Not complete as a certified learning product:** independent verified question banks, 80% assessment threshold per point, HSK syllabus mapping, expert Mandarin checking, graded speaking/listening, durable mastery evidence, and artwork reveals remain unimplemented. The 550 total is an authored-entry count, not a guarantee of 550 semantically nonoverlapping rules. Do not present these draft lessons as officially HSK-aligned.

## HSK 2025 grammar mapping (R489)

The 550 teaching entries are indexed by stable `gNNN-pNN` identifiers across the 100 courses. This count is unrelated to the HSK 2025 syllabus item count (593): its source also lists lexical classes and phrase categories, so one-to-one mapping would misstate coverage. `docs/learning-practice/GRAMMAR_TEACHING_INDEX.tsv` is a deterministic index of the **draft** lessons, not a linguistic classification or verified HSK mapping. The removed Python HSK importer previously prepared a review queue from a separately obtained third-party HSK transcription; it does not auto-award HSK coverage or learning mastery. See `docs/learning-practice/courses/hsk-grammar-alignment.md`. R493 adds tone-marked Pinyin beneath all 550 Chinese teaching and guided-review examples, using a pinned CC-CEDICT-assisted draft pronunciation inventory (`zipeek-pedagogy::grammar::pinyin`). These readings have not been independently verified by a Mandarin editor, and assessments remain Hanzi-only.

## R494 guided Grammar recognition interaction

Expanded-course guided recognition locks the answer once the correct example is chosen. Wrong choices remain retryable until correctness; a correct result does not give verified mastery. Each authored Hanzi choice has a separate browser Mandarin Listen control, still available after answer lock. Tone-marked Pinyin is a draft pronunciation aid and does not alter grading.

## Pedagogical locking and grammar progression (approved 2026-11-17)

When pedagogical mode is active and no validated evidence exists, grammar tiles remain visible, dimmed and locked without exposing the lesson via the tile. Unlocks must be derived from evidence, not the mere presence of authored courses. The proposed rolling grammar reveal requires ten independently scored qualifying successes per grammar pattern on each of any seven local days in the previous fourteen, using only eligible sentence translation, construction and transformation exercises. The day/window ledger, repeated-attempt policy and partial opacity remain implementation and scoring tasks. This is separate from the consecutive-day Hanzi recognition rule (D52).
