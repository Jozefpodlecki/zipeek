# Ux Practice Setup

> **Current approved target behavior:** [Vocabulary selection and exclusions](../../dictionary-data/vocabulary/vocabulary-selection.md), [profile portability](../../learning-practice/practice/practice-profile-portability.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


## Practice setup

- Offer HSK 2.0 levels 1-6 and HSK 3.0 levels 1-6 plus one combined 7-9 band.
  A word's membership can differ between systems. Do not show separate levels 8/9.
- Use the existing normalization defaults: Endless, HSK 3.0, level 1. A malformed
  level never selects all levels.
- Offer Endless and Fixed. Fixed has three count buttons: 5, 10, 20; default 10.
  No custom round-count input in v1.
- Show "Excluded words: N · Reset" only when N > 0. Reset clears
  saved/pending exclusions and generates a fresh practice URL without exclusions.
- Initial setup filters are HSK system + level. The URL supports PoS sets;
  a setup PoS picker is future work.

### Setup layout (UX-SETUP01)

Use a single scrollable column without decorative filler, in this order:

1. Activity: six variant tiles in a grid, three match and three MCQ. Each has
   an icon, label and subtitle: "Match four pairs" or "Pick the right answer".
   Select one or more with independent toggle buttons; keep at least one
   selected. The default set contains Hanzi to meaning only. Start chooses
   randomly from the selected set. Each subsequent round/question chooses a
   random selected type other than the previous type when more than one is
   selected; a single selected type repeats normally (D40).
2. HSK level: `HSK 2.0` / `HSK 3.0` segmented control, then conditional level
   buttons (1-6 or 1-6 plus the combined `HSK 7-9` band). Default level 1;
   no all-levels button.
3. Mode: `Endless` / `Fixed` segmented control. Fixed reveals count pills
   `5`, `10`, `20`, with `10` preselected; no custom input.
4. Start: one full-width primary button, with the exclusions/reset line above
   or beside it, shown only when the excluded count is greater than zero.

Do not add audio, example sentences or stroke order to setup.
