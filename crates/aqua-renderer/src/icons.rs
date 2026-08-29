//! Reviewed Aqua Core Icon SVG loading and scale-native raster caching.

use aqua_shell::AquaTheme;
use aqua_text::OutputScale;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

pub const AQUA_CORE_ICON_REVISION: &str = "aqua-core-icons-2026-08-29-r1";
pub const ICON_FIXTURE_REVISION: &str = "aqua-icon-fixtures-1";
pub const DEFAULT_ICON_RASTER_CACHE_CAPACITY: usize = 256;
pub const REQUIRED_LOGICAL_ICON_SIZES: [u16; 7] = [16, 20, 24, 32, 48, 64, 128];
const MAX_ICON_SOURCE_BYTES: usize = 16 * 1024;
const ICON_VIEW_BOX: &str = "viewbox=\"0 0 64 64\"";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconRole {
    AquaDrive,
    Battery,
    Browser,
    Files,
    Home,
    Notification,
    Settings,
    Software,
    Terminal,
    Trash,
    Updates,
    Volume,
    Wifi,
}

impl IconRole {
    pub const ALL: [Self; 13] = [
        Self::AquaDrive,
        Self::Battery,
        Self::Browser,
        Self::Files,
        Self::Home,
        Self::Notification,
        Self::Settings,
        Self::Software,
        Self::Terminal,
        Self::Trash,
        Self::Updates,
        Self::Volume,
        Self::Wifi,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::AquaDrive => "aqua-drive",
            Self::Battery => "battery",
            Self::Browser => "browser",
            Self::Files => "files",
            Self::Home => "home",
            Self::Notification => "notification",
            Self::Settings => "settings",
            Self::Software => "software",
            Self::Terminal => "terminal",
            Self::Trash => "trash",
            Self::Updates => "updates",
            Self::Volume => "volume",
            Self::Wifi => "wifi",
        }
    }

    pub const fn is_symbolic(self) -> bool {
        matches!(
            self,
            Self::Battery | Self::Home | Self::Notification | Self::Volume | Self::Wifi
        )
    }

    pub const fn source(self) -> &'static [u8] {
        match self {
            Self::AquaDrive => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/aqua-drive.svg")
            }
            Self::Battery => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/battery.svg")
            }
            Self::Browser => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/browser.svg")
            }
            Self::Files => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/files.svg")
            }
            Self::Home => include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/home.svg"),
            Self::Notification => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/notification.svg")
            }
            Self::Settings => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/settings.svg")
            }
            Self::Software => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/software.svg")
            }
            Self::Terminal => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/terminal.svg")
            }
            Self::Trash => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/trash.svg")
            }
            Self::Updates => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/updates.svg")
            }
            Self::Volume => {
                include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/volume.svg")
            }
            Self::Wifi => include_bytes!("../../../docs/aqua-linux/assets/icons/aqua/wifi.svg"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconState {
    #[default]
    Normal,
    Hover,
    Focused,
    Pressed,
    Selected,
    Disabled,
    Attention,
}

impl IconState {
    pub const ALL: [Self; 7] = [
        Self::Normal,
        Self::Hover,
        Self::Focused,
        Self::Pressed,
        Self::Selected,
        Self::Disabled,
        Self::Attention,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Hover => "hover",
            Self::Focused => "focused",
            Self::Pressed => "pressed",
            Self::Selected => "selected",
            Self::Disabled => "disabled",
            Self::Attention => "attention",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IconRasterKey {
    pub source_revision: &'static str,
    pub role: IconRole,
    pub theme: AquaTheme,
    pub state: IconState,
    pub logical_size: u16,
    pub output_scale: OutputScale,
}

impl IconRasterKey {
    pub fn new(
        role: IconRole,
        theme: AquaTheme,
        state: IconState,
        logical_size: u16,
        output_scale: OutputScale,
    ) -> Result<Self, IconError> {
        if !REQUIRED_LOGICAL_ICON_SIZES.contains(&logical_size) {
            return Err(IconError::UnsupportedLogicalSize(logical_size));
        }
        Ok(Self {
            source_revision: AQUA_CORE_ICON_REVISION,
            role,
            theme,
            state,
            logical_size,
            output_scale,
        })
    }

    pub const fn physical_size(self) -> u32 {
        let numerator = self.logical_size as u32 * self.output_scale.numerator() as u32;
        numerator.div_ceil(self.output_scale.denominator() as u32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconError {
    UnsupportedLogicalSize(u16),
    SourceTooLarge,
    SourceNotUtf8,
    MissingRoot,
    InvalidViewBox,
    ForbiddenFeature(&'static str),
    ParseFailed,
    AllocationFailed,
    InvalidRaster,
}

impl std::fmt::Display for IconError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedLogicalSize(size) => {
                write!(formatter, "unsupported logical icon size: {size}")
            }
            Self::SourceTooLarge => formatter.write_str("icon source exceeds the byte limit"),
            Self::SourceNotUtf8 => formatter.write_str("icon source is not UTF-8"),
            Self::MissingRoot => formatter.write_str("icon source is missing its SVG root"),
            Self::InvalidViewBox => formatter.write_str("icon source must use viewBox 0 0 64 64"),
            Self::ForbiddenFeature(feature) => {
                write!(
                    formatter,
                    "icon source contains forbidden SVG feature: {feature}"
                )
            }
            Self::ParseFailed => formatter.write_str("icon source could not be parsed"),
            Self::AllocationFailed => formatter.write_str("icon raster allocation failed"),
            Self::InvalidRaster => formatter.write_str("icon raster is empty or malformed"),
        }
    }
}

impl std::error::Error for IconError {}

pub fn validate_icon_source(source: &[u8]) -> Result<(), IconError> {
    if source.len() > MAX_ICON_SOURCE_BYTES {
        return Err(IconError::SourceTooLarge);
    }
    let text = std::str::from_utf8(source).map_err(|_| IconError::SourceNotUtf8)?;
    let compact = text.to_ascii_lowercase();
    if !compact.trim_start().starts_with("<svg") || !compact.contains("</svg>") {
        return Err(IconError::MissingRoot);
    }
    if !compact.contains(ICON_VIEW_BOX) {
        return Err(IconError::InvalidViewBox);
    }
    for (pattern, label) in [
        ("<!doctype", "doctype"),
        ("<!entity", "entity"),
        ("<?xml-stylesheet", "stylesheet"),
        ("<script", "script"),
        ("<style", "style"),
        ("<image", "image"),
        ("<text", "text"),
        ("<foreignobject", "foreignObject"),
        ("<filter", "filter"),
        ("<lineargradient", "linearGradient"),
        ("<radialgradient", "radialGradient"),
        ("<animate", "animation"),
        ("<set", "animation"),
        ("href=", "external reference"),
        ("url(", "external reference"),
        ("onload=", "event handler"),
        ("onclick=", "event handler"),
    ] {
        if compact.contains(pattern) {
            return Err(IconError::ForbiddenFeature(label));
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct IconRaster {
    pub key: IconRasterKey,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub non_transparent_pixels: usize,
    pub opaque_pixels: usize,
    pub checksum: u64,
}

impl IconRaster {
    pub fn is_ready(&self) -> bool {
        self.width == self.key.physical_size()
            && self.height == self.width
            && self.rgba.len() == self.width as usize * self.height as usize * 4
            && self.non_transparent_pixels > 0
            && self.non_transparent_pixels < self.width as usize * self.height as usize
            && (self.opaque_pixels > 0 || self.key.state == IconState::Disabled)
            && self.checksum != 0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IconRasterCacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub parsed_sources: usize,
}

pub struct IconRasterCache {
    capacity: usize,
    rasters: HashMap<IconRasterKey, Arc<IconRaster>>,
    order: VecDeque<IconRasterKey>,
    trees: HashMap<IconRole, Tree>,
    stats: IconRasterCacheStats,
}

impl Default for IconRasterCache {
    fn default() -> Self {
        Self::new(DEFAULT_ICON_RASTER_CACHE_CAPACITY)
    }
}

impl IconRasterCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            rasters: HashMap::new(),
            order: VecDeque::new(),
            trees: HashMap::new(),
            stats: IconRasterCacheStats::default(),
        }
    }

    pub fn get_or_render(&mut self, key: IconRasterKey) -> Result<Arc<IconRaster>, IconError> {
        if let Some(raster) = self.rasters.get(&key).cloned() {
            self.stats.hits += 1;
            self.touch(key);
            return Ok(raster);
        }
        self.stats.misses += 1;
        if let std::collections::hash_map::Entry::Vacant(entry) = self.trees.entry(key.role) {
            let source = key.role.source();
            validate_icon_source(source)?;
            let tree =
                Tree::from_data(source, &Options::default()).map_err(|_| IconError::ParseFailed)?;
            entry.insert(tree);
            self.stats.parsed_sources += 1;
        }
        let tree = self.trees.get(&key.role).ok_or(IconError::ParseFailed)?;
        let raster = Arc::new(render_icon_tree(key, tree)?);
        if self.rasters.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.rasters.remove(&oldest);
                self.stats.evictions += 1;
            }
        }
        self.rasters.insert(key, Arc::clone(&raster));
        self.order.push_back(key);
        Ok(raster)
    }

    pub fn len(&self) -> usize {
        self.rasters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rasters.is_empty()
    }

    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    pub const fn stats(&self) -> IconRasterCacheStats {
        self.stats
    }

    fn touch(&mut self, key: IconRasterKey) {
        if let Some(index) = self.order.iter().position(|candidate| *candidate == key) {
            self.order.remove(index);
        }
        self.order.push_back(key);
    }
}

fn render_icon_tree(key: IconRasterKey, tree: &Tree) -> Result<IconRaster, IconError> {
    let physical_size = key.physical_size();
    let mut pixmap =
        Pixmap::new(physical_size, physical_size).ok_or(IconError::AllocationFailed)?;
    let tree_size = tree.size();
    let transform = Transform::from_scale(
        physical_size as f32 / tree_size.width(),
        physical_size as f32 / tree_size.height(),
    );
    resvg::render(tree, transform, &mut pixmap.as_mut());
    let mut rgba = pixmap.take();
    unpremultiply_rgba(&mut rgba);
    apply_theme_and_state(&mut rgba, key);
    let non_transparent_pixels = rgba.chunks_exact(4).filter(|pixel| pixel[3] > 0).count();
    let opaque_pixels = rgba
        .chunks_exact(4)
        .filter(|pixel| pixel[3] == u8::MAX)
        .count();
    let raster = IconRaster {
        key,
        width: physical_size,
        height: physical_size,
        checksum: checksum_bytes(&rgba),
        rgba,
        non_transparent_pixels,
        opaque_pixels,
    };
    if !raster.is_ready() {
        return Err(IconError::InvalidRaster);
    }
    Ok(raster)
}

fn unpremultiply_rgba(rgba: &mut [u8]) {
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 || alpha == 255 {
            continue;
        }
        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
        }
    }
}

fn apply_theme_and_state(rgba: &mut [u8], key: IconRasterKey) {
    let (ink, accent) = symbolic_palette(key.theme);
    for pixel in rgba.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        if key.role.is_symbolic() {
            let source_is_accent =
                pixel[2] > pixel[0].saturating_add(24) && pixel[2] > pixel[1].saturating_add(24);
            pixel[..3].copy_from_slice(if source_is_accent { &accent } else { &ink });
        }
        match key.state {
            IconState::Normal => {}
            IconState::Hover => adjust_rgb(pixel, 108, None),
            IconState::Focused => adjust_rgb(pixel, 104, Some(accent)),
            IconState::Pressed => adjust_rgb(pixel, 84, None),
            IconState::Selected => adjust_rgb(pixel, 100, Some(accent)),
            IconState::Disabled => pixel[3] = ((u16::from(pixel[3]) * 42) / 100) as u8,
            IconState::Attention => adjust_rgb(pixel, 96, Some(attention_color(key.theme))),
        }
    }
}

