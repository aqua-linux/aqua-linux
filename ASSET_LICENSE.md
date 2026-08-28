# Aqua Linux Asset Policy

The MIT license in [LICENSE](LICENSE) applies to Aqua Linux source code and
project-authored documentation. It does not automatically apply to visual
assets.

## Project Identity And Concept Assets

Unless a file has a separate license notice, no license is granted for:

- Aqua Linux names, logos, wordmarks, and brand marks.
- Wallpapers under `docs/aqua-linux/assets/`.
- Screenshots, videos, and other promotional artwork.

Runtime screenshots under `docs/aqua-linux/assets/runtime/` are project
documentation captures governed by this screenshot policy. Their manifest
records technical provenance and integrity, not permission to reuse Aqua Linux
branding independently.

These files may not be reused as another product's identity or redistributed
separately without permission from the relevant rights holder. Private concept
boards are stored outside the public Git payload. They are design inputs, not
runtime icon sheets; do not extract logos, application icons, device frames, or
other embedded artwork from them.

## Separately Licensed Assets

- Aqua Core Icons are project-authored and MIT licensed. See
  `docs/aqua-linux/assets/icons/aqua/LICENSE`.
- Noto Sans is SIL Open Font License 1.1. See
  `docs/aqua-linux/assets/fonts/OFL.txt`.

Additional notices are recorded in
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

## Release Gate

Before a public source push or binary release, a maintainer must confirm that
the project has publication and distribution rights for every project-specific
logo, wallpaper, and reference image included in that publication. Assets with
unconfirmed provenance must be removed or replaced.
