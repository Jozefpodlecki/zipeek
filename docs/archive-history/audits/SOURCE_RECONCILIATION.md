# Source-to-documentation reconciliation

**Inspection date:** 2026-11-09. **Baseline:** `zipeek-R282-WIP-documentation-consolidation-20261109-153451.zip`. This is a source-to-documentation review, not a Rust/Chrome execution report.

## Method and precedence

Implementation statements were checked against relevant Rust/JS source, `Cargo.toml`, `package.json`/lockfiles, `schema.sql`, `Makefile.toml` and the installed JSON manifest. Earlier release contracts are preserved as historical rationale, not reinterpreted as present runtime behavior. For product intent, pending decisions remain explicitly proposals. This inspection cannot independently attest that the installed DuckDB file matches every manifest table or that an authored E2E suite passes without executing Rust/Chrome.

## Evidence map

| Implemented area | Current source | Maintained documentation | Finding |
| --- | --- | --- | --- |
| Routing and 34 route cases | [`crates/zipeek-web/src/route.rs`](../../../crates/zipeek-web/src/route.rs) | [docs/architecture/architecture.md](../../architecture/architecture.md) | Confirmed source routes; 34 concrete E2E cases are not every parameterized route. |
| Welcome/preferences JSON | [`crates/zipeek-web/src/preferences.rs`](../../../crates/zipeek-web/src/preferences.rs) | [docs/learning-practice/practice/settings-workspace.md](../../learning-practice/practice/settings-workspace.md) | Nine serde-default fields; the preferences key is the onboarding marker. |
| AI credentials and settings | [`crates/zipeek-web/src/services/ai/config.js`](../../../crates/zipeek-web/src/services/ai/config.js) | [docs/learning-practice/practice/settings-workspace.md](../../learning-practice/practice/settings-workspace.md) | Endpoint and model in browser preferences; optional API token only in JS module memory. |
| Six Settings tabs | [`crates/zipeek-web/src/pages/system/settings/navigation.rs`](../../../crates/zipeek-web/src/pages/system/settings/navigation.rs) | [docs/learning-practice/practice/settings-workspace.md](../../learning-practice/practice/settings-workspace.md) | Appearance, Characters, AI, Images, Practice and Personal Data. |
| Dictionary manifest facts | [`crates/zipeek-web/static/hanzi.manifest.json`](../../../crates/zipeek-web/static/hanzi.manifest.json) | [docs/dictionary-data/schema/db.md](../../dictionary-data/schema/db.md) | 12,188 declared readings, installed versions, listed artifacts and normalized tables. |
| Generator SQL | [`crates/zipeek-db-gen/src/schema.sql`](../../../crates/zipeek-db-gen/src/schema.sql) | [docs/dictionary-data/schema/schema.md](../../dictionary-data/schema/schema.md) | Normalized tables and ten-column words projection; character_readings in SQL. |
| Dictionary expansion and fallback | [`crates/zipeek-db-gen/src/generator.rs`](../../../crates/zipeek-db-gen/src/generator.rs) | [docs/dictionary-data/schema/artifacts.md](../../dictionary-data/schema/artifacts.md) | HSK source + CC-CEDICT expansion and fallback. |
| Artifact export and reopened validation | [`crates/zipeek-db-gen/src/artifact/writer.rs`](../../../crates/zipeek-db-gen/src/artifact/writer.rs) | [docs/architecture/modules/db-gen.md](../../architecture/modules/db-gen.md) | Staged, logical comparison and read-only reopen. |
| Installed WASM package pin | [`crates/zipeek-web/package.json`](../../../crates/zipeek-web/package.json) | [docs/dictionary-data/schema/runtime.md](../../dictionary-data/schema/runtime.md) | 1.33.1-dev114.0 in package/lock and generator policy, approved 2026-11-09, superseding older policy. |
| Browser query facade and metadata | [`crates/zipeek-web/src/services/database/runtime.rs`](../../../crates/zipeek-web/src/services/database/runtime.rs) | [docs/architecture/modules/browser-db.md](../../architecture/modules/browser-db.md) | Private generic query with public typed storage_statistics(). |
| Browse UI | [`crates/zipeek-web/src/pages/explore/browse.rs`](../../../crates/zipeek-web/src/pages/explore/browse.rs) | [docs/architecture/modules/browser-db.md](../../architecture/modules/browser-db.md) | Delegates to shared Browse form and results. |
| Browse typed SQL | [`crates/zipeek-web/src/services/database/browse_queries.rs`](../../../crates/zipeek-web/src/services/database/browse_queries.rs) | [docs/dictionary-data/schema/queries.md](../../dictionary-data/schema/queries.md) | Connected parameterized normalized Browse search. |
| Practice draft and UI | [`crates/zipeek-web/src/pages/practice/setup/page.rs`](../../../crates/zipeek-web/src/pages/practice/setup/page.rs) | [docs/learning-practice/practice/practice-session-builder.md](../../learning-practice/practice/practice-session-builder.md) | Source/steps/exercise setup uses typed draft. |
| Practice launch | [`crates/zipeek-web/src/services/practice_launch.rs`](../../../crates/zipeek-web/src/services/practice_launch.rs) | [docs/learning-practice/practice/practice-session-builder.md](../../learning-practice/practice/practice-session-builder.md) | Translation-only UUID session, count, direction and endless flag. |
| Practice temporary persistence | [`crates/zipeek-web/src/services/practice_session_store.rs`](../../../crates/zipeek-web/src/services/practice_session_store.rs) | [docs/dictionary-data/schema/personal-data.md](../../dictionary-data/schema/personal-data.md) | Validated temporary UUID records and scored outcomes. |
| Practice session lifecycle | [`crates/zipeek-web/src/pages/practice/local_session.rs`](../../../crates/zipeek-web/src/pages/practice/local_session.rs) | [docs/architecture/modules/sentence-exercises.md](../../architecture/modules/sentence-exercises.md) | Generated step preparation, resume, score and clear-on-completion. |
| Translation generation and canonical tiles | `crates/zipeek-ai/src/lib.rs` (retired location; ownership requires re-audit) | [docs/learning-practice/translation/translate-sentence.md](../../learning-practice/translation/translate-sentence.md) | Seeded ten-word pool; model-generated Chinese/English only. |
| Generated-translation rendering/evaluation | [`crates/zipeek-web/src/pages/practice/generated_session.rs`](../../../crates/zipeek-web/src/pages/practice/generated_session.rs) | [docs/architecture/modules/sentence-exercises.md](../../architecture/modules/sentence-exercises.md) | Per-step generated answer and AI evaluation. |
| Backup/import and selective deletion | [`crates/zipeek-web/src/services/personal_data/mod.rs`](../../../crates/zipeek-web/src/services/personal_data/mod.rs) | [docs/dictionary-data/personal/personal-data-storage.md](../../dictionary-data/personal/personal-data-storage.md) | No durable completed-learning-result writer. |
| Personal OPFS test | [`crates/zipeek-db/src/wasm/probe.rs`](../../../crates/zipeek-db/src/wasm/probe.rs) | [docs/dictionary-data/schema/personal-data.md](../../dictionary-data/schema/personal-data.md) | Isolated probe, not learner-history authority. |
| Rust fake AI API | [`crates/zipeek-e2e/src/harness/ai_fixture.rs`](../../../crates/zipeek-e2e/src/harness/ai_fixture.rs) | [docs/archive-history/audits/e2e.md](e2e.md) | In-process deterministic fixture replaces removed JS server. |
| Live and fixture translation browser suites | [`crates/zipeek-e2e/tests/translation_fixture.rs`](../../../crates/zipeek-e2e/tests/translation_fixture.rs) | [docs/testing-quality/acceptance/testing.md](../../testing-quality/acceptance/testing.md) | Authored tests; newest UUID workflow still unaccepted in Chrome. |
| Full E2E command | [`Makefile.toml`](../../../Makefile.toml) | [docs/testing-quality/acceptance/testing.md](../../testing-quality/acceptance/testing.md) | cargo make e2e-full compiles and runs managed suites. |
| Read-only schema proposal status | [`crates/zipeek-db-gen/src/schema.sql`](../../../crates/zipeek-db-gen/src/schema.sql) | [docs/dictionary-data/schema/personal-data.md](../../dictionary-data/schema/personal-data.md) | No authoritative personal DB tables/writer in this generator. |