fn adjust_rgb(pixel: &mut [u8], percent: u16, mix: Option<[u8; 3]>) {
    for (index, channel) in pixel[..3].iter_mut().enumerate() {
        let adjusted = (u16::from(*channel) * percent / 100).min(255) as u8;
        *channel = if let Some(mix) = mix {
            ((u16::from(adjusted) * 4 + u16::from(mix[index])) / 5) as u8
        } else {
            adjusted
        };
    }
}

const fn symbolic_palette(theme: AquaTheme) -> ([u8; 3], [u8; 3]) {
    match theme {
        AquaTheme::LightWhite => ([0x10, 0x18, 0x28], [0x16, 0x86, 0xf5]),
        AquaTheme::Softtouch => ([0x26, 0x3b, 0x53], [0x00, 0x9a, 0xce]),
        AquaTheme::Deepside => ([0xe1, 0xee, 0xfa], [0x62, 0xc6, 0xff]),
        AquaTheme::Nightmare => ([0xe8, 0xef, 0xf6], [0x60, 0xb7, 0xff]),
    }
}

const fn attention_color(theme: AquaTheme) -> [u8; 3] {
    match theme {
        AquaTheme::LightWhite | AquaTheme::Softtouch => [0xd8, 0x55, 0x32],
        AquaTheme::Deepside | AquaTheme::Nightmare => [0xff, 0xa1, 0x79],
    }
}

