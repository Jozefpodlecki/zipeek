# Ui State Ownership

> **Current approved target behavior:** [Shared visitation boundaries](../dictionary-data/vocabulary/vocabulary-visitation.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


## State ownership

| State | Authority and lifetime | Access and status |
| --- | --- | --- |
| Theme | App lifetime; existing approved saved preference | Implemented ThemeProvider, read/command hooks and persistence effect |
| Motion preference | App lifetime; live prefers-reduced-motion value | MotionProvider publishes a small preference snapshot; no new saved setting proposed |
| Database client | One app-owned runtime session | Implemented stable Dictionary context; typed read-only Database facade |
| Database readiness | App lifetime, independent of routes | Implemented status context, typed Retry command and pure reducer |
| HSK, PoS, mode, count, seed, offset and URL exclusions | Current URL, normalized by A08 | Route adapter produces immutable typed inputs; no second writable global filter store |
| Saved and pending exclusions | Explicit user actions, approved D13 lifecycle | A11 local setup hook validates saved IDs and exposes Retry/Reset; generation only, never union into loaded URLs. Shared feature commands can follow their first consumers. |
| Browse request/results/pagination | Browse feature scope | Feature reducer plus cancellable query effect; URL authority follows approved UX |
| Practice round, guesses, mistakes and progress | Practice session scope | Feature reducer; A07 builds round data, core FixedProgress tracks completion |
| Drawer expansion | AppShell; current reload-reset behaviour | Keep local unless another component actually needs shared commands |
| Hover, temporary focus and individual animated values | Owning component/controller | Local state/refs; never broadcast every frame through app context |

Do not create one application-wide reducer containing every result list, input
and animation tick. A provider belongs at the nearest shared ownership boundary.
Use props for leaf inputs such as a card label, selected state or click callback.
Components that need a shared service or feature state consume its hook directly;
intermediate layout components must not relay those values through several levels.

## Hanzi recognition feature ownership (R545 WIP)

The `recognize_hanzi` feature reducer owns input classification, Undo availability, request status, errors, candidate results, and the monotonic request generation. The page owns scoped async orchestration and workspace mounting; child components receive typed props. The browser crate owns canvas, pointer, upload and clipboard resources. Candidate parsing remains a pure feature module. Browser verification is pending.
