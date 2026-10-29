# Web and UI

Approved shell, motion, and styling decisions. Each entry links back to the
[index](index.md) and to its owning document. Bodies are the approved
requirement as recorded on the date shown; they do not imply the feature is
implemented.

---

## D24 Web shell and landing direction

**Status:** Approved 2026-10-27.
**Owner:** [UX](../../product/UX.md), [PRODUCT](../../product/PRODUCT.md),
[ARCHITECTURE](../../architecture/ARCHITECTURE.md).
**Index row:** [D24](index.md#approved-decisions).

The user selected the shell/landing direction on 2026-10-27:

- **Palette:** neutral slate dark/light with blue accents.
- **Layout:** desktop left rail; compact mobile header.
- **Navigation:** shared Home/Practice/Browse links with active-route indication;
  brand links Home. Repository link, theme control, keyboard skip-to-content.
- **Landing:** prominent daily-word area followed by exactly three tiles —
  Practice, Browse, Random.
- **Attribution:** copyright footer plus required CC-CEDICT attribution.

Navigation/shell work comes first; dictionary-backed content remains A06/A12
and no daily word is fabricated. Daily-card fields and unrelated UX decisions
remain open.

Implementation status: A27 delivered the shell. Current UI uses agreed unavailable
copy for the daily word; actual selection is A12.

Superseded in part by D25's theme placement change.

---

## D25 Compact collapsible shell

**Status:** Approved 2026-10-27 by direct user request; supersedes D24's
theme-in-sidebar placement.
**Owner:** [UX](../../product/UX.md), [PRODUCT](../../product/PRODUCT.md),
[ARCHITECTURE](../../architecture/ARCHITECTURE.md).
**Index row:** [D25](index.md#approved-decisions).

Amendments to D24:

- Remove decorative headers such as "Chinese vocabulary" and "A fresh starting
  point" and landing marketing filler. Do not restore them.
- Move theme to the top-right of a **permanent top panel**.
- Make the drawer collapsible via an accessible keyboard-operable toggle
  exposing `aria-expanded` and `aria-controls`; hidden navigation state is
  respected by assistive technology.
- Remove unnecessary vertical scrolling; use natural document scrolling when
  content genuinely exceeds the viewport. No global overflow hiding.
- Preserve functional page/card/action labels and access to drawer/theme controls
  in both expanded and collapsed states.

Implementation: A28 owns the amendment. A29 refines:
- no decorative tile arrows;
- `--topbar-height: 3rem` (48px) shared across grid/drawer/main;
- reduced footer padding;
- borderless resting theme control, with hover and focus-visible feedback preserved.

Drawer expansion state is AppShell-local, initially true, preserved across
routes, reset on reload.

---

## D26 Persistent landscape and loading presentation

**Status:** Approved 2026-10-27 by direct user request.
**Owner:** [UX](../../product/UX.md), [PRODUCT](../../product/PRODUCT.md),
[ARCHITECTURE](../../architecture/ARCHITECTURE.md).
**Index row:** [D26](index.md#approved-decisions).

Reuse the existing unmodified landscape image as the shared loading and app
backdrop; compose light/dark surfaces with it; replace the clipped loading
treatment.

Requirements:

- The image lives **outside** Yew's `#root` mount point (fixed `.app-backdrop`
  before `#root`). Yew mounting, route changes, and theme toggles cannot remove
  or recreate it.
- Image bytes unchanged. Current SHA-256 for both the embedded WebP and
  `web/public/background.webp` is
  `64fc1cb7d2b34ff89ff03f5cef4bcfe3df64eae9f27c6ca073f72e8d2e201f93`.
- `main.css` provides theme-specific shade/surface/card-tint variables; surfaces
  are translucent with backdrop blur. `inline.css` holds critical backdrop/
  loading geometry so initial render does not depend on percentage height or
  wasm.
- Static HTML and Yew loaders share a small centered spinner/Loading panel with
  accessible status text and reduced-motion support. No artificial progress or
  loading delay.
- Reduced-motion preferences disable spinner animation; the landscape itself
  remains visible in both themes.

This is presentation work only. A06/A21 still own database initialization,
readiness, and retry. A30 implements D26; the actual DB readiness pipeline is
A06/A21.

---

## D29 Scoped UI state and motion (proposed)

**Status:** Requested 2026-10-27. A31 scoped theme/database state implementation
approved 2026-10-28; A37 selects the initial 150 ms fade-only profile on that
date. A32 implementation remains separate.
**Owner:** [UI_ARCHITECTURE](../../architecture/UI_ARCHITECTURE.md), [UX](../../product/UX.md).
**Index row:** [D29](index.md#approved-decisions).

The user's 2026-10-27 code review requested modular e2e, scoped UI state, and
motion support. This decision records the *requirement*; the detailed plan in
[UI_ARCHITECTURE](../../architecture/UI_ARCHITECTURE.md) is a proposal, not approval
to add crates or change dependency features.

Approved requirements:

- Scoped read/command contexts and reducers; persistent database/theme providers;
  motion preference provider, small Animato adapter, interruptible route host.
- Preserve the user-added Animato 1.7.3 with `path`, `physics`, and `color`
  features. Do not enable `yew-csr` or add another animation library without
  explicit approval.
- Do not expose raw state setters as another feature's public command interface.
- Motion must not change canonical URL state, replay inputs, RNG draws, scoring,
  or Fixed completion.
- Honor reduced-motion preferences; essential semantic changes happen even when
  animation is disabled.
- No app-wide reducer containing every result list, input, and animation tick.
  Providers belong at the nearest shared ownership boundary.

A31 implements pure theme/readiness reducers, stable typed dictionary access,
read/command hooks and DatabaseBoundary separate from ownership above the router.
Browse/practice reducers are added with their concrete feature tasks. A32 owns
the still-proposed motion profile and route transitions. Choose motion profiles
before enabling Animato features; A31 changes no dependencies or features.

---

## D31 Tailwind-first component styling

**Status:** Required 2026-10-27; implemented by A35 on 2026-10-28.
**Owner:** [CODING_STANDARDS](../CODING_STANDARDS.md),
[UI_ARCHITECTURE](../../architecture/UI_ARCHITECTURE.md).
**Index row:** [D31](index.md#approved-decisions).

Correct the drift from Tailwind to bespoke component CSS. Tailwind 4.3.3
remains installed/imported and built by Trunk; the drift is in *how components
are authored* — recent shell/card work uses bespoke main.css selectors instead
of utilities.

Requirements:

- Tailwind utilities are the default for layout, spacing, sizing, typography,
  borders, responsive rules and interaction states.
- Keep shared theme/design tokens in CSS where appropriate and consume them
  through utilities. Preserve the slate/blue palette and translucent landscape
  surfaces.
- Allow focused custom CSS for critical pre-WASM bootstrap styling, shared
  tokens, keyframes, or a concrete effect that benefits from it. A reason for
  one exception does not justify moving ordinary component styling into
  stylesheets.
- Do not hide the same bespoke component system behind blanket `@apply` rules.
  Factor repeated UI into Yew components and small explicit utility variants.
- Keep conditional variants as complete class strings Tailwind can detect; do
  not construct class names from fragments at runtime.
- Verify Tailwind 4 source detection for Rust/Yew files and remove obsolete
  conflicting rules as components migrate.

A35 owns migration with visual/behaviour parity: before/after screenshots at
`/` and the `/zipeek/` build, all five gates and Chrome coverage. Update e2e
selectors to stable semantic/data hooks when a styling class is removed.

This direction does not approve dependency upgrades, a redesign, or a new
styling framework. A35 completed the migration with root/Pages visual and behavioural coverage.

## A22 amendments to D24/D25 — 2026-10-27

Record these shell requirements for later implementation: navigation labels have
no numbering or active-route decoration, with aria-current="page" retained.
Move the existing repository icon/link into the single dot-separated footer row
with copyright and attribution, preserving its accessible name, link attributes
and licence notice. Do not replace the removed decoration with other chrome.

Daily-card fields are now recorded in [UX](../../product/UX.md#daily-card-fields-ux-land01).
The 400px width policy and Tailwind defaults are also specified there. Current
shell markup/CSS still reflects A27–A30; this amendment is documentation only.
D29 motion profiles and implementation review remain open.

## D32 Web Speech API for pronunciation

**Status:** Approved UX requirement, recorded 2026-10-27 in A22; not implemented.
**Owner:** [UX](../../product/UX.md#pronunciation-d32),
[PRODUCT](../../product/PRODUCT.md),
[web-shell](../../architecture/modules/web-shell.md#planned-pronunciation-boundary-d32).
**Index row:** [D32](index.md#approved-decisions).

Provide the existing yew_icons/lucide volume icon as a Listen action on the daily
card, match cards and MCQ prompts/answers displaying hanzi or pinyin. Use
window.speechSynthesis.speak with a SpeechSynthesisUtterance and lang='zh-CN'.
No audio files, assets or DB columns. Voice support/quality varies by OS/browser.

Apply the supplied fallback assumption: speak the simplified hanzi of the word
the control represents regardless of its displayed label. No alternative
per-surface rule has been selected. Tests spy/stub the invocation and its text/
language, never assert that sound was audible. Browser interop stays in web;
any feature/dependency additions still require an explicit implementation plan.


## A35 implementation amendment — 2026-10-28

D31 is implemented: Rust-source utilities, @theme aliases and critical bootstrap
exceptions replace the bespoke shell/card stylesheet. The legacy v3 config is
removed; no dependency pins/features change. D24/D25 shell cleanup is implemented
with label-only navigation and the repository icon in the footer. The user
explicitly approved a single horizontal footer row with attribution text allowed
to wrap inside its item at 400px. This refines the previous single-text-line
requirement without changing attribution or external repository-link semantics.

Light muted text changes from slate #64748b to #475569 to meet conservative
contrast checks over translucent surfaces. The landscape and other palette tokens
are preserved. Tailwind md/lg boundaries and full-width content implement the
recorded responsive policy; future exercise/Browse layouts remain their feature
tasks. D29 state/motion proposals and D32 pronunciation remain unimplemented.

## A31 implementation amendment — 2026-10-28

D29 state scope is implemented for the current theme and database consumers.
Read-only snapshots and named commands replace exposed status setters. Native
reducers reject stale and duplicate results; a synchronous request gate also
rejects results before old effect cleanup. Pending JS opens are logically
cancelled and stale successful handles disposed after settlement, using existing
async last-handle cleanup. No awaited shutdown contract is claimed. Theme DOM
and storage writes are effect-owned; rapid toggles use current reducer state.
DatabaseBoundary retains existing loading/failure visuals inside the stable shell.
Motion profiles, Animato integration and route transitions remain A32 proposals;
future feature state belongs with its UI task. URL/RNG authority is unchanged.

## A37 amendment to D29 — 2026-10-28

Approved initial profile: 150 ms fade-only route and drawer transitions, no slide
or scale. Query/pagination changes do not animate the whole route. Existing
practice feedback timings remain. Reduced motion commits movement/fades instantly
with semantic/color changes intact. Easing and phase allocation are implementation
details to review within that profile in A32, alongside cancellation, focus and
live preferences. This approval adds no Animato feature or dependency.

## D34 Static social cards and preview image

**Status:** Design and copy approved 2026-10-28 during A37; not implemented.
**Owner:** [UX](../../product/UX.md#social-metadata-and-preview-image-a37--d34),
[PRODUCT](../../product/PRODUCT.md#site-sharing-metadata-d34).
**Index row:** [D34](index.md#approved-decisions).

Use one static site-wide 1200×630 preview for Open Graph and Twitter/X large-image
cards, shared by all current hash routes. Reuse the existing landscape, dark slate
overlay, Zipeek wordmark and blue accent, without a round-specific word or score.
Approved title: "Zipeek — Chinese hanzi practice". Approved description:
"Look up Chinese words and practise hanzi, pinyin, and meanings."

A40 owns image creation and initial HTML metadata/asset validation before v1.
Keep /zipeek/ in canonical/image URLs and retain route-specific browser titles.
No dependency or hosting change is approved by this design decision.

## D35 Fixed summary and chart packaging

**Status:** Approved 2026-10-29 during A10 implementation.
**Owner:** [UX](../../product/UX.md#summary-screen),
[PRODUCT](../../product/PRODUCT.md#practice),
[AGENTS](../../../AGENTS.md#4-version-and-dependency-policy).
**Index row:** [D35](index.md#approved-decisions).

Apply the previously approved UX-SUM01 summary: after the last Fixed Continue/
Next round only, show Session complete, a doughnut ring and four real DOM stats:
First-attempt, Mistakes, Time, Highest streak. No summary in Endless. Matching's
approved expanded label is First-attempt correct pairs: a pair counts only if
matched on its first attempt, with denominator rounds × 4. During A10 the user
explicitly approved MCQ's compact First-attempt label: correct first selections /
total questions (5/10/20). One locked wrong selection is one MCQ mistake.

Time runs from first round render to final Continue/Next round, never pauses on
hidden tabs, freezes at completion and is not persisted. Keep the maximum
consecutive-correct streak. Play again retains mode/count/filter/exclusions but
starts a fresh session at a random in-pool offset, replacing the earlier D13/
product reset-to-zero restart behavior. Back to setup returns to the default
setup, without preselection. Leaving/reloading loses session progress and scores;
ordinary reload still honors its URL offset. Wrapping does not reset progress.

The user approved exact chart.js@4.6.1 and build-only esbuild@0.28.2 after npm
metadata confirmed those releases. Chart.js is restricted to the summary ring;
import/register only DoughnutController and ArcElement with Chart, never auto,
scales, tooltips or legend. esbuild emits a local tree-shaken ESM bundle; no CDN,
runtime bundler or general charting scope. Retain Chart.js, its color dependency
and build-tool licenses alongside packaged assets. Preserve unrelated locks.
Use opaque wasm-bindgen handles, mount in an effect and destroy on unmount,
including late async creation. Failure keeps all numeric text and actions usable.
Count and arc fill animate for 300 ms; reduced motion makes both instant and
an active preference change finishes them immediately. A09 reuses this summary
infrastructure when matching is implemented. No persisted chart/session data.


## D39 Matching first-attempt pair accounting

Approved explicitly during A09 on 2026-10-29. A wrong attempt using a card for
word A and a card for word B disqualifies BOTH represented pairs from the
First-attempt correct pairs numerator. Later correct matches still increase
live correct and consecutive-correct/streak counts. Every wrong attempt adds
one Fixed mistake, regardless of whether either pair was already disqualified.
Per-round disqualification resets on the next round. Summary denominator remains
four required pairs per matching round. This clarifies D35 without changing
WordIds, label comparisons, layout, canonical inputs or the round RNG stream.


## D41 Practice board readability and pinyin

Approved 2026-10-29 under A49. Center the practice board at a 56rem maximum
width with an opaque theme-aware surface. Matching uses two equal columns
including phones, aligned card heights, contained separate audio controls and
readable matched cards instead of whole-card opacity. Keep feedback timing,
keyboard behavior, shuffled positions and persistent progression actions.
Show display pinyin beneath hanzi/traditional and above Listen for Hanzi-to-
English matching and Hanzi-to-meaning MCQ. Existing comparison keys, canonical
round input, RNG stream, DB and dependencies remain unchanged. This supersedes
the earlier full-width/stacked-on-mobile exercise presentation in UX.

The user's additional drawer-icon/width and setup exercise-help dialog requests
are recorded as A50/A51 follow-ups, with accessible interaction requirements in
UX. They are documentation-only in A49, not implemented by this decision.


D41 amendment, A52 (user-directed 2026-10-29): make matching cards smaller and
place Listen at the right edge alongside the text, removing the audio footer.
Pinyin stays under hanzi/traditional; MCQ's pinyin-above-audio layout is unchanged.
Keep full-page matching free of vertical scroll at representative desktop sizes,
while allowing natural scrolling for long content or small/short viewports.
Maintain native independent audio and a minimum 44px target.
