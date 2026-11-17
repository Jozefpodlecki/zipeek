# Zipeek agent guide

This is the **only root-level agent instruction file**. Do not introduce a second agent prompt or competing policy file. Read relevant linked documents when the task touches their area; do not treat historical decisions or release notes as current implementation evidence.

## 1. Project and precedence

Zipeek is a Chinese Hanzi dictionary and practice application using Rust 2024, Yew CSR, WASM, Tailwind and a pregenerated read-only DuckDB dictionary. Its hosted base path is **`/zipeek/`**. Browser-local personal learning data is distinct from packaged dictionary data; do not move it to a server or disclose it to optional AI/image services without an approved design.

- **Authority:** Explicit current user decisions and approved project policies govern intended behavior. Source code, manifests and tests establish *what is implemented*, not what is approved or verified. If sources disagree, explain the discrepancy and request approval before changing controlled requirements.
- **Approved decisions:** Read affected entries under [decisions](docs/project-management/decisions/index.md), but do not treat historical compatibility commitments as active requirements.
- **Current task status:** [CURRENT](docs/project-management/current.md), [TASKS](docs/project-management/tasks.md), and dated [COMPLETED](docs/archive-history/completed/) are the task-state authorities. Unresolved architecture proposals belong in TASKS; recorded decisions live under [decisions](docs/project-management/decisions/index.md).
- **Documentation index:** [docs/README.md](docs/README.md) identifies the maintained owner of each specification. The root README is for first-clone setup, not checkpoint history.

## 2. Before work and approval

1. Read this guide, CURRENT, TASKS, and the architecture/product or specialist documents relevant to the request. For read-only reviews, do not mutate task or decision records.
2. Inspect relevant current source, manifests, tests, and changed files before claiming behavior or proposing technical changes. Record uncertainties; do not extrapolate completion from docs or prior snapshots.
3. Follow the user's specified task. Select the next ready TASKS entry only if asked to choose work.
4. Propose the task, affected paths, behavior and data impacts, validation, dependencies, and decisions requiring approval. **Wait for approval before editing.** Approval covers the agreed scope, not unrelated refactoring or additional dependencies.
5. During approved implementation, preserve user modifications, including staged changes. Ask again only for a material change of scope, destructive consequence not covered by the plan, dependency change, or unresolved decision. Routine steps need no second approval.
6. Apply approved decisions to the owning documents and record their outcomes in the relevant decision and task history. Keep unresolved questions in TASKS until decided; do not delete unresolved work.

## 3. Development-phase policy: zero backward compatibility

**Zipeek is in active development. No backward compatibility is required, with no exceptions.** This includes old application interfaces, database schemas, stored browser data, generated files, URLs, seed formats, serialized data and replay encodings (including earlier D12/D19 formats). Do **not** introduce migrations, dual readers/writers, legacy parsers, old-format fallbacks, compatibility shims, or version-selection mechanisms solely to support older representations.

Choose one clean current representation within *approved scope*. Changes that may invalidate links or discard locally stored data must be disclosed and approved in advance; recovery or regeneration steps must be documented. Zero backward compatibility does **not** waive deterministic current behavior, data integrity, validation of untrusted inputs, source licensing, security, dependency pins or testing.

## 4. Protected project contracts

