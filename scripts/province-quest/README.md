# Province Quest AreaCity gameplay inputs

The current project snapshot includes the user-generated geographic assets from AreaCity release `2025.251231.260403` at `crates/zipeek-web/static/province-quest/`:

- `provinces.json`: 34 provincial-level map regions.
- `prefectures.json`: 34 parent provinces plus 367 prefecture-level regions. The pinned AreaCity release lacks polygons for 20 Taiwan prefecture-level hierarchy entries, so those entries are not playable.

Generate updated assets with `zipeek-cli geo fetch/audit/build --provider areacity` using the pinned inputs. The app loads these assets from the same origin and validates them before starting a round. Two question formats alternate: map selection from a Hanzi prompt and three-choice Hanzi identification of a highlighted region. The pure Rust session reducer owns attempts and review transitions. Full redistribution rights for upstream boundaries have **not** been established. Do not deploy the generated geographic assets publicly until that review is complete.

Check installed JSON structure with:

```powershell
cargo test --locked -p zipeek-exercises province_quest
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo test --release --locked -p zipeek-e2e --test province_quest_gameplay -- --nocapture
```

## Prior geoBoundaries approach (historical)

# Province Quest geographic assets

The app renders region polygons using Yew SVG `<path>` nodes, **not** raster maps or a commercial web map SDK. Geometry is build-time, served from the same origin. This project does not ship geometry until a verified source is selected; an absent asset produces a visible setup error rather than guessed boundaries.

## Acquisition with Rust

Use the built-in HTTPS downloader to retrieve the **reviewed** geoBoundaries China ADM1 layer, representing 34 provincial-level divisions. No Python, API key, or external GIS program is required:

```powershell
cargo run --release --locked -p zipeek-cli -- geo fetch --output C:\geo
```

The command downloads `C:\geo\china-adm1.geojson` and writes `C:\geo\china-adm1.source.json`, recording the provider, licensing, reference year, boundary ID, original URL, byte count, and SHA-256. Rerunning it verifies the existing files and does not contact the server. Use `--force` to redownload a reviewed snapshot. Modified, partial, or stale cached inputs are rejected; they are not silently trusted.

The reviewed source is geoBoundaries `gbOpen/CHN/ADM1`, boundary ID `CHN-ADM1-43563684`, represented year **2019**, originally attributed to geoBoundaries and Wikimedia Commons, metadata declaring Public Domain. The provider metadata is available at <https://www.geoboundaries.org/api/current/gbOpen/CHN/ADM1/>. Because the API uses `current`, the downloader checks the boundary ID and refuses upstream changes until the snapshot and licensing can be reviewed. Check boundary and territorial presentation independently before publication.

**Hard mode is not included in this download.** The geoBoundaries China ADM2 endpoint represents **2,391 county-level polygons**, not the 333-prefecture definition used by Province Quest. The executable deliberately does not relabel or bundle those counties as prefectures. A separately licensed, dated and audited prefecture-level GeoJSON and matching catalog are still needed before Hard can be enabled.

## Dataset review

The downloaded 34 polygons pass source-level type/count checks, but the converter still requires complete matching administrative names against `provinces.catalog.json`. Check name aliases, territorial coverage, and each region's location with the existing strict build step; an API response alone cannot establish that all game questions are geographically correct. The map is projected to a fixed view box; it does not invoke any external map service at runtime.

## Reproducible conversion

Run from the repository root, after reviewing inputs and filling the prefecture catalog (append its entries to `provinces.catalog.json` or supply an equivalent audited combined file):

```powershell
cargo run --release --locked -p zipeek-cli -- geo build `
  --provinces C:\geo\china-adm1.geojson `
  --catalog scripts/province-quest/provinces.catalog.json `
  --source "Reviewed boundary provider" `
  --version "2026-11 snapshot" `
  --attribution "Copyright and license attribution required by source" `
  --output crates/zipeek-web/static/province-quest
```

After procuring and auditing prefecture geometry, add `--prefectures C:\geo\china-prefectures.geojson`. Hard builds are rejected unless the catalog describes at least 300 prefecture-level entries. **333 is not hardcoded as a current official total.** Both asset files must include all 34 parental divisions.

The builder refuses unknown/missing source names, duplicate IDs, orphaned prefectures, invalid geometries and incomplete province lists. All boundaries are projected into a shared SVG coordinate system, with viewport bounds supplied for province-level zoom. No source-file names, data version or map rights are silently assumed.

**Run converter checks:**

```powershell
cargo test --locked -p zipeek-geo-assets
cargo test --locked -p zipeek-cli
```

**Run app validation after installing both assets:**

```powershell
cargo test --locked -p zipeek-exercises province_quest
cargo check --locked --target wasm32-unknown-unknown -p zipeek-web
cargo test --release --locked -p zipeek-e2e --test province_quest_gallery -- --nocapture
cargo test --release --locked -p zipeek-e2e --test province_quest_setup -- --nocapture
```

The SVG paths allow hover and keyboard selection; Yew dispatches stable region IDs to the independent game reducer. Geometry is never used to grade text or inferred dynamically from map pixels.
