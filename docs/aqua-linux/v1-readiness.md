# Aqua Linux V1 Readiness Gates

Status date: 2026-08-31

Aqua Linux has a real Buildroot image, a custom Smithay compositor, first-party
desktop surfaces, a guarded installer, and repeatable packaged-QEMU evidence.
That makes it a packaged-QEMU-proven prototype. It is not yet hardware-proven,
release-ready, or suitable for daily use.

This document separates two measurements that answer different questions:

- **Roadmap implementation progress** is the arithmetic milestone percentage in
  `progress.json`. It records completion of the scoped M0-M12 engineering work.
- **Product readiness** is determined by the mandatory gates below. A high
  milestone percentage cannot substitute for missing performance, security,
  service, compatibility, recovery, or hardware evidence.

## Architecture Decision

Buildroot and a custom Wayland compositor remain the correct architecture for
the current product: a focused, controlled OS image with a small first-party
desktop and a deliberately narrow app model. Buildroot is not itself a desktop
platform; Aqua must own the session, service integration, update lifecycle,
security boundaries, compatibility surface, and recovery behavior that a
general-purpose distribution normally supplies.

Reconsider the base through a new architecture decision record only if the
product goal changes to an open-ended, user-managed package ecosystem or broad
binary compatibility. Do not change the base merely to fill an individual
service or protocol gap.

## Evidence Levels

| Level | Meaning |
| --- | --- |
| Contract-proven | A bounded design, state, or safety contract is checked without running the packaged OS path. |
| Host-proven | Deterministic tests pass on the development host. This is not runtime or hardware evidence. |
| Packaged-QEMU-proven | The packaged Buildroot image exercises the complete feature in a declared QEMU configuration and returns safely to recovery. |
| Hardware-proven | The same user-visible workflow passes on an identified physical target with recorded driver, firmware, failure, and recovery evidence. |
| User-ready | The feature meets its compatibility, security, accessibility, performance, recovery, and documentation acceptance criteria as part of a release candidate. |

Evidence does not automatically move upward. In particular, a QEMU pass is not
physical hardware evidence, and a visual capture is not performance, input, or
accessibility evidence.

## Mandatory Gates

### R1: Component And Experience Closure

Scope: finish M12 and remove prototype-only UI paths.

Pass criteria:

- The shared component catalog covers every shell, installer, and first-party
  application control named by the interface contract.
- Themes, supported scales and viewports, localization expansion, keyboard and
  pointer states, reduced motion, focus indication, and disabled/error states
  have deterministic regression fixtures.
- Login or first-run, notifications, system status, session controls, failure
  states, and recovery entry use real runtime data rather than illustrative
  values.
- Screen-specific copies are removed when a shared primitive exists.

### R2: Presentation Performance And Frame Correctness

Scope: prove that the live desktop presentation path is production-shaped, not
only visually correct.

Pass criteria:

- Normal live composition renders into GBM/KMS scanout buffers without a CPU
  framebuffer copy or full-frame GPU readback. Diagnostic readback remains
  isolated from production presentation and is never required per frame.
- Page flips and frame callbacks drive scheduling; vblank completion, damage,
  missed-frame handling, and idle suppression are bounded and observable.
- No hidden or settled animation causes an unbounded repaint timer.
- Representative idle, window interaction, animation, and multi-client loads
  have recorded frame-time, input-to-present, CPU, memory, and dropped-frame
  evidence on QEMU and on the selected physical target.
- Performance budgets are recorded before release-candidate testing and are
  enforced by repeatable checks. QEMU TCG timings are tracked as regression
  evidence but are not used as a physical responsiveness claim.

The first R2 baseline contract now lives in the renderer-independent
`aqua-compositor` library. A bounded report requires exactly one production
sample for idle, window interaction, animation, and multi-client workloads;
checks GBM/KMS path identity, frame/page-flip accounting, client callbacks and
damage for the multi-client workload, settled-idle suppression, explicit timing
and resource budgets, and dropped frames; and rejects CPU framebuffer copies or full-frame readback in
production samples. Diagnostic readback has a separate evidence record and
must show that it neither reads nor blocks a production frame. Deterministic
host fixtures prove fail-closed evaluation, but `supports_release_claim`
remains false. Live QEMU telemetry and a fixed QEMU regression budget now exist;
soak evidence and physical-target measurements and budgets are still required
before R2 can pass.

