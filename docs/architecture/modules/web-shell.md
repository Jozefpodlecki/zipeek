# Web shell and startup boundaries

This page describes the checked-in Rust/Yew implementation. Approved changes that have not been implemented are kept in [interface and platform policy](../../product-ux/foundation/interface-and-platform-policy.md), [visual review](../../product-ux/references/ui-screenshot-review.md) and [tasks](../../project-management/tasks.md).

## Bootstrap and theme

`crates/zipeek-web/index.html` provides the static loading shell before WASM mounts. [`src/setup.rs`](../../../crates/zipeek-web/src/setup.rs) applies the stored theme and appearance preferences before rendering [`src/app.rs`](../../../crates/zipeek-web/src/app.rs). A static bootstrap failure must be distinguished from a dictionary initialization failure after Yew starts.

## Database ownership

[`DatabaseProvider`](../../../crates/zipeek-web/src/components/providers/database_provider.rs) owns database state and retry commands. [`DatabaseBoundary`](../../../crates/zipeek-web/src/components/providers/database_boundary.rs) renders loading, ready, and failure states; the failure state includes a retry action. [`route.rs`](../../../crates/zipeek-web/src/route.rs) applies this boundary to database-dependent routes only. Docs, Settings, Stats, courses and several game routes are explicitly exempt, so the approved policy of blocking **the entire application** on essential dictionary failure is **not yet implemented**. A browser test is required before claiming startup recovery is complete.

## Navigation and practice shell

[`AppShell`](../../../crates/zipeek-web/src/components/shell/app_shell/mod.rs) owns the shared header, sidebar, footer, session reminder and route-specific layouts. Active local Practice sessions use a dedicated immersive shell; course lessons also use a dedicated layout. The general shell still computes and passes a centered route title to its header. Removing it to make space for the approved long dragon progress indicator is future work, not current source behavior.

## Other-tab detection

[`OtherTabIndicator`](../../../crates/zipeek-web/src/components/shell/app_shell/other_tab/mod.rs) uses a [`BroadcastChannel` coordinator](../../../crates/zipeek-web/src/components/shell/app_shell/other_tab/coordinator.rs) with periodic peer heartbeats. Its current UI is a bottom-right informational warning when peers exist. The coordinator observes presence, not ordered exclusive ownership, and the source does not implement the approved first-tab-wins full-screen blocking protocol. Do not claim multi-tab write exclusivity or crash takeover safety based on this warning.

## Deployment and verification

The hash router runs at local root and the hosted `/zipeek/` prefix. Static source checks establish file wiring, not successful boot, database loading, browser storage persistence or layout. Verify relevant startup and navigation journeys at both base paths with an actual build and browser test.

## Source provenance

Older descriptions are preserved in [historical architecture audits](../../archive-history/audits/SOURCE_RECONCILIATION.md) and release history. They do not establish current runtime acceptance.