## Resolved documentation drift

- Replaced original HSK-only/single-table claims with the normalized generator schema and CC-CEDICT expansion.
- Replaced historical 12,100-row artifact figures in current guides with the **installed manifest's declaration of 12,188 words**. Previous run counts remain historical evidence.
- Corrected runtime package documentation to the installed **`1.33.1-dev114.0`** pin, without changing dependencies or silently amending approved policy.
- Restored the current six Settings tabs and nine-field serialized preferences contract instead of the old five-tab design.
- Replaced R54 model-authored word-bank claims with application-owned canonical answer tiles and the UUID translation Practice workflow.
- Updated current Browse documentation: the normalized search endpoint is connected; the “not yet connected” statement was obsolete.
- Removed current-facing use of retired `crates/db-gen` paths in generator/testing docs; old paths remain only in historic decisions and completion logs.
- Rewrote personal-storage documentation to distinguish `localStorage`, JSON backups and OPFS probes from non-existent persistent pedagogical history.
- Separated root/Pages browser acceptance evidence from the latest unexecuted translation integration.

## Unresolved issues requiring code/artifact or policy decisions

1. **DuckDB-WASM pin reconciled (2026-11-09):** the user explicitly approved `1.33.1-dev114.0`. D08 and AGENTS now name the exact installed version; the former `1.32.0` pin is retained only as historical provenance. No dependency or artifact bytes were changed in this documentation pass.
2. **Manifest table inventory:** `crates/zipeek-db-gen/src/schema.sql` declares `character_readings` and `src/artifact/writer.rs` bulk-copies it; the checked-in manifest `tables` list lacks this table. Verify the actual binary (and manifest report generator) before claiming an installed `character_readings` query is supported. Do not regenerate or alter the database solely for documentation.
3. **Acceptance gap:** Latest translation-only UUID Practice integration and the full root/Pages E2E matrix lack user-provided Windows passing results for that exact source. Previous Windows results cover the 31 E2E library unit tests, accessibility, Browse and an older standalone live AI translation path. Do not carry them forward as proof of the new journey.
4. **Architecture proposals:** Remaining unapproved technical choices are tracked in [active tasks](../../project-management/tasks.md). Retired UX requests are not active specifications.
5. **Legacy archived records:** `docs/process/completed/`, `docs/process/decisions/`, `docs/archive-history/RELEASE_HISTORY.md`, and dated mockups describe their own historical state. Their old paths, counts and dependency choices are retained for traceability, never treated as active contracts.

