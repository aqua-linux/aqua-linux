use super::{
    checksum_bytes, draw_fitted_bitmap_text, draw_rect_outline, draw_system_surface_primitives,
    fill_rounded_rect, inset_rect, window_chrome_palette, FittedTextOptions,
};
use aqua_scene::{Rect, Viewport};
use aqua_shell::AquaTheme;
use aqua_text::{OutputScale, TextRole};

pub const COMPONENT_FIXTURE_REVISION: &str = "aqua-component-fixtures-1";

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
        matches!(self, Self::StandardButton)
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
    pub const STANDARD_BUTTON_STATES: [Self; 10] = [
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardButtonVariant {
    Secondary,
    Primary,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonActivationKey {
    Enter,
    Space,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonAccessibility<'a> {
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
        inset_rect(self.rect, 8, 0)
    }

    pub const fn focus_rect(self) -> Rect {
        Rect {
            x: self.rect.x.saturating_sub(3),
            y: self.rect.y.saturating_sub(3),
            width: self.rect.width.saturating_add(6),
            height: self.rect.height.saturating_add(6),
        }
    }

    pub const fn can_activate(self) -> bool {
        !matches!(
            self.state,
            ComponentState::Disabled | ComponentState::Loading
        )
    }

    pub const fn pointer_hit(self, x: u32, y: u32) -> bool {
        self.can_activate()
            && x >= self.rect.x
            && x < self.rect.right()
            && y >= self.rect.y
            && y < self.rect.bottom()
    }

    pub const fn keyboard_activates(self, key: ButtonActivationKey) -> bool {
        self.can_activate()
            && matches!(key, ButtonActivationKey::Enter | ButtonActivationKey::Space)
    }

    pub const fn accessibility(self) -> ButtonAccessibility<'a> {
        ButtonAccessibility {
            role: "button",
            name: self.label,
            disabled: matches!(self.state, ComponentState::Disabled),
            busy: matches!(self.state, ComponentState::Loading),
            selected: matches!(self.state, ComponentState::Selected),
        }
    }
}

