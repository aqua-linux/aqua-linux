# Aqua Compositor Notes

Milestone 3 starts with a Rust skeleton for the future Aqua Wayland compositor.

Current crate:

`crates/aqua-compositor/`

Current commands:

```sh
cargo run -p aqua-compositor -- status
cargo run -p aqua-compositor -- smoke-loop
cargo run -p aqua-compositor -- smoke-wayland
cargo run -p aqua-compositor -- smoke-socket
cargo run -p aqua-compositor -- smoke-calloop-socket
cargo run -p aqua-compositor -- probe-session-config
cargo run -p aqua-compositor -- probe-session-env
cargo run -p aqua-compositor -- probe-session-bootstrap /etc/aqua/compositor-session.conf /run/aqua
cargo run -p aqua-compositor -- probe-session
cargo run -p aqua-compositor -- probe-scene
cargo run -p aqua-compositor -- dump-scene
cargo run -p aqua-compositor -- probe-render-plan
cargo run -p aqua-compositor -- dump-render-plan
cargo run -p aqua-compositor -- probe-paint-plan
cargo run -p aqua-compositor -- dump-paint-plan
cargo run -p aqua-compositor -- probe-frame-plan
cargo run -p aqua-compositor -- dump-frame-plan
cargo run -p aqua-compositor -- probe-frame-buffer
cargo run -p aqua-compositor -- dump-frame-buffer
cargo run -p aqua-compositor -- probe-raster
cargo run -p aqua-compositor -- dump-raster
cargo run -p aqua-compositor -- probe-raster-export
cargo run -p aqua-compositor -- export-raster-ppm build/aqua-raster.ppm
cargo run -p aqua-compositor -- smoke-run-once
cargo run -p aqua-compositor -- smoke-session-loop
cargo run -p aqua-compositor -- smoke-nested-preview-loop
cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
cargo run -p aqua-compositor -- run-manual-nested-preview-execution
cargo run -p aqua-compositor -- probe-client-window-model
cargo run -p aqua-compositor -- probe-client-surface-lifecycle
cargo run -p aqua-compositor -- probe-client-surface-registry
cargo run -p aqua-compositor -- probe-renderer-surface-sources
cargo run -p aqua-compositor -- probe-client-layer-pipeline
cargo run -p aqua-compositor -- probe-xdg-shell-binding
cargo run -p aqua-compositor -- probe-xdg-toplevel-client
cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
cargo run -p aqua-compositor -- probe-assets /usr/share/aqua
```

Local validation:

```sh
scripts/check-compositor.sh
```

Manual host-side preview:

```sh
cargo run -p aqua-host-tools -- probe-preview-window
cargo run -p aqua-host-tools -- probe-nested-output-presenter
cargo run -p aqua-host-tools -- probe-host-window-lifecycle
cargo run -p aqua-host-tools -- probe-manual-execution-window-bridge
cargo run -p aqua-host-tools -- handoff-summary
cargo run -p aqua-host-tools --features host-window-preview -- smoke-host-window-lifecycle
cargo run -p aqua-host-tools --features host-window-preview -- smoke-manual-execution-window
cargo run -p aqua-host-tools --features host-window-preview -- preview-window
```

These probes consume the compositor display-output handoff, manual nested
preview execution, and nested output surface lifecycle, then prove the manual
nested output presenter, host window lifecycle, and manual execution window
bridge paths without starting boot graphics. The smoke commands open a
feature-gated host preview for at most three frames. They are not part of the
Buildroot rootfs and they do not enable compositor autostart or a persistent
desktop session.
`handoff-summary` reads the rootfs-exported `visible-preview-launch.txt`
artifact and pairs it with the host-side `smoke-manual-execution-window`
command, without opening a host window by itself.

QEMU recovery shell manual launch plan:

```sh
aqua-compositor-manual-launch
```

This rootfs command validates the packaged session config, derived session environment, runtime directory, and display activation readiness while keeping `autostart=false`, `boot_graphics=false`, `display_output_started=false`, and the desktop shell stopped.

Guarded bounded run:

```sh
aqua-compositor-guarded-run
```

This command first validates the manual launch plan, then runs the three-frame nested-dev display-output smoke and confirms `display_output_stopped=true`, `fallback_tty_available=true`, `boot_graphics=false`, and `desktop_shell_started=false`.

Nested preview handoff gate:

```sh
aqua-compositor-handoff-gate
```

This command validates the guarded run, display-output handoff, visible preview readiness, nested preview loop, and manual nested backend path. It reports `promotion_decision=ready-for-manual-operator` and `automatic_promotion=false`; it does not open a real preview window.

Manual nested preview backend probe:

```sh
aqua-compositor probe-manual-nested-preview-backend
```

This rootfs-verifiable command joins the display-output handoff, nested output surface lifecycle, visible preview export, and bounded preview loop into the first manual nested backend path while keeping `display_output_started=false`, `fallback_tty_available=true`, `autostart=false`, and `boot_graphics=false`.

Operator-controlled manual preview execution:

```sh
aqua-compositor-preview-exec
aqua-visible-preview-request
aqua-visible-preview-launch
aqua-recovery-help
aqua-operator-transcript
aqua-graphics-enable-gate
aqua-graphics-launch-candidate
aqua-graphics-rollback-drill
aqua-graphics-startup-preflight
aqua-graphics-startup-rehearsal
aqua-graphics-qemu-display-gate
aqua-graphics-visible-qemu-attempt
aqua-graphics-visible-attempt-transcript
aqua-graphics-visible-attempt-result
aqua-graphics-visible-attempt-runner
aqua-graphics-qemu-visible-boot-check
aqua-graphics-qemu-observation-marker
aqua-compositor run-manual-nested-preview-execution
```

The rootfs command runs the handoff gate and then executes the manual nested
preview path for three bounded frames. It verifies that display output starts,
stops, cleans up, and returns to recovery while keeping
`preview_window_started=false`, `autostart=false`, `boot_graphics=false`, and
the desktop shell stopped.
`aqua-visible-preview-request` records the next manual host-visible preview
request under `/run/aqua` after that execution path passes. It still does not
open a window in QEMU and does not package `aqua-host-tools` into the rootfs.
`aqua-visible-preview-launch` is the recovery-shell operator command for the
next level up: it validates the request command, writes
`/run/aqua/visible-preview-launch.plan`, and leaves the actual visible window
handoff to the host-side feature-gated `aqua-host-tools` command.
`aqua-recovery-help` keeps the QEMU shell usable by listing the supported
manual commands, the host-side preview command, the
`pass_report_required=true` runbook rule, and the post-evidence
`aqua-qemu-visible-pass-report` step without starting graphics.
`aqua-operator-transcript` writes the dry-run operator sequence under
`/run/aqua`, including the matching host commands, while keeping the recovery
shell as the active runtime.
`aqua-graphics-enable-gate` records the first manual graphics enable request,
evaluates the handoff/manual-execution preflight logs, then refuses startup
until fail-safe compositor boot criteria are allowed. A positive dry-run mode
can report `currently_allowable=true` without starting graphics; default
recovery still keeps `autostart=false`, `boot_graphics=false`, and fallback TTY
intact.
`aqua-graphics-launch-candidate` consumes the positive dry-run gate and writes
a supervised no-start launch candidate with rollback metadata. It keeps
`actual_graphics_started=false` and `display_output_started=false`.

`aqua-graphics-rollback-drill` consumes that launch candidate and simulates the
operator-cancel and startup-failure return paths. It verifies
`rollback_command=/usr/bin/aqua-recovery`, keeps graphics/display output
stopped, and records `safe_return_to_recovery=ok`.

`aqua-graphics-startup-preflight` consumes the rollback drill and records the
bounded manual startup criteria: operator acknowledgement, fallback TTY,
rollback metadata, a three-frame limit, and a five-second timeout. It still
keeps `actual_graphics_started=false` and `display_output_started=false`.

`aqua-graphics-startup-rehearsal` consumes that preflight and the bounded
display-output smoke log. It proves a three-frame manual display-output run
started and stopped while `autostart=false`, `boot_graphics=false`, and
`desktop_shell_started=false`.

`aqua-graphics-qemu-display-gate` consumes the rehearsal and records the first
visible QEMU compositor step decision. It allows only an operator-triggered
manual step and keeps `visible_qemu_step_started=false`.

`aqua-graphics-visible-qemu-attempt` consumes that gate and writes the first
visible QEMU compositor attempt plan. The recorded command is
`/usr/bin/aqua-compositor-guarded-run`, but the attempt remains unstarted until
the operator explicitly runs it.

`aqua-graphics-visible-attempt-transcript` consumes the attempt plan and records
the manual operator sequence plus the expected safe return to recovery. It keeps
`persistent_graphical_session_started=false`.

`aqua-graphics-visible-attempt-result` consumes that transcript and writes the
attempt result contract. The default recovery-safe state is `manual-not-run`;
an operator-provided bounded run log can later be collected without enabling
`autostart`, `boot_graphics`, or a persistent graphical session.

`aqua-graphics-visible-attempt-runner` is the explicit recovery command that
runs the bounded guarded attempt and feeds the resulting log back into the
result collector. It records `completed-bounded-run` while still keeping
`persistent_graphical_session_started=false`.

`aqua-graphics-qemu-visible-boot-check` consumes that completed runner result
and records that the QEMU-visible boot path is ready for manual observation.
It deliberately records `qemu_vm_display_observed=false` until an operator
confirms the VM display separately.

`aqua-graphics-qemu-observation-marker` records that separate observation
state. Default recovery keeps `observation_status=not-observed`; a positive
observation must be requested explicitly. The exported
`graphics-qemu-observation-positive.txt` dry-run sets
`AQUA_QEMU_VM_DISPLAY_OBSERVED=true` for contract validation only; it still
keeps `persistent_graphical_session_started=false`, `boot_graphics=false`, and
`autostart=false`.

