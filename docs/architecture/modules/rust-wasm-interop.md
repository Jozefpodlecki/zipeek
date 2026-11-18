# Rust/WASM browser interoperability

This is the maintained contract for Zipeek-owned web implementation. It complements [coding standards](../../engineering/coding-standards.md), [architecture](../architecture.md), and the existing [browser dictionary adapter](browser-db.md). It describes design intent and code ownership, not proof that every historical JavaScript source has already been migrated.

## Default: application logic belongs in Rust

- Write Zipeek-owned browser behavior in Rust compiled to WASM. This includes feature state, validation, scheduling, cancellation, response parsing, error mapping, and lifecycle cleanup.
- Use the typed `web-sys` and `js-sys` interfaces where available. Use `wasm-bindgen` when the pinned browser or vendor API is not adequately exposed by those bindings.
- `zipeek-browser` owns reusable, browser-specific integration such as clipboard, microphone, canvas, speech, media playback, dynamic module loading, and opaque vendor handles. `zipeek-db/src/wasm/` owns DuckDB-WASM-specific initialization and transport. These are deliberate domain boundaries; do not move the database adapter into `zipeek-browser` just to centralize all browser code.
- `zipeek-web` owns Yew components, routing, application presentation and composition. Pass typed Rust values and callbacks across that boundary rather than exposing raw JavaScript objects to UI components.
- Domain rules that do not need browser APIs belong in a platform-independent Rust module or existing domain crate. Do not pull `web-sys` or Yew into those domains.

## Thin interop rule

JavaScript is a foreign-function boundary, not a second application implementation. When a browser primitive or pinned vendor module needs JavaScript:

1. Prefer direct typed browser bindings. If necessary, define a small `wasm-bindgen(inline_js = ...)` bridge with a narrow, named operation, such as `import(url)`.
2. Implement configuration, validation, retries, cancellation, state transitions and cleanup on the Rust side. Do not create a JavaScript service that wraps a JavaScript service and holds the real state.
3. Wrap vendor `JsValue`/`Object`/`Function` instances in an opaque Rust newtype. Keep internal fields private and expose domain-specific methods. Ensure that any required `destroy`, event unsubscribe, object URL revocation or stream-track stop is explicit and called once. Use `Drop` where the underlying cleanup is synchronous and safe; async shutdown needs an explicit API.
4. Accept `Window` and `Document` from the application's existing `AppEnvironment` in browser-service constructors instead of repeatedly discovering browser globals. Pass those handles to child operations that need them; do not create new global lookups merely to query focus or create an element.
5. Treat generic `Event` callbacks as a runtime browser boundary. Use checked `dyn_ref`/`dyn_into` to narrow to `PointerEvent`, `ClipboardEvent`, or another subtype unless a typed abstraction proves the event type at the call boundary. `unchecked_into` is appropriate for proven invariants, not as a default shortcut for an event name registered through `add_event_listener_with_callback`.
6. Represent meaningful structured values with named Rust types. For example, canvas coordinates use `CanvasPoint { x, y }` rather than nested anonymous coordinate tuples; keep browser-independent geometry and its tests out of Yew.
7. Convert browser exceptions into actionable Rust errors. Avoid unchecked casts, swallowed `JsValue` errors, and panics on externally supplied input. Preserve cancellation and stale-response protection across async calls.
8. Keep the bridge independent of deployment root assumptions. Resolve asset URLs with the configured base path or `document.baseURI` as appropriate; verify root `/` and Pages `/zipeek/` paths.

For example, use an inline JavaScript bridge only to perform dynamic `import(url)` when direct Rust bindings cannot express it. Rust then resolves the pinned vendor export, configures its arguments, and owns the returned opaque object. The application should not introduce `chart_bridge.js` whose only job is to re-export another JavaScript loader.

This is an ownership guideline, not a requirement to wrap every primitive in a newtype. Introduce opaque wrappers when they model meaningful identity, lifetime or invariants; avoid decorative abstraction.

## Deliberate JavaScript exceptions

- **Third-party dependencies:** Keep approved pinned libraries, including Chart.js under D35 and DuckDB-WASM, with their required JavaScript/WASM artifacts, build steps, notices and license obligations. Using a library through thin bindings does not mean rewriting the vendor implementation in Rust.
- **Build tooling:** Node scripts for asset preparation, checks, licensing and code generation execute at build time, not as an application service.
- **Tests:** Browser probes and E2E helpers may use JavaScript when they are the appropriate browser-instrumentation boundary. Keep production behavior in Rust.
- **Early bootstrap:** The HTML/theme/loading shell may need minimal JavaScript before Rust/WASM is available. Do not use this exception to create another long-lived state manager.

A new substantial handwritten runtime `.js` module needs a documented technical reason and approval if it changes an existing controlled requirement. Do not migrate or remove supported vendor tooling merely to achieve a zero-JavaScript file count.

## Examples from the current source layout

- `zipeek-browser` provides the Rust speech and recognition-workspace browser adapters; `zipeek-web` consumes their typed interfaces.
- `zipeek-browser` owns the Chart.js import and instance lifetime. Chart configuration specific to the application belongs in Rust application code, while the pinned Chart.js bundle remains a third-party asset.
- `zipeek-db/src/wasm/` follows the same pattern for its specialized DuckDB-WASM transport and worker integration, while keeping the database adapter in its owning crate.

These examples establish the intended boundary. R541 and earlier WIP migrations still require target compilation and browser acceptance before their behavior can be considered verified.

## Review and acceptance checklist

- Does this change add Zipeek-owned runtime logic in JavaScript that could live in Rust? If so, justify it first.
- Is browser-specific code in `zipeek-browser` or the specialized DuckDB-WASM adapter, and is Yew-specific behavior in `zipeek-web`?
- Are JavaScript object ownership, cleanup, listener lifetimes, cancellation, and error mapping explicit?
- Are supported vendor packages pinned and packaged with licensing preserved, and do import URLs work at both deployment bases?
- Are boundary tests and Rust tests present? For behavior changes, use locked `wasm32-unknown-unknown` checks, asset preparation/integrity checks and browser E2E acceptance. Source review or Node syntax checks alone do not verify WASM integration.

## Recognition workspace state events

The recognition canvas communicates changes to Yew using an injected Rust `RecognitionWorkspaceCallback` and `RecognitionWorkspaceEvent`. The browser crate emits `RecognitionInputState` and typed `RecognitionWorkspaceError` values. The web crate translates them into reducer actions. DOM attributes and synthetic custom events must not be used to round-trip internal recognition state. Browser events themselves remain checked at the runtime boundary.
