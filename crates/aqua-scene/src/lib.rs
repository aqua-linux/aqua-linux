pub const PRODUCT: &str = "Aqua Linux";
pub const SCENE_STATUS: &str = "static-shell-model";
pub const RUNTIME_ASSET_ROOT: &str = "/usr/share/aqua";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn is_supported(self) -> bool {
        self.width >= 800 && self.height >= 600
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn right(self) -> u32 {
        self.x + self.width
    }

    pub const fn bottom(self) -> u32 {
        self.y + self.height
    }

    pub const fn contains_viewport(self, viewport: Viewport) -> bool {
        self.x == 0 && self.y == 0 && self.width == viewport.width && self.height == viewport.height
    }

    pub const fn fits_in(self, viewport: Viewport) -> bool {
        self.right() <= viewport.width && self.bottom() <= viewport.height
    }

    pub const fn overlaps(self, other: Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Wallpaper,
    TopPanel,
    DesktopIconColumn,
    Dock,
    Launcher,
    SystemOverview,
    NotificationToast,
}

impl SurfaceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wallpaper => "wallpaper",
            Self::TopPanel => "top-panel",
            Self::DesktopIconColumn => "desktop-icon-column",
            Self::Dock => "dock",
            Self::Launcher => "launcher",
            Self::SystemOverview => "system-overview",
            Self::NotificationToast => "notification-toast",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialKind {
    Image,
    SystemSurface,
    IconGrid,
}

impl MaterialKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::SystemSurface => "system-surface",
            Self::IconGrid => "icon-grid",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSurface {
    pub id: &'static str,
    pub kind: SurfaceKind,
    pub material: MaterialKind,
    pub rect: Rect,
    pub mock_data: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellScene {
    pub product: &'static str,
    pub status: &'static str,
    pub viewport: Viewport,
    pub surfaces: Vec<SceneSurface>,
    pub assets: Vec<SceneAssetUse>,
    pub material_tokens: Vec<SceneMaterialToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneAssetUse {
    pub surface_id: &'static str,
    pub role: &'static str,
    pub runtime_path: &'static str,
    pub temporary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMaterialToken {
    pub surface_id: &'static str,
    pub role: &'static str,
    pub token_path: &'static str,
}

impl ShellScene {
    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("product={}", self.product),
            format!("scene_status={}", self.status),
            format!("viewport={}x{}", self.viewport.width, self.viewport.height),
            format!("surface_count={}", self.surfaces.len()),
        ];

        lines.extend(self.surfaces.iter().map(|surface| {
            format!(
                "surface id={} kind={} material={} rect={},{},{},{} mock_data={}",
                surface.id,
                surface.kind.as_str(),
                surface.material.as_str(),
                surface.rect.x,
                surface.rect.y,
                surface.rect.width,
                surface.rect.height,
                surface.mock_data
            )
        }));

        lines.extend(self.assets.iter().map(|asset| {
            format!(
                "asset surface={} role={} path={} temporary={}",
                asset.surface_id, asset.role, asset.runtime_path, asset.temporary
            )
        }));

        lines.extend(self.material_tokens.iter().map(|token| {
            format!(
                "material surface={} role={} token={}",
                token.surface_id, token.role, token.token_path
            )
        }));

        lines
    }

    pub fn required_assets_present(&self) -> bool {
        self.has_asset(
            "wallpaper",
            "/usr/share/aqua/wallpapers/default-wallpaper.png",
        ) && self.has_asset("launcher", "/usr/share/aqua/brand/aqua-symbol-primary.png")
            && self.has_asset("dock", "/usr/share/aqua/icons/aqua/browser.svg")
            && self.has_asset(
                "notification-toast",
                "/usr/share/aqua/icons/aqua/updates.svg",
            )
    }

    pub fn permanent_assets_only(&self) -> bool {
        self.assets
            .iter()
            .all(|asset| !asset.temporary && !asset.runtime_path.contains("/icons/temp/"))
    }

    pub fn required_material_tokens_present(&self) -> bool {
        SYSTEM_SURFACE_SURFACES
            .iter()
            .all(|surface_id| self.surface_has_system_surface_tokens(surface_id))
    }

    pub fn simulated_surface_is_labeled(&self) -> bool {
        SYSTEM_SURFACE_SURFACES.iter().all(|surface_id| {
            self.has_material_token(surface_id, "effect", "surface.layeredSurface")
        })
    }

    pub fn required_surface_count(&self) -> usize {
        REQUIRED_KINDS
            .iter()
            .filter(|kind| self.has_surface(**kind))
            .count()
    }

    pub fn has_surface(&self, kind: SurfaceKind) -> bool {
        self.surfaces.iter().any(|surface| surface.kind == kind)
    }

    pub fn surface_is_visible(&self, kind: SurfaceKind) -> bool {
        self.surfaces
            .iter()
            .find(|surface| surface.kind == kind)
            .is_some_and(|surface| surface.visible)
    }

    pub fn set_surface_visible(&mut self, kind: SurfaceKind, visible: bool) -> bool {
        let Some(surface) = self
            .surfaces
            .iter_mut()
            .find(|surface| surface.kind == kind)
        else {
            return false;
        };

        surface.visible = visible;
        true
    }

    pub fn visible_surface_count(&self) -> usize {
        self.surfaces
            .iter()
            .filter(|surface| surface.visible)
            .count()
    }

    pub fn surfaces_fit_viewport(&self) -> bool {
        self.surfaces
            .iter()
            .all(|surface| surface.rect.fits_in(self.viewport))
    }

    pub fn wallpaper_covers_viewport(&self) -> bool {
        self.surfaces.iter().any(|surface| {
            surface.kind == SurfaceKind::Wallpaper && surface.rect.contains_viewport(self.viewport)
        })
    }

    pub fn toast_avoids_dock(&self) -> bool {
        non_overlapping(
            self.surface_rect(SurfaceKind::NotificationToast),
            self.surface_rect(SurfaceKind::Dock),
        )
    }

    pub fn launcher_avoids_dock(&self) -> bool {
        non_overlapping(
            self.surface_rect(SurfaceKind::Launcher),
            self.surface_rect(SurfaceKind::Dock),
        )
    }

    pub fn mock_surfaces_are_labeled(&self) -> bool {
        self.surfaces.iter().any(|surface| surface.mock_data)
    }

    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == SCENE_STATUS
            && self.viewport.is_supported()
            && self.required_surface_count() == REQUIRED_KINDS.len()
            && self.surfaces_fit_viewport()
            && self.wallpaper_covers_viewport()
            && self.toast_avoids_dock()
            && self.launcher_avoids_dock()
            && self.mock_surfaces_are_labeled()
            && self.required_assets_present()
            && self.permanent_assets_only()
            && self.required_material_tokens_present()
            && self.simulated_surface_is_labeled()
    }

    fn surface_has_system_surface_tokens(&self, surface_id: &str) -> bool {
        self.has_material_token(surface_id, "fill", "surface.panelFill")
            && self.has_material_token(surface_id, "border", "surface.panelBorder")
            && self.has_material_token(surface_id, "highlight", "surface.topHighlight")
            && self.has_material_token(surface_id, "shadow", "surface.bottomShadow")
            && self.has_material_token(surface_id, "glow", "surface.outerGlow")
            && self.has_material_token(surface_id, "blur", "surface.blurRadius")
    }

    fn has_material_token(&self, surface_id: &str, role: &str, token_path: &str) -> bool {
        self.material_tokens.iter().any(|token| {
            token.surface_id == surface_id && token.role == role && token.token_path == token_path
        })
    }

    fn has_asset(&self, surface_id: &str, runtime_path: &str) -> bool {
        self.assets
            .iter()
            .any(|asset| asset.surface_id == surface_id && asset.runtime_path == runtime_path)
    }

    pub fn surface_rect(&self, kind: SurfaceKind) -> Option<Rect> {
        self.surfaces
            .iter()
            .find(|surface| surface.kind == kind)
            .map(|surface| surface.rect)
    }
}

pub const REQUIRED_KINDS: [SurfaceKind; 7] = [
    SurfaceKind::Wallpaper,
    SurfaceKind::TopPanel,
    SurfaceKind::DesktopIconColumn,
    SurfaceKind::Dock,
    SurfaceKind::Launcher,
    SurfaceKind::SystemOverview,
    SurfaceKind::NotificationToast,
];

pub const SYSTEM_SURFACE_SURFACES: [&str; 5] = [
    "top-panel",
    "dock",
    "launcher",
    "system-overview",
    "notification-toast",
];

pub fn static_shell_scene(viewport: Viewport) -> ShellScene {
    let margin = 24;
    let top_panel_height = 36;
    let dock_width = 760_u32.min(viewport.width.saturating_sub(margin * 2));
    let dock_height = 72;
    let launcher_width = 560_u32.min(viewport.width.saturating_sub(margin * 2));
    let launcher_height = 520_u32.min(viewport.height.saturating_sub(160));
    let system_width = 320_u32.min(viewport.width.saturating_sub(margin * 2));
    let toast_width = 360_u32.min(viewport.width.saturating_sub(margin * 2));

    ShellScene {
        product: PRODUCT,
        status: SCENE_STATUS,
        viewport,
        surfaces: vec![
            surface(
                "wallpaper",
                SurfaceKind::Wallpaper,
                MaterialKind::Image,
                Rect {
                    x: 0,
                    y: 0,
                    width: viewport.width,
                    height: viewport.height,
                },
                false,
            ),
            surface(
                "top-panel",
                SurfaceKind::TopPanel,
                MaterialKind::SystemSurface,
                Rect {
                    x: 0,
                    y: 0,
                    width: viewport.width,
                    height: top_panel_height,
                },
                true,
            ),
            surface(
                "desktop-icons",
                SurfaceKind::DesktopIconColumn,
                MaterialKind::IconGrid,
                Rect {
                    x: margin,
                    y: top_panel_height + margin,
                    width: 232,
                    height: 312_u32.min(
                        viewport
                            .height
                            .saturating_sub(top_panel_height + dock_height + margin * 3),
                    ),
                },
                true,
            ),
            surface(
                "dock",
                SurfaceKind::Dock,
                MaterialKind::SystemSurface,
                Rect {
                    x: (viewport.width - dock_width) / 2,
                    y: viewport.height - dock_height - margin,
                    width: dock_width,
                    height: dock_height,
                },
                true,
            ),
            surface(
                "launcher",
                SurfaceKind::Launcher,
                MaterialKind::SystemSurface,
                Rect {
                    x: margin,
                    y: top_panel_height + margin,
                    width: launcher_width,
                    height: launcher_height,
                },
                true,
            ),
            surface(
                "system-overview",
                SurfaceKind::SystemOverview,
                MaterialKind::SystemSurface,
                Rect {
                    x: viewport.width - system_width - margin,
                    y: top_panel_height + margin,
                    width: system_width,
                    height: 220,
                },
                true,
            ),
            surface(
                "notification-toast",
                SurfaceKind::NotificationToast,
                MaterialKind::SystemSurface,
                Rect {
                    x: viewport.width - toast_width - margin,
                    y: viewport.height - dock_height - 112,
                    width: toast_width,
                    height: 88,
                },
                true,
            ),
        ],
        assets: scene_assets(),
        material_tokens: scene_material_tokens(),
    }
}

fn scene_material_tokens() -> Vec<SceneMaterialToken> {
    SYSTEM_SURFACE_SURFACES
        .iter()
        .flat_map(|surface_id| system_surface_tokens(surface_id))
        .collect()
}

fn system_surface_tokens(surface_id: &'static str) -> Vec<SceneMaterialToken> {
    vec![
        material_token(surface_id, "fill", "surface.panelFill"),
        material_token(surface_id, "border", "surface.panelBorder"),
        material_token(surface_id, "highlight", "surface.topHighlight"),
        material_token(surface_id, "shadow", "surface.bottomShadow"),
        material_token(surface_id, "glow", "surface.outerGlow"),
        material_token(surface_id, "blur", "surface.blurRadius"),
        material_token(surface_id, "effect", "surface.layeredSurface"),
    ]
}

fn material_token(
    surface_id: &'static str,
    role: &'static str,
    token_path: &'static str,
) -> SceneMaterialToken {
    SceneMaterialToken {
        surface_id,
        role,
        token_path,
    }
}

fn scene_assets() -> Vec<SceneAssetUse> {
    vec![
        asset(
            "wallpaper",
            "background",
            "/usr/share/aqua/wallpapers/default-wallpaper.png",
            false,
        ),
        asset(
            "top-panel",
            "brand-mark",
            "/usr/share/aqua/brand/aqua-symbol-primary.png",
            false,
        ),
        asset(
            "desktop-icons",
            "home-icon",
            "/usr/share/aqua/icons/aqua/home.svg",
            false,
        ),
        asset(
            "desktop-icons",
            "files-icon",
            "/usr/share/aqua/icons/aqua/files.svg",
            false,
        ),
        asset(
            "desktop-icons",
            "drive-icon",
            "/usr/share/aqua/icons/aqua/aqua-drive.svg",
            false,
        ),
        asset(
            "desktop-icons",
            "trash-icon",
            "/usr/share/aqua/icons/aqua/trash.svg",
            false,
        ),
        asset(
            "dock",
            "aqua-icon",
            "/usr/share/aqua/brand/aqua-symbol-accent.png",
            false,
        ),
        asset(
            "dock",
            "browser-icon",
            "/usr/share/aqua/icons/aqua/browser.svg",
            false,
        ),
        asset(
            "dock",
            "terminal-icon",
            "/usr/share/aqua/icons/aqua/terminal.svg",
            false,
        ),
        asset(
            "dock",
            "settings-icon",
            "/usr/share/aqua/icons/aqua/settings.svg",
            false,
        ),
        asset(
            "system-overview",
            "wifi-icon",
            "/usr/share/aqua/icons/aqua/wifi.svg",
            false,
        ),
        asset(
            "system-overview",
            "volume-icon",
            "/usr/share/aqua/icons/aqua/volume.svg",
            false,
        ),
        asset(
            "system-overview",
            "battery-icon",
            "/usr/share/aqua/icons/aqua/battery.svg",
            false,
        ),
        asset(
            "launcher",
            "brand-symbol",
            "/usr/share/aqua/brand/aqua-symbol-primary.png",
            false,
        ),
        asset(
            "notification-toast",
            "update-icon",
            "/usr/share/aqua/icons/aqua/updates.svg",
            false,
        ),
    ]
}

fn asset(
    surface_id: &'static str,
    role: &'static str,
    runtime_path: &'static str,
    temporary: bool,
) -> SceneAssetUse {
    SceneAssetUse {
        surface_id,
        role,
        runtime_path,
        temporary,
    }
}

fn surface(
    id: &'static str,
    kind: SurfaceKind,
    material: MaterialKind,
    rect: Rect,
    mock_data: bool,
) -> SceneSurface {
    SceneSurface {
        id,
        kind,
        material,
        rect,
        mock_data,
        visible: true,
    }
}

fn non_overlapping(first: Option<Rect>, second: Option<Rect>) -> bool {
    match (first, second) {
        (Some(first), Some(second)) => !first.overlaps(second),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_scene_contains_visual_reference_surfaces() {
        let scene = static_shell_scene(Viewport::new(1536, 1024));

        assert!(scene.has_surface(SurfaceKind::Wallpaper));
        assert!(scene.has_surface(SurfaceKind::TopPanel));
        assert!(scene.has_surface(SurfaceKind::DesktopIconColumn));
        assert!(scene.has_surface(SurfaceKind::Dock));
        assert!(scene.has_surface(SurfaceKind::Launcher));
        assert!(scene.has_surface(SurfaceKind::SystemOverview));
        assert!(scene.has_surface(SurfaceKind::NotificationToast));
        assert_eq!(scene.required_surface_count(), REQUIRED_KINDS.len());
        assert_eq!(scene.visible_surface_count(), REQUIRED_KINDS.len());
    }

    #[test]
    fn interactive_surface_visibility_can_be_changed_without_removing_geometry() {
        let mut scene = static_shell_scene(Viewport::new(1536, 1024));

        assert!(scene.surface_is_visible(SurfaceKind::Launcher));
        assert!(scene.set_surface_visible(SurfaceKind::Launcher, false));
        assert!(!scene.surface_is_visible(SurfaceKind::Launcher));
        assert!(scene.has_surface(SurfaceKind::Launcher));
        assert_eq!(scene.visible_surface_count(), REQUIRED_KINDS.len() - 1);
    }

    #[test]
    fn static_scene_geometry_is_ready_for_reference_viewport() {
        let scene = static_shell_scene(Viewport::new(1536, 1024));

        assert!(scene.is_ready());
        assert!(scene.surfaces_fit_viewport());
        assert!(scene.wallpaper_covers_viewport());
        assert!(scene.toast_avoids_dock());
        assert!(scene.launcher_avoids_dock());
        assert!(scene.required_assets_present());
        assert!(scene.permanent_assets_only());
        assert!(scene.required_material_tokens_present());
        assert!(scene.simulated_surface_is_labeled());
    }

    #[test]
    fn static_scene_scales_to_small_supported_viewport() {
        let scene = static_shell_scene(Viewport::new(800, 600));

        assert!(scene.is_ready());
        assert!(scene.surfaces_fit_viewport());
    }

    #[test]
    fn dump_lines_are_stable_and_renderer_friendly() {
        let scene = static_shell_scene(Viewport::new(1536, 1024));
        let lines = scene.dump_lines();

        assert_eq!(lines[0], "product=Aqua Linux");
        assert_eq!(lines[1], "scene_status=static-shell-model");
        assert_eq!(lines[2], "viewport=1536x1024");
        assert_eq!(lines[3], "surface_count=7");
        assert!(lines.contains(
            &"surface id=launcher kind=launcher material=system-surface rect=24,60,560,520 mock_data=true"
                .to_string()
        ));
        assert!(lines.contains(
            &"asset surface=wallpaper role=background path=/usr/share/aqua/wallpapers/default-wallpaper.png temporary=false"
                .to_string()
        ));
        assert!(lines.contains(
            &"asset surface=dock role=browser-icon path=/usr/share/aqua/icons/aqua/browser.svg temporary=false"
                .to_string()
        ));
        assert!(lines
            .contains(&"material surface=launcher role=fill token=surface.panelFill".to_string()));
        assert!(lines.contains(
            &"material surface=notification-toast role=effect token=surface.layeredSurface"
                .to_string()
        ));
    }
}
