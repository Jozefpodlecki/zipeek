# Current task: Desktop development model path

Approved 2026-11-27: make `cargo make desktop-dev` always set `ZIPEEK_AI_MODEL_PATH` to the specified Qwen3.5-9B Q4_K_M GGUF in the user's LM Studio model directory.

Implemented one task-scoped Makefile environment entry. The existing GGUF is referenced in place. Other task environments, dependencies and application data are unchanged.

Verification: parsed Makefile.toml with Python tomllib, checked the exact approved value, verified that the GGUF exists, confirmed that the variable is scoped only to desktop-dev, and passed `git diff --check`. Rust/frontend tests are unnecessary for this configuration-only edit. No desktop launch, model load or inference was performed.

Next: restart `cargo make desktop-dev`, click Start for Local AI, then discover/select/save the model under AI Translation. Earlier runtime gates and known test/lint issues remain in [TASKS](tasks.md).
