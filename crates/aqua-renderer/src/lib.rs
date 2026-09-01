use aqua_installer::{
    InstallMode, InstallProgressEvent, InstallProgressPhase, InstallerChoiceOption,
    InstallerFocusTarget, InstallerFormState, InstallerModel, InstallerStep, InstallerUiState,
    InstallerUserField, InstallerWindowLayout, INSTALL_ESP_LABEL, INSTALL_ESP_SIZE_MIB,
    INSTALL_ROOT_LABEL, KEYBOARD_OPTIONS, LANGUAGE_OPTIONS, TIMEZONE_OPTIONS,
};
use aqua_scene::{MaterialKind, Rect, ShellScene, SurfaceKind, Viewport};
use aqua_shell::{
    desktop_context_menu_with_selection, desktop_grid_cell, files_back_button,
    files_forward_button, files_preview_visible_lines, files_sidebar_navigation, files_toolbar,
    files_visible_rows, running_app_dock, top_system_bar, workspace_switcher, AquaTheme,
    AudioControlStatus, DesktopIconState, DesktopPropertiesModel, DockItem, DockState,
    FilesEntryKind, FilesWindowModel, LauncherCategory, LauncherMode, LauncherState,
    NotificationCenter, SessionAction, SessionMenuState, SettingsWindowModel, SystemOverviewModel,
    TerminalView, TopBarState, DESKTOP_ICONS, SETTINGS_SIDEBAR_NAVIGATION,
};
pub use aqua_text::UI_FONT_FAMILY;
use aqua_text::{GlyphCacheKey, OutputScale, RenderingMode, ShapedLine, TextRole, TextService};
use std::sync::{Mutex, OnceLock};

pub mod components;
mod elevation;
pub mod icons;
pub use components::*;
pub use elevation::*;

pub const UI_FONT_SOURCE: &str = "embedded-ttf";
static TEXT_SERVICE: OnceLock<Option<Mutex<TextService>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowChromePalette {
    pub surface: [u8; 4],
    pub titlebar: [u8; 4],
    pub toolbar: [u8; 4],
    pub sidebar: [u8; 4],
    pub field: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub secondary_text: [u8; 4],
    pub accent: [u8; 4],
    pub accent_soft: [u8; 4],
    pub hover: [u8; 4],
    pub row_alternate: [u8; 4],
}

pub const LIGHTWHITE_WINDOW_CHROME: WindowChromePalette = WindowChromePalette {
    surface: [0xf8, 0xfa, 0xfc, 0xff],
    titlebar: [0xf2, 0xf5, 0xf9, 0xff],
    toolbar: [0xf6, 0xf8, 0xfb, 0xff],
    sidebar: [0xed, 0xf2, 0xf7, 0xff],
    field: [0xff, 0xff, 0xff, 0xff],
    border: [0xc8, 0xd2, 0xde, 0xff],
    text: [0x16, 0x20, 0x2f, 0xff],
    secondary_text: [0x5e, 0x6b, 0x7d, 0xff],
    accent: [0x16, 0x77, 0xff, 0xff],
    accent_soft: [0xdb, 0xe9, 0xfd, 0xff],
    hover: [0xe5, 0xec, 0xf5, 0xff],
    row_alternate: [0xf1, 0xf4, 0xf8, 0xff],
};

pub const SOFTTOUCH_WINDOW_CHROME: WindowChromePalette = WindowChromePalette {
    surface: [0xf7, 0xf6, 0xf3, 0xff],
    titlebar: [0xef, 0xee, 0xea, 0xff],
    toolbar: [0xf2, 0xf1, 0xee, 0xff],
    sidebar: [0xe7, 0xe6, 0xe2, 0xff],
    field: [0xfb, 0xfa, 0xf7, 0xff],
    border: [0xcf, 0xce, 0xc9, 0xff],
    text: [0x20, 0x21, 0x24, 0xff],
    secondary_text: [0x68, 0x69, 0x66, 0xff],
    accent: [0x23, 0x7b, 0xe5, 0xff],
    accent_soft: [0xd9, 0xe7, 0xf8, 0xff],
    hover: [0xe0, 0xe1, 0xdf, 0xff],
    row_alternate: [0xee, 0xed, 0xe9, 0xff],
};

pub const DEEPSIDE_WINDOW_CHROME: WindowChromePalette = WindowChromePalette {
    surface: [0x0d, 0x27, 0x47, 0xff],
    titlebar: [0x0a, 0x20, 0x3b, 0xff],
    toolbar: [0x10, 0x2d, 0x50, 0xff],
    sidebar: [0x12, 0x32, 0x58, 0xff],
    field: [0x09, 0x20, 0x3c, 0xff],
    border: [0x29, 0x4d, 0x73, 0xff],
    text: [0xf6, 0xf9, 0xff, 0xff],
    secondary_text: [0xa9, 0xbd, 0xd2, 0xff],
    accent: [0x3d, 0x9c, 0xff, 0xff],
    accent_soft: [0x17, 0x47, 0x78, 0xff],
    hover: [0x16, 0x3b, 0x65, 0xff],
    row_alternate: [0x10, 0x2d, 0x50, 0xff],
};

pub const NIGHTMARE_WINDOW_CHROME: WindowChromePalette = WindowChromePalette {
    surface: [0x1a, 0x1c, 0x1f, 0xff],
    titlebar: [0x15, 0x17, 0x19, 0xff],
    toolbar: [0x20, 0x22, 0x26, 0xff],
    sidebar: [0x24, 0x27, 0x2b, 0xff],
    field: [0x16, 0x18, 0x1b, 0xff],
    border: [0x39, 0x3d, 0x42, 0xff],
    text: [0xf5, 0xf6, 0xf7, 0xff],
    secondary_text: [0xad, 0xb1, 0xb7, 0xff],
    accent: [0x4a, 0x92, 0xe8, 0xff],
    accent_soft: [0x29, 0x3f, 0x5b, 0xff],
    hover: [0x31, 0x34, 0x39, 0xff],
    row_alternate: [0x20, 0x22, 0x26, 0xff],
};

pub const fn window_chrome_palette(theme: AquaTheme) -> WindowChromePalette {
    match theme {
        AquaTheme::LightWhite => LIGHTWHITE_WINDOW_CHROME,
        AquaTheme::Softtouch => SOFTTOUCH_WINDOW_CHROME,
        AquaTheme::Deepside => DEEPSIDE_WINDOW_CHROME,
        AquaTheme::Nightmare => NIGHTMARE_WINDOW_CHROME,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellPalette {
    pub surface: [u8; 4],
    pub elevated: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub secondary_text: [u8; 4],
    pub accent: [u8; 4],
    pub selection: [u8; 4],
}

pub const fn shell_palette(theme: AquaTheme) -> ShellPalette {
    let chrome = window_chrome_palette(theme);
    ShellPalette {
        surface: chrome.surface,
        elevated: chrome.field,
        border: chrome.border,
        text: chrome.text,
        secondary_text: chrome.secondary_text,
        accent: chrome.accent,
        selection: chrome.accent_soft,
    }
}

pub const TYPOGRAPHY_LAYOUT_FIXTURE_REVISION: &str = "aqua-typography-layout-fixtures-1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypographyLayoutAcceptanceProbe {
    pub viewport: Viewport,
    pub theme: AquaTheme,
    pub scale: OutputScale,
    pub critical_labels_fit: bool,
    pub long_label_contained: bool,
    pub long_label_truncated: bool,
    pub fallback_glyphs: usize,
    pub missing_glyphs: usize,
    pub regions_are_separated: bool,
    pub checksum: u64,
}

impl TypographyLayoutAcceptanceProbe {
    pub const fn is_ready(self) -> bool {
        self.critical_labels_fit
            && self.long_label_contained
            && self.fallback_glyphs > 0
            && self.missing_glyphs == 0
            && self.regions_are_separated
    }
}

pub fn render_typography_layout_acceptance_rgba(
    viewport: Viewport,
    theme: AquaTheme,
    scale: OutputScale,
) -> Option<(Vec<u8>, TypographyLayoutAcceptanceProbe)> {
    let layout = InstallerWindowLayout::for_viewport(viewport).ok()?;
    let palette = window_chrome_palette(theme);
    let mut buffer = vec![0_u8; viewport.width as usize * viewport.height as usize * 4];
    fill_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        Rect {
            x: 0,
            y: 0,
            width: viewport.width,
            height: viewport.height,
        },
        palette.sidebar,
        255,
    );
    fill_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        layout.window,
        palette.surface,
        255,
    );
    fill_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        layout.titlebar,
        palette.titlebar,
        255,
    );
    draw_window_controls(
        &mut buffer,
        viewport.width,
        viewport.height,
        layout.titlebar.x + 18,
        layout.titlebar.y + layout.titlebar.height.saturating_sub(14) / 2,
    );
    let title = draw_fitted_bitmap_text(
        &mut buffer,
        (viewport.width, viewport.height),
        Rect {
            x: layout.titlebar.x + 92,
            y: layout.titlebar.y,
            width: layout.titlebar.width.saturating_sub(112),
            height: layout.titlebar.height,
        },
        "Aqua Linux Gelişmiş Erişilebilirlik ve Yerelleştirme Ayarları",
        palette.text,
        FittedTextOptions::new(TextRole::Title, scale, false),
    );

    let content_padding = layout.content_padding();
    let row_width = layout.content.width.saturating_sub(content_padding * 2);
    let first_row = Rect {
        x: layout.content.x + content_padding,
        y: layout.content.y + 72,
        width: row_width,
        height: 48,
    };
    let second_row = Rect {
        y: first_row.bottom() + 16,
        ..first_row
    };
    for row in [first_row, second_row] {
        fill_rounded_rect(
            &mut buffer,
            viewport.width,
            viewport.height,
            row,
            8,
            palette.field,
            255,
        );
    }
    let long_label = draw_fitted_bitmap_text(
        &mut buffer,
        (viewport.width, viewport.height),
        inset_rect(first_row, 16, 0),
        "Ekran okuyucu ve yüksek karşıtlık seçeneklerini tüm çalışma alanlarında etkinleştir",
        palette.text,
        FittedTextOptions::new(TextRole::Body, scale, false),
    );
    let arabic = draw_fitted_bitmap_text(
        &mut buffer,
        (viewport.width, viewport.height),
        inset_rect(second_row, 16, 0),
        "إعدادات إمكانية الوصول واللغة في أكوا لينكس",
        palette.text,
        FittedTextOptions::new(TextRole::Body, scale, false),
    );

    fill_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        layout.footer,
        palette.toolbar,
        255,
    );
    let controls = [
        (layout.language_control, "Türkçe (Türkiye)", false),
        (layout.cancel_button, "Vazgeç", false),
        (layout.back_button, "Geri", false),
        (layout.forward_button, "Kurulumu Başlat", true),
    ];
    let mut critical_labels_fit = true;
    let mut fallback_glyphs =
        title.fallback_glyphs + long_label.fallback_glyphs + arabic.fallback_glyphs;
    let mut missing_glyphs =
        title.missing_glyphs + long_label.missing_glyphs + arabic.missing_glyphs;
    for (rect, label, primary) in controls {
        fill_rounded_rect(
            &mut buffer,
            viewport.width,
            viewport.height,
            rect,
            8,
            if primary {
                palette.accent
            } else {
                palette.field
            },
            255,
        );
        let outcome = draw_fitted_bitmap_text(
            &mut buffer,
            (viewport.width, viewport.height),
            inset_rect(rect, 8, 0),
            label,
            if primary {
                [0xff, 0xff, 0xff, 0xff]
            } else {
                palette.text
            },
            FittedTextOptions::new(TextRole::Control, scale, true),
        );
        critical_labels_fit &= !outcome.truncated
            && outcome.original_width <= rect.width.saturating_sub(16) as f32
            && outcome.rendered_width <= rect.width.saturating_sub(16) as f32;
        fallback_glyphs += outcome.fallback_glyphs;
        missing_glyphs += outcome.missing_glyphs;
    }

    let checksum = checksum_bytes(&buffer);
    let probe = TypographyLayoutAcceptanceProbe {
        viewport,
        theme,
        scale,
        critical_labels_fit,
        long_label_contained: long_label.rendered_width
            <= first_row.width.saturating_sub(32) as f32,
        long_label_truncated: long_label.truncated,
        fallback_glyphs,
        missing_glyphs,
        regions_are_separated: layout.regions_are_separated()
            && first_row.bottom() < layout.footer.y
            && second_row.bottom() < layout.footer.y,
        checksum,
    };
    Some((buffer, probe))
}

