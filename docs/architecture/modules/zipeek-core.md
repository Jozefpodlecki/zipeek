# zipeek-core module

Shared domain types and pure logic: Word/WordId, HSK and PoS enums, comparison-only
pinyin normalization, practice URL state, and the deterministic round builder.
Part of the [architecture overview](../ARCHITECTURE.md). Approved shared-crate
decision in [D01](../../process/decisions/foundations.md#d01-shared-crate).

## Constraints

`zipeek-core` must compile on both `wasm32-unknown-unknown` and native targets,
and must not import `web_sys`, `js_sys`, or `wasm_bindgen`. It holds types and
pure logic only; queries take data as arguments, never a database handle. Public
API is versioned by the repository, not semver; breaking changes are acceptable
during development (D10). The only dependency is approved `rand` 0.10.3 with
`chacha` (D17).

## Shared domain modules (A02)

`hsk.rs` holds distinct `Hsk2Level` and `Hsk3Level` enums, `HskSystem` and a
tagged `HskLevel`. Checked integer conversions reject invalid levels; `SevenToNine`
stores 7. `word.rs` holds the ten-field `Word` representation and `WordId`, checked
against positive SQL INTEGER row-number bounds. Optional metadata stays optional;
empty PoS vectors represent SQL NULL. `Word` is a transport record; the A03 importer
enforces nonblank labels, at least one membership, grouping and conflict policies
before insertion.

`part_of_speech.rs` exposes strict canonical-token parsing separately from
case-insensitive source-code mapping. `from_source_codes` rejects the entire list
on an unknown code, deduplicates recognized values and sorts by canonical token.
The A03 importer catches the error and logs/drops the entire source entry before
grouping or IDs. A08 implements tolerant URL filtering in `practice/query.rs`. No
serde or database library was added to the shared crate.

Six unit tests include the 42-row mapping fixture; native schema fixtures test
exact columns, constraints and both indexes (schema test lives in db-gen, not
here).

## Comparison and fallback boundary (A04)

`normalize_for_comparison` is dependency-free logic in the shared crate, reused
by A07 label comparison. It lowercases, maps umlaut/`u:` to `v`, converts tone
marks to trailing numbers, and removes a neutral 5 only from a vowel-containing
syllable. Numeric digit boundaries are processed before removing separators,
preserving `r5`/`ng5`. Spaces and apostrophe/hyphen syllable separators do not
enter the key. Common decomposed tone/umlaut marks are supported. Marked
concatenated syllables use a fixed syllable vocabulary and longest complete
segmentation; unknown marked chunks retain their mapped letters/marks rather than
guessing a valid syllable. Stored transcriptions and exact primary grouping keys
never use this function.

`CedictIndex` validates UTF-8 and dictionary line structure and keeps source order
within simplified/normalized-reading keys. `apply_cedict_fallback` parses the
index only when any row lacks primary English, ignores `variant of` entries, and
uses the first matching entry with usable glosses. It shares gloss serialization
with the primary importer and returns filled/unresolved/skipped-variant counts.
Unknown matches remain NULL with warnings; IDs, memberships and transcriptions
stay fixed. Fallback diagnostics identify WordIds and include CC-CEDICT line
numbers where available. `ImportDiagnostic.entry` is the source entry number for
primary events and WordId for fallback events (including fallback pipe
replacements); the detail and diagnostic kind identify the context. Primary
`MissingEnglish` events describe the earlier primary-selection stage, not the
final post-fallback outcome.

## Practice URL boundary (A08)

The dependency-free practice module separates:

| File | Responsibility |
| --- | --- |
| `types.rs` | Six approved variants, filters and modes |
| `query.rs` | Decoded pairs and diagnostics |
| `normalize.rs` | Word lookup, authoritative exclusions and seed disposition |
| `canonical.rs` | v1 formatting and FNV-1a |
| `state.rs` | Offset, exclusion union and Fixed progress |

Canonical `query_pairs` returns decoded values for the router to serialize once.
Loading a URL has no saved-exclusion input and does not rewrite browser history.

`web/services/practice_url.rs` uses the existing router decoder; Practice logs
diagnostics only in debug builds. A21 supplies dictionary-backed ID lookup
and calls normalization before loading ordered candidate views. `RecoveryRequired` is an A07 builder input,
not an RNG operation: original URL seed/filter stay canonical and the builder
owns the single recovery/generation stream.

Normalization accepts a pure `Word` lookup, supplied by A21 from validated
browser dictionary rows. `RecoveryRequired` retains the original seed/filter at
offset zero; A07 performs recovery and generation with its single RNG stream.
The published canonical golden remains `0xa0e531d76df62844`.

## Pure round builder (A07)

`round/builder.rs` owns one `ChaCha12Rng` and orchestration; `pool.rs` validates
ordered, nested, exclusion-clean and context-correct views; `recovery.rs` handles
bounded recovery and effective seed context; `labels.rs` handles first-gloss
display and right-label keys; `selection.rs` handles windows, partners and reserve
requests; `content.rs` performs only the final successful layout shuffles;
`error.rs` contains actionable errors. No database, browser, clock or ambient
randomness enters them.

`RoundCandidates` borrows initial views, original seed metadata, ordered recovery
rows and `ContextPool` entries keyed by recovered filter. A06 must supply these
as pure data, including views for possible recovered contexts when recovery is
needed. Ready seeds use their own context; canonical hashing always uses original
normalized URL state. Missing or inconsistent data fails rather than silently
substituting filters or sorting rows. `RoundContent` exposes only playable cards;
`BuildTrace` separately records internal reserves and requests for diagnostics.

`tests/support` loads a small TSV fixture and produces 18 full-round cases shared
by native tests and the `replay_vectors` wasm example. The Node runner compares
the wasm digest with the checked-in native fixture and rejects any host call.
It needs no added dependency. Web has no exercise UI or DB connection yet at the
time A07 landed.

### Module map

| File/module | Responsibility |
| --- | --- |
| `round/builder.rs` | RNG ownership and orchestration |
| `round/pool.rs` | CandidatePool validation |
| `round/recovery.rs` | Bounded seed recovery, effective context |
| `round/labels.rs` | First-gloss display, right-label keys |
| `round/selection.rs` | Windows, partners, reserve requests |
| `round/content.rs` | Final layout shuffles |
| `round/error.rs` | `BuildError` variants |

### Builder API

`build_round(RoundCandidates, ExerciseVariant, &NormalizedPracticeQuery)` returns
`Result<Round, BuildError>`. `RoundCandidates` borrows an initial `CandidatePool`,
the original seed row if retained, ordered/exclusion-clean recovery rows and
`ContextPool` entries for possible recovered contexts. For a Ready seed, initial
views must use its HSK/PoS context; unseeded views use the URL filter. Recovery
prefers HSK 3.0 membership and derives the seed's canonical PoS set, then selects
the supplied views for that context. It never performs a query or rehashes.
The original seed/filter remain canonical even when effective context changes.

The builder rejects unordered/duplicate IDs, inconsistent or nonnested rows,
excluded rows, wrong-system/level/PoS views and missing seed/context data with
actionable errors. It validates caller ordering rather than sorting the views.
SQL construction and exclusion binding remain A06 responsibilities.

## What zipeek-core is not

- **Not a database client.** No `web_sys`, no `js_sys`, no DuckDB bindings.
- **Not a UI library.** No Yew, no `Html`, no components.
- **Not a domain-model owner.** That is `web`'s `models` module for UI-facing
  types; `zipeek-core` types are the shared domain layer used by both `web` and
  `db-gen`.
- **Not an importer.** The importer lives in `db-gen`; core supplies shared types
  and pure algorithms (pinyin normalization, gloss serialization helpers).