## Verification scope

This pass updates documentation only. It does not change Rust, JS, Cargo, manifest, DuckDB or E2E source. Link and repository checks are recorded below; tests listed in source were not executed by this audit. **Completeness boundary:** this mapping covers major implemented features and known contracts, but is not a formal proof that every line of every historical product document exactly matches runtime code. Where a discrepancy could not be settled by source inspection, it remains openly listed here.


Final documentation scan: 709 local Markdown links checked, 0 missing targets. Node source suite: 661 passed, 14 pre-existing failures (675 total). Source comparison against the input ZIP: 28 modified Markdown/release-note files, 1 new Markdown files, 0 removed files and 0 non-document byte changes.

## Additional product-direction reconciliation (2026-11-09)

The [Courses mosaic specification](../../learning-practice/courses/courses-mosaic.md) defines the target behavior. The older concept image has been retired because it no longer represents the current UI. The specification is approved future requirements, **not current shipped features**. Current source [`pages/learning/courses/listing.rs`](../../../crates/zipeek-web/src/pages/learning/courses/listing.rs) still presents a card catalogue and `progress_store.rs` (retired location) stores a list of completed Pinyin lesson slugs. Course-point assessments, mosaic geometry and permanent grammar-cell reveal are not there.

The user also approved the direction of [session-first pedagogical history](../../learning-practice/practice/learning-evidence.md): derived per-Hanzi, exercise, category, HSK and grammar observations, a minimum seven-day observation window, ten encounters across multiple sessions and demonstrated recognition/listening/speaking/sentence-use. The current implementation does not write these records. Detailed scoring, number of distinct sessions, audio reliability and retention rules remain unresolved. The source-to-documentation audit treats these policies as future design, not implemented runtime contracts.

## R526 source map follow-up

The earlier source mapping above contains references to modules that no longer exist at their original paths. The old `zipeek-ai` crate has been split, and Courses progress implementation has moved. The associated behavioral assertions are historical and require a dedicated source reconciliation before being treated as current evidence.
