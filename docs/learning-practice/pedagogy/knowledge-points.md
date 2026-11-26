# Knowledge Points (R608 approved scoring model)

**Status:** Product rules approved on 2026-11-22; R612 extends the standalone pure-Rust scoring-core candidate, not connected to live Practice, AI, persistence or Statistics. Cargo compilation/tests are unverified in this environment.

## Canonical catalogue and access

The catalogue is fixed and final, with no catalogue evolution, unlock mechanism or HSK-based vocabulary filtering. All vocabulary is exposed from the beginning. Every unique active canonical entity contributes one point to the global maximum, even if unseen. Course and HSK views are descriptive projections of the same globally unique entities, not separate awards.

| Category | Canonical item | Scored targets | Weights |
| --- | --- | --- | --- |
| Pinyin | Valid syllable-tone combination | Reading, Listening, Speaking | 1/3 each |
| Hanzi | Canonical character, with curated simplified/traditional variants | Recognition, Pronunciation, Handwriting | 1/3 each |
| Vocabulary | Curated word sense, including curated expressions | Recognition, Listening, Speaking, Writing | 1/4 each |
| Grammar | Curated pattern | Whole grammar pattern, no subordinate skill dimensions | 1 |

`max_points = count(Pinyin) + count(Hanzi) + count(Vocabulary) + count(Grammar)`. No Contextual Use scoring dimension exists; context diversity is an evidence requirement. `score(target) = earned_chunks / required_days / number_of_dimensions`, with grammar using one target. Store chunks as integers and round only in presentation.

## Qualification, debt, relapse

- Default base success quota is 10 independently supported, meaningfully distinct context families per scoring target per local scoring day. Default required qualifying days: 7. Reusing a context family on a different day is permitted.
- Each target qualifies at most once per day; days need not be consecutive. A confidently attributable failure before qualification disqualifies that target for that day. Once qualified, its chunk cannot be revoked by a later same-day failure. Other targets are unaffected.
- Events process chronologically. Snapshot each target's effective success quota on its first event of the local day. Subsequent debt changes cannot retroactively alter that quota.
- Debt activates after 2 to 5 previously earned chunks, default 3; threshold must not exceed required days. At most one debt-generating failure per target per day. Debt is capped at 5. Add 2 successes per debt unit to the next day's quota; one successfully qualified day repays one debt unit.
- At full mastery, a confidently attributable failure starts one severe relapse episode, removes exactly one chunk, and sets debt to 5. Further failures cannot remove more chunks during that episode. Each subsequent qualified recovery day repays one unit; only the day reducing debt to zero restores the one missing chunk. No further relapse before recovery completes.

## Hidden Comeback recovery (approved)

After an ordinary pre-qualification disqualification, 100 consecutive independently valid correct observations for the same target on the same local day cancel that day's disqualification and trigger a one-time hidden **Comeback** notification. A further attributable mistake before 100 resets the streak. Invalid, duplicate or context-repeated observations do not advance it. The daily quota, meaningful context requirements and one-chunk/day ceiling remain in force; the 100 successes count toward the existing quota and the day may qualify immediately upon recovery. Previously generated debt remains until the normal qualification repayment, and Comeback does not override an active severe mastery relapse. The milestone must not be advertised as a progress counter or award bonus points.

## Evidence eligibility

One exercise can award multiple manifest-authorized entity/dimension observations, across categories, only when each is independently demonstrated; never automatically cascade credit to contained characters, syllables or senses. One attempt yields at most one observation per target. Distinct contexts must be genuine semantic-equivalence families certified upstream, not arbitrary generated IDs.

- Pinyin is assessed for the specific syllable and tone. Producing a whole vocabulary word does not automatically constitute Pinyin assessment.
- Hanzi Recognition may demonstrate character identity, meaning, function or contribution. Correctly producing a whole word may establish independently attributable Hanzi Pronunciation or Handwriting, but recognizing the word alone does not establish its characters. Polyphonic Hanzi Pronunciation requires all curated readings balanced over the daily success quota, without a predetermined extra-reading assignment. Correct structure recalled without visible guidance qualifies Handwriting; stroke order is feedback only. Pinyin IME candidate selection, tracing and copying do not earn Hanzi Handwriting.
- Vocabulary uses curated sense identities. Semantically equivalent translations qualify; independent contextual inference may qualify. Word banks only support dimensions they genuinely test. Curated expressions and component words earn evidence separately only if justified.
- Grammar must actually be assessed; incidental appearance of a pattern does not earn evidence.
- Ambiguous prompts, uncertain attribution, invalid observations and technical AI failures do not cause score gains, disqualification, debt or relapse. A sentence-wide error never automatically condemns every embedded target.

## Architecture and persistence boundary

The exercise manifest authorizes entity IDs, dimensions, context, modalities and cue visibility. The local or AI evaluator reports diagnostic outcomes, never point deltas. The Rust evidence validator checks authorization, independently delivered modality, attribution, context independence, per-attempt deduplication and policy epoch. The pure scoring engine accepts only validated observations. Observations and derived scoring state must persist durably with attempt IDs. R612 adds a pure manifest-aware evidence validator and an atomic in-memory batch transition ledger, but does **not** implement browser storage, durable cross-tab transactions, real catalogue ID authorization or upstream semantic context certification. The application must persist a complete ledger snapshot transactionally before publishing successful scoring results.

AI evaluation failures after bounded retries are technical failures. AI-only sessions end; mixed sessions skip remaining AI-dependent steps while locally assessed steps continue. Already validated daily evidence survives session interruption. Scoring policy changes require confirmation and a reset of pedagogical scoring observations/state, without deleting unrelated course content or history. No scoring policy migrations or catalogue migrations are required.

## Remaining integration work

Validate actual fixed catalogue IDs and polyphonic reading manifests; implement exercise authorization, semantic context certification, durable storage/transaction boundaries, AI/locally assessed report adapters and statistics projection. The pure Rust core is not an authoritative live learner score until connected and tested.
