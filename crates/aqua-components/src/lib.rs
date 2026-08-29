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
            Self::TopSystemBar
                | Self::WindowFrame
                | Self::Menu
                | Self::MetadataRow
                | Self::SectionGroup
                | Self::Toolbar
                | Self::SegmentedControl
                | Self::SearchField
                | Self::StandardButton
                | Self::IconButton
                | Self::Switch
                | Self::ListRow
                | Self::GridCell
                | Self::ApplicationOverview
                | Self::GlobalSearch
                | Self::RunningAppDock
                | Self::WorkspaceSwitcher
                | Self::Notification
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
    pub const GRID_CELL_STATES: [Self; 10] = Self::INTERACTIVE_STATES;
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
pub enum TopSystemStatus {
    Audio,
    Network,
    Battery,
}

impl TopSystemStatus {
    pub const ALL: [Self; 3] = [Self::Audio, Self::Network, Self::Battery];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Audio => "Audio",
            Self::Network => "Network",
            Self::Battery => "Battery",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopSystemBarAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopSystemStatusAccessibility {
    pub role: &'static str,
    pub name: &'static str,
    pub available: bool,
    pub percent: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopSystemBar<'a> {
    pub rect: Rect,
    pub name: &'a str,
}

impl<'a> TopSystemBar<'a> {
    const BRAND_WIDTH: u32 = 112;
    const STATUS_ITEM_WIDTH: u32 = 20;
    const STATUS_GAP: u32 = 19;
    const SESSION_WIDTH: u32 = 28;
    const SESSION_GAP: u32 = 10;
    const CLOCK_MAX_WIDTH: u32 = 280;

    pub const fn new(rect: Rect, name: &'a str) -> Self {
        Self { rect, name }
    }

    pub const fn brand_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(12),
            y: self.rect.y,
            width: min_u32(Self::BRAND_WIDTH, self.rect.width.saturating_sub(12)),
            height: self.rect.height,
        }
    }

    pub const fn session_rect(self) -> Rect {
        Rect {
            x: self.rect.right().saturating_sub(Self::SESSION_WIDTH),
            y: self.rect.y,
            width: min_u32(Self::SESSION_WIDTH, self.rect.width),
            height: self.rect.height,
        }
    }

    pub const fn status_group_rect(self) -> Rect {
        let width = Self::STATUS_ITEM_WIDTH
            .saturating_mul(TopSystemStatus::ALL.len() as u32)
            .saturating_add(
                Self::STATUS_GAP
                    .saturating_mul(TopSystemStatus::ALL.len().saturating_sub(1) as u32),
            );
        Rect {
            x: self
                .session_rect()
                .x
                .saturating_sub(Self::SESSION_GAP.saturating_add(width)),
            y: self.rect.y,
            width,
            height: self.rect.height,
        }
    }

    pub const fn status_rect(self, status: TopSystemStatus) -> Rect {
        let index = match status {
            TopSystemStatus::Audio => 0,
            TopSystemStatus::Network => 1,
            TopSystemStatus::Battery => 2,
        };
        let group = self.status_group_rect();
        Rect {
            x: group
                .x
                .saturating_add(index * Self::STATUS_ITEM_WIDTH.saturating_add(Self::STATUS_GAP)),
            y: group.y,
            width: Self::STATUS_ITEM_WIDTH,
            height: group.height,
        }
    }

    pub const fn clock_rect(self) -> Rect {
        let left = self.brand_rect().right().saturating_add(8);
        let right = self.status_group_rect().x.saturating_sub(8);
        let available = right.saturating_sub(left);
        let width = min_u32(Self::CLOCK_MAX_WIDTH, available);
        let centered = self
            .rect
            .x
            .saturating_add(self.rect.width.saturating_sub(width) / 2);
        let max_x = right.saturating_sub(width);
        Rect {
            x: max_u32(left, min_u32(centered, max_x)),
            y: self.rect.y,
            width,
            height: self.rect.height,
        }
    }

    pub const fn separator_rect(self) -> Rect {
        Rect {
            x: self.rect.x,
            y: self.rect.bottom().saturating_sub(1),
            width: self.rect.width,
            height: 1,
        }
    }

    pub const fn session_hit(self, x: u32, y: u32) -> bool {
        self.is_valid() && rect_contains(self.session_rect(), x, y)
    }

    pub const fn status_at(self, x: u32, y: u32) -> Option<TopSystemStatus> {
        if !self.is_valid() {
            return None;
        }
        let mut index = 0;
        while index < TopSystemStatus::ALL.len() {
            let status = TopSystemStatus::ALL[index];
            if rect_contains(self.status_rect(status), x, y) {
                return Some(status);
            }
            index += 1;
        }
        None
    }

    pub const fn is_valid(self) -> bool {
        !self.name.is_empty()
            && self.rect.width >= 480
            && self.rect.height >= 28
            && self.brand_rect().right() <= self.clock_rect().x
            && self.clock_rect().right() <= self.status_group_rect().x
            && self.status_group_rect().right() <= self.session_rect().x
            && self.session_rect().right() == self.rect.right()
    }

    pub const fn accessibility(self) -> TopSystemBarAccessibility<'a> {
        TopSystemBarAccessibility {
            role: "banner",
            name: self.name,
        }
    }

    pub const fn session_accessibility(self) -> ComponentAccessibility<'static> {
        ComponentAccessibility {
            role: "button",
            name: "Session controls",
            disabled: false,
            busy: false,
            selected: false,
        }
    }

    pub const fn status_accessibility(
        self,
        status: TopSystemStatus,
        available: bool,
        percent: Option<u8>,
    ) -> TopSystemStatusAccessibility {
        TopSystemStatusAccessibility {
            role: "status",
            name: status.name(),
            available,
            percent,
        }
    }
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
pub enum WindowControl {
    Close,
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowFrameAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowFrame<'a> {
    pub rect: Rect,
    pub title: &'a str,
    pub titlebar_height: u32,
    pub resize_grip_size: u32,
    pub focused: bool,
}

impl<'a> WindowFrame<'a> {
    pub const fn new(rect: Rect, title: &'a str, titlebar_height: u32) -> Self {
        Self {
            rect,
            title,
            titlebar_height,
            resize_grip_size: 24,
            focused: true,
        }
    }

    pub const fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub const fn titlebar_rect(self) -> Rect {
        Rect {
            x: self.rect.x,
            y: self.rect.y,
            width: self.rect.width,
            height: min_u32(self.titlebar_height, self.rect.height),
        }
    }

    pub const fn title_rect(self) -> Rect {
        let titlebar = self.titlebar_rect();
        Rect {
            x: titlebar.x.saturating_add(92),
            y: titlebar.y,
            width: titlebar.width.saturating_sub(112),
            height: titlebar.height,
        }
    }

    pub const fn control_rect(self, control: WindowControl) -> Rect {
        let index = match control {
            WindowControl::Close => 0,
            WindowControl::Minimize => 1,
            WindowControl::Maximize => 2,
        };
        let titlebar = self.titlebar_rect();
        Rect {
            x: titlebar.x.saturating_add(18_u32.saturating_add(index * 22)),
            y: titlebar
                .y
                .saturating_add(titlebar.height.saturating_sub(14) / 2),
            width: 14,
            height: 14,
        }
    }

    pub const fn control_at(self, x: u32, y: u32) -> Option<WindowControl> {
        if rect_contains(self.control_rect(WindowControl::Close), x, y) {
            Some(WindowControl::Close)
        } else if rect_contains(self.control_rect(WindowControl::Minimize), x, y) {
            Some(WindowControl::Minimize)
        } else if rect_contains(self.control_rect(WindowControl::Maximize), x, y) {
            Some(WindowControl::Maximize)
        } else {
            None
        }
    }

    pub const fn move_hit(self, x: u32, y: u32) -> bool {
        self.is_valid()
            && rect_contains(self.titlebar_rect(), x, y)
            && self.control_at(x, y).is_none()
    }