- **Dependency change control:** Do not add, upgrade, move or replace dependencies without explicit approval. Use the root `Cargo.lock` and `--locked`; do not create a crate-local lockfile. Report conflicts with approved decisions instead of silently changing dependencies.
- **Toolchain and dependency versions:** Inspect `rust-toolchain.toml`, the relevant `Cargo.toml` manifests, the root `Cargo.lock`, and frontend package manifests and lockfiles for actual configured versions. Do not duplicate version numbers in this guide. Consult [decisions](docs/project-management/decisions/index.md) for approved constraints and rationale; if these disagree with the manifests, report the discrepancy rather than assuming either is current approval to change versions.
- **Browser database:** Do not hand-edit `*.duckdb` or generated vendor binaries. Generate, verify and install through approved commands, never as an incidental Trunk rebuild. Preserve third-party licenses and notices. See [DB](docs/dictionary-data/schema/db.md) and [web assets](docs/architecture/modules/web-assets.md).
- **Deployment:** Keep GitHub Pages builds, assets and navigation under `/zipeek/`; root `/` is used deliberately by development E2E tests. Check both when routing, assets or startup change.
- **Determinism:** Current round generation uses a single seeded `ChaCha12Rng` stream and approved 64-bit FNV-1a reproducible hashes. Maintain deterministic current-format tests; any change to draw order, seed semantics or fallback is a deliberate approved behavior change, **not** a reason to support old formats. See [round generation](docs/games/round-generation.md).
- **Trust boundaries:** Validate URL parameters, imports, browser storage and external responses before converting to domain values. Do not put credentials or user-private data in web assets, fixtures or release archives.

## 5. Engineering rules

Use [CODING_STANDARDS](docs/engineering/coding-standards.md) as the detailed implementation contract. For relevant work, apply the following principles:

- Prefer readable idiomatic Rust/Yew, coherent domain boundaries, testable business rules and minimal justified abstractions. Do not split by arbitrary file length or add crates to match a speculative architecture.
- Keep coupled workflow state and transitions coherent; assess `use_reducer` where invariants span fields, and Yew contexts only where genuinely shared. Distinguish required providers from intentionally optional contexts after verifying the root provider tree.
- Define named callbacks outside `html!` element attributes; favor typed identifiers/actions over round-tripping values through DOM `data-*`. Keep Yew markup reviewable and use `yew_icons` for application chrome rather than emoji.
- Maintain the existing Tailwind design system, responsive presentation, accessible semantic controls, focus and keyboard behavior, and explicit loading/empty/error/retry/completion states. Do not perform unrelated broad redesigns.
- Keep async ownership, cancellation, listener cleanup and stale-response protection explicit. Do not mistake deferral of synchronous CPU work for nonblocking execution.
- Follow the no-code-comments policy without exceptions. Clarify implementation through naming, types, and module structure; keep rationale in documentation. Avoid em dashes and long dash typography in code, UI text, documentation and replies.
- Test modifications at the correct level and preserve meaningful E2E assertions. See [testing](docs/testing-quality/acceptance/testing.md) and [E2E architecture](docs/testing-quality/e2e/e2e-architecture.md).

## 6. Verification and honest evidence

- Assess available tools before planning checks. Static review, authored tests, commands actually run, supplied user logs and browser observation are different evidence types. Never invent results, source inspection, test passes, screenshots, benchmarks or timestamps.
- Choose focused checks proportional to risk. For routine changes run affected checks and representative behavior; for major work or release readiness use the comprehensive gate in [TESTING](docs/testing-quality/acceptance/testing.md). A native Rust check is not a WASM build; neither is a browser acceptance test.
- Browser changes require appropriate Chrome/user-flow review when available; routes/Pages changes require `/` and `/zipeek/` coverage. Missing tools mean **not run/blocked**, not passed. Provide exact local commands for remaining checks.
- Record what was modified, checks that ran and results, checks not run, known pre-existing failures and next actions. Do not call unverified work accepted or production-ready.

## 7. Documentation and task-state updates

For tracked, approved implementation, keep `CURRENT.md` restricted to the active task, immediate state, blockers, verification and next steps; never accumulate prior release notes or completed checkpoint chronicles there. Preserve historical work in release history, release notes and dated completed records. Update TASKS, append completed outcomes to the correct dated COMPLETED file before removing completed TASKS entries, and update the relevant owning product, architecture, testing or decision documents. Completed date files are append-only; historical archive records are frozen. Do not recreate `STATUS.md`.

