# Aqua Linux Product Plan

Aqua Linux is a graphical Linux operating system project. The product goal is a real bootable Linux desktop with a bright, calm, precise, Aqua-owned interface and Chromium OS-like product simplicity.

This document replaces the earlier terminal-first product direction. Terminal access remains required as a system tool, but the desktop experience is no longer centered on a permanent terminal workspace.

## Product Direction

Aqua Linux should feel like a complete operating system, not a shell skin.

Product simplicity target: Aqua Linux should stay closer to Chromium OS simplicity than a traditional general-purpose Linux desktop. The system should boot quickly into one clear graphical session, expose a small set of polished first-party surfaces, avoid scattered control panels, and keep recovery paths simple. This is a scope rule, not a decision to clone Chromium OS.

Required user-facing surfaces:

- Boot splash
- Login or first-run setup
- Desktop wallpaper
- Panel or menu bar
- App launcher
- Window manager or compositor
- File manager
- Settings
- Notifications
- Network, battery, clock, volume, and session controls
- Terminal app
- Installer
- Recovery or fallback shell

Visual principles:

- Pale blue-white desktop, near-white application surfaces, dark text, and a focused system-blue accent.
- Compact desktop information density with clear sidebars, toolbars, rows, grids, and interaction states.
- A full-width top system bar plus separate bottom groups for applications/search, running apps, and workspaces.
- Subtle translucency may support depth, but readability never depends on blur, refraction, glow, or decorative glass effects.
- The result must be an original Aqua Linux interface and must not reproduce a proprietary desktop or import another desktop environment.

## Canonical V1 Screen Contract

The final visual targets are recorded as implementation rules in
[visual-reference.md](visual-reference.md). Their source boards remain in the
Git-ignored local reference library and are not public repository assets.

All new UI implementation must follow [ui-contract.md](ui-contract.md) and [interface-style.md](interface-style.md).

### Active Delivery Style

As of 2026-08-27, the bright Aqua interface is the permanent v1 direction, not an interim or deferred style.

Product consequences:

- The idle desktop uses a full-width compact system bar, a pale low-contrast wallpaper, and three bottom control groups.
- The bottom-left group contains separate Applications and Search controls.
- The centered bottom group is a real pinned/running application dock.
- The bottom-right group displays workspaces and their active state.
- Applications and global search open as separate centered panels.
- First-party applications share bright window chrome, sidebars, toolbars, controls, and semantic states.
- Dates, users, application names, and system values shown in the boards are illustrative. Runtime values must be real and localized.
- Private reference boards are not redistributable icon sheets. Final runtime icons must be Aqua-owned or independently licensed.

The first desktop visual target is documented in [visual-reference.md](visual-reference.md).
Detailed implementation milestones are listed in [milestones.md](milestones.md).

## Boot Requirements

A bootable Aqua Linux system needs these layers:

- Firmware handoff: BIOS or UEFI.
- Bootloader: GRUB2 x86_64 UEFI for the first v1 installation path.
- Kernel: Linux kernel with GPU, input, storage, filesystem, and networking support.
- Init system: BusyBox init, runit, s6, OpenRC, or systemd.
- Root filesystem: base userspace, device nodes, mounts, config, fonts, icons, wallpapers, and shell components.
- Graphics stack: framebuffer, DRM/KMS, Wayland compositor, X11 stack, or a full desktop environment base.
- Input stack: keyboard, mouse, touchpad, libinput if using Wayland/X11.
- Session startup: display/login manager or direct autologin into Aqua shell.
- Recovery path: fallback TTY or emergency shell.
- Installer: partitioning, filesystem creation, rootfs copy, bootloader install, user creation, locale/timezone, and first boot config.

Selected packaging base: Buildroot. Aqua Linux should use Buildroot to produce a small, controlled image rather than starting from a full existing desktop distribution. The goal is a focused OS image with only the packages needed for boot, graphics, input, compositor, first-party apps, networking, audio, installer, and recovery.

Current repo status:

- Buildroot image tooling exists.
- QEMU boot scripts exist.
- The `aqua-installer` crate provides guarded state, bounded validation, eight shell-free commands, 11 target-confined actions, and a failure-aware 20-step graph. Disposable-QEMU tests execute the separately gated transaction, verify failure cleanup, boot the installed filesystem, and validate the GRUB2 UEFI fallback path. The graphical Wayland client is packaged and tested; physical-disk installation remains outside the supported development scope.
- The Buildroot image retains a text recovery shell while an explicitly gated graphical session runs the custom compositor in QEMU.
- The current QEMU desktop, launcher, and first-party applications are functional prototypes. Near-term work will converge them on the canonical interface contract.

## Buildroot Direction

Buildroot is selected as the OS image base.

Buildroot responsibilities:

- Cross-build the Linux kernel and userspace.
- Produce the root filesystem and bootable disk image.
- Package Aqua compositor, session startup, assets, installer, and recovery tools.
- Keep the image small and understandable.
- Make QEMU boot testing repeatable.

Expected Buildroot packages or capabilities:

- Linux kernel with DRM/KMS, input, virtio, framebuffer console, storage, and networking support.
- GRUB2 x86_64 UEFI support with the standard `EFI/BOOT/BOOTX64.EFI` fallback path, as fixed by ADR 0002.
- BusyBox or another small init/userspace base.
- `udev`/`eudev` or equivalent device handling if required by the compositor stack.
- `libinput` for keyboard, mouse, and touchpad input.
- `mesa`, `gbm`, `egl`, and GPU driver pieces if the Wayland stack requires them.
- Fontconfig or a simpler font path, depending on renderer choice.
- Image decoding support for PNG assets and wallpaper pipeline.
- Network tooling sufficient for Ethernet/Wi-Fi milestones.
- Audio stack only when audio work begins.
- Fallback shell and diagnostic logging.

Buildroot constraints:

- Do not add a full desktop environment.
- Do not add a package manager as the default early runtime model unless explicitly decided later.
- Prefer one reliable graphical session over many partially integrated desktop options.
- Keep QEMU as the first reproducible target before expanding to real hardware.

Chromium OS-like simplicity principles:

- One obvious login/first-run path.
- One obvious launcher.
- One settings app.
- One file manager.
- One terminal app for advanced users.
- No overlapping desktop paradigms in the first demo.
- System updates, user accounts, and app model can stay simple until the desktop is stable.

## Architecture Options

Decision: Aqua Linux will not be a theme pack or desktop remix. The target architecture is a custom Linux distribution with its own graphical shell built as a custom Wayland compositor. Existing desktop environments may be studied for reference, but they are not the product base.

### Option A: Custom Wayland Compositor

Selected. Build Aqua Linux as a real Wayland compositor.

Pros:

- Best long-term fit for a real Linux desktop.
- Supports real windows, input, scaling, multi-monitor, screenshots, and modern Linux graphics.
- The compositor can implement Aqua's shared surfaces, shadows, optional translucency, and responsive window behavior directly.

Cons:

- Highest engineering cost.
- Requires careful graphics, input, window protocol, and session work.
- More difficult than a single-app shell.

Likely Rust stack:

- `smithay` for Wayland compositor foundations.
- `wgpu`, OpenGL, or GLES for rendering effects.
- `calloop` for event loop integration.
- `libinput`/`udev` integration through compositor stack.

### Option B: DRM/KMS Fullscreen Shell

Rejected as the main path. A DRM/KMS fullscreen shell can still be useful for boot splash, installer, recovery UI, or early graphics experiments, but it is not the desktop target.

Pros:

- Good for a custom OS image and controlled boot flow.
- Avoids a full desktop environment.
- Easier than a complete Wayland compositor.
- Can deliver boot splash, launcher, settings, installer, and first-party apps.

Cons:

- Third-party graphical Linux apps will not work normally until Wayland/X11 support exists.
- Windowing is first-party only unless we later add a compositor.
- Advanced visual effects are possible, but they are not required by the selected interface direction.

Likely Rust stack:

- `drm`, `gbm`, `input`, `calloop`, `wgpu` or `softbuffer` depending on rendering path.
- First-party UI toolkit inside the shell.

### Option C: Existing Desktop Base With Aqua Shell

Rejected. Do not build Aqua Linux as KDE, GNOME, XFCE, LXQt, Openbox, or another existing desktop with a skin.

Possible bases:

- KDE Plasma
- GNOME
- XFCE/LXQt/Openbox