The model now also owns a bounded `PresentationTelemetry` collector. It accepts
ordered frame requests, page flips or explicit drops, callbacks, damage,
input-to-present timing, readback, CPU-copy, settled-idle, and final resource
measurements; rejects presentation without an outstanding request, incomplete
frame accounting, zero timing/window values, and more than 100,000 events; and
builds the immutable sample consumed by the report. `LegacyCpuCopy` is a
separate path and cannot satisfy production acceptance.

The live DRM-Wayland frame loop now has an opt-in event bridge controlled by
`AQUA_R2_PRESENTATION_TELEMETRY=true` and an explicit
`AQUA_R2_PRESENTATION_WORKLOAD`. It records every initial and repaint request
before KMS submission, measures submit-to-page-flip completion, and identifies
the native GBM/KMS and virtio CPU-copy fallback paths without conflating them.
The same stream baselines aggregate Smithay damage and frame-callback counters,
records only monotonic deltas at DRM event boundaries, rejects counter
regression, and performs a final synchronization after the last presentation.
This avoids multiplying session-wide counters by the number of mapped surfaces
and captures callbacks emitted after the last page flip. Libinput keyboard,
pointer-motion, and pointer-button events contribute their real monotonic
microsecond timestamps; the bridge retains the earliest unpresented event and
measures it at the next real page-flip boundary. The snapshot exposes both the
bounded sample count and maximum input-to-present latency so a missing sample
cannot be mistaken for zero latency. During an idle workload, the same live
loop counts quiet dispatch intervals only when there is no input, client or
shell state change, process event, session action, motion, or repaint request.
Real activity resets the settled state. Once settled, a repaint without an
external cause is counted as an idle violation and an active motion timer is
reported separately; the normal bounded event wait is not classified as a
repaint. The observation window uses monotonic elapsed time, CPU cost uses the
process CPU clock, and bounded Linux `VmRSS` samples retain the highest resident
set observed during the workload so a transient increase is not hidden by a
lower final reading. CPU time and peak RSS growth are attached to the same event
snapshot. The bounded event snapshot is emitted only after clean CRTC
restoration and is explicitly marked
`r2_presentation_acceptance_complete=false`. This marker remains false because
the QEMU regression profile is not a physical responsiveness or release claim.

Each live run now encloses its fields in a versioned `v1` serial-record boundary.
The host-side `scripts/check-r2-presentation-log.py` validator rejects missing,
duplicate, unknown, malformed, unbounded, legacy-path, or incomplete records;
requires exactly one QEMU GBM/KMS record for every workload; requires real input
and multiple page flips for shell interaction and animation; requires client
callbacks and damage for multi-client; and enforces idle, timing, and resource
evidence. The same validator requires exactly one separately framed QEMU
diagnostic-readback record. That record must identify the offscreen diagnostic
path, account for bounded captures and readbacks, show zero production frames
read or blocked, and show that neither KMS nor display output started. It then
emits `r2_diagnostic_isolation_recorded=true` and enforces the named
`qemu-tcg-bochs-v1` regression profile. The profile limits maximum page-flip
wait to 50,000 us, input-to-present latency to 60,000,000 us, process CPU time
to 180,000,000 us, peak RSS growth to 163,840 KiB, and dropped frames to zero.
Its deterministic self-test rejects each over-budget metric. This profile is
QEMU-only, and `r2_physical_budget_selected=false` remains explicit.

The bounded `scripts/check-r2-presentation-qemu.sh` runner now defines one
recovery-safe QEMU boot with four isolated compositor sessions. Idle receives no
injected activity; window interaction uses real virtio keyboard input;
animation combines the frame-driven motion scenario with a real input sample;
and multi-client launches packaged Files and Settings through the launcher. The
runner refuses a rootfs older than the compositor source, uses snapshot disk
mode, returns through recovery between workloads, and then executes the existing
deterministic two-frame offscreen GLES probe under explicit QEMU evidence gates.
The combined validator records diagnostic isolation and enforces the selected
QEMU regression profile. The runner contract alone is not runtime evidence.

