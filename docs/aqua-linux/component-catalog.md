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
| Top system bar | Shared host-proven primitive | Shell brand, clock, status slots, and pointer-enabled Session controls share render/input geometry; packaged-QEMU acceptance remains open |
| Window frame and title bar | Shared host-proven primitive | Terminal, Files, Settings, and Properties renderer/input geometry; packaged-QEMU acceptance remains open |
| Sidebar navigation | Shared host-proven primitive | Files and Settings share render/input geometry; installer uses its row composition |
| Toolbar | Shared host-proven primitive | Files navigation and location controls |
| Segmented control | Shared host-proven primitive | Settings theme selection |
| Search field | Shared host-proven primitive | Applications and Global Search share render/input geometry |
| Standard button | Shared host-proven primitive | Installer footer; packaged-QEMU component acceptance remains open |
| Icon button | Shared host-proven primitive | Files back/forward navigation; broader toolbar adoption remains open |
| Checkbox | Planned | Settings and installer options |
| Switch | Shared host-proven primitive | Settings motion, desktop-icon, and key-repeat toggles |
| Slider | Planned | Audio and future bounded value controls |
| Menu | Shared host-proven primitive | Desktop icon context actions and Session action layout; packaged-QEMU component acceptance remains open |
| List row | Shared host-proven primitive | Files and Settings navigation plus installer steps |
| Grid cell | Shared host-proven primitive | Applications cards and desktop icon cells share render/input geometry; Files remains a list because no grid mode exists yet |
| Metadata row | Shared host-proven primitive | Properties and System Overview share bounded read-only label/value columns; packaged-QEMU acceptance remains open |
| Section group | Shared host-proven primitive | Settings and Properties share bounded heading, row, trailing-control, and footer geometry; packaged-QEMU acceptance remains open |
| Application overview | Shared host-proven primitive | Applications panel surface, title, search, grid layout, and pointer geometry; packaged-QEMU acceptance remains open |
| Global search | Shared host-proven primitive | Split result list, quick-action panel, and exact pointer geometry; packaged-QEMU acceptance remains open |
| Running-app dock | Planned | Bottom-center shell group |
| Workspace switcher | Planned | Bottom-right shell group |
| Notification | Planned | Shell notification center |
| Confirmation dialog | Planned | Destructive and session confirmation paths |

“Planned” means that runtime behavior may exist, but its current one-off path
has not yet passed the shared-component completion contract.

## Top System Bar Contract

### Anatomy And Geometry

- The top system bar owns one full-width surface, bottom separator, bounded
  brand slot, centered clock slot, Audio/Network/Battery status slots, and a
  trailing Session controls target.
- Clock width adapts to the space between brand and status groups while fitted
  text prevents either side from being displaced. Status and session slots
  retain stable geometry as live values change.
- The current compact contract requires at least 480x28. Invalid dimensions or
  an empty accessible name fail closed.

### Input And Accessibility

- Only the trailing Session controls rectangle is actionable. Its pointer hit
  opens or closes the existing bounded Session menu; status slots and their
  inter-item gaps do not activate it.
- The container exposes the `banner` role, each live status exposes a named
  `status` role with availability and optional battery percentage, and Session
  controls retain an independent `button` role.
- The renderer, cached Aqua Core status icons, and compositor pointer routing
  all consume the same shared rectangles.

The deterministic matrix covers compact bounds, slot separation, live-status
semantics, session hit rejection, and four themes at 800x600, 1280x800, and
fractional-scale 1536x1024. Packaged-QEMU component acceptance remains open.

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

## Search Field And Icon Button Contract

### Anatomy And Geometry

- A search field owns one stable outer rectangle plus bounded leading-icon,
  text, and trailing-action slots. Its accessible label and non-empty
  placeholder are required even when the current value is empty.
- An icon button owns one square outer rectangle, a centered glyph slot, and a
  non-empty accessible label. The glyph never substitutes for the label.
- Pointer hit testing uses the same half-open rectangles consumed by the
  renderer. State changes and focus indication do not resize either control.
- The current shared glyph inventory covers back, forward, search, and close;
  consumers must extend this project-authored vocabulary rather than derive
  third-party artwork.

