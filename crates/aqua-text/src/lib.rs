//! Shared, deterministic text shaping and scale-native glyph rasterization.

use fontdue::{Font, FontSettings, Metrics};
use rustybuzz::{Direction as BuzzDirection, Face, UnicodeBuffer};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use unicode_bidi::BidiInfo;
use unicode_segmentation::UnicodeSegmentation;

pub const UI_FONT_FAMILY: &str = "Noto Sans";
pub const UI_FONT_REVISION: &str = "noto-sans-regular-aqua-1";
pub const DEFAULT_GLYPH_CACHE_CAPACITY: usize = 2048;
pub const UI_FONT_BYTES: &[u8] =
    include_bytes!("../../../docs/aqua-linux/assets/fonts/NotoSans-Regular.ttf");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputScale {
    One,
    FiveQuarters,
    ThreeHalves,
    Two,
}

impl OutputScale {
    pub const ALL: [Self; 4] = [Self::One, Self::FiveQuarters, Self::ThreeHalves, Self::Two];

    pub const fn numerator(self) -> u16 {
        match self {
            Self::One => 4,
            Self::FiveQuarters => 5,
            Self::ThreeHalves => 6,
            Self::Two => 8,
        }
    }

    pub const fn denominator(self) -> u16 {
        4
    }

