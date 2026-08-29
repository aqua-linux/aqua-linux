use super::{
    checksum_bytes, draw_fitted_bitmap_text, draw_rect_outline, draw_system_surface_primitives,
    fill_rect, fill_rounded_rect, window_chrome_palette, FittedTextOptions,
};
pub use aqua_components::*;
use aqua_scene::{Rect, Viewport};
use aqua_shell::AquaTheme;
use aqua_text::{OutputScale, TextRole};

pub const COMPONENT_FIXTURE_REVISION: &str = "aqua-component-fixtures-2";

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

pub(crate) fn draw_sidebar_navigation(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    navigation: SidebarNavigation<'_>,
    theme: AquaTheme,
) -> usize {
    let palette = window_chrome_palette(theme);
    fill_rect(buffer, width, height, navigation.rect, palette.sidebar, 255);
    fill_rect(
        buffer,
        width,
        height,
        navigation.separator_rect(),
        palette.border,
        255,
    );
    2
}

pub(crate) fn draw_list_row(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    row: ListRow<'_>,
    theme: AquaTheme,
    scale: OutputScale,
) -> usize {
    let palette = window_chrome_palette(theme);
    let (fill, text, opacity) = match row.state {
        ComponentState::Idle => (palette.sidebar, palette.text, 0),
        ComponentState::Hover => (palette.hover, palette.text, 255),
        ComponentState::KeyboardFocus => (palette.sidebar, palette.text, 0),
        ComponentState::Pressed => (palette.accent_soft, palette.text, 220),
        ComponentState::Selected => (palette.accent_soft, palette.accent, 255),
        ComponentState::Disabled => (palette.row_alternate, palette.secondary_text, 190),
        ComponentState::Loading => (palette.row_alternate, palette.secondary_text, 220),
        ComponentState::Error => ([0xc9, 0x3c, 0x47, 0xff], [0xff, 0xff, 0xff, 0xff], 245),
        ComponentState::Success => ([0x2c, 0x8a, 0x59, 0xff], [0xff, 0xff, 0xff, 0xff], 245),
        ComponentState::Attention => ([0xd1, 0x8b, 0x24, 0xff], [0x1b, 0x20, 0x27, 0xff], 245),
    };
    let mut primitives = 1;
    if opacity > 0 {
        fill_rounded_rect(buffer, width, height, row.rect, 6, fill, opacity);
        primitives += 1;
    }
    let label = if row.state == ComponentState::Loading {
        "Yükleniyor…"
    } else {
        row.label
    };
    draw_fitted_bitmap_text(
        buffer,
        (width, height),
        row.slots().label,
        label,
        text,
        FittedTextOptions::new(TextRole::Control, scale, true),
    );
    if row.state == ComponentState::KeyboardFocus {
        draw_rect_outline(buffer, width, height, row.focus_rect(), palette.accent, 220);
        primitives += 1;
    }
    primitives
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentAcceptanceProbe {
    pub viewport: Viewport,
    pub theme: AquaTheme,
    pub scale: OutputScale,
    pub button_state_count: usize,
    pub list_row_state_count: usize,
    pub sidebar_row_count: usize,
    pub stable_geometry: bool,
    pub input_semantics: bool,
    pub accessibility_semantics: bool,
    pub checksum: u64,
}

impl ComponentAcceptanceProbe {
    pub const fn is_ready(self) -> bool {
        self.button_state_count == ComponentState::STANDARD_BUTTON_STATES.len()
            && self.list_row_state_count == ComponentState::LIST_ROW_STATES.len()
            && self.sidebar_row_count == ComponentState::LIST_ROW_STATES.len()
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
        input_semantics &= button.keyboard_activates(ActivationKey::Enter) == button.can_activate();
        input_semantics &= !button.keyboard_activates(ActivationKey::Other);
        let semantics = button.accessibility();
        accessibility_semantics &= semantics.role == "button" && !semantics.name.is_empty();
        accessibility_semantics &= semantics.disabled == (state == ComponentState::Disabled);
        accessibility_semantics &= semantics.busy == (state == ComponentState::Loading);
        accessibility_semantics &= semantics.selected == (state == ComponentState::Selected);
    }

    let sidebar_x = viewport.width / 2 + 16;
    let sidebar = SidebarNavigation::new(
        Rect {
            x: sidebar_x,
            y: 24,
            width: viewport.width.saturating_sub(sidebar_x + 24),
            height: viewport.height.saturating_sub(48),
        },
        "Fixture navigation",
        Rect {
            x: sidebar_x + 12,
            y: 42,
            width: viewport.width.saturating_sub(sidebar_x + 48),
            height: scale.apply(38.0).round() as u32,
        },
        scale.apply(46.0).round() as u32,
    );
    draw_sidebar_navigation(&mut buffer, viewport.width, viewport.height, sidebar, theme);
    let sidebar_semantics = sidebar.accessibility();
    accessibility_semantics &=
        sidebar_semantics.role == "navigation" && !sidebar_semantics.name.is_empty();
    for (index, state) in ComponentState::LIST_ROW_STATES.into_iter().enumerate() {
        let rect = sidebar.row_rect(index);
        let row = ListRow::new(rect, state.id(), ListRowRole::Navigation).with_state(state);
        draw_list_row(
            &mut buffer,
            viewport.width,
            viewport.height,
            row,
            theme,
            scale,
        );
        let slots = row.slots();
        stable_geometry &= slots.leading.right() <= slots.label.x
            && slots.label.right() <= slots.trailing.x
            && slots.trailing.right() <= rect.right();
        input_semantics &= row.pointer_hit(rect.x + 1, rect.y + 1) == row.can_activate();
        input_semantics &= row.keyboard_activates(ActivationKey::Space) == row.can_activate();
        input_semantics &= sidebar.hit_test(rect.x + 1, rect.y + 1, 10) == Some(index);
        let semantics = row.accessibility();
        accessibility_semantics &= semantics.role == "navigation-item"
            && semantics.disabled == (state == ComponentState::Disabled)
            && semantics.busy == (state == ComponentState::Loading)
            && semantics.selected == (state == ComponentState::Selected);
    }

    let probe = ComponentAcceptanceProbe {
        viewport,
        theme,
        scale,
        button_state_count: ComponentState::STANDARD_BUTTON_STATES.len(),
        list_row_state_count: ComponentState::LIST_ROW_STATES.len(),
        sidebar_row_count: ComponentState::LIST_ROW_STATES.len(),
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
                "components=standard-button,list-row,sidebar-navigation viewport={}x{} scale={}/{} theme={} button_states={} list_row_states={} sidebar_rows={} stable_geometry={} input_semantics={} accessibility_semantics={} ready={} checksum={:016x}",
                viewport.width,
                viewport.height,
                scale.numerator(),
                scale.denominator(),
                theme.id(),
                probe.button_state_count,
                probe.list_row_state_count,
                probe.sidebar_row_count,
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
            vec![
                SharedComponentKind::SidebarNavigation,
                SharedComponentKind::StandardButton,
                SharedComponentKind::ListRow,
            ]
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
            assert!(!button.keyboard_activates(ActivationKey::Enter));
            assert!(!button.keyboard_activates(ActivationKey::Space));
        }
    }

    #[test]
    fn shared_component_matrix_is_ready_for_every_theme_and_viewport() {
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
        assert_eq!(
            first
                .matches("components=standard-button,list-row,sidebar-navigation")
                .count(),
            12
        );
        assert_eq!(first.matches("ready=true").count(), 12);
    }
}
