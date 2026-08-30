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
an ALSA default that targets PipeWire, and installs
`/usr/bin/aqua-audio-probe`, `/usr/bin/aqua-audio-rootfs-check`, and the
production-adapter acceptance binary at
`/usr/libexec/aqua-tests/aqua-audio-adapter-probe`. The checker
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

`scripts/check-audio-output-replug-qemu.sh` records the restoration boundary
of this declared HDA model instead of claiming unsupported recovery. After the
selected PCI 05.0 controller is removed and fallback playback succeeds, QMP
can add a replacement HDA controller but QEMU rejects runtime insertion of its
`hda-output` codec because that child bus is not hotpluggable. The bounded QMP
helper requires that specific rejection, deletes the incomplete controller,
waits for `DEVICE_DELETED`, and the guest then requires exactly one ALSA and
one authoritative native output plus a second non-silent playback on PCI 04.0.
This proves fail-closed rollback and stable fallback, not output restoration;
positive replug remains open for a future hotpluggable virtual audio model and
for separately authorized physical-hardware evidence.

`scripts/check-audio-nondefault-unplug-qemu.sh` covers the inverse topology
boundary without changing the declared devices. The native probe first requires
two outputs with PCI 04.0 authoritative, QMP removes the non-default PCI 05.0
controller and acknowledges `DEVICE_DELETED`, and ALSA plus the native graph
must converge to one output while 04.0 remains the default. Non-silent playback
before and after removal is captured only on that stable primary WAV backend.
This is bounded virtual non-default-device loss evidence, not general hotplug or
physical hardware support.

`scripts/check-audio-active-nondefault-unplug-qemu.sh` moves that removal into
an active default-route stream. The probe publishes an active marker after 480
frames, fails on any subsequent PCM error instead of recovering it, and must
complete the same client at exactly 48,000 frames after QMP removes PCI 05.0.
The native graph must still converge to one authoritative PCI 04.0 output, the
primary WAV must remain non-silent, and no interruption marker is accepted.
This proves only the declared virtual non-default-removal boundary.

`scripts/check-audio-active-default-unplug-qemu.sh` covers the selected-route
counterpart. It first makes PCI 05.0 authoritative, starts route-aware playback,
and removes that controller through QMP after the 480-frame active marker. The
ALSA compatibility PCM can continue accepting buffered writes after device
deletion, so PCM write success is not treated as route survival. The probe
instead watches the authoritative native topology, requires convergence to the
sole PCI 04.0 default, emits `reason=route-loss`, exits with the dedicated
interrupted status, and emits no playback-success marker. A new client must then
complete 48,000 frames into the non-silent fallback WAV backend. The removed
backend's short buffered prefix is not used as host transport evidence. This is
bounded QEMU virtual-topology evidence, not physical hotplug support.

`scripts/check-audio-active-service-loss-qemu.sh` adds an active-client service
failure boundary. The acceptance probe first writes 480 frames to the declared
HDA/WAV path and publishes an active marker, after which the test terminates
the exact PipeWire PID owned by the per-user supervisor. The client must exit
with its dedicated interrupted status and `Broken pipe` detail without a
playback-success marker. The supervisor must then expose a different PipeWire
PID, `attempts=2`, a running authoritative graph, and a successful new
48,000-frame playback. The host requires the combined WAV to remain non-silent
and the recovery shell to remain responsive. This does not prove every client
policy or physical-device failure mode.

`scripts/check-audio-active-policy-loss-qemu.sh` covers the complementary
policy-service failure. It terminates the exact supervisor-owned WirePlumber
PID after an active playback client reaches 480 frames, requires explicit
`Broken pipe` interruption without playback success, and observes the ordered
full-stack response: the old PipeWire process is stopped, both old PIDs retire,
and attempt 2 starts new PipeWire and WirePlumber processes with restart 1. A
new client must then complete 48,000 frames into the non-silent WAV backend
while recovery remains available. This proves only the declared supervisor
policy for virtual Intel HDA; it is not seamless playback or physical-device
evidence.

`scripts/check-audio-restart-exhaustion-qemu.sh` exercises the opt-in profile's
real `max_restarts=3` policy rather than a fixture. It terminates four distinct
supervisor-owned PipeWire processes, requires exactly three ordered full-stack
restarts, and then requires `state=degraded`, `attempts=4`, `restarts=3`, and
`failed_service=pipewire`. The supervisor PID file, both recorded service PIDs,
and `/run/user/1000/pipewire-0` must be gone. `wpctl` and a new playback probe
must fail closed while the recovery shell remains responsive. This is bounded
QEMU service-exhaustion evidence; it does not enable audio by default or prove
physical-device behavior.

