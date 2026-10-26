# Product specification

This document describes intended v1 behaviour, not completed functionality.
See [ARCHITECTURE](ARCHITECTURE.md) for the current implementation and
[DECISIONS](DECISIONS.md) for choices still requiring approval.

## Purpose

Query Chinese words by hanzi, pinyin, English, and part of speech, and practise
them through matching and multiple-choice exercises. The app runs client-side
on GitHub Pages under
`/zipeek/` and reads a pregenerated DuckDB database.

D07 limits the runtime word pool to the union of HSK 2.0 and HSK 3.0 words,
provided by D16's pinned primary dataset, with CC-CEDICT only for empty-gloss
fallback. "Full pool" below means that HSK-union table, not all of CC-CEDICT.
The audited primary source has 11,494 entries; final reading-row counts differ.

## Startup loading experience

Show the static loading shell from `web/index.html` while the app boots. When
Yew first mounts, its first render must also be the loading state, with a continuous
handoff from the HTML loader. Keep loading visible until DuckDB-WASM is initialized
and the fetched database is open and ready for queries; mounting Yew is not the
end of startup loading. Avoid a blank screen or a flash of empty/ready content.

On a database load failure, show the retry state described for Word of the Day
instead of leaving a permanent spinner. See [ARCHITECTURE](ARCHITECTURE.md) for
the startup stages. This behaviour is required but not yet implemented in the
current route-scaffold App.

## Routes

Paths below are app-relative; deployment retains the `/zipeek/` prefix.

| Route | Intended view |
| --- | --- |
| `/` | Left icon panel with theme switch, Word of the Day, Practice/Browse/Random tiles |
| `/practice` | Setup: HSK system/level, practice mode, Fixed count, exclusions reset; PoS picker is future work |
| `/practice/eng-pinyin` | English to pinyin matching |
| `/practice/pinyin-hanzi` | Pinyin to hanzi matching |
| `/practice/hanzi-eng` | Hanzi to English matching |
| `/practice/mcq/hanzi-to-meaning` | MCQ: simplified hanzi prompt, English answers |
| `/practice/mcq/meaning-to-hanzi` | MCQ: English prompt, simplified hanzi answers |
| `/practice/mcq/meaning-to-pinyin` | MCQ: English prompt, display-pinyin answers |
| `/browse` | Paginated results with part-of-speech, pinyin, and hanzi-character filters |

All three matching variants and the D20 MCQ variants are in scope for v1. The overall query requirement
also includes English; its exact browse control is still to be specified.
Pinyin browse queries support both display-tone and original numbered forms (D07).
Preserve D16's source transcriptions, including their spacing and conventions.
Both English-bearing variants show the first gloss: left in eng-pinyin, right in
hanzi-eng. Exclude rows without English from those variants; pinyin-hanzi can
still use them. Duplicate detection normalizes labels, not display casing.

## Practice

### HSK filters

Treat HSK system and level/tier as separate concepts. HSK 2.0 has levels 1-6;
HSK 3.0 has levels 1-6 plus a combined advanced tier 7-9. These are distinct word
lists. A word can have different levels in each, belong to both, or belong to only
one. For example, a word may be level 2 in HSK 2.0 and level 3 in HSK 3.0.
Missing membership is not a level or an inferred equivalence between systems.

The HSK 3.0 advanced band has one domain value, `Hsk3Level::SevenToNine`, stored
and canonically encoded as integer `7`. Values 8 and 9 are not separate levels.
Accept `level=7-9` as an alias for HSK 3.0 and normalize to `level=7`. Display
"HSK 7–9" or "Advanced (7–9)", never three separate level buttons. HSK 2.0 still
accepts only 1-6. Missing/invalid levels default to 1 in the selected system (D15),
never all levels. Invalid values produce a debug-only warning, not a page error.

The setup screen's initial filter scope is HSK system + level. URL filtering
already supports PoS; adding a setup PoS picker later does not change the URL scheme.

### Round generation (D02: approved)

The visible round has exactly **four cards on the left and four on the right**:
eight labels total. The user matches four corresponding pairs.

