# Aqua Linux Shared Component Catalog

Status date: 2026-09-01

This is the implementation inventory for the shared components required by
`interface-style.md`. An entry is complete only when its anatomy, content
bounds, tokens, pointer and keyboard behavior, accessibility semantics,
applicable state matrix, deterministic host fixtures, and packaged-QEMU use are
accepted. A screen-specific drawing helper is not a shared primitive.

## Inventory

| Component | Status | Current consumer or next boundary |
| --- | --- | --- |
| Top system bar | Shared packaged-QEMU-proven primitive | Shell brand, clock, status slots, and pointer-enabled Session controls share render/input geometry |
| Window frame and title bar | Shared packaged-QEMU-proven primitive | Terminal, Files, Settings, and Properties renderer/input geometry |
| Sidebar navigation | Shared packaged-QEMU-proven primitive | Files and Settings share render/input geometry; installer uses its row composition |
| Toolbar | Shared packaged-QEMU-proven primitive | Files navigation and location controls |
| Segmented control | Shared packaged-QEMU-proven primitive | Settings theme selection |
| Search field | Shared packaged-QEMU-proven primitive | Applications and Global Search share render/input geometry |
| Standard button | Shared packaged-QEMU-proven primitive | Installer footer and Settings Network actions |
| Icon button | Shared packaged-QEMU-proven primitive | Files back/forward navigation; broader toolbar adoption remains open |
| Checkbox | Shared packaged-QEMU-proven primitive | Installer Summary target-bound destructive acknowledgement |
| Switch | Shared packaged-QEMU-proven primitive | Settings motion, desktop-icon, and key-repeat toggles |
| Slider | Shared packaged-QEMU-proven primitive | Settings Audio output-volume preference |
| Menu | Shared packaged-QEMU-proven primitive | Desktop icon context actions and Session action layout |
| List row | Shared packaged-QEMU-proven primitive | Files navigation and content entries, Settings navigation and Wi-Fi discovery, and installer steps |
| Grid cell | Shared packaged-QEMU-proven primitive | Applications cards and desktop icon cells share render/input geometry; Files remains a list because no grid mode exists yet |
| Metadata row | Shared packaged-QEMU-proven primitive | Properties and System Overview share bounded read-only label/value columns |
| Section group | Shared packaged-QEMU-proven primitive | Settings and Properties share bounded heading, row, trailing-control, and footer geometry |
| Application overview | Shared packaged-QEMU-proven primitive | Applications panel surface, title, search, grid layout, and pointer geometry |
| Global search | Shared packaged-QEMU-proven primitive | Split result list, quick-action panel, and exact pointer geometry |
| Running-app dock | Shared packaged-QEMU-proven primitive | Centered Files, Settings, and Trash targets with running indicators |
| Workspace switcher | Shared packaged-QEMU-proven primitive | Three real workspace targets, thumbnails, and active indicator |
| Notification | Shared packaged-QEMU-proven primitive | Shell toast content, dismissal target, timeout/queue model, and compositor pointer routing |
| Confirmation dialog | Shared packaged-QEMU-proven primitive | Session, Empty Trash, and Installer confirmation presentation; authorization remains model-owned |

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
fractional-scale 1536x1024. Packaged-QEMU component acceptance is covered by the aggregate four-theme run below.

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
- A valid sidebar resolves Previous and Next with bounded wrap behavior and
  Home and End directly to the first and last rendered rows. Empty,
  out-of-range, oversized, or surface-overflowing row sets fail closed.
- A valid list-navigation contract resolves Previous and Next by one row, Page
  Previous and Page Next by the declared visible-row count, and Home and End to
  the collection bounds. Row and page movement wrap consistently, and the same
  contract computes the smallest valid offset that reveals the target. Empty
  collections, zero visible rows, invalid selections, invalid targets, and
  out-of-range offsets fail closed.

