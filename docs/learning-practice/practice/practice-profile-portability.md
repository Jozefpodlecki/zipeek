# Practice profile portability

## Product contract

Saved Practice profiles are local-first. Provide **Export JSON** on each profile and **Import JSON** in the saved-profile workspace. Export produces a readable, versioned configuration document; import validates before presenting a preview and saving a new profile. This is configuration portability, not a means of copying an active session or sending a seeded session URL.

The export includes the profile name, exercise types, exercise-specific configuration, eligible vocabulary source and relevant selection settings. It must **not** include progress, session results, attempts, mastery, global exclusions, visitation history, AI service credentials or unrelated personal settings.

Use an explicit export envelope with a schema version independent from the internal local-storage shape. Validate file type, size, parsed structure, exercise options, identifiers, source references and unexpected fields before importing. Imported profiles receive a new local identifier and should permit renaming to resolve duplicates. Never execute or trust user-supplied file content.

A reference to a personal Word Set is not itself a copy of that Word Set. If the destination does not contain the referenced set, show a clear missing-source error and require explicit replacement or cancellation. Do not silently switch to a public HSK source. AI-dependent options must respect the recipient's configured provider availability and cannot carry credentials.

Profile actions belong in the **Saved Practice profiles** workspace: Rename, Duplicate, Export JSON and Delete on the profile menu; Import JSON next to Create profile. Preserve accessible labels, focus order, useful errors, and confirmation before destructive operations. See [Practice outcomes](practice-outcomes.md) and [acceptance](../../testing-quality/acceptance/approved-behavior-acceptance.md).