pub fn typography_layout_acceptance_report() -> String {
    let cases = [
        (Viewport::new(800, 600), OutputScale::One),
        (Viewport::new(1280, 800), OutputScale::One),
        (Viewport::new(1536, 1024), OutputScale::FiveQuarters),
    ];
    let mut lines = vec![format!(
        "fixture_revision={TYPOGRAPHY_LAYOUT_FIXTURE_REVISION}"
    )];
    for (viewport, scale) in cases {
        for theme in AquaTheme::ALL {
            let (_, probe) = render_typography_layout_acceptance_rgba(viewport, theme, scale)
                .expect("supported typography acceptance viewport");
            lines.push(format!(
                "viewport={}x{} scale={}/{} theme={} ready={} critical_labels_fit={} long_label_contained={} long_label_truncated={} fallback_glyphs={} missing_glyphs={} regions_are_separated={} checksum={:016x}",
                viewport.width,
                viewport.height,
                scale.numerator(),
                scale.denominator(),
                theme.id(),
                probe.is_ready(),
                probe.critical_labels_fit,
                probe.long_label_contained,
                probe.long_label_truncated,
                probe.fallback_glyphs,
                probe.missing_glyphs,
                probe.regions_are_separated,
                probe.checksum,
            ));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn apply_shell_palette(rgba: &mut [u8], theme: AquaTheme) {
    if theme == AquaTheme::LightWhite {
        return;
    }
    let palette = shell_palette(theme);
    for pixel in rgba.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        let replacement = match [pixel[0], pixel[1], pixel[2]] {
            [0xf8, 0xfb, 0xff] | [0xf7, 0xfa, 0xfe] | [0xf4, 0xf8, 0xfc] => Some(palette.surface),
            [0xff, 0xff, 0xff] => Some(palette.elevated),
            [0xb7, 0xc8, 0xdc] | [0x86, 0xdd, 0xf3] | [0x58, 0x9c, 0xb7] => Some(palette.border),
            [0x16, 0x22, 0x32] | [0x15, 0x26, 0x39] | [0x16, 0x24, 0x34] => Some(palette.text),
            [0x42, 0x56, 0x6d]
            | [0x7f, 0x8c, 0x9d]
            | [0x9f, 0xd7, 0xe8]
            | [0xb5, 0xde, 0xe8]
            | [0xc8, 0xe3, 0xec]
            | [0xd1, 0xe8, 0xef] => Some(palette.secondary_text),
            [0xf1, 0xfb, 0xff]
            | [0xf4, 0xfb, 0xff]
            | [0xf4, 0xfd, 0xff]
            | [0xf5, 0xfb, 0xff]
            | [0xee, 0xfb, 0xff] => Some(palette.text),
            [0xd8, 0xea, 0xff] | [0x16, 0x78, 0xa9] | [0x08, 0x45, 0x70] => Some(palette.selection),
            [0x08, 0x69, 0xc8]
            | [0x0b, 0x76, 0xe5]
            | [0x27, 0xc8, 0xec]
            | [0x30, 0xcf, 0xe9]
            | [0x62, 0xdd, 0xf2]
            | [0x72, 0xe3, 0xf5]
            | [0x9b, 0xeb, 0xf7] => Some(palette.accent),
            [0x02, 0x20, 0x36]
            | [0x02, 0x1c, 0x32]
            | [0x02, 0x18, 0x2b]
            | [0x03, 0x18, 0x29]
            | [0x03, 0x2c, 0x49]
            | [0x01, 0x1b, 0x2c] => Some(palette.surface),
            _ => None,
        };
        if let Some(mut color) = replacement {
            color[3] = pixel[3];
            pixel.copy_from_slice(&color);
        }
    }
}

pub fn embedded_ui_font_ready() -> bool {
    text_service().is_some()
}

fn text_service() -> Option<&'static Mutex<TextService>> {
    TEXT_SERVICE
        .get_or_init(|| TextService::new().ok().map(Mutex::new))
        .as_ref()
}

pub const RENDERER_STATUS: &str = "plan-only";
pub const RENDER_BACKEND: &str = "headless-command-plan";
pub const CLIENT_SAMPLE_GRID_PIXELS: usize = 4;

pub fn render_pale_wave_wallpaper_rgba(width: u32, height: u32) -> Vec<u8> {
    if width == 0 || height == 0 {
        return Vec::new();
    }

    const PALETTE: [[u8; 3]; 6] = [
        [0xf6, 0xfa, 0xff],
        [0xd9, 0xe8, 0xf9],
        [0xf0, 0xf6, 0xfd],
        [0xcf, 0xe0, 0xf4],
        [0xe7, 0xf0, 0xfb],
        [0xb9, 0xd2, 0xee],
    ];
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    let tau = std::f32::consts::TAU;
    for y in 0..height {
        let normalized_y = y as f32 / height.saturating_sub(1).max(1) as f32;
        for x in 0..width {
            let normalized_x = x as f32 / width.saturating_sub(1).max(1) as f32;
            let boundaries = [
                0.25 + 0.060 * (normalized_x * tau * 1.05 + 0.35).sin(),
                0.39 + 0.050 * (normalized_x * tau * 0.92 + 2.10).sin(),
                0.53 + 0.055 * (normalized_x * tau * 1.12 + 3.45).sin(),
                0.68 + 0.060 * (normalized_x * tau * 0.86 + 5.10).sin(),
                0.83 + 0.050 * (normalized_x * tau * 1.18 + 1.45).sin(),
            ];
            let mut color = PALETTE[0].map(f32::from);
            for (index, boundary) in boundaries.into_iter().enumerate() {
                let blend = smoothstep(boundary - 0.006, boundary + 0.006, normalized_y);
                let next = PALETTE[index + 1].map(f32::from);
                for channel in 0..3 {
                    color[channel] = color[channel] * (1.0 - blend) + next[channel] * blend;
                }
            }
            let soft_light = (1.0 - normalized_y) * 3.0;
            rgba.extend_from_slice(&[
                (color[0] + soft_light).min(255.0).round() as u8,
                (color[1] + soft_light).min(255.0).round() as u8,
                (color[2] + soft_light).min(255.0).round() as u8,
                0xff,
            ]);
        }
    }
    rgba
}

pub fn export_pale_wave_wallpaper_png(width: u32, height: u32) -> Vec<u8> {
    let rgba = render_pale_wave_wallpaper_rgba(width, height);
    encode_png_rgba(width, height, &rgba)
}

fn smoothstep(edge_start: f32, edge_end: f32, value: f32) -> f32 {
    let normalized = ((value - edge_start) / (edge_end - edge_start)).clamp(0.0, 1.0);
    normalized * normalized * (3.0 - 2.0 * normalized)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallerImageSource<'a> {
    width: u32,
    height: u32,
    rgba: &'a [u8],
}

impl<'a> InstallerImageSource<'a> {
    pub fn new(width: u32, height: u32, rgba: &'a [u8]) -> Result<Self, &'static str> {
        let expected = width as usize * height as usize * 4;
        if width == 0 || height == 0 || rgba.len() != expected {
            return Err("installer image source must be non-empty rgba8888");
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererPreference {
    Auto,
    Gpu,
    Software,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuRuntimeCapabilities {
    pub drm: bool,
    pub gbm: bool,
    pub egl: bool,
    pub gles2: bool,
}

impl GpuRuntimeCapabilities {
    pub fn is_ready(self) -> bool {
        self.drm && self.gbm && self.egl && self.gles2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererBackendDecision {
    pub preference: RendererPreference,
    pub selected_backend: &'static str,
    pub gpu_runtime_ready: bool,
    pub software_fallback: bool,
    pub can_start: bool,
}

impl RendererBackendDecision {
    pub fn dump_lines(self) -> Vec<String> {
        vec![
            format!("renderer_preference={}", self.preference.as_str()),
            format!("renderer_selected_backend={}", self.selected_backend),
            format!("renderer_gpu_runtime_ready={}", self.gpu_runtime_ready),
            format!("renderer_software_fallback={}", self.software_fallback),
            format!("renderer_can_start={}", self.can_start),
        ]
    }
}

impl RendererPreference {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "gpu" => Some(Self::Gpu),
            "software" => Some(Self::Software),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Gpu => "gpu",
            Self::Software => "software",
        }
    }
}

pub fn select_renderer_backend(
    preference: RendererPreference,
    capabilities: GpuRuntimeCapabilities,
) -> RendererBackendDecision {
    let gpu_runtime_ready = capabilities.is_ready();
    match preference {
        RendererPreference::Auto if gpu_runtime_ready => RendererBackendDecision {
            preference,
            selected_backend: "smithay-gles2-gbm",
            gpu_runtime_ready,
            software_fallback: false,
            can_start: true,
        },
        RendererPreference::Auto => RendererBackendDecision {
            preference,
            selected_backend: "aqua-software-raster",
            gpu_runtime_ready,
            software_fallback: true,
            can_start: true,
        },
        RendererPreference::Gpu => RendererBackendDecision {
            preference,
            selected_backend: if gpu_runtime_ready {
                "smithay-gles2-gbm"
            } else {
                "unavailable"
            },
            gpu_runtime_ready,
            software_fallback: false,
            can_start: gpu_runtime_ready,
        },
        RendererPreference::Software => RendererBackendDecision {
            preference,
            selected_backend: "aqua-software-raster",
            gpu_runtime_ready,
            software_fallback: false,
            can_start: true,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherOverlayProbe {
    pub rendered: bool,
    pub mode: &'static str,
    pub category_count: usize,
    pub visible_app_count: usize,
    pub selected_index: usize,
    pub query_visible: bool,
    pub primitive_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMenuOverlay {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub selected_action: &'static str,
    pub confirmation_visible: bool,
    pub primitive_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationToastOverlay {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub notification_id: Option<u64>,
    pub primitive_count: usize,
}

pub fn render_notification_toast_rgba(
    width: u32,
    height: u32,
    center: &NotificationCenter,
) -> NotificationToastOverlay {
    render_notification_toast_rgba_with_theme(width, height, center, AquaTheme::LightWhite)
}

pub fn render_notification_toast_rgba_with_theme(
    width: u32,
    height: u32,
    center: &NotificationCenter,
    theme: AquaTheme,
) -> NotificationToastOverlay {
    let mut overlay = render_notification_toast_rgba_base(width, height, center, true);
    apply_shell_palette(&mut overlay.rgba, theme);
    overlay
}

pub fn render_notification_toast_rgba_with_cached_icons(
    width: u32,
    height: u32,
    center: &NotificationCenter,
    theme: AquaTheme,
    cache: &mut icons::IconRasterCache,
) -> Result<NotificationToastOverlay, icons::IconError> {
    let mut overlay = render_notification_toast_rgba_base(width, height, center, false);
    apply_shell_palette(&mut overlay.rgba, theme);
    if let Some(notification) = center.active() {
        let toast = NotificationToast::new(
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            &notification.source,
            &notification.title,
            &notification.body,
        );
        if !toast.is_valid() {
            return Ok(overlay);
        }
        let icon = toast.slots().icon;
        let key = icons::IconRasterKey::new(
            icons::IconRole::Notification,
            theme,
            icons::IconState::Normal,
            u16::try_from(icon.width).expect("notification icon size fits u16"),
            aqua_text::OutputScale::One,
        )?;
        let raster = cache.get_or_render(key)?;
        icons::composite_icon(&mut overlay.rgba, width, height, icon.x, icon.y, &raster);
        overlay.primitive_count += 1;
    }
    Ok(overlay)
}

fn render_notification_toast_rgba_base(
    width: u32,
    height: u32,
    center: &NotificationCenter,
    draw_placeholder_icon: bool,
) -> NotificationToastOverlay {
    let mut rgba = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    let Some(notification) = center.active() else {
        return NotificationToastOverlay {
            width,
            height,
            rgba,
            notification_id: None,
            primitive_count: 0,
        };
    };
    if width == 0 || height == 0 {
        return NotificationToastOverlay {
            width,
            height,
            rgba,
            notification_id: Some(notification.id),
            primitive_count: 0,
        };
    }

    let toast = NotificationToast::new(
        Rect {
            x: 0,
            y: 0,
            width,
            height,
        },
        &notification.source,
        &notification.title,
        &notification.body,
    );
    if !toast.is_valid() {
        return NotificationToastOverlay {
            width,
            height,
            rgba,
            notification_id: Some(notification.id),
            primitive_count: 0,
        };
    }
    let high_resolution = width >= 360;
    let scale = if high_resolution { 2 } else { 1 };
    let slots = toast.slots();
    fill_transparent_rect(
        &mut rgba,
        width,
        height,
        Rect {
            x: toast.rect.x,
            y: toast.rect.y,
            width: toast.rect.width,
            height: toast.rect.height,
        },
        [0x02, 0x20, 0x36, 0x78],
    );
    if draw_placeholder_icon {
        fill_transparent_rect(
            &mut rgba,
            width,
            height,
            slots.icon,
            [0x27, 0xc8, 0xec, 0xb8],
        );
        draw_bitmap_text(
            &mut rgba,
            (width, height),
            (
                slots.icon.x + slots.icon.width / 4,
                slots.icon.y + slots.icon.height / 5,
            ),
            "A",
            [0xf4, 0xfd, 0xff, 0xff],
            scale,
        );
    }
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (slots.title.x, slots.title.y),
        &notification.title,
        [0xf4, 0xfb, 0xff, 0xff],
        scale,
    );
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (slots.body.x, slots.body.y),
        &notification.body,
        [0xc8, 0xe3, 0xec, 0xff],
        scale,
    );
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (slots.source.x, slots.source.y),
        &notification.source,
        [0x62, 0xdd, 0xf2, 0xff],
        scale,
    );
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (slots.dismiss_icon.x, slots.dismiss_icon.y),
        "X",
        [0xd8, 0xf3, 0xfa, 0xff],
        scale,
    );
    NotificationToastOverlay {
        width,
        height,
        rgba,
        notification_id: Some(notification.id),
        primitive_count: 7,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemOverviewOverlay {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub memory_used_percent: u8,
    pub primitive_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopIconsOverlay {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub selected: Option<usize>,
    pub context_menu: Option<usize>,
    pub context_menu_selected_row: Option<usize>,
    pub primitive_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockOverlay {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub running_item_count: usize,
    pub active_workspace: usize,
    pub group_count: usize,
    pub primitive_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopBarOverlay {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub primitive_count: usize,
}

pub fn render_top_bar_rgba(width: u32, height: u32, state: &TopBarState) -> TopBarOverlay {
    render_top_bar_rgba_with_theme(width, height, state, AquaTheme::LightWhite)
}

pub fn render_top_bar_rgba_with_theme(
    width: u32,
    height: u32,
    state: &TopBarState,
    theme: AquaTheme,
) -> TopBarOverlay {
    let mut overlay = render_top_bar_rgba_base(width, height, state, true);
    apply_shell_palette(&mut overlay.rgba, theme);
    overlay
}

pub fn render_top_bar_rgba_with_cached_icons(
    width: u32,
    height: u32,
    state: &TopBarState,
    theme: AquaTheme,
    cache: &mut icons::IconRasterCache,
) -> Result<TopBarOverlay, icons::IconError> {
    let mut overlay = render_top_bar_rgba_base(width, height, state, false);
    apply_shell_palette(&mut overlay.rgba, theme);
    let bar = top_system_bar(width, height);
    if bar.is_valid() {
        for (role, icon_state, status) in [
            (
                icons::IconRole::Volume,
                if state.audio_available {
                    icons::IconState::Normal
                } else {
                    icons::IconState::Disabled
                },
                TopSystemStatus::Audio,
            ),
            (
                icons::IconRole::Wifi,
                if state.network_connected {
                    icons::IconState::Normal
                } else {
                    icons::IconState::Disabled
                },
                TopSystemStatus::Network,
            ),
            (
                icons::IconRole::Battery,
                state
                    .battery_percent
                    .map_or(icons::IconState::Disabled, |_| icons::IconState::Normal),
                TopSystemStatus::Battery,
            ),
        ] {
            let slot = bar.status_rect(status);
            let key = icons::IconRasterKey::new(
                role,
                theme,
                icon_state,
                20,
                aqua_text::OutputScale::One,
            )?;
            let icon = cache.get_or_render(key)?;
            icons::composite_icon(
                &mut overlay.rgba,
                width,
                height,
                slot.x + slot.width.saturating_sub(20) / 2,
                slot.y + slot.height.saturating_sub(20) / 2,
                &icon,
            );
            overlay.primitive_count += 1;
        }
    }
    Ok(overlay)
}

fn render_top_bar_rgba_base(
    width: u32,
    height: u32,
    state: &TopBarState,
    draw_placeholder_icons: bool,
) -> TopBarOverlay {
    let mut rgba = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    let bar = top_system_bar(width, height);
    if !bar.is_valid() {
        return TopBarOverlay {
            width,
            height,
            rgba,
            primitive_count: 0,
        };
    }

    fill_transparent_rect(&mut rgba, width, height, bar.rect, [0xf8, 0xfb, 0xff, 0xf2]);
    fill_transparent_rect(
        &mut rgba,
        width,
        height,
        bar.separator_rect(),
        [0xb7, 0xc8, 0xdc, 0xc0],
    );

    let mut primitive_count = 2;
    primitive_count += draw_top_bar_brand_mark(&mut rgba, width, height, bar);
    let brand = bar.brand_rect();
    draw_fitted_bitmap_text(
        &mut rgba,
        (width, height),
        Rect {
            x: brand.x.saturating_add(34),
            width: brand.width.saturating_sub(34),
            ..brand
        },
        &state.product_label,
        [0x16, 0x22, 0x32, 0xff],
        FittedTextOptions::new(TextRole::Control, OutputScale::One, false),
    );
    primitive_count += 1;

    draw_fitted_bitmap_text(
        &mut rgba,
        (width, height),
        bar.clock_rect(),
        &state.clock_label,
        [0x16, 0x22, 0x32, 0xff],
        FittedTextOptions::new(TextRole::Control, OutputScale::One, true),
    );
    primitive_count += 1;
    primitive_count += if draw_placeholder_icons {
        draw_top_bar_status_icons(&mut rgba, width, height, state, bar)
    } else {
        draw_top_bar_power_icon(&mut rgba, width, height, bar)
    };

    TopBarOverlay {
        width,
        height,
        rgba,
        primitive_count,
    }
}

fn draw_top_bar_brand_mark(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    bar: TopSystemBar<'_>,
) -> usize {
    let color = [0x08, 0x69, 0xc8, 0xff];
    let brand = bar.brand_rect();
    let center_y = brand.y + brand.height / 2;
    let left = brand.x + 6;
    let peak = brand.x + 15;
    let right = brand.x + 24;
    for offset in 0..2 {
        draw_transparent_line(
            rgba,
            width,
            height,
            (left + offset, center_y + 9),
            (peak, center_y.saturating_sub(9) + offset),
            2,
            color,
        );
        draw_transparent_line(
            rgba,
            width,
            height,
            (peak, center_y.saturating_sub(9) + offset),
            (right - offset, center_y + 9),
            2,
            color,
        );
    }
    draw_transparent_line(
        rgba,
        width,
        height,
        (left, center_y + 9),
        (peak, center_y + 4),
        2,
        color,
    );
    draw_transparent_line(
        rgba,
        width,
        height,
        (peak, center_y + 4),
        (right, center_y + 9),
        2,
        color,
    );
    6
}

fn draw_top_bar_status_icons(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    state: &TopBarState,
    bar: TopSystemBar<'_>,
) -> usize {
    let color = [0x16, 0x22, 0x32, 0xff];
    let muted = [0x7f, 0x8c, 0x9d, 0xff];
    let audio = bar.status_rect(TopSystemStatus::Audio);
    let center_y = audio.y + audio.height / 2;
    let start_x = audio.x;
    let audio_color = if state.audio_available { color } else { muted };

    fill_transparent_rect(
        rgba,
        width,
        height,
        Rect {
            x: start_x,
            y: center_y.saturating_sub(3),
            width: 4,
            height: 7,
        },
        audio_color,
    );
    draw_transparent_line(
        rgba,
        width,
        height,
        (start_x + 4, center_y.saturating_sub(3)),
        (start_x + 9, center_y.saturating_sub(7)),
        2,
        audio_color,
    );
    draw_transparent_line(
        rgba,
        width,
        height,
        (start_x + 9, center_y.saturating_sub(7)),
        (start_x + 9, center_y + 7),
        2,
        audio_color,
    );
    draw_transparent_line(
        rgba,
        width,
        height,
        (start_x + 9, center_y + 7),
        (start_x + 4, center_y + 3),
        2,
        audio_color,
    );
    if state.audio_available {
        draw_transparent_line(
            rgba,
            width,
            height,
            (start_x + 13, center_y.saturating_sub(4)),
            (start_x + 16, center_y),
            1,
            audio_color,
        );
        draw_transparent_line(
            rgba,
            width,
            height,
            (start_x + 16, center_y),
            (start_x + 13, center_y + 4),
            1,
            audio_color,
        );
    } else {
        draw_transparent_line(
            rgba,
            width,
            height,
            (start_x + 12, center_y.saturating_sub(4)),
            (start_x + 18, center_y + 4),
            1,
            muted,
        );
    }

    let network_x = bar.status_rect(TopSystemStatus::Network).x;
    let network_color = if state.network_connected {
        color
    } else {
        muted
    };
    for inset in 0..3 {
        draw_transparent_line(
            rgba,
            width,
            height,
            (
                network_x + inset * 3,
                center_y.saturating_sub(7) + inset * 3,
            ),
            (network_x + 9, center_y + 2 + inset),
            1,
            network_color,
        );
        draw_transparent_line(
            rgba,
            width,
            height,
            (
                network_x + 18 - inset * 3,
                center_y.saturating_sub(7) + inset * 3,
            ),
            (network_x + 9, center_y + 2 + inset),
            1,
            network_color,
        );
    }
    fill_transparent_circle(
        rgba,
        width,
        height,
        network_x + 9,
        center_y + 7,
        2,
        network_color,
    );

    let battery_x = bar.status_rect(TopSystemStatus::Battery).x;
    fill_transparent_rounded_rect(
        rgba,
        width,
        height,
        Rect {
            x: battery_x,
            y: center_y.saturating_sub(6),
            width: 24,
            height: 12,
        },
        2,
        color,
    );
    fill_transparent_rect(
        rgba,
        width,
        height,
        Rect {
            x: battery_x + 2,
            y: center_y.saturating_sub(4),
            width: 20,
            height: 8,
        },
        [0xf8, 0xfb, 0xff, 0xff],
    );
    if let Some(percent) = state.battery_percent {
        fill_transparent_rect(
            rgba,
            width,
            height,
            Rect {
                x: battery_x + 3,
                y: center_y.saturating_sub(3),
                width: 18 * u32::from(percent) / 100,
                height: 6,
            },
            [0x08, 0x69, 0xc8, 0xff],
        );
    }
    fill_transparent_rect(
        rgba,
        width,
        height,
        Rect {
            x: battery_x + 24,
            y: center_y.saturating_sub(2),
            width: 2,
            height: 5,
        },
        color,
    );

    draw_top_bar_power_icon(rgba, width, height, bar);
    21
}

fn draw_top_bar_power_icon(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    bar: TopSystemBar<'_>,
) -> usize {
    let color = [0x16, 0x22, 0x32, 0xff];
    let session = bar.session_rect();
    let center_y = session.y + session.height / 2;
    let power_x = session.right().saturating_sub(12);
    fill_transparent_circle(rgba, width, height, power_x, center_y, 8, color);
    fill_transparent_circle(
        rgba,
        width,
        height,
        power_x,
        center_y,
        5,
        [0xf8, 0xfb, 0xff, 0xff],
    );
    draw_transparent_line(
        rgba,
        width,
        height,
        (power_x, center_y.saturating_sub(9)),
        (power_x, center_y),
        2,
        color,
    );
    3
}

pub fn render_dock_rgba(width: u32, height: u32, state: &DockState) -> DockOverlay {
    render_dock_rgba_with_theme(width, height, state, AquaTheme::LightWhite)
}

pub fn render_dock_rgba_with_theme(
    width: u32,
    height: u32,
    state: &DockState,
    theme: AquaTheme,
) -> DockOverlay {
    let mut overlay = render_dock_rgba_base(width, height, state, true);
    apply_shell_palette(&mut overlay.rgba, theme);
    overlay
}

pub fn render_dock_rgba_with_cached_icons(
    width: u32,
    height: u32,
    state: &DockState,
    theme: AquaTheme,
    cache: &mut icons::IconRasterCache,
) -> Result<DockOverlay, icons::IconError> {
    let mut overlay = render_dock_rgba_base(width, height, state, false);
    apply_shell_palette(&mut overlay.rgba, theme);
    let running_dock = running_app_dock(width, height);
    if width >= 640 && height >= 48 && running_dock.is_valid() {
        for (index, item) in DockItem::ALL.iter().copied().enumerate() {
            let role = match item {
                DockItem::Files => icons::IconRole::Files,
                DockItem::Settings => icons::IconRole::Settings,
                DockItem::Trash => icons::IconRole::Trash,
            };
            let key = icons::IconRasterKey::new(
                role,
                theme,
                icons::IconState::Normal,
                48,
                aqua_text::OutputScale::One,
            )?;
            let icon = cache.get_or_render(key)?;
            let icon_rect = running_dock.raster_icon_rect(index);
            icons::composite_icon(
                &mut overlay.rgba,
                width,
                height,
                icon_rect.x,
                icon_rect.y,
                &icon,
            );
            overlay.primitive_count += 1;
        }
    }
    Ok(overlay)
}

fn render_dock_rgba_base(
    width: u32,
    height: u32,
    state: &DockState,
    draw_placeholder_icons: bool,
) -> DockOverlay {
    let mut rgba = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    if width < 640 || height < 48 {
        return DockOverlay {
            width,
            height,
            rgba,
            running_item_count: 0,
            active_workspace: 0,
            group_count: 0,
            primitive_count: 0,
        };
    }
    let surface = [0xf7, 0xfa, 0xfe, 0xe8];
    let selected = [0xd8, 0xea, 0xff, 0xff];
    let ink = [0x15, 0x26, 0x39, 0xff];
    let left_width = 132;
    let running_dock = running_app_dock(width, height);
    let workspaces = workspace_switcher(
        width,
        height,
        state.active_workspace.min(aqua_shell::WORKSPACE_COUNT - 1),
    );
    let mut primitives = 3;
    let mut running_item_count = 0;

    for rect in [
        Rect {
            x: 0,
            y: 0,
            width: left_width,
            height,
        },
        Rect {
            x: running_dock.rect.x,
            y: running_dock.rect.y,
            width: running_dock.rect.width,
            height: running_dock.rect.height,
        },
        workspaces.rect,
    ] {
        fill_transparent_rounded_rect(&mut rgba, width, height, rect, 12, surface);
    }

    if state.applications_open {
        fill_transparent_rounded_rect(
            &mut rgba,
            width,
            height,
            Rect {
                x: 6,
                y: 6,
                width: 52,
                height: height - 12,
            },
            9,
            selected,
        );
        primitives += 1;
    }
    for row in 0..3 {
        for column in 0..3 {
            fill_transparent_circle(
                &mut rgba,
                width,
                height,
                18 + column * 12,
                22 + row * 12,
                3,
                ink,
            );
            primitives += 1;
        }
    }

    if state.search_open {
        fill_transparent_rounded_rect(
            &mut rgba,
            width,
            height,
            Rect {
                x: 72,
                y: 6,
                width: 54,
                height: height - 12,
            },
            9,
            selected,
        );
        primitives += 1;
    }
    fill_transparent_circle(&mut rgba, width, height, 96, 32, 12, ink);
    fill_transparent_circle(&mut rgba, width, height, 96, 32, 8, surface);
    draw_transparent_line(&mut rgba, width, height, (104, 41), (115, 52), 3, ink);
    primitives += 3;

    for (index, item) in DockItem::ALL.iter().copied().enumerate() {
        let icon_rect = running_dock.icon_rect(index);
        if draw_placeholder_icons {
            primitives += draw_desktop_icon(
                &mut rgba,
                width,
                height,
                icon_rect.x,
                icon_rect.y,
                item.id(),
            );
        }
        if state.item_running(item) {
            running_item_count += 1;
            let indicator = running_dock.indicator_rect(index);
            fill_transparent_circle(
                &mut rgba,
                width,
                height,
                indicator.x + indicator.width / 2,
                indicator.y + indicator.height / 2,
                indicator.width / 2,
                [0x0b, 0x76, 0xe5, 0xff],
            );
            primitives += 1;
        }
    }

    for index in 0..workspaces.workspace_count {
        let thumbnail = workspaces.thumbnail_rect(index);
        let active = workspaces.is_active(index);
        fill_transparent_rounded_rect(
            &mut rgba,
            width,
            height,
            thumbnail,
            8,
            if active {
                selected
            } else {
                [0xff, 0xff, 0xff, 0xb8]
            },
        );
        if active {
            let indicator = workspaces.active_indicator_rect();
            fill_transparent_rect(
                &mut rgba,
                width,
                height,
                indicator,
                [0x0b, 0x76, 0xe5, 0xff],
            );
            primitives += 1;
        }
        primitives += 1;
    }
    DockOverlay {
        width,
        height,
        rgba,
        running_item_count,
        active_workspace: workspaces.active_index,
        group_count: 3,
        primitive_count: primitives,
    }
}

pub fn export_dock_png(width: u32, height: u32, state: &DockState) -> Vec<u8> {
    let overlay = render_dock_rgba(width, height, state);
    encode_png_rgba(overlay.width, overlay.height, &overlay.rgba)
}

pub fn render_desktop_icons_rgba(
    width: u32,
    height: u32,
    state: &DesktopIconState,
) -> DesktopIconsOverlay {
    render_desktop_icons_rgba_with_theme(width, height, state, AquaTheme::LightWhite)
}

pub fn render_desktop_icons_rgba_with_theme(
    width: u32,
    height: u32,
    state: &DesktopIconState,
    theme: AquaTheme,
) -> DesktopIconsOverlay {
    let mut overlay = render_desktop_icons_rgba_base(width, height, state, true);
    apply_shell_palette(&mut overlay.rgba, theme);
    overlay
}

pub fn render_desktop_icons_rgba_with_cached_icons(
    width: u32,
    height: u32,
    state: &DesktopIconState,
    theme: AquaTheme,
    cache: &mut icons::IconRasterCache,
) -> Result<DesktopIconsOverlay, icons::IconError> {
    let mut overlay = render_desktop_icons_rgba_base(width, height, state, false);
    apply_shell_palette(&mut overlay.rgba, theme);
    for (index, icon) in DESKTOP_ICONS.iter().enumerate() {
        let cell = desktop_grid_cell(index, icon.label, state.selected() == Some(index), 0, 0);
        let slots = cell.slots();
        let role = match icon.id {
            "files" => icons::IconRole::Files,
            "settings" => icons::IconRole::Settings,
            "trash" => icons::IconRole::Trash,
            _ => continue,
        };
        let key = icons::IconRasterKey::new(
            role,
            theme,
            if state.selected() == Some(index) {
                icons::IconState::Selected
            } else {
                icons::IconState::Normal
            },
            64,
            aqua_text::OutputScale::One,
        )?;
        let raster = cache.get_or_render(key)?;
        icons::composite_icon(
            &mut overlay.rgba,
            width,
            height,
            slots.icon.x,
            slots.icon.y,
            &raster,
        );
        overlay.primitive_count += 1;
    }
    Ok(overlay)
}

fn render_desktop_icons_rgba_base(
    width: u32,
    height: u32,
    state: &DesktopIconState,
    draw_placeholder_icons: bool,
) -> DesktopIconsOverlay {
    let mut rgba = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    let mut primitives = 0;
    for (index, icon) in DESKTOP_ICONS.iter().enumerate() {
        let cell = desktop_grid_cell(index, icon.label, state.selected() == Some(index), 0, 0);
        let slots = cell.slots();
        primitives += draw_grid_cell(&mut rgba, width, height, cell, AquaTheme::LightWhite);
        if draw_placeholder_icons {
            primitives += draw_desktop_icon(
                &mut rgba,
                width,
                height,
                slots.icon.x,
                slots.icon.y,
                icon.id,
            );
        }
        let label_x = match icon.id {
            "files" => 37,
            "settings" => 27,
            "trash" => 35,
            _ => 20,
        };
        let label_width = match icon.id {
            "settings" => 68,
            _ => 54,
        };
        fill_transparent_rounded_rect(
            &mut rgba,
            width,
            height,
            Rect {
                x: (104 - label_width) / 2,
                y: slots.primary.y,
                width: label_width,
                height: 21,
            },
            6,
            [0x01, 0x1b, 0x2c, 0x9c],
        );
        draw_bitmap_text(
            &mut rgba,
            (width, height),
            (label_x, slots.primary.y + 2),
            icon.label,
            [0xf5, 0xfb, 0xff, 0xff],
            1,
        );
        primitives += 2;
    }
    if let Some((index, menu)) = state
        .context_menu()
        .zip(state.context_menu_selected_row())
        .and_then(|(index, selected_row)| {
            desktop_context_menu_with_selection(index, selected_row).map(|menu| (index, menu))
        })
    {
        let first_row = menu.item_rect(0);
        let second_row = menu.item_rect(1);
        fill_transparent_rect(
            &mut rgba,
            width,
            height,
            menu.rect,
            [0x03, 0x2c, 0x49, 0xf0],
        );
        fill_transparent_rounded_rect(
            &mut rgba,
            width,
            height,
            menu.item_rect(menu.selected_index),
            6,
            [0x25, 0x84, 0xa8, 0x88],
        );
        fill_transparent_rect(
            &mut rgba,
            width,
            height,
            Rect {
                x: menu.rect.x + 1,
                y: menu.rect.y + 1,
                width: menu.rect.width.saturating_sub(2),
                height: 2,
            },
            [0x9f, 0xe9, 0xff, 0xc0],
        );
        fill_transparent_rect(
            &mut rgba,
            width,
            height,
            Rect {
                x: menu.rect.x + 8,
                y: first_row.bottom().saturating_sub(1),
                width: menu.rect.width.saturating_sub(16),
                height: 1,
            },
            [0x58, 0x9c, 0xb7, 0xb0],
        );
        draw_bitmap_text(
            &mut rgba,
            (width, height),
            (first_row.x + 12, first_row.y + 9),
            "OPEN",
            [0xf1, 0xfb, 0xff, 0xff],
            1,
        );
        let trash_confirmation = if DESKTOP_ICONS[index].id == "trash" {
            state.trash_confirmation_dialog(second_row)
        } else {
            None
        };
        draw_bitmap_text(
            &mut rgba,
            (width, height),
            trash_confirmation.map_or((second_row.x + 12, second_row.y + 6), |dialog| {
                let title = dialog.slots().title;
                (title.x, title.y)
            }),
            if DESKTOP_ICONS[index].id == "trash" {
                trash_confirmation.map_or("EMPTY TRASH", |dialog| dialog.title)
            } else {
                "PROPERTIES"
            },
            if state.trash_empty_confirmation() {
                [0xff, 0xc1, 0x8f, 0xff]
            } else {
                [0xb5, 0xde, 0xe8, 0xff]
            },
            1,
        );
        primitives += 6;
    }
    DesktopIconsOverlay {
        width,
        height,
        rgba,
        selected: state.selected(),
        context_menu: state.context_menu(),
        context_menu_selected_row: state.context_menu_selected_row(),
        primitive_count: primitives,
    }
}

pub fn export_desktop_icons_png(width: u32, height: u32, state: &DesktopIconState) -> Vec<u8> {
    let overlay = render_desktop_icons_rgba(width, height, state);
    encode_png_rgba(overlay.width, overlay.height, &overlay.rgba)
}

fn draw_desktop_icon(
    rgba: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    icon_id: &str,
) -> usize {
    fill_transparent_rounded_rect(
        rgba,
        width,
        height,
        Rect {
            x: x + 2,
            y: y + 3,
            width: 60,
            height: 64,
        },
        12,
        [0x00, 0x18, 0x2b, 0x88],
    );
    fill_transparent_rounded_rect(
        rgba,
        width,
        height,
        Rect {
            x,
            y,
            width: 64,
            height: 64,
        },
        12,
        [0x8d, 0xe7, 0xff, 0xb8],
    );
    fill_transparent_rounded_rect(
        rgba,
        width,
        height,
        Rect {
            x: x + 2,
            y: y + 2,
            width: 60,
            height: 60,
        },
        10,
        [0x05, 0x4c, 0x78, 0xc8],
    );
    fill_transparent_rounded_rect(
        rgba,
        width,
        height,
        Rect {
            x: x + 5,
            y: y + 4,
            width: 54,
            height: 4,
        },
        2,
        [0xd9, 0xf8, 0xff, 0xa8],
    );
    match icon_id {
        "files" => {
            fill_transparent_rounded_rect(
                rgba,
                width,
                height,
                Rect {
                    x: x + 8,
                    y: y + 19,
                    width: 48,
                    height: 40,
                },
                5,
                [0xd7, 0xf8, 0xff, 0xff],
            );
            fill_transparent_rounded_rect(
                rgba,
                width,
                height,
                Rect {
                    x: x + 11,
                    y: y + 14,
                    width: 24,
                    height: 13,
                },
                4,
                [0x8b, 0xe9, 0xff, 0xff],
            );
            fill_transparent_rounded_rect(
                rgba,
                width,
                height,
                Rect {
                    x: x + 10,
                    y: y + 29,
                    width: 44,
                    height: 28,
                },
                4,
                [0xa8, 0xe8, 0xf8, 0xff],
            );
            fill_offset_transparent_rect(
                rgba,
                width,
                height,
                (x, y),
                Rect {
                    x: 14,
                    y: 31,
                    width: 36,
                    height: 3,
                },
                [0xc5, 0xf6, 0xff, 0xc8],
            );
            fill_transparent_rounded_rect(
                rgba,
                width,
                height,
                Rect {
                    x: x + 16,
                    y: y + 39,
                    width: 32,
                    height: 10,
                },
                3,
                [0x18, 0x84, 0xb8, 0xff],
            );
            fill_offset_transparent_rect(
                rgba,
                width,
                height,
                (x, y),
                Rect {
                    x: 19,
                    y: 41,
                    width: 26,
                    height: 2,
                },
                [0xda, 0xf8, 0xff, 0xb8],
            );
            10
        }
        "settings" => {
            for (x1, y1, x2, y2) in [
                (32, 15, 32, 22),
                (32, 48, 32, 55),
                (15, 35, 22, 35),
                (42, 35, 49, 35),
                (20, 23, 25, 28),
                (39, 42, 44, 47),
                (20, 47, 25, 42),
                (39, 28, 44, 23),
            ] {
                draw_transparent_line(
                    rgba,
                    width,
                    height,
                    (x + x1, y + y1),
                    (x + x2, y + y2),
                    5,
                    [0xd2, 0xf5, 0xff, 0xff],
                );
            }
            fill_transparent_circle(
                rgba,
                width,
                height,
                x + 32,
                y + 35,
                17,
                [0xb5, 0xdf, 0xe8, 0xff],
            );
            fill_transparent_circle(
                rgba,
                width,
                height,
                x + 32,
                y + 35,
                12,
                [0x44, 0x91, 0xaa, 0xff],
            );
            fill_transparent_circle(
                rgba,
                width,
                height,
                x + 32,
                y + 35,
                6,
                [0x03, 0x35, 0x54, 0xff],
            );
            fill_transparent_rounded_rect(
                rgba,
                width,
                height,
                Rect {
                    x: x + 23,
                    y: y + 24,
                    width: 15,
                    height: 3,
                },
                1,
                [0xf2, 0xfd, 0xff, 0xb8],
            );
            16
        }
        "trash" => {
            fill_transparent_rounded_rect(
                rgba,
                width,
                height,
                Rect {
                    x: x + 18,
                    y: y + 23,
                    width: 28,
                    height: 31,
                },
                5,
                [0xc8, 0xe8, 0xed, 0xff],
            );
            fill_offset_transparent_rect(
                rgba,
                width,
                height,
                (x, y),
                Rect {
                    x: 16,
                    y: 20,
                    width: 32,
                    height: 5,
                },
                [0xf0, 0xfd, 0xff, 0xff],
            );
            fill_transparent_rounded_rect(
                rgba,
                width,
                height,
                Rect {
                    x: x + 24,
                    y: y + 16,
                    width: 16,
                    height: 6,
                },
                3,
                [0x9b, 0xcb, 0xd5, 0xff],
            );
            for dx in [24, 31, 38] {
                fill_offset_transparent_rect(
                    rgba,
                    width,
                    height,
                    (x, y),
                    Rect {
                        x: dx,
                        y: 29,
                        width: 2,
                        height: 19,
                    },
                    [0x54, 0x91, 0xa1, 0xff],
                );
            }
            fill_offset_transparent_rect(
                rgba,
                width,
                height,
                (x, y),
                Rect {
                    x: 21,
                    y: 26,
                    width: 22,
                    height: 3,
                },
                [0xf7, 0xff, 0xff, 0xb8],
            );
            11
        }
        _ => 0,
    }
}

fn fill_offset_transparent_rect(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    origin: (u32, u32),
    rect: Rect,
    color: [u8; 4],
) {
    fill_transparent_rect(
        buffer,
        width,
        height,
        Rect {
            x: origin.0 + rect.x,
            y: origin.1 + rect.y,
            width: rect.width,
            height: rect.height,
        },
        color,
    );
}

pub fn render_system_overview_rgba(
    width: u32,
    height: u32,
    model: &SystemOverviewModel,
) -> SystemOverviewOverlay {
    render_system_overview_rgba_with_theme(width, height, model, AquaTheme::LightWhite)
}

pub fn render_system_overview_rgba_with_theme(
    width: u32,
    height: u32,
    model: &SystemOverviewModel,
    theme: AquaTheme,
) -> SystemOverviewOverlay {
    let mut overlay = render_system_overview_rgba_base(width, height, model);
    apply_shell_palette(&mut overlay.rgba, theme);
    overlay
}

fn render_system_overview_rgba_base(
    width: u32,
    height: u32,
    model: &SystemOverviewModel,
) -> SystemOverviewOverlay {
    let mut rgba = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    if width == 0 || height == 0 {
        return SystemOverviewOverlay {
            width,
            height,
            rgba,
            memory_used_percent: model.memory_used_percent(),
            primitive_count: 0,
        };
    }
    let high_resolution = width >= 480;
    let scale = if high_resolution { 2 } else { 1 };
    let padding = if high_resolution { 22 } else { 12 };
    let row = if high_resolution { 38 } else { 25 };
    fill_transparent_rect(
        &mut rgba,
        width,
        height,
        Rect {
            x: 0,
            y: 0,
            width,
            height,
        },
        [0x02, 0x1c, 0x32, 0x70],
    );
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (padding, padding),
        "AQUA LINUX",
        [0xf1, 0xfb, 0xff, 0xff],
        scale,
    );
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (
            width.saturating_sub(if high_resolution { 150 } else { 96 }),
            padding,
        ),
        &model.clock_utc,
        [0x72, 0xe3, 0xf5, 0xff],
        scale,
    );
    let labels = ["HOST", "KERNEL", "UPTIME", "LOAD", "MEMORY"];
    let values = [
        model.hostname.clone(),
        model.kernel.clone(),
        model.uptime_label(),
        format!(
            "{}.{:02}",
            model.load_average_x100 / 100,
            model.load_average_x100 % 100
        ),
        format!("{}%", model.memory_used_percent()),
    ];
    let metadata_height = if high_resolution { 28 } else { 18 };
    let label_width = if high_resolution { 104 } else { 64 };
    for (index, value) in values.iter().enumerate() {
        let metadata = MetadataRow::new(
            Rect {
                x: padding,
                y: padding
                    .saturating_add(row * (index as u32 + 1))
                    .saturating_sub(if high_resolution { 5 } else { 3 }),
                width: width.saturating_sub(padding * 2),
                height: metadata_height,
            },
            labels[index],
            value,
        )
        .with_columns(label_width, if high_resolution { 12 } else { 8 })
        .with_emphasis(index == values.len() - 1);
        draw_metadata_row(
            &mut rgba,
            width,
            height,
            metadata,
            MetadataRowStyle {
                label_color: [0xd1, 0xe8, 0xef, 0xff],
                value_color: if metadata.emphasized {
                    [0x9b, 0xeb, 0xf7, 0xff]
                } else {
                    [0xd1, 0xe8, 0xef, 0xff]
                },
                role: if high_resolution {
                    TextRole::Caption
                } else {
                    TextRole::Body
                },
                scale: if high_resolution {
                    OutputScale::Two
                } else {
                    OutputScale::One
                },
            },
        );
    }
    let track = Rect {
        x: padding,
        y: height.saturating_sub(if high_resolution { 28 } else { 18 }),
        width: width.saturating_sub(padding * 2),
        height: if high_resolution { 10 } else { 6 },
    };
    fill_transparent_rect(&mut rgba, width, height, track, [0x03, 0x18, 0x29, 0xb8]);
    fill_transparent_rect(
        &mut rgba,
        width,
        height,
        Rect {
            width: track.width * u32::from(model.memory_used_percent()) / 100,
            ..track
        },
        [0x30, 0xcf, 0xe9, 0xe8],
    );
    SystemOverviewOverlay {
        width,
        height,
        rgba,
        memory_used_percent: model.memory_used_percent(),
        primitive_count: 15,
    }
}

pub fn render_session_menu_overlay_rgba(
    width: u32,
    height: u32,
    menu: &SessionMenuState,
) -> SessionMenuOverlay {
    render_session_menu_overlay_rgba_with_theme(width, height, menu, AquaTheme::LightWhite)
}

pub fn render_session_menu_overlay_rgba_with_theme(
    width: u32,
    height: u32,
    menu: &SessionMenuState,
    theme: AquaTheme,
) -> SessionMenuOverlay {
    let mut overlay = render_session_menu_overlay_rgba_base(width, height, menu);
    apply_shell_palette(&mut overlay.rgba, theme);
    overlay
}

fn render_session_menu_overlay_rgba_base(
    width: u32,
    height: u32,
    menu: &SessionMenuState,
) -> SessionMenuOverlay {
    let mut rgba = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    if !menu.is_open() || width == 0 || height == 0 {
        return SessionMenuOverlay {
            width,
            height,
            rgba,
            selected_action: menu.selected_action().id(),
            confirmation_visible: false,
            primitive_count: 0,
        };
    }

    let high_resolution = width >= 480 || height >= 280;
    let text_scale = if high_resolution { 2 } else { 1 };
    let outer_padding = if high_resolution { 22 } else { 12 };
    let header_y = if high_resolution { 18 } else { 12 };
    let icon_scale = if high_resolution { 2 } else { 1 };
    let menu_layout = menu.menu_layout(width, height);

    fill_transparent_rect(
        &mut rgba,
        width,
        height,
        Rect {
            x: 0,
            y: 0,
            width,
            height,
        },
        [0x02, 0x18, 0x2b, 0x30],
    );
    for edge in [
        Rect {
            x: 1,
            y: 1,
            width: width.saturating_sub(2),
            height: 1,
        },
        Rect {
            x: 1,
            y: height.saturating_sub(2),
            width: width.saturating_sub(2),
            height: 1,
        },
        Rect {
            x: 1,
            y: 1,
            width: 1,
            height: height.saturating_sub(2),
        },
        Rect {
            x: width.saturating_sub(2),
            y: 1,
            width: 1,
            height: height.saturating_sub(2),
        },
    ] {
        fill_transparent_rect(&mut rgba, width, height, edge, [0x86, 0xdd, 0xf3, 0x78]);
    }

    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (outer_padding, header_y),
        "Aqua Session",
        [0xee, 0xfb, 0xff, 0xff],
        text_scale,
    );
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        (
            width.saturating_sub(if high_resolution { 62 } else { 40 }),
            header_y,
        ),
        "F10",
        [0x9f, 0xd7, 0xe8, 0xff],
        text_scale,
    );
    let separator_y = if high_resolution { 50 } else { 32 };
    fill_transparent_rect(
        &mut rgba,
        width,
        height,
        Rect {
            x: outer_padding,
            y: separator_y,
            width: width.saturating_sub(outer_padding * 2),
            height: 1,
        },
        [0x8a, 0xd9, 0xec, 0x68],
    );
    let mut primitives = 7;
    let labels = ["Log Out", "Restart", "Shut Down", "Recovery"];
    for (index, action) in SessionAction::ALL.iter().copied().enumerate() {
        let row = menu_layout.item_rect(index);
        if action == menu.selected_action() {
            fill_transparent_rect(&mut rgba, width, height, row, [0x72, 0xda, 0xf2, 0x82]);
            fill_transparent_rect(
                &mut rgba,
                width,
                height,
                Rect {
                    x: row.x + 1,
                    y: row.y + 1,
                    width: row.width.saturating_sub(2),
                    height: row.height.saturating_sub(2),
                },
                if menu.confirmation() == Some(action) {
                    [0x0a, 0x70, 0xa0, 0xde]
                } else {
                    [0x16, 0x86, 0xb7, 0xb8]
                },
            );
            fill_transparent_rect(
                &mut rgba,
                width,
                height,
                Rect {
                    x: row.x + 1,
                    y: row.y + 1,
                    width: row.width.saturating_sub(2),
                    height: 1,
                },
                [0xc7, 0xf7, 0xff, 0xc8],
            );
            fill_transparent_rect(
                &mut rgba,
                width,
                height,
                Rect {
                    x: row.x + 1,
                    y: row.y + 2,
                    width: if high_resolution { 4 } else { 3 },
                    height: row.height.saturating_sub(4),
                },
                [0x5d, 0xe6, 0xff, 0xf0],
            );
            primitives += 4;
        } else {
            fill_transparent_rect(
                &mut rgba,
                width,
                height,
                Rect {
                    x: row.x,
                    y: row.y + row.height.saturating_sub(1),
                    width: row.width,
                    height: 1,
                },
                [0x61, 0xb4, 0xcc, 0x24],
            );
            primitives += 1;
        }
        draw_session_action_icon(
            &mut rgba,
            width,
            height,
            row.x + if high_resolution { 12 } else { 10 },
            row.y + if high_resolution { 5 } else { 7 },
            action,
            icon_scale,
        );
        draw_bitmap_text(
            &mut rgba,
            (width, height),
            (row.x + if high_resolution { 56 } else { 40 }, row.y + 7),
            labels[index],
            if action == menu.selected_action() {
                [0xf5, 0xfd, 0xff, 0xff]
            } else {
                [0xd7, 0xeb, 0xf1, 0xff]
            },
            text_scale,
        );
        primitives += 2;
    }

    let confirmation_visible = menu.confirmation().is_some();
    let footer_y = height.saturating_sub(if high_resolution { 30 } else { 24 });
    let confirmation_dialog = menu.confirmation_dialog(width, height);
    if let Some(dialog) = confirmation_dialog {
        fill_transparent_rect(
            &mut rgba,
            width,
            height,
            dialog.rect,
            [0x08, 0x5b, 0x83, 0xb8],
        );
        primitives += 1;
    }
    draw_bitmap_text(
        &mut rgba,
        (width, height),
        confirmation_dialog.map_or((outer_padding, footer_y), |dialog| {
            let title = dialog.slots().title;
            (title.x, title.y)
        }),
        if confirmation_visible {
            "Enter again to confirm"
        } else {
            "Enter to select   Esc to close"
        },
        if confirmation_visible {
            [0x8e, 0xeb, 0xff, 0xff]
        } else {
            [0xa8, 0xcf, 0xdc, 0xff]
        },
        1,
    );
    primitives += 1;

    SessionMenuOverlay {
        width,
        height,
        rgba,
        selected_action: menu.selected_action().id(),
        confirmation_visible,
        primitive_count: primitives,
    }
}

fn fill_transparent_rect(buffer: &mut [u8], width: u32, height: u32, rect: Rect, color: [u8; 4]) {
    let max_x = rect.x.saturating_add(rect.width).min(width);
    let max_y = rect.y.saturating_add(rect.height).min(height);
    for y in rect.y..max_y {
        for x in rect.x..max_x {
            let offset = ((y * width + x) * 4) as usize;
            buffer[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn fill_transparent_rounded_rect(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rect: Rect,
    radius: u32,
    color: [u8; 4],
) {
    let max_x = rect.x.saturating_add(rect.width).min(width);
    let max_y = rect.y.saturating_add(rect.height).min(height);
    let radius = radius.min(rect.width / 2).min(rect.height / 2);
    let radius_squared = i64::from(radius) * i64::from(radius);

    for y in rect.y..max_y {
        for x in rect.x..max_x {
            let local_x = x - rect.x;
            let local_y = y - rect.y;
            let corner_x = if local_x < radius {
                radius - local_x
            } else if local_x >= rect.width.saturating_sub(radius) {
                local_x - rect.width.saturating_sub(radius).saturating_sub(1)
            } else {
                0
            };
            let corner_y = if local_y < radius {
                radius - local_y
            } else if local_y >= rect.height.saturating_sub(radius) {
                local_y - rect.height.saturating_sub(radius).saturating_sub(1)
            } else {
                0
            };
            if corner_x > 0
                && corner_y > 0
                && i64::from(corner_x) * i64::from(corner_x)
                    + i64::from(corner_y) * i64::from(corner_y)
                    > radius_squared
            {
                continue;
            }
            let offset = ((y * width + x) * 4) as usize;
            buffer[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn fill_transparent_circle(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    center_x: u32,
    center_y: u32,
    radius: u32,
    color: [u8; 4],
) {
    let start_x = center_x.saturating_sub(radius);
    let start_y = center_y.saturating_sub(radius);
    let end_x = center_x.saturating_add(radius).min(width.saturating_sub(1));
    let end_y = center_y
        .saturating_add(radius)
        .min(height.saturating_sub(1));
    let radius_squared = i64::from(radius) * i64::from(radius);
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let dx = i64::from(x) - i64::from(center_x);
            let dy = i64::from(y) - i64::from(center_y);
            if dx * dx + dy * dy > radius_squared {
                continue;
            }
            let offset = ((y * width + x) * 4) as usize;
            buffer[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn draw_transparent_line(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    from: (u32, u32),
    to: (u32, u32),
    thickness: u32,
    color: [u8; 4],
) {
    let (mut x, mut y) = (from.0 as i32, from.1 as i32);
    let (target_x, target_y) = (to.0 as i32, to.1 as i32);
    let dx = (target_x - x).abs();
    let sx = if x < target_x { 1 } else { -1 };
    let dy = -(target_y - y).abs();
    let sy = if y < target_y { 1 } else { -1 };
    let mut error = dx + dy;
    let radius = thickness / 2;

    loop {
        if x >= 0 && y >= 0 {
            fill_transparent_circle(buffer, width, height, x as u32, y as u32, radius, color);
        }
        if x == target_x && y == target_y {
            break;
        }
        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn draw_session_action_icon(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    action: SessionAction,
    scale: u32,
) {
    let bars: &[(u32, u32, u32, u32)] = match action {
        SessionAction::Logout => &[(0, 7, 13, 2), (9, 3, 2, 10), (12, 5, 5, 2), (12, 9, 5, 2)],
        SessionAction::Restart => &[(2, 2, 12, 2), (2, 4, 2, 10), (4, 13, 10, 2), (12, 9, 2, 4)],
        SessionAction::Shutdown => &[(7, 0, 2, 8), (2, 5, 2, 8), (12, 5, 2, 8), (4, 13, 8, 2)],
        SessionAction::Recovery => &[(1, 2, 14, 2), (1, 4, 2, 11), (13, 4, 2, 11), (5, 8, 6, 2)],
    };
    for (dx, dy, bar_width, bar_height) in bars {
        fill_transparent_rect(
            buffer,
            width,
            height,
            Rect {
                x: x + dx * scale,
                y: y + dy * scale,
                width: *bar_width * scale,
                height: *bar_height * scale,
            },
            [0x9b, 0xe8, 0xff, 0xff],
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesWindowProbe {
    pub rendered: bool,
    pub sidebar_item_count: usize,
    pub entry_count: usize,
    pub selected_sidebar: usize,
    pub empty_state_rendered: bool,
    pub primitive_count: usize,
    pub checksum: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindowProbe {
    pub rendered: bool,
    pub category_count: usize,
    pub selected_category: usize,
    pub reduced_motion: bool,
    pub desktop_icons: bool,
    pub key_repeat: bool,
    pub audio_available: bool,
    pub audio_controls_enabled: bool,
    pub audio_backend_applied: bool,
    pub audio_control_status: &'static str,
    pub audio_desired_volume_percent: u8,
    pub audio_volume_percent: u8,
    pub audio_muted: bool,
    pub network_interface_count: usize,
    pub network_status_available: bool,
    pub wifi_control_available: bool,
    pub wifi_controls_enabled: bool,
    pub wifi_connected: bool,
    pub wifi_credential_saved: bool,
    pub wifi_scan_result_count: usize,
    pub wifi_credential_entry: bool,
    pub wifi_connect_attempts_remaining: u8,
    pub primitive_count: usize,
    pub checksum: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertiesWindowProbe {
    pub rendered: bool,
    pub target: &'static str,
    pub item_count: Option<usize>,
    pub refresh_generation: u32,
    pub primitive_count: usize,
    pub checksum: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalWindowProbe {
    pub rendered: bool,
    pub rows: u16,
    pub cols: u16,
    pub visible_line_count: usize,
    pub primitive_count: usize,
    pub checksum: u64,
}

pub fn render_terminal_window_rgba(
    width: u32,
    height: u32,
    view: &TerminalView,
) -> (Vec<u8>, TerminalWindowProbe) {
    render_terminal_window_rgba_with_theme(width, height, view, AquaTheme::LightWhite)
}

pub fn render_terminal_window_rgba_with_theme(
    width: u32,
    height: u32,
    view: &TerminalView,
    theme: AquaTheme,
) -> (Vec<u8>, TerminalWindowProbe) {
    let mut buffer = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    if width == 0 || height == 0 {
        return (
            buffer,
            TerminalWindowProbe {
                rendered: false,
                rows: view.rows,
                cols: view.cols,
                visible_line_count: 0,
                primitive_count: 0,
                checksum: 0,
            },
        );
    }

    let canvas = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let palette = window_chrome_palette(theme);
    let mut primitives = draw_window_frame(
        &mut buffer,
        width,
        height,
        WindowFrame::new(canvas, "Terminal", 48),
        palette,
    );

    let scrim = Rect {
        x: 10,
        y: 58,
        width: width.saturating_sub(20),
        height: height.saturating_sub(68),
    };
    fill_rect(
        &mut buffer,
        width,
        height,
        scrim,
        [0x00, 0x0d, 0x17, 0xff],
        232,
    );
    primitives += 1;

    let visible_rows = ((scrim.height.saturating_sub(20)) / 18) as usize;
    let visible_line_count = view.lines.len().min(visible_rows);
    for (index, line) in view.lines.iter().take(visible_rows).enumerate() {
        let bounded = line.chars().take(view.cols as usize).collect::<String>();
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (22, 70 + index as u32 * 18),
            &bounded,
            [0xd9, 0xf6, 0xee, 0xff],
            1,
        );
    }

    if usize::from(view.cursor_row) < visible_rows {
        fill_rect(
            &mut buffer,
            width,
            height,
            Rect {
                x: 22 + u32::from(view.cursor_col) * 8,
                y: 72 + u32::from(view.cursor_row) * 18,
                width: 8,
                height: 14,
            },
            [0x61, 0xe7, 0xff, 0xff],
            168,
        );
        primitives += 1;
    }

    let checksum = checksum_bytes(&buffer);
    (
        buffer,
        TerminalWindowProbe {
            rendered: true,
            rows: view.rows,
            cols: view.cols,
            visible_line_count,
            primitive_count: primitives,
            checksum,
        },
    )
}

pub fn render_properties_window_rgba(
    width: u32,
    height: u32,
    model: &DesktopPropertiesModel,
) -> (Vec<u8>, PropertiesWindowProbe) {
    render_properties_window_rgba_with_theme(width, height, model, AquaTheme::LightWhite)
}

pub fn render_properties_window_rgba_with_theme(
    width: u32,
    height: u32,
    model: &DesktopPropertiesModel,
    theme: AquaTheme,
) -> (Vec<u8>, PropertiesWindowProbe) {
    let mut buffer = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    if width == 0 || height == 0 {
        return (
            buffer,
            PropertiesWindowProbe {
                rendered: false,
                target: model.icon_id,
                item_count: model.item_count,
                refresh_generation: model.refresh_generation,
                primitive_count: 0,
                checksum: 0,
            },
        );
    }

    let canvas = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let palette = window_chrome_palette(theme);
    let mut primitives = draw_window_frame(
        &mut buffer,
        width,
        height,
        WindowFrame::new(canvas, &model.title, 52),
        palette,
    );

    let badge = Rect {
        x: 24,
        y: 78,
        width: 86,
        height: 86,
    };
    fill_rect(&mut buffer, width, height, badge, palette.accent_soft, 255);
    let glyph = match model.icon_id {
        "files" => "DIR",
        "settings" => "SET",
        "trash" => "BIN",
        _ => "ITEM",
    };
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (42, 112),
        glyph,
        palette.accent,
        2,
    );
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (132, 84),
        model.name,
        palette.text,
        2,
    );
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (132, 116),
        model.kind,
        palette.secondary_text,
        1,
    );
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (132, 140),
        model.status,
        palette.accent,
        1,
    );
    primitives += 2;

    let details = model.details_section_group(width, height);
    primitives += draw_section_group(&mut buffer, width, height, details, theme);
    let footer = details.footer_rect();
    let location = model.details_metadata_row(width, height, 0, "Location", &model.location);
    primitives += draw_metadata_row(
        &mut buffer,
        width,
        height,
        location,
        MetadataRowStyle {
            label_color: palette.secondary_text,
            value_color: palette.text,
            role: TextRole::Body,
            scale: OutputScale::One,
        },
    );
    if let Some(item_count) = model.item_count {
        let suffix = if model.enumeration_capped { "+" } else { "" };
        let value = format!("{item_count}{suffix}");
        let items = model.details_metadata_row(width, height, 1, "Items", &value);
        primitives += draw_metadata_row(
            &mut buffer,
            width,
            height,
            items,
            MetadataRowStyle {
                label_color: palette.secondary_text,
                value_color: palette.text,
                role: TextRole::Body,
                scale: OutputScale::One,
            },
        );
    }
    let action = details.footer_trailing_rect(138, 30);
    fill_rect(&mut buffer, width, height, action, palette.accent, 255);
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (action.x + 12, action.y + 10),
        match model.primary_action() {
            aqua_shell::DesktopPropertiesAction::RefreshContents => "Refresh (F5)",
            aqua_shell::DesktopPropertiesAction::VerifyApplication => "Verify (F5)",
        },
        [0xff, 0xff, 0xff, 0xff],
        1,
    );
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (footer.x, footer.y + 10),
        &format!("Updated {}", model.refresh_generation),
        palette.secondary_text,
        1,
    );
    primitives += 3;

    let checksum = checksum_bytes(&buffer);
    (
        buffer,
        PropertiesWindowProbe {
            rendered: true,
            target: model.icon_id,
            item_count: model.item_count,
            refresh_generation: model.refresh_generation,
            primitive_count: primitives,
            checksum,
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerWindowProbe {
    pub rendered: bool,
    pub layout_valid: bool,
    pub step: InstallerStep,
    pub focus: InstallerFocusTarget,
    pub step_count: usize,
    pub logo_rendered: bool,
    pub progress_percent: Option<u8>,
    pub primitive_count: usize,
    pub checksum: u64,
}

impl InstallerWindowProbe {
    pub fn is_ready(&self) -> bool {
        self.rendered
            && self.layout_valid
            && self.step_count == InstallerStep::ALL.len()
            && (self.step != InstallerStep::Welcome || self.logo_rendered)
            && self.primitive_count >= 40
            && self.checksum != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InstallerRenderOptions<'a> {
    pub progress: Option<&'a InstallProgressEvent>,
    pub theme: AquaTheme,
}

impl Default for InstallerRenderOptions<'_> {
    fn default() -> Self {
        Self {
            progress: None,
            theme: AquaTheme::LightWhite,
        }
    }
}

pub fn render_installer_window_rgba(
    width: u32,
    height: u32,
    model: &InstallerModel,
    ui: &InstallerUiState,
    forms: &InstallerFormState,
    progress: Option<&InstallProgressEvent>,
    logo: InstallerImageSource<'_>,
) -> Result<(Vec<u8>, InstallerWindowProbe), String> {
    render_installer_window_rgba_with_theme(
        width,
        height,
        model,
        ui,
        forms,
        logo,
        InstallerRenderOptions {
            progress,
            ..InstallerRenderOptions::default()
        },
    )
}

pub fn render_installer_window_rgba_with_theme(
    width: u32,
    height: u32,
    model: &InstallerModel,
    ui: &InstallerUiState,
    forms: &InstallerFormState,
    logo: InstallerImageSource<'_>,
    options: InstallerRenderOptions<'_>,
) -> Result<(Vec<u8>, InstallerWindowProbe), String> {
    let InstallerRenderOptions { progress, theme } = options;
    if model.step() != ui.step() {
        return Err("installer UI step does not match installer model".to_string());
    }
    let viewport = aqua_scene::Viewport::new(width, height);
    let layout =
        InstallerWindowLayout::for_viewport(viewport).map_err(|error| error.to_string())?;
    let mut buffer = vec![0_u8; width as usize * height as usize * 4];
    let canvas = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let palette = window_chrome_palette(theme);
    let canvas_color = if theme == AquaTheme::LightWhite {
        [0x9a, 0xb9, 0xd9, 0xff]
    } else {
        palette.sidebar
    };
    fill_rect(&mut buffer, width, height, canvas, canvas_color, 255);
    let mut primitives = 1;

    let shadow = Rect {
        x: layout.window.x + 7,
        y: layout.window.y + 9,
        width: layout.window.width,
        height: layout.window.height,
    };
    fill_rounded_rect(
        &mut buffer,
        width,
        height,
        shadow,
        8,
        if theme == AquaTheme::LightWhite {
            [0x1d, 0x45, 0x72, 0xff]
        } else {
            palette.border
        },
        72,
    );
    fill_rounded_rect(
        &mut buffer,
        width,
        height,
        layout.window,
        8,
        palette.surface,
        255,
    );
    primitives += 2 + draw_system_surface_primitives(&mut buffer, width, height, layout.window);

    primitives += draw_bright_window_titlebar(
        &mut buffer,
        width,
        height,
        layout.titlebar,
        "Aqua Linux Kurulumu",
        palette,
    );

    fill_rect(
        &mut buffer,
        width,
        height,
        layout.step_rail,
        palette.sidebar,
        255,
    );
    fill_rect(
        &mut buffer,
        width,
        height,
        layout.content,
        palette.surface,
        255,
    );
    fill_rect(
        &mut buffer,
        width,
        height,
        layout.footer,
        palette.toolbar,
        236,
    );
    primitives += 3;

    let rail_padding = if width >= 1200 { 24 } else { 12 };
    let rail_top = layout.step_rail.y + 16;
    let row_height = (layout.step_rail.height.saturating_sub(24) / 9).min(54);
    let step_navigation = SidebarNavigation::new(
        layout.step_rail,
        "Installation steps",
        Rect {
            x: layout.step_rail.x + rail_padding,
            y: rail_top,
            width: layout.step_rail.width - rail_padding * 2,
            height: row_height.saturating_sub(6),
        },
        row_height,
    );
    for (index, step) in InstallerStep::ALL.iter().copied().enumerate() {
        let row_rect = step_navigation.row_rect(index);
        let selected = step == model.step();
        let row = ListRow::new(row_rect, step.label_tr(), ListRowRole::Step)
            .with_slots(30, 8)
            .with_state(if selected {
                ComponentState::Selected
            } else {
                ComponentState::Idle
            });
        primitives += draw_list_row(&mut buffer, width, height, row, theme, OutputScale::One);
        let marker_color = if step == model.step() {
            palette.accent
        } else {
            palette.secondary_text
        };
        fill_transparent_circle(
            &mut buffer,
            width,
            height,
            row_rect.x + 17,
            row_rect.y + row_rect.height / 2,
            7,
            marker_color,
        );
        if selected {
            fill_transparent_circle(
                &mut buffer,
                width,
                height,
                row_rect.x + 17,
                row_rect.y + row_rect.height / 2,
                3,
                palette.surface,
            );
        }
        primitives += if selected { 2 } else { 1 };
    }

    let logo_rendered = matches!(
        model.step(),
        InstallerStep::Welcome | InstallerStep::Completed
    );
    primitives += draw_installer_content(
        &mut buffer,
        width,
        height,
        InstallerContentContext {
            layout: &layout,
            model,
            forms,
            progress,
            logo,
            palette,
            theme,
        },
    );
    primitives += draw_installer_footer(&mut buffer, width, height, &layout, ui, theme);
    primitives += draw_installer_focus(&mut buffer, width, height, &layout, ui.focus(), palette);
    let checksum = checksum_bytes(&buffer);
    Ok((
        buffer,
        InstallerWindowProbe {
            rendered: true,
            layout_valid: layout.fits_viewport() && layout.regions_are_separated(),
            step: model.step(),
            focus: ui.focus(),
            step_count: InstallerStep::ALL.len(),
            logo_rendered,
            progress_percent: progress.map(InstallProgressEvent::percent),
            primitive_count: primitives,
            checksum,
        },
    ))
}

pub fn export_installer_window_png(
    width: u32,
    height: u32,
    model: &InstallerModel,
    ui: &InstallerUiState,
    forms: &InstallerFormState,
    progress: Option<&InstallProgressEvent>,
    logo: InstallerImageSource<'_>,
) -> Result<(Vec<u8>, InstallerWindowProbe), String> {
    export_installer_window_png_with_theme(
        width,
        height,
        model,
        ui,
        forms,
        logo,
        InstallerRenderOptions {
            progress,
            ..InstallerRenderOptions::default()
        },
    )
}

pub fn export_installer_window_png_with_theme(
    width: u32,
    height: u32,
    model: &InstallerModel,
    ui: &InstallerUiState,
    forms: &InstallerFormState,
    logo: InstallerImageSource<'_>,
    options: InstallerRenderOptions<'_>,
) -> Result<(Vec<u8>, InstallerWindowProbe), String> {
    let (rgba, probe) =
        render_installer_window_rgba_with_theme(width, height, model, ui, forms, logo, options)?;
    Ok((encode_png_rgba(width, height, &rgba), probe))
}

struct InstallerContentContext<'a> {
    layout: &'a InstallerWindowLayout,
    model: &'a InstallerModel,
    forms: &'a InstallerFormState,
    progress: Option<&'a InstallProgressEvent>,
    logo: InstallerImageSource<'a>,
    palette: WindowChromePalette,
    theme: AquaTheme,
}

fn draw_installer_content(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    context: InstallerContentContext<'_>,
) -> usize {
    let InstallerContentContext {
        layout,
        model,
        forms,
        progress,
        logo,
        palette,
        theme,
    } = context;
    let padding = layout.content_padding();
    let text_x = layout.content.x + padding;
    let heading_y = layout.content_heading_y();
    let logo_size = if width >= 1200 { 270 } else { 180 };
    let logo_rect = Rect {
        x: layout.content.right() - padding - logo_size,
        y: layout.content.y + 54,
        width: logo_size,
        height: logo_size,
    };
    let mut themed_logo = Vec::new();
    let logo_rgba = if palette.text[0] > 0x80 {
        themed_logo.extend_from_slice(logo.rgba);
        for pixel in themed_logo.chunks_exact_mut(4) {
            if pixel[3] > 0 && pixel[0].max(pixel[1]).max(pixel[2]) < 0x80 {
                pixel[..3].copy_from_slice(&palette.text[..3]);
            }
        }
        themed_logo.as_slice()
    } else {
        logo.rgba
    };
    let source = RgbaImageSource {
        width: logo.width,
        height: logo.height,
        rgba: logo_rgba,
    };
    let logo_visible = matches!(
        model.step(),
        InstallerStep::Welcome | InstallerStep::Completed
    );
    if logo_visible {
        fill_rgba_image_rect(buffer, width, height, logo_rect, source, 255);
    }
    let mut primitives = usize::from(logo_visible);

    match model.step() {
        InstallerStep::Welcome => {
            draw_bitmap_text(
                buffer,
                (width, height),
                (text_x, heading_y),
                "Aqua Linux'a",
                palette.text,
                2,
            );
            draw_bitmap_text(
                buffer,
                (width, height),
                (text_x, heading_y + 42),
                "Hoş Geldiniz",
                palette.accent,
                2,
            );
            draw_bitmap_text(
                buffer,
                (width, height),
                (text_x, heading_y + 112),
                "Aqua Linux; sadelik, performans ve",
                palette.secondary_text,
                1,
            );
            draw_bitmap_text(
                buffer,
                (width, height),
                (text_x, heading_y + 136),
                "özgürlük için tasarlandı.",
                palette.secondary_text,
                1,
            );
            draw_bitmap_text(
                buffer,
                (width, height),
                (text_x, heading_y + 184),
                "Kuruluma devam etmek için İleri'yi seçin.",
                palette.secondary_text,
                1,
            );
            primitives += 5;

            let feature_y = layout.content.bottom().saturating_sub(86);
            let available = layout.content.width.saturating_sub(padding * 2 + 24);
            let feature_width = available / 3;
            for (index, (title, detail)) in [
                ("Hafif ve Hızlı", "Minimum kaynak"),
                ("Güvenli ve Kararlı", "Güncel sistem"),
                ("Özgür ve Açık", "Topluluk gücü"),
            ]
            .into_iter()
            .enumerate()
            {
                let x = text_x + index as u32 * (feature_width + 12);
                fill_rounded_rect(
                    buffer,
                    width,
                    height,
                    Rect {
                        x,
                        y: feature_y,
                        width: feature_width,
                        height: 58,
                    },
                    8,
                    palette.field,
                    210,
                );
                draw_bitmap_text(
                    buffer,
                    (width, height),
                    (x + 12, feature_y + 9),
                    title,
                    palette.text,
                    1,
                );
                draw_bitmap_text(
                    buffer,
                    (width, height),
                    (x + 12, feature_y + 31),
                    detail,
                    palette.secondary_text,
                    1,
                );
                primitives += 3;
            }
        }
        InstallerStep::Language => {
            primitives += draw_installer_choice_form(
                buffer,
                width,
                height,
                InstallerChoiceForm {
                    layout,
                    x: text_x,
                    y: heading_y,
                    step: model.step(),
                    options: &LANGUAGE_OPTIONS,
                    selected_index: forms.language_index(),
                    applied_value: model.locale(),
                    palette,
                },
            );
        }
        InstallerStep::Keyboard => {
            primitives += draw_installer_choice_form(
                buffer,
                width,
                height,
                InstallerChoiceForm {
                    layout,
                    x: text_x,
                    y: heading_y,
                    step: model.step(),
                    options: &KEYBOARD_OPTIONS,
                    selected_index: forms.keyboard_index(),
                    applied_value: model.keyboard_layout(),
                    palette,
                },
            );
        }
        InstallerStep::Partitions => {
            primitives += draw_installer_disk_form(
                buffer,
                width,
                height,
                InstallerDiskForm {
                    layout,
                    x: text_x,
                    y: heading_y,
                    forms,
                    applied_device: model.target().map(|target| target.disk.device()),
                    palette,
                },
            );
        }
        InstallerStep::TimeZone => {
            primitives += draw_installer_choice_form(
                buffer,
                width,
                height,
                InstallerChoiceForm {
                    layout,
                    x: text_x,
                    y: heading_y,
                    step: model.step(),
                    options: &TIMEZONE_OPTIONS,
                    selected_index: forms.timezone_index(),
                    applied_value: model.timezone(),
                    palette,
                },
            );
        }
        InstallerStep::UserInformation => {
            primitives += draw_installer_user_form(
                buffer,
                width,
                height,
                InstallerUserForm {
                    layout,
                    x: text_x,
                    y: heading_y,
                    forms,
                    applied: model.user().is_some(),
                    palette,
                },
            );
        }
        InstallerStep::Summary => {
            primitives += draw_installer_summary(
                buffer,
                width,
                height,
                InstallerSummaryView {
                    layout,
                    x: text_x,
                    y: heading_y,
                    model,
                    forms,
                    palette,
                    theme,
                },
            );
        }
        InstallerStep::Installation => {
            draw_installer_step_heading(
                buffer,
                width,
                height,
                text_x,
                heading_y,
                model.step(),
                palette,
            );
            let percent = progress.map(InstallProgressEvent::percent).unwrap_or(0);
            let phase_label = progress
                .map(|progress| installer_progress_phase_label(progress.phase()))
                .unwrap_or("Kurulum hazırlanıyor");
            draw_bitmap_text(
                buffer,
                (width, height),
                (text_x, heading_y + 74),
                phase_label,
                palette.text,
                2,
            );
            if let Some(progress) = progress {
                draw_bitmap_text(
                    buffer,
                    (width, height),
                    (text_x, heading_y + 116),
                    &format!(
                        "İşlem {} / {}",
                        progress.completed_steps(),
                        progress.total_steps()
                    ),
                    palette.secondary_text,
                    1,
                );
            }
            fill_rounded_rect(
                buffer,
                width,
                height,
                layout.progress_track,
                4,
                palette.border,
                210,
            );
            if percent > 0 {
                fill_rounded_rect(
                    buffer,
                    width,
                    height,
                    Rect {
                        width: layout.progress_track.width * u32::from(percent) / 100,
                        ..layout.progress_track
                    },
                    4,
                    palette.accent,
                    255,
                );
            }
            draw_bitmap_text(
                buffer,
                (width, height),
                (layout.progress_track.x, layout.progress_track.y + 24),
                &format!("Kurulum ilerlemesi: %{percent}"),
                palette.text,
                1,
            );
            primitives += 6;
        }
        InstallerStep::Completed => {
            draw_installer_step_heading(
                buffer,
                width,
                height,
                text_x,
                heading_y,
                model.step(),
                palette,
            );
            draw_bitmap_text(
                buffer,
                (width, height),
                (text_x, heading_y + 62),
                "Aqua Linux kullanıma hazır.",
                palette.secondary_text,
                1,
            );
            primitives += 2;
        }
    }
    primitives
}

fn installer_progress_phase_label(phase: InstallProgressPhase) -> &'static str {
    match phase {
        InstallProgressPhase::PreparingTarget => "Hedef hazırlanıyor",
        InstallProgressPhase::Partitioning => "Disk bölümleniyor",
        InstallProgressPhase::Formatting => "Dosya sistemleri hazırlanıyor",
        InstallProgressPhase::InstallingSystem => "Sistem dosyaları kuruluyor",
        InstallProgressPhase::InstallingBootloader => "Başlatıcı kuruluyor",
        InstallProgressPhase::ConfiguringSystem => "Sistem yapılandırılıyor",
        InstallProgressPhase::Finalizing => "Son işlemler uygulanıyor",
        InstallProgressPhase::Completed => "Kurulum tamamlandı",
    }
}

fn draw_installer_step_heading(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    step: InstallerStep,
    palette: WindowChromePalette,
) {
    draw_bitmap_text(
        buffer,
        (width, height),
        (x, y),
        step.label_tr(),
        palette.text,
        2,
    );
}

struct InstallerChoiceForm<'a> {
    layout: &'a InstallerWindowLayout,
    x: u32,
    y: u32,
    step: InstallerStep,
    options: &'a [InstallerChoiceOption],
    selected_index: usize,
    applied_value: Option<&'a str>,
    palette: WindowChromePalette,
}

fn draw_installer_choice_form(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    form: InstallerChoiceForm<'_>,
) -> usize {
    let InstallerChoiceForm {
        layout,
        x,
        y,
        step,
        options,
        selected_index,
        applied_value,
        palette,
    } = form;
    draw_installer_step_heading(buffer, width, height, x, y, step, palette);
    draw_bitmap_text(
        buffer,
        (width, height),
        (x, y + 38),
        match step {
            InstallerStep::Language => "Sistem dilini seçin.",
            InstallerStep::Keyboard => "Klavye düzenini seçin.",
            InstallerStep::TimeZone => "Bulunduğunuz zaman dilimini seçin.",
            _ => "Bir seçenek belirleyin.",
        },
        palette.secondary_text,
        1,
    );
    let mut primitives = 2;
    for (index, option) in options.iter().enumerate() {
        let row = layout.choice_row(index);
        let selected = index == selected_index;
        fill_rounded_rect(
            buffer,
            width,
            height,
            row,
            8,
            if selected {
                palette.accent_soft
            } else {
                palette.field
            },
            if selected { 245 } else { 205 },
        );
        if selected {
            draw_rect_outline(buffer, width, height, row, palette.accent, 190);
        }
        fill_transparent_circle(
            buffer,
            width,
            height,
            row.x + 24,
            row.y + row.height / 2,
            8,
            if selected {
                palette.accent
            } else {
                palette.border
            },
        );
        if applied_value == Some(option.value) {
            fill_transparent_circle(
                buffer,
                width,
                height,
                row.right() - 28,
                row.y + row.height / 2,
                7,
                [0x2a, 0xb8, 0x70, 0xff],
            );
        }
        draw_bitmap_text(
            buffer,
            (width, height),
            (row.x + 48, row.y + 10),
            option.label,
            palette.text,
            1,
        );
        draw_bitmap_text(
            buffer,
            (width, height),
            (row.x + 48, row.y + 34),
            option.detail,
            palette.secondary_text,
            1,
        );
        primitives += 5;
    }
    primitives
}

struct InstallerDiskForm<'a> {
    layout: &'a InstallerWindowLayout,
    x: u32,
    y: u32,
    forms: &'a InstallerFormState,
    applied_device: Option<&'a str>,
    palette: WindowChromePalette,
}

fn draw_installer_disk_form(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    form: InstallerDiskForm<'_>,
) -> usize {
    draw_installer_step_heading(
        buffer,
        width,
        height,
        form.x,
        form.y,
        InstallerStep::Partitions,
        form.palette,
    );
    draw_bitmap_text(
        buffer,
        (width, height),
        (form.x, form.y + 38),
        "Aqua Linux'in kurulacağı diski seçin.",
        form.palette.secondary_text,
        1,
    );
    let list_y = form.y + 72;
    let row_width = form.layout.content_row_width();
    let mut primitives = 2;
    if form.forms.disk_options().is_empty() {
        draw_bitmap_text(
            buffer,
            (width, height),
            (form.x, list_y + 18),
            "Uygun kurulum diski bulunamadı.",
            [0x9b, 0x4b, 0x4b, 0xff],
            1,
        );
        return primitives + 1;
    }

    for (index, option) in form.forms.disk_options().iter().take(4).enumerate() {
        let row = form.layout.disk_row(index);
        let selected = form.forms.disk_index() == Some(index);
        let eligible = option.is_eligible();
        fill_rounded_rect(
            buffer,
            width,
            height,
            row,
            8,
            if selected && eligible {
                form.palette.accent_soft
            } else {
                form.palette.field
            },
            if eligible { 225 } else { 150 },
        );
        if selected && eligible {
            draw_rect_outline(buffer, width, height, row, form.palette.accent, 190);
        }
        fill_transparent_circle(
            buffer,
            width,
            height,
            row.x + 23,
            row.y + row.height / 2,
            8,
            if selected && eligible {
                form.palette.accent
            } else {
                form.palette.border
            },
        );
        draw_bitmap_text(
            buffer,
            (width, height),
            (row.x + 46, row.y + 8),
            &format!("{}  {}", option.device(), option.model()),
            if eligible {
                form.palette.text
            } else {
                form.palette.secondary_text
            },
            1,
        );
        let detail = if eligible {
            format!(
                "{:.1} GiB  •  Tüm disk kullanılacak",
                option.capacity_bytes() as f64 / 1_073_741_824.0
            )
        } else {
            format!("Kullanılamıyor: {}", option.blocked_reasons()[0].id())
        };
        draw_bitmap_text(
            buffer,
            (width, height),
            (row.x + 46, row.y + 31),
            &detail,
            form.palette.secondary_text,
            1,
        );
        if form.applied_device == Some(option.device()) {
            fill_transparent_circle(
                buffer,
                width,
                height,
                row.right() - 27,
                row.y + row.height / 2,
                7,
                [0x2a, 0xb8, 0x70, 0xff],
            );
        }
        primitives += 5;
    }

    if let Some(index) = form.forms.disk_index() {
        if form.forms.disk_options()[index].is_eligible() {
            let panel_y = list_y + form.forms.disk_options().len().min(4) as u32 * 68 + 8;
            let panel = Rect {
                x: form.x,
                y: panel_y,
                width: row_width,
                height: 88,
            };
            fill_rounded_rect(buffer, width, height, panel, 8, form.palette.toolbar, 210);
            draw_bitmap_text(
                buffer,
                (width, height),
                (panel.x + 16, panel.y + 12),
                "Bölüm planı - seçilen diskteki veriler silinecek",
                [0x7a, 0x45, 0x31, 0xff],
                1,
            );
            draw_bitmap_text(
                buffer,
                (width, height),
                (panel.x + 16, panel.y + 39),
                &format!("{}  FAT32  {} MiB", INSTALL_ESP_LABEL, INSTALL_ESP_SIZE_MIB),
                form.palette.text,
                1,
            );
            draw_bitmap_text(
                buffer,
                (width, height),
                (panel.x + row_width / 2, panel.y + 39),
                &format!("{}  ext4  kalan alan", INSTALL_ROOT_LABEL),
                form.palette.text,
                1,
            );
            primitives += 4;
        }
    }
    primitives
}

struct InstallerUserForm<'a> {
    layout: &'a InstallerWindowLayout,
    x: u32,
    y: u32,
    forms: &'a InstallerFormState,
    applied: bool,
    palette: WindowChromePalette,
}

fn draw_installer_user_form(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    form: InstallerUserForm<'_>,
) -> usize {
    draw_installer_step_heading(
        buffer,
        width,
        height,
        form.x,
        form.y,
        InstallerStep::UserInformation,
        form.palette,
    );
    draw_bitmap_text(
        buffer,
        (width, height),
        (form.x, form.y + 38),
        "İlk kullanıcı hesabını oluşturun.",
        form.palette.secondary_text,
        1,
    );
    let user = form.forms.user();
    let row_width = form.layout.content_row_width();
    let password_status = if user.password_configured() {
        "Parola yapılandırıldı"
    } else {
        "Parola gerekli"
    };
    let fields = [
        (
            InstallerUserField::Username,
            "Kullanıcı adı",
            if user.username().is_empty() {
                "aqua"
            } else {
                user.username()
            },
        ),
        (
            InstallerUserField::DisplayName,
            "Görünen ad",
            if user.display_name().is_empty() {
                "Aqua Kullanıcısı"
            } else {
                user.display_name()
            },
        ),
        (InstallerUserField::Password, "Parola", password_status),
    ];
    let mut primitives = 2;
    for (field, label, value) in fields {
        let row = form.layout.user_field_row(field);
        let selected = user.active_field() == field;
        fill_rounded_rect(
            buffer,
            width,
            height,
            row,
            8,
            if selected {
                form.palette.accent_soft
            } else {
                form.palette.field
            },
            if selected { 245 } else { 205 },
        );
        if selected {
            draw_rect_outline(buffer, width, height, row, form.palette.accent, 190);
        }
        draw_bitmap_text(
            buffer,
            (width, height),
            (row.x + 18, row.y + 9),
            label,
            form.palette.secondary_text,
            1,
        );
        draw_bitmap_text(
            buffer,
            (width, height),
            (row.x + 18, row.y + 35),
            value,
            if field == InstallerUserField::Password && !user.password_configured() {
                [0x9b, 0x4b, 0x4b, 0xff]
            } else {
                form.palette.text
            },
            1,
        );
        primitives += 4;
    }
    if form.applied {
        fill_transparent_circle(
            buffer,
            width,
            height,
            form.x + row_width - 10,
            form.y + 10,
            7,
            [0x2a, 0xb8, 0x70, 0xff],
        );
        primitives += 1;
    }
    primitives
}

struct InstallerSummaryView<'a> {
    layout: &'a InstallerWindowLayout,
    x: u32,
    y: u32,
    model: &'a InstallerModel,
    forms: &'a InstallerFormState,
    palette: WindowChromePalette,
    theme: AquaTheme,
}

fn draw_installer_summary(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    view: InstallerSummaryView<'_>,
) -> usize {
    draw_installer_step_heading(
        buffer,
        width,
        height,
        view.x,
        view.y,
        InstallerStep::Summary,
        view.palette,
    );
    draw_bitmap_text(
        buffer,
        (width, height),
        (view.x, view.y + 38),
        "Kurulum seçimleri",
        view.palette.secondary_text,
        1,
    );
    let target = view.model.target().expect("summary requires target");
    let user = view.model.user().expect("summary requires user");
    let available_width = view
        .layout
        .content
        .width
        .saturating_sub((view.x - view.layout.content.x) * 2);
    let column_gap = 12;
    let tile_width = (available_width - column_gap) / 2;
    let tile_height = 54;
    let tiles_y = view.y + 68;
    let capacity = format!(
        "{} · {:.1} GiB",
        target.disk.device(),
        target.disk.capacity_bytes() as f64 / 1_073_741_824.0
    );
    let locale_keyboard = format!(
        "{} · {}",
        view.model.locale().unwrap_or("-"),
        view.model.keyboard_layout().unwrap_or("-")
    );
    let partition_plan = format!(
        "{} {} MiB · {} kalan",
        INSTALL_ESP_LABEL, INSTALL_ESP_SIZE_MIB, INSTALL_ROOT_LABEL
    );
    let items = [
        ("Disk", capacity.as_str()),
        ("Dil ve klavye", locale_keyboard.as_str()),
        ("Zaman dilimi", view.model.timezone().unwrap_or("-")),
        ("Kullanıcı", user.display_name()),
        ("Bölümler", partition_plan.as_str()),
        (
            "Kurulum modu",
            if view.model.mode() == InstallMode::Real {
                "Gerçek kurulum"
            } else {
                "Dry-run"
            },
        ),
    ];
    let mut primitives = 2;
    for (index, (label, value)) in items.into_iter().enumerate() {
        let column = index % 2;
        let row = index / 2;
        let tile = Rect {
            x: view.x + column as u32 * (tile_width + column_gap),
            y: tiles_y + row as u32 * (tile_height + 10),
            width: tile_width,
            height: tile_height,
        };
        fill_rounded_rect(buffer, width, height, tile, 8, view.palette.field, 210);
        draw_bitmap_text(
            buffer,
            (width, height),
            (tile.x + 14, tile.y + 7),
            label,
            view.palette.secondary_text,
            1,
        );
        draw_bitmap_text(
            buffer,
            (width, height),
            (tile.x + 14, tile.y + 29),
            value,
            view.palette.text,
            1,
        );
        primitives += 3;
    }

    let confirmation_panel = view.layout.summary_confirmation_panel();
    let confirmation_surface = if view.palette.text[0] > 0x80 {
        view.palette.hover
    } else if view.model.mode() == InstallMode::Real {
        [0xff, 0xed, 0xe5, 0xff]
    } else {
        [0xe7, 0xf5, 0xed, 0xff]
    };
    fill_rounded_rect(
        buffer,
        width,
        height,
        confirmation_panel,
        8,
        confirmation_surface,
        220,
    );
    let (title, detail) = if view.model.mode() == InstallMode::Real {
        (
            "Seçilen diskteki tüm veriler silinecek".to_string(),
            if view.model.destructive_confirmed() {
                "Hedefe bağlı onay doğrulandı".to_string()
            } else {
                view.model
                    .confirmation_phrase()
                    .unwrap_or_else(|| "Onay ifadesi kullanılamıyor".to_string())
            },
        )
    } else {
        (
            "Dry-run önizlemesi".to_string(),
            "Disk komutları ve dosya sistemi yazımları yürütülmeyecek".to_string(),
        )
    };
    let confirmation_dialog = (view.model.mode() == InstallMode::Real).then(|| {
        ConfirmationDialog::new(
            confirmation_panel,
            "Install confirmation",
            (&title, &detail),
            ConfirmationPresentation::Inline,
            ConfirmationSeverity::Destructive,
            ConfirmationRequirement::ExactText,
            if view.model.destructive_confirmed() {
                ConfirmationState::Confirmed
            } else {
                ConfirmationState::Pending
            },
        )
    });
    let title_position = confirmation_dialog.map_or(
        (confirmation_panel.x + 16, confirmation_panel.y + 14),
        |dialog| {
            let title = dialog.slots().title;
            (title.x, title.y)
        },
    );
    let detail_position = (confirmation_panel.x + 16, confirmation_panel.y + 62);
    draw_bitmap_text(
        buffer,
        (width, height),
        title_position,
        &title,
        if view.model.mode() == InstallMode::Real {
            [0x8b, 0x43, 0x2f, 0xff]
        } else {
            [0x2c, 0x6b, 0x4b, 0xff]
        },
        1,
    );
    if view.model.mode() == InstallMode::Real {
        let checkbox = view.forms.summary().acknowledgement_checkbox(
            view.model,
            view.layout,
            "Hedef diskin silineceğini anlıyorum",
        );
        primitives += draw_checkbox(
            buffer,
            width,
            height,
            checkbox,
            view.theme,
            OutputScale::One,
        );
    }
    draw_bitmap_text(
        buffer,
        (width, height),
        detail_position,
        &detail,
        view.palette.secondary_text,
        1,
    );
    if view.forms.summary().can_begin_install(view.model) {
        let status = confirmation_dialog.map_or(
            Rect {
                x: confirmation_panel.right().saturating_sub(31),
                y: confirmation_panel.y.saturating_add(17),
                width: 14,
                height: 14,
            },
            |dialog| dialog.slots().status,
        );
        fill_transparent_circle(
            buffer,
            width,
            height,
            status.x + status.width / 2,
            status.y + status.height / 2,
            7,
            [0x2a, 0xb8, 0x70, 0xff],
        );
        primitives += 1;
    }
    primitives + 3
}

fn draw_installer_footer(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    layout: &InstallerWindowLayout,
    ui: &InstallerUiState,
    theme: AquaTheme,
) -> usize {
    let focus = ui.focus();
    let state_for = |target| {
        if focus == target {
            ComponentState::KeyboardFocus
        } else {
            ComponentState::Idle
        }
    };
    let mut primitives = draw_standard_button(
        buffer,
        width,
        height,
        StandardButton::new(
            layout.language_control,
            "Türkçe",
            StandardButtonVariant::Secondary,
        )
        .with_state(state_for(InstallerFocusTarget::LanguageControl)),
        theme,
        OutputScale::One,
    );
    if ui.cancel_visible() {
        primitives += draw_standard_button(
            buffer,
            width,
            height,
            StandardButton::new(
                layout.cancel_button,
                "Vazgeç",
                StandardButtonVariant::Secondary,
            )
            .with_state(state_for(InstallerFocusTarget::Cancel)),
            theme,
            OutputScale::One,
        );
    }
    if ui.back_visible() {
        primitives += draw_standard_button(
            buffer,
            width,
            height,
            StandardButton::new(layout.back_button, "Geri", StandardButtonVariant::Secondary)
                .with_state(state_for(InstallerFocusTarget::Back)),
            theme,
            OutputScale::One,
        );
    }
    if let Some(label) = ui.forward_label() {
        let target = if focus == InstallerFocusTarget::Finish {
            InstallerFocusTarget::Finish
        } else {
            InstallerFocusTarget::Forward
        };
        primitives += draw_standard_button(
            buffer,
            width,
            height,
            StandardButton::new(layout.forward_button, label, StandardButtonVariant::Primary)
                .with_state(state_for(target)),
            theme,
            OutputScale::One,
        );
    }
    primitives
}

fn draw_installer_focus(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    layout: &InstallerWindowLayout,
    focus: InstallerFocusTarget,
    palette: WindowChromePalette,
) -> usize {
    let rect = match focus {
        InstallerFocusTarget::StepContent => Some(layout.content),
        InstallerFocusTarget::ProgressStatus => Some(layout.progress_track),
        InstallerFocusTarget::LanguageControl
        | InstallerFocusTarget::Cancel
        | InstallerFocusTarget::Back
        | InstallerFocusTarget::Forward
        | InstallerFocusTarget::Finish => None,
    };
    let Some(rect) = rect else { return 0 };
    draw_rect_outline(
        buffer,
        width,
        height,
        Rect {
            x: rect.x.saturating_sub(3),
            y: rect.y.saturating_sub(3),
            width: rect.width + 6,
            height: rect.height + 6,
        },
        palette.accent,
        220,
    );
    1
}

pub fn render_settings_window_rgba(
    width: u32,
    height: u32,
    model: &SettingsWindowModel,
) -> (Vec<u8>, SettingsWindowProbe) {
    let mut buffer = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    if width == 0 || height == 0 {
        return (
            buffer,
            SettingsWindowProbe {
                rendered: false,
                category_count: model.categories.len(),
                selected_category: model.selected_category,
                reduced_motion: model.reduced_motion,
                desktop_icons: model.desktop_icons,
                key_repeat: model.key_repeat,
                audio_available: model.audio.available(),
                audio_controls_enabled: model.audio.controls_enabled(),
                audio_backend_applied: model.audio.backend_applied(),
                audio_control_status: model.audio.control_status().id(),
                audio_desired_volume_percent: model.audio.volume_percent(),
                audio_volume_percent: model
                    .audio
                    .authoritative_volume_percent()
                    .unwrap_or_else(|| model.audio.volume_percent()),
                audio_muted: model
                    .audio
                    .authoritative_muted()
                    .unwrap_or_else(|| model.audio.muted()),
                network_interface_count: model.network.interfaces().len(),
                network_status_available: model.network.status_available(),
                wifi_control_available: model.wifi.available(),
                wifi_controls_enabled: model.wifi.controls_enabled(),
                wifi_connected: model.wifi.connected(),
                wifi_credential_saved: model.wifi.credential_saved(),
                wifi_scan_result_count: model.wifi.networks().len(),
                wifi_credential_entry: model.wifi.credential_entry(),
                wifi_connect_attempts_remaining: model.wifi.connect_attempts_remaining(),
                primitive_count: 0,
                checksum: 0,
            },
        );
    }

    let canvas = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let palette = window_chrome_palette(model.theme);
    let mut primitives = draw_window_frame(
        &mut buffer,
        width,
        height,
        WindowFrame::new(canvas, model.title, 58),
        palette,
    );

    let sidebar = Rect {
        x: 2,
        y: 60,
        width: 188,
        height: height.saturating_sub(62),
    };
    let navigation = SidebarNavigation::new(
        sidebar,
        SETTINGS_SIDEBAR_NAVIGATION.label,
        SETTINGS_SIDEBAR_NAVIGATION.first_row,
        SETTINGS_SIDEBAR_NAVIGATION.row_stride,
    );
    primitives += draw_sidebar_navigation(&mut buffer, width, height, navigation, model.theme);
    for (index, category) in model.categories.iter().enumerate() {
        let row_rect = navigation.row_rect(index);
        let state = if model.selected_category == index {
            ComponentState::Selected
        } else if model.hovered_category == Some(index) {
            ComponentState::Hover
        } else {
            ComponentState::Idle
        };
        let row = ListRow::new(row_rect, category, ListRowRole::Navigation).with_state(state);
        primitives += draw_list_row(
            &mut buffer,
            width,
            height,
            row,
            model.theme,
            OutputScale::One,
        );
        draw_category_icon(
            &mut buffer,
            width,
            height,
            row.slots().leading.x + 2,
            row_rect.y + 10,
            index,
        );
        primitives += 1;
    }

    let section = model.section_group();
    primitives += draw_section_group(&mut buffer, width, height, section, model.theme);
    let heading = model.categories[model.selected_category];
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (section.heading_rect().x, section.heading_rect().y + 16),
        heading,
        palette.text,
        2,
    );
    let first_row = section.row_rect(0);
    if model.selected_category == 0 {
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 16),
            "REDUCED MOTION",
            palette.text,
            1,
        );
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 38),
            "Limit desktop animation",
            palette.secondary_text,
            1,
        );
        primitives += draw_switch_control(
            &mut buffer,
            width,
            height,
            model.active_switch().expect("appearance switch"),
            model.theme,
        );
        primitives += draw_segmented_control(
            &mut buffer,
            width,
            height,
            model.theme_segmented_control(),
            &["LightWhite", "Softtouch", "Deepside", "Nightmare"],
            model.theme,
            OutputScale::One,
        );
    } else if model.selected_category == 1 {
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 16),
            "SHOW DESKTOP ICONS",
            palette.text,
            1,
        );
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 38),
            "Show home and storage items",
            palette.secondary_text,
            1,
        );
        primitives += draw_switch_control(
            &mut buffer,
            width,
            height,
            model.active_switch().expect("desktop switch"),
            model.theme,
        );
    } else if model.selected_category == 2 {
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 16),
            "KEY REPEAT",
            palette.text,
            1,
        );
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 38),
            "Repeat held keyboard keys",
            palette.secondary_text,
            1,
        );
        primitives += draw_switch_control(
            &mut buffer,
            width,
            height,
            model.active_switch().expect("input switch"),
            model.theme,
        );
    } else if model.selected_category == 3 {
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 16),
            "WI-FI ASSOCIATION",
            palette.text,
            1,
        );
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 30),
            if model.wifi.available() {
                "AUTHENTICATED BROKER"
            } else {
                "CONTROL UNAVAILABLE"
            },
            palette.secondary_text,
            1,
        );
        primitives += 2;
        primitives += draw_switch_control(
            &mut buffer,
            width,
            height,
            model.active_switch().expect("network switch"),
            model.theme,
        );
        let wifi_status = model
            .wifi
            .status_label()
            .chars()
            .take(42)
            .collect::<String>()
            .to_ascii_uppercase();
        if model.wifi.credential_entry() {
            let selected = model
                .wifi
                .selected_network()
                .and_then(|network| std::str::from_utf8(network.ssid.bytes()).ok())
                .unwrap_or("UNKNOWN")
                .chars()
                .take(28)
                .collect::<String>();
            draw_bitmap_text(
                &mut buffer,
                (width, height),
                (section.row_rect(1).x, section.row_rect(1).y + 18),
                &format!("NETWORK  {selected}"),
                palette.text,
                1,
            );
            draw_bitmap_text(
                &mut buffer,
                (width, height),
                (section.row_rect(2).x, section.row_rect(2).y + 18),
                &format!("PASSWORD  {}", model.wifi.masked_passphrase()),
                palette.accent,
                1,
            );
            let credential_hint = if model.wifi.status_label().starts_with("connection-failed") {
                format!(
                    "RETRY  {} ATTEMPT LEFT  ESC CANCEL",
                    model.wifi.connect_attempts_remaining()
                )
            } else if model.wifi.passphrase_ready() {
                "ENTER TO CONNECT  ESC TO CANCEL".to_owned()
            } else {
                "8-63 CHARACTERS  ESC TO CANCEL".to_owned()
            };
            draw_bitmap_text(
                &mut buffer,
                (width, height),
                (section.row_rect(3).x, section.row_rect(3).y + 18),
                &credential_hint,
                palette.secondary_text,
                1,
            );
            primitives += 3;
        } else if model.wifi.networks().is_empty() {
            draw_bitmap_text(
                &mut buffer,
                (width, height),
                (section.row_rect(1).x, section.row_rect(1).y + 18),
                &format!("WI-FI {wifi_status}"),
                palette.accent,
                1,
            );
            draw_bitmap_text(
                &mut buffer,
                (width, height),
                (section.row_rect(2).x, section.row_rect(2).y + 18),
                "NO DISCOVERED NETWORKS",
                palette.secondary_text,
                1,
            );
            primitives += 2;
        } else {
            for (index, network) in model
                .wifi
                .networks()
                .iter()
                .take(aqua_shell::MAX_VISIBLE_WIFI_NETWORKS)
                .enumerate()
            {
                let row = model
                    .wifi_network_row(index)
                    .expect("visible Wi-Fi network row");
                primitives += draw_list_row(
                    &mut buffer,
                    width,
                    height,
                    row,
                    model.theme,
                    OutputScale::One,
                );
                draw_bitmap_text(
                    &mut buffer,
                    (width, height),
                    (row.slots().trailing.x, row.rect.y + 18),
                    &format!(
                        "{} DBM  {}",
                        network.signal_dbm,
                        network.security.id().to_ascii_uppercase()
                    ),
                    if row.state != ComponentState::Disabled {
                        palette.text
                    } else {
                        palette.secondary_text
                    },
                    1,
                );
                primitives += 1;
            }
        }
        if !model.wifi.credential_entry() {
            primitives += draw_standard_button(
                &mut buffer,
                width,
                height,
                model.wifi_rescan_button(),
                model.theme,
                OutputScale::One,
            );
            primitives += draw_standard_button(
                &mut buffer,
                width,
                height,
                model.wifi_forget_button(),
                model.theme,
                OutputScale::One,
            );
        }
    } else if model.selected_category == 4 {
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 16),
            "OUTPUT VOLUME",
            palette.text,
            1,
        );
        let authoritative_volume = model
            .audio
            .authoritative_volume_percent()
            .unwrap_or_else(|| model.audio.volume_percent());
        let status = match model.audio.control_status() {
            AudioControlStatus::Unavailable => "UNAVAILABLE".to_string(),
            AudioControlStatus::Starting => "STARTING".to_string(),
            AudioControlStatus::Degraded => "DEGRADED".to_string(),
            AudioControlStatus::Applying => format!("APPLYING {authoritative_volume}%"),
            AudioControlStatus::Applied => format!("{authoritative_volume}%"),
        };
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 38),
            &status,
            palette.secondary_text,
            1,
        );
        primitives += draw_slider(
            &mut buffer,
            width,
            height,
            model.audio_slider(),
            model.theme,
        );
        let mute_row = section.row_rect(1);
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (mute_row.x, mute_row.y + 28),
            "MUTE OUTPUT",
            palette.text,
            1,
        );
        primitives += draw_switch_control(
            &mut buffer,
            width,
            height,
            model.active_switch().expect("audio mute switch"),
            model.theme,
        );
        primitives += 3;
    } else {
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (first_row.x, first_row.y + 16),
            "SETTINGS WILL APPEAR HERE",
            palette.secondary_text,
            1,
        );
        primitives += 1;
    }
    let checksum = checksum_bytes(&buffer);
    (
        buffer,
        SettingsWindowProbe {
            rendered: true,
            category_count: model.categories.len(),
            selected_category: model.selected_category,
            reduced_motion: model.reduced_motion,
            desktop_icons: model.desktop_icons,
            key_repeat: model.key_repeat,
            audio_available: model.audio.available(),
            audio_controls_enabled: model.audio.controls_enabled(),
            audio_backend_applied: model.audio.backend_applied(),
            audio_control_status: model.audio.control_status().id(),
            audio_desired_volume_percent: model.audio.volume_percent(),
            audio_volume_percent: model
                .audio
                .authoritative_volume_percent()
                .unwrap_or_else(|| model.audio.volume_percent()),
            audio_muted: model
                .audio
                .authoritative_muted()
                .unwrap_or_else(|| model.audio.muted()),
            network_interface_count: model.network.interfaces().len(),
            network_status_available: model.network.status_available(),
            wifi_control_available: model.wifi.available(),
            wifi_controls_enabled: model.wifi.controls_enabled(),
            wifi_connected: model.wifi.connected(),
            wifi_credential_saved: model.wifi.credential_saved(),
            wifi_scan_result_count: model.wifi.networks().len(),
            wifi_credential_entry: model.wifi.credential_entry(),
            wifi_connect_attempts_remaining: model.wifi.connect_attempts_remaining(),
            primitive_count: primitives,
            checksum,
        },
    )
}