`aqua-qemu-visible-manual-runbook` records the first real operator procedure
for a non-Docker QEMU VM-display pass. The matching host entrypoint is
`scripts/run-qemu-visible-manual.sh`; the final observed marker remains manual
and must only be run after the VM display is visually confirmed.
`aqua-qemu-visible-evidence-record` records the capture metadata that permits a
positive observed marker; without it, `AQUA_QEMU_VM_DISPLAY_OBSERVED=true`
does not pass the observation contract.
`scripts/preflight-qemu-visible-manual.sh` verifies the host QEMU binary,
kernel/rootfs artifacts, host visible-flow scripts, capture tool availability,
and packaged recovery commands before the manual visible pass begins. It writes
`build/qemu-visible-manual-preflight.txt` when the manual QEMU launch
preconditions pass.
`scripts/write-qemu-visible-preflight-summary.sh` and
`scripts/check-qemu-visible-preflight-summary.sh` expose that preflight as a
JSON contract for CI/reporting and the host evidence flow without starting QEMU
or graphics.
`scripts/watch-qemu-visible-readiness.sh` watches the QEMU serial log for the
`recovery-ready` boot marker and prints the next capture/evidence commands once
the VM is ready for operator-visible capture. It does not start QEMU or enable
graphical boot.
`scripts/capture-qemu-visible-manual.sh` is the host-side helper for producing
that capture metadata from an already visible QEMU window. It does not start
QEMU or enable graphical boot.
`scripts/run-qemu-visible-ready-capture-flow.sh` combines the serial readiness
watch, host capture, verification, bundle writing, and recovery-shell paste-prep
for an already launched manual QEMU pass. It reports
`capture_hash_verified=true` before printing the VM apply heredoc, and still
does not launch QEMU, enable boot graphics, or start a persistent desktop
session.
`scripts/qemu-visible-status.sh` reads the preflight summary, image manifest,
runbook, evidence apply outputs, positive-observation output, and rejected
unverified bundle artifacts, including capture-hash rejection, to produce a single host-side
`ready-for-operator-pass` status in text and JSON form without starting QEMU or
graphics.
`scripts/first-graphics-session-status.sh` combines that status with the checked
boot summary and image manifest. It emits
`ready-for-controlled-visible-attempt` only when the packaged compositor,
manual nested execution, bounded visible runner, QEMU visible boot path,
recovery marker, operator gates, and fallback safety all agree. This remains a
no-launch readiness result, not a claim that a graphical session exists.
The Buildroot image now packages `aqua-graphics-fbdev-present`. It uses the
compositor's composited RGBA client frame, scales it to the QEMU fbdev mode,
converts it to 16-, 24-, or 32-bit framebuffer pixels, respects row stride, and
writes one bounded frame only after explicit operator confirmation. The kernel
and QEMU path use framebuffer console support with `virtio-vga`; visual output
still requires the operator-confirmed QEMU pass before it can be marked observed.
The headless boot contract now requires an `fbdev-device` serial marker before
that pass. The current QEMU target proves `/dev/fb0` is available at 1280x800,
32 bits per pixel, with a 5120-byte stride, and carries that status into both
boot-summary formats and both image-manifest formats.
`scripts/check-fbdev-presenter-qemu.sh` then boots the packaged image with no
display window, logs into the serial recovery shell, writes one composited frame
to `/dev/fb0`, and requires a safe return marker. This uses the distinct
`headless-qemu-test` confirmation source and records
`visible_frame_observed=false`; it does not replace the manual visible pass.
After the write, the check captures the QEMU scanout through its monitor socket
and exports both PPM and PNG artifacts at 1280x800. Their deterministic hashes
and the explicit unobserved state are enforced by the image manifest.
The same headless check now exercises `aqua-compositor present-drm-kms` behind
its separate `headless-qemu-test` confirmation gate. The compositor creates a
legacy KMS framebuffer, activates the 1280x800 `Virtual-1` mode for a bounded
hold, and emits an active-frame marker before the monitor capture. It then
restores the original CRTC and destroys both framebuffer and dumb-buffer
resources before returning to the recovery shell. This is the first real KMS
scanout proof; it does not yet submit a page flip or run a persistent graphical
session.
The follow-up `present-drm-page-flip` command now submits one real event-backed
page flip between two 1280x800 XRGB8888 KMS framebuffers. A bounded poll wait
must receive the matching CRTC page-flip event before the active marker and
QEMU monitor capture are produced. Both framebuffers and dumb buffers are then
destroyed after restoring the original CRTC. This proves one flip lifecycle,
not yet a repeating compositor frame loop.
The bounded `run-drm-frame-loop` stage now repeats that lifecycle for three
ordered flips while alternating the two framebuffer handles. Every submission
is gated on receiving the previous completion event and has its own timeout.
QEMU virtio-gpu returns zero for the optional event sequence field, so the
contract does not invent monotonic sequence metadata; it requires three
submitted and three received events instead. Cleanup and recovery behavior are
unchanged.
`run-drm-session-loop` then replaces direct polling with a calloop `Generic`
event source over the DRM fd. The compositor owns the source for three bounded
dispatches, receives all three flip completions, releases the source, restores
the CRTC, and returns to recovery. The Wayland display is deliberately not
started in this step; joining Wayland client dispatch and DRM dispatch in one
living session remains the final M6 integration task.
`run-drm-wayland-session` now provides that bounded shared lifecycle. It binds
`/run/aqua/aqua-wayland-drm-0`, creates the real Smithay compositor, shm,
xdg-shell, and Aqua Seat globals, then starts two separate xdg-toplevel client
processes through the bound socket. Both clients complete configure/ack, commit
independent 384x256 wl_shm buffers, and their overlapping surfaces are
composited into KMS scanout before
three calloop-owned DRM completion events. The command then stops the client,
removes the socket, releases KMS resources, restores the CRTC, and returns to
recovery. This closes M6 without enabling boot graphics or
autostart. During the bounded active interval it discovers seat0 keyboard and
pointer devices through libinput/udev and dispatches normalized key,
relative-motion, and button events into the same Aqua Seat. The session returns
both frame callbacks, accepts full and partial damage commits, hit-tests pointer
motion against overlapping surface geometry, and changes keyboard focus plus
server stacking order on a button press. The Super launcher shortcut remains
owned by the shell while normal keys reach both clients across the focus
transition. The changed stacking order is rendered into the inactive KMS dumb
buffer and presented through an additional event-confirmed page flip; the QEMU
capture records that repainted scanout. External-client cleanup and a persistent
session remain M3/M5/M7 work. The QEMU lifecycle test additionally destroys one
client's xdg-toplevel, xdg-surface, and wl-surface, removes its renderer record,
clears stale pointer focus, reassigns keyboard focus, and presents the surviving
surface set through another event-confirmed page flip.
The packaged compositor decodes the runtime Aqua wallpaper PNG and composites
it beneath the existing shell and client layers before pixel-format conversion.
The headless capture must report `wallpaper_source=runtime-asset`; host-only
probes can still use the deterministic fallback when no runtime asset root is
present.
`scripts/write-qemu-visible-operator-plan.sh` consumes that checked status and
writes `build/qemu-visible-operator-plan.txt` plus
`build/qemu-visible-operator-plan.json`. The plan is the deterministic operator
handoff for the first visible QEMU pass: host preflight, preflight summary,
manual QEMU launch, the explicit VM-side bounded fbdev frame command, operator
visual confirmation, ready-capture flow, recovery
shell bundle paste, and the explicit VM-side observed-marker apply command. It
does not start QEMU, enable graphical boot, or mark the display observed by
itself, and it carries `capture_hash_verification_required=true` as an operator
gate.
`scripts/write-qemu-visible-operator-packet.sh` wraps the checked status,
operator plan, boot summary, image manifest, checked first-graphics-session
readiness artifacts, and SHA-256 fingerprints into
`build/qemu-visible-operator-packet.txt` and
`build/qemu-visible-operator-packet.json`. This is the final host-side handoff
artifact before an actual operator-confirmed visible QEMU pass. It is blocked
unless readiness is `ready-for-controlled-visible-attempt` and has no failed
checks.
It includes the capture-hash verification and hash-rejection source statuses, so
the handoff cannot hide an unverified capture path. It also carries the
pass-report evidence gates so the VM-side sequence ends by summarizing the
bounded attempt, observation marker, and evidence record.
`scripts/write-qemu-visible-operator-checklist.sh` renders that packet as
`build/qemu-visible-operator-checklist.md`, keeping the same stop rule and
artifact fingerprints plus capture-hash gate checks in a human-readable form.
The checklist includes the final `aqua-qemu-visible-pass-report` VM command.
`scripts/run-qemu-visible-operator-pass.sh` is the guarded host entrypoint for
the manual pass. It refreshes and validates the status, plan, packet, and
checklist before delegating to `scripts/run-qemu-visible-manual.sh`; in
print-only mode it proves the launch path without opening QEMU.
`AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true` runs the full artifact-chain
rehearsal and exits with `no-launch-ready` instead of opening QEMU, writing
`build/qemu-visible-operator-pass.txt` and
`build/qemu-visible-operator-pass.json` as the rehearsal record.
Actual launch through the same entrypoint is blocked unless
`AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true` is set.
The pass text and JSON also carry the no-launch rehearsal command, confirmed
launch command, preflight summary path, preflight source hash/timestamp, serial
log path, capture directory, ready-capture flow, capture verifier,
capture-hash gate/status, evidence flow, VM apply command, VM pass-report
command, and the same stop rule as the operator packet.
`scripts/verify-qemu-visible-capture.sh` validates the capture file or metadata
env file, rejects SHA-256 mismatches when metadata pins a capture hash, and
prints the exact `aqua-qemu-visible-evidence-record` command for the recovery
shell.
`scripts/write-qemu-visible-evidence-bundle.sh` consumes the verified capture
metadata, validates the preflight summary JSON, and writes the exact recovery
shell command bundle for the evidence record, positive observation marker, and
pass-report step, without starting QEMU or graphics. It fails before writing the
bundle when the capture hash is not pinned and verified.
`aqua-qemu-visible-evidence-bundle-apply` is the rootfs-side recovery command
that consumes that copied bundle. It requires `preflight_summary_verified=true`
and `capture_hash_verified=true`, defaults to
`waiting-for-operator-confirmation` and only applies the positive observation
when `AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true` is explicit.
`aqua-qemu-visible-pass-report` joins the bounded visible-attempt result,
observation marker, and evidence record into one recovery-side pass report. A
positive VM display observation in that report requires both a completed bounded
attempt and recorded operator capture evidence; the report still records
`boot_graphics=false`, `autostart=false`, and no persistent desktop shell.
Host-side status, packet, checklist, and operator-pass artifacts also carry the
manual runbook `pass_report_required` gate so the final VM report cannot be
dropped from the handoff.
The rootfs contract exports separate rejection artifacts for missing preflight
verification and missing capture-hash verification, and both keep the VM display
unobserved.
`scripts/prepare-qemu-visible-evidence-apply.sh` is the host-side helper that
prints the recovery-shell heredoc for copying the bundle into `/run/aqua` and
then applying it after visual confirmation. It refuses to print that heredoc
unless the bundle already carries `capture_hash_verified=true`.
`scripts/run-qemu-visible-evidence-flow.sh` chains the host verify, bundle, and
paste-prep steps for an already captured QEMU display, without starting QEMU.

