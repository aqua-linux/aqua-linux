use super::{
    checksum_bytes, draw_fitted_bitmap_text, draw_rect_outline, draw_system_surface_primitives,
    draw_window_frame, fill_rect, fill_rounded_rect, window_chrome_palette, FittedTextOptions,
};
pub use aqua_components::*;
use aqua_scene::{Rect, Viewport};
use aqua_shell::AquaTheme;
use aqua_text::{OutputScale, TextRole};

pub const COMPONENT_FIXTURE_REVISION: &str = "aqua-component-fixtures-11";

fn draw_component_glyph(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rect: Rect,
    glyph: IconButtonGlyph,
    color: [u8; 4],
) -> usize {
    let center_y = rect.y + rect.height / 2;
    match glyph {
        IconButtonGlyph::Back | IconButtonGlyph::Forward => {
            fill_rect(
                buffer,
                width,
                height,
                Rect {
                    x: rect.x + 4,
                    y: center_y.saturating_sub(1),
                    width: rect.width.saturating_sub(8),
                    height: 2,
                },
                color,
                255,
            );
            let arrow_x = if glyph == IconButtonGlyph::Back {
                rect.x + 4
            } else {
                rect.right().saturating_sub(6)
            };
            for offset in 0..5 {
                let x = if glyph == IconButtonGlyph::Back {
                    arrow_x + 4 - offset
                } else {
                    arrow_x.saturating_sub(4 - offset)
                };
                fill_rect(
                    buffer,
                    width,
                    height,
                    Rect {
                        x,
                        y: center_y.saturating_sub(5).saturating_add(offset),
                        width: 2,
                        height: 2,
                    },
                    color,
                    255,
                );
                fill_rect(
                    buffer,
                    width,
                    height,
                    Rect {
                        x,
                        y: center_y.saturating_add(4).saturating_sub(offset),
                        width: 2,
                        height: 2,
                    },
                    color,
                    255,
                );
            }
            11
        }
        IconButtonGlyph::Search => {
            let lens = Rect {
                x: rect.x + 2,
                y: rect.y + 2,
                width: rect.width.saturating_sub(8),
                height: rect.height.saturating_sub(8),
            };
            draw_rect_outline(buffer, width, height, lens, color, 255);
            fill_rect(
                buffer,
                width,
                height,
                Rect {
                    x: lens.right().saturating_sub(1),
                    y: lens.bottom().saturating_sub(1),
                    width: 6,
                    height: 2,
                },
                color,
                255,
            );
            2
        }
        IconButtonGlyph::Close => {
            for offset in 0..rect.width.min(rect.height) {
                fill_rect(
                    buffer,
                    width,
                    height,
                    Rect {
                        x: rect.x + offset,
                        y: rect.y + offset,
                        width: 1,
                        height: 1,
                    },
                    color,
                    255,
                );
                fill_rect(
                    buffer,
                    width,
                    height,
                    Rect {
                        x: rect.right().saturating_sub(offset + 1),
                        y: rect.y + offset,
                        width: 1,
                        height: 1,
                    },
                    color,
                    255,
                );
            }
            2
        }
    }
}

pub(crate) fn draw_icon_button(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    button: IconButton<'_>,
    theme: AquaTheme,
) -> usize {
    let palette = window_chrome_palette(theme);
    let (fill, glyph_color, opacity) = match button.state {
        ComponentState::Idle | ComponentState::KeyboardFocus => (palette.field, palette.text, 235),
        ComponentState::Hover => (palette.hover, palette.text, 255),
        ComponentState::Pressed | ComponentState::Selected => {
            (palette.accent_soft, palette.accent, 255)
        }
        ComponentState::Disabled => (palette.row_alternate, palette.secondary_text, 180),
        ComponentState::Loading => (palette.row_alternate, palette.secondary_text, 210),
        ComponentState::Error => ([0xc9, 0x3c, 0x47, 0xff], [0xff, 0xff, 0xff, 0xff], 245),
        ComponentState::Success => ([0x2c, 0x8a, 0x59, 0xff], [0xff, 0xff, 0xff, 0xff], 245),
        ComponentState::Attention => ([0xd1, 0x8b, 0x24, 0xff], [0x1b, 0x20, 0x27, 0xff], 245),
    };
    fill_rounded_rect(buffer, width, height, button.rect, 6, fill, opacity);
    let mut primitives = 1 + draw_component_glyph(
        buffer,
        width,
        height,
        button.icon_rect(),
        button.glyph,
        glyph_color,
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
        primitives += 1;
    }
    primitives
}