pub fn composite_icon(
    destination: &mut [u8],
    destination_width: u32,
    destination_height: u32,
    x: u32,
    y: u32,
    icon: &IconRaster,
) {
    for source_y in 0..icon.height.min(destination_height.saturating_sub(y)) {
        for source_x in 0..icon.width.min(destination_width.saturating_sub(x)) {
            let source_offset = ((source_y * icon.width + source_x) * 4) as usize;
            let target_offset = (((y + source_y) * destination_width + x + source_x) * 4) as usize;
            let source_alpha = u16::from(icon.rgba[source_offset + 3]);
            let inverse = 255 - source_alpha;
            for channel in 0..3 {
                destination[target_offset + channel] =
                    ((u16::from(icon.rgba[source_offset + channel]) * source_alpha
                        + u16::from(destination[target_offset + channel]) * inverse
                        + 127)
                        / 255) as u8;
            }
            destination[target_offset + 3] = (source_alpha
                + (u16::from(destination[target_offset + 3]) * inverse + 127) / 255)
                .min(255) as u8;
        }
    }
}

#[derive(Debug, Clone)]
pub struct IconAcceptanceProbe {
    pub revision: &'static str,
    pub role_count: usize,
    pub case_count: usize,
    pub dimension_count: usize,
    pub all_sources_valid: bool,
    pub all_rasters_ready: bool,
    pub transparent_canvas: bool,
    pub silhouette_stable: bool,
    pub state_key_distinct: bool,
    pub cache_entries: usize,
    pub cache_stats: IconRasterCacheStats,
    pub checksum: u64,
}

