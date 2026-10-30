# Deterministic round generation (D12)

These approved requirements are implemented by A07 in `zipeek-core/round`. See
[PRODUCT](PRODUCT.md) for the four-pair exercise and
[DECISIONS](../process/decisions/index.md) for approved source contracts (D18)
and the resolved v1 replay contract (D19). The pure builder and native/wasm golden
fixtures exist; A21 integrates database queries/normalization and calls the
builder. Playable UI remains A09/A10 work.

## Pure builder and candidate bundle

`build_round` is a pure function of `(CandidatePool, variant, filter, seed, offset)`.
It never queries DuckDB or reads the clock, localStorage, or ambient randomness.
It can be tested natively using plain Word fixtures.

The caller supplies three nested views:

```rust
struct CandidatePool {
    strict: Vec<Word>,
    hsk_widened: Vec<Word>,
    hsk_widened_any_pos: Vec<Word>,
}
```

strict: exact HSK system/level and requested PoS set.

hsk_widened: same system, original level plus valid adjacent levels, same PoS.

hsk_widened_any_pos: same widened HSK levels, with the PoS filter dropped.

strict is a subset of hsk_widened, which is a subset of
hsk_widened_any_pos. A wasm-side helper issues one DuckDB query per view.
Every query removes exclusions in SQL and includes ORDER BY id ASC. Do not
depend on implicit row order or substitute a Rust-side sort. Bind exclusion IDs;
an empty exclusion set needs no NOT IN predicate.

The caller also clamps HSK widening in SQL, never crossing systems:

Context	Widened levels, including original
HSK 2.0 level 1	1, 2
HSK 2.0 level 6	5, 6
HSK 3.0 level 1	1, 2
HSK 3.0 level 7 (combined 7-9)	6, 7
Intermediate levels include original +/-1 within the system's bounds. HSK 2.0
has no 7, and HSK 3.0 has no separate 8 or 9.

Try views in order: strict, widened HSK, then any PoS. Each matching attempt
must produce four correct pairs plus four distinct-label reserve distractors. Advance to the
next view if it cannot. All reserves remain internal; only four matching cards
on each side render.

Canonical seed input
Normalize raw URL inputs before calling the builder. For a seeded round, force
offset to zero at this boundary. For an unseeded round, retain the entered offset.
The formatter itself is a pure formatter with no offset policy.

text
zipeek-round-v1|{variant}|{hsk}|{level}|{pos}|{offset}|{seed}
All fields are hand-written ASCII tokens separated by |, with no extra whitespace:

Field	Encoding
Prefix	Literal zipeek-round-v1
Variant	Match: eng-pinyin, pinyin-hanzi, hanzi-eng; D20 MCQ: hanzi-to-meaning, meaning-to-hanzi, meaning-to-pinyin. Use an explicit seed_token() method, matching each variant route token.
HSK system	hsk2 or hsk3
Level	Unpadded unsigned decimal: 1-6 for hsk2, 1-7 for hsk3
PoS	Sort canonical lowercase token() strings, deduplicate, and join with commas; empty set is an empty field
Offset	Unpadded unsigned decimal; zero when unset or seeded
Seed	Decimal WordId, or literal lowercase none
Never derive fields from Debug, Display, iterator order, or HashSet ordering.
For PoS, explicitly collect tokens, sort_unstable, dedup, then join(",").
An absent seed encodes as literal none, never Debug's None or Some(...).
Empty PoS is no characters between separators, never {} or None. Equivalent
parsed filters must produce identical canonical bytes.

Apply D14's 64-bit FNV-1a to the canonical string's bytes; this string is the sole
hash input. Initialize rand::rngs::ChaCha12Rng::seed_from_u64(rng_seed) with
rand::SeedableRng in scope and rand's chacha feature enabled (D17).
StdRng is forbidden for round generation. Do not replace FNV-1a with DefaultHasher.

Documentation golden vector:

text
zipeek-round-v1|eng-pinyin|hsk3|3|noun,verb|10|none
fnv1a64 = 0xa0e531d76df62844
Under D19 (approved 2026-10-26), v1 is the only format version. There are no
v1 URLs outside development fixtures and no installed base to migrate. A future
replay break requires a v2 canonical prefix, a URL parameter selecting the format,
and a documented compatibility path, with explicit approval for the shared-URL
break. None of that v2 machinery exists today: do not add a v= parameter or
version parser speculatively. The earlier COMPLETED v2 forecast is superseded
by an append-only correction, not a commitment to implement v2 now.

