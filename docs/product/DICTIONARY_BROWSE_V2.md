# Dictionary Browse v2 — approved UX and implemented slice

Approved 2026-11-04. The design reference is the dark-theme Dictionary mockup.

## Responsibilities

Browse is for **finding, hearing, inspecting and collecting words**, not for
choosing quiz types or launching an ambiguous one-word quiz. Any selected Word
Set can be handed off to Practice with its identity preserved. Practice then
chooses exercise types, mode and any relevant specialized configuration.

## Results presentation (R79 implemented)

- Five visible rows per page; issue the database query with one lookahead row
  so Next is enabled only if another page exists. Keep Previous/Next inside
  the result panel and show the current page number. No synthetic total count.
- Each list row places simplified hanzi and pinyin **on the same line** when
  horizontal space allows; traditional characters remain supplemental.
  Meanings wrap. Long definitions do not force horizontal page overflow.
- Each row has an accessible **Listen** icon using the existing `ListenButton`
  and Web Speech synthesis `zh-CN`, plus Add/Remove from the chosen Word Set.
  Do not imply TTS provides authoritative tone/reading validation for
  heteronyms; speech synthesis uses simplified word text and device voices.
- No generic per-word Practice action, and no bottom-of-Browse “Translate
  selected vocabulary” button that might mean either word translation or
  exercise generation. The selected Word Set panel offers a named, disabled-
  when-empty “Use <name> in Practice” action instead.
- Search is automatic with a short debounce for text; filters change
  immediately. Refresh uses a Lucide icon with an accessible name. Clear
  resets filters. The underlying reducer/query contract is retained.

## Later (not yet implemented)

- Row details disclosure with sense/reading provenance, usage examples,
  full meanings and an explicit pronunciation target if disambiguated.
- Optional bulk selection and Save selected rows, with clear scope semantics
  (current page versus complete filtered result set). No implicit “all”
  operation across undisplayed pages.
- A full URL-authoritative Browse state codec and history handling remain
  separate work. Current route-local reducer does not implement that design.
- Additional numbered page navigation only after a trustworthy total-count
  query exists; do not invent “Page N of M”.

## E2E acceptance

Verify 5 rows per page, independent next/previous, query lookahead,
state-cancelled searches, keyboard search/refresh, row Listen accessibility,
selected-set Add/Remove, empty saved-set start disabled, and route handoff with
saved-set preselection. Run at 320, 768 and 1440 px in both themes. Speech
synthesis calls may be mocked at the browser boundary for deterministic CI;
voice quality is a device test, not an automated correctness assertion.