`scripts/check-r2-presentation-repeated-qemu.sh` now makes that review input
repeatable. It accepts only three through ten independent boots, refuses to
overwrite an existing evidence directory, preserves every serial log and
single-run report, and revalidates each complete workload plus diagnostic set.
The resulting versioned review record reports workload-specific observed maxima
for frame time, input-to-present latency, CPU time, and memory growth. Three
independent packaged Bochs boots completed on 2026-08-30, producing 12 workload
records and three isolated diagnostic records. The reviewed maxima were 22,681
us page-flip wait, 46,522,278 us input-to-present latency, 147,346,897 us CPU
time, and 130,476 KiB peak RSS growth. These observations informed the explicit
`qemu-tcg-bochs-v1` limits above with bounded headroom; every original log also
passes those limits independently. The review records
`r2_review_budget_selected=true` and
`r2_review_physical_budget_selected=false`.

The separate `scripts/check-r2-presentation-soak-qemu.sh` runner keeps one
production GBM/KMS compositor process active for at least five minutes. It maps
Files and Settings together, sends ten bounded real virtio-input cycles, and
requires at least five distinct input-to-present samples after normal repaint
coalescing. The `qemu-tcg-bochs-soak-v1` profile retains the 50,000 us
page-flip, 60,000,000 us input-latency, 163,840 KiB peak RSS-growth, and
zero-drop limits while allowing at most 720,000,000 us process CPU time for the
two-vCPU five-minute window. Its crash budget is zero. The runner stops through
the compositor's graceful-stop file, requires client reap, CRTC restoration,
GBM release, recovery return, and isolated diagnostic readback, and refuses to
overwrite its raw log and report directory.

The first packaged soak completed on 2026-08-30. It observed 338,040 ms, 42
dispatched keyboard events, nine presented frames, five distinct
input-to-present samples, 9,879 us maximum
page-flip wait, 39,054,481 us maximum input-to-present latency, 128,457,628 us
CPU time, and 130,252 KiB peak RSS growth. It recorded zero crashes, dropped
frames, production readbacks, and CPU framebuffer copies, then returned cleanly
to recovery. This is an initial QEMU regression soak, not release-qualification
duration, physical responsiveness, or hardware stability evidence.

The separate `scripts/check-r2-presentation-qualification-qemu.sh` profile
raises the minimum observation window to 900,000 ms. Fifteen launcher
open/dismiss cycles must each wait for their own DRM repaint acknowledgement;
the validator requires at least 15 input-to-present samples, 45 dispatched
keyboard events, zero crashes and dropped frames, the unchanged 50,000 us
page-flip, 60,000,000 us input-latency, and 163,840 KiB RSS-growth limits, plus
a duration-scaled 2,160,000,000 us two-vCPU CPU ceiling. The first accepted run
on 2026-08-30 observed 1,344,436 ms, 94 keyboard events, 71 presented frames,
38 input samples, 8,131 us maximum page-flip wait, 44,045,721 us maximum input
latency, 1,232,394,556 us CPU time, and 134,200 KiB peak RSS growth. Client
reap, CRTC restoration, GBM release, recovery return, and isolated diagnostic
readback passed. The record explicitly retains `physical_evidence=false` and
`release_ready=false`.

`scripts/check-r2-presentation-repeated-qualification-qemu.sh` now accepts only
three through ten fresh qualification boots, refuses to overwrite evidence, and
revalidates every independent serial log before emitting a bounded aggregate
review. Three independent cold boots completed on 2026-08-30 with three
isolated diagnostic records and 213 presented frames. Every run dispatched 94
keyboard events; their minimum observation window was 1,319,699 ms and minimum
input-sample count was 38. The aggregate maxima were 8,505 us page-flip wait,
44,045,721 us input latency, 1,232,394,556 us CPU time, and 134,348 KiB RSS
growth. The review explicitly retains `physical_evidence=false` and
`release_ready=false`; repeated QEMU qualification is not physical stability or
release evidence.

The packaged kernel now enables Bochs DRM and the macOS QEMU runner defaults to
`bochs-display`. This reaches Aqua's `production-gbm-kms` path with GLES
software rendering and direct GBM dma-buf scanout, while emitting zero
production full-frame readbacks and CPU framebuffer copies. The virtio target
remains a separately identified `legacy-cpu-copy` fallback. Repeated collection,
QEMU budget review, the initial bounded soak, and three independently validated
acknowledgement-gated qualification runs are complete for these profiles. A
separately selected physical-target budget remains downstream; QEMU TCG
measurements are not physical responsiveness evidence.

