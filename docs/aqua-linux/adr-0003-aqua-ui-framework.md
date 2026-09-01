# ADR 0003: Aqua UI Design System And Framework

## Status

Accepted on 2026-08-29. Framework consolidation began on 2026-09-01 after the
core desktop functionality supplied real consumers. The first thirty-five bounded
slices replaced Settings Network's duplicated Wi-Fi row/action hit geometry,
Files content-row render/input geometry, and launcher panel/child pointer
routing with existing shared component contracts, then consolidated Files list
and text-preview scrollbar layout behind one renderer-neutral model and routed
both visible variants through the same bounded drag behavior. Files preview
keyboard routing now targets the visible text instead of mutating the hidden
file list. Bottom-shell pointer routing now consumes the dock rectangle produced
for the actual output viewport instead of rescaling the 800-by-600 reference;
the Smithay pointer path likewise clamps motion and resolves launcher hover
targets from the active output dimensions. Session-menu pointer routing now
maps the actual output surface into the shared `Menu` rows and retains the
second-activation confirmation gate;
Session-menu keyboard navigation now delegates Up, Down, Home, and End to the
same shared `Menu` target contract while preserving that confirmation gate;
workspace Previous, Next, Home, and End shortcuts now delegate to the shared
`WorkspaceSwitcher` target contract for both activation and window movement;
desktop context menus now expose their shared selected row to the renderer and
consume Up, Down, Home, End, Enter, Space, and Escape in the compositor without
forwarding those keys to an underlying client, while Trash retains its
repeat-activation confirmation gate;
Settings category Up, Down, Home, and End navigation now resolves through the
shared `SidebarNavigation` target contract, including bounded wrap behavior,
while Wi-Fi credential entry retains ownership of those navigation keys;
launcher Previous, Next, Home, and End navigation now delegates to the shared
visible-item targets owned separately by `ApplicationOverview` and
`GlobalSearch`, with Left/Right joining Up/Down without changing search typing;
those navigation targets now come from the active output viewport and invalid
compact layouts leave selection unchanged instead of consulting the
800-by-600 reference;
Files list Up, Down, Page Up, Page Down, Home, and End navigation now resolves
selection and the offset required to reveal it through the shared bounded list
navigation contract without changing preview-key ownership;
Settings Appearance Left and Right now resolve the persisted theme through the
shared `SegmentedControl` Previous and Next targets while category navigation
and Wi-Fi or Audio key ownership remain unchanged;
Settings Enter activation now resolves Reduced Motion, Desktop Icons, Key
Repeat, Wi-Fi, and Audio Mute through the same shared `SwitchControl` gate used
by pointer input, retaining credential-entry and authoritative disabled gates;
Settings Network Left and Right shortcuts now resolve Rescan and Forget through
the same shared `StandardButton` keyboard gates used by pointer input, with
credential-entry and broker-authority disabled states remaining fail closed;
Launcher Enter now activates the selected Applications `GridCell` or Search
`ListRow` only through its shared keyboard gate before producing the unchanged
allowlisted launch request;
that keyboard gate now builds the target from the active output viewport and
rejects invalid compact layouts instead of consulting the 800-by-600 reference;
the launcher input/scene acceptance probe now routes its full open, search,
activate, and dismiss sequence through the supplied viewport and proves an
invalid compact Search composition cannot emit a launch request;
Files Enter now activates only the selected visible content `ListRow` through
that row's shared keyboard gate before preserving the existing confined folder
navigation or read-only text-preview decision;
Files Left/Right now transfers keyboard focus between content and a separate
sidebar cursor; its Up/Down/Home/End movement consumes the shared
`SidebarNavigation` target without changing location until Enter, pointer input
clears keyboard focus, and text preview retains key ownership;
blocked Files sidebar navigation now restores the prior active location and
content selection instead of rendering a destination that was never opened;
Files sidebar rendering, pointer selection, hover, keyboard navigation, and
Enter activation now consume one shared row built for the actual client height,
so compact layouts that cannot contain every location fail closed instead of
targeting hidden rows;
Files content rendering, pointer and hover targets, scrolling, scrollbar
geometry, keyboard reveal, and Enter activation now share the row count derived
from the actual client viewport, keeping rows inert when height clips them below
the status bar or width cannot contain their minimum anatomy;
Files text-preview rendering, scrolling, scrollbar geometry, and keyboard
navigation now share the visible-line count derived from the client viewport,
with compact or narrow layouts hiding the complete preview composition and
rejecting zero-line scroll input;
Files empty-folder rendering now consumes a renderer-neutral layout derived
from the client viewport, preserving the reference composition while compact
or narrow layouts reposition or omit the group before it can overlap chrome;
Files toolbar rendering and Back/Forward pointer routing now consume one
viewport-validated layout for the toolbar, buttons, and location field, so
clipped compact controls are neither drawn nor actionable;
Files content focus transfer and rendering now consume the same
viewport-validated content rectangle, so layouts without a visible list,
preview, or empty-state composition retain sidebar focus and omit the content
focus ring;
Files status rendering now consumes a renderer-neutral bar and fitted-label
layout, omitting the complete status composition before it can overlap the
toolbar or lose its minimum label anatomy;
Files preview header, fitted title, visible text region, line stride, and
read-only label now share one renderer-neutral viewport layout with focus,
scrollbar, wheel, and keyboard routing;
the broader consolidation remains in progress.

