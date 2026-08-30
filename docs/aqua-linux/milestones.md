# Aqua Linux Milestones

This roadmap defines the staged delivery of Aqua Linux, a Buildroot-based
graphical Linux distribution with a custom Wayland compositor, a bright
Aqua-owned desktop interface, and Chromium OS-like product simplicity.

## Fixed Decisions

- Product: Aqua Linux.
- Not a theme pack or desktop remix.
- OS base: Buildroot.
- Graphics architecture: custom Wayland compositor.
- Development target: QEMU x86_64 first.
- Hardware validation target: MSI Sword 17 later.
- Visual target: the public visual/UI contracts derived from private, Git-ignored desktop and installer boards.
- Desktop composition: compact top bar; separate Applications/Search controls; centered pinned/running app dock; bottom-right workspace switcher.
- Desktop scope: simple, polished, first-party surfaces before broad app/platform complexity.

## Active Implementation Priority

From 2026-08-27, the bright Aqua interface contract is the permanent v1 direction. Aqua remains custom compositor work and does not use GNOME Shell, Mutter, GTK desktop components, KDE Plasma, or an existing desktop session.

## Milestone 0: Repository And Build Skeleton

Goal: create the project foundation.

Tasks:

- Create Rust workspace.
- Add Buildroot external tree.
- Add QEMU run script.
- Add image build script.
- Add docs folder with product plan, visual reference, architecture, and milestone docs.
- Add asset folder for source handoff.
- Add basic CI or local check script.

Expected output:

- Repo builds a minimal host-side Rust binary.
- Buildroot starts from a known defconfig.
- QEMU command can boot a minimal image or documented placeholder.

Done when:

- `cargo fmt --check` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes for existing Rust code.
- `cargo test --workspace` passes.
- Minimal Buildroot image build path is documented.

## Milestone 1: Buildroot Boot To Text Recovery

Goal: prove the OS image boots reliably before graphics.

Tasks:

- Build Linux kernel with virtio storage/network and framebuffer console support.
- Use GRUB or selected bootloader.
- Boot BusyBox or selected init.
- Mount `proc`, `sysfs`, `devtmpfs`, `/tmp`, and `/run`.
- Provide fallback shell on TTY.
- Emit clear boot markers to serial log.

Expected output:

- QEMU boots to recovery shell.
- Serial log proves boot success.

Done when:

- `scripts/build-image.sh` or equivalent creates `disk.img`.
- `scripts/run-qemu.sh` boots the image.
- Serial log contains stable boot success markers.
- Recovery shell is available.

## Milestone 2: Asset And Visual System Intake

Goal: lock the Aqua Linux visual language before compositor rendering work expands.

Tasks:

- Preserve and index private desktop and installer boards in the Git-ignored local reference library.
- Import wallpaper, logo, app icon, boot mark, and first icon set.
- Define surface, color, typography, spacing, radius, elevation, motion, icon,
  and component-state tokens.
- Record text shaping, font fallback, scale-native rasterization, icon export,
  reduced-motion, and visual-fixture requirements.
- Create a static visual spec from the reference desktop.
- Mark any mock values as mock.

Required design coverage:

- Desktop workspace, applications overview, global search, terminal, calendar,
  photos, files/trash, idle desktop, and settings.
- Installer welcome/language, keyboard, region/date, installation mode,
  account, summary, theme selection, completion, and restart states.

- Official Aqua wallpaper collection: `assets/wallpaper-*.png`; current default source is `assets/wallpaper-sunlit-water.png` through `assets/default-wallpaper.png`.
- Primary dark symbol: `assets/aqua-symbol-primary.png`.
- Inverse symbol for dark surfaces: `assets/aqua-symbol-inverse.png`.
- Accent symbol for active and focused states: `assets/aqua-symbol-accent.png`.
- Two-line wordmark: `assets/aqua-wordmark-primary.png`.
- Combined symbol and wordmark: `assets/aqua-logo-primary.png`.
- Aqua app icon.
- Home, Files, Aqua Drive, Trash icons.
- Applications/search controls and Aqua-owned application icons for the first grid and dock.
- Status icons: Wi-Fi, volume, battery.
- Notification/update icon.

Expected output:

- Asset tree is organized and documented.
- Visual reference can be rendered or previewed outside the booted OS.

Done when:

- All required MVP assets exist in source form.
- Runtime export requirements are documented.
- Visual tokens are committed.
- Shared component/material contracts can account for every canonical screen without treating private reference artwork or third-party icons as runtime assets.

## Milestone 3: Nested Aqua Compositor Prototype

Goal: run the compositor in a developer environment before booting it in the OS image.

Tasks:

- Create `aqua-compositor` crate.
- Use `smithay` or selected Wayland compositor foundation.
- Run nested on the development host.
- Render wallpaper, cursor, top bar, Applications/Search controls, dock, workspace switcher, and notification toast.
- Use mock system data where necessary and label it.

Expected output:

- Host-side compositor preview opens.
- Static shell resembles the reference desktop.

Done when:

- Screenshot comparison shows the main layout is recognizable.
- No real app/window support is required yet.
- Input can at least move the cursor or close the preview.

## Milestone 4: Scene And Surface Renderer

Goal: build reusable rendering primitives for Aqua surfaces.

