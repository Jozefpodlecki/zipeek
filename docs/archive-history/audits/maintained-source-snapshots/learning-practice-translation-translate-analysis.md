# Translate & Analyse - R136

## Approved product boundary

The standalone `#/translate-sentence` page uses the endpoint, model and optional browser-memory key configured **only** under Settings → AI Translation. There is no model selector or credential entry on the page. The user chooses Chinese→English or English→Simplified Chinese, types a sentence, then requests **Translate & Analyse**. A two-column editor/result layout is followed by four sections: estimated difficulty, language structures, vocabulary observations and learning notes. Clear, Swap and Copy are available. The global route title remains in the centered app header.

## Typed response contract

`zipeek-ai/src/sentence_analysis.rs` owns the typed response, the prompt, size limits and structural validation. The model must return JSON with `translation`, `difficulty`, `estimated_hsk`, `rationale`, `vocabulary`, `grammar` and `notes`. Allowed difficulty values: beginner, elementary, intermediate and advanced. HSK estimate may be null or 1-9. Vocabulary Hanzi and grammar example excerpts must occur literally in the original Chinese sentence (Chinese→English) or the translated Chinese sentence (English→Chinese). Unrelated lexical or grammatical evidence is rejected. Arrays and text fields have size limits; unknown fields are rejected. Prompts instruct the model to treat the source sentence as data rather than as instructions.

**Classification is AI-estimated, not dictionary verified**. R136 does **not** assert official HSK levels or word-frequency values for individual tokens. Dictionary-backed lexical lookups, validated tokenization, grammar taxonomy, calibrated difficulty, alternate translations and persisted history are future work. Avoid presenting AI output as a certified educational assessment.

## Requests and failure modes

`crates/zipeek-web/src/services/ai/mod.rs` uses the existing OpenAI-compatible transport. A first call asks for translation plus structured analysis. If it returns text but cannot be validated as structured data, the client makes **a second request** for translation-only. The page shows that translation and a brief notice that language analysis was unavailable. Both requests may use model tokens or incur provider charges. If the transport itself fails, the existing HTTP/timeout/CORS errors are shown. Changing the source sentence, direction, swapping or clearing increments a generation ID to ignore outdated responses.

No local translation history, database migration, model discovery or new server is introduced. Existing AI key handling is unchanged: it is held in browser-page memory only; endpoint/model preferences are stored in localStorage.

## Acceptance and limitations

- Focused Node checks verify page contracts and existing AI settings/transport behaviour.
- Pure Rust unit tests cover typical JSON, unrelated Chinese vocabulary, unsupported grammar evidence, out-of-range HSK, English→Chinese and unexpected fields.
- Windows `cargo fmt --all --check`, native Rust unit tests, WASM `cargo check` and actual browser/provider verification remain **pending**, because Cargo is unavailable in the packaging environment.
- Verify status, error, Copy, Clear, Swap, no model selection and keyboard navigation on desktop/mobile. Try a JSON-capable LM Studio model and a model that returns malformed output to verify fallback and usage.
- Results should be considered model suggestions. A later milestone must join segmented vocabulary to the local dictionary for authoritative HSK/frequency annotations.

## R137 - Workspace interactions and accessibility

The translation editor and result use stacked cards on small and medium displays, with side-by-side cards from the large breakpoint. Cards and translation output have `min-width: 0`/word wrapping to prevent long provider text from overflowing the viewport. The source field exposes a keyboard hint: **Ctrl+Enter**, or **Command+Enter** on macOS, submits using the same guarded request callback as the primary button; normal Enter remains available for line breaks, and IME composition does not trigger translation.

Copy now reports clipboard success, unavailable clipboard or denied permission in a polite status region. The feedback is guarded by the same generation token as result requests, so a delayed clipboard response cannot claim success for a subsequently edited result. Editing the source, changing direction, swapping and clearing invalidate in-flight translation results and clear obsolete errors and clipboard feedback. Output declares busy state, the request button has a loading indicator, and provider errors remain role=alert.

The compact AI indicator says whether a model is selected; **it does not claim that the endpoint is reachable**. Connection tests and all model/credential settings remain exclusively in Settings. Translation and analysis are still sent directly to the configured provider and may incur charges, including an extra fallback call. Automated Node tests inspect these contracts. **Real keyboard, clipboard, responsive layout, screen-reader, Cargo/WASM and provider tests are still required on Windows.**

## Interpretation of sentence evidence

`crates/zipeek-ai/src/sentence_insights.rs` owns the pure `GrammarFamily`, `GrammarEvidence` and `LexicalCoverage` interpretation types. Grammar families are heuristic labels for AI-proposed example fragments anchored in the observed Chinese sentence, not independent syntactic verification. Dictionary evidence summarizes which model-suggested words had exact or ambiguous matches and HSK membership. It is **not** complete sentence segmentation, certification of HSK difficulty, or proof that unmatched words are absent. If enrichment fails, do not imply all vocabulary was verified.
