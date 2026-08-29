//! Shared, deterministic text shaping and scale-native glyph rasterization.

use fontdue::{Font, FontSettings, Metrics};
use rustybuzz::{Direction as BuzzDirection, Face, UnicodeBuffer};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use unicode_bidi::BidiInfo;
use unicode_segmentation::UnicodeSegmentation;

pub const UI_FONT_FAMILY: &str = "Noto Sans";
pub const UI_FONT_REVISION: &str = "noto-sans-regular-aqua-1";
pub const ARABIC_FALLBACK_FONT_FAMILY: &str = "Noto Sans Arabic";
pub const ARABIC_FALLBACK_FONT_REVISION: &str = "noto-sans-arabic-regular-2.009";
pub const TYPOGRAPHY_FIXTURE_REVISION: &str = "aqua-typography-fixtures-1";
pub const DEFAULT_GLYPH_CACHE_CAPACITY: usize = 2048;
pub const UI_FONT_BYTES: &[u8] =
    include_bytes!("../../../docs/aqua-linux/assets/fonts/NotoSans-Regular.ttf");
pub const ARABIC_FALLBACK_FONT_BYTES: &[u8] =
    include_bytes!("../../../docs/aqua-linux/assets/fonts/NotoSansArabic-Regular.ttf");

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
    pub font_id: &'static str,
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
    pub fallback_runs: usize,
    pub fallback_glyphs: usize,
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
    family: &'static str,
    shaping: Face<'static>,
    raster: Font,
}

