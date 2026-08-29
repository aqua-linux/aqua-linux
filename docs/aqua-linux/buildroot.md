# Aqua Linux Buildroot Notes

Milestone 0 and 1 use a Buildroot external tree at:

`br2-external/aqua/`

The first development target is QEMU x86_64. The default image boots to a BusyBox-based text recovery shell and emits stable serial boot markers. The custom Wayland compositor is packaged and runs through explicit graphical-session gates; default boot keeps `boot_graphics=false` and `autostart=false` so recovery remains deterministic.

## Build

```sh
scripts/build-image.sh
```

The script pins the supported Buildroot `2025.02.17` LTS release and SHA-256
`13618704563ad0b928a4564aaa73e2db97e12e8df0ed5ae874744a83964a023a` by
default, then places output in:

`build/buildroot-output/images/`

Buildroot needs GNU gcc/g++ host compilers. On macOS where `/usr/bin/gcc` is Apple clang, use a Linux host or:

```sh
scripts/build-image-docker.sh
```

For Docker Desktop on macOS, the volume-backed path is usually more stable because Buildroot's working tree stays inside Docker's Linux filesystem:

```sh
scripts/build-image-docker-volume.sh
```

The volume-backed workflow defaults to
`aqua-linux-buildroot-2025-02-work`, so output from the earlier series cannot
be mistaken for the LTS build. The build verifies cached and newly downloaded
archives. It also validates critical extracted-source files; an incomplete
source or output tree is preserved with an `.invalid.<timestamp>` suffix before
a verified archive is extracted through a temporary directory and installed
atomically.

Expected artifacts:

- `bzImage`
- `rootfs.ext2`
- `disk.img`, currently an alias of `rootfs.ext2` for the Milestone 1 QEMU path.

The image also carries the installer prerequisites selected for Milestone 9: util-linux `sfdisk`, dosfstools `mkfs.fat`, e2fsprogs `mkfs.ext4`, GNU tar, and the BusyBox mount tools. The post-image hook treats their executable paths as an image contract and fails the build when one is absent.

## Run

```sh
scripts/run-qemu.sh
```

Serial output is written to:

`build/qemu-serial.log`

The image check normalizes QEMU serial markers into:

```text
build/aqua-boot-summary.txt
build/aqua-boot-summary.json
```

Success markers:

```text
[AQUA-BOOT] stage=rcS-start product="Aqua Linux"
[AQUA-BOOT] stage=filesystems-mounted status=ok
[AQUA-BOOT] stage=os-release id=aqua pretty="Aqua Linux Milestone 1"
[AQUA-BOOT] stage=session-config status=ok autostart=false boot_graphics=false recovery_tty=true
[AQUA-BOOT] stage=session-runtime status=ok user=aqua uid=1000 runtime_dir=/run/user/1000 control_dir=/run/aqua mode=0700
[AQUA-BOOT] stage=session-env status=ok wayland=aqua-wayland-0 xdg=/run/user/1000 assets=/usr/share/aqua
[AQUA-BOOT] stage=session-bootstrap status=ok runtime_dir=/run/user/1000 autostart=false boot_graphics=false session_started=false
[AQUA-BOOT] stage=compositor-assets status=ok root=/usr/share/aqua
[AQUA-BOOT] stage=output-plan status=ok backend=nested-dev-window boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=visible-preview-plan status=ok preview_window_started=false boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=scene-contract status=ok surfaces=7 boot_graphics=false
[AQUA-BOOT] stage=render-plan status=ok commands=7 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=paint-plan status=ok steps=7 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=frame-plan status=ok format=rgba8888 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=frame-buffer status=ok bytes=6291456 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=raster status=ok rects=7 glass_layers=15 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=glass-primitives status=ok layers=15 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=raster-export status=ok bytes=4718609 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=raster-png-export status=ok bytes=6293028 boot_graphics=false renderer_started=false
[AQUA-BOOT] stage=session-check status=ok no_graphics=true
[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh
```

## Runtime Session Contract

The image writes a recovery-safe compositor session contract to:

`/etc/aqua/compositor-session.conf`

Current values:

```text
wayland_socket=aqua-wayland-0
runtime_dir=/run/user/1000
runtime_asset_root=/usr/share/aqua
autostart=false
boot_graphics=false
recovery_tty_required=true
```

This file keeps Milestone 1 recovery behavior stable while explicit graphical validation profiles switch individual fields deliberately.

The source-tree safety gate is:

```sh
scripts/check-default-recovery-safety.sh
```