impl FilesWindowProbe {
    pub fn is_ready(&self) -> bool {
        self.rendered
            && self.sidebar_item_count == 5
            && self.selected_sidebar < self.sidebar_item_count
            && self.primitive_count >= 19
            && self.checksum != 0
    }
}

pub fn render_files_window_rgba(
    width: u32,
    height: u32,
    model: &FilesWindowModel,
) -> (Vec<u8>, FilesWindowProbe) {
    render_files_window_rgba_with_theme(width, height, model, AquaTheme::LightWhite)
}

pub fn render_files_window_rgba_with_theme(
    width: u32,
    height: u32,
    model: &FilesWindowModel,
    theme: AquaTheme,
) -> (Vec<u8>, FilesWindowProbe) {
    let mut buffer = vec![0_u8; width.saturating_mul(height).saturating_mul(4) as usize];
    if width == 0 || height == 0 {
        return (
            buffer,
            FilesWindowProbe {
                rendered: false,
                sidebar_item_count: model.sidebar_items.len(),
                entry_count: model.entries.len(),
                selected_sidebar: model.selected_sidebar,
                empty_state_rendered: model.is_empty(),
                primitive_count: 0,
                checksum: 0,
            },
        );
    }

    let canvas = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let palette = window_chrome_palette(theme);
    let mut primitives = draw_window_frame(
        &mut buffer,
        width,
        height,
        WindowFrame::new(canvas, model.title, 48),
        palette,
    );

    let toolbar = files_toolbar(width);
    primitives += draw_toolbar(&mut buffer, width, height, toolbar, theme);
    primitives += draw_icon_button(
        &mut buffer,
        width,
        height,
        files_back_button().with_state(if model.can_go_back {
            ComponentState::Idle
        } else {
            ComponentState::Disabled
        }),
        theme,
    );
    primitives += draw_icon_button(
        &mut buffer,
        width,
        height,
        files_forward_button().with_state(if model.can_go_forward {
            ComponentState::Idle
        } else {
            ComponentState::Disabled
        }),
        theme,
    );
    let location = Rect {
        x: 96,
        y: 64,
        width: width.saturating_sub(118),
        height: 32,
    };
    fill_rect(&mut buffer, width, height, location, palette.field, 255);
    draw_rect_outline(&mut buffer, width, height, location, palette.border, 255);
    primitives += 2;
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (location.x + 14, location.y + 10),
        &model.location,
        palette.text,
        1,
    );

    let sidebar_width = 170;
    let navigation = files_sidebar_navigation(height);
    primitives += draw_sidebar_navigation(&mut buffer, width, height, navigation, theme);
    for index in 0..model.sidebar_items.len() {
        let Some(row) = model.sidebar_row(height, index) else {
            continue;
        };
        primitives += draw_list_row(&mut buffer, width, height, row, theme, OutputScale::One);
        draw_sidebar_icon(
            &mut buffer,
            width,
            height,
            row.slots().leading.x + 2,
            row.rect.y + 9,
            index,
        );
        primitives += 1;
    }
    let list_x = sidebar_width + 18;
    if let Some(preview) = model.preview.as_ref() {
        let visible_lines = files_preview_visible_lines(height);
        draw_file_icon(&mut buffer, width, height, list_x + 12, 136);
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (list_x + 54, 142),
            &preview.name,
            palette.text,
            1,
        );
        for (line_index, line) in preview
            .content
            .lines()
            .skip(preview.scroll_offset)
            .take(visible_lines)
            .enumerate()
        {
            let bounded = line.chars().take(48).collect::<String>();
            draw_bitmap_text(
                &mut buffer,
                (width, height),
                (list_x + 16, 194 + line_index as u32 * 22),
                &bounded,
                palette.text,
                1,
            );
        }
        if let Some(status_y) = height.checked_sub(80).filter(|status_y| *status_y >= 220) {
            draw_bitmap_text(
                &mut buffer,
                (width, height),
                (list_x + 16, status_y.min(340)),
                "READ ONLY",
                palette.secondary_text,
                1,
            );
        }
        let line_count = preview.content.lines().count();
        primitives += 2 + line_count.min(visible_lines);
        if let Some(scrollbar) = model.preview_scrollbar_in_viewport(width, height) {
            fill_rect(
                &mut buffer,
                width,
                height,
                scrollbar.track,
                palette.border,
                255,
            );
            fill_rect(
                &mut buffer,
                width,
                height,
                scrollbar.thumb,
                palette.accent,
                255,
            );
            primitives += 2;
        }
    } else if model.is_empty() {
        draw_folder_icon(
            &mut buffer,
            width,
            height,
            width / 2 - 24,
            190,
            [0x58, 0xc9, 0xf3, 0xff],
        );
        draw_bitmap_text(
            &mut buffer,
            (width, height),
            (width / 2 - 50, 252),
            "EMPTY FOLDER",
            palette.secondary_text,
            1,
        );
        primitives += 2;
    } else {
        for (row_index, (index, entry)) in model
            .entries
            .iter()
            .enumerate()
            .skip(model.scroll_offset)
            .take(files_visible_rows(height))
            .enumerate()
        {
            let row = model
                .entry_row_in_viewport(width, height, index)
                .expect("visible Files entry row");
            debug_assert_eq!(row.rect.y, 124 + row_index as u32 * 64);
            primitives += draw_list_row(&mut buffer, width, height, row, theme, OutputScale::One);
            match entry.kind {
                FilesEntryKind::Folder => draw_folder_icon(
                    &mut buffer,
                    width,
                    height,
                    row.slots().leading.x + 2,
                    row.rect.y + 9,
                    [0x4f, 0xc9, 0xff, 0xff],
                ),
                FilesEntryKind::File => draw_file_icon(
                    &mut buffer,
                    width,
                    height,
                    row.slots().leading.x + 6,
                    row.rect.y + 8,
                ),
            }
            draw_fitted_bitmap_text(
                &mut buffer,
                (width, height),
                row.slots().trailing,
                &entry.detail,
                palette.secondary_text,
                FittedTextOptions::new(TextRole::Caption, OutputScale::One, false),
            );
            primitives += 2;
        }
        if let Some(scrollbar) = model.list_scrollbar_in_viewport(width, height) {
            fill_rect(
                &mut buffer,
                width,
                height,
                scrollbar.track,
                palette.border,
                255,
            );
            fill_rect(
                &mut buffer,
                width,
                height,
                scrollbar.thumb,
                palette.accent,
                255,
            );
            primitives += 2;
        }
    }

    if model.keyboard_focus && model.focused_sidebar.is_none() {
        let focus = Rect {
            x: sidebar_width + 6,
            y: 112,
            width: width.saturating_sub(sidebar_width + 12),
            height: height.saturating_sub(150),
        };
        for edge in [
            Rect {
                x: focus.x,
                y: focus.y,
                width: focus.width,
                height: 1,
            },
            Rect {
                x: focus.x,
                y: focus.y + focus.height.saturating_sub(1),
                width: focus.width,
                height: 1,
            },
            Rect {
                x: focus.x,
                y: focus.y,
                width: 1,
                height: focus.height,
            },
            Rect {
                x: focus.x + focus.width.saturating_sub(1),
                y: focus.y,
                width: 1,
                height: focus.height,
            },
        ] {
            fill_rect(&mut buffer, width, height, edge, palette.accent, 255);
            primitives += 1;
        }
    }

    let status = Rect {
        x: sidebar_width + 3,
        y: height.saturating_sub(34),
        width: width.saturating_sub(sidebar_width + 5),
        height: 32,
    };
    fill_rect(&mut buffer, width, height, status, palette.toolbar, 255);
    let status_text = format!("{} ITEMS", model.entries.len());
    draw_bitmap_text(
        &mut buffer,
        (width, height),
        (status.x + 14, status.y + 11),
        &status_text,
        palette.secondary_text,
        1,
    );
    primitives += 1;

    let checksum = checksum_bytes(&buffer);
    (
        buffer,
        FilesWindowProbe {
            rendered: true,
            sidebar_item_count: model.sidebar_items.len(),
            entry_count: model.entries.len(),
            selected_sidebar: model.selected_sidebar,
            empty_state_rendered: model.is_empty(),
            primitive_count: primitives,
            checksum,
        },
    )
}