Internally, the builder uses four correct pairs plus up to four distractor
candidates per side: up to eight candidates per column before selection. Extra
candidates never render and are discarded after the four winners are chosen.
Each visible left card must retain its corresponding right match; candidate
sampling must not turn the visible exercise into an unmatchable set.

1. Choose four correct pairs from the filtered HSK + PoS pool. D02 originally
   specified random selection; D13 supersedes that for unseeded practice with
   deterministic ordered slices at the URL's offset. Random still chooses its
   initial variant/seed randomly, then must rebuild deterministically from its URL.
2. Draw distractor candidates from the same PoS and HSK pool. D13 requires any
   random-looking candidate choices to reproduce from the URL inputs (D12).
3. No two entries in a round may share the visible right-column label for the
   selected variant. Deduplicate candidates and resample to achieve uniqueness.
4. If the deduplicated pool cannot supply eight candidates, widen HSK to the
   selected level plus or minus one, then drop the PoS filter if still too small.
   Reaching the duplicate-resampling retry limit also triggers this widening.
5. If no candidate view can supply four correct pairs plus four distinct-label
   reserves, return D12's insufficient-pool error and show the distinct-words
   message with level/exclusion recovery controls.

The four additional candidates are internal reserves, never extra visible cards.
D12 specifies the pure CandidatePool builder, canonical seed string, portable
ChaCha12Rng, SQL ordering/exclusions, bounded sliding/rejection, and system-clamped
widening in [ROUND_GENERATION](ROUND_GENERATION.md). Unseeded windows slide past
duplicate right labels; widened views start at zero. Seeded rounds never slide.

### Practice modes (D03: approved)

The user picks the mode on `/practice`:

- **Endless:** infinite rounds with a Next round button between rounds. No
  summary and no score persistence. A running streak/tally in the header is allowed.
- **Fixed:** choose N rounds up front. After round N, show correct pairs, total
  mistakes, and time taken, with Play again and Back to setup actions. Offer
  exactly three round-count buttons: **5 / 10 / 20**, default **10**. No custom-N
  input in v1 (D13).

### Random shortcut (D04: approved)

Random reuses the practice round generator with randomized inputs. It ignores
setup-screen HSK/PoS filters and always starts Endless mode.

1. Pick a variant randomly from `eng-pinyin`, `pinyin-hanzi`, and `hanzi-eng`.
2. Pick a seed word randomly from the entire database, without setup filters.
3. Select the seed plus three other correct pairs matching its PoS and HSK
   membership, with distractor candidates from the same pool. Apply D02's widening
   and duplicate-label rules unchanged.
4. Navigate directly to the selected practice variant with `mode=endless`, `seed`,
   and the seed's `hsk`/`level` context, plus exclusions if nonempty (D13 below).
5. Retry a seed with no HSK membership up to ten times, then use the first ordered
   word with a membership. Prefer HSK 3.0 when both exist. If the three candidate
   views still fail, report D12's typed insufficient-pool error. D12 replaces the
   older unfiltered-round escape hatch.

The seed's attributes become the effective filter. There is no separate
Random-specific round-building implementation; all deterministic selection uses
the shared builder. Normalize seeded offset to zero before forming its seed string.

### Practice URLs and replay (D13: approved)

Unseeded practice from setup uses this query contract (bracketed parts optional):

```text
/practice/<variant>?mode=<endless|fixed>&rounds=<5|10|20>&hsk=<2.0|3.0>&level=<1..7>[&pos=<noun,verb>][&exclude=<id,id,id>][&offset=<n>]
```

- Canonical URLs include `mode`, `hsk`, and `level`. Include `rounds` iff
  `mode=fixed`; in Endless it is ignored. Defaults for missing/invalid values are
  `mode=endless`, `rounds=10`, `hsk=3.0`, `level=1`, `offset=0`.
- `hsk` and `level` identify the active filter across all rounds and on deep links.
  HSK 2.0 allows 1-6; HSK 3.0 also allows 7 for the combined advanced tier.
  Missing/invalid `level` uses 1 in the selected system, including values such as
  99, abc, or level 7 with HSK 2.0. Never broaden an invalid level to all levels.