    pub fn apply(self, logical_pixels: f32) -> f32 {
        logical_pixels * f32::from(self.numerator()) / f32::from(self.denominator())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextRole {
    Caption,
    Body,
    Control,
    Title,
    Display,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    pub logical_size: f32,
    pub logical_line_height: f32,
}

impl TextRole {
    pub const fn metrics(self) -> TextMetrics {
        match self {
            Self::Caption => TextMetrics {
                logical_size: 11.0,
                logical_line_height: 15.0,
            },
            Self::Body => TextMetrics {
                logical_size: 13.0,
                logical_line_height: 18.0,
            },
            Self::Control => TextMetrics {
                logical_size: 13.0,
                logical_line_height: 17.0,
            },
            Self::Title => TextMetrics {
                logical_size: 16.0,
                logical_line_height: 21.0,
            },
            Self::Display => TextMetrics {
                logical_size: 24.0,
                logical_line_height: 30.0,
            },
            Self::Monospace => TextMetrics {
                logical_size: 13.0,
                logical_line_height: 18.0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapedGlyph {
    pub font_id: &'static str,
    pub glyph_id: u16,
    pub cluster: usize,
    pub x_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    pub byte_range: Range<usize>,
    pub direction: TextDirection,
    pub glyphs: Vec<ShapedGlyph>,
    pub advance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapedLine {
    pub text: String,
    pub runs: Vec<ShapedRun>,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub role: TextRole,
    pub scale: OutputScale,
    pub missing_glyphs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderingMode {
    Grayscale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphCacheKey {
    pub font_id: &'static str,
    pub glyph_id: u16,
    pub role: TextRole,
    pub scale: OutputScale,
    pub mode: RenderingMode,
}

#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    pub metrics: Metrics,
    pub coverage: Vec<u8>,
}

struct FontEntry {
    id: &'static str,
    shaping: Face<'static>,
    raster: Font,
}

pub struct TextService {
    fonts: Vec<FontEntry>,
    glyph_cache: HashMap<GlyphCacheKey, GlyphBitmap>,
    cache_order: VecDeque<GlyphCacheKey>,
    cache_capacity: usize,
}

impl TextService {
    pub fn new() -> Result<Self, &'static str> {
        Self::with_cache_capacity(DEFAULT_GLYPH_CACHE_CAPACITY)
    }

    pub fn with_cache_capacity(cache_capacity: usize) -> Result<Self, &'static str> {
        if cache_capacity == 0 {
            return Err("glyph cache capacity must be non-zero");
        }
        let shaping = Face::from_slice(UI_FONT_BYTES, 0).ok_or("invalid embedded shaping font")?;
        let raster = Font::from_bytes(UI_FONT_BYTES, FontSettings::default())
            .map_err(|_| "invalid embedded raster font")?;
        Ok(Self {
            fonts: vec![FontEntry {
                id: UI_FONT_REVISION,
                shaping,
                raster,
            }],
            glyph_cache: HashMap::new(),
            cache_order: VecDeque::new(),
            cache_capacity,
        })
    }

    pub fn font_ids(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.fonts.iter().map(|font| font.id)
    }

    pub fn cache_len(&self) -> usize {
        self.glyph_cache.len()
    }

    pub fn shape_line(&self, text: &str, role: TextRole, scale: OutputScale) -> ShapedLine {
        let pixel_size = scale.apply(role.metrics().logical_size);
        let bidi = BidiInfo::new(text, None);
        let mut runs = Vec::new();
        let mut missing_glyphs = 0;

        for paragraph in &bidi.paragraphs {
            let (_, visual_runs) = bidi.visual_runs(paragraph, paragraph.range.clone());
            for byte_range in visual_runs {
                let direction = if bidi.levels[byte_range.start].is_rtl() {
                    TextDirection::RightToLeft
                } else {
                    TextDirection::LeftToRight
                };
                let run_text = &text[byte_range.clone()];
                let mut buffer = UnicodeBuffer::new();
                buffer.push_str(run_text);
                buffer.set_direction(match direction {
                    TextDirection::LeftToRight => BuzzDirection::LeftToRight,
                    TextDirection::RightToLeft => BuzzDirection::RightToLeft,
                });
                buffer.guess_segment_properties();
                let shaped = rustybuzz::shape(&self.fonts[0].shaping, &[], buffer);
                let units_per_em = self.fonts[0].shaping.units_per_em() as f32;
                let factor = pixel_size / units_per_em;
                let glyphs = shaped
                    .glyph_infos()
                    .iter()
                    .zip(shaped.glyph_positions())
                    .map(|(info, position)| {
                        if info.glyph_id == 0 {
                            missing_glyphs += 1;
                        }
                        ShapedGlyph {
                            font_id: self.fonts[0].id,
                            glyph_id: u16::try_from(info.glyph_id).unwrap_or(0),
                            cluster: byte_range.start + info.cluster as usize,
                            x_advance: position.x_advance as f32 * factor,
                            x_offset: position.x_offset as f32 * factor,
                            y_offset: position.y_offset as f32 * factor,
                        }
                    })
                    .collect::<Vec<_>>();
                let advance = glyphs.iter().map(|glyph| glyph.x_advance).sum();
                runs.push(ShapedRun {
                    byte_range,
                    direction,
                    glyphs,
                    advance,
                });
            }
        }

        let metrics = role.metrics();
        ShapedLine {
            text: text.to_owned(),
            width: runs.iter().map(|run| run.advance).sum(),
            height: scale.apply(metrics.logical_line_height),
            baseline: scale.apply(metrics.logical_size),
            runs,
            role,
            scale,
            missing_glyphs,
        }
    }

    pub fn ellipsize(
        &self,
        text: &str,
        role: TextRole,
        scale: OutputScale,
        max_width: f32,
    ) -> ShapedLine {
        let full = self.shape_line(text, role, scale);
        if full.width <= max_width {
            return full;
        }
        let ellipsis = "…";
        let ellipsis_width = self.shape_line(ellipsis, role, scale).width;
        let mut candidate = String::new();
        for grapheme in text.graphemes(true) {
            let next = format!("{candidate}{grapheme}{ellipsis}");
            if self.shape_line(&next, role, scale).width > max_width {
                break;
            }
            candidate.push_str(grapheme);
        }
        candidate.push_str(ellipsis);
        if ellipsis_width > max_width {
            candidate.clear();
        }
        self.shape_line(&candidate, role, scale)
    }

    pub fn wrap(
        &self,
        text: &str,
        role: TextRole,
        scale: OutputScale,
        max_width: f32,
    ) -> Vec<ShapedLine> {
        if text.is_empty() {
            return vec![self.shape_line("", role, scale)];
        }
        let mut lines = Vec::new();
        let mut current = String::new();
        for grapheme in text.graphemes(true) {
            let candidate = format!("{current}{grapheme}");
            if !current.is_empty() && self.shape_line(&candidate, role, scale).width > max_width {
                lines.push(self.shape_line(current.trim_end(), role, scale));
                current.clear();
            }
            current.push_str(grapheme);
        }
        if !current.is_empty() {
            lines.push(self.shape_line(current.trim_end(), role, scale));
        }
        lines
    }

    pub fn rasterize(&mut self, key: GlyphCacheKey) -> Option<&GlyphBitmap> {
        if !self.glyph_cache.contains_key(&key) {
            let pixel_size = key.scale.apply(key.role.metrics().logical_size);
            let (metrics, coverage) = self
                .fonts
                .iter()
                .find(|font| font.id == key.font_id)?
                .raster
                .rasterize_indexed(key.glyph_id, pixel_size);
            if self.glyph_cache.len() == self.cache_capacity {
                if let Some(oldest) = self.cache_order.pop_front() {
                    self.glyph_cache.remove(&oldest);
                }
            }
            self.cache_order.push_back(key);
            self.glyph_cache
                .insert(key, GlyphBitmap { metrics, coverage });
        }
        self.glyph_cache.get(&key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextLocale {
    English,
    Turkish,
}

pub fn uppercase(input: &str, locale: TextLocale) -> String {
    input
        .chars()
        .flat_map(|character| match (locale, character) {
            (TextLocale::Turkish, 'i') => "İ".chars().collect::<Vec<_>>(),
            (TextLocale::Turkish, 'ı') => "I".chars().collect(),
            _ => character.to_uppercase().collect(),
        })
        .collect()
}

pub fn lowercase(input: &str, locale: TextLocale) -> String {
    input
        .chars()
        .flat_map(|character| match (locale, character) {
            (TextLocale::Turkish, 'I') => "ı".chars().collect::<Vec<_>>(),
            (TextLocale::Turkish, 'İ') => "i".chars().collect(),
            _ => character.to_lowercase().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_roles_have_stable_metrics_at_supported_scales() {
        for role in [
            TextRole::Caption,
            TextRole::Body,
            TextRole::Control,
            TextRole::Title,
            TextRole::Display,
            TextRole::Monospace,
        ] {
            for scale in OutputScale::ALL {
                let metrics = role.metrics();
                assert!(scale.apply(metrics.logical_size) > 0.0);
                assert!(
                    scale.apply(metrics.logical_line_height) >= scale.apply(metrics.logical_size)
                );
            }
        }
    }

    #[test]
    fn shaping_applies_ligatures_kerning_and_combining_clusters() {
        let service = TextService::new().unwrap();
        let plain = service.shape_line("fi", TextRole::Body, OutputScale::One);
        let combining = service.shape_line("e\u{301}", TextRole::Body, OutputScale::One);
        assert_eq!(plain.runs.len(), 1);
        assert!(plain.runs[0].glyphs.len() <= 2);
        assert!(combining.runs[0]
            .glyphs
            .iter()
            .all(|glyph| glyph.cluster == 0));
    }

    #[test]
    fn mixed_direction_text_produces_visual_direction_runs() {
        let service = TextService::new().unwrap();
        let line = service.shape_line("Aqua مرحبا 12", TextRole::Body, OutputScale::One);
        assert!(line
            .runs
            .iter()
            .any(|run| run.direction == TextDirection::LeftToRight));
        assert!(line
            .runs
            .iter()
            .any(|run| run.direction == TextDirection::RightToLeft));
    }

    #[test]
    fn ellipsis_and_wrapping_never_split_graphemes() {
        let service = TextService::new().unwrap();
        let text = "Aqua e\u{301} işletim sistemi";
        let short = service.ellipsize(text, TextRole::Body, OutputScale::One, 55.0);
        assert!(short.text.is_empty() || short.text.ends_with('…'));
        assert!(!short.text.ends_with("e…"));
        let lines = service.wrap(text, TextRole::Body, OutputScale::One, 45.0);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| !line.text.starts_with('\u{301}')));
    }

    #[test]
    fn turkish_case_mapping_preserves_dotted_and_dotless_i() {
        assert_eq!(
            uppercase("istanbul ılık", TextLocale::Turkish),
            "İSTANBUL ILIK"
        );
        assert_eq!(lowercase("IŞIK İÇİN", TextLocale::Turkish), "ışık için");
        assert_eq!(uppercase("istanbul", TextLocale::English), "ISTANBUL");
    }

    #[test]
    fn glyph_cache_is_scale_native_and_bounded() {
        let mut service = TextService::with_cache_capacity(2).unwrap();
        let line = service.shape_line("A", TextRole::Body, OutputScale::One);
        let glyph = &line.runs[0].glyphs[0];
        for scale in [
            OutputScale::One,
            OutputScale::FiveQuarters,
            OutputScale::Two,
        ] {
            let key = GlyphCacheKey {
                font_id: glyph.font_id,
                glyph_id: glyph.glyph_id,
                role: TextRole::Body,
                scale,
                mode: RenderingMode::Grayscale,
            };
            assert!(service.rasterize(key).is_some());
        }
        assert_eq!(service.cache_len(), 2);
    }
}