impl LauncherOverlayProbe {
    pub fn is_ready(&self) -> bool {
        self.rendered
            && matches!(self.mode, "applications" | "search")
            && self.category_count == LauncherCategory::ALL.len()
            && self.visible_app_count > 0
            && self.selected_index < self.visible_app_count
            && self.primitive_count >= 12
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawCommandKind {
    ImageLayer,
    SystemSurfacePanel,
    IconGroup,
}

impl DrawCommandKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImageLayer => "image-layer",
            Self::SystemSurfacePanel => "system-surface-panel",
            Self::IconGroup => "icon-group",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawCommand {
    pub surface_id: &'static str,
    pub kind: DrawCommandKind,
    pub rect: Rect,
    pub asset_count: usize,
    pub material_token_count: usize,
    pub simulated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSurfaceSource {
    pub client_id: &'static str,
    pub surface_id: &'static str,
    pub window_id: &'static str,
    pub z_index: u8,
    pub focused: bool,
    pub rect: Rect,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: &'static str,
    pub source: &'static str,
    pub sample_checksum: u64,
    pub sample_pixel: [u8; 4],
    pub sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    pub client_buffer_rgba: Vec<u8>,
    pub renderer_import_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSurfaceSourcePlan {
    pub status: &'static str,
    pub backend: &'static str,
    pub renderer_started: bool,
    pub sources: Vec<ClientSurfaceSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLayerPaintStep {
    pub order: usize,
    pub client_id: &'static str,
    pub surface_id: &'static str,
    pub window_id: &'static str,
    pub focused: bool,
    pub rect: Rect,
    pub opacity: u8,
    pub blend_mode: &'static str,
    pub effect: &'static str,
    pub sample_checksum: u64,
    pub sample_pixel: [u8; 4],
    pub sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    pub source_width: u32,
    pub source_height: u32,
    pub client_buffer_rgba: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLayerPaintPlan {
    pub status: &'static str,
    pub backend: &'static str,
    pub renderer_started: bool,
    pub steps: Vec<ClientLayerPaintStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLayerRasterProbe {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub layer_count: usize,
    pub expected_layer_count: usize,
    pub active_layer_sample: [u8; 4],
    pub inactive_layer_sample: [u8; 4],
    pub layer_checksum: u64,
    pub source_checksum_fold: u64,
    pub renderer_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlan {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintStep {
    pub surface_id: &'static str,
    pub order: usize,
    pub kind: DrawCommandKind,
    pub rect: Rect,
    pub opacity: u8,
    pub blend_mode: &'static str,
    pub effect: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintPlan {
    pub status: &'static str,
    pub backend: &'static str,
    pub renderer_started: bool,
    pub steps: Vec<PaintStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramePlan {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub stride_bytes: u32,
    pub buffer_bytes: u64,
    pub clear_color: &'static str,
    pub damage_rect: Rect,
    pub paint_step_count: usize,
    pub renderer_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBufferProbe {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub buffer_bytes: u64,
    pub allocated_bytes: usize,
    pub clear_color: &'static str,
    pub first_pixel: [u8; 4],
    pub last_pixel: [u8; 4],
    pub renderer_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareRasterProbe {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub pixel_format: &'static str,
    pub filled_rect_count: usize,
    pub expected_rect_count: usize,
    pub wallpaper_sample: [u8; 4],
    pub surface_sample: [u8; 4],
    pub dock_sample: [u8; 4],
    pub surface_border_sample: [u8; 4],
    pub surface_highlight_sample: [u8; 4],
    pub surface_corner_sample: [u8; 4],
    pub surface_shadow_sample: [u8; 4],
    pub raster_checksum: u64,
    pub surface_primitive_count: usize,
    pub buffer_bytes: u64,
    pub renderer_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterPpmExport {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub format: &'static str,
    pub header: String,
    pub bytes: Vec<u8>,
    pub byte_count: usize,
    pub checksum: u64,
    pub renderer_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterPngExport {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub format: &'static str,
    pub bytes: Vec<u8>,
    pub byte_count: usize,
    pub checksum: u64,
    pub renderer_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterRgbaExport {
    pub status: &'static str,
    pub backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub format: &'static str,
    pub bytes: Vec<u8>,
    pub byte_count: usize,
    pub checksum: u64,
    pub renderer_started: bool,
}

impl RenderPlan {
    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("renderer_status={}", self.status),
            format!("renderer_backend={}", self.backend),
            format!("viewport={}x{}", self.width, self.height),
            format!("draw_command_count={}", self.commands.len()),
        ];

        lines.extend(self.commands.iter().map(|command| {
            format!(
                "draw surface={} kind={} rect={},{},{},{} asset_count={} material_token_count={} simulated={}",
                command.surface_id,
                command.kind.as_str(),
                command.rect.x,
                command.rect.y,
                command.rect.width,
                command.rect.height,
                command.asset_count,
                command.material_token_count,
                command.simulated
            )
        }));

        lines
    }

    pub fn is_ready(&self) -> bool {
        self.status == RENDERER_STATUS
            && self.backend == RENDER_BACKEND
            && self.width >= 800
            && self.height >= 600
            && self.commands.len() == 7
            && self.has_command("wallpaper", DrawCommandKind::ImageLayer)
            && self.has_command("launcher", DrawCommandKind::SystemSurfacePanel)
            && self.has_command("dock", DrawCommandKind::SystemSurfacePanel)
            && self.has_command("desktop-icons", DrawCommandKind::IconGroup)
            && self.system_surface_commands_are_simulated()
    }

    pub fn system_surface_commands_are_simulated(&self) -> bool {
        self.commands
            .iter()
            .filter(|command| command.kind == DrawCommandKind::SystemSurfacePanel)
            .all(|command| command.simulated && command.material_token_count >= 7)
    }

    fn has_command(&self, surface_id: &str, kind: DrawCommandKind) -> bool {
        self.commands
            .iter()
            .any(|command| command.surface_id == surface_id && command.kind == kind)
    }
}

impl ClientSurfaceSourcePlan {
    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("surface_source_status={}", self.status),
            format!("surface_source_backend={}", self.backend),
            format!("renderer_started={}", self.renderer_started),
            format!("surface_source_count={}", self.sources.len()),
        ];

        lines.extend(self.sources.iter().map(|source| {
            format!(
                "source client={} surface={} window={} z_index={} focused={} buffer={}x{} stride={} format={} source={} sample_checksum={:016x} sample_pixel={} sample_grid={} buffer_snapshot_bytes={} renderer_import_ready={} rect={},{},{},{}",
                source.client_id,
                source.surface_id,
                source.window_id,
                source.z_index,
                source.focused,
                source.width,
                source.height,
                source.stride,
                source.format,
                source.source,
                source.sample_checksum,
                pixel_as_hex(source.sample_pixel),
                sample_grid_as_hex(source.sample_grid),
                source.client_buffer_rgba.len(),
                source.renderer_import_ready,
                source.rect.x,
                source.rect.y,
                source.rect.width,
                source.rect.height
            )
        }));

        lines
    }

    pub fn is_ready(&self) -> bool {
        self.status == "client-surface-sources-ready"
            && self.backend == RENDER_BACKEND
            && !self.renderer_started
            && self.sources.len() == 2
            && self.sources.iter().all(ClientSurfaceSource::is_ready)
            && self
                .sources
                .windows(2)
                .all(|pair| pair[0].z_index >= pair[1].z_index)
            && self.sources.first().is_some_and(|source| {
                source.client_id == "wayland-client-1"
                    && source.surface_id == "xdg-toplevel-1"
                    && source.focused
            })
    }
}

impl ClientSurfaceSource {
    pub fn is_ready(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.stride == self.width * 4
            && self.rect.width > 0
            && self.rect.height > 0
            && self.format == "argb8888"
            && self.source == "client-committed-wl-shm"
            && self.sample_checksum != 0
            && self.sample_pixel[3] == 0xff
            && self.sample_grid.iter().all(|pixel| pixel[3] == 0xff)
            && self.renderer_import_ready
    }
}

impl ClientLayerPaintPlan {
    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("client_layer_paint_status={}", self.status),
            format!("client_layer_paint_backend={}", self.backend),
            format!("renderer_started={}", self.renderer_started),
            format!("client_layer_paint_step_count={}", self.steps.len()),
        ];

        lines.extend(self.steps.iter().map(|step| {
            format!(
                "client-layer-paint order={} client={} surface={} window={} rect={},{},{},{} source_buffer={}x{} opacity={} blend={} effect={} sample_checksum={:016x} sample_pixel={} sample_grid={} buffer_snapshot_bytes={}",
                step.order,
                step.client_id,
                step.surface_id,
                step.window_id,
                step.rect.x,
                step.rect.y,
                step.rect.width,
                step.rect.height,
                step.source_width,
                step.source_height,
                step.opacity,
                step.blend_mode,
                step.effect,
                step.sample_checksum,
                pixel_as_hex(step.sample_pixel),
                sample_grid_as_hex(step.sample_grid),
                step.client_buffer_rgba.len()
            )
        }));

        lines
    }

    pub fn is_ready(&self) -> bool {
        self.status == "client-layer-paint-ready"
            && self.backend == RENDER_BACKEND
            && !self.renderer_started
            && self.steps.len() == 2
            && self.steps.iter().enumerate().all(|(order, step)| {
                step.order == order
                    && step.rect.width > 0
                    && step.rect.height > 0
                    && step.opacity == 255
                    && step.blend_mode == "source-over"
                    && step.effect == "sampled-wl-shm-client-buffer"
                    && step.sample_checksum != 0
                    && step.sample_pixel[3] == 0xff
                    && step.sample_grid.iter().all(|pixel| pixel[3] == 0xff)
            })
            && self.steps.first().is_some_and(|step| {
                step.client_id == "wayland-client-1" && step.surface_id == "xdg-toplevel-1"
            })
    }
}

impl ClientLayerRasterProbe {
    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("client_layer_raster_status={}", self.status),
            format!("client_layer_raster_backend={}", self.backend),
            format!("frame_size={}x{}", self.width, self.height),
            format!("client_layer_count={}", self.layer_count),
            format!("expected_client_layer_count={}", self.expected_layer_count),
            format!(
                "active_layer_sample={}",
                pixel_as_hex(self.active_layer_sample)
            ),
            format!(
                "inactive_layer_sample={}",
                pixel_as_hex(self.inactive_layer_sample)
            ),
            format!("client_layer_checksum={:016x}", self.layer_checksum),
            format!("source_checksum_fold={:016x}", self.source_checksum_fold),
            format!("renderer_started={}", self.renderer_started),
        ]
    }

    pub fn is_ready(&self) -> bool {
        self.status == "client-layer-rasterized"
            && self.backend == RENDER_BACKEND
            && self.width >= 800
            && self.height >= 600
            && self.layer_count == self.expected_layer_count
            && self.expected_layer_count == 2
            && self.active_layer_sample[3] == 0xff
            && self.inactive_layer_sample[3] == 0xff
            && self.layer_checksum != 0
            && self.source_checksum_fold != 0
            && !self.renderer_started
    }
}

impl PaintPlan {
    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("paint_status={}", self.status),
            format!("paint_backend={}", self.backend),
            format!("renderer_started={}", self.renderer_started),
            format!("paint_step_count={}", self.steps.len()),
        ];

        lines.extend(self.steps.iter().map(|step| {
            format!(
                "paint order={} surface={} kind={} rect={},{},{},{} opacity={} blend={} effect={}",
                step.order,
                step.surface_id,
                step.kind.as_str(),
                step.rect.x,
                step.rect.y,
                step.rect.width,
                step.rect.height,
                step.opacity,
                step.blend_mode,
                step.effect
            )
        }));

        lines
    }

    pub fn is_ready(&self) -> bool {
        self.status == RENDERER_STATUS
            && self.backend == RENDER_BACKEND
            && !self.renderer_started
            && self.steps.len() == 7
            && self.steps.first().is_some_and(|step| {
                step.order == 0
                    && step.surface_id == "wallpaper"
                    && step.kind == DrawCommandKind::ImageLayer
                    && step.opacity == 255
                    && step.effect == "none"
            })
            && self.system_surface_steps_are_translucent()
            && self.orders_are_stable()
    }

    pub fn system_surface_steps_are_translucent(&self) -> bool {
        self.steps
            .iter()
            .filter(|step| step.kind == DrawCommandKind::SystemSurfacePanel)
            .all(|step| {
                step.opacity == 184
                    && step.blend_mode == "source-over"
                    && step.effect == "layered-system-surface"
            })
    }

    pub fn orders_are_stable(&self) -> bool {
        self.steps
            .iter()
            .enumerate()
            .all(|(expected, step)| step.order == expected)
    }
}

impl FramePlan {
    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("frame_status={}", self.status),
            format!("frame_backend={}", self.backend),
            format!("frame_size={}x{}", self.width, self.height),
            format!("pixel_format={}", self.pixel_format),
            format!("stride_bytes={}", self.stride_bytes),
            format!("buffer_bytes={}", self.buffer_bytes),
            format!("clear_color={}", self.clear_color),
            format!(
                "damage_rect={},{},{},{}",
                self.damage_rect.x,
                self.damage_rect.y,
                self.damage_rect.width,
                self.damage_rect.height
            ),
            format!("paint_step_count={}", self.paint_step_count),
            format!("renderer_started={}", self.renderer_started),
        ]
    }

    pub fn is_ready(&self) -> bool {
        self.status == RENDERER_STATUS
            && self.backend == RENDER_BACKEND
            && self.width >= 800
            && self.height >= 600
            && self.pixel_format == "rgba8888"
            && self.stride_bytes == self.width * 4
            && self.buffer_bytes == u64::from(self.stride_bytes) * u64::from(self.height)
            && self.clear_color == "#001725ff"
            && self.damage_rect
                == (Rect {
                    x: 0,
                    y: 0,
                    width: self.width,
                    height: self.height,
                })
            && self.paint_step_count == 7
            && !self.renderer_started
    }
}

impl FrameBufferProbe {
    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("buffer_status={}", self.status),
            format!("buffer_backend={}", self.backend),
            format!("frame_size={}x{}", self.width, self.height),
            format!("pixel_format={}", self.pixel_format),
            format!("buffer_bytes={}", self.buffer_bytes),
            format!("allocated_bytes={}", self.allocated_bytes),
            format!("clear_color={}", self.clear_color),
            format!("first_pixel={}", pixel_as_hex(self.first_pixel)),
            format!("last_pixel={}", pixel_as_hex(self.last_pixel)),
            format!("renderer_started={}", self.renderer_started),
        ]
    }

    pub fn is_ready(&self) -> bool {
        self.status == "allocated"
            && self.backend == RENDER_BACKEND
            && self.width >= 800
            && self.height >= 600
            && self.pixel_format == "rgba8888"
            && self.buffer_bytes == self.allocated_bytes as u64
            && self.clear_color == "#001725ff"
            && self.first_pixel == [0x00, 0x17, 0x25, 0xff]
            && self.last_pixel == [0x00, 0x17, 0x25, 0xff]
            && !self.renderer_started
    }
}

impl SoftwareRasterProbe {
    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("raster_status={}", self.status),
            format!("raster_backend={}", self.backend),
            format!("frame_size={}x{}", self.width, self.height),
            format!("pixel_format={}", self.pixel_format),
            format!("filled_rect_count={}", self.filled_rect_count),
            format!("expected_rect_count={}", self.expected_rect_count),
            format!("wallpaper_sample={}", pixel_as_hex(self.wallpaper_sample)),
            format!("surface_sample={}", pixel_as_hex(self.surface_sample)),
            format!("dock_sample={}", pixel_as_hex(self.dock_sample)),
            format!(
                "surface_border_sample={}",
                pixel_as_hex(self.surface_border_sample)
            ),
            format!(
                "surface_highlight_sample={}",
                pixel_as_hex(self.surface_highlight_sample)
            ),
            format!(
                "surface_corner_sample={}",
                pixel_as_hex(self.surface_corner_sample)
            ),
            format!(
                "surface_shadow_sample={}",
                pixel_as_hex(self.surface_shadow_sample)
            ),
            format!("raster_checksum={:016x}", self.raster_checksum),
            format!("surface_primitive_count={}", self.surface_primitive_count),
            format!("buffer_bytes={}", self.buffer_bytes),
            format!("renderer_started={}", self.renderer_started),
        ]
    }

