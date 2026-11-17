# Credits and acknowledgements

## Purpose

This page separates **source resources and their licenses** from **product inspirations**. A design inspiration is not a dependency, license grant, affiliation or endorsement. Third-party licensing must be verified against the actual shipped assets and exact source versions before distribution.

## Resources with repository evidence

| Resource | Evidence in this repository | Treatment |
| --- | --- | --- |
| Hanzi Writer Data | Bundled `external/hanzi-writer-data/data/` character geometry and `external/hanzi-writer-data/ARPHICPL.TXT` | Attribute the stroke-data ecosystem and review each upstream data-source licensing obligation. Do not equate the Arphic license file with a blanket license for all geometry or the Hanzi Writer JavaScript library. |
| Chinese dictionary and HSK sources | [Dictionary sources](../dictionary-data/schema/sources.md), [import rules](../dictionary-data/schema/import-rules.md), [bundled source notices](../build-deployment/handoff.md) | Preserve the applicable CC-CEDICT, HSK and derived-data notices; confirm provenance of the installed generated database. |
| HSK source dataset | `crates/zipeek-web/static/notices/HSK-LICENSE.txt` and `HSK-SOURCE-README.txt` | Retain their full terms and source attribution as provided. |
| Shared artwork and UI assets | `crates/zipeek-web/static/ATTRIBUTION.md`, `static/courses/` and `static/practice/` | Verify provenance, generation rights, adaptation and any required attribution before publishing or claiming exclusive ownership. |
| Fonts, icons, JS/WASM and other Rust dependencies | Manifest and lockfile dependencies, bundled notices and generated frontend assets | Generate or review a complete distribution-specific notice inventory; do not assert that every package uses the same license. |

External sources relevant to verification include [Hanzi Writer Data](https://github.com/chanind/hanzi-writer-data), [Hanzi Writer](https://github.com/chanind/hanzi-writer) and [CC-CEDICT](https://www.mdbg.net/chinese/dictionary?page=cedict). These links identify resources, not confirmation of all obligations in a particular distribution.

## Learning and design inspirations

- [Duolingo](https://www.duolingo.com/) is an example of a language-learning product whose structured exercises and progress feedback can inform design exploration.
- [Ninchanese](https://ninchanese.com/) is an example of a Chinese-learning product whose vocabulary and gamification approaches can inform exploration.

These are acknowledgements of inspiration only, not claims that Zipeek uses their code, artwork, trademarks or lesson content. Zipeek is not presented as affiliated with, authorized by or endorsed by these services.

## Maintenance and release requirements

Before a public distribution, compare actual bundled datasets and artwork with the attribution manifest. Include required full license texts, source details and any attribution of modified material in the appropriate shipped notices. Keep the in-app footer's Credits link discoverable and keep mandatory license access independent of an optional web connection. Track unresolved provenance questions as engineering tasks rather than inventing a license.
