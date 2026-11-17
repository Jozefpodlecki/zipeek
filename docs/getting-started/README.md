# Getting started with Zipeek

Zipeek is a Rust/Yew WASM application for Chinese learning. The application uses a pregenerated DuckDB dictionary and has separate build, test, and E2E workflows.

## Before the first run

Read the [repository README](../../README.md) for current prerequisites and platform instructions. Consult [AGENTS](../../AGENTS.md) for code policy, approved-decision precedence and validation expectations. Do not run dependency upgrades without explicit approval.

## Find your way around

- [Product and UX](../product-ux/product.md) introduces the application.
- [Practice overview](../learning-practice/practice/practice-overview.md) explains the learner workflow and authoritative outcomes.
- [Architecture](../architecture/architecture.md) identifies the main runtime components.
- [Dictionary and regeneration](../dictionary-data/schema/db.md) describes data preparation and validation.
- [Build and deployment](../build-deployment/ci.md) documents build gates and CI tasks.
- [Testing](../testing-quality/acceptance/testing.md) explains the validation levels.
- [Project tasks](../project-management/tasks.md) distinguishes unfinished work from verified implementation.

## First verification

From the project root, use the commands appropriate to the change. For documentation edits, regenerate the Docs index with `cargo make docs`, then run `cargo make docs-validate`. For Rust and browser work, follow the [testing specification](../testing-quality/acceptance/testing.md).

A successful documentation validation is not evidence that Rust, WASM or Chrome builds pass.
