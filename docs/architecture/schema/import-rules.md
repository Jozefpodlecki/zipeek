# Import rules

How source rows become `words` rows: reading identity, level mapping, text
preservation, gloss rules, and key conflicts. Part of the
[database documentation](DB.md). Approved rules in
[D16](../../process/decisions/database-and-artifacts.md#d16-primary-import-and-ids)
and [D18](../../process/decisions/database-and-artifacts.md#d18-source-comparison-and-serialization).

## Import identity, forms, and deterministic IDs (D16)

The logical import key is **`(simplified, numeric_pinyin)`**, where staging
`numeric_pinyin` maps to the persisted `pinyin_numeric` column. Expand distinct
readings from `forms[]`; do not add a separate word merely for a traditional
variant sharing the same reading key. Retain the first form's traditional value
in source-array order and log a warning when multiple forms share that key.

D07 still enforces `UNIQUE (simplified, pinyin)` on display pinyin. These are
different representations, not universally interchangeable keys. The audited
snapshot has no numeric-key group with conflicting display pinyin and no
display-key group with distinct numeric pinyin (case-sensitive checks). Assert
that relationship when generating; do not silently normalize or collapse readings
if a future input violates it.

After reading-key grouping, assign:

```sql
ROW_NUMBER() OVER (ORDER BY simplified, numeric_pinyin)
```

Here numeric_pinyin is the staging name; ordering the final column would use
pinyin_numeric. Persist the assigned value as the INTEGER id, checking its
range. IDs are snapshot-local. Changing data between runs may shift IDs; saved
exclusions and shared seed=<word_id> URLs can break or identify another word.
Committing the DB with its consumer does not make old URLs portable across DB
regenerations. No cross-snapshot ID guarantee or migration is required.

Level mapping (D16)
Source token	Stored membership
old-1 through old-6	Corresponding hsk2_level, else NULL
new-1 through new-6	Corresponding hsk3_level, else NULL
new-7 or new-7+	hsk3_level = 7, the combined 7-9 band
The pinned JSON actually uses new-7+, while its README illustrates new-7.
Accept both exact spellings as known tag-format variance; do not accept arbitrary
suffixes, separate levels 8/9, or other unrecognized tokens. Log and skip unknown
level tokens. Deduplicate repeated identical tags. If different levels occur
within one system, choose the lowest and warn; retain the independent value in
the other system. No conflicting distinct levels were found in the audited input.

Exact text preservation and field coverage (D16)
Target	Source / import rule
simplified	Entry's simplified text, unchanged
traditional	Selected form's traditional text, unchanged
pinyin	Selected form's transcriptions.pinyin, unchanged; do not derive from numeric
pinyin_numeric	Selected form's transcriptions.numeric, unchanged
english	Nonempty trimmed meanings in source order, pipe-separated; preserve display casing
part_of_speech	Map source pos codes to canonical lowercase enum tokens; nullable if unavailable
frequency	Source integer ranking when provided; lower means more common; otherwise NULL
Preserve spacing, case, and hanzi codepoints exactly, without extra Unicode
normalization of stored hanzi/pinyin. Missing or blank pinyin makes a word unusable:
drop it and log it rather than synthesizing a reading. The audited input has none,
but the generator must handle bad future input.

Verified format differences: the pinned data is not uniformly CC-CEDICT's
u:/neutral-5 convention. It includes ban4 lü3, unnumbered neutral syllables
such as zhen1 de, spaced pinyin, and display text such as bàn lu:3.
Preserve those originals per D16. Lookup and round-comparison normalization is
separate from storage; do not claim raw numeric strings are interchangeable with
CC-CEDICT without the comparison-only D18 policy below.

Gloss selection and CC-CEDICT fallback (D18)
For each reading, trim meanings and remove blank strings. Use the first form's
nonempty meanings when available; inspect remaining same-reading forms if needed.
Keep source order and original case. Replace literal ASCII | inside each gloss
with fullwidth ｜ (U+FF5C) before joining glosses with ASCII |. Log both the
original and replaced string for each occurrence. This is deliberately lossy;
never substitute /, which already carries meaning in glosses such as he/she.
Do not reorder glosses merely to choose a different match label.

Consult CC-CEDICT if and only if the resulting primary gloss list is empty.
Match simplified hanzi and disambiguate by numeric reading as needed. Ignore
fallback entries whose gloss begins with variant of. If no usable fallback
exists, store english = NULL and warn; do not drop an otherwise usable reading.

The first serialized gloss is the English label in both English-bearing variants:
the right column in hanzi-eng, the left column in eng-pinyin. The right
column of eng-pinyin shows the source's diacritic/display pinyin. A row with
NULL English remains eligible for pinyin-hanzi, but is ineligible for either
English-bearing variant. Filter missing required labels before candidate indexing.

The exact trigger is an empty list, not a heuristic quality score or a preference
for another dictionary. Use D18's comparison-only reading keys for fallback lookup.

Numeric/display key conflicts and seed eligibility (D18)
Different exact numeric reading keys that produce the same (simplified, pinyin)
display key violate the approved UNIQUE constraint. Fail generation and log both
source rows verbatim; never silently select one. Conversely, forms with the same
(simplified, numeric_pinyin) and differing display renderings are one reading:
keep the first source form and warn, per D16. Do not fail for display drift alone.

A nonexcluded seed lacking English is ineligible for eng-pinyin and hanzi-eng.
Use D15's deterministic URL recovery: up to ten ChaCha draws from the ordered,
exclusion-clean pool, then the first eligible word by id. pinyin-hanzi does not
require English. The same eligibility condition applies to D20's English-bearing
MCQ variants; D20 separately approves their routes and scope.

What the importer rejects
Any entry with an unknown PoS code after lowercasing (whole entry, all forms;
warning logged, skip before grouping).

Any reading with missing required pinyin (dropped, warning logged).

Any reading with no HSK membership at all (rejected entry).

Any display-key collision between distinct numeric keys (fatal; fails
generation and logs both source rows).

What the importer preserves
First form's traditional text; all distinct readings stay separate.

Exact source spacing, case, and hanzi codepoints.

Both display and numeric pinyin, unchanged.

Source gloss order and casing after trim, with only the | → U+FF5C substitution.

Lowest within-system level with a warning if repeated tags disagree.

Related
Schema the importer writes into: schema.md.

PoS token mapping: pos-mapping.md.

Source bytes this reads: sources.md.

Implemented importer behavior: artifacts.md.
