# Current approved product policy

This record consolidates final user decisions from the current product review. It supersedes conflicting historical proposals about Fixed summaries, URL replay and reset, copy tone, vocabulary exclusions, appearance and session visibility. Historical decision entries remain archived as evidence and are not rewritten. The linked maintained specifications state current target behavior, not implemented or verified behavior.

| Topic | Approved outcome | Owner |
| --- | --- | --- |
| Session summaries | Correct, Partially correct and Incorrect distribution with no universal percentage | [Practice outcomes](../../learning-practice/practice/practice-outcomes.md) |
| Exercise scoring | Preserve type-specific pair counts, handwriting assistance and meaningful attempts | [Practice outcomes](../../learning-practice/practice/practice-outcomes.md) |
| Session lifecycle | Persist locally, Skip and Replace, no Reset, no URL state replay | [Practice outcomes](../../learning-practice/practice/practice-outcomes.md) |
| Copy | Contextual tone, specific feedback, sentence case, HSK 7-9 and secondary traditional display | [Interface policy](../../product-ux/foundation/interface-and-platform-policy.md) |
| Vocabulary selection | Recent rare mistakes, unseen words, visitation-based rotation when pedagogy is off | [Vocabulary selection](../../dictionary-data/vocabulary/vocabulary-selection.md) |
| Format rotation | Balance enabled multiple-choice formats within a session | [Vocabulary selection](../../dictionary-data/vocabulary/vocabulary-selection.md) |
| Exclusions | Persistent global exact-entry list in Word Sets, mandatory for Practice | [Vocabulary selection](../../dictionary-data/vocabulary/vocabulary-selection.md) |
| Pool exhaustion | Global Settings policy, default least-recent reuse, optional early finish | [Vocabulary selection](../../dictionary-data/vocabulary/vocabulary-selection.md) |
| Profiles | JSON export/import, settings only, no personal history or credentials | [Profile portability](../../learning-practice/practice/practice-profile-portability.md) |
| Hanzi recognition | One drawing/paste/upload canvas, single and multiple Hanzi, no saved history | [AI Hanzi recognition](../../learning-practice/hanzi/ai-hanzi-recognition.md) |
| Clipboard | Optional permission for detection, no mount-time prompt, explicit recognition upload | [AI Hanzi recognition](../../learning-practice/hanzi/ai-hanzi-recognition.md) |
| Dragon | Durable learning evidence, distinct from session completion | [Practice outcomes](../../learning-practice/practice/practice-outcomes.md) |
| Database startup | Blocking retry screen for essential database failure | [Interface policy](../../product-ux/foundation/interface-and-platform-policy.md) |
| Theme | System by default with persistent Light/Dark override | [Interface policy](../../product-ux/foundation/interface-and-platform-policy.md) |
| SEO | Public indexing and appropriate metadata without learner private state | [Interface policy](../../product-ux/foundation/interface-and-platform-policy.md) |
| Code comments | None, including lint rationales | [Coding standards](../../engineering/coding-standards.md) |
| Dependencies | Prefer latest stable but require explicit approval for all updates | [Coding standards](../../engineering/coding-standards.md) |

Older technical proposals about E2E layout, state boundaries, motion and crate extraction require source-based reconciliation before authorization. They remain open in the decision inbox.
