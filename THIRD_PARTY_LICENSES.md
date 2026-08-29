# Third-Party Licenses

This file summarizes third-party material intentionally used by the Aqua Linux
source tree. Exact Rust versions are authoritative in `Cargo.lock`; Buildroot
release obligations are authoritative in its generated `legal-info` output.

## Committed Assets

### Noto Sans

- Use: embedded UI text in the software renderer and first-party applications.
- Source: official Noto Fonts project.
- License: SIL Open Font License 1.1.
- Local text: `docs/aqua-linux/assets/fonts/OFL.txt`.

## Compatibility Fixture

### Weston Simple SHM

- Use: Wayland protocol compatibility fixture in QEMU.
- Upstream: Weston 12.0.1.
- License: MIT.
- Scope: only the reference client is installed under
  `/usr/libexec/aqua-tests`. Weston compositor, shells, backends, and desktop
  session are not packaged or started.

## Direct Rust Dependencies

| Package | Locked version | Declared license |
| --- | --- | --- |
| `calloop` | 0.14.4 | MIT |
| `drm` | 0.14.1 | MIT |
| `fontdue` | 0.9.3 | MIT OR Apache-2.0 OR Zlib |
| `minifb` | 0.28.0 | MIT |
| `png` | 0.18.1 | MIT OR Apache-2.0 |
| `polling` | 3.11.0 | Apache-2.0 OR MIT |
| `portable-pty` | 0.9.0 | MIT |
| `rustybuzz` | 0.20.1 | MIT |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 |
| `smithay` | 0.7.0 | MIT |
| `unicode-bidi` | 0.3.18 | MIT OR Apache-2.0 |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 |
| `vt100` | 0.16.2 | MIT |
| `wayland-server` | 0.31.14 | MIT |

Linux-only Smithay support also uses `input`, `libc`, `tempfile`,
`wayland-client`, and `wayland-protocols`; their exact versions and
permissive license expressions are recorded in `Cargo.lock`.

The full transitive Rust graph was inspected through `cargo metadata` on
2026-08-27. Every package reported a license expression. Observed expressions
use MIT, Apache-2.0, BSD-2-Clause, 0BSD, Zlib, Unicode-3.0, and Unlicense
combinations. This summary does not replace upstream license texts.

## Buildroot Images

Buildroot downloads and assembles the Linux kernel, BusyBox, GRUB, Mesa, and
other packages under their own licenses. Those packages are not relicensed by
the Aqua Linux MIT license.

Before publishing a binary image, generate and review Buildroot legal
information:

```sh
make -C build/buildroot-output legal-info
```

Archive that output with the release, including notices and corresponding
source required by the selected package licenses.

## Aqua-Specific Artwork

Project logos, wallpapers, screenshots, and promotional
artwork are governed by [ASSET_LICENSE.md](ASSET_LICENSE.md), not the MIT code
license. Their provenance is a blocking review item before the first public
push or binary release.
