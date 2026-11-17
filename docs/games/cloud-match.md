# Cloud Match

## Purpose and interaction

Cloud Match is a Chinese vocabulary matching game. Its presentation uses legible moving cloud tiles and consistent controls in the established Zipeek visual language. The game starts through its own setup route and makes gameplay, feedback and results distinct; the learner should not have to infer whether a match was accepted solely from the disappearance of a tile.

## Behavior contract

- Present readable Hanzi, corresponding meanings or other explicitly selected variants. Keep labels legible over the background and during movement.
- Show selection, accepted and rejected match states with accessible textual equivalents, not color alone.
- Keep motion and crossing paths stable. Controls remain keyboard-operable, and reduced-motion users get an equivalent non-moving path to complete the game.
- On exit, distinguish leaving a running game from discarding progress. Do not attribute successful learning evidence simply to game completion.
- Do not use content-specific progress or scoring as a substitute for the shared pedagogical evidence policy.

## Implementation boundary

Actual movement algorithms, match semantics and selected game settings must be verified in the game source before refactoring. See [games source audit](games-source-audit.md), [learning evidence](../learning-practice/practice/learning-evidence.md), and [testing](../testing-quality/acceptance/testing.md).

Historical source observations and prior visual experiments are preserved in [Cloud Match implementation history](../archive-history/audits/superseded-maintained/games--cloud-match-source.md).