pub(crate) fn draw_search_field(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    field: SearchField<'_>,
    theme: AquaTheme,
    scale: OutputScale,
) -> usize {
    let palette = window_chrome_palette(theme);
    let (fill, border, text) = match field.state {
        ComponentState::Disabled => (
            palette.row_alternate,
            palette.border,
            palette.secondary_text,
        ),
        ComponentState::Loading => (
            palette.row_alternate,
            palette.border,
            palette.secondary_text,
        ),
        ComponentState::Error => (palette.field, [0xc9, 0x3c, 0x47, 0xff], palette.text),
        ComponentState::Success => (palette.field, [0x2c, 0x8a, 0x59, 0xff], palette.text),
        ComponentState::Attention => (palette.field, [0xd1, 0x8b, 0x24, 0xff], palette.text),
        ComponentState::Hover => (palette.hover, palette.border, palette.text),
        ComponentState::Idle
        | ComponentState::KeyboardFocus
        | ComponentState::Pressed
        | ComponentState::Selected => (palette.field, palette.border, palette.text),
    };
    fill_rounded_rect(buffer, width, height, field.rect, 6, fill, 245);
    draw_rect_outline(buffer, width, height, field.rect, border, 255);
    let slots = field.slots();
    let search_icon = IconButton::new(slots.leading, "Search", IconButtonGlyph::Search);
    draw_component_glyph(
        buffer,
        width,
        height,
        search_icon.icon_rect(),
        IconButtonGlyph::Search,
        palette.secondary_text,
    );
    let label = if field.state == ComponentState::Loading {
        "SEARCHING..."
    } else {
        field.display_text()
    };
    draw_fitted_bitmap_text(
        buffer,
        (width, height),
        slots.text,
        label,
        if field.value.is_empty() {
            palette.secondary_text
        } else {
            text
        },
        FittedTextOptions::new(TextRole::Control, scale, false),
    );
    if field.state == ComponentState::KeyboardFocus {
        draw_rect_outline(
            buffer,
            width,
            height,
            field.focus_rect(),
            palette.accent,
            220,
        );
        5
    } else {
        4
    }
}

pub(crate) fn draw_switch_control(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    control: SwitchControl<'_>,
    theme: AquaTheme,
) -> usize {
    let palette = window_chrome_palette(theme);
    let track = match control.state {
        ComponentState::Disabled | ComponentState::Loading => palette.row_alternate,
        ComponentState::Error => [0xc9, 0x3c, 0x47, 0xff],
        ComponentState::Success => [0x2c, 0x8a, 0x59, 0xff],
        ComponentState::Attention => [0xd1, 0x8b, 0x24, 0xff],
        ComponentState::Hover | ComponentState::Pressed if control.checked => palette.accent_soft,
        _ if control.checked => palette.accent,
        ComponentState::Hover | ComponentState::Pressed => palette.hover,
        _ => palette.border,
    };
    fill_rounded_rect(
        buffer,
        width,
        height,
        control.rect,
        control.rect.height / 2,
        track,
        235,
    );
    draw_rect_outline(buffer, width, height, control.rect, palette.border, 230);
    fill_rounded_rect(
        buffer,
        width,
        height,
        control.thumb_rect(),
        control.thumb_rect().height / 2,
        palette.field,
        255,
    );
    if control.state == ComponentState::KeyboardFocus {
        draw_rect_outline(
            buffer,
            width,
            height,
            control.focus_rect(),
            palette.accent,
            220,
        );
        4
    } else {
        3
    }
}

