use std::collections::VecDeque;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const SETTINGS_CONFIG_VERSION: u8 = 1;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
pub const FILES_SCROLLBAR_X: u32 = 620;
pub const FILES_SCROLLBAR_Y: u32 = 124;
pub const FILES_SCROLLBAR_HEIGHT: u32 = 248;
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

    let item_width = 72_u32;
    let content_width = item_width * DOCK_ITEM_COUNT as u32;
    let start_x = width.saturating_sub(content_width) / 2;
    if (start_x..start_x + content_width).contains(&local_x) {
        return DockItem::ALL
            .get(((local_x - start_x) / item_width) as usize)
            .copied()
            .map(BottomShellTarget::Application);
    }

    let workspace_width = 60_u32;
    let workspace_group_width = workspace_width * WORKSPACE_COUNT as u32;
    let workspace_start = width.saturating_sub(workspace_group_width);
    (workspace_start..width).contains(&local_x).then(|| {
        BottomShellTarget::Workspace(((local_x - workspace_start) / workspace_width) as usize)
    })
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

    pub fn trash_empty_confirmation(&self) -> bool {
        self.trash_empty_confirmation
    }

    fn context_menu_y(index: usize) -> u32 {
        DESKTOP_ICON_Y
            + (index as u32 * DESKTOP_ICON_ROW_HEIGHT + 32)
                .min(DESKTOP_ICON_LAYER_HEIGHT.saturating_sub(76))
    }

    fn context_menu_row(&self, x: u32, y: u32) -> Option<(usize, usize)> {
        let icon_index = self.context_menu?;
        let menu_y = Self::context_menu_y(icon_index);
        if !(DESKTOP_CONTEXT_MENU_X..DESKTOP_CONTEXT_MENU_X + DESKTOP_CONTEXT_MENU_WIDTH)
            .contains(&x)
            || !(menu_y..menu_y + DESKTOP_CONTEXT_MENU_ROW_HEIGHT * 2).contains(&y)
        {
            return None;
        }
        Some((
            icon_index,
            ((y - menu_y) / DESKTOP_CONTEXT_MENU_ROW_HEIGHT) as usize,
        ))
    }

    pub fn pointer_target(x: u32, y: u32) -> Option<usize> {
        if !(DESKTOP_ICON_X..DESKTOP_ICON_X + DESKTOP_ICON_WIDTH).contains(&x) || y < DESKTOP_ICON_Y
        {
            return None;
        }
        let index = ((y - DESKTOP_ICON_Y) / DESKTOP_ICON_ROW_HEIGHT) as usize;
        (index < DESKTOP_ICONS.len()).then_some(index)
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
                self.last_primary_click = None;
                if row == 0 {
                    self.context_menu = None;
                    self.trash_empty_confirmation = false;
                    let launch_request = DESKTOP_ICONS[icon_index].launch.clone();
                    return DesktopIconUpdate {
                        redraw_requested: true,
                        launch_request,
                        context_action: None,
                    };
                }
                if DESKTOP_ICONS[icon_index].id == "trash" {
                    if self.trash_empty_confirmation {
                        self.context_menu = None;
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
                self.trash_empty_confirmation = false;
                return DesktopIconUpdate {
                    redraw_requested: true,
                    launch_request: None,
                    context_action: Some(DesktopContextAction::Properties(
                        DESKTOP_ICONS[icon_index].id,
                    )),
                };
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindowModel {
    pub title: &'static str,
    pub categories: [&'static str; 5],
    pub selected_category: usize,
    pub hovered_category: Option<usize>,
    pub reduced_motion: bool,
    pub desktop_icons: bool,
    pub key_repeat: bool,
    pub network_interfaces: Vec<NetworkInterfaceStatus>,
    pub network_status_available: bool,
    pub keyboard_focus: bool,
    pub theme: AquaTheme,
}

impl Default for SettingsWindowModel {
    fn default() -> Self {
        Self {
            title: "System Settings",
            categories: ["Appearance", "Desktop", "Input", "Network", "About"],
            selected_category: 0,
            hovered_category: None,
            reduced_motion: false,
            desktop_icons: true,
            key_repeat: true,
            network_interfaces: Vec::new(),
            network_status_available: false,
            keyboard_focus: false,
            theme: AquaTheme::default(),
        }
    }
}

impl SettingsWindowModel {
    pub fn refresh_network_status(&mut self, class_net: &Path) -> io::Result<()> {
        self.network_interfaces = read_network_interfaces(class_net)?;
        self.network_status_available = true;
        Ok(())
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
                _ => return Err(SettingsConfigError::InvalidFormat),
            }
        }
        if version != Some(SETTINGS_CONFIG_VERSION) {
            return Err(SettingsConfigError::UnsupportedVersion);
        }
        let reduced_motion = reduced_motion.ok_or(SettingsConfigError::InvalidFormat)?;
        Ok(Self {
            reduced_motion,
            desktop_icons: desktop_icons.unwrap_or(true),
            key_repeat: key_repeat.unwrap_or(true),
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
            "version={SETTINGS_CONFIG_VERSION}\nreduced_motion={}\ndesktop_icons={}\nkey_repeat={}\ntheme={}\n",
            self.reduced_motion,
            self.desktop_icons,
            self.key_repeat,
            self.theme.id()
        )
    }

    pub fn handle_pointer(&mut self, x: u32, y: u32) -> SettingsUpdate {
        if x < 190 && (92..342).contains(&y) {
            let category = ((y - 92) / 50) as usize;
            if category < self.categories.len() {
                self.selected_category = category;
                return SettingsUpdate::CategorySelected(category);
            }
        }
        if self.selected_category == 0 && (430..570).contains(&x) && (128..178).contains(&y) {
            self.reduced_motion = !self.reduced_motion;
            return SettingsUpdate::ReducedMotionChanged(self.reduced_motion);
        }
        if self.selected_category == 0 && (218..570).contains(&x) && (214..262).contains(&y) {
            let index = ((x - 218) / 88) as usize;
            if let Some(theme) = AquaTheme::ALL.get(index).copied() {
                self.theme = theme;
                return SettingsUpdate::ThemeChanged(theme);
            }
        }
        if self.selected_category == 1 && (430..570).contains(&x) && (128..178).contains(&y) {
            self.desktop_icons = !self.desktop_icons;
            return SettingsUpdate::DesktopIconsChanged(self.desktop_icons);
        }
        if self.selected_category == 2 && (430..570).contains(&x) && (128..178).contains(&y) {
            self.key_repeat = !self.key_repeat;
            return SettingsUpdate::KeyRepeatChanged(self.key_repeat);
        }
        SettingsUpdate::None
    }

    pub fn handle_hover(&mut self, x: u32, y: u32) -> bool {
        let previous = self.hovered_category;
        self.hovered_category = None;
        if x < 190 && (92..342).contains(&y) {
            let category = ((y - 92) / 50) as usize;
            if category < self.categories.len() {
                self.hovered_category = Some(category);
            }
        }
        previous != self.hovered_category
    }

    pub fn handle_key(&mut self, key: SettingsKey) -> SettingsUpdate {
        self.keyboard_focus = true;
        match key {
            SettingsKey::Home => {
                self.selected_category = 0;
                SettingsUpdate::CategorySelected(self.selected_category)
            }
            SettingsKey::Up => {
                self.selected_category = self
                    .selected_category
                    .checked_sub(1)
                    .unwrap_or(self.categories.len() - 1);
                SettingsUpdate::CategorySelected(self.selected_category)
            }
            SettingsKey::Down => {
                self.selected_category = (self.selected_category + 1) % self.categories.len();
                SettingsUpdate::CategorySelected(self.selected_category)
            }
            SettingsKey::Activate if self.selected_category == 0 => {
                self.reduced_motion = !self.reduced_motion;
                SettingsUpdate::ReducedMotionChanged(self.reduced_motion)
            }
            SettingsKey::Activate if self.selected_category == 1 => {
                self.desktop_icons = !self.desktop_icons;
                SettingsUpdate::DesktopIconsChanged(self.desktop_icons)
            }
            SettingsKey::Activate if self.selected_category == 2 => {
                self.key_repeat = !self.key_repeat;
                SettingsUpdate::KeyRepeatChanged(self.key_repeat)
            }
            SettingsKey::Activate => SettingsUpdate::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterfaceStatus {
    pub name: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopBarState {
    pub product_label: String,
    pub clock_label: String,
    pub network_connected: bool,
    pub battery_percent: Option<u8>,
    pub audio_available: bool,
}

impl TopBarState {
    pub fn read(root: &Path, epoch_seconds: u64) -> Self {
        let network_connected = read_network_interfaces(&root.join("sys/class/net"))
            .unwrap_or_default()
            .iter()
            .any(|interface| interface.state == "up");

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

pub fn read_network_interfaces(class_net: &Path) -> io::Result<Vec<NetworkInterfaceStatus>> {
    let mut interfaces = Vec::new();
    for entry in fs::read_dir(class_net)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name == "lo"
            || name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        {
            continue;
        }
        let state = fs::read_to_string(entry.path().join("operstate"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_ascii_lowercase();
        let state = match state.as_str() {
            "up" | "down" | "dormant" | "lowerlayerdown" | "notpresent" | "testing" | "unknown" => {
                state
            }
            _ => "unknown".to_string(),
        };
        interfaces.push(NetworkInterfaceStatus { name, state });
        if interfaces.len() == 8 {
            break;
        }
    }
    interfaces.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(interfaces)
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
    Up,
    Down,
    Activate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsUpdate {
    None,
    CategorySelected(usize),
    ReducedMotionChanged(bool),
    DesktopIconsChanged(bool),
    KeyRepeatChanged(bool),
    ThemeChanged(AquaTheme),
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

    pub fn select_at(&mut self, x: u32, y: u32) -> FilesSelection {
        if (18..46).contains(&x) && (64..98).contains(&y) {
            return FilesSelection::Back;
        }
        if (50..78).contains(&x) && (64..98).contains(&y) {
            return FilesSelection::Forward;
        }
        if x < 170 && (126..356).contains(&y) {
            let index = ((y - 126) / 46) as usize;
            if index < self.sidebar_items.len() {
                self.selected_sidebar = index;
                self.selected_entry = None;
                return FilesSelection::Sidebar(index);
            }
        }
        if x >= 170 && (124..380).contains(&y) {
            let index = self.scroll_offset + ((y - 124) / 64) as usize;
            if index < self.entries.len() {
                self.selected_entry = Some(index);
                return FilesSelection::Entry(index);
            }
        }
        FilesSelection::None
    }

    pub fn hover_at(&mut self, x: u32, y: u32) -> bool {
        let previous = (self.hovered_sidebar, self.hovered_entry);
        self.hovered_sidebar = None;
        self.hovered_entry = None;
        if x < 170 && (126..356).contains(&y) {
            let index = ((y - 126) / 46) as usize;
            if index < self.sidebar_items.len() {
                self.hovered_sidebar = Some(index);
            }
        } else if x >= 170 && (124..380).contains(&y) {
            let index = self.scroll_offset + ((y - 124) / 64) as usize;
            if index < self.entries.len() {
                self.hovered_entry = Some(index);
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

    pub fn handle_pointer(&mut self, x: u32, y: u32) -> FilesNavigation {
        let previously_selected = self.window.selected_entry;
        match self.window.select_at(x, y) {
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

    pub fn handle_hover(&mut self, x: u32, y: u32) -> bool {
        self.window.hover_at(x, y)
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

    pub fn scrollbar_hit(&self, x: u32, y: u32) -> bool {
        self.window.preview.is_none()
            && self.window.entries.len() > FILES_VISIBLE_ROWS
            && x >= FILES_SCROLLBAR_X
            && (FILES_SCROLLBAR_Y..=FILES_SCROLLBAR_Y + FILES_SCROLLBAR_HEIGHT).contains(&y)
    }

    pub fn handle_scrollbar_drag(&mut self, y: u32) -> FilesNavigation {
        if self.window.preview.is_some() || self.window.entries.len() <= FILES_VISIBLE_ROWS {
            return FilesNavigation::None;
        }
        let max_offset = self.window.entries.len() - FILES_VISIBLE_ROWS;
        let position = y.clamp(
            FILES_SCROLLBAR_Y,
            FILES_SCROLLBAR_Y + FILES_SCROLLBAR_HEIGHT,
        ) - FILES_SCROLLBAR_Y;
        let offset = (position as usize * max_offset + FILES_SCROLLBAR_HEIGHT as usize / 2)
            / FILES_SCROLLBAR_HEIGHT as usize;
        if offset == self.window.scroll_offset {
            return FilesNavigation::None;
        }
        self.window.scroll_offset = offset;
        self.window.hovered_entry = None;
        FilesNavigation::Scrolled
    }

    pub fn handle_key(&mut self, key: FilesKey) -> FilesNavigation {
        self.window.keyboard_focus = true;
        match key {
            FilesKey::Up => self.move_selection(-1),
            FilesKey::Down => self.move_selection(1),
            FilesKey::PageUp => self.move_selection(-(FILES_VISIBLE_ROWS as isize)),
            FilesKey::PageDown => self.move_selection(FILES_VISIBLE_ROWS as isize),
            FilesKey::Home => self.select_edge(false),
            FilesKey::End => self.select_edge(true),
            FilesKey::Activate => {
                let Some(index) = self.window.selected_entry else {
                    return FilesNavigation::None;
                };
                let Some(entry) = self.window.entries.get(index).cloned() else {
                    return FilesNavigation::None;
                };
                if entry.kind == FilesEntryKind::Folder {
                    self.navigate(self.current.join(entry.name), None)
                } else {
                    self.open_text_preview(index)
                }
            }
            FilesKey::Back => {
                if self.window.preview.take().is_some() {
                    FilesNavigation::PreviewClosed
                } else {
                    self.go_back()
                }
            }
        }
    }

    fn move_selection(&mut self, offset: isize) -> FilesNavigation {
        let count = self.window.entries.len();
        if count == 0 {
            return FilesNavigation::None;
        }
        let current = self.window.selected_entry.unwrap_or_else(|| {
            if offset < 0 {
                0
            } else {
                count.saturating_sub(1)
            }
        });
        let selected = (current as isize + offset).rem_euclid(count as isize) as usize;
        self.window.selected_entry = Some(selected);
        if selected < self.window.scroll_offset {
            self.window.scroll_offset = selected;
        } else if selected >= self.window.scroll_offset + FILES_VISIBLE_ROWS {
            self.window.scroll_offset = selected + 1 - FILES_VISIBLE_ROWS;
        }
        FilesNavigation::Selected(selected)
    }

    fn select_edge(&mut self, end: bool) -> FilesNavigation {
        let Some(selected) = end
            .then(|| self.window.entries.len().checked_sub(1))
            .flatten()
            .or_else(|| (!end && !self.window.entries.is_empty()).then_some(0))
        else {
            return FilesNavigation::None;
        };
        self.window.selected_entry = Some(selected);
        self.window.scroll_offset = if end {
            self.window.entries.len().saturating_sub(FILES_VISIBLE_ROWS)
        } else {
            0
        };
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
    MoveSelection(isize),
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

    pub fn move_selection(&mut self, offset: isize) {
        let count = self.visible_apps().len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }

        self.selected_index =
            (self.selected_index as isize + offset).rem_euclid(count as isize) as usize;
    }

    pub fn select_visible_index(&mut self, index: usize) -> bool {
        if index >= self.visible_apps().len() {
            return false;
        }
        self.selected_index = index;
        true
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
        let panel = self.panel_bounds(viewport_width, viewport_height);
        if !self.open
            || !(panel.x..panel.x + panel.width).contains(&x)
            || !(panel.y..panel.y + panel.height).contains(&y)
        {
            return None;
        }

        let content_y = panel.y + 96;
        match self.mode {
            LauncherMode::Applications => {
                if y >= content_y {
                    let column_width = (panel.width.saturating_sub(48) / 3).max(1);
                    let column = ((x.saturating_sub(panel.x + 24)) / column_width).min(2);
                    let row = (y - content_y) / 112;
                    let index = (row * 3 + column) as usize;
                    if index < self.visible_apps().len().min(6) {
                        return Some(LauncherPointerTarget::Application(index));
                    }
                }
            }
            LauncherMode::Search => {
                if y >= content_y {
                    if x < panel.x + panel.width / 2 {
                        let index = ((y - content_y) / 58) as usize;
                        if index < self.visible_apps().len().min(6) {
                            return Some(LauncherPointerTarget::Application(index));
                        }
                    } else {
                        let index = ((y - content_y) / 62) as usize;
                        let action = match index {
                            0 => Some(LauncherQuickAction::Applications),
                            1 => Some(LauncherQuickAction::Settings),
                            2 => Some(LauncherQuickAction::Files),
                            _ => None,
                        };
                        if let Some(action) = action {
                            return Some(LauncherPointerTarget::QuickAction(action));
                        }
                    }
                }
            }
        }
        Some(LauncherPointerTarget::Panel)
    }

    pub fn activate_selected(&self) -> Option<LaunchRequest> {
        if !self.open {
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
            LauncherEvent::MoveSelection(offset) => {
                if !self.open || self.visible_apps().is_empty() {
                    return LauncherUpdate::unchanged();
                }
                self.move_selection(offset);
                LauncherUpdate {
                    redraw_requested: true,
                    visibility_changed: false,
                    launch_request: None,
                }
            }
            LauncherEvent::Activate => LauncherUpdate {
                redraw_requested: false,
                visibility_changed: false,
                launch_request: self.activate_selected(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionMenuEvent {
    Toggle,
    Dismiss,
    MoveSelection(isize),
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

    pub fn close(&mut self) {
        self.open = false;
        self.selected_index = 0;
        self.confirmation = None;
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
            SessionMenuEvent::MoveSelection(offset) => {
                if !self.open {
                    return SessionMenuUpdate::unchanged();
                }
                self.selected_index = (self.selected_index as isize + offset)
                    .rem_euclid(SessionAction::ALL.len() as isize)
                    as usize;
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
    launcher.move_selection(-1);
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
        fs::create_dir_all(root.join("sys/class/power_supply/AC")).expect("adapter fixture");
        fs::create_dir_all(root.join("sys/class/power_supply/BAT0")).expect("battery fixture");
        fs::create_dir_all(root.join("dev/snd")).expect("audio fixture");
        fs::write(root.join("sys/class/net/eth0/operstate"), "up\n").expect("network state");
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

        fs::remove_dir_all(root).expect("remove top bar fixture");
    }

    #[test]
    fn desktop_icons_select_activate_and_open_context_menu() {
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
        assert_eq!(
            model.handle_pointer(500, 150),
            SettingsUpdate::ReducedMotionChanged(true)
        );
        assert!(model.reduced_motion);
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
            model.handle_pointer(40, 300),
            SettingsUpdate::CategorySelected(4)
        );
        assert!(model.handle_hover(40, 100));
        assert_eq!(model.hovered_category, Some(0));
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
            "version=1\nreduced_motion=true\ndesktop_icons=true\nkey_repeat=true\ntheme=LightWhite\n"
        );
        let reloaded = SettingsWindowModel::load_or_default(&path).expect("settings should reload");
        assert!(reloaded.reduced_motion);
        assert!(reloaded.desktop_icons);
        assert!(reloaded.key_repeat);
        assert_eq!(reloaded.theme, AquaTheme::LightWhite);
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
        fs::remove_dir_all(root).expect("remove settings fixture");
    }

    #[test]
    fn settings_theme_selection_is_bounded_and_persistent() {
        let mut model = SettingsWindowModel::default();
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
        fs::write(root.join("eth0/operstate"), b"up\n").expect("ethernet state");
        fs::create_dir_all(root.join("wlan0")).expect("wireless fixture");
        fs::write(root.join("wlan0/operstate"), b"unexpected\n").expect("wireless state");

        let interfaces = read_network_interfaces(&root).expect("network status should read");
        assert_eq!(
            interfaces,
            vec![
                NetworkInterfaceStatus {
                    name: "eth0".to_string(),
                    state: "up".to_string(),
                },
                NetworkInterfaceStatus {
                    name: "wlan0".to_string(),
                    state: "unknown".to_string(),
                },
            ]
        );
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
        launcher.move_selection(-1);
        assert_eq!(launcher.selected_index(), 5);
        launcher.move_selection(1);
        assert_eq!(launcher.selected_index(), 0);
    }

    #[test]
    fn launcher_activation_returns_a_request_without_executing_it() {
        let mut launcher = LauncherState::default();
        launcher.open();
        launcher.select_category(LauncherCategory::Settings);
        assert_eq!(
            launcher.activate_selected(),
            Some(LaunchRequest {
                app_id: "settings",
                command: "/usr/bin/aqua-settings",
                target: None,
            })
        );
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

        let ignored = launcher.handle_event(LauncherEvent::MoveSelection(1));
        assert!(!ignored.redraw_requested);

        let opened = launcher.handle_event(LauncherEvent::Toggle);
        assert!(opened.redraw_requested);
        assert!(opened.visibility_changed);

        let searched = launcher.handle_event(LauncherEvent::ReplaceQuery("settings".into()));
        assert!(searched.redraw_requested);
        assert!(!searched.visibility_changed);

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

        assert_eq!(
            launcher.pointer_target(130, 180),
            Some(LauncherPointerTarget::Application(0))
        );
        launcher.open_search();
        launcher.set_query("settings");
        assert_eq!(
            launcher.pointer_target(70, 180),
            Some(LauncherPointerTarget::Application(0))
        );
        assert_eq!(launcher.pointer_target(900, 700), None);
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
        assert_eq!(
            launcher.pointer_target(500, 190),
            Some(LauncherPointerTarget::QuickAction(
                LauncherQuickAction::Applications
            ))
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
    fn session_menu_requires_explicit_confirmation_and_wraps_selection() {
        let mut menu = SessionMenuState::default();
        assert!(!menu.is_open());
        assert_eq!(menu.selected_action(), SessionAction::Logout);

        assert!(
            menu.handle_event(SessionMenuEvent::Toggle)
                .visibility_changed
        );
        menu.handle_event(SessionMenuEvent::MoveSelection(-1));
        assert_eq!(menu.selected_action(), SessionAction::Recovery);

        let armed = menu.handle_event(SessionMenuEvent::Activate);
        assert!(armed.confirmation_changed);
        assert_eq!(armed.action_request, None);
        assert_eq!(menu.confirmation(), Some(SessionAction::Recovery));

        let confirmed = menu.handle_event(SessionMenuEvent::Activate);
        assert_eq!(confirmed.action_request, Some(SessionAction::Recovery));
        assert!(confirmed.visibility_changed);
        assert!(!menu.is_open());
        assert_eq!(menu.confirmation(), None);
    }

    #[test]
    fn session_menu_dismiss_and_selection_change_clear_confirmation() {
        let mut menu = SessionMenuState::default();
        menu.handle_event(SessionMenuEvent::Toggle);
        menu.handle_event(SessionMenuEvent::Activate);
        let changed = menu.handle_event(SessionMenuEvent::MoveSelection(1));
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
        assert_eq!(model.select_at(220, 140), FilesSelection::Entry(0));
        assert_eq!(model.selected_entry, Some(0));
        assert_eq!(model.select_at(40, 180), FilesSelection::Sidebar(1));
        assert_eq!(model.selected_sidebar, 1);
        assert_eq!(model.selected_entry, None);

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
            navigator.handle_pointer(220, 140),
            FilesNavigation::Selected(0)
        );
        assert_eq!(
            navigator.handle_pointer(220, 140),
            FilesNavigation::Navigated
        );
        assert_eq!(navigator.current(), canonical_home.join("Documents"));
        assert!(navigator.can_go_back());
        assert!(navigator.window().can_go_back);
        assert!(!navigator.window().can_go_forward);
        assert_eq!(
            navigator.handle_key(FilesKey::Down),
            FilesNavigation::Selected(0)
        );
        assert_eq!(
            navigator.handle_key(FilesKey::Activate),
            FilesNavigation::Navigated
        );
        assert_eq!(
            navigator.current(),
            canonical_home.join("Documents/Projects")
        );
        assert_eq!(
            navigator.handle_key(FilesKey::Back),
            FilesNavigation::NavigatedBack
        );
        assert_eq!(
            navigator.handle_pointer(28, 78),
            FilesNavigation::NavigatedBack
        );
        assert_eq!(navigator.current(), canonical_home);
        assert!(navigator.can_go_forward());
        assert!(navigator.window().can_go_forward);
        assert_eq!(
            navigator.handle_pointer(60, 78),
            FilesNavigation::NavigatedForward
        );
        assert_eq!(navigator.current(), canonical_home.join("Documents"));
        assert_eq!(
            navigator.handle_pointer(40, 272),
            FilesNavigation::Navigated
        );
        assert_eq!(navigator.current(), canonical_home.join("Pictures"));
        assert_eq!(navigator.window().selected_sidebar, 3);
        assert!(navigator.handle_hover(40, 180));
        assert_eq!(navigator.window().hovered_sidebar, Some(1));
        assert_eq!(navigator.handle_pointer(40, 318), FilesNavigation::Blocked);
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
            navigator.handle_key(FilesKey::PageDown),
            FilesNavigation::Selected(3)
        );
        assert_eq!(
            navigator.handle_key(FilesKey::Down),
            FilesNavigation::Selected(4)
        );
        assert_eq!(
            navigator.handle_key(FilesKey::Down),
            FilesNavigation::Selected(5)
        );
        assert_eq!(navigator.window().selected_entry, Some(5));
        assert_eq!(navigator.window().scroll_offset, 2);
        assert!(navigator.window().keyboard_focus);
        assert_eq!(
            navigator.handle_key(FilesKey::Activate),
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
        assert_eq!(
            navigator.handle_key(FilesKey::Back),
            FilesNavigation::PreviewClosed
        );
        assert_eq!(
            navigator.handle_key(FilesKey::PageUp),
            FilesNavigation::Selected(1)
        );
        assert_eq!(navigator.window().scroll_offset, 1);
        assert_eq!(
            navigator.handle_key(FilesKey::Home),
            FilesNavigation::Selected(0)
        );
        assert_eq!(navigator.window().scroll_offset, 0);
        assert_eq!(
            navigator.handle_key(FilesKey::End),
            FilesNavigation::Selected(7)
        );
        assert_eq!(navigator.window().scroll_offset, 4);
        assert!(navigator.scrollbar_hit(FILES_SCROLLBAR_X, FILES_SCROLLBAR_Y));
        assert_eq!(
            navigator.handle_scrollbar_drag(FILES_SCROLLBAR_Y),
            FilesNavigation::Scrolled
        );
        assert_eq!(navigator.window().scroll_offset, 0);
        assert_eq!(
            navigator.handle_scrollbar_drag(FILES_SCROLLBAR_Y + FILES_SCROLLBAR_HEIGHT),
            FilesNavigation::Scrolled
        );
        assert_eq!(navigator.window().scroll_offset, 4);
        navigator.window.selected_entry = Some(6);
        assert_eq!(
            navigator.handle_key(FilesKey::Activate),
            FilesNavigation::PreviewBlocked
        );
        navigator.window.selected_entry = Some(7);
        assert_eq!(
            navigator.handle_key(FilesKey::Activate),
            FilesNavigation::PreviewBlocked
        );
        assert!(navigator.window().preview.is_none());

        fs::remove_dir_all(root).expect("remove preview fixture");
    }
}