### R3: Wayland Compatibility And Display Behavior

Scope: support the protocol set required for a coherent desktop and its
declared application model.

Pass criteria:

- Existing compositor, shared-memory, seat, output, frame-callback, and
  `xdg-shell` paths retain packaged-QEMU interoperability tests.
- Clipboard and primary data transfer, drag and drop, popups, subsurfaces,
  activation, and client lifecycle behavior pass independent-client tests.
- Linux dma-buf import and the required synchronization path are implemented
  for accelerated clients, or the v1 app model explicitly excludes them.
- Output discovery, hotplug, mode selection, logical coordinates, fractional
  scaling, and viewport behavior are correct for the supported display matrix.
- Text input and input-method behavior support the declared keyboard and locale
  matrix.
- Screenshot, screencopy, activation, and privileged shell protocols are
  unavailable to arbitrary clients unless an explicit authorization policy
  permits them.

Current R3 evidence establishes the standard data-transfer boundary. The
compositor publishes only the standard clipboard, primary-selection, and
data-device managers, makes their focus follow Aqua Seat keyboard focus,
rejects ownership changes from an unfocused client, and transfers existing
offers when focus moves between two independent clients. The Linux Smithay
probe and packaged-rootfs contract also
verify UTF-8 MIME negotiation, rejection of an unsupported type, exact byte
transfer through protocol file descriptors, owner-disconnect cleanup, a
4096-byte and two-second probe bound, and the absence of compositor payload
buffering. No data-control manager is advertised. The separate two-client
drag-and-drop probe rejects a start without an implicit
pointer grab, routes enter/drop only to the pointer-focused target without
changing keyboard focus, negotiates UTF-8 text and the copy action, transfers
an exact bounded payload directly, finishes an accepted drop, and cancels a
rejected drop without target delivery. The separate output-matrix probe
publishes `wl_output` v4 plus xdg-output v3, fractional-scale v1, and
viewporter v1 to two independent clients. Both clients first discover a
1280x800 output and then receive three globals added while they remain
connected. The resulting four-output matrix verifies current/preferred 60 Hz
modes, exact logical coordinates and sizes, declared scales 1.0, 1.25, 1.5,
and 2.0, integer fallbacks 1, 2, 2, and 2, and normal, 90, 180, and 270 degree
transforms. It also verifies a 1.25 preference encoded as 150/120, committed
viewport crop and destination state, and fourth-output global removal while
the original output remains usable. Hardware-driven connector hotplug and
the broader application matrix remain open. A separate two-client lifecycle
probe assigns one connection an `xdg_popup` and the other a `wl_subsurface`.
It verifies popup parent binding, exact initial and repositioned geometry, two
configure acknowledgements, reposition token delivery, synchronized and
desynchronized subsurface commits at the declared parent-relative position,
child-role destruction, and continued independence of both parent surfaces.
The packaged rootfs runs the same feature-enabled probe. The v1 application model is explicitly
limited to first-party and independently tested `wl_shm` ARGB8888 clients.
Accelerated client buffers are outside that model: the compositor does not
advertise `zwp_linux_dmabuf_v1`, `wp_linux_drm_syncobj_manager_v1`, or
`zwp_linux_explicit_synchronization_v1`. A two-client Linux registry probe and
the packaged-rootfs contract verify that boundary while retaining
`wl_buffer.release` and `wl_surface.frame` as the shared-memory lifecycle and
presentation synchronization scope. Compositor-owned GBM dma-buf scanout is a
separate output implementation detail and does not expand the client contract.
The packaged-rootfs independent-application matrix now launches the upstream
Weston 14.0.1 `weston-simple-shm`, `weston-simple-damage`, and
`weston-simple-touch`, plus `weston-terminal` as four external processes and
an Aqua-authored GLFW 3.4 native-Wayland client against one Aqua compositor
session. It requires all five app IDs; exact and distinct 250x250, 320x200,
600x500, 726x443, and 400x240 `wl_shm` buffers;
`wl_surface.damage_buffer` and frame
callback progress; and a real `wl_touch` down/motion/up sequence that paints
exact red points at 120,140 and 180,200 through two new client commits. The
terminal opens a real PTY backed by the packaged shell and must redraw after
the compositor delivers `echo aqua` plus Enter through `wl_keyboard`. The GLFW
client uses no OpenGL API and must redraw from a second nonzero-offset shm
buffer after its GLFW key callback receives `G`; this verifies offset-aware
buffer sampling. All five clients receive independent compositor close
delivery, exit cleanly, and leave zero surfaces. Four terminal frame PNGs are retained only as fixture assets;
the image neither packages nor starts the Weston compositor or its shells.
This is protocol-level touch evidence, not physical touchscreen evidence.
General toolkit interoperability beyond the bounded Weston and GLFW evidence
remains part of R3 acceptance. The
three-client text-input probe separately publishes text-input v3 to normal
clients while hiding input-method v2 from them and exposing it only to an
authorized client. It proves keyboard-focus activation, stale-client
rejection, bounded surrounding/content/cursor state, synchronized serials,
Turkish UTF-8 preedit and commit delivery, deletion, focus handoff, and popup
repositioning. The declared v1 matrix is now bounded to the installer's
`tr_TR.UTF-8`, `en_US.UTF-8`, and `de_DE.UTF-8` locales crossed with Turkish Q
(`trq`), Turkish F (`trf`), and US (`us`) layouts. A separate Linux probe
creates an Aqua Seat for each layout and requires two independent clients to
receive the real `wl_keyboard` XKB keymap and the declared 400 ms/25 Hz repeat
information. Both clients recompile every delivered map and resolve a
layout-distinguishing UTF-8 key (`ı`, `f`, or `q`). The keymaps also reserve
the Menu key as `Multi_key`. Across all nine locale/layout combinations, both
clients feed a bounded Compose sequence through libxkbcommon and obtain `é`;
the Turkish Q and Turkish F maps additionally resolve their real
Shift+Level3 dead-acute key and pass six locale/layout dead-key cases. Invalid
Compose input cancels without producing text in every declared locale. The
same bounded table is packaged at `/usr/share/aqua/compose/Compose` and exported
to graphical clients through `XCOMPOSEFILE`; the packaged-rootfs contract runs
the feature-enabled probe and verifies that session binding. Physical keyboard
behavior and broader toolkit/application coverage remain open and are not
implied by these bounded matrices.