Settings and Files now consume the same sidebar geometry in both `aqua-shell`
input routing and `aqua-renderer` drawing. Settings category Up, Down, Home,
and End input resolves through the shared sidebar target; Wi-Fi credential
entry consumes those keys without moving the category. Settings Wi-Fi discovery
rows also use `ListRow` for one render/input/state/accessibility contract;
unsupported security remains disabled at the component boundary. Files content
entries use the same row object for rendered bounds, leading icon and trailing
metadata slots, selected/hover state, pointer activation, and `option`
semantics. Up, Down, Page Up, Page Down, Home, and End now resolve through the
shared list-navigation target and reveal-offset contract; the text preview
continues to own those keys while it is visible. Their eight-pixel inter-row
gaps and area beyond the rendered right edge reject input. Files list and
text-preview scrollbars now derive their track, thumb,
bounded offset, and resized-window placement from one renderer-neutral layout;
list and text-preview pointer/drag routing consume their active exact half-open
track and retain distinct `Scrolled` or `PreviewScrolled` behavior. This remains
a Files layout contract rather than a new catalog primitive until another real
consumer establishes reusable scrollbar semantics. Installer steps consume the
same list-row composition with a step-specific leading marker. Settings Network
rescan and saved-credential removal use `StandardButton`, including a
non-actionable inter-button gap and broker-authority disabled states. Network
Left and Right shortcuts resolve through those same Rescan and Forget keyboard
activation gates; credential entry retains both keys and unavailable broker
authority rejects both paths. The generic deterministic matrix is recorded
in `component-fixtures.txt`, while consumer tests cover the Settings and Files
compositions.

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

## Switch, Segmented Control, And Slider Contract

### Anatomy And Geometry

- A switch owns one stable track rectangle and a bounded thumb whose position
  reflects the checked value without changing the outer hit target.
- A segmented control owns its group rectangle, segment count, selected index,
  inter-segment gap, and deterministic per-index rectangles. Any remainder is
  retained by the final segment so the group ends at its declared right edge.
- A slider owns one stable track, fill, and thumb. Its minimum, maximum, step,
  and current value must form a valid bounded range; value changes never alter
  the outer hit target.
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
- Sliders expose the `slider` role, accessible name, current value, minimum,
  maximum, disabled, and busy values. Pointer input maps the half-open track to
  a stepped bounded value; decrease/increase, home, and end keyboard actions
  clamp to the same range.
- All three primitives cover idle, hover, keyboard focus, pressed, disabled,
  loading, error, success, and non-repeating attention states. Selection is
  represented by checked/selected-index semantics instead of a duplicate
  generic selected state.

Settings now uses one switch geometry and activation gate for reduced motion,
desktop icons, key repeat, Wi-Fi association, and Audio mute. Enter and pointer
input both consume the shared switch state, so unavailable Wi-Fi and Audio
controls reject activation while Wi-Fi credential entry retains Enter. The
compact Network row bounds its switch to the rendered 32-pixel row instead of
producing an invalid oversized target. Its
four-theme selector uses one segmented-control geometry for
renderer, pointer routing, and compositor-owned Left/Right selection;
inter-segment gaps reject pointer input, and Previous/Next keyboard targets
wrap through the four persisted themes. Up, Down, Home, and End retain Settings
category ownership. The Audio
category uses the slider for a persistent 0–100 output-volume preference and a
shared switch for mute. Availability is derived only from a ready authoritative
`aqua-service-adapters` snapshot with a valid default output route; `/dev/snd`
alone cannot enable the controls. Desired values persist across unavailable or
degraded service state, while the displayed value and `backend_applied` status
remain bound to reconciliation. Three consecutive native submission failures
for one authoritative graph generation disable the controls and expose the
degraded state; the adapter does not submit again until a newer synchronized
generation arrives. A packaged QEMU probe now validates this control boundary
through the production native bridge, including stable unchanged-graph
generation, a bridge-blocked fourth call, and recovery after a real graph
change. This does not claim physical-hardware support. The typed transport has
an opt-in bounded native WirePlumber binding for graph snapshots, mixer
requests, and configured default output. It remains disabled in the default
image; physical media behavior remains a separate R4 gate. The
four-theme, three-viewport deterministic matrix is recorded in
`component-fixtures.txt`.

## Checkbox Contract

### Anatomy And Geometry

- A checkbox owns one stable outer rectangle, a square indicator slot, and a
  bounded label slot. Checked, focused, and semantic state changes never move
  either slot or alter the half-open pointer target.
- Invalid dimensions or an empty accessible label fail closed. Focus rings may
  expand outside the outer rectangle without changing indicator, label, or hit
  geometry.

### Input, Semantics, And Consumption

- Pointer input, Enter, and Space toggle only enabled, non-loading checkboxes.
  The primitive exposes the `checkbox` role with independent accessible name,
  checked, disabled, and busy values.
