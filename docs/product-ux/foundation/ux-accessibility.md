# Ux Accessibility

## Accessibility

Pragmatic accessibility (UX-A11Y01): implement the rules below without claiming
formal WCAG conformance. Practice feedback announcements are deliberately
omitted; a future summary canvas must have its numeric stats in adjacent DOM text.

- All actions support keyboard use: Tab, Enter/Space, plus documented shortcuts.
  This includes cards, answers, navigation, theme/drawer toggles, pronunciation,
  GitHub and progression actions. No mouse-only elements.
- Every focusable element has a visible focus ring/outline with at least 3:1
  contrast against its surroundings; never remove outlines without replacement.
- Prefer native buttons, links, nav, main and aside. Avoid redundant roles,
  unnecessary ARIA or tabindex; use tabindex only for a specific focus target.
- Correct/wrong feedback includes visible ✓/✗ alongside green/red, independent
  of hover. This applies to match cards and MCQ answers.
- Text contrast is at least 4.5:1; UI boundaries, focus rings and icon strokes
  at least 3:1. Check muted traditional hanzi, dimmed answers, theme control,
  footer and feedback states in both themes.
- Touch targets are at least 24×24 CSS px or sufficiently spaced for a 24px
  circle not to overlap another target. Aim for 44×44px on mobile controls.
  The footer's icon-only GitHub link must meet the 24×24px minimum.
- Committed route navigation moves focus to the new main or heading and updates
  `document.title` so routes are distinguishable, e.g. "Practice - Zipeek".
- A mobile overlay drawer contains focus with a close action returning it to
  the toggle, or uses an inline/nonmodal arrangement without moving focus.
  Never strand focus, expose obscured content through Tab, or trap the user.
- Retain the existing skip link: focus main without changing the route.
- Keep `<html lang="en">`; standalone hanzi uses `lang="zh-CN"` where practical.
- Search/future inputs need a visible label or aria-label, not only a placeholder.
- Reduced motion disables nonessential motion and keeps semantic colour changes.
  Practice selection, correctness and score updates have no live announcements.