pub(crate) fn draw_standard_button(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    button: StandardButton<'_>,
    theme: AquaTheme,
    scale: OutputScale,
) -> usize {
    let palette = window_chrome_palette(theme);
    let primary = matches!(button.variant, StandardButtonVariant::Primary);
    let destructive = matches!(button.variant, StandardButtonVariant::Destructive);
    let (fill, text) = match button.state {
        ComponentState::Disabled => (palette.row_alternate, palette.secondary_text),
        ComponentState::Hover => (palette.hover, palette.text),
        ComponentState::Pressed => (palette.accent_soft, palette.text),
        ComponentState::Selected => (palette.accent_soft, palette.accent),
        ComponentState::Error => ([0xc9, 0x3c, 0x47, 0xff], [0xff, 0xff, 0xff, 0xff]),
        ComponentState::Success => ([0x2c, 0x8a, 0x59, 0xff], [0xff, 0xff, 0xff, 0xff]),
        ComponentState::Attention => ([0xd1, 0x8b, 0x24, 0xff], [0x1b, 0x20, 0x27, 0xff]),
        ComponentState::Idle | ComponentState::KeyboardFocus | ComponentState::Loading => {
            if destructive {
                ([0xb9, 0x32, 0x3e, 0xff], [0xff, 0xff, 0xff, 0xff])
            } else if primary {
                (palette.accent, [0xff, 0xff, 0xff, 0xff])
            } else {
                (palette.field, palette.text)
            }
        }
    };

    fill_rounded_rect(buffer, width, height, button.rect, 6, fill, 245);
    draw_system_surface_primitives(buffer, width, height, button.rect);
    let label = if button.state == ComponentState::Loading {
        "Bekleyin…"
    } else {
        button.label
    };
    draw_fitted_bitmap_text(
        buffer,
        (width, height),
        button.content_rect(),
        label,
        text,
        FittedTextOptions::new(TextRole::Control, scale, true),
    );
    if button.state == ComponentState::KeyboardFocus {
        draw_rect_outline(
            buffer,
            width,
            height,
            button.focus_rect(),
            palette.accent,
            220,
        );
        3
    } else {
        2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentAcceptanceProbe {
    pub viewport: Viewport,
    pub theme: AquaTheme,
    pub scale: OutputScale,
    pub state_count: usize,
    pub stable_geometry: bool,
    pub input_semantics: bool,
    pub accessibility_semantics: bool,
    pub checksum: u64,
}

impl ComponentAcceptanceProbe {
    pub const fn is_ready(self) -> bool {
        self.state_count == ComponentState::STANDARD_BUTTON_STATES.len()
            && self.stable_geometry
            && self.input_semantics
            && self.accessibility_semantics
    }
}

pub fn render_component_acceptance_rgba(
    viewport: Viewport,
    theme: AquaTheme,
    scale: OutputScale,
) -> Option<(Vec<u8>, ComponentAcceptanceProbe)> {
    if viewport.width < 800 || viewport.height < 600 {
        return None;
    }
    let palette = window_chrome_palette(theme);
    let mut buffer = vec![
        0_u8;
        viewport
            .width
            .saturating_mul(viewport.height)
            .saturating_mul(4) as usize
    ];
    buffer
        .chunks_exact_mut(4)
        .for_each(|pixel| pixel.copy_from_slice(&palette.surface));

    let button_width = scale.apply(168.0).round() as u32;
    let button_height = scale.apply(40.0).round() as u32;
    let horizontal_gap = scale.apply(28.0).round() as u32;
    let vertical_gap = scale.apply(14.0).round() as u32;
    let start_x = 56;
    let start_y = 48;
    let mut stable_geometry = true;
    let mut input_semantics = true;
    let mut accessibility_semantics = true;

    for (index, state) in ComponentState::STANDARD_BUTTON_STATES
        .into_iter()
        .enumerate()
    {
        let column = index as u32 % 2;
        let row = index as u32 / 2;
        let rect = Rect {
            x: start_x + column * (button_width + horizontal_gap),
            y: start_y + row * (button_height + vertical_gap),
            width: button_width,
            height: button_height,
        };
        let variant = match state {
            ComponentState::Error => StandardButtonVariant::Destructive,
            ComponentState::Selected | ComponentState::Success => StandardButtonVariant::Primary,
            _ => StandardButtonVariant::Secondary,
        };
        let button = StandardButton::new(rect, state.id(), variant).with_state(state);
        draw_standard_button(
            &mut buffer,
            viewport.width,
            viewport.height,
            button,
            theme,
            scale,
        );
        stable_geometry &=
            button.rect == rect && button.content_rect().width == rect.width.saturating_sub(16);
        input_semantics &= button.pointer_hit(rect.x + 1, rect.y + 1) == button.can_activate();
        input_semantics &=
            button.keyboard_activates(ButtonActivationKey::Enter) == button.can_activate();
        input_semantics &= !button.keyboard_activates(ButtonActivationKey::Other);
        let semantics = button.accessibility();
        accessibility_semantics &= semantics.role == "button" && !semantics.name.is_empty();
        accessibility_semantics &= semantics.disabled == (state == ComponentState::Disabled);
        accessibility_semantics &= semantics.busy == (state == ComponentState::Loading);
        accessibility_semantics &= semantics.selected == (state == ComponentState::Selected);
    }

    let probe = ComponentAcceptanceProbe {
        viewport,
        theme,
        scale,
        state_count: ComponentState::STANDARD_BUTTON_STATES.len(),
        stable_geometry,
        input_semantics,
        accessibility_semantics,
        checksum: checksum_bytes(&buffer),
    };
    Some((buffer, probe))
}

pub fn component_acceptance_report() -> String {
    let cases = [
        (Viewport::new(800, 600), OutputScale::One),
        (Viewport::new(1280, 800), OutputScale::One),
        (Viewport::new(1536, 1024), OutputScale::FiveQuarters),
    ];
    let shared_count = SharedComponentKind::ALL
        .into_iter()
        .filter(|component| component.is_shared_primitive())
        .count();
    let mut lines = vec![format!(
        "fixture_revision={COMPONENT_FIXTURE_REVISION} catalog={} shared={shared_count}",
        SharedComponentKind::ALL.len()
    )];
    for (viewport, scale) in cases {
        for theme in AquaTheme::ALL {
            let (_, probe) = render_component_acceptance_rgba(viewport, theme, scale)
                .expect("supported component acceptance viewport");
            lines.push(format!(
                "component=standard-button viewport={}x{} scale={}/{} theme={} states={} stable_geometry={} input_semantics={} accessibility_semantics={} ready={} checksum={:016x}",
                viewport.width,
                viewport.height,
                scale.numerator(),
                scale.denominator(),
                theme.id(),
                probe.state_count,
                probe.stable_geometry,
                probe.input_semantics,
                probe.accessibility_semantics,
                probe.is_ready(),
                probe.checksum,
            ));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_matches_the_documented_inventory() {
        assert_eq!(SharedComponentKind::ALL.len(), 22);
        assert_eq!(
            SharedComponentKind::ALL
                .into_iter()
                .filter(|component| component.is_shared_primitive())
                .collect::<Vec<_>>(),
            vec![SharedComponentKind::StandardButton]
        );
    }

    #[test]
    fn disabled_and_loading_buttons_reject_activation() {
        let rect = Rect {
            x: 10,
            y: 20,
            width: 120,
            height: 40,
        };
        for state in [ComponentState::Disabled, ComponentState::Loading] {
            let button = StandardButton::new(rect, "Action", StandardButtonVariant::Primary)
                .with_state(state);
            assert!(!button.pointer_hit(11, 21));
            assert!(!button.keyboard_activates(ButtonActivationKey::Enter));
            assert!(!button.keyboard_activates(ButtonActivationKey::Space));
        }
    }

    #[test]
    fn standard_button_matrix_is_ready_for_every_theme_and_viewport() {
        for (viewport, scale) in [
            (Viewport::new(800, 600), OutputScale::One),
            (Viewport::new(1280, 800), OutputScale::One),
            (Viewport::new(1536, 1024), OutputScale::FiveQuarters),
        ] {
            for theme in AquaTheme::ALL {
                let (_, probe) = render_component_acceptance_rgba(viewport, theme, scale).unwrap();
                assert!(probe.is_ready());
                assert_ne!(probe.checksum, 0);
            }
        }
    }

    #[test]
    fn component_report_is_complete_and_deterministic() {
        let first = component_acceptance_report();
        assert_eq!(first, component_acceptance_report());
        assert_eq!(first.matches("component=standard-button").count(), 12);
        assert_eq!(first.matches("ready=true").count(), 12);
    }
}
