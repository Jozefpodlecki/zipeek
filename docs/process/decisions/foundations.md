# Foundations

Approved product and shared-domain decisions. Each entry links back to the
[index](index.md) and to its owning document. Bodies are the approved requirement
as recorded on the date shown; they do not imply the feature is implemented.

---

## D01 Shared crate

**Status:** Approved 2026-10-26.
**Owner:** [ARCHITECTURE](../architecture/ARCHITECTURE.md).
**Index row:** [D01](index.md#approved-decisions).

Use `zipeek-core` for shared domain/query types. The old `zipseek-core`
reference is a typo; implementation and dependency changes remain separate work.

`zipeek-core` must compile on both `wasm32-unknown-unknown` and native targets,
and must not import `web_sys`, `js_sys`, or `wasm_bindgen`. It holds types and
pure logic only; queries take data as arguments, never a database handle. Public
API is versioned by the repository, not semver; breaking changes are acceptable
during development (D10).

Implementation status: Word/WordId, HSK system/levels, all 42 PartOfSpeech values,
practice URL state and the pure deterministic round builder are implemented.
SQL query and playable exercise integration remain future work.

---

## D02 Round generation

**Status:** Approved 2026-10-26; amended by D12 and D13.
**Owner:** [PRODUCT](../product/PRODUCT.md), [ROUND_GENERATION](../product/ROUND_GENERATION.md).
**Index row:** [D02](index.md#approved-decisions).

Originally random correct four from HSK + PoS; **D13 supersedes unseeded selection
with deterministic ordered slices.** The following remain in force:

- Same-pool reserve candidates.
- Eight-unique-candidate target (four correct pairs + four distinct-label reserves).
- Widen HSK level ±1 within the same system, then drop PoS.
- Bounded resampling; typed insufficient-pool error.
- Four visible cards per side; internal reserves never render.

D12 refines the algorithm ([ROUND_GENERATION](../product/ROUND_GENERATION.md));
D04's later refinements (membership retries, typed pool failure replacing the
unfiltered escape hatch) also apply.

---

## D03 Practice progression

**Status:** Approved 2026-10-26.
**Owner:** [PRODUCT](../product/PRODUCT.md).
**Index row:** [D03](index.md#approved-decisions).

Offer Endless and Fixed on setup.

- **Endless:** Next round between rounds. No summary, no persisted scores. A
  running streak/tally in the header is allowed.
- **Fixed:** choose N rounds up front (D13 fixes counts to 5/10/20, default 10).
  After round N, show a summary of correct pairs, total mistakes, and time taken,
  with Play again and Back to setup actions.

---

## D04 Random tile

**Status:** Approved 2026-10-26; amended by D12.
**Owner:** [PRODUCT](../product/PRODUCT.md), [ROUND_GENERATION](../product/ROUND_GENERATION.md).
**Index row:** [D04](index.md#approved-decisions).

Shared generator, random variant and whole-DB seed, seed-anchored HSK/PoS,
ignore remembered filters, always Endless.

D12 refines membership retries/preference and replaces the original unfiltered
escape hatch with a typed pool failure. The seed's attributes become the
effective filter; there is no separate Random-specific round-building
implementation.

---

## D05 Word of the Day

**Status:** Approved 2026-10-26; parameters fixed by D14.
**Owner:** [PRODUCT](../product/PRODUCT.md), [DB](../architecture/schema/DB.md),
[TESTING](../testing/word-of-the-day.md).
**Index row:** [D05](index.md#approved-decisions).

Full-pool pick using browser-local `yyyy-mm-dd`, date hash modulo count,
deterministic SQL ordering, neutral empty state, load-failure retry, and
debug-only `wotd` override.

- Select from the full word pool, independently of practice HSK/PoS filters.
- Read the browser's local calendar date using `js_sys::Date`; format as
  `yyyy-mm-dd`. Do not substitute a UTC date.
- Count with `COUNT(*)`; when zero, show "No word available" without modulo or
  offset work.
- Otherwise use D14's 64-bit FNV-1a over the UTF-8 date bytes, reduce modulo the
  count, and pick that offset in a deterministically ordered query.
- Ordering: `ORDER BY simplified, pinyin` (D07). Never `LIMIT 1 OFFSET n` without
  deterministic ordering.
- DB load failure shows the same placeholder plus Retry. An empty successfully
  loaded pool does not require the failure-only retry state.
- Debug override `/?wotd=2026-10-26` forces that local calendar date; release
  builds ignore `wotd` entirely. Invalid overrides silently use the real local
  date. No testing Cargo feature in v1.

D14 fixes FNV-1a exactly and forbids `DefaultHasher` here.

---

## D10 Development lifecycle

**Status:** Approved 2026-10-26.
**Owner:** [AGENTS](../../../AGENTS.md), [DB](../architecture/schema/DB.md).
**Index row:** [D10](index.md#approved-decisions).

Breaking changes are acceptable during development; regenerate data/artifacts.
No app/database versioning or backward-compatibility requirement for old
generated outputs. Existing dependency controls still apply.

Narrow exception: D12/D19's explicitly versioned round-seed encoding must
preserve replay-format compatibility. A future v2 format requires a canonical
prefix bump, a URL parameter selecting it, and a documented compatibility path,
all with explicit approval. No v2 format, version-selection parameter, or
compatibility parser exists today.

This does not relax dependency pins, approval requirements, source licensing,
COMPLETED history preservation, or the rule against hand-editing DuckDB files.

---

## D11 HSK abstraction

**Status:** Approved 2026-10-26.
**Owner:** [PRODUCT](../product/PRODUCT.md), [DB](../architecture/schema/DB.md),
[ARCHITECTURE](../architecture/ARCHITECTURE.md).
**Index row:** [D11](index.md#approved-decisions).

Distinct HSK 2.0 (1-6) and HSK 3.0 (1-6 plus advanced tier 7-9) memberships.
A word may be in both at different levels, or only one.

- HSK 2.0 has levels 1-6.
- HSK 3.0 has levels 1-6 plus a combined advanced tier 7-9.
- Stored as `Hsk3Level::SevenToNine`, mapped to integer 7 in storage and canonical
  URLs. Values 8 and 9 are not separate stored levels.
- A single universal integer level cannot represent independent memberships.
- Do not infer membership or convert a level from one system to the other.

D16 resolves conflicting within-system levels to the lowest value with a warning;
identical repeated tags are deduplicated.