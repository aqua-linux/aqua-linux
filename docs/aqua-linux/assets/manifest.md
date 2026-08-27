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

Theme mapping is fixed: LightWhite and Softtouch use the primary symbol;
Deepside and Nightmare use the inverse symbol. All four themes use the accent
symbol for active/focused brand states only.

## Current Runtime Assets

The approved brand exports and wallpaper collection are installed in the
working image. The reproducible pale-wave master is the current default. The
ocean wallpapers predate the current desktop direction and remain optional
legacy material.

| Asset | Path | Status | Runtime role |
| --- | --- | --- | --- |
| Default wallpaper alias | `assets/default-wallpaper.png` | current-runtime | Alias to the pale-wave master |
| Pale Waves wallpaper | `assets/wallpaper-pale-waves.png` | current-runtime | Current default source; SHA-256 `bd749fee349ce50ceeba89457d0b24a2b3578a4a06d8366e1fef4683d9bfe455` |
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
| UI font license | `assets/fonts/OFL.txt` | present | SIL OFL license copy |

## Temporary Icons

Lucide icons under `assets/temp-icons/lucide/` have `temporary` status as licensed development placeholders. They can support current implementation but are not the final Aqua icon identity.

## Final Assets Still Needed

- Aqua-owned or independently licensed Files, Terminal, Browser, Calendar, Photos, Music, Camera, Settings, Trash, application-overview, and search icons.
- Status icons for network, volume, battery/power, locale, and notifications.
- File-type and sidebar icons for first-party applications.

## Asset Rules

- New runtime artwork must follow [../interface-style.md](../interface-style.md).
- Keep placeholders explicitly marked until an approved replacement exists.
- Preserve aspect ratio for wallpapers; use a centered cover crop instead of stretching.
- Reproduce the pale-wave master with `cargo run -p aqua-renderer --example export-pale-wallpaper -- <output.png>`.
- Mock values must be identified in design-only output and must not enter runtime defaults.
- Do not add assets from GNOME, KDE, XFCE, LXQt, Apple, or another desktop product as Aqua identity assets without explicit license and product review.