### Input, Semantics, And States

- Search-field pointer input requests focus only inside its rendered bounds;
  its semantic role is `searchbox`, with independent name, value, disabled,
  busy, and invalid values.
- Icon buttons use the `button` role and share pointer, Enter, and Space
  activation behavior with standard buttons. Disabled and loading states
  reject activation.
- Icon buttons cover idle, hover, keyboard focus, pressed, selected, disabled,
  loading, error, success, and non-repeating attention states.
- Search fields cover idle, hover, keyboard focus, disabled, loading, error,
  success, and non-repeating attention states. Pressed and selected are not
  applicable to an editable searchbox.

Applications and Global Search now render and hit-test the same shared search
field. Files back and forward actions now render and hit-test the same shared
icon-button rectangles, including disabled navigation gates. Their four-theme,
three-viewport deterministic matrix is recorded in `component-fixtures.txt`.

## Switch And Segmented Control Contract

### Anatomy And Geometry

- A switch owns one stable track rectangle and a bounded thumb whose position
  reflects the checked value without changing the outer hit target.
- A segmented control owns its group rectangle, segment count, selected index,
  inter-segment gap, and deterministic per-index rectangles. Any remainder is
  retained by the final segment so the group ends at its declared right edge.
- Pointer input uses only rendered half-open rectangles. Label rows,
  inter-segment gaps, and surrounding padding do not activate a value.
- Focus rings expand outside the component without moving the track, thumb, or
  segments.

### Input, Semantics, And States

- Switches toggle with pointer input, Enter, or Space and expose the `switch`
  role, accessible name, checked, disabled, and busy values.
- Segmented controls expose the `radiogroup` role, group name, selected index,
  segment count, disabled, and busy values. Previous, next, home, and end
  navigation is bounded and wraps only for previous/next.
- Both primitives cover idle, hover, keyboard focus, pressed, disabled,
  loading, error, success, and non-repeating attention states. Selection is
  represented by checked/selected-index semantics instead of a duplicate
  generic selected state.

Settings now uses one switch geometry for reduced motion, desktop icons, and
key repeat. Its four-theme selector uses one segmented-control geometry for
both renderer and pointer routing; inter-segment gaps reject input. Their
four-theme, three-viewport deterministic matrix is recorded in
`component-fixtures.txt`.

## Toolbar Contract

### Anatomy And Geometry

- A toolbar owns one stable surface rectangle, accessible name, horizontal and
  vertical content insets, item gap, and bottom separator.
- Leading-item rectangles are derived from the toolbar content bounds. Child
  controls retain their own input and accessibility contracts while sharing
  toolbar placement.
- Insets and item dimensions are saturating and bounded; invalid or empty
  toolbar labels fail closed.
- The toolbar container uses half-open bounds and exposes no implicit action
  for its padding or unused surface.

### Semantics And Consumption

- The semantic role is `toolbar`; child icon buttons keep independent button
  names, states, and activation gates.
- Files now derives its toolbar surface and back/forward placement from the
  shared geometry in both shell input routing and renderer drawing.
- Location display remains a read-only Files child control and does not turn
  toolbar padding into an input target.

The deterministic matrix covers the toolbar in four themes at 800x600,
1280x800, and fractional-scale 1536x1024. Packaged-QEMU component acceptance
remains open.

## Window Frame And Title Bar Contract

### Anatomy And Geometry

- A window frame owns the full surface rectangle, bounded title-bar and title
  rectangles, bottom separator, three 14-pixel traffic-control targets, and a
  bottom-right resize grip.
- The title-bar height is explicit per application and shared by rendering and
  pointer routing. The move target excludes all traffic controls, and the
  resize grip never overlaps the title bar.
- Invalid frames fail closed: the current desktop contract requires at least
  240x160, a 36-72 pixel title bar, and a 16-32 pixel resize grip.

### Input, Semantics, And Consumption

- The controls expose close, minimize, and maximize actions in that order;
  title-bar padding requests an xdg-toplevel move and the bottom-right grip
  requests an xdg-toplevel resize.
- The frame exposes the `window` role, a non-empty title as its accessible
  name, and focused state independently from the selected theme.
