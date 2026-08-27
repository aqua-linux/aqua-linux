# ADR 0001: Compositor Foundation

Status: accepted and started. Smithay is linked for Linux targets with a minimal Wayland frontend feature set; Calloop is linked for the event-loop smoke test.

## Context

Aqua Linux is not a theme pack or desktop remix. The long-term graphics target is a custom Wayland compositor, with QEMU x86_64 as the first development target and MSI Sword 17 hardware validation later.

The project now has:

- Buildroot image booting to recovery shell.
- Runtime assets installed under `/usr/share/aqua`.
- A Rust `aqua-compositor` skeleton with asset probing.
- A deterministic Calloop event-loop smoke test.

The next step needs minimal Wayland display, socket lifecycle, Calloop dispatch, client insert, and session-state smoke checks before backend, rendering, and shell code are written.

## Decision

Use Smithay as the Rust compositor foundation.

Other mature Smithay compositors may be studied as reference implementations for
protocol flow, output lifecycle, input routing, and failure handling. Aqua does
not transplant their source, shaders, tests, visual assets, or product behavior.
Reference-derived work is independently implemented against Aqua's own state,
rendering, Buildroot, recovery, and acceptance contracts. Direct source reuse is
allowed only after its license and attribution requirements are reviewed and
recorded explicitly.

Pinned planning metadata:

- Crate: `smithay`
- Version: `0.7.0`
- License: MIT
- Minimum Rust version from crate metadata: `1.80.1`
- Enabled feature set for the first spike: `wayland_frontend`

Smithay is linked only for Linux targets with `default-features = false`. The default Smithay feature set is intentionally avoided because it pulls in backend and renderer families that are not part of the first spike.

On macOS development hosts, `aqua-compositor smoke-wayland` uses a stub and reports `host_stub=true`, because Smithay's Wayland frontend links the system `xkbcommon` library. Linux/QEMU remains the product validation target.

The first socket smoke uses a temporary `ListeningSocket::bind_absolute` lifecycle. It verifies bind, nonblocking accept with no clients, one local client connection, `DisplayHandle::insert_client`, and cleanup on drop. It does not start a persistent session.

The Calloop socket smoke wraps the temporary `ListeningSocket` in `Generic<ListeningSocket>` and verifies that one event-loop dispatch invokes the callback, accepts the local client, inserts it into the display, then runs one `dispatch_clients` and `flush_clients` pass. The local smoke client does not send protocol requests, so zero dispatched requests is acceptable as long as the server calls succeed.

`AquaCompositorSession` is introduced as the narrow state boundary around the Wayland display and Smithay compositor state. Current smoke paths use it for client insert, dispatch, and flush so later work can grow toward a real loop without spreading display ownership across ad hoc functions.

The session config defaults are explicitly recovery-safe: socket `aqua-wayland-0`, runtime directory `/run/aqua`, runtime asset root `/usr/share/aqua`, `autostart=false`, `boot_graphics=false`, and fallback TTY required. This prevents early compositor work from accidentally changing the Milestone 1 boot contract.

`probe-session-config` validates those defaults and can also read the Buildroot-generated `/etc/aqua/compositor-session.conf` file. Rootfs contract export runs that file through the packaged compositor binary, so the image and Rust session assumptions are checked together.

`probe-session-env` derives `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`, `AQUA_ASSET_ROOT`, autostart, and boot graphics flags from the same config. Buildroot writes that environment to `/etc/aqua/session.env`; recovery shells source it, but graphics remain disabled.

`probe-session-bootstrap` is the first config-driven startup guard. It prepares a private runtime directory, reports the configured `/run/aqua` target, validates the derived environment, and exits with `session_started=false` and `desktop_shell_started=false`.

The session run-once smoke is the first single entrypoint for the compositor lifecycle. It binds a temporary socket, accepts one local client through Calloop, inserts the client, dispatches, flushes, cleans up, and exits. This is intentionally not a long-running service.

The bounded session-loop smoke extends that entrypoint without starting a persistent service. It accepts one local client, then runs three explicit event-loop iterations with one `dispatch_clients` and one `flush_clients` pass per iteration. This keeps lifecycle progress measurable while preserving recovery-safe behavior.

`aqua-scene` is introduced as the static shell scene model before rendering. It defines the first visual-reference surfaces and geometry checks for wallpaper, top panel, desktop icons, dock, launcher, system overview, and notification toast. The compositor can probe this model without starting boot graphics.

## Rationale

Smithay is purpose-built for writing Wayland compositors in Rust. It keeps Aqua Linux aligned with the custom compositor direction without pulling in KDE, GNOME, XFCE, LXQt, or another desktop environment.

Limiting the dependency to Linux keeps local host checks working while preserving the real target path for QEMU and later hardware validation.

## Consequences

Near term:

- Keep `aqua-compositor` compiling on macOS hosts without requiring `xkbcommon`.
- Add foundation metadata and validation.
- Prove a deterministic event-loop smoke test.
- Prove a minimal Smithay display/compositor-global smoke path on Linux.
- Prove a temporary Smithay socket lifecycle and local client insert smoke path on Linux.
- Prove a Calloop-driven socket dispatch, local client insert, `dispatch_clients`, and `flush_clients` smoke path on Linux.
- Keep display and compositor state ownership behind an `AquaCompositorSession` skeleton.
- Keep the config-derived session environment visible in the rootfs and rootfs contract export.
- Add a config-driven session bootstrap guard before creating any persistent compositor process.
- Add a session run-once entrypoint before creating any persistent compositor process.
- Add a bounded session-loop smoke before creating any persistent compositor process.
- Add a manual-dev nested preview frame loop before enabling a real host window backend.
- Add a rootfs-verified manual nested preview backend path behind the handoff gate before opening it as an operator-controlled preview.
- Add an operator-controlled manual nested preview execution command before boot integration.
- Bridge that manual execution frame to a feature-gated host preview window before enabling any QEMU boot graphics.
- Add a QEMU recovery request command for the host-visible preview path without packaging host preview tooling into the rootfs.
- Add a QEMU recovery launcher command that converts the visible preview request into a bounded manual launch plan while keeping graphical boot disabled.
- Prepare nested-dev first, then QEMU DRM/KMS.

Later:

- Add socket/session handling around the Wayland display.
- Add nested backend before booted DRM/KMS backend.
- Keep fallback recovery shell available in the image.