impl IconAcceptanceProbe {
    pub fn is_ready(&self) -> bool {
        self.revision == ICON_FIXTURE_REVISION
            && self.role_count == IconRole::ALL.len()
            && self.case_count
                == IconRole::ALL.len()
                    * AquaTheme::ALL.len()
                    * OutputScale::ALL.len()
                    * REQUIRED_LOGICAL_ICON_SIZES.len()
            && self.dimension_count >= REQUIRED_LOGICAL_ICON_SIZES.len()
            && self.all_sources_valid
            && self.all_rasters_ready
            && self.transparent_canvas
            && self.silhouette_stable
            && self.state_key_distinct
            && self.cache_entries == DEFAULT_ICON_RASTER_CACHE_CAPACITY
            && self.cache_stats.parsed_sources == IconRole::ALL.len()
            && self.cache_stats.hits == self.case_count + 1
            && self.cache_stats.misses == self.case_count + IconState::ALL.len() - 1
            && self.cache_stats.evictions
                == self.cache_stats.misses - DEFAULT_ICON_RASTER_CACHE_CAPACITY
            && self.checksum != 0
    }
}

pub fn icon_acceptance_report() -> String {
    let (role_lines, probe) = run_icon_acceptance();
    let mut report = format!("revision={}\n", probe.revision);
    for line in role_lines {
        report.push_str(&line);
        report.push('\n');
    }
    report.push_str(&format!(
        "summary roles={} cases={} dimensions={} sources_valid={} rasters_ready={} transparent={} silhouette_stable={} state_keys={} entries={} hits={} misses={} evictions={} parsed_sources={} checksum={:016x} ready={}\n",
        probe.role_count,
        probe.case_count,
        probe.dimension_count,
        probe.all_sources_valid,
        probe.all_rasters_ready,
        probe.transparent_canvas,
        probe.silhouette_stable,
        probe.state_key_distinct,
        probe.cache_entries,
        probe.cache_stats.hits,
        probe.cache_stats.misses,
        probe.cache_stats.evictions,
        probe.cache_stats.parsed_sources,
        probe.checksum,
        probe.is_ready(),
    ));
    report
}