`scripts/check-audio-control-restart-exhaustion-qemu.sh` applies the real
PipeWire restart budget to native volume/mute acknowledgement. A bounded
precondition must complete one successful control cycle before four successive
PipeWire losses. The first three losses renew the complete media pair; the
fourth reaches the cleaned degraded state at attempt 4/restart 3. Native control
open must then fail without another acknowledgement, with all recorded media
processes and the PipeWire socket gone while recovery remains responsive. This
is opt-in virtual service-exhaustion evidence only.

`scripts/check-audio-capture-restart-exhaustion-qemu.sh` proves the same real
PipeWire budget against the controlled input path. QEMU's timer-backed `none`
ADC first supplies one exact 4,800-frame zero-PCM capture. The test then
terminates four successive PipeWire processes, requires complete media-pair
renewal after the first three losses, and reaches the cleaned degraded state at
attempt 4/restart 3. A second capture must fail at open without a success marker,
while the recovery shell remains responsive. No host microphone is requested;
this is bounded virtual capture-exhaustion evidence.

`scripts/check-audio-policy-restart-exhaustion-qemu.sh` applies the same real
budget to policy-service failure. It terminates four successive supervisor-owned
WirePlumber processes and requires a different PipeWire/WirePlumber pair after
each of the first three losses. The fourth loss must leave `state=degraded`,
`attempts=4`, `restarts=3`, and `failed_service=wireplumber`; all eight retired
media PIDs, the supervisor PID file, and the PipeWire socket must be gone.
`wpctl` and a new playback probe must fail closed while recovery remains
responsive. This is bounded virtual policy-exhaustion evidence, not default
audio enablement or physical-device support.

`scripts/check-audio-control-policy-restart-exhaustion-qemu.sh` applies that
WirePlumber budget to native volume/mute acknowledgement. One bounded healthy
control cycle must succeed before four successive policy-service losses. The
first three losses renew both media processes; the fourth must reach the cleaned
`failed_service=wireplumber` degraded state at attempt 4/restart 3. A new native
control open must fail without false acknowledgement after every retired media
PID, the supervisor PID file, and the PipeWire socket are gone. Recovery remains
responsive; this is opt-in virtual policy-exhaustion evidence only.

`scripts/check-audio-capture-policy-restart-exhaustion-qemu.sh` closes the
matching controlled-input policy boundary. One exact 4,800-frame zero-PCM
capture establishes the healthy precondition before the test terminates four
successive WirePlumber processes. The first three losses must renew both media
processes; the fourth must produce the cleaned degraded state with
`failed_service=wireplumber`. A new capture must fail without a success marker,
while recovery remains responsive and no host microphone is requested. This is
bounded virtual capture policy-exhaustion evidence only.

`scripts/check-audio-active-capture-loss-qemu.sh` adds the matching active-input
service boundary without requesting a host microphone. QEMU's timer-backed
`none` ADC supplies controlled zero PCM; the probe publishes an active marker
after 480 frames, then the test terminates its exact supervisor-owned PipeWire
PID. The client must exit with status 3 and `Broken pipe` from capture I/O,
without any capture-success marker. A different PipeWire PID and `attempts=2`
must restore the graph before a new client captures exactly 4,800 zero-PCM
frames. This is virtual service-loss evidence, not physical microphone or
device-disconnect support.

`scripts/check-audio-active-capture-policy-loss-qemu.sh` covers the active-input
policy-service boundary against the same controlled source. It records the
supervisor-owned PipeWire and WirePlumber PIDs, publishes the 480-frame capture
checkpoint, and terminates only WirePlumber. The client must exit with status 3
and `Broken pipe` without a capture-success marker. The supervisor must stop the
old PipeWire process, retire both old PIDs, and start a new ordered pair at
attempt 2/restart 1 before a new client captures exactly 4,800 zero-PCM frames.
This proves full-stack recovery for the declared virtual policy failure; it is
not seamless capture or physical microphone evidence.

