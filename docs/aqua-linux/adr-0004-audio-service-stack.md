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
  default-output calls into that contract. Native control submission has a
  fixed three-attempt budget per authoritative graph generation. Exhaustion
  preserves the desired preference, blocks further submission, and exposes a
  degraded Settings state until a newer synchronized generation reopens the
  acknowledgement cycle. Native snapshot generations advance only when the
  canonical authoritative graph payload changes; polling an unchanged graph
  cannot silently reset the attempt budget. The packaged QEMU probe verifies
  three rejected submissions, a bridge-blocked fourth call, a real graph
  generation change, and final authoritative acknowledgement. The opt-in
  adapter now also binds volume and mute requests to the authoritative default
  output that received them. A route-generation change rejects the old request
  even when the fallback output coincidentally already has the desired value;
  that coincidence may satisfy the retained preference, but it cannot
  acknowledge a command sent to the removed output. The native snapshot bridge
  records default sink/source IDs only when their node media kinds match, so an
  output-only graph cannot expose a sink as `default_input`. The opt-in
  `aqua-audio-native` package now implements that typed boundary against
  WirePlumber 0.5 with a versioned, fixed-size C ABI, bounded waits, and strict
  Rust-side validation. The opt-in rootfs now packages and enables it without
  changing the default image; declared Intel HDA output behavior is covered by
  the QEMU acceptance path described below.
- `aqua_x86_64_audio_rehearsal_defconfig` resolves the exact package closure
  without changing the default image. The 2026-08-30 rehearsal verified
  PipeWire 1.2.8, WirePlumber 0.5.5, alsa-lib 1.2.13, eudev 3.2.14, Lua 5.4.8,
  and GLib 2.82.5 in Buildroot's generated `legal-info` manifest. Bluetooth,
  D-Bus, JACK, PulseAudio, GStreamer, V4L2, and FFmpeg stayed unselected. This
  satisfies dependency rehearsal, not release clearance or runtime acceptance.

## Acceptance Gates

Packaging is only the beginning of R4 audio work. Acceptance requires:

1. **Satisfied on 2026-08-30:** the opt-in rootfs contract records exact package
   versions, checks configuration and module paths, rejects unneeded
   compatibility daemons, proves the fixed session identity, and rejects an
   automatic root service. The default rootfs stays disabled and package-free.
2. **Satisfied for deterministic lifecycle fixtures:** the service probe proves
   ordered start, ready, bounded restart after one forced failure, state
   reporting, clean reverse-order stop, degradation, and retained graphical
   recovery behavior. Real media-device recovery remains part of gate 3.
