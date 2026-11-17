# AI dictionary classification - operational guide

## Safety

`cargo make db-full` is a potentially long, cache-writing classification and bundle-publication operation. It is **not** a smoke test. Publication is gated on zero pending classifications; a failed run must leave the installed database unchanged. The workflow checkpoints successful classifications, but confirm this against local output.

## Why the 2026-11-05 run stalled

The single-sense smoke succeeded, but the subsequent 20-sense requests repeatedly failed strict JSON decoding (`expected value at line 1 column 1`). The batch scheduler split failures into smaller batches, then individual requests; several individual responses also failed. This is a **response-format / response-extraction compatibility problem**, not evidence of bad source data. The exact response prefix was not present in the supplied logs, so we cannot yet conclude whether it is Markdown, a `<think>` block, empty final text, or some other prefix.

## Changes in R173

- Live smoke tests **one single sense and one two-sense batch**, without cache/database writes. A model that only handles singles now fails before the full job.
- Known response envelopes (`<think>…</think>` and Markdown JSON fences) can be stripped before strict parsing. Validation of allowed labels, IDs, completeness and field names remains strict.
- Rejected batch/single responses log an escaped, bounded 120-character prefix for diagnosis. Do not log or share private inputs; classification source data is dictionary content.

## Run order (PowerShell)

```powershell
cargo test -p db-gen --locked
cargo make db-ai-smoke
```

If smoke fails, stop. Check the `response_prefix` field and LM Studio logs; adjust output formatting or token limits based on actual evidence, not repeated prompt guesswork. For bounded classification after smoke, use `cargo run -p db-gen --locked -- classify --help` to obtain the actual argument names. Start with `--limit 20 --batch-size 2` and the same model, cache and endpoint. Back up the cache before changing model/prompt versions. Do not run `cargo make db-full` until the bounded job produces valid, reproducible results at useful throughput.

## Verification

A successful local 2-sense smoke is **necessary, not sufficient** for 20-sense batches. Evaluate 5-, 10- and 20-sense batches progressively using bounded classify runs before raising the default from 2. The 2026-11-05 full-run output provides no proof that larger batches can be decoded correctly.

## R174: expanded semantic taxonomy and rejected response evidence

The v6 prompt has a controlled taxonomy with more than 50 defined subjects, shared between the prompt and the database schema. The taxonomy version is 2, so old classification caches cannot be reused. This is intentional: rebuild from pinned dictionary sources.

Malformed model completions are now saved in full, by default, to `target/db-workflow/classification-rejections.jsonl` (one JSON object per line, including model, sense IDs, rejection reason, raw response and timestamp). This diagnostic file is not the classification cache and is not included in the generated dictionary. Override the path with `ZIPEEK_CLASSIFICATION_REJECTION_LOG`; set it to `off` to disable logging. Treat raw outputs as untrusted data and review them locally before sharing.

For example, PowerShell: `Get-Content .\target\db-workflow\classification-rejections.jsonl -Tail 1 | ConvertFrom-Json | Select-Object -ExpandProperty response`.

**Current model issue:** responses beginning with `Assistant` or `system` are reasoning-like prose, not strict JSON. More detailed prompts alone cannot force this model to obey structured output. Smoke must pass first. If it continues to fail, use an LM Studio model or inference setting that supports JSON schema/constrained generation, or implement a provider-specific structured-output mode with tests before running all senses. Do not extract plausible classification labels from unstructured reasoning.

## R175: use Aimux's structured output contract

The R174 smoke logs prove that the Qwen model sometimes returned role-labelled reasoning in the **final text** (`system\n...` or `Assistant\n...`). These strings are not dictionary classifications. They must never be salvaged as labels from prose.

R175 sends `GenerateTextOptions.output = Some(Output::Object { schema, ... })` specifically for classifier requests. The schema is constructed directly from the shared `POS` and `CATEGORIES` allowlists, with exact top-level keys, nonempty PoS, unique labels, and required ID fields in batches. The existing domain validator still checks sense identity, duplicate IDs, taxonomy, and other invariants after Aimux's parsed `output()`. Classifier requests use temperature 0 and a bounded final-output token budget. The generic LLM client retains its prior behavior for other features.

Structured classification remains **streamed** because Aimux's `StreamTextResult::consume` preserves a `GenerateTextResult` when the output parser rejects the model text, so full rejected text and metadata can be saved. Non-streamed `generate_text` can return a parse error without exposing that result. A provider that does not support JSON-schema output may reject the call outright; do not silently switch to an unrestricted request.

When a model response is rejected, `classification-rejections.jsonl` includes the raw final text and a `diagnostics` field with the final step's reported reasoning, unified/raw finish reasons, usage, and provider warnings. This helps distinguish reasoning-only text, truncation, and unsupported provider options. The diagnostics file is local and must never be published as part of a generated bundle. Avoid sharing prompts or API credentials from local logs.

Run `cargo test -p db-gen --locked`, then `cargo make db-ai-smoke`. Confirm that single-sense and two-sense responses both pass. If it fails, inspect the entire last diagnostic entry:

```powershell
Get-Content .\target\db-workflow\classification-rejections.jsonl -Tail 1 |
  ConvertFrom-Json | Format-List *
```

Do not start `db-full` based on passing Node source tests. The structured-output request's actual behavior against the installed Aimux provider and LM Studio model remains a live acceptance test.

## R177: 100-sense smoke and observed JSON suffix

The 2026-11-05 `db-full` log contains a nearly valid two-sense JSON response ending in `..`, which Aimux rejects as trailing characters. It also contains individual responses beginning `system` followed by reasoning prose; those remain invalid and must not be mined for labels. R177 narrowly normalizes **one or two trailing periods after an otherwise complete JSON object** when Aimux parsing fails, then applies the existing domain validator for IDs, PoS and categories. Prose is rejected as before.

`cargo make db-ai-smoke` now runs a single sense plus one **100-sense** request, chosen evenly from the real imported dictionary; it may be much slower than the earlier two-sense smoke. No classification cache/database/published bundle is changed. The batch output allowance scales to 16,384 tokens for 100 senses. If the model does not handle that size, inspect `target/db-workflow/classification-rejections.jsonl` and retry at an intermediate size via `cargo run -p db-gen --locked -- ai-smoke --model <MODEL> --batch-size 20`. The `db-full` workflow deliberately retains a small preflight matching its production batch setting; a failure of the optional 100-sense stress test does not imply that 2-sense classification must fail. Do not increase `ZIPEEK_AI_BATCH_SIZE` to 100 until its live response has passed strict validation and throughput is acceptable.

## R178: deterministic CC-CEDICT classification export

`node scripts/cedict-classify.cjs` reproduces the full CC-CEDICT gloss-level JSON export under `crates/zipeek-db-gen/data/classifications/`. This is a **heuristic annotation aid** for reviewing the taxonomy, **not** a replacement for model/lexicographer review and **not** automatically ingested by `db-full`. Raw dictionary line/gloss locations differ from the 32,500 HSK source-sense IDs. Use `needs_review` and `evidence` to audit the suggested assignments; records with insufficient evidence retain `pos: ["unclassified"]` and empty categories. The source is CC BY-SA 4.0; retain attribution and license with any redistribution.

## R179: offline CC-CEDICT classification import is the default for `db-full`

`cargo make db-full` no longer contacts LM Studio by default. It converts the bundled R178 rule-generated gloss classifications to the exact `CachedClassification` records required by the current imported HSK senses, then uses the existing verified publisher and atomic installer. Source-line plus gloss-index are used for CC-CEDICT fallback senses. HSK-primary senses match only when simplified form, normalized pinyin, and exact English gloss agree uniquely. Conflicting or missing matches become `pos:["unclassified"]`, `categories:[]`; **they are not guessed or assigned invented IDs**. The command prints exact matches, review flags, ambiguous matches, and unmatched counts. The full 199,699-gloss export remains alongside the pinned source and CC BY-SA attribution.

The import verifies the source checksum, taxonomy version, export record count, duplicate locators, and all allowed labels. It refuses to overwrite an existing generated classification cache and validates all newly generated records against their database-sense digests before publication. The default cache is `target/db-workflow/cedict-gloss-rules-v1.jsonl`. If source inputs change and the cache becomes stale, remove that generated cache and retry; the installed database remains unchanged on failure.

**Quality warning:** This is a deterministic heuristic export, not AI-reviewed data. A large proportion of imported senses may remain unclassified, and some exact matches are flagged for review. `db-full` now makes this data available to Browse without claiming expert linguistic validation. Review the summary and inspect a sample of classified senses before relying on the labels for pedagogy.

`cargo make db-ai-smoke` remains an explicit LM Studio diagnostic task; it is **not necessary for the offline default**. If needed, opt into the previous full LM Studio pipeline for `db-full` with PowerShell `$env:ZIPEEK_DB_CLASSIFIER='lmstudio'`, then remove the variable after use. To regenerate the rule-classification export, run `node scripts/cedict-classify.cjs` and delete the generated cache before the next `db-full` run.

## R181 - independent offline classification stage

Use `cargo make db-generate` to create the base candidate, then
`cargo make db-classify` to validate/import the shipped CC-CEDICT rule labels
and generate a **second, enriched candidate**. Finally run
`cargo make db-install` to install the latest verified candidate with the
existing backup/rollback publisher. Classification never directly modifies
an installed database. It may reuse the validated JSONL cache at
`target/db-workflow/cedict-gloss-rules-v1.jsonl`; remove that cache if the
bundled export has been regenerated and the cache is stale.

`cargo make db-full` retains the one-command convenience workflow.
`ZIPEEK_DB_CLASSIFIER=lmstudio` remains an optional path when the local model
has passed its independent smoke tests. `db-install` intentionally accepts
unenriched candidates for fast development; ensure `db-classify` completed
before installing when enrichment is required.
