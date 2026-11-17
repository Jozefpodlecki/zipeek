# Regeneration

**Current installation:** consult [the installed manifest](../../../../crates/zipeek-web/static/hanzi.manifest.json) for the actual 12,188-row bundle; the old A05 snapshots mentioned later are historical. Commands and tasks below should be checked against the current `Makefile.toml` before execution. Production dictionary generation and installation must not run as an unintended side effect of documentation or E2E work.

Generation is explicit and manual. Commands below run from the repository root.
No source acquisition or database generation runs during application builds.

## Live AI smoke and offline streaming regression coverage

`cargo make db-ai-smoke` sends one single-sense and one two-sense batch classification request through the
AiMux streaming client and taxonomy validator, using the same model, endpoint,
timeout and retry environment settings as `db-full`. It has no database/cache
writes and does not import the full HSK source. A missing final-text response
is reported as an actionable error; this does **not** establish whether the
model only produced reasoning, exceeded its token budget, or the streaming
provider dropped content. Inspect LM Studio's output when this occurs.

For an opt-in direct native invocation:

```powershell
cargo run -p zipeek-db-gen --locked -- ai-smoke --model "qwen3-14b-claude-4.5-opus-high-reasoning-distill" --endpoint http://127.0.0.1:1234
```

When classification work is pending, `db-full` runs the smoke request first
and refuses to generate/install on failure. Offline Rust mock HTTP tests
exercise AiMux aggregation of split SSE content and empty final text; Node
workflow tests verify the preflight gate. They do not replace a live model
or Windows end-to-end acceptance test.

## One-command AI-enriched regeneration

Run `cargo make db-full` from the repository root while the configured LM Studio
model and local inference server are available. This command performs a complete
resumable classification pass **before** creating a candidate, so it does not
create an intermediate unclassified artifact. It then runs the existing native
`zipeek-db-gen generate --classification-cache` export and the existing transactional
bundle installer. The generator verifies exact sense digests, prompt/taxonomy
versions and classification records; logical snapshots include AI-derived tables.

The command checks `zipeek-db-gen classify-plan` before and after inference and refuses
publication unless zero senses remain pending. AI failures preserve checkpoints
in `target/db-workflow/classification-cache/` and leave the installed bundle
untouched. A separate cache file is selected for each model. Environment
settings: `ZIPEEK_AI_MODEL` (default
`qwen3-14b-claude-4.5-opus-high-reasoning-distill`), `ZIPEEK_AI_ENDPOINT`
(default `http://127.0.0.1:1234`), `ZIPEEK_AI_TIMEOUT_MS` (default
`300000`), `ZIPEEK_AI_RETRIES` (default `1`). `cargo make db-full-dev`
performs this workflow before serving the web app.

`cargo make db-generate` and `cargo make dev` remain intentionally offline:
no AI inference and no classification cache is applied. This distinction keeps
routine development independent of the local model. Generated artifacts are
always source-and-cache specific; never hand-edit the DuckDB file.

### Incremental classification checkpoints (R57)

The cache path printed by `db-full` still identifies the model-specific
`<hash>.jsonl` snapshot. Existing classified senses in that JSONL remain
readable. Instead of rewriting the entire growing snapshot after each new sense,
R57 writes one synchronized and atomically renamed
`<hash>.jsonl.records/<word_id>-<sense_order>.json` checkpoint per newly
accepted sense. Native `classify-plan`, `classify`, and enriched export combine
both sources; neither path is a complete standalone export once new records
exist. **Do not delete, compact, or move either cache part while classification
is running.** Unexpected, truncated or duplicate checkpoints halt processing
rather than being silently skipped. Unresolved `.tmp` checkpoint files require
manual inspection before resuming. No installed database is modified unless
all senses pass validation.

Classification-level recovery is implemented separately in
`crates/zipeek-db-gen/src/llm/recovery.rs`: for an empty final response, invalid JSON,
or an invalid taxonomy payload, it attempts the same sense at most three times.
The source gloss, v3 classification prompt and model stay the same on retry.
This policy does not retry storage failures or override AiMux's existing
transport retry setting (`ZIPEEK_AI_RETRIES`). Failed batches never publish
an incomplete database; re-running `cargo make db-full` resumes valid cached
records. The current workflow defers an exhausted sense rather than inventing labels;
the final completeness gate still prohibits installing partial enrichment.

## Choose the command

For the complete local workflow, run from the repository root:

```powershell
cargo make dev
```

This generates a fresh candidate, verifies and installs the whole bundle with
a backup, then starts Trunk at http://localhost:1420. It runs once per invocation;
Trunk rebuilds do not regenerate data. Prerequisites are installed cargo-make,
Node, Rust/native generator prerequisites, Trunk and the locked web npm packages
(`cargo make setup` from the repository root after a clean unpack). Tasks do not install
or upgrade tools/packages. Tested cargo-make version: 0.37.24.

