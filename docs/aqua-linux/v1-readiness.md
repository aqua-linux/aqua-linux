# Aqua Linux V1 Readiness Gates

Status date: 2026-08-29

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

The packaged kernel now enables Bochs DRM and the macOS QEMU runner defaults to
`bochs-display`. This reaches Aqua's `production-gbm-kms` path with GLES
software rendering and direct GBM dma-buf scanout, while emitting zero
production full-frame readbacks and CPU framebuffer copies. The virtio target
remains a separately identified `legacy-cpu-copy` fallback. Repeated collection,
QEMU budget review, and the initial bounded QEMU soak are complete for these
profiles. Longer release-qualification soak and a separately selected
physical-target budget remain downstream; QEMU TCG measurements are not
physical responsiveness evidence.

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

XWayland is not automatically required for v1. If it remains excluded, the
supported application model and incompatibility boundary must be public.

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

The concrete networking implementation must be selected by an ADR before
packaging. [ADR 0004](adr-0004-audio-service-stack.md) selects ALSA,
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
for a newer synchronized generation before retrying; deterministic adapter and
shell tests cover failure, blocking, recovery, and acknowledgement. This is
host control-plane evidence rather than packaged media or hardware evidence. A separate
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
loss reaches `failed_service=wireplumber` degradation. Other control and UI
failure matrices remain open. The remaining runtime native-backend integration
gate still blocks default-image enablement.
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
