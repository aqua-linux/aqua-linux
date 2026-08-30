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
input only after authoritative reconciliation reaches applied. A separate
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