Tasks:

- Add `aqua-scene` or equivalent module.
- Implement shared panel and window primitives.
- Implement fill, border, separator, shadow, and rounded clipping.
- Implement tokenized elevation levels with bounded shadow damage and reusable
  masks or textures.
- Add optional low-cost translucency without making blur a dependency.
- Add shaped text layout, fallback, scale-native glyph caching, text contrast,
  and focus-ring rules.
- Add reviewed SVG loading, symbolic recoloring, scale-native icon
  rasterization, fallback diagnostics, and bounded icon caching.
- Add frame-callback-driven, interruptible state transitions with a
  deterministic reduced-motion path.
- Add progress bars, list rows, search field, dock shelf, and notification surface primitives.

Expected output:

- Shell surfaces use shared material primitives rather than one-off drawing.

Done when:

- Top bar, applications, search, dock, workspaces, windows, and toast use the same component system.
- Optional effects are documented and do not define acceptance.
- Text, icons, elevation, and motion pass deterministic fixtures at supported
  integer and fractional output scales.

Current implementation note:

- The native GLES path caches textures by real surface commit revision and may omit hidden wallpaper work only when the client supplies a complete standard Wayland opaque region over the reference output. The first frame still validates GLES composition; later installer frames use the QEMU dumb-buffer bridge. Packaged acceptance on 2026-08-22 recorded 30 bridged frames at a 61 ms median total, down from the earlier 10.3-13.5 second frame range, without reducing the 1280x800 output or changing the physical GBM path.

## Milestone 5: Basic Wayland Client Support

Goal: make the compositor a real desktop foundation.

Tasks:

- Launch and display a simple Wayland client.
- Handle keyboard and pointer focus.
- Bind standard clipboard and primary-selection ownership to keyboard focus;
  reject replacement attempts from unfocused clients without exposing a
  privileged data-control protocol.
- Transfer clipboard and primary-selection payloads directly between clients
  through the negotiated standard MIME type, clear dead owners, and keep the
  compositor out of the payload data plane.
- Route standard drag-and-drop through an implicit pointer grab with bounded
  MIME/action negotiation, direct client transfer, target-only drop, and
  cancellation.
- Route text-input v3 through keyboard focus and expose input-method v2 only to
  an explicitly authorized client, with bounded UTF-8 state and popup geometry.
- Support move, resize, close, and basic stacking.
- Add Aqua window chrome.
- Keep compositor stable when a client exits.

Expected output:

- A basic Wayland app window appears inside the Aqua desktop.

Done when:

- At least one standard Wayland client can open.
- The user can focus, move, resize, and close it.
- Crash or client exit does not take down the compositor.

Current compatibility extension:

- Two independent Linux Wayland clients now prove clipboard and primary
  ownership rejection, acceptance, focus handoff, and offer visibility through
  the packaged compositor contract. The same clients negotiate UTF-8 text,
  reject an unsupported MIME choice, transfer exact clipboard and primary
  bytes through file descriptors, and receive cleared selections after the
  owner disconnects.
- A separate two-client Linux probe rejects drag start without an implicit
  pointer grab, routes enter/drop only to the pointer-focused target while
  preserving keyboard focus, negotiates UTF-8 text and `copy`, transfers an
  exact bounded 28-byte payload directly, completes the accepted source, and
  cancels a rejected second drop without target delivery. The packaged rootfs
  runs the same feature-enabled probe with `host_stub=false`.
- A three-client Linux probe exposes text-input v3 to two normal clients while
  hiding input-method v2 from both and publishing it only to an authorized
  input-method client. It proves focus-bound activation, stale-client
  rejection, surrounding/content/cursor state forwarding, synchronized serials,
  Turkish UTF-8 preedit and commit delivery, deletion, focus handoff, and
  parent-bound popup repositioning. The packaged rootfs runs the same probe
  with `host_stub=false`; the broader keyboard and locale matrix remains open.

## Milestone 6: Boot Aqua Compositor In QEMU

Goal: boot the real graphical Aqua desktop from the Buildroot image.

Tasks:

- Package compositor and assets into Buildroot image.
- Add session startup.
- Configure runtime directories and environment variables.
- Start Aqua compositor after boot.
- Keep fallback TTY available.
- Capture QEMU screenshot.

Expected output:

- QEMU boots into Aqua desktop, not text fallback.

Done when:

- QEMU screenshot shows wallpaper, top bar, Applications/Search controls, dock, workspaces, and target surfaces.
- Serial log proves compositor startup.
- Fallback shell remains reachable.

## Milestone 7: First-Party Shell Surfaces

Goal: turn the static desktop into usable first-party OS surfaces.

Tasks:

- Implement app launcher interactions.
- Apply the documented bright Aqua hierarchy through shared surface and component tokens.
- Implement settings surface MVP.
- Implement file manager surface MVP.
- Implement notification center/toasts.
- Implement system overview data providers.
- Implement session menu: lock, restart, power off.

Expected output:

- Aqua desktop can perform basic OS navigation without relying on terminal-first workflows.

Done when:

- Launcher opens apps/surfaces.
- Applications and global search selection, focus, pressed states, results, and opening motion are visually and functionally coherent.
- Settings and file manager MVPs open.
- Notification toast can be generated and dismissed.
- System overview displays real clock and at least one real metric.