The arbitrary-client privileged-protocol boundary is now covered by a separate
three-client Linux registry probe and the packaged-rootfs contract. Two normal
clients and the one narrowly authorized input-method client must all retain
the eleven baseline desktop globals. Only the authorized client may see
`zwp_input_method_manager_v2`; that authorization does not reveal any of the
sixteen audited screenshot, screencopy/export, activation, privileged-shell,
virtual-input, foreign-toplevel, output-management, gamma/power, DRM-lease, or
session-lock globals. Aqua currently publishes none of those privileged
interfaces, so there is no arbitrary-client capture, activation, or shell
control path and no broader authorization claim.

The v1 application boundary is publicly defined in
[application-compatibility.md](application-compatibility.md). Both Buildroot
profiles explicitly disable Xorg and XWayland; the packaged contract rejects
their server binaries and modules, an X11 socket directory, and any `DISPLAY`
assignment in Aqua session environments. `/usr/share/X11/xkb` remains only as
libxkbcommon keyboard data for native Wayland clients. X11-only applications
are unsupported, while broader Wayland toolkit coverage remains open.

### R4: Unprivileged Session And Core System Services

Scope: make the desktop a user session rather than a collection of privileged
prototype processes.

Pass criteria:

- The graphical session, shell, and applications run as an unprivileged user.
  Root-only operations cross a narrow authenticated broker with an auditable
  allowlist; the installer keeps its existing independent execution gates.
- Login or first-run creates and enters a persistent user session with correct
  ownership, runtime directories, environment, logout, restart, and recovery.
- Network configuration covers link state, Wi-Fi association where supported,
  DHCP, DNS, reconnect, offline/error states, and Settings integration.
- Audio covers device discovery, output, input, mute, volume, routing, and
  restart recovery through one documented service stack.
- Time, timezone, locale, storage, removable media, battery, power action,
  suspend/resume, and Bluetooth behavior are integrated where the target
  hardware requires them.