Pros:

- Fastest path to a usable graphical Linux.
- Existing app compatibility, settings, display handling, network tray, audio, and login flows.
- Easier to test on real hardware.

Cons:

- Harder to feel like its own OS.
- Risk of becoming a theme pack instead of Aqua Linux.
- A deeply customized visual direction may fight the chosen desktop toolkit.

### Option D: Hybrid Path

Rejected as a product direction. Temporary host-side mockups are allowed for design validation, but the booted OS target remains the custom compositor.

Reason:

- It would speed up early visuals, but it risks turning Aqua Linux into a theme pack.
- Any temporary host-side design work must be treated as throwaway validation, not the real OS architecture.

## Target Desktop Architecture

The Aqua desktop should be split into independent components:

- `aqua-compositor`: Wayland compositor and shell process.
- `aqua-scene`: rendering primitives, shared surfaces, animation, shadows, and wallpaper composition.
- `aqua-text`: Unicode shaping, font fallback, text layout, and scale-native glyph caching shared by shell and first-party applications.
- `aqua-icons`: reviewed SVG loading, symbolic recoloring, scale-native rasterization, fallback diagnostics, and bounded icon caching.
- `aqua-components`: shared component anatomy, state machines, input semantics, motion, and deterministic visual fixtures.
- `aqua-shell`: desktop UI surfaces such as panel, launcher, window chrome, notifications, and session menu.
- `aqua-settings`: first-party settings app.
- `aqua-files`: first-party file manager.
- `aqua-terminal`: terminal emulator app, not the primary shell identity.
- `aqua-installer`: graphical installer.

The installer state and safety contract is documented in [installer.md](installer.md).
- `aqua-session`: session startup, environment setup, and fallback behavior.
- `aqua-assets`: packaged icons, wallpapers, cursors, sounds, and themes.

The text, icon, and component names above describe owned logical modules. They
may initially live inside existing crates and should be split into independent
crates only when reuse or compile-time ownership makes that boundary useful.

The existing `espresso-*` crate names can stay temporarily while the prototype is still moving, but new architecture should use Aqua naming unless there is a compatibility reason not to.

## Wayland Compositor Milestones

Milestone 1: nested developer compositor.

- Runs inside the current development machine as a nested Wayland compositor or test window.
- Shows wallpaper, cursor, a panel, and at least one Aqua-managed test window.
- Uses fake system status data where needed and labels it in docs/screenshots.

Milestone 2: simple Wayland clients.

- Launches and displays at least one standard Wayland client.
- Supports basic pointer and keyboard focus.
- Supports window move, resize, close, and stacking or tiling policy.
- Keeps Aqua window chrome visually consistent with the shared interface style.

Milestone 3: booted VM compositor.

- Boots the Aqua Linux image in QEMU or UTM.
- Starts the compositor after login or through a display/session manager.
- Shows the real Aqua desktop, not a text fallback.
- Keeps a TTY or emergency shell available for recovery.

Milestone 4: first-party desktop apps.

- Settings, file manager, launcher, terminal app, and graphical installer exist as Aqua surfaces or clients.
- System status integrates real clock, power, network, and volume providers where available.

Milestone 5: hardware path.

- Tests on a real PC-class machine.
- Handles common GPU/input paths through DRM/KMS, GBM/EGL, libinput, and udev as required by the chosen compositor stack.
- Adds HiDPI, scaling, suspend/resume, and crash recovery.

## Interface Requirements

The interface needs:

- Shared responsive geometry for top bar, windows, bottom controls, panels, and workspaces.
- Near-white and pale-blue surfaces with fine borders and soft shadows.
- Stable hover, pressed, focus, selected, disabled, loading, empty, and error states.
- Shared sidebars, toolbars, search fields, segmented controls, rows, grids, buttons, and dialogs.
- Real localized system data and complete keyboard navigation.
- An asset pipeline for PNG input and framebuffer/GPU-ready runtime formats.
- Unicode shaping, kerning, fallback, grapheme-safe layout, and scale-native text rasterization.
- SVG-master icon rasterization and caching for every required logical size, theme, state, and output scale.
- Tokenized elevation levels with bounded shadow damage and measured rendering cost.
- Interruptible state motion with deterministic reduced-motion behavior.
- A complete reusable component catalog with state-matrix and visual-regression coverage.