- Optional `pos` is a comma-separated set of canonical lowercase enum strings
  (for example `pos=verb,noun`). Include words whose PoS is in the set. Omitted
  or empty means all parts of speech; do not emit `pos=`. Drop unrecognized
  entries while retaining recognized entries; if none remain, use no PoS filter.
  The DB and query layer share the vocabulary documented in [DB](DB.md).
- `exclude` contains comma-separated integer WordIds. Apply only the router's
  normal query serialization; no additional encoding pass. Omit empty exclusions
  entirely, rather than emitting `exclude=`. No new serialization dependency is
  approved merely by this contract.
- `offset` is a nonnegative index into the filtered, ordered pool, not a word ID.
  Every candidate-view query uses `ORDER BY id ASC`, after exclusions in SQL.
- Parse values strictly and fall back to defaults for unknown/malformed values;
  malformed links must never panic. Canonical output requirements do not prevent
  safe defaults when parsing incomplete input.

Random seed replay uses a distinct contract:

```text
/practice/<variant>?mode=endless&seed=<word_id>&hsk=<2.0|3.0>&level=<1..7>&pos=<seed_pos>[&exclude=<ids>]
```

The seed anchors one of the four correct pairs. The Random tile writes the seed's
HSK system/level and canonical PoS into the URL so reload uses those same attributes.
For example, a verb seed emits `pos=verb`. HSK 3.0 advanced seeds use `level=7`.
Retained seeded rounds require Endless. If a URL combines Fixed with a seed,
ignore the seed, warn only in debug, and run normal Fixed with its original
filter/offset. Offset is zeroed only after a seed is validated and retained.

### URL normalization and fallbacks (D15: approved)

Normalize input deterministically in memory; malformed URLs must not panic or
error the page. Preserve the current URL until an explicit new URL is generated.

1. Parse mode/count, HSK system/level, PoS, exclusions, and entered offset using
   the defaults above. The URL's exclusions are authoritative, not saved browser
   exclusions. Invalid levels use 1 with a warning only in debug builds.
2. If mode is Fixed, ignore any seed before seed lookup or offset-zeroing. Keep
   Fixed, its count, the same filter, and the entered offset; emit a debug warning.
3. Otherwise, an absent/malformed seed, nonexistent ID, or seed excluded by this
   URL falls back to an unseeded round with the same filter and entered offset.
   An ID excluded only in localStorage must not invalidate a shared URL's seed.
4. An existing, nonexcluded seed with neither HSK membership uses deterministic
   recovery: up to ten ChaCha draws from the ordered, exclusion-clean pool, then
   the first eligible word by id. No ambient randomness on URL loads. Keep the
   original bad seed in the URL; recovery is internal and reproducible.
5. Only a retained/resolved seed normalizes offset to zero before canonical RNG
   input construction. A rejected seed must not erase the unseeded fallback's
   entered offset. See ROUND_GENERATION for the two seed-selection paths.

The same URL and database snapshot must reproduce the same round, including
candidate choices and label layout. Later resampling/shuffling must therefore be
deterministic from the canonical inputs; using ambient randomness on reload would
violate replay. D12's [round-generation specification](ROUND_GENERATION.md) defines
the algorithm; D17 selects rand's ChaCha feature and D19 fixes the sole v1
format and single-stream recovery/selection/shuffle order. Golden-round
implementation checks remain future work.
The database is committed in Git, but regeneration may change it; the development
policy does not promise compatibility or identical rounds across database changes.

### Word exclusions (D13: approved)

Marking a word "don't show again" adds its WordId to the pending next-round
exclusion set and the saved set persisted in `localStorage` through gloo storage.
Do not change the current round's URL, active candidate set, cards, or matches
mid-round. Apply the new exclusion only when generating the next practice URL.
A suggested storage key is `zipeek.excluded_words`; the exact key is not fixed.
Persisting WordIds does not persist hash state or practice scores.

Exclusions have two distinct lifecycle rules:

1. **Generate a URL** (setup, Random, Next round): serialize the union of the
   saved set and the current round/per-round exclusions. Saved state contributes
   here; the emitted URL carries the complete set and is self-contained.