| Task from root | Behavior |
| --- | --- |
| cargo make | List available tasks without changing the database |
| cargo make db-check | Verify/import in memory without writing a database |
| cargo make db-generate | Generate a base candidate under target/db-workflow/generated |
| cargo make db-classify | Validate offline CC-CEDICT labels and generate a NEW enriched candidate; no install |
| cargo make db-install | Install the latest successful candidate; never generate implicitly |
| cargo make db-ai-smoke | Diagnose single-sense and 100-sense LM Studio output; no data/cache writes |
| cargo make db-full | Import offline CC-CEDICT classifications, generate enriched bundle and install (LM Studio opt-in) |
| cargo make db-full-dev | Run db-full, then start Trunk |
| cargo make serve | Start Trunk using the installed bundle |
| cargo make dev | Generate, install and serve in that order; stop on failure |

For explicit review and installation stages, run `cargo make db-generate`,
`cargo make db-classify`, then `cargo make db-install` in that order. The
classification step imports and checks the bundled JSON, then builds a **new**
classified candidate and updates `target/db-workflow/latest.json`; it does not
edit the first candidate in place or replace the installed database. The
installer always uses the latest candidate, and can still install a base
candidate when classification has not been run. Do not mistake a successful
rule-based import for expert-reviewed classifications.

Each run prints absolute candidate/database/backup paths. The latest successful
candidate is recorded in target/db-workflow/latest.json. Installation validates
policy, every declared file's size/hash, required reports/notices and the pinned
INPUTS.toml before any replacement. It stages and rechecks the complete bundle,
backs up the previous crates/zipeek-web/static under target/db-workflow/backups and restores
that backup if publication fails. Candidate/backup directories are retained.
Unsafe paths, symlinks, missing notices and checksum mismatches are refused.
Copy the whole bundle, never just hanzi.duckdb. These operations are explicit
local tasks, not Trunk hooks or application startup behavior.

Direct generator commands remain available:

```powershell
# Show help and examples; no database is written
cargo run -p zipeek-db-gen --locked -- --help

# Verify sources, native engine/storage and import in memory; write no database
cargo run -p zipeek-db-gen --locked -- check

# Write a full bundle to a NEW directory
cargo run -p zipeek-db-gen --locked -- generate --output target/reviewed-dictionary
```

A bare cargo run -p zipeek-db-gen now displays Clap help, does no import/generation and
returns usage status 2. Previously it imported in memory without writing a file;
that behavior is now the explicit check command. Successful check ends with
"No database file was written" plus the generation command to use next.

Generate prints the absolute output directory before work and the absolute
hanzi.duckdb and hanzi.manifest.json paths after success, even if informational
logging is disabled. Relative --output paths resolve from the current working
directory. Paths with spaces can be quoted. --help and --version exit successfully.

## Default output and existing directories

```powershell
# Uses the repository crates/zipeek-web/static directory; only works if it does not exist
cargo run -p zipeek-db-gen --locked -- generate
```

The default is repository-relative crates/zipeek-web/static, independent of the shell's working
directory. Its normal checked-in existence means this command refuses to overwrite
it. Use an explicit fresh review directory for regeneration. Existing outputs are
always refused; there is no implicit overwrite or --force option.

## Review and replacement

1. Generate into a fresh review directory.
2. Compare hanzi.manifest.json, ordered logical rows, schema, indexes and
   hanzi.diagnostics.json against the current crates/zipeek-web/static bundle. Database file
   hashes can differ with identical logical content under D22.
3. Preserve a backup of the previous complete bundle. Never patch database bytes
   or replace only the database without its corresponding reports/notices.
4. After approval and verification, replace the complete crates/zipeek-web/static bundle:
   hanzi.duckdb, manifest, diagnostics, INPUTS.toml, ATTRIBUTION.md and notices.
5. Repeat the applicable native/read-only and pinned browser compatibility checks
   in [browser-db](../../../testing-quality/e2e/browser-db.md). Old compatibility evidence does
   not automatically validate a new artifact.

The approved sources/schema/dependencies do not change merely by regeneration.
Native generation requires v2.0.0 and v1.4.0 storage compatibility before output
creation. Artifact export checks complete logical equality and read-only reopening.

## Static serving and measured size

Trunk copies crates/zipeek-web/static contents to the distribution root. The Pages database
URL remains /zipeek/hanzi.duckdb; development uses /hanzi.duckdb. Reports and
notices accompany the database. The earlier A05 snapshot had 12,100 readings (historical, not the installed manifest),
514 diagnostics and a 3,158,016-byte database; file size is not a performance claim.

## References