If the Rust target `x86_64-unknown-linux-musl` is installed, this also runs:

```sh
cargo check -p aqua-compositor --target x86_64-unknown-linux-musl
cargo check -p aqua-compositor --target x86_64-unknown-linux-musl --features smithay-smoke
```

Image packaging binary build:

```sh
scripts/build-compositor-linux-docker.sh
```

Rootfs packaged binary execution check:

```sh
scripts/check-compositor-rootfs-docker.sh
```

Full image validation:

```sh
scripts/check-image.sh
```

This writes `build/aqua-boot-summary.txt`, `build/aqua-boot-summary.json`, `build/aqua-image-manifest.txt`, and `build/aqua-image-manifest.json`, then records that the compositor binary is packaged but not autostarted. The manifest also includes the non-graphical boot summary, scene, asset, and shared surface-token contract status.

## Current Scope

Implemented now:

- Workspace crate boundary for `aqua-compositor`.
- CLI status output.
- Deterministic Calloop event-loop smoke test.
- Minimal Smithay Wayland display/compositor-global smoke path on Linux.
- Minimal Smithay Wayland socket lifecycle and local client insert smoke path on Linux.
- Minimal Calloop-driven socket dispatch, local client insert, `dispatch_clients`, and `flush_clients` smoke path on Linux.
- Session skeleton that owns the Wayland display and compositor state behind a narrow insert/dispatch/flush API.
- Recovery-safe session config defaults for `/run/aqua`, `aqua-wayland-0`, `/usr/share/aqua`, `autostart=false`, `boot_graphics=false`, and fallback TTY.
- Buildroot writes the same recovery-safe session contract to `/etc/aqua/compositor-session.conf`.
- `probe-session-config` can validate either the built-in defaults or the rootfs config file.
- `probe-session-env` derives the recovery-safe shell environment from the same defaults or rootfs config file.
- `probe-session-bootstrap` prepares the runtime directory from the config while proving compositor autostart, boot graphics, and desktop shell startup remain disabled.
- Boot creates `/run/aqua` for the Aqua Wayland socket; the default recovery profile does not start a graphical session, while explicit QEMU profiles do.
- Session run-once smoke that accepts one local client, dispatches, flushes, and cleans up through the session entrypoint.
- Bounded session-loop smoke that runs three dispatch/flush passes after accepting one local client.
- Manual-dev nested preview frame-loop smoke that runs a three-frame preview clock over the deterministic visible preview export without autostarting graphics.
- Static `aqua-scene` shell model and deterministic scene dump for wallpaper, top panel, desktop icons, dock, launcher, system overview, notification toast geometry, runtime asset bindings, and shared surface-token bindings.
- Real Linux Smithay `Aqua Seat` probe with a `wl_seat` global, keyboard and pointer capabilities, launcher toggle interception, pointer dispatch, application/search selection, and launcher scene/render-plan visibility. The same feature-enabled musl binary, `libxkbcommon`, and XKB rules are packaged in the Buildroot rootfs and executed by the rootfs contract check with `host_stub=false`.
- Bounded packaged QEMU input validation with two layers: a low-level evdev diagnostic probe and the shared DRM-Wayland session's libinput/udev seat0 discovery. One persistent HMP connection serves every keyboard, relative pointer, button, and screendump request through a Unix control socket. Normalized events reach Aqua Seat without stopping dispatch after cumulative probe counters are satisfied. The production path opens the launcher, dismisses a notification, promotes the FIFO queue, selects desktop icons, opens and refreshes Aqua Properties, confirms confined Trash emptying, forwards a 24-event Settings keyboard burst plus 17 host pointer commands with explicit virtio motion coalescing, captures the session menu, and completes two clean compositor cycles.
- Smithay GPU texture damage is expressed in destination-local coordinates. This prevents non-origin icon, client, overview, notification, and session textures from being clipped by a second application of the destination offset. Clean-desktop and session-menu QEMU captures cover the direct GPU overlay path; session content replaces overview content on their shared surface surface.
- The launcher state exposes distinct Applications and Global Search modes. Applications renders a centered application grid; typing transitions into Search, which renders bounded catalog results and functional Applications, Settings, and Files quick actions. Panel geometry and pointer hit testing share active-output dimensions so visual and input targets remain aligned.
- The compositor-owned bottom shell is one transparent RGBA layer containing three separated groups: Applications/Search controls, a centered Files/Settings/Trash application dock, and three workspace thumbnails. Applications and Search open their matching surfaces; app entries route through allowlisted launch requests and retain running indicators. Each mapped XDG toplevel belongs to exactly one of the three workspaces. Bottom-shell selection and `Ctrl+Alt+Left/Right` switch the active workspace, while `Ctrl+Alt+Shift+Left/Right` moves the focused window and transfers focus to the next visible surface. Rendering, pointer hit testing, raising, damage revision tracking, and cleanup are scoped to the active workspace. Repeated app activation switches to the owning workspace and raises the existing surface instead of spawning a duplicate. Dedicated Linux tests cover keyboard and bottom-shell activation; QEMU moves Files and Settings together, captures empty and populated workspaces, verifies a visible frame delta, and proves both processes survive two workspace transitions.
- Packaged `aqua-properties` first-party client with strict Files, Settings, and Trash target allowlisting, bounded filesystem metadata, a 480x300 system-surface wl_shm surface, `aqua.properties` XDG identity, and `restart_policy=never`. Its target-specific read-only F5 action refreshes folder contents or verifies the Settings binary, increments a visible generation, and commits a replacement buffer. Production QEMU verifies mapping, two-surface composition, generation 1 repaint, target metadata, clean process exit, and stale-surface removal.
- Native-resolution 512x293 session-menu content over the compositor-owned system-surface surface. The overlay leaves wallpaper depth visible and draws compact title-case actions, a layered cyan focus row, fine edge light, and a small confirmation footer. The persistent HMP QEMU run captures the resulting 1280x800 scanout before executing the confirmed Recovery action.
- Separately executed packaged Wayland test client that connects through the real QEMU session socket, completes xdg-toplevel configure/ack, commits a 384x256 wl_shm buffer, appears in KMS scanout, receives a frame callback, submits a partial damage update, gains keyboard and pointer focus, and is cleaned up before recovery return.
- Headless `aqua-renderer` plan-only layer that converts the scene into draw commands, deterministic paint steps, and an output frame contract without drawing.
- Optional recovery-safe `/usr/bin/aqua-compositor` packaging path for the Buildroot image.
- Docker execution check for the packaged rootfs binary.
- macOS host stub for the Wayland smoke command, because Smithay's Wayland frontend links `xkbcommon`.
- Runtime asset probe for the M2 `/usr/share/aqua` layout.
- Runtime design-token probe for the shared Aqua surface contract.
- Manual host-side nested output presenter probe in `aqua-host-tools`, consuming the compositor display-output handoff and nested output surface lifecycle while staying outside the Buildroot rootfs.
- Manual host-side preview window lifecycle probe in `aqua-host-tools`, feature-gated behind `host-window-preview`, using the display-output handoff composited raw rgba8888 client-layer frame and a bounded 600-frame limit.
- Manual execution window bridge in `aqua-host-tools`, proving the rootfs-safe manual execution checksum can feed the host `minifb` visible preview path without packaging host tooling into the image.
- Host/dev handoff summary in `aqua-host-tools` that pairs the QEMU recovery visible-preview launcher artifact with the host-side bounded preview smoke command.
- QEMU recovery `aqua-visible-preview-request` command that records a manual host-visible preview request after the manual execution path passes, while keeping boot graphics disabled.
- QEMU recovery `aqua-visible-preview-launch` command that turns the request into a bounded manual launch plan without opening a QEMU window, enabling autostart, or packaging host tooling.
- QEMU recovery `aqua-recovery-help` command that lists the supported manual operator commands, pass-report-required rule, and post-evidence pass-report step while keeping the shell as the active runtime.
- QEMU recovery `aqua-operator-transcript` command that writes the dry-run manual command sequence and matching host commands before any graphical boot is enabled.
- QEMU recovery `aqua-graphics-enable-gate` command that evaluates handoff/manual-execution preflight logs, writes both refused and positive dry-run manual graphics enable plans, and preserves text recovery, `autostart=false`, and `boot_graphics=false`.
- QEMU recovery `aqua-graphics-launch-candidate` command that consumes the positive dry-run gate, records rollback metadata, and still keeps actual graphics/display startup disabled.
- QEMU recovery `aqua-graphics-rollback-drill` command that consumes the no-start launch candidate, proves cancel/failure rollback metadata, and still keeps actual graphics/display startup disabled.
- QEMU recovery `aqua-graphics-startup-preflight` command that consumes the rollback drill, records bounded startup criteria, and still keeps actual graphics/display startup disabled.
- QEMU recovery `aqua-graphics-startup-rehearsal` command that consumes the guarded preflight, proves a bounded three-frame manual display-output run started and stopped, and still keeps graphical boot/autostart disabled.
- QEMU recovery `aqua-graphics-qemu-display-gate` command that consumes the startup rehearsal and records the decision to allow only a manual first visible QEMU compositor step.
- QEMU recovery `aqua-graphics-visible-qemu-attempt` command that consumes the display-step gate and records the first visible QEMU compositor attempt command without starting it.
- QEMU recovery `aqua-graphics-visible-attempt-transcript` command that records the manual visible-attempt operator sequence and expected recovery return before any persistent graphical session exists.
- QEMU recovery `aqua-graphics-visible-attempt-result` command that records the visible-attempt result contract while preserving the safe default manual-not-run state.
- QEMU recovery `aqua-graphics-visible-attempt-runner` command that explicitly executes the bounded visible-attempt wrapper and records the completed bounded result without persistent graphical boot.
- QEMU recovery `aqua-graphics-qemu-visible-boot-check` command that records the QEMU-visible boot path as ready for manual observation without enabling autostart.
- QEMU recovery `aqua-graphics-qemu-observation-marker` command that records VM-display observation state separately from autostart and graphical boot.
- QEMU recovery `aqua-qemu-visible-evidence-record` command that records operator capture metadata before any positive VM-display observation is accepted.
- Host `scripts/capture-qemu-visible-manual.sh` helper that saves a manual QEMU display capture and prints the matching recovery evidence command.
- Host `scripts/verify-qemu-visible-capture.sh` helper that validates the saved capture before the recovery evidence command is used.
- QEMU recovery positive VM-display observation dry-run artifact that records `qemu_vm_display_observed=true` only under explicit `AQUA_QEMU_VM_DISPLAY_OBSERVED=true` contract validation.
- QEMU manual VM-display runbook artifact and host `scripts/run-qemu-visible-manual.sh` entrypoint for non-Docker operator validation.
- Client window model probe for focus, move, resize, close, stacking, and shared Aqua chrome before a real Wayland client is started.
- Client surface lifecycle probe for created, configured, committed, mapped, focused, unmapped, and destroyed xdg-toplevel-style flow before a real Wayland client is started.
- Client surface registry probe for two xdg-toplevel-style records with one active focused client, one inactive mapped client, configure serials, stacking order, per-surface buffer attach/commit metadata, server-side shm import/sample checksum, and no renderer binding.
- Smithay xdg-shell binding probe for `XdgShellState`, handler, toplevel callback, and popup callback readiness before a real Wayland client is started.
- Minimal xdg-toplevel client probe that drives two `wl_compositor` and `xdg_wm_base` clients into Smithay, records server-side toplevels, and exits without rendering.
- Recovery-safe DRM discovery that opens `/dev/dri/card0` read-only and records QEMU's connected `Virtual-1` connector and advertised modes without acquiring DRM master or activating KMS.
- Bounded QEMU DRM dumb-buffer probe that allocates and maps a 1280x800 Xrgb8888 buffer, copies the composited Aqua frame with the reported pitch, verifies its checksum, destroys the buffer, and returns to recovery without creating a KMS framebuffer or submitting a page flip.
- Local temp-root validation through the Buildroot post-build script.
- Optional Linux-target Rust check for the real Smithay path.

