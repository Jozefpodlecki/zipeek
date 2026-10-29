# Matching and mixed-session checks

A09 implements the three four-pair variants. A48 adds mixed matching/MCQ sessions
under D40. [UX](../product/UX.md#match-interaction-ux-match01) owns interaction
rules; D39 clarifies both-pair first-attempt disqualification.

## Focused commands

From the repository root, using the owned headless Chrome harness:

```powershell
cargo test -p zipeek --lib --locked
cargo test -p e2e --test matching --locked
cargo test -p e2e --test mixed --locked
```

The matching binary runs three named scenarios; the mixed binary runs two.
Pages runs the same five alongside its existing ten. Each binary owns one
managed Chrome session plus isolated server/build lifecycle. Do not start a
separate ChromeDriver or external Trunk server for these tests. Use D36 focused
commands during routine iteration; full coverage belongs at major milestones.

## Native behavior

- Matching selection: any side first, same-side alternative is ignored, same
  card toggles off, Escape clears, invalid/stale choices are ignored.
- Wrong attempt locks input until its guarded timer resolves, adds one Fixed
  mistake and disqualifies both represented pairs. Repeated commands do not
  double count. Correct retries still contribute to live correct/streak.
- Only four matched pairs permit advancement. Fixed progress survives wrapping,
  final completion freezes time once, replay resets and old timers cannot clear
  a later round. WordIds/order and deterministic builder behavior stay unchanged.
- Shared active-round session: matching/MCQ transitions retain correct, eligible
  first-attempt, mistakes, streak/highest, progress and time. A five-item mixed
  example yields 11 / 14, not 11 / 20. Duplicate Ready/answer and stale commands
  do not count; external changes reset.
- Selected types are nonempty/canonical, omit the URL field for single types,
  ignore unknown/duplicate tokens safely and preserve the current-round canonical
  bytes. Random choices exclude the previous type except in a single-type set.

## Chrome behavior

Matching checks cover all three variants, eight stable card nodes, native Space/
Enter and Escape, input locking/feedback, paired completion, streak/flame, nominal
offset +4, saved URL exclusions, reload, seeded Next and overflow wrapping.
Fixed checks cover 5/10/20 across variants, first-attempt/mistakes/highest/timing,
fresh replay, chart teardown, and empty-strict-view same-URL continuation. Reduced
motion removes shake/background transitions; Listen calls zh-CN simplified hanzi without selecting.
400/768/1280px light/dark screenshots assert no horizontal overflow and visible
Next. Insufficient pools show setup recovery rather than a fatal error.

Mixed checks cover multi-toggle setup including last-selection retention, random
non-repeating type transitions, shared progress and weighted summary, mixed replay
and default setup return, single query encoding, raw-link preservation, seeded
Next, current-round reload determinism and malformed types fallback. Controlled
entropy and wall clocks make assertions stable without changing the round RNG.

Route navigation keeps AppShell's initial main-content focus. Later matching
rounds focus their heading; MCQ prompts and summaries keep their feature focus.
The broader Pages regression caught the initial matching focus conflict; its
exact route/retry scenario is rerun directly after correction, retaining other
successful scenario evidence instead of restarting the whole regression.


## A49 presentation sample

The interaction/layout scenario also captures fresh cards at 400/1280px in both
themes and the hanzi board at 1920px. It checks the 896px cap, opaque board,
equal card heights within rows, contained Listen buttons, no horizontal overflow
and pinyin beneath hanzi on Hanzi-to-English cards. The same sample visits
Hanzi-to-meaning MCQ to verify/capture its pinyin placement. A52 changes desktop
samples to 1280?720/1920?720 and asserts no document vertical scroll. Matching
pinyin remains left of the right-edge audio; MCQ retains pinyin above audio. Reduced-motion
recovery checks expect opaque, readable matched cards and no animation.