fn run_icon_acceptance() -> (Vec<String>, IconAcceptanceProbe) {
    let mut cache = IconRasterCache::default();
    let mut lines = Vec::new();
    let mut dimensions = HashSet::new();
    let mut case_count = 0;
    let mut all_sources_valid = true;
    let mut all_rasters_ready = true;
    let mut transparent_canvas = true;
    let mut silhouette_stable = true;
    let mut checksum = 0xcbf2_9ce4_8422_2325;

    for role in IconRole::ALL {
        all_sources_valid &= validate_icon_source(role.source()).is_ok();
        let mut role_cases = 0;
        let mut role_checksum = 0xcbf2_9ce4_8422_2325;
        let mut coverage_min = f32::MAX;
        let mut coverage_max = 0.0_f32;
        for theme in AquaTheme::ALL {
            for scale in OutputScale::ALL {
                for logical_size in REQUIRED_LOGICAL_ICON_SIZES {
                    let key =
                        IconRasterKey::new(role, theme, IconState::Normal, logical_size, scale)
                            .expect("fixed acceptance icon key");
                    let raster = cache
                        .get_or_render(key)
                        .expect("reviewed Aqua icon should render");
                    let reused = cache
                        .get_or_render(key)
                        .expect("cached Aqua icon should render");
                    all_rasters_ready &= raster.is_ready() && Arc::ptr_eq(&raster, &reused);
                    transparent_canvas &= raster.non_transparent_pixels
                        < raster.width as usize * raster.height as usize;
                    let coverage = raster.non_transparent_pixels as f32
                        / (raster.width as f32 * raster.height as f32);
                    coverage_min = coverage_min.min(coverage);
                    coverage_max = coverage_max.max(coverage);
                    dimensions.insert(raster.width);
                    role_checksum ^= raster.checksum;
                    role_checksum = role_checksum.wrapping_mul(0x100_0000_01b3);
                    checksum ^= raster.checksum;
                    checksum = checksum.wrapping_mul(0x100_0000_01b3);
                    role_cases += 1;
                    case_count += 1;
                }
            }
        }
        let role_silhouette_stable =
            coverage_min > 0.08 && coverage_max < 0.94 && coverage_max - coverage_min < 0.20;
        silhouette_stable &= role_silhouette_stable;
        lines.push(format!(
            "role={} kind={} cases={} source_bytes={} coverage={:.3}-{:.3} silhouette_stable={} checksum={:016x} ready={}",
            role.id(),
            if role.is_symbolic() { "symbolic" } else { "full-color" },
            role_cases,
            role.source().len(),
            coverage_min,
            coverage_max,
            role_silhouette_stable,
            role_checksum,
            role_cases == AquaTheme::ALL.len() * OutputScale::ALL.len() * REQUIRED_LOGICAL_ICON_SIZES.len(),
        ));
    }

    let mut state_checksums = HashSet::new();
    for state in IconState::ALL {
        let key = IconRasterKey::new(
            IconRole::Wifi,
            AquaTheme::Nightmare,
            state,
            32,
            OutputScale::FiveQuarters,
        )
        .expect("fixed state acceptance key");
        state_checksums.insert(
            cache
                .get_or_render(key)
                .expect("state icon should render")
                .checksum,
        );
    }
    let stats = cache.stats();
    (
        lines,
        IconAcceptanceProbe {
            revision: ICON_FIXTURE_REVISION,
            role_count: IconRole::ALL.len(),
            case_count,
            dimension_count: dimensions.len(),
            all_sources_valid,
            all_rasters_ready,
            transparent_canvas,
            silhouette_stable,
            state_key_distinct: state_checksums.len() == IconState::ALL.len(),
            cache_entries: cache.len(),
            cache_stats: stats,
            checksum,
        },
    )
}