- Service failure cannot silently hang the shell; bounded restart, user-visible
  degradation, logs, and recovery paths are tested.

The concrete network stack is selected by
[ADR 0005](adr-0005-network-service-stack.md): eudev plus Linux provide device
and link state, BusyBox `udhcpc` remains the initial Ethernet DHCP client, and
`wpa_supplicant` is reserved for gated Wi-Fi targets. The new unprivileged
adapter reports bounded typed interface, default-route, DNS, and
offline/configuring/online/degraded state without spawning management commands.
Settings consumes that state but remains read-only; the root-owned supervisor,
authenticated broker, opt-in Wi-Fi package rehearsal, reconnect behavior, and
QEMU and physical evidence remain open. The root-owned DHCP supervisor is now
packaged with a finite readiness timeout, three-restart budget, lease-loss
grace period, atomic non-secret state, a checked hook around Buildroot's default
lease script, and deterministic start/failure/stop fixtures. Its fail-closed
configuration keeps it disabled. Aqua's custom `rcS` does not invoke the
generated Buildroot `S40network`, so default boot has no DHCP policy owner. The
new boot gate requires both `aqua.boot_network=1` and a separate QEMU-only
profile before dispatching the supervisor; DHCP, DNS, renewal, and reconnect
still require QEMU evidence.
[ADR 0004](adr-0004-audio-service-stack.md) selects ALSA,
PipeWire, WirePlumber, and eudev for audio. Buildroot 2025.02.17 now supplies
the supported LTS baseline. The locked `aqua` UID/GID 1000 session,
user-owned `/run/user/1000`, and explicit `video`, `audio`, and `input` groups
now satisfy the identity and base device-permission prerequisite. The packaged
per-user media supervisor now owns finite readiness, PipeWire-before-WirePlumber
startup, bounded restart, reverse-order shutdown, and degraded state. It stays
disabled while the packages are absent. The `aqua-service-adapters` crate now
provides bounded typed device and route state, monotonic reconciliation,
deferred desired volume and mute, and fail-closed unavailable/degraded behavior.
Its typed PipeWire transport maps only synchronized snapshots and typed native
volume, mute, and configured-default operations; Settings can drive this backend
without treating submission as acknowledgement. Its UI model and renderer expose
unavailable, starting, degraded, applying, and applied states, keep the last
authoritative volume visible while an intent is pending, and enable slider/mute
input only after authoritative reconciliation reaches applied. A
control-submission guard now permits at most three failed native submissions
per authoritative graph generation. Exhaustion retains the desired preference,
keeps the last authoritative value visible, disables Settings input, and waits
for a newer synchronized generation before retrying. Deterministic adapter and
shell tests cover failure, blocking, recovery, and acknowledgement. The packaged
QEMU probe now exercises the production native bridge as well: three rejected
submissions share one stable generation, the fourth is blocked before reaching
the bridge, and a real authoritative graph change advances the generation and
reopens submission through final acknowledgement. This remains virtual runtime
evidence rather than physical-hardware evidence. A separate
Buildroot rehearsal profile has resolved the exact package delta, packages a versioned bounded
WirePlumber 0.5 native bridge, and passed `legal-info` while the default image
stayed unchanged. Its second, audio-only rootfs overlay now explicitly enables
the per-user supervisor, records exact stack versions, and installs a checker
that proves required configuration, modules, identity, compatibility-layer
exclusions, and the absence of an automatic root service. The complete final
rootfs artifact passes that checker. The opt-in Intel HDA QEMU baseline now
proves sink/source-node discovery, output playback into a non-silent 48 kHz
capture, volume/mute, bounded WirePlumber restart recovery, and a separate
controlled 4,800-frame zero-PCM input stream while the default kernel and
rootfs stay sound-free. The input profile avoids host microphone access and
proves deterministic stream delivery, not acoustic quality. Non-silent input
is now separately proven by a private QEMU D-Bus `AudioInListener` that serves
a fixed 1 kHz bipolar square wave. The guest captures exactly 4,800 stereo
S16LE frames through HDA, ALSA, and PipeWire with a 4,096 peak, all 9,600
samples non-zero, and balanced polarity; the host independently verifies the
declared injector format and byte count. This remains virtual transport
evidence and does not claim a physical microphone. A bounded failure variant
serves exactly 9,600 bytes of one-polarity PCM before rejecting later D-Bus
reads. The guest reports `invalid-injected-signal` rather than a false bipolar
success, while the media graph and recovery shell remain responsive. This
covers one virtual input-source failure boundary, not physical input loss. A
separate
two-controller QEMU profile now proves two authoritative output nodes,
configured-default submission through the native API, effective-route
acknowledgement, and non-silent playback captures on both routes. A bounded QMP
profile then selects the second virtual PCI output as default, removes it,
requires the asynchronous deletion event and one-device ALSA topology, observes
one authoritative fallback default through the native adapter, and proves
non-silent playback resumes on the remaining output. A complementary profile
removes the non-default PCI output instead, proves the authoritative PCI 04.0
route does not change while the topology shrinks, and captures non-silent
playback on that same route before and after removal. Its active-stream variant
removes the non-default controller after 480 frames and requires the same
default-route client to complete 48,000 frames without interruption or hidden
PCM recovery. Its selected-route counterpart removes authoritative PCI 05.0
after the same checkpoint, detects the resulting default change through the
native graph because buffered PCM writes are not authoritative, requires an
explicit `route-loss` interruption with no false completion, and proves a new
client can deliver 48,000 non-silent frames on fallback PCI 04.0. A separate active-stream
profile kills PipeWire after the client has written 480 frames, requires an
explicit `Broken pipe` interruption with no false completion, observes the
supervisor's second ordered service attempt, and proves a new 48,000-frame
playback produces non-silent output while recovery remains live. A separate
active policy-service profile terminates WirePlumber at the same 480-frame
checkpoint, requires the client to abort without false completion, proves the
supervisor retires the old PipeWire and WirePlumber processes before starting
attempt 2, and requires a new non-silent 48,000-frame playback. A separate
real-service exhaustion profile kills four successive PipeWire processes,
requires exactly three restart transitions before the configured budget leaves
the graph degraded, confirms process/socket cleanup, rejects new playback and
`wpctl` access, and preserves the recovery shell. Its policy-service counterpart
kills four successive WirePlumber processes, requires a new complete media pair
after each permitted restart, then proves the same degraded counters, cleanup,
playback rejection, and recovery-shell availability with
`failed_service=wireplumber`. A controlled-input exhaustion profile separately
proves one exact 4,800-frame zero-PCM capture before four PipeWire losses, then
requires capture open to fail without a success marker after the cleaned
degraded state, with no host-microphone access. Its policy-service counterpart
proves the same capture precondition before four WirePlumber losses, requires
three complete media-pair renewals, and rejects a new capture after the cleaned
`failed_service=wireplumber` degraded state without host-microphone access. A
separate controlled active-capture profile kills PipeWire after
480 zero-PCM frames, requires an explicit interrupted result without false
capture success, observes ordered recovery, and captures a new exact 4,800
frames without host-microphone access. Its policy-service counterpart kills
WirePlumber at the same active checkpoint, requires the capture client to fail
explicitly, proves both old media processes retire before attempt 2/restart 1,
and captures a new exact 4,800 zero-PCM frames on the replacement graph. A
distinct device-loss profile validates
one complete D-Bus-injected bipolar capture, removes the sole duplex HDA
controller during a second client's 480-frame checkpoint, requires native
topology to report zero inputs and explicit `input-route-loss` without false
completion, and blocks new capture while services remain responsive. It uses no
host microphone and proves no physical-device behavior. Other media error
matrices remain open.
Native volume/mute controls are now separately proven to reject acknowledgement
while the real PipeWire socket is absent and to succeed again only after the
supervisor restores a new authoritative graph. The complementary policy-loss
profile terminates WirePlumber, proves its full-stack response retires both old
media processes, rejects control acknowledgement during that bounded outage,
and succeeds again only on the replacement attempt 2/restart 1 graph. A
restart-budget control profile separately proves one healthy acknowledgement
before four PipeWire losses, then rejects control open without false
acknowledgement after the cleaned attempt 4/restart 3 degraded state. Its
WirePlumber counterpart proves the same healthy precondition, three complete
media-pair renewals, and fail-closed control rejection after the fourth policy
loss reaches `failed_service=wireplumber` degradation. The packaged production
adapter probe additionally proves the per-generation submission budget and its
generation-gated recovery against the real native graph. A distinct two-output
profile binds a pending volume request to selected PCI 05.0 while fallback PCI
04.0 already has the desired value, removes 05.0 through acknowledged QMP, and
proves the fallback coincidence cannot falsely acknowledge the removed target.
The old request becomes rejected or lost, the preference remains satisfied
without resubmission, and services plus recovery stay available. A symmetric
mute profile prepares the fallback muted, binds a pending mute to the unmuted
selected output, removes that output, and requires the same lost-request and
no-resubmission result without false acknowledgement. Other control and UI
failure matrices remain open. Default-image enablement remains a separate gate.
Buildroot availability or these bounded QEMU profiles are not a complete audio
integration decision.

