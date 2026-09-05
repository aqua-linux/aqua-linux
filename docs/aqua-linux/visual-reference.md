# Aqua Linux Visual Reference

## Binding Direction

The public rules in this document are the binding Aqua Linux v1 visual
contract as of 2026-08-27. They were derived from project-owner-supplied
desktop and installer boards stored under the Git-ignored
`local-references/` tree. The boards must not be committed, packaged, or used
as runtime source sheets.

The contract defines composition and interaction hierarchy. Example dates,
filenames, applications, account names, hardware frames, and third-party icons
in private boards are illustrative. Runtime values must be real and localized,
and runtime artwork must be Aqua-owned or independently licensed.

## Canonical Screen Set

### Desktop Workspace

- A compact full-width top bar contains product identity, centered date/time, and right-aligned system status.
- The bottom area is split into launcher/search controls, a centered running-app dock, and workspace thumbnails.
- Windows use bright, restrained chrome, thin separators, moderate corner radii, and soft shadows.
- The wallpaper is pale blue and low contrast so content stays dominant.

### Applications

- Applications open in a compact centered panel rather than a full-screen mobile grid.
- Search is first, followed by a regular icon grid with concise names.
- The applications button in the bottom-left control group shows a clear active state.

### Global Search

- Search supports applications, documents, folders, images, music, settings, and web providers.
- Suggestions, recent files, and quick actions share one predictable hierarchy.
- Category chips and results use blue only for active emphasis.

### Terminal

- Terminal is a normal window inside the desktop, not the operating system's primary identity.
- Dark terminal content is framed by the same bright title bar and window controls as other applications.

### Calendar

- Dense productivity views use sidebars, toolbars, segmented controls, and clear content columns.
- Color identifies calendar categories and current-time state without tinting the entire application.

### Photos

- Media-heavy applications use an opaque, bright content canvas and a quiet navigation sidebar.
- Thumbnails carry the visual color; surrounding UI remains neutral.

### Files And Trash

- File management supports grid/list switching, search, safe destructive actions, metadata, and a stable sidebar.
- Trash exists both as a file-manager destination and as an optional desktop item.

### Idle Desktop

- The idle desktop preserves the same top bar and three-part bottom controls.
- No desktop icons are required by default; open space is intentional.
- Workspace thumbnails remain visible and communicate the active workspace.

### Settings

- Settings uses a persistent category sidebar and a structured detail canvas.
- Related actions are grouped in bordered sections, not decorative cards.
- Search, status, storage, account, privacy, update, and developer controls follow the same shared component system.

## Installer Screen Set

The installer follows one stable workflow across both themes:

1. Welcome and language selection.
2. Keyboard layout selection and test input.
3. Region, time zone, date, and automatic-time preference.
4. Easy, custom, or live/trial installation mode.
5. User, host name, account name, and password setup.
6. Theme selection for Light or Dark.
7. Review of every selected value before the destructive install gate.
8. Progress, success, and restart state.

- A restrained identity rail may carry the Aqua symbol and wordmark without
  competing with the active form.
- Step progress remains visible and stable; Back and Next/Install actions keep
  fixed positions.
- The two themes share geometry, controls, content order, and validation.
  Only theme tokens and appropriate mark variants change.
- Dark uses neutral near-black surfaces, avoids blue-washed panels, and keeps
  blue for focus and action emphasis.
- Light uses ice-white surfaces and quiet cool-gray separators while preserving
  WCAG readability targets.
- Completion explicitly confirms success and offers restart only after the
  installer transaction reports a durable completed state.

## Cross-Screen Contract

- Default mode is light, with near-white and pale blue surfaces, black or deep navy text, and one saturated blue accent.
- Transparency is optional and subtle. Readability never depends on background blur, refraction, glow, grain, or glass simulation.
- Window chrome, toolbars, sidebars, fields, rows, buttons, dialogs, and selection states are shared primitives.
- Borders are fine and cool gray-blue. Shadows establish window elevation without dark halos.
- Corner radii are moderate and consistent; controls remain compact and desktop-oriented.
- Color is semantic: blue for selection/primary actions, red for destructive actions, green/orange/purple for domain data.
- Layouts favor useful density, alignment, scanning, and keyboard navigation over ornamental effects.
- The custom Smithay Wayland compositor and Rust-rendered Aqua surfaces remain the implementation target. The references do not authorize GNOME, KDE, proprietary UI code, or a theme-pack base.

Detailed state and behavior rules are in [ui-contract.md](ui-contract.md). Shared visual construction is in [interface-style.md](interface-style.md).

## Existing Runtime Assets

The existing ocean wallpapers remain temporary build inputs until replacement
artwork matching this direction is supplied. They are not visual acceptance
references. The approved Aqua brand exports are separate runtime assets; do
not derive application icons by cropping private boards.

Private reference boards are design-only and must not be committed or packaged
into the root filesystem. The public contract in this document is sufficient
for implementation and review.

Current implementation evidence is maintained separately in
[runtime-screenshots.md](runtime-screenshots.md). Those images are labeled QEMU
captures and hash-verified; they do not replace this visual contract.
