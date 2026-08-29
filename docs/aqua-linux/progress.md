# Aqua Linux v1.0 Progress Report

> Generated from `docs/aqua-linux/progress.json`. Update the changed phase date, then run `scripts/write-progress-report.sh`.

**Roadmap implementation progress: 92%**

| Field | Value |
| --- | --- |
| Updated | 2026-08-30 |
| OS base | Buildroot |
| Graphics target | custom Wayland compositor |
| Development target | QEMU x86_64 |
| Hardware target | MSI Sword 17 later |

## Product Readiness

Roadmap progress and product readiness are separate measurements. Mandatory release gates are defined in [v1-readiness.md](v1-readiness.md).

| Field | Value |
| --- | --- |
| Classification | packaged-QEMU-proven prototype |
| Evidence level | packaged-QEMU-proven |
| Daily-use ready | No |
| Hardware-proven | No |
| Release-ready | No |
| Summary | The booted Buildroot image and custom compositor have substantial packaged-QEMU evidence, but the mandatory R1-R7 product gates remain open. Roadmap progress is not a release-readiness score. |

## Current Stage

Milestone 12 is complete with accepted typography, elevation, Aqua Core Icon integration, semantic state motion, and all twenty-two shared component primitives. The independent aqua-components crate owns renderer-neutral anatomy, stable state geometry, input activation, and accessibility semantics. Settings Audio supplies the final real slider consumer through a bounded persistent 0-100 output-volume preference and mute state without claiming playback, routing, or hardware support. ADR 0004 selects ALSA/eudev, per-user PipeWire, WirePlumber policy, and an authoritative Aqua adapter for the R4 audio path. The OS baseline is pinned to Buildroot 2025.02.17 LTS. The graphical path runs as locked aqua UID/GID 1000 with private /run/user/1000 and explicit video, audio, and input groups while recovery remains separate. Its packaged media supervisor proves finite ordered lifecycle and degraded state. The renderer-independent aqua-service-adapters crate now proves bounded typed device and route state, monotonic reconciliation, retained desired volume and mute across service loss, and acknowledgement-gated backend application. Settings no longer treats /dev/snd alone as service readiness. Audio packages remain disabled pending the supported PipeWire/WirePlumber API transport, legal-info review, and real QEMU media evidence. Physical MSI Sword 17 validation remains unauthorized until read-only evidence is reviewed.

## Phases

Phases are ordered by their most recent update.