    pub fn is_ready(&self) -> bool {
        self.status == "software-rasterized"
            && self.backend == RENDER_BACKEND
            && self.width >= 800
            && self.height >= 600
            && self.pixel_format == "rgba8888"
            && self.filled_rect_count == self.expected_rect_count
            && self.expected_rect_count == 7
            && self.wallpaper_sample == [0x04, 0x3b, 0x5c, 0xff]
            && self.surface_sample == [0x51, 0xac, 0xd2, 0xff]
            && self.dock_sample == [0x51, 0xac, 0xd2, 0xff]
            && self.surface_border_sample == [0x3d, 0x72, 0x8c, 0xff]
            && self.surface_highlight_sample == [0xa3, 0xd3, 0xe7, 0xff]
            && self.surface_corner_sample == [0x2a, 0x6c, 0x8c, 0xff]
            && self.surface_shadow_sample == [0x33, 0x86, 0xaa, 0xff]
            && self.raster_checksum == 0x7015_58d1_5395_21df
            && self.surface_primitive_count == 15
            && self.buffer_bytes == u64::from(self.width) * u64::from(self.height) * 4
            && !self.renderer_started
    }
}

impl RasterPpmExport {
    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("export_status={}", self.status),
            format!("export_backend={}", self.backend),
            format!("frame_size={}x{}", self.width, self.height),
            format!("export_format={}", self.format),
            format!("ppm_header={}", self.header.escape_default()),
            format!("export_bytes={}", self.byte_count),
            format!("export_checksum={:016x}", self.checksum),
            format!("renderer_started={}", self.renderer_started),
        ]
    }

    pub fn is_ready(&self) -> bool {
        self.status == "ppm-ready"
            && self.backend == RENDER_BACKEND
            && self.width == 1536
            && self.height == 1024
            && self.format == "ppm-p6-rgb888"
            && self.header == "P6\n1536 1024\n255\n"
            && self.byte_count == 4_718_609
            && self.bytes.len() == self.byte_count
            && self.checksum == 0xefdc_ba78_578c_2cd5
            && !self.renderer_started
    }
}

impl RasterPngExport {
    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("export_status={}", self.status),
            format!("export_backend={}", self.backend),
            format!("frame_size={}x{}", self.width, self.height),
            format!("export_format={}", self.format),
            format!("export_bytes={}", self.byte_count),
            format!("export_checksum={:016x}", self.checksum),
            format!("renderer_started={}", self.renderer_started),
        ]
    }

    pub fn is_ready(&self) -> bool {
        self.status == "png-ready"
            && self.backend == RENDER_BACKEND
            && self.width == 1536
            && self.height == 1024
            && self.format == "png-rgba8888"
            && self.byte_count == self.bytes.len()
            && self.byte_count == 6_293_028
            && self.checksum == 0x2cdb_1d86_a1ba_9300
            && !self.renderer_started
    }
}

impl RasterRgbaExport {
    pub fn to_png(&self) -> Vec<u8> {
        encode_png_rgba(self.width, self.height, &self.bytes)
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("export_status={}", self.status),
            format!("export_backend={}", self.backend),
            format!("frame_size={}x{}", self.width, self.height),
            format!("export_format={}", self.format),
            format!("export_bytes={}", self.byte_count),
            format!("export_checksum={:016x}", self.checksum),
            format!("renderer_started={}", self.renderer_started),
        ]
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.status, "rgba-ready" | "rgba-composited-preview-ready")
            && self.backend == RENDER_BACKEND
            && self.width == 1536
            && self.height == 1024
            && matches!(
                self.format,
                "raw-rgba8888" | "raw-rgba8888-composited-client-preview"
            )
            && self.byte_count == 6_291_456
            && self.byte_count == self.bytes.len()
            && self.checksum != 0
            && !self.renderer_started
    }
}

pub fn plan_static_scene(scene: &ShellScene) -> RenderPlan {
    RenderPlan {
        status: RENDERER_STATUS,
        backend: RENDER_BACKEND,
        width: scene.viewport.width,
        height: scene.viewport.height,
        commands: scene
            .surfaces
            .iter()
            .filter(|surface| surface.visible)
            .map(|surface| DrawCommand {
                surface_id: surface.id,
                kind: draw_kind_for_material(surface.material),
                rect: surface.rect,
                asset_count: scene
                    .assets
                    .iter()
                    .filter(|asset| asset.surface_id == surface.id)
                    .count(),
                material_token_count: scene
                    .material_tokens
                    .iter()
                    .filter(|token| token.surface_id == surface.id)
                    .count(),
                simulated: surface.material == MaterialKind::SystemSurface,
            })
            .collect(),
    }
}

pub fn plan_paint_steps(render_plan: &RenderPlan) -> PaintPlan {
    PaintPlan {
        status: RENDERER_STATUS,
        backend: RENDER_BACKEND,
        renderer_started: false,
        steps: render_plan
            .commands
            .iter()
            .enumerate()
            .map(|(order, command)| PaintStep {
                surface_id: command.surface_id,
                order,
                kind: command.kind,
                rect: command.rect,
                opacity: opacity_for_command(command),
                blend_mode: "source-over",
                effect: effect_for_command(command),
            })
            .collect(),
    }
}

fn draw_kind_for_material(material: MaterialKind) -> DrawCommandKind {
    match material {
        MaterialKind::Image => DrawCommandKind::ImageLayer,
        MaterialKind::SystemSurface => DrawCommandKind::SystemSurfacePanel,
        MaterialKind::IconGrid => DrawCommandKind::IconGroup,
    }
}

pub fn render_plan_for_static_scene(viewport: aqua_scene::Viewport) -> RenderPlan {
    let scene = aqua_scene::static_shell_scene(viewport);
    plan_static_scene(&scene)
}

pub fn plan_client_surface_sources(
    mut sources: Vec<ClientSurfaceSource>,
) -> ClientSurfaceSourcePlan {
    sources.sort_by(|left, right| {
        right
            .z_index
            .cmp(&left.z_index)
            .then_with(|| left.surface_id.cmp(right.surface_id))
    });

    ClientSurfaceSourcePlan {
        status: "client-surface-sources-ready",
        backend: RENDER_BACKEND,
        renderer_started: false,
        sources,
    }
}

pub fn plan_client_layer_paint_steps(
    source_plan: &ClientSurfaceSourcePlan,
) -> ClientLayerPaintPlan {
    ClientLayerPaintPlan {
        status: "client-layer-paint-ready",
        backend: RENDER_BACKEND,
        renderer_started: false,
        steps: source_plan
            .sources
            .iter()
            .enumerate()
            .map(|(order, source)| ClientLayerPaintStep {
                order,
                client_id: source.client_id,
                surface_id: source.surface_id,
                window_id: source.window_id,
                focused: source.focused,
                rect: source.rect,
                opacity: 255,
                blend_mode: "source-over",
                effect: "sampled-wl-shm-client-buffer",
                sample_checksum: source.sample_checksum,
                sample_pixel: source.sample_pixel,
                sample_grid: source.sample_grid,
                source_width: source.width,
                source_height: source.height,
                client_buffer_rgba: source.client_buffer_rgba.clone(),
            })
            .collect(),
    }
}

pub fn probe_client_layer_raster(
    viewport: aqua_scene::Viewport,
    paint_plan: &ClientLayerPaintPlan,
) -> ClientLayerRasterProbe {
    let frame_plan = frame_plan_for_static_scene(viewport);
    let mut buffer = vec![0_u8; frame_plan.buffer_bytes as usize];

    for pixel in buffer.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0x00, 0x17, 0x25, 0xff]);
    }

    for step in &paint_plan.steps {
        fill_client_layer_rect(
            &mut buffer,
            frame_plan.width,
            frame_plan.height,
            ClientLayerPixelSource::from_step(step),
        );
    }

    let active_sample = paint_plan
        .steps
        .first()
        .map(|step| sample_pixel(&buffer, viewport.width, step.rect.x + 8, step.rect.y + 8))
        .unwrap_or([0, 0, 0, 0]);
    let inactive_sample = paint_plan
        .steps
        .get(1)
        .map(|step| sample_pixel(&buffer, viewport.width, step.rect.x + 8, step.rect.y + 8))
        .unwrap_or([0, 0, 0, 0]);

    ClientLayerRasterProbe {
        status: "client-layer-rasterized",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        layer_count: paint_plan.steps.len(),
        expected_layer_count: 2,
        active_layer_sample: active_sample,
        inactive_layer_sample: inactive_sample,
        layer_checksum: checksum_bytes(&buffer),
        source_checksum_fold: paint_plan.steps.iter().fold(0_u64, |acc, step| {
            acc ^ step.sample_checksum.rotate_left(step.order as u32)
        }),
        renderer_started: false,
    }
}

pub fn paint_plan_for_static_scene(viewport: aqua_scene::Viewport) -> PaintPlan {
    let render_plan = render_plan_for_static_scene(viewport);
    plan_paint_steps(&render_plan)
}

pub fn frame_plan_for_static_scene(viewport: aqua_scene::Viewport) -> FramePlan {
    let paint_plan = paint_plan_for_static_scene(viewport);
    FramePlan {
        status: RENDERER_STATUS,
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        pixel_format: "rgba8888",
        stride_bytes: viewport.width * 4,
        buffer_bytes: u64::from(viewport.width) * u64::from(viewport.height) * 4,
        clear_color: "#001725ff",
        damage_rect: Rect {
            x: 0,
            y: 0,
            width: viewport.width,
            height: viewport.height,
        },
        paint_step_count: paint_plan.steps.len(),
        renderer_started: false,
    }
}

pub fn probe_frame_buffer_for_static_scene(viewport: aqua_scene::Viewport) -> FrameBufferProbe {
    let frame_plan = frame_plan_for_static_scene(viewport);
    let clear_pixel = [0x00, 0x17, 0x25, 0xff];
    let mut buffer = vec![0_u8; frame_plan.buffer_bytes as usize];

    for pixel in buffer.chunks_exact_mut(4) {
        pixel.copy_from_slice(&clear_pixel);
    }

    let first_pixel = buffer
        .get(0..4)
        .and_then(|pixel| pixel.try_into().ok())
        .unwrap_or([0, 0, 0, 0]);
    let last_pixel = buffer
        .get(buffer.len().saturating_sub(4)..buffer.len())
        .and_then(|pixel| pixel.try_into().ok())
        .unwrap_or([0, 0, 0, 0]);

    FrameBufferProbe {
        status: "allocated",
        backend: frame_plan.backend,
        width: frame_plan.width,
        height: frame_plan.height,
        pixel_format: frame_plan.pixel_format,
        buffer_bytes: frame_plan.buffer_bytes,
        allocated_bytes: buffer.len(),
        clear_color: frame_plan.clear_color,
        first_pixel,
        last_pixel,
        renderer_started: false,
    }
}

pub fn probe_software_raster_for_static_scene(
    viewport: aqua_scene::Viewport,
) -> SoftwareRasterProbe {
    let image = rasterize_static_scene(viewport);
    SoftwareRasterProbe {
        status: "software-rasterized",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        pixel_format: "rgba8888",
        filled_rect_count: image.filled_rect_count,
        expected_rect_count: image.expected_rect_count,
        wallpaper_sample: sample_pixel(&image.rgba, viewport.width, 1400, 500),
        surface_sample: sample_pixel(&image.rgba, viewport.width, 300, 300),
        dock_sample: sample_pixel(&image.rgba, viewport.width, 768, 960),
        surface_border_sample: sample_pixel(&image.rgba, viewport.width, 24, 60),
        surface_highlight_sample: sample_pixel(&image.rgba, viewport.width, 300, 61),
        surface_corner_sample: sample_pixel(&image.rgba, viewport.width, 25, 61),
        surface_shadow_sample: sample_pixel(&image.rgba, viewport.width, 300, 578),
        raster_checksum: checksum_bytes(&image.rgba),
        surface_primitive_count: image.surface_primitive_count,
        buffer_bytes: image.rgba.len() as u64,
        renderer_started: false,
    }
}

pub fn export_software_raster_ppm_for_static_scene(
    viewport: aqua_scene::Viewport,
) -> RasterPpmExport {
    let image = rasterize_static_scene(viewport);
    let header = format!("P6\n{} {}\n255\n", viewport.width, viewport.height);
    let mut bytes = Vec::with_capacity(header.len() + image.rgba.len() / 4 * 3);
    bytes.extend_from_slice(header.as_bytes());

    for pixel in image.rgba.chunks_exact(4) {
        bytes.extend_from_slice(&pixel[0..3]);
    }

    let byte_count = bytes.len();
    let checksum = checksum_bytes(&bytes);

    RasterPpmExport {
        status: "ppm-ready",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        format: "ppm-p6-rgb888",
        header,
        bytes,
        byte_count,
        checksum,
        renderer_started: false,
    }
}

pub fn export_software_raster_png_for_static_scene(
    viewport: aqua_scene::Viewport,
) -> RasterPngExport {
    let image = rasterize_static_scene(viewport);
    let bytes = encode_png_rgba(viewport.width, viewport.height, &image.rgba);
    let byte_count = bytes.len();
    let checksum = checksum_bytes(&bytes);

    RasterPngExport {
        status: "png-ready",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        format: "png-rgba8888",
        bytes,
        byte_count,
        checksum,
        renderer_started: false,
    }
}

pub fn export_composited_preview_png_with_client_layers(
    viewport: aqua_scene::Viewport,
    client_paint_plan: &ClientLayerPaintPlan,
) -> RasterPngExport {
    let mut image = rasterize_static_scene(viewport);

    for step in &client_paint_plan.steps {
        fill_client_layer_rect(
            &mut image.rgba,
            viewport.width,
            viewport.height,
            ClientLayerPixelSource::from_step(step),
        );
    }

    let bytes = encode_png_rgba(viewport.width, viewport.height, &image.rgba);
    let byte_count = bytes.len();
    let checksum = checksum_bytes(&bytes);

    RasterPngExport {
        status: "png-composited-preview-ready",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        format: "png-rgba8888-composited-client-preview",
        bytes,
        byte_count,
        checksum,
        renderer_started: false,
    }
}

pub fn export_composited_preview_rgba_with_client_layers(
    viewport: aqua_scene::Viewport,
    client_paint_plan: &ClientLayerPaintPlan,
) -> RasterRgbaExport {
    let mut image = rasterize_static_scene(viewport);

    for step in &client_paint_plan.steps {
        fill_client_layer_rect(
            &mut image.rgba,
            viewport.width,
            viewport.height,
            ClientLayerPixelSource::from_step(step),
        );
    }

    let byte_count = image.rgba.len();
    let checksum = checksum_bytes(&image.rgba);

    RasterRgbaExport {
        status: "rgba-composited-preview-ready",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        format: "raw-rgba8888-composited-client-preview",
        bytes: image.rgba,
        byte_count,
        checksum,
        renderer_started: false,
    }
}

pub fn export_composited_preview_rgba_with_wallpaper_and_client_layers(
    viewport: aqua_scene::Viewport,
    wallpaper_width: u32,
    wallpaper_height: u32,
    wallpaper_rgba: &[u8],
    client_paint_plan: &ClientLayerPaintPlan,
) -> Result<RasterRgbaExport, String> {
    let expected_bytes = wallpaper_width as usize * wallpaper_height as usize * 4;
    if wallpaper_width == 0 || wallpaper_height == 0 || wallpaper_rgba.len() != expected_bytes {
        return Err("runtime wallpaper must be non-empty rgba8888".to_string());
    }
    let mut image = rasterize_static_scene_with_wallpaper(
        viewport,
        Some(RgbaImageSource {
            width: wallpaper_width,
            height: wallpaper_height,
            rgba: wallpaper_rgba,
        }),
    );

    for step in &client_paint_plan.steps {
        fill_client_layer_rect(
            &mut image.rgba,
            viewport.width,
            viewport.height,
            ClientLayerPixelSource::from_step(step),
        );
    }

    let byte_count = image.rgba.len();
    let checksum = checksum_bytes(&image.rgba);
    Ok(RasterRgbaExport {
        status: "rgba-runtime-wallpaper-composited-preview-ready",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        format: "raw-rgba8888-runtime-wallpaper-composited-client-preview",
        bytes: image.rgba,
        byte_count,
        checksum,
        renderer_started: false,
    })
}