`scripts/check-audio-active-input-unplug-qemu.sh` separates input-device loss
from media-service loss. A private D-Bus injector first proves one complete
4,800-frame bipolar capture without host microphone access. A second capture
publishes its 480-frame active checkpoint before QMP removes the sole duplex HDA
controller at PCI 04.0 and acknowledges `DEVICE_DELETED`. The route-aware probe
requires the native graph to converge to zero inputs and no default input,
reports `input-route-loss` with no capture-success marker, and then confirms a
new capture is blocked by the missing authoritative route while PipeWire,
WirePlumber, and recovery remain responsive. This is bounded virtual
input-topology evidence, not physical microphone, jack, USB, or Bluetooth
hotplug support.

`scripts/check-audio-control-service-loss-qemu.sh` exercises the native
volume/mute acknowledgement boundary during a deterministic real graph outage.
It first requires one successful control cycle, briefly holds the supervisor
with `SIGSTOP`, terminates that supervisor's PipeWire process, and waits for
the private socket to disappear. The control probe must then fail at native
`open`, return non-zero, and emit no successful acknowledgement. `SIGCONT`
releases the unchanged supervisor policy; a different PipeWire PID,
`attempts=2`, and a second successful control cycle prove recovery. This does
not make submission equivalent to acknowledgement or enable audio by default.

`scripts/check-audio-control-submission-budget-qemu.sh` runs the packaged Rust
adapter against the production `aqua-audio-native` bridge. Its deterministic
fault boundary rejects the first three native control calls while repeated
snapshots of the unchanged canonical graph retain one generation. The adapter
must block the fourth call before it reaches the bridge. A direct real native
volume change then advances the authoritative graph generation, reopens one
adapter submission, and requires a later snapshot to acknowledge the desired
value. This is opt-in virtual runtime evidence; the default image stays
sound-free and physical hardware is not claimed.

`scripts/check-audio-control-route-loss-qemu.sh` proves that an in-flight
volume request belongs to the output generation that received it. The profile
prepares fallback PCI 04.0 at 37%, selects PCI 05.0 at 63%, submits 37% to 05.0,
and removes that selected controller after the adapter exposes the pending
target-bound request. Although fallback 04.0 already equals 37%, the adapter
must reject the old request as lost rather than acknowledge it, avoid a
redundant resubmission, and preserve the running media services and recovery
shell. The rootfs rehearsal explicitly rebuilds the local native bridge and C
probe before image finalization so cached Buildroot stamps cannot hide source
changes. This remains opt-in virtual-device evidence.

`scripts/check-audio-mute-route-loss-qemu.sh` supplies the symmetric mute
boundary. Both routes use 42% volume, fallback PCI 04.0 is prepared muted, and
selected PCI 05.0 is kept unmuted before a pending mute request is bound to it.
After acknowledged removal of 05.0, the already-muted fallback must not confirm
the old request. The adapter reports it rejected or lost, observes the retained
preference already satisfied without resubmission, and leaves services plus the
recovery shell responsive.

`scripts/check-audio-control-policy-service-loss-qemu.sh` proves the matching
WirePlumber boundary. After one acknowledged volume/mute cycle, it terminates
the supervisor-owned policy process and pauses the supervisor only after that
loss has retired PipeWire and entered its bounded restart delay. Native control
open must fail without an acknowledgement while the complete graph is absent.
Recovery must replace both old media processes at attempt 2/restart 1 before a
second control cycle succeeds. This remains opt-in virtual-device evidence and
does not enable audio by default.

`/usr/bin/aqua-session-check` is the recovery-safe aggregate checker for the same contract. Boot writes its output to `/run/aqua-session-check.log`, and users can run it manually from the recovery shell.

## Current Boot Choice

QEMU's direct kernel loader with `rootfs.ext2` attached as a virtio disk remains the deterministic development and recovery-validation path.

The selected installed-system bootloader is GRUB2 x86_64 UEFI. Buildroot generates `images/efi-part/EFI/BOOT/bootx64.efi` with Linux, ext2/ext4, FAT, GPT, EFI GOP, and filesystem-label search modules embedded. Aqua's post-image hook replaces Buildroot's generic `/dev/sda1` menu with the versioned `board/aqua/x86_64/grub.cfg` contract. The installer plan copies the EFI artifact to the UEFI fallback path `EFI/BOOT/BOOTX64.EFI` and writes the same configuration; it does not require `grub-install` in the target rootfs. GPT names identify the installed partitions and GRUB passes `root=PARTLABEL=AQUA_ROOT`, which the built-in kernel resolves without an initramfs. EDK2 QEMU validation boots this chain to recovery. See [ADR 0002](adr-0002-bootloader.md).
