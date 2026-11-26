# Personal learner data and storage

## Existing browser storage

Zipeek uses a generated read-only DuckDB dictionary and separate learner-owned browser storage. The dictionary must not be described as the writable destination for learning results.

- `zipeek.preferences.v1`: consolidated appearance/character preferences, explicit default-off Practice-history recording consent (`learning_history_enabled`), and non-secret AI endpoint/model configuration. The optional AI API key is memory-only.
- `zipeek.user_word_sets.v1`: personal Word Sets. Sets acquired through Courses carry an optional typed `source` with one-based `region_id` and `set_index`. Only this source identity establishes acquisition; matching display names are not proof of acquisition. Renaming or editing words preserves the source, deletion removes acquisition. Pre-existing source-less sets remain personal but are not automatically attributed to courses.
- `zipeek.excluded_words`: vocabulary exclusions.
- `zipeek.crosswords.rotation.v1`: optional browser-local crossword variety history. Contains the 100 most recently used distinct Hanzi answers with usage sequence/count and the 10 most recent puzzle word lists and session IDs; assists seed selection but does not change restored crossword solutions or Practice learning evidence. Separate from `zipeek.crosswords.v5.<session-id>` puzzle snapshots. This key is not included in Word Set personal-data exports.
- `zipeek.practice.session.v1.<uuid-v7>`: temporary session JSON containing one configuration per exercise type, ordered step instances with ISO start/finish timestamps, and session lifecycle timestamps. Finished records are removed after successful optional learning-history handling. Existing records with an incompatible JSON shape are not migrated; they fail current-session validation and must be cleared explicitly if they block new sessions.

The Settings Data & Storage panel supports a versioned JSON personal-data backup/import for Word Sets and exclusions, plus selective deletion. This does **not** imply backups include temporary Practice records or an authoritative learning-history database. Existing importer validation and deletion confirmation remain application features.

Pinyin modules now persist passed **written recognition checks** in `zipeek.courses.progress.v1` after two correct answers. Opening a lesson or listening to an example never marks a module as passed. The same browser-local key saves Grammar step checkpoints and best draft assessment scores, but **never** awards full listening/speaking competence, independently verified grammar mastery or mosaic completion. This category is not included in the current Word Set/exclusions backup/export interface; clearing browser site data can remove it. Browser storage failures are surfaced in the lesson UI. No old manual-completion key is read or written.

## Personal OPFS/DuckDB status

The inspected but browser-unverified source includes an opt-in personal DuckDB Practice-outcome writer with commit/readback-before-cleanup behavior, separate from the diagnostics and snapshot/probe functionality. A browser-local OPFS snapshot is a point-in-time backup, not a live transactional learning store. Do not claim real-world persistence until a browser-verified commit and readback succeed. The completion view only reports success after that operation returns successfully.

The Practice completion screen now distinguishes recorded, disabled and retryable failure outcomes; recording defaults to off. The **approved future direction** is session-first pedagogical evidence with derivation of per-Hanzi/word, exercise, HSK/category and grammar observations. The eligibility floor is seven distinct active dates and ten qualifying scored encounters over at least three sessions with a per-session cap, plus separately assessed recognition, listening, speaking and sentence use; this does **not** constitute a running proficiency algorithm. See [learning evidence](../../learning-practice/practice/learning-evidence.md). The new default-off opt-in and minimal v1 outcome table are source-implemented but not browser-verified; retention, portable export/import, migrations and durability acceptance remain open.

## Development-only persistence simulator (R613 WIP)

The debug DuckDB diagnostics panel has explicit Start/Stop controls for a synthetic persistence run: at most ten cycles scheduled five seconds apart. Every cycle uses the existing personal DuckDB-WASM/OPFS file and exclusive Web Lock, atomically updates the isolated `persistence_simulator` table, checkpoints, closes/reopens, and verifies the revision and deterministic payload. The latest saved run can be checked again after a page reload with **Verify saved state after reload**. Background tab timer throttling and DuckDB startup time can extend the actual interval. Stop prevents later cycles; an in-flight transaction may still finish.

The simulator neither writes real Practice sessions nor changes R608 learner scores. Its synthetic table is dropped when personal learning history is erased. Browser acceptance and failure-injection checks have not been executed; the implementation does **not** establish durable Knowledge Points persistence.

## Data risks and planned acceptance

Browser-local storage can be deleted by site-data clearing and may fail due to quotas or browser restrictions. JSON export is the user-controlled portability path for the categories that it actually exports. OPFS and future personal DuckDB adoption require close/reopen, reload, cross-tab, quota, corrupt-record, export/import and deletion coverage.

See [schema design](../schema/personal-data.md), [current implementation](../../project-management/current.md) and [open tasks](../../project-management/tasks.md).

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/dictionary-data-personal-personal-data-storage.md). Dated implementation claims in that snapshot are not current build or browser verification.

## Confirmed app reset (2026-11-26)

Settings has a Reset app section with an explicit destructive confirmation. It permanently removes Zipeek-owned preferences, course progress, Word Sets, exclusions, saved sessions, personal snapshots and learning history, then reloads the current deployment base at `#/` to show fresh landing/onboarding. The packaged dictionary, model files, exported files outside application storage and unrelated same-origin data are retained. This reset scope was explicitly approved by the user; the browser confirmation is still required when using it.

Confirmation stores a one-use session authorization and fully reloads into the isolated `#/reset-app` bootstrap. Normal UI providers, microphone state, credentials in WASM memory and browser workers are discarded before cleanup. Opening that URL directly does not authorize deletion. Normal documents hold a shared `zipeek-app-storage-session` Web Lock; the reset runner requires an exclusive lock and refuses while another app tab/window is active. A newly opened app document waits for the reset lock before normal startup. Browsers without the required storage/locking APIs show a failure instead of claiming success.

Browser/WebView cleanup removes `zipeek.*` local/session keys and the app's `RUST_LOG` preference, and deletes only top-level personal OPFS entries beginning `zipeek-` or `zipeek.`. It does not clear the whole origin or WebView profile. Current DuckDB personal history, its WAL, snapshot and probe files use those owned names. Native `personal_data_reset` requires an explicit confirmed argument; its serialized database worker closes owned connections, removes only the expected `zipeek-personal.duckdb` and its WAL, and reinitializes empty personal storage on the next normal open. Unexpected paths, symlinks/directories and filesystem errors are rejected.

Cleanup across browser and native storage is not atomic. Errors remain visible in the isolated runner with an explicit partial-cleanup message and Retry. Success navigation occurs only after every required stage succeeds; retries are idempotent. Data reset cannot be undone. Exported backups can restore only the categories supported by the existing importer, not all removed history. Canceling the initial confirmation leaves data unchanged.