- The state matrix covers idle, hover, keyboard focus, pressed, disabled,
  loading, error, success, and non-repeating attention states. Checked remains
  a value and is not duplicated as a generic selected state.
- Installer Summary real mode consumes the shared checkbox as a target-bound
  acknowledgement that the selected disk will be erased. A target change
  invalidates the acknowledgement. This UI gate is additional to, and cannot
  authorize or replace, the model-owned exact `ERASE /dev/...` confirmation,
  disk identity revalidation, QEMU/operator opt-ins, or transaction gates.
- Dry-run presentation does not require the destructive acknowledgement.

The deterministic matrix covers stable slots, exact boundary rejection,
keyboard toggling, accessibility values, all nine states, four themes, and the
three required viewports including fractional scale. The packaged installer
QEMU flow additionally proves the real Summary consumer before exact-text
confirmation, while the aggregate component run covers all themed states.

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
1280x800, and fractional-scale 1536x1024. Packaged-QEMU component acceptance is covered by the aggregate four-theme run below.

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
not overlap. Packaged-QEMU component acceptance is covered by the aggregate four-theme run below.

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
  explicit confirmation path retain their existing execution gates. Their
  visible selected row follows the shared Previous, Next, Home, and End targets;
  Enter and Space activate that row, Escape dismisses the menu, and the
  compositor consumes every key press and release while the menu is open so an
  underlying client cannot receive it. Moving away from the destructive Trash
  row clears its armed repeat-activation confirmation.
- The Session menu derives all four action rows and gaps from the same
  primitive. Compositor pointer routing maps the actual output's
  `SystemOverview` surface into the runtime menu coordinate space, rejects row
  gaps and panel edges, and prevents click-through to client surfaces. Pointer
  activation retains the same second-activation confirmation requirement as
  keyboard Enter. Up, Down, Home, and End resolve through the primitive's
  keyboard targets; moving to another action clears an armed confirmation.

The deterministic matrix covers menu drawing, gap rejection, keyboard
navigation, and accessibility semantics in four themes at 800x600, 1280x800,
and fractional-scale 1536x1024. The aggregate four-theme packaged-QEMU run
covers this shared primitive.

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
1280x800, and fractional-scale 1536x1024. Packaged-QEMU component acceptance is covered by the aggregate four-theme run below.

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
- Previous and Next keyboard selection wrap within the six visible grid cells;
  Home and End resolve directly to the visible bounds. Empty collections and
  out-of-range selections fail closed.
- Enter activates the selected application only when its rendered shared grid
  cell accepts keyboard activation. Closed launchers, invalid selections, and
  empty or invalid compositions cannot produce a launch request.
- The container exposes a named `region` with item and column counts. Its search
  field retains `searchbox`; each child retains independent `gridcell` state and
  activation semantics.
- The Applications launcher mode now obtains its panel, title, search field,
  grid, cards, and pointer target resolution from this composition. Panel
  containment uses `ApplicationOverview::contains`, while application targets
  activate only through the rendered `GridCell::pointer_hit` contract. Global
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
- Result Previous and Next selection wraps within the five visible result rows;
  Home and End resolve directly to the visible bounds. Quick actions retain
  their separate pointer contract.
- Enter activates the selected result only through its shared list-row keyboard
  gate. Empty results and selections beyond the visible result limit fail
  closed before the unchanged allowlisted request is produced.
- The container exposes a named `search` landmark with result and quick-action
  counts. Its child search field retains `searchbox`; result rows retain
  `option`; quick actions retain `button` semantics.
- Launcher Search mode now derives its panel, headings, divider, search field,
  result rows, quick-action controls, rendering, and pointer targets from this
  composition. Panel containment fails closed through `GlobalSearch::contains`
  when compact geometry is invalid; result and quick-action activation passes
  directly through the rendered `ListRow::pointer_hit` and
  `StandardButton::pointer_hit` contracts. Its real Applications, Settings, and
  Files actions are unchanged.
- Smithay pointer motion is clamped to the active output's half-open pixel
  bounds, and launcher hover resolution receives those same output dimensions
  instead of falling back to the canonical 800-by-600 viewport.

The deterministic matrix covers compact split geometry, result/action limits,
gap rejection, accessibility semantics, four themes, and the three required
viewports including fractional scale. The aggregate four-theme packaged-QEMU
run covers this shared primitive.

