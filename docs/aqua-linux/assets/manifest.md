# Aqua Linux Asset Manifest

This manifest records Milestone 2 design inputs and runtime assets. Canonical references define layout and visual acceptance but are not runtime source sheets.

## Private Interface References

Project-owner-supplied desktop and installer boards live under the Git-ignored
`docs/aqua-linux/local-references/` tree. They are not public assets and are
intentionally absent from this manifest. Their derived, distributable
implementation requirements are recorded in `visual-reference.md`,
`ui-contract.md`, `interface-style.md`, and `design-tokens.json`.

Rules:

- Do not commit or package private boards.
- Do not crop logos, icons, wallpapers, or controls from them for runtime use.
- Dates, data, hardware frames, applications, and third-party icons shown in
  them are illustrative.

## Approved Aqua Brand Exports

The project owner supplied the identity source on 2026-08-27. Its complete
board is retained in the Git-ignored local reference library. The public
runtime exports below preserve the approved symbol and wordmark geometry with
real PNG alpha.

| Asset | Path | Size | SHA-256 | Runtime role |
| --- | --- | --- | --- | --- |
| Primary symbol | `assets/aqua-symbol-primary.png` | 1024x1024 | `8b83ddc6778e34abeb92d244429a92f5046da5b578b52f7a3b1192065e19ab09` | Bright surfaces and installer |
| Inverse symbol | `assets/aqua-symbol-inverse.png` | 1024x1024 | `2f09895f6a0901c12e1628e66e9351f68bebf571aedcc5a72854428593272e12` | Dark surfaces |
| Accent symbol | `assets/aqua-symbol-accent.png` | 1024x1024 | `f83dec9094237bd6a04e4411a89d81a8eae115c462f53710c59d4902a41d7584` | Active and focused states |
| Primary wordmark | `assets/aqua-wordmark-primary.png` | 1024x400 | `451a7470a2eeeebada4302895ad73967dd53e7b67925e46c7b8f476c1bdbed7b` | Two-line Aqua Linux wordmark |
| Combined logo | `assets/aqua-logo-primary.png` | 1024x1024 | `b9c9437279f9a55ee2691d06f68226d5b1c10ae7782f980c7dee1aabdedea78b` | Installer, About, and future splash |

All five runtime exports use RGBA PNG. Their transparent pixels must remain
transparent when resized, packaged, rendered, or captured.

Theme mapping is fixed: Light uses the primary symbol and Dark uses the inverse
symbol. Both themes use the accent symbol for active or focused brand states.

## Current Runtime Assets

The approved brand exports and wallpaper collection are installed in the
working image. The owner-supplied Light and Dark wallpapers define the current
two-mode desktop direction. The older wallpapers remain optional legacy
material.

| Asset | Path | Status | Runtime role |
| --- | --- | --- | --- |
| Light wallpaper | `assets/wallpaper-light.png` | current-runtime | Light mode; owner-supplied 1672x941 PNG; SHA-256 `2254725043229e1801bd4524c651ca9cdf6b6c156aaaa8d2e36b1603cf19e672` |
| Dark wallpaper | `assets/wallpaper-dark.png` | current-runtime | Dark mode; owner-supplied 1672x941 PNG; SHA-256 `7a54206a067c7d3a50b9fc263b68566fcab80d32765b0b3fe86d34f049b698d0` |
| Default wallpaper alias | `assets/default-wallpaper.png` | compatibility-runtime | Legacy fallback alias |
| Pale Waves wallpaper | `assets/wallpaper-pale-waves.png` | legacy-runtime | Previous reproducible default; SHA-256 `bd749fee349ce50ceeba89457d0b24a2b3578a4a06d8366e1fef4683d9bfe455` |
| Surf wallpaper | `assets/wallpaper-surf.png` | legacy-runtime | Optional wallpaper |
| Reef wallpaper | `assets/wallpaper-reef.png` | legacy-runtime | Optional wallpaper |
| Sunlit Water wallpaper | `assets/wallpaper-sunlit-water.png` | legacy-runtime | Optional wallpaper |
| Moonlit Lagoon wallpaper | `assets/wallpaper-moonlit-lagoon.png` | legacy-runtime | Optional wallpaper |
| Primary symbol | `assets/aqua-symbol-primary.png` | approved | Bright-surface symbol |
| Inverse symbol | `assets/aqua-symbol-inverse.png` | approved | Dark-surface symbol |
| Accent symbol | `assets/aqua-symbol-accent.png` | approved | Active/focused symbol |
| Primary wordmark | `assets/aqua-wordmark-primary.png` | approved | Aqua Linux wordmark |
| Combined logo | `assets/aqua-logo-primary.png` | approved | Installer/About/future splash source |
| UI font | `assets/fonts/NotoSans-Regular.ttf` | present | First-party UI text |
| Arabic fallback font | `assets/fonts/NotoSansArabic-Regular.ttf` | present | Deterministic Arabic fallback from the official `notofonts/noto-fonts` repository; Noto Sans Arabic 2.009; SHA-256 `ceea25b464a656dc3b26849bab9356740401af62aedf1bfa8b7f0d9b75925b1b` |
| UI font license | `assets/fonts/OFL.txt` | present | SIL OFL license copy |

## Aqua Core Icons

The 13 SVGs under `assets/icons/aqua/` are project-authored permanent runtime
assets. They cover Home, Files, Aqua Drive, Trash, Browser, Terminal, Settings,
Software, Wi-Fi, volume, battery, notifications, and updates. Every icon uses
a 64 x 64 view box, flat colors, and the adjacent MIT license. No icon was
extracted from a private board or another desktop environment.

## Final Assets Still Needed

- The authoritative SVG filename, priority, and delivery list is maintained in
  [icon-production.md](../icon-production.md).
- Delivery 1 covers shell, window, session, and shared-control icons that
  replace current procedural glyphs.
- Later deliveries cover status variants, Files/places, Settings categories,
  and explicitly planned first-party application roles.

## Asset Rules

- New runtime artwork must follow [../interface-style.md](../interface-style.md).
- Package only permanent or explicitly reviewed assets in the default image.
- Preserve aspect ratio for wallpapers; use a centered cover crop instead of stretching.
- Reproduce the pale-wave master with `cargo run -p aqua-renderer --example export-pale-wallpaper -- <output.png>`.
- Mock values must be identified in design-only output and must not enter runtime defaults.
- Do not add assets from GNOME, KDE, XFCE, LXQt, Apple, or another desktop product as Aqua identity assets without explicit license and product review.
- Do not derive, trace, recolor, or redistribute elementary icons as Aqua Core
  Icons. The owner-production list defines roles and filenames only.