Not implemented yet:

- Broader third-party toolkit coverage beyond the packaged upstream
  `weston-simple-shm` xdg-toplevel compatibility fixture.
- Additional Aqua-owned icon artwork for later first-party applications.
- Runtime visual convergence with the canonical Aqua visual/UI contracts.
- MSI Sword 17 DRM, input, suspend, networking, audio, and storage validation.

## Aqua Terminal (Milestone 8 complete)

- `aqua-terminal` is packaged as a first-party `aqua.terminal` xdg-toplevel and uses the existing strict launch preflight, duplicate rejection/raise behavior, process supervision, and stale-surface cleanup.
- `portable-pty` owns a real `/bin/sh` pseudo-terminal while `vt100` provides terminal escape parsing and a bounded 1,000-line scrollback buffer.
- The renderer provides shared window-chrome geometry, title-bar primitives, and bounded window and shell palettes. Files, Settings, Terminal, Properties, and all Installer setup screens load the persisted LightWhite, Softtouch, Deepside, or Nightmare palette when launched. The compositor loads the same selection at session startup for the top bar, Applications, Search, three-group bottom shell, desktop icons and context menu, system overview, session menu, and notifications. Both CPU fallback and GPU texture paths consume the palette, and themed textures replace the old generic system-surface shading. LightWhite remains the fallback for legacy or invalid settings. Terminal content keeps a dark readable monospace scrim under every frame palette, while Installer preserves semantic success, warning, and destructive colors. The running desktop and first-party clients poll the atomically persisted selection at a bounded 100 ms interval; a real change invalidates Shell texture caches and redraws open Files, Settings, Terminal, Properties, and Installer Wayland buffers without restarting their processes. Identical selections do not redraw. Client redraws remain frame-coalesced so input bursts do not create one expensive QEMU GPU repaint per key.
- The shared `aqua-text` service shapes renderer text with Rustybuzz before
  rasterization. It resolves Unicode bidi runs, preserves grapheme boundaries
  for wrapping and ellipsis, defines caption, body, control, title, display,
  and monospace roles, and rasterizes glyph IDs into a bounded cache keyed by
  role and the supported 1.0, 1.25, 1.5, or 2.0 output scale. The embedded Noto
  Sans face is followed by a packaged Noto Sans Arabic face in a deterministic
  fallback order. Contiguous graphemes assigned to the same face are shaped
  together, fallback keeps the role baseline and control height stable, and
  unsupported glyphs remain explicit diagnostics. A committed 16-case fixture
  report locks Latin ligatures, Turkish text, combining marks, mixed bidi text,
  font selection, bounds, baselines, and all four scales. A separate deterministic
  renderer acceptance matrix covers 800x600, 1280x800, and fractional-scale
  1536x1024 output in every theme. It clips shaped glyph pixels to their layout
  bounds, keeps critical installer actions untruncated, contains a long Turkish
  accessibility label, exercises Arabic fallback without missing glyphs, and
  locks all 12 RGBA checksums in `typography-layout-fixtures.txt`. These are
  host-rendered fixtures. The packaged `aqua.typography-acceptance` wl_shm
  client presents the accepted 1280x800 Turkish and Arabic layout through the
  real Smithay/DRM path without shell chrome. A single recovery-safe QEMU boot
  captures LightWhite, Softtouch, Deepside, and Nightmare as four distinct
  nonblank PNGs; each bounded session closes its client, restores the CRTC,
  releases scanout resources, and returns to the recovery shell.