pub fn export_runtime_desktop_rgba_with_launcher(
    viewport: aqua_scene::Viewport,
    wallpaper_width: u32,
    wallpaper_height: u32,
    wallpaper_rgba: &[u8],
    client_paint_plan: &ClientLayerPaintPlan,
    launcher: &LauncherState,
) -> Result<(RasterRgbaExport, LauncherOverlayProbe), String> {
    export_runtime_desktop_rgba_with_launcher_and_theme(
        viewport,
        wallpaper_width,
        wallpaper_height,
        wallpaper_rgba,
        client_paint_plan,
        launcher,
        AquaTheme::LightWhite,
    )
}

pub fn export_runtime_desktop_rgba_with_launcher_and_theme(
    viewport: aqua_scene::Viewport,
    wallpaper_width: u32,
    wallpaper_height: u32,
    wallpaper_rgba: &[u8],
    client_paint_plan: &ClientLayerPaintPlan,
    launcher: &LauncherState,
    theme: AquaTheme,
) -> Result<(RasterRgbaExport, LauncherOverlayProbe), String> {
    let expected_bytes = wallpaper_width as usize * wallpaper_height as usize * 4;
    if wallpaper_width == 0 || wallpaper_height == 0 || wallpaper_rgba.len() != expected_bytes {
        return Err("runtime wallpaper must be non-empty rgba8888".to_string());
    }

    let mut scene = aqua_scene::static_shell_scene(viewport);
    scene.set_surface_visible(SurfaceKind::Launcher, false);
    let mut image = rasterize_scene_with_wallpaper(
        &scene,
        Some(RgbaImageSource {
            width: wallpaper_width,
            height: wallpaper_height,
            rgba: wallpaper_rgba,
        }),
    );
    for step in &client_paint_plan.steps {
        fill_client_layer_rect(
            &mut image.rgba,
            viewport.width,
            viewport.height,
            ClientLayerPixelSource::from_step(step),
        );
    }

    let probe = draw_launcher_overlay(&mut image.rgba, viewport, launcher, theme);
    let byte_count = image.rgba.len();
    let checksum = checksum_bytes(&image.rgba);
    Ok((
        RasterRgbaExport {
            status: "rgba-runtime-desktop-launcher-ready",
            backend: RENDER_BACKEND,
            width: viewport.width,
            height: viewport.height,
            format: "raw-rgba8888-runtime-desktop-launcher",
            bytes: image.rgba,
            byte_count,
            checksum,
            renderer_started: false,
        },
        probe,
    ))
}

pub fn render_launcher_overlay_rgba_with_theme(
    viewport: aqua_scene::Viewport,
    launcher: &LauncherState,
    theme: AquaTheme,
) -> (Vec<u8>, LauncherOverlayProbe) {
    let mut rgba = vec![
        0_u8;
        viewport
            .width
            .saturating_mul(viewport.height)
            .saturating_mul(4) as usize
    ];
    let probe = draw_launcher_overlay(&mut rgba, viewport, launcher, theme);
    (rgba, probe)
}

pub fn export_software_raster_rgba_for_static_scene(
    viewport: aqua_scene::Viewport,
) -> RasterRgbaExport {
    let image = rasterize_static_scene(viewport);
    let byte_count = image.rgba.len();
    let checksum = checksum_bytes(&image.rgba);

    RasterRgbaExport {
        status: "rgba-ready",
        backend: RENDER_BACKEND,
        width: viewport.width,
        height: viewport.height,
        format: "raw-rgba8888",
        bytes: image.rgba,
        byte_count,
        checksum,
        renderer_started: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SoftwareRasterImage {
    rgba: Vec<u8>,
    filled_rect_count: usize,
    expected_rect_count: usize,
    surface_primitive_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct RgbaImageSource<'a> {
    width: u32,
    height: u32,
    rgba: &'a [u8],
}

fn rasterize_static_scene(viewport: aqua_scene::Viewport) -> SoftwareRasterImage {
    rasterize_static_scene_with_wallpaper(viewport, None)
}

fn rasterize_static_scene_with_wallpaper(
    viewport: aqua_scene::Viewport,
    wallpaper: Option<RgbaImageSource<'_>>,
) -> SoftwareRasterImage {
    let scene = aqua_scene::static_shell_scene(viewport);
    rasterize_scene_with_wallpaper(&scene, wallpaper)
}

fn rasterize_scene_with_wallpaper(
    scene: &ShellScene,
    wallpaper: Option<RgbaImageSource<'_>>,
) -> SoftwareRasterImage {
    let paint_plan = plan_paint_steps(&plan_static_scene(scene));
    let frame_plan = FramePlan {
        status: RENDERER_STATUS,
        backend: RENDER_BACKEND,
        width: scene.viewport.width,
        height: scene.viewport.height,
        pixel_format: "rgba8888",
        stride_bytes: scene.viewport.width * 4,
        buffer_bytes: u64::from(scene.viewport.width) * u64::from(scene.viewport.height) * 4,
        clear_color: "#001725ff",
        damage_rect: Rect {
            x: 0,
            y: 0,
            width: scene.viewport.width,
            height: scene.viewport.height,
        },
        paint_step_count: paint_plan.steps.len(),
        renderer_started: false,
    };
    let mut buffer = vec![0_u8; frame_plan.buffer_bytes as usize];
    let mut filled_rect_count = 0;
    let mut surface_primitive_count = 0;

    for step in &paint_plan.steps {
        if step.kind == DrawCommandKind::ImageLayer {
            if let Some(source) = wallpaper {
                fill_rgba_image_rect(
                    &mut buffer,
                    frame_plan.width,
                    frame_plan.height,
                    step.rect,
                    source,
                    step.opacity,
                );
                filled_rect_count += 1;
                continue;
            }
        }
        let source = raster_color_for_step(step);
        fill_rect(
            &mut buffer,
            frame_plan.width,
            frame_plan.height,
            step.rect,
            source,
            step.opacity,
        );
        filled_rect_count += 1;

        if step.kind == DrawCommandKind::SystemSurfacePanel {
            surface_primitive_count += draw_system_surface_primitives(
                &mut buffer,
                frame_plan.width,
                frame_plan.height,
                step.rect,
            );
        }
    }

    SoftwareRasterImage {
        rgba: buffer,
        filled_rect_count,
        expected_rect_count: paint_plan.steps.len(),
        surface_primitive_count,
    }
}

fn draw_launcher_overlay(
    buffer: &mut [u8],
    viewport: aqua_scene::Viewport,
    launcher: &LauncherState,
    theme: AquaTheme,
) -> LauncherOverlayProbe {
    if !launcher.is_open() {
        return LauncherOverlayProbe {
            rendered: false,
            mode: launcher.mode().id(),
            category_count: LauncherCategory::ALL.len(),
            visible_app_count: 0,
            selected_index: 0,
            query_visible: false,
            primitive_count: 0,
        };
    }

    let overview = (launcher.mode() == LauncherMode::Applications)
        .then(|| launcher.application_overview(viewport.width, viewport.height));
    let global_search = (launcher.mode() == LauncherMode::Search)
        .then(|| launcher.global_search(viewport.width, viewport.height));
    let bounds = launcher.panel_bounds(viewport.width, viewport.height);
    let panel = overview.map_or_else(
        || {
            global_search.map_or(
                Rect {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
                |search| search.rect,
            )
        },
        |overview| overview.rect,
    );
    let palette = shell_palette(theme);
    let mut primitives = 0;
    if let Some(overview) = overview {
        primitives += draw_application_overview(
            buffer,
            viewport.width,
            viewport.height,
            overview,
            theme,
            OutputScale::One,
        );
    } else if let Some(search) = global_search {
        primitives += draw_global_search(
            buffer,
            viewport.width,
            viewport.height,
            search,
            theme,
            OutputScale::One,
        );
    } else {
        fill_rect(
            buffer,
            viewport.width,
            viewport.height,
            panel,
            palette.surface,
            244,
        );
        draw_rect_outline(
            buffer,
            viewport.width,
            viewport.height,
            panel,
            palette.border,
            255,
        );
        draw_bitmap_text(
            buffer,
            (viewport.width, viewport.height),
            (panel.x + 24, panel.y + 24),
            "SEARCH",
            palette.text,
            2,
        );
        primitives += 2;
    }

    let search = launcher.search_field(viewport.width, viewport.height);
    primitives += draw_search_field(
        buffer,
        viewport.width,
        viewport.height,
        search,
        theme,
        OutputScale::One,
    );

    let visible_apps = launcher.visible_apps();
    match launcher.mode() {
        LauncherMode::Applications => {
            for (index, app) in visible_apps.iter().take(6).enumerate() {
                let Some(cell) =
                    launcher.application_grid_cell(index, viewport.width, viewport.height)
                else {
                    continue;
                };
                let slots = cell.slots();
                primitives += draw_grid_cell(buffer, viewport.width, viewport.height, cell, theme);
                draw_app_icon(
                    buffer,
                    viewport.width,
                    viewport.height,
                    slots.icon.x,
                    slots.icon.y,
                    index,
                );
                draw_fitted_bitmap_text(
                    buffer,
                    (viewport.width, viewport.height),
                    slots.primary,
                    app.name,
                    palette.text,
                    FittedTextOptions::new(TextRole::Control, OutputScale::One, false),
                );
                draw_fitted_bitmap_text(
                    buffer,
                    (viewport.width, viewport.height),
                    slots.secondary,
                    app.description,
                    palette.secondary_text,
                    FittedTextOptions::new(TextRole::Control, OutputScale::One, false),
                );
                primitives += 2;
            }
        }
        LauncherMode::Search => {
            let search_layout = global_search.expect("search mode has shared layout");
            for (index, app) in visible_apps
                .iter()
                .take(search_layout.visible_result_count())
                .enumerate()
            {
                let Some(row) = launcher.search_result_row(index, viewport.width, viewport.height)
                else {
                    continue;
                };
                if index == launcher.selected_index() {
                    fill_rect(
                        buffer,
                        viewport.width,
                        viewport.height,
                        row.rect,
                        palette.selection,
                        230,
                    );
                    primitives += 1;
                }
                let slots = row.slots();
                draw_app_icon(
                    buffer,
                    viewport.width,
                    viewport.height,
                    slots.leading.x,
                    row.rect.y + 6,
                    index,
                );
                draw_bitmap_text(
                    buffer,
                    (viewport.width, viewport.height),
                    (slots.label.x, row.rect.y + 10),
                    app.name,
                    palette.text,
                    1,
                );
                draw_bitmap_text(
                    buffer,
                    (viewport.width, viewport.height),
                    (slots.label.x, row.rect.y + 29),
                    app.description,
                    palette.secondary_text,
                    1,
                );
                primitives += 1;
            }
            for (index, label) in ["OPEN APPLICATIONS", "SYSTEM SETTINGS", "BROWSE FILES"]
                .iter()
                .enumerate()
            {
                let Some(action) =
                    launcher.search_quick_action_button(index, viewport.width, viewport.height)
                else {
                    continue;
                };
                fill_rect(
                    buffer,
                    viewport.width,
                    viewport.height,
                    action.rect,
                    palette.elevated,
                    220,
                );
                draw_category_icon(
                    buffer,
                    viewport.width,
                    viewport.height,
                    action.rect.x + 12,
                    action.rect.y + 17,
                    index,
                );
                draw_bitmap_text(
                    buffer,
                    (viewport.width, viewport.height),
                    (action.rect.x + 42, action.rect.y + 18),
                    label,
                    palette.text,
                    1,
                );
                primitives += 2;
            }
        }
    }

    LauncherOverlayProbe {
        rendered: true,
        mode: launcher.mode().id(),
        category_count: LauncherCategory::ALL.len(),
        visible_app_count: visible_apps.len(),
        selected_index: launcher.selected_index(),
        query_visible: !launcher.query().is_empty(),
        primitive_count: primitives,
    }
}

fn draw_category_icon(buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32, index: usize) {
    let colors = [
        [0xff, 0xe1, 0x69, 0xff],
        [0x62, 0xd8, 0xff, 0xff],
        [0x72, 0xea, 0xb5, 0xff],
        [0xff, 0x9b, 0xd2, 0xff],
        [0x65, 0xb5, 0xff, 0xff],
        [0xb7, 0x9b, 0xff, 0xff],
        [0x83, 0xe7, 0xef, 0xff],
        [0x6e, 0xc9, 0xff, 0xff],
        [0xc5, 0xe7, 0xff, 0xff],
    ];
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x,
            y,
            width: 16,
            height: 16,
        },
        colors[index % colors.len()],
        235,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: x + 4,
            y: y + 4,
            width: 8,
            height: 8,
        },
        [0x08, 0x4a, 0x6d, 0xff],
        190,
    );
}

fn draw_app_icon(buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32, index: usize) {
    let colors = [
        [0x4f, 0xc9, 0xff, 0xff],
        [0x38, 0x8f, 0xf0, 0xff],
        [0x1c, 0x31, 0x48, 0xff],
        [0x85, 0xd7, 0xe9, 0xff],
        [0xe8, 0xb5, 0x54, 0xff],
        [0x36, 0xb9, 0xf3, 0xff],
    ];
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x,
            y,
            width: 38,
            height: 38,
        },
        colors[index % colors.len()],
        245,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: x + 3,
            y: y + 3,
            width: 32,
            height: 2,
        },
        [0xff, 0xff, 0xff, 0xff],
        180,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: x + 10,
            y: y + 11,
            width: 18,
            height: 16,
        },
        [0xee, 0xfb, 0xff, 0xff],
        150,
    );
}

fn draw_window_controls(buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32) {
    for (index, color) in [
        [0xff, 0x6f, 0x67, 0xff],
        [0xff, 0xd0, 0x59, 0xff],
        [0x65, 0xd4, 0x73, 0xff],
    ]
    .into_iter()
    .enumerate()
    {
        draw_window_control(
            buffer,
            width,
            height,
            Rect {
                x: x + index as u32 * 22,
                y,
                width: 14,
                height: 14,
            },
            color,
        );
    }
}

fn draw_window_control(buffer: &mut [u8], width: u32, height: u32, rect: Rect, color: [u8; 4]) {
    let center_x = rect.x + rect.width / 2;
    let center_y = rect.y + rect.height / 2;
    fill_transparent_circle(
        buffer,
        width,
        height,
        center_x,
        center_y,
        7,
        [0xa7, 0xb2, 0xbf, 0xff],
    );
    fill_transparent_circle(buffer, width, height, center_x, center_y, 6, color);
    fill_transparent_circle(
        buffer,
        width,
        height,
        center_x.saturating_sub(2),
        center_y.saturating_sub(2),
        2,
        [0xff, 0xff, 0xff, 0x78],
    );
}

pub(crate) fn draw_window_frame(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    frame: WindowFrame<'_>,
    palette: WindowChromePalette,
) -> usize {
    fill_rect(buffer, width, height, frame.rect, palette.surface, 255);
    fill_rect(
        buffer,
        width,
        height,
        frame.titlebar_rect(),
        palette.titlebar,
        255,
    );
    fill_rect(
        buffer,
        width,
        height,
        frame.separator_rect(),
        palette.border,
        255,
    );
    for (control, color) in [
        (WindowControl::Close, [0xff, 0x6f, 0x67, 0xff]),
        (WindowControl::Minimize, [0xff, 0xd0, 0x59, 0xff]),
        (WindowControl::Maximize, [0x65, 0xd4, 0x73, 0xff]),
    ] {
        draw_window_control(buffer, width, height, frame.control_rect(control), color);
    }
    draw_fitted_bitmap_text(
        buffer,
        (width, height),
        frame.title_rect(),
        frame.title,
        palette.text,
        FittedTextOptions::new(TextRole::Body, OutputScale::One, false),
    );
    draw_rect_outline(buffer, width, height, frame.rect, palette.border, 255);
    8
}

fn draw_bright_window_titlebar(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    titlebar: Rect,
    title: &str,
    palette: WindowChromePalette,
) -> usize {
    fill_rect(buffer, width, height, titlebar, palette.titlebar, 255);
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: titlebar.x,
            y: titlebar.bottom().saturating_sub(1),
            width: titlebar.width,
            height: 1,
        },
        palette.border,
        255,
    );
    draw_window_controls(
        buffer,
        width,
        height,
        titlebar.x + 18,
        titlebar.y + titlebar.height.saturating_sub(14) / 2,
    );
    draw_fitted_bitmap_text(
        buffer,
        (width, height),
        Rect {
            x: titlebar.x + 92,
            y: titlebar.y,
            width: titlebar.width.saturating_sub(112),
            height: titlebar.height,
        },
        title,
        palette.text,
        FittedTextOptions::new(TextRole::Body, OutputScale::One, false),
    );
    6
}

fn draw_sidebar_icon(buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32, index: usize) {
    let color = [0x75, 0xdc, 0xf7, 0xff];
    if index == 4 {
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: x + 3,
                y: y + 4,
                width: 14,
                height: 14,
            },
            color,
            220,
        );
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x,
                y: y + 1,
                width: 20,
                height: 3,
            },
            color,
            220,
        );
    } else {
        draw_folder_icon(buffer, width, height, x, y + 1, color);
    }
}

fn draw_folder_icon(buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32, color: [u8; 4]) {
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x,
            y: y + 5,
            width: 42,
            height: 28,
        },
        color,
        238,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: x + 3,
            y,
            width: 17,
            height: 9,
        },
        color,
        238,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: x + 3,
            y: y + 7,
            width: 36,
            height: 3,
        },
        [0xe9, 0xfb, 0xff, 0xff],
        150,
    );
}

fn draw_file_icon(buffer: &mut [u8], width: u32, height: u32, x: u32, y: u32) {
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x,
            y,
            width: 30,
            height: 38,
        },
        [0xe8, 0xfa, 0xff, 0xff],
        235,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: x + 20,
            y,
            width: 10,
            height: 10,
        },
        [0x72, 0xc9, 0xe6, 0xff],
        220,
    );
    for row in 0..3 {
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: x + 6,
                y: y + 16 + row * 6,
                width: 18,
                height: 2,
            },
            [0x4c, 0x91, 0xab, 0xff],
            190,
        );
    }
}

fn draw_bitmap_text(
    buffer: &mut [u8],
    canvas: (u32, u32),
    origin: (u32, u32),
    text: &str,
    color: [u8; 4],
    scale: u32,
) {
    if let Some(service) = text_service() {
        if let Ok(mut service) = service.lock() {
            draw_shaped_text(buffer, canvas, origin, text, color, scale, &mut service);
            return;
        }
    }
    draw_legacy_bitmap_text(buffer, canvas, origin, text, color, scale);
}

fn draw_shaped_text(
    buffer: &mut [u8],
    canvas: (u32, u32),
    origin: (u32, u32),
    text: &str,
    color: [u8; 4],
    scale: u32,
    service: &mut TextService,
) {
    let (role, output_scale) = if scale > 1 {
        (TextRole::Caption, OutputScale::Two)
    } else {
        (TextRole::Body, OutputScale::One)
    };
    let line = service.shape_line(text, role, output_scale);
    draw_shaped_line(buffer, canvas, origin, color, &line, service, None);
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextDrawOutcome {
    original_width: f32,
    rendered_width: f32,
    truncated: bool,
    fallback_glyphs: usize,
    missing_glyphs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FittedTextOptions {
    role: TextRole,
    scale: OutputScale,
    centered: bool,
}

impl FittedTextOptions {
    const fn new(role: TextRole, scale: OutputScale, centered: bool) -> Self {
        Self {
            role,
            scale,
            centered,
        }
    }
}

fn draw_fitted_bitmap_text(
    buffer: &mut [u8],
    canvas: (u32, u32),
    rect: Rect,
    text: &str,
    color: [u8; 4],
    options: FittedTextOptions,
) -> TextDrawOutcome {
    let Some(service) = text_service() else {
        draw_legacy_bitmap_text(buffer, canvas, (rect.x, rect.y), text, color, 1);
        return TextDrawOutcome {
            original_width: 0.0,
            rendered_width: 0.0,
            truncated: false,
            fallback_glyphs: 0,
            missing_glyphs: text
                .chars()
                .filter(|character| !character.is_ascii())
                .count(),
        };
    };
    let Ok(mut service) = service.lock() else {
        return TextDrawOutcome {
            original_width: 0.0,
            rendered_width: 0.0,
            truncated: false,
            fallback_glyphs: 0,
            missing_glyphs: text.chars().count(),
        };
    };
    let original = service.shape_line(text, options.role, options.scale);
    let fitted = service.ellipsize(text, options.role, options.scale, rect.width as f32);
    let rendered_width = fitted.width;
    let x = if options.centered {
        rect.x + rect.width.saturating_sub(rendered_width.ceil() as u32) / 2
    } else {
        rect.x
    };
    let y = rect.y + rect.height.saturating_sub(fitted.height.ceil() as u32) / 2;
    draw_shaped_line(
        buffer,
        canvas,
        (x, y),
        color,
        &fitted,
        &mut service,
        Some(rect),
    );
    TextDrawOutcome {
        original_width: original.width,
        rendered_width,
        truncated: fitted.text != text,
        fallback_glyphs: fitted.fallback_glyphs,
        missing_glyphs: fitted.missing_glyphs,
    }
}

fn draw_shaped_line(
    buffer: &mut [u8],
    canvas: (u32, u32),
    origin: (u32, u32),
    color: [u8; 4],
    line: &ShapedLine,
    service: &mut TextService,
    clip: Option<Rect>,
) {
    let (width, height) = canvas;
    let baseline = origin.1 as i32 + line.baseline.ceil() as i32;
    let mut cursor = origin.0 as f32;
    for run in &line.runs {
        for shaped in &run.glyphs {
            let key = GlyphCacheKey {
                font_id: shaped.font_id,
                glyph_id: shaped.glyph_id,
                role: line.role,
                scale: line.scale,
                mode: RenderingMode::Grayscale,
            };
            let Some(glyph) = service.rasterize(key) else {
                cursor += shaped.x_advance;
                continue;
            };
            let glyph_x = (cursor + shaped.x_offset).floor() as i32 + glyph.metrics.xmin;
            let glyph_y = baseline
                - glyph.metrics.height as i32
                - glyph.metrics.ymin
                - shaped.y_offset.round() as i32;
            for row in 0..glyph.metrics.height {
                for column in 0..glyph.metrics.width {
                    let alpha = glyph.coverage[row * glyph.metrics.width + column];
                    if alpha == 0 {
                        continue;
                    }
                    let x = glyph_x + column as i32;
                    let y = glyph_y + row as i32;
                    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                        continue;
                    }
                    if clip.is_some_and(|clip| {
                        x < clip.x as i32
                            || y < clip.y as i32
                            || x >= clip.right() as i32
                            || y >= clip.bottom() as i32
                    }) {
                        continue;
                    }
                    let offset = ((y as u32 * width + x as u32) * 4) as usize;
                    let destination = [
                        buffer[offset],
                        buffer[offset + 1],
                        buffer[offset + 2],
                        buffer[offset + 3],
                    ];
                    buffer[offset..offset + 4].copy_from_slice(&blend_source_over(
                        color,
                        destination,
                        alpha,
                    ));
                }
            }
            cursor += shaped.x_advance;
        }
    }
}

const fn inset_rect(rect: Rect, horizontal: u32, vertical: u32) -> Rect {
    Rect {
        x: rect.x + horizontal,
        y: rect.y + vertical,
        width: rect.width.saturating_sub(horizontal.saturating_mul(2)),
        height: rect.height.saturating_sub(vertical.saturating_mul(2)),
    }
}

fn draw_legacy_bitmap_text(
    buffer: &mut [u8],
    canvas: (u32, u32),
    origin: (u32, u32),
    text: &str,
    color: [u8; 4],
    scale: u32,
) {
    let (width, height) = canvas;
    let (x, y) = origin;
    let mut cursor = x;
    for character in text.to_ascii_uppercase().chars() {
        let glyph = bitmap_glyph(character);
        for (row, bits) in glyph.iter().copied().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    fill_rect(
                        buffer,
                        width,
                        height,
                        Rect {
                            x: cursor + column * scale,
                            y: y + row as u32 * scale,
                            width: scale,
                            height: scale,
                        },
                        color,
                        235,
                    );
                }
            }
        }
        cursor = cursor.saturating_add(6 * scale);
    }
}

fn bitmap_glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        _ => [0; 7],
    }
}

fn fill_rgba_image_rect(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rect: Rect,
    source: RgbaImageSource<'_>,
    opacity: u8,
) {
    let max_x = rect.x.saturating_add(rect.width).min(width);
    let max_y = rect.y.saturating_add(rect.height).min(height);
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    for y in rect.y..max_y {
        let source_y = ((y - rect.y) as u64 * source.height as u64 / rect.height as u64)
            .min(source.height.saturating_sub(1) as u64) as u32;
        for x in rect.x..max_x {
            let source_x = ((x - rect.x) as u64 * source.width as u64 / rect.width as u64)
                .min(source.width.saturating_sub(1) as u64) as u32;
            let source_offset = ((source_y * source.width + source_x) * 4) as usize;
            let target_offset = ((y * width + x) * 4) as usize;
            let source = [
                source.rgba[source_offset],
                source.rgba[source_offset + 1],
                source.rgba[source_offset + 2],
                source.rgba[source_offset + 3],
            ];
            let destination = [
                buffer[target_offset],
                buffer[target_offset + 1],
                buffer[target_offset + 2],
                buffer[target_offset + 3],
            ];
            let effective_opacity = ((u16::from(source[3]) * u16::from(opacity) + 127) / 255) as u8;
            buffer[target_offset..target_offset + 4].copy_from_slice(&blend_source_over(
                source,
                destination,
                effective_opacity,
            ));
        }
    }
}

fn pixel_as_hex(pixel: [u8; 4]) -> String {
    format!(
        "{:02x},{:02x},{:02x},{:02x}",
        pixel[0], pixel[1], pixel[2], pixel[3]
    )
}

fn sample_grid_as_hex(sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS]) -> String {
    sample_grid
        .into_iter()
        .map(pixel_as_hex)
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
fn solid_sample_grid(pixel: [u8; 4]) -> [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS] {
    [pixel; CLIENT_SAMPLE_GRID_PIXELS]
}

#[cfg(test)]
fn gradient_client_buffer_rgba(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&[
                ((x * 255) / width) as u8,
                ((y * 255) / height) as u8,
                0x7f,
                0xff,
            ]);
        }
    }
    pixels
}

fn raster_color_for_step(step: &PaintStep) -> [u8; 4] {
    match step.kind {
        DrawCommandKind::ImageLayer => [0x04, 0x3b, 0x5c, 0xff],
        DrawCommandKind::SystemSurfacePanel => [0x6f, 0xd7, 0xff, 0xff],
        DrawCommandKind::IconGroup => [0xb9, 0xf6, 0xff, 0xff],
    }
}

struct ClientLayerPixelSource<'a> {
    rect: Rect,
    source_width: u32,
    source_height: u32,
    source_rgba: &'a [u8],
    sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    opacity: u8,
}

impl<'a> ClientLayerPixelSource<'a> {
    fn from_step(step: &'a ClientLayerPaintStep) -> Self {
        Self {
            rect: step.rect,
            source_width: step.source_width,
            source_height: step.source_height,
            source_rgba: &step.client_buffer_rgba,
            sample_grid: step.sample_grid,
            opacity: step.opacity,
        }
    }
}

fn fill_rect(buffer: &mut [u8], width: u32, height: u32, rect: Rect, source: [u8; 4], opacity: u8) {
    let max_x = rect.x.saturating_add(rect.width).min(width);
    let max_y = rect.y.saturating_add(rect.height).min(height);

    for y in rect.y..max_y {
        for x in rect.x..max_x {
            let offset = ((y * width + x) * 4) as usize;
            let destination = [
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ];
            let blended = blend_source_over(source, destination, opacity);
            buffer[offset..offset + 4].copy_from_slice(&blended);
        }
    }
}

fn fill_rounded_rect(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rect: Rect,
    radius: u32,
    source: [u8; 4],
    opacity: u8,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    let max_x = rect.right().min(width);
    let max_y = rect.bottom().min(height);
    let radius = radius.min(rect.width / 2).min(rect.height / 2);
    let radius_squared = i64::from(radius) * i64::from(radius);
    for y in rect.y..max_y {
        for x in rect.x..max_x {
            let local_x = x - rect.x;
            let local_y = y - rect.y;
            let corner_x = if local_x < radius {
                radius - local_x
            } else if local_x >= rect.width.saturating_sub(radius) {
                local_x - rect.width.saturating_sub(radius).saturating_sub(1)
            } else {
                0
            };
            let corner_y = if local_y < radius {
                radius - local_y
            } else if local_y >= rect.height.saturating_sub(radius) {
                local_y - rect.height.saturating_sub(radius).saturating_sub(1)
            } else {
                0
            };
            if corner_x > 0
                && corner_y > 0
                && i64::from(corner_x) * i64::from(corner_x)
                    + i64::from(corner_y) * i64::from(corner_y)
                    > radius_squared
            {
                continue;
            }
            let offset = ((y * width + x) * 4) as usize;
            let destination = [
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ];
            buffer[offset..offset + 4].copy_from_slice(&blend_source_over(
                source,
                destination,
                opacity,
            ));
        }
    }
}

fn draw_rect_outline(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rect: Rect,
    color: [u8; 4],
    opacity: u8,
) {
    if rect.width < 2 || rect.height < 2 {
        return;
    }
    for edge in [
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 2,
        },
        Rect {
            x: rect.x,
            y: rect.bottom().saturating_sub(2),
            width: rect.width,
            height: 2,
        },
        Rect {
            x: rect.x,
            y: rect.y,
            width: 2,
            height: rect.height,
        },
        Rect {
            x: rect.right().saturating_sub(2),
            y: rect.y,
            width: 2,
            height: rect.height,
        },
    ] {
        fill_rect(buffer, width, height, edge, color, opacity);
    }
}

fn fill_client_layer_rect(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    source: ClientLayerPixelSource<'_>,
) {
    if source.source_width > 0
        && source.source_height > 0
        && source.source_rgba.len()
            >= (source.source_width as usize * source.source_height as usize * 4)
    {
        fill_scaled_client_buffer_rect(buffer, width, height, &source);
        return;
    }

    let rect = source.rect;
    let max_x = rect.x.saturating_add(rect.width).min(width);
    let max_y = rect.y.saturating_add(rect.height).min(height);
    let half_width = rect.width.max(1).div_ceil(2);
    let half_height = rect.height.max(1).div_ceil(2);

    for y in rect.y..max_y {
        for x in rect.x..max_x {
            let local_x = x.saturating_sub(rect.x);
            let local_y = y.saturating_sub(rect.y);
            let grid_x = usize::from(local_x >= half_width);
            let grid_y = usize::from(local_y >= half_height);
            let pixel_source = source.sample_grid[grid_y * 2 + grid_x];
            let offset = ((y * width + x) * 4) as usize;
            let destination = [
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ];
            let blended = blend_source_over(pixel_source, destination, source.opacity);
            buffer[offset..offset + 4].copy_from_slice(&blended);
        }
    }
}

fn fill_scaled_client_buffer_rect(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    source: &ClientLayerPixelSource<'_>,
) {
    let rect = source.rect;
    let max_x = rect.x.saturating_add(rect.width).min(width);
    let max_y = rect.y.saturating_add(rect.height).min(height);
    let rect_width = rect.width.max(1);
    let rect_height = rect.height.max(1);

    for y in rect.y..max_y {
        for x in rect.x..max_x {
            let local_x = x.saturating_sub(rect.x);
            let local_y = y.saturating_sub(rect.y);
            let source_x = ((local_x as u64 * source.source_width as u64) / rect_width as u64)
                .min(source.source_width.saturating_sub(1) as u64)
                as u32;
            let source_y = ((local_y as u64 * source.source_height as u64) / rect_height as u64)
                .min(source.source_height.saturating_sub(1) as u64)
                as u32;
            let source_offset = ((source_y * source.source_width + source_x) * 4) as usize;
            let pixel_source = [
                source.source_rgba[source_offset],
                source.source_rgba[source_offset + 1],
                source.source_rgba[source_offset + 2],
                source.source_rgba[source_offset + 3],
            ];
            let offset = ((y * width + x) * 4) as usize;
            let destination = [
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ];
            let blended = blend_source_over(pixel_source, destination, source.opacity);
            buffer[offset..offset + 4].copy_from_slice(&blended);
        }
    }
}