## Milestone 8: Terminal App

Goal: provide terminal access as an app, not the desktop identity.

Tasks:

- Add `aqua-terminal`.
- Use a mature terminal emulation library.
- Connect to PTY.
- Support text input/output, resize, copy/paste later.
- Use Aqua window chrome and readable terminal scrim.

Expected output:

- Terminal opens as a normal Aqua app window.

Done when:

- Shell prompt appears inside terminal app.
- Commands execute.
- Resizing does not corrupt the terminal.

## Milestone 9: Graphical Installer MVP

Goal: install Aqua Linux through a graphical flow.

Current checkpoint:

- The `aqua-installer` crate implements the canonical nine-step state model, required-selection gates, dry-run/real mode separation, validated target identity, and exact target-bound destructive confirmation.
- A bounded read-only Linux storage probe inventories whole disks, excludes partitions and pseudo devices, and blocks the running root disk, read-only media, and zero-capacity targets.
- A deterministic dry-run plan records target identity, labeled GPT/EFI/ext4 layout, artifact destinations, system configuration, GRUB2 x86_64 UEFI installation, and a stable fingerprint while keeping execution disabled.
- GRUB2 is selected for x86_64 UEFI and Buildroot produces the removable-media `bootx64.efi` artifact; systemd-boot and first-pass BIOS installation are out of scope.
- Buildroot packages and post-image validates `sfdisk`, FAT/ext4 formatters, GNU tar, mount, and umount; the installer validates their executable paths and revalidates exact target identity against a fresh bounded probe.
- The canonical plan compiles to eight bounded `program + argv` specifications with no shell interpolation. A non-executing runner records the sequence while kernel, bootloader, system configuration, and target revalidation remain explicit deferred Rust operations.
- Eleven target-confined Rust actions now define mountpoint preparation, atomic kernel/EFI copies, generated GRUB configuration, and atomic locale, keyboard, timezone, and password-free first-user metadata writes. Their rehearsal runner performs no filesystem writes.
- A fingerprint-bound 20-step transaction graph interleaves target revalidation, commands, and internal actions. Failure injection proves conditional EFI-then-root cleanup ordering while normal rehearsal avoids redundant cleanup; all execution remains disabled.
- A root-remapped fixture executor performs the 11 internal actions only inside an empty system-temporary directory. Real atomic copies/writes pass byte/content checks, while symlink roots, path components, and artifact sources fail closed and disk commands remain disabled.
- A temporary-root tool-shim runner executes exact argv/stdin with a cleared environment, bounded timeout, exit-status handling, and transaction cleanup propagation. Programs outside its capability root and symlinks fail closed; real Buildroot disk tools remain unexecuted.
- The packaged `aqua-installer-probe` runs from the Buildroot recovery shell. QEMU proves that the only attached system disk is blocked, no install target is selected, all six tool paths validate, and the synthetic non-executing readiness path compiles and rehearses 13 operations, eight commands, 11 internal actions, and 20 transaction steps without disk commands or filesystem writes.
- A second QEMU contract attaches one writable 4 GiB disposable disk without a snapshot layer. The storage probe blocks `/dev/vda`, binds only eligible `/dev/vdb` to the non-executing plan, and verifies the qcow2 SHA-256 is identical before and after the guest run.
- A QEMU-only execution gate now requires the kernel opt-in, exact operator enable value, three non-empty regular staged artifacts, one freshly revalidated eligible disk, and the exact `ERASE /dev/vdb` confirmation. QEMU rejects a mismatched confirmation, authorizes only the complete conjunction, starts no transaction, and proves the disposable qcow2 SHA-256 remains unchanged.
- The real Buildroot `rootfs.tar`, `bzImage`, and `bootx64.efi` outputs are assembled into a dedicated ext4 staging disk with a three-entry SHA-256 manifest. QEMU attaches it read-only as blocked `/dev/vdc`; both the recovery shell and the Rust gate verify every artifact digest while `/dev/vdb` remains the only eligible target. Host-side before/after hashes prove neither disk changed.
- A second exact transaction opt-in unlocks execution only for disposable QEMU `/dev/vdb`. The executor writes a sector-defined GPT, FAT EFI and ext4 root filesystems, extracts the real Buildroot rootfs, atomically installs kernel/EFI/GRUB/configuration files, and unmounts cleanly. Host hashes prove only the target changed.
- QEMU verifies labels and installed content, then boots the installed `/dev/vda2` root with the external development kernel to the recovery marker. A separate post-EFI-mount failure injection proves cleanup completes in EFI-then-root order, clears both mountpoints, preserves readable filesystems, and leaves the read-only artifact disk unchanged.
- The fresh installed qcow2 now boots without QEMU's external kernel: EDK2 discovers the standard fallback EFI application, GRUB 2.12 loads the installed kernel, the kernel resolves `root=PARTLABEL=AQUA_ROOT`, and recovery confirms EFI runtime presence.
- A transaction-bound progress model maps all 20 real steps to stable UI phases and monotonic percentages. The packaged executor emits live running, failed, and completed records; QEMU acceptance fixes successful completion at 100% and the injected EFI-mount failure at its last completed step rather than showing false completion.
- The graphical window contract now defines responsive 800x600, 1280x800, and 1536x1024 geometry for the installer window, title bar, step rail, content, footer, controls, and progress track. Canonical Turkish labels plus Tab/Shift+Tab/directional/Home/End/Enter/Escape focus behavior are tested across editable, installation, and completion states; the recovery-only probe remains intentionally headless.
- `aqua-renderer` produces a deterministic 1280x800 PNG for the Welcome step using the shared layout, synchronized installer/UI state, embedded Noto Sans text, current legacy bright surfaces, focus state, and the current transparent Aqua symbol. The same raster now appears through the packaged Wayland client in QEMU; its styling must be converged on the new interface contract.
- Language and Keyboard use bounded three-entry catalogs, model-validated activation, and real raster form rows with selected and applied states. Deterministic renders and packaged QEMU navigation cover these surfaces.
- The Partitions form consumes the bounded storage inventory, skips blocked devices during keyboard navigation, and applies an install target only through the existing eligible-candidate conversion. Its raster shows device identity, capacity, destructive warning, and the canonical EFI/root layout; a fourth deterministic installer PNG now covers this step.
- Time Zone now uses a bounded IANA catalog with keyboard selection, model synchronization, and a real selected/applied raster screen. The fifth deterministic installer PNG covers İstanbul, UTC, Berlin, and New York choices while preserving the existing installed timezone metadata path.
- User Information now provides bounded username/display-name editing and accepts only a password-configured status from the future secure input component; password characters are explicitly ignored and never enter state, model, raster, or metadata. The sixth deterministic installer PNG shows the applied profile without password content.
- Summary now renders all validated selections and the real/dry-run mode as a seventh deterministic installer PNG. Real mode first consumes the shared target-bound checkbox acknowledgement and then accepts the exact current-target confirmation phrase; target or mode changes invalidate readiness, and the recovery-safe Buildroot probe exports the same contract without granting execution authority.
- A packaged `/usr/bin/aqua-installer` opens as a real `aqua.installer` wl_shm xdg-toplevel, loads the canonical symbol and real Linux storage inventory, and remains manual with live transaction execution disabled by default. QEMU proves the surface fills the 1280x800 DRM output without desktop chrome, applies Turkish locale, Turkish Q, `/dev/vdb`, Europe/Istanbul, bounded username/display name, password-configured status, and exact target-bound `ERASE /dev/vdb` confirmation. Password characters are not accepted or logged.
- An acceptance-only presentation rehearsal now compiles the confirmed model into the canonical 20-step graph, proves the non-executing runner reports `executed=false`, renders graph-bound 40%, 65%, and 95% Installation states, reaches Completed only at 20/20 and 100%, and captures both final live Wayland surfaces. The full packaged path records 106 forwarded key events, real virtio pointer footer and form clicks, target-bound destructive acknowledgement, exact uppercase confirmation, 35 client rerenders, and seven distinct QEMU screendumps without dispatching disk commands.
- Responsive footer pointer hit-testing now covers Language, Cancel, Back, Forward/Install, and Restart. It rejects controls hidden by the active installer step and routes accepted Wayland button presses through the same `InstallerUiAction` and model validation path as keyboard activation.
- Choice, disk, and user-field rows now share exact responsive rectangles between `aqua-renderer` and installer pointer hit-testing. Bounded catalog clicks update validated selections, blocked disks reject hits, eligible disks remain unapplied until explicit activation, and field clicks cannot create a user profile.
- Packaged QEMU acceptance proves the pointer path end to end. The installer full-output input origin is `(0,0)`, a real virtio mouse advances Welcome and selects the Turkish Language row, repaint synchronization is required between clicks, and keyboard navigation then completes the transaction rehearsal without relaxing any execution gate.
- Password contents are never stored in the model or installed metadata.

