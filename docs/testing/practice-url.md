# Practice URL checks

D13/D15 URL, exclusion, offset, and fallback coverage. Part of the
[testing index](TESTING.md). Approved requirements in
[D13](../process/decisions/rounds-and-replay.md#d13-counts-urls-exclusions-offsets)
and [D15](../process/decisions/rounds-and-replay.md#d15-url-defaults-and-session-lifecycle).

## URL, exclusion, and offset coverage (D13)

- Round-trip mode, Fixed count, HSK context, exclusions, and unseeded offset or
  seed. Verify malformed/unknown values safely default to Endless, 10 rounds,
  HSK 3.0, level 1, and offset 0. Invalid level must never broaden to all levels.
- Test HSK 3.0's combined advanced tier as `level=7`, alias `7-9` normalization,
  UI display as one 7–9 band, and rejection of separate levels 8/9. HSK 2.0 has
  no level 7. PoS strings use the DB's canonical lowercase vocabulary; comma sets
  retain known entries and ignore unknown entries; empty/absent means no filter.
- Seeded Random emits the seed's PoS and HSK membership, including advanced tier
  7 where applicable. A later setup PoS picker must use the same URL contract.
- Require counts only for Fixed; ignore them for Endless. Seeded rounds use
  Endless and effective offset zero. Fixed+seed ignores seed; invalid/excluded
  seed becomes unseeded while preserving the entered filter and offset.
- Verify same-URL/same-DB replay across reloads and browsers using D12's canonical
  input and ordered candidate views. Use D17's rand 0.10.3/chacha provider and
  the approved D19 draw/shuffle sequence when recording full-round golden vectors.
- Store excluded WordIds in localStorage, serialize nonempty comma-separated IDs
  in links, omit empty `exclude`, and filter before all candidate selection/widening.
  Test setup's excluded count and Reset, and the insufficient-after-exclusions
  message/reset action. Generated URLs union saved/current-round IDs; loaded
  URLs use exactly their own set, and absent `exclude` means empty. Test sharing
  across browsers with different saved sets, no writes on link load, unknown IDs,
  deduplication, and noninteger entries dropped with debug-only warnings. Marking
  a new exclusion must not change current URL/cards; it applies to the next URL.
  Setup Reset clears storage/pending state and emits a URL without `exclude`.
- Verify offsets 0, 4, 8; distinct unchanged strict slices until exhaustion; wrap
  at offset + 4 > pool_size; one looping hint; Play again resets to zero. Fixed
  must still finish after N rounds even when wrapping occurs. Reload resets
  in-memory progress while honoring URL offset; do not infer progress from a
  wrapping offset. Sliding and widened offset-zero fallback may overlap correct
  sets before wrap.

## URL fallback and session coverage (D15)

- Missing, malformed, out-of-range, or wrong-system levels use 1 in the selected
  HSK system, with debug-only warnings for invalid values. No panic or all-levels
  fallback; test `level=99`, `level=abc`, and HSK 2.0 `level=7`.
- Malformed/nonexistent seed, or a seed in the URL exclusion set, becomes unseeded
  with the original filter/offset. Exclusion only in another browser's saved set
  must not change that shared round. Validate seed before offset-zeroing.
- Fixed+seed preserves Fixed/count/filter/offset, ignores seed and warns only in
  debug. The next generated URL omits the ignored seed.
- Membership-less seed fixtures: two independent URL-normalization/build pipelines
  produce identical recovered seed and Round. Ten ineligible ChaCha draws must
  fall back to the first eligible word by id. No ambient RNG on URL reads and no
  URL replacement with the recovered seed. An excluded seed takes the unseeded
  path instead of membership recovery. Cover an empty recovery pool safely.
- Mark a word mid-round: saved/pending state changes, active URL/cards do not.
  Next URL includes the new exclusion; Setup Reset clears it and omits `exclude`.
- Reload Fixed at a nonzero offset: progress starts at zero while the pool offset
  is preserved. Wrap mid-session without resetting progress; finish after N rounds.
  Do not persist the counter or score.
- Seeded Next produces unseeded Endless with the same filter and offset 4 before
  normal pool handling. No seed or raw ignored offset survives into that round.

## A08 evidence

All five gates pass with 65 domain/generator tests and five Chrome tests
(`target/a08-workspace-tests.log`). Ten new pure practice tests cover canonical
v1 bytes/hash, strict malformed values, HSK-specific defaults, recognized PoS
retention, exclusions, Fixed seed handling, original recovery intent, offset
wrapping and independent Fixed completion. Chrome exercises the real router
decoder with percent-encoded commas and verifies malformed defaults and unchanged
shared URLs. A07 covers recovery draws and full-round golden vectors;
database-backed normalization remains A06/A21. No dependency was added.

## Current status

A08 implements the pure parsing, normalization, canonical encoding and session
state helpers in `crates/zipeek-core/src/practice/`. A21 supplies dictionary rows
and calls normalization/candidate queries/build_round for all six routes. A07
continues to own every recovery/selection RNG draw. Chrome verifies full card
ID/label replay, Fixed seed ignoring, unknown/excluded seed fallback, ID validation,
unchanged loaded URLs and saved-exclusion independence at root and Pages. These checks describe required coverage; they are
not all passing today. Where a check is blocked by an unimplemented feature,
report it as blocked rather than passed.

## A11 generated setup URLs

Native tests cover invalid/stale cross-system level selection, advanced-band
fallback, count preservation across modes, and canonical omission of seed/PoS/
empty exclusions/Endless count. Saved-ID parsing tests reject noninteger/string/
invalid/out-of-range inputs and deduplicate typed IDs. Root/Pages setup scenarios
start every variant through the UI and validate actual decoded query values,
count/exclusion semantics and dictionary-backed Ready preparation. Unknown saved
IDs do not enter Start, and initial validation never writes storage. Reset updates
the next URL to omit exclude; shared practice URLs still ignore saved state.
