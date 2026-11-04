# zipeek-ai prompt architecture

`crates/zipeek-ai/src/prompts/` contains independent, versioned system instructions and input builders for `sentence_generation` and `evaluation`. `generation_messages` and `evaluation_messages` in `lib.rs` preserve the existing OpenAI-compatible role/message format and delegate to these modules.

Generation creates one short Mandarin translation exercise using vocabulary from the selected set, without leaking its English answer. Evaluation scores semantic fidelity, permits natural synonyms, and returns narrowly structured feedback. Both treat learner or vocabulary text as data rather than instructions. Each remains subject to application-side parsing and validation.

The native offline dictionary sense classifier is maintained in `crates/db-gen/src/llm/prompts/classification.rs` and has its own cache-invalidating version because it uses a different taxonomy and execution environment. No cross-crate dependency was added.

Future exercises and optional AI-driven E2E agents should add dedicated, versioned prompts with role separation and separate, opt-in execution; deterministic tests remain the source of pass/fail outcomes. The agent must not be used as the sole judge of its own success. An AI quiz-runner is not implemented or approved by this checkpoint.
