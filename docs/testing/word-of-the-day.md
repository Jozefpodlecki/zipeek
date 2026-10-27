# Word of the Day checks

D05 coverage plan. Part of the [testing index](TESTING.md). Approved requirement
in [D05](../process/decisions/foundations.md#d05-word-of-the-day); hashing
contract in [D14](../process/decisions/rounds-and-replay.md#d14-reproducible-hashing-and-date-override).

## Required coverage

- Navigate to `http://127.0.0.1:<port>/?wotd=2026-10-26` in debug e2e. Assert the
  specific expected word for the known fixture pool, approved hash, and ordering;
  reloading the same date/pool must reproduce it using D14's 64-bit FNV-1a.
- Check browser-local date handling, including a case where local and UTC calendar
  dates differ. The override represents a calendar date, not a UTC timestamp.
- Verify the full pool is used independently of practice filters, and offset
  selection uses deterministic ordering with tie handling.
- An empty pool shows "No word available" without a panic; a DB load failure
  shows the same card plus a working retry action.
- Verify release/production builds ignore `wotd` regardless of validity. Ordinary
  debug Trunk e2e alone cannot establish that release-build behaviour; use an
  appropriate separate release check when implementing the feature.
- Verify the exact FNV-1a basis/prime with fixed input/output vectors, wrapping
  multiplication, UTF-8 inputs, and `u64` arithmetic independent of pointer width.
  Word of the Day must never use `DefaultHasher` or persist hash state.
- In debug builds, wrong format, impossible dates such as `2026-13-40`, and dates
  outside the supported range silently fall back to real local date. No page error
  or panic; any warning is debug-only. No testing Cargo feature is needed for v1.

## FNV-1a reference

For date string `yyyy-mm-dd` as UTF-8 bytes:

```text
basis   = 0xcbf29ce484222325
prime   = 0x100000001b3
for each byte b:
    h = h XOR b
    h = h * prime   (wrapping multiply, u64 arithmetic)
offset  = h mod count

Use u64 for hash arithmetic and modulo reduction; do not use usize. 32-bit
and 64-bit builds must agree.

DefaultHasher is forbidden for Word of the Day and any reproducible value. It
is permitted only for ephemeral in-session use such as a HashMap.

Debug date override
/?wotd=2026-10-26 forces that local calendar date in debug builds; the
override represents a calendar date, not a UTC timestamp.

Invalid overrides silently fall back to the real local date; do not panic or
show a page error. Optional warn! is allowed only in debug.

Release builds may parse the query string but ignore wotd entirely: zero
behavioural effect in Pages deploys.

No testing Cargo feature is needed for v1.

Current status
D05 is approved. Selection logic exists in
A06 browser database queries (count,
ordered offset, typed row). A12 still owns the UI wiring of the daily card
(DailyWordCard currently renders "No word available" with no query or retry).
This file describes the coverage required once A12 lands; it is not evidence
that Word of the Day is implemented.