2. **Read a URL** (page load, deep link, reload): the round's set is exactly the
   query's exclusions. Never union saved exclusions into a loaded URL or rewrite
   it implicitly. An absent `exclude` means the empty set, even if localStorage
   contains saved exclusions. A recipient's saved set cannot alter a shared round.

Read saved IDs for URL generation (and display their count in setup), not to
augment loaded rounds. Write saved IDs only on an explicit don't-show-again action
or the explicit Reset action. Loading a shared link must not persist its IDs.
Deduplicate IDs, ignore unknown integer IDs, and drop noninteger entries with a
debug-only `warn!`. Omit the query parameter when the effective set is empty.

Filter excluded IDs **before** choosing correct pairs and distractor candidates,
including after each D02 widening step. If fewer than eight candidates remain
after widening, show "not enough words after exclusions — reset exclusions?"
with a button that clears the set. The setup screen also shows
"Excluded words: N · Reset". Clicking Reset on setup clears saved and pending
exclusions and generates a new practice URL with `exclude` omitted. It does not
silently rewrite an in-progress round when a word is marked.

### Offset progression (D13: approved)

New unseeded Fixed and Endless sessions from setup default to offset 0. Without
wrapping, a fresh session's zero-based round r uses `offset = r * 4`: displayed
round 1 starts at 0, round 2 at 4. Deep links/reloads retain their entered offset
even when in-memory progress restarts. Next round advances nominal offset by four.
Fixed stops after the selected number of
rounds and shows the summary; Play again resets offset to 0. Endless continues.

If `offset + 4 > pool_size`, wrap to offset 0 and show a small
"pool exhausted, looping" hint once. Strict slices without sliding are distinct
until exhaustion. D12's duplicate-label sliding and widened-view offset-zero
fallback can overlap correct sets even before wrapping. The eight-candidate minimum
and visible-label rules still apply. Fixed completion must track completed rounds
separately from the wrapping offset, as specified below.

### Fixed progress, reload, and seeded Next round (D15)

Fixed mode keeps completed/current-round progress and total rounds in memory.
Initialize completed progress to zero (display round 1/N); stop after N completed
rounds. Do not persist progress. Reload resets that counter while honoring the
URL's pool offset: a bookmarked offset 40 does not imply round 11. Wrapping the
pool to offset zero mid-session leaves progress unchanged, such as round 4/5.

For seeded Endless, Next round clears `seed`, retains the same filter, and
generates an unseeded URL. All v1 variants match four pairs, so advance by **4**:
the seeded normalized offset is zero and Next starts at 4, subject to normal
pool-exhaustion handling. Do not advance by one for v1. Freshly generated URLs
fold in saved/pending exclusions per D13; the current seeded URL is unchanged
until that navigation.

## MCQ practice (D20: approved)

D20 is approved on 2026-10-26. Each question presents one prompt and exactly
three answer options: one correct answer and two distractors (N=2).

| Variant token | Prompt | Answer label |
| --- | --- | --- |
| `hanzi-to-meaning` | Simplified hanzi | First English gloss |
| `meaning-to-hanzi` | First English gloss | Simplified hanzi |
| `meaning-to-pinyin` | First English gloss | Display pinyin |

Routes are `/practice/mcq/<variant>` as listed above. These routes are specified
but are not yet declared in `web/src/route.rs`.

Use the same HSK/PoS filtering, exclusions, deterministic URL principles, and
D02/D12 widening/answer-label uniqueness rules, adapted to two distractors.
All three variants require English, so apply D18 eligibility rules. D19 fixes
MCQ prompt selection, two distractor samples, and final option shuffle on one
continuous RNG stream; full-round golden tests remain implementation work.

