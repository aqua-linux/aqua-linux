# Aqua Linux Shared Component Catalog

Status date: 2026-08-29

This is the implementation inventory for the shared components required by
`interface-style.md`. An entry is complete only when its anatomy, content
bounds, tokens, pointer and keyboard behavior, accessibility semantics,
applicable state matrix, deterministic host fixtures, and packaged-QEMU use are
accepted. A screen-specific drawing helper is not a shared primitive.

## Inventory

| Component | Status | Current consumer or next boundary |
| --- | --- | --- |
| Top system bar | Planned | Shell top-level status and session controls |
| Window frame and title bar | Planned | Compositor and first-party window chrome |
| Sidebar navigation | Shared host-proven primitive | Files and Settings share render/input geometry; installer uses its row composition |
| Toolbar | Planned | Files and first-party applications |
| Segmented control | Planned | Settings and bounded mode selection |
| Search field | Planned | Applications, Global Search, Files, and Settings |
| Standard button | Shared host-proven primitive | Installer footer; packaged-QEMU component acceptance remains open |
| Icon button | Planned | Window, toolbar, and status actions |
| Checkbox | Planned | Settings and installer options |
| Switch | Planned | Settings toggles |
| Slider | Planned | Audio and future bounded value controls |
| Menu | Planned | Session, context, and overflow actions |
| List row | Shared host-proven primitive | Files and Settings navigation plus installer steps |
| Grid cell | Planned | Applications and file grid modes |
| Metadata row | Planned | Properties and system information |
| Section group | Planned | Settings and Properties |
| Application overview | Planned | Applications panel |
| Global search | Planned | Search panel and result actions |
| Running-app dock | Planned | Bottom-center shell group |
| Workspace switcher | Planned | Bottom-right shell group |
| Notification | Planned | Shell notification center |
| Confirmation dialog | Planned | Destructive and session confirmation paths |

“Planned” means that runtime behavior may exist, but its current one-off path
has not yet passed the shared-component completion contract.

## Standard Button Contract

### Anatomy And Content

- One stable outer rectangle with the shared control radius.
- An eight-logical-pixel horizontal content inset and the shared control text
  role; state changes never resize the rectangle or content bounds.
- Secondary, primary, and destructive variants. Destructive activation remains
  subject to the calling workflow's explicit confirmation policy.
- Labels are accessible names, must be non-empty, and use fitted shaped text.
  Loading replaces the visible label with a bounded progress label without
  changing geometry.

### Input And Accessibility

- Pointer activation accepts only coordinates inside the half-open component
  rectangle.
- Enter and Space are activation keys. Other keys do not activate the button.
- Disabled and loading states reject both pointer and keyboard activation.
- The semantic role is `button`; name, disabled, busy, and selected values are
  exposed independently from visual styling.
- Keyboard focus draws an external accent ring without moving the control.

### Applicable States

| State | Behavior |
| --- | --- |
| Idle | Variant-default fill and text |
| Hover | Shared hover feedback |
| Keyboard focus | Stable layout plus visible accent focus ring |
| Pressed | Shared pressed feedback |
| Selected | Selected fill and semantic selected value |
| Disabled | Muted appearance and no activation |
| Loading | Busy semantic, bounded label, and no repeated animation |
| Error | Error feedback while retaining button semantics |
| Success | Success feedback while retaining button semantics |
| Attention | Non-repeating attention feedback |

The generic empty state is not applicable because an empty accessible name is
invalid for a standard button. The fixture matrix covers the ten applicable
states in LightWhite, Softtouch, Deepside, and Nightmare at 800x600, 1280x800,
and 1536x1024, including a 1.25 output-scale case. Its canonical report is
`component-fixtures.txt`.

## List Row And Sidebar Navigation Contract

### Anatomy And Geometry

- `aqua-components` owns the renderer-independent outer rectangle, leading,
  label, and trailing slots for list rows.
- A sidebar owns its surface rectangle, accessible label, first-row rectangle,
  row stride, per-index row geometry, separator, and hit testing.
- Hit testing uses the rendered half-open row rectangles. Inter-row gaps and
  the sidebar padding do not activate adjacent items.
- Leading icons remain consumer-provided but are positioned inside the shared
  leading slot; labels use the shared fitted control-text path.

### Input, Semantics, And States

- Option, navigation-item, and step roles map to explicit accessibility roles.
- Pointer, Enter, and Space activation share the same disabled/loading gate as
  standard buttons.
- Name, selected, disabled, and busy semantics are independent of styling.
- Idle, hover, keyboard focus, pressed, selected, disabled, loading, error,
  success, and non-repeating attention states retain stable geometry.
- Sidebar containers expose the `navigation` role and a non-empty name.

Settings and Files now consume the same sidebar geometry in both `aqua-shell`
input routing and `aqua-renderer` drawing. Installer steps consume the same
list-row composition with a step-specific leading marker. Their deterministic
matrix is recorded with the standard button in `component-fixtures.txt`.

## Next Extraction Order

1. Search field and icon button.
2. Switch, checkbox, slider, and segmented control.
3. Window frame, toolbar, menu, and section structures.
4. Shell-level panels, dock, workspaces, notification, and confirmation dialog.