- The graphical QEMU acceptance run captures LightWhite and Deepside frames with Files and Settings open. It requires the Shell broadcast and both client redraw markers, unchanged application PIDs, an increased compositor repaint sequence, and a visible pixel delta before accepting the live switch.
- Resize updates the kernel PTY dimensions and VT parser grid together. The packaged `aqua-terminal --probe-pty` path executes a shell command and validates resize without requiring a display.
- Aqua `rcS` mounts `devpts`, exposes `/dev/ptmx`, and emits `stage=devpts-ready`; this is OS runtime infrastructure rather than a host-only test dependency.
- QEMU confirms the shell prompt in a normal Aqua window. Terminal launches omit the redundant application-opened notification, and the compositor caches client revisions after frame presentation so a delivered frame callback does not force a false follow-up repaint.
- Desktop-only system overview content now yields while a client window is mapped, keeping the terminal and other application surfaces unobscured; the session menu can still promote that shell region explicitly.
- The QEMU acceptance path waits for a stable repaint queue, forwards all 40 press/release events for `echo aquaterminalok`, observes Enter, verifies parsed VT100 output, repaints the result, closes with Alt+F4, reaps the process, and confirms the first-party `never` restart policy.
- Milestone 8 acceptance covers typed command execution and Alt+F8 resize through a real xdg configure, including a 640x478 buffer, 74x21 parser grid, PTY resize, repaint, and clean close. Pointer titlebar move and bottom-right resize requests are implemented; broader key and clipboard support remain later enhancements.

