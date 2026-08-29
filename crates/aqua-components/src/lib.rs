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
    fn catalog_tracks_the_thirteen_shared_primitives() {
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
                SharedComponentKind::MetadataRow,
                SharedComponentKind::SectionGroup,
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
        let top_bar = TopSystemBar::new(rect, "");
        let row = ListRow::new(rect, "", ListRowRole::Option);
        let icon = IconButton::new(rect, "", IconButtonGlyph::Back);
        let search = SearchField::new(rect, "", "", "Search");
        let switch = SwitchControl::new(rect, "", false);
        let segmented = SegmentedControl::new(rect, "", 3, 0);
        let toolbar = Toolbar::new(rect, "");
        let navigation = SidebarNavigation::new(rect, "", rect, 40);
        let menu = Menu::new(rect, "", 2, 0, 0, 20, 0);
        let metadata = MetadataRow::new(rect, "", "Value");
        let section = SectionGroup::new(rect, "", 1);
        assert!(!button.can_activate());
        assert!(!top_bar.is_valid());
        assert!(!row.can_activate());
        assert!(!icon.can_activate());
        assert!(!search.accepts_input());
        assert!(!switch.can_toggle());
        assert!(!segmented.is_valid());
        assert!(!toolbar.is_valid());
        assert!(!navigation.is_valid());
        assert!(!menu.is_valid());
        assert!(!metadata.is_valid());
        assert!(!section.is_valid());
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