fn checksum_bytes(bytes: &[u8]) -> u64 {
    let mut checksum = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        checksum ^= u64::from(*byte);
        checksum = checksum.wrapping_mul(0x100_0000_01b3);
    }
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_sources_reject_active_and_external_svg_features() {
        for role in IconRole::ALL {
            validate_icon_source(role.source()).expect("committed icon should be reviewed");
        }
        assert_eq!(
            validate_icon_source(br#"<svg viewBox="0 0 64 64"><script/></svg>"#),
            Err(IconError::ForbiddenFeature("script"))
        );
        assert_eq!(
            validate_icon_source(br#"<svg viewBox="0 0 64 64"><image href="x"/></svg>"#),
            Err(IconError::ForbiddenFeature("image"))
        );
    }

    #[test]
    fn cache_is_scale_native_bounded_and_key_complete() {
        let mut cache = IconRasterCache::new(2);
        let first = IconRasterKey::new(
            IconRole::Files,
            AquaTheme::LightWhite,
            IconState::Normal,
            20,
            OutputScale::FiveQuarters,
        )
        .unwrap();
        let selected = IconRasterKey::new(
            IconRole::Files,
            AquaTheme::LightWhite,
            IconState::Selected,
            20,
            OutputScale::FiveQuarters,
        )
        .unwrap();
        let scaled = IconRasterKey::new(
            IconRole::Files,
            AquaTheme::LightWhite,
            IconState::Normal,
            20,
            OutputScale::Two,
        )
        .unwrap();
        let original = cache.get_or_render(first).unwrap();
        let reused = cache.get_or_render(first).unwrap();
        assert!(Arc::ptr_eq(&original, &reused));
        assert_eq!((original.width, original.height), (25, 25));
        assert_ne!(
            original.checksum,
            cache.get_or_render(selected).unwrap().checksum
        );
        assert_eq!(cache.get_or_render(scaled).unwrap().width, 40);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 3);
        assert_eq!(cache.stats().evictions, 1);
        assert_eq!(cache.stats().parsed_sources, 1);
    }

    #[test]
    fn acceptance_matrix_covers_roles_themes_states_sizes_and_scales() {
        let (_, probe) = run_icon_acceptance();
        assert!(probe.is_ready(), "{probe:#?}");
    }
}
