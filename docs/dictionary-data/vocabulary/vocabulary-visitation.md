# Vocabulary visitation and selection boundaries

## Responsibilities

A shared browser-local vocabulary visitation service records exposure without interpreting it as mastery. Each entry should identify a canonical vocabulary item and preserve most recent visit order and bounded visit frequency. The storage schema and capacity must be validated; different games may use different rotation windows, but the underlying API must not let one game's eviction policy silently discard all Practice history.

The selector consumes eligible vocabulary, global exact-entry exclusions, current session constraints, exposure history and, when enabled, separately stored pedagogical outcome evidence. Do not let game-specific randomness or local-storage handling leak into core assessment logic. The selector must be testable with deterministic inputs, and tie breaking must be stable for a fixed session seed.

Model exposure, assessment attempts and mastery as independent typed records. A visited word is not a scored word. An exercise skipped without submission does not award mastery. A previously scored mistake stays in evidence even if the learner skips a subsequent attempt. Recent successes can lower review priority without deleting prior evidence.

Apply the global Settings pool-exhaustion policy captured at session start. Do not expand outside the vocabulary source, ignore exclusions or spin indefinitely on an exhausted pool. Maintain a bounded replacement search with a clear no-eligible-content state. Browser-storage failures must degrade safely and transparently, without fabricating persisted history.

This document is the target architecture, not a claim that a shared manager already exists. See [vocabulary requirements](vocabulary-selection.md) and [learning evidence](../../learning-practice/practice/learning-evidence.md).
