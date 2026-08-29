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
- Support move, resize, close, and basic stacking.
- Add Aqua window chrome.
- Keep compositor stable when a client exits.

Expected output:

- A basic Wayland app window appears inside the Aqua desktop.

Done when:

- At least one standard Wayland client can open.
- The user can focus, move, resize, and close it.
- Crash or client exit does not take down the compositor.

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
- Summary now renders all validated selections and the real/dry-run mode as a seventh deterministic installer PNG. Its bounded control accepts the exact current-target confirmation phrase only in real mode, invalidates readiness when target or mode changes, and exports the same contract through the recovery-safe Buildroot probe.
- A packaged `/usr/bin/aqua-installer` opens as a real `aqua.installer` wl_shm xdg-toplevel, loads the canonical symbol and real Linux storage inventory, and remains manual with live transaction execution disabled by default. QEMU proves the surface fills the 1280x800 DRM output without desktop chrome, applies Turkish locale, Turkish Q, `/dev/vdb`, Europe/Istanbul, bounded username/display name, password-configured status, and exact target-bound `ERASE /dev/vdb` confirmation. Password characters are not accepted or logged.
- An acceptance-only presentation rehearsal now compiles the confirmed model into the canonical 20-step graph, proves the non-executing runner reports `executed=false`, renders graph-bound 40%, 65%, and 95% Installation states, reaches Completed only at 20/20 and 100%, and captures both final live Wayland surfaces. The full packaged path records 104 forwarded key events, real virtio pointer footer and form clicks, 34 client rerenders, and seven distinct QEMU screendumps without dispatching disk commands.
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
switch, segmented control, toolbar, list row, and sidebar navigation are
host-proven across their applicable states, four themes, three required
viewports, and a fractional scale. Installer,
Applications, Global Search, Terminal, Properties, Settings, and Files consume
the applicable primitives; shared geometry drives both renderer and input
routing. Packaged-QEMU component acceptance and the remaining catalog entries
stay open.

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
