# Application visual and interaction review

## Scope and visual baseline

The approved visual direction preserves Zipeek's dark violet panels, Chinese landscape imagery, purple selection highlights, compact navigation, rounded cards and restrained iconography. Review and improve hierarchy, legibility, accessible controls and responsiveness without replacing the visual identity. The observations below come from user-provided browser screenshots; source behavior and acceptance remain unverified.

## Global header and dragon

- Replace the redundant centered page title in the global header with a [long, detailed Chinese dragon](../foundation/header-and-dragon.md) that itself serves as the evidence-based progress shape. Do not show level or percentage labels above the dragon. The artwork must faithfully follow the approved design reference rather than a simplified generic progress bar.
- Open acquisition details only on interaction. On narrow viewports, retain a recognizable compact version without displacing navigation and theme controls. Verify accessible progress information and reduced motion.

## Landing page and application shell

- Center the attribution footer within the **main content region**, excluding the sidebar; allow accessible wrapping on narrow layouts without overlapping floating controls.
- Keep the illustrated landing page and Word of the Day card as the main focal point; check pinyin, HSK badges, navigation labels and legal text for legibility and contrast.
- The persistent Practice session reminder should have compact icon-based Resume and End actions with accessible labels and tooltips. Omit the duplicate exercise count; detailed progress belongs in the session view. Dismissing the reminder must never delete the session. Destructive End requires confirmation when it would discard unfinished work.
- Update the reminder reactively after in-app session changes and after cross-tab storage updates. The browser `storage` event does not fire in the document making the change, so use shared state or app events for same-tab mutations and check persisted status again on focus or visibility restoration. Removing a saved session must cause the reminder to disappear without an unrelated page reload.

## Duplicate-tab ownership

- The first active tab keeps exclusive application ownership. Every duplicate displays a **full-screen blocking state**, replacing accessible navigation and main content. The small dismissible bottom-right warning is not sufficient.
- The blocked view directs the user to the original tab. It has no bypass, dismissal or manual takeover action.
- Closing the original tab releases ownership; a waiting tab may become active when exclusivity can be established safely. On crashes, stale ownership must expire so the application is not blocked forever. If multiple duplicates are waiting, grant ownership to only one.
- Use a robust ownership protocol rather than an indefinitely persistent localStorage boolean. Cross-tab coordination, crash recovery, cleanup and write isolation require implementation and browser verification. Do not assume `beforeunload` always runs.

## Word Sets

- Preserve the two-panel vocabulary-management layout and existing compact selected-set list.
- Remove the redundant explanation beginning "Words saved from Dictionary go to this set" from the selection panel.
- Replace "Use [full set name] in Practice" with **Practice this set**, a concise primary action. It opens Practice Setup with the selected set preselected, preserves other configuration and does not start a session before review. Respect global exact-entry exclusions and explain if no eligible words remain.
- Keep global **Excluded** vocabulary management separate from personal Word Sets, with clear navigation and a count. Maintain readability of Hanzi, pinyin and meanings, keyboard access to row actions, safe deletion, and predictable search behavior.

## Write Hanzi, Translate and Games

- Preserve the Write Hanzi vocabulary finder and large dedicated writing workspace, the Translate direction toggle and primary input area, and the Games card hierarchy, subject to responsive and accessible sizing.
- Investigate the Write Hanzi screenshot showing a selected Word Set with zero words while the suggestion list displays many dictionary entries. Do not assert the source-selection logic is wrong before inspecting the implementation; if suggestions are global, label their source clearly rather than implying they belong to the empty set.
- Clarify AI service availability and disabled control states in Translate and AI-assisted games. Avoid silently showing activity generation before the user starts an AI-dependent game.
- Review the oversized China Map Challenge card in Games for intentional hierarchy and consistent small-screen behavior without removing its prominence solely based on this screenshot.
- Sentence Relay interaction and correction behavior are owned by [Sentence Relay](../../games/sentence-relay.md).

## Credits and third-party acknowledgements

A dedicated Docs **Credits and acknowledgements** article should distinguish resources actually bundled or used in Zipeek from **design and learning inspirations**. Audit the repository before declaring any particular license obligation or asset provenance. Candidate resources to verify include Hanzi Writer Data, its underlying stroke-data sources, CC-CEDICT, fonts, icons and landscape artwork. Duolingo and Ninchanese can be listed as inspirations where genuinely relevant, without suggesting affiliation or endorsement. A concise Credits link may supplement legally required footer notices; copyright and license notices must remain available.

## Acceptance status

These are approved design requirements and audit targets. Do not label them fixed or tested until implementation and browser checks demonstrate the behavior. Existing issue reports and screenshots remain evidence; reconcile overlapping issues rather than replacing or duplicating them without review.
