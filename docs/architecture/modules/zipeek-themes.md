# `zipeek-themes` — visual context and background providers

The approved `zipeek-themes` crate is a dependency-free domain boundary for
resolving category/topic hints to a visual theme and loading background asset
metadata. The crate must not depend on Yew, exercise sessions, AI language
clients, HTTP image-generation SDKs, or the dictionary runtime.

`ThemeRequest` carries canonical topic/category labels and a seed. The resolver
counts recognized hints and selects a dominant theme; ties use the seed, while
unrecognized inputs select Neutral. This is deliberately independent of the
practice round-seed format `zipeek-round-v1`.

`StaticBackgroundProvider::load` is asynchronous and currently returns the
existing bundled `public/background.webp` for every theme. No image generation,
image caching, remote call, privacy transfer, or per-category artwork is claimed.
`web/components/themed_background.rs` performs the asynchronous request, ignores
late results after request changes/unmount and renders a fallback asset while
loading. The image is decorative, beneath content, and never blocks interaction.

Future image-generation integration should implement a provider adapter outside
the pure domain crate, with explicit consent, retention/cache policy, accessibility
and error handling. A stable topic should ordinarily be chosen at session start.
