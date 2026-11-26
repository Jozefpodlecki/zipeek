# Knowledge scoring architecture (R612 WIP)

The R608 Knowledge Points product rules are consolidated in [Knowledge Points](../../learning-practice/pedagogy/knowledge-points.md). The pure Rust candidate is `zipeek_pedagogy::knowledge`. It is not connected to live Practice, AI, storage or Statistics.

| Module | Responsibility |
| --- | --- |
| `model.rs` | Fixed categories, category-specific dimensions, scoring target state, reading sets and retained observations |
| `policy.rs` | Default seven-day/ten-success policy, bounded debt constants, configurable days/quota/eligibility |
| `engine.rs` | Chronological event transitions, context uniqueness, daily quota lock, balanced reading coverage, debt and relapse |
| `evidence.rs` | Manifest-aware evidence validation, cue/modality authorization and unscorable outcome filtering |
| `ledger.rs` | All-or-nothing in-memory validated-event batches and cross-target attempt replay protection |
| `projection.rs` | Integer-ratio score aggregation, category-specific 1-point entities and mixed-category normalization |
| `tests.rs` | Focused authored tests for core transitions and category scoring |

`AssessmentEvent` is an already-validated domain observation, **not** raw untrusted AI output. The engine cannot determine whether a translation demonstrates a character or whether generated context families are semantically distinct. The pure validator covers authorization and modality, while semantic context-equivalence certification and fixed catalogue membership remain upstream responsibilities.

`KnowledgeState.observations` retains accepted target observations and `seen_attempts` enforces target-scoped replay protection. These structures are in-memory only: a durable store and atomic write of the entire ledger remain to be integrated. One shared attempt may authorize different targets, but a target may only count the attempt once.

Polyphonic Hanzi requires `required_readings` to be supplied from the fixed curated catalogue before scoring Pronunciation; each successful event must identify its required reading. The quota must be covered by an evenly distributed subset (base `floor(quota/readings)` for each, remainder from distinct readings). Extra earlier successes for one reading cannot substitute for missing coverage.

R612 source review does not establish Cargo, WASM, Clippy, E2E or persistence acceptance. Run locked Rust tests and verification before promoting the candidate.

## R612 validated evidence and Comeback core

The pure `knowledge::evidence` module validates manifest-authorized targets, dimension compatibility, modality, independent demonstration, cue leakage, required reading keys and per-attempt target uniqueness before producing `AssessmentEvent` values. A manifest must be built from trusted exercise-generation metadata and the fixed catalogue; the standalone validator does not itself verify catalogue membership or certify semantic context equivalence.

`knowledge::ledger` applies whole validated batches against a cloned ledger and commits only if every target transition succeeds. It tracks completed attempt IDs across scoring targets; the owner must durably persist the entire ledger as one atomic write before making the updated state authoritative. This module itself is in-memory and does not claim IndexedDB transaction or cross-tab coordination.

`Transition::Comeback` occurs only when a disqualified target reaches 100 consecutive distinct eligible successes on the same day; the transition is the UI notification signal and the day may qualify on the same event. The streak is never user-visible. An active severe relapse prevents Comeback.
