use aqua_components::{
    ActivationKey, ApplicationOverview, ComponentState, ConfirmationDialog,
    ConfirmationPresentation, ConfirmationRequirement, ConfirmationSeverity, ConfirmationState,
    GlobalSearch, GridCell, GridCellLayout, IconButton, IconButtonGlyph, ListNavigation,
    ListNavigationKey, ListRow, ListRowRole, Menu, MetadataRow, RunningAppDock, SearchField,
    SectionGroup, SegmentNavigationKey, SegmentedControl, SidebarNavigation, SidebarNavigationKey,
    Slider, SliderKey, StandardButton, StandardButtonVariant, SwitchControl, Toolbar, TopSystemBar,
    WorkspaceSwitcher,
};
pub use aqua_components::{CollectionNavigationKey, MenuNavigationKey, WorkspaceNavigationKey};
use aqua_scene::Rect;
use aqua_service_adapters::network_broker::{
    request_wifi_broker, request_wifi_connect, request_wifi_scan, WifiBrokerOperation,
};
use aqua_service_adapters::wifi_control::{
    WifiPassphrase, WifiScanNetwork, WifiScanSecurity, MAX_WIFI_PASSPHRASE_BYTES,
    MIN_WIFI_PASSPHRASE_BYTES,
};
use aqua_service_adapters::{
    read_network_snapshot, AudioAdapterError, AudioAuthoritativeState, AudioBackend,
    AudioBackendDriveError, AudioBackendDriveOutcome, AudioRequest, AudioServiceAdapter,
    AudioServiceHealth, NetworkAuthoritativeState, NetworkSnapshotError,
};
use std::collections::VecDeque;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const SETTINGS_CONFIG_VERSION: u8 = 1;
pub const SETTINGS_WIFI_BROKER_SOCKET_PATH: &str =
    aqua_service_adapters::network_broker::NETWORK_BROKER_SOCKET_PATH;
pub const MAX_WIFI_CONNECT_ATTEMPTS: u8 = 2;
pub const MAX_VISIBLE_WIFI_NETWORKS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiSettingsControl {
    available: bool,
    state: String,
    network_id: Option<u16>,
    authoritative: bool,
    credential_saved: bool,
    last_error: Option<String>,
    networks: Vec<WifiScanNetwork>,
    selected_network: Option<usize>,
    credential_entry: bool,
    passphrase: WifiSecretInput,
    connect_attempts: u8,
}

#[derive(Clone, PartialEq, Eq)]
struct WifiSecretInput {
    bytes: [u8; MAX_WIFI_PASSPHRASE_BYTES],
    length: usize,
}

impl Default for WifiSecretInput {
    fn default() -> Self {
        Self {
            bytes: [0; MAX_WIFI_PASSPHRASE_BYTES],
            length: 0,
        }
    }
}

impl std::fmt::Debug for WifiSecretInput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WifiSecretInput")
            .field("bytes", &"[redacted]")
            .field("length", &self.length)
            .finish()
    }
}

impl Drop for WifiSecretInput {
    fn drop(&mut self) {
        self.clear();
    }
}

impl WifiSecretInput {
    fn push(&mut self, character: char) -> bool {
        if self.length >= MAX_WIFI_PASSPHRASE_BYTES || !character.is_ascii() {
            return false;
        }
        let byte = character as u8;
        if !(0x20..=0x7e).contains(&byte) {
            return false;
        }
        self.bytes[self.length] = byte;
        self.length += 1;
        true
    }

    fn pop(&mut self) -> bool {
        if self.length == 0 {
            return false;
        }
        self.length -= 1;
        unsafe { std::ptr::write_volatile(&mut self.bytes[self.length], 0) };
        true
    }

    fn with_bytes<T>(&self, operation: impl FnOnce(&[u8]) -> T) -> T {
        operation(&self.bytes[..self.length])
    }

    fn clear(&mut self) {
        for byte in &mut self.bytes {
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
        self.length = 0;
    }
}

impl Default for WifiSettingsControl {
    fn default() -> Self {
        Self {
            available: false,
            state: "unavailable".to_owned(),
            network_id: None,
            authoritative: false,
            credential_saved: false,
            last_error: None,
            networks: Vec::new(),
            selected_network: None,
            credential_entry: false,
            passphrase: WifiSecretInput::default(),
            connect_attempts: 0,
        }
    }
}

impl WifiSettingsControl {
    pub const fn available(&self) -> bool {
        self.available
    }

    pub const fn controls_enabled(&self) -> bool {
        self.available
    }

    pub fn connected(&self) -> bool {
        self.state == "completed" && self.network_id.is_some() && self.authoritative
    }

    pub const fn credential_saved(&self) -> bool {
        self.credential_saved
    }

    pub const fn connect_attempts_remaining(&self) -> u8 {
        MAX_WIFI_CONNECT_ATTEMPTS.saturating_sub(self.connect_attempts)
    }

    pub fn status_label(&self) -> &str {
        self.last_error.as_deref().unwrap_or(&self.state)
    }

    pub fn networks(&self) -> &[WifiScanNetwork] {
        &self.networks
    }

    pub const fn credential_entry(&self) -> bool {
        self.credential_entry
    }

    pub fn selected_network(&self) -> Option<&WifiScanNetwork> {
        self.selected_network
            .and_then(|index| self.networks.get(index))
    }

    pub fn masked_passphrase(&self) -> String {
        "*".repeat(self.passphrase.length)
    }

    pub fn passphrase_ready(&self) -> bool {
        (MIN_WIFI_PASSPHRASE_BYTES..=MAX_WIFI_PASSPHRASE_BYTES).contains(&self.passphrase.length)
    }

    pub fn refresh(&mut self, socket_path: &Path) -> bool {
        match request_wifi_broker(socket_path, WifiBrokerOperation::Status) {
            Ok(status) => {
                self.available = true;
                self.state = status.state;
                self.network_id = status.network_id;
                self.authoritative = status.authoritative;
                self.credential_saved = status.credential_saved;
                self.last_error = None;
                true
            }
            Err(error) => {
                self.available = false;
                self.state = "unavailable".to_owned();
                self.network_id = None;
                self.authoritative = false;
                self.credential_saved = false;
                self.last_error = Some(error.to_string());
                false
            }
        }
    }

    pub fn set_connected(&mut self, socket_path: &Path, connected: bool) -> bool {
        if !self.available {
            return false;
        }
        let operation = if connected {
            WifiBrokerOperation::Reconnect
        } else {
            WifiBrokerOperation::Disconnect
        };
        match request_wifi_broker(socket_path, operation) {
            Ok(status) => {
                self.state = status.state;
                self.network_id = status.network_id;
                self.authoritative = status.authoritative;
                self.credential_saved = status.credential_saved;
                self.last_error = None;
                true
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
                false
            }
        }
    }

    pub fn scan(&mut self, socket_path: &Path) -> bool {
        if !self.available {
            return false;
        }
        match request_wifi_scan(socket_path) {
            Ok(networks) => {
                self.networks = networks;
                self.selected_network = None;
                self.credential_entry = false;
                self.passphrase.clear();
                self.connect_attempts = 0;
                self.last_error = None;
                true
            }
            Err(error) => {
                self.networks.clear();
                self.last_error = Some(error.to_string());
                false
            }
        }
    }

    pub fn begin_credential_entry(&mut self, index: usize) -> bool {
        let Some(network) = self.networks.get(index) else {
            return false;
        };
        if !matches!(
            network.security,
            WifiScanSecurity::Wpa2Personal | WifiScanSecurity::Wpa3Personal
        ) {
            self.last_error = Some("unsupported-security".to_owned());
            return false;
        }
        self.passphrase.clear();
        self.connect_attempts = 0;
        self.selected_network = Some(index);
        self.credential_entry = true;
        self.last_error = None;
        true
    }

    pub fn input_passphrase_character(&mut self, character: char) -> bool {
        self.credential_entry && self.passphrase.push(character)
    }

    pub fn remove_passphrase_character(&mut self) -> bool {
        self.credential_entry && self.passphrase.pop()
    }

    pub fn cancel_credential_entry(&mut self) -> bool {
        if !self.credential_entry {
            return false;
        }
        self.passphrase.clear();
        self.credential_entry = false;
        self.selected_network = None;
        self.connect_attempts = 0;
        true
    }

    pub fn connect_selected(&mut self, socket_path: &Path) -> bool {
        if !self.available || !self.credential_entry || !self.passphrase_ready() {
            self.last_error = Some("invalid-passphrase".to_owned());
            return false;
        }
        let Some(network) = self.selected_network().cloned() else {
            self.last_error = Some("missing-network".to_owned());
            return false;
        };
        let passphrase = self
            .passphrase
            .with_bytes(WifiPassphrase::from_bytes)
            .map_err(|error| error.to_string());
        let result = match passphrase {
            Ok(passphrase) => {
                request_wifi_connect(socket_path, network.security, &network.ssid, &passphrase)
            }
            Err(error) => {
                self.last_error = Some(error);
                return false;
            }
        };
        self.passphrase.clear();
        match result {
            Ok(status) => {
                self.state = status.state;
                self.network_id = status.network_id;
                self.authoritative = status.authoritative;
                self.credential_saved = status.credential_saved;
                self.credential_entry = false;
                self.selected_network = None;
                self.connect_attempts = 0;
                self.last_error = None;
                true
            }
            Err(_) => {
                self.connect_attempts = self.connect_attempts.saturating_add(1);
                let remaining = self.connect_attempts_remaining();
                if remaining == 0 {
                    self.credential_entry = false;
                    self.selected_network = None;
                    self.last_error = Some("connection-retry-limit".to_owned());
                } else {
                    self.last_error = Some(format!("connection-failed-retry-{remaining}"));
                }
                false
            }
        }
    }