### R5: Accessibility, Internationalization, And Input

Scope: make core workflows operable beyond the current visual pointer path.

Pass criteria:

- Every required workflow is keyboard operable with deterministic focus order,
  visible focus, escape behavior, and no focus trap.
- Components expose roles, names, values, states, and actions through a chosen
  accessibility architecture; screen-reader and magnification boundaries are
  documented and tested.
- Text scaling, high-contrast behavior, reduced motion, and non-color state
  cues preserve critical content and actions.
- Locale, Unicode, bidirectional text, Turkish casing, keyboard layouts, input
  methods, date/time, number formatting, and localization expansion pass the
  declared language matrix.
- Pointer, touchpad, key repeat, shortcuts, compose/dead keys, and hotplug have
  explicit acceptance coverage on supported targets.

### R6: Updates, Supply Chain, Security, And Recovery

Scope: maintain an installed system safely after first boot.

Pass criteria:

- The image and update artifacts are signed and verified before activation;
  interrupted, corrupt, incompatible, and rollback attempts fail safely.
- The update strategy defines atomicity, rollback, boot-success confirmation,
  retained recovery state, version compatibility, and user-visible status.
- A supported application installation/update model is declared. Absence of a
  general package manager must not leave first-party apps without a lifecycle.
- Buildroot legal information, corresponding-source obligations, SBOM output,
  dependency review, vulnerability intake, security response, and key handling
  are release procedures rather than ad hoc checks.