- [Sources and notices](../../../dictionary-data/schema/sources.md)
- [Import rules](../../../dictionary-data/schema/import-rules.md)
- [Artifacts](../../../dictionary-data/schema/artifacts.md)
- [Logical reproducibility](../../../dictionary-data/schema/reproducibility.md)
- [Generator module](../../../architecture/modules/db-gen.md)

## R56 source-tag normalization and deferred inference

The primary HSK feed uses ICTPOS-style subtypes that the 42 canonical labels
cannot represent losslessly. `import/pos_mapping.rs` handles source-specific
normalization without weakening `PartOfSpeech::from_source_code`:

| Source code | Source meaning | Canonical label | Tradeoff |
| --- | --- | --- | --- |
| `qv` | Verbal/event measure word | `classifier` | Subtype collapsed |
| `qt` | Temporal measure word | `classifier` | Subtype collapsed; not automatically a `time_word` |
| `mq` | Numeral-measure expression | `numeral` | Multi-PoS construction collapsed to single canonical label |
| `cc` | Coordinating conjunction | `conjunction` | Coordination subtype collapsed |

Unknown codes outside the reviewed set still produce diagnostics and do not
invalidate the whole entry. The mappings describe **source-level metadata**, not
AI-derived sense labels. Revisit `mq` and `cc` if the canonical taxonomy gains
additional subtypes. Reference: ICTPOS 3.0 tag descriptions in
https://github.com/memect/kg-beijing/wiki/%E4%B8%AD%E6%96%87%E8%AF%8D%E6%80%A7%E6%A0%87%E8%AE%B0%E9%9B%86 .

`zipeek-db-gen classify` now defers empty/invalid JSON/taxonomy outputs after the
existing three-attempt classifier policy, continues the normal pass, and tries
those deferred senses once more at the end (again at most three attempts).
Reports are JSONL next to the model cache at `<hash>.failures.jsonl`, with
sense ID, original gloss, model, v3 prompt identity, attempt count, error and
Unix timestamp. Successful classifications retain the same model-specific
cache; neither prompt version nor taxonomy version changes.

`zipeek-db-gen classify` may finish with nonzero `Remaining` and `Deferred` while
returning successfully for its batch. `db-full` independently calls
`classify-plan` and refuses generation **and installation** until `Remaining=0`.
Running the command again retries the still-pending senses; it does not accept
empty classifications or invent missing labels. Fatal transport, cache, and
storage failures still abort the batch immediately. The diagnostic report is
recreated for the current run and cleared when all attempted senses recover;
if interrupted, the last saved report survives until the next run begins.


### R111 - Offline AI classification V4 (2026-11-04)

**Developer-only db-gen workflow.** No learner-facing route, AI dashboard, or public model endpoint is added. `zipeek-db-gen classify` sends complete batches of *senses*, not words, to the configured local LM Studio model. The standalone `classify` CLI defaults to 20 senses/request; the `db-full` workflow defaults to a conservative 2 senses/request pending live verification; override with `--batch-size 10`, `20`, `40`, or another positive size up to 100. `ZIPEEK_AI_BATCH_SIZE` controls the `crates/zipeek-db-gen/src/ai_workflow.rs` orchestration; `ZIPEEK_AI_MODEL`, endpoint and timeout retain existing semantics. Context capacity (30k tokens) is not a promise that a 100-sense response fits the output-token budget.

The V4 system prompt distinguishes English sense glosses, grammatical functions and category evidence, prohibits cross-sense label leakage, and treats lexical relationships and source annotations as data rather than an excuse to infer additional semantics. `results` must contain **exactly one** record per requested `(word_id,sense_order)`, no unexpected/duplicated IDs, and only allowed labels. All records are validated before any records from a request are checkpointed. On malformed output or invalid classification, a multi-sense request is split into halves recursively (iteratively scheduled), while single-sense fallback uses the previous bounded recovery procedure. Non-recoverable model/transport errors abort with previous checkpoints intact; malformed single-sense output is deferred and reported.

The Rust workflow now uses a **V6-specific** `<model-hash>-v6.jsonl` cache and adjacent per-sense checkpoint directory. Do not point V6 generation at a V5 cache: old prompt labels are intentionally reclassified, not silently reused, and old checkpoint filenames collide with new records. Cache identity remains model + prompt version + taxonomy version + digest; individual accepted records remain usable after interruptions. `db-full` refuses to generate/install the database until classify-plan reports zero pending senses. The base DuckDB schema, deterministically extracted lexical relations and schema generator are unchanged.

**Benchmark before increasing batch sizes.** Compare 10, 20, 40, and optionally 100 on the same representative set including polysemous senses, function words, classifiers, proper names and explicit lexical relations. Capture wall time, accepted senses/minute, requests, retries and deferrals. The CLI logs `generated_senses_per_minute` and `requests`; record local model and generation settings with the benchmark. Observe response completeness and classification precision, not only throughput. No benchmark measurement against the user's LM Studio model is claimed in this checkpoint.
