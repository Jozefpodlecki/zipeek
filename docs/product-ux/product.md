# Product specification

## Purpose and boundaries

Zipeek helps learners study Chinese vocabulary, Hanzi, pronunciation, grammar and contextual usage. The application includes Dictionary, Word discovery, Word Sets, Practice, Courses, Games, translation and writing tools. Product requirements are not evidence that a route has been implemented or accepted by browser tests.

## User journeys

- **Discover:** Look up an entry, inspect its simplified and traditional Hanzi, reading, senses and available HSK metadata, then save it to a personal Word Set or open a relevant learning tool.
- **Organize:** Create and edit personal Word Sets; manage exact-entry global exclusions separately from sets. Entries excluded from Practice remain discoverable in Dictionary.
- **Practice:** Configure a path, exercise types, vocabulary source and optional exercise settings. Persist the resulting session so it can be resumed, or create fresh content with the saved profile configuration.
- **Learn:** Use Courses for Vocabulary, Grammar and Pinyin. Pedagogical unlocking depends on qualifying persisted evidence, not simply opening a page.
- **Play:** Start games through explicit setup and preparation stages when applicable. Games must not silently initiate AI generation during setup.

## Product requirements by owner

| Area | Maintained requirements |
| --- | --- |
| Application shell, copy, accessibility and appearance | [UX overview](ux.md), [platform policy](foundation/interface-and-platform-policy.md) |
| Header, long dragon and responsive navigation | [Header and dragon](foundation/header-and-dragon.md) |
| Practice setup, session and exercise foundation | [Practice overview](../learning-practice/practice/practice-overview.md), [shared exercise foundation](../learning-practice/practice/shared-exercise-foundation.md) |
| Outcomes, skip/replacement and persisted sessions | [Practice outcomes](../learning-practice/practice/practice-outcomes.md) |
| Vocabulary filters and global exclusions | [Vocabulary selection](../dictionary-data/vocabulary/vocabulary-selection.md) |
| Nonblocking eligible vocabulary calculation | [Eligibility counting](../dictionary-data/vocabulary/eligibility-counting.md) |
| Personal Practice profile import and export | [Profile portability](../learning-practice/practice/practice-profile-portability.md) |
| Writing and Hanzi recognition | [Write Hanzi](../learning-practice/hanzi/write-hanzi-activity.md), [AI recognition](../learning-practice/hanzi/ai-hanzi-recognition.md) |
| Courses and grammar acquisition | [Course mosaics](../learning-practice/courses/courses-mosaic.md), [learning evidence](../learning-practice/practice/learning-evidence.md) |
| Game rules | [Sentence Relay](../games/sentence-relay.md), [Cloud Match](../games/cloud-match.md) |
| Third-party resources and inspirations | [Credits](credits.md) |

## State and verification

The browser stores learner-local data separately from the packaged dictionary and optional AI services. An application route, drafted feature or test file alone does not establish successful behavior. Consult [current verification](../project-management/current.md), [open tasks](../project-management/tasks.md), [issues](../issues/OPEN.md) and the [E2E matrix](../testing-quality/e2e/e2e-requirements-matrix.md).

Historical implementation chronicles are preserved in [archived product source](../archive-history/audits/superseded-maintained/product-ux--product-source.md) and are not requirements.
