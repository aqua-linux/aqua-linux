# Aqua Linux Interface Style

This document defines the shared visual system derived from the canonical screens in [visual-reference.md](visual-reference.md).

## Product Character

Aqua Linux is calm, precise, and desktop-oriented. Its identity comes from
disciplined layout, four deliberate light/dark palettes, responsive system
surfaces, restrained depth, and a small set of recognizable Aqua controls.

## Palette

The product ships four named themes. Theme names and capitalization are part of
the public UI contract:

| Theme | Character | Brand symbol |
| --- | --- | --- |
| **LightWhite** | White and ice-white with very light gray surfaces | Primary dark symbol |
| **Softtouch** | Warm light gray with softer contrast | Primary dark symbol |
| **Deepside** | Aqua navy and deep-sea blue | Inverse white symbol |
| **Nightmare** | Neutral charcoal and near-black gray | Inverse white symbol |

LightWhite is the default. Themes change color tokens, not layout, dimensions,
typography, control placement, or interaction behavior. The accent symbol may
replace the normal theme symbol only for selected, focused, or active states.

The current runtime implementation exposes all four choices in Aqua Settings,
persists the selected theme in the v1 settings file, and applies it when Files,
Settings, Terminal, Properties, or a desktop session starts. The desktop theme
covers the top bar, Applications, Search, bottom shell, desktop overlays,
system overview, session menu, and notifications. Existing settings files
without a theme key continue as LightWhite. Installer loads the same persisted
or `AQUA_THEME`-selected palette on launch across all setup screens. A running
desktop polls the persisted selection at a bounded interval; Shell surfaces and
open first-party windows redraw only when the selected theme actually changes.

- Desktop backgrounds use the selected theme's base color with low-contrast wave forms.
- Primary and secondary surfaces use their matching theme tokens.
- Text uses the paired primary and secondary contrast tokens.
- Accent: clear system blue.
- Destructive: red, reserved for irreversible actions.
- Domain colors: green, orange, purple, and cyan only where content semantics require them.

Do not wash LightWhite or Softtouch in one hue. Deepside may use the Aqua navy
family and Nightmare may use neutral charcoal, but both must retain readable
surface separation without cyan glow, gloss streaks, grain, refraction, or
high-opacity blur.

## Surface Construction

Build production surfaces in this order:

1. Opaque or lightly translucent cool-white fill.
2. Fine neutral border or separator.
3. Soft low-spread shadow for floating windows and panels.
4. Content and explicit interaction state.

Blur is optional and must never be required for text contrast. Large panels should not contain nested decorative cards. Use rows, sections, dividers, and columns for hierarchy.

## Geometry

- Top bar: compact and full width.
- Windows: moderate radius, stable title-bar height, soft shadow.
- Panels: moderate radius and constrained width.
- Buttons and fields: smaller radius than windows, fixed control height.
- Icon buttons: square hit areas with familiar symbols and tooltips where needed.
- Bottom controls: three stable groups for applications/search, running apps, and workspaces.

Exact dimensions remain responsive renderer tokens. UI content must fit at 800x600, 1280x800, and 1536x1024 without overlap.

## Typography

- Use the packaged Noto Sans family until an Aqua-owned typography decision replaces it.
- Keep desktop UI sizes compact and readable.
- Use weight and spacing for hierarchy; do not use glow, outlines, or oversized headings.
- Use a monospace font only inside terminal and code content.

Production typography is a renderer contract, not a collection of hard-coded
pixel labels. The shared text pipeline must:

- shape Unicode text before rasterization, including kerning, ligatures,
  combining marks, bidirectional runs, and Turkish dotted/dotless I behavior;
- use deterministic font fallback without changing the baseline or control
  height when a fallback face is selected;
- expose named caption, body, control, title, and display roles with shared
  size, weight, and line-height tokens;
- rasterize from source metrics at every supported output scale instead of
  scaling a previously rendered bitmap;
- truncate only at grapheme boundaries and perform wrapping, ellipsis, and
  alignment after shaping; and
- preserve readable grayscale antialiasing in all four themes. Subpixel color
  antialiasing must not be required because output order and rotation vary.

Typography acceptance covers 1.0, 1.25, 1.5, and 2.0 output scales, Turkish
and English UI strings, mixed Latin/Arabic fixture text, long labels, disabled
text, and keyboard-focus states. Baselines must remain stable when content or
state changes.

## Elevation And Shadows

Shadows communicate ownership and elevation; they are not decoration. Use the
shared elevation levels for controls, floating panels, menus/dialogs, and
active windows. Each level defines ambient color, key color, offset, blur, and
spread. Theme palettes may adjust shadow opacity, but not elevation geometry.

- Shadows must be soft, neutral, and bounded. Do not substitute cyan glow,
  gloss, or a dark halo.