It regenerates the default session files, requires exactly one recovery-safe
value for `autostart`, `boot_graphics`, and the fallback TTY, confirms both
console gettys remain supervised, and rejects a default QEMU or GRUB command
line containing `aqua.boot_graphics=1`. Its launcher fixture also proves that
the compositor supervisor is not invoked when the kernel opt-in is absent, the
graphics profile is missing, or the fallback TTY requirement is disabled.

Buildroot creates a locked `aqua` desktop account with fixed UID/GID 1000 and
membership in `video`, `audio`, and `input`. Boot rejects symlink or
non-directory runtime targets, then prepares `/run/user/1000` as `1000:1000`
with mode `0700`. The Wayland socket and user-session IPC live there.
`/run/aqua` remains a separate user-owned control directory for bounded
supervisor state and recovery evidence. The root boot process enters graphics
only through `aqua-session-user-launch`, which drops to the `aqua` identity;
the root recovery TTY remains independent.

The graphical supervisor also owns `/usr/bin/aqua-media-service-supervisor`
inside that unprivileged session. Its checked-in policy at
`/etc/aqua/media-services.conf` is disabled by default because PipeWire and
WirePlumber are not packaged yet. When explicitly enabled, it requires the
private session runtime, starts PipeWire before WirePlumber, waits a finite
time for the PipeWire socket, restarts the complete pair within a bounded
budget, stops WirePlumber before PipeWire, and records disabled, running,
restarting, stopped, or degraded state under `/run/aqua`. Media failure does
not block the compositor or remove the independent recovery TTY.

The image also writes the derived session environment to:

`/etc/aqua/session.env`

Current exported values:

```text
WAYLAND_DISPLAY=aqua-wayland-0
XDG_RUNTIME_DIR=/run/user/1000
AQUA_ASSET_ROOT=/usr/share/aqua
AQUA_SESSION_MODE=nested-dev
AQUA_COMPOSITOR_AUTOSTART=false
AQUA_BOOT_GRAPHICS=false
```

The recovery profile sources this file, but the graphical session is still not autostarted.

If `/usr/bin/aqua-compositor` is packaged, boot also runs the recovery-safe bootstrap probe against `/etc/aqua/compositor-session.conf` and `/run/user/1000`. The probe writes `/run/aqua-compositor-bootstrap.log`, emits a serial marker, and exits without starting a compositor session.

Boot also runs non-graphical compositor contract probes for runtime assets, static scene geometry, and the headless render plan. These write logs under `/run/aqua-compositor-*.log` and emit serial markers, but they do not draw pixels or start a desktop.

## Audio Packaging Boundary

[ADR 0004](adr-0004-audio-service-stack.md) selects ALSA/eudev below per-user
PipeWire and WirePlumber, with Aqua consuming authoritative service state
through a bounded adapter. The Buildroot 2025.02.17 LTS defconfig keeps
PipeWire, WirePlumber, alsa-lib, Lua, and GLib unselected. eudev is already
packaged for general device discovery. Aqua now has a locked unprivileged
graphical-session identity, a private user-owned runtime directory, and
explicit `video`, `audio`, and `input` group membership. Audio packaging must
still add the supported PipeWire/WirePlumber API transport, exact dependency
and legal-info evidence, and real QEMU media evidence. The ordered per-user
supervisor and fail-closed `aqua-service-adapters` state/intent boundary are now
present, but neither makes `/dev/snd` alone sufficient to enable Settings. No
root-owned media daemon, command-output parser, or globally writable `/dev/snd`
fallback is permitted.

`/usr/bin/aqua-session-check` is the recovery-safe aggregate checker for the same contract. Boot writes its output to `/run/aqua-session-check.log`, and users can run it manually from the recovery shell.

## Current Boot Choice

QEMU's direct kernel loader with `rootfs.ext2` attached as a virtio disk remains the deterministic development and recovery-validation path.

The selected installed-system bootloader is GRUB2 x86_64 UEFI. Buildroot generates `images/efi-part/EFI/BOOT/bootx64.efi` with Linux, ext2/ext4, FAT, GPT, EFI GOP, and filesystem-label search modules embedded. Aqua's post-image hook replaces Buildroot's generic `/dev/sda1` menu with the versioned `board/aqua/x86_64/grub.cfg` contract. The installer plan copies the EFI artifact to the UEFI fallback path `EFI/BOOT/BOOTX64.EFI` and writes the same configuration; it does not require `grub-install` in the target rootfs. GPT names identify the installed partitions and GRUB passes `root=PARTLABEL=AQUA_ROOT`, which the built-in kernel resolves without an initramfs. EDK2 QEMU validation boots this chain to recovery. See [ADR 0002](adr-0002-bootloader.md).
