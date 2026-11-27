# User experience

Zipeek uses a consistent violet visual system, accessible controls, and a Chinese landscape aesthetic. Current product choices are described in the topic specifications below. Historical implementations and screenshots are not current requirements.

## Global visual system

- [Interface and platform policy](foundation/interface-and-platform-policy.md): navigation, copy, startup recovery, SEO and settings.
- [Accessibility](foundation/ux-accessibility.md) and [appearance](foundation/ux-appearance.md): readable text, keyboard support, system theme and reduced motion.
- [Header and dragon](foundation/header-and-dragon.md): the dragon itself is the evidence-based progress display, with no labels above it.
- [Credits](credits.md): distribution resources, licensing evidence and inspirations.
- [Screenshot-based review](references/ui-screenshot-review.md): approved changes derived from the working application.
- [Landing page](pages/landing-page.md) and [Browse](pages/ux-browse.md): individual screen contracts.

## Practice presentation

- [Practice setup](practice/ux-practice-setup.md), [Matching](practice/ux-match.md), [Multiple Choice](practice/ux-mcq.md), and [session summary](practice/ux-summary.md).
- [Shared Practice and outcome requirements](../learning-practice/practice/practice-outcomes.md) govern correct, partially correct and incorrect feedback, session completion and Skip and Replace.
- [Acceptance](../testing-quality/acceptance/approved-behavior-acceptance.md) lists testable criteria.

Sentence Builder provides drag-to-reorder tiles with a keyboard-accessible alternative and setup-selected automatic checking. Theme-specific creature progress appears above the current exercise within the local Practice workspace.

The global header omits active page and exercise titles where these merely repeat context. The long dragon-shaped indicator represents persisted pedagogical evidence, not a plain exercise completion score. A dedicated Practice shell supplies consistent feedback and navigation without forcing different gameplay layouts into one component.

The earlier dated UX compilation is retained only in [historical UX notes](../archive-history/audits/UX.md).

For the approved Statistics 2.0 design, including the deterministic sample-data preview and the separate production evidence gate, see [Statistics product and interaction design](pages/learning-statistics-preview.md).


## Practice setup navigation and service feedback

Approved 2026-11-27: place the icon-only Back action at the left of the four-step Practice path widget. Place Continue at the right until Summary, where a Play icon starts practice. Keep accessible names, tooltips, visible focus and disabled states, and remove duplicate navigation controls from the summary and separate step footer. Preserve existing setup state transitions. Picture vocabulary warnings distinguish the saved text-model requirement from image backend readiness, link to the appropriate Settings section, and offer image-service retry. Warning actions wrap below the message on narrow screens. See [Practice setup](../learning-practice/practice/practice-session-builder.md).
