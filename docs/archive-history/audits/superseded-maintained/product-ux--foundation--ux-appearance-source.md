# Archived source snapshot: product-ux/foundation/ux-appearance.md

Archived observations may be superseded by current approved requirements.

# Ux Appearance

> **Current approved target behavior:** [Interface and platform policy](../../../product-ux/foundation/interface-and-platform-policy.md). Older implementation descriptions below record prior contracts and may not match the latest approved behavior or verified source.


## Theme and dark mode

- Dark is the preferred default when there is no saved choice. Offer a dark/light
  toggle in the top panel at the top-right (D25), and remember the selection across reloads.
- A saved light choice overrides the dark default. The loading shell, Yew loading
  view, and ready app use the same choice without a light flash.
- Place the GitHub repository icon in the footer alongside copyright and
  attribution, linked to https://github.com/Jozefpodlecki/zipeek.
- Show Jozef Podlecki's copyright with the current year in the footer, alongside
  the required dictionary attribution. Example: "© 2026 Jozef Podlecki".

Approved direction (D24/A27): neutral slate dark/light with blue accents.
D25 amends the placement: a permanent top panel contains brand, drawer toggle
and theme control at the top-right. Home/Practice/Browse remain in an
expandable/collapsible drawer; A22 moves the repository link into the footer.
Narrow layouts show the expanded
navigation below the top panel; the collapsed drawer is hidden and content uses
the available width. Copyright/attribution remain visible in both themes.
Native buttons/links support keyboard use; the theme switch exposes its state.
The footer is one dot-separated row: copyright, CC-CEDICT attribution, GitHub
icon. Keep the repository link's accessible name "Zipeek repository on GitHub",
`target="_blank"`, `rel="noopener noreferrer"`, existing `simple_icons` icon and
licence notice. On 2026-10-28 the user approved attribution text wrapping
inside its item while copyright, separators and repository icon remain in one
horizontal row. A35 implements that layout without horizontal overflow at 400px.
A29 refines the same user-selected shell: no decorative tile arrows, a 48px top
panel, less footer padding and no outer border on the theme button. Hover and
keyboard-focus feedback remain.
D26/A30 reuses the original landscape in both themes, through loading and app
rendering. Translucent surfaces and theme overlays keep text readable. Loading
uses a small centered spinner/label panel, with animation disabled for reduced
motion; the landscape itself is unchanged and is not hidden in light mode.
