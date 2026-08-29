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
disabled while the packages are absent. Packaging remains blocked until the
fail-closed adapter contract, legal-info review, and real QEMU media evidence
exist. Buildroot availability alone is not an integration decision.

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
