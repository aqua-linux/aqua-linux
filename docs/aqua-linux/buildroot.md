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
WirePlumber are absent from the default image. When explicitly enabled, it requires the
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
PipeWire, WirePlumber, alsa-lib, Lua, and GLib unselected. The separate
`aqua_x86_64_audio_rehearsal_defconfig` selects the narrow audio closure without
changing that default. Its restricted ALSA plugin lists include `ioplug` and
the `ext` control plugin, which PipeWire's ALSA compatibility modules require
for their external PCM and control APIs; it does not enable the full plugin
set. The profile also selects the project-authored MIT `aqua-audio-native`
package, whose versioned bounded ABI connects the typed transport to
WirePlumber's object-manager, mixer, and default-node APIs, and the test-only
MIT `aqua-audio-probe` used to submit bounded playback/capture streams through
ALSA and verify acknowledged volume/mute changes through the native bridge. The
probe is acceptance instrumentation, not a policy or product daemon.
Because the pinned Buildroot WirePlumber package installs only to the target
tree, the Aqua package first invokes its standard Meson staging install so the
public headers, library, and pkg-config metadata are available to dependents.
`scripts/rehearse-audio-buildroot-closure.sh` compiles that bridge, runs
`show-info` and `legal-info` against the pinned tree, and writes local evidence
under `build/audio-rehearsal/`; the 2026-08-30 run verified the recorded stack
and bridge versions and kept D-Bus, Bluetooth, JACK, PulseAudio, FFmpeg,
GStreamer, and V4L2 disabled. Generated evidence remains untracked and is not
release clearance. eudev is already packaged for general device discovery.

The audio profile applies `rootfs-overlay` first and the dedicated
`audio-rootfs-overlay` second. Only that second overlay changes
`media-services.conf` to `enabled=true`; the default defconfig continues to use
only the disabled base overlay and excludes every audio package. The opt-in
rootfs records exact stack versions in `/etc/aqua/audio-stack.conf`, installs
an ALSA default that targets PipeWire, and installs both
`/usr/bin/aqua-audio-probe` and `/usr/bin/aqua-audio-rootfs-check`. The checker
requires the `aqua` UID/GID
1000 identity and its audio/video/input groups, exact PipeWire and WirePlumber
configuration and module paths, the native bridge, regular executable service
binaries, absent PulseAudio/JACK daemons, and no root init or inittab service.
On a live system it additionally requires the private 1000:1000 mode-0700
`/run/user/1000` directory. `scripts/check-audio-rootfs-contract.sh` exercises
positive and fail-closed fixtures, while
`scripts/rehearse-audio-rootfs-contract.sh` builds the complete opt-in image and
runs the same checker against Buildroot's final `rootfs.tar` artifact after the
fakeroot user and ownership stage. Its generated
hashes, config, and result stay under `build/audio-rootfs-contract/` and remain
untracked. The same profile applies
`board/aqua/x86_64/linux-audio-qemu.config`, which enables the Intel HDA and
generic codec kernel path plus ACPI PCI hotplug only for this opt-in image. The
rehearsal rejects a zero-length kernel artifact left by an interrupted build
and reinstalls the kernel image before publishing local evidence. The default
kernel configuration remains sound-free.

