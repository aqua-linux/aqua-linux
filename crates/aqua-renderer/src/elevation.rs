use aqua_scene::{Rect, Viewport};
use aqua_shell::AquaTheme;
use aqua_text::OutputScale;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub const DEFAULT_SHADOW_MASK_CACHE_CAPACITY: usize = 64;
pub const ELEVATION_FIXTURE_REVISION: &str = "aqua-elevation-fixtures-2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElevationLevel {
    Control,
    Panel,
    Dialog,
    ActiveWindow,
}

impl ElevationLevel {
    pub const ALL: [Self; 4] = [Self::Control, Self::Panel, Self::Dialog, Self::ActiveWindow];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Panel => "panel",
            Self::Dialog => "dialog",
            Self::ActiveWindow => "active-window",
        }
    }

    pub const fn token(self) -> ElevationToken {
        match self {
            Self::Control => ElevationToken::new(
                1,
                3,
                0,
                ShadowColor::new([42, 65, 94], 26),
                ShadowColor::new([42, 65, 94], 31),
            ),
            Self::Panel => ElevationToken::new(
                4,
                14,
                0,
                ShadowColor::new([42, 65, 94], 31),
                ShadowColor::new([42, 65, 94], 41),
            ),
            Self::Dialog => ElevationToken::new(
                8,
                24,
                1,
                ShadowColor::new([24, 42, 66], 41),
                ShadowColor::new([24, 42, 66], 51),
            ),
            Self::ActiveWindow => ElevationToken::new(
                10,
                30,
                1,
                ShadowColor::new([24, 42, 66], 41),
                ShadowColor::new([24, 42, 66], 56),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowColor {
    pub rgb: [u8; 3],
    pub alpha: u8,
}

impl ShadowColor {
    pub const fn new(rgb: [u8; 3], alpha: u8) -> Self {
        Self { rgb, alpha }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElevationToken {
    pub offset_y: u16,
    pub blur_radius: u16,
    pub spread: u16,
    pub ambient: ShadowColor,
    pub key: ShadowColor,
}

impl ElevationToken {
    pub const fn new(
        offset_y: u16,
        blur_radius: u16,
        spread: u16,
        ambient: ShadowColor,
        key: ShadowColor,
    ) -> Self {
        Self {
            offset_y,
            blur_radius,
            spread,
            ambient,
            key,
        }
    }

    pub fn scaled(self, scale: OutputScale) -> ScaledElevationToken {
        ScaledElevationToken {
            offset_y: scale_pixels(self.offset_y.into(), scale),
            blur_radius: scale_pixels(self.blur_radius.into(), scale).max(1),
            spread: scale_pixels(self.spread.into(), scale),
            ambient: self.ambient,
            key: self.key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaledElevationToken {
    pub offset_y: u32,
    pub blur_radius: u32,
    pub spread: u32,
    pub ambient: ShadowColor,
    pub key: ShadowColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowExtents {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl ScaledElevationToken {
    pub const fn extents(self) -> ShadowExtents {
        let ambient = self.blur_radius + self.spread;
        ShadowExtents {
            left: ambient,
            top: ambient,
            right: ambient,
            bottom: ambient + self.offset_y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadowMaskKey {
    pub surface_width: u32,
    pub surface_height: u32,
    pub corner_radius: u32,
    pub scale: OutputScale,
    pub theme: AquaTheme,
    pub elevation: ElevationLevel,
}

impl ShadowMaskKey {
    pub const fn from_physical(
        surface_width: u32,
        surface_height: u32,
        corner_radius: u32,
        scale: OutputScale,
        theme: AquaTheme,
        elevation: ElevationLevel,
    ) -> Self {
        Self {
            surface_width,
            surface_height,
            corner_radius,
            scale,
            theme,
            elevation,
        }
    }

    pub fn from_logical(
        surface_width: u32,
        surface_height: u32,
        corner_radius: u32,
        scale: OutputScale,
        theme: AquaTheme,
        elevation: ElevationLevel,
    ) -> Self {
        Self::from_physical(
            scale_pixels(surface_width, scale),
            scale_pixels(surface_height, scale),
            scale_pixels(corner_radius, scale),
            scale,
            theme,
            elevation,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowMask {
    pub key: ShadowMaskKey,
    pub width: u32,
    pub height: u32,
    pub surface_x: u32,
    pub surface_y: u32,
    pub rgba: Vec<u8>,
    pub non_zero_alpha_pixels: usize,
    pub max_alpha: u8,
}

impl ShadowMask {
    pub fn is_ready(&self) -> bool {
        self.width > self.key.surface_width
            && self.height > self.key.surface_height
            && self.rgba.len() == self.width as usize * self.height as usize * 4
            && self.non_zero_alpha_pixels > 0
            && self.max_alpha > 0
            && self.max_alpha < 128
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShadowMaskCacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
}

#[derive(Debug)]
pub struct ShadowMaskCache {
    capacity: usize,
    masks: HashMap<ShadowMaskKey, Arc<ShadowMask>>,
    order: VecDeque<ShadowMaskKey>,
    stats: ShadowMaskCacheStats,
}

impl Default for ShadowMaskCache {
    fn default() -> Self {
        Self::new(DEFAULT_SHADOW_MASK_CACHE_CAPACITY)
    }
}

impl ShadowMaskCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            masks: HashMap::new(),
            order: VecDeque::new(),
            stats: ShadowMaskCacheStats::default(),
        }
    }

    pub fn get_or_render(&mut self, key: ShadowMaskKey) -> Arc<ShadowMask> {
        if let Some(mask) = self.masks.get(&key).cloned() {
            self.stats.hits += 1;
            self.touch(key);
            return mask;
        }
        self.stats.misses += 1;
        let mask = Arc::new(render_shadow_mask(key));
        while self.masks.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                if self.masks.remove(&oldest).is_some() {
                    self.stats.evictions += 1;
                    break;
                }
            }
        }
        self.masks.insert(key, mask.clone());
        self.order.push_back(key);
        mask
    }

    pub fn len(&self) -> usize {
        self.masks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.masks.is_empty()
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub const fn stats(&self) -> ShadowMaskCacheStats {
        self.stats
    }

    fn touch(&mut self, key: ShadowMaskKey) {
        if let Some(index) = self.order.iter().position(|candidate| *candidate == key) {
            self.order.remove(index);
        }
        self.order.push_back(key);
    }
}

pub fn elevation_damage_rect(
    surface: Rect,
    viewport: Viewport,
    scale: OutputScale,
    elevation: ElevationLevel,
) -> Rect {
    let extents = elevation.token().scaled(scale).extents();
    let x = surface.x.saturating_sub(extents.left).min(viewport.width);
    let y = surface.y.saturating_sub(extents.top).min(viewport.height);
    let right = surface
        .right()
        .saturating_add(extents.right)
        .min(viewport.width);
    let bottom = surface
        .bottom()
        .saturating_add(extents.bottom)
        .min(viewport.height);
    Rect {
        x,
        y,
        width: right.saturating_sub(x),
        height: bottom.saturating_sub(y),
    }
}

fn render_shadow_mask(key: ShadowMaskKey) -> ShadowMask {
    let token = key.elevation.token().scaled(key.scale);
    let extents = token.extents();
    let width = key.surface_width + extents.left + extents.right;
    let height = key.surface_height + extents.top + extents.bottom;
    let surface_x = extents.left;
    let surface_y = extents.top;
    let mut rgba = vec![0_u8; width as usize * height as usize * 4];
    let opacity = theme_opacity_percent(key.theme);
    let mut non_zero_alpha_pixels = 0;
    let mut max_alpha = 0;

    for y in 0..height {
        for x in 0..width {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let original_distance = rounded_rect_distance(
                px,
                py,
                surface_x as f32,
                surface_y as f32,
                key.surface_width as f32,
                key.surface_height as f32,
                key.corner_radius as f32,
            );
            if original_distance <= 0.0 {
                continue;
            }
            let ambient = shadow_coverage(
                original_distance,
                token.spread as f32,
                token.blur_radius as f32,
            );
            let key_distance = rounded_rect_distance(
                px,
                py,
                surface_x as f32,
                surface_y as f32 + token.offset_y as f32,
                key.surface_width as f32,
                key.surface_height as f32,
                key.corner_radius as f32,
            );
            let key_coverage =
                shadow_coverage(key_distance, token.spread as f32, token.blur_radius as f32);
            let ambient_alpha = scaled_alpha(token.ambient.alpha, opacity, ambient);
            let key_alpha = scaled_alpha(token.key.alpha, opacity, key_coverage);
            let alpha = combine_alpha(ambient_alpha, key_alpha);
            if alpha == 0 {
                continue;
            }
            let index = ((y * width + x) * 4) as usize;
            let total = u16::from(ambient_alpha) + u16::from(key_alpha);
            for channel in 0..3 {
                rgba[index + channel] = if total == 0 {
                    token.ambient.rgb[channel]
                } else {
                    ((u16::from(token.ambient.rgb[channel]) * u16::from(ambient_alpha)
                        + u16::from(token.key.rgb[channel]) * u16::from(key_alpha))
                        / total) as u8
                };
            }
            rgba[index + 3] = alpha;
            non_zero_alpha_pixels += 1;
            max_alpha = max_alpha.max(alpha);
        }
    }

    ShadowMask {
        key,
        width,
        height,
        surface_x,
        surface_y,
        rgba,
        non_zero_alpha_pixels,
        max_alpha,
    }
}

fn rounded_rect_distance(
    px: f32,
    py: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
) -> f32 {
    let half_width = width * 0.5;
    let half_height = height * 0.5;
    let radius = radius.min(half_width).min(half_height).max(0.0);
    let qx = (px - (x + half_width)).abs() - (half_width - radius);
    let qy = (py - (y + half_height)).abs() - (half_height - radius);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    outside + qx.max(qy).min(0.0) - radius
}

fn shadow_coverage(distance: f32, spread: f32, blur: f32) -> f32 {
    if distance <= spread {
        return 1.0;
    }
    let progress = ((distance - spread) / blur.max(1.0)).clamp(0.0, 1.0);
    let smooth = progress * progress * (3.0 - 2.0 * progress);
    1.0 - smooth
}

fn scaled_alpha(alpha: u8, opacity_percent: u16, coverage: f32) -> u8 {
    (f32::from(alpha) * f32::from(opacity_percent) / 100.0 * coverage)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn combine_alpha(first: u8, second: u8) -> u8 {
    let inverse = u16::from(255 - first) * u16::from(255 - second) / 255;
    (255_u16 - inverse) as u8
}

const fn theme_opacity_percent(theme: AquaTheme) -> u16 {
    match theme {
        AquaTheme::Light => 100,
        AquaTheme::Dark => 74,
    }
}

fn scale_pixels(logical: u32, scale: OutputScale) -> u32 {
    let numerator = u32::from(scale.numerator());
    let denominator = u32::from(scale.denominator());
    logical
        .saturating_mul(numerator)
        .saturating_add(denominator - 1)
        / denominator
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElevationAcceptanceProbe {
    pub revision: &'static str,
    pub viewport: Viewport,
    pub theme: AquaTheme,
    pub scale: OutputScale,
    pub elevation_count: usize,
    pub cache_entries: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub overlap_visible: bool,
    pub edge_clipped: bool,
    pub bounded_damage: bool,
    pub non_zero_shadow_pixels: usize,
    pub max_shadow_alpha: u8,
    pub checksum: u64,
}

impl ElevationAcceptanceProbe {
    pub fn is_ready(&self) -> bool {
        self.revision == ELEVATION_FIXTURE_REVISION
            && self.elevation_count == ElevationLevel::ALL.len()
            && self.cache_entries == ElevationLevel::ALL.len() + 1
            && self.cache_hits == self.cache_entries
            && self.cache_misses == self.cache_entries
            && self.overlap_visible
            && self.edge_clipped
            && self.bounded_damage
            && self.non_zero_shadow_pixels > 0
            && self.max_shadow_alpha > 0
            && self.max_shadow_alpha < 128
            && self.checksum != 0
    }
}

pub fn render_elevation_acceptance_rgba(
    viewport: Viewport,
    theme: AquaTheme,
    scale: OutputScale,
) -> Result<(Vec<u8>, ElevationAcceptanceProbe), &'static str> {
    if viewport.width < 960 || viewport.height < 640 {
        return Err("elevation fixture viewport must be at least 960x640");
    }
    let surfaces = [
        (
            ElevationLevel::Control,
            Rect {
                x: 60,
                y: 72,
                width: 220,
                height: 52,
            },
            10,
        ),
        (
            ElevationLevel::Panel,
            Rect {
                x: 340,
                y: 68,
                width: 320,
                height: 176,
            },
            16,
        ),
        (
            ElevationLevel::Dialog,
            Rect {
                x: 150,
                y: 300,
                width: 430,
                height: 250,
            },
            18,
        ),
        (
            ElevationLevel::ActiveWindow,
            Rect {
                x: 470,
                y: 340,
                width: 470,
                height: 260,
            },
            20,
        ),
    ];
    let edge_surface = Rect {
        x: viewport.width - 220,
        y: 102,
        width: 220,
        height: 140,
    };
    let mut buffer = vec![0_u8; viewport.width as usize * viewport.height as usize * 4];
    for pixel in buffer.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0xe8, 0xee, 0xf5, 0xff]);
    }
    let palette = super::window_chrome_palette(theme);
    let mut cache = ShadowMaskCache::new(16);
    let mut non_zero_shadow_pixels = 0;
    let mut max_shadow_alpha = 0;

    for (level, surface, radius) in surfaces {
        let key = ShadowMaskKey::from_physical(
            surface.width,
            surface.height,
            radius,
            scale,
            theme,
            level,
        );
        let mask = cache.get_or_render(key);
        composite_shadow(&mut buffer, viewport, surface, &mask);
        non_zero_shadow_pixels += mask.non_zero_alpha_pixels;
        max_shadow_alpha = max_shadow_alpha.max(mask.max_alpha);
        super::fill_rounded_rect(
            &mut buffer,
            viewport.width,
            viewport.height,
            surface,
            radius,
            palette.surface,
            255,
        );
    }

    let edge_key = ShadowMaskKey::from_physical(
        edge_surface.width,
        edge_surface.height,
        16,
        scale,
        theme,
        ElevationLevel::Panel,
    );
    let edge_mask = cache.get_or_render(edge_key);
    composite_shadow(&mut buffer, viewport, edge_surface, &edge_mask);
    non_zero_shadow_pixels += edge_mask.non_zero_alpha_pixels;
    max_shadow_alpha = max_shadow_alpha.max(edge_mask.max_alpha);
    super::fill_rounded_rect(
        &mut buffer,
        viewport.width,
        viewport.height,
        edge_surface,
        16,
        palette.surface,
        255,
    );

    let keys: Vec<_> = cache.order.iter().copied().collect();
    for key in keys {
        cache.get_or_render(key);
    }
    let edge_damage = elevation_damage_rect(edge_surface, viewport, scale, ElevationLevel::Panel);
    let bounded_damage = surfaces.iter().all(|(level, surface, _)| {
        elevation_damage_rect(*surface, viewport, scale, *level).fits_in(viewport)
    }) && edge_damage.fits_in(viewport);
    let stats = cache.stats();
    let checksum = super::checksum_bytes(&buffer);
    let probe = ElevationAcceptanceProbe {
        revision: ELEVATION_FIXTURE_REVISION,
        viewport,
        theme,
        scale,
        elevation_count: ElevationLevel::ALL.len(),
        cache_entries: cache.len(),
        cache_hits: stats.hits,
        cache_misses: stats.misses,
        overlap_visible: surfaces[2].1.overlaps(surfaces[3].1),
        edge_clipped: edge_damage.right() == viewport.width,
        bounded_damage,
        non_zero_shadow_pixels,
        max_shadow_alpha,
        checksum,
    };
    Ok((buffer, probe))
}

pub fn elevation_acceptance_report() -> String {
    let viewport = Viewport::new(1280, 800);
    let mut report = format!("revision={ELEVATION_FIXTURE_REVISION}\n");
    for theme in AquaTheme::ALL {
        for scale in OutputScale::ALL {
            let (_, probe) = render_elevation_acceptance_rgba(viewport, theme, scale)
                .expect("fixed elevation acceptance viewport");
            report.push_str(&format!(
                "theme={} scale={}/{} elevations={} entries={} hits={} misses={} overlap={} edge_clipped={} bounded_damage={} shadow_pixels={} max_alpha={} checksum={:016x} ready={}\n",
                theme.id(),
                scale.numerator(),
                scale.denominator(),
                probe.elevation_count,
                probe.cache_entries,
                probe.cache_hits,
                probe.cache_misses,
                probe.overlap_visible,
                probe.edge_clipped,
                probe.bounded_damage,
                probe.non_zero_shadow_pixels,
                probe.max_shadow_alpha,
                probe.checksum,
                probe.is_ready(),
            ));
        }
    }
    report
}

fn composite_shadow(buffer: &mut [u8], viewport: Viewport, surface: Rect, mask: &ShadowMask) {
    let origin_x = i64::from(surface.x) - i64::from(mask.surface_x);
    let origin_y = i64::from(surface.y) - i64::from(mask.surface_y);
    for mask_y in 0..mask.height {
        let destination_y = origin_y + i64::from(mask_y);
        if destination_y < 0 || destination_y >= i64::from(viewport.height) {
            continue;
        }
        for mask_x in 0..mask.width {
            let destination_x = origin_x + i64::from(mask_x);
            if destination_x < 0 || destination_x >= i64::from(viewport.width) {
                continue;
            }
            let source_offset = ((mask_y * mask.width + mask_x) * 4) as usize;
            let alpha = u16::from(mask.rgba[source_offset + 3]);
            if alpha == 0 {
                continue;
            }
            let destination_offset =
                ((destination_y as u32 * viewport.width + destination_x as u32) * 4) as usize;
            for channel in 0..3 {
                let source = u16::from(mask.rgba[source_offset + channel]);
                let destination = u16::from(buffer[destination_offset + channel]);
                buffer[destination_offset + channel] =
                    ((source * alpha + destination * (255 - alpha) + 127) / 255) as u8;
            }
            buffer[destination_offset + 3] = 255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_geometry_matches_the_public_contract() {
        let cases = [
            (ElevationLevel::Control, 1, 3, 0),
            (ElevationLevel::Panel, 4, 14, 0),
            (ElevationLevel::Dialog, 8, 24, 1),
            (ElevationLevel::ActiveWindow, 10, 30, 1),
        ];
        for (level, offset, blur, spread) in cases {
            let token = level.token();
            assert_eq!(token.offset_y, offset);
            assert_eq!(token.blur_radius, blur);
            assert_eq!(token.spread, spread);
        }
    }

    #[test]
    fn cache_reuses_exact_keys_and_evicts_at_capacity() {
        let mut cache = ShadowMaskCache::new(2);
        let first = ShadowMaskKey::from_logical(
            120,
            60,
            10,
            OutputScale::One,
            AquaTheme::Light,
            ElevationLevel::Panel,
        );
        let second = ShadowMaskKey {
            surface_width: 121,
            ..first
        };
        let third = ShadowMaskKey {
            surface_width: 122,
            ..first
        };
        let original = cache.get_or_render(first);
        let reused = cache.get_or_render(first);
        assert!(Arc::ptr_eq(&original, &reused));
        cache.get_or_render(second);
        cache.get_or_render(third);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 3);
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn damage_expands_and_clips_to_the_viewport() {
        let viewport = Viewport::new(800, 600);
        let center = Rect {
            x: 100,
            y: 100,
            width: 300,
            height: 200,
        };
        let expanded = elevation_damage_rect(
            center,
            viewport,
            OutputScale::One,
            ElevationLevel::ActiveWindow,
        );
        assert_eq!(
            expanded,
            Rect {
                x: 69,
                y: 69,
                width: 362,
                height: 272
            }
        );
        let edge = elevation_damage_rect(
            Rect {
                x: 0,
                y: 0,
                width: 200,
                height: 100,
            },
            viewport,
            OutputScale::Two,
            ElevationLevel::Dialog,
        );
        assert_eq!(edge.x, 0);
        assert_eq!(edge.y, 0);
        assert!(edge.fits_in(viewport));
    }

    #[test]
    fn acceptance_matrix_covers_every_theme_and_scale() {
        for theme in AquaTheme::ALL {
            for scale in OutputScale::ALL {
                let (_, probe) =
                    render_elevation_acceptance_rgba(Viewport::new(1280, 800), theme, scale)
                        .unwrap();
                assert!(probe.is_ready(), "{theme:?} {scale:?}: {probe:?}");
            }
        }
    }

    #[test]
    fn themes_change_opacity_without_changing_geometry() {
        let key = ShadowMaskKey::from_logical(
            240,
            160,
            18,
            OutputScale::ThreeHalves,
            AquaTheme::Light,
            ElevationLevel::Dialog,
        );
        let light = render_shadow_mask(key);
        let dark = render_shadow_mask(ShadowMaskKey {
            theme: AquaTheme::Dark,
            ..key
        });
        assert_eq!((light.width, light.height), (dark.width, dark.height));
        assert_eq!(
            (light.surface_x, light.surface_y),
            (dark.surface_x, dark.surface_y)
        );
        assert_ne!(light.rgba, dark.rgba);
        assert!(light.max_alpha > dark.max_alpha);
    }
}
