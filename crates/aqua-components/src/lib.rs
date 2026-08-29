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
            Self::SearchField
                | Self::StandardButton
                | Self::IconButton
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
    fn catalog_tracks_the_five_shared_primitives() {
        assert_eq!(SharedComponentKind::ALL.len(), 22);
        assert_eq!(
            SharedComponentKind::ALL
                .into_iter()
                .filter(|component| component.is_shared_primitive())
                .collect::<Vec<_>>(),
            vec![
                SharedComponentKind::SidebarNavigation,
                SharedComponentKind::SearchField,
                SharedComponentKind::StandardButton,
                SharedComponentKind::IconButton,
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
            assert!(!button.pointer_hit(11, 21));
            assert!(!row.pointer_hit(11, 21));
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
        let navigation = SidebarNavigation::new(rect, "", rect, 40);
        assert!(!button.can_activate());
        assert!(!row.can_activate());
        assert!(!icon.can_activate());
        assert!(!search.accepts_input());
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
}
