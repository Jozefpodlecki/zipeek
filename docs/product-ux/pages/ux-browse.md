# Ux Browse

## Browse

### Search and filters (UX-BROWSE01)

Browse uses the **approved compact-list behavior**: a prominent
search bar and compact filter controls above five-row dictionary lists, with a
separate personal Word Set management panel. On wide screens, results and Word
Set management sit side by side; on narrow screens, the management panel stacks
below or opens from a clearly labelled disclosure. This replaces the previously
required smart-combobox and desktop-table presentation, **not** the A37 URL,
query, token, or pagination semantics documented below.

The implemented behaviour and the approved exercise-first setup are described in
[Dictionary Browse](../../dictionary-data/dictionary-browse.md) and
[Practice setup history](../../archive-history/completed/PRACTICE_SETUP_HISTORY.md). Older URL/chip and numbered-page
requirements in this section are still future work, not implemented features.

The main search input accepts free text. Filter controls can expose dropdowns
or an add-filter affordance; choosing a supported filter creates a removable,
visible chip. Typed text is a text query unless the user explicitly chooses to
create a filter. Recognized tokens may be suggested to accelerate entry, but
an autocomplete combobox is no longer a mandatory interaction. Each chip shows
its canonical filter token and a keyboard-operable remove action. Removing a
chip commits the changed filter set and immediately refreshes results.

Word Set management is visually distinct from search results and exposes saved
set creation, rename, deletion, and backup actions. Selecting a predefined or
saved Word Set for browsing is a filter, not permission to broaden a failed or
invalid saved-set query into unrestricted results. The right-hand panel may
include quick selection and bulk-add actions when their behaviour and
accessibility are implemented and tested.

| Token | Filter |
| --- | --- |
| Any of the 42 canonical PoS tokens, e.g. `noun`, `verb` | Complete-token PoS match |
| `hsk2`, `hsk3` | HSK system membership |
| `hsk2:1` … `hsk2:6` | HSK 2.0 level |
| `hsk3:1` … `hsk3:7`, `hsk3:7-9` | HSK 3.0 level; 7 and 7-9 mean the advanced band |
| `hsk2:1-4`, `hsk3:2-7` | Inclusive level range within the chosen system |
| `freq:100`, `freq:100-500` | Exact frequency rank or inclusive range; no artificial cap |
| `id:42` | One WordId per chip; comma-separated IDs do not parse |
| `simp:吃`, `trad:吃` | Exact simplified/traditional hanzi |
| `pinyin:chi` | Exact pinyin match |

Filter chips always use exact matching independently of the global strategy.
Combine multiple chips with AND. Each chip displays its token and a visible,
keyboard-operable ×; removal reruns the query immediately.

HSK range bounds are clamped to 1-6 or 1-7 for their respective systems.
Invalid single levels such as `hsk2:0` and `hsk3:8` create no chip and show a
small inline valid-range message. Reversed ranges, missing endpoints,
non-integers and negative values create no chip and show an inline error. Do not
swap reversed bounds. Keep numeric HSK range clamping; frequency ranks must be
positive integers, without an artificial product cap.

The settings icon opens a modal with global free-text match strategy radio
buttons: Exact, Prefix, Substring (default). Selection closes the modal and
updates the URL immediately; there is no Save button or per-field override.

Free text returns the union of matches across:

- Pinyin: global strategy against a tone-stripped, whitespace-free, case-folded
  comparison value. Support source display and numbered forms without rewriting
  stored pinyin; this search key is distinct from D18's tone-preserving replay key.
- English: case-insensitive substring, regardless of global strategy.
- Simplified hanzi: prefix, including exact full-sequence matches, regardless
  of global strategy.

Order by pinyin ascending, with simplified-hanzi bytes as the tie-break.
Page sizes are 5, 10, 15, 20, default 5. Show numbered page links with previous/
next arrows and ellipses for gaps, e.g. `← 1 2 [3] 4 5 … 100 →`.

### Browse URL state

All Browse state belongs in the URL. The approved A37 contract is:

| Parameter | Meaning / default |
| --- | --- |
| `q` | Free text; empty when omitted |
| `strategy` | `exact`, `prefix`, `substring`; default `substring` |
| `f` | Ordered comma-separated encoded filter tokens; empty when omitted |
| `page` | One-based page number; default `1` |
| `page_size` | `5`, `10`, `15`, `20`; default `5` |

Canonical filter tokens are `pos:<canonical-pos>`, `hsk2`, `hsk3`,
`hsk2:<level-or-range>`, `hsk3:<level-or-range>`, `freq:<rank-or-range>`,
`id:<id>`, `simp:<hanzi>`, `trad:<hanzi>` and `pinyin:<text>`. A bare PoS token selected through an optional suggestion or filter control
emits `pos:<token>` on the wire. HSK 3.0 advanced `7-9` single-tier
alias canonicalizes to `7`; ordinary inclusive ranges use their normalized bounds.

Encode each complete filter token individually, join tokens with literal commas
in insertion order, then serialize that entire `f` value once in the outer URL
query. Decode the outer query once, split its `f` value at literal commas, then
decode each token once. This intentional two-layer codec keeps commas/percent
signs inside token values distinct from filter separators; it is specific to
Browse and changes no practice URL encoding rule. Example semantic filters:
`pos:verb`, `hsk3:2`, `freq:100-500`.

Query, chip/filter and page-size changes reset page to 1. A requested page beyond
the available result pages uses the last available page; zero results use the
initial page 1 with the approved empty view. Malformed loaded values use safe
defaults/validated filters and never trigger an implicit URL rewrite. Explicit
new state generation uses the canonical grammar. Full codec tests must cover
Unicode, commas, percent signs, malformed escapes and reload roundtrips.
Debounce live search by 300 ms. Enter forces an immediate query and commits the
URL. Typing alone does not update it. Chip changes, strategy changes and page
navigation query and update immediately. All Browse state updates use
`replaceState`, never `pushState`; Back returns to the page before Browse.

Tab focuses the input, filter controls and chips in document order. Enter in
the search input commits free text immediately. If optional suggestions are
shown, Up/Down moves among them and Escape closes them. Chip removal supports
Enter/Space and has an accessible name.

### Browse fields (UX-BROWSE01, resolved A37 and amended by D45)

Use the same five fields on vocabulary cards, in this order:
simplified hanzi, pinyin, English, HSK and PoS. English uses the first stored
gloss. HSK shows every available system and its level/band; do not collapse
HSK 2.0 and 3.0 into one inferred level. Missing metadata is not invented.
Do not add traditional-hanzi or frequency columns. Their exact filter tokens
remain supported independently of the displayed field set.

### Browse states (UX-BROWSE02)

With no query/chips, show all words on page 1 and "Type to search or add a filter".
Zero results shows "No words matched" and Clear search, which clears query,
chips and page number and returns to the initial view.

Show no spinner for normal queries; after a query takes more than 200 ms,
show a subtle results-area spinner. This is a performance guard, not a measured
latency promise. Query failure shows "Couldn't load results" and Retry inline;
retry reruns the last query without changing the URL.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/product-ux-pages-ux-browse.md). Dated implementation claims in that snapshot are not current build or browser verification.
