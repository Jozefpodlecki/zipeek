# Landing Page (R127)

## Purpose and scope

Only new visitors to the root Home route receive a short first-visit landing experience. The current application Home, Word of the Day, navigation drawer and all functional routes are unchanged after entry. This is not a mandatory account sign-in or a wizard.

## Entry and persistence

- On root `#/`, show the landing if `zipeek.preferences.v1` is absent and the browser has no existing personal Word Sets, Pinyin course progress, or excluded-vocabulary record.
- Clicking **Enter Zipeek** initializes `zipeek.preferences.v1` with default settings only if it does not already exist. No independent welcome marker is stored.
- Existing users with preferences or learning records bypass the landing.
- Direct links such as `#/browse`, `#/write-hanzi`, and `#/games/cloud-match` bypass the welcome screen.
- App startup does not write default appearance preferences merely to render the initial theme; user actions persist preferences explicitly.
- localStorage is a required application environment dependency. If browser policy denies access, startup reports an explicit initialization failure instead of silently treating the visitor as new.

## UI contract

- Full-height introductory hero with a concise proposition, a single prominent Enter action using the Yew Lucide play icon, and a theme-aware, static product preview. The preview includes a 16:9 media slot and feature labels for Practice, Browse, and Word Sets; it must not imply that a video is playable until a trailer exists.
- No drawer, application header, promotional tagline badge or footer on the first-visit landing. Normal shell appears immediately after entry.
- Keyboard-operable button and reduced-motion behavior. The layout stacks on narrow displays.
- Do not initialize the dictionary for the landing itself. The preview is static and does not require data or network requests. A future trailer may replace its media slot without changing the first-visit entry contract.

## Verification

Source contract tests are not equivalent to a compiled/browser pass. Run Windows Rust/WASM checks, then verify first root visit, entry and reload, existing-user bypass, direct deep links, theme colors, touch/keyboard navigation and the explicit startup failure when localStorage is blocked. Do not reset actual learning data to test onboarding; use a clean browser profile or an isolated origin.