Tasks:

- Restyle the window, step rail, content region, navigation bar, language control, and progress states with the shared bright Aqua component system.
- Disk selection.
- Partition confirmation.
- User creation.
- Timezone and keyboard selection.
- Copy root filesystem.
- Install bootloader.

Expected output:

- Installer can perform dry-run and real install paths with explicit confirmation.

Done when:

- Dry-run prints or displays exact plan.
- Real install requires explicit destructive confirmation.
- Installed disk boots in QEMU.
- Every installer stage remains keyboard navigable, readable, and visually consistent with the shared interface contract at the supported QEMU viewport.

## Milestone 10: MSI Sword 17 Hardware Validation

Goal: test Aqua Linux on the real target machine.

Tasks:

- Keep the public support matrix explicit about QEMU evidence and physical
  hardware unknowns.
- Identify CPU, GPU, Wi-Fi, audio, touchpad, keyboard, storage, and display hardware.
- Confirm UEFI boot path.
- Decide Secure Boot posture.
- Add required kernel drivers and firmware.
- Test display output, input, network, audio, suspend/resume, and battery.
- Keep recovery boot option.

Expected output:

- Aqua Linux boots on MSI Sword 17.
- Current status before physical testing: `hardware-support.md` records every
  MSI area as not tested and prohibits a physical support claim.

Done when:

- Internal display works.
- Keyboard and touchpad work.
- Wi-Fi or Ethernet works.
- Aqua compositor starts.
- Recovery path works if graphics fails.