pub(crate) fn draw_segmented_control(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    control: SegmentedControl<'_>,
    labels: &[&str],
    theme: AquaTheme,
    scale: OutputScale,
) -> usize {
    let palette = window_chrome_palette(theme);
    let mut primitives = 0;
    for (index, label) in labels.iter().take(control.segment_count).enumerate() {
        let rect = control.segment_rect(index);
        let selected = index == control.selected_index;
        let fill = match control.state {
            ComponentState::Disabled | ComponentState::Loading => palette.row_alternate,
            ComponentState::Error => [0xc9, 0x3c, 0x47, 0xff],
            ComponentState::Success => [0x2c, 0x8a, 0x59, 0xff],
            ComponentState::Attention => [0xd1, 0x8b, 0x24, 0xff],
            ComponentState::Hover | ComponentState::Pressed if selected => palette.accent_soft,
            _ if selected => palette.accent_soft,
            ComponentState::Hover | ComponentState::Pressed => palette.hover,
            _ => palette.field,
        };
        fill_rounded_rect(buffer, width, height, rect, 5, fill, 245);
        draw_rect_outline(
            buffer,
            width,
            height,
            rect,
            if selected {
                palette.accent
            } else {
                palette.border
            },
            255,
        );
        draw_fitted_bitmap_text(
            buffer,
            (width, height),
            Rect {
                x: rect.x.saturating_add(5),
                y: rect.y,
                width: rect.width.saturating_sub(10),
                height: rect.height,
            },
            label,
            palette.text,
            FittedTextOptions::new(TextRole::Control, scale, true),
        );
        primitives += 3;
    }
    if control.state == ComponentState::KeyboardFocus {
        draw_rect_outline(
            buffer,
            width,
            height,
            control.focus_rect(),
            palette.accent,
            220,
        );
        primitives += 1;
    }
    primitives
}

pub(crate) fn draw_toolbar(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    toolbar: Toolbar<'_>,
    theme: AquaTheme,
) -> usize {
    let palette = window_chrome_palette(theme);
    fill_rect(buffer, width, height, toolbar.rect, palette.toolbar, 255);
    fill_rect(
        buffer,
        width,
        height,
        toolbar.separator_rect(),
        palette.border,
        220,
    );
    2
}

pub(crate) fn draw_section_group(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    section: SectionGroup<'_>,
    theme: AquaTheme,
) -> usize {
    if !section.is_valid() {
        return 0;
    }
    let palette = window_chrome_palette(theme);
    fill_rounded_rect(buffer, width, height, section.rect, 8, palette.field, 255);
    draw_rect_outline(buffer, width, height, section.rect, palette.border, 255);
    let mut primitives = 2;
    if section.header_height > 0 {
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: section.rect.x,
                y: section.header_rect().bottom().saturating_sub(1),
                width: section.rect.width,
                height: 1,
            },
            palette.border,
            255,
        );
        primitives += 1;
    }
    if section.focused {
        draw_rect_outline(
            buffer,
            width,
            height,
            Rect {
                x: section.rect.x.saturating_sub(2),
                y: section.rect.y.saturating_sub(2),
                width: section.rect.width.saturating_add(4),
                height: section.rect.height.saturating_add(4),
            },
            palette.accent,
            220,
        );
        primitives += 1;
    }
    primitives
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MetadataRowStyle {
    pub label_color: [u8; 4],
    pub value_color: [u8; 4],
    pub role: TextRole,
    pub scale: OutputScale,
}