- Terminal, Files, Settings, and Properties render and hit-test the same frame
  geometry. Their existing 48, 48, 58, and 52 pixel title-bar sizes are now
  explicit instead of being duplicated as unrelated renderer and client
  constants.
- Installer keeps its responsive full-output layout contract and is not
  claimed as a consumer until that distinct presentation path is migrated.

The deterministic matrix covers focused window geometry and input boundaries
in four themes at 800x600, 1280x800, and fractional-scale 1536x1024. A pure
compositor routing test proves controls, move, content, and resize targets do
not overlap. Packaged-QEMU component acceptance remains open.

## Menu Contract

### Anatomy And Geometry

- A menu owns one bounded surface rectangle, non-empty accessible name, item
  count, selected index, row start, row height, and inter-row gap.
- Every item rectangle is derived from the same row geometry used for drawing
  and pointer routing. Half-open item bounds reject menu padding and gaps.
- Invalid menus fail closed: empty names, zero or more than 32 items, an
  out-of-range selection, zero-sized surfaces or rows, and rows extending past
  the declared surface are rejected.
- Local menu geometry can be translated into compositor coordinates without
  changing row dimensions or selection semantics.

### Input, Semantics, And Consumption

- Previous and next keyboard movement wrap within the bounded item count;
  Home and End resolve directly to the first and last item.
- The container exposes the `menu` role, name, and item count. Each valid item
  exposes `menuitem`, name, selected, disabled, and destructive semantics.
- Desktop icon context menus now share exact local renderer geometry with
  global desktop pointer hit testing. Open, Properties, Empty Trash, and its
  explicit confirmation path retain their existing execution gates.
- The Session menu derives all four action rows and gaps from the same
  primitive while retaining its existing keyboard selection and second-Enter
  confirmation requirement.

The deterministic matrix covers menu drawing, gap rejection, keyboard
navigation, and accessibility semantics in four themes at 800x600, 1280x800,
and fractional-scale 1536x1024. Packaged-QEMU component acceptance remains
open.

## Grid Cell Contract

### Anatomy And Geometry

- A grid cell owns one stable outer rectangle plus bounded icon, primary-label,
  and optional secondary-label slots. `IconLeading` supports Applications cards;
  `IconAbove` supports desktop items without changing activation semantics.
- Icon size, inset, label gap, and secondary-row height are explicit. Selected,
  focused, loading, and feedback states do not move any slot.
- Empty accessible names, zero-sized icons, or dimensions that cannot contain
  the declared slots fail closed.

### Input, Semantics, And Consumption

- Pointer activation uses the exact half-open rectangle drawn by the renderer;
  card gaps and panel padding do not activate neighboring cells. Enter and Space
  use the same disabled/loading gate.
- Every valid cell exposes `gridcell` with independent name, selected, disabled,
  and busy values. An icon remains visual content and never replaces the name.
- Applications now derives all six visible card rectangles, icon and text slots,
  selection surfaces, and pointer targets from the shared primitive. Desktop
  Files, Settings, and Trash items use the vertical variant for selection,
  icon placement, labels, and global pointer routing.
- Files currently has a list view only, so it continues to use `ListRow`. A file
  grid consumer will be added only with a real runtime grid mode.

The deterministic matrix covers both layouts, the ten applicable states, gap
rejection, keyboard activation, accessibility semantics, four themes, and the
three required viewports including fractional scale. Packaged-QEMU component
acceptance remains open.

## Metadata Row Contract

### Anatomy And Geometry

- A metadata row owns one stable outer rectangle, non-empty label and value,
  explicit label-column width, and an inter-column gap.
- Label and value rectangles are derived from the same bounds; the value slot
  consumes the remaining width and both columns use fitted, ellipsized text.
- Empty labels or values, zero-sized rows, and column declarations that leave
  no value space fail closed.

### Semantics And Consumption

- The row exposes a read-only `definition` role whose accessible name is the
  label and whose accessible value is the displayed value. Optional emphasis
  is semantic and does not alter geometry.
- Metadata rows never accept pointer or keyboard activation; enclosing actions
  remain separate controls with their own semantics.