## Milestone 11: Polish And Public Readiness

Goal: prepare the project for public code release and outside contributors.

Tasks:

- Converge desktop, Applications, Search, Terminal, Files, Settings, installer, window chrome, and shared controls on the canonical visual/UI contracts.
- Expand the permanent Aqua Core Icons as new first-party features require them.
- Validate real localized clock/date, account, status, disk, and progress values; no mock values may appear in runtime acceptance captures.
- Capture desktop, Applications, and Search at desktop and compact supported viewports and compare composition against the canonical references.
- Rename remaining legacy Espresso components where practical.
- Write contributor guide.
- Write build and QEMU guide.
- Write hardware support status.
- Add screenshots.
- Add issue labels and roadmap.
- Add license.
- Add code of conduct if desired.

Expected output:

- New developers can understand the goal, build the image, run the compositor, and find useful first tasks.

Done when:

- Visual acceptance records each canonical screen as pass, known deviation, or explicitly deferred behavior.
- The bottom-left controls open Applications and Search as separate surfaces.
- The centered bottom group is a permanent pinned/running application dock.
- The bottom-right workspace group remains visible on the idle desktop.
- Fresh checkout instructions work.
- QEMU demo can be reproduced.
- Known missing features are documented honestly.
- Public README matches the actual state of the code.

## Milestone 12: Visual Fidelity And Component System

Goal: replace prototype-grade drawing with a production-quality visual and
interaction foundation shared by the compositor and first-party applications.

### Workstream 1: Advanced Typography

- Introduce one shared Unicode shaping and text-layout path.
- Support kerning, ligatures, combining marks, bidirectional runs, Turkish
  case behavior, deterministic fallback, grapheme-safe wrapping, and ellipsis.
- Cache glyphs by font identity, glyph, rendering mode, and output scale.
- Rasterize from source metrics at 1.0, 1.25, 1.5, and 2.0 scales.
- Validate named caption, body, control, title, display, and monospace roles.

Done when shaped fixture text has stable baselines and bounds in all four
themes and supported scales, long localized labels do not overlap critical
actions, and shell and first-party applications use the same text service.

### Workstream 2: Elevation And Shadow Quality

- Implement control, panel, dialog, and active-window elevation tokens.
- Use reusable shadow masks or textures keyed by geometry, scale, theme, and
  elevation.
- Expand compositor damage by the full shadow extent and preserve correct
  rounded clipping and stacking.
- Measure isolated and overlapping surface rendering against the documented
  frame budget.

Done when deterministic captures show consistent neutral depth without glow,
corner seams, clipping, or viewport-edge artifacts in all four themes.

### Workstream 3: Scalable Icon Processing

- Treat reviewed Aqua Core SVG files as canonical masters.
- Rasterize requested logical sizes directly for each output scale; never
  enlarge a smaller cached bitmap.
- Support symbolic theme/state coloring, full-color application icons, alpha,
  source revision invalidation, and one documented diagnostic fallback.
- Validate 16, 20, 24, 32, 48, 64, and 128 pixel roles at integer and
  fractional scales.

Done when every shipped icon role has reviewed provenance and remains crisp,
aligned, identifiable, and layout-stable in each theme and interaction state.

### Workstream 4: Detailed State Motion

- Implement semantic duration and easing tokens for feedback, panels, menus,
  windows, workspaces, notifications, progress, and attention.
- Drive motion from compositor frame callbacks and allow interruption and
  reversal from the currently rendered value.
- Stop hidden repeating motion and provide a deterministic reduced-motion path.
- Keep input targets and component geometry stable throughout transitions.

Done when start, midpoint, completion, interruption, reversal, and
reduced-motion tests pass without jumps, stuck input, or unbounded timers.

### Workstream 5: Complete Design Components

- Create a catalog for every shared component named in
  [interface-style.md](interface-style.md).
- Record anatomy, content bounds, token dependencies, keyboard/pointer
  behavior, accessibility semantics, and applicable state matrix.
- Remove screen-specific copies once the equivalent shared primitive passes.
- Add deterministic fixtures for all themes, supported scales, compact and
  desktop viewports, localization expansion, and applicable states.

