# Aqua Linux UI Contract

This contract records the implementation requirements derived from the private
canonical desktop and installer reference boards. The boards themselves are
local-only and excluded from Git.

## Invariants

- Aqua Linux remains a Buildroot distribution with its own Smithay Wayland compositor.
- LightWhite is the default theme; Softtouch, Deepside, and Nightmare are equal supported theme targets.
- Theme changes never alter geometry, workflow, keyboard order, or content hierarchy.
- A compact top bar is always available during the desktop session.
- Bottom controls are split into applications/search, centered running applications, and workspace switching.
- Applications and global search are separate surfaces with separate activation states.
- Terminal is an application, not the desktop identity.
- Runtime labels, clock, system status, files, and results use real localized data.
- Every screen uses the shared Aqua layout, surface, typography, and interaction tokens.

## Top Bar

- Left: Aqua mark and `Aqua Linux` session identity.
- LightWhite and Softtouch use the primary dark mark; Deepside and Nightmare use the inverse mark.
- Center: localized date and time.
- Right: volume, network, power/battery, and session controls.
- Status items are keyboard reachable and expose menus only when activated.

## Bottom Controls

- Left group: applications overview and global search.
- Center group: pinned/running application icons with active and attention states.
- Right group: workspace thumbnails with a clearly selected workspace.
- Groups keep stable dimensions and do not shift when state changes.

## Applications Overview

- Opens as a centered bounded panel.
- Contains search and a categorized or filtered application grid.
- Supports keyboard navigation, activation, escape-to-close, pointer selection, and empty results.
- Launching an application dismisses the panel and focuses the resulting window.

## Global Search

- Searches applications, documents, folders, images, music, settings, and optional web providers.
- Shows provider filters, suggestions, recent files, and quick actions.
- Search remains useful without network access; web results must be clearly identified.

## Windows

- Support focus, stacking, move, resize, minimize, maximize/restore, and close.
- Use shared title-bar, toolbar, sidebar, content, and dialog primitives.
- Active and inactive windows are distinguishable without reducing text contrast.
- Destructive window actions must not be adjacent to unrelated primary actions without spacing.

## First-Party Applications

- Files: sidebar, grid/list modes, search, metadata, safe Trash behavior.
- Settings: persistent categories, searchable controls, grouped detail sections.
- Terminal: mature terminal emulation in a standard Aqua window.
- Calendar: day/week/month/agenda views and semantic event colors.
- Photos: library navigation, scalable thumbnail grid, selection and import states.
- Installer: the same light component system, with persistent steps, explicit disk identity, confirmation, progress, failure, and completion states.

Calendar and Photos are visual system references; their full product scope may land after the core Files, Settings, Terminal, launcher, search, and installer surfaces.

## Required States

Every interactive surface must define:

1. Idle.
2. Hover.
3. Keyboard focus.
4. Pressed.
5. Selected or active.
6. Disabled.
7. Loading or in progress where applicable.
8. Empty state where applicable.
9. Error and recovery.

State changes must use the shared motion tokens and must not resize or shift a
control. Components that do not support a listed state document why it is not
applicable. Hover, pointer press, keyboard focus, selected, and disabled are
separate states even when two states currently share colors.

## Visual Fidelity Acceptance

The v1 interface is not visually complete until all of these contracts pass:

- Typography: shaped Unicode runs, kerning, fallback, grapheme-safe wrapping
  and truncation, stable baselines, and scale-native rasterization pass the
  fixture set defined by [interface-style.md](interface-style.md).
- Elevation: shared shadow levels produce consistent stacking, rounded edges,
  damage bounds, and frame-time behavior for windows, panels, menus, and
  dialogs in every theme.
- Icons: reviewed SVG masters rasterize without bitmap enlargement at every
  required logical size and supported scale; theme and state variants preserve
  a stable layout box.
- Motion: opening, closing, focus, selection, attention, progress, and workspace
  transitions pass start, midpoint, completion, interruption, reversal, and
  reduced-motion checks.
- Components: every shared component declares anatomy, tokens, input behavior,
  accessibility semantics, content bounds, and its applicable state matrix.

Acceptance evidence includes deterministic renderer fixtures plus packaged
QEMU captures at 800x600, 1280x800, and 1536x1024. At least one fractional-scale
fixture is required for typography and iconography. A static mockup, token-only
definition, or single idle-state screenshot is not completion evidence.

The deterministic typography layout report in
[`typography-layout-fixtures.txt`](typography-layout-fixtures.txt) verifies long
localized-label containment, untruncated critical actions, fallback coverage,
region separation, and RGBA checksums for the required viewports and all four
themes. It is host-rendered acceptance evidence and does not replace packaged
QEMU captures.

## Accessibility And Localization

- Complete primary workflows with keyboard only.
- Maintain readable contrast without relying on blur or wallpaper darkness.
- Keep hit targets stable at supported output scales.
- Allow Turkish and English text expansion without overlap or truncating critical actions.
- Respect reduced-motion settings for panel and workspace transitions.
- Preserve logical reading order, baseline alignment, and focus indication when
  font fallback or bidirectional shaping is active.

## Acceptance Order

1. Idle desktop, top bar, and bottom controls.
2. Applications overview and global search.
3. Shared window chrome and terminal.
4. Files and Trash.
5. Settings.
6. Installer restyle.
7. Calendar and Photos visual primitives.
8. Expand Aqua Core Icons only when corresponding first-party features ship.

Shared appearance rules are defined in [interface-style.md](interface-style.md).
