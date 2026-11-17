# Startup loading checks

**Historical acceptance scenarios:** old 12,100-row readiness numbers refer to an earlier artifact, not the checked-in 120,970-row manifest. Current startup implementation and current acceptance are in [browser-db](../../architecture/modules/browser-db.md) and [CURRENT](../../project-management/current.md).

Static loading shell, handoff, and database readiness coverage. Part of the
[testing index](../acceptance/testing.md). Approved requirement in
[D26](../../project-management/decisions/web-and-ui.md#d26-persistent-landscape-and-loading-presentation);
product contract in [PRODUCT](../../product-ux/product.md#startup-loading-experience).

## Required coverage

- Delay app bootstrap to verify the static HTML loading shell is visible before
  Yew mounts. Check its theme/background/loading indicator without depending on
  the database or Yew having initialized.
- Delay database loading independently of bootstrap. Verify Yew's first render
  is Loading and that the loading view remains visible after mount while the
  database is fetching/opening; no blank or premature ready content.
- Verify Ready appears only once the read-only database is queryable, including
  when its valid word pool is empty (D05's neutral card, not a stuck spinner).
- Simulate a failed download/open, verify loading ends in the centered dictionary
  error card with a readable alert and Try again action, and verify retry
  transitions through Loading to success or another handled failure. Confirm
  technical exception details remain in the console rather than the error card.

Use controlled delays/failures when implementing this coverage; fixed sleeps do
not establish the loading-state lifecycle.

## Related runtime requirements

The startup lifecycle is defined in
[ARCHITECTURE §Startup](../../architecture/modules/web-shell.md#startup-static-loading-shell-and-yew-loading-state)
and implemented by A06, with A31 pure readiness reducer/provider and
DatabaseBoundary separation. The visual
presentation (landscape outside `#root`, centered spinner, reduced-motion) is
[D26](../../project-management/decisions/web-and-ui.md#d26-persistent-landscape-and-loading-presentation)
and implemented by A30. This file covers the *state transitions* during startup;
the *visual presentation* is covered in [shell.md](shell.md).

## Current status

A06 implements Loading/Ready/Failed+Retry; A31 separates provider ownership
above the router from DatabaseBoundary presentation inside the shell. A30
implements the static loading shell and Yew Loader composition. Chrome tests
exercise mounted Loading while a controlled fetch is held, download failure and
Retry, and the real 12,100-row Ready state (see
[browser-db.md](browser-db.md)). The checks above are the target coverage once
feature tasks wire their remaining query surfaces; the loading-state tests described in the first
four bullets are already implemented by A06.

## A21 route/query coverage

After dictionary Ready, controlled worker-message delays prove practice Loading
persists until queries settle. A delayed old query failure released after a new
variant is Ready cannot replace it. A one-shot query failure reaches Retry, which
prepares the same URL/offset successfully. Run these at root and /zipeek/. An
unknown route displays its 404 even with dictionary fetch blocked, then Home
recovery can use the ordinary dictionary Retry. Ready practice placeholders replace
the previous permanent spinner; gameplay is still unimplemented.

## Rust startup preferences

`crates/zipeek-web/src/setup.rs` applies dark/light mode, Hanzi font, preset
and layout density through the existing typed settings before calling Yew
`Renderer::render()`. `index.html` contains no duplicate localStorage policy.
The static HTML loader and hidden fatal panel remain available independently
of Yew, but a WASM download failure cannot be caught by Rust code that did not
start. Check dark and light saved preferences, invalid and blocked localStorage,
root absence, and the browser console manually. The initial HTML defaults to
dark, so a saved light theme may flash dark before WASM initialization.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/testing-quality-e2e-startup.md). Dated implementation claims in that snapshot are not current build or browser verification.