The implementation inventory and extraction order are maintained in
[component-catalog.md](component-catalog.md). Top system bar, window frame,
menu, metadata row, section group, standard button, icon button, search field,
checkbox, switch, segmented control, toolbar, list row, grid cell, application overview,
global search, running-app dock, workspace switcher, notification, confirmation
dialog, sidebar navigation, and slider are packaged-QEMU-proven across their applicable states,
four themes, three required
viewports, and a fractional scale. Installer,
Applications, Global Search, Terminal, Properties, Settings, and Files consume
the applicable primitives; the Applications surface composes the shared search
field and grid cells through one bounded overview contract, Global Search
composes its real split results and quick actions through a separate bounded
contract, and Files correctly remains on its real list-row path. Shared geometry
drives both renderer and input routing; the centered dock additionally shares
its item, icon, running-indicator, and pointer geometry, while the workspace
group shares its target, thumbnail, active-indicator, and pointer geometry.
Notification rendering and compositor dismissal now share the toast content,
icon, close-control, pointer, keyboard, and live-region geometry while retaining
the existing bounded queue, timeout, and motion model.
Session, Empty Trash, and Installer real-mode confirmation presentation now
share compact/detailed geometry, explicit repeat-activation versus exact-text
requirements, intent-only keyboard semantics, and alert accessibility. Their
existing action authorization, target identity, and execution gates remain
model-owned.
Settings Audio now supplies the real bounded slider consumer: a persistent
0–100 output-volume preference and mute state backed by the fail-closed
`aqua-service-adapters` contract. Controls require a ready authoritative state
with a valid output route; `/dev/snd` alone is insufficient, and desired values
are not reported as applied before reconciliation. Settings now exposes explicit
unavailable, starting, degraded, applying, and applied control states. Slider and
mute input are enabled only in the applied state; applying renders the last
authoritative value while retaining the desired preference separately, and a
degraded transition blocks further input without discarding that preference. It
also bounds native control submission to three failed attempts for one graph
generation, then exposes degraded and waits for a newer synchronized generation
before retrying. Deterministic adapter and Settings tests prove the retained
preference, blocked fourth submission, generation-gated recovery, and final
acknowledgement. A packaged QEMU probe now verifies the same boundary through
the production native bridge: one unchanged authoritative graph keeps a stable
generation across three rejected calls, the fourth call never reaches the
bridge, and a real graph change advances the generation before a final
acknowledgement. This is virtual runtime evidence and deliberately does not
claim physical hardware support. The typed PipeWire/WirePlumber
transport maps synchronized graph snapshots and typed control calls into the
same acknowledgement gate. The non-default Buildroot profile now packages the
bounded native WirePlumber 0.5 bridge and proves its dependency and legal-info
closure. Its ordered audio-only overlay enables the per-user supervisor, pins
the exact stack manifest, and passes a fail-closed checker against the complete
Buildroot rootfs artifact without changing the default image. The audio-only
Intel HDA kernel fragment and declared QEMU device now prove sink/source-node
discovery, 48 kHz stereo S16LE output into a non-silent WAV capture,
volume/mute, bounded WirePlumber recovery, and a separate controlled
4,800-frame zero-PCM input stream. A two-controller output profile additionally
proves native configured-default switching, effective-route acknowledgement,
and non-silent playback on both independent WAV backends. A selected-output
removal profile additionally requires QMP deletion acknowledgement, the ALSA
topology to shrink from two devices to one, the native graph to expose the
remaining output as authoritative default, and non-silent fallback playback.
A complementary non-default removal profile proves PCI 04.0 is authoritative
before PCI 05.0 is removed, requires acknowledged QMP and one-device ALSA/native
topology convergence, and preserves the same default with non-silent playback
before and after removal.
An active variant publishes a 480-frame checkpoint on PCI 04.0 before removing
the non-default controller and requires that same client to finish 48,000 frames
without interruption or recovery while the authoritative route stays unchanged.
A selected-route active variant instead removes authoritative PCI 05.0 after
480 frames. Because compatibility PCM writes alone can remain successful after
the route disappears, the probe uses the native topology to report explicit
`route-loss`, forbids false playback completion, and requires a new client to
complete 48,000 non-silent frames on fallback PCI 04.0.
A private QEMU D-Bus input profile additionally injects a deterministic 1 kHz
bipolar signal without host microphone access and proves an exact 4,800-frame
capture through HDA, ALSA, and PipeWire with a 4,096 peak and balanced sample
polarity. A separate bounded failure profile serves 9,600 bytes of one-polarity
PCM before rejecting later D-Bus reads; the guest refuses the retained buffer
as a valid bipolar signal while the media services and recovery shell stay
responsive. An active playback failure profile additionally writes 480 frames,
terminates the owning PipeWire process, requires the client to report
`Broken pipe` without false completion, proves bounded ordered service restart,
and verifies a new 48,000-frame non-silent playback. A complementary active
policy-service profile terminates WirePlumber after the active 480-frame
checkpoint. The client must report explicit interruption, the supervisor must
retire both old media processes and restart the complete ordered pair at
attempt 2/restart 1, and a new client must complete 48,000 non-silent frames. A
separate restart-budget profile terminates four successive real PipeWire
processes, requires exactly
three restarts before `degraded` with attempts=4/restarts=3, proves media
process and socket cleanup, blocks new playback, and retains recovery-shell
access. Its policy-service counterpart terminates four successive WirePlumber
processes, requires complete pair renewal on the first three losses, and proves
the fourth loss reaches the same cleaned, playback-blocked degraded state with
`failed_service=wireplumber`. A controlled-input exhaustion profile proves an
exact 4,800-frame zero-PCM capture before four PipeWire losses and rejects a new
capture without false success after the cleaned degraded state, without host
microphone access. Its policy-service counterpart establishes the same capture
precondition before four WirePlumber losses, requires three complete media-pair
renewals, and rejects a new capture after the cleaned
`failed_service=wireplumber` degraded state. An active capture profile
additionally reads 480 controlled zero-PCM frames, terminates PipeWire,
requires explicit `Broken pipe` interruption with no false capture completion,
then proves ordered recovery and a new exact
4,800-frame zero-PCM capture without host-microphone access. An active
capture policy-service profile terminates WirePlumber after the same 480-frame
checkpoint, rejects false capture completion, requires both old media processes
to retire before the ordered attempt 2/restart 1 pair becomes authoritative,
and proves a new exact 4,800-frame zero-PCM capture. An active
input-device-loss profile additionally proves one full deterministic bipolar
capture, removes the sole duplex HDA controller at a second client's 480-frame
checkpoint, and requires native topology to expose zero inputs, explicit
`input-route-loss`, no false completion, and blocked new capture while services
remain responsive. Other error evidence now also includes a bounded
native-control outage: volume/mute succeeds
before loss, fails at bridge open with no false acknowledgement while PipeWire
is absent, and succeeds again only after a new authoritative graph is running.
Its policy-service counterpart terminates WirePlumber, requires the resulting
full-stack outage to reject acknowledgement, retires both old media processes,
and succeeds again only after attempt 2/restart 1 restores the complete graph.
A restart-budget control profile also proves one healthy acknowledgement before
four PipeWire losses and rejects control open without false acknowledgement
after the cleaned attempt 4/restart 3 degraded state. Its WirePlumber counterpart
proves the same healthy precondition, three complete media-pair renewals, and
fail-closed control rejection after the fourth policy loss reaches
`failed_service=wireplumber` degradation. The packaged production adapter also
proves its three-attempt per-generation submission budget, blocked fourth call,
and recovery only after a real native graph generation change. A separate
two-output adapter profile now binds a pending volume request to selected PCI
05.0, prepares fallback PCI 04.0 at the same desired value, removes 05.0 through
QMP, and proves the matching fallback cannot falsely acknowledge the removed
target. The request is rejected or lost and no resubmission is needed because
the retained preference is already authoritative on the fallback. A symmetric
mute profile keeps selected 05.0 unmuted, prepares fallback 04.0 muted, removes
05.0 with mute pending, and proves the matching fallback state cannot
acknowledge the removed target. Other error behavior remains open R4 work;
physical hardware support is not claimed.
The packaged acceptance-only component client proves all twenty-two shared
primitives through the real Smithay/GLES/DRM path in all four themes and returns
to recovery after each bounded session.