    pub const fn resize_grip_rect(self) -> Rect {
        let size = min_u32(
            self.resize_grip_size,
            min_u32(self.rect.width, self.rect.height),
        );
        Rect {
            x: self.rect.right().saturating_sub(size),
            y: self.rect.bottom().saturating_sub(size),
            width: size,
            height: size,
        }
    }

    pub const fn resize_hit(self, x: u32, y: u32) -> bool {
        self.is_valid() && rect_contains(self.resize_grip_rect(), x, y)
    }

    pub const fn separator_rect(self) -> Rect {
        let titlebar = self.titlebar_rect();
        Rect {
            x: titlebar.x,
            y: titlebar.bottom().saturating_sub(1),
            width: titlebar.width,
            height: 1,
        }
    }

    pub const fn is_valid(self) -> bool {
        !self.title.is_empty()
            && self.rect.width >= 240
            && self.rect.height >= 160
            && self.titlebar_height >= 36
            && self.titlebar_height <= 72
            && self.titlebar_height < self.rect.height
            && self.resize_grip_size >= 16
            && self.resize_grip_size <= 32
    }

    pub const fn accessibility(self) -> WindowFrameAccessibility<'a> {
        WindowFrameAccessibility {
            role: "window",
            name: self.title,
            focused: self.focused,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuNavigationKey {
    Previous,
    Next,
    Home,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub item_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItemAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub selected: bool,
    pub disabled: bool,
    pub destructive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Menu<'a> {
    pub rect: Rect,
    pub name: &'a str,
    pub item_count: usize,
    pub selected_index: usize,
    pub row_start: u32,
    pub row_height: u32,
    pub row_gap: u32,
}

impl<'a> Menu<'a> {
    pub const fn new(
        rect: Rect,
        name: &'a str,
        item_count: usize,
        selected_index: usize,
        row_start: u32,
        row_height: u32,
        row_gap: u32,
    ) -> Self {
        Self {
            rect,
            name,
            item_count,
            selected_index,
            row_start,
            row_height,
            row_gap,
        }
    }

    pub const fn translated(mut self, x: u32, y: u32) -> Self {
        self.rect.x = self.rect.x.saturating_add(x);
        self.rect.y = self.rect.y.saturating_add(y);
        self
    }

    pub const fn item_rect(self, index: usize) -> Rect {
        if index >= self.item_count {
            return Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: self.rect.x,
            y: self.rect.y.saturating_add(self.row_start).saturating_add(
                (index as u32).saturating_mul(self.row_height.saturating_add(self.row_gap)),
            ),
            width: self.rect.width,
            height: self.row_height,
        }
    }

    pub const fn item_at(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        let mut index = 0;
        while index < self.item_count {
            if rect_contains(self.item_rect(index), x, y) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub const fn keyboard_target(self, key: MenuNavigationKey) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        Some(match key {
            MenuNavigationKey::Previous => {
                if self.selected_index == 0 {
                    self.item_count - 1
                } else {
                    self.selected_index - 1
                }
            }
            MenuNavigationKey::Next => (self.selected_index + 1) % self.item_count,
            MenuNavigationKey::Home => 0,
            MenuNavigationKey::End => self.item_count - 1,
        })
    }

    pub const fn is_valid(self) -> bool {
        if self.name.is_empty()
            || self.item_count == 0
            || self.item_count > 32
            || self.selected_index >= self.item_count
            || self.rect.width == 0
            || self.rect.height == 0
            || self.row_height == 0
        {
            return false;
        }
        self.item_rect(self.item_count - 1).bottom() <= self.rect.bottom()
    }

    pub const fn accessibility(self) -> MenuAccessibility<'a> {
        MenuAccessibility {
            role: "menu",
            name: self.name,
            item_count: self.item_count,
        }
    }

    pub const fn item_accessibility(
        self,
        index: usize,
        name: &str,
        disabled: bool,
        destructive: bool,
    ) -> Option<MenuItemAccessibility<'_>> {
        if !self.is_valid() || index >= self.item_count || name.is_empty() {
            return None;
        }
        Some(MenuItemAccessibility {
            role: "menuitem",
            name,
            selected: index == self.selected_index,
            disabled,
            destructive,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataRowAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub value: &'a str,
    pub read_only: bool,
    pub emphasized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataRowSlots {
    pub label: Rect,
    pub value: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataRow<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub value: &'a str,
    pub label_width: u32,
    pub column_gap: u32,
    pub emphasized: bool,
}

impl<'a> MetadataRow<'a> {
    pub const fn new(rect: Rect, label: &'a str, value: &'a str) -> Self {
        Self {
            rect,
            label,
            value,
            label_width: 80,
            column_gap: 8,
            emphasized: false,
        }
    }

    pub const fn with_columns(mut self, label_width: u32, column_gap: u32) -> Self {
        self.label_width = label_width;
        self.column_gap = column_gap;
        self
    }

    pub const fn with_emphasis(mut self, emphasized: bool) -> Self {
        self.emphasized = emphasized;
        self
    }

    pub const fn slots(self) -> MetadataRowSlots {
        let label_width = min_u32(self.label_width, self.rect.width);
        let value_x = self
            .rect
            .x
            .saturating_add(label_width)
            .saturating_add(self.column_gap);
        MetadataRowSlots {
            label: Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: label_width,
                height: self.rect.height,
            },
            value: Rect {
                x: value_x,
                y: self.rect.y,
                width: self.rect.right().saturating_sub(value_x),
                height: self.rect.height,
            },
        }
    }

    pub const fn is_valid(self) -> bool {
        !self.label.is_empty()
            && !self.value.is_empty()
            && self.rect.width > 0
            && self.rect.height > 0
            && self.label_width > 0
            && self.label_width.saturating_add(self.column_gap) < self.rect.width
            && self.slots().value.width > 0
    }

    pub const fn accepts_input(self) -> bool {
        false
    }

    pub const fn accessibility(self) -> MetadataRowAccessibility<'a> {
        MetadataRowAccessibility {
            role: "definition",
            name: self.label,
            value: self.value,
            read_only: true,
            emphasized: self.emphasized,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionGroupAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionGroup<'a> {
    pub rect: Rect,
    pub name: &'a str,
    pub row_count: usize,
    pub header_height: u32,
    pub footer_height: u32,
    pub inset_x: u32,
    pub inset_y: u32,
    pub row_height: u32,
    pub row_gap: u32,
    pub focused: bool,
}

impl<'a> SectionGroup<'a> {
    pub const fn new(rect: Rect, name: &'a str, row_count: usize) -> Self {
        Self {
            rect,
            name,
            row_count,
            header_height: 0,
            footer_height: 0,
            inset_x: 16,
            inset_y: 12,
            row_height: 32,
            row_gap: 4,
            focused: false,
        }
    }

    pub const fn with_structure(
        mut self,
        header_height: u32,
        footer_height: u32,
        inset_x: u32,
        inset_y: u32,
        row_height: u32,
        row_gap: u32,
    ) -> Self {
        self.header_height = header_height;
        self.footer_height = footer_height;
        self.inset_x = inset_x;
        self.inset_y = inset_y;
        self.row_height = row_height;
        self.row_gap = row_gap;
        self
    }

    pub const fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub const fn header_rect(self) -> Rect {
        Rect {
            x: self.rect.x,
            y: self.rect.y,
            width: self.rect.width,
            height: min_u32(self.header_height, self.rect.height),
        }
    }

    pub const fn heading_rect(self) -> Rect {
        let header = self.header_rect();
        Rect {
            x: header.x.saturating_add(self.inset_x),
            y: header.y,
            width: header.width.saturating_sub(self.inset_x.saturating_mul(2)),
            height: header.height,
        }
    }

    pub const fn footer_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(self.inset_x),
            y: self.rect.bottom().saturating_sub(self.footer_height),
            width: self
                .rect
                .width
                .saturating_sub(self.inset_x.saturating_mul(2)),
            height: min_u32(self.footer_height, self.rect.height),
        }
    }

    pub const fn content_rect(self) -> Rect {
        let reserved = self
            .header_height
            .saturating_add(self.footer_height)
            .saturating_add(self.inset_y.saturating_mul(2));
        Rect {
            x: self.rect.x.saturating_add(self.inset_x),
            y: self
                .rect
                .y
                .saturating_add(self.header_height)
                .saturating_add(self.inset_y),
            width: self
                .rect
                .width
                .saturating_sub(self.inset_x.saturating_mul(2)),
            height: self.rect.height.saturating_sub(reserved),
        }
    }

    pub const fn row_rect(self, index: usize) -> Rect {
        let content = self.content_rect();
        if index >= self.row_count {
            return Rect {
                x: content.x,
                y: content.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: content.x,
            y: content.y.saturating_add(
                (index as u32).saturating_mul(self.row_height.saturating_add(self.row_gap)),
            ),
            width: content.width,
            height: self.row_height,
        }
    }

    pub const fn row_at(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        let mut index = 0;
        while index < self.row_count {
            if rect_contains(self.row_rect(index), x, y) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub const fn trailing_rect(self, index: usize, width: u32, height: u32) -> Rect {
        let row = self.row_rect(index);
        if row.width < width || row.height < height {
            return Rect {
                x: row.x,
                y: row.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: row.right().saturating_sub(width),
            y: row.y.saturating_add(row.height.saturating_sub(height) / 2),
            width,
            height,
        }
    }

    pub const fn footer_trailing_rect(self, width: u32, height: u32) -> Rect {
        let footer = self.footer_rect();
        if footer.width < width || footer.height < height {
            return Rect {
                x: footer.x,
                y: footer.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: footer.right().saturating_sub(width),
            y: footer
                .y
                .saturating_add(footer.height.saturating_sub(height) / 2),
            width,
            height,
        }
    }

    pub const fn is_valid(self) -> bool {
        if self.name.is_empty()
            || self.row_count == 0
            || self.row_count > 32
            || self.rect.width == 0
            || self.rect.height == 0
            || self.row_height == 0
            || self.inset_x.saturating_mul(2) >= self.rect.width
        {
            return false;
        }
        let reserved = self
            .header_height
            .saturating_add(self.footer_height)
            .saturating_add(self.inset_y.saturating_mul(2));
        reserved < self.rect.height
            && self.row_rect(self.row_count - 1).bottom() <= self.content_rect().bottom()
    }

    pub const fn accessibility(self) -> SectionGroupAccessibility<'a> {
        SectionGroupAccessibility {
            role: "group",
            name: self.name,
            focused: self.focused,
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
pub enum GridCellLayout {
    IconLeading,
    IconAbove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCellSlots {
    pub icon: Rect,
    pub primary: Rect,
    pub secondary: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCell<'a> {
    pub rect: Rect,
    pub label: &'a str,
    pub state: ComponentState,
    pub layout: GridCellLayout,
    pub icon_size: u32,
    pub inset: u32,
    pub gap: u32,
    pub secondary_height: u32,
    pub idle_surface: bool,
}

impl<'a> GridCell<'a> {
    pub const fn new(rect: Rect, label: &'a str, layout: GridCellLayout) -> Self {
        Self {
            rect,
            label,
            state: ComponentState::Idle,
            layout,
            icon_size: 40,
            inset: 8,
            gap: 6,
            secondary_height: 20,
            idle_surface: true,
        }
    }

    pub const fn with_state(mut self, state: ComponentState) -> Self {
        self.state = state;
        self
    }

    pub const fn with_spacing(
        mut self,
        icon_size: u32,
        inset: u32,
        gap: u32,
        secondary_height: u32,
    ) -> Self {
        self.icon_size = icon_size;
        self.inset = inset;
        self.gap = gap;
        self.secondary_height = secondary_height;
        self
    }

    pub const fn with_idle_surface(mut self, visible: bool) -> Self {
        self.idle_surface = visible;
        self
    }

    pub const fn slots(self) -> GridCellSlots {
        let inner_width = self.rect.width.saturating_sub(self.inset.saturating_mul(2));
        let inner_height = self
            .rect
            .height
            .saturating_sub(self.inset.saturating_mul(2));
        let icon_size = min_u32(self.icon_size, min_u32(inner_width, inner_height));
        match self.layout {
            GridCellLayout::IconLeading => {
                let primary_x = self
                    .rect
                    .x
                    .saturating_add(self.inset)
                    .saturating_add(icon_size)
                    .saturating_add(self.gap);
                let secondary_height = min_u32(self.secondary_height, inner_height);
                GridCellSlots {
                    icon: Rect {
                        x: self.rect.x.saturating_add(self.inset),
                        y: self.rect.y.saturating_add(self.inset),
                        width: icon_size,
                        height: icon_size,
                    },
                    primary: Rect {
                        x: primary_x,
                        y: self.rect.y.saturating_add(self.inset),
                        width: self
                            .rect
                            .right()
                            .saturating_sub(self.inset)
                            .saturating_sub(primary_x),
                        height: icon_size,
                    },
                    secondary: Rect {
                        x: self.rect.x.saturating_add(self.inset),
                        y: self
                            .rect
                            .bottom()
                            .saturating_sub(self.inset.saturating_add(secondary_height)),
                        width: inner_width,
                        height: secondary_height,
                    },
                }
            }
            GridCellLayout::IconAbove => {
                let icon_x = self
                    .rect
                    .x
                    .saturating_add(self.rect.width.saturating_sub(icon_size) / 2);
                let primary_y = self
                    .rect
                    .y
                    .saturating_add(self.inset)
                    .saturating_add(icon_size)
                    .saturating_add(self.gap);
                GridCellSlots {
                    icon: Rect {
                        x: icon_x,
                        y: self.rect.y.saturating_add(self.inset),
                        width: icon_size,
                        height: icon_size,
                    },
                    primary: Rect {
                        x: self.rect.x.saturating_add(self.inset),
                        y: primary_y,
                        width: inner_width,
                        height: self
                            .rect
                            .bottom()
                            .saturating_sub(self.inset)
                            .saturating_sub(primary_y),
                    },
                    secondary: Rect {
                        x: self.rect.x,
                        y: self.rect.bottom(),
                        width: 0,
                        height: 0,
                    },
                }
            }
        }
    }

    pub const fn is_valid(self) -> bool {
        let slots = self.slots();
        !self.label.is_empty()
            && self.rect.width > self.inset.saturating_mul(2)
            && self.rect.height > self.inset.saturating_mul(2)
            && self.icon_size > 0
            && slots.icon.width > 0
            && slots.primary.width > 0
            && slots.primary.height > 0
    }

    pub const fn focus_rect(self) -> Rect {
        expanded_rect(self.rect, 2)
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
        accessibility("gridcell", self.label, self.state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationOverviewAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub item_count: usize,
    pub column_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationOverview<'a> {
    pub rect: Rect,
    pub name: &'a str,
    pub search_name: &'a str,
    pub search_placeholder: &'a str,
    pub item_count: usize,
    pub column_count: usize,
    pub visible_limit: usize,
    pub horizontal_inset: u32,
    pub search_offset_y: u32,
    pub search_height: u32,
    pub grid_gap: u32,
    pub cell_height: u32,
    pub row_stride: u32,
}

impl<'a> ApplicationOverview<'a> {
    pub const fn new(
        rect: Rect,
        name: &'a str,
        search_name: &'a str,
        search_placeholder: &'a str,
        item_count: usize,
    ) -> Self {
        Self {
            rect,
            name,
            search_name,
            search_placeholder,
            item_count,
            column_count: 3,
            visible_limit: 6,
            horizontal_inset: 24,
            search_offset_y: 54,
            search_height: 42,
            grid_gap: 12,
            cell_height: 100,
            row_stride: 112,
        }
    }

    pub const fn title_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(self.horizontal_inset),
            y: self.rect.y.saturating_add(18),
            width: self
                .rect
                .width
                .saturating_sub(self.horizontal_inset.saturating_mul(2)),
            height: 28,
        }
    }

    pub const fn search_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(self.horizontal_inset),
            y: self.rect.y.saturating_add(self.search_offset_y),
            width: self
                .rect
                .width
                .saturating_sub(self.horizontal_inset.saturating_mul(2)),
            height: self.search_height,
        }
    }

    pub const fn search_field(self, value: &'a str, state: ComponentState) -> SearchField<'a> {
        SearchField::new(
            self.search_rect(),
            self.search_name,
            value,
            self.search_placeholder,
        )
        .with_state(state)
    }

    pub const fn grid_rect(self) -> Rect {
        let search = self.search_rect();
        Rect {
            x: self.rect.x.saturating_add(self.horizontal_inset),
            y: search.bottom().saturating_add(18),
            width: self
                .rect
                .width
                .saturating_sub(self.horizontal_inset.saturating_mul(2)),
            height: self
                .rect
                .bottom()
                .saturating_sub(self.horizontal_inset)
                .saturating_sub(search.bottom().saturating_add(18)),
        }
    }

    pub const fn visible_item_count(self) -> usize {
        if self.item_count < self.visible_limit {
            self.item_count
        } else {
            self.visible_limit
        }
    }

    pub const fn cell_rect(self, index: usize) -> Rect {
        if index >= self.visible_item_count() || self.column_count == 0 {
            return Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: 0,
                height: 0,
            };
        }
        let grid = self.grid_rect();
        let columns = self.column_count as u32;
        let total_gap = self.grid_gap.saturating_mul(columns.saturating_sub(1));
        let cell_width = grid.width.saturating_sub(total_gap) / columns;
        let column = index as u32 % columns;
        let row = index as u32 / columns;
        let x = grid
            .x
            .saturating_add(column.saturating_mul(cell_width.saturating_add(self.grid_gap)));
        Rect {
            x,
            y: grid.y.saturating_add(row.saturating_mul(self.row_stride)),
            width: if column + 1 == columns {
                grid.right().saturating_sub(x)
            } else {
                cell_width
            },
            height: self.cell_height,
        }
    }

    pub const fn cell_at(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        let mut index = 0;
        while index < self.visible_item_count() {
            if rect_contains(self.cell_rect(index), x, y) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub const fn contains(self, x: u32, y: u32) -> bool {
        self.is_valid() && rect_contains(self.rect, x, y)
    }

    pub const fn is_valid(self) -> bool {
        if self.name.is_empty()
            || self.search_name.is_empty()
            || self.search_placeholder.is_empty()
            || self.item_count > 128
            || self.column_count == 0
            || self.column_count > 8
            || self.visible_limit == 0
            || self.visible_limit > 32
            || self.horizontal_inset.saturating_mul(2) >= self.rect.width
            || self.search_height < 32
            || self.cell_height == 0
            || self.row_stride < self.cell_height
        {
            return false;
        }
        let search = self.search_rect();
        let grid = self.grid_rect();
        if search.bottom() > self.rect.bottom() || grid.height < self.cell_height {
            return false;
        }
        let visible = self.visible_item_count();
        visible == 0 || self.cell_rect(visible - 1).bottom() <= grid.bottom()
    }

    pub const fn accessibility(self) -> ApplicationOverviewAccessibility<'a> {
        ApplicationOverviewAccessibility {
            role: "region",
            name: self.name,
            item_count: self.item_count,
            column_count: self.column_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalSearchAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub result_count: usize,
    pub quick_action_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalSearch<'a> {
    pub rect: Rect,
    pub name: &'a str,
    pub search_name: &'a str,
    pub search_placeholder: &'a str,
    pub results_name: &'a str,
    pub quick_actions_name: &'a str,
    pub result_count: usize,
    pub quick_action_count: usize,
    pub visible_result_limit: usize,
    pub horizontal_inset: u32,
    pub search_offset_y: u32,
    pub search_height: u32,
    pub section_gap: u32,
    pub result_row_height: u32,
    pub result_row_stride: u32,
    pub quick_action_height: u32,
    pub quick_action_stride: u32,
}

impl<'a> GlobalSearch<'a> {
    pub const fn new(
        rect: Rect,
        name: &'a str,
        search_name: &'a str,
        search_placeholder: &'a str,
        section_names: (&'a str, &'a str),
        result_count: usize,
        quick_action_count: usize,
    ) -> Self {
        Self {
            rect,
            name,
            search_name,
            search_placeholder,
            results_name: section_names.0,
            quick_actions_name: section_names.1,
            result_count,
            quick_action_count,
            visible_result_limit: 5,
            horizontal_inset: 24,
            search_offset_y: 54,
            search_height: 42,
            section_gap: 18,
            result_row_height: 52,
            result_row_stride: 58,
            quick_action_height: 50,
            quick_action_stride: 62,
        }
    }

    pub const fn title_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(self.horizontal_inset),
            y: self.rect.y.saturating_add(18),
            width: self
                .rect
                .width
                .saturating_sub(self.horizontal_inset.saturating_mul(2)),
            height: 28,
        }
    }

    pub const fn search_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(self.horizontal_inset),
            y: self.rect.y.saturating_add(self.search_offset_y),
            width: self
                .rect
                .width
                .saturating_sub(self.horizontal_inset.saturating_mul(2)),
            height: self.search_height,
        }
    }

    pub const fn search_field(self, value: &'a str, state: ComponentState) -> SearchField<'a> {
        SearchField::new(
            self.search_rect(),
            self.search_name,
            value,
            self.search_placeholder,
        )
        .with_state(state)
    }

    pub const fn content_y(self) -> u32 {
        self.search_rect().bottom().saturating_add(self.section_gap)
    }

    pub const fn split_x(self) -> u32 {
        self.rect.x.saturating_add(self.rect.width / 2)
    }

    pub const fn divider_rect(self) -> Rect {
        Rect {
            x: self.split_x(),
            y: self.content_y(),
            width: 1,
            height: self
                .rect
                .bottom()
                .saturating_sub(self.horizontal_inset)
                .saturating_sub(self.content_y()),
        }
    }

    pub const fn results_header_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_add(self.horizontal_inset),
            y: self.content_y(),
            width: self
                .split_x()
                .saturating_sub(self.rect.x.saturating_add(self.horizontal_inset)),
            height: 16,
        }
    }

    pub const fn quick_actions_header_rect(self) -> Rect {
        let x = self.split_x().saturating_add(22);
        Rect {
            x,
            y: self.content_y(),
            width: self
                .rect
                .right()
                .saturating_sub(self.horizontal_inset)
                .saturating_sub(x),
            height: 16,
        }
    }

    pub const fn visible_result_count(self) -> usize {
        if self.result_count < self.visible_result_limit {
            self.result_count
        } else {
            self.visible_result_limit
        }
    }

    pub const fn result_rect(self, index: usize) -> Rect {
        if index >= self.visible_result_count() {
            return Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: self.rect.x.saturating_add(18),
            y: self
                .content_y()
                .saturating_add(24)
                .saturating_add((index as u32).saturating_mul(self.result_row_stride)),
            width: (self.rect.width / 2).saturating_sub(30),
            height: self.result_row_height,
        }
    }

    pub const fn quick_action_rect(self, index: usize) -> Rect {
        if index >= self.quick_action_count {
            return Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: self.split_x().saturating_add(18),
            y: self
                .content_y()
                .saturating_add(24)
                .saturating_add((index as u32).saturating_mul(self.quick_action_stride)),
            width: (self.rect.width / 2).saturating_sub(42),
            height: self.quick_action_height,
        }
    }

    pub const fn result_at(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        let mut index = 0;
        while index < self.visible_result_count() {
            if rect_contains(self.result_rect(index), x, y) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub const fn quick_action_at(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        let mut index = 0;
        while index < self.quick_action_count {
            if rect_contains(self.quick_action_rect(index), x, y) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub const fn contains(self, x: u32, y: u32) -> bool {
        self.is_valid() && rect_contains(self.rect, x, y)
    }

    pub const fn is_valid(self) -> bool {
        if self.name.is_empty()
            || self.search_name.is_empty()
            || self.search_placeholder.is_empty()
            || self.results_name.is_empty()
            || self.quick_actions_name.is_empty()
            || self.result_count > 128
            || self.quick_action_count == 0
            || self.quick_action_count > 8
            || self.visible_result_limit == 0
            || self.visible_result_limit > 32
            || self.rect.width < 480
            || self.horizontal_inset.saturating_mul(2) >= self.rect.width
            || self.search_height < 32
            || self.result_row_height == 0
            || self.result_row_stride < self.result_row_height
            || self.quick_action_height == 0
            || self.quick_action_stride < self.quick_action_height
        {
            return false;
        }
        let search = self.search_rect();
        if search.bottom() > self.rect.bottom() || self.divider_rect().bottom() > self.rect.bottom()
        {
            return false;
        }
        let visible_results = self.visible_result_count();
        let results_fit = visible_results == 0
            || self.result_rect(visible_results - 1).bottom() <= self.rect.bottom();
        results_fit
            && self.quick_action_rect(self.quick_action_count - 1).bottom() <= self.rect.bottom()
    }

    pub const fn accessibility(self) -> GlobalSearchAccessibility<'a> {
        GlobalSearchAccessibility {
            role: "search",
            name: self.name,
            result_count: self.result_count,
            quick_action_count: self.quick_action_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunningAppDockAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub item_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunningAppDockItemAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunningAppDock<'a> {
    pub rect: Rect,
    pub name: &'a str,
    pub item_count: usize,
    pub item_width: u32,
    pub icon_size: u32,
    pub raster_icon_size: u32,
    pub indicator_size: u32,
    pub indicator_bottom_inset: u32,
}

impl<'a> RunningAppDock<'a> {
    pub const fn new(rect: Rect, name: &'a str, item_count: usize) -> Self {
        Self {
            rect,
            name,
            item_count,
            item_width: 72,
            icon_size: 64,
            raster_icon_size: 48,
            indicator_size: 6,
            indicator_bottom_inset: 1,
        }
    }

    pub const fn item_rect(self, index: usize) -> Rect {
        if index >= self.item_count {
            return Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: self
                .rect
                .x
                .saturating_add((index as u32).saturating_mul(self.item_width)),
            y: self.rect.y,
            width: self.item_width,
            height: self.rect.height,
        }
    }

    pub const fn icon_rect(self, index: usize) -> Rect {
        let item = self.item_rect(index);
        if item.width == 0 {
            return item;
        }
        let size = min_u32(self.icon_size, min_u32(item.width, item.height));
        Rect {
            x: item.x.saturating_add(item.width.saturating_sub(size) / 2),
            y: item.y.saturating_add(item.height.saturating_sub(size) / 2),
            width: size,
            height: size,
        }
    }

    pub const fn indicator_rect(self, index: usize) -> Rect {
        let item = self.item_rect(index);
        if item.width == 0 {
            return item;
        }
        Rect {
            x: item
                .x
                .saturating_add(item.width.saturating_sub(self.indicator_size) / 2),
            y: item
                .bottom()
                .saturating_sub(self.indicator_bottom_inset)
                .saturating_sub(self.indicator_size),
            width: self.indicator_size,
            height: self.indicator_size,
        }
    }

    pub const fn raster_icon_rect(self, index: usize) -> Rect {
        let icon = self.icon_rect(index);
        if icon.width == 0 {
            return icon;
        }
        let size = min_u32(self.raster_icon_size, min_u32(icon.width, icon.height));
        Rect {
            x: icon.x.saturating_add(icon.width.saturating_sub(size) / 2),
            y: icon.y.saturating_add(icon.height.saturating_sub(size) / 2),
            width: size,
            height: size,
        }
    }

    pub const fn item_at(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() || !rect_contains(self.rect, x, y) {
            return None;
        }
        let index = ((x - self.rect.x) / self.item_width) as usize;
        if index < self.item_count {
            Some(index)
        } else {
            None
        }
    }

    pub const fn is_valid(self) -> bool {
        if self.name.is_empty()
            || self.item_count == 0
            || self.item_count > 16
            || self.item_width < 48
            || self.rect.height < 48
            || self.icon_size == 0
            || self.icon_size > self.item_width
            || self.icon_size > self.rect.height
            || self.raster_icon_size == 0
            || self.raster_icon_size > self.icon_size
            || self.indicator_size == 0
            || self.indicator_size > self.item_width
            || self
                .indicator_size
                .saturating_add(self.indicator_bottom_inset)
                > self.rect.height
        {
            return false;
        }
        self.rect.width == self.item_width.saturating_mul(self.item_count as u32)
    }

    pub const fn accessibility(self) -> RunningAppDockAccessibility<'a> {
        RunningAppDockAccessibility {
            role: "toolbar",
            name: self.name,
            item_count: self.item_count,
        }
    }

    pub const fn item_accessibility(
        self,
        index: usize,
        name: &'a str,
        running: bool,
    ) -> Option<RunningAppDockItemAccessibility<'a>> {
        if !self.is_valid() || index >= self.item_count || name.is_empty() {
            return None;
        }
        Some(RunningAppDockItemAccessibility {
            role: "button",
            name,
            running,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceSwitcherAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub workspace_count: usize,
    pub active_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceNavigationKey {
    Previous,
    Next,
    Home,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceSwitcher<'a> {
    pub rect: Rect,
    pub name: &'a str,
    pub workspace_count: usize,
    pub active_index: usize,
    pub item_width: u32,
    pub thumbnail_horizontal_inset: u32,
    pub thumbnail_vertical_inset: u32,
    pub indicator_horizontal_inset: u32,
    pub indicator_bottom_inset: u32,
    pub indicator_height: u32,
}

impl<'a> WorkspaceSwitcher<'a> {
    pub const fn new(
        rect: Rect,
        name: &'a str,
        workspace_count: usize,
        active_index: usize,
    ) -> Self {
        Self {
            rect,
            name,
            workspace_count,
            active_index,
            item_width: 60,
            thumbnail_horizontal_inset: 5,
            thumbnail_vertical_inset: 10,
            indicator_horizontal_inset: 5,
            indicator_bottom_inset: 0,
            indicator_height: 3,
        }
    }

    pub const fn item_rect(self, index: usize) -> Rect {
        if index >= self.workspace_count {
            return Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: 0,
                height: 0,
            };
        }
        Rect {
            x: self
                .rect
                .x
                .saturating_add((index as u32).saturating_mul(self.item_width)),
            y: self.rect.y,
            width: self.item_width,
            height: self.rect.height,
        }
    }

    pub const fn thumbnail_rect(self, index: usize) -> Rect {
        let item = self.item_rect(index);
        if item.width == 0 {
            return item;
        }
        Rect {
            x: item.x.saturating_add(self.thumbnail_horizontal_inset),
            y: item.y.saturating_add(self.thumbnail_vertical_inset),
            width: item
                .width
                .saturating_sub(self.thumbnail_horizontal_inset.saturating_mul(2)),
            height: item
                .height
                .saturating_sub(self.thumbnail_vertical_inset.saturating_mul(2)),
        }
    }

    pub const fn active_indicator_rect(self) -> Rect {
        let thumbnail = self.thumbnail_rect(self.active_index);
        if thumbnail.width == 0 {
            return thumbnail;
        }
        Rect {
            x: thumbnail.x.saturating_add(self.indicator_horizontal_inset),
            y: thumbnail
                .bottom()
                .saturating_sub(self.indicator_bottom_inset)
                .saturating_sub(self.indicator_height),
            width: thumbnail
                .width
                .saturating_sub(self.indicator_horizontal_inset.saturating_mul(2)),
            height: self.indicator_height,
        }
    }

    pub const fn item_at(self, x: u32, y: u32) -> Option<usize> {
        if !self.is_valid() || !rect_contains(self.rect, x, y) {
            return None;
        }
        let index = ((x - self.rect.x) / self.item_width) as usize;
        if index < self.workspace_count {
            Some(index)
        } else {
            None
        }
    }

    pub const fn is_active(self, index: usize) -> bool {
        self.is_valid() && index < self.workspace_count && index == self.active_index
    }

    pub const fn keyboard_target(self, key: WorkspaceNavigationKey) -> Option<usize> {
        if !self.is_valid() {
            return None;
        }
        match key {
            WorkspaceNavigationKey::Previous => self.active_index.checked_sub(1),
            WorkspaceNavigationKey::Next => {
                let next = self.active_index + 1;
                if next < self.workspace_count {
                    Some(next)
                } else {
                    None
                }
            }
            WorkspaceNavigationKey::Home => Some(0),
            WorkspaceNavigationKey::End => Some(self.workspace_count - 1),
        }
    }

    pub const fn is_valid(self) -> bool {
        if self.name.is_empty()
            || self.workspace_count == 0
            || self.workspace_count > 16
            || self.active_index >= self.workspace_count
            || self.item_width < 44
            || self.rect.height < 40
            || self.thumbnail_horizontal_inset.saturating_mul(2) >= self.item_width
            || self.thumbnail_vertical_inset.saturating_mul(2) >= self.rect.height
            || self.indicator_height == 0
        {
            return false;
        }
        let thumbnail = self.thumbnail_rect(self.active_index);
        self.rect.width == self.item_width.saturating_mul(self.workspace_count as u32)
            && self.indicator_horizontal_inset.saturating_mul(2) < thumbnail.width
            && self
                .indicator_height
                .saturating_add(self.indicator_bottom_inset)
                <= thumbnail.height
    }

    pub const fn accessibility(self) -> WorkspaceSwitcherAccessibility<'a> {
        WorkspaceSwitcherAccessibility {
            role: "tablist",
            name: self.name,
            workspace_count: self.workspace_count,
            active_index: self.active_index,
        }
    }

    pub const fn item_accessibility(
        self,
        index: usize,
        name: &'a str,
    ) -> Option<WorkspaceAccessibility<'a>> {
        if !self.is_valid() || index >= self.workspace_count || name.is_empty() {
            return None;
        }
        Some(WorkspaceAccessibility {
            role: "tab",
            name,
            selected: index == self.active_index,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationAccessibility<'a> {
    pub role: &'static str,
    pub name: &'a str,
    pub description: &'a str,
    pub source: &'a str,
    pub live: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationSlots {
    pub icon: Rect,
    pub title: Rect,
    pub body: Rect,
    pub source: Rect,
    pub dismiss: Rect,
    pub dismiss_icon: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKey {
    Escape,
    Activate(ActivationKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationToast<'a> {
    pub rect: Rect,
    pub source: &'a str,
    pub title: &'a str,
    pub body: &'a str,
}

impl<'a> NotificationToast<'a> {
    pub const fn new(rect: Rect, source: &'a str, title: &'a str, body: &'a str) -> Self {
        Self {
            rect,
            source,
            title,
            body,
        }
    }

    pub const fn slots(self) -> NotificationSlots {
        let spacious = self.rect.width >= 360;
        let padding: u32 = if spacious { 18 } else { 10 };
        let icon_size: u32 = if spacious { 48 } else { 30 };
        let dismiss_size: u32 = if spacious { 48 } else { 40 };
        let text_x = self
            .rect
            .x
            .saturating_add(padding)
            .saturating_add(icon_size)
            .saturating_add(padding);
        let text_right = self
            .rect
            .right()
            .saturating_sub(dismiss_size.saturating_add(4));
        let dismiss = Rect {
            x: self.rect.right().saturating_sub(dismiss_size),
            y: self.rect.y,
            width: dismiss_size,
            height: dismiss_size,
        };
        let dismiss_icon_size: u32 = if spacious { 14 } else { 12 };
        NotificationSlots {
            icon: Rect {
                x: self.rect.x.saturating_add(padding),
                y: self.rect.y.saturating_add(padding),
                width: icon_size,
                height: icon_size,
            },
            title: Rect {
                x: text_x,
                y: self.rect.y.saturating_add(padding.saturating_sub(2)),
                width: text_right.saturating_sub(text_x),
                height: if spacious { 22 } else { 16 },
            },
            body: Rect {
                x: text_x,
                y: self
                    .rect
                    .y
                    .saturating_add(padding)
                    .saturating_add(if spacious { 26 } else { 18 }),
                width: text_right.saturating_sub(text_x),
                height: if spacious { 20 } else { 14 },
            },
            source: Rect {
                x: text_x,
                y: self
                    .rect
                    .bottom()
                    .saturating_sub(if spacious { 24 } else { 16 }),
                width: text_right.saturating_sub(text_x),
                height: if spacious { 18 } else { 12 },
            },
            dismiss,
            dismiss_icon: Rect {
                x: dismiss
                    .x
                    .saturating_add(dismiss.width.saturating_sub(dismiss_icon_size) / 2),
                y: dismiss
                    .y
                    .saturating_add(dismiss.height.saturating_sub(dismiss_icon_size) / 2),
                width: dismiss_icon_size,
                height: dismiss_icon_size,
            },
        }
    }

    pub const fn dismiss_hit(self, x: u32, y: u32) -> bool {
        self.is_valid() && rect_contains(self.slots().dismiss, x, y)
    }

    pub const fn keyboard_dismisses(self, key: NotificationKey) -> bool {
        self.is_valid()
            && matches!(
                key,
                NotificationKey::Escape
                    | NotificationKey::Activate(ActivationKey::Enter | ActivationKey::Space)
            )
    }

    pub const fn is_valid(self) -> bool {
        if self.source.is_empty()
            || self.title.is_empty()
            || self.rect.width < 240
            || self.rect.height < 72
        {
            return false;
        }
        let slots = self.slots();
        slots.icon.right() <= self.rect.right()
            && slots.icon.bottom() <= self.rect.bottom()
            && slots.title.right() <= slots.dismiss.x
            && slots.body.right() <= slots.dismiss.x
            && slots.source.right() <= slots.dismiss.x
            && slots.source.bottom() <= self.rect.bottom()
            && slots.dismiss.right() == self.rect.right()
            && slots.dismiss.bottom() <= self.rect.bottom()
    }

    pub const fn accessibility(self) -> NotificationAccessibility<'a> {
        NotificationAccessibility {
            role: "status",
            name: self.title,
            description: self.body,
            source: self.source,
            live: "polite",
        }
    }

    pub const fn dismiss_accessibility(self) -> ComponentAccessibility<'static> {
        ComponentAccessibility {
            role: "button",
            name: "Dismiss notification",
            disabled: false,
            busy: false,
            selected: false,
        }
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

const fn max_u32(left: u32, right: u32) -> u32 {
    if left > right {
        left
    } else {
        right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_tracks_the_nineteen_shared_primitives() {
        assert_eq!(SharedComponentKind::ALL.len(), 22);
        assert_eq!(
            SharedComponentKind::ALL
                .into_iter()
                .filter(|component| component.is_shared_primitive())
                .collect::<Vec<_>>(),
            vec![
                SharedComponentKind::TopSystemBar,
                SharedComponentKind::WindowFrame,
                SharedComponentKind::SidebarNavigation,
                SharedComponentKind::Toolbar,
                SharedComponentKind::SegmentedControl,
                SharedComponentKind::SearchField,
                SharedComponentKind::StandardButton,
                SharedComponentKind::IconButton,
                SharedComponentKind::Switch,
                SharedComponentKind::Menu,
                SharedComponentKind::ListRow,
                SharedComponentKind::GridCell,
                SharedComponentKind::MetadataRow,
                SharedComponentKind::SectionGroup,
                SharedComponentKind::ApplicationOverview,
                SharedComponentKind::GlobalSearch,
                SharedComponentKind::RunningAppDock,
                SharedComponentKind::WorkspaceSwitcher,
                SharedComponentKind::Notification,
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
            let cell = GridCell::new(rect, "Item", GridCellLayout::IconLeading).with_state(state);
            let switch = SwitchControl::new(rect, "Setting", false).with_state(state);
            let segmented = SegmentedControl::new(rect, "Theme", 3, 0).with_state(state);
            assert!(!button.pointer_hit(11, 21));
            assert!(!row.pointer_hit(11, 21));
            assert!(!cell.pointer_hit(11, 21));
            assert!(!switch.pointer_toggles(11, 21));
            assert_eq!(segmented.hit_test(11, 21), None);
            assert!(!button.keyboard_activates(ActivationKey::Enter));
            assert!(!row.keyboard_activates(ActivationKey::Space));
            assert!(!cell.keyboard_activates(ActivationKey::Space));
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
        let top_bar = TopSystemBar::new(rect, "");
        let row = ListRow::new(rect, "", ListRowRole::Option);
        let cell = GridCell::new(rect, "", GridCellLayout::IconLeading);
        let icon = IconButton::new(rect, "", IconButtonGlyph::Back);
        let search = SearchField::new(rect, "", "", "Search");
        let switch = SwitchControl::new(rect, "", false);
        let segmented = SegmentedControl::new(rect, "", 3, 0);
        let toolbar = Toolbar::new(rect, "");
        let navigation = SidebarNavigation::new(rect, "", rect, 40);
        let menu = Menu::new(rect, "", 2, 0, 0, 20, 0);
        let metadata = MetadataRow::new(rect, "", "Value");
        let section = SectionGroup::new(rect, "", 1);
        let overview = ApplicationOverview::new(rect, "", "Search", "Search apps", 3);
        let global_search = GlobalSearch::new(
            rect,
            "",
            "Search",
            "Search apps",
            ("Results", "Actions"),
            1,
            1,
        );
        let running_dock = RunningAppDock::new(rect, "", 1);
        let workspace_switcher = WorkspaceSwitcher::new(rect, "", 1, 0);
        let notification = NotificationToast::new(rect, "", "Update ready", "Restart later");
        assert!(!button.can_activate());
        assert!(!top_bar.is_valid());
        assert!(!row.can_activate());
        assert!(!cell.can_activate());
        assert!(!icon.can_activate());
        assert!(!search.accepts_input());
        assert!(!switch.can_toggle());
        assert!(!segmented.is_valid());
        assert!(!toolbar.is_valid());
        assert!(!navigation.is_valid());
        assert!(!menu.is_valid());
        assert!(!metadata.is_valid());
        assert!(!section.is_valid());
        assert!(!overview.is_valid());
        assert!(!global_search.is_valid());
        assert!(!running_dock.is_valid());
        assert!(!workspace_switcher.is_valid());
        assert!(!notification.is_valid());
        assert_eq!(navigation.hit_test(11, 21, 1), None);
    }

    #[test]
    fn grid_cell_layouts_share_stable_activation_and_semantics() {
        let leading = GridCell::new(
            Rect {
                x: 24,
                y: 184,
                width: 180,
                height: 100,
            },
            "Files",
            GridCellLayout::IconLeading,
        )
        .with_spacing(40, 14, 6, 22)
        .with_state(ComponentState::Selected);
        let leading_slots = leading.slots();
        assert_eq!(leading_slots.icon.x, 38);
        assert_eq!(leading_slots.primary.x, 84);
        assert!(leading_slots.secondary.bottom() < leading.rect.bottom());
        assert!(leading.pointer_hit(24, 184));
        assert!(leading.keyboard_activates(ActivationKey::Enter));
        assert_eq!(leading.accessibility().role, "gridcell");
        assert!(leading.accessibility().selected);

        let above = GridCell::new(
            Rect {
                x: 24,
                y: 60,
                width: 104,
                height: 104,
            },
            "Settings",
            GridCellLayout::IconAbove,
        )
        .with_spacing(64, 8, 5, 0);
        let above_slots = above.slots();
        assert_eq!(above_slots.icon.x, 44);
        assert_eq!(above_slots.primary.y, 137);
        assert_eq!(above_slots.secondary.width, 0);
        assert!(above.is_valid());
    }

    #[test]
    fn application_overview_composes_search_and_exact_grid_geometry() {
        let overview = ApplicationOverview::new(
            Rect {
                x: 90,
                y: 70,
                width: 620,
                height: 460,
            },
            "Applications",
            "Search applications",
            "Search apps",
            6,
        );
        assert!(overview.is_valid());
        assert_eq!(overview.search_rect().x, 114);
        assert_eq!(overview.search_rect().bottom(), 166);
        assert_eq!(overview.grid_rect().y, 184);
        assert_eq!(overview.cell_rect(0).width, 182);
        assert_eq!(overview.cell_rect(2).right(), 686);
        assert_eq!(overview.cell_rect(3).y, 296);
        assert_eq!(overview.cell_at(114, 184), Some(0));
        assert_eq!(overview.cell_at(296, 184), None);
        assert_eq!(overview.cell_at(308, 184), Some(1));
        assert!(overview.contains(90, 70));
        assert!(!overview.contains(710, 70));
        assert_eq!(
            overview.search_field("term", ComponentState::Idle).value,
            "term"
        );
        let semantics = overview.accessibility();
        assert_eq!(semantics.role, "region");
        assert_eq!(semantics.item_count, 6);
        assert_eq!(semantics.column_count, 3);
    }

    #[test]
    fn global_search_composes_split_content_and_rejects_gaps() {
        let search = GlobalSearch::new(
            Rect {
                x: 40,
                y: 70,
                width: 720,
                height: 460,
            },
            "Global Search",
            "Search applications",
            "Search apps",
            ("Results", "Quick actions"),
            6,
            3,
        );
        assert!(search.is_valid());
        assert_eq!(search.search_rect().x, 64);
        assert_eq!(search.content_y(), 184);
        assert_eq!(search.split_x(), 400);
        assert_eq!(
            search.result_rect(0),
            Rect {
                x: 58,
                y: 208,
                width: 330,
                height: 52
            }
        );
        assert_eq!(search.result_rect(4).bottom(), 492);
        assert_eq!(search.result_rect(5).width, 0);
        assert_eq!(
            search.quick_action_rect(0),
            Rect {
                x: 418,
                y: 208,
                width: 318,
                height: 50
            }
        );
        assert_eq!(search.result_at(58, 208), Some(0));
        assert_eq!(search.result_at(58, 260), None);
        assert_eq!(search.quick_action_at(418, 208), Some(0));
        assert_eq!(search.quick_action_at(418, 258), None);
        assert_eq!(search.result_at(64, 184), None);
        assert!(search.contains(40, 70));
        assert!(!search.contains(760, 70));
        let semantics = search.accessibility();
        assert_eq!(semantics.role, "search");
        assert_eq!(semantics.result_count, 6);
        assert_eq!(semantics.quick_action_count, 3);
    }

    #[test]
    fn running_app_dock_centers_content_and_exposes_running_semantics() {
        let dock = RunningAppDock::new(
            Rect {
                x: 272,
                y: 0,
                width: 216,
                height: 72,
            },
            "Running applications",
            3,
        );
        assert!(dock.is_valid());
        assert_eq!(
            dock.item_rect(0),
            Rect {
                x: 272,
                y: 0,
                width: 72,
                height: 72,
            }
        );
        assert_eq!(
            dock.icon_rect(0),
            Rect {
                x: 276,
                y: 4,
                width: 64,
                height: 64,
            }
        );
        assert_eq!(
            dock.indicator_rect(0),
            Rect {
                x: 305,
                y: 65,
                width: 6,
                height: 6,
            }
        );
        assert_eq!(
            dock.raster_icon_rect(0),
            Rect {
                x: 284,
                y: 12,
                width: 48,
                height: 48,
            }
        );
        assert_eq!(dock.item_at(272, 0), Some(0));
        assert_eq!(dock.item_at(343, 71), Some(0));
        assert_eq!(dock.item_at(344, 20), Some(1));
        assert_eq!(dock.item_at(488, 20), None);
        assert_eq!(dock.item_rect(3).width, 0);
        let semantics = dock.accessibility();
        assert_eq!(semantics.role, "toolbar");
        assert_eq!(semantics.item_count, 3);
        let item = dock.item_accessibility(1, "Settings", true).unwrap();
        assert_eq!(item.role, "button");
        assert!(item.running);
        assert!(dock.item_accessibility(3, "Missing", false).is_none());
    }

    #[test]
    fn workspace_switcher_owns_targets_thumbnails_and_active_semantics() {
        let switcher = WorkspaceSwitcher::new(
            Rect {
                x: 580,
                y: 0,
                width: 180,
                height: 72,
            },
            "Workspaces",
            3,
            1,
        );
        assert!(switcher.is_valid());
        assert_eq!(
            switcher.item_rect(0),
            Rect {
                x: 580,
                y: 0,
                width: 60,
                height: 72,
            }
        );
        assert_eq!(
            switcher.thumbnail_rect(1),
            Rect {
                x: 645,
                y: 10,
                width: 50,
                height: 52,
            }
        );
        assert_eq!(
            switcher.active_indicator_rect(),
            Rect {
                x: 650,
                y: 59,
                width: 40,
                height: 3,
            }
        );
        assert_eq!(switcher.item_at(580, 0), Some(0));
        assert_eq!(switcher.item_at(639, 71), Some(0));
        assert_eq!(switcher.item_at(640, 20), Some(1));
        assert_eq!(switcher.item_at(760, 20), None);
        assert!(switcher.is_active(1));
        assert!(!switcher.is_active(2));
        assert_eq!(
            switcher.keyboard_target(WorkspaceNavigationKey::Previous),
            Some(0)
        );
        assert_eq!(
            switcher.keyboard_target(WorkspaceNavigationKey::Next),
            Some(2)
        );
        assert_eq!(
            switcher.keyboard_target(WorkspaceNavigationKey::Home),
            Some(0)
        );
        assert_eq!(
            switcher.keyboard_target(WorkspaceNavigationKey::End),
            Some(2)
        );
        let semantics = switcher.accessibility();
        assert_eq!(semantics.role, "tablist");
        assert_eq!(semantics.active_index, 1);
        let item = switcher.item_accessibility(1, "Workspace 2").unwrap();
        assert_eq!(item.role, "tab");
        assert!(item.selected);
        assert!(switcher.item_accessibility(3, "Missing").is_none());
    }

    #[test]
    fn notification_owns_content_dismissal_and_live_semantics() {
        let notification = NotificationToast::new(
            Rect {
                x: 416,
                y: 404,
                width: 360,
                height: 88,
            },
            "Aqua System",
            "Update ready",
            "Restart when convenient.",
        );
        assert!(notification.is_valid());
        let slots = notification.slots();
        assert_eq!(
            slots.icon,
            Rect {
                x: 434,
                y: 422,
                width: 48,
                height: 48,
            }
        );
        assert_eq!(
            slots.dismiss,
            Rect {
                x: 728,
                y: 404,
                width: 48,
                height: 48,
            }
        );
        assert_eq!(slots.title.x, 500);
        assert_eq!(
            slots.source.bottom(),
            notification.rect.bottom().saturating_sub(6)
        );
        assert!(!notification.dismiss_hit(727, 404));
        assert!(notification.dismiss_hit(728, 404));
        assert!(notification.dismiss_hit(775, 451));
        assert!(!notification.dismiss_hit(776, 451));
        assert!(notification.keyboard_dismisses(NotificationKey::Escape));
        assert!(notification.keyboard_dismisses(NotificationKey::Activate(ActivationKey::Enter)));
        assert!(!notification.keyboard_dismisses(NotificationKey::Activate(ActivationKey::Other)));
        let semantics = notification.accessibility();
        assert_eq!(semantics.role, "status");
        assert_eq!(semantics.live, "polite");
        assert_eq!(semantics.name, "Update ready");
        let dismiss = notification.dismiss_accessibility();
        assert_eq!(dismiss.role, "button");
        assert_eq!(dismiss.name, "Dismiss notification");
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

    #[test]
    fn window_frame_unifies_title_controls_move_and_resize_geometry() {
        let frame = WindowFrame::new(
            Rect {
                x: 0,
                y: 0,
                width: 680,
                height: 430,
            },
            "Terminal",
            48,
        );
        assert!(frame.is_valid());
        assert_eq!(frame.titlebar_rect().height, 48);
        assert_eq!(frame.control_at(20, 20), Some(WindowControl::Close));
        assert!(!frame.move_hit(20, 20));
        assert!(frame.move_hit(120, 47));
        assert!(!frame.move_hit(120, 48));
        assert!(frame.resize_hit(679, 429));
        assert!(!frame.resize_hit(655, 405));
        assert_eq!(frame.title_rect().x, 92);
        assert_eq!(frame.accessibility().role, "window");
        assert_eq!(frame.accessibility().name, "Terminal");
    }

    #[test]
    fn menu_rows_share_pointer_keyboard_and_accessibility_geometry() {
        let menu = Menu::new(
            Rect {
                x: 108,
                y: 32,
                width: 120,
                height: 72,
            },
            "Files actions",
            2,
            0,
            0,
            32,
            4,
        );
        assert!(menu.is_valid());
        assert_eq!(menu.item_rect(1).y, 68);
        assert_eq!(menu.item_at(120, 63), Some(0));
        assert_eq!(menu.item_at(120, 65), None);
        assert_eq!(menu.item_at(120, 68), Some(1));
        assert_eq!(menu.keyboard_target(MenuNavigationKey::Previous), Some(1));
        assert_eq!(menu.keyboard_target(MenuNavigationKey::End), Some(1));
        assert_eq!(menu.accessibility().role, "menu");
        let item = menu
            .item_accessibility(1, "Properties", false, false)
            .expect("bounded item semantics should exist");
        assert_eq!(item.role, "menuitem");
        assert!(!item.selected);
    }

    #[test]
    fn section_group_bounds_header_content_footer_rows_and_trailing_controls() {
        let section = SectionGroup::new(
            Rect {
                x: 24,
                y: 184,
                width: 432,
                height: 92,
            },
            "File details",
            2,
        )
        .with_structure(0, 34, 16, 8, 18, 4)
        .with_focus(true);
        assert!(section.is_valid());
        assert_eq!(section.content_rect().y, 192);
        assert_eq!(section.row_rect(1).y, 214);
        assert_eq!(section.footer_rect().y, 242);
        assert_eq!(section.trailing_rect(1, 138, 18).x, 302);
        assert_eq!(section.footer_trailing_rect(138, 30).x, 302);
        assert_eq!(section.row_at(40, 198), Some(0));
        assert_eq!(section.row_at(40, 212), None);
        assert_eq!(section.accessibility().role, "group");
        assert!(section.accessibility().focused);
    }

    #[test]
    fn metadata_row_bounds_read_only_label_and_value_columns() {
        let row = MetadataRow::new(
            Rect {
                x: 40,
                y: 192,
                width: 400,
                height: 18,
            },
            "Location",
            "/home/aqua",
        )
        .with_columns(80, 8)
        .with_emphasis(true);
        assert!(row.is_valid());
        assert_eq!(row.slots().label.width, 80);
        assert_eq!(row.slots().value.x, 128);
        assert_eq!(row.slots().value.right(), row.rect.right());
        assert!(!row.accepts_input());
        let semantics = row.accessibility();
        assert_eq!(semantics.role, "definition");
        assert_eq!(semantics.name, "Location");
        assert_eq!(semantics.value, "/home/aqua");
        assert!(semantics.read_only);
        assert!(semantics.emphasized);
    }

    #[test]
    fn top_system_bar_unifies_brand_clock_status_and_session_geometry() {
        let bar = TopSystemBar::new(
            Rect {
                x: 0,
                y: 0,
                width: 1536,
                height: 36,
            },
            "Aqua system bar",
        );
        assert!(bar.is_valid());
        assert_eq!(bar.status_rect(TopSystemStatus::Audio).x, 1400);
        assert_eq!(bar.status_rect(TopSystemStatus::Network).x, 1439);
        assert_eq!(bar.status_rect(TopSystemStatus::Battery).x, 1478);
        assert_eq!(bar.session_rect().x, 1508);
        assert_eq!(bar.separator_rect().y, 35);
        assert_eq!(bar.status_at(1439, 18), Some(TopSystemStatus::Network));
        assert_eq!(bar.status_at(1430, 18), None);
        assert!(bar.session_hit(1535, 18));
        assert!(!bar.session_hit(1507, 18));
        assert_eq!(bar.accessibility().role, "banner");
        assert_eq!(bar.session_accessibility().role, "button");
        let battery = bar.status_accessibility(TopSystemStatus::Battery, true, Some(87));
        assert_eq!(battery.name, "Battery");
        assert_eq!(battery.percent, Some(87));
    }
}