| Updated | Phase | Status | Progress | Summary |
| --- | --- | --- | ---: | --- |
| 2026-08-29 | M12: Visual Fidelity and Component System | Complete | 100% | The shared aqua-text crate, tokenized elevation path, Aqua Core Icon pipeline, and semantic motion system have deterministic host fixtures plus packaged QEMU evidence. The independent aqua-components crate owns renderer-neutral anatomy, stable state geometry, input activation, and accessibility semantics for all 22 catalog entries. Settings Audio supplies the final real slider consumer through a bounded persistent 0-100 output-volume preference, mute state, exact renderer/input geometry, and fail-closed authoritative adapter availability. Desired state persists, while displayed service state and backend application require reconciliation; no playback, production transport, or physical-hardware support is claimed. All twenty-two primitives have deterministic four-theme acceptance at 800x600, 1280x800, and fractional-scale 1536x1024 plus packaged-QEMU acceptance through the real Smithay, GLES, and DRM path at 1280x800 in all four themes. |
| 2026-08-28 | M2: Asset and Visual System Intake | Complete | 100% | Public contracts capture the permanent desktop, application, search, first-party app, installer, completion, and four-theme direction derived from private Git-ignored boards. Approved transparent brand exports and the reproducible pale-wave wallpaper are used by the runtime. Thirteen project-authored Aqua Core Icons permanently cover core application, desktop, notification, and status roles with explicit MIT licensing; no temporary icon package remains. |
| 2026-08-28 | M11: Polish and Public Readiness | Complete | 100% | The v1 desktop and installer contracts are documented. The runtime has a compact top bar, reproducible pale-wave wallpaper, permanent Aqua Core Icons, separate Applications and Global Search surfaces, three bottom shell groups, shared first-party window chrome, four live-refreshed themes, and three real workspaces. Current QEMU captures carry a provenance manifest, the public hardware matrix prevents physical support overclaims, and the default-image gate enforces recovery-safe startup. Structured issue forms, private security routing, canonical labels, a pull request safety checklist, and CI-enforced contributor contracts now define public intake and triage. |
| 2026-08-22 | M4: Scene and Surface Renderer | Complete | 100% | The Smithay GLES2 renderer composes the packaged wallpaper, shared surfaces, optional two-pass blur, and live wl_shm textures at the native output size. Physical DRM retains XRGB8888 GBM front/back dma-buf scanout. QEMU keeps GLES readback for the desktop, but a full-output client that explicitly supplies a complete Wayland opaque region may use the dumb-buffer bridge after one GPU validation frame. Packaged installer acceptance measured 30 bridged frames at a 61 ms median total while preserving distinct 1280x800 captures. |
| 2026-08-21 | M9: Graphical Installer MVP | Complete | 100% | The packaged installer executes and boots the complete separately gated installation, proves failure cleanup, and emits QEMU-validated transaction progress. Seven deterministic Rust-rendered setup screens feed a packaged aqua.installer wl_shm xdg-toplevel client; QEMU proves full-output composition and navigation from Welcome through Completed. An explicit presentation-only rehearsal consumes the canonical non-executing 20-step graph, renders progress at 40%, 65%, and 95%, reaches Completed at 100%, emits seven distinct screendumps, and proves transaction_executed=false. Real virtio pointer input activates the Welcome footer and selects a Language row through renderer-shared geometry before keyboard navigation resumes. Summary real mode requires a shared target-bound destructive-acknowledgement checkbox before the unchanged exact uppercase ERASE phrase; target changes invalidate the acknowledgement. Full-output client focus takes precedence over hidden shell hit targets, modifier state reaches the focused Wayland client, blocked disks and hidden controls reject hits, and disk/profile application retains explicit activation. |
| 2026-08-16 | M8: Terminal App | Complete | 100% | A packaged aqua-terminal opens as a supervised aqua.terminal xdg-toplevel with Aqua chrome and a dark readable scrim. It uses portable-pty for a real /bin/sh session and vt100 for mature terminal parsing, renders the prompt, coalesces PTY output into bounded redraws, and resizes both the PTY and parser grid. Aqua boot mounts devpts and provides /dev/ptmx. QEMU forwards all 40 press/release events for 'echo aquaterminalok', observes Enter, parses output, applies Alt+F8 resize through a real xdg configure to a 640x478 buffer and 74x21 grid, resizes the PTY, repaints, captures the window, closes with Alt+F4, reaps the process, and confirms restart=never. Pointer titlebar and bottom-right resize requests are implemented; broader key and clipboard support remain later enhancements. |
| 2026-08-16 | M7: First-Party Shell Surfaces | Complete | 100% | The state-driven shell starts packaged Aqua Files and Aqua Settings through strict executable preflight and reusable process supervision. Production QEMU maps and raises first-party windows by XDG app_id, tracks damage, closes applications, reaps failures, and removes stale surfaces. Notifications, real system overview, desktop icon interactions, target-aware Properties, confined Trash, and the confirmed session menu are integrated. Applications and Global Search are separate centered modes with scaled pointer targets and real quick actions. The bottom shell separates these controls from the centered app dock and three real workspaces; keyboard and pointer paths activate workspaces, and focused windows can move between them. |
| 2026-08-14 | M6: Boot Aqua Compositor in QEMU | Complete | 100% | The packaged compositor boots through the explicit aqua.boot_graphics=1 gate while default boot remains text recovery. In one aqua-compositor process, real Smithay compositor, shm, xdg-shell, and Aqua Seat globals bind /run/aqua/aqua-wayland-drm-0, dispatch clients and DRM page flips, and discover real virtio input through libinput/udev. The supervised boot profile now remains active until controlled stop or fatal error. QEMU verifies two complete start-stop cycles, socket and PID cleanup, client termination, CRTC restoration, GBM release, and recovery return. |
| 2026-08-14 | M3: Nested Aqua Compositor Prototype | Complete | 100% | Smithay/calloop, host preview, recovery-safe output, and QEMU DRM/KMS integration are complete for the prototype milestone. The packaged supervisor limits rapid failures and returns to recovery after budget exhaustion; QEMU proves three real compositor failures and two bounded restarts. The boot gate requires aqua.boot_graphics=1 plus a separate graphics profile, tracks PID/state, rejects duplicates, stays disabled by default, and starts the real DRM-Wayland compositor from rcS when explicitly enabled while preserving recovery TTY access. |
| 2026-08-13 | M5: Basic Wayland Client Support | Complete | 100% | The packaged QEMU DRM session runs the upstream Weston simple-shm C reference client as an independent compatibility fixture beside an Aqua state client. Aqua's compositor exposes the xdg-shell and wl_shm protocol path, composites their 250x250 and 384x256 buffers, returns frame callbacks, records damage, changes focus and stacking, handles move, resize, size constraints, maximize/restore, fullscreen/restore, repeated configure acknowledgements, compositor close, stale-focus cleanup, surviving-window repaint, final empty-desktop repaint, and recovery return. The Weston compositor, shells, backends, and desktop session are neither packaged nor started. |
| 2026-08-05 | M1: Buildroot Boot to Text Recovery | Complete | 100% | QEMU x86_64 boots a minimal Buildroot image to BusyBox recovery shell with serial boot markers. |
| 2026-08-05 | M0: Repository and Build Skeleton | Complete | 100% | Rust workspace, Buildroot external tree, scripts, docs, assets folder, and local checks exist. |
| 2026-08-04 | M10: MSI Sword 17 Hardware Validation | Not Started | 0% | Real hardware validation is intentionally later; QEMU remains the first reproducible target. |

## Completion Rules

- Roadmap implementation progress is the rounded arithmetic mean of the 13 M0-M12 phase percentages so the public value is reproducible.
- The roadmap percentage measures scoped implementation work; it is not a daily-use, hardware-support, or release-readiness score.
- Aqua Linux v1.0 readiness is governed separately by the mandatory gates in docs/aqua-linux/v1-readiness.md.
- Progress advances only when bootable OS, compositor, installer, or validation contracts move forward.
- Host mockups and website work do not count as OS completion unless they become packaged runtime assets.
- The project remains Buildroot-based and does not use Ubuntu, Debian, KDE, GNOME, XFCE, LXQt, or a theme-pack base.

## Next Developments

1. Implement the supported PipeWire/WirePlumber API transport behind aqua-service-adapters and rehearse the exact Buildroot dependency and legal-info closure without enabling audio packages by default.
2. Baseline the R2 presentation path and enforce production no-readback, frame scheduling, damage, latency, resource, and dropped-frame acceptance while keeping diagnostic readback isolated.
3. Implement the R3-R6 Wayland compatibility, unprivileged session, system service, accessibility, internationalization, signed update, rollback, and security gates.
4. After core desktop functionality supplies real consumers, consolidate the proven typography, component, layout, focus, accessibility, lifecycle, and renderer contracts into the internal Aqua UI framework defined by ADR 0003.
5. Collect and review a sanitized read-only inventory from the MSI Sword 17 before authorizing any physical boot or installation validation.
