# Character Hunt - exercise contract (R215)

Character Hunt is a dictionary-backed Hanzi assembly exercise. It is available without AI in Quick Practice (and also Advanced Practice) under **Exercises → Choose** and configured under **Exercises → Configure**. Its difficulty is stored in the Rust `PracticeDraft`, not selected again during gameplay.

| Difficulty | Grid | Hanzi typography |
| --- | --- | --- |
| Easy | 2 × 2 | 5xl mobile, 7xl desktop |
| Medium (default) | 3 × 3 | 4xl mobile, 6xl desktop |
| Difficult | 4 × 4 | 3xl mobile, 5xl desktop |
| Hardcore | 5 × 5 | xl mobile, 3xl desktop |

## Rules and generation

The pure Rust domain engine (`zipeek-exercises::character_hunt`) takes an eligible vocabulary snapshot, a difficulty, and a seed. It accepts distinct Simplified-Hanzi entries containing exactly two or three characters. It allocates character tiles from dictionary entries, shuffles them deterministically, and retains the validated word-to-tile mapping privately. Word groups do not share characters or tiles, so removing completed words never makes another target unplayable. The generator deterministically retries 32 candidate orders to avoid rejecting a solvable board merely because a greedy word selection dead-ended. If a board cannot be filled from the supplied words, generation fails visibly; it never inserts fabricated answer words.

A single noninteractive decoy character from the eligible vocabulary may fill an odd-sized grid when the source lacks compatible three-character words. If there is no unused source character, generation fails rather than inventing a filler. The decoy does not count toward the word total. Completed words are presented as Hanzi with an optional word-level pinyin reading and the dictionary-sourced English gloss (not merely a list of Hanzi). A completion message also identifies the word and its meaning. Alternative dictionary glosses are deduplicated and displayed together, capped at three to keep the card readable; missing definitions are marked unavailable instead of fabricated. Pinyin is withheld if source readings cannot be aligned or do not agree. Each non-decoy tile displays a large Hanzi with smaller dictionary-sourced pinyin beneath when word-level syllables align with the characters. Multi-pronunciation entries with conflicting readings do not display an arbitrary reading. In Easy/Medium/Difficult, the large Hanzi typography is preserved; Hardcore uses compact typography. Learners select two or three active tiles in order; a matching known entry grays out those tiles and records the word. Incorrect combinations increment a local mistake counter and reset the selection without removing tiles. Three consecutive mistakes briefly highlight the first tile of a remaining solution; after five, the entire 2-3-character solution briefly highlights for approximately 950 ms. After seven consecutive mistakes the user may explicitly reveal one word; revealed words count as completed but are tracked separately as assisted. A correct match resets the consecutive-mistake escalation. Automatic visual hints appear only on the third and fifth consecutive mistakes, then clear after 950 ms or immediately on the next tile interaction. The tile highlight is static rather than continuously pulsing. Manual Hint works only when no tile selection is in progress, so it cannot obscure an active attempt. The hint button is disabled until the partial selection is cleared or finished. Completion requires finding every hidden target. No user-generated text or AI model is needed for verdicts.

## Session integration

The existing four-step setup (`Path → Exercises: Choose/Configure → Vocabulary → Summary`) launches `#/practice/character-hunt` with difficulty and vocabulary source. It uses Fixed 5 by default, with Fixed 10, Fixed 20, and Endless available. The UI keeps difficulty fixed across boards and shows round-level progress, completed words, mistakes, and an explicit Next round action. Learners can deselect individual tiles by tapping them again without counting an incorrect attempt. Fixed session completion shows cumulative rounds, words found and incorrect attempts; these totals are only recorded after a board is completed. A new round uses a new seed and the same eligible source. Exit practice is separate from puzzle controls and asks for confirmation before abandoning an unfinished session. The final summary distinguishes assisted reveals from independently found words. In Quick Practice, selecting Character Hunt replaces the core Matching/MCQ selection and vice versa, because the standalone engines are not yet mixed in one session.

Dictionary selection now supports one or more HSK bands, category-only sources, and any approved combination of HSK bands, categories and parts of speech. The setup route serializes these as the same validated dictionary-filter token used by Practice. The Character Hunt loader decodes that token and calls a parameterized database query with the shared OR-within-group/AND-across-group predicate. The query selects two- and three-character entries only, with a deterministic 1,024-row upper limit to bound browser memory. A saved Word Set is loaded by exact IDs instead and does not inherit dictionary filters. Legacy single-HSK game links remain readable. Extremely restrictive filters can legitimately yield too few playable words; this produces an explicit board-generation error instead of silently broadening the vocabulary. Mixed Character Hunt/Matching/MCQ plans are **not** supported yet; the existing scored mixed-session planner remains unchanged. The game has its own round progression and does not claim to publish scores to the central planned-session ledger.

## Verification

Unit tests cover grid sizes, default difficulty, reproducibility, word completion, three-character words, invalid combinations and insufficient vocabulary. A Chrome journey checks Quick Practice setup, choosing Easy, selecting an additional HSK band, preserving both filters in the URL, then solving five 2×2 boards using only visible tile controls and checking the cumulative Fixed 5 summary. Database SQL tests assert shared filtering semantics for multiple HSK bands, category and part-of-speech intersections, and category-only sources. The browser tests and Rust/WASM compilation require execution on a machine with Cargo and Chrome; Node source regression tests alone are not sufficient.

## R218 interaction and layout refinement

Gameplay has one bottom action bar: Hint/Reveal on the left and confirmed
Exit practice / Continue on the right. There is no Clear selection button:
re-tapping an already selected Hanzi deselects it, and invalid combinations
clear automatically. The practice round header shows the round and selected
difficulty without restating the grid dimensions.

The completed-word panel displays the exact dictionary-backed Hanzi, contextual
pinyin when available, English definitions and the shared Listen button for
audio playback. The completed-word list scrolls inside its panel on desktop,
without pushing the board or bottom actions offscreen. Progress is displayed
with a blue accessible determinate bar rather than native browser styling.
Both the game board and controls remain accessible by keyboard. Browser
viewport validation is pending.
