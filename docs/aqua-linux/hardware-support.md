# Aqua Linux Hardware Support Status

Status date: 2026-08-31

Aqua Linux is an active prototype. QEMU x86_64 is the only validated machine
target. MSI Sword 17 is the planned physical validation target, not a currently
supported installation target. Do not infer support for a physical component
from a matching kernel option or from successful virtual hardware tests.
Hardware evidence is one mandatory part of the broader
[v1 readiness gates](v1-readiness.md); roadmap milestone percentages do not
change the support status recorded here.

## Status Meanings

- **Validated:** exercised by an automated test against the current Aqua Linux
  image, with a checked success marker.
- **Present, unvalidated:** code, configuration, or a driver is included, but
  the complete user-visible workflow has not been proven.
- **Not tested:** no successful test exists for the named target.
- **Deferred:** intentionally outside the current prototype validation scope.

## QEMU x86_64

| Area | Status | Current boundary | Evidence |
| --- | --- | --- | --- |
| CPU and basic boot | Validated | QEMU x86_64 TCG boots the Buildroot image to the recovery shell. | `scripts/check-boot.sh` |
| Recovery path | Validated | The fallback text shell remains available before and after graphical sessions. | `scripts/check-graphical-boot-qemu.sh` |
| Graphical session | Validated | The custom compositor starts through the explicit graphics gate, presents through virtio-gpu DRM/KMS, stops cleanly, and can start again. | `scripts/check-graphical-boot-qemu.sh` |
| Keyboard and pointer | Validated | virtio keyboard and mouse events reach the Aqua seat through libinput/evdev. | `scripts/check-fbdev-presenter-qemu.sh` |
| Storage | Validated | virtio block devices boot the image and provide disposable installer targets. | `scripts/check-installer-transaction-qemu.sh` |
| UEFI installed boot | Validated | EDK2 loads GRUB from `EFI/BOOT/BOOTX64.EFI`, then boots `PARTLABEL=AQUA_ROOT` to recovery. | `scripts/check-installer-transaction-qemu.sh` |
| Desktop and first-party windows | Validated | Applications, Search, Files, Settings, Terminal, workspaces, and session cleanup have QEMU acceptance paths. | `scripts/check-public-runtime-qemu.sh` and `scripts/check-terminal-qemu.sh` |
| Network adapter | Validated | The opt-in QEMU profile proves virtio-net DHCP, a default route, external DNS lookup, lease renewal, route-loss recovery, forced DHCP-client recovery, and continuing recovery-shell access. Network management and default activation remain disabled. | `scripts/check-network-qemu.sh` |
| Audio | Validated | Opt-in Intel HDA profiles prove duplex discovery, output, controlled zero-PCM input, deterministic non-silent D-Bus-injected input, rejection of a bounded injected input-source failure without hanging recovery, explicit active playback and controlled-capture interruption plus recovery after PipeWire loss, active playback and controlled-capture interruption plus ordered full-stack recovery after WirePlumber loss, explicit route-aware active-capture interruption and blocked new capture when the sole virtual input device is removed, fail-closed native volume/mute controls during PipeWire and WirePlumber-triggered full-graph outages, the production adapter's three-attempt per-generation submission budget and graph-change-gated recovery, degradation after the real three-restart budget is exhausted independently through repeated PipeWire and WirePlumber loss, plus fail-closed native controls after both PipeWire and WirePlumber budget exhaustion, controlled capture rejection after PipeWire and WirePlumber restart exhaustion, acknowledged default switching across two independent output devices, fallback playback after the selected virtual PCI output is removed, stable default playback when the non-default virtual PCI output is removed, uninterrupted active default playback during that non-default removal, and explicit route-aware interruption without false completion when the actively selected virtual PCI output is removed before a new client plays on fallback. Runtime HDA codec insertion is rejected by QEMU; the replug profile proves rollback of the incomplete controller and continued fallback playback, not successful restoration. No host microphone is used; this is virtual-device evidence only. | `scripts/check-audio-qemu.sh`, `scripts/check-audio-input-qemu.sh`, `scripts/check-audio-signal-input-qemu.sh`, `scripts/check-audio-input-disconnect-qemu.sh`, `scripts/check-audio-active-input-unplug-qemu.sh`, `scripts/check-audio-active-service-loss-qemu.sh`, `scripts/check-audio-active-policy-loss-qemu.sh`, `scripts/check-audio-active-capture-loss-qemu.sh`, `scripts/check-audio-active-capture-policy-loss-qemu.sh`, `scripts/check-audio-control-submission-budget-qemu.sh`, `scripts/check-audio-control-service-loss-qemu.sh`, `scripts/check-audio-control-policy-service-loss-qemu.sh`, `scripts/check-audio-restart-exhaustion-qemu.sh`, `scripts/check-audio-control-restart-exhaustion-qemu.sh`, `scripts/check-audio-capture-restart-exhaustion-qemu.sh`, `scripts/check-audio-policy-restart-exhaustion-qemu.sh`, `scripts/check-audio-control-policy-restart-exhaustion-qemu.sh`, `scripts/check-audio-capture-policy-restart-exhaustion-qemu.sh`, `scripts/check-audio-multi-route-qemu.sh`, `scripts/check-audio-hotplug-qemu.sh`, `scripts/check-audio-output-replug-qemu.sh`, `scripts/check-audio-nondefault-unplug-qemu.sh`, `scripts/check-audio-active-nondefault-unplug-qemu.sh`, and `scripts/check-audio-active-default-unplug-qemu.sh` |
| Wi-Fi and Bluetooth | Not tested | No virtual radio workflow is claimed. | None |
| Battery and power reporting | Not tested | QEMU is not used as evidence for laptop battery behavior. | None |
| Suspend and resume | Deferred | Resume integrity and graphical restoration require a later dedicated test. | None |