## Running-App Dock Contract

### Anatomy And Geometry

- The running-app dock owns the centered bottom-shell surface and three stable
  72-pixel item targets for Files, Settings, and Trash.
- Each target derives a centered 64-pixel visual container, a centered 48-pixel
  production icon raster slot, and a bounded bottom running indicator. Cached
  Aqua Core icons and deterministic placeholder rendering consume these slots.
- Item count, target width, icon sizes, indicator size, and bottom inset are
  explicit. Empty names, unsupported counts, undersized bounds, overflowing
  slots, or a surface width inconsistent with its item count fail closed.

### Input, Semantics, And Consumption

- Pointer routing uses the exact half-open item rectangles rendered in the
  centered group. The surrounding transparent bottom-shell space cannot launch
  an application, and the dock's right edge does not spill into workspaces.
- The container exposes a named `toolbar` role. Files, Settings, and Trash retain
  independent `button` names plus running state; their existing allowlisted
  launch requests and duplicate-instance behavior remain unchanged.
- Both cached-icon and deterministic renderer paths use the shared surface,
  visual, raster, and running-indicator geometry. Shell and compositor input use
  the same item lookup. The compositor resolves the outer dock against the
  actual output viewport's scene rectangle, subtracts that real origin, and
  passes its unchanged width and height into the shared local lookup. It no
  longer scales the 800-by-600 reference dock into a different hit surface.

The deterministic matrix covers item boundaries, visual/raster centering,
running status, accessibility semantics, four themes, and the three required
viewports including fractional scale. Packaged-QEMU acceptance is covered by the aggregate four-theme run below.

## Workspace Switcher Contract

### Anatomy And Geometry

- The switcher owns the bottom-right surface and three stable 60-pixel targets
  matching the compositor's three real workspaces.
- Every target derives a bounded inset thumbnail. The selected workspace derives
  one 40-by-3 active indicator from its thumbnail without changing target or
  thumbnail geometry.
- Workspace count, active index, item width, thumbnail insets, and indicator
  dimensions are explicit. Empty names, invalid active indexes, unsupported
  counts, undersized bounds, and overflowing geometry fail closed.

### Input, Semantics, And Consumption

- Pointer routing uses exact half-open workspace targets; the switcher's right
  edge and space outside its surface do not activate a workspace.
- Previous/next keyboard navigation is bounded rather than wrapping; Home and
  End resolve the first and last workspace. Ctrl+Alt+Left/Right/Home/End feed
  these exact shared targets into the three-workspace activation model; adding
  Shift moves the active window to the same bounded destination.
- The container exposes a named `tablist` with workspace count and active index.
  Each named workspace exposes `tab` and selected state.
- The bottom-shell renderer derives its surface, thumbnails, active fill, and
  indicator from the shared contract. Compositor pointer routing resolves the
  same targets before invoking the existing bounded workspace activation path;
  window ownership, focus transfer, and move-between-workspace behavior remain
  unchanged.

The deterministic matrix covers target boundaries, active/inactive thumbnails,
indicator containment, accessibility semantics, four themes, and the three
required viewports including fractional scale. The aggregate four-theme
packaged-QEMU run covers this shared primitive.

## Notification Contract

### Anatomy And Geometry

- A notification owns one bounded toast surface plus icon, title, body, source,
  dismiss target, and centered dismiss-glyph slots. The current shell surface
  uses the spacious 360-by-88 layout; the contract also retains a bounded
  compact layout down to 240-by-72.
- Content slots leave the entire trailing dismiss target unobstructed. Empty
  source or title values, undersized surfaces, and overflowing slots fail
  closed; an empty body remains valid for short status announcements.
- Runtime text stays bounded by the existing notification-center limits and
  control-character filtering. Queue promotion and expiry timing remain owned
  by the shell model rather than presentation geometry.

### Input, Semantics, And Consumption

- Pointer dismissal accepts only the exact half-open dismiss rectangle. The
  compositor now resolves this rectangle from the same primitive used by the
  renderer instead of maintaining a separate top-right calculation.
- Escape dismisses an active notification. Enter and Space activate the
  focused dismiss control; other keys do not dismiss it.
- The toast exposes a polite live `status` with title, body, and source values.
  Its independently named dismiss control exposes the `button` role.
