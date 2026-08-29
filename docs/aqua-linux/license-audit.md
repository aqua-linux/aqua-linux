# Aqua Linux License Audit

Audit date: 2026-08-29

Scope: the OS repository, Rust workspace, Buildroot external tree, committed
fonts/icons, and design assets. The separately maintained `website/` repository
is outside this audit.

## Result

- Project Rust crates declare MIT and now have a matching root `LICENSE`.
- Cargo metadata reports a license expression for every locked Rust package.
- Observed Rust licenses are permissive: MIT, Apache-2.0, BSD-2-Clause, 0BSD,
  Zlib, Unicode-3.0, and Unlicense combinations.
- Aqua Core Icons are project-authored, MIT licensed, and isolated from private
  design-reference boards.
- elementary Icons is not a source dependency or runtime asset. No elementary
  SVG, path data, recolor, trace, or derivative is included; the owner-facing
  Aqua production inventory defines functional roles and filenames only.
- Noto Sans and the packaged Noto Sans Arabic fallback include their shared SIL
  Open Font License 1.1 text.
- The shared text service uses the permissively licensed Rustybuzz,
  unicode-bidi, and unicode-segmentation crates for shaping and layout.
- Reviewed Aqua Core SVGs use resvg 0.45.1 without its default text,
  system-font, memory-map, or raster-image features. The pure-Rust parsing and
  raster graph is MIT, Apache-2.0, or BSD-3-Clause licensed.
- The Weston simple-shm compatibility fixture is MIT licensed and is isolated
  from the Aqua desktop product stack.
- No private keys or environment-secret files were found in the OS source tree.

## Unresolved Publication Gate

The Aqua identity source and wallpapers were supplied by the project owner on
2026-08-27 and remain separate from the MIT code license. The complete identity
and interface source boards are stored in a Git-ignored local-only tree; only
approved transparent logo exports and runtime wallpapers are part of the
planned public payload. Project identity assets remain governed by
`ASSET_LICENSE.md`.

Before the initial public push, the repository owner must confirm that the
owner-supplied public artwork may be published under `ASSET_LICENSE.md`. Any
asset that cannot be cleared must be removed or replaced. Private reference
boards must never be used as sources for third-party icons or proprietary UI
artwork.

## Binary Distribution

Buildroot assembles many packages that are not vendored in this repository and
may include copyleft licenses. A source audit is not sufficient for distributing
an Aqua Linux image.

Every release build must run Buildroot's legal information target:

```sh
make -C build/buildroot-output legal-info
```

Release automation must archive the generated license texts, package manifest,
and corresponding-source material required by the selected Buildroot packages.
No binary image should be published until that output has been reviewed.

## Reproduce The Rust Audit

```sh
cargo metadata --format-version 1 |
  jq -r '.packages[] | [.name, .version, (.license // "UNKNOWN")] | @tsv'
```

The audit fails if any package reports `UNKNOWN`. Versions and conclusions
must be refreshed whenever `Cargo.lock`, Buildroot, fonts, icons, or runtime
assets change.
