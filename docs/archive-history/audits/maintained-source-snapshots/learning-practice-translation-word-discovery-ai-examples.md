# Word Discovery: AI examples and app notifications (R138)

The Random Word and Word of the Day cards share `WordSpotlight`. Copying Hanzi uses browser Clipboard, with visible success and error toasts provided by `ToastProvider` at app root. The toast automatically dismisses after 3.8 seconds, can be dismissed manually, and exposes accessible `status`/`alert` roles. This is an ephemeral notification; it is not persisted or exported.

**Example sentence** calls the configured OpenAI-compatible model only after the user presses the button. The request includes the displayed Hanzi, pinyin and dictionary display meaning, asks for one Chinese sentence and one English translation, and checks that the output contains the exact chosen Hanzi. It enforces a 1,200-character response cap. Successful output is explicitly **AI-generated and unverified**-not CC-CEDICT data or an authoritative pedagogical example. A missing model, endpoint failure, malformed response or absent target shows an error. Toggling the panel does not reissue the same request; choosing another random word clears the old example and invalidates an in-flight result. This functionality may incur provider fees.

The new Settings AI connection diagnostics show an inspectable bounded transcript of the test, including the exact fixed system/user prompts and returned text, without storing or rendering authentication credentials. A successful endpoint test is not a quality guarantee. The diagnostics are not exported with learning data.

The standalone About page retains licence and attribution information; Settings no longer duplicates that page. A legacy Themes deep link redirects to Settings. Debug is only rendered when `cfg!(debug_assertions)` is true and contains only non-secret runtime metadata. Release-build users see NotFound for the Debug route.

The next milestone still needs real Rust/Yew compilation and browser/LM Studio acceptance. The `scripts/` directory and root images are R200-deferred.

## Ordered meanings and metadata

Word Discovery and the daily spotlight now display one ordered dictionary sense at a time. The left and right controls are bounded and the selection resets on word changes. Topics and explicit qualifiers belong to each sense, not the entire word; the HSK and part-of-speech metadata remain at word level. AI examples use the currently selected sense and invalidate stale examples when that sense changes.

Generated databases must be rebuilt with the updated `sense_qualifiers` table. The importer preserves the source gloss in `word_senses`, removes only recognized leading annotations from `sense_display`, and stores explicit region, usage, and register labels with source provenance. The first supported tags are Taiwan, Loanword, Figurative, and Colloquial. Unknown annotations remain verbatim; the application does not infer regional or usage tags from arbitrary text. No database migration is supplied.

## Ordered meanings and source labels (R329)

Word Discovery and expanded Browse results use ordered `sense_display` records rather than reconstructing meanings from a combined English string. `sense_categories` provides the categories for an individual sense and `sense_qualifiers` stores strictly parsed leading source annotations. Only recognized markers are converted into structured labels; unrecognized tags remain visible as source gloss text. Example generation uses the currently selected sense and its response is invalidated when the sense changes. The generator requires a full database rebuild; there is no migration.
