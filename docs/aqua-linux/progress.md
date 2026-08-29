# Aqua Linux v1.0 Progress Report

> Generated from `docs/aqua-linux/progress.json`. Update the changed phase date, then run `scripts/write-progress-report.sh`.

**Overall progress: 85%**

| Field | Value |
| --- | --- |
| Updated | 2026-08-29 |
| OS base | Buildroot |
| Graphics target | custom Wayland compositor |
| Development target | QEMU x86_64 |
| Hardware target | MSI Sword 17 later |

## Current Stage

Milestone 12 typography now includes deterministic Noto Sans to Noto Sans Arabic fallback, stable fallback baselines, explicit missing-glyph diagnostics, and a committed 16-case fixture report across the four supported scales. Packaged QEMU typography captures and long-label acceptance remain open. Physical MSI Sword 17 validation remains unauthorized until read-only evidence is reviewed.

## Phases

Phases are ordered by their most recent update.

| Updated | Phase | Status | Progress | Summary |
| --- | --- | --- | ---: | --- |
| 2026-08-29 | M12: Visual Fidelity and Component System | In Progress | 10% | The shared aqua-text crate owns Rustybuzz shaping, Unicode bidi visual runs, grapheme-safe wrapping and ellipsis, Turkish case behavior, six named text roles, four supported output scales, and a bounded scale-native glyph cache. A packaged Noto Sans Arabic face now provides deterministic grapheme-aware fallback without changing role baselines, and a committed 16-case report locks font selection and metrics. Packaged QEMU typography evidence, long-label acceptance, elevation, icons, motion, and the component catalog remain open. |
| 2026-08-28 | M2: Asset and Visual System Intake | Complete | 100% | Public contracts capture the permanent desktop, application, search, first-party app, installer, completion, and four-theme direction derived from private Git-ignored boards. Approved transparent brand exports and the reproducible pale-wave wallpaper are used by the runtime. Thirteen project-authored Aqua Core Icons permanently cover core application, desktop, notification, and status roles with explicit MIT licensing; no temporary icon package remains. |
| 2026-08-28 | M11: Polish and Public Readiness | Complete | 100% | The v1 desktop and installer contracts are documented. The runtime has a compact top bar, reproducible pale-wave wallpaper, permanent Aqua Core Icons, separate Applications and Global Search surfaces, three bottom shell groups, shared first-party window chrome, four live-refreshed themes, and three real workspaces. Current QEMU captures carry a provenance manifest, the public hardware matrix prevents physical support overclaims, and the default-image gate enforces recovery-safe startup. Structured issue forms, private security routing, canonical labels, a pull request safety checklist, and CI-enforced contributor contracts now define public intake and triage. |
| 2026-08-22 | M4: Scene and Surface Renderer | Complete | 100% | The Smithay GLES2 renderer composes the packaged wallpaper, shared surfaces, optional two-pass blur, and live wl_shm textures at the native output size. Physical DRM retains XRGB8888 GBM front/back dma-buf scanout. QEMU keeps GLES readback for the desktop, but a full-output client that explicitly supplies a complete Wayland opaque region may use the dumb-buffer bridge after one GPU validation frame. Packaged installer acceptance measured 30 bridged frames at a 61 ms median total while preserving distinct 1280x800 captures. |
| 2026-08-21 | M9: Graphical Installer MVP | Complete | 100% | The packaged installer executes and boots the complete separately gated installation, proves failure cleanup, and emits QEMU-validated transaction progress. Seven deterministic Rust-rendered setup screens feed a packaged aqua.installer wl_shm xdg-toplevel client; QEMU proves full-output composition and navigation from Welcome through Completed. An explicit presentation-only rehearsal consumes the canonical non-executing 20-step graph, renders progress at 40%, 65%, and 95%, reaches Completed at 100%, emits seven distinct screendumps, and proves transaction_executed=false. Real virtio pointer input activates the Welcome footer and selects a Language row through renderer-shared geometry before keyboard navigation resumes. Full-output input origin now matches presentation origin, blocked disks and hidden controls reject hits, and disk/profile application retains explicit activation. |
| 2026-08-16 | M8: Terminal App | Complete | 100% | A packaged aqua-terminal opens as a supervised aqua.terminal xdg-toplevel with Aqua chrome and a dark readable scrim. It uses portable-pty for a real /bin/sh session and vt100 for mature terminal parsing, renders the prompt, coalesces PTY output into bounded redraws, and resizes both the PTY and parser grid. Aqua boot mounts devpts and provides /dev/ptmx. QEMU forwards all 40 press/release events for 'echo aquaterminalok', observes Enter, parses output, applies Alt+F8 resize through a real xdg configure to a 640x478 buffer and 74x21 grid, resizes the PTY, repaints, captures the window, closes with Alt+F4, reaps the process, and confirms restart=never. Pointer titlebar and bottom-right resize requests are implemented; broader key and clipboard support remain later enhancements. |
| 2026-08-16 | M7: First-Party Shell Surfaces | Complete | 100% | The state-driven shell starts packaged Aqua Files and Aqua Settings through strict executable preflight and reusable process supervision. Production QEMU maps and raises first-party windows by XDG app_id, tracks damage, closes applications, reaps failures, and removes stale surfaces. Notifications, real system overview, desktop icon interactions, target-aware Properties, confined Trash, and the confirmed session menu are integrated. Applications and Global Search are separate centered modes with scaled pointer targets and real quick actions. The bottom shell separates these controls from the centered app dock and three real workspaces; keyboard and pointer paths activate workspaces, and focused windows can move between them. |
| 2026-08-14 | M6: Boot Aqua Compositor in QEMU | Complete | 100% | The packaged compositor boots through the explicit aqua.boot_graphics=1 gate while default boot remains text recovery. In one aqua-compositor process, real Smithay compositor, shm, xdg-shell, and Aqua Seat globals bind /run/aqua/aqua-wayland-drm-0, dispatch clients and DRM page flips, and discover real virtio input through libinput/udev. The supervised boot profile now remains active until controlled stop or fatal error. QEMU verifies two complete start-stop cycles, socket and PID cleanup, client termination, CRTC restoration, GBM release, and recovery return. |
| 2026-08-14 | M3: Nested Aqua Compositor Prototype | Complete | 100% | Smithay/calloop, host preview, recovery-safe output, and QEMU DRM/KMS integration are complete for the prototype milestone. The packaged supervisor limits rapid failures and returns to recovery after budget exhaustion; QEMU proves three real compositor failures and two bounded restarts. The boot gate requires aqua.boot_graphics=1 plus a separate graphics profile, tracks PID/state, rejects duplicates, stays disabled by default, and starts the real DRM-Wayland compositor from rcS when explicitly enabled while preserving recovery TTY access. |
| 2026-08-13 | M5: Basic Wayland Client Support | Complete | 100% | The packaged QEMU DRM session runs the upstream Weston simple-shm C reference client as an independent compatibility fixture beside an Aqua state client. Aqua's compositor exposes the xdg-shell and wl_shm protocol path, composites their 250x250 and 384x256 buffers, returns frame callbacks, records damage, changes focus and stacking, handles move, resize, size constraints, maximize/restore, fullscreen/restore, repeated configure acknowledgements, compositor close, stale-focus cleanup, surviving-window repaint, final empty-desktop repaint, and recovery return. The Weston compositor, shells, backends, and desktop session are neither packaged nor started. |
| 2026-08-05 | M1: Buildroot Boot to Text Recovery | Complete | 100% | QEMU x86_64 boots a minimal Buildroot image to BusyBox recovery shell with serial boot markers. |
| 2026-08-05 | M0: Repository and Build Skeleton | Complete | 100% | Rust workspace, Buildroot external tree, scripts, docs, assets folder, and local checks exist. |
| 2026-08-04 | M10: MSI Sword 17 Hardware Validation | Not Started | 0% | Real hardware validation is intentionally later; QEMU remains the first reproducible target. |

## Completion Rules

- Aqua Linux v1.0 is treated as 100%.
- Overall progress is the rounded arithmetic mean of the 13 phase percentages so the public value is reproducible.
- Progress advances only when bootable OS, compositor, installer, or validation contracts move forward.
- Host mockups and website work do not count as OS completion unless they become packaged runtime assets.
- The project remains Buildroot-based and does not use Ubuntu, Debian, KDE, GNOME, XFCE, LXQt, or a theme-pack base.

## Next Developments

1. Add packaged QEMU typography captures and long localized-label overlap acceptance.
2. Implement tokenized elevation levels with bounded damage and reusable shadow masks.
3. Implement reviewed SVG loading and scale-native Aqua Core Icon rasterization and caching.
4. Implement semantic, interruptible, frame-driven state motion and reduced-motion behavior.
5. Build the complete shared component catalog and deterministic visual-regression matrix.
6. Collect and review a sanitized read-only inventory from the MSI Sword 17 before authorizing any physical boot or installation validation.
