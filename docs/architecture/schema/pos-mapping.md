# PoS mapping

The 42-code canonical part-of-speech mapping and audit counts. Part of the
[database documentation](DB.md). Approved mapping in
[D18](../../process/decisions/database-and-artifacts.md#d18-source-comparison-and-serialization).

## Canonical part-of-speech mapping (D18: approved)

The shared `PartOfSpeech` mapping and fixture tests are implemented in A02. The
A03 importer logs and rejects entries on mapping failure. Pinyin and hanzi remain
stored exactly as supplied under D16; the derived PoS column intentionally stores
mapped canonical values.

Lowercase every source code **before lookup**. The source codes are
case-insensitive: `Mg` -> `mg` -> `numeric_morpheme`; `Rg` -> `rg` ->
`pronoun_morpheme`. Implement locale-independent lowercasing for these ASCII
codes, preserving source provenance rather than rewriting input data.

| Source code after lowercasing | Canonical enum string |
| --- | --- |
| `a` | `adjective` |
| `ad` | `adjective_adverbial` |
| `ag` | `adjective_morpheme` |
| `an` | `adjective_nominal` |
| `b` | `non_predicate_adjective` |
| `c` | `conjunction` |
| `d` | `adverb` |
| `dg` | `adverb_morpheme` |
| `e` | `interjection` |
| `f` | `directional_locality` |
| `g` | `morpheme` |
| `h` | `prefix` |
| `i` | `idiom` |
| `j` | `abbreviation` |
| `k` | `suffix` |
| `l` | `fixed_expression` |
| `m` | `numeral` |
| `mg` | `numeric_morpheme` |
| `n` | `noun` |
| `ng` | `noun_morpheme` |
| `nr` | `personal_name` |
| `ns` | `place_name` |
| `nt` | `organization_name` |
| `nx` | `nominal_string` |
| `nz` | `other_proper_noun` |
| `o` | `onomatopoeia` |
| `p` | `preposition` |
| `q` | `classifier` |
| `r` | `pronoun` |
| `rg` | `pronoun_morpheme` |
| `s` | `space_word` |
| `t` | `time_word` |
| `tg` | `time_word_morpheme` |
| `u` | `auxiliary` |
| `v` | `verb` |
| `vd` | `verb_adverbial` |
| `vg` | `verb_morpheme` |
| `vn` | `verb_nominal` |
| `w` | `symbol_punctuation` |
| `x` | `unclassified` |
| `y` | `modal_particle` |
| `z` | `descriptive` |

## Rejection policy

If **any** code in a source entry is unrecognized after lowercasing, skip the
**entire word entry and all its forms**, and log a warning identifying the word
and unknown code(s). Do not guess a mapping, silently drop only the bad tag, or
keep the entry with NULL PoS. Apply the skip before reading-key merging and
row-number ID assignment, so counts and IDs reflect the accepted input.

A missing/empty PoS list is different from a present unknown code: retain the
word with `part_of_speech = NULL` under D07. Recognized multi-PoS values use the
existing comma-separated canonical-string column; ensure deterministic output
and no duplicate canonical tokens. Query by complete tokens, never substrings.

## Naming notes

This full table supersedes the earlier illustrative vocabulary:
- `q` maps to `classifier`, not `measure_word`.
- `y` maps to `modal_particle`, not generic `particle`.

The canonical enum, DB values, query filters, and RNG seed tokens must agree.
URL `pos` still follows D13's different input policy: ignore unknown query tokens
while keeping recognized tokens; empty/omitted or all-unknown means no filter.
Source import's whole-word rejection does not apply to malformed URL filters.

## Audit against the pinned source

Audit against pinned `a66fd30b9580da2c2af7eb19e4b9d8099a29c061`, using the
case-insensitive table above:

| Observation | Count / consequence |
| --- | --- |
| Mapping entries | 42 |
| Source entries containing mixed-case codes | 6; case alone does not reject them |
| Unknown lowercase codes | `cc`, `mq`, `qt`, `qv`; do not invent extensions |
| Source entries rejected by the unknown-code rule | 64 out of 11,494 |
| Forms belonging to those rejected entries | 105 out of 12,655 |

Counts describe source entries/forms before other import rules and reading-key
grouping, not final generated DB rows. Some entries have more than one unknown
code. Log/report these exclusions during generation and warn on newly observed
codes rather than silently changing the approved table.

## Related

- Import rules that apply this mapping: [import-rules.md](import-rules.md).
- Schema column storing these tokens: [schema.md](schema.md).
- URL filter policy: [queries.md](queries.md).
- Fixture test requirements: [../../testing/provenance.md](../../testing/provenance.md).