For read-only audit and proposals, do not mutate task or decision state. For documentation-only edits, validate Markdown, links and factual consistency; Rust/e2e execution is unnecessary unless code or generated outputs changed. Keep release history in [RELEASE_HISTORY](docs/archive-history/RELEASE_HISTORY.md), not the root README. Maintain one root `RELEASE_NOTES.md` for handoff notes; do not create additional `R<number>-RELEASE-NOTES.md` snapshots. Historical release summaries belong in `docs/archive-history/RELEASE_HISTORY.md`.

## 8. Release and source handoffs

- Numbered source releases and WIP handoffs must follow the current [release naming and acceptance contract](docs/build-deployment/release-naming.md). The native `cargo make pack` snapshot format is separate.
- The approved **20 substantive Rust-file minimum applies only to explicitly requested numbered major-release handoffs**. Never pad or expand routine work to satisfy it. Before expanding an approved major release, propose additional justified work and receive approval. Validate against the actual final diff.
- Preserve the source archive root layout, verify the actual unpack contract, and distinguish completed acceptance from an unverified source candidate. Do not infer a release number or timestamp.
- **ChatGPT session archive hygiene:** When working in a ChatGPT session with filesystem access, inspect ZIPs created in that session and delete superseded **agent-generated project/release ZIPs** after the replacement handoff has been built and integrity-checked. Retain the current delivery and any required comparison inputs until validation finishes. Never delete user-uploaded archives, external Downloads, repository backups, evidence bundles, or unknown files; if ownership or replacement is uncertain, leave the ZIP intact. When operating via Codex, follow the same non-destructive ownership boundary, but do not assume ChatGPT session files are accessible or perform cleanup outside the working scope.

### Delivery format for requested changes

- **Deliver complete, ready-to-use files, not patches.** When modifying the repository, the default deliverable is a complete updated project source ZIP with the approved changes already applied and the expected archive root layout preserved.
- Do not provide patch-only ZIPs, unified diffs, isolated replacement snippets, or instructions requiring the user to manually copy edits into existing files as the primary deliverable. Provide these only if the user explicitly requests them.
- For a request limited to one standalone document, provide the complete updated document. A full project archive may also be provided when the request concerns repository integration or the user asks for it.
- **Release handoff is not a single artifact:** When a numbered release or WIP handoff is requested, provide the installable full-source ZIP **and** the complete separately accessible changed primary file(s), plus the canonical root `RELEASE_NOTES.md` describing the scope, archive identity and verification status. A standalone document by itself is not a release handoff. Do not mislabel documentation-only WIP work as an accepted major release.
- Keep untouched project content intact, verify ZIP integrity and relevant paths, and state what changed and what was actually validated. Do not claim a build passed merely because a modified archive was packaged successfully.

## 9. Task-specific references

| Task type | Read before acting |
| --- | --- |
| Product and UX | [PRODUCT](docs/product-ux/product.md), [UX](docs/product-ux/ux.md) and relevant feature spec |
| Crate boundaries, data flow, providers | [ARCHITECTURE](docs/architecture/architecture.md), [UI architecture](docs/architecture/ui-architecture.md) |
| Rust, Yew, Tailwind, hooks, E2E code style | [CODING_STANDARDS](docs/engineering/coding-standards.md) |
| DuckDB, regeneration, personal storage | [DB](docs/dictionary-data/schema/db.md), [personal data](docs/dictionary-data/personal/personal-data-storage.md) |
| Browser/e2e/CI validation | [TESTING](docs/testing-quality/acceptance/testing.md), [E2E](docs/testing-quality/e2e/e2e-architecture.md), [CI](docs/build-deployment/ci.md) |
| Release and source archive | [RELEASE_NAMING](docs/build-deployment/release-naming.md), [release history](docs/archive-history/RELEASE_HISTORY.md) |
| Current tasks, decisions, evidence | [CURRENT](docs/project-management/current.md), [TASKS](docs/project-management/tasks.md), [decisions](docs/project-management/decisions/index.md) |