fn draw_system_surface_primitives(buffer: &mut [u8], width: u32, height: u32, rect: Rect) -> usize {
    let border = [0xc8, 0xf6, 0xff, 0xff];
    let highlight = [0xff, 0xff, 0xff, 0xff];
    let corner_softener = [0x04, 0x3b, 0x5c, 0xff];
    let inset_shadow = [0x00, 0x46, 0x68, 0xff];

    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 1,
        },
        border,
        220,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x,
            y: rect.y + rect.height - 1,
            width: rect.width,
            height: 1,
        },
        border,
        220,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x,
            y: rect.y,
            width: 1,
            height: rect.height,
        },
        border,
        220,
    );
    fill_rect(
        buffer,
        width,
        height,
        Rect {
            x: rect.x + rect.width - 1,
            y: rect.y,
            width: 1,
            height: rect.height,
        },
        border,
        220,
    );

    if rect.width > 4 {
        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: rect.x + 2,
                y: rect.y + 1,
                width: rect.width - 4,
                height: 1,
            },
            highlight,
            120,
        );
    }

    if rect.width > 4 && rect.height > 4 {
        let corner_pixels = [
            (rect.x, rect.y),
            (rect.x + 1, rect.y + 1),
            (rect.x + rect.width - 1, rect.y),
            (rect.x + rect.width - 2, rect.y + 1),
            (rect.x, rect.y + rect.height - 1),
            (rect.x + 1, rect.y + rect.height - 2),
            (rect.x + rect.width - 1, rect.y + rect.height - 1),
            (rect.x + rect.width - 2, rect.y + rect.height - 2),
        ];

        for (x, y) in corner_pixels {
            fill_rect(
                buffer,
                width,
                height,
                Rect {
                    x,
                    y,
                    width: 1,
                    height: 1,
                },
                corner_softener,
                180,
            );
        }

        fill_rect(
            buffer,
            width,
            height,
            Rect {
                x: rect.x + 2,
                y: rect.y + rect.height - 2,
                width: rect.width - 4,
                height: 1,
            },
            inset_shadow,
            96,
        );
    }

    3
}

fn blend_source_over(source: [u8; 4], destination: [u8; 4], opacity: u8) -> [u8; 4] {
    if opacity == 255 {
        return [source[0], source[1], source[2], 0xff];
    }

    let alpha = u16::from(opacity);
    [
        blend_channel(source[0], destination[0], alpha),
        blend_channel(source[1], destination[1], alpha),
        blend_channel(source[2], destination[2], alpha),
        0xff,
    ]
}

fn blend_channel(source: u8, destination: u8, alpha: u16) -> u8 {
    ((u16::from(source) * alpha + u16::from(destination) * (255 - alpha) + 127) / 255) as u8
}

fn sample_pixel(buffer: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * width + x) * 4) as usize;
    buffer
        .get(offset..offset + 4)
        .and_then(|pixel| pixel.try_into().ok())
        .unwrap_or([0, 0, 0, 0])
}

fn checksum_bytes(buffer: &[u8]) -> u64 {
    buffer.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

pub fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let row_bytes = width as usize * 4;
    let mut raw = Vec::with_capacity((row_bytes + 1) * height as usize);

    for row in rgba.chunks_exact(row_bytes) {
        raw.push(0);
        raw.extend_from_slice(row);
    }

    let compressed = zlib_stored_blocks(&raw);
    let mut png = Vec::with_capacity(8 + 25 + compressed.len() + 12);
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_png_chunk(&mut png, b"IHDR", &ihdr);
    write_png_chunk(&mut png, b"IDAT", &compressed);
    write_png_chunk(&mut png, b"IEND", &[]);

    png
}

fn zlib_stored_blocks(raw: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(raw.len() + raw.len() / 65_535 * 5 + 8);
    bytes.extend_from_slice(&[0x78, 0x01]);

    let mut remaining = raw;
    while !remaining.is_empty() {
        let block_len = remaining.len().min(65_535);
        let is_final = block_len == remaining.len();
        bytes.push(if is_final { 0x01 } else { 0x00 });

        let len = block_len as u16;
        bytes.extend_from_slice(&len.to_le_bytes());
        bytes.extend_from_slice(&(!len).to_le_bytes());
        bytes.extend_from_slice(&remaining[..block_len]);
        remaining = &remaining[block_len..];
    }

    bytes.extend_from_slice(&adler32(raw).to_be_bytes());
    bytes
}

