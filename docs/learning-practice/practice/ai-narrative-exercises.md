# AI narrative Practice exercises

## Scope

Story Builder, Reading Comprehension, and Conversation Practice are AI-generated, resumable Practice exercise types. They reuse the configured Local Llama or LM Studio provider and the selected Practice vocabulary source. No personal history or learning evidence is sent to the model.

## Setup and limits

- Story Builder: a coherent multiline passage with **exactly 5 to 10 complete Chinese sentences**, selectable in Practice setup. Default: 7. The sentences must be kept in their authored order, with 2 to 6 single-span gaps and a unique answer for every gap. The word bank contains those answers plus plausible distractors.
- Reading Comprehension: an article or story with **exactly 5 to 10 complete Chinese sentences**, selectable in Practice setup. Default: 7. AI chooses whether to ask about the main idea, a character's action, or circumstances, returning three distinct options and one answer index.
- Conversation Practice: **5 to 10 dialogue turns**, configured separately. Default: 7. Each turn presents an AI-authored utterance and three candidate replies. The first experimental implementation generates a coherent fixed dialogue plan rather than branching dynamically based on every learner response.
- Grammar focus: the setup offers automatic grammar selection or one of the existing curated grammar patterns. It is passed to the generator as a generation instruction; semantic grammatical accuracy is not automatically verified.

The configured passage sentence count is *not* the number of exercises in the Practice session. One generated passage constitutes one scheduled exercise. The saved instance contains an ordered `sentences` array and is validated against the original session's configured count, both at generation time and on reload.

## Trust and scoring

All model responses are untrusted. The parser rejects unknown fields, unexpected sentence counts, missing text, invalid answer indices, duplicate choices, malformed story gaps, oversized passages and mismatched exercise kinds. The server-generated answer key, rather than a second model judgement, determines the learner's result. This is automated answer-key grading, not verified reading proficiency or pedagogical mastery. The question's factual answerability and stylistic quality still require model-backed human acceptance tests.

The current conversation prototype advances along a pre-generated plan even when the learner selects a different reply. Adaptive branching, answer explanations before leaving the step, generated-content caching, full grammar-pattern multi-selection, and real provider acceptance tests are outstanding work, not implemented claims.

## Acceptance

- Build `zipeek-exercises`, `zipeek-web` and the WASM target with `--locked`.
- Test all six passage sizes from 5 to 10 sentences, invalid and duplicate word-bank entries, three-option answer validity, and 5 to 10 dialogue turns.
- Exercise setup, resume, failure/retry, session scoring and statistics against both browser OPFS and Tauri persistence.
- Test a genuine Local Llama GGUF and LM Studio provider with long-form response constraints, including invalid JSON and generation-time cancellation.
- Verify layout, keyboard access and theme contrast on narrow and desktop displays.