- Placeholder and cached Aqua Core icon render paths consume the same icon and
  content slots. Existing notification IDs, bounded queue, timeout promotion,
  compositor motion, and surface visibility behavior remain unchanged.

The deterministic matrix covers content containment, exact dismiss boundaries,
keyboard dismissal, live-region semantics, four themes, and the three required
viewports including fractional scale. Packaged-QEMU acceptance is covered by the aggregate four-theme run below.

## Confirmation Dialog Contract

### Anatomy And Geometry

- A confirmation surface owns one bounded rectangle, non-empty accessible name,
  title, optional detail, and optional status slot. Compact inline prompts serve
  repeat-activation flows; detailed prompts require at least 120-by-72 and keep
  title, detail, and status geometry separate.
- Standard and destructive severity, inline and modal presentation, and pending,
  armed, and confirmed states are explicit without changing content geometry.
  Empty names or titles, undersized surfaces, overflowing slots, and compact
  exact-text prompts fail closed.
- The shared contract distinguishes `RepeatActivation` from `ExactText`.
  Exact-text prompts always report that external validation is required; the
  component never treats a click, key, or rendered state as authorization.

### Input, Semantics, And Consumption

- Escape yields only a cancel intent. Enter and Space yield only a confirmation
  intent; other keys do nothing, and confirmed prompts emit no further intent.
  The owning Session, Trash, or Installer model decides whether that intent
  arms, confirms, rejects, or executes an action.
- The surface rectangle has no implicit pointer activation. Session and Trash
  retain their existing menu-row activation bounds, while Installer retains its
  bounded confirmation field and footer controls. Clicking presentation padding
  cannot bypass those child or parent input gates.
- Modal prompts expose `alertdialog`; inline prompts expose `alert`. Accessible
  name, description, modality, destructive state, and confirmed state remain
  independent from theme styling.
- Session and Empty Trash now derive their compact armed prompts from the shared
  contract. The Installer Summary real-mode panel derives its detailed title,
  exact target-bound phrase detail, status slot, and confirmed state from the
  same contract. Disk identity revalidation, exact `ERASE /dev/...` matching,
  QEMU/operator opt-ins, and transaction execution gates are unchanged.

The deterministic matrix covers compact and detailed geometry, half-open
containment, cancel/confirm intent separation, exact-text external validation,
accessibility semantics, four themes, and the three required viewports including
fractional scale.

## Packaged QEMU Acceptance

`scripts/check-component-wayland-qemu.sh` boots the generated Buildroot image
once and launches the packaged `aqua.component-acceptance` `wl_shm`
`xdg-toplevel` through the real Smithay, GLES, and DRM path for LightWhite,
Softtouch, Deepside, and Nightmare. The acceptance client renders the complete
22-primitive shared matrix at 1280x800 from fixture revision
`aqua-component-fixtures-19`; serial gates verify the 22-entry catalog and
22 shared primitives, while HMP screendumps must be nonblank, theme-distinct,
and exactly 1280x800.

Every bounded run keeps shell chrome disabled for an unobstructed full-output
surface, stops the managed client, restores the compositor/CRTC state, and
returns to the recovery shell. The generated log and screenshots remain local
build evidence and are not repository artifacts.

## Next Extraction Order

