# ADR 0004: Audio Service Stack

## Status

Accepted on 2026-08-29. Selection is complete; packaging and runtime enablement
remain blocked by the prerequisites and evidence gates below.

## Context

Settings owns a persistent, bounded output-volume preference and mute state,
but no service currently applies either value. The Buildroot image uses BusyBox
init and now runs its graphical path as the locked `aqua` user. The current
defconfig deliberately excludes ALSA userspace, PipeWire, and WirePlumber.

At the time of this decision, the pinned Buildroot 2024.02.12 tree contained
PipeWire 0.3.81 and WirePlumber 0.4.8 and was no longer an upstream-supported
LTS. Aqua has since moved to Buildroot 2025.02.17 LTS. That exact tree provides
PipeWire 1.2.8, WirePlumber 0.5.5, alsa-lib 1.2.13, eudev 3.2.14, Lua 5.4.8,
and GLib 2.82.5 metadata. PipeWire, WirePlumber, alsa-lib, Lua, and GLib remain
unselected; eudev is already present as Aqua's general device manager but has
no audio-session permission or routing policy. Package availability is not
sufficient evidence for a desktop audio architecture; session ownership,
service supervision, client permissions, and failure behavior must exist
before the media stack is enabled.

PipeWire supplies the media graph and enforces permissions assigned to clients,
while a session manager decides how devices, nodes, routes, links, and client
access are managed. WirePlumber is the selected session and policy manager.
ALSA remains the Linux kernel/userspace hardware boundary, with eudev and ALSA
UCM data providing device discovery and configuration inputs.

## Decision

Aqua Linux will use this audio stack:

1. ALSA kernel drivers and `alsa-lib` form the hardware-facing PCM, control,
   mixer, and UCM boundary.
2. PipeWire is the per-user media server and application-facing media graph.
3. WirePlumber is the per-user PipeWire session and policy manager. Aqua will
   not ship PipeWire without a policy manager.
4. eudev provides device discovery. Access to `/dev/snd` must come from an
   explicit least-privilege session/device policy. Aqua will not run the media
   graph as root or make sound devices globally writable.
5. An `aqua-service-adapters` audio boundary will translate authoritative
   device, default-route, volume, mute, and service-health state into Aqua
   models. Production code will use the supported PipeWire/WirePlumber API,
   not parse `wpctl` output or invoke shell pipelines. `wpctl` may remain a
   bounded diagnostic and acceptance tool.
6. PipeWire starts inside the authenticated Aqua user session after its
   user-owned `XDG_RUNTIME_DIR` exists. WirePlumber starts only after PipeWire.
   The Aqua session supervisor owns ordered startup, bounded restart, shutdown,
   logs, and degraded-state reporting on BusyBox systems. This decision does
   not require changing the system init implementation to systemd.
7. Settings writes user intent through the adapter and then displays
   authoritative service state. A saved preference is not reported as applied
   until the service confirms it. Service loss disables controls, preserves the
   preference for later reconciliation, and produces a visible degraded state
   without hanging the shell.
8. The initial scope is local ALSA output and input. Bluetooth audio remains
   disabled until BlueZ, D-Bus, codec, permission, reconnect, and routing
   behavior receive their own evidence. PulseAudio and JACK compatibility
   layers are also disabled until the supported application matrix requires
   them.
9. Input-source discovery never grants microphone capture by itself. Client
   visibility and recording access must pass the selected WirePlumber access
   policy and Aqua's future user-consent contract.

## Packaging Prerequisites

Audio packages remain disabled in `aqua_x86_64_defconfig` until all of these
conditions are met:

- The supported Buildroot LTS baseline remains pinned and the compatible
  PipeWire, WirePlumber, ALSA, Lua, GLib, eudev, and optional D-Bus metadata is
  recorded from that exact source tree. Enabling the packages requires a fresh
  `legal-info` audit of the selected dependency closure. Buildroot 2025.02.17
  satisfies the baseline portion of this prerequisite; the remaining gates
  below still block packaging.
- The graphical session runs as the locked `aqua` UID/GID 1000 identity with
  private `XDG_RUNTIME_DIR=/run/user/1000`. Membership in the fixed `audio`
  group is the current fail-closed `/dev/snd` access policy. This prerequisite
  is satisfied; QEMU must keep proving the compositor and clients do not run as
  root.
- The session supervisor can start, observe, restart with a finite budget, and
  stop per-user services without falling back to a root-owned media daemon.
  This prerequisite is satisfied by the packaged Aqua media-service supervisor
  and deterministic fixture lifecycle. PipeWire and WirePlumber remain disabled
  until the adapter and packaged runtime evidence gates also pass.