QEMU validation applies only to the virtual devices declared by the repository
scripts. It is not evidence for unrelated PCI, USB, ACPI, firmware, or laptop
platform behavior.

## MSI Sword 17

No MSI Sword 17 hardware validation has started. The exact machine variant and
device identifiers have not been recorded, so Aqua Linux does not currently
claim support for any MSI Sword 17 configuration.

The [hardware inventory contract](hardware-inventory.md) defines the bounded
read-only record that must be collected and reviewed before these rows can
advance. Its existence does not change the current support status.

| Area | Status | Required before a support claim |
| --- | --- | --- |
| UEFI boot and Secure Boot posture | Not tested | Record firmware mode, decide the unsigned-development-image procedure, and prove recovery boot from removable media. |
| CPU and platform | Not tested | Record DMI and CPU identifiers and complete a stable recovery boot. |
| Internal and discrete graphics | Not tested | Record PCI identifiers, identify the active GPU path, and prove internal display output plus compositor recovery. |
| Internal display | Not tested | Record native mode and prove stable DRM/KMS presentation. |
| Keyboard and touchpad | Not tested | Record input identifiers and prove complete libinput event delivery. |
| NVMe or SATA storage | Not tested | Record controller and disk identifiers; installation to physical storage remains prohibited until a separate destructive-test plan is approved. |
| Ethernet and Wi-Fi | Not tested | Record PCI/USB identifiers, required firmware, association or link, DHCP, DNS, and reconnect behavior. |
| Audio | Not tested | Record codec/controller identifiers and prove output, input, mute, and volume behavior. |
| Bluetooth | Not tested | Record controller identifier and required firmware, then prove discovery and reconnect behavior. |
| Battery and charging | Not tested | Prove AC state, charge percentage, charging transitions, and low-power reporting. |
| Suspend and resume | Not tested | Prove repeated suspend/resume with display, input, storage, network, and audio restoration. |

Until those checks exist, Aqua Linux must be run on this machine only through a
non-destructive external test image with recovery access. Physical-disk
installation and daily-use claims remain unsupported.

## Claim Rules

1. A kernel option means only that a driver was selected; it does not prove a
   device works.
2. QEMU evidence cannot be reused as physical MSI Sword 17 evidence.
3. A physical component becomes validated only after its identifier, firmware
   dependency, test procedure, result, and recovery outcome are recorded.
4. Failed and partial results stay visible; they are not converted into a
   general support claim.
5. Milestone 10 remains at 0% until testing occurs on the named physical
   machine.