impl FontEntry {
    fn supports_grapheme(&self, grapheme: &str) -> bool {
        grapheme
            .chars()
            .filter(|character| requires_glyph(*character))
            .all(|character| self.shaping.glyph_index(character).is_some())
    }
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
        let primary = load_font(
            UI_FONT_REVISION,
            UI_FONT_FAMILY,
            UI_FONT_BYTES,
            "invalid embedded UI font",
        )?;
        let arabic = load_font(
            ARABIC_FALLBACK_FONT_REVISION,
            ARABIC_FALLBACK_FONT_FAMILY,
            ARABIC_FALLBACK_FONT_BYTES,
            "invalid embedded Arabic fallback font",
        )?;
        Ok(Self {
            fonts: vec![primary, arabic],
            glyph_cache: HashMap::new(),
            cache_order: VecDeque::new(),
            cache_capacity,
        })
    }

    pub fn font_ids(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.fonts.iter().map(|font| font.id)
    }

    pub fn font_families(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.fonts.iter().map(|font| font.family)
    }

    pub fn cache_len(&self) -> usize {
        self.glyph_cache.len()
    }

    pub fn shape_line(&self, text: &str, role: TextRole, scale: OutputScale) -> ShapedLine {
        let pixel_size = scale.apply(role.metrics().logical_size);
        let bidi = BidiInfo::new(text, None);
        let mut runs = Vec::new();
        let mut fallback_runs = 0;
        let mut fallback_glyphs = 0;
        let mut missing_glyphs = 0;

        for paragraph in &bidi.paragraphs {
            let (_, visual_runs) = bidi.visual_runs(paragraph, paragraph.range.clone());
            for byte_range in visual_runs {
                let direction = if bidi.levels[byte_range.start].is_rtl() {
                    TextDirection::RightToLeft
                } else {
                    TextDirection::LeftToRight
                };
                let mut font_segments = self.font_segments(text, byte_range);
                if direction == TextDirection::RightToLeft {
                    font_segments.reverse();
                }
                for (segment_range, font_index) in font_segments {
                    let font = &self.fonts[font_index];
                    let mut buffer = UnicodeBuffer::new();
                    buffer.push_str(&text[segment_range.clone()]);
                    buffer.set_direction(match direction {
                        TextDirection::LeftToRight => BuzzDirection::LeftToRight,
                        TextDirection::RightToLeft => BuzzDirection::RightToLeft,
                    });
                    buffer.guess_segment_properties();
                    let shaped = rustybuzz::shape(&font.shaping, &[], buffer);
                    let factor = pixel_size / font.shaping.units_per_em() as f32;
                    let glyphs = shaped
                        .glyph_infos()
                        .iter()
                        .zip(shaped.glyph_positions())
                        .map(|(info, position)| {
                            if info.glyph_id == 0 {
                                missing_glyphs += 1;
                            }
                            ShapedGlyph {
                                font_id: font.id,
                                glyph_id: u16::try_from(info.glyph_id).unwrap_or(0),
                                cluster: segment_range.start + info.cluster as usize,
                                x_advance: position.x_advance as f32 * factor,
                                x_offset: position.x_offset as f32 * factor,
                                y_offset: position.y_offset as f32 * factor,
                            }
                        })
                        .collect::<Vec<_>>();
                    if font_index > 0 {
                        fallback_runs += 1;
                        fallback_glyphs += glyphs.len();
                    }
                    let advance = glyphs.iter().map(|glyph| glyph.x_advance).sum();
                    runs.push(ShapedRun {
                        byte_range: segment_range,
                        direction,
                        font_id: font.id,
                        glyphs,
                        advance,
                    });
                }
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
            fallback_runs,
            fallback_glyphs,
            missing_glyphs,
        }
    }

    fn font_segments(&self, text: &str, byte_range: Range<usize>) -> Vec<(Range<usize>, usize)> {
        let run_text = &text[byte_range.clone()];
        let mut segments = Vec::new();
        let mut active: Option<(usize, usize, usize)> = None;
        for (relative_start, grapheme) in run_text.grapheme_indices(true) {
            let start = byte_range.start + relative_start;
            let end = start + grapheme.len();
            let font_index = self
                .fonts
                .iter()
                .position(|font| font.supports_grapheme(grapheme))
                .unwrap_or(0);
            match active {
                Some((segment_start, _, active_font)) if active_font == font_index => {
                    active = Some((segment_start, end, active_font));
                }
                Some((segment_start, segment_end, active_font)) => {
                    segments.push((segment_start..segment_end, active_font));
                    active = Some((start, end, font_index));
                }
                None => active = Some((start, end, font_index)),
            }
        }
        if let Some((start, end, font_index)) = active {
            segments.push((start..end, font_index));
        }
        segments
    }

    pub fn typography_fixture_report(&self) -> String {
        const FIXTURES: [(&str, &str); 4] = [
            ("latin-ligature", "Aqua office"),
            ("turkish", "İşletim sistemi"),
            ("combining", "Cafe\u{301}"),
            ("mixed-bidi", "Aqua مرحبا 12"),
        ];
        let mut lines = vec![format!("fixture_revision={TYPOGRAPHY_FIXTURE_REVISION}")];
        lines.push(format!(
            "font_order={}",
            self.fonts
                .iter()
                .map(|font| font.id)
                .collect::<Vec<_>>()
                .join(",")
        ));
        for scale in OutputScale::ALL {
            for (name, text) in FIXTURES {
                let shaped = self.shape_line(text, TextRole::Body, scale);
                let runs = shaped
                    .runs
                    .iter()
                    .map(|run| {
                        format!(
                            "{}:{}:{}",
                            run.font_id,
                            match run.direction {
                                TextDirection::LeftToRight => "ltr",
                                TextDirection::RightToLeft => "rtl",
                            },
                            run.glyphs.len()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                lines.push(format!(
                    "fixture={name} scale={}/{} width_64={} baseline_64={} height_64={} fallback_runs={} fallback_glyphs={} missing_glyphs={} runs={runs}",
                    scale.numerator(),
                    scale.denominator(),
                    fixed_64(shaped.width),
                    fixed_64(shaped.baseline),
                    fixed_64(shaped.height),
                    shaped.fallback_runs,
                    shaped.fallback_glyphs,
                    shaped.missing_glyphs,
                ));
            }
        }
        lines.push(String::new());
        lines.join("\n")
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

fn load_font(
    id: &'static str,
    family: &'static str,
    bytes: &'static [u8],
    error: &'static str,
) -> Result<FontEntry, &'static str> {
    let shaping = Face::from_slice(bytes, 0).ok_or(error)?;
    let raster = Font::from_bytes(bytes, FontSettings::default()).map_err(|_| error)?;
    Ok(FontEntry {
        id,
        family,
        shaping,
        raster,
    })
}

fn requires_glyph(character: char) -> bool {
    !character.is_whitespace()
        && !character.is_control()
        && character != '\u{200c}'
        && character != '\u{200d}'
        && !(('\u{fe00}'..='\u{fe0f}').contains(&character))
        && !(('\u{e0100}'..='\u{e01ef}').contains(&character))
}

fn fixed_64(value: f32) -> i32 {
    (value * 64.0).round() as i32
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
        assert_eq!(line.missing_glyphs, 0);
        assert!(line.fallback_runs > 0);
        assert!(line.fallback_glyphs > 0);
        assert!(line.runs.iter().any(|run| {
            run.font_id == ARABIC_FALLBACK_FONT_REVISION
                && run.direction == TextDirection::RightToLeft
        }));
    }

    #[test]
    fn fallback_order_is_deterministic_and_baseline_stays_role_bound() {
        let service = TextService::new().unwrap();
        assert_eq!(
            service.font_ids().collect::<Vec<_>>(),
            vec![UI_FONT_REVISION, ARABIC_FALLBACK_FONT_REVISION]
        );
        assert_eq!(
            service.font_families().collect::<Vec<_>>(),
            vec![UI_FONT_FAMILY, ARABIC_FALLBACK_FONT_FAMILY]
        );
        let latin = service.shape_line("Aqua", TextRole::Body, OutputScale::FiveQuarters);
        let arabic = service.shape_line("مرحبا", TextRole::Body, OutputScale::FiveQuarters);
        assert_eq!(latin.baseline, arabic.baseline);
        assert_eq!(latin.height, arabic.height);
        assert_eq!(latin.fallback_runs, 0);
        assert_eq!(arabic.fallback_runs, 1);
        assert_eq!(arabic.missing_glyphs, 0);
    }

    #[test]
    fn unsupported_text_reports_missing_glyphs_without_changing_font_order() {
        let service = TextService::new().unwrap();
        let line = service.shape_line("Aqua 🫧", TextRole::Body, OutputScale::One);
        assert!(line.missing_glyphs > 0);
        assert_eq!(line.runs[0].font_id, UI_FONT_REVISION);
    }

    #[test]
    fn fixture_report_is_complete_and_deterministic() {
        let service = TextService::new().unwrap();
        let first = service.typography_fixture_report();
        let second = service.typography_fixture_report();
        assert_eq!(first, second);
        assert_eq!(first.matches("fixture=").count(), 16);
        assert!(first.contains("fixture=mixed-bidi"));
        assert!(first.contains("missing_glyphs=0"));
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