- The audio adapter contract and its fail-closed unavailable/degraded behavior
  pass deterministic tests before Settings can report `backend_applied=true`.
  This prerequisite is satisfied by the renderer-independent
  `aqua-service-adapters` crate: it validates bounded typed devices and routes,
  rejects stale or conflicting generations, preserves desired volume and mute
  across service loss, and requires authoritative reconciliation before an
  intent is reported as applied. The typed `PipeWireApiTransport` now maps only
  graph-synchronized native API snapshots and typed volume, mute, and configured
  default-output calls into that contract. Its native library binding and
  packaged runtime evidence remain separate gates.
- `aqua_x86_64_audio_rehearsal_defconfig` resolves the exact package closure
  without changing the default image. The 2026-08-30 rehearsal verified
  PipeWire 1.2.8, WirePlumber 0.5.5, alsa-lib 1.2.13, eudev 3.2.14, Lua 5.4.8,
  and GLib 2.82.5 in Buildroot's generated `legal-info` manifest. Bluetooth,
  D-Bus, JACK, PulseAudio, GStreamer, V4L2, and FFmpeg stayed unselected. This
  satisfies dependency rehearsal, not release clearance or runtime acceptance.

## Acceptance Gates

Packaging is only the beginning of R4 audio work. Acceptance requires:

1. A rootfs contract proves exact package versions, configuration paths,
   disabled unneeded compatibility layers, user ownership, and no automatic
   root service.
2. A service lifecycle probe proves ordered start, ready, bounded restart after
   one forced failure, state reconciliation, clean stop, and retained recovery
   access.
3. QEMU uses a declared emulated audio device and proves device discovery,
   output playback to a captured sink, input from a controlled source, bounded
   mute and volume, default-route changes, unplug/error behavior, and service
   restart recovery. Logs must identify the virtual device and exact stack
   versions.
4. Settings and the top bar consume authoritative adapter state. Pointer and
   keyboard changes must be acknowledged by the service before the UI claims
   application; service failure must visibly degrade and recover.
5. Physical hardware support remains `Not tested` until the separate sanitized
   inventory and authorized hardware procedure prove the same matrix on the
   exact machine variant.

## Rejected Alternatives

- **ALSA only:** too low-level for desktop routing, per-application streams,
  hotplug policy, and a coherent client permission model.
- **PulseAudio as the primary server:** adds a separate legacy architecture
  when PipeWire provides the selected graph and compatibility can be enabled
  later only if required.
- **PipeWire without WirePlumber:** leaves device enablement, routing, linking,
  and permissions without the required policy owner.
- **A root-owned system-wide media graph:** conflicts with the unprivileged
  session and least-privilege R4 boundary.
- **Parsing `wpctl` in Settings:** makes a human-oriented diagnostic CLI an
  unstable production protocol and obscures service errors.
- **Enabling the versions in Buildroot 2024.02.12 immediately:** bypasses the
  supported-baseline, lifecycle, permission, and evidence prerequisites.

## Consequences

- Settings consumes the adapter's fail-closed state. A bare `/dev/snd`
  directory cannot enable controls, and saved preferences remain unapplied
  until a ready service snapshot with a valid output route acknowledges them.
- Aqua now has a bounded per-user media-service supervisor with ordered
  PipeWire/WirePlumber startup, reverse-order shutdown, finite readiness,
  restart, and degraded-state handling. Its packaged default remains disabled.
- The typed PipeWire/WirePlumber API transport core and exact Buildroot
  dependency/legal-info rehearsal are complete. The next audio implementation
  item is the native library binding behind that transport, followed by
  opt-in packaging and declared-device QEMU media evidence; command-output
  parsing remains prohibited.
- Aqua gains one documented audio stack for device discovery, output, input,
  mute, volume, routing, permissions, and restart recovery.
- Adding Bluetooth, PulseAudio compatibility, JACK compatibility, portals, or
  microphone consent expands the acceptance matrix and must not happen
  implicitly through package dependencies.

## Primary References

- [PipeWire session-manager responsibilities](https://docs.pipewire.org/page_session_manager.html)
- [PipeWire access-control model](https://docs.pipewire.org/page_access.html)
- [WirePlumber session-management design](https://pipewire.pages.freedesktop.org/wireplumber/design/understanding_session_management.html)
- [WirePlumber daemon startup on non-systemd systems](https://pipewire.pages.freedesktop.org/wireplumber/daemon/running.html)
- [Buildroot init and service integration](https://buildroot.org/downloads/manual/manual.html)
- [Buildroot supported release series](https://buildroot.org/download.html)
