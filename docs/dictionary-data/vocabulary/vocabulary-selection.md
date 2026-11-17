# Vocabulary selection and exclusions

## Selection principles

This document defines approved target behavior, separate from current source verification. Vocabulary selection, exposure history and learning evidence are three distinct concerns.

With pedagogical mode enabled, prioritize **rarely encountered words with recent meaningful mistakes**, then **unseen words**, then words with unresolved partial correctness. Repeated successful assessments lower their priority, while mastered words remain eligible for occasional review. A historical mistake must not permanently outrank newer successful evidence. Use only actual assessment verdicts to affect learning evidence.

With pedagogical mode disabled, favor less recently visited eligible words using persisted visitation history. Exposure records do not imply proficiency. Share the underlying visitation store across vocabulary-driven activities where feasible, while allowing activity-specific ranking and bounded history policies. Do not reuse a game's 100-entry cap as a global Practice cap without capacity analysis.

If multiple multiple-choice formats are enabled, distribute those formats approximately evenly over the session. Honor the enabled set, the configured length, accessibility and vocabulary prioritization. Exact equal counts are not required when the length is not divisible by the number of formats.

## Global exclusions

An exclusion applies to one **exact vocabulary entry**, preferably identified by a stable dictionary key. Excluding `好` does not exclude `你好` or `好吃`. Exclusion is a deliberate user preference, not proof of mastery. Excluded entries remain discoverable in Dictionary and Word Sets.

Manage, search, add and restore exclusions in **Word Sets > Excluded vocabulary**. In **Practice Setup > Vocabulary**, display the number of selected-source entries excluded and a link to the manager, but no bypass toggle. Apply the exclusion list always, regardless of selected HSK bands, saved Word Sets or active profile. During an active session, apply changes to subsequently generated exercises. Excluding the current exercise's word is compatible with Skip and Replace; for multiword exercises, ask which vocabulary entry is meant.

## Global pool exhaustion policy

The preference lives in **Settings > Learning > Vocabulary selection**, not Practice Setup. Default: **Reuse eligible words**, preferring least recently visited eligible entries and varying contexts when possible. Alternative: **Finish early** when all unique eligible entries have been used. Persist the preference globally and snapshot it when starting a session so changing Settings does not unexpectedly alter an active session.

When eligible words exist but none is unused, follow the configured policy. When exclusions and source constraints leave **zero eligible words**, explain that no content can be generated, regardless of policy. Never silently override exclusions or expand beyond the selected HSK or Word Set source. A skip cannot endlessly recreate the identical prompt; use bounded retries and explain an exhausted selection pool.

## Storage and privacy

Store global exclusions and visitation locally through the established application storage abstraction. Validate loaded identifiers and data structures. Keep exposure, submitted outcomes, and course acquisition evidence separate. No automatic server transfer of personal histories. See [visitation architecture](vocabulary-visitation.md) and [acceptance](../../testing-quality/acceptance/approved-behavior-acceptance.md).
