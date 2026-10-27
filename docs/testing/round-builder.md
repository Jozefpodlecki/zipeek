# Round builder checks

D12 and D19 coverage for the pure round builder. Part of the
[testing index](TESTING.md). Approved requirement in
[D12](../process/decisions/rounds-and-replay.md#d12-deterministic-round-builder)
and [D19](../process/decisions/rounds-and-replay.md#d19-v1-replay-stream-and-draw-order).

## Deterministic round builder coverage (D12)

Use plain `CandidatePool`/`Word` fixtures for native unit tests; no wasm is needed
to exercise the pure builder. A07 also executes the shared full golden outputs
on wasm32 with the approved dependency versions/features.

Verify the generator is `rand::rngs::ChaCha12Rng` with rand 0.10.3's `chacha`
feature, seeded through `rand::SeedableRng`. No direct `rand_chacha` dependency or
`StdRng` substitution. The feature is approved and enabled in the core manifest
after the A07 move; the builder exists in `core/round`. Use the current `sample`
and `rng()` APIs rather than deprecated aliases, retaining the existing
`-D warnings` gate.

- Permute PoS insertion order and include duplicate PoS inputs: canonical bytes,
  FNV-1a, and round output must match. Empty PoS produces an empty field, and an
  absent seed is literal lowercase `none`, not Debug's `None` or `{}`.
- Pin the canonical encoding with the fixed ASCII string
  `zipeek-round-v1|eng-pinyin|hsk3|3|noun,verb|10|none` and expected `u64` hash
  `0xa0e531d76df62844`. Changed separators, tokens, ordering, or whitespace must fail.
- Vary raw URL offset for a seeded round: boundary normalization forces zero
  before canonical formatting, so the hash and round stay identical. Unseeded
  canonical input retains entered offset even after sliding or fallback.
- Verify each caller query applies exclusions and `ORDER BY id ASC` in SQL,
  and produces nested strict/widened/any-PoS views. Check same-system boundaries:
  HSK 2.0 level 6 -> {5,6}, level 1 -> {1,2}; HSK 3.0 level 7 -> {6,7},
  level 1 -> {1,2}. No level 8/9 or cross-system candidates.
- A fixture whose first clean window is at slide 3 must choose it. A view with
  no clean window must fall through to the next view, independently of the cap's
  numeric value. Each fallback starts at zero and has its own attempt cap.
  Do not assert `MAX_SLIDE == 16` or test the constant instead of the behaviour.
- Check exclusion-before-offset indexing, wrap when fewer than four entries
  remain at the entered position, first-clean-window selection, retained entered
  URL offset, and internal-only effective start.
- Seeded fixtures always retain the seed, select three distinct-label partners
  with ChaCha, and never slide. Distractors exclude all correct IDs/seed and
  normalized-label collisions; rejection is bounded by `min(candidate_count,50)`,
  then falls through and reapplies dedup in the next view.
- Test actual right-column keys only: homophones collide for eng-pinyin;
  行 with different readings collides for pinyin-hanzi; English case/whitespace
  variants collide for hanzi-eng. `to eat` versus `eat` remains distinct.
- Empty/pathological bundles exhaust all views into `InsufficientPool` without
  panic. UI displays the distinct-words message and level/reset recovery controls.
- Random entry retries seeds lacking membership at most ten times, then uses
  the first eligible ordered word; if none exists, fail safely. Prefer HSK 3.0
  for dual memberships. Ambient RNG must not be used inside the pure builder.

See [ROUND_GENERATION](../product/ROUND_GENERATION.md) for the complete contract
and replay limitations of changing dependencies, tuning, or the v1 algorithm.

## Single-stream and call-order vectors (D19)

- Record original canonical bytes/hash, recovered ID, accepted/rejected candidate
  requests, selected correct IDs/reserves/options, and final label layout for
  native and wasm32 fixtures using rand 0.10.3. Do not call these tests passing
  until the builder and fixtures exist.
- Cover recovery succeeding on attempts 1 and 10, ten failures followed by the
  first eligible id, empty/no-eligible recovery pools, and D18 missing-English
  recovery. Verify early success consumes no further recovery requests and the
  by-id fallback consumes no RNG. Hash the original URL seed, not the recovered ID.
- Verify one RNG initialization and continuation into generation, including
  failed views; no reseeding, cloned-stream replay, or caller-owned second RNG.
- Match vectors must lock correct selection, four accepted reserve distractors,
  then left shuffle followed by right shuffle; unseeded windows use no RNG.
  Seeded partner draws remove attempted rows while preserving remaining id order.
- MCQ vectors must lock prompt selection, two accepted distractors, then one
  shuffle of initially ordered `[correct, distractor_1, distractor_2]`.
- Include collisions, the bounded rejection limit, failed-view continuation
  without shuffling, repeated recovery candidates, and an unchanged amount-1
  slice `sample` primitive. API attempts are not counts of internal RNG words.
- Keep the existing v1 hash vector. Verify no `v=` selector/parser is added; a
  future replay-changing sampler/order/tuning change requires the approved
  prefix/URL-selection/compatibility path before replacing golden outputs.

## A07 evidence

All five gates pass: fmt, workspace Clippy with warnings denied, workspace tests
(84 domain/generator tests plus five Chrome tests), wasm check and `/zipeek/`
Trunk build. Nineteen new tests cover the algorithm, malformed inputs, failures,
first-gloss/right-label semantics, exclusions and exact recovery draw consumption.
Eighteen full-round vectors execute on native and wasm32 with matching digest
`a94efad5496532a0`; they include all variants, seeded/unseeded layouts, recovery
success on draws 1/10, ten-failure ordered fallback, missing-English recovery,
sliding/widening and failed-view stream continuation. The Node runner traps any
host call. The published canonical input hash remains unchanged.

Workspace log: `target/a07-workspace-tests.log`. The owned hidden Trunk server
was stopped and absence confirmed. Schema, source snapshots, generated dictionary,
original background and replay contract are unchanged. No commit/staging action
was performed; prior user changes/history were preserved. A06/A21 must supply
real candidate views; A09/A10 still own playable UI. This task does not claim
browser database or gameplay integration.

## Current status

The pure builder exists in `crates/zipeek-core/src/round/` (A07). The executable
wasm replay check is documented in
[browser-db.md](browser-db.md#executable-nativewasm-replay-check-a07). Database
and UI integration remain A21/A09/A10 and are not claimed by the pure builder's
passing tests.