## Developer Probe Index

The public README intentionally describes supported workflows rather than internal
probe names. These identifiers remain documented here for implementation and CI
traceability:

- `qemu-visible-manual-preflight.json`: structured result from the manual visible-QEMU preflight.
- `export-visible-preview-html`: host-only HTML preview export used during renderer inspection.
- `probe-launcher-model`: deterministic launcher state and application-model probe.
- `probe-launcher-input-scene`: launcher input-to-scene integration probe.
- `probe-smithay-launcher-seat`: real Smithay seat and launcher input-routing probe.
- `scripts/check-smithay-seat-docker.sh`: Linux dependency and non-stub seat validation.

## Foundation Decision

The product target remains a custom Wayland compositor. Smithay is selected and pinned in `compositor-foundation.toml`. The default binary is recovery-safe; the real Smithay path is behind the `smithay-smoke` feature with `default-features = false` and the `wayland_frontend`, `backend_libinput`, and `udev` features. Buildroot provides the libinput/udev target sysroot used by the canonical Docker cross-build. DRM/KMS remains implemented through the bounded Aqua backend while X11 and Vulkan feature groups stay disabled. Calloop remains the deterministic event-loop dependency.

The skeleton keeps M3 moving without creating a fake desktop or starting a graphical session before runtime assumptions are verified. The current socket smokes bind temporary sockets, accept one local client, insert it into the Wayland display, and drop the socket. The bootstrap probe prepares the runtime directory and validates the config-derived environment without starting the compositor. The rootfs manual launch plan packages those checks behind `aqua-compositor-manual-launch` for QEMU recovery use and still keeps display output stopped. The guarded run command adds the first recovery-shell bounded run by chaining the launch plan into `smoke-display-output`, then proving the three-frame nested-dev output stopped and fallback TTY remains available. The handoff gate validates that bounded run, the display-output handoff, visible preview readiness, and nested preview loop before allowing manual operator promotion; it still blocks automatic promotion. The run-once variant routes the same accept/dispatch/flush path through `AquaCompositorSession`; it exits immediately and does not run a compositor service. The bounded loop variant repeats dispatch/flush for three iterations so the session lifecycle can grow toward a real loop without becoming an autostarted desktop process yet. The display-output handoff and host nested presenter probes prove the composited frame and full-buffer client snapshots can be handed to a manual nested output path without starting display output. The nested preview loop adds a bounded frame clock over the visible preview export path; it proves manual preview lifecycle timing before a real host window backend is enabled. The client window model, surface lifecycle, registry, renderer surface source, client-layer pipeline, xdg-shell binding, and xdg-toplevel client probes start M5 deliberately: they fix the focus/move/resize/close/stacking/chrome, created/configured/committed/mapped/focused/unmapped/destroyed, two-client active/inactive registry with wl_shm buffer attach/import/sample metadata, renderer-facing source plan, client-layer paint/raster contract, Smithay handler/global, and two recorded server-side toplevel window-model entries without display output or boot graphics.

`aqua-scene` now holds the static shell scene model that the renderer consumes. It validates visual-reference surfaces, layout geometry, runtime asset bindings, and shared design-token bindings. `aqua-renderer` also exposes paint-plan, frame-plan, software framebuffer, software raster, and PPM export contracts for draw order, opacity, blend mode, legacy effect compatibility, rgba8888 format, stride, buffer size, clear color, sample pixels, full-buffer checksum, and inspectable image artifacts before display output is enabled.

Decision record:

`adr-0001-compositor-foundation.md`
