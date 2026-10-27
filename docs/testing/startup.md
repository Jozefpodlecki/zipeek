# Startup loading checks

Static loading shell, handoff, and database readiness coverage. Part of the
[testing index](TESTING.md). Approved requirement in
[D26](../process/decisions/web-and-ui.md#d26-persistent-landscape-and-loading-presentation);
product contract in [PRODUCT](../product/PRODUCT.md#startup-loading-experience).

## Required coverage

- Delay app bootstrap to verify the static HTML loading shell is visible before
  Yew mounts. Check its theme/background/loading indicator without depending on
  the database or Yew having initialized.
- Delay database loading independently of bootstrap. Verify Yew's first render
  is Loading and that the loading view remains visible after mount while the
  database is fetching/opening; no blank or premature ready content.
- Verify Ready appears only once the read-only database is queryable, including
  when its valid word pool is empty (D05's neutral card, not a stuck spinner).
- Simulate a failed download/open, verify loading ends in the retry state, and
  verify Retry transitions through Loading to success or another handled failure.

Use controlled delays/failures when implementing this coverage; fixed sleeps do
not establish the loading-state lifecycle.

## Related runtime requirements

The startup lifecycle is defined in
[ARCHITECTURE §Startup](../architecture/modules/web-shell.md#startup-static-loading-shell-and-yew-loading-state)
and implemented by A06's `DatabaseState`/`DatabaseProvider`. The visual
presentation (landscape outside `#root`, centered spinner, reduced-motion) is
[D26](../process/decisions/web-and-ui.md#d26-persistent-landscape-and-loading-presentation)
and implemented by A30. This file covers the *state transitions* during startup;
the *visual presentation* is covered in [shell.md](shell.md).

## Current status

A06 implements the `DatabaseProvider` Loading/Ready/Failed+Retry lifecycle. A30
implements the static loading shell and Yew Loader composition. Chrome tests
exercise mounted Loading while a controlled fetch is held, download failure and
Retry, and the real 12,100-row Ready state (see
[browser-db.md](browser-db.md)). The checks above are the target coverage once
A21 wires route-level queries; the loading-state tests described in the first
four bullets are already implemented by A06.