## Asset Requirements

Source handoff folder:

`docs/aqua-linux/assets/`

Needed assets:

- Default wallpaper compatibility alias: `assets/default-wallpaper.png`, currently matching `assets/wallpaper-sunlit-water.png`.
- Official wallpaper collection: `assets/wallpaper-surf.png`, `assets/wallpaper-reef.png`, `assets/wallpaper-sunlit-water.png`, and `assets/wallpaper-moonlit-lagoon.png`.
- Primary dark Aqua symbol: `assets/aqua-symbol-primary.png`.
- Inverse Aqua symbol: `assets/aqua-symbol-inverse.png`.
- Accent Aqua symbol: `assets/aqua-symbol-accent.png`.
- Primary wordmark: `assets/aqua-wordmark-primary.png`.
- Combined logo: `assets/aqua-logo-primary.png`.
- App icon.
- Installer icon.
- Folder, disk, settings, terminal, network, volume, battery, lock, power, update, and file-type icons.
- Default wallpaper.
- Optional alternate wallpapers.
- Boot splash mark. Current source: `assets/aqua-logo-primary.png`.
- Cursor set.
- Nine canonical interface boards for desktop, applications, search, terminal, calendar, photos, files/trash, idle desktop, and settings.

Recommended source formats:

- PNG with alpha for raster icons.
- SVG only when shapes are simple and we will render/export them ourselves.
- 1024x1024 master icon.
- 512x512, 256x256, 128x128, 64x64, 32x32 icon exports.
- 2560x1440 or larger wallpaper.

## Implementation Phases

### Phase 0: Decisions

Graphics architecture path:

- Selected: custom Wayland compositor.

Pick first target hardware:

- QEMU x86_64 only.
- Real PC laptop/desktop.
- Mac virtualization through UTM.

Pick packaging base:

- Selected: Buildroot.
- Rejected for now: Alpine-based image, Debian/Ubuntu remix, Arch-based image.

### Phase 1: Visual System

- Finalize Aqua Linux name and visible copy.
- Import supplied logo, icons, and wallpaper.
- Define color, surface, typography, spacing, radius, shadow, and animation tokens.
- Define typography roles and shaping/fallback acceptance, elevation levels,
  scalable icon processing, semantic motion, and the complete component state matrix.
- Treat the private desktop and installer boards as design inputs and the public visual/UI contracts as the implementation source of truth.
- Convert their shared top bar, bottom controls, windows, applications, search, sidebar, toolbar, icon, and control anatomy into renderer tokens and component contracts.
- Document which visual effects are simulated and which are real.

### Phase 2: Bootable Graphical Demo

- Boot into a graphical Aqua session.
- Show wallpaper, panel, launcher, basic windows, and settings/about surfaces.
- Provide a terminal app rather than making terminal the whole shell.
- Keep fallback TTY available.
- Capture QEMU screenshots as proof.

### Phase 3: Core Desktop

- Window move/resize/focus.
- Bottom-left Applications and Search controls plus centered panels for each surface.
- Centered pinned/running application dock and bottom-right workspace switcher.
- File manager MVP.
- Settings MVP.
- Notification center.
- System tray/status controls.
- Power/session menu.
- Theme and wallpaper settings.

### Phase 4: Installer

- Graphical installer.
- Disk selection.
- Partitioning confirmation.
- User creation.
- Timezone/keyboard selection.
- Install progress.
- First boot handoff.

### Phase 5: Hardware and Polish

- Audio, Wi-Fi, Bluetooth, battery, suspend/resume.
- GPU acceleration if not already present.
- HiDPI and scaling.
- Accessibility basics.
- Update mechanism.
- Crash logs and recovery mode.

## Open Decisions

These decisions need owner input before implementation changes:

- Should the UI be written in Rust only, or can system components use C/C++ projects where Linux desktops already depend on them?
- Should Aqua Linux support third-party Linux GUI apps in the first demo, or only first-party Aqua apps?
- Authentication policy may choose first-run setup followed by login, but the canonical login/session screen is required before v1.
- Visual direction is resolved by the public Aqua visual/UI contracts derived from the private boards supplied on 2026-08-27.