- Properties derives Location and optional Items rows from its shared section
  geometry. System Overview uses the same label/value contract for Host,
  Kernel, Uptime, Load, and Memory while retaining its live bounded metrics.

The deterministic matrix covers column bounds, fitted text, read-only input
behavior, emphasis, and accessibility semantics in four themes at 800x600,
1280x800, and fractional-scale 1536x1024. Packaged-QEMU component acceptance
remains open.

## Section Group Contract

### Anatomy And Geometry

- A section group owns one bordered surface, a non-empty accessible name,
  optional header and footer regions, content insets, row count, row height,
  and inter-row gap.
- Heading, content, footer, and row rectangles are derived from the declared
  surface. Trailing controls are aligned inside a specific row or footer and
  cannot extend beyond its content bounds.
- Row hit testing uses the same half-open rectangles as rendering. Insets,
  header and footer space, and inter-row gaps reject row selection.
- Invalid names, empty row sets, zero-sized geometry, overflowing reserved
  regions, and rows that exceed the content region fail closed.

### Semantics And Consumption

- The container exposes the `group` role, accessible name, and focused state;
  child switches, segmented controls, and actions retain their own roles and
  activation contracts.
- Settings derives its Appearance and preference-section heading, rows,
  trailing switch, and theme control placement from the shared geometry.
- Properties derives its metadata-row containers plus the refresh-status
  footer and trailing action from the same primitive.

The deterministic matrix covers section bounds, gap rejection, trailing
placement, focus indication, and accessibility semantics in four themes at
800x600, 1280x800, and fractional-scale 1536x1024. Packaged-QEMU component
acceptance remains open.

## Application Overview Contract

### Anatomy And Geometry

- The overview owns one bounded panel surface, title region, shared search field,
  and a three-column grid region. Horizontal inset, search offset and height,
  column gap, cell height, row stride, and visible-item limit are explicit.
- Every application cell is derived from the overview grid. Division remainder
  belongs to the final column, keeping the grid flush with its declared edge
  without moving earlier columns.
- Invalid names, search metadata, item limits, column counts, undersized search
  controls, overlapping row strides, and overflowing cells fail closed.

### Input, Semantics, And Composition

- Panel containment, search focus, and application activation use the same
  rectangles consumed by rendering. Insets and column or row gaps do not select
  an application.
- The container exposes a named `region` with item and column counts. Its search
  field retains `searchbox`; each child retains independent `gridcell` state and
  activation semantics.
- The Applications launcher mode now obtains its panel, title, search field,
  grid, cards, and pointer target resolution from this composition. Global
  Search remains separate because its split results/actions layout is different.

The deterministic matrix covers compact bounds, search and grid containment,
three-column remainder handling, gap rejection, semantics, four themes, and the
three required viewports including fractional scale. Packaged-QEMU component
acceptance remains open.

## Global Search Contract

### Anatomy And Geometry

- Global Search owns one bounded panel surface, title region, focused shared
  search field, split divider, named Results and Quick Actions sections, five
  visible result slots, and a bounded quick-action list.
- Result rows and quick-action controls derive their rectangles from the same
  panel metrics consumed by the renderer. Row height, stride, section inset,
  and visible-result limit are explicit and saturating.
- Empty accessible names, invalid counts, undersized panels, overlapping
  strides, and result or action rectangles that exceed the panel fail closed.

### Input, Semantics, And Composition

- Pointer routing tests only exact half-open result and quick-action rectangles.
  Section headings, the center divider, panel padding, and inter-row gaps remain
  non-interactive.
- The container exposes a named `search` landmark with result and quick-action
  counts. Its child search field retains `searchbox`; result rows retain
  `option`; quick actions retain `button` semantics.
- Launcher Search mode now derives its panel, headings, divider, search field,
  result rows, quick-action controls, rendering, and pointer targets from this
  composition. Its real Applications, Settings, and Files actions are unchanged.

The deterministic matrix covers compact split geometry, result/action limits,
gap rejection, accessibility semantics, four themes, and the three required
viewports including fractional scale. Packaged-QEMU component acceptance remains
open.

## Next Extraction Order

1. Running-app dock and workspace switcher.
2. Notification and confirmation dialog.
3. Checkbox and slider remain deferred until real option and bounded-value models exist.