## Context

Aqua Linux already owns renderer-neutral component contracts, typography,
themes, icons, elevation, motion, software/GPU rendering, input routing, and
deterministic plus packaged-QEMU acceptance. Shell, Files, Settings, Terminal,
Properties, and Installer nevertheless still coordinate parts of layout,
focus, validation, accessibility, and redraw behavior at the application level.

As the desktop grows, leaving those responsibilities distributed would create
screen-specific copies and inconsistent behavior. Turning the current pieces
into a general framework too early would create the opposite problem: an API
based on mock screens and unproven abstractions rather than real operating
system workflows.

## Decision

Aqua Linux will maintain its own Aqua UI design system and runtime framework
for first-party shell and application interfaces.

The framework will consolidate proven behavior from real consumers. It will
not be designed as a speculative widget toolkit before general desktop
functionality exists. Existing crates remain the implementation foundation:

- `aqua-text` owns shaping, fallback, metrics, wrapping, truncation, and
  scale-native text rasterization.
- `aqua-components` owns renderer-independent component anatomy, stable
  geometry, states, input intent, and accessibility semantics.
- `aqua-renderer` owns theme-aware software/GPU drawing adapters and consumes
  the same geometry used for hit testing.
- `aqua-shell` and first-party applications remain the real behavior consumers
  from which reusable layout, focus, form, overlay, and lifecycle contracts are
  extracted.

The consolidated Aqua UI surface must eventually cover:

- typography roles and localized text layout;
- color, spacing, radius, elevation, icon, and motion tokens;
- buttons, icon buttons, text/search inputs, checkbox, switch, slider, and
  bounded selection controls;
- menus, toolbars, sidebars, lists, grids, sections, dialogs, popovers,
  notifications, and tooltips;
- layout primitives for stacks, grids, insets, alignment, responsive
  viewports, and output scaling;
- keyboard and pointer routing, focus traversal, shortcuts, validation, and
  disabled/loading/error/success behavior;
- accessibility role, name, description, value, state, reading order, and
  focus semantics;
- invalidation, frame scheduling, reduced motion, and renderer-independent
  lifecycle rules;
- deterministic theme/state/viewport fixtures and packaged-QEMU acceptance.

Shared geometry and state remain the single source for rendering, input hit
testing, focus indication, and accessibility bounds. Workflow authorization
continues to belong to domain models: a component may express user intent but
must not authorize session, filesystem, installer, update, or other privileged
operations.

The first consolidation target is an internal first-party API, not a promise
of third-party ABI or source stability. Crate boundaries may be refined only
when doing so follows proven ownership; names such as layout, theme, or runtime
modules are architectural responsibilities, not a requirement to create a new
crate for each responsibility.

## Sequencing And Graduation Gates

Framework consolidation proceeds now that the core desktop functionality
needed to exercise it is present, including a real audio-volume model for
Slider and readiness work that exposes focus, accessibility, localization,
service, and Wayland integration requirements. Each slice remains bounded to a
real consumer and must preserve the graduation gates below.

A behavior graduates into the stable Aqua UI surface only when:

1. At least two real first-party consumers need the same behavior, unless it is
   a fundamental primitive such as typography, focus, or accessibility.
2. Renderer and input paths consume one shared geometry/state contract.
3. Keyboard, pointer, accessibility, localization, theme, scale, and applicable
   semantic states are documented.
4. Deterministic fixtures cover the required themes and viewports.
5. A packaged-QEMU path proves representative runtime use through the real
   Smithay, GLES, and DRM stack.
6. Screen-specific copies are removed only after the shared replacement passes.

## Non-Goals

- Replacing Smithay, Buildroot, or the custom compositor architecture.
- Importing GTK, Qt, another desktop environment, or another product's visual
  identity as Aqua's first-party UI foundation.
- Freezing a public third-party application SDK before the internal API and
  compatibility policy have real evidence.
- Treating screenshots, idle mockups, or token definitions as proof of
  component completion.
- Moving authorization or destructive-operation gates into presentation
  components.

## Consequences

- New UI work continues through the existing Aqua crates and extracts shared
  behavior from real product needs.
- Typography, menus, buttons, inputs, layouts, focus, accessibility, and
  overlays converge toward one documented internal developer surface.
- Application-specific drawing and hit-testing copies are temporary debt and
  must not become alternative component systems.
- M12 remains responsible for the current component evidence; later framework
  consolidation is tracked after functional closure and does not retroactively
  claim release readiness.
- A future decision is required before promising third-party API stability,
  binary compatibility, or a separately distributed toolkit.