- Secrets, logs, crash reports, permissions, session IPC, device access, and
  installer authority have documented boundaries and negative tests.
- A failed graphical start or update preserves a usable, documented recovery
  path without weakening the existing boot and installer safety gates.

### R7: Hardware, Stability, And Release Qualification

Scope: convert a reproducible virtual prototype into a supportable release.

Pass criteria:

- The sanitized read-only inventory in `hardware-inventory.md` is reviewed
  before any physical boot plan, and destructive disk testing receives separate
  explicit approval.
- The MSI Sword 17 matrix in `hardware-support.md` records successful boot,
  display, input, storage, network, audio, Bluetooth, battery, suspend/resume,
  thermal, and recovery evidence for the exact tested variant.
- QEMU and hardware release candidates pass repeated cold boot, logout/login,
  compositor restart, suspend/resume where applicable, update rollback, disk
  pressure, low-memory, service failure, and clean shutdown scenarios.
- A minimum soak duration, workload, resource-growth threshold, crash budget,
  and log-retention rule is fixed before running qualification; results and
  known failures remain visible.
- Installation, recovery, upgrade, backup expectations, known limitations,
  release notes, and support boundaries match the shipped image.

## Release Rule

M0-M12 completion alone does not authorize a v1.0 or daily-use claim. A v1.0
release candidate requires R1-R7 to pass at their required evidence level, no
open release-blocking defect, reproducible release artifacts, and an explicit
release decision. Physical disk operations and release publication remain
separate approval actions.

## Recommended Execution Order

1. Close the M12 component catalog and its regression matrix.
2. Baseline R2 and integrate production presentation/performance checks.
3. Implement the R3 protocol and display compatibility matrix.
4. Establish the unprivileged session and select the R4 service architecture.
5. Build R5 accessibility, internationalization, and complete input coverage
   alongside shared components rather than after visual freeze.
6. Implement the R6 update, supply-chain, security, and rollback model.
7. After read-only inventory review, run R7 hardware bring-up and release
   qualification without weakening recovery or installer gates.
