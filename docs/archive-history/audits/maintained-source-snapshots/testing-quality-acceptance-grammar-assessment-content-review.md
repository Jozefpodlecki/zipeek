# Grammar assessment draft - content review inventory (R363)

This inventory records all **24 draft items** in the five authored grammar courses. It is a **review checklist**, not evidence that a qualified Mandarin reviewer has approved the content. The question prompts are presently English; the three choices are Chinese. Each item has one declared answer key and a short rationale. The automated validator checks structural uniqueness after whitespace/punctuation normalization, not grammatical or semantic equivalence.

| Course | Point ID | Assessment item | Intended distinction | Human review |
| --- | --- | --- | --- | --- |
| 1 | `g01-subject-predicate` | `a01-order` | Subject-verb-object order | Pending |
| 1 | `g01-shi-identity` | `a01-identity` | 是 for identity vs possession | Pending |
| 1 | `g01-bu-negation` | `a01-negation` | 不 before a habitual action | Pending |
| 1 | `g01-ma-question` | `a01-question` | Sentence-final 吗 | Pending |
| 2 | `g02-time-placement` | `a02-time` | Time expression before the verb | Pending |
| 2 | `g02-zai-location` | `a02-location` | 在 + place before the action | Pending |
| 2 | `g02-time-location` | `a02-both` | Time + place + action | Pending |
| 2 | `g02-qu-destination` | `a02-go` | 去 vs 在 | Pending |
| 2 | `g02-nali-question` | `a02-where` | 在哪里 + action | Pending |
| 3 | `g03-you-possession` | `a03-have` | 有 + classifier + noun | Pending |
| 3 | `g03-meiyou-negation` | `a03-not-have` | 没有 vs 不有 | Pending |
| 3 | `g03-classifiers` | `a03-classifier` | 本 for bound books | Pending |
| 3 | `g03-ji-count` | `a03-ji` | 几 + 只 + 猫 | Pending |
| 3 | `g03-duoshao-quantity` | `a03-duoshao` | 多少人, open-ended count | Pending |

| 4 | `g04-adjective-predicate` | `a04-adjective` | Adjective predicates without 是 | Pending |
| 4 | `g04-hen-degree` | `a04-hen` | 很 before the adjective; contextual degree | Pending |
| 4 | `g04-bu-adjective` | `a04-bu` | 不 for negating an adjective | Pending |
| 4 | `g04-tai-le` | `a04-tai` | 太…了 expressing excess/strong degree | Pending |
| 4 | `g04-bi-comparison` | `a04-bi` | A 比 B adjective ordering and semantic direction | Pending |

| 5 | `g05-xiang-intent` | `a05-xiang` | 想 + intended action, not inverted word order | Pending |
| 5 | `g05-yao-want` | `a05-yao` | 要 before wanted noun phrase; request politeness context | Pending |
| 5 | `g05-hui-learned-ability` | `a05-hui` | 会 for acquired language skill | Pending |
| 5 | `g05-neng-possibility` | `a05-neng` | 不能 before action, after time phrase | Pending |
| 5 | `g05-keyi-permission` | `a05-keyi` | 可以 + action, 吗 at sentence end | Pending |

## Acceptance checklist before any grammar mastery authority

- [ ] A qualified Mandarin reviewer verifies the answer key, rationale, naturalness, and **absence of plausible alternate correct answers** for every item.
- [ ] Independently review whether questions isolate the intended grammar skill rather than testing unrelated vocabulary or classifier knowledge.
- [ ] Record reviewer, revision, date, and a stable content version; re-review any changed question or distractor.
- [ ] Add additional assessment formats where multiple-choice recognition alone is inadequate.
- [ ] Define retry, threshold, item version and durable evidence policy, under the repository approval rules.
- [ ] Complete native Rust, WASM and managed-browser tests on the exact released source.

## R363 review improvements

Assessment submissions now provide a read-only per-point diagnostic (point ID and correctness) after submission. The report comes from the locked, full authored question snapshot; changing question content/order invalidates it. Switching assessment courses resets the in-memory attempt. These improvements **do not** publish competency progress, unlock art, write OPFS, or assert qualified linguistic review.