Single-stream RNG and draw order (D19: approved)
AGENTS §5 is the authoritative contract if any
wording here conflicts. For each build_round, create exactly one
rand::rngs::ChaCha12Rng, seeded once with
ChaCha12Rng::seed_from_u64(fnv1a64(canonical_seed_input)). Recovery consumes
draws from this stream; generation continues from the resulting state. No reset,
no second RNG, no re-seed, including on candidate-view fallback. Never clone the
RNG to restart a phase. First-eligible-by-id recovery fallback consumes no RNG.

Before creating the RNG, apply D15's nonrandom URL validation: Fixed ignores a
seed; malformed, nonexistent, or URL-excluded seeds use unseeded canonical input
with seed=none and the entered offset. For a retained seed needing recovery,
canonical input uses the original URL seed and filter, with offset zero. Keep
that canonical input unchanged after recovery; the recovered ID and effective
context affect selection, not a second hash or initialization.

The caller parses inputs and supplies ordered data. The pure build_round
pipeline owns RNG initialization, recovery, selection, and shuffling; a recovery
helper borrows that same RNG as &mut ChaCha12Rng. The caller must not recover
with an independent RNG and then invoke a builder which seeds another one.
Ordered recovery data and original seed metadata are explicit pure inputs, not
hidden database queries. The implemented API is described below.

A08 implements the nonrandom boundary in zipeek-core/practice: decoded query
parsing, lookup-based normalization, Ready/Unseeded/RecoveryRequired disposition,
canonical v1 bytes and FNV-1a. RecoveryRequired preserves original seed/filter
with offset zero. These helpers never initialize or consume an RNG. A07's
build_round owns all recovery draws.

Implemented API (A07)
build_round(RoundCandidates, ExerciseVariant, &NormalizedPracticeQuery) returns
Result<Round, BuildError>. RoundCandidates borrows an initial CandidatePool,
the original seed row if retained, ordered/exclusion-clean recovery rows and
ContextPool entries for possible recovered contexts. For a Ready seed, initial
views must use its HSK/PoS context; unseeded views use the URL filter. Recovery
prefers HSK 3.0 membership and derives the seed's canonical PoS set, then selects
the supplied views for that context. It never performs a query or rehashes.
The original seed/filter remain canonical even when effective context changes.

The builder rejects unordered/duplicate IDs, inconsistent or nonnested rows,
excluded rows, wrong-system/level/PoS views and missing seed/context data with
actionable errors. It validates caller ordering rather than sorting the views.
SQL construction and exclusion binding remain A06 responsibilities.

RoundContent has Matching columns of four Cards, or an MCQ prompt and three
options with its correct ID. Card labels preserve display text; English uses
only the first stored pipe-separated gloss. Matching reserves never enter the
visible content. BuildTrace records recovery requests, view attempts, partner
and reserve requests, accepted IDs and rejection counts for diagnostics/replay
tests. It is not UI content or persisted state.

The 18 full-round fixtures in tests/fixtures/round-golden.txt lock canonical
bytes/hash, recovery, attempted/accepted IDs and final layouts. Their report
digest is a94efad5496532a0 on native and wasm32. Test-report Debug formatting is
not a URL or production serialization format. See
testing/round-builder.md for the executable wasm
check; it fails if pure fixture execution calls any host import.

Sampling primitive and recovery
Use rand 0.10.3's slice API, importing rand::seq::IndexedRandom:

rust
let candidate = ordered_candidates.as_slice().sample(&mut rng, 1).next();
Each candidate request uses exactly this sample call with amount 1. Do not
substitute iterator sampling, choose, deprecated choose_multiple, or combine
successive requests into a batch. An API request may consume multiple internal
RNG words; the locked order below is the order of API calls, not a promise of
one underlying RNG word per candidate. Check an empty pool before sampling.

Recovery makes at most ten such requests from the unchanged, id-ordered,
exclusion-clean recovery pool. Separate requests may select the same row again.
After each request, check membership and the selected variant's required labels;
stop immediately at the first eligible word. Do not consume the remaining
attempts after success. After ten ineligible candidates, take the first eligible
word by id without any RNG call. An empty pool or no eligible fallback fails
safely. Generation starts at exactly the stream position recovery leaves behind.
If recovery is unnecessary, it consumes no draws.

Generation and final shuffle
MCQ requires one prompt with its correct answer plus two distinct-answer
distractors, using the same view order; it does not require four correct pairs.

Try strict, widened-HSK, and widened-HSK-any-PoS views in that order. For each
view, all candidate slices preserve id ASC order. Eligibility/label checks,
unseeded window sliding, and offset wrapping consume no RNG. The sequence is:

Select correct content. Unseeded matching uses the first valid ordered
four-word window. Seeded matching puts the retained/recovered seed first,
then selects three partners using successive amount-1 sample calls. Start
with the view's ordered candidates excluding the seed; remove each sampled
row from that partner slice, preserving the order of the rest, even when its
right label collides. Accept distinct labels in sampling order; exhaust the
remaining slice safely if three partners cannot be found. MCQ selects its
one prompt at the ordered offset, or uses its retained/recovered seed;
prompt selection consumes no additional RNG.

Select distractors. After all correct content is selected, request four
reserve distractors for matching, or two distractor options for MCQ. Use
successive amount-1 sample calls on the unchanged active view, accepting
candidates in draw order. Reject correct IDs, the seed, and normalized answer
labels already used by correct content or accepted distractors. Count each
rejection against the existing per-view min(candidate_count, 50) retry
limit. Stop immediately at four accepted reserves or two accepted options;
do not make a fifth or third successful selection. Reserves never render.

Finish selection before shuffling. If the view fails, discard its partial
selection, continue to the next view from the current RNG position, and do
no shuffle for the failed view. Recover the seed only once, before view
attempts. Do not hash again or rewind for the new view.

Shuffle the successful result only. Import rand::seq::SliceRandom.
Matching constructs both four-card columns in correct-pair selection order,
calls left.as_mut_slice().shuffle(&mut rng) first, then
right.as_mut_slice().shuffle(&mut rng). Discard the four reserves without
shuffling them. MCQ constructs [correct, distractor_1, distractor_2] and
calls options.as_mut_slice().shuffle(&mut rng) once. Do not shuffle the
prompt, candidate views, or any intermediate list. Return without more draws.

This locks recovery, correct selection, four matching reserves or two MCQ
distractors, then final label layout on one stream. A07/A10 must record full
native/wasm golden rounds and recovery outcomes against these calls; A07 now
records them for all six variants. The hash vector above alone does not establish
full-round replay. Changing this sequence,
sampling API, recovery attempt count, shuffle order, or a tuning value that
changes selection is a replay break subject to the future-format approval path.

Correct pairs, windows, and offsets
For unseeded rounds, begin with the four-word window at the entered strict-pool
offset. Exclusions have already been removed before indexing. If offset + 4 is
beyond the view, wrap the window start to zero; never split a window across the
end of the array. Short views cannot produce a four-word window.

Validate right-label uniqueness in that window. If it fails, advance the start
by one and try again, wrapping to zero as needed. Select the first clean
window, not a best-scoring window. If the per-view cap is exhausted, try the next
view. Both widened views start their scan at zero, regardless of the entered
offset. If an otherwise clean window cannot yield the reserve distractors, advance
to the next view rather than choosing a different best window.

The URL retains the entered offset. Effective window start is internal and is
not persisted. The RNG seed includes the entered strict offset for unseeded
rounds, even if window wrapping/sliding/widening changes the effective start.
Seeded inputs hash offset zero, regardless of an offset in the raw URL.

Each view has its own min(pool.len(), MAX_SLIDE) attempt cap. Initial tuning:

rust
const MAX_SLIDE: usize = 16;
The cap is tunable; deterministic first-window selection and per-view fallthrough
are fixed rules. See DB for tuning and replay
caveats. Nominal Next round offsets still advance by four, but sliding/fallback
can overlap correct sets across rounds; unconditional session-wide non-repetition
is no longer guaranteed.

For seeded rounds, include the seed and sample three others from the active view,
excluding the seed and rejecting labels equal to the seed or another selected
correct word. Use the seeded ChaCha RNG; never slide past a fixed seed. Try the
same fallback views when needed. D15 resolves malformed/missing/excluded seeds
to unseeded inputs before building; Fixed always ignores a seed. Keep the original
entered offset in those unseeded fallbacks. Never reinsert an excluded word into
the bundle to force success.

Distractors and duplicate labels
Draw up to four reserve distractors from the active PoS/HSK candidate view using
the seeded RNG. Exclude all four correct WordIds, the seed, and any normalized
right-label key matching a correct word or an already-selected distractor.
Reject collisions with a bound of min(candidate_count, 50) retries per view.
If still short, widen in the prescribed order and reapply all exclusions/dedup.

Only the actual right-column label participates in uniqueness:

Variant	Right label	Comparison key
eng-pinyin	Pinyin	Canonical numeric tone form, no whitespace, lowercase; e.g. chīfàn becomes chi1fan4
pinyin-hanzi	Simplified hanzi	Exact bytes; no case folding or whitespace stripping
hanzi-eng	Displayed English gloss	Trim, lowercase, collapse internal whitespace
Compare normalized keys byte-for-byte. Homophones collide in eng-pinyin; 行 with
different readings collides in pinyin-hanzi; equal normalized English labels
collide in hanzi-eng. English to eat and eat remain different. Earlier examples
checking the left column were corrected and are not part of the rule. See
DB for the shared normalization contract.