All catalog entries now satisfy the shared primitive contract. The first thirty-eight
ADR 0003 consolidation slices replace Settings Network's duplicated Wi-Fi
row/action hit geometry, Files content-entry render/input geometry, launcher
panel/child pointer routing, and Files scrollbar render/input geometry with
shared contracts, then complete active scrollbar drag parity for Files text
preview. Files preview keyboard routing now applies arrows, Page Up/Down, Home,
and End to the visible text while preserving the hidden list selection and
offset; activation is inert and Back closes the preview. Bottom-shell pointer
routing now uses the actual viewport's scene geometry for its outer surface and
local targets. Smithay motion bounds and launcher hover routing now use those
same active output dimensions. Session-menu pointer routing maps the active
output panel into the shared menu rows without weakening its confirmation
gate, and its Up, Down, Home, and End handling now consumes the same shared
keyboard target contract. Workspace activation and active-window movement now
consume the shared switcher's bounded Previous, Next, Home, and End targets.
Desktop context-menu rendering now exposes the same shared selected row used by
compositor-owned Up, Down, Home, End, Enter, Space, and Escape routing without
client key-through, while preserving Trash's repeat-activation gate.
Settings category Up, Down, Home, and End routing now consumes the shared
sidebar's bounded targets, with Wi-Fi credential entry retaining key ownership.
Launcher Up/Down and Left/Right now consume the shared Applications or Search
Previous/Next target at the active output viewport, while Home/End select the
respective visible bounds and invalid compact compositions leave selection
unchanged.
Files content-list arrows, Page Up/Down, Home, and End now consume the shared
list target and reveal-offset contract while preview-key routing remains
separate.
Settings Appearance Left/Right now consumes the shared segmented-control
Previous/Next targets and persists the selected theme without changing
category, Wi-Fi credential, or Audio key ownership.
Settings Enter now consumes the active shared switch's keyboard gate for
Reduced Motion, Desktop Icons, Key Repeat, Wi-Fi, and Audio Mute, preserving
credential-entry and authoritative disabled behavior.
Settings Network Left/Right now consumes the shared Rescan/Forget standard
button keyboard gates, preserving credential ownership and broker-authority
disabled behavior.
Launcher Enter now consumes the selected Applications grid-cell or Search
list-row keyboard gate at the active output viewport before emitting the
unchanged allowlisted request; invalid compact compositions remain inert.
The launcher input/scene acceptance probe consumes its supplied viewport for
the same event path and records no request for an invalid compact Search
composition.
Files Enter now consumes the selected visible content-row keyboard gate before
the existing confined folder navigation or read-only text-preview path; hidden
and out-of-range selections remain inert.
Files Left/Right now transfers keyboard focus between content and a distinct
sidebar cursor. Sidebar Up/Down/Home/End consumes the shared navigation target
without changing the active location before Enter, pointer input clears that
focus, and text preview keeps ownership of its keys.
Blocked sidebar navigation now restores the prior active location and content
selection, so the selected render state cannot claim an unopened destination.
Files sidebar rendering, pointer selection, hover, keyboard navigation, and
Enter activation now consume the same shared row at the actual client height.
Compact layouts that cannot contain all five locations reject hover, selection,
focus, and activation rather than targeting a hidden row.
Files content rendering, pointer and hover routing, scrolling, scrollbar
geometry, keyboard reveal, and Enter activation now consume the same row count
derived from the client viewport. Rows clipped by the status area or too narrow
to contain their leading and label slots remain inert.
Files text-preview rendering, wheel and keyboard scrolling, and scrollbar
geometry now consume the same visible-line count derived from the client
viewport. Compact layouts expose only complete lines, while layouts without
enough width or height for a line reject scrolling and omit the complete
preview composition.
Files empty-folder rendering now consumes one renderer-neutral viewport layout.
The reference coordinates remain stable, compact layouts move the group above
the status area, and layouts without sufficient width or height omit it.
Files toolbar rendering and Back/Forward pointer routing now consume one
viewport-validated layout for the toolbar, both icon buttons, and the location
field. Layouts that would clip that composition omit it and reject its input.
Files content focus transfer and rendering now consume the same
viewport-validated rectangle. Layouts without a visible list, preview, or
empty-state composition retain sidebar focus and omit the content focus ring.
Files status rendering now consumes one renderer-neutral bar and fitted-label
layout. Compact layouts omit the complete composition before it can overlap the
toolbar or lose its minimum label anatomy.
Files preview rendering, focus, scrollbar geometry, and wheel and keyboard
scrolling now consume one renderer-neutral viewport layout for the header,
fitted title, text region, line stride, and read-only label.
Files toolbar layout now owns the location label bounds. Path text uses that
fitted region and cannot escape the field at the minimum valid toolbar width.
Files sidebar rendering, pointer selection, hover, focus transfer, navigation,
and activation now consume one width-and-height validated full-composition
gate. A clipped sidebar is omitted and remains inert.
Settings category rendering, pointer selection, hover, and keyboard navigation
now consume one width-and-height validated sidebar composition derived from the
actual client buffer. Clipped category rows are omitted and remain inert.
Further extractions must continue from real first-party consumers and repeat
the same geometry, input, accessibility, deterministic-fixture, and
packaged-QEMU evidence path. The actual audio service/backend remains a
separate R4 decision and acceptance item; the Settings preference must not be
treated as playback or hardware evidence.