fn write_png_chunk(png: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(chunk_type);
    png.extend_from_slice(data);

    let mut crc_input = Vec::with_capacity(chunk_type.len() + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    png.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn adler32(bytes: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;
    let mut a = 1_u32;
    let mut b = 0_u32;

    for byte in bytes {
        a = (a + u32::from(*byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }

    (b << 16) | a
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;

    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }

    !crc
}

fn opacity_for_command(command: &DrawCommand) -> u8 {
    match command.kind {
        DrawCommandKind::ImageLayer | DrawCommandKind::IconGroup => 255,
        DrawCommandKind::SystemSurfacePanel => 184,
    }
}

fn effect_for_command(command: &DrawCommand) -> &'static str {
    match command.kind {
        DrawCommandKind::ImageLayer | DrawCommandKind::IconGroup => "none",
        DrawCommandKind::SystemSurfacePanel => "layered-system-surface",
    }
}

pub fn required_surface_kinds_are_planned(plan: &RenderPlan) -> bool {
    [
        SurfaceKind::Wallpaper.as_str(),
        SurfaceKind::TopPanel.as_str(),
        SurfaceKind::DesktopIconColumn.as_str(),
        SurfaceKind::Dock.as_str(),
        SurfaceKind::Launcher.as_str(),
        SurfaceKind::SystemOverview.as_str(),
        SurfaceKind::NotificationToast.as_str(),
    ]
    .iter()
    .all(|kind| {
        plan.commands
            .iter()
            .any(|command| command.surface_id == surface_id_for_kind(kind))
    })
}

fn surface_id_for_kind(kind: &str) -> &'static str {
    match kind {
        "wallpaper" => "wallpaper",
        "top-panel" => "top-panel",
        "desktop-icon-column" => "desktop-icons",
        "dock" => "dock",
        "launcher" => "launcher",
        "system-overview" => "system-overview",
        "notification-toast" => "notification-toast",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aqua_installer::{
        DiskIdentity, InstallTarget, InstallerFormKey, InstallerSummaryKey, InstallerUserFormKey,
        UserProfile,
    };

    fn complete_gpu_runtime() -> GpuRuntimeCapabilities {
        GpuRuntimeCapabilities {
            drm: true,
            gbm: true,
            egl: true,
            gles2: true,
        }
    }

    #[test]
    fn pale_wave_wallpaper_is_bright_opaque_varied_and_deterministic() {
        let first = render_pale_wave_wallpaper_rgba(320, 200);
        let second = render_pale_wave_wallpaper_rgba(320, 200);
        assert_eq!(first, second);
        assert_eq!(first.len(), 320 * 200 * 4);
        assert!(first.chunks_exact(4).all(|pixel| pixel[3] == 0xff));
        assert!(first
            .chunks_exact(4)
            .all(|pixel| pixel[0] >= 0xb8 && pixel[1] >= 0xd2 && pixel[2] >= 0xee));
        let unique = first
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<std::collections::BTreeSet<_>>();
        assert!(unique.len() > 64);
        assert_eq!(checksum_bytes(&first), checksum_bytes(&second));
    }

    #[test]
    fn renderer_backend_auto_prefers_gpu_and_falls_back_safely() {
        let gpu = select_renderer_backend(RendererPreference::Auto, complete_gpu_runtime());
        assert_eq!(gpu.selected_backend, "smithay-gles2-gbm");
        assert!(gpu.can_start);
        assert!(!gpu.software_fallback);

        let fallback = select_renderer_backend(
            RendererPreference::Auto,
            GpuRuntimeCapabilities {
                drm: true,
                gbm: false,
                egl: false,
                gles2: false,
            },
        );
        assert_eq!(fallback.selected_backend, "aqua-software-raster");
        assert!(fallback.can_start);
        assert!(fallback.software_fallback);
    }

    #[test]
    fn renderer_backend_forced_gpu_fails_closed_without_runtime() {
        let decision = select_renderer_backend(
            RendererPreference::Gpu,
            GpuRuntimeCapabilities {
                drm: true,
                gbm: true,
                egl: false,
                gles2: false,
            },
        );
        assert_eq!(decision.selected_backend, "unavailable");
        assert!(!decision.can_start);
        assert!(!decision.software_fallback);
    }

    #[test]
    fn embedded_ui_font_rasterizes_antialiased_latin_text() {
        assert!(embedded_ui_font_ready());
        let service = text_service().expect("embedded Noto Sans should parse");
        let mut service = service.lock().expect("text service should not be poisoned");
        let line = service.shape_line("Aqğİş", TextRole::Body, OutputScale::One);
        for shaped in line.runs.iter().flat_map(|run| &run.glyphs) {
            let glyph = service
                .rasterize(GlyphCacheKey {
                    font_id: shaped.font_id,
                    glyph_id: shaped.glyph_id,
                    role: TextRole::Body,
                    scale: OutputScale::One,
                    mode: RenderingMode::Grayscale,
                })
                .expect("shaped glyph should rasterize");
            assert!(glyph.metrics.width > 0);
            assert!(glyph.metrics.height > 0);
            assert!(glyph
                .coverage
                .iter()
                .any(|alpha| *alpha > 0 && *alpha < 255));
        }
        assert_eq!(aqua_text::UI_FONT_REVISION, "noto-sans-regular-aqua-1");
    }

    #[test]
    fn typography_layout_acceptance_keeps_critical_actions_clear() {
        for (viewport, scale) in [
            (Viewport::new(800, 600), OutputScale::One),
            (Viewport::new(1280, 800), OutputScale::One),
            (Viewport::new(1536, 1024), OutputScale::FiveQuarters),
        ] {
            for theme in AquaTheme::ALL {
                let (rgba, probe) =
                    render_typography_layout_acceptance_rgba(viewport, theme, scale).unwrap();
                assert!(probe.is_ready());
                assert_eq!(
                    rgba.len(),
                    viewport.width as usize * viewport.height as usize * 4
                );
                assert!(probe.critical_labels_fit);
                assert!(probe.long_label_contained);
                assert!(probe.fallback_glyphs > 0);
                assert_eq!(probe.missing_glyphs, 0);
                assert!(probe.regions_are_separated);
                assert_ne!(probe.checksum, 0);
            }
        }
    }

    #[test]
    fn typography_layout_acceptance_report_is_stable_and_complete() {
        let first = typography_layout_acceptance_report();
        assert_eq!(first, typography_layout_acceptance_report());
        assert_eq!(first.matches("viewport=").count(), 12);
        assert_eq!(first.matches("ready=true").count(), 12);
        assert_eq!(first.matches("missing_glyphs=0").count(), 12);
    }

    #[test]
    fn installer_welcome_window_renders_real_layout_logo_and_png() {
        let model = InstallerModel::default();
        let ui = InstallerUiState::new(&model);
        let forms = InstallerFormState::default();
        let logo_pixels = [
            0x36, 0xd8, 0xf2, 0xff, 0x65, 0xe8, 0xfa, 0xff, 0x08, 0x72, 0xc7, 0xff, 0x14, 0xa9,
            0xe2, 0xff,
        ];
        let logo = InstallerImageSource::new(2, 2, &logo_pixels).unwrap();
        let (rgba, probe) =
            render_installer_window_rgba(1280, 800, &model, &ui, &forms, None, logo).unwrap();

        assert!(probe.is_ready());
        assert_eq!(probe.step, InstallerStep::Welcome);
        assert_eq!(probe.focus, InstallerFocusTarget::LanguageControl);
        assert_eq!(probe.step_count, 9);
        assert!(probe.logo_rendered);
        assert_eq!(probe.progress_percent, None);
        assert_eq!(rgba.len(), 1280 * 800 * 4);
        assert_eq!(sample_pixel(&rgba, 1280, 0, 0), [0x9a, 0xb9, 0xd9, 0xff]);
        assert_ne!(
            sample_pixel(&rgba, 1280, 640, 400),
            [0x9a, 0xb9, 0xd9, 0xff]
        );
        let layout = InstallerWindowLayout::for_viewport(Viewport::new(1280, 800)).unwrap();
        assert_eq!(
            sample_pixel(&rgba, 1280, layout.titlebar.x + 400, layout.titlebar.y + 40),
            LIGHTWHITE_WINDOW_CHROME.titlebar
        );

        let (png, png_probe) =
            export_installer_window_png(1280, 800, &model, &ui, &forms, None, logo).unwrap();
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert_eq!(png_probe.checksum, probe.checksum);
    }

    #[test]
    fn installer_window_renders_all_runtime_theme_palettes() {
        let model = InstallerModel::default();
        let ui = InstallerUiState::new(&model);
        let forms = InstallerFormState::default();
        let logo_pixels = [0x18, 0x78, 0xc8, 0xff];
        let logo = InstallerImageSource::new(1, 1, &logo_pixels).unwrap();
        let layout = InstallerWindowLayout::for_viewport(Viewport::new(1280, 800)).unwrap();
        let mut checksums = Vec::new();

        for theme in AquaTheme::ALL {
            let (rgba, probe) = render_installer_window_rgba_with_theme(
                1280,
                800,
                &model,
                &ui,
                &forms,
                logo,
                InstallerRenderOptions {
                    progress: None,
                    theme,
                },
            )
            .unwrap();
            assert!(
                probe.is_ready(),
                "{} Installer render is not ready",
                theme.id()
            );
            assert_eq!(
                sample_pixel(&rgba, 1280, layout.titlebar.x + 400, layout.titlebar.y + 40,),
                window_chrome_palette(theme).titlebar
            );
            checksums.push(probe.checksum);
        }

        checksums.sort_unstable();
        checksums.dedup();
        assert_eq!(checksums.len(), AquaTheme::ALL.len());
    }

    #[test]
    fn installer_language_and_keyboard_forms_render_distinct_applied_choices() {
        let logo_pixels = [0x36, 0xd8, 0xf2, 0xff];
        let logo = InstallerImageSource::new(1, 1, &logo_pixels).unwrap();
        let mut model = InstallerModel::default();
        let mut forms = InstallerFormState::default();

        model.advance().unwrap();
        forms
            .handle_key(&mut model, InstallerFormKey::Down)
            .unwrap();
        forms
            .handle_key(&mut model, InstallerFormKey::Activate)
            .unwrap();
        let language_ui = InstallerUiState::new(&model);
        let (_, language_probe) =
            render_installer_window_rgba(1280, 800, &model, &language_ui, &forms, None, logo)
                .unwrap();
        assert!(language_probe.is_ready());
        assert_eq!(language_probe.step, InstallerStep::Language);
        assert!(!language_probe.logo_rendered);

        model.advance().unwrap();
        forms.handle_key(&mut model, InstallerFormKey::End).unwrap();
        forms
            .handle_key(&mut model, InstallerFormKey::Activate)
            .unwrap();
        let keyboard_ui = InstallerUiState::new(&model);
        let (_, keyboard_probe) =
            render_installer_window_rgba(1280, 800, &model, &keyboard_ui, &forms, None, logo)
                .unwrap();
        assert!(keyboard_probe.is_ready());
        assert_eq!(keyboard_probe.step, InstallerStep::Keyboard);
        assert!(!keyboard_probe.logo_rendered);
        assert_ne!(language_probe.checksum, keyboard_probe.checksum);
    }

    #[test]
    fn installer_partitions_form_renders_applied_disk_and_layout_plan() {
        let logo_pixels = [0x36, 0xd8, 0xf2, 0xff];
        let logo = InstallerImageSource::new(1, 1, &logo_pixels).unwrap();
        let mut model = InstallerModel::default();
        let mut forms = InstallerFormState::default();
        model.advance().unwrap();
        forms
            .handle_key(&mut model, InstallerFormKey::Activate)
            .unwrap();
        model.advance().unwrap();
        forms
            .handle_key(&mut model, InstallerFormKey::Activate)
            .unwrap();
        model.advance().unwrap();
        let target = InstallTarget::erase_disk(
            DiskIdentity::new(
                "/dev/vdb",
                "renderer-partitions-target",
                "QEMU HARDDISK",
                32 * 1024 * 1024 * 1024,
            )
            .unwrap(),
        );
        forms.load_selected_target(&target);
        forms
            .handle_disk_key(&mut model, InstallerFormKey::Activate)
            .unwrap();
        let ui = InstallerUiState::new(&model);
        let (_, probe) =
            render_installer_window_rgba(1280, 800, &model, &ui, &forms, None, logo).unwrap();

        assert!(probe.is_ready());
        assert_eq!(probe.step, InstallerStep::Partitions);
        assert!(!probe.logo_rendered);
        assert_eq!(model.target().unwrap().disk.device(), "/dev/vdb");
    }

    #[test]
    fn installer_timezone_form_renders_applied_catalog_value() {
        let logo_pixels = [0x36, 0xd8, 0xf2, 0xff];
        let logo = InstallerImageSource::new(1, 1, &logo_pixels).unwrap();
        let mut model = InstallerModel::default();
        let mut forms = InstallerFormState::default();
        model.advance().unwrap();
        forms
            .handle_key(&mut model, InstallerFormKey::Activate)
            .unwrap();
        model.advance().unwrap();
        forms
            .handle_key(&mut model, InstallerFormKey::Activate)
            .unwrap();
        model.advance().unwrap();
        model.set_target(InstallTarget::erase_disk(
            DiskIdentity::new(
                "/dev/vdb",
                "renderer-timezone-target",
                "QEMU HARDDISK",
                32 * 1024 * 1024 * 1024,
            )
            .unwrap(),
        ));
        model.advance().unwrap();
        forms.handle_key(&mut model, InstallerFormKey::End).unwrap();
        let ui = InstallerUiState::new(&model);
        let (_, probe) =
            render_installer_window_rgba(1280, 800, &model, &ui, &forms, None, logo).unwrap();

        assert!(probe.is_ready());
        assert_eq!(probe.step, InstallerStep::TimeZone);
        assert!(!probe.logo_rendered);
        assert_eq!(model.timezone(), Some("America/New_York"));
    }

    #[test]
    fn installer_user_form_renders_applied_profile_without_password_content() {
        let logo_pixels = [0x36, 0xd8, 0xf2, 0xff];
        let logo = InstallerImageSource::new(1, 1, &logo_pixels).unwrap();
        let mut model = InstallerModel::default();
        model.advance().unwrap();
        model.set_locale("tr_TR.UTF-8").unwrap();
        model.advance().unwrap();
        model.set_keyboard_layout("trq").unwrap();
        model.advance().unwrap();
        model.set_target(InstallTarget::erase_disk(
            DiskIdentity::new(
                "/dev/vdb",
                "renderer-user-target",
                "QEMU HARDDISK",
                32 * 1024 * 1024 * 1024,
            )
            .unwrap(),
        ));
        model.advance().unwrap();
        model.set_timezone("Europe/Istanbul").unwrap();
        model.advance().unwrap();
        let mut forms = InstallerFormState::default();
        for character in "aqua".chars() {
            forms
                .user_mut()
                .handle_key(&mut model, InstallerUserFormKey::Character(character))
                .unwrap();
        }
        forms
            .user_mut()
            .handle_key(&mut model, InstallerUserFormKey::NextField)
            .unwrap();
        for character in "Aqua User".chars() {
            forms
                .user_mut()
                .handle_key(&mut model, InstallerUserFormKey::Character(character))
                .unwrap();
        }
        forms
            .user_mut()
            .handle_key(&mut model, InstallerUserFormKey::NextField)
            .unwrap();
        forms
            .user_mut()
            .handle_key(
                &mut model,
                InstallerUserFormKey::SetPasswordConfigured(true),
            )
            .unwrap();
        forms
            .user_mut()
            .handle_key(&mut model, InstallerUserFormKey::Activate)
            .unwrap();
        let ui = InstallerUiState::new(&model);
        let (_, probe) =
            render_installer_window_rgba(1280, 800, &model, &ui, &forms, None, logo).unwrap();

        assert!(probe.is_ready());
        assert_eq!(probe.step, InstallerStep::UserInformation);
        assert!(!probe.logo_rendered);
        assert_eq!(model.user().unwrap().username(), "aqua");
    }

    #[test]
    fn installer_summary_renders_target_bound_real_install_confirmation() {
        let logo_pixels = [0x36, 0xd8, 0xf2, 0xff];
        let logo = InstallerImageSource::new(1, 1, &logo_pixels).unwrap();
        let mut model = InstallerModel::default();
        model.advance().unwrap();
        model.set_locale("tr_TR.UTF-8").unwrap();
        model.advance().unwrap();
        model.set_keyboard_layout("trq").unwrap();
        model.advance().unwrap();
        model.set_target(InstallTarget::erase_disk(
            DiskIdentity::new(
                "/dev/vdb",
                "renderer-summary-target",
                "QEMU HARDDISK",
                32 * 1024 * 1024 * 1024,
            )
            .unwrap(),
        ));
        model.advance().unwrap();
        model.set_timezone("Europe/Istanbul").unwrap();
        model.advance().unwrap();
        model.set_user(UserProfile::new("aqua", "Aqua User", true).unwrap());
        model.advance().unwrap();
        model.set_mode(InstallMode::Real);
        let mut forms = InstallerFormState::default();
        assert_eq!(
            forms
                .summary_mut()
                .handle_key(&mut model, InstallerSummaryKey::Activate)
                .unwrap(),
            aqua_installer::InstallerSummaryUpdate::AcknowledgementChanged(true)
        );
        let confirmation = model.confirmation_phrase().unwrap();
        for character in confirmation.chars() {
            forms
                .summary_mut()
                .handle_key(&mut model, InstallerSummaryKey::Character(character))
                .unwrap();
        }
        forms
            .summary_mut()
            .handle_key(&mut model, InstallerSummaryKey::Activate)
            .unwrap();
        let ui = InstallerUiState::new(&model);
        let (_, probe) =
            render_installer_window_rgba(1280, 800, &model, &ui, &forms, None, logo).unwrap();

        assert!(probe.is_ready());
        assert_eq!(probe.step, InstallerStep::Summary);
        assert!(!probe.logo_rendered);
        assert!(model.destructive_confirmed());
        assert!(forms.summary().can_begin_install(&model));
    }

    #[test]
    fn installer_renderer_rejects_stale_ui_and_unsupported_viewport() {
        let mut model = InstallerModel::default();
        let ui = InstallerUiState::new(&model);
        let forms = InstallerFormState::default();
        let logo_pixels = [0x36, 0xd8, 0xf2, 0xff];
        let logo = InstallerImageSource::new(1, 1, &logo_pixels).unwrap();
        model.advance().unwrap();
        assert_eq!(
            render_installer_window_rgba(1280, 800, &model, &ui, &forms, None, logo),
            Err("installer UI step does not match installer model".to_string())
        );

        let model = InstallerModel::default();
        let ui = InstallerUiState::new(&model);
        let forms = InstallerFormState::default();
        assert_eq!(
            render_installer_window_rgba(799, 600, &model, &ui, &forms, None, logo),
            Err("unsupported installer viewport 799x600".to_string())
        );
        assert_eq!(
            InstallerImageSource::new(2, 2, &logo_pixels),
            Err("installer image source must be non-empty rgba8888")
        );
    }

    #[test]
    fn rgba_image_layers_preserve_transparent_source_pixels() {
        let mut destination = vec![0xe8, 0xf2, 0xfc, 0xff];
        let transparent_black = [0x00, 0x00, 0x00, 0x00];
        fill_rgba_image_rect(
            &mut destination,
            1,
            1,
            Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            RgbaImageSource {
                width: 1,
                height: 1,
                rgba: &transparent_black,
            },
            255,
        );
        assert_eq!(destination, [0xe8, 0xf2, 0xfc, 0xff]);
    }

    #[test]
    fn settings_audio_category_renders_real_bounded_slider_state() {
        let mut model = SettingsWindowModel {
            selected_category: 4,
            ..SettingsWindowModel::default()
        };
        let output = aqua_service_adapters::AudioDevice::new(
            "sink.1",
            "Aqua Test Output",
            aqua_service_adapters::AudioDeviceKind::Output,
        )
        .expect("output fixture");
        assert!(model.audio.set_volume_percent(85));
        model
            .reconcile_audio_state(
                aqua_service_adapters::AudioAuthoritativeState::new(
                    1,
                    aqua_service_adapters::AudioServiceHealth::Ready,
                    vec![output],
                    Some("sink.1".to_string()),
                    None,
                    85,
                    false,
                )
                .expect("authoritative audio fixture"),
            )
            .expect("adapter state should be accepted");
        let (pixels, probe) = render_settings_window_rgba(600, 400, &model);
        assert!(probe.rendered);
        assert_eq!(probe.category_count, 6);
        assert_eq!(probe.selected_category, 4);
        assert!(probe.audio_available);
        assert!(probe.audio_controls_enabled);
        assert!(probe.audio_backend_applied);
        assert_eq!(probe.audio_control_status, "applied");
        assert_eq!(probe.audio_desired_volume_percent, 85);
        assert_eq!(probe.audio_volume_percent, 85);
        assert!(!probe.audio_muted);
        assert_ne!(probe.checksum, 0);
        assert_eq!(pixels.len(), 600 * 400 * 4);
    }

    #[test]
    fn settings_network_category_disables_wifi_without_authenticated_broker() {
        let model = SettingsWindowModel {
            selected_category: 3,
            ..SettingsWindowModel::default()
        };
        let (pixels, probe) = render_settings_window_rgba(600, 400, &model);
        assert!(probe.rendered);
        assert_eq!(probe.selected_category, 3);
        assert!(!probe.wifi_control_available);
        assert!(!probe.wifi_controls_enabled);
        assert!(!probe.wifi_connected);
        assert_ne!(probe.checksum, 0);
        assert_eq!(pixels.len(), 600 * 400 * 4);
    }

    #[test]
    fn settings_audio_category_distinguishes_applying_and_degraded_states() {
        let mut model = SettingsWindowModel {
            selected_category: 4,
            ..SettingsWindowModel::default()
        };
        let output = aqua_service_adapters::AudioDevice::new(
            "sink.1",
            "Aqua Test Output",
            aqua_service_adapters::AudioDeviceKind::Output,
        )
        .expect("output fixture");
        model
            .reconcile_audio_state(
                aqua_service_adapters::AudioAuthoritativeState::new(
                    1,
                    aqua_service_adapters::AudioServiceHealth::Ready,
                    vec![output],
                    Some("sink.1".to_string()),
                    None,
                    70,
                    false,
                )
                .expect("authoritative audio fixture"),
            )
            .expect("adapter state should be accepted");
        assert!(model.audio.set_volume_percent(85));
        let (_, applying) = render_settings_window_rgba(600, 400, &model);
        assert!(applying.audio_available);
        assert!(!applying.audio_controls_enabled);
        assert!(!applying.audio_backend_applied);
        assert_eq!(applying.audio_control_status, "applying");
        assert_eq!(applying.audio_desired_volume_percent, 85);
        assert_eq!(applying.audio_volume_percent, 70);

        model
            .reconcile_audio_state(
                aqua_service_adapters::AudioAuthoritativeState::unavailable(
                    2,
                    aqua_service_adapters::AudioServiceHealth::Degraded,
                )
                .expect("degraded audio fixture"),
            )
            .expect("degraded state should be accepted");
        let (_, degraded) = render_settings_window_rgba(600, 400, &model);
        assert!(!degraded.audio_available);
        assert!(!degraded.audio_controls_enabled);
        assert!(!degraded.audio_backend_applied);
        assert_eq!(degraded.audio_control_status, "degraded");
        assert_eq!(degraded.audio_desired_volume_percent, 85);
        assert_ne!(applying.checksum, degraded.checksum);
    }

    #[test]
    fn terminal_window_renders_vt_screen_and_cursor() {
        let view = TerminalView {
            lines: vec![
                "Aqua Linux".to_string(),
                "aqua@aqua:~$ echo ready".to_string(),
                "ready".to_string(),
            ],
            cursor_row: 3,
            cursor_col: 0,
            rows: 18,
            cols: 72,
        };
        let (pixels, probe) = render_terminal_window_rgba(680, 430, &view);
        assert!(probe.rendered);
        assert_eq!(probe.rows, 18);
        assert_eq!(probe.cols, 72);
        assert_eq!(probe.visible_line_count, 3);
        assert!(probe.primitive_count >= 7);
        assert_ne!(probe.checksum, 0);
        assert_eq!(pixels.len(), 680 * 430 * 4);
    }

    #[test]
    fn first_party_windows_share_each_selected_theme_titlebar() {
        let terminal_view = TerminalView::empty(18, 72);
        let properties_model = DesktopPropertiesModel {
            icon_id: "files",
            title: "Files Properties".to_string(),
            name: "Files",
            kind: "Folder",
            location: "/home/aqua".to_string(),
            status: "Available",
            item_count: Some(4),
            enumeration_capped: false,
            refresh_generation: 1,
        };
        let mut files_checksums = Vec::new();
        for theme in AquaTheme::ALL {
            let palette = window_chrome_palette(theme);
            let (terminal, _) =
                render_terminal_window_rgba_with_theme(680, 430, &terminal_view, theme);
            let (files, files_probe) =
                render_files_window_rgba_with_theme(640, 420, &FilesWindowModel::default(), theme);
            let settings_model = SettingsWindowModel {
                theme,
                ..SettingsWindowModel::default()
            };
            let (settings, _) = render_settings_window_rgba(640, 420, &settings_model);
            let (properties, _) =
                render_properties_window_rgba_with_theme(480, 300, &properties_model, theme);

            for (pixels, width) in [
                (&terminal, 680),
                (&files, 640),
                (&settings, 640),
                (&properties, 480),
            ] {
                assert_eq!(sample_pixel(pixels, width, 400, 40), palette.titlebar);
            }
            files_checksums.push(files_probe.checksum);
        }
        files_checksums.sort_unstable();
        files_checksums.dedup();
        assert_eq!(files_checksums.len(), AquaTheme::ALL.len());
    }

    #[test]
    fn properties_window_renders_target_metadata() {
        let model = DesktopPropertiesModel {
            icon_id: "files",
            title: "Files Properties".to_string(),
            name: "Files",
            kind: "Folder",
            location: "/home/aqua".to_string(),
            status: "Available",
            item_count: Some(4),
            enumeration_capped: false,
            refresh_generation: 2,
        };
        let (pixels, probe) = render_properties_window_rgba(480, 300, &model);
        assert!(probe.rendered);
        assert_eq!(probe.target, "files");
        assert_eq!(probe.item_count, Some(4));
        assert_eq!(probe.refresh_generation, 2);
        assert!(probe.primitive_count >= 8);
        assert_ne!(probe.checksum, 0);
        assert_eq!(pixels.len(), 480 * 300 * 4);
    }
    use aqua_scene::Viewport;

    #[test]
    fn dock_overlay_renders_items_and_running_indicators() {
        let overlay = render_dock_rgba(
            760,
            72,
            &DockState {
                applications_open: true,
                search_open: false,
                files_running: true,
                settings_running: false,
                active_workspace: 1,
            },
        );
        assert_eq!(overlay.rgba.len(), 760 * 72 * 4);
        assert_eq!(overlay.running_item_count, 2);
        assert_eq!(overlay.active_workspace, 1);
        assert_eq!(overlay.group_count, 3);
        assert!(overlay.primitive_count > 40);
        assert!(overlay.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0));
    }

    #[test]
    fn top_bar_overlay_renders_real_state_labels() {
        let overlay = render_top_bar_rgba(
            1536,
            36,
            &TopBarState {
                product_label: "Aqua Linux".to_string(),
                clock_label: "Thu, 27 Aug 2026  10:30 UTC".to_string(),
                network_connected: true,
                battery_percent: Some(87),
                audio_available: true,
            },
        );
        assert_eq!(overlay.rgba.len(), 1536 * 36 * 4);
        assert!(overlay.primitive_count >= 30);
        assert!(overlay.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0));
    }

    #[test]
    fn cached_aqua_core_icons_replace_shell_placeholders_and_reuse_rasters() {
        let top_bar = TopBarState {
            product_label: "Aqua Linux".to_string(),
            clock_label: "Sat, 29 Aug 2026  10:30 UTC".to_string(),
            network_connected: true,
            battery_percent: Some(87),
            audio_available: true,
        };
        let dock = DockState::default();
        let desktop = DesktopIconState::default();
        let mut notifications = NotificationCenter::default();
        notifications.post(
            1,
            "Aqua Desktop",
            "Scale-native icons",
            "Aqua Core Icon cache is active.",
            5_000,
        );
        let mut cache = icons::IconRasterCache::default();

        let cached_top = render_top_bar_rgba_with_cached_icons(
            1536,
            36,
            &top_bar,
            AquaTheme::Nightmare,
            &mut cache,
        )
        .unwrap();
        let cached_desktop = render_desktop_icons_rgba_with_cached_icons(
            aqua_shell::DESKTOP_ICON_LAYER_WIDTH,
            aqua_shell::DESKTOP_ICON_LAYER_HEIGHT,
            &desktop,
            AquaTheme::Nightmare,
            &mut cache,
        )
        .unwrap();
        let cached_dock =
            render_dock_rgba_with_cached_icons(760, 72, &dock, AquaTheme::Nightmare, &mut cache)
                .unwrap();
        let cached_notification = render_notification_toast_rgba_with_cached_icons(
            420,
            112,
            &notifications,
            AquaTheme::Nightmare,
            &mut cache,
        )
        .unwrap();

        assert_ne!(
            cached_top.rgba,
            render_top_bar_rgba_with_theme(1536, 36, &top_bar, AquaTheme::Nightmare).rgba
        );
        assert_ne!(
            cached_desktop.rgba,
            render_desktop_icons_rgba_with_theme(
                aqua_shell::DESKTOP_ICON_LAYER_WIDTH,
                aqua_shell::DESKTOP_ICON_LAYER_HEIGHT,
                &desktop,
                AquaTheme::Nightmare,
            )
            .rgba
        );
        assert_ne!(
            cached_dock.rgba,
            render_dock_rgba_with_theme(760, 72, &dock, AquaTheme::Nightmare).rgba
        );
        assert_ne!(
            cached_notification.rgba,
            render_notification_toast_rgba_with_theme(
                420,
                112,
                &notifications,
                AquaTheme::Nightmare,
            )
            .rgba
        );
        assert_eq!(cache.len(), 10);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 10);
        assert_eq!(cache.stats().parsed_sources, 7);
        assert_eq!(cache.stats().evictions, 0);

        render_dock_rgba_with_cached_icons(760, 72, &dock, AquaTheme::Nightmare, &mut cache)
            .unwrap();
        assert_eq!(cache.len(), 10);
        assert_eq!(cache.stats().hits, 3);
        assert_eq!(cache.stats().misses, 10);
    }

    #[test]
    fn shell_overlays_render_distinct_runtime_theme_palettes() {
        let top_bar = TopBarState {
            product_label: "Aqua Linux".to_string(),
            clock_label: "Thu, 27 Aug 2026  10:30 UTC".to_string(),
            network_connected: true,
            battery_percent: Some(87),
            audio_available: true,
        };
        let dock = DockState {
            applications_open: true,
            search_open: false,
            files_running: true,
            settings_running: false,
            active_workspace: 1,
        };
        let viewport = Viewport::new(1536, 1024);
        let mut launcher = LauncherState::default();
        launcher.open();
        let mut checksums = Vec::new();

        for theme in AquaTheme::ALL {
            let top_bar = render_top_bar_rgba_with_theme(1536, 36, &top_bar, theme);
            let dock = render_dock_rgba_with_theme(760, 72, &dock, theme);
            let (launcher, probe) =
                render_launcher_overlay_rgba_with_theme(viewport, &launcher, theme);
            assert!(probe.is_ready());
            checksums.push((
                checksum_bytes(&top_bar.rgba),
                checksum_bytes(&dock.rgba),
                checksum_bytes(&launcher),
            ));
        }

        checksums.sort_unstable();
        checksums.dedup();
        assert_eq!(checksums.len(), AquaTheme::ALL.len());
    }

    #[test]
    fn runtime_launcher_overlay_renders_categories_apps_and_selection() {
        let viewport = Viewport::new(1536, 1024);
        let wallpaper = vec![0x04, 0x3b, 0x5c, 0xff];
        let client_plan = ClientLayerPaintPlan {
            status: "client-layer-paint-ready",
            backend: RENDER_BACKEND,
            renderer_started: false,
            steps: Vec::new(),
        };
        let mut launcher = LauncherState::default();
        launcher.open();
        let (frame, probe) = export_runtime_desktop_rgba_with_launcher(
            viewport,
            1,
            1,
            &wallpaper,
            &client_plan,
            &launcher,
        )
        .expect("runtime launcher composition");

        assert!(probe.is_ready());
        assert_eq!(probe.category_count, 9);
        assert_eq!(probe.mode, "applications");
        assert_eq!(probe.visible_app_count, 6);
        assert_eq!(probe.selected_index, 0);
        assert!(!probe.query_visible);
        assert_eq!(frame.status, "rgba-runtime-desktop-launcher-ready");
        assert_ne!(frame.checksum, 0);
    }

    #[test]
    fn runtime_launcher_overlay_renders_filtered_search_state() {
        let viewport = Viewport::new(1536, 1024);
        let wallpaper = vec![0x04, 0x3b, 0x5c, 0xff];
        let client_plan = ClientLayerPaintPlan {
            status: "client-layer-paint-ready",
            backend: RENDER_BACKEND,
            renderer_started: false,
            steps: Vec::new(),
        };
        let mut launcher = LauncherState::default();
        launcher.open();
        launcher.select_category(LauncherCategory::AllApplications);
        launcher.set_query("settings");
        let (_, probe) = export_runtime_desktop_rgba_with_launcher(
            viewport,
            1,
            1,
            &wallpaper,
            &client_plan,
            &launcher,
        )
        .expect("filtered launcher composition");

        assert!(probe.is_ready());
        assert_eq!(probe.mode, "search");
        assert_eq!(probe.visible_app_count, 1);
        assert!(probe.query_visible);
    }

    #[test]
    fn session_menu_overlay_renders_selection_and_confirmation() {
        let mut menu = SessionMenuState::default();
        menu.handle_event(aqua_shell::SessionMenuEvent::Toggle);
        menu.handle_event(aqua_shell::SessionMenuEvent::Navigate(
            aqua_shell::MenuNavigationKey::Previous,
        ));
        let normal = render_session_menu_overlay_rgba(320, 220, &menu);
        assert_eq!(normal.selected_action, "recovery");
        assert!(!normal.confirmation_visible);
        assert!(normal.primitive_count >= 11);
        assert!(normal.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0));

        menu.handle_event(aqua_shell::SessionMenuEvent::Activate);
        let confirmed = render_session_menu_overlay_rgba(320, 220, &menu);
        assert!(confirmed.confirmation_visible);
        assert_ne!(normal.rgba, confirmed.rgba);

        let native = render_session_menu_overlay_rgba(512, 293, &menu);
        assert_eq!(native.rgba.len(), 512 * 293 * 4);
        assert!(native.confirmation_visible);
        assert_ne!(native.rgba, vec![0; 512 * 293 * 4]);
    }

    #[test]
    fn notification_toast_overlay_contains_active_text_and_clears_when_dismissed() {
        let mut center = NotificationCenter::default();
        center.post(10, "Aqua Files", "Files opened", "Home is ready.", 5_000);
        let visible = render_notification_toast_rgba(420, 112, &center);
        assert_eq!(visible.notification_id, Some(1));
        assert_eq!(visible.primitive_count, 7);
        assert!(visible.rgba.iter().any(|channel| *channel != 0));

        center.dismiss(20);
        let hidden = render_notification_toast_rgba(420, 112, &center);
        assert_eq!(hidden.notification_id, None);
        assert_eq!(hidden.primitive_count, 0);
        assert!(hidden.rgba.iter().all(|channel| *channel == 0));
    }

    #[test]
    fn system_overview_overlay_renders_real_metric_values() {
        let model = SystemOverviewModel {
            clock_utc: "10:30 UTC".to_string(),
            os_name: "Aqua Linux".to_string(),
            hostname: "aqua-linux".to_string(),
            kernel: "6.6.32-aqua".to_string(),
            uptime_seconds: 90_061,
            load_average_x100: 125,
            memory_total_kib: 1_000_000,
            memory_available_kib: 625_000,
        };
        let overlay = render_system_overview_rgba(512, 352, &model);
        assert_eq!(overlay.rgba.len(), 512 * 352 * 4);
        assert_eq!(overlay.memory_used_percent, 37);
        assert_eq!(overlay.primitive_count, 15);
        assert!(overlay.rgba.chunks_exact(4).any(|pixel| pixel[3] != 0));
    }

    #[test]
    fn desktop_icons_overlay_tracks_selection_and_context_menu() {
        let mut state = DesktopIconState::default();
        state.pointer_press(48, 90, aqua_shell::DesktopPointerButton::Primary, 100);
        state.pointer_press(48, 194, aqua_shell::DesktopPointerButton::Secondary, 200);
        let overlay = render_desktop_icons_rgba(
            aqua_shell::DESKTOP_ICON_LAYER_WIDTH,
            aqua_shell::DESKTOP_ICON_LAYER_HEIGHT,
            &state,
        );
        assert_eq!(overlay.selected, Some(1));
        assert_eq!(overlay.context_menu, Some(1));
        assert_eq!(overlay.context_menu_selected_row, Some(0));
        assert_eq!(overlay.width, 232);
        assert_eq!(overlay.height, 312);
        assert!(overlay.primitive_count >= 25);
        assert!(overlay.rgba.iter().any(|channel| *channel != 0));
        let files_face = ((45 * overlay.width + 40) * 4) as usize;
        assert_eq!(
            &overlay.rgba[files_face..files_face + 4],
            &[0xa8, 0xe8, 0xf8, 0xff]
        );
        state.handle_context_menu_key(aqua_shell::DesktopContextMenuKey::Navigate(
            aqua_shell::MenuNavigationKey::Next,
        ));
        let second_row = render_desktop_icons_rgba(
            aqua_shell::DESKTOP_ICON_LAYER_WIDTH,
            aqua_shell::DESKTOP_ICON_LAYER_HEIGHT,
            &state,
        );
        assert_eq!(second_row.context_menu_selected_row, Some(1));
        assert_ne!(overlay.rgba, second_row.rgba);
        let png = export_desktop_icons_png(overlay.width, overlay.height, &state);
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn files_window_renders_sidebar_location_entries_and_empty_state() {
        let (home, home_probe) = render_files_window_rgba(640, 420, &FilesWindowModel::default());
        assert!(home_probe.is_ready());
        assert_eq!(home_probe.entry_count, 4);
        assert!(!home_probe.empty_state_rendered);
        assert_eq!(home.len(), 640 * 420 * 4);

        let (empty, empty_probe) =
            render_files_window_rgba(640, 420, &FilesWindowModel::empty("Aqua / Empty"));
        assert!(empty_probe.is_ready(), "{empty_probe:?}");
        assert_eq!(empty_probe.entry_count, 0);
        assert!(empty_probe.empty_state_rendered);
        assert_ne!(home_probe.checksum, empty_probe.checksum);
        assert_ne!(home, empty);

        let mut selected = FilesWindowModel::default();
        selected.select_at(640, 220, 140);
        let (_, selected_probe) = render_files_window_rgba(640, 420, &selected);
        assert_ne!(home_probe.checksum, selected_probe.checksum);

        let sidebar_focused = FilesWindowModel {
            keyboard_focus: true,
            focused_sidebar: Some(1),
            ..FilesWindowModel::default()
        };
        let (_, sidebar_focused_probe) = render_files_window_rgba(640, 420, &sidebar_focused);
        assert_ne!(home_probe.checksum, sidebar_focused_probe.checksum);
    }

    #[test]
    fn runtime_wallpaper_is_scaled_under_static_shell_surfaces() {
        let viewport = Viewport::new(1536, 1024);
        let wallpaper = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ];
        let client_plan = ClientLayerPaintPlan {
            status: "client-layer-paint-ready",
            backend: RENDER_BACKEND,
            renderer_started: false,
            steps: Vec::new(),
        };
        let frame = export_composited_preview_rgba_with_wallpaper_and_client_layers(
            viewport,
            2,
            2,
            &wallpaper,
            &client_plan,
        )
        .expect("runtime wallpaper composition");
        let offset = ((700 * viewport.width + 768) * 4) as usize;

        assert_eq!(&frame.bytes[offset..offset + 4], &[255, 255, 0, 255]);
        assert_eq!(
            frame.byte_count,
            (viewport.width * viewport.height * 4) as usize
        );
        assert_eq!(
            frame.status,
            "rgba-runtime-wallpaper-composited-preview-ready"
        );
        assert!(!frame.renderer_started);
    }

    #[test]
    fn runtime_wallpaper_rejects_invalid_rgba_payload() {
        let client_plan = ClientLayerPaintPlan {
            status: "client-layer-paint-ready",
            backend: RENDER_BACKEND,
            renderer_started: false,
            steps: Vec::new(),
        };
        assert!(
            export_composited_preview_rgba_with_wallpaper_and_client_layers(
                Viewport::new(1536, 1024),
                2,
                2,
                &[0; 8],
                &client_plan,
            )
            .is_err()
        );
    }

    #[test]
    fn render_plan_contains_static_scene_commands() {
        let plan = render_plan_for_static_scene(Viewport::new(1536, 1024));

        assert!(plan.is_ready());
        assert_eq!(plan.commands.len(), 7);
        assert!(required_surface_kinds_are_planned(&plan));
    }

    #[test]
    fn render_plan_excludes_hidden_interactive_surfaces() {
        let mut scene = aqua_scene::static_shell_scene(Viewport::new(1536, 1024));
        scene.set_surface_visible(aqua_scene::SurfaceKind::Launcher, false);

        let plan = plan_static_scene(&scene);
        assert_eq!(plan.commands.len(), 6);
        assert!(!plan
            .commands
            .iter()
            .any(|command| command.surface_id == "launcher"));
    }

    #[test]
    fn render_plan_dump_is_stable() {
        let plan = render_plan_for_static_scene(Viewport::new(1536, 1024));
        let lines = plan.dump_lines();

        assert_eq!(lines[0], "renderer_status=plan-only");
        assert_eq!(lines[1], "renderer_backend=headless-command-plan");
        assert!(lines.contains(
            &"draw surface=launcher kind=system-surface-panel rect=24,60,560,520 asset_count=1 material_token_count=7 simulated=true"
                .to_string()
        ));
    }

    #[test]
    fn client_surface_source_plan_sorts_importable_sources_without_starting_renderer() {
        let plan = plan_client_surface_sources(vec![
            ClientSurfaceSource {
                client_id: "wayland-client-2",
                surface_id: "xdg-toplevel-2",
                window_id: "aqua-settings-client",
                z_index: 1,
                focused: false,
                rect: Rect {
                    x: 464,
                    y: 248,
                    width: 704,
                    height: 436,
                },
                width: 320,
                height: 220,
                stride: 1280,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
            ClientSurfaceSource {
                client_id: "wayland-client-1",
                surface_id: "xdg-toplevel-1",
                window_id: "wayland-test-client",
                z_index: 2,
                focused: true,
                rect: Rect {
                    x: 416,
                    y: 220,
                    width: 704,
                    height: 436,
                },
                width: 384,
                height: 256,
                stride: 1536,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
        ]);

        assert!(plan.is_ready());
        assert!(!plan.renderer_started);
        assert_eq!(plan.sources[0].client_id, "wayland-client-1");
    }

    #[test]
    fn client_surface_source_plan_dump_is_stable() {
        let plan = plan_client_surface_sources(vec![ClientSurfaceSource {
            client_id: "wayland-client-1",
            surface_id: "xdg-toplevel-1",
            window_id: "wayland-test-client",
            z_index: 2,
            focused: true,
            rect: Rect {
                x: 416,
                y: 220,
                width: 704,
                height: 436,
            },
            width: 384,
            height: 256,
            stride: 1536,
            format: "argb8888",
            source: "client-committed-wl-shm",
            sample_checksum: 0xd28e_e773_3dd0_fd7e,
            sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
            sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
            client_buffer_rgba: Vec::new(),
            renderer_import_ready: true,
        }]);
        let lines = plan.dump_lines();

        assert_eq!(
            lines[0],
            "surface_source_status=client-surface-sources-ready"
        );
        assert_eq!(lines[1], "surface_source_backend=headless-command-plan");
        assert!(lines.contains(
            &"source client=wayland-client-1 surface=xdg-toplevel-1 window=wayland-test-client z_index=2 focused=true buffer=384x256 stride=1536 format=argb8888 source=client-committed-wl-shm sample_checksum=d28ee7733dd0fd7e sample_pixel=7f,7f,7f,ff sample_grid=7f,7f,7f,ff|7f,7f,7f,ff|7f,7f,7f,ff|7f,7f,7f,ff buffer_snapshot_bytes=0 renderer_import_ready=true rect=416,220,704,436"
                .to_string()
        ));
    }

    #[test]
    fn client_layer_paint_plan_uses_surface_sources_without_starting_renderer() {
        let source_plan = plan_client_surface_sources(vec![
            ClientSurfaceSource {
                client_id: "wayland-client-1",
                surface_id: "xdg-toplevel-1",
                window_id: "wayland-test-client",
                z_index: 2,
                focused: true,
                rect: Rect {
                    x: 416,
                    y: 220,
                    width: 704,
                    height: 436,
                },
                width: 384,
                height: 256,
                stride: 1536,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
            ClientSurfaceSource {
                client_id: "wayland-client-2",
                surface_id: "xdg-toplevel-2",
                window_id: "aqua-settings-client",
                z_index: 1,
                focused: false,
                rect: Rect {
                    x: 464,
                    y: 248,
                    width: 704,
                    height: 436,
                },
                width: 320,
                height: 220,
                stride: 1280,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
        ]);
        let paint_plan = plan_client_layer_paint_steps(&source_plan);

        assert!(paint_plan.is_ready());
        assert_eq!(paint_plan.steps.len(), 2);
        assert_eq!(paint_plan.steps[0].client_id, "wayland-client-1");
        assert_eq!(paint_plan.steps[0].effect, "sampled-wl-shm-client-buffer");
        assert!(!paint_plan.renderer_started);
    }

    #[test]
    fn client_layer_raster_probe_fills_source_layers_without_display_output() {
        let source_plan = plan_client_surface_sources(vec![
            ClientSurfaceSource {
                client_id: "wayland-client-1",
                surface_id: "xdg-toplevel-1",
                window_id: "wayland-test-client",
                z_index: 2,
                focused: true,
                rect: Rect {
                    x: 416,
                    y: 220,
                    width: 704,
                    height: 436,
                },
                width: 384,
                height: 256,
                stride: 1536,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
            ClientSurfaceSource {
                client_id: "wayland-client-2",
                surface_id: "xdg-toplevel-2",
                window_id: "aqua-settings-client",
                z_index: 1,
                focused: false,
                rect: Rect {
                    x: 464,
                    y: 248,
                    width: 704,
                    height: 436,
                },
                width: 320,
                height: 220,
                stride: 1280,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
        ]);
        let paint_plan = plan_client_layer_paint_steps(&source_plan);
        let probe = probe_client_layer_raster(Viewport::new(1536, 1024), &paint_plan);

        assert!(probe.is_ready());
        assert_eq!(probe.layer_count, 2);
        assert_ne!(probe.layer_checksum, 0);
        assert_eq!(probe.active_layer_sample[3], 0xff);
        assert_eq!(probe.inactive_layer_sample[3], 0xff);
        assert!(!probe.renderer_started);
    }

    #[test]
    fn client_layer_raster_uses_full_client_buffer_when_available() {
        let source_plan = plan_client_surface_sources(vec![ClientSurfaceSource {
            client_id: "wayland-client-1",
            surface_id: "xdg-toplevel-1",
            window_id: "wayland-test-client",
            z_index: 2,
            focused: true,
            rect: Rect {
                x: 416,
                y: 220,
                width: 704,
                height: 436,
            },
            width: 384,
            height: 256,
            stride: 1536,
            format: "argb8888",
            source: "client-committed-wl-shm",
            sample_checksum: 0xd28e_e773_3dd0_fd7e,
            sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
            sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
            client_buffer_rgba: gradient_client_buffer_rgba(384, 256),
            renderer_import_ready: true,
        }]);
        let paint_plan = plan_client_layer_paint_steps(&source_plan);
        let probe = probe_client_layer_raster(Viewport::new(1536, 1024), &paint_plan);

        assert_eq!(probe.status, "client-layer-rasterized");
        assert_ne!(probe.active_layer_sample, [0x7f, 0x7f, 0x7f, 0xff]);
        assert_eq!(probe.active_layer_sample[2], 0x7f);
        assert_eq!(probe.active_layer_sample[3], 0xff);
    }

    #[test]
    fn composited_preview_png_includes_client_layers_without_display_output() {
        let viewport = Viewport::new(1536, 1024);
        let source_plan = plan_client_surface_sources(vec![
            ClientSurfaceSource {
                client_id: "wayland-client-1",
                surface_id: "xdg-toplevel-1",
                window_id: "wayland-test-client",
                z_index: 2,
                focused: true,
                rect: Rect {
                    x: 416,
                    y: 220,
                    width: 704,
                    height: 436,
                },
                width: 384,
                height: 256,
                stride: 1536,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
            ClientSurfaceSource {
                client_id: "wayland-client-2",
                surface_id: "xdg-toplevel-2",
                window_id: "aqua-settings-client",
                z_index: 1,
                focused: false,
                rect: Rect {
                    x: 464,
                    y: 248,
                    width: 704,
                    height: 436,
                },
                width: 320,
                height: 220,
                stride: 1280,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
        ]);
        let paint_plan = plan_client_layer_paint_steps(&source_plan);
        let static_export = export_software_raster_png_for_static_scene(viewport);
        let composited_export =
            export_composited_preview_png_with_client_layers(viewport, &paint_plan);

        assert_eq!(
            composited_export.format,
            "png-rgba8888-composited-client-preview"
        );
        assert_eq!(composited_export.byte_count, static_export.byte_count);
        assert_eq!(composited_export.checksum, 0x3a53_6b4f_39fb_7751);
        assert_ne!(composited_export.checksum, static_export.checksum);
        assert!(!composited_export.renderer_started);
    }

    #[test]
    fn composited_preview_rgba_includes_client_layers_without_display_output() {
        let viewport = Viewport::new(1536, 1024);
        let source_plan = plan_client_surface_sources(vec![
            ClientSurfaceSource {
                client_id: "wayland-client-1",
                surface_id: "xdg-toplevel-1",
                window_id: "wayland-test-client",
                z_index: 2,
                focused: true,
                rect: Rect {
                    x: 416,
                    y: 220,
                    width: 704,
                    height: 436,
                },
                width: 384,
                height: 256,
                stride: 1536,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
            ClientSurfaceSource {
                client_id: "wayland-client-2",
                surface_id: "xdg-toplevel-2",
                window_id: "aqua-settings-client",
                z_index: 1,
                focused: false,
                rect: Rect {
                    x: 464,
                    y: 248,
                    width: 704,
                    height: 436,
                },
                width: 320,
                height: 220,
                stride: 1280,
                format: "argb8888",
                source: "client-committed-wl-shm",
                sample_checksum: 0xd28e_e773_3dd0_fd7e,
                sample_pixel: [0x7f, 0x7f, 0x7f, 0xff],
                sample_grid: solid_sample_grid([0x7f, 0x7f, 0x7f, 0xff]),
                client_buffer_rgba: Vec::new(),
                renderer_import_ready: true,
            },
        ]);
        let paint_plan = plan_client_layer_paint_steps(&source_plan);
        let static_export = export_software_raster_rgba_for_static_scene(viewport);
        let composited_export =
            export_composited_preview_rgba_with_client_layers(viewport, &paint_plan);

        assert!(composited_export.is_ready());
        assert_eq!(
            composited_export.format,
            "raw-rgba8888-composited-client-preview"
        );
        assert_eq!(composited_export.byte_count, 6_291_456);
        assert_eq!(composited_export.bytes.len(), 6_291_456);
        assert_ne!(composited_export.checksum, 0);
        assert_ne!(composited_export.checksum, static_export.checksum);
        assert!(!composited_export.renderer_started);
    }

    #[test]
    fn paint_plan_is_deterministic_without_starting_renderer() {
        let plan = paint_plan_for_static_scene(Viewport::new(1536, 1024));

        assert!(plan.is_ready());
        assert_eq!(plan.steps.len(), 7);
        assert!(plan.orders_are_stable());
        assert!(plan.system_surface_steps_are_translucent());
        assert!(!plan.renderer_started);
    }

    #[test]
    fn paint_plan_dump_is_stable() {
        let plan = paint_plan_for_static_scene(Viewport::new(1536, 1024));
        let lines = plan.dump_lines();

        assert_eq!(lines[0], "paint_status=plan-only");
        assert_eq!(lines[1], "paint_backend=headless-command-plan");
        assert!(lines.contains(
            &"paint order=4 surface=launcher kind=system-surface-panel rect=24,60,560,520 opacity=184 blend=source-over effect=layered-system-surface"
                .to_string()
        ));
    }

    #[test]
    fn frame_plan_defines_output_without_starting_renderer() {
        let plan = frame_plan_for_static_scene(Viewport::new(1536, 1024));

        assert!(plan.is_ready());
        assert_eq!(plan.pixel_format, "rgba8888");
        assert_eq!(plan.stride_bytes, 6144);
        assert_eq!(plan.buffer_bytes, 6_291_456);
        assert!(!plan.renderer_started);
    }

    #[test]
    fn frame_plan_dump_is_stable() {
        let plan = frame_plan_for_static_scene(Viewport::new(1536, 1024));
        let lines = plan.dump_lines();

        assert_eq!(lines[0], "frame_status=plan-only");
        assert_eq!(lines[1], "frame_backend=headless-command-plan");
        assert!(lines.contains(&"frame_size=1536x1024".to_string()));
        assert!(lines.contains(&"damage_rect=0,0,1536,1024".to_string()));
    }

    #[test]
    fn frame_buffer_probe_allocates_clear_buffer_without_starting_renderer() {
        let probe = probe_frame_buffer_for_static_scene(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.buffer_bytes, 6_291_456);
        assert_eq!(probe.allocated_bytes, 6_291_456);
        assert_eq!(probe.first_pixel, [0x00, 0x17, 0x25, 0xff]);
        assert_eq!(probe.last_pixel, [0x00, 0x17, 0x25, 0xff]);
        assert!(!probe.renderer_started);
    }

    #[test]
    fn frame_buffer_probe_dump_is_stable() {
        let probe = probe_frame_buffer_for_static_scene(Viewport::new(1536, 1024));
        let lines = probe.dump_lines();

        assert_eq!(lines[0], "buffer_status=allocated");
        assert_eq!(lines[1], "buffer_backend=headless-command-plan");
        assert!(lines.contains(&"first_pixel=00,17,25,ff".to_string()));
        assert!(lines.contains(&"last_pixel=00,17,25,ff".to_string()));
    }

    #[test]
    fn software_raster_probe_draws_static_scene_into_memory() {
        let probe = probe_software_raster_for_static_scene(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.filled_rect_count, 7);
        assert_eq!(probe.wallpaper_sample, [0x04, 0x3b, 0x5c, 0xff]);
        assert_eq!(probe.surface_sample, [0x51, 0xac, 0xd2, 0xff]);
        assert_eq!(probe.dock_sample, [0x51, 0xac, 0xd2, 0xff]);
        assert_eq!(probe.surface_border_sample, [0x3d, 0x72, 0x8c, 0xff]);
        assert_eq!(probe.surface_highlight_sample, [0xa3, 0xd3, 0xe7, 0xff]);
        assert_eq!(probe.surface_corner_sample, [0x2a, 0x6c, 0x8c, 0xff]);
        assert_eq!(probe.surface_shadow_sample, [0x33, 0x86, 0xaa, 0xff]);
        assert_eq!(probe.raster_checksum, 0x7015_58d1_5395_21df);
        assert_eq!(probe.surface_primitive_count, 15);
        assert!(!probe.renderer_started);
    }

    #[test]
    fn software_raster_probe_dump_is_stable() {
        let probe = probe_software_raster_for_static_scene(Viewport::new(1536, 1024));
        let lines = probe.dump_lines();

        assert_eq!(lines[0], "raster_status=software-rasterized");
        assert!(lines.contains(&"filled_rect_count=7".to_string()));
        assert!(lines.contains(&"wallpaper_sample=04,3b,5c,ff".to_string()));
        assert!(lines.contains(&"surface_sample=51,ac,d2,ff".to_string()));
        assert!(lines.contains(&"surface_border_sample=3d,72,8c,ff".to_string()));
        assert!(lines.contains(&"surface_highlight_sample=a3,d3,e7,ff".to_string()));
        assert!(lines.contains(&"surface_corner_sample=2a,6c,8c,ff".to_string()));
        assert!(lines.contains(&"surface_shadow_sample=33,86,aa,ff".to_string()));
        assert!(lines.contains(&"raster_checksum=701558d1539521df".to_string()));
    }

    #[test]
    fn raster_ppm_export_is_deterministic_without_display_output() {
        let export = export_software_raster_ppm_for_static_scene(Viewport::new(1536, 1024));

        assert!(export.is_ready());
        assert_eq!(export.format, "ppm-p6-rgb888");
        assert_eq!(export.header, "P6\n1536 1024\n255\n");
        assert_eq!(export.byte_count, 4_718_609);
        assert_eq!(export.checksum, 0xefdc_ba78_578c_2cd5);
        assert!(!export.renderer_started);
    }

    #[test]
    fn raster_ppm_export_dump_is_stable() {
        let export = export_software_raster_ppm_for_static_scene(Viewport::new(1536, 1024));
        let lines = export.dump_lines();

        assert_eq!(lines[0], "export_status=ppm-ready");
        assert!(lines.contains(&"export_format=ppm-p6-rgb888".to_string()));
        assert!(lines.contains(&"export_bytes=4718609".to_string()));
        assert!(lines.contains(&"export_checksum=efdcba78578c2cd5".to_string()));
    }

    #[test]
    fn raster_png_export_is_deterministic_without_display_output() {
        let export = export_software_raster_png_for_static_scene(Viewport::new(1536, 1024));

        assert!(export.is_ready());
        assert_eq!(export.format, "png-rgba8888");
        assert_eq!(export.byte_count, 6_293_028);
        assert_eq!(export.checksum, 0x2cdb_1d86_a1ba_9300);
        assert_eq!(&export.bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert!(!export.renderer_started);
    }

    #[test]
    fn raster_png_export_dump_is_stable() {
        let export = export_software_raster_png_for_static_scene(Viewport::new(1536, 1024));
        let lines = export.dump_lines();

        assert_eq!(lines[0], "export_status=png-ready");
        assert!(lines.contains(&"export_format=png-rgba8888".to_string()));
        assert!(lines.contains(&"export_bytes=6293028".to_string()));
        assert!(lines.contains(&"export_checksum=2cdb1d86a1ba9300".to_string()));
    }

    #[test]
    fn raster_rgba_export_is_window_ready_without_display_output() {
        let export = export_software_raster_rgba_for_static_scene(Viewport::new(1536, 1024));

        assert!(export.is_ready());
        assert_eq!(export.format, "raw-rgba8888");
        assert_eq!(export.byte_count, 6_291_456);
        assert_eq!(export.bytes.len(), 6_291_456);
        assert_eq!(export.checksum, 0x7015_58d1_5395_21df);
        assert!(!export.renderer_started);
    }

    #[test]
    fn raster_rgba_export_dump_is_stable() {
        let export = export_software_raster_rgba_for_static_scene(Viewport::new(1536, 1024));
        let lines = export.dump_lines();

        assert_eq!(lines[0], "export_status=rgba-ready");
        assert!(lines.contains(&"export_format=raw-rgba8888".to_string()));
        assert!(lines.contains(&"export_bytes=6291456".to_string()));
        assert!(lines.contains(&"export_checksum=701558d1539521df".to_string()));
    }
}
