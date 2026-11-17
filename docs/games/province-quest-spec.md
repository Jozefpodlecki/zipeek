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
