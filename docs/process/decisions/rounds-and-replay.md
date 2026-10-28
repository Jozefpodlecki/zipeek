# Rounds and replay

Approved round-construction, URL, and RNG decisions. Each entry links back to
the [index](index.md) and to its owning document. Bodies are the approved
requirement as recorded on the date shown; they do not imply the feature is
implemented.

---

## D12 Deterministic round builder

**Status:** Approved 2026-10-26; refined by D17 and D19.
**Owner:** [ROUND_GENERATION](../../product/ROUND_GENERATION.md),
[DB](../../architecture/schema/DB.md), [TESTING](../../testing/round-builder.md).
**Index row:** [D12](index.md#approved-decisions).

Pure `CandidatePool` function; three exclusion-clean SQL-ordered nested views;
canonical `zipeek-round-v1` string and FNV-1a to named `ChaCha12Rng`; no `StdRng`.
Deterministic sliding/sampling, seeded offset zero, bounded rejection, clamped
widening, membership retries/preference, right-label normalization and typed
failure. D17 selects rand's provider; D19 fixes the v1 stream and draw order.

Key contract points:

- **Pure function.** `build_round` never queries DuckDB, reads the clock or
  localStorage, or uses ambient randomness. Testable natively with plain Word
  fixtures.
- **Three views.** `strict`, `hsk_widened`, `hsk_widened_any_pos`; nested
  supersets. Every query removes exclusions in SQL and includes `ORDER BY id ASC`.
  Widening is clamped within the same HSK system; never crosses systems.
- **Four visible cards per side.** Four correct pairs + four internal reserve
  distractors per column. Reserves never render; only four cards per side.
- **Right-label uniqueness.** Only the actual right-column label participates;
  comparison normalizes per variant (`eng-pinyin`: numeric tone, no whitespace,
  lowercase; `pinyin-hanzi`: exact bytes; `hanzi-eng`: trim, lowercase, collapse
  whitespace).
- **Seeded rounds never slide.** Unseeded windows slide past duplicate labels
  (bounded by `MAX_SLIDE`); widened views start at zero.
- **Typed failure.** `BuildError::InsufficientPool { .. }` when no view can
  supply the required candidates; the UI shows the distinct-words recovery
  message.

D17 fixes the RNG provider; D19 fixes the single-stream draw order and v1 as
the only format.

---

## D13 Counts, URLs, exclusions, offsets

**Status:** Approved 2026-10-26; refined by D15.
**Owner:** [PRODUCT](../../product/PRODUCT.md), [DB](../../architecture/schema/DB.md),
[TESTING](../../testing/practice-url.md).
**Index row:** [D13](index.md#approved-decisions).

Fixed 5/10/20, default 10, no custom count. Canonical HSK/level and optional PoS
set, exclusions, offset or Endless seed in URLs. Generate exclusions from
saved/current/pending union; load exactly the URL set (absent means empty), never
merge saved state. D15 defers new exclusions until the next URL. Deterministic
replay and four-step nominal offsets. Advanced tier uses 7 (7-9 alias).

Amendment recorded by A22 on 2026-10-27: mid-round exclusion affordance
deferred, not part of v1. Keep the saved/pending lifecycle below; setup Reset
remains available. No replay bytes, comparison key, RNG behaviour or dependency
changes are approved by this amendment.

Canonical unseeded URL:

```text
/practice/<variant>?mode=<endless|fixed>&rounds=<5|10|20>&hsk=<2.0|3.0>&level=<1..7>[&pos=<noun,verb>][&exclude=<id,id,id>][&offset=<n>]
```

rounds included iff mode=fixed; ignored in Endless.

Defaults for missing/invalid values: mode=endless, rounds=10, hsk=3.0,
level=1, offset=0.

pos is a comma-separated set of canonical lowercase enum strings; omitted
or empty means all; unknown tokens are dropped; all-unknown means no filter.

exclude is comma-separated integer WordIds; omit empty rather than emitting
exclude=.

offset is a nonnegative index into the filtered, ordered pool, not a word ID.

Seeded rounds require Endless; Fixed+seed ignores the seed with a debug-only
warning. Offset is zeroed only after a seed is validated and retained.

Exclusions have two lifecycle rules:

Generate a URL — union saved, current-round, and pending sets; the emitted
URL carries the complete set and is self-contained.

Load a URL — the round's set is exactly the query's exclusions; saved
exclusions never augment a loaded URL. Absent exclude means empty.

Marking a new exclusion changes saved/pending state for the next generated URL
only; current-round cards, active inputs, and current URL are unchanged. Setup
Reset clears saved/pending sets and generates a new URL without exclude.

Nominal offsets advance by four per round; offset + 4 > pool_size wraps to
zero with a one-time "pool exhausted, looping" hint. Fixed completion tracking
is independent of offset wrapping.

## D14 Reproducible hashing and date override

Status: Approved 2026-10-26.
Owner: AGENTS, PRODUCT,
TESTING.
Index row: D14.

64-bit FNV-1a over UTF-8 date bytes:

text
basis   = 0xcbf29ce484222325
prime   = 0x100000001b3
for each byte b: h = (h XOR b) * prime   (wrapping multiply, u64 arithmetic)
Zero dependencies. Implement as a small dependency-free Rust const fn.

Use u64 for hash arithmetic and modulo reduction; do not use usize. 32-bit
and 64-bit builds must agree.

DefaultHasher is forbidden for Word of the Day and any reproducible value.
It is permitted only for ephemeral in-session use such as a HashMap.

Do not persist hash state anywhere.

Invalid debug override silently uses the real local date; release builds
ignore wotd entirely. No testing Cargo feature in v1.

## D15 URL defaults and session lifecycle

Status: Approved 2026-10-26.
Owner: PRODUCT, ROUND_GENERATION,
TESTING.
Index row: D15.

Level defaults to 1 in the selected system; malformed/missing/excluded seed
becomes unseeded with the same filter/offset; Fixed ignores the seed.
Membership-less URL seeds use deterministic ten-draw ChaCha recovery then the
first eligible id, retaining the original URL. New exclusions are pending until
the next URL; Setup Reset clears storage and omits exclude. The Fixed counter
is memory-only; reload resets progress but honors offset. Seeded Next clears the
seed and advances by four.

Normalization order (URL loads):

Parse mode/count, HSK system/level, PoS, exclusions, and entered offset with
the defaults above. The URL's exclusions are authoritative, not saved
exclusions. Invalid levels use 1, with a debug-only warning.

If Fixed, ignore any seed before seed lookup or offset-zeroing. Keep Fixed,
its count, the same filter, and the entered offset.

Otherwise, absent/malformed/nonexistent/excluded seeds fall back to unseeded
with the same filter and entered offset. Exclusion only in saved state must
not invalidate a shared URL's seed.

An existing, nonexcluded seed with neither HSK membership uses deterministic
recovery: up to ten ChaCha draws from the ordered, exclusion-clean pool, then
the first eligible word by id. No ambient randomness on URL loads. Keep the
original bad seed in the URL; recovery is internal and reproducible.

Only a retained/resolved seed normalizes offset to zero before canonical RNG
input construction. A rejected seed must not erase the unseeded fallback's
entered offset.

Fixed progress and scores are in-memory only; reload resets progress but honors
the URL's pool offset. Wrapping does not reset the session counter. Seeded Next
clears seed, keeps the filter, and advances the normalized offset from zero
to four for the unseeded next round. Raw ignored offsets must not affect that
transition.

## D17 RNG provider

Status: Approved 2026-10-26; supersedes D12's direct-crate proposal.
Owner: AGENTS, ROUND_GENERATION,
TESTING.
Index row: D17.

Use existing rand 0.10.3 with the chacha feature, explicitly approved:

toml
rand = { version = "0.10.3", features = ["chacha"] }
Import rand::rngs::ChaCha12Rng and rand::SeedableRng; do not add rand_chacha.
Use current rand 0.10 API names (sample / rng()), not the deprecated
choose_multiple alias. This supersedes D12's direct-crate proposal; D19 fixes
the algorithm/replay call order and the sole v1 format.

The user approved moving rand from web to zipeek-core for A07. web
retains its existing getrandom wasm_js support. The core manifest enables
chacha. Do not silently substitute StdRng or change versions.

## D19 V1 replay stream and draw order

Status: Approved 2026-10-26; supersedes the earlier speculative v2 forecast.
Owner: ROUND_GENERATION,
TESTING, AGENTS.
Index row: D19.

Keep v1 as the only format, with no version URL parameter/parser today.

Exactly one ChaCha12Rng per build_round, seeded once with
ChaCha12Rng::seed_from_u64(fnv1a64(canonical_seed_input)).

Recovery consumes draws from this same stream; generation continues from the
resulting state. No reset, no second RNG, no re-seed — including on candidate-
view fallback. Never clone the RNG to restart a phase.

Recovery makes up to ten amount-1 slice sample calls, stops at the first
eligible word, and uses first-eligible-by-id fallback without any RNG draw.

Generation continues on that stream: correct content (unseeded window or
seeded partners), four matching reserves or two MCQ distractors, then final
shuffles in the exact order specified in
ROUND_GENERATION.

Lock full native/wasm golden outputs during implementation; A07 records 18
full-round vectors with digest a94efad5496532a0.

Future replay breaks require a v2 prefix, a selecting URL parameter, documented
compatibility, and explicit approval for the shared-URL break. None of that v2
machinery exists today; do not add a v= parameter or version parser
speculatively. The earlier COMPLETED v2 forecast is superseded by an append-only
correction, not a commitment to implement v2 now.

Authoritative single-stream contract lives at AGENTS §5;
if this file and AGENTS conflict, AGENTS wins.

## A37 display clarification to D12 — 2026-10-28

Traditional hanzi is secondary presentation on hanzi-labeled match/MCQ surfaces.
Primary simplified labels and existing comparison/uniqueness keys remain unchanged.
Do not concatenate traditional text into core Card.label, hash/canonical bytes
or answer keys. This is not the earlier conditional composite-label proposal;
there is no replay break or v2 selection.
