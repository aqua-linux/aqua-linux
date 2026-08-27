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

## Asset Policy

- The owner-supplied identity board is retained locally outside Git. Public
  code and runtime images use only the approved transparent exports.
- Use `aqua-symbol-primary.png` on bright surfaces, `aqua-symbol-inverse.png` on dark surfaces, and `aqua-symbol-accent.png` only for active or focused states.
- Preserve transparent pixels and clear space. Do not add a tile, glow, shadow, gradient, or background to the symbol itself.
- Third-party icons shown in references are composition examples only.
- Temporary Lucide icons remain acceptable during implementation and must be replaced by Aqua-owned or independently licensed final icons before v1.

## Engineering Constraints

- Keep the custom Wayland compositor and Rust rendering stack.
- Do not implement the shell in HTML/CSS or Electron.
- Do not add GNOME Shell, Mutter, KDE Plasma, or another desktop environment.
- Existing renderer functions and identifiers containing `glass` may remain as internal compatibility names until a scoped refactor; they no longer define product acceptance.