    pub fn forget_saved(&mut self, socket_path: &Path) -> bool {
        if !self.available || !self.credential_saved {
            return false;
        }
        match request_wifi_broker(socket_path, WifiBrokerOperation::Forget) {
            Ok(status) => {
                self.state = status.state;
                self.network_id = status.network_id;
                self.authoritative = status.authoritative;
                self.credential_saved = status.credential_saved;
                self.passphrase.clear();
                self.credential_entry = false;
                self.selected_network = None;
                self.connect_attempts = 0;
                self.last_error = None;
                true
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
                false
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AquaTheme {
    #[default]
    LightWhite,
    Softtouch,
    Deepside,
    Nightmare,
}

impl AquaTheme {
    pub const ALL: [Self; 4] = [
        Self::LightWhite,
        Self::Softtouch,
        Self::Deepside,
        Self::Nightmare,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::LightWhite => "LightWhite",
            Self::Softtouch => "Softtouch",
            Self::Deepside => "Deepside",
            Self::Nightmare => "Nightmare",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.id() == value)
    }
}

pub const LAUNCHER_STATUS: &str = "interactive-launcher-model";
pub const LAUNCHER_DESIGN_ERA: &str = "bright-aqua-desktop";
pub const LAUNCHER_MATERIAL: &str = "aqua-light-surface";
pub const LAUNCHER_PANEL_X: u32 = 90;
pub const LAUNCHER_PANEL_Y: u32 = 70;
pub const LAUNCHER_PANEL_WIDTH: u32 = 620;
pub const LAUNCHER_PANEL_HEIGHT: u32 = 460;
pub const FILES_VISIBLE_ROWS: usize = 4;
pub const FILES_TEXT_PREVIEW_LIMIT: u64 = 4096;
pub const FILES_PREVIEW_VISIBLE_LINES: usize = 6;
const FILES_SCROLLBAR_TRAILING_INSET: u32 = 12;
const FILES_SCROLLBAR_WIDTH: u32 = 5;
const FILES_SCROLLBAR_MIN_THUMB_HEIGHT: u32 = 24;
const FILES_LIST_SCROLLBAR_Y: u32 = 124;
const FILES_LIST_SCROLLBAR_HEIGHT: u32 = 248;
const FILES_PREVIEW_SCROLLBAR_Y: u32 = 188;
const FILES_PREVIEW_SCROLLBAR_HEIGHT: u32 = 136;
pub const NOTIFICATION_DEFAULT_TIMEOUT_MS: u64 = 30_000;
pub const NOTIFICATION_QUEUE_LIMIT: usize = 8;
pub const SYSTEM_OVERVIEW_REFRESH_MS: u64 = 60_000;
pub const DESKTOP_ICON_X: u32 = 24;
pub const DESKTOP_ICON_Y: u32 = 60;
pub const DESKTOP_ICON_WIDTH: u32 = 104;
pub const DESKTOP_ICON_ROW_HEIGHT: u32 = 104;
pub const DESKTOP_ICON_LAYER_WIDTH: u32 = 232;
pub const DESKTOP_ICON_LAYER_HEIGHT: u32 = DESKTOP_ICON_ROW_HEIGHT * DESKTOP_ICONS.len() as u32;
pub const DESKTOP_ICON_DOUBLE_CLICK_MS: u64 = 500;
pub const DESKTOP_CONTEXT_MENU_X: u32 = DESKTOP_ICON_X + 108;
pub const DESKTOP_CONTEXT_MENU_WIDTH: u32 = 120;
pub const DESKTOP_CONTEXT_MENU_ROW_HEIGHT: u32 = 36;
pub const TRASH_ENTRY_LIMIT: usize = 256;
pub const DOCK_ITEM_COUNT: usize = 3;
pub const WORKSPACE_COUNT: usize = 3;
pub const MOTION_DURATION_IMMEDIATE_MS: u64 = 0;
pub const MOTION_DURATION_FEEDBACK_MS: u64 = 90;
pub const MOTION_DURATION_SHORT_MS: u64 = 140;
pub const MOTION_DURATION_STANDARD_MS: u64 = 200;
pub const MOTION_DURATION_SPATIAL_MS: u64 = 280;
pub const SETTINGS_SIDEBAR_NAVIGATION: SidebarNavigation<'static> = SidebarNavigation::new(
    Rect {
        x: 2,
        y: 60,
        width: 188,
        height: 418,
    },
    "Settings categories",
    Rect {
        x: 12,
        y: 92,
        width: 166,
        height: 42,
    },
    50,
);
pub const FILES_SIDEBAR_NAVIGATION: SidebarNavigation<'static> = SidebarNavigation::new(
    Rect {
        x: 2,
        y: 108,
        width: 170,
        height: 370,
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
pub const fn files_toolbar(width: u32) -> Toolbar<'static> {
    Toolbar::new(
        Rect {
            x: 2,
            y: 50,
            width: width.saturating_sub(4),
            height: 58,
        },
        "File navigation",
    )
    .with_spacing(16, 14, 8)
}

pub const fn files_back_button() -> IconButton<'static> {
    IconButton::new(
        files_toolbar(640).leading_item_rect(0, 28, 28),
        "Back",
        IconButtonGlyph::Back,
    )
}

pub const fn files_forward_button() -> IconButton<'static> {
    IconButton::new(
        files_toolbar(640).leading_item_rect(1, 28, 28),
        "Forward",
        IconButtonGlyph::Forward,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionEasing {
    Standard,
    Enter,
    Exit,
}

impl MotionEasing {
    pub const fn control_points(self) -> [f32; 4] {
        match self {
            Self::Standard => [0.2, 0.0, 0.0, 1.0],
            Self::Enter => [0.0, 0.0, 0.0, 1.0],
            Self::Exit => [0.3, 0.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticMotion {
    Feedback,
    Panel,
    Menu,
    Window,
    Workspace,
    Notification,
    Progress,
    Attention,
}

impl SemanticMotion {
    pub const fn duration_ms(self) -> u64 {
        match self {
            Self::Feedback => MOTION_DURATION_FEEDBACK_MS,
            Self::Menu => MOTION_DURATION_SHORT_MS,
            Self::Panel | Self::Window | Self::Notification | Self::Progress => {
                MOTION_DURATION_STANDARD_MS
            }
            Self::Workspace | Self::Attention => MOTION_DURATION_SPATIAL_MS,
        }
    }

    pub const fn is_spatial(self) -> bool {
        matches!(
            self,
            Self::Panel
                | Self::Menu
                | Self::Window
                | Self::Workspace
                | Self::Notification
                | Self::Attention
        )
    }

    pub const fn repeats(self) -> bool {
        matches!(self, Self::Attention)
    }

    pub const fn repeats_allowed(self, reduced_motion: bool, visible: bool) -> bool {
        self.repeats() && !reduced_motion && visible
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionSample {
    pub value: f32,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionValue {
    rendered: f32,
    start: f32,
    target: f32,
    started_at_ms: u64,
    duration_ms: u64,
    easing: MotionEasing,
    active: bool,
}

impl MotionValue {
    pub const fn new(value: f32) -> Self {
        Self {
            rendered: value,
            start: value,
            target: value,
            started_at_ms: 0,
            duration_ms: 0,
            easing: MotionEasing::Standard,
            active: false,
        }
    }

    pub fn rendered(&self) -> f32 {
        self.rendered
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn settle(&mut self, value: f32) -> MotionSample {
        let value = value.clamp(0.0, 1.0);
        self.rendered = value;
        self.start = value;
        self.target = value;
        self.duration_ms = 0;
        self.active = false;
        MotionSample {
            value,
            active: false,
        }
    }

    pub fn retarget(
        &mut self,
        now_ms: u64,
        target: f32,
        role: SemanticMotion,
        reduced_motion: bool,
    ) -> MotionSample {
        let current = self.sample(now_ms).value;
        self.start = current;
        self.rendered = current;
        self.target = target.clamp(0.0, 1.0);
        self.started_at_ms = now_ms;
        self.duration_ms = if reduced_motion && role.is_spatial() {
            MOTION_DURATION_IMMEDIATE_MS
        } else {
            role.duration_ms()
        };
        self.easing = if self.target >= current {
            MotionEasing::Enter
        } else {
            MotionEasing::Exit
        };
        self.active = self.duration_ms > 0 && (self.target - current).abs() > f32::EPSILON;
        if !self.active {
            self.rendered = self.target;
            self.start = self.target;
        }
        MotionSample {
            value: self.rendered,
            active: self.active,
        }
    }

    pub fn sample(&mut self, now_ms: u64) -> MotionSample {
        if !self.active {
            return MotionSample {
                value: self.rendered,
                active: false,
            };
        }
        let elapsed_ms = now_ms.saturating_sub(self.started_at_ms);
        if elapsed_ms >= self.duration_ms {
            self.rendered = self.target;
            self.start = self.target;
            self.active = false;
        } else {
            let progress = elapsed_ms as f32 / self.duration_ms as f32;
            let eased = cubic_bezier_progress(progress, self.easing.control_points());
            self.rendered = self.start + (self.target - self.start) * eased;
        }
        MotionSample {
            value: self.rendered,
            active: self.active,
        }
    }
}

fn cubic_bezier_progress(progress: f32, points: [f32; 4]) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let [x1, y1, x2, y2] = points;
    let component = |time: f32, first: f32, second: f32| {
        let inverse = 1.0 - time;
        3.0 * inverse * inverse * time * first
            + 3.0 * inverse * time * time * second
            + time * time * time
    };
    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    for _ in 0..16 {
        let time = (low + high) * 0.5;
        if component(time, x1, x2) < progress {
            low = time;
        } else {
            high = time;
        }
    }
    component((low + high) * 0.5, y1, y2).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMotionSurface {
    Launcher,
    SessionMenu,
    Notification,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellSurfaceMotionSample {
    pub opacity: f32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub active: bool,
}

impl ShellSurfaceMotionSample {
    const fn hidden() -> Self {
        Self {
            opacity: 0.0,
            offset_x: 0,
            offset_y: 0,
            active: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellMotionFrame {
    pub launcher: ShellSurfaceMotionSample,
    pub session_menu: ShellSurfaceMotionSample,
    pub notification: ShellSurfaceMotionSample,
}

impl ShellMotionFrame {
    pub fn is_active(self) -> bool {
        self.launcher.active || self.session_menu.active || self.notification.active
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShellMotionController {
    reduced_motion: bool,
    launcher: MotionValue,
    session_menu: MotionValue,
    notification: MotionValue,
}

impl Default for ShellMotionController {
    fn default() -> Self {
        Self {
            reduced_motion: false,
            launcher: MotionValue::new(0.0),
            session_menu: MotionValue::new(0.0),
            notification: MotionValue::new(0.0),
        }
    }
}

impl ShellMotionController {
    pub fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    pub fn is_active(&self) -> bool {
        self.launcher.is_active() || self.session_menu.is_active() || self.notification.is_active()
    }

    pub fn set_reduced_motion(&mut self, now_ms: u64, reduced_motion: bool) -> bool {
        if self.reduced_motion == reduced_motion {
            return false;
        }
        self.reduced_motion = reduced_motion;
        if reduced_motion {
            for (motion, role) in [
                (&mut self.launcher, SemanticMotion::Panel),
                (&mut self.session_menu, SemanticMotion::Menu),
                (&mut self.notification, SemanticMotion::Notification),
            ] {
                motion.retarget(now_ms, motion.target(), role, true);
            }
        }
        true
    }

    pub fn set_visible(&mut self, surface: ShellMotionSurface, visible: bool, now_ms: u64) {
        let target = if visible { 1.0 } else { 0.0 };
        let (motion, role) = match surface {
            ShellMotionSurface::Launcher => (&mut self.launcher, SemanticMotion::Panel),
            ShellMotionSurface::SessionMenu => (&mut self.session_menu, SemanticMotion::Menu),
            ShellMotionSurface::Notification => {
                (&mut self.notification, SemanticMotion::Notification)
            }
        };
        if (motion.target() - target).abs() > f32::EPSILON {
            motion.retarget(now_ms, target, role, self.reduced_motion);
        }
    }

    pub fn settle_visible(&mut self, surface: ShellMotionSurface, visible: bool) {
        let value = if visible { 1.0 } else { 0.0 };
        match surface {
            ShellMotionSurface::Launcher => self.launcher.settle(value),
            ShellMotionSurface::SessionMenu => self.session_menu.settle(value),
            ShellMotionSurface::Notification => self.notification.settle(value),
        };
    }

    pub fn sample(&mut self, now_ms: u64) -> ShellMotionFrame {
        let reduced_motion = self.reduced_motion;
        ShellMotionFrame {
            launcher: surface_motion_sample(self.launcher.sample(now_ms), 0, 18, reduced_motion),
            session_menu: surface_motion_sample(
                self.session_menu.sample(now_ms),
                0,
                12,
                reduced_motion,
            ),
            notification: surface_motion_sample(
                self.notification.sample(now_ms),
                20,
                0,
                reduced_motion,
            ),
        }
    }
}

fn surface_motion_sample(
    sample: MotionSample,
    travel_x: i32,
    travel_y: i32,
    reduced_motion: bool,
) -> ShellSurfaceMotionSample {
    if sample.value <= 0.0 && !sample.active {
        return ShellSurfaceMotionSample::hidden();
    }
    let travel = if reduced_motion {
        0.0
    } else {
        1.0 - sample.value
    };
    ShellSurfaceMotionSample {
        opacity: sample.value,
        offset_x: (travel_x as f32 * travel).round() as i32,
        offset_y: (travel_y as f32 * travel).round() as i32,
        active: sample.active,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalView {
    pub lines: Vec<String>,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub rows: u16,
    pub cols: u16,
}

impl TerminalView {
    pub fn empty(rows: u16, cols: u16) -> Self {
        Self {
            lines: vec![String::new(); rows as usize],
            cursor_row: 0,
            cursor_col: 0,
            rows,
            cols,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockItem {
    Files,
    Settings,
    Trash,
}

impl DockItem {
    pub const ALL: [Self; DOCK_ITEM_COUNT] = [Self::Files, Self::Settings, Self::Trash];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Files => "files",
            Self::Settings => "settings",
            Self::Trash => "trash",
        }
    }

    pub const fn accessibility_name(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Settings => "Settings",
            Self::Trash => "Trash",
        }
    }

    pub const fn launch_request(self) -> Option<LaunchRequest> {
        match self {
            Self::Files | Self::Trash => Some(LaunchRequest {
                app_id: "files",
                command: "/usr/bin/aqua-files",
                target: None,
            }),
            Self::Settings => Some(LaunchRequest {
                app_id: "settings",
                command: "/usr/bin/aqua-settings",
                target: None,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockState {
    pub applications_open: bool,
    pub search_open: bool,
    pub files_running: bool,
    pub settings_running: bool,
    pub active_workspace: usize,
}

impl DockState {
    pub fn item_running(&self, item: DockItem) -> bool {
        match item {
            DockItem::Files | DockItem::Trash => self.files_running,
            DockItem::Settings => self.settings_running,
        }
    }

    pub fn workspace_active(&self, index: usize) -> bool {
        index < WORKSPACE_COUNT && index == self.active_workspace
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomShellTarget {
    Applications,
    Search,
    Application(DockItem),
    Workspace(usize),
}

pub const fn running_app_dock(width: u32, height: u32) -> RunningAppDock<'static> {
    let item_width = 72_u32;
    let content_width = item_width.saturating_mul(DOCK_ITEM_COUNT as u32);
    RunningAppDock::new(
        Rect {
            x: width.saturating_sub(content_width) / 2,
            y: 0,
            width: content_width,
            height,
        },
        "Running applications",
        DOCK_ITEM_COUNT,
    )
}

pub const fn workspace_switcher(
    width: u32,
    height: u32,
    active_workspace: usize,
) -> WorkspaceSwitcher<'static> {
    let item_width = 60_u32;
    let group_width = item_width.saturating_mul(WORKSPACE_COUNT as u32);
    WorkspaceSwitcher::new(
        Rect {
            x: width.saturating_sub(group_width),
            y: 0,
            width: group_width,
            height,
        },
        "Workspaces",
        WORKSPACE_COUNT,
        active_workspace,
    )
}

pub const fn workspace_keyboard_target(
    active_workspace: usize,
    key: WorkspaceNavigationKey,
) -> Option<usize> {
    workspace_switcher(WORKSPACE_COUNT as u32 * 60, 72, active_workspace).keyboard_target(key)
}

pub fn dock_pointer_target(
    local_x: u32,
    local_y: u32,
    width: u32,
    height: u32,
) -> Option<BottomShellTarget> {
    if local_x >= width || local_y >= height || width < 640 || height < 48 {
        return None;
    }

    if local_x < 64 {
        return Some(BottomShellTarget::Applications);
    }
    if (68..132).contains(&local_x) {
        return Some(BottomShellTarget::Search);
    }

    let running_dock = running_app_dock(width, height);
    if let Some(index) = running_dock.item_at(local_x, local_y) {
        return DockItem::ALL
            .get(index)
            .copied()
            .map(BottomShellTarget::Application);
    }

    workspace_switcher(width, height, 0)
        .item_at(local_x, local_y)
        .map(BottomShellTarget::Workspace)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopIcon {
    pub id: &'static str,
    pub label: &'static str,
    pub glyph: &'static str,
    pub launch: Option<LaunchRequest>,
}

pub const DESKTOP_ICONS: [DesktopIcon; 3] = [
    DesktopIcon {
        id: "files",
        label: "Files",
        glyph: "DIR",
        launch: Some(LaunchRequest {
            app_id: "files",
            command: "/usr/bin/aqua-files",
            target: None,
        }),
    },
    DesktopIcon {
        id: "settings",
        label: "Settings",
        glyph: "SET",
        launch: Some(LaunchRequest {
            app_id: "settings",
            command: "/usr/bin/aqua-settings",
            target: None,
        }),
    },
    DesktopIcon {
        id: "trash",
        label: "Trash",
        glyph: "BIN",
        launch: Some(LaunchRequest {
            app_id: "files",
            command: "/usr/bin/aqua-files",
            target: None,
        }),
    },
];

pub const fn desktop_grid_cell(
    index: usize,
    label: &str,
    selected: bool,
    origin_x: u32,
    origin_y: u32,
) -> GridCell<'_> {
    GridCell::new(
        Rect {
            x: origin_x,
            y: origin_y.saturating_add((index as u32).saturating_mul(DESKTOP_ICON_ROW_HEIGHT)),
            width: DESKTOP_ICON_WIDTH,
            height: DESKTOP_ICON_ROW_HEIGHT,
        },
        label,
        GridCellLayout::IconAbove,
    )
    .with_spacing(64, 8, 5, 0)
    .with_idle_surface(false)
    .with_state(if selected {
        ComponentState::Selected
    } else {
        ComponentState::Idle
    })
}

pub fn properties_launch_request(icon_id: &'static str) -> Option<LaunchRequest> {
    DESKTOP_ICONS
        .iter()
        .any(|icon| icon.id == icon_id)
        .then_some(LaunchRequest {
            app_id: "properties",
            command: "/usr/bin/aqua-properties",
            target: Some(icon_id),
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopPropertiesAction {
    RefreshContents,
    VerifyApplication,
}

impl DesktopPropertiesAction {
    pub const fn label(self) -> &'static str {
        match self {
            Self::RefreshContents => "Refresh Contents",
            Self::VerifyApplication => "Verify Application",
        }
    }

    pub const fn log_name(self) -> &'static str {
        match self {
            Self::RefreshContents => "refresh-contents",
            Self::VerifyApplication => "verify-application",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopPropertiesModel {
    pub icon_id: &'static str,
    pub title: String,
    pub name: &'static str,
    pub kind: &'static str,
    pub location: String,
    pub status: &'static str,
    pub item_count: Option<usize>,
    pub enumeration_capped: bool,
    pub refresh_generation: u32,
}

impl DesktopPropertiesModel {
    pub fn load(icon_id: &'static str, home_root: &Path, system_root: &Path) -> io::Result<Self> {
        let (name, kind, path) = match icon_id {
            "files" => ("Files", "Folder", home_root.to_path_buf()),
            "settings" => (
                "System Settings",
                "Application",
                system_root.join("usr/bin/aqua-settings"),
            ),
            "trash" => ("Trash", "Folder", home_root.join("Trash")),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unsupported desktop properties target",
                ));
            }
        };
        let metadata = fs::symlink_metadata(&path).ok();
        let status = match metadata.as_ref() {
            Some(metadata) if metadata.file_type().is_symlink() => "Symbolic link",
            Some(_) => "Available",
            None => "Not found",
        };
        let (item_count, enumeration_capped) = if metadata
            .as_ref()
            .is_some_and(|metadata| metadata.file_type().is_dir())
        {
            match fs::read_dir(&path) {
                Ok(entries) => {
                    let count = entries.take(TRASH_ENTRY_LIMIT + 1).count();
                    (
                        Some(count.min(TRASH_ENTRY_LIMIT)),
                        count > TRASH_ENTRY_LIMIT,
                    )
                }
                Err(_) => (None, false),
            }
        } else {
            (None, false)
        };

        Ok(Self {
            icon_id,
            title: format!("{name} Properties"),
            name,
            kind,
            location: path.display().to_string(),
            status,
            item_count,
            enumeration_capped,
            refresh_generation: 0,
        })
    }

    pub fn primary_action(&self) -> DesktopPropertiesAction {
        match self.icon_id {
            "settings" => DesktopPropertiesAction::VerifyApplication,
            _ => DesktopPropertiesAction::RefreshContents,
        }
    }

    pub fn details_section_group(&self, width: u32, height: u32) -> SectionGroup<'_> {
        SectionGroup::new(
            Rect {
                x: 24,
                y: 184,
                width: width.saturating_sub(48),
                height: height.saturating_sub(208),
            },
            &self.title,
            2,
        )
        .with_structure(0, 34, 16, 8, 18, 4)
    }

    pub fn details_metadata_row<'a>(
        &self,
        width: u32,
        height: u32,
        index: usize,
        label: &'a str,
        value: &'a str,
    ) -> MetadataRow<'a> {
        MetadataRow::new(
            self.details_section_group(width, height).row_rect(index),
            label,
            value,
        )
        .with_columns(80, 8)
    }

    pub fn refresh(
        &mut self,
        home_root: &Path,
        system_root: &Path,
    ) -> io::Result<DesktopPropertiesAction> {
        let generation = self.refresh_generation.saturating_add(1);
        let action = self.primary_action();
        let mut refreshed = Self::load(self.icon_id, home_root, system_root)?;
        refreshed.refresh_generation = generation;
        *self = refreshed;
        Ok(action)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopPointerButton {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopContextAction {
    MenuOpened,
    Properties(&'static str),
    TrashEmptyConfirmationRequested,
    TrashEmptyConfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopContextMenuKey {
    Dismiss,
    Navigate(MenuNavigationKey),
    Activate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopIconUpdate {
    pub redraw_requested: bool,
    pub launch_request: Option<LaunchRequest>,
    pub context_action: Option<DesktopContextAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesktopIconState {
    selected: Option<usize>,
    context_menu: Option<usize>,
    context_menu_selected_row: usize,
    trash_empty_confirmation: bool,
    last_primary_click: Option<(usize, u64)>,
}

impl DesktopIconState {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn context_menu(&self) -> Option<usize> {
        self.context_menu
    }

    pub fn context_menu_selected_row(&self) -> Option<usize> {
        self.context_menu.map(|_| self.context_menu_selected_row)
    }

    pub fn trash_empty_confirmation(&self) -> bool {
        self.trash_empty_confirmation
    }

    pub fn trash_confirmation_dialog(&self, rect: Rect) -> Option<ConfirmationDialog<'static>> {
        self.trash_empty_confirmation.then(|| {
            ConfirmationDialog::new(
                rect,
                "Empty Trash confirmation",
                ("CONFIRM EMPTY", ""),
                ConfirmationPresentation::Inline,
                ConfirmationSeverity::Destructive,
                ConfirmationRequirement::RepeatActivation,
                ConfirmationState::Armed,
            )
        })
    }

    fn context_menu_row(&self, x: u32, y: u32) -> Option<(usize, usize)> {
        let icon_index = self.context_menu?;
        let menu = desktop_context_menu_with_selection(icon_index, self.context_menu_selected_row)?
            .translated(DESKTOP_ICON_X, DESKTOP_ICON_Y);
        menu.item_at(x, y).map(|row| (icon_index, row))
    }

    fn activate_context_menu_row(&mut self, icon_index: usize, row: usize) -> DesktopIconUpdate {
        self.context_menu_selected_row = row;
        self.last_primary_click = None;
        if row == 0 {
            self.context_menu = None;
            self.context_menu_selected_row = 0;
            self.trash_empty_confirmation = false;
            return DesktopIconUpdate {
                redraw_requested: true,
                launch_request: DESKTOP_ICONS[icon_index].launch.clone(),
                context_action: None,
            };
        }
        if DESKTOP_ICONS[icon_index].id == "trash" {
            if self.trash_empty_confirmation {
                self.context_menu = None;
                self.context_menu_selected_row = 0;
                self.trash_empty_confirmation = false;
                return DesktopIconUpdate {
                    redraw_requested: true,
                    launch_request: None,
                    context_action: Some(DesktopContextAction::TrashEmptyConfirmed),
                };
            }
            self.trash_empty_confirmation = true;
            return DesktopIconUpdate {
                redraw_requested: true,
                launch_request: None,
                context_action: Some(DesktopContextAction::TrashEmptyConfirmationRequested),
            };
        }
        self.context_menu = None;
        self.context_menu_selected_row = 0;
        self.trash_empty_confirmation = false;
        DesktopIconUpdate {
            redraw_requested: true,
            launch_request: None,
            context_action: Some(DesktopContextAction::Properties(
                DESKTOP_ICONS[icon_index].id,
            )),
        }
    }

    pub fn handle_context_menu_key(&mut self, key: DesktopContextMenuKey) -> DesktopIconUpdate {
        let Some(icon_index) = self.context_menu else {
            return DesktopIconUpdate {
                redraw_requested: false,
                launch_request: None,
                context_action: None,
            };
        };
        match key {
            DesktopContextMenuKey::Dismiss => {
                self.context_menu = None;
                self.context_menu_selected_row = 0;
                self.trash_empty_confirmation = false;
                self.last_primary_click = None;
                DesktopIconUpdate {
                    redraw_requested: true,
                    launch_request: None,
                    context_action: None,
                }
            }
            DesktopContextMenuKey::Navigate(key) => {
                let Some(row) =
                    desktop_context_menu_with_selection(icon_index, self.context_menu_selected_row)
                        .and_then(|menu| menu.keyboard_target(key))
                else {
                    return DesktopIconUpdate {
                        redraw_requested: false,
                        launch_request: None,
                        context_action: None,
                    };
                };
                if row == self.context_menu_selected_row {
                    return DesktopIconUpdate {
                        redraw_requested: false,
                        launch_request: None,
                        context_action: None,
                    };
                }
                self.context_menu_selected_row = row;
                self.trash_empty_confirmation = false;
                DesktopIconUpdate {
                    redraw_requested: true,
                    launch_request: None,
                    context_action: None,
                }
            }
            DesktopContextMenuKey::Activate => {
                self.activate_context_menu_row(icon_index, self.context_menu_selected_row)
            }
        }
    }

    pub fn pointer_target(x: u32, y: u32) -> Option<usize> {
        DESKTOP_ICONS.iter().enumerate().find_map(|(index, icon)| {
            desktop_grid_cell(index, icon.label, false, DESKTOP_ICON_X, DESKTOP_ICON_Y)
                .pointer_hit(x, y)
                .then_some(index)
        })
    }

    pub fn pointer_press(
        &mut self,
        x: u32,
        y: u32,
        button: DesktopPointerButton,
        now_ms: u64,
    ) -> DesktopIconUpdate {
        if button == DesktopPointerButton::Primary {
            if let Some((icon_index, row)) = self.context_menu_row(x, y) {
                return self.activate_context_menu_row(icon_index, row);
            }
        }
        let target = Self::pointer_target(x, y);
        match (button, target) {
            (DesktopPointerButton::Primary, Some(index)) => {
                let activate = self
                    .last_primary_click
                    .is_some_and(|(previous, previous_ms)| {
                        previous == index
                            && now_ms.saturating_sub(previous_ms) <= DESKTOP_ICON_DOUBLE_CLICK_MS
                    });
                self.selected = Some(index);
                self.context_menu = None;
                self.context_menu_selected_row = 0;
                self.trash_empty_confirmation = false;
                self.last_primary_click = (!activate).then_some((index, now_ms));
                DesktopIconUpdate {
                    redraw_requested: true,
                    launch_request: activate
                        .then(|| DESKTOP_ICONS[index].launch.clone())
                        .flatten(),
                    context_action: None,
                }
            }
            (DesktopPointerButton::Secondary, Some(index)) => {
                self.selected = Some(index);
                self.context_menu = Some(index);
                self.context_menu_selected_row = 0;
                self.trash_empty_confirmation = false;
                self.last_primary_click = None;
                DesktopIconUpdate {
                    redraw_requested: true,
                    launch_request: None,
                    context_action: Some(DesktopContextAction::MenuOpened),
                }
            }
            (_, None) => {
                let had_selection = self.selected.take().is_some();
                let had_context_menu = self.context_menu.take().is_some();
                self.context_menu_selected_row = 0;
                self.trash_empty_confirmation = false;
                let redraw_requested = had_selection || had_context_menu;
                self.last_primary_click = None;
                DesktopIconUpdate {
                    redraw_requested,
                    launch_request: None,
                    context_action: None,
                }
            }
        }
    }
}

pub fn desktop_context_menu(icon_index: usize) -> Option<Menu<'static>> {
    desktop_context_menu_with_selection(icon_index, 0)
}

pub fn desktop_context_menu_with_selection(
    icon_index: usize,
    selected_row: usize,
) -> Option<Menu<'static>> {
    if icon_index >= DESKTOP_ICONS.len() || selected_row >= 2 {
        return None;
    }
    let local_y = (icon_index as u32 * DESKTOP_ICON_ROW_HEIGHT + 32)
        .min(DESKTOP_ICON_LAYER_HEIGHT.saturating_sub(76));
    Some(Menu::new(
        Rect {
            x: DESKTOP_CONTEXT_MENU_X.saturating_sub(DESKTOP_ICON_X),
            y: local_y,
            width: DESKTOP_CONTEXT_MENU_WIDTH,
            height: DESKTOP_CONTEXT_MENU_ROW_HEIGHT * 2,
        },
        "Desktop icon actions",
        2,
        selected_row,
        0,
        DESKTOP_CONTEXT_MENU_ROW_HEIGHT,
        0,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashEntryKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashEntry {
    pub name: String,
    pub kind: TrashEntryKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashModel {
    root: PathBuf,
    entries: Vec<TrashEntry>,
}

impl TrashModel {
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into();
        match fs::symlink_metadata(&root) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "trash root must not be a symlink",
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "trash root must be a directory",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir_all(&root)?,
            Err(error) => return Err(error),
        }
        let mut model = Self {
            root,
            entries: Vec::new(),
        };
        model.refresh()?;
        Ok(model)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn entries(&self) -> &[TrashEntry] {
        &self.entries
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        let paths = self.collect_paths()?;
        self.entries = paths
            .iter()
            .map(|path| {
                let metadata = fs::symlink_metadata(path)?;
                let kind = if metadata.file_type().is_symlink() {
                    TrashEntryKind::Symlink
                } else if metadata.is_dir() {
                    TrashEntryKind::Directory
                } else {
                    TrashEntryKind::File
                };
                Ok(TrashEntry {
                    name: path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Invalid name")
                        .chars()
                        .take(80)
                        .collect(),
                    kind,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        self.entries
            .sort_by(|left, right| left.name.cmp(&right.name));
        Ok(())
    }

    pub fn empty(&mut self) -> io::Result<usize> {
        let paths = self.collect_paths()?;
        for path in &paths {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        self.entries.clear();
        Ok(paths.len())
    }

    fn collect_paths(&self) -> io::Result<Vec<PathBuf>> {
        let metadata = fs::symlink_metadata(&self.root)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trash root changed or is not a directory",
            ));
        }
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            if paths.len() == TRASH_ENTRY_LIMIT {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "trash entry limit exceeded",
                ));
            }
            paths.push(entry?.path());
        }
        Ok(paths)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemOverviewModel {
    pub clock_utc: String,
    pub os_name: String,
    pub hostname: String,
    pub kernel: String,
    pub uptime_seconds: u64,
    pub load_average_x100: u32,
    pub memory_total_kib: u64,
    pub memory_available_kib: u64,
}

impl SystemOverviewModel {
    pub fn read(root: &Path, epoch_seconds: u64) -> io::Result<Self> {
        let uptime = fs::read_to_string(root.join("proc/uptime"))?;
        let loadavg = fs::read_to_string(root.join("proc/loadavg"))?;
        let meminfo = fs::read_to_string(root.join("proc/meminfo"))?;
        let kernel = read_bounded_line(&root.join("proc/sys/kernel/osrelease"), 96)?;
        let hostname = read_bounded_line(&root.join("etc/hostname"), 64)?;
        let os_release = fs::read_to_string(root.join("etc/os-release"))?;

        let uptime_seconds = uptime
            .split_whitespace()
            .next()
            .and_then(|value| value.split('.').next())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid /proc/uptime"))?;
        let load_average_x100 = parse_decimal_x100(
            loadavg
                .split_whitespace()
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid loadavg"))?,
        )?;
        let memory_total_kib = parse_meminfo_kib(&meminfo, "MemTotal")?;
        let memory_available_kib = parse_meminfo_kib(&meminfo, "MemAvailable")?;
        let os_name = os_release
            .lines()
            .find_map(|line| line.strip_prefix("PRETTY_NAME="))
            .map(|value| value.trim_matches('"'))
            .filter(|value| !value.is_empty())
            .unwrap_or("Aqua Linux");
        let seconds_today = epoch_seconds % 86_400;

        Ok(Self {
            clock_utc: format!(
                "{:02}:{:02} UTC",
                seconds_today / 3_600,
                seconds_today % 3_600 / 60
            ),
            os_name: bounded_notification_text(os_name, 64),
            hostname: bounded_notification_text(&hostname, 64),
            kernel: bounded_notification_text(&kernel, 96),
            uptime_seconds,
            load_average_x100,
            memory_total_kib,
            memory_available_kib: memory_available_kib.min(memory_total_kib),
        })
    }

    pub fn memory_used_percent(&self) -> u8 {
        if self.memory_total_kib == 0 {
            return 0;
        }
        let used = self
            .memory_total_kib
            .saturating_sub(self.memory_available_kib);
        (used.saturating_mul(100) / self.memory_total_kib).min(100) as u8
    }

    pub fn uptime_label(&self) -> String {
        let days = self.uptime_seconds / 86_400;
        let hours = self.uptime_seconds % 86_400 / 3_600;
        let minutes = self.uptime_seconds % 3_600 / 60;
        if days > 0 {
            format!("{days}d {hours}h")
        } else {
            format!("{hours}h {minutes}m")
        }
    }
}

fn read_bounded_line(path: &Path, limit: usize) -> io::Result<String> {
    let contents = fs::read_to_string(path)?;
    Ok(contents
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(limit)
        .collect())
}

fn parse_decimal_x100(value: &str) -> io::Result<u32> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, "0"));
    let whole = whole
        .parse::<u32>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid decimal"))?;
    let mut digits = fraction.bytes().take(2);
    let tenths = digits
        .next()
        .filter(u8::is_ascii_digit)
        .map_or(0, |digit| u32::from(digit - b'0'));
    let hundredths = digits
        .next()
        .filter(u8::is_ascii_digit)
        .map_or(0, |digit| u32::from(digit - b'0'));
    Ok(whole
        .saturating_mul(100)
        .saturating_add(tenths * 10 + hundredths))
}

fn parse_meminfo_kib(contents: &str, key: &str) -> io::Result<u64> {
    contents
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name == key)
                .then(|| value.split_whitespace().next()?.parse::<u64>().ok())
                .flatten()
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("missing {key}")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioControlStatus {
    Unavailable,
    Starting,
    Degraded,
    Applying,
    Applied,
}

impl AudioControlStatus {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Starting => "starting",
            Self::Degraded => "degraded",
            Self::Applying => "applying",
            Self::Applied => "applied",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioVolumeModel {
    adapter: AudioServiceAdapter,
}

impl AudioVolumeModel {
    pub fn available(&self) -> bool {
        self.adapter.state().health() == AudioServiceHealth::Ready
            && self.adapter.state().default_output().is_some()
    }

    pub const fn volume_percent(&self) -> u8 {
        self.adapter.desired_volume_percent()
    }

    pub const fn muted(&self) -> bool {
        self.adapter.desired_muted()
    }

    pub const fn service_health(&self) -> AudioServiceHealth {
        self.adapter.state().health()
    }

    pub fn backend_applied(&self) -> bool {
        self.adapter.backend_applied()
    }

    pub fn submission_attempts(&self) -> u8 {
        self.adapter.consecutive_submission_failures()
    }

    pub fn submission_retry_exhausted(&self) -> bool {
        self.adapter.submission_retry_exhausted()
    }

    pub fn control_status(&self) -> AudioControlStatus {
        if self.submission_retry_exhausted() {
            return AudioControlStatus::Degraded;
        }
        match self.service_health() {
            AudioServiceHealth::Unavailable => AudioControlStatus::Unavailable,
            AudioServiceHealth::Starting => AudioControlStatus::Starting,
            AudioServiceHealth::Degraded => AudioControlStatus::Degraded,
            AudioServiceHealth::Ready if !self.available() => AudioControlStatus::Unavailable,
            AudioServiceHealth::Ready if self.backend_applied() => AudioControlStatus::Applied,
            AudioServiceHealth::Ready => AudioControlStatus::Applying,
        }
    }

    pub fn controls_enabled(&self) -> bool {
        self.control_status() == AudioControlStatus::Applied
    }

    pub fn authoritative_volume_percent(&self) -> Option<u8> {
        self.available()
            .then(|| self.adapter.state().output_volume_percent())
    }

    pub fn authoritative_muted(&self) -> Option<bool> {
        self.available()
            .then(|| self.adapter.state().output_muted())
    }

    pub fn output_device_name(&self) -> Option<&str> {
        self.adapter
            .state()
            .output_device()
            .map(|device| device.name())
    }

    pub fn reconcile(&mut self, state: AudioAuthoritativeState) -> Result<(), AudioAdapterError> {
        self.adapter.reconcile(state)?;
        Ok(())
    }

    pub fn next_reconciliation_request(
        &mut self,
    ) -> Result<Option<AudioRequest>, AudioAdapterError> {
        self.adapter.next_reconciliation_request()
    }

    pub fn synchronize_backend<B: AudioBackend>(
        &mut self,
        backend: &mut B,
    ) -> Result<AudioBackendDriveOutcome, AudioBackendDriveError<B::Error>> {
        self.adapter.drive_backend_once(backend)
    }

    pub fn set_volume_percent(&mut self, volume_percent: u8) -> bool {
        self.adapter
            .set_desired_volume(volume_percent)
            .unwrap_or(false)
    }

    pub fn set_muted(&mut self, muted: bool) -> bool {
        self.adapter.set_desired_muted(muted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindowModel {
    pub title: &'static str,
    pub categories: [&'static str; 6],
    pub selected_category: usize,
    pub hovered_category: Option<usize>,
    pub reduced_motion: bool,
    pub desktop_icons: bool,
    pub key_repeat: bool,
    pub audio: AudioVolumeModel,
    pub network: NetworkAuthoritativeState,
    pub wifi: WifiSettingsControl,
    pub keyboard_focus: bool,
    pub theme: AquaTheme,
}

impl Default for SettingsWindowModel {
    fn default() -> Self {
        Self {
            title: "System Settings",
            categories: [
                "Appearance",
                "Desktop",
                "Input",
                "Network",
                "Audio",
                "About",
            ],
            selected_category: 0,
            hovered_category: None,
            reduced_motion: false,
            desktop_icons: true,
            key_repeat: true,
            audio: AudioVolumeModel::default(),
            network: NetworkAuthoritativeState::default(),
            wifi: WifiSettingsControl::default(),
            keyboard_focus: false,
            theme: AquaTheme::default(),
        }
    }
}

impl SettingsWindowModel {
    pub fn section_group(&self) -> SectionGroup<'static> {
        let (row_count, row_height, row_gap) = match self.selected_category {
            0 => (2, 48, 40),
            3 => (4, 32, 6),
            4 => (2, 48, 20),
            _ => (1, 48, 0),
        };
        SectionGroup::new(
            Rect {
                x: 202,
                y: 76,
                width: 378,
                height: 202,
            },
            self.categories[self.selected_category],
            row_count,
        )
        .with_structure(34, 0, 16, 16, row_height, row_gap)
        .with_focus(self.keyboard_focus)
    }

    pub fn active_switch(&self) -> Option<SwitchControl<'static>> {
        let (label, checked, row_index) = match self.selected_category {
            0 => ("Reduced motion", self.reduced_motion, 0),
            1 => ("Show desktop icons", self.desktop_icons, 0),
            2 => ("Key repeat", self.key_repeat, 0),
            3 => ("Wi-Fi association", self.wifi.connected(), 0),
            4 => (
                "Mute output",
                self.audio
                    .authoritative_muted()
                    .unwrap_or_else(|| self.audio.muted()),
                1,
            ),
            _ => return None,
        };
        let section = self.section_group();
        let switch_height = section.row_rect(row_index).height.min(36);
        Some(
            SwitchControl::new(
                section.trailing_rect(row_index, 82, switch_height),
                label,
                checked,
            )
            .with_state(
                if (self.selected_category == 3 && !self.wifi.controls_enabled())
                    || (!self.audio.controls_enabled() && self.selected_category == 4)
                {
                    ComponentState::Disabled
                } else if self.keyboard_focus {
                    ComponentState::KeyboardFocus
                } else {
                    ComponentState::Idle
                },
            ),
        )
    }

    pub fn wifi_network_row(&self, index: usize) -> Option<ListRow<'_>> {
        let network = self.wifi.networks().get(index)?;
        let label = std::str::from_utf8(network.ssid.bytes()).unwrap_or("UNKNOWN");
        let supported = matches!(
            network.security,
            WifiScanSecurity::Wpa2Personal | WifiScanSecurity::Wpa3Personal
        );
        Some(
            ListRow::new(
                self.section_group().row_rect(index + 1),
                label,
                ListRowRole::Option,
            )
            .with_slots(0, 148)
            .with_state(if !self.wifi.controls_enabled() || !supported {
                ComponentState::Disabled
            } else {
                ComponentState::Idle
            }),
        )
    }

    pub fn wifi_rescan_button(&self) -> StandardButton<'static> {
        StandardButton::new(
            self.wifi_action_button_rect(false),
            "Rescan",
            StandardButtonVariant::Secondary,
        )
        .with_state(if self.wifi.controls_enabled() {
            ComponentState::Idle
        } else {
            ComponentState::Disabled
        })
    }

    pub fn wifi_forget_button(&self) -> StandardButton<'static> {
        StandardButton::new(
            self.wifi_action_button_rect(true),
            "Forget saved",
            StandardButtonVariant::Destructive,
        )
        .with_state(
            if self.wifi.controls_enabled() && self.wifi.credential_saved() {
                ComponentState::Idle
            } else {
                ComponentState::Disabled
            },
        )
    }

    fn wifi_action_button_rect(&self, trailing: bool) -> Rect {
        let row = self.section_group().row_rect(3);
        let gap = 6;
        let width = row.width.saturating_sub(gap) / 2;
        Rect {
            x: if trailing {
                row.x.saturating_add(width + gap)
            } else {
                row.x
            },
            y: row.y,
            width,
            height: row.height,
        }
    }

    pub fn theme_segmented_control(&self) -> SegmentedControl<'static> {
        let selected_index = AquaTheme::ALL
            .iter()
            .position(|theme| *theme == self.theme)
            .unwrap_or(0);
        SegmentedControl::new(
            self.section_group().row_rect(1),
            "Desktop theme",
            AquaTheme::ALL.len(),
            selected_index,
        )
        .with_gap(6)
    }

    pub fn audio_slider(&self) -> Slider<'static> {
        let row = self.section_group().row_rect(0);
        Slider::new(
            Rect {
                x: row.x + 120,
                y: row.y + 8,
                width: row.width.saturating_sub(136),
                height: 32,
            },
            "Output volume",
            u16::from(
                self.audio
                    .authoritative_volume_percent()
                    .unwrap_or_else(|| self.audio.volume_percent()),
            ),
            0,
            100,
            5,
        )
        .with_state(if !self.audio.controls_enabled() {
            ComponentState::Disabled
        } else if self.keyboard_focus {
            ComponentState::KeyboardFocus
        } else {
            ComponentState::Idle
        })
    }

    pub fn refresh_network_status(
        &mut self,
        class_net: &Path,
        ipv4_route: &Path,
        resolver: &Path,
    ) -> Result<(), NetworkSnapshotError> {
        self.network = read_network_snapshot(class_net, ipv4_route, resolver)?;
        Ok(())
    }

    pub fn refresh_wifi_control(&mut self, socket_path: &Path) -> bool {
        self.wifi.refresh(socket_path)
    }

    pub fn refresh_wifi_networks(&mut self, socket_path: &Path) -> bool {
        self.wifi.scan(socket_path)
    }

    pub fn apply_wifi_control(&mut self, socket_path: &Path, connected: bool) -> bool {
        self.wifi.set_connected(socket_path, connected)
    }

    pub fn apply_wifi_connection(&mut self, socket_path: &Path) -> bool {
        self.wifi.connect_selected(socket_path)
    }

    pub fn forget_saved_wifi_network(&mut self, socket_path: &Path) -> bool {
        self.wifi.forget_saved(socket_path)
    }

    pub fn input_wifi_passphrase(&mut self, character: char) -> bool {
        self.selected_category == 3 && self.wifi.input_passphrase_character(character)
    }

    pub fn remove_wifi_passphrase_character(&mut self) -> bool {
        self.selected_category == 3 && self.wifi.remove_passphrase_character()
    }

    pub fn cancel_wifi_credential_entry(&mut self) -> bool {
        self.wifi.cancel_credential_entry()
    }

    pub fn reconcile_audio_state(
        &mut self,
        state: AudioAuthoritativeState,
    ) -> Result<(), AudioAdapterError> {
        self.audio.reconcile(state)
    }

    pub fn synchronize_audio_backend<B: AudioBackend>(
        &mut self,
        backend: &mut B,
    ) -> Result<AudioBackendDriveOutcome, AudioBackendDriveError<B::Error>> {
        self.audio.synchronize_backend(backend)
    }

    pub fn load_or_default(path: &Path) -> Result<Self, SettingsConfigError> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(SettingsConfigError::SymlinkNotAllowed)
            }
            Ok(metadata) if !metadata.is_file() => return Err(SettingsConfigError::NotRegularFile),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(SettingsConfigError::Io(error)),
        }

        let contents = fs::read_to_string(path).map_err(SettingsConfigError::Io)?;
        Self::from_config(&contents)
    }

    pub fn from_config(contents: &str) -> Result<Self, SettingsConfigError> {
        let mut version = None;
        let mut reduced_motion = None;
        let mut desktop_icons = None;
        let mut key_repeat = None;
        let mut audio_volume = None;
        let mut audio_muted = None;
        let mut theme = None;
        for line in contents.lines() {
            let (key, value) = line
                .split_once('=')
                .ok_or(SettingsConfigError::InvalidFormat)?;
            match key {
                "version" if version.is_none() => {
                    version = Some(
                        value
                            .parse::<u8>()
                            .map_err(|_| SettingsConfigError::InvalidFormat)?,
                    );
                }
                "reduced_motion" if reduced_motion.is_none() => {
                    reduced_motion = Some(match value {
                        "true" => true,
                        "false" => false,
                        _ => return Err(SettingsConfigError::InvalidFormat),
                    });
                }
                "desktop_icons" if desktop_icons.is_none() => {
                    desktop_icons = Some(match value {
                        "true" => true,
                        "false" => false,
                        _ => return Err(SettingsConfigError::InvalidFormat),
                    });
                }
                "key_repeat" if key_repeat.is_none() => {
                    key_repeat = Some(match value {
                        "true" => true,
                        "false" => false,
                        _ => return Err(SettingsConfigError::InvalidFormat),
                    });
                }
                "theme" if theme.is_none() => {
                    theme = AquaTheme::parse(value);
                    if theme.is_none() {
                        return Err(SettingsConfigError::InvalidFormat);
                    }
                }
                "audio_volume" if audio_volume.is_none() => {
                    let value = value
                        .parse::<u8>()
                        .map_err(|_| SettingsConfigError::InvalidFormat)?;
                    if value > 100 {
                        return Err(SettingsConfigError::InvalidFormat);
                    }
                    audio_volume = Some(value);
                }
                "audio_muted" if audio_muted.is_none() => {
                    audio_muted = Some(match value {
                        "true" => true,
                        "false" => false,
                        _ => return Err(SettingsConfigError::InvalidFormat),
                    });
                }
                _ => return Err(SettingsConfigError::InvalidFormat),
            }
        }
        if version != Some(SETTINGS_CONFIG_VERSION) {
            return Err(SettingsConfigError::UnsupportedVersion);
        }
        let reduced_motion = reduced_motion.ok_or(SettingsConfigError::InvalidFormat)?;
        let audio = AudioVolumeModel {
            adapter: AudioServiceAdapter::with_preferences(
                audio_volume.unwrap_or(70),
                audio_muted.unwrap_or(false),
            )
            .map_err(|_| SettingsConfigError::InvalidFormat)?,
        };
        Ok(Self {
            reduced_motion,
            desktop_icons: desktop_icons.unwrap_or(true),
            key_repeat: key_repeat.unwrap_or(true),
            audio,
            theme: theme.unwrap_or_default(),
            ..Self::default()
        })
    }

    pub fn persist(&self, path: &Path) -> Result<(), SettingsConfigError> {
        let parent = path.parent().ok_or(SettingsConfigError::MissingParent)?;
        fs::create_dir_all(parent).map_err(SettingsConfigError::Io)?;
        if let Ok(metadata) = fs::symlink_metadata(path) {
            if metadata.file_type().is_symlink() {
                return Err(SettingsConfigError::SymlinkNotAllowed);
            }
            if !metadata.is_file() {
                return Err(SettingsConfigError::NotRegularFile);
            }
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(SettingsConfigError::InvalidPath)?;
        let temporary = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
        let _ = fs::remove_file(&temporary);
        let result = (|| {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            let mut file = options.open(&temporary).map_err(SettingsConfigError::Io)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(0o600))
                    .map_err(SettingsConfigError::Io)?;
            }
            file.write_all(self.to_config().as_bytes())
                .map_err(SettingsConfigError::Io)?;
            file.sync_all().map_err(SettingsConfigError::Io)?;
            fs::rename(&temporary, path).map_err(SettingsConfigError::Io)?;
            #[cfg(unix)]
            fs::File::open(parent)
                .and_then(|directory| directory.sync_all())
                .map_err(SettingsConfigError::Io)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    pub fn to_config(&self) -> String {
        format!(
            "version={SETTINGS_CONFIG_VERSION}\nreduced_motion={}\ndesktop_icons={}\nkey_repeat={}\ntheme={}\naudio_volume={}\naudio_muted={}\n",
            self.reduced_motion,
            self.desktop_icons,
            self.key_repeat,
            self.theme.id(),
            self.audio.volume_percent(),
            self.audio.muted()
        )
    }

    pub fn handle_pointer(&mut self, x: u32, y: u32) -> SettingsUpdate {
        self.keyboard_focus = false;
        if let Some(category) = SETTINGS_SIDEBAR_NAVIGATION.hit_test(x, y, self.categories.len()) {
            self.selected_category = category;
            return SettingsUpdate::CategorySelected(category);
        }
        if let Some(control) = self.active_switch() {
            if control.pointer_toggles(x, y) {
                return match self.selected_category {
                    0 => {
                        self.reduced_motion = !self.reduced_motion;
                        SettingsUpdate::ReducedMotionChanged(self.reduced_motion)
                    }
                    1 => {
                        self.desktop_icons = !self.desktop_icons;
                        SettingsUpdate::DesktopIconsChanged(self.desktop_icons)
                    }
                    2 => {
                        self.key_repeat = !self.key_repeat;
                        SettingsUpdate::KeyRepeatChanged(self.key_repeat)
                    }
                    3 if self.wifi.controls_enabled() => {
                        SettingsUpdate::WifiControlRequested(!self.wifi.connected())
                    }
                    4 => {
                        let muted = !self.audio.muted();
                        self.audio.set_muted(muted);
                        SettingsUpdate::AudioMutedChanged(muted)
                    }
                    _ => SettingsUpdate::None,
                };
            }
        }
        if self.selected_category == 3 && !self.wifi.credential_entry() {
            for index in 0..self.wifi.networks().len().min(MAX_VISIBLE_WIFI_NETWORKS) {
                if self
                    .wifi_network_row(index)
                    .is_some_and(|row| row.pointer_hit(x, y))
                    && self.wifi.begin_credential_entry(index)
                {
                    return SettingsUpdate::WifiNetworkSelected(index);
                }
            }
            if self.wifi_rescan_button().pointer_hit(x, y) {
                return SettingsUpdate::WifiScanRequested;
            }
            if self.wifi_forget_button().pointer_hit(x, y) {
                return SettingsUpdate::WifiForgetRequested;
            }
        }
        if self.selected_category == 4 {
            if let Some(value) = self.audio_slider().value_for_pointer(x, y) {
                let value = value as u8;
                if self.audio.set_volume_percent(value) {
                    return SettingsUpdate::AudioVolumeChanged(value);
                }
            }
        }
        if self.selected_category == 0 {
            if let Some(theme) = self
                .theme_segmented_control()
                .hit_test(x, y)
                .and_then(|index| AquaTheme::ALL.get(index).copied())
            {
                self.theme = theme;
                return SettingsUpdate::ThemeChanged(theme);
            }
        }
        SettingsUpdate::None
    }

    pub fn handle_hover(&mut self, x: u32, y: u32) -> bool {
        let previous = self.hovered_category;
        self.hovered_category = None;
        self.hovered_category = SETTINGS_SIDEBAR_NAVIGATION.hit_test(x, y, self.categories.len());
        previous != self.hovered_category
    }

    pub fn handle_key(&mut self, key: SettingsKey) -> SettingsUpdate {
        self.keyboard_focus = true;
        match key {
            SettingsKey::Activate
                if self.selected_category == 3 && self.wifi.credential_entry() =>
            {
                SettingsUpdate::WifiConnectRequested
            }
            SettingsKey::Home | SettingsKey::End | SettingsKey::Up | SettingsKey::Down
                if self.selected_category == 3 && self.wifi.credential_entry() =>
            {
                SettingsUpdate::None
            }
            SettingsKey::Home | SettingsKey::End | SettingsKey::Up | SettingsKey::Down => {
                let navigation_key = match key {
                    SettingsKey::Home => SidebarNavigationKey::Home,
                    SettingsKey::End => SidebarNavigationKey::End,
                    SettingsKey::Up => SidebarNavigationKey::Previous,
                    SettingsKey::Down => SidebarNavigationKey::Next,
                    _ => unreachable!(),
                };
                let Some(selected_category) = SETTINGS_SIDEBAR_NAVIGATION.keyboard_target(
                    self.selected_category,
                    self.categories.len(),
                    navigation_key,
                ) else {
                    return SettingsUpdate::None;
                };
                self.selected_category = selected_category;
                SettingsUpdate::CategorySelected(self.selected_category)
            }
            SettingsKey::Decrease | SettingsKey::Increase if self.selected_category == 0 => {
                let navigation_key = if key == SettingsKey::Decrease {
                    SegmentNavigationKey::Previous
                } else {
                    SegmentNavigationKey::Next
                };
                let Some(selected_index) = self
                    .theme_segmented_control()
                    .keyboard_target(navigation_key)
                else {
                    return SettingsUpdate::None;
                };
                let Some(theme) = AquaTheme::ALL.get(selected_index).copied() else {
                    return SettingsUpdate::None;
                };
                self.theme = theme;
                SettingsUpdate::ThemeChanged(theme)
            }
            SettingsKey::Decrease | SettingsKey::Increase if self.selected_category == 3 => {
                self.activate_wifi_action(key)
            }
            SettingsKey::Decrease if self.selected_category == 4 => {
                self.adjust_audio_volume(SliderKey::Decrease)
            }
            SettingsKey::Increase if self.selected_category == 4 => {
                self.adjust_audio_volume(SliderKey::Increase)
            }
            SettingsKey::Activate => self.activate_switch(),
            SettingsKey::Decrease | SettingsKey::Increase => SettingsUpdate::None,
        }
    }

    fn activate_switch(&mut self) -> SettingsUpdate {
        if !self
            .active_switch()
            .is_some_and(|control| control.keyboard_toggles(ActivationKey::Enter))
        {
            return SettingsUpdate::None;
        }
        match self.selected_category {
            0 => {
                self.reduced_motion = !self.reduced_motion;
                SettingsUpdate::ReducedMotionChanged(self.reduced_motion)
            }
            1 => {
                self.desktop_icons = !self.desktop_icons;
                SettingsUpdate::DesktopIconsChanged(self.desktop_icons)
            }
            2 => {
                self.key_repeat = !self.key_repeat;
                SettingsUpdate::KeyRepeatChanged(self.key_repeat)
            }
            3 => SettingsUpdate::WifiControlRequested(!self.wifi.connected()),
            4 => {
                let muted = !self.audio.muted();
                self.audio.set_muted(muted);
                SettingsUpdate::AudioMutedChanged(muted)
            }
            _ => SettingsUpdate::None,
        }
    }

    fn activate_wifi_action(&self, key: SettingsKey) -> SettingsUpdate {
        if self.wifi.credential_entry() {
            return SettingsUpdate::None;
        }
        let (button, update) = if key == SettingsKey::Decrease {
            (self.wifi_rescan_button(), SettingsUpdate::WifiScanRequested)
        } else {
            (
                self.wifi_forget_button(),
                SettingsUpdate::WifiForgetRequested,
            )
        };
        if button.keyboard_activates(ActivationKey::Enter) {
            update
        } else {
            SettingsUpdate::None
        }
    }

    fn adjust_audio_volume(&mut self, key: SliderKey) -> SettingsUpdate {
        let Some(value) = self.audio_slider().keyboard_value(key) else {
            return SettingsUpdate::None;
        };
        let value = value as u8;
        if self.audio.set_volume_percent(value) {
            SettingsUpdate::AudioVolumeChanged(value)
        } else {
            SettingsUpdate::None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopBarState {
    pub product_label: String,
    pub clock_label: String,
    pub network_connected: bool,
    pub battery_percent: Option<u8>,
    pub audio_available: bool,
}

pub fn top_system_bar(width: u32, height: u32) -> TopSystemBar<'static> {
    TopSystemBar::new(
        Rect {
            x: 0,
            y: 0,
            width,
            height,
        },
        "Aqua system bar",
    )
}

impl TopBarState {
    pub fn read(root: &Path, epoch_seconds: u64) -> Self {
        let network_connected = read_network_snapshot(
            &root.join("sys/class/net"),
            &root.join("proc/net/route"),
            &root.join("etc/resolv.conf"),
        )
        .is_ok_and(|state| state.health() == aqua_service_adapters::NetworkServiceHealth::Online);

        Self {
            product_label: "Aqua Linux".to_string(),
            clock_label: format_top_bar_clock(epoch_seconds),
            network_connected,
            battery_percent: read_battery_percent(&root.join("sys/class/power_supply")),
            audio_available: root.join("dev/snd").is_dir(),
        }
    }
}

fn read_battery_percent(power_supply: &Path) -> Option<u8> {
    let entries = fs::read_dir(power_supply).ok()?;
    for entry in entries.flatten().take(16) {
        let path = entry.path();
        let Ok(supply_type) = fs::read_to_string(path.join("type")) else {
            continue;
        };
        if supply_type.trim() != "Battery" {
            continue;
        }
        let Ok(capacity) = fs::read_to_string(path.join("capacity")) else {
            continue;
        };
        let Ok(capacity) = capacity.trim().parse::<u8>() else {
            continue;
        };
        return Some(capacity.min(100));
    }
    None
}

fn format_top_bar_clock(epoch_seconds: u64) -> String {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let days = (epoch_seconds / 86_400) as i64;
    let seconds_today = epoch_seconds % 86_400;
    let (year, month, day) = civil_date_from_unix_days(days);
    let weekday = WEEKDAYS[(days + 4).rem_euclid(7) as usize];
    format!(
        "{weekday}, {day:02} {} {year}  {:02}:{:02} UTC",
        MONTHS[(month - 1) as usize],
        seconds_today / 3_600,
        seconds_today % 3_600 / 60
    )
}

fn civil_date_from_unix_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_piece = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_piece + 2) / 5 + 1;
    let month = month_piece + if month_piece < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
}

#[derive(Debug)]
pub enum SettingsConfigError {
    Io(io::Error),
    InvalidFormat,
    UnsupportedVersion,
    MissingParent,
    InvalidPath,
    SymlinkNotAllowed,
    NotRegularFile,
}

impl std::fmt::Display for SettingsConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "settings config I/O error: {error}"),
            Self::InvalidFormat => formatter.write_str("invalid settings config format"),
            Self::UnsupportedVersion => formatter.write_str("unsupported settings config version"),
            Self::MissingParent => formatter.write_str("settings config path has no parent"),
            Self::InvalidPath => formatter.write_str("settings config path is invalid"),
            Self::SymlinkNotAllowed => {
                formatter.write_str("settings config symlink is not allowed")
            }
            Self::NotRegularFile => formatter.write_str("settings config is not a regular file"),
        }
    }
}

impl std::error::Error for SettingsConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsKey {
    Home,
    End,
    Up,
    Down,
    Activate,
    Decrease,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsUpdate {
    None,
    CategorySelected(usize),
    ReducedMotionChanged(bool),
    DesktopIconsChanged(bool),
    KeyRepeatChanged(bool),
    WifiControlRequested(bool),
    WifiNetworkSelected(usize),
    WifiScanRequested,
    WifiConnectRequested,
    WifiForgetRequested,
    ThemeChanged(AquaTheme),
    AudioVolumeChanged(u8),
    AudioMutedChanged(bool),
}

impl SettingsUpdate {
    pub const fn changed(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesEntryKind {
    Folder,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesEntry {
    pub name: String,
    pub detail: String,
    pub kind: FilesEntryKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesWindowModel {
    pub title: &'static str,
    pub location: String,
    pub sidebar_items: [&'static str; 5],
    pub selected_sidebar: usize,
    pub selected_entry: Option<usize>,
    pub hovered_sidebar: Option<usize>,
    pub hovered_entry: Option<usize>,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub scroll_offset: usize,
    pub keyboard_focus: bool,
    pub preview: Option<FilesTextPreview>,
    pub entries: Vec<FilesEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesTextPreview {
    pub name: String,
    pub content: String,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilesScrollbarLayout {
    pub track: Rect,
    pub thumb: Rect,
    maximum_offset: usize,
}

impl FilesScrollbarLayout {
    fn new(track: Rect, item_count: usize, visible_count: usize, offset: usize) -> Option<Self> {
        let maximum_offset = item_count.checked_sub(visible_count)?;
        if maximum_offset == 0 || track.height == 0 {
            return None;
        }
        let thumb_height = ((track.height as usize).saturating_mul(visible_count) / item_count)
            .max(FILES_SCROLLBAR_MIN_THUMB_HEIGHT as usize)
            .min(track.height as usize) as u32;
        let thumb_travel = track.height.saturating_sub(thumb_height);
        let bounded_offset = offset.min(maximum_offset);
        let thumb_offset = (thumb_travel as usize).saturating_mul(bounded_offset) / maximum_offset;
        Some(Self {
            track,
            thumb: Rect {
                x: track.x,
                y: track.y + thumb_offset as u32,
                width: track.width,
                height: thumb_height,
            },
            maximum_offset,
        })
    }

    pub const fn pointer_hit(self, x: u32, y: u32) -> bool {
        x >= self.track.x && x < self.track.right() && y >= self.track.y && y < self.track.bottom()
    }

    pub fn offset_for_pointer(self, y: u32) -> usize {
        let position = y.clamp(self.track.y, self.track.bottom()) - self.track.y;
        ((u128::from(position) * self.maximum_offset as u128 + u128::from(self.track.height) / 2)
            / u128::from(self.track.height)) as usize
    }
}

impl Default for FilesWindowModel {
    fn default() -> Self {
        Self {
            title: "Files",
            location: "Aqua / Home".to_string(),
            sidebar_items: ["Home", "Documents", "Downloads", "Pictures", "Trash"],
            selected_sidebar: 0,
            selected_entry: None,
            hovered_sidebar: None,
            hovered_entry: None,
            can_go_back: false,
            can_go_forward: false,
            scroll_offset: 0,
            keyboard_focus: false,
            preview: None,
            entries: vec![
                FilesEntry {
                    name: "Documents".to_string(),
                    detail: "Folder".to_string(),
                    kind: FilesEntryKind::Folder,
                },
                FilesEntry {
                    name: "Downloads".to_string(),
                    detail: "Folder".to_string(),
                    kind: FilesEntryKind::Folder,
                },
                FilesEntry {
                    name: "Pictures".to_string(),
                    detail: "Folder".to_string(),
                    kind: FilesEntryKind::Folder,
                },
                FilesEntry {
                    name: "Welcome.txt".to_string(),
                    detail: "1 KB text file".to_string(),
                    kind: FilesEntryKind::File,
                },
            ],
        }
    }
}

impl FilesWindowModel {
    pub fn empty(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            entries: Vec::new(),
            ..Self::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn list_scrollbar(&self, width: u32) -> Option<FilesScrollbarLayout> {
        if self.preview.is_some() {
            return None;
        }
        FilesScrollbarLayout::new(
            Rect {
                x: width.saturating_sub(FILES_SCROLLBAR_TRAILING_INSET),
                y: FILES_LIST_SCROLLBAR_Y,
                width: FILES_SCROLLBAR_WIDTH,
                height: FILES_LIST_SCROLLBAR_HEIGHT,
            },
            self.entries.len(),
            FILES_VISIBLE_ROWS,
            self.scroll_offset,
        )
    }

    pub fn preview_scrollbar(&self, width: u32) -> Option<FilesScrollbarLayout> {
        let preview = self.preview.as_ref()?;
        FilesScrollbarLayout::new(
            Rect {
                x: width.saturating_sub(FILES_SCROLLBAR_TRAILING_INSET),
                y: FILES_PREVIEW_SCROLLBAR_Y,
                width: FILES_SCROLLBAR_WIDTH,
                height: FILES_PREVIEW_SCROLLBAR_HEIGHT,
            },
            preview.content.lines().count(),
            FILES_PREVIEW_VISIBLE_LINES,
            preview.scroll_offset,
        )
    }

    pub fn active_scrollbar(&self, width: u32) -> Option<FilesScrollbarLayout> {
        self.preview_scrollbar(width)
            .or_else(|| self.list_scrollbar(width))
    }

    pub fn entry_row(&self, width: u32, index: usize) -> Option<ListRow<'_>> {
        if self.preview.is_some() {
            return None;
        }
        let visible_index = index.checked_sub(self.scroll_offset)?;
        if visible_index >= FILES_VISIBLE_ROWS {
            return None;
        }
        let entry = self.entries.get(index)?;
        let state = if self.selected_entry == Some(index) {
            ComponentState::Selected
        } else if self.hovered_entry == Some(index) {
            ComponentState::Hover
        } else {
            ComponentState::Idle
        };
        let row_width = width.saturating_sub(204);
        let trailing_width = row_width
            .saturating_sub(16)
            .saturating_sub(54 + 80)
            .min(130);
        Some(
            ListRow::new(
                Rect {
                    x: 188,
                    y: 124 + visible_index as u32 * 64,
                    width: row_width,
                    height: 56,
                },
                &entry.name,
                ListRowRole::Option,
            )
            .with_slots(54, trailing_width)
            .with_state(state),
        )
    }

    pub fn read_only_directory(
        allowed_root: &Path,
        requested: &Path,
    ) -> Result<Self, FilesReadError> {
        let root = allowed_root
            .canonicalize()
            .map_err(|_| FilesReadError::UnavailableRoot)?;
        let directory = requested
            .canonicalize()
            .map_err(|_| FilesReadError::UnavailableDirectory)?;
        if !directory.starts_with(&root) {
            return Err(FilesReadError::OutsideAllowedRoot);
        }

        let mut entries = fs::read_dir(&directory)
            .map_err(|_| FilesReadError::UnreadableDirectory)?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                if file_type.is_symlink() || (!file_type.is_dir() && !file_type.is_file()) {
                    return None;
                }
                let name = entry.file_name().into_string().ok()?;
                if name.starts_with('.') {
                    return None;
                }
                let kind = if file_type.is_dir() {
                    FilesEntryKind::Folder
                } else {
                    FilesEntryKind::File
                };
                let detail = match kind {
                    FilesEntryKind::Folder => "Folder".to_string(),
                    FilesEntryKind::File => entry
                        .metadata()
                        .ok()
                        .map(|metadata| format!("{} bytes", metadata.len()))
                        .unwrap_or_else(|| "File".to_string()),
                };
                Some(FilesEntry { name, detail, kind })
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            let left_rank = matches!(left.kind, FilesEntryKind::File);
            let right_rank = matches!(right.kind, FilesEntryKind::File);
            left_rank.cmp(&right_rank).then_with(|| {
                left.name
                    .to_ascii_lowercase()
                    .cmp(&right.name.to_ascii_lowercase())
            })
        });
        let relative = directory.strip_prefix(&root).unwrap_or(Path::new(""));
        let location = if relative.as_os_str().is_empty() {
            "Aqua / Home".to_string()
        } else {
            format!("Aqua / Home / {}", relative.display())
        };
        Ok(Self {
            location,
            entries,
            ..Self::default()
        })
    }

    pub fn select_at(&mut self, width: u32, x: u32, y: u32) -> FilesSelection {
        let back = files_back_button().with_state(if self.can_go_back {
            ComponentState::Idle
        } else {
            ComponentState::Disabled
        });
        if back.pointer_hit(x, y) {
            return FilesSelection::Back;
        }
        let forward = files_forward_button().with_state(if self.can_go_forward {
            ComponentState::Idle
        } else {
            ComponentState::Disabled
        });
        if forward.pointer_hit(x, y) {
            return FilesSelection::Forward;
        }
        if let Some(index) = FILES_SIDEBAR_NAVIGATION.hit_test(x, y, self.sidebar_items.len()) {
            self.selected_sidebar = index;
            self.selected_entry = None;
            return FilesSelection::Sidebar(index);
        }
        for index in self.scroll_offset
            ..self
                .entries
                .len()
                .min(self.scroll_offset + FILES_VISIBLE_ROWS)
        {
            if self
                .entry_row(width, index)
                .is_some_and(|row| row.pointer_hit(x, y))
            {
                self.selected_entry = Some(index);
                return FilesSelection::Entry(index);
            }
        }
        FilesSelection::None
    }

    pub fn hover_at(&mut self, width: u32, x: u32, y: u32) -> bool {
        let previous = (self.hovered_sidebar, self.hovered_entry);
        self.hovered_sidebar = None;
        self.hovered_entry = None;
        if let Some(index) = FILES_SIDEBAR_NAVIGATION.hit_test(x, y, self.sidebar_items.len()) {
            self.hovered_sidebar = Some(index);
        } else {
            for index in self.scroll_offset
                ..self
                    .entries
                    .len()
                    .min(self.scroll_offset + FILES_VISIBLE_ROWS)
            {
                if self
                    .entry_row(width, index)
                    .is_some_and(|row| row.pointer_hit(x, y))
                {
                    self.hovered_entry = Some(index);
                    break;
                }
            }
        }
        previous != (self.hovered_sidebar, self.hovered_entry)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesSelection {
    None,
    Back,
    Forward,
    Sidebar(usize),
    Entry(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesNavigator {
    root: PathBuf,
    current: PathBuf,
    back: Vec<PathBuf>,
    forward: Vec<PathBuf>,
    window: FilesWindowModel,
}

impl FilesNavigator {
    pub fn open(root: &Path) -> Result<Self, FilesReadError> {
        let root = root
            .canonicalize()
            .map_err(|_| FilesReadError::UnavailableRoot)?;
        let window = FilesWindowModel::read_only_directory(&root, &root)?;
        Ok(Self {
            current: root.clone(),
            root,
            back: Vec::new(),
            forward: Vec::new(),
            window,
        })
    }

    pub fn window(&self) -> &FilesWindowModel {
        &self.window
    }

    pub fn current(&self) -> &Path {
        &self.current
    }

    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    pub fn handle_pointer(&mut self, width: u32, x: u32, y: u32) -> FilesNavigation {
        let previously_selected = self.window.selected_entry;
        match self.window.select_at(width, x, y) {
            FilesSelection::None => FilesNavigation::None,
            FilesSelection::Back => self.go_back(),
            FilesSelection::Forward => self.go_forward(),
            FilesSelection::Sidebar(index) => {
                let destination = match index {
                    0 => self.root.clone(),
                    1 => self.root.join("Documents"),
                    2 => self.root.join("Downloads"),
                    3 => self.root.join("Pictures"),
                    4 => self.root.join("Trash"),
                    _ => return FilesNavigation::None,
                };
                self.navigate(destination, Some(index))
            }
            FilesSelection::Entry(index) => {
                let Some(entry) = self.window.entries.get(index).cloned() else {
                    return FilesNavigation::None;
                };
                if previously_selected == Some(index) {
                    if entry.kind == FilesEntryKind::Folder {
                        self.navigate(self.current.join(entry.name), None)
                    } else {
                        self.open_text_preview(index)
                    }
                } else {
                    self.window.selected_entry = Some(index);
                    FilesNavigation::Selected(index)
                }
            }
        }
    }

    pub fn handle_hover(&mut self, width: u32, x: u32, y: u32) -> bool {
        self.window.hover_at(width, x, y)
    }

    pub fn handle_scroll(&mut self, rows: isize) -> FilesNavigation {
        if let Some(preview) = self.window.preview.as_mut() {
            let max_offset = preview
                .content
                .lines()
                .count()
                .saturating_sub(FILES_PREVIEW_VISIBLE_LINES);
            let offset =
                (preview.scroll_offset as isize + rows).clamp(0, max_offset as isize) as usize;
            if offset == preview.scroll_offset {
                return FilesNavigation::None;
            }
            preview.scroll_offset = offset;
            return FilesNavigation::PreviewScrolled;
        }
        if self.window.entries.len() <= FILES_VISIBLE_ROWS {
            return FilesNavigation::None;
        }
        let max_offset = self.window.entries.len() - FILES_VISIBLE_ROWS;
        let offset =
            (self.window.scroll_offset as isize + rows).clamp(0, max_offset as isize) as usize;
        if offset == self.window.scroll_offset {
            return FilesNavigation::None;
        }
        self.window.scroll_offset = offset;
        self.window.hovered_entry = None;
        FilesNavigation::Scrolled
    }

    pub fn scrollbar_hit(&self, width: u32, x: u32, y: u32) -> bool {
        self.window
            .active_scrollbar(width)
            .is_some_and(|scrollbar| scrollbar.pointer_hit(x, y))
    }

    pub fn handle_scrollbar_drag(&mut self, width: u32, y: u32) -> FilesNavigation {
        let Some(scrollbar) = self.window.active_scrollbar(width) else {
            return FilesNavigation::None;
        };
        let offset = scrollbar.offset_for_pointer(y);
        if let Some(preview) = self.window.preview.as_mut() {
            if offset == preview.scroll_offset {
                return FilesNavigation::None;
            }
            preview.scroll_offset = offset;
            return FilesNavigation::PreviewScrolled;
        }
        if offset == self.window.scroll_offset {
            return FilesNavigation::None;
        }
        self.window.scroll_offset = offset;
        self.window.hovered_entry = None;
        FilesNavigation::Scrolled
    }

    pub fn handle_key(&mut self, width: u32, key: FilesKey) -> FilesNavigation {
        self.window.keyboard_focus = true;
        if self.window.preview.is_some() {
            return self.handle_preview_key(key);
        }
        match key {
            FilesKey::Up => self.navigate_list(ListNavigationKey::Previous),
            FilesKey::Down => self.navigate_list(ListNavigationKey::Next),
            FilesKey::PageUp => self.navigate_list(ListNavigationKey::PagePrevious),
            FilesKey::PageDown => self.navigate_list(ListNavigationKey::PageNext),
            FilesKey::Home => self.navigate_list(ListNavigationKey::Home),
            FilesKey::End => self.navigate_list(ListNavigationKey::End),
            FilesKey::Activate => {
                let Some(index) = self.window.selected_entry else {
                    return FilesNavigation::None;
                };
                if !self
                    .window
                    .entry_row(width, index)
                    .is_some_and(|row| row.keyboard_activates(ActivationKey::Enter))
                {
                    return FilesNavigation::None;
                }
                let Some(entry) = self.window.entries.get(index).cloned() else {
                    return FilesNavigation::None;
                };
                if entry.kind == FilesEntryKind::Folder {
                    self.navigate(self.current.join(entry.name), None)
                } else {
                    self.open_text_preview(index)
                }
            }
            FilesKey::Back => self.go_back(),
        }
    }

    fn handle_preview_key(&mut self, key: FilesKey) -> FilesNavigation {
        match key {
            FilesKey::Up => self.handle_scroll(-1),
            FilesKey::Down => self.handle_scroll(1),
            FilesKey::PageUp => self.handle_scroll(-(FILES_PREVIEW_VISIBLE_LINES as isize)),
            FilesKey::PageDown => self.handle_scroll(FILES_PREVIEW_VISIBLE_LINES as isize),
            FilesKey::Home => self.set_preview_scroll_offset(0),
            FilesKey::End => {
                let maximum_offset = self
                    .window
                    .preview
                    .as_ref()
                    .map(|preview| {
                        preview
                            .content
                            .lines()
                            .count()
                            .saturating_sub(FILES_PREVIEW_VISIBLE_LINES)
                    })
                    .unwrap_or(0);
                self.set_preview_scroll_offset(maximum_offset)
            }
            FilesKey::Activate => FilesNavigation::None,
            FilesKey::Back => {
                self.window.preview = None;
                FilesNavigation::PreviewClosed
            }
        }
    }

    fn set_preview_scroll_offset(&mut self, offset: usize) -> FilesNavigation {
        let Some(preview) = self.window.preview.as_mut() else {
            return FilesNavigation::None;
        };
        let maximum_offset = preview
            .content
            .lines()
            .count()
            .saturating_sub(FILES_PREVIEW_VISIBLE_LINES);
        let offset = offset.min(maximum_offset);
        if offset == preview.scroll_offset {
            return FilesNavigation::None;
        }
        preview.scroll_offset = offset;
        FilesNavigation::PreviewScrolled
    }

    fn navigate_list(&mut self, key: ListNavigationKey) -> FilesNavigation {
        let navigation = ListNavigation::new(self.window.entries.len(), FILES_VISIBLE_ROWS);
        let Some(selected) = navigation.keyboard_target(self.window.selected_entry, key) else {
            return FilesNavigation::None;
        };
        let Some(scroll_offset) = navigation.reveal_offset(selected, self.window.scroll_offset)
        else {
            return FilesNavigation::None;
        };
        self.window.selected_entry = Some(selected);
        self.window.scroll_offset = scroll_offset;
        FilesNavigation::Selected(selected)
    }

    fn open_text_preview(&mut self, index: usize) -> FilesNavigation {
        let Some(entry) = self.window.entries.get(index) else {
            return FilesNavigation::None;
        };
        if entry.kind != FilesEntryKind::File
            || Path::new(&entry.name)
                .extension()
                .and_then(|value| value.to_str())
                != Some("txt")
        {
            return FilesNavigation::PreviewBlocked;
        }
        let requested = self.current.join(&entry.name);
        let Ok(path) = requested.canonicalize() else {
            return FilesNavigation::PreviewBlocked;
        };
        if !path.starts_with(&self.root) {
            return FilesNavigation::PreviewBlocked;
        }
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            return FilesNavigation::PreviewBlocked;
        };
        if !metadata.file_type().is_file() || metadata.len() > FILES_TEXT_PREVIEW_LIMIT {
            return FilesNavigation::PreviewBlocked;
        }
        let Ok(content) = fs::read_to_string(path) else {
            return FilesNavigation::PreviewBlocked;
        };
        self.window.preview = Some(FilesTextPreview {
            name: entry.name.clone(),
            content,
            scroll_offset: 0,
        });
        FilesNavigation::PreviewOpened
    }

    fn navigate(&mut self, requested: PathBuf, sidebar: Option<usize>) -> FilesNavigation {
        let Ok(destination) = requested.canonicalize() else {
            return FilesNavigation::Blocked;
        };
        if !destination.starts_with(&self.root) {
            return FilesNavigation::Blocked;
        }
        if destination == self.current {
            self.window.selected_entry = None;
            if let Some(index) = sidebar {
                self.window.selected_sidebar = index;
            }
            return FilesNavigation::SelectedSidebar(self.window.selected_sidebar);
        }
        let Ok(mut window) = FilesWindowModel::read_only_directory(&self.root, &destination) else {
            return FilesNavigation::Blocked;
        };
        window.selected_sidebar = sidebar.unwrap_or_else(|| self.sidebar_for(&destination));
        self.back.push(self.current.clone());
        self.forward.clear();
        self.current = destination;
        self.window = window;
        self.sync_history_state();
        FilesNavigation::Navigated
    }

    fn go_back(&mut self) -> FilesNavigation {
        let Some(destination) = self.back.pop() else {
            return FilesNavigation::None;
        };
        let Ok(mut window) = FilesWindowModel::read_only_directory(&self.root, &destination) else {
            return FilesNavigation::Blocked;
        };
        window.selected_sidebar = self.sidebar_for(&destination);
        self.forward.push(self.current.clone());
        self.current = destination;
        self.window = window;
        self.sync_history_state();
        FilesNavigation::NavigatedBack
    }

    fn go_forward(&mut self) -> FilesNavigation {
        let Some(destination) = self.forward.pop() else {
            return FilesNavigation::None;
        };
        let Ok(mut window) = FilesWindowModel::read_only_directory(&self.root, &destination) else {
            return FilesNavigation::Blocked;
        };
        window.selected_sidebar = self.sidebar_for(&destination);
        self.back.push(self.current.clone());
        self.current = destination;
        self.window = window;
        self.sync_history_state();
        FilesNavigation::NavigatedForward
    }

    fn sidebar_for(&self, destination: &Path) -> usize {
        ["", "Documents", "Downloads", "Pictures", "Trash"]
            .iter()
            .position(|name| self.root.join(name) == destination)
            .unwrap_or(0)
    }

    fn sync_history_state(&mut self) {
        self.window.can_go_back = !self.back.is_empty();
        self.window.can_go_forward = !self.forward.is_empty();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesKey {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Activate,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesNavigation {
    None,
    Selected(usize),
    SelectedSidebar(usize),
    Navigated,
    NavigatedBack,
    NavigatedForward,
    PreviewOpened,
    PreviewClosed,
    PreviewScrolled,
    PreviewBlocked,
    Scrolled,
    Blocked,
}

impl FilesNavigation {
    pub const fn changed(self) -> bool {
        !matches!(self, Self::None | Self::Blocked | Self::PreviewBlocked)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesReadError {
    UnavailableRoot,
    UnavailableDirectory,
    OutsideAllowedRoot,
    UnreadableDirectory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherPointerTarget {
    Panel,
    SearchField,
    Category(LauncherCategory),
    Application(usize),
    QuickAction(LauncherQuickAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherQuickAction {
    Applications,
    Settings,
    Files,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherMode {
    Applications,
    Search,
}

impl LauncherMode {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Applications => "applications",
            Self::Search => "search",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LauncherPanelBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherCategory {
    Favorites,
    AllApplications,
    Development,
    Graphics,
    Internet,
    Multimedia,
    Settings,
    System,
    Utilities,
}

impl LauncherCategory {
    pub const ALL: [Self; 9] = [
        Self::Favorites,
        Self::AllApplications,
        Self::Development,
        Self::Graphics,
        Self::Internet,
        Self::Multimedia,
        Self::Settings,
        Self::System,
        Self::Utilities,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Favorites => "favorites",
            Self::AllApplications => "all-applications",
            Self::Development => "development",
            Self::Graphics => "graphics",
            Self::Internet => "internet",
            Self::Multimedia => "multimedia",
            Self::Settings => "settings",
            Self::System => "system",
            Self::Utilities => "utilities",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Favorites => "Favorites",
            Self::AllApplications => "All Applications",
            Self::Development => "Development",
            Self::Graphics => "Graphics",
            Self::Internet => "Internet",
            Self::Multimedia => "Multimedia",
            Self::Settings => "Settings",
            Self::System => "System",
            Self::Utilities => "Utilities",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LauncherApp {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: LauncherCategory,
    pub favorite: bool,
    pub command: &'static str,
    pub icon_path: &'static str,
}

pub const DEFAULT_APPS: [LauncherApp; 6] = [
    LauncherApp {
        id: "files",
        name: "Files",
        description: "File Manager",
        category: LauncherCategory::Utilities,
        favorite: true,
        command: "/usr/bin/aqua-files",
        icon_path: "/usr/share/aqua/icons/aqua/files.svg",
    },
    LauncherApp {
        id: "browser",
        name: "Browser",
        description: "Web Browser",
        category: LauncherCategory::Internet,
        favorite: true,
        command: "/usr/bin/aqua-browser",
        icon_path: "/usr/share/aqua/icons/aqua/browser.svg",
    },
    LauncherApp {
        id: "terminal",
        name: "Terminal",
        description: "System Terminal",
        category: LauncherCategory::System,
        favorite: true,
        command: "/usr/bin/aqua-terminal",
        icon_path: "/usr/share/aqua/icons/aqua/terminal.svg",
    },
    LauncherApp {
        id: "settings",
        name: "System Settings",
        description: "Preferences",
        category: LauncherCategory::Settings,
        favorite: true,
        command: "/usr/bin/aqua-settings",
        icon_path: "/usr/share/aqua/icons/aqua/settings.svg",
    },
    LauncherApp {
        id: "software",
        name: "Software Manager",
        description: "Install and remove software",
        category: LauncherCategory::System,
        favorite: true,
        command: "/usr/bin/aqua-software",
        icon_path: "/usr/share/aqua/icons/aqua/software.svg",
    },
    LauncherApp {
        id: "updates",
        name: "Updates",
        description: "Check for updates",
        category: LauncherCategory::System,
        favorite: true,
        command: "/usr/bin/aqua-updates",
        icon_path: "/usr/share/aqua/icons/aqua/updates.svg",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub app_id: &'static str,
    pub command: &'static str,
    pub target: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherEvent {
    Toggle,
    OpenApplications,
    OpenSearch,
    Dismiss,
    SelectCategory(LauncherCategory),
    ReplaceQuery(String),
    Navigate(CollectionNavigationKey),
    Activate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherUpdate {
    pub redraw_requested: bool,
    pub visibility_changed: bool,
    pub launch_request: Option<LaunchRequest>,
}

impl LauncherUpdate {
    fn unchanged() -> Self {
        Self {
            redraw_requested: false,
            visibility_changed: false,
            launch_request: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherState {
    open: bool,
    mode: LauncherMode,
    category: LauncherCategory,
    query: String,
    selected_index: usize,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self {
            open: false,
            mode: LauncherMode::Applications,
            category: LauncherCategory::Favorites,
            query: String::new(),
            selected_index: 0,
        }
    }
}

impl LauncherState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn category(&self) -> LauncherCategory {
        self.category
    }

    pub fn mode(&self) -> LauncherMode {
        self.mode
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn open(&mut self) {
        self.open_applications();
    }

    pub fn open_applications(&mut self) {
        self.open = true;
        self.mode = LauncherMode::Applications;
        self.category = LauncherCategory::AllApplications;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn open_search(&mut self) {
        self.open = true;
        self.mode = LauncherMode::Search;
        self.category = LauncherCategory::AllApplications;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.mode = LauncherMode::Applications;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn select_category(&mut self, category: LauncherCategory) {
        self.mode = LauncherMode::Applications;
        self.category = category;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        if !self.query.is_empty() {
            self.mode = LauncherMode::Search;
            self.category = LauncherCategory::AllApplications;
        }
        self.selected_index = 0;
    }

    pub fn panel_bounds(&self, viewport_width: u32, viewport_height: u32) -> LauncherPanelBounds {
        let requested_width = match self.mode {
            LauncherMode::Applications => 620,
            LauncherMode::Search => 720,
        };
        let width = requested_width.min(viewport_width.saturating_sub(48));
        let height = 460_u32.min(viewport_height.saturating_sub(140));
        LauncherPanelBounds {
            x: viewport_width.saturating_sub(width) / 2,
            y: 70_u32.min(viewport_height.saturating_sub(height) / 2),
            width,
            height,
        }
    }

    pub fn search_field(&self, viewport_width: u32, viewport_height: u32) -> SearchField<'_> {
        if self.mode == LauncherMode::Applications {
            return self
                .application_overview(viewport_width, viewport_height)
                .search_field(self.query(), ComponentState::Idle);
        }
        self.global_search(viewport_width, viewport_height)
            .search_field(self.query(), ComponentState::KeyboardFocus)
    }

    pub fn application_overview(
        &self,
        viewport_width: u32,
        viewport_height: u32,
    ) -> ApplicationOverview<'static> {
        let panel = self.panel_bounds(viewport_width, viewport_height);
        ApplicationOverview::new(
            Rect {
                x: panel.x,
                y: panel.y,
                width: panel.width,
                height: panel.height,
            },
            "Applications",
            "Search applications",
            "SEARCH APPS...",
            self.visible_apps().len(),
        )
    }

    pub fn global_search(
        &self,
        viewport_width: u32,
        viewport_height: u32,
    ) -> GlobalSearch<'static> {
        let panel = self.panel_bounds(viewport_width, viewport_height);
        GlobalSearch::new(
            Rect {
                x: panel.x,
                y: panel.y,
                width: panel.width,
                height: panel.height,
            },
            "Global Search",
            "Search applications",
            "SEARCH APPS...",
            ("Results", "Quick actions"),
            self.visible_apps().len(),
            3,
        )
    }

    pub fn visible_apps(&self) -> Vec<&'static LauncherApp> {
        let query = self.query.trim().to_ascii_lowercase();

        DEFAULT_APPS
            .iter()
            .filter(|app| self.matches_category(app))
            .filter(|app| {
                query.is_empty()
                    || app.name.to_ascii_lowercase().contains(&query)
                    || app.description.to_ascii_lowercase().contains(&query)
                    || app.id.contains(&query)
            })
            .collect()
    }

    pub fn application_grid_cell(
        &self,
        index: usize,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Option<GridCell<'static>> {
        if self.mode != LauncherMode::Applications {
            return None;
        }
        let app = *self.visible_apps().get(index)?;
        let overview = self.application_overview(viewport_width, viewport_height);
        if !overview.is_valid() {
            return None;
        }
        Some(
            GridCell::new(
                overview.cell_rect(index),
                app.name,
                GridCellLayout::IconLeading,
            )
            .with_spacing(40, 14, 6, 22)
            .with_state(if index == self.selected_index {
                ComponentState::Selected
            } else {
                ComponentState::Idle
            }),
        )
    }

    pub fn search_result_row(
        &self,
        index: usize,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Option<ListRow<'static>> {
        if self.mode != LauncherMode::Search {
            return None;
        }
        let app = *self.visible_apps().get(index)?;
        let search = self.global_search(viewport_width, viewport_height);
        if !search.is_valid() || index >= search.visible_result_count() {
            return None;
        }
        Some(
            ListRow::new(search.result_rect(index), app.name, ListRowRole::Option)
                .with_slots(46, 8)
                .with_state(if index == self.selected_index {
                    ComponentState::Selected
                } else {
                    ComponentState::Idle
                }),
        )
    }

    pub fn search_quick_action_button(
        &self,
        index: usize,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Option<StandardButton<'static>> {
        if self.mode != LauncherMode::Search {
            return None;
        }
        let label = *["OPEN APPLICATIONS", "SYSTEM SETTINGS", "BROWSE FILES"].get(index)?;
        Some(StandardButton::new(
            self.global_search(viewport_width, viewport_height)
                .quick_action_rect(index),
            label,
            StandardButtonVariant::Secondary,
        ))
    }

    pub fn navigate_selection(&mut self, key: CollectionNavigationKey) -> bool {
        self.navigate_selection_in_viewport(key, 800, 600)
    }

    pub fn navigate_selection_in_viewport(
        &mut self,
        key: CollectionNavigationKey,
        viewport_width: u32,
        viewport_height: u32,
    ) -> bool {
        let target = match self.mode {
            LauncherMode::Applications => self
                .application_overview(viewport_width, viewport_height)
                .keyboard_target(self.selected_index, key),
            LauncherMode::Search => self
                .global_search(viewport_width, viewport_height)
                .result_keyboard_target(self.selected_index, key),
        };
        let Some(target) = target else {
            return false;
        };
        if target == self.selected_index {
            return false;
        }
        self.selected_index = target;
        true
    }

    pub fn select_visible_index(&mut self, index: usize) -> bool {
        if index >= self.navigable_item_count() {
            return false;
        }
        self.selected_index = index;
        true
    }

    fn navigable_item_count(&self) -> usize {
        match self.mode {
            LauncherMode::Applications => self.visible_apps().len().min(6),
            LauncherMode::Search => self.visible_apps().len().min(5),
        }
    }

    pub fn pointer_target(&self, x: u32, y: u32) -> Option<LauncherPointerTarget> {
        self.pointer_target_in_viewport(x, y, 800, 600)
    }

    pub fn pointer_target_in_viewport(
        &self,
        x: u32,
        y: u32,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Option<LauncherPointerTarget> {
        if !self.open {
            return None;
        }
        let panel_contains = match self.mode {
            LauncherMode::Applications => self
                .application_overview(viewport_width, viewport_height)
                .contains(x, y),
            LauncherMode::Search => self
                .global_search(viewport_width, viewport_height)
                .contains(x, y),
        };
        if !panel_contains {
            return None;
        }

        if self
            .search_field(viewport_width, viewport_height)
            .pointer_focuses(x, y)
        {
            return Some(LauncherPointerTarget::SearchField);
        }

        match self.mode {
            LauncherMode::Applications => {
                let overview = self.application_overview(viewport_width, viewport_height);
                for index in 0..overview.visible_item_count() {
                    if self
                        .application_grid_cell(index, viewport_width, viewport_height)
                        .is_some_and(|cell| cell.pointer_hit(x, y))
                    {
                        return Some(LauncherPointerTarget::Application(index));
                    }
                }
            }
            LauncherMode::Search => {
                let search = self.global_search(viewport_width, viewport_height);
                for index in 0..search.visible_result_count() {
                    if self
                        .search_result_row(index, viewport_width, viewport_height)
                        .is_some_and(|row| row.pointer_hit(x, y))
                    {
                        return Some(LauncherPointerTarget::Application(index));
                    }
                }
                for (index, action) in [
                    LauncherQuickAction::Applications,
                    LauncherQuickAction::Settings,
                    LauncherQuickAction::Files,
                ]
                .into_iter()
                .enumerate()
                {
                    if self
                        .search_quick_action_button(index, viewport_width, viewport_height)
                        .is_some_and(|button| button.pointer_hit(x, y))
                    {
                        return Some(LauncherPointerTarget::QuickAction(action));
                    }
                }
            }
        }
        Some(LauncherPointerTarget::Panel)
    }

    pub fn activate_selected(&self) -> Option<LaunchRequest> {
        self.activate_selected_in_viewport(800, 600)
    }

    pub fn activate_selected_in_viewport(
        &self,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Option<LaunchRequest> {
        if !self.open {
            return None;
        }

        let can_activate = match self.mode {
            LauncherMode::Applications => self
                .application_grid_cell(self.selected_index, viewport_width, viewport_height)
                .is_some_and(|cell| cell.keyboard_activates(ActivationKey::Enter)),
            LauncherMode::Search => self
                .search_result_row(self.selected_index, viewport_width, viewport_height)
                .is_some_and(|row| row.keyboard_activates(ActivationKey::Enter)),
        };
        if !can_activate {
            return None;
        }

        self.visible_apps()
            .get(self.selected_index)
            .map(|app| LaunchRequest {
                app_id: app.id,
                command: app.command,
                target: None,
            })
    }

    pub fn activate_quick_action(&mut self, action: LauncherQuickAction) -> LauncherUpdate {
        if !self.open || self.mode != LauncherMode::Search {
            return LauncherUpdate::unchanged();
        }
        match action {
            LauncherQuickAction::Applications => {
                self.open_applications();
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed: false,
                    launch_request: None,
                }
            }
            LauncherQuickAction::Settings | LauncherQuickAction::Files => {
                let app_id = match action {
                    LauncherQuickAction::Settings => "settings",
                    LauncherQuickAction::Files => "files",
                    LauncherQuickAction::Applications => unreachable!(),
                };
                LauncherUpdate {
                    redraw_requested: false,
                    visibility_changed: false,
                    launch_request: DEFAULT_APPS.iter().find(|app| app.id == app_id).map(|app| {
                        LaunchRequest {
                            app_id: app.id,
                            command: app.command,
                            target: None,
                        }
                    }),
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: LauncherEvent) -> LauncherUpdate {
        self.handle_event_in_viewport(event, 800, 600)
    }

    pub fn handle_event_in_viewport(
        &mut self,
        event: LauncherEvent,
        viewport_width: u32,
        viewport_height: u32,
    ) -> LauncherUpdate {
        match event {
            LauncherEvent::Toggle => {
                self.toggle();
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed: true,
                    launch_request: None,
                }
            }
            LauncherEvent::OpenApplications => {
                let visibility_changed = !self.open;
                self.open_applications();
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed,
                    launch_request: None,
                }
            }
            LauncherEvent::OpenSearch => {
                let visibility_changed = !self.open;
                self.open_search();
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed,
                    launch_request: None,
                }
            }
            LauncherEvent::Dismiss => {
                if !self.open {
                    return LauncherUpdate::unchanged();
                }
                self.close();
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed: true,
                    launch_request: None,
                }
            }
            LauncherEvent::SelectCategory(category) => {
                if !self.open {
                    return LauncherUpdate::unchanged();
                }
                self.select_category(category);
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed: false,
                    launch_request: None,
                }
            }
            LauncherEvent::ReplaceQuery(query) => {
                if !self.open {
                    return LauncherUpdate::unchanged();
                }
                self.set_query(query);
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed: false,
                    launch_request: None,
                }
            }
            LauncherEvent::Navigate(key) => {
                if !self.open
                    || !self.navigate_selection_in_viewport(key, viewport_width, viewport_height)
                {
                    return LauncherUpdate::unchanged();
                }
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed: false,
                    launch_request: None,
                }
            }
            LauncherEvent::Activate => LauncherUpdate {
                redraw_requested: false,
                visibility_changed: false,
                launch_request: self.activate_selected_in_viewport(viewport_width, viewport_height),
            },
        }
    }

    fn matches_category(&self, app: &LauncherApp) -> bool {
        match self.category {
            LauncherCategory::Favorites => app.favorite,
            LauncherCategory::AllApplications => true,
            category => app.category == category,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub id: u64,
    pub source: String,
    pub title: String,
    pub body: String,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NotificationUpdate {
    pub redraw_requested: bool,
    pub visibility_changed: bool,
    pub active_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationCenter {
    active: Option<Notification>,
    queued: VecDeque<Notification>,
    next_id: u64,
}

impl Default for NotificationCenter {
    fn default() -> Self {
        Self {
            active: None,
            queued: VecDeque::new(),
            next_id: 1,
        }
    }
}

impl NotificationCenter {
    pub fn active(&self) -> Option<&Notification> {
        self.active.as_ref()
    }

    pub fn queued_count(&self) -> usize {
        self.queued.len()
    }

    pub fn post(
        &mut self,
        now_ms: u64,
        source: &str,
        title: &str,
        body: &str,
        timeout_ms: u64,
    ) -> NotificationUpdate {
        let notification = Notification {
            id: self.next_id,
            source: bounded_notification_text(source, 32),
            title: bounded_notification_text(title, 64),
            body: bounded_notification_text(body, 160),
            created_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(timeout_ms.max(1)),
        };
        self.next_id = self.next_id.saturating_add(1);
        if self.active.is_none() {
            self.active = Some(notification);
            NotificationUpdate {
                redraw_requested: true,
                visibility_changed: true,
                active_changed: true,
            }
        } else {
            if self.queued.len() == NOTIFICATION_QUEUE_LIMIT {
                self.queued.pop_front();
            }
            self.queued.push_back(notification);
            NotificationUpdate::default()
        }
    }

    pub fn dismiss(&mut self, now_ms: u64) -> NotificationUpdate {
        if self.active.is_none() {
            return NotificationUpdate::default();
        }
        let was_visible = self.active.is_some();
        self.active = self.queued.pop_front().map(|mut notification| {
            let lifetime = notification
                .expires_at_ms
                .saturating_sub(notification.created_at_ms);
            notification.created_at_ms = now_ms;
            notification.expires_at_ms = now_ms.saturating_add(lifetime);
            notification
        });
        NotificationUpdate {
            redraw_requested: true,
            visibility_changed: was_visible != self.active.is_some(),
            active_changed: true,
        }
    }

    pub fn tick(&mut self, now_ms: u64) -> NotificationUpdate {
        if self
            .active
            .as_ref()
            .is_some_and(|notification| now_ms >= notification.expires_at_ms)
        {
            self.dismiss(now_ms)
        } else {
            NotificationUpdate::default()
        }
    }
}

fn bounded_notification_text(value: &str, limit: usize) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .take(limit)
        .collect::<String>()
        .trim()
        .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Logout,
    Restart,
    Shutdown,
    Recovery,
}

impl SessionAction {
    pub const ALL: [Self; 4] = [Self::Logout, Self::Restart, Self::Shutdown, Self::Recovery];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Logout => "logout",
            Self::Restart => "restart",
            Self::Shutdown => "shutdown",
            Self::Recovery => "recovery",
        }
    }
}

pub const SESSION_MENU_RUNTIME_WIDTH: u32 = 512;
pub const SESSION_MENU_RUNTIME_HEIGHT: u32 = 293;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionMenuEvent {
    Toggle,
    Dismiss,
    Navigate(MenuNavigationKey),
    Activate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMenuUpdate {
    pub redraw_requested: bool,
    pub visibility_changed: bool,
    pub confirmation_changed: bool,
    pub action_request: Option<SessionAction>,
}

impl SessionMenuUpdate {
    fn unchanged() -> Self {
        Self {
            redraw_requested: false,
            visibility_changed: false,
            confirmation_changed: false,
            action_request: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionMenuState {
    open: bool,
    selected_index: usize,
    confirmation: Option<SessionAction>,
}

impl SessionMenuState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_action(&self) -> SessionAction {
        SessionAction::ALL[self.selected_index]
    }

    pub fn confirmation(&self) -> Option<SessionAction> {
        self.confirmation
    }

    pub fn menu_layout(&self, width: u32, height: u32) -> Menu<'static> {
        let high_resolution = width >= 480 || height >= 280;
        let outer_padding = if high_resolution { 22 } else { 12 };
        let row_start = if high_resolution { 62 } else { 40 };
        let row_stride = if high_resolution { 45 } else { 34 };
        let row_height = if high_resolution { 38 } else { 30 };
        Menu::new(
            Rect {
                x: outer_padding,
                y: 0,
                width: width.saturating_sub(outer_padding * 2),
                height,
            },
            "Aqua Session",
            SessionAction::ALL.len(),
            self.selected_index,
            row_start,
            row_height,
            row_stride - row_height,
        )
    }

    pub fn confirmation_dialog(
        &self,
        width: u32,
        height: u32,
    ) -> Option<ConfirmationDialog<'static>> {
        self.confirmation?;
        let high_resolution = width >= 480 || height >= 280;
        let outer_padding = if high_resolution { 22 } else { 12 };
        let footer_y = height.saturating_sub(if high_resolution { 30 } else { 24 });
        Some(ConfirmationDialog::new(
            Rect {
                x: outer_padding,
                y: footer_y.saturating_sub(5),
                width: width.saturating_sub(outer_padding * 2),
                height: if high_resolution { 23 } else { 19 },
            },
            "Session action confirmation",
            ("Enter again to confirm", ""),
            ConfirmationPresentation::Inline,
            ConfirmationSeverity::Destructive,
            ConfirmationRequirement::RepeatActivation,
            ConfirmationState::Armed,
        ))
    }

    pub fn close(&mut self) {
        self.open = false;
        self.selected_index = 0;
        self.confirmation = None;
    }

    pub fn handle_pointer(&mut self, width: u32, height: u32, x: u32, y: u32) -> SessionMenuUpdate {
        if !self.open {
            return SessionMenuUpdate::unchanged();
        }
        let Some(index) = self.menu_layout(width, height).item_at(x, y) else {
            return SessionMenuUpdate::unchanged();
        };
        let cleared_confirmation = if self.selected_index != index {
            self.selected_index = index;
            self.confirmation.take().is_some()
        } else {
            false
        };
        let mut update = self.handle_event(SessionMenuEvent::Activate);
        update.confirmation_changed |= cleared_confirmation;
        update
    }

    pub fn handle_event(&mut self, event: SessionMenuEvent) -> SessionMenuUpdate {
        match event {
            SessionMenuEvent::Toggle => {
                if self.open {
                    self.close();
                } else {
                    self.open = true;
                }
                SessionMenuUpdate {
                    redraw_requested: true,
                    visibility_changed: true,
                    confirmation_changed: false,
                    action_request: None,
                }
            }
            SessionMenuEvent::Dismiss => {
                if !self.open {
                    return SessionMenuUpdate::unchanged();
                }
                self.close();
                SessionMenuUpdate {
                    redraw_requested: true,
                    visibility_changed: true,
                    confirmation_changed: false,
                    action_request: None,
                }
            }
            SessionMenuEvent::Navigate(key) => {
                if !self.open {
                    return SessionMenuUpdate::unchanged();
                }
                let Some(selected_index) = self
                    .menu_layout(SESSION_MENU_RUNTIME_WIDTH, SESSION_MENU_RUNTIME_HEIGHT)
                    .keyboard_target(key)
                else {
                    return SessionMenuUpdate::unchanged();
                };
                if selected_index == self.selected_index {
                    return SessionMenuUpdate::unchanged();
                }
                self.selected_index = selected_index;
                let confirmation_changed = self.confirmation.take().is_some();
                SessionMenuUpdate {
                    redraw_requested: true,
                    visibility_changed: false,
                    confirmation_changed,
                    action_request: None,
                }
            }
            SessionMenuEvent::Activate => {
                if !self.open {
                    return SessionMenuUpdate::unchanged();
                }
                let selected = self.selected_action();
                if self.confirmation == Some(selected) {
                    self.close();
                    SessionMenuUpdate {
                        redraw_requested: true,
                        visibility_changed: true,
                        confirmation_changed: true,
                        action_request: Some(selected),
                    }
                } else {
                    self.confirmation = Some(selected);
                    SessionMenuUpdate {
                        redraw_requested: true,
                        visibility_changed: false,
                        confirmation_changed: true,
                        action_request: None,
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherProbe {
    pub status: &'static str,
    pub design_era: &'static str,
    pub material: &'static str,
    pub category_count: usize,
    pub app_count: usize,
    pub favorites_count: usize,
    pub search_result_count: usize,
    pub selected_app_id: Option<&'static str>,
    pub launch_request: Option<LaunchRequest>,
    pub keyboard_wrap_ready: bool,
    pub closed_activation_blocked: bool,
}

impl LauncherProbe {
    pub fn is_ready(&self) -> bool {
        self.status == LAUNCHER_STATUS
            && self.design_era == LAUNCHER_DESIGN_ERA
            && self.material == LAUNCHER_MATERIAL
            && self.category_count == LauncherCategory::ALL.len()
            && self.app_count == DEFAULT_APPS.len()
            && self.favorites_count == DEFAULT_APPS.len()
            && self.search_result_count == 1
            && self.selected_app_id == Some("settings")
            && self.launch_request
                == Some(LaunchRequest {
                    app_id: "settings",
                    command: "/usr/bin/aqua-settings",
                    target: None,
                })
            && self.keyboard_wrap_ready
            && self.closed_activation_blocked
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("launcher_status={}", self.status),
            format!("design_era={}", self.design_era),
            format!("material={}", self.material),
            format!("category_count={}", self.category_count),
            format!("app_count={}", self.app_count),
            format!("favorites_count={}", self.favorites_count),
            format!("search_result_count={}", self.search_result_count),
            format!("selected_app_id={}", self.selected_app_id.unwrap_or("none")),
            format!(
                "launch_command={}",
                self.launch_request
                    .as_ref()
                    .map(|request| request.command)
                    .unwrap_or("none")
            ),
            format!("keyboard_wrap_ready={}", self.keyboard_wrap_ready),
            format!(
                "closed_activation_blocked={}",
                self.closed_activation_blocked
            ),
        ]
    }
}

pub fn probe_launcher_model() -> LauncherProbe {
    let mut launcher = LauncherState::default();
    let closed_activation_blocked = launcher.activate_selected().is_none();
    launcher.open();

    let favorites_count = launcher.visible_apps().len();
    launcher.navigate_selection(CollectionNavigationKey::Previous);
    let keyboard_wrap_ready = launcher.selected_index() == favorites_count.saturating_sub(1);

    launcher.select_category(LauncherCategory::AllApplications);
    launcher.set_query("settings");
    let visible = launcher.visible_apps();
    let selected_app_id = visible.get(launcher.selected_index()).map(|app| app.id);
    let launch_request = launcher.activate_selected();

    LauncherProbe {
        status: LAUNCHER_STATUS,
        design_era: LAUNCHER_DESIGN_ERA,
        material: LAUNCHER_MATERIAL,
        category_count: LauncherCategory::ALL.len(),
        app_count: DEFAULT_APPS.len(),
        favorites_count,
        search_result_count: visible.len(),
        selected_app_id,
        launch_request,
        keyboard_wrap_ready,
        closed_activation_blocked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aqua_service_adapters::{
        read_network_interfaces, AudioDevice, AudioDeviceKind, NetworkLinkState,
    };

    #[derive(Debug)]
    struct RejectingAudioBackend {
        state: AudioAuthoritativeState,
        reject_submissions: bool,
    }

    impl AudioBackend for RejectingAudioBackend {
        type Error = ();

        fn authoritative_state(&mut self) -> Result<AudioAuthoritativeState, Self::Error> {
            Ok(self.state.clone())
        }

        fn submit(&mut self, _request: &AudioRequest) -> Result<(), Self::Error> {
            if self.reject_submissions {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    fn ready_audio_state(
        generation: u64,
        volume_percent: u8,
        muted: bool,
    ) -> AudioAuthoritativeState {
        AudioAuthoritativeState::new(
            generation,
            AudioServiceHealth::Ready,
            vec![
                AudioDevice::new("sink.1", "Aqua Test Output", AudioDeviceKind::Output)
                    .expect("output fixture"),
                AudioDevice::new("source.1", "Aqua Test Input", AudioDeviceKind::Input)
                    .expect("input fixture"),
            ],
            Some("sink.1".to_string()),
            Some("source.1".to_string()),
            volume_percent,
            muted,
        )
        .expect("authoritative audio fixture")
    }

    fn close(left: f32, right: f32) {
        assert!((left - right).abs() < 0.0001, "{left} != {right}");
    }

    #[test]
    fn semantic_motion_tokens_match_the_published_contract() {
        assert_eq!(SemanticMotion::Feedback.duration_ms(), 90);
        assert_eq!(SemanticMotion::Menu.duration_ms(), 140);
        assert_eq!(SemanticMotion::Panel.duration_ms(), 200);
        assert_eq!(SemanticMotion::Workspace.duration_ms(), 280);
        assert_eq!(
            MotionEasing::Standard.control_points(),
            [0.2, 0.0, 0.0, 1.0]
        );
        assert_eq!(MotionEasing::Enter.control_points(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(MotionEasing::Exit.control_points(), [0.3, 0.0, 1.0, 1.0]);
        assert!(SemanticMotion::Attention.repeats());
        assert!(!SemanticMotion::Progress.repeats());
        assert!(SemanticMotion::Attention.repeats_allowed(false, true));
        assert!(!SemanticMotion::Attention.repeats_allowed(false, false));
        assert!(!SemanticMotion::Attention.repeats_allowed(true, true));
    }

    #[test]
    fn motion_is_frame_time_driven_across_refresh_rates() {
        let sample_at = |step_ms: u64| {
            let mut motion = MotionValue::new(0.0);
            motion.retarget(1_000, 1.0, SemanticMotion::Panel, false);
            let mut now = 1_000;
            while now < 1_100 {
                now = (now + step_ms).min(1_100);
                motion.sample(now);
            }
            motion.rendered()
        };
        let at_60_hz = sample_at(17);
        let at_90_hz = sample_at(11);
        let at_120_hz = sample_at(8);
        let at_144_hz = sample_at(7);
        close(at_60_hz, at_90_hz);
        close(at_60_hz, at_120_hz);
        close(at_60_hz, at_144_hz);
        assert!(at_60_hz > 0.8 && at_60_hz < 1.0);

        let mut motion = MotionValue::new(0.0);
        motion.retarget(1_000, 1.0, SemanticMotion::Panel, false);
        close(motion.sample(1_000).value, 0.0);
        assert_eq!(motion.sample(1_200).value, 1.0);
        assert!(!motion.is_active());
    }

    #[test]
    fn interruption_and_reversal_start_from_the_rendered_value() {
        let mut motion = MotionValue::new(0.0);
        motion.retarget(0, 1.0, SemanticMotion::Panel, false);
        let interrupted = motion.sample(80).value;
        assert!(interrupted > 0.0 && interrupted < 1.0);
        let retargeted = motion.retarget(80, 0.0, SemanticMotion::Panel, false);
        close(retargeted.value, interrupted);
        close(motion.sample(80).value, interrupted);
        let reversing = motion.sample(120).value;
        assert!(reversing < interrupted);

        let second_interruption = motion.sample(140).value;
        let reversed_again = motion.retarget(140, 1.0, SemanticMotion::Panel, false);
        close(reversed_again.value, second_interruption);
        assert_eq!(motion.sample(340).value, 1.0);
    }

    #[test]
    fn reduced_motion_removes_spatial_travel_without_stuck_frames() {
        let mut controller = ShellMotionController::default();
        assert!(controller.set_reduced_motion(0, true));
        controller.set_visible(ShellMotionSurface::Launcher, true, 10);
        controller.set_visible(ShellMotionSurface::SessionMenu, true, 10);
        controller.set_visible(ShellMotionSurface::Notification, true, 10);
        let frame = controller.sample(10);
        assert_eq!(frame.launcher.opacity, 1.0);
        assert_eq!(frame.launcher.offset_y, 0);
        assert_eq!(frame.session_menu.offset_y, 0);
        assert_eq!(frame.notification.offset_x, 0);
        assert!(!frame.is_active());

        controller.set_visible(ShellMotionSurface::Launcher, false, 11);
        let hidden = controller.sample(11);
        assert_eq!(hidden.launcher.opacity, 0.0);
        assert!(!controller.is_active());
    }

    #[test]
    fn enabling_reduced_motion_finishes_an_active_transition_immediately() {
        let mut controller = ShellMotionController::default();
        controller.set_visible(ShellMotionSurface::Notification, true, 0);
        let partial = controller.sample(60).notification;
        assert!(partial.active);
        assert!(partial.offset_x > 0);
        assert!(controller.set_reduced_motion(60, true));
        let settled = controller.sample(60).notification;
        assert_eq!(settled.opacity, 1.0);
        assert_eq!(settled.offset_x, 0);
        assert!(!settled.active);
    }

    #[test]
    fn system_overview_reads_bounded_real_metrics_from_proc_contract() {
        let root = std::env::temp_dir().join(format!("aqua-overview-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("proc/sys/kernel")).expect("create proc fixture");
        fs::create_dir_all(root.join("etc")).expect("create etc fixture");
        fs::write(root.join("proc/uptime"), "90061.42 100.00\n").expect("write uptime");
        fs::write(root.join("proc/loadavg"), "1.25 0.80 0.50 1/10 42\n").expect("write loadavg");
        fs::write(
            root.join("proc/meminfo"),
            "MemTotal:       1000000 kB\nMemAvailable:    625000 kB\n",
        )
        .expect("write meminfo");
        fs::write(root.join("proc/sys/kernel/osrelease"), "6.6.32-aqua\n").expect("write kernel");
        fs::write(root.join("etc/hostname"), "aqua-linux\n").expect("write hostname");
        fs::write(
            root.join("etc/os-release"),
            "NAME=Aqua Linux\nPRETTY_NAME=\"Aqua Linux Development\"\n",
        )
        .expect("write os-release");

        let model = SystemOverviewModel::read(&root, 10 * 3_600 + 30 * 60)
            .expect("overview fixture should parse");
        assert_eq!(model.clock_utc, "10:30 UTC");
        assert_eq!(model.os_name, "Aqua Linux Development");
        assert_eq!(model.hostname, "aqua-linux");
        assert_eq!(model.kernel, "6.6.32-aqua");
        assert_eq!(model.uptime_label(), "1d 1h");
        assert_eq!(model.load_average_x100, 125);
        assert_eq!(model.memory_used_percent(), 37);
        fs::remove_dir_all(root).expect("remove overview fixture");
    }

    #[test]
    fn top_bar_reads_clock_network_battery_and_audio_from_system_contract() {
        let root = std::env::temp_dir().join(format!("aqua-top-bar-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("sys/class/net/eth0")).expect("network fixture");
        fs::create_dir(root.join("sys/class/net/eth0/device")).expect("network device fixture");
        fs::create_dir_all(root.join("proc/net")).expect("route directory fixture");
        fs::create_dir_all(root.join("etc")).expect("resolver directory fixture");
        fs::create_dir_all(root.join("sys/class/power_supply/AC")).expect("adapter fixture");
        fs::create_dir_all(root.join("sys/class/power_supply/BAT0")).expect("battery fixture");
        fs::create_dir_all(root.join("dev/snd")).expect("audio fixture");
        fs::write(root.join("sys/class/net/eth0/operstate"), "up\n").expect("network state");
        fs::write(
            root.join("proc/net/route"),
            "Iface Destination Gateway Flags\neth0 00000000 0100000A 0003\n",
        )
        .expect("route state");
        fs::write(root.join("etc/resolv.conf"), "nameserver 1.1.1.1\n").expect("resolver state");
        fs::write(root.join("sys/class/power_supply/AC/type"), "Mains\n").expect("adapter type");
        fs::write(root.join("sys/class/power_supply/BAT0/type"), "Battery\n")
            .expect("battery type");
        fs::write(root.join("sys/class/power_supply/BAT0/capacity"), "87\n")
            .expect("battery capacity");

        let state = TopBarState::read(&root, 0);
        assert_eq!(state.product_label, "Aqua Linux");
        assert_eq!(state.clock_label, "Thu, 01 Jan 1970  00:00 UTC");
        assert!(state.network_connected);
        assert_eq!(state.battery_percent, Some(87));
        assert!(state.audio_available);
        let bar = top_system_bar(1536, 36);
        assert!(bar.is_valid());
        assert_eq!(
            bar.status_rect(aqua_components::TopSystemStatus::Audio).x,
            1400
        );
        assert!(bar.session_hit(1535, 18));

        fs::remove_dir_all(root).expect("remove top bar fixture");
    }

    #[test]
    fn desktop_icons_select_activate_and_open_context_menu() {
        let geometry = desktop_context_menu(1).expect("bounded icon menu should exist");
        assert!(geometry.is_valid());
        assert_eq!(geometry.item_rect(0).y, 136);
        assert_eq!(geometry.item_rect(1).y, 172);
        assert!(desktop_context_menu(DESKTOP_ICONS.len()).is_none());

        let mut state = DesktopIconState::default();
        let first = state.pointer_press(48, 90, DesktopPointerButton::Primary, 1_000);
        assert!(first.redraw_requested);
        assert_eq!(state.selected(), Some(0));
        assert!(first.launch_request.is_none());

        let second = state.pointer_press(48, 90, DesktopPointerButton::Primary, 1_300);
        assert_eq!(
            second.launch_request.map(|request| request.app_id),
            Some("files")
        );

        let context = state.pointer_press(48, 194, DesktopPointerButton::Secondary, 2_000);
        assert_eq!(state.selected(), Some(1));
        assert_eq!(state.context_menu(), Some(1));
        assert_eq!(
            context.context_action,
            Some(DesktopContextAction::MenuOpened)
        );

        let properties = state.pointer_press(150, 245, DesktopPointerButton::Primary, 2_100);
        assert_eq!(
            properties.context_action,
            Some(DesktopContextAction::Properties("settings"))
        );
        assert_eq!(state.context_menu(), None);

        state.pointer_press(48, 298, DesktopPointerButton::Secondary, 2_200);
        let request = state.pointer_press(150, 345, DesktopPointerButton::Primary, 2_300);
        assert_eq!(
            request.context_action,
            Some(DesktopContextAction::TrashEmptyConfirmationRequested)
        );
        assert!(state.trash_empty_confirmation());
        let trash_row = desktop_context_menu(2).unwrap().item_rect(1);
        let trash_dialog = state
            .trash_confirmation_dialog(trash_row)
            .expect("armed Trash action should expose shared confirmation");
        assert!(trash_dialog.is_valid());
        assert!(trash_dialog.is_compact());
        assert_eq!(
            trash_dialog.requirement,
            ConfirmationRequirement::RepeatActivation
        );
        let confirmed = state.pointer_press(150, 345, DesktopPointerButton::Primary, 2_400);
        assert_eq!(
            confirmed.context_action,
            Some(DesktopContextAction::TrashEmptyConfirmed)
        );
        assert!(!state.trash_empty_confirmation());

        let cleared = state.pointer_press(400, 700, DesktopPointerButton::Primary, 3_000);
        assert!(cleared.redraw_requested);
        assert_eq!(state.selected(), None);
        assert_eq!(state.context_menu(), None);
    }

    #[test]
    fn desktop_context_menu_keyboard_uses_shared_rows_and_confirmation_gate() {
        let mut state = DesktopIconState::default();
        let closed = state.handle_context_menu_key(DesktopContextMenuKey::Activate);
        assert!(!closed.redraw_requested);

        state.pointer_press(48, 194, DesktopPointerButton::Secondary, 1_000);
        assert_eq!(state.context_menu_selected_row(), Some(0));
        let next =
            state.handle_context_menu_key(DesktopContextMenuKey::Navigate(MenuNavigationKey::Next));
        assert!(next.redraw_requested);
        assert_eq!(state.context_menu_selected_row(), Some(1));
        let properties = state.handle_context_menu_key(DesktopContextMenuKey::Activate);
        assert_eq!(
            properties.context_action,
            Some(DesktopContextAction::Properties("settings"))
        );
        assert_eq!(state.context_menu(), None);

        state.pointer_press(48, 298, DesktopPointerButton::Secondary, 2_000);
        state.handle_context_menu_key(DesktopContextMenuKey::Navigate(MenuNavigationKey::End));
        let armed = state.handle_context_menu_key(DesktopContextMenuKey::Activate);
        assert_eq!(
            armed.context_action,
            Some(DesktopContextAction::TrashEmptyConfirmationRequested)
        );
        assert!(state.trash_empty_confirmation());

        let home =
            state.handle_context_menu_key(DesktopContextMenuKey::Navigate(MenuNavigationKey::Home));
        assert!(home.redraw_requested);
        assert!(!state.trash_empty_confirmation());
        assert_eq!(state.context_menu_selected_row(), Some(0));

        state.handle_context_menu_key(DesktopContextMenuKey::Navigate(MenuNavigationKey::End));
        state.handle_context_menu_key(DesktopContextMenuKey::Activate);
        let confirmed = state.handle_context_menu_key(DesktopContextMenuKey::Activate);
        assert_eq!(
            confirmed.context_action,
            Some(DesktopContextAction::TrashEmptyConfirmed)
        );
        assert_eq!(state.context_menu(), None);

        state.pointer_press(48, 90, DesktopPointerButton::Secondary, 3_000);
        let dismissed = state.handle_context_menu_key(DesktopContextMenuKey::Dismiss);
        assert!(dismissed.redraw_requested);
        assert_eq!(state.context_menu(), None);
    }

    #[test]
    fn desktop_properties_model_is_targeted_and_bounded() {
        let root = std::env::temp_dir().join(format!(
            "aqua-properties-model-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let home = root.join("home/aqua");
        let system = root.join("system");
        fs::create_dir_all(home.join("Trash")).expect("create Trash fixture");
        fs::create_dir_all(system.join("usr/bin")).expect("create system fixture");
        fs::write(system.join("usr/bin/aqua-settings"), b"settings")
            .expect("create Settings fixture");
        fs::write(home.join("note.txt"), b"note").expect("create home item");

        let files =
            DesktopPropertiesModel::load("files", &home, &system).expect("load Files properties");
        assert_eq!(files.name, "Files");
        assert_eq!(files.kind, "Folder");
        assert_eq!(files.item_count, Some(2));
        assert!(!files.enumeration_capped);
        assert_eq!(files.primary_action().label(), "Refresh Contents");
        let details = files.details_section_group(480, 300);
        assert!(details.is_valid());
        assert_eq!(details.row_rect(0).y, 192);
        assert_eq!(details.footer_trailing_rect(138, 30).x, 302);
        let location = files.details_metadata_row(480, 300, 0, "Location", &files.location);
        assert!(location.is_valid());
        assert_eq!(location.slots().label.width, 80);
        assert_eq!(location.slots().value.x, 128);
        assert_eq!(location.accessibility().role, "definition");

        let mut settings = DesktopPropertiesModel::load("settings", &home, &system)
            .expect("load Settings properties");
        assert_eq!(settings.kind, "Application");
        assert_eq!(settings.status, "Available");
        assert_eq!(settings.item_count, None);
        assert_eq!(settings.primary_action().label(), "Verify Application");
        fs::remove_file(system.join("usr/bin/aqua-settings")).expect("remove Settings fixture");
        assert_eq!(
            settings
                .refresh(&home, &system)
                .expect("refresh Settings properties"),
            DesktopPropertiesAction::VerifyApplication
        );
        assert_eq!(settings.status, "Not found");
        assert_eq!(settings.refresh_generation, 1);
        assert!(DesktopPropertiesModel::load("unknown", &home, &system).is_err());
        assert_eq!(
            properties_launch_request("trash"),
            Some(LaunchRequest {
                app_id: "properties",
                command: "/usr/bin/aqua-properties",
                target: Some("trash"),
            })
        );
        assert_eq!(properties_launch_request("unknown"), None);
        fs::remove_dir_all(root).expect("remove properties fixture");
    }

    #[test]
    fn trash_model_lists_and_empties_only_its_bounded_root() {
        let root =
            std::env::temp_dir().join(format!("aqua-trash-test-{}-{}", std::process::id(), 1));
        let outside = root.with_extension("outside");
        fs::create_dir_all(root.join("folder")).expect("create trash fixture");
        fs::write(root.join("note.txt"), "discard").expect("write trash file");
        fs::write(&outside, "keep").expect("write outside file");

        let mut trash = TrashModel::open(&root).expect("open trash model");
        assert_eq!(trash.entries().len(), 2);
        assert_eq!(trash.empty().expect("empty trash"), 2);
        assert!(trash.entries().is_empty());
        assert!(fs::read_dir(&root)
            .expect("read empty trash")
            .next()
            .is_none());
        assert_eq!(fs::read_to_string(&outside).expect("read outside"), "keep");

        fs::remove_dir_all(root).expect("remove trash fixture");
        fs::remove_file(outside).expect("remove outside fixture");
    }

    #[cfg(unix)]
    #[test]
    fn trash_model_rejects_symlink_root() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!("aqua-trash-link-{}", std::process::id()));
        let target = root.with_extension("target");
        fs::create_dir_all(&target).expect("create target");
        symlink(&target, &root).expect("create trash root symlink");
        assert_eq!(
            TrashModel::open(&root)
                .expect_err("symlink root must fail")
                .kind(),
            io::ErrorKind::InvalidInput
        );
        fs::remove_file(root).expect("remove root symlink");
        fs::remove_dir_all(target).expect("remove target");
    }

    #[test]
    fn settings_model_selects_categories_and_toggles_reduced_motion() {
        let mut model = SettingsWindowModel::default();
        let appearance = model.section_group();
        assert!(appearance.is_valid());
        assert_eq!(appearance.heading_rect().x, 218);
        assert_eq!(appearance.row_rect(1), model.theme_segmented_control().rect);
        assert_eq!(model.active_switch().unwrap().rect.x, 482);
        assert_eq!(
            model.handle_pointer(500, 150),
            SettingsUpdate::ReducedMotionChanged(true)
        );
        assert!(model.reduced_motion);
        assert_eq!(model.handle_pointer(440, 150), SettingsUpdate::None);
        assert_eq!(
            model.handle_key(SettingsKey::Activate),
            SettingsUpdate::ReducedMotionChanged(false)
        );
        assert!(model.keyboard_focus);
        assert_eq!(
            model.handle_key(SettingsKey::Home),
            SettingsUpdate::CategorySelected(0)
        );
        assert_eq!(
            model.handle_key(SettingsKey::Decrease),
            SettingsUpdate::ThemeChanged(AquaTheme::Nightmare)
        );
        assert_eq!(model.theme, AquaTheme::Nightmare);
        assert_eq!(
            model.handle_key(SettingsKey::Increase),
            SettingsUpdate::ThemeChanged(AquaTheme::LightWhite)
        );
        assert_eq!(model.theme, AquaTheme::LightWhite);
        assert_eq!(
            model.handle_key(SettingsKey::Down),
            SettingsUpdate::CategorySelected(1)
        );
        assert_eq!(
            model.handle_key(SettingsKey::Activate),
            SettingsUpdate::DesktopIconsChanged(false)
        );
        assert!(!model.desktop_icons);
        assert_eq!(
            model.handle_key(SettingsKey::Down),
            SettingsUpdate::CategorySelected(2)
        );
        assert_eq!(
            model.handle_key(SettingsKey::Activate),
            SettingsUpdate::KeyRepeatChanged(false)
        );
        assert!(!model.key_repeat);
        assert_eq!(
            model.handle_key(SettingsKey::End),
            SettingsUpdate::CategorySelected(5)
        );
        assert_eq!(
            model.handle_key(SettingsKey::Down),
            SettingsUpdate::CategorySelected(0)
        );
        assert_eq!(
            model.handle_key(SettingsKey::Up),
            SettingsUpdate::CategorySelected(5)
        );
        assert_eq!(
            model.handle_key(SettingsKey::Home),
            SettingsUpdate::CategorySelected(0)
        );
        assert_eq!(
            model.handle_pointer(40, 300),
            SettingsUpdate::CategorySelected(4)
        );
        model
            .reconcile_audio_state(ready_audio_state(1, 70, false))
            .expect("ready audio state");
        let slider = model.audio_slider();
        assert_eq!(
            model.handle_pointer(slider.rect.right() - 1, slider.rect.y),
            SettingsUpdate::AudioVolumeChanged(100)
        );
        assert_eq!(model.audio.volume_percent(), 100);
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applying);
        assert!(!model.audio.controls_enabled());
        assert_eq!(
            model.handle_key(SettingsKey::Decrease),
            SettingsUpdate::None
        );
        model
            .reconcile_audio_state(ready_audio_state(2, 100, false))
            .expect("volume acknowledgement");
        assert_eq!(
            model.handle_key(SettingsKey::Decrease),
            SettingsUpdate::AudioVolumeChanged(95)
        );
        assert_eq!(model.audio.volume_percent(), 95);
        model
            .reconcile_audio_state(ready_audio_state(3, 95, false))
            .expect("decreased volume acknowledgement");
        assert_eq!(
            model.handle_key(SettingsKey::Activate),
            SettingsUpdate::AudioMutedChanged(true)
        );
        assert!(model.audio.muted());
        assert!(model.handle_hover(40, 100));
        assert_eq!(model.hovered_category, Some(0));
        assert_eq!(model.handle_pointer(40, 138), SettingsUpdate::None);
        assert!(model.handle_hover(40, 138));
        assert_eq!(model.hovered_category, None);
    }

    #[cfg(unix)]
    #[test]
    fn settings_wifi_control_is_broker_gated_and_drives_typed_actions() {
        use std::io::{Read as _, Write as _};
        use std::os::unix::net::UnixListener;
        use std::thread;

        let socket = PathBuf::from(format!(
            "/tmp/aqua-settings-wifi-{}-{}.sock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let _ = fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).expect("bind broker fixture");
        let server = thread::spawn(move || {
            let exchanges = [
                (
                    "AQUA-NETWORK/1 WIFI_STATUS wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-status interface=wlan0 state=completed network_id=7 authoritative=true credential_saved=true\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_DISCONNECT wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-disconnect interface=wlan0 authoritative=true credential_saved=true\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_RECONNECT wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-reconnect interface=wlan0 network_id=8 authoritative=true credential_saved=true\n",
                ),
            ];
            for (expected, response) in exchanges {
                let (mut stream, _) = listener.accept().expect("accept Settings request");
                let mut request = String::new();
                stream
                    .read_to_string(&mut request)
                    .expect("read Settings request");
                assert_eq!(request, expected);
                stream
                    .write_all(response.as_bytes())
                    .expect("write broker response");
            }
        });

        let mut model = SettingsWindowModel::default();
        assert!(!model.wifi.controls_enabled());
        assert!(model.refresh_wifi_control(&socket));
        assert!(model.wifi.connected());
        model.selected_category = 3;
        let wifi_switch = model.active_switch().expect("Wi-Fi switch");
        assert!(wifi_switch.checked);
        assert!(wifi_switch.is_valid());
        assert!(wifi_switch.pointer_toggles(wifi_switch.rect.x, wifi_switch.rect.y));
        assert_eq!(
            model.handle_key(SettingsKey::Activate),
            SettingsUpdate::WifiControlRequested(false)
        );
        assert!(model.apply_wifi_control(&socket, false));
        assert!(!model.wifi.connected());
        assert_eq!(
            model.handle_key(SettingsKey::Activate),
            SettingsUpdate::WifiControlRequested(true)
        );
        assert!(model.apply_wifi_control(&socket, true));
        assert!(model.wifi.connected());

        server.join().expect("join broker fixture");
        fs::remove_file(socket).expect("remove broker fixture");
    }

    #[cfg(unix)]
    #[test]
    fn settings_wifi_discovery_and_secret_entry_are_bounded_and_redacted() {
        use std::io::{Read as _, Write as _};
        use std::os::unix::net::UnixListener;
        use std::thread;

        let socket = PathBuf::from(format!(
            "/tmp/aqua-settings-wifi-discovery-{}-{}.sock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let _ = fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).expect("bind broker fixture");
        let server = thread::spawn(move || {
            let exchanges = [
                (
                    "AQUA-NETWORK/1 WIFI_STATUS wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-status interface=wlan0 state=disconnected network_id=none authoritative=false credential_saved=false\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_SCAN wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-scan interface=wlan0 count=1 authoritative=true network_0=417175612d51454d55,-28,wpa3-personal\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_CONNECT wlan0 wpa3-personal 417175612d51454d55 70617373776f7264\n",
                    "AQUA-NETWORK/1 OK operation=wifi-connect interface=wlan0 security=wpa3-personal network_id=9 authoritative=true credential_saved=false\n",
                ),
            ];
            for (expected, response) in exchanges {
                let (mut stream, _) = listener.accept().expect("accept Settings request");
                let mut request = String::new();
                stream
                    .read_to_string(&mut request)
                    .expect("read Settings request");
                assert_eq!(request, expected);
                stream
                    .write_all(response.as_bytes())
                    .expect("write broker response");
            }
        });

        let mut model = SettingsWindowModel::default();
        assert!(model.refresh_wifi_control(&socket));
        assert!(model.refresh_wifi_networks(&socket));
        assert_eq!(model.wifi.networks().len(), 1);
        model.selected_category = 3;
        let network_row = model.wifi_network_row(0).expect("shared network row");
        let row = model.section_group().row_rect(1);
        assert_eq!(network_row.rect, row);
        assert_eq!(network_row.accessibility().role, "option");
        assert_eq!(network_row.accessibility().name, "Aqua-QEMU");
        assert!(!network_row.accessibility().disabled);
        assert_eq!(network_row.slots().trailing.width, 148);
        assert_eq!(
            model.handle_pointer(row.x + 1, row.y + 1),
            SettingsUpdate::WifiNetworkSelected(0)
        );
        for key in [
            SettingsKey::Home,
            SettingsKey::End,
            SettingsKey::Up,
            SettingsKey::Down,
            SettingsKey::Decrease,
            SettingsKey::Increase,
        ] {
            assert_eq!(model.handle_key(key), SettingsUpdate::None);
            assert_eq!(model.selected_category, 3);
        }
        for character in "password".chars() {
            assert!(model.input_wifi_passphrase(character));
        }
        assert_eq!(model.wifi.masked_passphrase(), "********");
        assert!(!format!("{model:?}").contains("password"));
        assert_eq!(
            model.handle_key(SettingsKey::Activate),
            SettingsUpdate::WifiConnectRequested
        );
        assert!(model.apply_wifi_connection(&socket));
        assert!(model.wifi.connected());
        assert!(!model.wifi.credential_saved());
        assert!(!model.wifi.credential_entry());

        server.join().expect("join broker fixture");
        fs::remove_file(socket).expect("remove broker fixture");
    }

    #[cfg(unix)]
    #[test]
    fn settings_wifi_rescan_forget_and_retry_flow_is_bounded() {
        use std::io::{Read as _, Write as _};
        use std::os::unix::net::UnixListener;
        use std::thread;

        let socket = PathBuf::from(format!(
            "/tmp/aqua-settings-wifi-actions-{}-{}.sock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let _ = fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).expect("bind broker fixture");
        let server = thread::spawn(move || {
            let exchanges = [
                (
                    "AQUA-NETWORK/1 WIFI_STATUS wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-status interface=wlan0 state=disconnected network_id=none authoritative=false credential_saved=true\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_SCAN wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-scan interface=wlan0 count=1 authoritative=true network_0=417175612d51454d55,-28,wpa2-personal\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_SCAN wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-scan interface=wlan0 count=1 authoritative=true network_0=417175612d51454d55,-24,wpa2-personal\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_SCAN wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-scan interface=wlan0 count=1 authoritative=true network_0=417175612d51454d55,-22,wpa2-personal\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_CONNECT wlan0 wpa2-personal 417175612d51454d55 70617373776f7264\n",
                    "AQUA-NETWORK/1 ERROR wifi-control-timeout\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_CONNECT wlan0 wpa2-personal 417175612d51454d55 70617373776f7264\n",
                    "AQUA-NETWORK/1 ERROR wifi-control-timeout\n",
                ),
                (
                    "AQUA-NETWORK/1 WIFI_FORGET wlan0\n",
                    "AQUA-NETWORK/1 OK operation=wifi-forget interface=wlan0 authoritative=true credential_saved=false\n",
                ),
            ];
            for (expected, response) in exchanges {
                let (mut stream, _) = listener.accept().expect("accept Settings request");
                let mut request = String::new();
                stream.read_to_string(&mut request).expect("read request");
                assert_eq!(request, expected);
                stream
                    .write_all(response.as_bytes())
                    .expect("write response");
            }
        });

        let mut model = SettingsWindowModel {
            selected_category: 3,
            ..SettingsWindowModel::default()
        };
        assert!(model.refresh_wifi_control(&socket));
        assert!(model.wifi.credential_saved());
        assert!(model.refresh_wifi_networks(&socket));
        let rescan = model.wifi_rescan_button();
        let forget = model.wifi_forget_button();
        let action_row = model.section_group().row_rect(3);
        assert_eq!(rescan.rect.x, action_row.x);
        assert_eq!(forget.rect.right(), action_row.right());
        assert_eq!(forget.rect.x.saturating_sub(rescan.rect.right()), 6);
        assert_eq!(rescan.accessibility().role, "button");
        assert_eq!(forget.accessibility().name, "Forget saved");
        assert!(!forget.accessibility().disabled);
        assert_eq!(
            model.handle_pointer(rescan.rect.right(), action_row.y + 1),
            SettingsUpdate::None
        );
        assert_eq!(
            model.handle_pointer(rescan.rect.x + 1, rescan.rect.y + 1),
            SettingsUpdate::WifiScanRequested
        );
        assert!(model.refresh_wifi_networks(&socket));
        assert_eq!(
            model.handle_key(SettingsKey::Decrease),
            SettingsUpdate::WifiScanRequested
        );
        assert!(model.refresh_wifi_networks(&socket));

        let network_row = model.section_group().row_rect(1);
        assert_eq!(
            model.handle_pointer(network_row.x + 1, network_row.y + 1),
            SettingsUpdate::WifiNetworkSelected(0)
        );
        for _ in 0..MAX_WIFI_CONNECT_ATTEMPTS {
            for character in "password".chars() {
                assert!(model.input_wifi_passphrase(character));
            }
            assert!(!model.apply_wifi_connection(&socket));
            assert_eq!(model.wifi.masked_passphrase(), "");
        }
        assert!(!model.wifi.credential_entry());
        assert_eq!(model.wifi.connect_attempts_remaining(), 0);
        assert_eq!(model.wifi.status_label(), "connection-retry-limit");

        assert_eq!(
            model.handle_key(SettingsKey::Increase),
            SettingsUpdate::WifiForgetRequested
        );

        assert_eq!(
            model.handle_pointer(forget.rect.right() - 1, forget.rect.y + 1),
            SettingsUpdate::WifiForgetRequested
        );
        assert!(model.forget_saved_wifi_network(&socket));
        assert!(!model.wifi.credential_saved());

        server.join().expect("join broker fixture");
        fs::remove_file(socket).expect("remove broker fixture");
    }

    #[test]
    fn settings_wifi_shared_actions_disable_without_broker_authority() {
        let mut model = SettingsWindowModel {
            selected_category: 3,
            ..SettingsWindowModel::default()
        };
        let rescan = model.wifi_rescan_button();
        let forget = model.wifi_forget_button();

        assert_eq!(rescan.state, ComponentState::Disabled);
        assert_eq!(forget.state, ComponentState::Disabled);
        assert!(rescan.accessibility().disabled);
        assert!(forget.accessibility().disabled);
        assert!(!rescan.pointer_hit(rescan.rect.x, rescan.rect.y));
        assert!(!forget.pointer_hit(forget.rect.x, forget.rect.y));
        assert_eq!(
            model.handle_key(SettingsKey::Decrease),
            SettingsUpdate::None
        );
        assert_eq!(
            model.handle_key(SettingsKey::Increase),
            SettingsUpdate::None
        );
    }

    #[test]
    fn settings_config_persists_atomically_and_loads_strictly() {
        let root = std::env::temp_dir().join(format!(
            "aqua-settings-config-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let path = root.join("aqua/settings.conf");
        assert!(
            !SettingsWindowModel::load_or_default(&path)
                .expect("missing config should use defaults")
                .reduced_motion
        );

        let model = SettingsWindowModel {
            reduced_motion: true,
            ..SettingsWindowModel::default()
        };
        model.persist(&path).expect("settings should persist");
        assert_eq!(
            fs::read_to_string(&path).expect("persisted config"),
            "version=1\nreduced_motion=true\ndesktop_icons=true\nkey_repeat=true\ntheme=LightWhite\naudio_volume=70\naudio_muted=false\n"
        );
        let reloaded = SettingsWindowModel::load_or_default(&path).expect("settings should reload");
        assert!(reloaded.reduced_motion);
        assert!(reloaded.desktop_icons);
        assert!(reloaded.key_repeat);
        assert_eq!(reloaded.theme, AquaTheme::LightWhite);
        assert_eq!(reloaded.audio.volume_percent(), 70);
        assert!(!reloaded.audio.muted());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path)
                    .expect("config metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }

        fs::write(&path, b"version=2\nreduced_motion=true\n").expect("invalid fixture");
        assert!(matches!(
            SettingsWindowModel::load_or_default(&path),
            Err(SettingsConfigError::UnsupportedVersion)
        ));
        let legacy = SettingsWindowModel::from_config("version=1\nreduced_motion=false\n")
            .expect("version 1 config without later optional key should remain compatible");
        assert!(legacy.desktop_icons);
        assert!(legacy.key_repeat);
        assert_eq!(legacy.theme, AquaTheme::LightWhite);
        assert_eq!(legacy.audio.volume_percent(), 70);
        assert!(!legacy.audio.muted());
        fs::remove_dir_all(root).expect("remove settings fixture");
    }

    #[test]
    fn settings_theme_selection_is_bounded_and_persistent() {
        let mut model = SettingsWindowModel::default();
        assert_eq!(model.handle_pointer(301, 230), SettingsUpdate::None);
        assert_eq!(
            model.handle_pointer(410, 230),
            SettingsUpdate::ThemeChanged(AquaTheme::Deepside)
        );
        assert_eq!(model.theme, AquaTheme::Deepside);
        assert!(model.to_config().contains("theme=Deepside\n"));
        assert_eq!(
            SettingsWindowModel::from_config(&model.to_config())
                .expect("theme config")
                .theme,
            AquaTheme::Deepside
        );
        assert!(matches!(
            SettingsWindowModel::from_config("version=1\nreduced_motion=false\ntheme=Unknown\n"),
            Err(SettingsConfigError::InvalidFormat)
        ));
    }

    #[test]
    fn audio_volume_model_is_bounded_persistent_and_device_aware() {
        let mut model = SettingsWindowModel::default();
        assert!(!model.audio.available());
        assert_eq!(
            model.audio.service_health(),
            AudioServiceHealth::Unavailable
        );
        assert!(!model.audio.backend_applied());
        assert_eq!(
            model.audio.control_status(),
            AudioControlStatus::Unavailable
        );
        assert!(!model.audio.controls_enabled());
        model
            .reconcile_audio_state(ready_audio_state(1, 70, false))
            .expect("ready adapter state should enable controls");
        assert!(model.audio.available());
        assert!(model.audio.backend_applied());
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applied);
        assert!(model.audio.controls_enabled());
        assert_eq!(model.audio.output_device_name(), Some("Aqua Test Output"));
        assert!(model.audio.set_volume_percent(85));
        assert!(!model.audio.set_volume_percent(101));
        assert!(model.audio.set_muted(true));
        assert!(!model.audio.backend_applied());
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applying);
        assert!(!model.audio.controls_enabled());
        let volume_request = model
            .audio
            .next_reconciliation_request()
            .expect("valid reconciliation")
            .expect("volume request");
        assert_eq!(
            volume_request.intent(),
            &aqua_service_adapters::AudioIntent::SetOutputVolume(85)
        );
        model
            .reconcile_audio_state(ready_audio_state(2, 85, false))
            .expect("volume acknowledgement");
        let mute_request = model
            .audio
            .next_reconciliation_request()
            .expect("valid reconciliation")
            .expect("mute request");
        assert_eq!(
            mute_request.intent(),
            &aqua_service_adapters::AudioIntent::SetOutputMuted(true)
        );
        model
            .reconcile_audio_state(ready_audio_state(3, 85, true))
            .expect("mute acknowledgement");
        assert!(model.audio.backend_applied());
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applied);
        assert!(model.audio.controls_enabled());
        let restored = SettingsWindowModel::from_config(&model.to_config())
            .expect("bounded audio preference should reload");
        assert_eq!(restored.audio.volume_percent(), 85);
        assert!(restored.audio.muted());
        assert!(!restored.audio.available());
        assert!(!restored.audio.backend_applied());
        assert_eq!(
            restored.audio.control_status(),
            AudioControlStatus::Unavailable
        );
        assert!(matches!(
            SettingsWindowModel::from_config("version=1\nreduced_motion=false\naudio_volume=101\n"),
            Err(SettingsConfigError::InvalidFormat)
        ));
    }

    #[test]
    fn settings_audio_controls_follow_authoritative_acknowledgement_state() {
        let mut model = SettingsWindowModel {
            selected_category: 4,
            ..SettingsWindowModel::default()
        };
        model
            .reconcile_audio_state(
                AudioAuthoritativeState::unavailable(1, AudioServiceHealth::Starting)
                    .expect("starting state"),
            )
            .expect("starting state should reconcile");
        assert_eq!(model.audio.control_status(), AudioControlStatus::Starting);
        assert!(!model.audio.controls_enabled());

        model
            .reconcile_audio_state(ready_audio_state(2, 70, false))
            .expect("ready audio state");
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applied);
        assert!(model.audio.set_volume_percent(85));
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applying);
        assert_eq!(model.audio.authoritative_volume_percent(), Some(70));
        assert_eq!(model.audio_slider().value, 70);
        assert_eq!(
            model.handle_key(SettingsKey::Increase),
            SettingsUpdate::None
        );
        let slider = model.audio_slider();
        assert_eq!(
            model.handle_pointer(slider.rect.right() - 1, slider.rect.y),
            SettingsUpdate::None
        );
        model
            .audio
            .next_reconciliation_request()
            .expect("valid request")
            .expect("pending volume request");

        model
            .reconcile_audio_state(
                AudioAuthoritativeState::unavailable(3, AudioServiceHealth::Degraded)
                    .expect("degraded state"),
            )
            .expect("degraded state should reconcile");
        assert_eq!(model.audio.control_status(), AudioControlStatus::Degraded);
        assert_eq!(model.audio.volume_percent(), 85);
        assert!(!model.audio.controls_enabled());

        model
            .reconcile_audio_state(ready_audio_state(4, 70, false))
            .expect("replacement graph state");
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applying);
        model
            .audio
            .next_reconciliation_request()
            .expect("valid replacement request")
            .expect("replacement volume request");
        model
            .reconcile_audio_state(ready_audio_state(5, 85, false))
            .expect("replacement acknowledgement");
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applied);
        assert!(model.audio.controls_enabled());
        assert!(model.audio.backend_applied());
    }

    #[test]
    fn settings_audio_degrades_after_bounded_submission_failures_and_recovers() {
        let mut model = SettingsWindowModel {
            selected_category: 4,
            ..SettingsWindowModel::default()
        };
        let mut backend = RejectingAudioBackend {
            state: ready_audio_state(1, 40, false),
            reject_submissions: true,
        };

        for expected_attempts in 1..=aqua_service_adapters::MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS {
            assert!(model.synchronize_audio_backend(&mut backend).is_err());
            assert_eq!(model.audio.submission_attempts(), expected_attempts);
        }
        assert!(model.audio.submission_retry_exhausted());
        assert_eq!(model.audio.control_status(), AudioControlStatus::Degraded);
        assert!(!model.audio.controls_enabled());
        assert_eq!(model.audio.volume_percent(), 70);
        assert_eq!(model.audio.authoritative_volume_percent(), Some(40));
        assert_eq!(
            model
                .synchronize_audio_backend(&mut backend)
                .expect("same generation remains blocked")
                .submitted_request_id,
            None
        );

        backend.reject_submissions = false;
        backend.state = ready_audio_state(2, 40, false);
        assert_eq!(
            model
                .synchronize_audio_backend(&mut backend)
                .expect("new generation reopens submission")
                .submitted_request_id,
            Some(4)
        );
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applying);
        backend.state = ready_audio_state(3, 70, false);
        assert!(
            model
                .synchronize_audio_backend(&mut backend)
                .expect("replacement acknowledgement")
                .reconcile
                .request_confirmed
        );
        assert_eq!(model.audio.control_status(), AudioControlStatus::Applied);
        assert!(model.audio.controls_enabled());
    }

    #[cfg(unix)]
    #[test]
    fn settings_config_rejects_symlink_targets() {
        let root =
            std::env::temp_dir().join(format!("aqua-settings-symlink-{}", std::process::id()));
        fs::create_dir_all(&root).expect("settings symlink fixture");
        let target = root.join("target.conf");
        let path = root.join("settings.conf");
        fs::write(&target, b"unchanged\n").expect("target fixture");
        std::os::unix::fs::symlink(&target, &path).expect("symlink fixture");
        assert!(matches!(
            SettingsWindowModel::default().persist(&path),
            Err(SettingsConfigError::SymlinkNotAllowed)
        ));
        assert_eq!(fs::read_to_string(target).unwrap(), "unchanged\n");
        fs::remove_dir_all(root).expect("remove symlink fixture");
    }

    #[test]
    fn network_status_reads_bounded_non_loopback_sysfs_entries() {
        let root = std::env::temp_dir().join(format!(
            "aqua-network-status-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("lo")).expect("loopback fixture");
        fs::write(root.join("lo/operstate"), b"unknown\n").expect("loopback state");
        fs::create_dir_all(root.join("eth0")).expect("ethernet fixture");
        fs::create_dir(root.join("eth0/device")).expect("ethernet device fixture");
        fs::write(root.join("eth0/operstate"), b"up\n").expect("ethernet state");
        fs::create_dir_all(root.join("wlan0")).expect("wireless fixture");
        fs::create_dir(root.join("wlan0/wireless")).expect("wireless device fixture");
        fs::write(root.join("wlan0/operstate"), b"unexpected\n").expect("wireless state");

        let interfaces = read_network_interfaces(&root).expect("network status should read");
        assert_eq!(interfaces.len(), 2);
        assert_eq!(interfaces[0].name(), "eth0");
        assert_eq!(interfaces[0].link(), NetworkLinkState::Up);
        assert_eq!(interfaces[1].name(), "wlan0");
        assert_eq!(interfaces[1].link(), NetworkLinkState::Unknown);

        let route = root.join("route");
        let resolver = root.join("resolv.conf");
        fs::write(
            &route,
            "Iface Destination Gateway Flags\neth0 00000000 0100000A 0003\n",
        )
        .expect("route fixture");
        fs::write(&resolver, "nameserver 9.9.9.9\n").expect("resolver fixture");
        let mut settings = SettingsWindowModel::default();
        settings
            .refresh_network_status(&root, &route, &resolver)
            .expect("Settings network snapshot");
        assert_eq!(
            settings.network.health(),
            aqua_service_adapters::NetworkServiceHealth::Online
        );
        assert_eq!(settings.network.default_route(), Some("eth0"));
        assert_eq!(settings.network.dns_servers().len(), 1);
        fs::remove_dir_all(root).expect("remove network fixture");
    }

    #[test]
    fn launcher_defaults_to_closed_favorites() {
        let launcher = LauncherState::default();
        assert!(!launcher.is_open());
        assert_eq!(launcher.category(), LauncherCategory::Favorites);
        assert_eq!(launcher.visible_apps().len(), 6);
        assert!(launcher.activate_selected().is_none());
    }

    #[test]
    fn launcher_filters_categories_and_search_text() {
        let mut launcher = LauncherState::default();
        launcher.open();
        launcher.select_category(LauncherCategory::System);
        assert_eq!(launcher.visible_apps().len(), 3);

        launcher.set_query("update");
        assert_eq!(launcher.visible_apps()[0].id, "updates");
    }

    #[test]
    fn launcher_selection_wraps_in_both_directions() {
        let mut launcher = LauncherState::default();
        launcher.open();
        assert!(!launcher.navigate_selection_in_viewport(
            CollectionNavigationKey::Previous,
            800,
            180
        ));
        assert_eq!(launcher.selected_index(), 0);
        assert!(launcher.navigate_selection(CollectionNavigationKey::Previous));
        assert_eq!(launcher.selected_index(), 5);
        assert!(launcher.navigate_selection(CollectionNavigationKey::Next));
        assert_eq!(launcher.selected_index(), 0);
        assert!(launcher.navigate_selection(CollectionNavigationKey::End));
        assert_eq!(launcher.selected_index(), 5);
        assert!(launcher.navigate_selection(CollectionNavigationKey::Home));
        assert_eq!(launcher.selected_index(), 0);

        launcher.open_search();
        assert!(!launcher.navigate_selection_in_viewport(
            CollectionNavigationKey::Previous,
            500,
            600
        ));
        assert_eq!(launcher.selected_index(), 0);
        assert!(launcher.navigate_selection(CollectionNavigationKey::Previous));
        assert_eq!(launcher.selected_index(), 4);
        assert!(!launcher.select_visible_index(5));
    }

    #[test]
    fn launcher_activation_returns_a_request_without_executing_it() {
        let mut launcher = LauncherState::default();
        launcher.open();
        launcher.select_category(LauncherCategory::Settings);
        assert!(launcher
            .application_grid_cell(0, 800, 600)
            .is_some_and(|cell| cell.keyboard_activates(ActivationKey::Enter)));
        assert!(launcher.activate_selected_in_viewport(800, 180).is_none());
        assert_eq!(
            launcher.activate_selected_in_viewport(800, 600),
            Some(LaunchRequest {
                app_id: "settings",
                command: "/usr/bin/aqua-settings",
                target: None,
            })
        );

        launcher.selected_index = 3;
        assert!(launcher.activate_selected().is_none());
        launcher.set_query("settings");
        assert!(launcher.activate_selected_in_viewport(500, 600).is_none());
        assert_eq!(
            launcher
                .activate_selected_in_viewport(800, 600)
                .map(|request| request.app_id),
            Some("settings")
        );
        launcher.set_query("no matching application");
        assert!(launcher.activate_selected().is_none());
    }

    #[test]
    fn closing_launcher_resets_transient_state() {
        let mut launcher = LauncherState::default();
        launcher.open();
        launcher.set_query("files");
        launcher.close();
        assert!(!launcher.is_open());
        assert_eq!(launcher.query(), "");
        assert_eq!(launcher.selected_index(), 0);
    }

    #[test]
    fn launcher_probe_covers_interaction_and_design_contracts() {
        let probe = probe_launcher_model();
        assert!(probe.is_ready());
        assert!(probe
            .dump_lines()
            .contains(&"material=aqua-light-surface".to_string()));
    }

    #[test]
    fn launcher_events_request_only_required_scene_updates() {
        let mut launcher = LauncherState::default();

        let ignored = launcher.handle_event(LauncherEvent::Navigate(CollectionNavigationKey::Next));
        assert!(!ignored.redraw_requested);

        let opened = launcher.handle_event(LauncherEvent::Toggle);
        assert!(opened.redraw_requested);
        assert!(opened.visibility_changed);

        let searched = launcher.handle_event(LauncherEvent::ReplaceQuery("settings".into()));
        assert!(searched.redraw_requested);
        assert!(!searched.visibility_changed);
        let stable = launcher.handle_event(LauncherEvent::Navigate(CollectionNavigationKey::Next));
        assert!(!stable.redraw_requested);

        let activated = launcher.handle_event(LauncherEvent::Activate);
        assert_eq!(activated.launch_request.unwrap().app_id, "settings");

        let dismissed = launcher.handle_event(LauncherEvent::Dismiss);
        assert!(dismissed.visibility_changed);
        assert!(!launcher.is_open());
    }

    #[test]
    fn launcher_pointer_hit_test_maps_application_and_search_results() {
        let mut launcher = LauncherState::default();
        launcher.open();

        let overview = launcher.application_overview(800, 600);
        assert!(overview.is_valid());
        let first_cell = launcher.application_grid_cell(0, 800, 600).unwrap();
        assert_eq!(first_cell.rect, overview.cell_rect(0));
        assert_eq!(first_cell.accessibility().role, "gridcell");
        assert!(first_cell.pointer_hit(130, 190));
        assert_eq!(launcher.search_field(800, 600).rect, overview.search_rect());

        assert_eq!(
            launcher.pointer_target(130, 140),
            Some(LauncherPointerTarget::SearchField)
        );
        assert_eq!(
            launcher.pointer_target(130, 190),
            Some(LauncherPointerTarget::Application(0))
        );
        assert_eq!(
            launcher.pointer_target(300, 190),
            Some(LauncherPointerTarget::Panel)
        );
        assert_eq!(
            launcher.pointer_target(first_cell.rect.right(), first_cell.rect.y + 1),
            Some(LauncherPointerTarget::Panel)
        );
        launcher.open_search();
        launcher.set_query("settings");
        let search = launcher.global_search(800, 600);
        assert!(search.is_valid());
        assert_eq!(launcher.search_field(800, 600).rect, search.search_rect());
        let first_result = launcher.search_result_row(0, 800, 600).unwrap();
        assert_eq!(first_result.rect, search.result_rect(0));
        assert_eq!(first_result.accessibility().role, "option");
        assert!(first_result.pointer_hit(70, 220));
        assert_eq!(
            launcher.pointer_target(70, 220),
            Some(LauncherPointerTarget::Application(0))
        );
        assert_eq!(
            launcher.pointer_target(70, 260),
            Some(LauncherPointerTarget::Panel)
        );
        assert_eq!(
            launcher.pointer_target(70, 190),
            Some(LauncherPointerTarget::Panel)
        );
        assert_eq!(launcher.pointer_target(900, 700), None);
        assert_eq!(
            launcher.pointer_target_in_viewport(200, 150, 400, 300),
            None
        );
    }

    #[test]
    fn launcher_exposes_distinct_applications_and_search_modes() {
        let mut launcher = LauncherState::default();
        launcher.handle_event(LauncherEvent::OpenApplications);
        assert_eq!(launcher.mode(), LauncherMode::Applications);
        assert_eq!(launcher.category(), LauncherCategory::AllApplications);

        launcher.handle_event(LauncherEvent::OpenSearch);
        assert_eq!(launcher.mode(), LauncherMode::Search);
        assert_eq!(launcher.query(), "");

        launcher.handle_event(LauncherEvent::OpenApplications);
        launcher.handle_event(LauncherEvent::ReplaceQuery("files".into()));
        assert_eq!(launcher.mode(), LauncherMode::Search);
        assert_eq!(launcher.visible_apps()[0].id, "files");
    }

    #[test]
    fn launcher_search_quick_actions_are_real_and_bounded() {
        let mut launcher = LauncherState::default();
        launcher.open_search();
        let search = launcher.global_search(800, 600);
        let applications = launcher.search_quick_action_button(0, 800, 600).unwrap();
        assert_eq!(applications.rect, search.quick_action_rect(0));
        assert_eq!(applications.accessibility().role, "button");
        assert!(applications.pointer_hit(500, 220));
        assert_eq!(
            launcher.pointer_target(500, 220),
            Some(LauncherPointerTarget::QuickAction(
                LauncherQuickAction::Applications
            ))
        );
        assert_eq!(
            launcher.pointer_target(500, 258),
            Some(LauncherPointerTarget::Panel)
        );
        assert_eq!(
            launcher.pointer_target(applications.rect.right(), applications.rect.y + 1),
            Some(LauncherPointerTarget::Panel)
        );
        let settings = launcher.activate_quick_action(LauncherQuickAction::Settings);
        assert_eq!(settings.launch_request.unwrap().app_id, "settings");

        launcher.close();
        let blocked = launcher.activate_quick_action(LauncherQuickAction::Files);
        assert!(!blocked.redraw_requested);
        assert!(blocked.launch_request.is_none());
    }

    #[test]
    fn dock_hit_test_and_launch_requests_are_bounded() {
        let running_dock = running_app_dock(760, 72);
        assert!(running_dock.is_valid());
        assert_eq!(running_dock.rect.x, 272);
        assert_eq!(running_dock.item_rect(0).x, 272);
        assert_eq!(running_dock.icon_rect(1).x, 348);
        assert_eq!(running_dock.indicator_rect(2).x, 449);
        let workspaces = workspace_switcher(760, 72, 1);
        assert!(workspaces.is_valid());
        assert_eq!(workspaces.rect.x, 580);
        assert_eq!(workspaces.thumbnail_rect(1).x, 645);
        assert_eq!(workspaces.active_indicator_rect().x, 650);
        assert_eq!(
            dock_pointer_target(24, 36, 760, 72),
            Some(BottomShellTarget::Applications)
        );
        assert_eq!(
            dock_pointer_target(92, 36, 760, 72),
            Some(BottomShellTarget::Search)
        );
        assert_eq!(
            dock_pointer_target(280, 36, 760, 72),
            Some(BottomShellTarget::Application(DockItem::Files))
        );
        assert_eq!(
            dock_pointer_target(352, 36, 760, 72),
            Some(BottomShellTarget::Application(DockItem::Settings))
        );
        assert_eq!(
            dock_pointer_target(424, 36, 760, 72),
            Some(BottomShellTarget::Application(DockItem::Trash))
        );
        assert_eq!(
            dock_pointer_target(650, 36, 760, 72),
            Some(BottomShellTarget::Workspace(1))
        );
        assert_eq!(dock_pointer_target(200, 36, 760, 72), None);
        assert_eq!(dock_pointer_target(488, 36, 760, 72), None);
        assert_eq!(dock_pointer_target(760, 36, 760, 72), None);
        assert_eq!(
            DockItem::Settings.launch_request().unwrap().app_id,
            "settings"
        );
    }

    #[test]
    fn dock_running_state_tracks_real_application_ownership() {
        let state = DockState {
            applications_open: true,
            search_open: false,
            files_running: true,
            settings_running: false,
            active_workspace: 1,
        };
        assert!(state.item_running(DockItem::Files));
        assert!(state.item_running(DockItem::Trash));
        assert!(!state.item_running(DockItem::Settings));
        assert!(state.workspace_active(1));
        assert!(!state.workspace_active(2));
    }

    #[test]
    fn workspace_keyboard_routing_uses_shared_bounded_targets() {
        assert_eq!(
            workspace_keyboard_target(1, WorkspaceNavigationKey::Previous),
            Some(0)
        );
        assert_eq!(
            workspace_keyboard_target(1, WorkspaceNavigationKey::Next),
            Some(2)
        );
        assert_eq!(
            workspace_keyboard_target(1, WorkspaceNavigationKey::Home),
            Some(0)
        );
        assert_eq!(
            workspace_keyboard_target(1, WorkspaceNavigationKey::End),
            Some(2)
        );
        assert_eq!(
            workspace_keyboard_target(0, WorkspaceNavigationKey::Previous),
            None
        );
        assert_eq!(
            workspace_keyboard_target(2, WorkspaceNavigationKey::Next),
            None
        );
        assert_eq!(
            workspace_keyboard_target(WORKSPACE_COUNT, WorkspaceNavigationKey::Home),
            None
        );
    }

    #[test]
    fn session_menu_requires_explicit_confirmation_and_wraps_selection() {
        let mut menu = SessionMenuState::default();
        assert!(!menu.is_open());
        assert_eq!(menu.selected_action(), SessionAction::Logout);

        assert!(
            menu.handle_event(SessionMenuEvent::Toggle)
                .visibility_changed
        );
        menu.handle_event(SessionMenuEvent::Navigate(MenuNavigationKey::Previous));
        assert_eq!(menu.selected_action(), SessionAction::Recovery);
        let layout = menu.menu_layout(512, 293);
        assert!(layout.is_valid());
        assert_eq!(layout.item_rect(0).y, 62);
        assert_eq!(layout.item_rect(3).y, 197);
        assert_eq!(layout.item_at(24, 103), None);

        let armed = menu.handle_event(SessionMenuEvent::Activate);
        assert!(armed.confirmation_changed);
        assert_eq!(armed.action_request, None);
        assert_eq!(menu.confirmation(), Some(SessionAction::Recovery));
        let dialog = menu
            .confirmation_dialog(512, 293)
            .expect("armed session action should expose shared confirmation");
        assert!(dialog.is_valid());
        assert_eq!(
            dialog.requirement,
            ConfirmationRequirement::RepeatActivation
        );
        assert_eq!(dialog.state, ConfirmationState::Armed);
        assert_eq!(dialog.rect.y, 258);

        let confirmed = menu.handle_event(SessionMenuEvent::Activate);
        assert_eq!(confirmed.action_request, Some(SessionAction::Recovery));
        assert!(confirmed.visibility_changed);
        assert!(!menu.is_open());
        assert_eq!(menu.confirmation(), None);
        assert!(menu.confirmation_dialog(512, 293).is_none());
    }

    #[test]
    fn session_menu_dismiss_and_selection_change_clear_confirmation() {
        let mut menu = SessionMenuState::default();
        menu.handle_event(SessionMenuEvent::Toggle);
        menu.handle_event(SessionMenuEvent::Activate);
        let changed = menu.handle_event(SessionMenuEvent::Navigate(MenuNavigationKey::Next));
        assert!(changed.confirmation_changed);
        assert_eq!(menu.confirmation(), None);
        assert_eq!(menu.selected_action(), SessionAction::Restart);

        menu.handle_event(SessionMenuEvent::Dismiss);
        assert!(!menu.is_open());
        assert_eq!(
            SessionAction::ALL.map(SessionAction::id),
            ["logout", "restart", "shutdown", "recovery"]
        );
    }

    #[test]
    fn session_menu_keyboard_uses_shared_navigation_targets() {
        let mut menu = SessionMenuState::default();
        menu.handle_event(SessionMenuEvent::Toggle);

        let unchanged = menu.handle_event(SessionMenuEvent::Navigate(MenuNavigationKey::Home));
        assert!(!unchanged.redraw_requested);
        assert_eq!(menu.selected_action(), SessionAction::Logout);

        let end = menu.handle_event(SessionMenuEvent::Navigate(MenuNavigationKey::End));
        assert!(end.redraw_requested);
        assert_eq!(menu.selected_action(), SessionAction::Recovery);

        menu.handle_event(SessionMenuEvent::Activate);
        assert_eq!(menu.confirmation(), Some(SessionAction::Recovery));
        let home = menu.handle_event(SessionMenuEvent::Navigate(MenuNavigationKey::Home));
        assert!(home.confirmation_changed);
        assert_eq!(menu.selected_action(), SessionAction::Logout);
        assert_eq!(menu.confirmation(), None);

        menu.handle_event(SessionMenuEvent::Navigate(MenuNavigationKey::Previous));
        assert_eq!(menu.selected_action(), SessionAction::Recovery);
        menu.handle_event(SessionMenuEvent::Navigate(MenuNavigationKey::Next));
        assert_eq!(menu.selected_action(), SessionAction::Logout);
    }

    #[test]
    fn session_menu_pointer_uses_shared_rows_and_preserves_confirmation_gate() {
        let mut menu = SessionMenuState::default();
        let closed = menu.handle_pointer(
            SESSION_MENU_RUNTIME_WIDTH,
            SESSION_MENU_RUNTIME_HEIGHT,
            24,
            170,
        );
        assert!(!closed.redraw_requested);

        menu.handle_event(SessionMenuEvent::Toggle);
        let gap = menu.handle_pointer(
            SESSION_MENU_RUNTIME_WIDTH,
            SESSION_MENU_RUNTIME_HEIGHT,
            24,
            103,
        );
        assert!(!gap.redraw_requested);
        assert_eq!(menu.selected_action(), SessionAction::Logout);

        let armed = menu.handle_pointer(
            SESSION_MENU_RUNTIME_WIDTH,
            SESSION_MENU_RUNTIME_HEIGHT,
            24,
            170,
        );
        assert!(armed.redraw_requested);
        assert_eq!(armed.action_request, None);
        assert_eq!(menu.selected_action(), SessionAction::Shutdown);
        assert_eq!(menu.confirmation(), Some(SessionAction::Shutdown));

        let confirmed = menu.handle_pointer(
            SESSION_MENU_RUNTIME_WIDTH,
            SESSION_MENU_RUNTIME_HEIGHT,
            24,
            170,
        );
        assert_eq!(confirmed.action_request, Some(SessionAction::Shutdown));
        assert!(!menu.is_open());
    }

    #[test]
    fn notifications_queue_promote_and_expire_in_order() {
        let mut center = NotificationCenter::default();
        let first = center.post(100, "Files", "Files opened", "Home is ready.", 500);
        assert!(first.redraw_requested);
        assert!(first.visibility_changed);
        assert_eq!(center.active().map(|item| item.id), Some(1));

        let queued = center.post(200, "Settings", "Saved", "Preferences updated.", 300);
        assert!(!queued.redraw_requested);
        assert_eq!(center.queued_count(), 1);
        assert_eq!(center.tick(599), NotificationUpdate::default());

        let promoted = center.tick(600);
        assert!(promoted.redraw_requested);
        assert!(!promoted.visibility_changed);
        assert_eq!(center.active().map(|item| item.id), Some(2));
        assert_eq!(center.active().map(|item| item.expires_at_ms), Some(900));

        let expired = center.tick(900);
        assert!(expired.visibility_changed);
        assert!(center.active().is_none());
    }

    #[test]
    fn notifications_bound_queue_and_sanitize_text() {
        let mut center = NotificationCenter::default();
        center.post(0, "System\n", &"T".repeat(80), "Body\ttext", 1_000);
        for index in 0..NOTIFICATION_QUEUE_LIMIT + 2 {
            center.post(index as u64, "Aqua", "Queued", "Body", 1_000);
        }
        let active = center
            .active()
            .expect("first notification should remain active");
        assert_eq!(active.source, "System");
        assert_eq!(active.title.chars().count(), 64);
        assert_eq!(active.body, "Body text");
        assert_eq!(center.queued_count(), NOTIFICATION_QUEUE_LIMIT);
    }

    #[test]
    fn files_window_model_covers_home_and_empty_locations() {
        let home = FilesWindowModel::default();
        assert_eq!(home.title, "Files");
        assert_eq!(home.sidebar_items[home.selected_sidebar], "Home");
        assert_eq!(home.entries.len(), 4);
        assert!(!home.is_empty());

        let empty = FilesWindowModel::empty("Aqua / Empty");
        assert!(empty.is_empty());
        assert_eq!(empty.location, "Aqua / Empty");
    }

    #[test]
    fn files_window_model_reads_only_inside_root_and_tracks_pointer_selection() {
        let root = std::env::temp_dir().join(format!(
            "aqua-files-model-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let home = root.join("home/aqua");
        fs::create_dir_all(home.join("Documents")).expect("Documents fixture");
        fs::create_dir_all(home.join("Downloads")).expect("Downloads fixture");
        fs::write(home.join("Welcome.txt"), b"Aqua Linux\n").expect("file fixture");
        fs::write(home.join(".hidden"), b"hidden\n").expect("hidden fixture");
        #[cfg(unix)]
        std::os::unix::fs::symlink("/", home.join("escape")).expect("symlink fixture");

        let mut model =
            FilesWindowModel::read_only_directory(&home, &home).expect("home should be readable");
        assert_eq!(model.entries.len(), 3);
        assert_eq!(model.entries[0].kind, FilesEntryKind::Folder);
        assert!(model.entries.iter().all(|entry| entry.name != ".hidden"));
        assert!(model.entries.iter().all(|entry| entry.name != "escape"));
        let first_row = model.entry_row(640, 0).expect("shared Files entry row");
        assert_eq!(
            first_row.rect,
            Rect {
                x: 188,
                y: 124,
                width: 436,
                height: 56,
            }
        );
        assert_eq!(first_row.slots().leading.width, 54);
        assert_eq!(first_row.slots().trailing.width, 130);
        assert_eq!(first_row.accessibility().role, "option");
        assert_eq!(first_row.accessibility().name, "Documents");
        assert_eq!(model.select_at(640, 20, 70), FilesSelection::None);
        model.can_go_back = true;
        assert_eq!(model.select_at(640, 20, 70), FilesSelection::Back);
        assert_eq!(model.select_at(640, 20, 64), FilesSelection::None);
        assert_eq!(model.select_at(640, 220, 184), FilesSelection::None);
        assert_eq!(model.select_at(640, 624, 140), FilesSelection::None);
        assert_eq!(model.select_at(640, 220, 140), FilesSelection::Entry(0));
        assert_eq!(model.selected_entry, Some(0));
        assert_eq!(
            model.entry_row(640, 0).expect("selected row").state,
            ComponentState::Selected
        );
        assert!(model.hover_at(640, 220, 204));
        assert_eq!(model.hovered_entry, Some(1));
        assert_eq!(
            model.entry_row(640, 1).expect("hovered row").state,
            ComponentState::Hover
        );
        assert_eq!(model.select_at(640, 40, 180), FilesSelection::Sidebar(1));
        assert_eq!(model.selected_sidebar, 1);
        assert_eq!(model.selected_entry, None);
        assert_eq!(model.select_at(640, 40, 164), FilesSelection::None);
        assert!(model.hover_at(640, 40, 164));
        assert_eq!(model.hovered_entry, None);
        assert!(!model.hover_at(640, 40, 164));

        assert_eq!(
            FilesWindowModel::read_only_directory(&home, &root),
            Err(FilesReadError::OutsideAllowedRoot)
        );
        fs::remove_dir_all(root).expect("remove fixture root");
    }

    #[test]
    fn files_navigator_opens_folders_and_tracks_history_inside_root() {
        let root = std::env::temp_dir().join(format!(
            "aqua-files-navigation-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let home = root.join("home");
        fs::create_dir_all(home.join("Documents/Projects")).expect("navigation fixture");
        fs::create_dir_all(home.join("Downloads")).expect("downloads fixture");
        fs::create_dir_all(home.join("Pictures")).expect("pictures fixture");
        let canonical_home = home.canonicalize().expect("canonical home fixture");

        let mut navigator = FilesNavigator::open(&home).expect("navigator should open");
        assert_eq!(navigator.window().entries[0].name, "Documents");
        assert_eq!(
            navigator.handle_pointer(640, 220, 140),
            FilesNavigation::Selected(0)
        );
        assert_eq!(
            navigator.handle_pointer(640, 220, 140),
            FilesNavigation::Navigated
        );
        assert_eq!(navigator.current(), canonical_home.join("Documents"));
        assert!(navigator.can_go_back());
        assert!(navigator.window().can_go_back);
        assert!(!navigator.window().can_go_forward);
        assert_eq!(
            navigator.handle_key(640, FilesKey::Down),
            FilesNavigation::Selected(0)
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Activate),
            FilesNavigation::Navigated
        );
        assert_eq!(
            navigator.current(),
            canonical_home.join("Documents/Projects")
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Back),
            FilesNavigation::NavigatedBack
        );
        assert_eq!(
            navigator.handle_pointer(640, 28, 78),
            FilesNavigation::NavigatedBack
        );
        assert_eq!(navigator.current(), canonical_home);
        assert!(navigator.can_go_forward());
        assert!(navigator.window().can_go_forward);
        assert_eq!(
            navigator.handle_pointer(640, 60, 78),
            FilesNavigation::NavigatedForward
        );
        assert_eq!(navigator.current(), canonical_home.join("Documents"));
        assert_eq!(
            navigator.handle_pointer(640, 40, 272),
            FilesNavigation::Navigated
        );
        assert_eq!(navigator.current(), canonical_home.join("Pictures"));
        assert_eq!(navigator.window().selected_sidebar, 3);
        assert!(navigator.handle_hover(640, 40, 180));
        assert_eq!(navigator.window().hovered_sidebar, Some(1));
        assert_eq!(
            navigator.handle_pointer(640, 40, 318),
            FilesNavigation::Blocked
        );
        assert_eq!(navigator.current(), canonical_home.join("Pictures"));

        fs::remove_dir_all(root).expect("remove navigation fixture");
    }

    #[test]
    fn files_navigator_scrolls_and_previews_only_small_root_confined_text() {
        let root = std::env::temp_dir().join(format!(
            "aqua-files-preview-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("preview fixture root");
        for index in 0..5 {
            fs::create_dir(root.join(format!("Folder-{index}"))).expect("folder fixture");
        }
        fs::write(
            root.join("Notes.txt"),
            b"line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\n",
        )
        .expect("text fixture");
        fs::write(root.join("Program.bin"), b"not executable through Files\n")
            .expect("binary fixture");
        fs::write(
            root.join("TooLarge.txt"),
            vec![b'x'; FILES_TEXT_PREVIEW_LIMIT as usize + 1],
        )
        .expect("large fixture");
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc/passwd", root.join("Escape.txt"))
            .expect("preview symlink fixture");

        let mut navigator = FilesNavigator::open(&root).expect("preview navigator");
        assert_eq!(navigator.window().entries.len(), 8);
        assert_eq!(navigator.handle_scroll(1), FilesNavigation::Scrolled);
        assert_eq!(navigator.window().scroll_offset, 1);
        assert_eq!(
            navigator.handle_key(640, FilesKey::PageDown),
            FilesNavigation::Selected(3)
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Down),
            FilesNavigation::Selected(4)
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Down),
            FilesNavigation::Selected(5)
        );
        assert_eq!(navigator.window().selected_entry, Some(5));
        assert_eq!(navigator.window().scroll_offset, 2);
        assert!(navigator.window().keyboard_focus);
        assert_eq!(
            navigator.handle_key(640, FilesKey::Activate),
            FilesNavigation::PreviewOpened
        );
        assert_eq!(
            navigator
                .window()
                .preview
                .as_ref()
                .map(|preview| preview.name.as_str()),
            Some("Notes.txt")
        );
        assert_eq!(navigator.handle_scroll(1), FilesNavigation::PreviewScrolled);
        assert_eq!(
            navigator.window().preview.as_ref().unwrap().scroll_offset,
            1
        );
        let preview_scrollbar = navigator
            .window()
            .preview_scrollbar(640)
            .expect("scrollable Files preview");
        assert_eq!(
            preview_scrollbar.track,
            Rect {
                x: 628,
                y: 188,
                width: 5,
                height: 136,
            }
        );
        assert_eq!(preview_scrollbar.maximum_offset, 2);
        assert_eq!(preview_scrollbar.thumb.height, 102);
        assert_eq!(preview_scrollbar.thumb.y, 205);
        assert!(navigator.window().list_scrollbar(640).is_none());
        assert_eq!(
            navigator.window().active_scrollbar(640),
            Some(preview_scrollbar)
        );
        assert!(navigator.scrollbar_hit(640, preview_scrollbar.track.x, preview_scrollbar.track.y));
        assert!(!navigator.scrollbar_hit(
            640,
            preview_scrollbar.track.right(),
            preview_scrollbar.track.y
        ));
        assert!(!navigator.scrollbar_hit(
            640,
            preview_scrollbar.track.x,
            preview_scrollbar.track.bottom()
        ));
        assert_eq!(
            navigator.handle_scrollbar_drag(640, preview_scrollbar.track.y),
            FilesNavigation::PreviewScrolled
        );
        assert_eq!(
            navigator.window().preview.as_ref().unwrap().scroll_offset,
            0
        );
        assert_eq!(
            navigator.handle_scrollbar_drag(640, preview_scrollbar.track.bottom()),
            FilesNavigation::PreviewScrolled
        );
        assert_eq!(
            navigator.window().preview.as_ref().unwrap().scroll_offset,
            2
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Up),
            FilesNavigation::PreviewScrolled
        );
        assert_eq!(
            navigator.window().preview.as_ref().unwrap().scroll_offset,
            1
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::PageUp),
            FilesNavigation::PreviewScrolled
        );
        assert_eq!(
            navigator.window().preview.as_ref().unwrap().scroll_offset,
            0
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Home),
            FilesNavigation::None
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::End),
            FilesNavigation::PreviewScrolled
        );
        assert_eq!(
            navigator.window().preview.as_ref().unwrap().scroll_offset,
            2
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Down),
            FilesNavigation::None
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::Activate),
            FilesNavigation::None
        );
        assert_eq!(navigator.window().selected_entry, Some(5));
        assert_eq!(navigator.window().scroll_offset, 2);
        assert_eq!(
            navigator.handle_key(640, FilesKey::Back),
            FilesNavigation::PreviewClosed
        );
        assert_eq!(
            navigator.handle_key(640, FilesKey::PageUp),
            FilesNavigation::Selected(1)
        );
        assert_eq!(navigator.window().scroll_offset, 1);
        assert_eq!(
            navigator.handle_key(640, FilesKey::Home),
            FilesNavigation::Selected(0)
        );
        assert_eq!(navigator.window().scroll_offset, 0);
        assert_eq!(
            navigator.handle_key(640, FilesKey::End),
            FilesNavigation::Selected(7)
        );
        assert_eq!(navigator.window().scroll_offset, 4);
        let scrollbar = navigator
            .window()
            .list_scrollbar(640)
            .expect("scrollable Files list");
        assert_eq!(
            scrollbar.track,
            Rect {
                x: 628,
                y: 124,
                width: 5,
                height: 248,
            }
        );
        assert_eq!(scrollbar.maximum_offset, 4);
        assert_eq!(scrollbar.thumb.height, 124);
        assert_eq!(scrollbar.thumb.y, 248);
        assert!(navigator.scrollbar_hit(640, scrollbar.track.x, scrollbar.track.y));
        assert!(!navigator.scrollbar_hit(640, 620, scrollbar.track.y));
        assert!(!navigator.scrollbar_hit(640, scrollbar.track.right(), scrollbar.track.y));
        assert!(!navigator.scrollbar_hit(640, scrollbar.track.x, scrollbar.track.bottom()));
        assert_eq!(
            navigator.handle_scrollbar_drag(640, scrollbar.track.y),
            FilesNavigation::Scrolled
        );
        assert_eq!(navigator.window().scroll_offset, 0);
        assert_eq!(
            navigator.handle_scrollbar_drag(640, scrollbar.track.bottom()),
            FilesNavigation::Scrolled
        );
        assert_eq!(navigator.window().scroll_offset, 4);
        navigator.window.selected_entry = Some(0);
        assert_eq!(
            navigator.handle_key(640, FilesKey::Activate),
            FilesNavigation::None
        );
        navigator.window.selected_entry = Some(6);
        assert_eq!(
            navigator.handle_key(640, FilesKey::Activate),
            FilesNavigation::PreviewBlocked
        );
        navigator.window.selected_entry = Some(7);
        assert_eq!(
            navigator.handle_key(640, FilesKey::Activate),
            FilesNavigation::PreviewBlocked
        );
        assert!(navigator.window().preview.is_none());

        fs::remove_dir_all(root).expect("remove preview fixture");
    }
}