3. **Partially satisfied on 2026-08-30:** the opt-in kernel fragment enables
   only the Intel HDA/generic codec path needed by QEMU, while the default
   kernel stays sound-free. QEMU declares `ich9-intel-hda` plus `hda-duplex`;
   PipeWire/WirePlumber discovers one sink and one source, volume and mute are
   exercised, 48,000 stereo S16LE frames are played into a non-silent 48 kHz
   WAV capture, and a forced WirePlumber failure recovers within the supervisor
   budget. A separate run uses QEMU's timer-backed `none` ADC and requires the
   unprivileged session to capture 4,800 stereo S16LE frames with an exact
   zero-PCM peak. This proves controlled stream establishment and sample
   delivery without host microphone access. A third run declares two separate
   Intel HDA controllers with output-only codecs and independent WAV backends,
   requires two authoritative output nodes, changes the configured default
   through the native WirePlumber API, waits for the effective route to
   acknowledge that node, and proves non-silent playback in both captures.
   A fourth run selects PCI output 05.0 as the authoritative default, proves
   playback, removes its HDA controller through QMP, requires the matching
   asynchronous deletion event and a one-card ALSA topology, then requires the
   native snapshot to expose the sole remaining output as default and proves
   non-silent fallback playback. A separate inverse profile first proves PCI
   output 04.0 is the authoritative default, removes the non-default PCI 05.0
   controller, requires the same QMP and one-card topology evidence, and then
   proves 04.0 remains authoritative with non-silent playback before and after
   removal. A further active-stream variant removes 05.0 after the retained
   default client has written 480 frames; that same client must complete all
   48,000 frames without an interruption marker while the authoritative default
   remains 04.0. The selected-route counterpart starts on authoritative PCI
   05.0, removes that controller after 480 frames, and observes the native graph
   rather than trusting the ALSA compatibility write result. It must report a
   dedicated `route-loss` interruption with no playback-success marker, then a
   new client must complete 48,000 frames on fallback PCI 04.0. This distinction
   is required because the compatibility PCM accepted buffered writes after the
   removed route disappeared and therefore could otherwise report false
   completion. A fifth run attaches a private peer-to-peer
   QEMU D-Bus audio listener and injects a bounded 1 kHz bipolar square wave
   without host microphone access. The guest captures exactly 4,800 stereo
   S16LE frames through HDA, ALSA, and PipeWire; all 9,600 samples are non-zero,
   the measured peak is 4,096, and positive/negative counts are balanced.
   A sixth run makes the same host listener serve exactly 9,600 bytes of
   fixed-amplitude positive PCM and then reject every later D-Bus read. QEMU's
   retained input buffer cannot create a false success because the guest
   requires bipolar data: the probe reports `invalid-injected-signal` with
   9,600 positive and zero negative samples, while PipeWire, WirePlumber, and
   the recovery shell remain responsive. An active device-loss counterpart
   first proves a full 4,800-frame bipolar capture, starts a second capture,
   and removes the sole duplex HDA controller after its 480-frame checkpoint.
   The native graph must lose its default input and converge to zero inputs;
   the client must report `input-route-loss` without false completion, while a
   new capture is blocked and the media services and recovery shell remain
   responsive. A seventh run writes 480 playback
   frames, marks the stream active, and then kills its owning PipeWire process.
   The client reports `Broken pipe` and exits with the dedicated interrupted
   status instead of claiming completion; the supervisor performs its bounded
   ordered restart and a new 48,000-frame playback produces a non-silent WAV
   while recovery stays responsive. An eighth run terminates four successive
   real PipeWire processes under the opt-in profile's three-restart budget.
   It requires exactly three restart markers followed by `state=degraded`,
   `attempts=4`, and `restarts=3`; both service PIDs and the PipeWire socket
   must be gone, new playback and `wpctl` access must fail closed, and the
   recovery shell must remain responsive. A ninth run opens a controlled
   zero-PCM capture, publishes an active marker after 480 frames, and then
   terminates its owning PipeWire process. The client must report `Broken pipe`
   with a dedicated interrupted status and no false capture success; after the
   ordered restart, a new client must capture exactly 4,800 zero-PCM frames.
   The profile never requests a host microphone and keeps recovery responsive.
   A tenth run first proves acknowledged volume/mute controls, then holds the
   supervisor for a bounded interval while its PipeWire process is terminated.
   With the socket absent, the native control probe must fail at `open` and
   emit no success marker. Releasing the supervisor must produce a new
   PipeWire PID and a second acknowledged control cycle. This proves an
   unavailable graph cannot create a false control acknowledgement. An
   additional two-output run prepares PCI 04.0 at the desired volume, selects
   PCI 05.0 at a different volume, leaves the adapter request pending, and then
   removes 05.0 through acknowledged QMP deletion. The fallback's matching
   value must not confirm the old target-bound request; reconciliation must
   report it rejected or lost, avoid an unnecessary resubmission, and retain
   running services plus recovery-shell access. Other media error behavior
   remains open.
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
- The typed PipeWire/WirePlumber transport and native library binding are
  complete as an opt-in Buildroot package. The bridge uses the WirePlumber
  object manager, default-nodes API, mixer API, and synchronized acknowledgments
  without command-output parsing. The audio-only rootfs overlay enables the
  per-user supervisor and its final rootfs artifact passes the contract. An
  audio-only kernel fragment, ALSA-to-PipeWire default, and bounded test probe
  now support declared Intel HDA QEMU output and controlled zero-PCM input
  acceptance without changing the sound-free default image. Two-controller
  QEMU acceptance additionally proves acknowledged multi-device default-output
  switching and playback on both routes. Selected virtual PCI output removal
  additionally proves acknowledged device deletion, authoritative fallback,
  and resumed playback. A private QEMU D-Bus input listener additionally proves
   deterministic non-silent capture through the declared HDA device without
   requesting a host microphone. The same bounded listener now proves a
   mid-source read failure cannot be reported as valid bipolar input and does
   not hang the service graph or recovery shell. Active playback now also
   fails explicitly on PipeWire loss, followed by bounded service recovery and
   verified non-silent playback from a new client. Repeated real PipeWire loss
   now additionally proves the configured restart budget is finite, leaves no
   media processes or socket after degradation, and blocks new playback while
   recovery remains available. Active controlled capture now also fails
   explicitly on PipeWire loss and succeeds from a new client after ordered
   recovery without host-microphone access. Native volume/mute controls now
   additionally fail closed during a bounded real graph outage and regain
   acknowledgement only after ordered recovery. Non-default output removal is
   also bounded: the native topology proves the surviving selected route remains
   unchanged and playable rather than entering an unnecessary fallback
   transition. The same boundary now holds during an active 48,000-frame stream
   without hidden ALSA recovery or client replacement. Conversely, removal of
   the actively selected route is detected from the authoritative native
   topology at the 480-frame checkpoint, aborts explicitly without false
   completion, and permits only a new client to prove full playback on the
   fallback output. Controlled active input-device removal is also bounded:
   authoritative native topology, rather than PCM behavior alone, aborts the
   stream at 480 frames, exposes zero available inputs, and blocks a new
   capture without stopping the media services. This remains virtual-device
   evidence without host microphone or physical hotplug claims. Other error
   evidence now also includes the production adapter's packaged-QEMU submission
   budget: three failed calls retain one graph generation, the fourth is blocked,
   and only a real graph change permits the acknowledged recovery call. A
   separate selected-route loss run binds a pending volume request to PCI 05.0,
   removes that output, and proves PCI 04.0's already-matching value cannot
   falsely acknowledge the removed target; the request becomes lost while the
   retained preference is already satisfied without resubmission. Physical
   hardware behavior and other error evidence remain open.
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