Done when desktop, Applications, Search, Terminal, Files, Settings, and the
installer consume the shared catalog; visual regression evidence covers the
component matrix; and no screen claims completion from an idle-state mockup or
one-off drawing helper.

Expected output:

- Production-quality text, icons, elevation, and state transitions in the
  booted Aqua session.
- A reusable component system with deterministic and packaged QEMU evidence.

Milestone completion requires all five workstreams. Token definitions and
static mockups establish the contract but do not count as runtime completion.

## Post-M12: Aqua UI Framework Consolidation

[ADR 0003](adr-0003-aqua-ui-framework.md) fixes the long-term direction: Aqua
Linux will consolidate its typography, theme, icon, elevation, motion,
components, layout, focus, input, accessibility, lifecycle, and rendering
contracts into an Aqua-owned UI design system and internal framework.

This work follows general desktop functionality rather than preceding it. The
existing `aqua-text`, `aqua-components`, `aqua-renderer`, `aqua-shell`, and
first-party application paths continue to collect real requirements. Reusable
behavior graduates only after shared geometry drives rendering and input, real
consumers exist, semantic states and accessibility are specified, deterministic
fixtures pass, and representative packaged-QEMU evidence reaches the real
Smithay/GLES/DRM path.

The consolidation is not permission to introduce a speculative toolkit,
replace the custom compositor, weaken domain authorization, or claim a stable
third-party SDK. Its first target is one coherent internal developer surface
for Aqua's own Shell, Files, Settings, Terminal, Properties, Installer, and
future first-party applications.

## V1 Product Readiness Gates

M0-M12 are scoped implementation milestones. Their percentages record delivery
against those contracts and do not measure daily-use or release readiness.
After M12, work is governed by the mandatory, evidence-based gates in
[v1-readiness.md](v1-readiness.md):

1. Component and experience closure.
2. Presentation performance and frame correctness.
3. Wayland compatibility and display behavior.
4. Unprivileged session and core system services.
5. Accessibility, internationalization, and complete input behavior.
6. Signed updates, supply chain, security, and recovery.
7. Physical hardware, stability, and release qualification.

A milestone may remain complete when its original bounded contract is proven,
while a readiness gate stays open because it requires broader integration or a
higher evidence level. V1.0 cannot be declared from milestone arithmetic alone.

The R2 baseline now has a bounded acceptance model for the four required
workloads. It separates production GBM/KMS samples from diagnostic readback and
evaluates frame/page-flip accounting, callbacks, damage, idle suppression,
timing, CPU, memory growth, and dropped frames against caller-supplied budgets.
Host fixtures cover both acceptance and fail-closed cases without claiming
runtime performance. Live QEMU instrumentation and a fixed QEMU regression
budget now exist; soak evidence and physical-target measurements and budgets
remain open readiness work.