- Active and inactive windows keep identical geometry; elevation and border
  emphasis may change without reducing text contrast.
- Rounded clipping and shadow bounds must agree so corners do not show seams.
- Damage tracking must include the full shadow extent, and shadow textures or
  masks must be cached by geometry, scale, theme, and elevation.
- Overlapping surfaces must preserve a readable stacking order in every theme
  without relying on blur or wallpaper darkness.

Acceptance captures exercise isolated surfaces, overlapping windows, menus,
dialogs, and compact viewport edges at every supported output scale. Shadow
rendering must stay inside the compositor's documented frame budget.

## Scalable Iconography

Aqua Core Icons use reviewed SVG masters with a valid `viewBox` as their source
of truth. The runtime icon pipeline must rasterize a master for the requested
logical size and output scale, then cache the result by source revision, role,
theme, state, logical size, and scale. A small raster must never be enlarged to
serve a larger request.

- Required logical sizes are 16, 20, 24, 32, 48, 64, and 128 pixels.
- Symbolic status and control icons use token-driven foreground colors.
- Full-color application icons preserve their authored palette and alpha.
- Idle, hover, focused, selected, disabled, attention, and destructive roles
  must remain distinguishable without changing the icon's layout box.
- Missing or invalid assets use one documented Aqua fallback mark and emit a
  diagnostic; they must not silently display third-party artwork.
- Pixel alignment, stroke survival, alpha edges, and clear space are inspected
  at 1.0, 1.25, 1.5, and 2.0 scales in all four themes.

## Motion

Motion explains state changes and preserves spatial continuity. It must never
delay input, hide an error, or compensate for unstable layout. Use shared
duration and easing tokens for hover/press feedback, panel and menu entry,
window state changes, workspace movement, notifications, progress, and
attention states.

- State transitions begin from the currently rendered value so interruption
  and reversal do not jump.
- Layout dimensions remain stable during hover, focus, press, and selection.
- Input routing follows the interactive destination throughout a transition.
- Continuous and repeating animation pauses when fully occluded or inactive.
- Reduced-motion mode removes spatial travel and repeated attention motion
  while retaining immediate opacity, focus, progress, and error feedback.
- Animation scheduling uses compositor frame callbacks and must not create an
  independent unbounded timer loop.

Every animated component requires deterministic start, midpoint, end,
interruption, reversal, and reduced-motion acceptance cases.

## Interaction States

Every control requires idle, hover, pressed, keyboard-focus, selected, disabled, and error states where relevant. Focus rings use the blue accent and must remain visible on pale surfaces. Selected navigation rows use a pale blue fill with dark text. Destructive actions use red text or fill and require explicit confirmation when data loss is possible.

## Shared Components

- Top system bar
- Window frame and title bar
- Sidebar navigation
- Toolbar and segmented control
- Search field
- Standard button, icon button, checkbox, switch, slider, and menu
- List row, grid cell, metadata row, and section group
- Application overview panel
- Global search panel
- Running-app dock
- Workspace switcher
- Notification and confirmation dialog

The compositor and first-party applications must consume the same tokens and geometry contracts instead of duplicating one-off drawing logic.
Implementation status and per-component completion evidence are tracked in
[component-catalog.md](component-catalog.md).

Each shared component is complete only when its anatomy, content constraints,
keyboard and pointer behavior, token dependencies, accessibility semantics,
and applicable state matrix are documented and implemented. Required states
are idle, hover, keyboard focus, pressed, selected/active, disabled, loading,
empty, error, success, and attention where the component's behavior permits
them. Components must have deterministic renders for all four themes, compact
and desktop viewports, and every supported output scale. A screen-specific
drawing helper does not count as a shared component.

## Asset Policy

- The owner-supplied identity board is retained locally outside Git. Public
  code and runtime images use only the approved transparent exports.
- Use `aqua-symbol-primary.png` on bright surfaces, `aqua-symbol-inverse.png` on dark surfaces, and `aqua-symbol-accent.png` only for active or focused states.
- Preserve transparent pixels and clear space. Do not add a tile, glow, shadow, gradient, or background to the symbol itself.
- Third-party icons shown in references are composition examples only.
- Core application and status roles use the permanent project-authored Aqua
  Core Icons. New icon roles require the same provenance and license review.
- The required owner-production queue and SVG handoff rules are defined in
  [icon-production.md](icon-production.md). Aqua Core Icons are not derived,
  traced, recolored, or redistributed from elementary or another icon theme.

## Engineering Constraints

- Keep the custom Wayland compositor and Rust rendering stack.
- Do not implement the shell in HTML/CSS or Electron.
- Do not add GNOME Shell, Mutter, KDE Plasma, or another desktop environment.
- Existing renderer functions and identifiers containing `glass` may remain as internal compatibility names until a scoped refactor; they no longer define product acceptance.