Support Endless and Fixed. Fixed offers 5/10/20 questions, default 10, with
session progress kept independently of the pool offset. Advance the nominal
offset by one per question; matching continues to advance by four per round.
Endless has no Fixed summary. UI feedback timing, number-key hints, and answer
button styling remain [UX-MCQ01](UX.md#mcq-exercise) details for A22.

## Landing and theme

- Show exactly three navigation tiles: Practice, Browse, Random.
- Random starts the approved shortcut described above (D04).
- Show a Word of the Day card using the approved selection below (D05).
- Use the shared left sidebar, theme controls, and footer described below.

### App shell, theme selection, and footer

- Provide a left sidebar in the shared app layout across routes.
- **Dark is the preferred/default theme** when the user has no saved selection.
  Offer an explicit dark/light toggle in the sidebar so users can select either.
  Restore a saved choice ahead of the default and persist changes in `localStorage`
  through existing gloo storage facilities.
- Apply the same selected theme to the static HTML loading shell, Yew's initial
  loading view, and the ready app. Avoid a light flash during startup or route changes.
- Anchor a GitHub icon at the **bottom of the left sidebar**, linking to the
  project repository: `https://github.com/Jozefpodlecki/zipeek`. Give the icon link
  an accessible name identifying the repository.
- Include a footer with **Jozef Podlecki** and the copyright symbol/year, for
  example `© 2026 Jozef Podlecki`. Use the current year when implementing it.
  Keep the dictionary attribution below alongside the personal copyright.

These are target layout requirements; the current App does not yet render the
shared shell. Its current HTML uses the system theme when no preference is
saved, which must change to the preferred dark default during implementation.

When dictionary data is served, show this attribution in the visible site footer,
with links to the project and licence:

> Chinese dictionary data from CC-CEDICT ([cc-cedict.org](https://cc-cedict.org/)), licensed CC BY-SA 4.0.

See [DB](DB.md) for primary/fallback source notices and the pinned import rules.
D16 supplies PoS and frequency when available; genuinely missing metadata remains
NULL. D18's complete, case-insensitive source-code mapping defines canonical PoS
tokens; entries with unknown codes are excluded during import with a warning.
Do not infer missing values or display raw source codes as app enum tokens.

No unresolved default in DECISIONS is an implementation requirement yet.

### Word of the Day (D05: approved)

Select deterministically from the full word pool, independently of practice
HSK/PoS filters. Read the browser's local calendar date using `js_sys::Date` and
format it as `yyyy-mm-dd`; do not substitute a UTC calendar date.

1. Count the full word pool using `COUNT(*)`.
2. If the count is zero, show the neutral placeholder "No word available".
   Do not compute modulo zero or panic.
3. Hash the UTF-8 date string and reduce the hash modulo the word count.
4. Pick that offset in a deterministically ordered result. D07 uses synthetic
   IDs, so use `ORDER BY simplified, pinyin`, whose unique natural key resolves
   ties. This is D05's generic hanzi ordering mapped to the physical schema.
   Never use `LIMIT 1 OFFSET n` without deterministic ordering.

The same local date, pool, ordering, and hash implementation produce the same
selection. Data regeneration may change the pool or ordering and thus the word.

Use **64-bit FNV-1a** over the UTF-8 date bytes (D14): initialize `h` to offset
basis `0xcbf29ce484222325`; for each byte, apply `h ^= byte as u64` then
`h = h.wrapping_mul(0x100000001b3)`. Implement as a small dependency-free Rust
`const fn`. Use `u64`, not
`usize`, for hash arithmetic and modulo reduction so 32-bit and 64-bit builds
agree; convert only the bounded result where indexing requires it.

`DefaultHasher` is allowed only for ephemeral in-session use such as a HashMap.
It is forbidden for Word of the Day and any value that must reproduce across
builds or upgrades. Do not persist hash state anywhere. No hash dependency is needed.

If the DB fails to load, show the same "No word available" placeholder plus a
retry button. An empty successfully loaded pool does not require the failure-only
retry state.

For reproducible e2e, support the testing-only override `/?wotd=2026-10-26` to force
that local calendar date. Gate date-override behaviour behind
`#[cfg(debug_assertions)]`; debug e2e can use `cfg!(debug_assertions)` consistently.
Release builds may parse the query string but ignore `wotd` entirely: it has zero
behavioural effect in Pages deploys. No testing Cargo feature is needed for v1;
introducing one for release testing later requires a separate approved change.

Validate exact `yyyy-mm-dd` formatting, calendar validity, and supported date range.
An invalid override, such as `2026-13-40`, silently falls back to the real browser
local date. Do not panic or show a page error. An optional `warn!` is allowed only
in debug builds. Use the valid override to assert a specific word in e2e. No Clock
trait is required.
