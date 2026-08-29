# Aqua Linux Runtime Asset Requirements

Milestone 2 defines source assets and export rules. The Buildroot post-build step installs these assets into the root filesystem. It does not start a desktop, compositor, or splash service.

## Runtime Layout

The current Buildroot image installs runtime assets under:

```text
/usr/share/aqua/
  wallpapers/
  brand/
  icons/
  fonts/
  tokens/
```

The image also packages the acceptance-only
`/usr/libexec/aqua-tests/aqua-typography-acceptance` Wayland client. It is
never autostarted; the dedicated QEMU check launches it explicitly from the
recovery shell to validate the installed fonts and renderer path.

Expected first exports:

| Source | Runtime destination | Notes |
| --- | --- | --- |
| `docs/aqua-linux/assets/default-wallpaper.png` | `/usr/share/aqua/wallpapers/default-wallpaper.png` | Keep original source; compositor may request scaled copies later |
| `docs/aqua-linux/assets/wallpaper-pale-waves.png` | `/usr/share/aqua/wallpapers/wallpaper-pale-waves.png` | Reproducible pale-wave source master and current default |
| `docs/aqua-linux/assets/wallpaper-surf.png` | `/usr/share/aqua/wallpapers/wallpaper-surf.png` | Official alternate wallpaper |
| `docs/aqua-linux/assets/wallpaper-reef.png` | `/usr/share/aqua/wallpapers/wallpaper-reef.png` | Official alternate wallpaper |
| `docs/aqua-linux/assets/wallpaper-sunlit-water.png` | `/usr/share/aqua/wallpapers/wallpaper-sunlit-water.png` | Optional legacy wallpaper |
| `docs/aqua-linux/assets/wallpaper-moonlit-lagoon.png` | `/usr/share/aqua/wallpapers/wallpaper-moonlit-lagoon.png` | Official night/login-context alternate |
| `docs/aqua-linux/assets/aqua-symbol-primary.png` | `/usr/share/aqua/brand/aqua-symbol-primary.png` | Primary symbol |
| `docs/aqua-linux/assets/aqua-symbol-inverse.png` | `/usr/share/aqua/brand/aqua-symbol-inverse.png` | Dark-surface symbol |
| `docs/aqua-linux/assets/aqua-symbol-accent.png` | `/usr/share/aqua/brand/aqua-symbol-accent.png` | Active/focused symbol |
| `docs/aqua-linux/assets/aqua-wordmark-primary.png` | `/usr/share/aqua/brand/aqua-wordmark-primary.png` | Two-line wordmark |
| `docs/aqua-linux/assets/aqua-logo-primary.png` | `/usr/share/aqua/brand/aqua-logo-primary.png` | Combined logo for installer, About, and future splash use |
| `docs/aqua-linux/design-tokens.json` | `/usr/share/aqua/tokens/design-tokens.json` | Token source for shell/render code |
| `docs/aqua-linux/assets/icons/aqua/*.svg` | `/usr/share/aqua/icons/aqua/*.svg` | Permanent project-authored Aqua Core Icons |
| `docs/aqua-linux/assets/fonts/NotoSans-Regular.ttf` | `/usr/share/aqua/fonts/NotoSans-Regular.ttf` | Primary UI font |
| `docs/aqua-linux/assets/fonts/NotoSansArabic-Regular.ttf` | `/usr/share/aqua/fonts/NotoSansArabic-Regular.ttf` | Deterministic Arabic fallback font |
| `docs/aqua-linux/assets/fonts/OFL.txt` | `/usr/share/aqua/fonts/OFL.txt` | SIL Open Font License 1.1 |

## Export Rules

- Preserve source PNGs exactly in `docs/aqua-linux/assets/`.
- Preserve the alpha channel in every derived brand export; never flatten a symbol or wordmark onto a matte.
- Preserve wallpaper aspect ratio; scale with a centered cover crop and never stretch.
- Generate runtime-size variants only from committed source assets.
- Keep Aqua Core Icons in the permanent `/usr/share/aqua/icons/aqua/` namespace.
- Load only reviewed Aqua Core SVGs through the bounded static subset; rasterize
  directly at the target output scale and never enlarge a cached bitmap.
- Do not claim boot splash support until a boot splash process or kernel/userspace splash path exists.
- Do not claim real blur/refraction until the compositor or renderer implements it.
- Keep private design and brand-source boards under the Git-ignored `docs/aqua-linux/local-references/` tree. Do not install or publish complete boards, and do not crop runtime icons from interface references. Only the approved transparent brand exports belong in the public/runtime asset set.
- Label static or mocked values as mock in previews and screenshots.
- Keep recovery shell available even after graphical runtime starts in later milestones.

## Future Derived Sizes

These exports are expected later, but are not generated in M2 yet:

| Asset class | Sizes |
| --- | --- |
| App icon | 1024, 512, 256, 128, 64, 32 |
| Panel/status icons | 32, 24, 16 |
| Dock icons | 128, 96, 64 |
| Boot splash logo | Native framebuffer size plus centered fallback |
| Wallpaper | Native source plus target display-scaled variants |