pub(crate) fn draw_metadata_row(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    row: MetadataRow<'_>,
    style: MetadataRowStyle,
) -> usize {
    if !row.is_valid() {
        return 0;
    }
    let slots = row.slots();
    draw_fitted_bitmap_text(
        buffer,
        (width, height),
        slots.label,
        row.label,
        style.label_color,
        FittedTextOptions::new(style.role, style.scale, false),
    );
    draw_fitted_bitmap_text(
        buffer,
        (width, height),
        slots.value,
        row.value,
        style.value_color,
        FittedTextOptions::new(style.role, style.scale, false),
    );
    2
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

pub(crate) fn draw_grid_cell(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    cell: GridCell<'_>,
    theme: AquaTheme,
) -> usize {
    if !cell.is_valid() {
        return 0;
    }
    let palette = window_chrome_palette(theme);
    let (fill, opacity) = match cell.state {
        ComponentState::Idle if !cell.idle_surface => (palette.field, 0),
        ComponentState::Idle => (palette.field, 225),
        ComponentState::Hover => (palette.hover, 245),
        ComponentState::KeyboardFocus => (palette.field, 225),
        ComponentState::Pressed => (palette.accent_soft, 235),
        ComponentState::Selected => (palette.accent_soft, 255),
        ComponentState::Disabled => (palette.row_alternate, 180),
        ComponentState::Loading => (palette.row_alternate, 210),
        ComponentState::Error => ([0xc9, 0x3c, 0x47, 0xff], 245),
        ComponentState::Success => ([0x2c, 0x8a, 0x59, 0xff], 245),
        ComponentState::Attention => ([0xd1, 0x8b, 0x24, 0xff], 245),
    };
    let mut primitives = 0;
    if opacity > 0 {
        fill_rounded_rect(buffer, width, height, cell.rect, 8, fill, opacity);
        primitives += 1;
    }
    if cell.state == ComponentState::KeyboardFocus {
        draw_rect_outline(
            buffer,
            width,
            height,
            cell.focus_rect(),
            palette.accent,
            220,
        );
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
    pub icon_button_state_count: usize,
    pub list_row_state_count: usize,
    pub grid_cell_state_count: usize,
    pub search_field_state_count: usize,
    pub segmented_control_state_count: usize,
    pub switch_state_count: usize,
    pub sidebar_row_count: usize,
    pub toolbar_ready: bool,
    pub window_frame_ready: bool,
    pub menu_ready: bool,
    pub section_group_ready: bool,
    pub metadata_row_ready: bool,
    pub top_system_bar_ready: bool,
    pub stable_geometry: bool,
    pub input_semantics: bool,
    pub accessibility_semantics: bool,
    pub checksum: u64,
}

impl ComponentAcceptanceProbe {
    pub const fn is_ready(self) -> bool {
        self.button_state_count == ComponentState::STANDARD_BUTTON_STATES.len()
            && self.icon_button_state_count == ComponentState::ICON_BUTTON_STATES.len()
            && self.list_row_state_count == ComponentState::LIST_ROW_STATES.len()
            && self.grid_cell_state_count == ComponentState::GRID_CELL_STATES.len()
            && self.search_field_state_count == ComponentState::SEARCH_FIELD_STATES.len()
            && self.segmented_control_state_count == ComponentState::SEGMENTED_CONTROL_STATES.len()
            && self.switch_state_count == ComponentState::SWITCH_STATES.len()
            && self.sidebar_row_count == ComponentState::LIST_ROW_STATES.len()
            && self.toolbar_ready
            && self.window_frame_ready
            && self.menu_ready
            && self.section_group_ready
            && self.metadata_row_ready
            && self.top_system_bar_ready
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

    let fixture_frame = WindowFrame::new(
        Rect {
            x: 8,
            y: 8,
            width: viewport.width.saturating_sub(16),
            height: viewport.height.saturating_sub(16),
        },
        "Fixture window",
        40,
    );
    draw_window_frame(
        &mut buffer,
        viewport.width,
        viewport.height,
        fixture_frame,
        palette,
    );
    let frame_semantics = fixture_frame.accessibility();
    let close = fixture_frame.control_rect(WindowControl::Close);
    let window_frame_ready = fixture_frame.is_valid()
        && fixture_frame.titlebar_rect().fits_in(viewport)
        && fixture_frame.title_rect().fits_in(viewport)
        && fixture_frame.resize_grip_rect().fits_in(viewport)
        && fixture_frame.control_at(close.x, close.y) == Some(WindowControl::Close)
        && !fixture_frame.move_hit(close.x, close.y)
        && fixture_frame.move_hit(fixture_frame.title_rect().x, fixture_frame.title_rect().y)
        && fixture_frame.resize_hit(
            fixture_frame.rect.right().saturating_sub(1),
            fixture_frame.rect.bottom().saturating_sub(1),
        )
        && frame_semantics.role == "window"
        && !frame_semantics.name.is_empty()
        && frame_semantics.focused;

    let fixture_top_bar = TopSystemBar::new(
        Rect {
            x: 8,
            y: 8,
            width: viewport.width.saturating_sub(16),
            height: 40,
        },
        "Fixture system bar",
    );
    fill_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        fixture_top_bar.rect,
        palette.toolbar,
        255,
    );
    fill_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        fixture_top_bar.separator_rect(),
        palette.border,
        255,
    );
    let audio = fixture_top_bar.status_rect(TopSystemStatus::Audio);
    let session = fixture_top_bar.session_rect();
    let bar_semantics = fixture_top_bar.accessibility();
    let audio_semantics = fixture_top_bar.status_accessibility(TopSystemStatus::Audio, true, None);
    let top_system_bar_ready = fixture_top_bar.is_valid()
        && fixture_top_bar.brand_rect().fits_in(viewport)
        && fixture_top_bar.clock_rect().fits_in(viewport)
        && fixture_top_bar.status_group_rect().fits_in(viewport)
        && session.fits_in(viewport)
        && fixture_top_bar.status_at(audio.x, audio.y) == Some(TopSystemStatus::Audio)
        && fixture_top_bar.session_hit(session.x, session.y)
        && !fixture_top_bar.session_hit(session.x.saturating_sub(1), session.y)
        && bar_semantics.role == "banner"
        && !bar_semantics.name.is_empty()
        && fixture_top_bar.session_accessibility().role == "button"
        && audio_semantics.role == "status"
        && audio_semantics.available;

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

    let search_width = scale.apply(150.0).round() as u32;
    let search_height = scale.apply(36.0).round() as u32;
    let search_gap = scale.apply(14.0).round() as u32;
    let search_stride = scale.apply(50.0).round() as u32;
    for (index, state) in ComponentState::SEARCH_FIELD_STATES.into_iter().enumerate() {
        let column = index as u32 % 2;
        let row_index = index as u32 / 2;
        let rect = Rect {
            x: 56 + column * (search_width + search_gap),
            y: 330 + row_index * search_stride,
            width: search_width,
            height: search_height,
        };
        let value = if index % 2 == 0 { "" } else { state.id() };
        let field = SearchField::new(rect, "Search fixture", value, "Search").with_state(state);
        draw_search_field(
            &mut buffer,
            viewport.width,
            viewport.height,
            field,
            theme,
            scale,
        );
        let slots = field.slots();
        stable_geometry &= slots.leading.right() <= slots.text.x
            && slots.text.right() <= slots.trailing.x
            && slots.trailing.right() == rect.right();
        input_semantics &= field.pointer_focuses(rect.x + 1, rect.y + 1) == field.accepts_input();
        let semantics = field.accessibility();
        accessibility_semantics &= semantics.role == "searchbox"
            && !semantics.name.is_empty()
            && semantics.value == value
            && semantics.disabled == (state == ComponentState::Disabled)
            && semantics.busy == (state == ComponentState::Loading)
            && semantics.invalid == (state == ComponentState::Error);
    }

    let fixture_toolbar = Toolbar::new(
        Rect {
            x: 48,
            y: 526,
            width: 340,
            height: 72,
        },
        "Fixture actions",
    );
    draw_toolbar(
        &mut buffer,
        viewport.width,
        viewport.height,
        fixture_toolbar,
        theme,
    );
    let toolbar_semantics = fixture_toolbar.accessibility();
    let toolbar_ready = fixture_toolbar.is_valid()
        && fixture_toolbar.content_rect().fits_in(viewport)
        && fixture_toolbar.separator_rect().bottom() == fixture_toolbar.rect.bottom()
        && fixture_toolbar.contains(fixture_toolbar.rect.x, fixture_toolbar.rect.y)
        && toolbar_semantics.role == "toolbar"
        && !toolbar_semantics.name.is_empty();

    let icon_size = scale.apply(28.0).round() as u32;
    let icon_stride_x = scale.apply(58.0).round() as u32;
    let icon_stride_y = scale.apply(33.0).round() as u32;
    for (index, state) in ComponentState::ICON_BUTTON_STATES.into_iter().enumerate() {
        let rect = Rect {
            x: 56 + (index as u32 % 5) * icon_stride_x,
            y: 535 + (index as u32 / 5) * icon_stride_y,
            width: icon_size,
            height: icon_size,
        };
        let glyph = match index % 4 {
            0 => IconButtonGlyph::Back,
            1 => IconButtonGlyph::Forward,
            2 => IconButtonGlyph::Search,
            _ => IconButtonGlyph::Close,
        };
        let button = IconButton::new(rect, state.id(), glyph).with_state(state);
        draw_icon_button(&mut buffer, viewport.width, viewport.height, button, theme);
        stable_geometry &= button.icon_rect().fits_in(viewport)
            && button.icon_rect().width <= button.rect.width
            && button.icon_rect().height <= button.rect.height;
        input_semantics &= button.pointer_hit(rect.x + 1, rect.y + 1) == button.can_activate();
        input_semantics &= button.keyboard_activates(ActivationKey::Enter) == button.can_activate();
        let semantics = button.accessibility();
        accessibility_semantics &= semantics.role == "button"
            && !semantics.name.is_empty()
            && semantics.disabled == (state == ComponentState::Disabled)
            && semantics.busy == (state == ComponentState::Loading)
            && semantics.selected == (state == ComponentState::Selected);
    }

    for (index, state) in ComponentState::GRID_CELL_STATES.into_iter().enumerate() {
        let rect = Rect {
            x: viewport.width / 2 + 8 + (index as u32 % 5) * 70,
            y: 332 + (index as u32 / 5) * 64,
            width: 62,
            height: 56,
        };
        let layout = if index < 5 {
            GridCellLayout::IconLeading
        } else {
            GridCellLayout::IconAbove
        };
        let cell = GridCell::new(rect, state.id(), layout)
            .with_spacing(16, 5, 3, 12)
            .with_state(state);
        draw_grid_cell(&mut buffer, viewport.width, viewport.height, cell, theme);
        let slots = cell.slots();
        fill_rounded_rect(
            &mut buffer,
            viewport.width,
            viewport.height,
            slots.icon,
            4,
            palette.accent,
            230,
        );
        draw_fitted_bitmap_text(
            &mut buffer,
            (viewport.width, viewport.height),
            slots.primary,
            cell.label,
            palette.text,
            FittedTextOptions::new(TextRole::Control, scale, false),
        );
        stable_geometry &= cell.is_valid()
            && slots.icon.fits_in(viewport)
            && slots.primary.fits_in(viewport)
            && slots.secondary.fits_in(viewport);
        input_semantics &= cell.pointer_hit(rect.x, rect.y) == cell.can_activate();
        input_semantics &= cell.keyboard_activates(ActivationKey::Enter) == cell.can_activate();
        let semantics = cell.accessibility();
        accessibility_semantics &= semantics.role == "gridcell"
            && !semantics.name.is_empty()
            && semantics.disabled == (state == ComponentState::Disabled)
            && semantics.busy == (state == ComponentState::Loading)
            && semantics.selected == (state == ComponentState::Selected);
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

        if let Some(control_state) = ComponentState::SWITCH_STATES.get(index).copied() {
            let switch = SwitchControl::new(
                Rect {
                    x: rect.right().saturating_sub(58),
                    y: rect.y.saturating_add(5),
                    width: 52,
                    height: 28,
                },
                control_state.id(),
                index % 2 == 0,
            )
            .with_state(control_state);
            draw_switch_control(&mut buffer, viewport.width, viewport.height, switch, theme);
            stable_geometry &= switch.thumb_rect().fits_in(viewport)
                && switch.thumb_rect().x >= switch.rect.x
                && switch.thumb_rect().right() <= switch.rect.right();
            input_semantics &=
                switch.pointer_toggles(switch.rect.x, switch.rect.y) == switch.can_toggle();
            input_semantics &= switch.keyboard_toggles(ActivationKey::Space) == switch.can_toggle();
            let semantics = switch.accessibility();
            accessibility_semantics &= semantics.role == "switch"
                && !semantics.name.is_empty()
                && semantics.checked == (index % 2 == 0)
                && semantics.disabled == (control_state == ComponentState::Disabled)
                && semantics.busy == (control_state == ComponentState::Loading);
        }

        if let Some(control_state) = ComponentState::SEGMENTED_CONTROL_STATES.get(index).copied() {
            let segmented = SegmentedControl::new(
                Rect {
                    x: rect.x.saturating_add(130),
                    y: rect.y.saturating_add(5),
                    width: 100,
                    height: 28,
                },
                control_state.id(),
                2,
                index % 2,
            )
            .with_gap(2)
            .with_state(control_state);
            draw_segmented_control(
                &mut buffer,
                viewport.width,
                viewport.height,
                segmented,
                &["A", "B"],
                theme,
                scale,
            );
            stable_geometry &= segmented.segment_rect(0).fits_in(viewport)
                && segmented.segment_rect(1).right() == segmented.rect.right();
            input_semantics &= segmented.hit_test(segmented.rect.x, segmented.rect.y)
                == if segmented.state.can_activate() {
                    Some(0)
                } else {
                    None
                };
            input_semantics &= segmented.keyboard_target(SegmentNavigationKey::Next)
                == if segmented.state.can_activate() {
                    Some((segmented.selected_index + 1) % segmented.segment_count)
                } else {
                    None
                };
            let semantics = segmented.accessibility();
            accessibility_semantics &= semantics.role == "radiogroup"
                && !semantics.name.is_empty()
                && semantics.selected_index == index % 2
                && semantics.segment_count == 2
                && semantics.disabled == (control_state == ComponentState::Disabled)
                && semantics.busy == (control_state == ComponentState::Loading);
        }
    }

    let fixture_section = SectionGroup::new(
        Rect {
            x: viewport.width.saturating_sub(428),
            y: viewport.height.saturating_sub(184),
            width: 188,
            height: 152,
        },
        "Fixture section",
        2,
    )
    .with_structure(30, 28, 12, 8, 24, 4)
    .with_focus(true);
    draw_section_group(
        &mut buffer,
        viewport.width,
        viewport.height,
        fixture_section,
        theme,
    );
    draw_fitted_bitmap_text(
        &mut buffer,
        (viewport.width, viewport.height),
        fixture_section.heading_rect(),
        "Section",
        palette.text,
        FittedTextOptions::new(TextRole::Title, scale, true),
    );
    let fixture_values = ["Aqua", "Ready"];
    let mut metadata_row_ready = true;
    for (index, value) in fixture_values.into_iter().enumerate() {
        let row = fixture_section.row_rect(index);
        fill_rect(
            &mut buffer,
            viewport.width,
            viewport.height,
            row,
            if index == 0 {
                palette.row_alternate
            } else {
                palette.surface
            },
            255,
        );
        let metadata = MetadataRow::new(row, if index == 0 { "System" } else { "State" }, value)
            .with_columns(60, 8)
            .with_emphasis(index == 1);
        draw_metadata_row(
            &mut buffer,
            viewport.width,
            viewport.height,
            metadata,
            MetadataRowStyle {
                label_color: palette.secondary_text,
                value_color: if metadata.emphasized {
                    palette.accent
                } else {
                    palette.text
                },
                role: TextRole::Control,
                scale,
            },
        );
        let slots = metadata.slots();
        let semantics = metadata.accessibility();
        metadata_row_ready &= metadata.is_valid()
            && slots.label.fits_in(viewport)
            && slots.value.fits_in(viewport)
            && slots.label.right() < slots.value.x
            && !metadata.accepts_input()
            && semantics.role == "definition"
            && !semantics.name.is_empty()
            && !semantics.value.is_empty()
            && semantics.read_only;
    }
    let section_semantics = fixture_section.accessibility();
    let section_group_ready = fixture_section.is_valid()
        && fixture_section.header_rect().fits_in(viewport)
        && fixture_section.content_rect().fits_in(viewport)
        && fixture_section.footer_rect().fits_in(viewport)
        && fixture_section.row_at(fixture_section.row_rect(0).x, fixture_section.row_rect(0).y)
            == Some(0)
        && fixture_section
            .row_at(
                fixture_section.row_rect(0).x,
                fixture_section.row_rect(0).bottom(),
            )
            .is_none()
        && fixture_section.trailing_rect(1, 48, 18).fits_in(viewport)
        && section_semantics.role == "group"
        && !section_semantics.name.is_empty()
        && section_semantics.focused;

    let fixture_menu = Menu::new(
        Rect {
            x: viewport.width.saturating_sub(220),
            y: viewport.height.saturating_sub(184),
            width: 188,
            height: 152,
        },
        "Fixture menu",
        4,
        1,
        0,
        34,
        4,
    );
    fill_rounded_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        fixture_menu.rect,
        8,
        palette.surface,
        255,
    );
    draw_rect_outline(
        &mut buffer,
        viewport.width,
        viewport.height,
        fixture_menu.rect,
        palette.border,
        255,
    );
    let menu_labels = ["Open", "Properties", "Rename", "Remove"];
    for (index, label) in menu_labels.into_iter().enumerate() {
        let row = fixture_menu.item_rect(index);
        if index == fixture_menu.selected_index {
            fill_rect(
                &mut buffer,
                viewport.width,
                viewport.height,
                row,
                palette.accent_soft,
                255,
            );
        }
        draw_fitted_bitmap_text(
            &mut buffer,
            (viewport.width, viewport.height),
            Rect {
                x: row.x.saturating_add(12),
                width: row.width.saturating_sub(24),
                ..row
            },
            label,
            palette.text,
            FittedTextOptions::new(TextRole::Control, scale, true),
        );
    }
    let menu_semantics = fixture_menu.accessibility();
    let selected_row = fixture_menu.item_rect(fixture_menu.selected_index);
    let menu_ready = fixture_menu.is_valid()
        && selected_row.fits_in(viewport)
        && fixture_menu.item_at(selected_row.x, selected_row.y)
            == Some(fixture_menu.selected_index)
        && fixture_menu
            .item_at(
                fixture_menu.item_rect(0).x,
                fixture_menu.item_rect(0).bottom(),
            )
            .is_none()
        && fixture_menu.keyboard_target(MenuNavigationKey::Previous) == Some(0)
        && menu_semantics.role == "menu"
        && !menu_semantics.name.is_empty()
        && fixture_menu
            .item_accessibility(3, "Remove", false, true)
            .is_some_and(|item| item.role == "menuitem" && item.destructive);

    let probe = ComponentAcceptanceProbe {
        viewport,
        theme,
        scale,
        button_state_count: ComponentState::STANDARD_BUTTON_STATES.len(),
        icon_button_state_count: ComponentState::ICON_BUTTON_STATES.len(),
        list_row_state_count: ComponentState::LIST_ROW_STATES.len(),
        grid_cell_state_count: ComponentState::GRID_CELL_STATES.len(),
        search_field_state_count: ComponentState::SEARCH_FIELD_STATES.len(),
        segmented_control_state_count: ComponentState::SEGMENTED_CONTROL_STATES.len(),
        switch_state_count: ComponentState::SWITCH_STATES.len(),
        sidebar_row_count: ComponentState::LIST_ROW_STATES.len(),
        toolbar_ready,
        window_frame_ready,
        menu_ready,
        section_group_ready,
        metadata_row_ready,
        top_system_bar_ready,
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
                "components=top-system-bar,window-frame,menu,metadata-row,section-group,standard-button,icon-button,search-field,switch,segmented-control,list-row,grid-cell,sidebar-navigation,toolbar viewport={}x{} scale={}/{} theme={} button_states={} icon_button_states={} search_field_states={} switch_states={} segmented_control_states={} list_row_states={} grid_cell_states={} sidebar_rows={} toolbar_ready={} window_frame_ready={} menu_ready={} section_group_ready={} metadata_row_ready={} top_system_bar_ready={} stable_geometry={} input_semantics={} accessibility_semantics={} ready={} checksum={:016x}",
                viewport.width,
                viewport.height,
                scale.numerator(),
                scale.denominator(),
                theme.id(),
                probe.button_state_count,
                probe.icon_button_state_count,
                probe.search_field_state_count,
                probe.switch_state_count,
                probe.segmented_control_state_count,
                probe.list_row_state_count,
                probe.grid_cell_state_count,
                probe.sidebar_row_count,
                probe.toolbar_ready,
                probe.window_frame_ready,
                probe.menu_ready,
                probe.section_group_ready,
                probe.metadata_row_ready,
                probe.top_system_bar_ready,
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
                .matches("components=top-system-bar,window-frame,menu,metadata-row,section-group,standard-button,icon-button,search-field,switch,segmented-control,list-row,grid-cell,sidebar-navigation,toolbar")
                .count(),
            12
        );
        assert_eq!(first.matches(" ready=true checksum=").count(), 12);
    }
}