Random entry and failure
Outside the pure builder, Random chooses its variant/seed inputs and captures
them in the URL. Select a seed row from the full HSK-union pool without setup
filters. The requested thread-local randomness is called rand::rng() in the
existing rand 0.10 API. No ambient RNG is allowed inside build_round.

There are two distinct caller paths:

Fresh Random click: ambient randomness is permitted for selecting the
variant/seed. Retry a word without either HSK membership up to ten times, then
choose the first ordered word with a membership. Prefer HSK 3.0 when both
exist. Capture the selected seed and its HSK/PoS context immediately in the URL.

Read an existing URL: no ambient RNG. Malformed/nonexistent/excluded seeds
become unseeded with the same filter and entered offset. A Fixed URL ignores
seed with a debug-only warning. An existing nonexcluded seed without membership
invokes deterministic recovery using a ChaCha RNG derived from canonical URL
inputs, with seeded offset zero. Draw candidate IDs one at a time from the
ordered, exclusion-clean pool, at most ten draws. Reject membership-less or
excluded candidates. If none works, take the first eligible word in id order.
Keep the original seed in the URL and the recovered seed only in effective
state; reloading repeats recovery rather than writing a new random seed.

The caller supplies original normalized URL state and ordered recovery data to
the pure build pipeline. Recovery helpers consume its sole RNG as specified in
the D19 section above; they do not run an independently seeded caller pipeline.
Tests can exercise normalization and the entire recovery/build flow on native
fixture vectors. The same URL, DB snapshot, exclusions, and algorithm reproduce
the recovered seed and round; preserve original URL state for canonical hashing.

An empty recovery pool or no eligible seed must fail safely. D07's generated-table
contract should exclude words without either membership; this is defensive
handling for malformed/legacy data, not permission to import non-HSK words.

D15's Next round on seeded Endless clears the seed and advances the normalized
offset from zero to four for the unseeded next round, keeping its filter. Raw
ignored offsets must not affect that transition.

If all three candidate views fail, return Err(BuildError::InsufficientPool { .. }).
Show "Not enough distinct words for this filter. Try a different HSK level or
reset exclusions." Preserve the Reset controls from D13. This replaces the older
D04 proposal to escape into a separate unfiltered round after pool failure.
Expected large pools never replace a tested, nonpanicking failure path.

Approved RNG provider and reproducibility checks (D17)
Use the already-declared rand 0.10.3 with its chacha feature. The intended
manifest configuration is:

toml
rand = { version = "0.10.3", features = ["chacha"] }
The user approved moving this dependency from web to zipeek-core for A07.
The core manifest now enables chacha; the pure builder and portable full-round
vectors are implemented. Web retains its existing getrandom wasm_js support.
Do not add rand_chacha: D17 supersedes that earlier D12 proposal. Reuse
rand::rngs::ChaCha12Rng and rand::SeedableRng instead of a separate provider.

The rand 0.10.3 RNG documentation
lists ChaCha8Rng/ChaCha12Rng/ChaCha20Rng behind chacha as named portable
generators. Rand's backend uses chacha20; no additional direct ChaCha crate is
needed for this design. Stable rounds additionally depend on exact
sampling/shuffling algorithms, seed expansion, feature settings, normalization,
input ordering, and tuning. The Rand reproducibility policy
permits value changes between minor library releases. Control dependency versions
and verify full-round vectors on native and wasm32; a named RNG alone is not proof.

The requested choose_multiple sampling is renamed to sample in rand 0.10;
the old alias is deprecated and would conflict with warnings-as-errors. The
rand API and
thread-local RNG docs
describe the current names: use sample for sampling and rand::rng() for the
allowed fresh-Random path. D19 above fixes sampling/shuffle/recovery call order
and the only current format, v1. Do not silently change versions
or substitute StdRng to make it compile.



## Future duplicate-hanzi matching requirement (D42)

The user approved avoiding different readings with identical simplified hanzi
in the four target pairs of a matching round after reporting jing/Jing. A55
tracks this future policy. Current v1 implementation still applies the
right-label-only rules above; no selection, draw-order or canonical encoding
was changed by A53. Before altering the builder, present the replay impact and
obtain explicit format/compatibility approval required by AGENTS. Do not merge
source reading identities. See [FUTURE](FUTURE.md#matching-feedback-and-ambiguous-entries).