An ordered, bounded telemetry collector now constructs those samples from
frame requests, page flips or drops, callbacks, damage, latency, readback,
CPU-copy, idle, and resource events. It rejects out-of-order or incomplete
accounting and caps each counter at 100,000 events. The existing virtio
dumb-buffer fallback is represented separately as `LegacyCpuCopy` and is
therefore unable to pass the production-path gate. The live DRM-Wayland loop
now feeds opt-in initial and repaint request/page-flip events plus measured
submit-to-event timing into this collector, preserves the runtime-selected path,
and counts dumb-buffer copies separately. Aggregate Smithay frame-callback and
damage counters are baselined once, recorded as monotonic deltas without
per-surface multiplication, and synchronized once more after the final flip.
Real libinput keyboard, pointer-motion, and pointer-button timestamps are
retained from the earliest unpresented event and measured at the next real
page-flip boundary; the snapshot reports both a bounded sample count and maximum
input-to-present latency. The idle workload now counts quiet live dispatch
intervals, resets settling on real input or state activity, and records both
post-settle repaint violations and repeating motion timers without treating the
normal bounded event wait as a repaint. Monotonic observation duration, process
CPU-clock consumption, and the maximum bounded Linux `VmRSS` growth sampled
throughout the workload now complete the live resource bridge. The emitted
snapshot remains partial and cannot satisfy acceptance; packaged QEMU
measurements cannot satisfy physical or release acceptance.
Versioned serial boundaries now frame each live record, and a fail-closed host
validator requires exactly one structurally complete QEMU production-GBM/KMS
record per workload before reporting observed timing and resource maxima. The
validator compares those observations with a separately reviewed fixed profile
rather than deriving a moving limit from each run. It also requires one
separately framed offscreen diagnostic record and rejects it unless bounded
readbacks occur without reading or blocking production frames and without
activating KMS or display output.
A bounded packaged-QEMU runner now sequences idle, real-input window
interaction, frame-driven animation with real input, and two-client Files plus
Settings workloads in separate recovery-returning sessions, followed by the
existing deterministic offscreen GLES readback probe. It uses snapshot disk
mode, refuses a rootfs older than the compositor source, and validates both
production and isolated diagnostic records against the selected QEMU budget.

A bounded repeated-run wrapper now requires three through ten independent QEMU
boots, refuses to overwrite prior evidence, and revalidates every constituent
log before emitting a versioned review record. The review preserves per-workload
frame-time, input-latency, CPU, and memory maxima. The 2026-08-30 review contains
three independent packaged boots, 12 workload records, and three isolated
diagnostic records. Its overall maxima are 22,681 us page-flip wait, 46,522,278
us input-to-present latency, 147,346,897 us CPU time, and 130,476 KiB peak RSS
growth.

The packaged kernel now enables Bochs DRM and the R2 runner defaults to QEMU
`bochs-display`. A fresh image completed all four workloads at 1280x800 through
the `production-gbm-kms` path with direct GBM dma-buf scanout, zero production
full-frame readbacks and CPU copies, Files plus Settings client callbacks and
damage, clean recovery return, and a separately isolated diagnostic readback.
The reviewed `qemu-tcg-bochs-v1` profile enforces 50,000 us page-flip wait,
60,000,000 us input-to-present latency, 180,000,000 us CPU time, 163,840 KiB
peak RSS growth, and zero dropped frames. The virtio target remains the recorded
`legacy-cpu-copy` fallback. Repeated R2 collection and explicit QEMU budget
review are complete for this profile. Three independent longer qualification
runs are also complete; physical-target budgets remain open, and TCG timings
must not be used as physical responsiveness evidence.

An initial `qemu-tcg-bochs-soak-v1` profile now holds one compositor process for
at least five minutes with Files and Settings mapped and ten bounded real-input
cycles. It requires five distinct input-to-present samples after repaint
coalescing, zero crashes and dropped frames, no production readback or CPU copy,
bounded timing and resource growth, graceful client/process cleanup, CRTC and
GBM release, recovery return, and isolated diagnostic readback. The first run
observed 338,040 ms, 42 dispatched keyboard events, nine presented frames, five
input samples, 9,879 us maximum page-flip wait, 39,054,481 us maximum input
latency, 128,457,628 us CPU time, and 130,252 KiB peak RSS growth. The evidence
directory is local, bounded, and
never overwritten. This closes the initial QEMU soak gap, not longer
release-qualification soak or physical stability evidence.

The `qemu-tcg-bochs-qualification-v1` profile now requires a minimum 900,000 ms
window and fifteen real launcher open/dismiss cycles, each gated by its own DRM
repaint acknowledgement. It retains the established frame, input, memory, and
zero-drop limits, applies a duration-scaled 2,160,000,000 us two-vCPU CPU
ceiling, and requires at least 15 input samples, 45 keyboard events, zero
crashes, complete cleanup, recovery return, and diagnostic isolation. The first
accepted run observed 1,344,436 ms, 94 keyboard events, 71 frames, 38 input
samples, 8,131 us maximum page-flip wait, 44,045,721 us maximum input latency,
1,232,394,556 us CPU time, and 134,200 KiB RSS growth. Three independent cold
boots now pass the same fixed profile. Their bounded review contains three
isolated diagnostic records and 213 presented frames; the per-run minima are
1,319,699 ms observation, 38 input samples, and 94 keyboard events, while the
overall maxima are 8,505 us page-flip wait, 44,045,721 us input latency,
1,232,394,556 us CPU time, and 134,348 KiB RSS growth. This closes repeated QEMU
qualification, not physical stability or release readiness; the review remains
`physical_evidence=false` and `release_ready=false`.
