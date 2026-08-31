# Aqua Linux License Audit

Audit date: 2026-08-30

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
- The Weston simple-shm, simple-damage, simple-touch, and terminal compatibility
  fixtures are MIT licensed and are isolated from the Aqua desktop product
  stack. The four Weston terminal frame PNGs are retained only for that fixture.
  The Weston compositor, shells, backends, and desktop runtime are not packaged.
- GLFW 3.4 is packaged under the Zlib license as a bounded native-Wayland
  compatibility dependency. The associated Aqua test fixture is MIT licensed,
  uses no OpenGL client API, and is installed only under `/usr/libexec/aqua-tests`.
- ADR 0005 selects the already packaged BusyBox `udhcpc` path for initial
  Ethernet DHCP and reserves `wpa_supplicant` for a future opt-in Wi-Fi
  rehearsal. No network package or dependency closure changed in the default
  image. The project-authored fixed-argument `aqua-udhcpc-client` launcher,
  resolver normalization, supervisor health checks, and QEMU acceptance
  harness add no runtime dependency; `wpa_supplicant` requires generated
  `legal-info` review before it can be enabled.
- No private keys or environment-secret files were found in the OS source tree.
- The OS baseline is pinned to Buildroot 2025.02.17 LTS by SHA-256. Its audio
  package metadata records PipeWire 1.2.8 (MIT/LGPL-2.1+/GPL-2.0 components),
  WirePlumber 0.5.5 (MIT), alsa-lib 1.2.13 (LGPL-2.1+/GPL-2.0+ components),
  eudev 3.2.14 (GPL-2.0+/LGPL-2.1+ components), Lua 5.4.8 (MIT), and GLib
  2.82.5 (LGPL-2.1+). The generated current-image manifest contains eudev for
  general device management; PipeWire, WirePlumber, alsa-lib, Lua, and GLib are
  not selected. Enabling them requires a refreshed generated legal-info review
  and the ADR 0004 runtime gates.
- A separate non-default audio rehearsal profile completed Buildroot `show-info`
  and `legal-info` on 2026-08-30. The exact additions were the MIT
  `aqua-audio-native` bridge, the test-only MIT `aqua-audio-probe`, alsa-lib,
  PipeWire, WirePlumber, Lua, libglib2, pcre2, and their two host helpers; the
  generated manifest verified every
  recorded stack version. D-Bus, Bluetooth, JACK, PulseAudio, FFmpeg,
  GStreamer, and V4L2 remained disabled. This local evidence does not clear
  release publication or enable the packages in the default image.
- The project-authored `aqua-audio-native` bridge is MIT licensed and carries
  its license text in the Buildroot package source. It links to WirePlumber and
  GLib only in the opt-in audio profile; the default image does not select it.
- The project-authored `aqua-audio-probe` is MIT licensed, links to alsa-lib
  and the project-authored native bridge, and exists solely for bounded opt-in
  audio acceptance, including explicit active playback and capture interruption
  reporting.
  The restart-exhaustion profile reuses the same probe and packaged services;
  the control-service-loss profile likewise reuses the existing native bridge,
  probe, and supervisor. Probe versions 11 through 14 add only project-authored
  native topology assertions and an active-stream checkpoint for non-default
  virtual-output removal, plus authoritative route-loss detection for active
  selected-output and input-device removal. These profiles introduce no
  package, runtime dependency, or license-closure change. The default image
  does not select them.
- The project-authored QEMU D-Bus audio-input injector is MIT licensed under
  the root project license. It is compiled and run only on the development
  host, dynamically links to the host GLib/GIO installation, and supports both
  successful deterministic input and bounded injected-read-failure acceptance.
  It is neither copied into the Buildroot image nor included in release
  artifacts.

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

The Buildroot 2025.02.17 validation build generated `legal-info` successfully
on 2026-08-29 and verified the available license-file hashes for its selected
packages. Buildroot also warned that its own source code had not been saved.
The generated output is local build evidence, not a cleared release bundle;
source archiving and final owner review therefore remain publication gates.

## Reproduce The Rust Audit

```sh
cargo metadata --format-version 1 |
  jq -r '.packages[] | [.name, .version, (.license // "UNKNOWN")] | @tsv'
```

The audit fails if any package reports `UNKNOWN`. Versions and conclusions
must be refreshed whenever `Cargo.lock`, Buildroot, fonts, icons, or runtime
assets change.
