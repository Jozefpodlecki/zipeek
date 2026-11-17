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

The R283 unverified checkpoint adds an opt-in personal DuckDB Practice-outcome writer with commit/readback-before-cleanup behavior, separate from the diagnostics and snapshot/probe functionality. A browser-local OPFS snapshot is a point-in-time backup, not a live transactional learning store. Do not claim real-world persistence until a browser-verified commit and readback succeed. The completion view only reports success after that operation returns successfully.

The Practice completion screen now distinguishes recorded, disabled and retryable failure outcomes; recording defaults to off. The **approved future direction** is session-first pedagogical evidence with derivation of per-Hanzi/word, exercise, HSK/category and grammar observations. The eligibility floor is seven distinct active dates and ten qualifying scored encounters over at least three sessions with a per-session cap, plus separately assessed recognition, listening, speaking and sentence use; this does **not** constitute a running proficiency algorithm. See [learning evidence](../../../learning-practice/practice/learning-evidence.md). The new default-off opt-in and minimal v1 outcome table are source-implemented but not browser-verified; retention, portable export/import, migrations and durability acceptance remain open.

## Data risks and planned acceptance

Browser-local storage can be deleted by site-data clearing and may fail due to quotas or browser restrictions. JSON export is the user-controlled portability path for the categories that it actually exports. OPFS and future personal DuckDB adoption require close/reopen, reload, cross-tab, quota, corrupt-record, export/import and deletion coverage.

See [schema design](../../../dictionary-data/schema/personal-data.md), [current implementation](../../../project-management/current.md) and [open tasks](../../../project-management/tasks.md).
