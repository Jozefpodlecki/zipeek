# Province Quest - approved feature contract

## Learning progression

- Easy: locate provincial-level divisions from their Mandarin names. Show optional Pinyin and English reference during review.
- Medium: identify provincial-level divisions, their administrative seats, and region outlines. No invented provincial flags: the majority of mainland provinces do not have distinct official flags. Official symbols may be shown only with verified provenance and licensing.
- Hard: locate prefecture-level administrative divisions (prefecture-level cities, prefectures, autonomous prefectures, and leagues). Select the parent province first and then pick from a zoomed provincial map so touch targets remain usable.

The historic figure of 333 prefecture-level divisions is not a permanent dataset invariant. Display counts must be derived from the pinned geographic source. The source must state its census date, provenance, licensing, coordinate reference system, inclusion criteria, name translations and administrative codes.

## Data and acceptance requirements

- Version and bundle the map polygons locally instead of relying on a third-party server at play time.
- Use stable region codes for matching; display Chinese names and properly sourced Pinyin independently of polygon labels.
- Review legally reusable boundary geometry before committing it to the source archive. A GitHub repository license does not necessarily sublicense its upstream geographic data.
- Support polygons with multiple disconnected areas and maintain keyboard, pointer and touch selection.
- Separate geometry loading from pure game-state transitions. One game reducer owns the current prompt, selection, mistakes, review state and continuation; setup owns difficulty and learning-support options.
- Unit-test round generation and matching for every included code, including names that repeat at different administrative levels.
- Independently verify administrative seats/capitals; avoid using a single capital field for special administrative regions or municipalities without defining its meaning.
- No map route should be registered until a complete playable dataset and interactive hit-tested map are installed and validated.

## Gallery

The game gallery now shows six games per desktop page using previous/next controls, smaller cards and the same responsive grid. The gallery stays nine games until Province Quest is complete.

## Immersive gameplay presentation

Province Quest runs in the immersive game shell without the standard application header, drawer, or footer. The game provides its own accessible Exit and Settings controls. During play the map occupies available viewport space; map panning uses pointer dragging and wheel zoom, with accessible button alternatives. Results and choices remain keyboard-operable and review messages identify correct and incorrect selections in text, not color alone. Semantic map colors derive from the active theme. Geography provenance belongs in the maintained Credits page and distribution notices, not as a persistent label inside the map.

## R557 implementation boundaries

The `zipeek-exercises::province_quest::Session` domain remains the single scoring authority. A separate feature reducer coordinates setup selections, map asset loading and error states, active province selection, request invalidation, and session actions. The map owns only its typed viewport and pointer gesture state. Viewport math and drag thresholds are tested without browser APIs. Asynchronous load completion is accepted only for the active request generation. The UI retains keyboard map-region selection, immersive presentation and theme-driven semantic colors. Browser gesture acceptance remains necessary before considering the refactor complete.

## Hard-mode map navigation

The Hard-mode map renders a national overview immediately, rather than an empty map while waiting for parent selection. Selecting a province in the overview drills into its prefecture boundaries. The province selector lists only parent regions represented by usable prefecture geometry; an unpopulated administrative region must not produce a blank map. Returning the selector to its initial value shows the national overview again. Province navigation does not count as an answer.

The gameplay header uses accessible icon controls for leaving the game and opening settings, following the immersive game shell contract.

## R559 map gesture refinement

The SVG uses `preserveAspectRatio="xMidYMid meet"`. Wheel anchors and panning must therefore use the rendered map scale rather than the full rectangular SVG bounds, which may include letterboxing. Wheel deltas are normalized by delta mode and clamped per event. Dragging begins after a cumulative movement threshold; pointer capture starts on actual drag, preserving native region clicks and keyboard selection. Small incremental pointer moves must accumulate correctly. Zoom controls remain available as accessible alternatives.

## R560 region component contract

`RegionMap` owns viewport transforms, mouse-wheel/pointer gesture state and suppression of clicks after a drag. Its child `MapRegion` component owns one SVG path's theme-based visual state, accessible region label, pointer click, and Enter/Space activation. Child props reference the existing `Rc<GeoDataset>` and a stable region index rather than copy potentially large SVG path strings. Parent and child preserve the `data-region` hook and forward stable administrative IDs, not names or display indices. Event handlers bound directly to the SVG root may cast a present `current_target` from `EventTarget` to `Element` using `unchecked_into`; this assumption must not be reused for generic or delegated targets. Use named local `MapPoint` values for cursor and pointer geometry. Runtime browser verification remains outstanding.
