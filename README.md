# Aqua Linux

[![CI](https://github.com/aqua-linux/aqua-linux/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/aqua-linux/aqua-linux/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-1769aa.svg)](LICENSE)
[![Stage](https://img.shields.io/badge/stage-active%20prototype-0f766e.svg)](docs/aqua-linux/progress.md)

Aqua Linux is an independent, Buildroot-based graphical operating system built
around a custom Wayland compositor written in Rust with Smithay. The project
focuses on a controlled system image, a coherent first-party desktop, explicit
recovery paths, and evidence-driven engineering.

> **Project status: active prototype.** Aqua Linux boots and runs its graphical
> stack in the declared QEMU x86_64 environment. It is not ready for installation
> on a daily-use machine. Physical hardware support, production security, update
> delivery, and release qualification remain incomplete.

## Project Overview

Aqua Linux owns the complete path from image construction to the desktop
session:

- Buildroot produces the kernel, root filesystem, boot artifacts, and bounded
  runtime package set.
- BusyBox init preserves a serial-observable recovery boot.
- The Aqua session starts only through explicit, fail-closed activation gates.
- The custom compositor manages Wayland clients, input, outputs, window
  lifecycle, rendering, and first-party shell surfaces.
- Automated contracts cover source quality, boot behavior, session recovery,
  protocol boundaries, installer safety, and selected QEMU workflows.

The default image favors recovery and diagnosability over automatic graphical
startup. Optional graphics, audio, and network paths remain separately gated
until their acceptance requirements are satisfied.

## Current Capability

| Area | Current state |
| --- | --- |
| Image and boot | Reproducible x86_64 Buildroot image with stable serial markers and a recovery shell |
| Graphics | Custom Smithay compositor with QEMU-tested DRM/KMS, GBM/EGL, libinput, and Wayland session handling |
| Desktop | Prototype launcher, dock, workspaces, notifications, Files, Settings, Properties, and Terminal surfaces |
| Wayland | Bounded protocol and interoperability coverage for window lifecycle, input, selection, drag-and-drop, text input, scaling, outputs, popups, and subsurfaces |
| Installer | Guarded graphical workflow with target validation, explicit destructive confirmation, and disposable-QEMU transaction tests |
| Services | Typed observation boundaries and supervised, opt-in service lifecycles; default network ownership remains disabled pending QEMU acceptance |
| Physical hardware | Not validated; no physical installation or daily-use support claim is made |

Detailed status and evidence are maintained in the
[progress report](docs/aqua-linux/progress.md),
[hardware support matrix](docs/aqua-linux/hardware-support.md), and
[v1 readiness gates](docs/aqua-linux/v1-readiness.md). Milestone completion is
an engineering progress measure, not a release-readiness score.

## Architecture

```text
Firmware or QEMU
└── Linux kernel
    └── Buildroot userspace and BusyBox init
        ├── Recovery shell
        └── Aqua session supervision
            └── aqua-compositor
                ├── Wayland protocol and input handling
                ├── DRM/KMS and renderer integration
                ├── aqua-shell, aqua-scene, and aqua-renderer
                └── First-party Wayland applications
```

The Rust workspace is divided by responsibility:

| Crate | Responsibility |
| --- | --- |
| `aqua-compositor` | Wayland protocols, input, outputs, window lifecycle, rendering integration, and session behavior |
| `aqua-shell` | Desktop state, launcher, dock, workspaces, notifications, and first-party application models |
| `aqua-renderer` | Software and GLES rendering, rasterization, themes, icons, and shared visual output |
| `aqua-scene` | Renderer-independent scene geometry and surface contracts |
| `aqua-components` | Shared component anatomy, interaction state, input, and accessibility semantics |
| `aqua-text` | Text shaping, fallback, layout, and bounded glyph caching |
| `aqua-service-adapters` | Typed audio and network observation and reconciliation boundaries |
| `aqua-installer` | Installer state, storage validation, transaction planning, and execution gates |
| `aqua-host-tools` | Host-side development probes and bounded preview tooling |

Architecture decisions and subsystem documentation live in
[`docs/aqua-linux/`](docs/aqua-linux/).

## Development Environment

### Requirements

- Rust 1.85 with `rustfmt` and Clippy
- Docker Desktop or a compatible Linux container runtime
- QEMU x86_64
- Python 3, `expect`, and `jq` for extended validation

### Source Validation

Run the same core checks required by continuous integration:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
scripts/check.sh
```

Public-repository boundaries can be checked independently:

```sh
scripts/check-public-repo.sh
```

<details>
<summary>Validation layers</summary>

The validation stack is intentionally layered so a fast source check and a
full image check answer different questions:

| Layer | Scope |
| --- | --- |
| Rust checks | Formatting, Clippy, workspace tests, and doc tests |
| Repository checks | Licensing, asset provenance, public-file boundaries, and generated-report consistency |
| Runtime contracts | Boot markers, recovery behavior, compositor/session probes, and service supervisors |
| QEMU acceptance | Declared virtual hardware, installer transactions, and opt-in graphical workflows |

</details>

### Build The Image

The recommended Docker-backed build is:

```sh
scripts/build-image-docker-volume.sh
```

Alternative build paths are available for standard Docker storage and native
Linux hosts:

```sh
scripts/build-image-docker.sh
scripts/build-image.sh
```

Generated artifacts are written under `build/buildroot-output/images/` and are
excluded from version control. Validate a completed image with:

```sh
scripts/check-image.sh
```

### Boot In QEMU

Start the recovery-safe default image:

```sh
scripts/run-qemu.sh
```

Serial output is written to `build/qemu-serial.log`. A successful default boot
reaches:

```text
[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh
```

The graphical session is intentionally disabled in the default boot profile.
To open the interactive Aqua desktop in the QEMU window, run:

```sh
AQUA_KERNEL_APPEND=aqua.boot_graphics=1 scripts/run-qemu.sh
```

The interactive runner uses KVM automatically when `/dev/kvm` is accessible and
falls back to TCG otherwise. Set `QEMU_ACCELERATOR=tcg QEMU_CPU_MODEL=max` to
force software emulation.

Run its opt-in acceptance path with:

```sh
scripts/check-graphical-boot-qemu.sh
```

The installer execution path is restricted to disposable QEMU targets. Its
device identity, confirmation, and recovery gates must not be bypassed.

## Engineering Principles

- Preserve the Buildroot base, custom compositor architecture, and independent
  recovery path.
- Prefer explicit capability gates and bounded failure behavior over implicit
  service startup.
- Treat QEMU results as virtual-target evidence only.
- Do not claim physical hardware support without recorded device-specific
  validation.
- Keep installer and privileged operations target-bound, typed, and auditable.
- Keep runtime state authentic; mock values belong only in clearly identified
  design or test fixtures.

## Repository Boundaries

Private concept boards remain in the Git-ignored
`docs/aqua-linux/local-references/` directory. They are design inputs, not
redistributable runtime assets or evidence of implemented behavior.

Public runtime captures are maintained separately through the documented QEMU
validation and provenance flow. The README intentionally avoids embedded
screenshots; implementation status is represented by reproducible checks and
linked evidence records.

Build outputs, disk images, logs, temporary evidence, local environment files,
and the separately maintained `website/` repository must not be committed.

## Roadmap And Documentation

- [Milestones](docs/aqua-linux/milestones.md) — staged implementation plan
- [Progress report](docs/aqua-linux/progress.md) — generated engineering status
- [V1 readiness](docs/aqua-linux/v1-readiness.md) — mandatory release gates
- [Hardware support](docs/aqua-linux/hardware-support.md) — validated target boundaries
- [Buildroot](docs/aqua-linux/buildroot.md) — image and runtime integration
- [Compositor](docs/aqua-linux/compositor.md) — compositor architecture and evidence
- [Installer](docs/aqua-linux/installer.md) — safety model and transaction flow
- [Application compatibility](docs/aqua-linux/application-compatibility.md) — supported protocol boundary

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Changes
should be focused, testable, and preserve the architecture and safety boundaries
described above. The [contributor workflow](docs/aqua-linux/contributor-workflow.md)
defines issue intake, ownership, risk classification, and evidence requirements.

Report security vulnerabilities through the private process documented in
[SECURITY.md](SECURITY.md), not through public issue threads.

## Licensing

Source code is available under the [MIT License](LICENSE).

Artwork, branding, screenshots, and other media have separate distribution
terms described in [ASSET_LICENSE.md](ASSET_LICENSE.md). Dependency and asset
notices are maintained in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md)
and the [license audit](docs/aqua-linux/license-audit.md).
