//! Shared component anatomy, state, input, and accessibility contracts.

use aqua_scene::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedComponentKind {
    TopSystemBar,
    WindowFrame,
    SidebarNavigation,
    Toolbar,
    SegmentedControl,
    SearchField,
    StandardButton,
    IconButton,
    Checkbox,
    Switch,
    Slider,
    Menu,
    ListRow,
    GridCell,
    MetadataRow,
    SectionGroup,
    ApplicationOverview,
    GlobalSearch,
    RunningAppDock,
    WorkspaceSwitcher,
    Notification,
    ConfirmationDialog,
}

impl SharedComponentKind {
    pub const ALL: [Self; 22] = [
        Self::TopSystemBar,
        Self::WindowFrame,
        Self::SidebarNavigation,
        Self::Toolbar,
        Self::SegmentedControl,
        Self::SearchField,
        Self::StandardButton,
        Self::IconButton,
        Self::Checkbox,
        Self::Switch,
        Self::Slider,
        Self::Menu,
        Self::ListRow,
        Self::GridCell,
        Self::MetadataRow,
        Self::SectionGroup,
        Self::ApplicationOverview,
        Self::GlobalSearch,
        Self::RunningAppDock,
        Self::WorkspaceSwitcher,
        Self::Notification,
        Self::ConfirmationDialog,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::TopSystemBar => "top-system-bar",
            Self::WindowFrame => "window-frame",
            Self::SidebarNavigation => "sidebar-navigation",
            Self::Toolbar => "toolbar",
            Self::SegmentedControl => "segmented-control",
            Self::SearchField => "search-field",
            Self::StandardButton => "standard-button",
            Self::IconButton => "icon-button",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::Slider => "slider",
            Self::Menu => "menu",
            Self::ListRow => "list-row",
            Self::GridCell => "grid-cell",
            Self::MetadataRow => "metadata-row",
            Self::SectionGroup => "section-group",
            Self::ApplicationOverview => "application-overview",
            Self::GlobalSearch => "global-search",
            Self::RunningAppDock => "running-app-dock",
            Self::WorkspaceSwitcher => "workspace-switcher",
            Self::Notification => "notification",
            Self::ConfirmationDialog => "confirmation-dialog",
        }
    }

    pub const fn is_shared_primitive(self) -> bool {
        matches!(
            self,
            Self::Toolbar
                | Self::SegmentedControl
                | Self::SearchField
                | Self::StandardButton
                | Self::IconButton
                | Self::Switch
                | Self::ListRow
                | Self::SidebarNavigation
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    Idle,
    Hover,
    KeyboardFocus,
    Pressed,
    Selected,
    Disabled,
    Loading,
    Error,
    Success,
    Attention,
}

impl ComponentState {
    pub const INTERACTIVE_STATES: [Self; 10] = [
        Self::Idle,
        Self::Hover,
        Self::KeyboardFocus,
        Self::Pressed,
        Self::Selected,
        Self::Disabled,
        Self::Loading,
        Self::Error,
        Self::Success,
        Self::Attention,
    ];

    pub const STANDARD_BUTTON_STATES: [Self; 10] = Self::INTERACTIVE_STATES;
    pub const ICON_BUTTON_STATES: [Self; 10] = Self::INTERACTIVE_STATES;
    pub const LIST_ROW_STATES: [Self; 10] = Self::INTERACTIVE_STATES;
    pub const SEARCH_FIELD_STATES: [Self; 8] = [
        Self::Idle,
        Self::Hover,
        Self::KeyboardFocus,
        Self::Disabled,
        Self::Loading,
        Self::Error,
        Self::Success,
        Self::Attention,
    ];
    pub const SWITCH_STATES: [Self; 9] = [
        Self::Idle,
        Self::Hover,
        Self::KeyboardFocus,
        Self::Pressed,
        Self::Disabled,
        Self::Loading,
        Self::Error,
        Self::Success,
        Self::Attention,
    ];
    pub const SEGMENTED_CONTROL_STATES: [Self; 9] = Self::SWITCH_STATES;

    pub const fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Hover => "hover",
            Self::KeyboardFocus => "keyboard-focus",
            Self::Pressed => "pressed",
            Self::Selected => "selected",
            Self::Disabled => "disabled",
            Self::Loading => "loading",
            Self::Error => "error",
            Self::Success => "success",
            Self::Attention => "attention",
        }
    }

    pub const fn can_activate(self) -> bool {
        !matches!(self, Self::Disabled | Self::Loading)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationKey {
    Enter,
    Space,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardButtonVariant {
    Secondary,
    Primary,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub disabled: bool,
    pub busy: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardButton<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub variant: StandardButtonVariant,
    pub state: ComponentState,
}

impl<'a> StandardButton<'a> {
    pub const fn new(rect: Rect, label: &'a str, variant: StandardButtonVariant) -> Self {
        Self {
            rect,
            label,
            variant,
            state: ComponentState::Idle,
        }
    }

    pub const fn with_state(mut self, state: ComponentState) -> Self {
        self.state = state;
        self
    }

    pub const fn content_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(8),
            y: self.rect.y,
            width: self.rect.width.saturating_sub(16),
            height: self.rect.height,
        }
    }

    pub const fn focus_rect(self) -> Rect {
        expanded_rect(self.rect, 3)
    }

    pub const fn can_activate(self) -> bool {
        !self.label.is_empty() && self.state.can_activate()
    }

    pub const fn pointer_hit(self, x: u32, y: u32) -> bool {
        self.can_activate() && rect_contains(self.rect, x, y)
    }

    pub const fn keyboard_activates(self, key: ActivationKey) -> bool {
        self.can_activate() && matches!(key, ActivationKey::Enter | ActivationKey::Space)
    }

    pub const fn accessibility(self) -> ComponentAccessibility<'a> {
        accessibility("button", self.label, self.state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconButtonGlyph {
    Back,
    Forward,
    Search,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconButton<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub glyph: IconButtonGlyph,
    pub state: ComponentState,
}

impl<'a> IconButton<'a> {
    pub const fn new(rect: Rect, label: &'a str, glyph: IconButtonGlyph) -> Self {
        Self {
            rect,
            label,
            glyph,
            state: ComponentState::Idle,
        }
    }

    pub const fn with_state(mut self, state: ComponentState) -> Self {
        self.state = state;
        self
    }

    pub const fn icon_rect(self) -> Rect {
        let size = min_u32(min_u32(self.rect.width, self.rect.height), 20);
        Rect {
            x: self
                .rect
                .x
                .saturating_add(self.rect.width.saturating_sub(size) / 2),
            y: self
                .rect
                .y
                .saturating_add(self.rect.height.saturating_sub(size) / 2),
            width: size,
            height: size,
        }
    }

    pub const fn focus_rect(self) -> Rect {
        expanded_rect(self.rect, 2)
    }

    pub const fn is_valid(self) -> bool {
        !self.label.is_empty() && self.rect.width >= 28 && self.rect.height >= 28
    }

    pub const fn can_activate(self) -> bool {
        self.is_valid() && self.state.can_activate()
    }

    pub const fn pointer_hit(self, x: u32, y: u32) -> bool {
        self.can_activate() && rect_contains(self.rect, x, y)
    }

    pub const fn keyboard_activates(self, key: ActivationKey) -> bool {
        self.can_activate() && matches!(key, ActivationKey::Enter | ActivationKey::Space)
    }

    pub const fn accessibility(self) -> ComponentAccessibility<'a> {
        accessibility("button", self.label, self.state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchFieldSlots {
    pub leading: Rect,
    pub text: Rect,
    pub trailing: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchFieldAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub value: &'a str,
    pub disabled: bool,
    pub busy: bool,
    pub invalid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchField<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub value: &'a str,
    pub placeholder: &'a str,
    pub state: ComponentState,
}

impl<'a> SearchField<'a> {
    pub const fn new(rect: Rect, label: &'a str, value: &'a str, placeholder: &'a str) -> Self {
        Self {
            rect,
            label,
            value,
            placeholder,
            state: ComponentState::Idle,
        }
    }

    pub const fn with_state(mut self, state: ComponentState) -> Self {
        self.state = state;
        self
    }

    pub const fn slots(self) -> SearchFieldSlots {
        let leading_width = min_u32(40, self.rect.width);
        let remaining = self.rect.width.saturating_sub(leading_width);
        let trailing_width = min_u32(28, remaining);
        SearchFieldSlots {
            leading: Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: leading_width,
                height: self.rect.height,
            },
            text: Rect {
                x: self.rect.x.saturating_add(leading_width),
                y: self.rect.y,
                width: remaining.saturating_sub(trailing_width),
                height: self.rect.height,
            },
            trailing: Rect {
                x: self.rect.right().saturating_sub(trailing_width),
                y: self.rect.y,
                width: trailing_width,
                height: self.rect.height,
            },
        }
    }

    pub const fn display_text(self) -> &'a str {
        if self.value.is_empty() {
            self.placeholder
        } else {
            self.value
        }
    }

    pub const fn focus_rect(self) -> Rect {
        expanded_rect(self.rect, 2)
    }

    pub const fn is_valid(self) -> bool {
        !self.label.is_empty()
            && !self.placeholder.is_empty()
            && self.rect.width >= 96
            && self.rect.height >= 32
    }

    pub const fn accepts_input(self) -> bool {
        self.is_valid()
            && !matches!(
                self.state,
                ComponentState::Disabled | ComponentState::Loading
            )
    }

    pub const fn pointer_focuses(self, x: u32, y: u32) -> bool {
        self.accepts_input() && rect_contains(self.rect, x, y)
    }

    pub const fn accessibility(self) -> SearchFieldAccessibility<'a> {
        SearchFieldAccessibility {
            role: "searchbox",
            name: self.label,
            value: self.value,
            disabled: matches!(self.state, ComponentState::Disabled),
            busy: matches!(self.state, ComponentState::Loading),
            invalid: matches!(self.state, ComponentState::Error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub checked: bool,
    pub disabled: bool,
    pub busy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchControl<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub checked: bool,
    pub state: ComponentState,
}

impl<'a> SwitchControl<'a> {
    pub const fn new(rect: Rect, label: &'a str, checked: bool) -> Self {
        Self {
            rect,
            label,
            checked,
            state: ComponentState::Idle,
        }
    }

    pub const fn with_state(mut self, state: ComponentState) -> Self {
        self.state = state;
        self
    }

    pub const fn thumb_rect(self) -> Rect {
        let inset = 6;
        let size = self.rect.height.saturating_sub(inset * 2);
        let x = if self.checked {
            self.rect.right().saturating_sub(inset + size)
        } else {
            self.rect.x.saturating_add(inset)
        };
        Rect {
            x,
            y: self.rect.y.saturating_add(inset),
            width: size,
            height: size,
        }
    }

    pub const fn focus_rect(self) -> Rect {
        expanded_rect(self.rect, 2)
    }

    pub const fn is_valid(self) -> bool {
        !self.label.is_empty() && self.rect.width >= 44 && self.rect.height >= 28
    }

    pub const fn can_toggle(self) -> bool {
        self.is_valid() && self.state.can_activate()
    }

    pub const fn pointer_toggles(self, x: u32, y: u32) -> bool {
        self.can_toggle() && rect_contains(self.rect, x, y)
    }

    pub const fn keyboard_toggles(self, key: ActivationKey) -> bool {
        self.can_toggle() && matches!(key, ActivationKey::Enter | ActivationKey::Space)
    }

    pub const fn accessibility(self) -> SwitchAccessibility<'a> {
        SwitchAccessibility {
            role: "switch",
            name: self.label,
            checked: self.checked,
            disabled: matches!(self.state, ComponentState::Disabled),
            busy: matches!(self.state, ComponentState::Loading),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentNavigationKey {
    Previous,
    Next,
    Home,
    End,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentedControlAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub selected_index: usize,
    pub segment_count: usize,
    pub disabled: bool,
    pub busy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentedControl<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub segment_count: usize,
    pub selected_index: usize,
    pub segment_gap: u32,
    pub state: ComponentState,
}

impl<'a> SegmentedControl<'a> {
    pub const fn new(
        rect: Rect,
        label: &'a str,
        segment_count: usize,
        selected_index: usize,
    ) -> Self {
        Self {
            rect,
            label,
            segment_count,
            selected_index,
            segment_gap: 0,
            state: ComponentState::Idle,
        }
    }

    pub const fn with_gap(mut self, segment_gap: u32) -> Self {
        self.segment_gap = segment_gap;
        self
    }

    pub const fn with_state(mut self, state: ComponentState) -> Self {
        self.state = state;
        self
    }

    pub const fn is_valid(self) -> bool {
        if self.label.is_empty()
            || self.segment_count < 2
            || self.segment_count > 64
            || self.selected_index >= self.segment_count
            || self.rect.height < 28
        {
            return false;
        }
        let total_gap = self
            .segment_gap
            .saturating_mul(self.segment_count.saturating_sub(1) as u32);
        self.rect.width.saturating_sub(total_gap) >= (self.segment_count as u32).saturating_mul(32)
    }

    pub const fn segment_rect(self, index: usize) -> Rect {
        if index >= self.segment_count || self.segment_count == 0 {
            return Rect {
                x: self.rect.right(),
                y: self.rect.y,
                width: 0,
                height: self.rect.height,
            };
        }
        let gaps = self
            .segment_gap
            .saturating_mul(self.segment_count.saturating_sub(1) as u32);
        let content_width = self.rect.width.saturating_sub(gaps);
        let base_width = content_width / self.segment_count as u32;
        let consumed = (index as u32).saturating_mul(base_width + self.segment_gap);
        let width = if index + 1 == self.segment_count {
            content_width.saturating_sub(base_width.saturating_mul(index as u32))
        } else {
            base_width
        };
        Rect {
            x: self.rect.x.saturating_add(consumed),
            y: self.rect.y,
            width,
            height: self.rect.height,
        }
    }

    pub fn hit_test(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() || !self.state.can_activate() {
            return None;
        }
        (0..self.segment_count).find(|index| rect_contains(self.segment_rect(*index), x, y))
    }

    pub const fn keyboard_target(self, key: SegmentNavigationKey) -> Option<usize> {
        if !self.is_valid() || !self.state.can_activate() {
            return None;
        }
        match key {
            SegmentNavigationKey::Previous => Some(if self.selected_index == 0 {
                self.segment_count - 1
            } else {
                self.selected_index - 1
            }),
            SegmentNavigationKey::Next => Some((self.selected_index + 1) % self.segment_count),
            SegmentNavigationKey::Home => Some(0),
            SegmentNavigationKey::End => Some(self.segment_count - 1),
            SegmentNavigationKey::Other => None,
        }
    }

    pub const fn focus_rect(self) -> Rect {
        expanded_rect(self.segment_rect(self.selected_index), 2)
    }

    pub const fn accessibility(self) -> SegmentedControlAccessibility<'a> {
        SegmentedControlAccessibility {
            role: "radiogroup",
            name: self.label,
            selected_index: self.selected_index,
            segment_count: self.segment_count,
            disabled: matches!(self.state, ComponentState::Disabled),
            busy: matches!(self.state, ComponentState::Loading),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toolbar<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub horizontal_padding: u32,
    pub vertical_padding: u32,
    pub item_gap: u32,
}

impl<'a> Toolbar<'a> {
    pub const fn new(rect: Rect, label: &'a str) -> Self {
        Self {
            rect,
            label,
            horizontal_padding: 18,
            vertical_padding: 7,
            item_gap: 8,
        }
    }

    pub const fn with_spacing(
        mut self,
        horizontal_padding: u32,
        vertical_padding: u32,
        item_gap: u32,
    ) -> Self {
        self.horizontal_padding = horizontal_padding;
        self.vertical_padding = vertical_padding;
        self.item_gap = item_gap;
        self
    }

    pub const fn content_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(self.horizontal_padding),
            y: self.rect.y.saturating_add(self.vertical_padding),
            width: self
                .rect
                .width
                .saturating_sub(self.horizontal_padding.saturating_mul(2)),
            height: self
                .rect
                .height
                .saturating_sub(self.vertical_padding.saturating_mul(2)),
        }
    }

    pub const fn leading_item_rect(self, index: usize, width: u32, height: u32) -> Rect {
        let content = self.content_rect();
        let requested_x = content
            .x
            .saturating_add((index as u32).saturating_mul(width.saturating_add(self.item_gap)));
        let x = min_u32(requested_x, content.right());
        Rect {
            x,
            y: content
                .y
                .saturating_add(content.height.saturating_sub(height) / 2),
            width: min_u32(width, content.right().saturating_sub(x)),
            height: min_u32(height, content.height),
        }
    }

    pub const fn is_valid(self) -> bool {
        !self.label.is_empty()
            && self.rect.width >= 96
            && self.rect.height >= 36
            && self.horizontal_padding.saturating_mul(2) < self.rect.width
            && self.vertical_padding.saturating_mul(2) < self.rect.height
    }

    pub const fn contains(self, x: u32, y: u32) -> bool {
        self.is_valid() && rect_contains(self.rect, x, y)
    }

    pub const fn separator_rect(self) -> Rect {
        Rect {
            x: self.rect.x,
            y: self.rect.bottom().saturating_sub(1),
            width: self.rect.width,
            height: 1,
        }
    }

    pub const fn accessibility(self) -> ComponentAccessibility<'a> {
        ComponentAccessibility {
            role: "toolbar",
            name: self.label,
            disabled: false,
            busy: false,
            selected: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListRowRole {
    Option,
    Navigation,
    Step,
}

impl ListRowRole {
    pub const fn accessibility_role(self) -> &'static str {
        match self {
            Self::Option => "option",
            Self::Navigation => "navigation-item",
            Self::Step => "list-item",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListRowSlots {
    pub leading: Rect,
    pub label: Rect,
    pub trailing: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListRow<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub role: ListRowRole,
    pub state: ComponentState,
    pub leading_width: u32,
    pub trailing_width: u32,
}

impl<'a> ListRow<'a> {
    pub const fn new(rect: Rect, label: &'a str, role: ListRowRole) -> Self {
        Self {
            rect,
            label,
            role,
            state: ComponentState::Idle,
            leading_width: 32,
            trailing_width: 12,
        }
    }

    pub const fn with_state(mut self, state: ComponentState) -> Self {
        self.state = state;
        self
    }

    pub const fn with_slots(mut self, leading_width: u32, trailing_width: u32) -> Self {
        self.leading_width = leading_width;
        self.trailing_width = trailing_width;
        self
    }

    pub const fn slots(self) -> ListRowSlots {
        let horizontal_padding = 8;
        let inner_x = self.rect.x.saturating_add(horizontal_padding);
        let inner_width = self.rect.width.saturating_sub(horizontal_padding * 2);
        let leading_width = min_u32(self.leading_width, inner_width);
        let remaining = inner_width.saturating_sub(leading_width);
        let trailing_width = min_u32(self.trailing_width, remaining);
        ListRowSlots {
            leading: Rect {
                x: inner_x,
                y: self.rect.y,
                width: leading_width,
                height: self.rect.height,
            },
            label: Rect {
                x: inner_x.saturating_add(leading_width),
                y: self.rect.y,
                width: remaining.saturating_sub(trailing_width),
                height: self.rect.height,
            },
            trailing: Rect {
                x: self
                    .rect
                    .right()
                    .saturating_sub(horizontal_padding + trailing_width),
                y: self.rect.y,
                width: trailing_width,
                height: self.rect.height,
            },
        }
    }

    pub const fn focus_rect(self) -> Rect {
        expanded_rect(self.rect, 2)
    }

    pub const fn can_activate(self) -> bool {
        !self.label.is_empty() && self.state.can_activate()
    }

    pub const fn pointer_hit(self, x: u32, y: u32) -> bool {
        self.can_activate() && rect_contains(self.rect, x, y)
    }

    pub const fn keyboard_activates(self, key: ActivationKey) -> bool {
        self.can_activate() && matches!(key, ActivationKey::Enter | ActivationKey::Space)
    }

    pub const fn accessibility(self) -> ComponentAccessibility<'a> {
        accessibility(self.role.accessibility_role(), self.label, self.state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SidebarNavigation<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub first_row: Rect,
    pub row_stride: u32,
}

impl<'a> SidebarNavigation<'a> {
    pub const fn new(rect: Rect, label: &'a str, first_row: Rect, row_stride: u32) -> Self {
        Self {
            rect,
            label,
            first_row,
            row_stride,
        }
    }

    pub const fn row_rect(self, index: usize) -> Rect {
        Rect {
            x: self.first_row.x,
            y: self
                .first_row
                .y
                .saturating_add((index as u32).saturating_mul(self.row_stride)),
            width: self.first_row.width,
            height: self.first_row.height,
        }
    }

    pub fn hit_test(self, x: u32, y: u32, row_count: usize) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        (0..row_count).find(|index| rect_contains(self.row_rect(*index), x, y))
    }

    pub const fn is_valid(self) -> bool {
        !self.label.is_empty()
            && self.row_stride >= self.first_row.height
            && self.first_row.x >= self.rect.x
            && self.first_row.y >= self.rect.y
            && self.first_row.right() <= self.rect.right()
            && self.first_row.bottom() <= self.rect.bottom()
    }

    pub const fn separator_rect(self) -> Rect {
        Rect {
            x: self.rect.right(),
            y: self.rect.y,
            width: 1,
            height: self.rect.height,
        }
    }

    pub const fn accessibility(self) -> ComponentAccessibility<'a> {
        ComponentAccessibility {
            role: "navigation",
            name: self.label,
            disabled: false,
            busy: false,
            selected: false,
        }
    }
}

const fn accessibility<'a>(
    role: &'static str,
    name: &'a str,
    state: ComponentState,
) -> ComponentAccessibility<'a> {
    ComponentAccessibility {
        role,
        name,
        disabled: matches!(state, ComponentState::Disabled),
        busy: matches!(state, ComponentState::Loading),
        selected: matches!(state, ComponentState::Selected),
    }
}

const fn rect_contains(rect: Rect, x: u32, y: u32) -> bool {
    x >= rect.x && x < rect.right() && y >= rect.y && y < rect.bottom()
}

const fn expanded_rect(rect: Rect, amount: u32) -> Rect {
    Rect {
        x: rect.x.saturating_sub(amount),
        y: rect.y.saturating_sub(amount),
        width: rect.width.saturating_add(amount.saturating_mul(2)),
        height: rect.height.saturating_add(amount.saturating_mul(2)),
    }
}

const fn min_u32(left: u32, right: u32) -> u32 {
    if left < right {
        left
    } else {
        right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_tracks_the_eight_shared_primitives() {
        assert_eq!(SharedComponentKind::ALL.len(), 22);
        assert_eq!(
            SharedComponentKind::ALL
                .into_iter()
                .filter(|component| component.is_shared_primitive())
                .collect::<Vec<_>>(),
            vec![
                SharedComponentKind::SidebarNavigation,
                SharedComponentKind::Toolbar,
                SharedComponentKind::SegmentedControl,
                SharedComponentKind::SearchField,
                SharedComponentKind::StandardButton,
                SharedComponentKind::IconButton,
                SharedComponentKind::Switch,
                SharedComponentKind::ListRow,
            ]
        );
    }

    #[test]
    fn list_row_slots_are_bounded_and_stable() {
        let row = ListRow::new(
            Rect {
                x: 12,
                y: 20,
                width: 148,
                height: 38,
            },
            "Home",
            ListRowRole::Navigation,
        );
        let slots = row.slots();
        assert_eq!(slots.leading.width, 32);
        assert!(slots.leading.right() <= slots.label.x);
        assert!(slots.label.right() <= slots.trailing.x);
        assert_eq!(slots.trailing.right(), row.rect.right() - 8);
    }

    #[test]
    fn sidebar_hit_testing_uses_rendered_rows_not_gaps() {
        let navigation = SidebarNavigation::new(
            Rect {
                x: 2,
                y: 108,
                width: 170,
                height: 400,
            },
            "Files locations",
            Rect {
                x: 12,
                y: 126,
                width: 148,
                height: 38,
            },
            46,
        );
        assert_eq!(navigation.hit_test(12, 126, 4), Some(0));
        assert_eq!(navigation.hit_test(159, 163, 4), Some(0));
        assert_eq!(navigation.hit_test(12, 164, 4), None);
        assert_eq!(navigation.hit_test(12, 172, 4), Some(1));
        assert_eq!(navigation.hit_test(1, 126, 4), None);
    }

    #[test]
    fn disabled_and_loading_components_reject_activation() {
        let rect = Rect {
            x: 10,
            y: 20,
            width: 120,
            height: 40,
        };
        for state in [ComponentState::Disabled, ComponentState::Loading] {
            let button = StandardButton::new(rect, "Action", StandardButtonVariant::Primary)
                .with_state(state);
            let row = ListRow::new(rect, "Item", ListRowRole::Option).with_state(state);
            let switch = SwitchControl::new(rect, "Setting", false).with_state(state);
            let segmented = SegmentedControl::new(rect, "Theme", 3, 0).with_state(state);
            assert!(!button.pointer_hit(11, 21));
            assert!(!row.pointer_hit(11, 21));
            assert!(!switch.pointer_toggles(11, 21));
            assert_eq!(segmented.hit_test(11, 21), None);
            assert!(!button.keyboard_activates(ActivationKey::Enter));
            assert!(!row.keyboard_activates(ActivationKey::Space));
        }
    }

    #[test]
    fn empty_accessible_names_fail_closed() {
        let rect = Rect {
            x: 10,
            y: 20,
            width: 120,
            height: 40,
        };
        let button = StandardButton::new(rect, "", StandardButtonVariant::Primary);
        let row = ListRow::new(rect, "", ListRowRole::Option);
        let icon = IconButton::new(rect, "", IconButtonGlyph::Back);
        let search = SearchField::new(rect, "", "", "Search");
        let switch = SwitchControl::new(rect, "", false);
        let segmented = SegmentedControl::new(rect, "", 3, 0);
        let toolbar = Toolbar::new(rect, "");
        let navigation = SidebarNavigation::new(rect, "", rect, 40);
        assert!(!button.can_activate());
        assert!(!row.can_activate());
        assert!(!icon.can_activate());
        assert!(!search.accepts_input());
        assert!(!switch.can_toggle());
        assert!(!segmented.is_valid());
        assert!(!toolbar.is_valid());
        assert!(!navigation.is_valid());
        assert_eq!(navigation.hit_test(11, 21, 1), None);
    }

    #[test]
    fn icon_button_and_search_field_slots_are_bounded() {
        let icon = IconButton::new(
            Rect {
                x: 18,
                y: 68,
                width: 28,
                height: 28,
            },
            "Back",
            IconButtonGlyph::Back,
        );
        assert!(icon.is_valid());
        assert_eq!(icon.icon_rect().width, 20);
        assert!(icon.icon_rect().right() <= icon.rect.right());

        let search = SearchField::new(
            Rect {
                x: 24,
                y: 54,
                width: 532,
                height: 42,
            },
            "Search applications",
            "settings",
            "Search apps",
        );
        let slots = search.slots();
        assert!(search.is_valid());
        assert!(slots.leading.right() <= slots.text.x);
        assert!(slots.text.right() <= slots.trailing.x);
        assert_eq!(search.accessibility().role, "searchbox");
        assert_eq!(search.accessibility().value, "settings");
    }

    #[test]
    fn switch_and_segmented_control_geometry_matches_input() {
        let switch = SwitchControl::new(
            Rect {
                x: 474,
                y: 132,
                width: 82,
                height: 36,
            },
            "Reduced motion",
            true,
        );
        assert!(switch.thumb_rect().right() < switch.rect.right());
        assert!(switch.pointer_toggles(474, 132));
        assert!(!switch.pointer_toggles(473, 132));
        assert_eq!(switch.accessibility().role, "switch");
        assert!(switch.accessibility().checked);

        let segmented = SegmentedControl::new(
            Rect {
                x: 218,
                y: 214,
                width: 346,
                height: 48,
            },
            "Desktop theme",
            4,
            1,
        )
        .with_gap(6);
        assert!(segmented.is_valid());
        assert_eq!(segmented.segment_rect(0).width, 82);
        assert_eq!(segmented.segment_rect(3).right(), segmented.rect.right());
        assert_eq!(segmented.hit_test(307, 220), Some(1));
        assert_eq!(segmented.hit_test(301, 220), None);
        assert_eq!(
            segmented.keyboard_target(SegmentNavigationKey::Previous),
            Some(0)
        );
        assert_eq!(segmented.accessibility().role, "radiogroup");
    }

    #[test]
    fn toolbar_owns_bounded_content_and_leading_item_geometry() {
        let toolbar = Toolbar::new(
            Rect {
                x: 2,
                y: 50,
                width: 636,
                height: 58,
            },
            "File navigation",
        )
        .with_spacing(16, 14, 8);
        assert!(toolbar.is_valid());
        assert_eq!(toolbar.content_rect().y, 64);
        assert_eq!(toolbar.leading_item_rect(0, 28, 28).x, 18);
        assert_eq!(toolbar.leading_item_rect(1, 28, 28).x, 54);
        assert_eq!(toolbar.leading_item_rect(0, 28, 28).y, 65);
        assert_eq!(toolbar.leading_item_rect(99, 28, 28).width, 0);
        assert_eq!(toolbar.separator_rect().bottom(), toolbar.rect.bottom());
        assert!(toolbar.contains(637, 107));
        assert!(!toolbar.contains(638, 107));
        assert_eq!(toolbar.accessibility().role, "toolbar");
    }
}