Aqua has a locked unprivileged graphical-session identity, a private user-owned
runtime directory, and explicit `video`, `audio`, and `input` group membership.
The opt-in rootfs contract and declared Intel HDA QEMU output and controlled
input baselines are complete. `scripts/check-audio-qemu.sh` declares
`ich9-intel-hda` with `hda-duplex`, proves sink/source-node discovery, volume
and mute, writes 48,000 stereo S16LE playback frames to a non-silent 48 kHz WAV
capture, and forces WirePlumber restart recovery. The separate
`scripts/check-audio-input-qemu.sh` profile preserves that device declaration,
uses QEMU's timer-backed `none` ADC as an exact zero-PCM source, and requires
the unprivileged session to capture 4,800 stereo S16LE frames whose peak remains
zero. This controlled pattern proves stream establishment and unmodified sample
delivery without requesting a host microphone or its permissions; it does not
claim acoustic microphone quality. The WAV output run still records
`input_stream=false` because that backend has no ADC.
`scripts/check-audio-signal-input-qemu.sh` adds a separate non-silent input
profile. It compiles the repository's bounded GLib/GIO helper, attaches it to
QEMU's private peer-to-peer D-Bus display through QMP, and registers an
`AudioInListener` that serves a fixed-amplitude 1 kHz bipolar square wave. The
guest must capture exactly 4,800 stereo S16LE frames through HDA, ALSA, and
PipeWire with all 9,600 samples non-zero, a 4,096 peak, and balanced positive
and negative counts. The host separately requires the injector to report at
least 19,200 served bytes and the exact declared format. No session bus, host
microphone, or physical input permission is used. This is deterministic
virtual-device transport evidence, not acoustic quality or physical microphone
support.
`scripts/check-audio-input-disconnect-qemu.sh` reuses that private listener in
a fail-closed profile. It serves exactly 9,600 bytes of positive 4,096-amplitude
PCM, then rejects subsequent D-Bus reads. The guest must reject QEMU's retained
one-polarity buffer as `invalid-injected-signal`, must not emit the successful
bipolar marker, and must still prove PipeWire, WirePlumber, and the recovery
shell are responsive. The host independently requires the exact injected
failure reason and byte boundary. This is a bounded virtual input-source
failure, not physical cable, USB, or microphone failure evidence.
`scripts/check-audio-multi-route-qemu.sh` declares two independent Intel HDA
controllers with `hda-output` codecs and separate WAV backends. It requires two
ALSA playback devices and two authoritative PipeWire sinks, plays on the
initial default, changes the configured default through `aqua-audio-native`,
waits for an effective-route snapshot that acknowledges the alternate node,
then plays again. Both host WAV files must contain non-silent 48 kHz stereo
S16LE data, so a marker-only or unchanged-route result fails.
`scripts/check-audio-hotplug-qemu.sh` fixes those controllers at PCI slots 04.0
and 05.0, explicitly selects 05.0 as the configured default, proves playback
there, and removes that controller through a bounded QMP `device_del` request.
The host requires the matching asynchronous `DEVICE_DELETED` event; the guest
requires ALSA to shrink to one playback device, the native adapter to report
one authoritative default output, and playback to resume into the remaining
non-silent WAV backend. This is selected-device loss and fallback evidence for
the declared virtual topology, not general USB, Bluetooth, or physical-device
hotplug support. Additional media error matrices remain open. The typed
transport, native bridge, ordered per-user supervisor, dependency rehearsal,
rootfs contract, and fail-closed `aqua-service-adapters` state/intent boundary
are present, but none makes
`/dev/snd` alone sufficient to enable Settings. No root-owned media daemon,
command-output parser, or globally writable `/dev/snd` fallback is permitted.

`/usr/bin/aqua-session-check` is the recovery-safe aggregate checker for the same contract. Boot writes its output to `/run/aqua-session-check.log`, and users can run it manually from the recovery shell.

## Current Boot Choice

QEMU's direct kernel loader with `rootfs.ext2` attached as a virtio disk remains the deterministic development and recovery-validation path.

The selected installed-system bootloader is GRUB2 x86_64 UEFI. Buildroot generates `images/efi-part/EFI/BOOT/bootx64.efi` with Linux, ext2/ext4, FAT, GPT, EFI GOP, and filesystem-label search modules embedded. Aqua's post-image hook replaces Buildroot's generic `/dev/sda1` menu with the versioned `board/aqua/x86_64/grub.cfg` contract. The installer plan copies the EFI artifact to the UEFI fallback path `EFI/BOOT/BOOTX64.EFI` and writes the same configuration; it does not require `grub-install` in the target rootfs. GPT names identify the installed partitions and GRUB passes `root=PARTLABEL=AQUA_ROOT`, which the built-in kernel resolves without an initramfs. EDK2 QEMU validation boots this chain to recovery. See [ADR 0002](adr-0002-bootloader.md).
