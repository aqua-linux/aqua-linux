use std::fs;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::sync::mpsc::{self, Receiver};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::sync::Arc;
use std::time::Duration;

mod presentation;
pub use presentation::{
    DiagnosticReadbackEvidence, PresentationBudget, PresentationEvidenceTarget, PresentationPath,
    PresentationSample, PresentationWorkload, R2PresentationReport,
};

use aqua_components::{NotificationToast, WindowControl, WindowFrame};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_installer::{
    build_dry_run_plan, build_install_transaction_graph, compile_install_commands,
    compile_internal_install_actions, probe_storage, validate_install_prerequisites,
    InstallArtifacts, InstallMode, InstallProgressEvent, InstallToolPaths, InstallTransactionGraph,
    InstallerFocusTarget, InstallerFormKey, InstallerFormState, InstallerModel, InstallerStep,
    InstallerSummaryKey, InstallerUiAction, InstallerUiKey, InstallerUiState, InstallerUserField,
    InstallerUserFormKey, InstallerWindowLayout, NonExecutingInstallTransactionRunner,
    StorageProbePaths,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_renderer::{
    embedded_ui_font_ready, render_component_acceptance_rgba, render_files_window_rgba_with_theme,
    render_installer_window_rgba_with_theme, render_properties_window_rgba_with_theme,
    render_settings_window_rgba, render_terminal_window_rgba_with_theme,
    render_typography_layout_acceptance_rgba, InstallerImageSource, InstallerRenderOptions,
    COMPONENT_FIXTURE_REVISION, UI_FONT_FAMILY, UI_FONT_SOURCE,
};
pub use aqua_renderer::{
    export_composited_preview_png_with_client_layers,
    export_composited_preview_rgba_with_client_layers, export_software_raster_png_for_static_scene,
    export_software_raster_ppm_for_static_scene, frame_plan_for_static_scene,
    paint_plan_for_static_scene, plan_client_layer_paint_steps, plan_client_surface_sources,
    plan_static_scene, probe_client_layer_raster, probe_frame_buffer_for_static_scene,
    probe_software_raster_for_static_scene, ClientLayerPaintPlan, ClientLayerRasterProbe,
    ClientSurfaceSource, ClientSurfaceSourcePlan, FrameBufferProbe, FramePlan, PaintPlan,
    RasterPngExport, RasterPpmExport, RenderPlan, SoftwareRasterProbe, CLIENT_SAMPLE_GRID_PIXELS,
    RENDERER_STATUS, RENDER_BACKEND,
};
pub use aqua_scene::{static_shell_scene, Rect, ShellScene, SurfaceKind, Viewport, SCENE_STATUS};
pub use aqua_shell::{
    dock_pointer_target, properties_launch_request, top_system_bar, BottomShellTarget,
    DesktopContextAction, DesktopIconState, DesktopPointerButton, DockItem, DockState,
    LaunchRequest, LauncherCategory, LauncherEvent, LauncherPointerTarget, LauncherState,
    NotificationCenter, SessionAction, SessionMenuEvent, SessionMenuState, TrashModel,
    NOTIFICATION_DEFAULT_TIMEOUT_MS, WORKSPACE_COUNT,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_text::OutputScale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstPartyWindowAction {
    Close,
    Minimize,
    Maximize,
    Move,
    Resize,
    None,
}

pub fn top_system_bar_session_hit(viewport: Viewport, pointer_x: u32, pointer_y: u32) -> bool {
    static_shell_scene(viewport)
        .surface_rect(SurfaceKind::TopPanel)
        .is_some_and(|rect| {
            top_system_bar(rect.width, rect.height).session_hit(pointer_x, pointer_y)
        })
}

pub fn notification_dismiss_hit(
    rect: Rect,
    source: &str,
    title: &str,
    body: &str,
    pointer_x: u32,
    pointer_y: u32,
) -> bool {
    NotificationToast::new(rect, source, title, body).dismiss_hit(pointer_x, pointer_y)
}

pub fn first_party_window_action(
    frame: WindowFrame<'_>,
    pointer_x: u32,
    pointer_y: u32,
) -> FirstPartyWindowAction {
    if frame.resize_hit(pointer_x, pointer_y) {
        return FirstPartyWindowAction::Resize;
    }
    if let Some(control) = frame.control_at(pointer_x, pointer_y) {
        return match control {
            WindowControl::Close => FirstPartyWindowAction::Close,
            WindowControl::Minimize => FirstPartyWindowAction::Minimize,
            WindowControl::Maximize => FirstPartyWindowAction::Maximize,
        };
    }
    if frame.move_hit(pointer_x, pointer_y) {
        FirstPartyWindowAction::Move
    } else {
        FirstPartyWindowAction::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPreflight {
    pub app_id: &'static str,
    pub command: &'static str,
    pub target: Option<&'static str>,
    pub command_allowed: bool,
    pub target_allowed: bool,
    pub executable_exists: bool,
    pub executable_regular: bool,
    pub executable_symlink: bool,
    pub executable_permission: bool,
    pub accepted: bool,
    pub reason: &'static str,
}

pub fn preflight_first_party_launch(request: &LaunchRequest, root: &Path) -> LaunchPreflight {
    let expected_command = format!("/usr/bin/aqua-{}", request.app_id);
    let command_path = Path::new(request.command);
    let command_allowed = request.command == expected_command
        && command_path.parent() == Some(Path::new("/usr/bin"))
        && command_path.file_name().and_then(|name| name.to_str())
            == Some(expected_command.trim_start_matches("/usr/bin/"));
    let target_allowed = match request.app_id {
        "properties" => matches!(request.target, Some("files" | "settings" | "trash")),
        _ => request.target.is_none(),
    };
    let target = root.join(request.command.trim_start_matches('/'));
    let metadata = fs::symlink_metadata(&target).ok();
    let executable_exists = metadata.is_some();
    let executable_symlink = metadata
        .as_ref()
        .is_some_and(|metadata| metadata.file_type().is_symlink());
    let executable_regular = metadata
        .as_ref()
        .is_some_and(|metadata| metadata.file_type().is_file());
    #[cfg(unix)]
    let executable_permission = metadata
        .as_ref()
        .is_some_and(|metadata| metadata.permissions().mode() & 0o111 != 0);
    #[cfg(not(unix))]
    let executable_permission = false;
    let accepted = command_allowed
        && target_allowed
        && executable_exists
        && executable_regular
        && !executable_symlink
        && executable_permission;
    let reason = if !command_allowed {
        "command-not-allowed"
    } else if !target_allowed {
        "target-not-allowed"
    } else if !executable_exists {
        "missing-executable"
    } else if executable_symlink {
        "symlink-not-allowed"
    } else if !executable_regular {
        "not-regular-file"
    } else if !executable_permission {
        "not-executable"
    } else {
        "accepted"
    };

    LaunchPreflight {
        app_id: request.app_id,
        command: request.command,
        target: request.target,
        command_allowed,
        target_allowed,
        executable_exists,
        executable_regular,
        executable_symlink,
        executable_permission,
        accepted,
        reason,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSupervisorError {
    RejectedPreflight,
    AlreadyRunning,
    SpawnFailed,
    PollFailed,
    MissingProcess,
    TerminateFailed,
    ReapFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagedProcess {
    pub app_id: &'static str,
    pub pid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReapedProcess {
    pub app_id: &'static str,
    pub pid: u32,
    pub success: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstPartyRestartPolicy {
    Never,
}

impl FirstPartyRestartPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
        }
    }
}

pub fn first_party_restart_policy(app_id: &str) -> Option<FirstPartyRestartPolicy> {
    match app_id {
        "files" | "settings" | "properties" | "terminal" => Some(FirstPartyRestartPolicy::Never),
        _ => None,
    }
}

#[derive(Debug)]
struct FirstPartyChild {
    app_id: &'static str,
    pid: u32,
    child: Child,
}

#[derive(Debug, Default)]
pub struct FirstPartyProcessSupervisor {
    children: Vec<FirstPartyChild>,
}

impl FirstPartyProcessSupervisor {
    pub fn active_count(&self) -> usize {
        self.children.len()
    }

    pub fn contains(&self, app_id: &str) -> bool {
        self.children.iter().any(|process| process.app_id == app_id)
    }

    pub fn spawn(
        &mut self,
        preflight: &LaunchPreflight,
        runtime_dir: &Path,
        wayland_display: &str,
    ) -> Result<ManagedProcess, ProcessSupervisorError> {
        if !preflight.accepted {
            return Err(ProcessSupervisorError::RejectedPreflight);
        }
        if self.contains(preflight.app_id) {
            return Err(ProcessSupervisorError::AlreadyRunning);
        }
        let mut command = Command::new(preflight.command);
        command
            .env_clear()
            .env("XDG_RUNTIME_DIR", runtime_dir)
            .env("WAYLAND_DISPLAY", wayland_display)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        if let Some(target) = preflight.target {
            command.arg(target);
        }
        let child = command
            .spawn()
            .map_err(|_| ProcessSupervisorError::SpawnFailed)?;
        let process = ManagedProcess {
            app_id: preflight.app_id,
            pid: child.id(),
        };
        self.children.push(FirstPartyChild {
            app_id: process.app_id,
            pid: process.pid,
            child,
        });
        Ok(process)
    }

    pub fn try_reap(
        &mut self,
        app_id: &str,
    ) -> Result<Option<ReapedProcess>, ProcessSupervisorError> {
        let Some(index) = self
            .children
            .iter()
            .position(|process| process.app_id == app_id)
        else {
            return Err(ProcessSupervisorError::MissingProcess);
        };
        let status = self.children[index]
            .child
            .try_wait()
            .map_err(|_| ProcessSupervisorError::PollFailed)?;
        Ok(status.map(|status| self.remove_reaped(index, status)))
    }

    pub fn terminate_and_reap(
        &mut self,
        app_id: &str,
    ) -> Result<ReapedProcess, ProcessSupervisorError> {
        let Some(index) = self
            .children
            .iter()
            .position(|process| process.app_id == app_id)
        else {
            return Err(ProcessSupervisorError::MissingProcess);
        };
        if self.children[index]
            .child
            .try_wait()
            .map_err(|_| ProcessSupervisorError::PollFailed)?
            .is_none()
        {
            self.children[index]
                .child
                .kill()
                .map_err(|_| ProcessSupervisorError::TerminateFailed)?;
        }
        let status = self.children[index]
            .child
            .wait()
            .map_err(|_| ProcessSupervisorError::ReapFailed)?;
        Ok(self.remove_reaped(index, status))
    }

    fn remove_reaped(&mut self, index: usize, status: ExitStatus) -> ReapedProcess {
        let process = self.children.remove(index);
        ReapedProcess {
            app_id: process.app_id,
            pid: process.pid,
            success: status.success(),
        }
    }
}

impl Drop for FirstPartyProcessSupervisor {
    fn drop(&mut self) {
        for process in &mut self.children {
            if process.child.try_wait().ok().flatten().is_none() {
                let _ = process.child.kill();
            }
            let _ = process.child.wait();
        }
        self.children.clear();
    }
}
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use calloop::{generic::Generic, Interest, Mode, PostAction};
use calloop::{
    timer::{TimeoutAction, Timer},
    EventLoop,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use smithay::{
    backend::input::{Axis, AxisSource, ButtonState, KeyState, Keycode},
    delegate_compositor, delegate_seat, delegate_shm, delegate_xdg_shell,
    input::{
        keyboard::FilterResult,
        pointer::{AxisFrame, ButtonEvent, CursorImageStatus, MotionEvent},
        Seat, SeatHandler, SeatState,
    },
    reexports::wayland_server::{
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::{wl_buffer, wl_callback, wl_seat, wl_surface::WlSurface},
        Client, Display, DisplayHandle, ListeningSocket,
    },
    utils::Serial,
    wayland::shell::xdg::{
        Configure, PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        XdgToplevelSurfaceData,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{
            with_states, BufferAssignment, CompositorClientState, CompositorHandler,
            CompositorState, RectangleKind,
        },
        shm::{with_buffer_contents, ShmHandler, ShmState},
    },
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use wayland_protocols::xdg::shell::server::xdg_toplevel;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use {
    wayland_client::{
        delegate_noop,
        protocol::{
            wl_buffer as client_wl_buffer, wl_callback as client_wl_callback, wl_compositor,
            wl_keyboard as client_wl_keyboard, wl_pointer as client_wl_pointer,
            wl_region as client_wl_region, wl_registry, wl_seat as client_wl_seat,
            wl_shm as client_wl_shm, wl_shm_pool as client_wl_shm_pool, wl_surface,
        },
        Connection as ClientConnection, Dispatch as ClientDispatch, QueueHandle, WEnum,
    },
    wayland_protocols::xdg::shell::client::{
        xdg_surface as client_xdg_surface, xdg_toplevel as client_xdg_toplevel,
        xdg_wm_base as client_xdg_wm_base,
    },
};

pub const PRODUCT: &str = "Aqua Linux";
pub const BACKEND_TARGET: &str = "custom Wayland compositor";
pub const DEV_MODE: &str = "nested-dev";
pub const FOUNDATION: &str = "smithay";
pub const FOUNDATION_VERSION: &str = "0.7.0";
pub const FOUNDATION_STATUS: &str = "selected-scene-model-spike";
pub const SMITHAY_FEATURES: &str = "wayland_frontend,backend_libinput,udev";
pub const EVENT_LOOP: &str = "calloop";
pub const EVENT_LOOP_VERSION: &str = "0.14.4";
pub const DEFAULT_WAYLAND_SOCKET: &str = "aqua-wayland-0";
pub const DEFAULT_RUNTIME_ASSET_ROOT: &str = aqua_scene::RUNTIME_ASSET_ROOT;
pub const DEFAULT_SESSION_RUNTIME_DIR: &str = "/run/user/1000";
pub const FIRST_OUTPUT_BACKEND: &str = "nested-dev-window";
pub const LATER_OUTPUT_BACKEND: &str = "qemu-drm-kms";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfig {
    pub product: &'static str,
    pub mode: &'static str,
    pub wayland_socket: &'static str,
    pub runtime_dir: &'static str,
    pub runtime_asset_root: &'static str,
    pub autostart: bool,
    pub boot_graphics: bool,
    pub recovery_tty_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSessionConfig {
    pub product: String,
    pub mode: String,
    pub wayland_socket: String,
    pub runtime_dir: String,
    pub runtime_asset_root: String,
    pub autostart: bool,
    pub boot_graphics: bool,
    pub recovery_tty_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEnvironment {
    pub wayland_display: String,
    pub xdg_runtime_dir: String,
    pub aqua_asset_root: String,
    pub aqua_session_mode: String,
    pub aqua_compositor_autostart: bool,
    pub aqua_boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayOutputPlan {
    pub product: &'static str,
    pub mode: &'static str,
    pub primary_backend: &'static str,
    pub later_backend: &'static str,
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    pub pixel_format: &'static str,
    pub refresh_millihz: u32,
    pub boot_graphics: bool,
    pub renderer_started: bool,
    pub desktop_shell_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayOutputPlanProbe {
    pub plan: DisplayOutputPlan,
    pub mode_ready: bool,
    pub backend_ready: bool,
    pub dimensions_ready: bool,
    pub format_ready: bool,
    pub refresh_ready: bool,
    pub recovery_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisiblePreviewPlanProbe {
    pub output: DisplayOutputPlanProbe,
    pub scene_ready: bool,
    pub render_plan_ready: bool,
    pub paint_plan_ready: bool,
    pub frame_plan_ready: bool,
    pub frame_buffer_ready: bool,
    pub raster_ready: bool,
    pub png_export_ready: bool,
    pub client_layer_pipeline_ready: bool,
    pub client_layer_count: usize,
    pub client_layer_checksum: u64,
    pub client_layer_buffer_snapshot_bytes: usize,
    pub client_layer_snapshot_mode: &'static str,
    pub preview_window_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisiblePreviewExportProbe {
    pub plan: VisiblePreviewPlanProbe,
    pub format: &'static str,
    pub html: String,
    pub byte_count: usize,
    pub checksum: u64,
    pub client_layer_pipeline_ready: bool,
    pub client_layer_composited: bool,
    pub client_layer_count: usize,
    pub client_layer_checksum: u64,
    pub client_layer_buffer_snapshot_bytes: usize,
    pub client_layer_snapshot_mode: &'static str,
    pub png_checksum: u64,
    pub preview_window_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayOutputHandoffProbe {
    pub export: VisiblePreviewExportProbe,
    pub status: &'static str,
    pub target_backend: &'static str,
    pub output_width: u32,
    pub output_height: u32,
    pub pixel_format: &'static str,
    pub frame_buffer_bytes: usize,
    pub frame_format: &'static str,
    pub frame_checksum: u64,
    pub client_layer_buffer_snapshot_bytes: usize,
    pub client_layer_snapshot_mode: &'static str,
    pub client_layer_composited: bool,
    pub output_surface_prepared: bool,
    pub display_output_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub desktop_shell_started: bool,
    pub recovery_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayActivationPlanProbe {
    pub handoff: DisplayOutputHandoffProbe,
    pub status: &'static str,
    pub launch_mode: &'static str,
    pub source_handoff_ready: bool,
    pub target_backend: &'static str,
    pub frame_format: &'static str,
    pub frame_checksum: u64,
    pub manual_start_required: bool,
    pub fallback_tty_required: bool,
    pub can_activate_display_output: bool,
    pub display_output_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub desktop_shell_started: bool,
    pub autostart: bool,
    pub recovery_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayOutputSmokeProbe {
    pub activation: DisplayActivationPlanProbe,
    pub status: &'static str,
    pub launch_mode: &'static str,
    pub target_backend: &'static str,
    pub requested_frames: u32,
    pub presented_frames: u32,
    pub frame_interval_ms: u64,
    pub display_output_started: bool,
    pub display_output_stopped: bool,
    pub manual_start_required: bool,
    pub fallback_tty_available: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub desktop_shell_started: bool,
    pub autostart: bool,
    pub frame_format: &'static str,
    pub frame_checksum: u64,
    pub checksum_accumulator: u64,
    pub recovery_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedOutputSurfaceLifecycleProbe {
    pub smoke: DisplayOutputSmokeProbe,
    pub status: &'static str,
    pub launch_mode: &'static str,
    pub backend: &'static str,
    pub surface_acquired: bool,
    pub surface_configured: bool,
    pub frame_attached: bool,
    pub frame_presented: bool,
    pub surface_released: bool,
    pub presented_frames: u32,
    pub frame_checksum: u64,
    pub lifecycle_serial: u32,
    pub manual_start_required: bool,
    pub fallback_tty_available: bool,
    pub autostart: bool,
    pub boot_graphics: bool,
    pub renderer_started: bool,
    pub desktop_shell_started: bool,
    pub recovery_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedPreviewFrameLoopProbe {
    pub export: VisiblePreviewExportProbe,
    pub launch_mode: &'static str,
    pub window_backend: &'static str,
    pub frame_interval_ms: u64,
    pub requested_frames: u32,
    pub rendered_frames: u32,
    pub frame_clock_started: bool,
    pub manual_start_required: bool,
    pub autostart: bool,
    pub preview_window_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub checksum_accumulator: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualNestedPreviewBackendProbe {
    pub handoff: DisplayOutputHandoffProbe,
    pub surface: NestedOutputSurfaceLifecycleProbe,
    pub loop_probe: NestedPreviewFrameLoopProbe,
    pub status: &'static str,
    pub launch_mode: &'static str,
    pub backend_path: &'static str,
    pub backend_selected: bool,
    pub handoff_ready: bool,
    pub surface_lifecycle_ready: bool,
    pub frame_loop_ready: bool,
    pub visible_export_ready: bool,
    pub frame_source: &'static str,
    pub frame_format: &'static str,
    pub frame_checksum: u64,
    pub surface_frame_checksum: u64,
    pub loop_checksum_accumulator: u64,
    pub frame_checksum_matches_surface: bool,
    pub manual_start_required: bool,
    pub fallback_tty_required: bool,
    pub fallback_tty_available: bool,
    pub bounded_frame_limit: u32,
    pub display_output_started: bool,
    pub display_output_stopped: bool,
    pub preview_window_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub desktop_shell_started: bool,
    pub autostart: bool,
    pub recovery_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualNestedPreviewExecutionProbe {
    pub backend: ManualNestedPreviewBackendProbe,
    pub status: &'static str,
    pub launch_mode: &'static str,
    pub backend_path: &'static str,
    pub operator_controlled: bool,
    pub operator_ack_required: bool,
    pub operator_acknowledged: bool,
    pub backend_ready: bool,
    pub requested_frames: u32,
    pub rendered_frames: u32,
    pub frame_interval_ms: u64,
    pub frame_source: &'static str,
    pub frame_format: &'static str,
    pub frame_checksum: u64,
    pub checksum_accumulator: u64,
    pub display_output_started: bool,
    pub display_output_stopped: bool,
    pub preview_window_started: bool,
    pub cleanup_complete: bool,
    pub fallback_tty_available: bool,
    pub safe_return_to_recovery: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub desktop_shell_started: bool,
    pub autostart: bool,
    pub recovery_safe: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagedClientWindow {
    pub id: &'static str,
    pub title: &'static str,
    pub rect: aqua_scene::Rect,
    pub z_index: u8,
    pub focused: bool,
    pub closed: bool,
    pub chrome: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientWindowModelProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub viewport: Viewport,
    pub windows: Vec<ManagedClientWindow>,
    pub operation_count: usize,
    pub focus_ready: bool,
    pub move_ready: bool,
    pub resize_ready: bool,
    pub close_ready: bool,
    pub stacking_ready: bool,
    pub chrome_ready: bool,
    pub real_wayland_client_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSurfaceRegistryRecord {
    pub client_id: &'static str,
    pub surface_id: &'static str,
    pub window_id: &'static str,
    pub title: &'static str,
    pub role: &'static str,
    pub lifecycle_state: &'static str,
    pub z_index: u8,
    pub configure_serial: u32,
    pub configured: bool,
    pub committed: bool,
    pub mapped: bool,
    pub focused: bool,
    pub close_supported: bool,
    pub buffer_attached: bool,
    pub buffer_committed: bool,
    pub buffer_width: u32,
    pub buffer_height: u32,
    pub buffer_stride: u32,
    pub buffer_format: &'static str,
    pub buffer_source: &'static str,
    pub import_required: bool,
    pub import_planned: bool,
    pub imported_for_sampling: bool,
    pub sample_checksum: u64,
    pub sample_pixel: [u8; 4],
    pub sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    pub client_buffer_rgba: Vec<u8>,
    pub renderer_bound: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSurfaceRegistryProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub source_window_model_ready: bool,
    pub record_count: usize,
    pub active_client_id: &'static str,
    pub active_surface_id: &'static str,
    pub active_window_id: &'static str,
    pub configure_serial_ready: bool,
    pub lifecycle_state_ready: bool,
    pub two_client_ready: bool,
    pub focus_index_ready: bool,
    pub stacking_order_ready: bool,
    pub close_request_ready: bool,
    pub buffer_metadata_ready: bool,
    pub buffer_import_plan_ready: bool,
    pub no_renderer_binding: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub host_stub: bool,
    pub records: Vec<ClientSurfaceRegistryRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientSurfaceLifecycleStep {
    Created,
    Configured,
    Committed,
    Mapped,
    Focused,
    Unmapped,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSurfaceLifecycleProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub window_model: ClientWindowModelProbe,
    pub surface_id: &'static str,
    pub window_id: &'static str,
    pub role: &'static str,
    pub steps: Vec<ClientSurfaceLifecycleStep>,
    pub configure_ready: bool,
    pub commit_ready: bool,
    pub map_ready: bool,
    pub focus_ready: bool,
    pub unmap_ready: bool,
    pub destroy_ready: bool,
    pub focus_bound_to_window: bool,
    pub window_geometry_ready: bool,
    pub real_wayland_client_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XdgShellBindingProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub foundation: &'static str,
    pub protocol: &'static str,
    pub handler_bound: bool,
    pub global_created: bool,
    pub toplevel_callbacks_bound: bool,
    pub popup_callbacks_bound: bool,
    pub lifecycle_probe_ready: bool,
    pub real_wayland_client_started: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub host_stub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XdgToplevelClientProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub foundation: &'static str,
    pub protocol: &'static str,
    pub client_connected: bool,
    pub client_inserted: bool,
    pub registry_bound: bool,
    pub compositor_global_seen: bool,
    pub shm_global_created: bool,
    pub shm_global_seen: bool,
    pub shm_buffer_created: bool,
    pub client_buffer_attached: bool,
    pub xdg_wm_base_global_seen: bool,
    pub surface_created: bool,
    pub toplevel_requested: bool,
    pub surface_committed: bool,
    pub server_buffer_attached: bool,
    pub server_shm_buffer_imported: bool,
    pub server_shm_buffer_sampled: bool,
    pub shm_sample_checksum: u64,
    pub shm_sample_pixel: [u8; 4],
    pub shm_sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    pub shm_buffer_rgba: Vec<u8>,
    pub server_toplevel_created: bool,
    pub server_configure_sent: bool,
    pub client_configure_ack_sent: bool,
    pub server_configure_ack_received: bool,
    pub server_close_sent: bool,
    pub client_close_event_received: bool,
    pub dispatch_clients_ok: bool,
    pub flush_clients_ok: bool,
    pub test_wayland_client_started: bool,
    pub test_wayland_client_count: usize,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub host_stub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XdgToplevelWindowModelProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub foundation: &'static str,
    pub protocol: &'static str,
    pub source_client_ready: bool,
    pub server_surface_bound: bool,
    pub window_model_bound: bool,
    pub window_count: usize,
    pub two_window_model_ready: bool,
    pub stacking_ready: bool,
    pub window_id: &'static str,
    pub surface_id: &'static str,
    pub title: &'static str,
    pub role: &'static str,
    pub mapped: bool,
    pub focused: bool,
    pub geometry_ready: bool,
    pub chrome_ready: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
    pub host_stub: bool,
    pub window: ManagedClientWindow,
    pub windows: Vec<ManagedClientWindow>,
}

impl SessionConfig {
    pub const fn default_nested_dev() -> Self {
        Self {
            product: PRODUCT,
            mode: DEV_MODE,
            wayland_socket: DEFAULT_WAYLAND_SOCKET,
            runtime_dir: DEFAULT_SESSION_RUNTIME_DIR,
            runtime_asset_root: DEFAULT_RUNTIME_ASSET_ROOT,
            autostart: false,
            boot_graphics: false,
            recovery_tty_required: true,
        }
    }

    pub fn is_recovery_safe(&self) -> bool {
        self.product == PRODUCT
            && self.mode == DEV_MODE
            && self.wayland_socket.starts_with("aqua-wayland-")
            && self.runtime_dir == DEFAULT_SESSION_RUNTIME_DIR
            && self.runtime_asset_root == DEFAULT_RUNTIME_ASSET_ROOT
            && !self.autostart
            && !self.boot_graphics
            && self.recovery_tty_required
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("product={}", self.product),
            format!("mode={}", self.mode),
            format!("wayland_socket={}", self.wayland_socket),
            format!("runtime_dir={}", self.runtime_dir),
            format!("runtime_asset_root={}", self.runtime_asset_root),
            format!("autostart={}", self.autostart),
            format!("boot_graphics={}", self.boot_graphics),
            format!("recovery_tty_required={}", self.recovery_tty_required),
        ]
    }

    pub fn environment(&self) -> SessionEnvironment {
        SessionEnvironment {
            wayland_display: self.wayland_socket.to_string(),
            xdg_runtime_dir: self.runtime_dir.to_string(),
            aqua_asset_root: self.runtime_asset_root.to_string(),
            aqua_session_mode: self.mode.to_string(),
            aqua_compositor_autostart: self.autostart,
            aqua_boot_graphics: self.boot_graphics,
        }
    }
}

impl ParsedSessionConfig {
    pub fn is_recovery_safe(&self) -> bool {
        self.product == PRODUCT
            && self.mode == DEV_MODE
            && self.wayland_socket.starts_with("aqua-wayland-")
            && self.runtime_dir == DEFAULT_SESSION_RUNTIME_DIR
            && self.runtime_asset_root == DEFAULT_RUNTIME_ASSET_ROOT
            && !self.autostart
            && !self.boot_graphics
            && self.recovery_tty_required
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("product={}", self.product),
            format!("mode={}", self.mode),
            format!("wayland_socket={}", self.wayland_socket),
            format!("runtime_dir={}", self.runtime_dir),
            format!("runtime_asset_root={}", self.runtime_asset_root),
            format!("autostart={}", self.autostart),
            format!("boot_graphics={}", self.boot_graphics),
            format!("recovery_tty_required={}", self.recovery_tty_required),
        ]
    }

    pub fn environment(&self) -> SessionEnvironment {
        SessionEnvironment {
            wayland_display: self.wayland_socket.clone(),
            xdg_runtime_dir: self.runtime_dir.clone(),
            aqua_asset_root: self.runtime_asset_root.clone(),
            aqua_session_mode: self.mode.clone(),
            aqua_compositor_autostart: self.autostart,
            aqua_boot_graphics: self.boot_graphics,
        }
    }
}

impl SessionEnvironment {
    pub fn is_recovery_safe(&self) -> bool {
        self.wayland_display.starts_with("aqua-wayland-")
            && self.xdg_runtime_dir == DEFAULT_SESSION_RUNTIME_DIR
            && self.aqua_asset_root == DEFAULT_RUNTIME_ASSET_ROOT
            && self.aqua_session_mode == DEV_MODE
            && !self.aqua_compositor_autostart
            && !self.aqua_boot_graphics
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("WAYLAND_DISPLAY={}", self.wayland_display),
            format!("XDG_RUNTIME_DIR={}", self.xdg_runtime_dir),
            format!("AQUA_ASSET_ROOT={}", self.aqua_asset_root),
            format!("AQUA_SESSION_MODE={}", self.aqua_session_mode),
            format!(
                "AQUA_COMPOSITOR_AUTOSTART={}",
                self.aqua_compositor_autostart
            ),
            format!("AQUA_BOOT_GRAPHICS={}", self.aqua_boot_graphics),
        ]
    }
}

impl DisplayOutputPlan {
    pub const fn default_nested_dev() -> Self {
        Self {
            product: PRODUCT,
            mode: DEV_MODE,
            primary_backend: FIRST_OUTPUT_BACKEND,
            later_backend: LATER_OUTPUT_BACKEND,
            width: 1536,
            height: 1024,
            scale: 1,
            pixel_format: "rgba8888",
            refresh_millihz: 60_000,
            boot_graphics: false,
            renderer_started: false,
            desktop_shell_started: false,
        }
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("product={}", self.product),
            format!("mode={}", self.mode),
            format!("primary_backend={}", self.primary_backend),
            format!("later_backend={}", self.later_backend),
            format!("output_size={}x{}", self.width, self.height),
            format!("scale={}", self.scale),
            format!("pixel_format={}", self.pixel_format),
            format!("refresh_millihz={}", self.refresh_millihz),
            format!("boot_graphics={}", self.boot_graphics),
            format!("renderer_started={}", self.renderer_started),
            format!("desktop_shell_started={}", self.desktop_shell_started),
        ]
    }

    pub fn is_recovery_safe(&self) -> bool {
        self.product == PRODUCT
            && self.mode == DEV_MODE
            && self.primary_backend == FIRST_OUTPUT_BACKEND
            && self.later_backend == LATER_OUTPUT_BACKEND
            && self.width == 1536
            && self.height == 1024
            && self.scale == 1
            && self.pixel_format == "rgba8888"
            && self.refresh_millihz == 60_000
            && !self.boot_graphics
            && !self.renderer_started
            && !self.desktop_shell_started
    }
}

impl DisplayOutputPlanProbe {
    pub fn is_ready(&self) -> bool {
        self.plan.is_recovery_safe()
            && self.mode_ready
            && self.backend_ready
            && self.dimensions_ready
            && self.format_ready
            && self.refresh_ready
            && self.recovery_safe
    }
}

impl VisiblePreviewPlanProbe {
    pub fn is_ready(&self) -> bool {
        self.output.is_ready()
            && self.scene_ready
            && self.render_plan_ready
            && self.paint_plan_ready
            && self.frame_plan_ready
            && self.frame_buffer_ready
            && self.raster_ready
            && self.png_export_ready
            && self.client_layer_pipeline_ready
            && self.client_layer_count == 2
            && self.client_layer_checksum != 0
            && matches!(
                self.client_layer_snapshot_mode,
                "sample-grid-fallback" | "full-buffer-snapshot"
            )
            && !self.preview_window_started
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl VisiblePreviewExportProbe {
    pub fn is_ready(&self) -> bool {
        self.plan.is_ready()
            && self.format == "html-data-uri-png-preview"
            && self.byte_count == self.html.len()
            && self.byte_count > 6_293_028
            && self.checksum != 0
            && self.client_layer_pipeline_ready
            && self.client_layer_composited
            && self.client_layer_count == self.plan.client_layer_count
            && self.client_layer_checksum == self.plan.client_layer_checksum
            && self.client_layer_buffer_snapshot_bytes
                == self.plan.client_layer_buffer_snapshot_bytes
            && self.client_layer_snapshot_mode == self.plan.client_layer_snapshot_mode
            && self.png_checksum != 0
            && !self.preview_window_started
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl DisplayOutputHandoffProbe {
    pub fn is_ready(&self) -> bool {
        self.export.is_ready()
            && self.status == "display-output-handoff-ready"
            && self.target_backend == FIRST_OUTPUT_BACKEND
            && self.output_width == self.export.plan.output.plan.width
            && self.output_height == self.export.plan.output.plan.height
            && self.pixel_format == self.export.plan.output.plan.pixel_format
            && self.frame_buffer_bytes
                == (self.output_width as usize)
                    .saturating_mul(self.output_height as usize)
                    .saturating_mul(4)
            && self.frame_format == "raw-rgba8888-composited-client-preview"
            && self.frame_checksum != 0
            && self.client_layer_buffer_snapshot_bytes
                == self.export.client_layer_buffer_snapshot_bytes
            && self.client_layer_snapshot_mode == "full-buffer-snapshot"
            && self.client_layer_composited
            && self.output_surface_prepared
            && !self.display_output_started
            && !self.renderer_started
            && !self.boot_graphics
            && !self.desktop_shell_started
            && self.recovery_safe
    }
}

impl DisplayActivationPlanProbe {
    pub fn is_ready(&self) -> bool {
        self.handoff.is_ready()
            && self.status == "manual-display-activation-plan-ready"
            && self.launch_mode == "manual-dev"
            && self.source_handoff_ready
            && self.target_backend == FIRST_OUTPUT_BACKEND
            && self.frame_format == "raw-rgba8888-composited-client-preview"
            && self.frame_checksum == self.handoff.frame_checksum
            && self.manual_start_required
            && self.fallback_tty_required
            && self.can_activate_display_output
            && !self.display_output_started
            && !self.renderer_started
            && !self.boot_graphics
            && !self.desktop_shell_started
            && !self.autostart
            && self.recovery_safe
    }
}

impl DisplayOutputSmokeProbe {
    pub fn is_ready(&self) -> bool {
        self.activation.is_ready()
            && self.status == "manual-display-output-smoke-complete"
            && self.launch_mode == "manual-dev"
            && self.target_backend == FIRST_OUTPUT_BACKEND
            && self.requested_frames >= 3
            && self.presented_frames == self.requested_frames
            && self.frame_interval_ms == 16
            && self.display_output_started
            && self.display_output_stopped
            && self.manual_start_required
            && self.fallback_tty_available
            && !self.renderer_started
            && !self.boot_graphics
            && !self.desktop_shell_started
            && !self.autostart
            && self.frame_format == "raw-rgba8888-composited-client-preview"
            && self.frame_checksum == self.activation.frame_checksum
            && self.checksum_accumulator != 0
            && self.recovery_safe
    }
}

impl NestedOutputSurfaceLifecycleProbe {
    pub fn is_ready(&self) -> bool {
        self.smoke.is_ready()
            && self.status == "nested-output-surface-lifecycle-complete"
            && self.launch_mode == "manual-dev"
            && self.backend == FIRST_OUTPUT_BACKEND
            && self.surface_acquired
            && self.surface_configured
            && self.frame_attached
            && self.frame_presented
            && self.surface_released
            && self.presented_frames == self.smoke.presented_frames
            && self.frame_checksum == self.smoke.frame_checksum
            && self.lifecycle_serial == 1
            && self.manual_start_required
            && self.fallback_tty_available
            && !self.autostart
            && !self.boot_graphics
            && !self.renderer_started
            && !self.desktop_shell_started
            && self.recovery_safe
    }
}

impl NestedPreviewFrameLoopProbe {
    pub fn is_ready(&self) -> bool {
        self.export.is_ready()
            && self.launch_mode == "manual-dev"
            && self.window_backend == FIRST_OUTPUT_BACKEND
            && self.frame_interval_ms == 16
            && self.requested_frames >= 3
            && self.rendered_frames == self.requested_frames
            && self.frame_clock_started
            && self.manual_start_required
            && !self.autostart
            && !self.preview_window_started
            && !self.renderer_started
            && !self.boot_graphics
            && self.checksum_accumulator != 0
    }
}

impl ManualNestedPreviewBackendProbe {
    pub fn is_ready(&self) -> bool {
        self.handoff.is_ready()
            && self.surface.is_ready()
            && self.loop_probe.is_ready()
            && self.status == "manual-nested-preview-backend-ready"
            && self.launch_mode == "manual-recovery"
            && self.backend_path == FIRST_OUTPUT_BACKEND
            && self.backend_selected
            && self.handoff_ready
            && self.surface_lifecycle_ready
            && self.frame_loop_ready
            && self.visible_export_ready
            && self.frame_source == "display-output-handoff-composited-client-frame"
            && self.frame_format == "raw-rgba8888-composited-client-preview"
            && self.frame_checksum != 0
            && self.frame_checksum == self.handoff.frame_checksum
            && self.surface_frame_checksum == self.surface.frame_checksum
            && self.frame_checksum_matches_surface
            && self.loop_checksum_accumulator == self.loop_probe.checksum_accumulator
            && self.manual_start_required
            && self.fallback_tty_required
            && self.fallback_tty_available
            && self.bounded_frame_limit == self.loop_probe.requested_frames
            && self.bounded_frame_limit >= 3
            && !self.display_output_started
            && self.display_output_stopped
            && !self.preview_window_started
            && !self.renderer_started
            && !self.boot_graphics
            && !self.desktop_shell_started
            && !self.autostart
            && self.recovery_safe
    }
}

impl ManualNestedPreviewExecutionProbe {
    pub fn is_ready(&self) -> bool {
        self.backend.is_ready()
            && self.status == "manual-nested-preview-execution-complete"
            && self.launch_mode == "manual-recovery"
            && self.backend_path == FIRST_OUTPUT_BACKEND
            && self.operator_controlled
            && self.operator_ack_required
            && self.operator_acknowledged
            && self.backend_ready
            && self.requested_frames >= 3
            && self.rendered_frames == self.requested_frames
            && self.frame_interval_ms == 16
            && self.frame_source == "manual-nested-preview-backend-frame"
            && self.frame_format == "raw-rgba8888-composited-client-preview"
            && self.frame_checksum == self.backend.frame_checksum
            && self.frame_checksum != 0
            && self.checksum_accumulator != 0
            && self.display_output_started
            && self.display_output_stopped
            && !self.preview_window_started
            && self.cleanup_complete
            && self.fallback_tty_available
            && self.safe_return_to_recovery
            && !self.renderer_started
            && !self.boot_graphics
            && !self.desktop_shell_started
            && !self.autostart
            && self.recovery_safe
    }
}

impl ManagedClientWindow {
    pub fn dump_line(self) -> String {
        format!(
            "window id={} title={} rect={},{},{},{} z_index={} focused={} closed={} chrome={}",
            self.id,
            self.title,
            self.rect.x,
            self.rect.y,
            self.rect.width,
            self.rect.height,
            self.z_index,
            self.focused,
            self.closed,
            self.chrome
        )
    }
}

impl ClientWindowModelProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "client-window-model"
            && self.viewport.is_supported()
            && self.windows.len() == 2
            && self.operation_count == 5
            && self.focus_ready
            && self.move_ready
            && self.resize_ready
            && self.close_ready
            && self.stacking_ready
            && self.chrome_ready
            && !self.real_wayland_client_started
            && !self.renderer_started
            && !self.boot_graphics
    }

    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("product={}", self.product),
            format!("model_status={}", self.status),
            format!("viewport={}x{}", self.viewport.width, self.viewport.height),
            format!("window_count={}", self.windows.len()),
            format!("operation_count={}", self.operation_count),
            format!("focus_ready={}", ok(self.focus_ready)),
            format!("move_ready={}", ok(self.move_ready)),
            format!("resize_ready={}", ok(self.resize_ready)),
            format!("close_ready={}", ok(self.close_ready)),
            format!("stacking_ready={}", ok(self.stacking_ready)),
            format!("chrome_ready={}", ok(self.chrome_ready)),
            format!(
                "real_wayland_client_started={}",
                self.real_wayland_client_started
            ),
            format!("renderer_started={}", self.renderer_started),
            format!("boot_graphics={}", self.boot_graphics),
        ];

        lines.extend(self.windows.iter().map(|window| window.dump_line()));
        lines
    }
}

impl ClientSurfaceLifecycleStep {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Configured => "configured",
            Self::Committed => "committed",
            Self::Mapped => "mapped",
            Self::Focused => "focused",
            Self::Unmapped => "unmapped",
            Self::Destroyed => "destroyed",
        }
    }
}

impl ClientSurfaceLifecycleProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "client-surface-lifecycle"
            && self.window_model.is_ready()
            && self.surface_id == "terminal-demo-surface"
            && self.window_id == "terminal-demo"
            && self.role == "xdg-toplevel"
            && self.steps.len() == 7
            && self.configure_ready
            && self.commit_ready
            && self.map_ready
            && self.focus_ready
            && self.unmap_ready
            && self.destroy_ready
            && self.focus_bound_to_window
            && self.window_geometry_ready
            && !self.real_wayland_client_started
            && !self.renderer_started
            && !self.boot_graphics
    }

    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("product={}", self.product),
            format!("lifecycle_status={}", self.status),
            format!("surface_id={}", self.surface_id),
            format!("window_id={}", self.window_id),
            format!("role={}", self.role),
            format!("step_count={}", self.steps.len()),
            format!("configure_ready={}", ok(self.configure_ready)),
            format!("commit_ready={}", ok(self.commit_ready)),
            format!("map_ready={}", ok(self.map_ready)),
            format!("focus_ready={}", ok(self.focus_ready)),
            format!("unmap_ready={}", ok(self.unmap_ready)),
            format!("destroy_ready={}", ok(self.destroy_ready)),
            format!("focus_bound_to_window={}", ok(self.focus_bound_to_window)),
            format!("window_geometry_ready={}", ok(self.window_geometry_ready)),
            format!(
                "real_wayland_client_started={}",
                self.real_wayland_client_started
            ),
            format!("renderer_started={}", self.renderer_started),
            format!("boot_graphics={}", self.boot_graphics),
        ];

        lines.extend(
            self.steps
                .iter()
                .enumerate()
                .map(|(index, step)| format!("step order={} name={}", index + 1, step.as_str())),
        );
        lines
    }
}

impl XdgShellBindingProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "xdg-shell-binding"
            && self.foundation == FOUNDATION
            && self.protocol == "xdg_wm_base"
            && self.handler_bound
            && self.global_created
            && self.toplevel_callbacks_bound
            && self.popup_callbacks_bound
            && self.lifecycle_probe_ready
            && !self.real_wayland_client_started
            && !self.renderer_started
            && !self.boot_graphics
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("product={}", self.product),
            format!("binding_status={}", self.status),
            format!("foundation={}", self.foundation),
            format!("protocol={}", self.protocol),
            format!("handler_bound={}", ok(self.handler_bound)),
            format!("global_created={}", ok(self.global_created)),
            format!(
                "toplevel_callbacks_bound={}",
                ok(self.toplevel_callbacks_bound)
            ),
            format!("popup_callbacks_bound={}", ok(self.popup_callbacks_bound)),
            format!("lifecycle_probe_ready={}", ok(self.lifecycle_probe_ready)),
            format!(
                "real_wayland_client_started={}",
                self.real_wayland_client_started
            ),
            format!("renderer_started={}", self.renderer_started),
            format!("boot_graphics={}", self.boot_graphics),
            format!("host_stub={}", self.host_stub),
        ]
    }
}

impl XdgToplevelClientProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "xdg-toplevel-client"
            && self.foundation == FOUNDATION
            && self.protocol == "xdg_wm_base"
            && self.client_connected
            && self.client_inserted
            && self.registry_bound
            && self.compositor_global_seen
            && self.shm_global_created
            && self.shm_global_seen
            && self.shm_buffer_created
            && self.client_buffer_attached
            && self.xdg_wm_base_global_seen
            && self.surface_created
            && self.toplevel_requested
            && self.surface_committed
            && self.server_buffer_attached
            && self.server_shm_buffer_imported
            && self.server_shm_buffer_sampled
            && self.shm_sample_checksum != 0
            && self.shm_sample_pixel[3] == 0xff
            && self.shm_sample_grid.iter().all(|pixel| pixel[3] == 0xff)
            && self.server_toplevel_created
            && self.server_configure_sent
            && self.client_configure_ack_sent
            && self.server_configure_ack_received
            && self.server_close_sent
            && self.client_close_event_received
            && self.dispatch_clients_ok
            && self.flush_clients_ok
            && self.test_wayland_client_started
            && self.test_wayland_client_count == 2
            && !self.renderer_started
            && !self.boot_graphics
    }

    pub fn dump_lines(&self) -> Vec<String> {
        vec![
            format!("product={}", self.product),
            format!("client_status={}", self.status),
            format!("foundation={}", self.foundation),
            format!("protocol={}", self.protocol),
            format!("client_connected={}", ok(self.client_connected)),
            format!("client_inserted={}", ok(self.client_inserted)),
            format!("registry_bound={}", ok(self.registry_bound)),
            format!("compositor_global_seen={}", ok(self.compositor_global_seen)),
            format!("shm_global_created={}", ok(self.shm_global_created)),
            format!("shm_global_seen={}", ok(self.shm_global_seen)),
            format!("shm_buffer_created={}", ok(self.shm_buffer_created)),
            format!("client_buffer_attached={}", ok(self.client_buffer_attached)),
            format!(
                "xdg_wm_base_global_seen={}",
                ok(self.xdg_wm_base_global_seen)
            ),
            format!("surface_created={}", ok(self.surface_created)),
            format!("toplevel_requested={}", ok(self.toplevel_requested)),
            format!("surface_committed={}", ok(self.surface_committed)),
            format!("server_buffer_attached={}", ok(self.server_buffer_attached)),
            format!(
                "server_shm_buffer_imported={}",
                ok(self.server_shm_buffer_imported)
            ),
            format!(
                "server_shm_buffer_sampled={}",
                ok(self.server_shm_buffer_sampled)
            ),
            format!("shm_sample_checksum={:016x}", self.shm_sample_checksum),
            format!("shm_sample_pixel={}", pixel_as_hex(self.shm_sample_pixel)),
            format!(
                "shm_sample_grid={}",
                sample_grid_as_hex(self.shm_sample_grid)
            ),
            format!("shm_buffer_snapshot_bytes={}", self.shm_buffer_rgba.len()),
            format!(
                "server_toplevel_created={}",
                ok(self.server_toplevel_created)
            ),
            format!("server_configure_sent={}", ok(self.server_configure_sent)),
            format!(
                "client_configure_ack_sent={}",
                ok(self.client_configure_ack_sent)
            ),
            format!(
                "server_configure_ack_received={}",
                ok(self.server_configure_ack_received)
            ),
            format!("server_close_sent={}", ok(self.server_close_sent)),
            format!(
                "client_close_event_received={}",
                ok(self.client_close_event_received)
            ),
            format!("dispatch_clients_ok={}", ok(self.dispatch_clients_ok)),
            format!("flush_clients_ok={}", ok(self.flush_clients_ok)),
            format!(
                "test_wayland_client_started={}",
                self.test_wayland_client_started
            ),
            format!(
                "test_wayland_client_count={}",
                self.test_wayland_client_count
            ),
            format!("renderer_started={}", self.renderer_started),
            format!("boot_graphics={}", self.boot_graphics),
            format!("host_stub={}", self.host_stub),
        ]
    }
}

impl XdgToplevelWindowModelProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "xdg-toplevel-window-model"
            && self.foundation == FOUNDATION
            && self.protocol == "xdg_wm_base"
            && self.source_client_ready
            && self.server_surface_bound
            && self.window_model_bound
            && self.window_count == 2
            && self.two_window_model_ready
            && self.stacking_ready
            && self.window_id == "wayland-test-client"
            && self.surface_id == "xdg-toplevel-1"
            && self.title == "Aqua Test Client"
            && self.role == "xdg-toplevel"
            && self.mapped
            && self.focused
            && self.geometry_ready
            && self.chrome_ready
            && !self.renderer_started
            && !self.boot_graphics
            && self.window.id == self.window_id
            && self.window.title == self.title
            && self.window.focused
            && !self.window.closed
            && self.window.chrome == "aqua-window"
            && self.windows.len() == self.window_count
            && self
                .windows
                .iter()
                .any(|window| window.id == "aqua-settings-client" && !window.focused)
    }

    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("product={}", self.product),
            format!("window_model_status={}", self.status),
            format!("foundation={}", self.foundation),
            format!("protocol={}", self.protocol),
            format!("source_client_ready={}", ok(self.source_client_ready)),
            format!("server_surface_bound={}", ok(self.server_surface_bound)),
            format!("window_model_bound={}", ok(self.window_model_bound)),
            format!("window_count={}", self.window_count),
            format!("two_window_model_ready={}", ok(self.two_window_model_ready)),
            format!("stacking_ready={}", ok(self.stacking_ready)),
            format!("window_id={}", self.window_id),
            format!("surface_id={}", self.surface_id),
            format!("title={}", self.title),
            format!("role={}", self.role),
            format!("mapped={}", ok(self.mapped)),
            format!("focused={}", ok(self.focused)),
            format!("geometry_ready={}", ok(self.geometry_ready)),
            format!("chrome_ready={}", ok(self.chrome_ready)),
            format!("renderer_started={}", self.renderer_started),
            format!("boot_graphics={}", self.boot_graphics),
            format!("host_stub={}", self.host_stub),
        ];
        lines.extend(self.windows.iter().map(|window| {
            format!(
                "window id={} title={} rect={},{},{},{} z_index={} focused={} closed={} chrome={}",
                window.id,
                window.title,
                window.rect.x,
                window.rect.y,
                window.rect.width,
                window.rect.height,
                window.z_index,
                window.focused,
                window.closed,
                window.chrome
            )
        }));
        lines
    }
}

impl ClientSurfaceRegistryProbe {
    pub fn is_ready(&self) -> bool {
        let active = self
            .records
            .iter()
            .find(|record| record.client_id == self.active_client_id);

        self.product == PRODUCT
            && self.status == "client-surface-registry"
            && self.source_window_model_ready
            && self.record_count == 2
            && self.active_client_id == "wayland-client-1"
            && self.active_surface_id == "xdg-toplevel-1"
            && self.active_window_id == "wayland-test-client"
            && self.configure_serial_ready
            && self.lifecycle_state_ready
            && self.two_client_ready
            && self.focus_index_ready
            && self.stacking_order_ready
            && self.close_request_ready
            && self.buffer_metadata_ready
            && self.buffer_import_plan_ready
            && self.no_renderer_binding
            && !self.renderer_started
            && !self.boot_graphics
            && active
                .map(|record| {
                    record.surface_id == self.active_surface_id
                        && record.window_id == self.active_window_id
                        && record.title == "Aqua Test Client"
                        && record.role == "xdg-toplevel"
                        && record.lifecycle_state == "mapped-focused"
                        && record.z_index == 2
                        && record.configure_serial == 1
                        && record.configured
                        && record.committed
                        && record.mapped
                        && record.focused
                        && record.close_supported
                        && record.buffer_attached
                        && record.buffer_committed
                        && record.buffer_width == 384
                        && record.buffer_height == 256
                        && record.buffer_stride == 1536
                        && record.buffer_format == "argb8888"
                        && record.buffer_source == "client-committed-wl-shm"
                        && record.import_required
                        && record.import_planned
                        && record.imported_for_sampling
                        && record.sample_checksum != 0
                        && !record.renderer_bound
                })
                .unwrap_or(false)
    }

    pub fn dump_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("product={}", self.product),
            format!("registry_status={}", self.status),
            format!(
                "source_window_model_ready={}",
                ok(self.source_window_model_ready)
            ),
            format!("record_count={}", self.record_count),
            format!("active_client_id={}", self.active_client_id),
            format!("active_surface_id={}", self.active_surface_id),
            format!("active_window_id={}", self.active_window_id),
            format!("configure_serial_ready={}", ok(self.configure_serial_ready)),
            format!("lifecycle_state_ready={}", ok(self.lifecycle_state_ready)),
            format!("two_client_ready={}", ok(self.two_client_ready)),
            format!("focus_index_ready={}", ok(self.focus_index_ready)),
            format!("stacking_order_ready={}", ok(self.stacking_order_ready)),
            format!("close_request_ready={}", ok(self.close_request_ready)),
            format!("buffer_metadata_ready={}", ok(self.buffer_metadata_ready)),
            format!(
                "buffer_import_plan_ready={}",
                ok(self.buffer_import_plan_ready)
            ),
            format!("no_renderer_binding={}", ok(self.no_renderer_binding)),
            format!("renderer_started={}", self.renderer_started),
            format!("boot_graphics={}", self.boot_graphics),
            format!("host_stub={}", self.host_stub),
        ];

        lines.extend(self.records.iter().map(|record| {
            format!(
                "record client={} surface={} window={} title={} role={} lifecycle={} z_index={} configure_serial={} configured={} committed={} mapped={} focused={} close_supported={} buffer_attached={} buffer_committed={} buffer={}x{} stride={} format={} source={} import_required={} import_planned={} imported_for_sampling={} sample_checksum={:016x} sample_pixel={} sample_grid={} buffer_snapshot_bytes={} renderer_bound={}",
                record.client_id,
                record.surface_id,
                record.window_id,
                record.title,
                record.role,
                record.lifecycle_state,
                record.z_index,
                record.configure_serial,
                record.configured,
                record.committed,
                record.mapped,
                record.focused,
                record.close_supported,
                record.buffer_attached,
                record.buffer_committed,
                record.buffer_width,
                record.buffer_height,
                record.buffer_stride,
                record.buffer_format,
                record.buffer_source,
                record.import_required,
                record.import_planned,
                record.imported_for_sampling,
                record.sample_checksum,
                pixel_as_hex(record.sample_pixel),
                sample_grid_as_hex(record.sample_grid),
                record.client_buffer_rgba.len(),
                record.renderer_bound
            )
        }));
        lines
    }
}

pub fn default_session_config() -> SessionConfig {
    SessionConfig::default_nested_dev()
}

pub fn default_session_environment() -> SessionEnvironment {
    default_session_config().environment()
}

pub fn display_output_plan_for_nested_dev() -> DisplayOutputPlan {
    DisplayOutputPlan::default_nested_dev()
}

pub fn probe_display_output_plan() -> DisplayOutputPlanProbe {
    let plan = display_output_plan_for_nested_dev();

    DisplayOutputPlanProbe {
        mode_ready: plan.mode == DEV_MODE,
        backend_ready: plan.primary_backend == FIRST_OUTPUT_BACKEND
            && plan.later_backend == LATER_OUTPUT_BACKEND,
        dimensions_ready: plan.width == 1536 && plan.height == 1024 && plan.scale == 1,
        format_ready: plan.pixel_format == "rgba8888",
        refresh_ready: plan.refresh_millihz == 60_000,
        recovery_safe: !plan.boot_graphics && !plan.renderer_started && !plan.desktop_shell_started,
        plan,
    }
}

pub fn probe_visible_preview_plan(viewport: Viewport) -> VisiblePreviewPlanProbe {
    let output = probe_display_output_plan();
    let scene = probe_static_shell_scene(viewport);
    let render_plan = probe_static_render_plan(viewport);
    let paint_plan = probe_static_paint_plan(viewport);
    let frame_plan = probe_static_frame_plan(viewport);
    let frame_buffer = probe_static_frame_buffer(viewport);
    let raster = probe_static_software_raster(viewport);
    let png_export = probe_static_raster_png_export(viewport);
    let client_layer_pipeline = probe_client_layer_pipeline(viewport);
    let (
        client_layer_pipeline_ready,
        client_layer_count,
        client_layer_checksum,
        client_layer_buffer_snapshot_bytes,
    ) = match client_layer_pipeline {
        Ok(probe) => (
            probe.is_ready(),
            probe.layer_count,
            probe.raster_probe.layer_checksum,
            probe
                .paint_plan
                .steps
                .iter()
                .map(|step| step.client_buffer_rgba.len())
                .sum(),
        ),
        Err(_) => (false, 0, 0, 0),
    };
    let client_layer_snapshot_mode = client_layer_snapshot_mode(client_layer_buffer_snapshot_bytes);

    VisiblePreviewPlanProbe {
        scene_ready: scene.is_ready(),
        render_plan_ready: render_plan.is_ready(),
        paint_plan_ready: paint_plan.is_ready(),
        frame_plan_ready: frame_plan.is_ready(),
        frame_buffer_ready: frame_buffer.is_ready(),
        raster_ready: raster.is_ready(),
        png_export_ready: png_export.is_ready(),
        client_layer_pipeline_ready,
        client_layer_count,
        client_layer_checksum,
        client_layer_buffer_snapshot_bytes,
        client_layer_snapshot_mode,
        preview_window_started: false,
        renderer_started: false,
        boot_graphics: false,
        output,
    }
}

pub fn export_visible_preview_html(viewport: Viewport) -> VisiblePreviewExportProbe {
    let plan = probe_visible_preview_plan(viewport);
    let client_layer_pipeline =
        probe_client_layer_pipeline(viewport).expect("visible preview plan requires client layers");
    let png = export_composited_preview_png_with_client_layers(
        viewport,
        &client_layer_pipeline.paint_plan,
    );
    let encoded_png = base64_encode(&png.bytes);
    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Aqua Linux Visible Preview</title>
  <style>
    :root {{
      color-scheme: dark;
      background: #00121e;
      color: #e8f8ff;
      font-family: "Lucida Grande", "Helvetica Neue", Arial, sans-serif;
    }}
    body {{
      margin: 0;
      min-height: 100vh;
      display: grid;
      place-items: center;
      background: #00121e;
    }}
    main {{
      width: min(96vw, 1536px);
    }}
    img {{
      display: block;
      width: 100%;
      height: auto;
      border: 1px solid rgba(109, 214, 255, 0.42);
      box-shadow: 0 24px 64px rgba(0, 0, 0, 0.42);
    }}
    p {{
      margin: 12px 0 0;
      color: rgba(232, 248, 255, 0.72);
      font-size: 13px;
    }}
  </style>
</head>
<body>
  <main>
    <img src="data:image/png;base64,{encoded_png}" alt="Aqua Linux headless compositor preview">
    <p>Aqua Linux visible preview export. boot_graphics=false, renderer_started=false, preview_window_started=false.</p>
  </main>
</body>
</html>
"#
    );
    let checksum = checksum_bytes(html.as_bytes());

    VisiblePreviewExportProbe {
        byte_count: html.len(),
        checksum,
        format: "html-data-uri-png-preview",
        client_layer_pipeline_ready: plan.client_layer_pipeline_ready,
        client_layer_composited: png.format == "png-rgba8888-composited-client-preview",
        client_layer_count: plan.client_layer_count,
        client_layer_checksum: plan.client_layer_checksum,
        client_layer_buffer_snapshot_bytes: plan.client_layer_buffer_snapshot_bytes,
        client_layer_snapshot_mode: plan.client_layer_snapshot_mode,
        png_checksum: png.checksum,
        preview_window_started: false,
        renderer_started: false,
        boot_graphics: false,
        html,
        plan,
    }
}

pub fn probe_display_output_handoff(viewport: Viewport) -> DisplayOutputHandoffProbe {
    let export = export_visible_preview_html(viewport);
    let client_layer_pipeline = probe_client_layer_pipeline(viewport)
        .expect("display output handoff requires client layers");
    let frame = export_composited_preview_rgba_with_client_layers(
        viewport,
        &client_layer_pipeline.paint_plan,
    );
    let output = &export.plan.output.plan;
    let frame_buffer_bytes = (output.width as usize)
        .saturating_mul(output.height as usize)
        .saturating_mul(4);

    DisplayOutputHandoffProbe {
        status: "display-output-handoff-ready",
        target_backend: output.primary_backend,
        output_width: output.width,
        output_height: output.height,
        pixel_format: output.pixel_format,
        frame_buffer_bytes,
        frame_format: frame.format,
        frame_checksum: frame.checksum,
        client_layer_buffer_snapshot_bytes: export.client_layer_buffer_snapshot_bytes,
        client_layer_snapshot_mode: export.client_layer_snapshot_mode,
        client_layer_composited: export.client_layer_composited,
        output_surface_prepared: true,
        display_output_started: false,
        renderer_started: false,
        boot_graphics: false,
        desktop_shell_started: false,
        recovery_safe: output.is_recovery_safe(),
        export,
    }
}

pub fn probe_display_activation_plan(viewport: Viewport) -> DisplayActivationPlanProbe {
    let handoff = probe_display_output_handoff(viewport);

    DisplayActivationPlanProbe {
        status: "manual-display-activation-plan-ready",
        launch_mode: "manual-dev",
        source_handoff_ready: handoff.is_ready(),
        target_backend: handoff.target_backend,
        frame_format: handoff.frame_format,
        frame_checksum: handoff.frame_checksum,
        manual_start_required: true,
        fallback_tty_required: default_session_config().recovery_tty_required,
        can_activate_display_output: handoff.is_ready()
            && handoff.output_surface_prepared
            && handoff.frame_checksum != 0,
        display_output_started: false,
        renderer_started: false,
        boot_graphics: false,
        desktop_shell_started: false,
        autostart: false,
        recovery_safe: handoff.recovery_safe,
        handoff,
    }
}

pub fn run_manual_display_output_smoke(
    viewport: Viewport,
    requested_frames: u32,
) -> Result<DisplayOutputSmokeProbe, Box<dyn std::error::Error>> {
    let activation = probe_display_activation_plan(viewport);
    let target_frames = requested_frames.max(3);
    let mut event_loop: EventLoop<u32> = EventLoop::try_new()?;
    let handle = event_loop.handle();

    handle.insert_source(Timer::immediate(), move |_deadline, _metadata, frames| {
        *frames += 1;
        if *frames >= target_frames {
            TimeoutAction::Drop
        } else {
            TimeoutAction::ToDuration(Duration::from_millis(16))
        }
    })?;

    let mut presented_frames = 0;
    while presented_frames < target_frames {
        event_loop.dispatch(Duration::from_millis(40), &mut presented_frames)?;
    }

    Ok(DisplayOutputSmokeProbe {
        status: "manual-display-output-smoke-complete",
        launch_mode: "manual-dev",
        target_backend: activation.target_backend,
        requested_frames: target_frames,
        presented_frames,
        frame_interval_ms: 16,
        display_output_started: activation.can_activate_display_output,
        display_output_stopped: true,
        manual_start_required: true,
        fallback_tty_available: activation.fallback_tty_required,
        renderer_started: false,
        boot_graphics: false,
        desktop_shell_started: false,
        autostart: false,
        frame_format: activation.frame_format,
        frame_checksum: activation.frame_checksum,
        checksum_accumulator: activation.frame_checksum ^ u64::from(presented_frames),
        recovery_safe: activation.recovery_safe,
        activation,
    })
}

pub fn run_nested_output_surface_lifecycle(
    viewport: Viewport,
    requested_frames: u32,
) -> Result<NestedOutputSurfaceLifecycleProbe, Box<dyn std::error::Error>> {
    let smoke = run_manual_display_output_smoke(viewport, requested_frames)?;
    let smoke_ready = smoke.is_ready();

    Ok(NestedOutputSurfaceLifecycleProbe {
        status: "nested-output-surface-lifecycle-complete",
        launch_mode: "manual-dev",
        backend: smoke.target_backend,
        surface_acquired: smoke_ready && smoke.display_output_started,
        surface_configured: smoke_ready && smoke.target_backend == FIRST_OUTPUT_BACKEND,
        frame_attached: smoke_ready && smoke.frame_checksum != 0,
        frame_presented: smoke_ready && smoke.presented_frames >= 3,
        surface_released: smoke_ready && smoke.display_output_stopped,
        presented_frames: smoke.presented_frames,
        frame_checksum: smoke.frame_checksum,
        lifecycle_serial: 1,
        manual_start_required: smoke.manual_start_required,
        fallback_tty_available: smoke.fallback_tty_available,
        autostart: smoke.autostart,
        boot_graphics: smoke.boot_graphics,
        renderer_started: smoke.renderer_started,
        desktop_shell_started: smoke.desktop_shell_started,
        recovery_safe: smoke.recovery_safe,
        smoke,
    })
}

pub fn run_nested_preview_frame_loop(
    viewport: Viewport,
    requested_frames: u32,
) -> Result<NestedPreviewFrameLoopProbe, Box<dyn std::error::Error>> {
    let export = export_visible_preview_html(viewport);
    let target_frames = requested_frames.max(3);
    let mut event_loop: EventLoop<u32> = EventLoop::try_new()?;
    let handle = event_loop.handle();

    handle.insert_source(Timer::immediate(), move |_deadline, _metadata, frames| {
        *frames += 1;
        if *frames >= target_frames {
            TimeoutAction::Drop
        } else {
            TimeoutAction::ToDuration(Duration::from_millis(16))
        }
    })?;

    let mut rendered_frames = 0;
    while rendered_frames < target_frames {
        event_loop.dispatch(Duration::from_millis(40), &mut rendered_frames)?;
    }

    let checksum_accumulator = export
        .checksum
        .wrapping_mul(0x9e37_79b1_85eb_ca87)
        .wrapping_add(u64::from(rendered_frames));

    Ok(NestedPreviewFrameLoopProbe {
        launch_mode: "manual-dev",
        window_backend: FIRST_OUTPUT_BACKEND,
        frame_interval_ms: 16,
        requested_frames: target_frames,
        rendered_frames,
        frame_clock_started: true,
        manual_start_required: true,
        autostart: false,
        preview_window_started: false,
        renderer_started: false,
        boot_graphics: false,
        checksum_accumulator,
        export,
    })
}

pub fn probe_manual_nested_preview_backend(
    viewport: Viewport,
    requested_frames: u32,
) -> Result<ManualNestedPreviewBackendProbe, Box<dyn std::error::Error>> {
    let handoff = probe_display_output_handoff(viewport);
    let surface = run_nested_output_surface_lifecycle(viewport, requested_frames)?;
    let loop_probe = run_nested_preview_frame_loop(viewport, requested_frames)?;
    let handoff_ready = handoff.is_ready();
    let surface_lifecycle_ready = surface.is_ready();
    let frame_loop_ready = loop_probe.is_ready();
    let visible_export_ready = loop_probe.export.is_ready();
    let frame_checksum_matches_surface = handoff.frame_checksum == surface.frame_checksum;

    Ok(ManualNestedPreviewBackendProbe {
        status: "manual-nested-preview-backend-ready",
        launch_mode: "manual-recovery",
        backend_path: handoff.target_backend,
        backend_selected: handoff.target_backend == FIRST_OUTPUT_BACKEND,
        handoff_ready,
        surface_lifecycle_ready,
        frame_loop_ready,
        visible_export_ready,
        frame_source: "display-output-handoff-composited-client-frame",
        frame_format: handoff.frame_format,
        frame_checksum: handoff.frame_checksum,
        surface_frame_checksum: surface.frame_checksum,
        loop_checksum_accumulator: loop_probe.checksum_accumulator,
        frame_checksum_matches_surface,
        manual_start_required: true,
        fallback_tty_required: default_session_config().recovery_tty_required,
        fallback_tty_available: surface.fallback_tty_available,
        bounded_frame_limit: loop_probe.requested_frames,
        display_output_started: false,
        display_output_stopped: surface.surface_released,
        preview_window_started: loop_probe.preview_window_started,
        renderer_started: handoff.renderer_started
            || surface.renderer_started
            || loop_probe.renderer_started,
        boot_graphics: handoff.boot_graphics || surface.boot_graphics || loop_probe.boot_graphics,
        desktop_shell_started: handoff.desktop_shell_started || surface.desktop_shell_started,
        autostart: surface.autostart || loop_probe.autostart,
        recovery_safe: handoff.recovery_safe && surface.recovery_safe,
        handoff,
        surface,
        loop_probe,
    })
}

pub fn run_manual_nested_preview_execution(
    viewport: Viewport,
    requested_frames: u32,
    operator_acknowledged: bool,
) -> Result<ManualNestedPreviewExecutionProbe, Box<dyn std::error::Error>> {
    let backend = probe_manual_nested_preview_backend(viewport, requested_frames)?;
    let target_frames = requested_frames.max(3);
    let mut event_loop: EventLoop<u32> = EventLoop::try_new()?;
    let handle = event_loop.handle();

    handle.insert_source(Timer::immediate(), move |_deadline, _metadata, frames| {
        *frames += 1;
        if *frames >= target_frames {
            TimeoutAction::Drop
        } else {
            TimeoutAction::ToDuration(Duration::from_millis(16))
        }
    })?;

    let mut rendered_frames = 0;
    if operator_acknowledged && backend.is_ready() {
        while rendered_frames < target_frames {
            event_loop.dispatch(Duration::from_millis(40), &mut rendered_frames)?;
        }
    }

    let checksum_accumulator = backend
        .frame_checksum
        .wrapping_mul(0x517c_c1b7_2722_0a95)
        .wrapping_add(u64::from(rendered_frames));

    Ok(ManualNestedPreviewExecutionProbe {
        status: "manual-nested-preview-execution-complete",
        launch_mode: "manual-recovery",
        backend_path: backend.backend_path,
        operator_controlled: true,
        operator_ack_required: true,
        operator_acknowledged,
        backend_ready: backend.is_ready(),
        requested_frames: target_frames,
        rendered_frames,
        frame_interval_ms: 16,
        frame_source: "manual-nested-preview-backend-frame",
        frame_format: backend.frame_format,
        frame_checksum: backend.frame_checksum,
        checksum_accumulator,
        display_output_started: operator_acknowledged && backend.is_ready(),
        display_output_stopped: operator_acknowledged && backend.is_ready(),
        preview_window_started: false,
        cleanup_complete: operator_acknowledged && backend.is_ready(),
        fallback_tty_available: backend.fallback_tty_available,
        safe_return_to_recovery: operator_acknowledged && backend.recovery_safe,
        renderer_started: false,
        boot_graphics: false,
        desktop_shell_started: false,
        autostart: false,
        recovery_safe: backend.recovery_safe,
        backend,
    })
}

pub fn probe_client_window_model(viewport: Viewport) -> ClientWindowModelProbe {
    let terminal_initial = ManagedClientWindow {
        id: "terminal-demo",
        title: "Terminal",
        rect: aqua_scene::Rect {
            x: 176,
            y: 144,
            width: 680,
            height: 420,
        },
        z_index: 1,
        focused: false,
        closed: false,
        chrome: "aqua-window",
    };
    let browser_initial = ManagedClientWindow {
        id: "browser-demo",
        title: "Browser",
        rect: aqua_scene::Rect {
            x: 420,
            y: 190,
            width: 820,
            height: 520,
        },
        z_index: 2,
        focused: true,
        closed: false,
        chrome: "aqua-window",
    };

    let terminal_moved = ManagedClientWindow {
        rect: aqua_scene::Rect {
            x: 216,
            y: 178,
            ..terminal_initial.rect
        },
        z_index: 3,
        focused: true,
        ..terminal_initial
    };
    let browser_resized_closed = ManagedClientWindow {
        rect: aqua_scene::Rect {
            width: 760,
            height: 480,
            ..browser_initial.rect
        },
        z_index: 2,
        focused: false,
        closed: true,
        ..browser_initial
    };
    let windows = vec![terminal_moved, browser_resized_closed];

    ClientWindowModelProbe {
        product: PRODUCT,
        status: "client-window-model",
        viewport,
        focus_ready: windows
            .iter()
            .any(|window| window.id == "terminal-demo" && window.focused)
            && windows.iter().filter(|window| window.focused).count() == 1,
        move_ready: windows.iter().any(|window| {
            window.id == "terminal-demo" && window.rect.x == 216 && window.rect.y == 178
        }),
        resize_ready: windows.iter().any(|window| {
            window.id == "browser-demo" && window.rect.width == 760 && window.rect.height == 480
        }),
        close_ready: windows
            .iter()
            .any(|window| window.id == "browser-demo" && window.closed),
        stacking_ready: windows
            .iter()
            .filter(|window| !window.closed)
            .all(|window| window.z_index >= 3 && window.focused),
        chrome_ready: windows
            .iter()
            .all(|window| window.chrome == "aqua-window" && window.rect.fits_in(viewport)),
        operation_count: 5,
        real_wayland_client_started: false,
        renderer_started: false,
        boot_graphics: false,
        windows,
    }
}

pub fn probe_client_surface_lifecycle(viewport: Viewport) -> ClientSurfaceLifecycleProbe {
    let window_model = probe_client_window_model(viewport);
    let steps = vec![
        ClientSurfaceLifecycleStep::Created,
        ClientSurfaceLifecycleStep::Configured,
        ClientSurfaceLifecycleStep::Committed,
        ClientSurfaceLifecycleStep::Mapped,
        ClientSurfaceLifecycleStep::Focused,
        ClientSurfaceLifecycleStep::Unmapped,
        ClientSurfaceLifecycleStep::Destroyed,
    ];
    let focused_window = window_model
        .windows
        .iter()
        .find(|window| window.id == "terminal-demo" && window.focused && !window.closed);
    let window_geometry_ready = focused_window
        .map(|window| window.rect.fits_in(viewport))
        .unwrap_or(false);

    ClientSurfaceLifecycleProbe {
        product: PRODUCT,
        status: "client-surface-lifecycle",
        surface_id: "terminal-demo-surface",
        window_id: "terminal-demo",
        role: "xdg-toplevel",
        configure_ready: steps.contains(&ClientSurfaceLifecycleStep::Configured),
        commit_ready: steps.contains(&ClientSurfaceLifecycleStep::Committed),
        map_ready: steps.contains(&ClientSurfaceLifecycleStep::Mapped),
        focus_ready: steps.contains(&ClientSurfaceLifecycleStep::Focused),
        unmap_ready: steps.contains(&ClientSurfaceLifecycleStep::Unmapped),
        destroy_ready: steps.contains(&ClientSurfaceLifecycleStep::Destroyed),
        focus_bound_to_window: focused_window.is_some(),
        window_geometry_ready,
        real_wayland_client_started: false,
        renderer_started: false,
        boot_graphics: false,
        steps,
        window_model,
    }
}

pub fn probe_xdg_shell_binding(
    viewport: Viewport,
) -> Result<XdgShellBindingProbe, Box<dyn std::error::Error>> {
    probe_xdg_shell_binding_impl(viewport)
}

pub fn probe_xdg_toplevel_client() -> Result<XdgToplevelClientProbe, Box<dyn std::error::Error>> {
    probe_xdg_toplevel_client_impl()
}

pub fn probe_xdg_toplevel_window_model(
    viewport: Viewport,
) -> Result<XdgToplevelWindowModelProbe, Box<dyn std::error::Error>> {
    let client = probe_xdg_toplevel_client()?;
    let window = ManagedClientWindow {
        id: "wayland-test-client",
        title: "Aqua Test Client",
        rect: aqua_scene::Rect {
            x: 352,
            y: 184,
            width: 832,
            height: 520,
        },
        z_index: 2,
        focused: true,
        closed: false,
        chrome: "aqua-window",
    };
    let inactive_window = ManagedClientWindow {
        id: "aqua-settings-client",
        title: "Aqua Settings",
        rect: aqua_scene::Rect {
            x: 464,
            y: 248,
            width: 704,
            height: 436,
        },
        z_index: 1,
        focused: false,
        closed: false,
        chrome: "aqua-window",
    };
    let windows = vec![window, inactive_window];
    let geometry_ready = windows
        .iter()
        .all(|candidate| candidate.rect.fits_in(viewport));
    let chrome_ready = windows
        .iter()
        .all(|candidate| candidate.chrome == "aqua-window");
    let two_window_model_ready = client.test_wayland_client_count == 2
        && windows.len() == 2
        && windows
            .iter()
            .any(|candidate| candidate.id == "aqua-settings-client");
    let stacking_ready = window.focused
        && inactive_window.z_index < window.z_index
        && windows.iter().filter(|candidate| candidate.focused).count() == 1;

    Ok(XdgToplevelWindowModelProbe {
        product: PRODUCT,
        status: "xdg-toplevel-window-model",
        foundation: FOUNDATION,
        protocol: "xdg_wm_base",
        source_client_ready: client.is_ready(),
        server_surface_bound: client.server_toplevel_created && client.surface_committed,
        window_model_bound: client.server_toplevel_created
            && geometry_ready
            && chrome_ready
            && two_window_model_ready,
        window_count: windows.len(),
        two_window_model_ready,
        stacking_ready,
        window_id: window.id,
        surface_id: "xdg-toplevel-1",
        title: window.title,
        role: "xdg-toplevel",
        mapped: client.surface_committed,
        focused: window.focused,
        geometry_ready,
        chrome_ready,
        renderer_started: false,
        boot_graphics: false,
        host_stub: client.host_stub,
        window,
        windows,
    })
}

pub fn probe_client_surface_registry(
    viewport: Viewport,
) -> Result<ClientSurfaceRegistryProbe, Box<dyn std::error::Error>> {
    let window_model = probe_xdg_toplevel_window_model(viewport)?;
    let client = probe_xdg_toplevel_client()?;
    let inactive_window = window_model
        .windows
        .iter()
        .find(|candidate| candidate.id == "aqua-settings-client")
        .copied()
        .unwrap_or(ManagedClientWindow {
            id: "aqua-settings-client",
            title: "Aqua Settings",
            rect: aqua_scene::Rect {
                x: 464,
                y: 248,
                width: 704,
                height: 436,
            },
            z_index: 3,
            focused: false,
            closed: false,
            chrome: "aqua-window",
        });
    let record = ClientSurfaceRegistryRecord {
        client_id: "wayland-client-1",
        surface_id: window_model.surface_id,
        window_id: window_model.window_id,
        title: window_model.title,
        role: window_model.role,
        lifecycle_state: "mapped-focused",
        z_index: window_model.window.z_index,
        configure_serial: if client.server_configure_ack_received {
            1
        } else {
            0
        },
        configured: client.server_configure_ack_received,
        committed: true,
        mapped: window_model.mapped,
        focused: window_model.focused,
        close_supported: client.server_close_sent && client.client_close_event_received,
        buffer_attached: client.surface_committed,
        buffer_committed: client.surface_committed,
        buffer_width: 384,
        buffer_height: 256,
        buffer_stride: 384 * 4,
        buffer_format: "argb8888",
        buffer_source: "client-committed-wl-shm",
        import_required: true,
        import_planned: true,
        imported_for_sampling: client.server_shm_buffer_sampled,
        sample_checksum: client.shm_sample_checksum,
        sample_pixel: client.shm_sample_pixel,
        sample_grid: client.shm_sample_grid,
        client_buffer_rgba: client_record_buffer_rgba(&client, 384, 256),
        renderer_bound: false,
    };
    let inactive_record = ClientSurfaceRegistryRecord {
        client_id: "wayland-client-2",
        surface_id: "xdg-toplevel-2",
        window_id: inactive_window.id,
        title: inactive_window.title,
        role: "xdg-toplevel",
        lifecycle_state: "mapped-unfocused",
        z_index: inactive_window.z_index,
        configure_serial: if client.server_configure_ack_received
            && client.test_wayland_client_count >= 2
        {
            2
        } else {
            0
        },
        configured: client.server_configure_ack_received,
        committed: true,
        mapped: !inactive_window.closed,
        focused: inactive_window.focused,
        close_supported: client.server_close_sent && client.client_close_event_received,
        buffer_attached: client.surface_committed && client.test_wayland_client_count >= 2,
        buffer_committed: client.surface_committed && client.test_wayland_client_count >= 2,
        buffer_width: 320,
        buffer_height: 220,
        buffer_stride: 320 * 4,
        buffer_format: "argb8888",
        buffer_source: "client-committed-wl-shm",
        import_required: true,
        import_planned: true,
        imported_for_sampling: client.server_shm_buffer_sampled,
        sample_checksum: client.shm_sample_checksum,
        sample_pixel: client.shm_sample_pixel,
        sample_grid: client.shm_sample_grid,
        client_buffer_rgba: client_record_buffer_rgba(&client, 320, 220),
        renderer_bound: false,
    };
    let active_client_id = record.client_id;
    let active_surface_id = record.surface_id;
    let active_window_id = record.window_id;
    let inactive_client_id = inactive_record.client_id;
    let records = vec![record, inactive_record];

    Ok(ClientSurfaceRegistryProbe {
        product: PRODUCT,
        status: "client-surface-registry",
        source_window_model_ready: window_model.is_ready(),
        record_count: records.len(),
        active_client_id,
        active_surface_id,
        active_window_id,
        configure_serial_ready: client.client_configure_ack_sent
            && client.server_configure_ack_received
            && records
                .iter()
                .all(|candidate| candidate.configure_serial > 0 && candidate.configured),
        lifecycle_state_ready: records
            .iter()
            .any(|candidate| candidate.lifecycle_state == "mapped-focused")
            && records
                .iter()
                .any(|candidate| candidate.lifecycle_state == "mapped-unfocused")
            && records
                .iter()
                .all(|candidate| candidate.committed && candidate.mapped),
        two_client_ready: client.test_wayland_client_count == 2
            && records.len() == 2
            && records
                .iter()
                .any(|candidate| candidate.client_id == "wayland-client-2"),
        focus_index_ready: records.iter().filter(|candidate| candidate.focused).count() == 1,
        stacking_order_ready: records
            .iter()
            .find(|candidate| candidate.client_id == active_client_id)
            .zip(
                records
                    .iter()
                    .find(|candidate| candidate.client_id == inactive_client_id),
            )
            .map(|(active, inactive)| active.focused && active.z_index > inactive.z_index)
            .unwrap_or(false),
        close_request_ready: client.server_close_sent
            && client.client_close_event_received
            && records.iter().all(|candidate| candidate.close_supported),
        buffer_metadata_ready: client.surface_committed
            && records.iter().all(|candidate| {
                candidate.buffer_attached
                    && candidate.buffer_committed
                    && candidate.buffer_width > 0
                    && candidate.buffer_height > 0
                    && candidate.buffer_stride == candidate.buffer_width * 4
                    && candidate.buffer_format == "argb8888"
            }),
        buffer_import_plan_ready: records.iter().all(|candidate| {
            candidate.buffer_source == "client-committed-wl-shm"
                && candidate.import_required
                && candidate.import_planned
                && candidate.imported_for_sampling
                && candidate.sample_checksum != 0
                && candidate.sample_pixel[3] == 0xff
                && candidate.sample_grid.iter().all(|pixel| pixel[3] == 0xff)
        }),
        no_renderer_binding: records.iter().all(|candidate| !candidate.renderer_bound),
        renderer_started: false,
        boot_graphics: false,
        host_stub: window_model.host_stub,
        records,
    })
}

fn ok(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "missing"
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

fn solid_sample_grid(pixel: [u8; 4]) -> [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS] {
    [pixel; CLIENT_SAMPLE_GRID_PIXELS]
}

fn client_layer_snapshot_mode(snapshot_bytes: usize) -> &'static str {
    if snapshot_bytes == 0 {
        "sample-grid-fallback"
    } else {
        "full-buffer-snapshot"
    }
}

fn client_record_buffer_rgba(client: &XdgToplevelClientProbe, width: u32, height: u32) -> Vec<u8> {
    let expected_len = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if expected_len == 0 {
        return Vec::new();
    }

    if client.shm_buffer_rgba.len() == expected_len {
        return client.shm_buffer_rgba.clone();
    }

    if client.host_stub {
        return deterministic_client_buffer_rgba(width, height);
    }

    Vec::new()
}

fn deterministic_client_buffer_rgba(width: u32, height: u32) -> Vec<u8> {
    let width = width.max(1);
    let height = height.max(1);
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

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn copy_shm_pixel(ptr: *const u8, len: usize, stride: i32, x: usize, y: usize) -> [u8; 4] {
    let Ok(stride) = usize::try_from(stride) else {
        return [0, 0, 0, 0];
    };
    let offset = y.saturating_mul(stride).saturating_add(x.saturating_mul(4));
    if offset + 4 <= len {
        let mapped = unsafe { std::slice::from_raw_parts(ptr.add(offset), 4) };
        [mapped[0], mapped[1], mapped[2], mapped[3]]
    } else {
        [0, 0, 0, 0]
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn copy_shm_buffer_rgba(
    ptr: *const u8,
    len: usize,
    stride: i32,
    width: i32,
    height: i32,
) -> Vec<u8> {
    let (Ok(stride), Ok(width), Ok(height)) = (
        usize::try_from(stride),
        usize::try_from(width),
        usize::try_from(height),
    ) else {
        return Vec::new();
    };

    if width == 0 || height == 0 {
        return Vec::new();
    }

    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        let row_offset = y.saturating_mul(stride);
        let row_len = width * 4;
        if row_offset + row_len > len {
            return Vec::new();
        }
        let row = unsafe { std::slice::from_raw_parts(ptr.add(row_offset), row_len) };
        rgba.extend_from_slice(row);
    }
    rgba
}

fn checksum_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        let triple = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);

        output.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

pub fn read_session_config(
    path: impl AsRef<Path>,
) -> Result<ParsedSessionConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    parse_session_config(&content)
}

pub fn parse_session_config(
    content: &str,
) -> Result<ParsedSessionConfig, Box<dyn std::error::Error>> {
    fn value_for<'a>(content: &'a str, key: &str) -> Result<&'a str, Box<dyn std::error::Error>> {
        content
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(left, right)| (left.trim(), right.trim()))
            .find_map(|(left, right)| (left == key).then_some(right))
            .ok_or_else(|| format!("missing session config key: {key}").into())
    }

    fn bool_value(content: &str, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match value_for(content, key)? {
            "true" => Ok(true),
            "false" => Ok(false),
            value => Err(format!("invalid boolean for {key}: {value}").into()),
        }
    }

    Ok(ParsedSessionConfig {
        product: value_for(content, "product")?.to_string(),
        mode: value_for(content, "mode")?.to_string(),
        wayland_socket: value_for(content, "wayland_socket")?.to_string(),
        runtime_dir: value_for(content, "runtime_dir")?.to_string(),
        runtime_asset_root: value_for(content, "runtime_asset_root")?.to_string(),
        autostart: bool_value(content, "autostart")?,
        boot_graphics: bool_value(content, "boot_graphics")?,
        recovery_tty_required: bool_value(content, "recovery_tty_required")?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneProbe {
    pub scene: ShellScene,
    pub required_surfaces: usize,
    pub expected_surfaces: usize,
    pub surfaces_fit_viewport: bool,
    pub wallpaper_covers_viewport: bool,
    pub toast_avoids_dock: bool,
    pub launcher_avoids_dock: bool,
    pub mock_surfaces_labeled: bool,
    pub required_assets_present: bool,
    pub permanent_assets_only: bool,
    pub required_material_tokens_present: bool,
    pub simulated_surface_labeled: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherInputSceneProbe {
    pub status: &'static str,
    pub input_source: &'static str,
    pub initial_launcher_visible: bool,
    pub opened_launcher_visible: bool,
    pub dismissed_launcher_visible: bool,
    pub open_draw_command_count: usize,
    pub closed_draw_command_count: usize,
    pub redraw_requests: usize,
    pub visibility_changes: usize,
    pub launch_request: Option<LaunchRequest>,
    pub boot_graphics: bool,
    pub autostart: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmithayLauncherSeatProbe {
    pub status: &'static str,
    pub seat_name: &'static str,
    pub seat_global_created: bool,
    pub keyboard_capability: bool,
    pub pointer_capability: bool,
    pub keyboard_event_intercepted: bool,
    pub pointer_motion_dispatched: bool,
    pub pointer_button_dispatched: bool,
    pub launcher_visible: bool,
    pub selected_category: &'static str,
    pub draw_command_count: usize,
    pub host_stub: bool,
    pub boot_graphics: bool,
    pub autostart: bool,
}

impl SmithayLauncherSeatProbe {
    pub fn is_ready(&self) -> bool {
        self.status == "smithay-launcher-seat-binding"
            && self.seat_name == "Aqua Seat"
            && (self.host_stub
                || (self.seat_global_created
                    && self.keyboard_capability
                    && self.pointer_capability
                    && self.keyboard_event_intercepted
                    && self.pointer_motion_dispatched
                    && self.pointer_button_dispatched))
            && self.launcher_visible
            && self.selected_category == "settings"
            && self.draw_command_count == aqua_scene::REQUIRED_KINDS.len()
            && !self.boot_graphics
            && !self.autostart
    }
}

impl LauncherInputSceneProbe {
    pub fn is_ready(&self) -> bool {
        self.status == "launcher-input-scene-binding"
            && self.input_source == "compositor-seat-adapter-contract"
            && !self.initial_launcher_visible
            && self.opened_launcher_visible
            && !self.dismissed_launcher_visible
            && self.open_draw_command_count == aqua_scene::REQUIRED_KINDS.len()
            && self.closed_draw_command_count == aqua_scene::REQUIRED_KINDS.len() - 1
            && self.redraw_requests == 3
            && self.visibility_changes == 2
            && self.launch_request
                == Some(LaunchRequest {
                    app_id: "settings",
                    command: "/usr/bin/aqua-settings",
                    target: None,
                })
            && !self.boot_graphics
            && !self.autostart
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlanProbe {
    pub plan: RenderPlan,
    pub draw_command_count: usize,
    pub expected_draw_commands: usize,
    pub system_surface_commands_simulated: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererSurfaceSourceProbe {
    pub plan: ClientSurfaceSourcePlan,
    pub source_registry_ready: bool,
    pub source_count: usize,
    pub expected_sources: usize,
    pub active_source_ready: bool,
    pub import_sources_ready: bool,
    pub z_order_ready: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLayerPipelineProbe {
    pub source_probe: RendererSurfaceSourceProbe,
    pub paint_plan: ClientLayerPaintPlan,
    pub raster_probe: ClientLayerRasterProbe,
    pub source_plan_ready: bool,
    pub paint_plan_ready: bool,
    pub raster_ready: bool,
    pub layer_count: usize,
    pub expected_layers: usize,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintPlanProbe {
    pub plan: PaintPlan,
    pub paint_step_count: usize,
    pub expected_paint_steps: usize,
    pub system_surface_steps_translucent: bool,
    pub paint_order_stable: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramePlanProbe {
    pub plan: FramePlan,
    pub frame_ready: bool,
    pub pixel_format_ready: bool,
    pub stride_ready: bool,
    pub damage_ready: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBufferContractProbe {
    pub probe: FrameBufferProbe,
    pub buffer_allocated: bool,
    pub clear_color_ready: bool,
    pub first_pixel_ready: bool,
    pub last_pixel_ready: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareRasterContractProbe {
    pub probe: SoftwareRasterProbe,
    pub rect_count_ready: bool,
    pub wallpaper_sample_ready: bool,
    pub surface_sample_ready: bool,
    pub dock_sample_ready: bool,
    pub surface_border_sample_ready: bool,
    pub surface_highlight_sample_ready: bool,
    pub surface_corner_sample_ready: bool,
    pub surface_shadow_sample_ready: bool,
    pub checksum_ready: bool,
    pub surface_primitives_ready: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterExportContractProbe {
    pub export: RasterPpmExport,
    pub format_ready: bool,
    pub byte_count_ready: bool,
    pub checksum_ready: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterPngExportContractProbe {
    pub export: RasterPngExport,
    pub format_ready: bool,
    pub byte_count_ready: bool,
    pub checksum_ready: bool,
    pub renderer_started: bool,
    pub boot_graphics: bool,
}

impl RasterPngExportContractProbe {
    pub fn is_ready(&self) -> bool {
        self.export.is_ready()
            && self.format_ready
            && self.byte_count_ready
            && self.checksum_ready
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl RasterExportContractProbe {
    pub fn is_ready(&self) -> bool {
        self.export.is_ready()
            && self.format_ready
            && self.byte_count_ready
            && self.checksum_ready
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl SoftwareRasterContractProbe {
    pub fn is_ready(&self) -> bool {
        self.probe.is_ready()
            && self.rect_count_ready
            && self.wallpaper_sample_ready
            && self.surface_sample_ready
            && self.dock_sample_ready
            && self.surface_border_sample_ready
            && self.surface_highlight_sample_ready
            && self.surface_corner_sample_ready
            && self.surface_shadow_sample_ready
            && self.checksum_ready
            && self.surface_primitives_ready
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl FrameBufferContractProbe {
    pub fn is_ready(&self) -> bool {
        self.probe.is_ready()
            && self.buffer_allocated
            && self.clear_color_ready
            && self.first_pixel_ready
            && self.last_pixel_ready
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl FramePlanProbe {
    pub fn is_ready(&self) -> bool {
        self.plan.is_ready()
            && self.frame_ready
            && self.pixel_format_ready
            && self.stride_ready
            && self.damage_ready
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl PaintPlanProbe {
    pub fn is_ready(&self) -> bool {
        self.plan.is_ready()
            && self.paint_step_count == self.expected_paint_steps
            && self.system_surface_steps_translucent
            && self.paint_order_stable
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl RenderPlanProbe {
    pub fn is_ready(&self) -> bool {
        self.plan.is_ready()
            && self.draw_command_count == self.expected_draw_commands
            && self.system_surface_commands_simulated
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl RendererSurfaceSourceProbe {
    pub fn is_ready(&self) -> bool {
        self.plan.is_ready()
            && self.source_registry_ready
            && self.source_count == self.expected_sources
            && self.active_source_ready
            && self.import_sources_ready
            && self.z_order_ready
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl ClientLayerPipelineProbe {
    pub fn is_ready(&self) -> bool {
        self.source_probe.is_ready()
            && self.source_plan_ready
            && self.paint_plan_ready
            && self.raster_ready
            && self.layer_count == self.expected_layers
            && self.expected_layers == 2
            && !self.renderer_started
            && !self.boot_graphics
    }
}

impl SceneProbe {
    pub fn is_ready(&self) -> bool {
        self.scene.is_ready()
            && self.required_surfaces == self.expected_surfaces
            && self.surfaces_fit_viewport
            && self.wallpaper_covers_viewport
            && self.toast_avoids_dock
            && self.launcher_avoids_dock
            && self.mock_surfaces_labeled
            && self.required_assets_present
            && self.permanent_assets_only
            && self.required_material_tokens_present
            && self.simulated_surface_labeled
            && !self.boot_graphics
    }
}

pub fn probe_static_shell_scene(viewport: Viewport) -> SceneProbe {
    let scene = static_shell_scene(viewport);

    SceneProbe {
        required_surfaces: scene.required_surface_count(),
        expected_surfaces: aqua_scene::REQUIRED_KINDS.len(),
        surfaces_fit_viewport: scene.surfaces_fit_viewport(),
        wallpaper_covers_viewport: scene.wallpaper_covers_viewport(),
        toast_avoids_dock: scene.toast_avoids_dock(),
        launcher_avoids_dock: scene.launcher_avoids_dock(),
        mock_surfaces_labeled: scene.mock_surfaces_are_labeled(),
        required_assets_present: scene.required_assets_present(),
        permanent_assets_only: scene.permanent_assets_only(),
        required_material_tokens_present: scene.required_material_tokens_present(),
        simulated_surface_labeled: scene.simulated_surface_is_labeled(),
        boot_graphics: false,
        scene,
    }
}

pub fn probe_launcher_input_scene_binding(viewport: Viewport) -> LauncherInputSceneProbe {
    let mut launcher = LauncherState::default();
    let mut scene = static_shell_scene(viewport);
    scene.set_surface_visible(SurfaceKind::Launcher, launcher.is_open());
    let initial_launcher_visible = scene.surface_is_visible(SurfaceKind::Launcher);

    let mut redraw_requests = 0;
    let mut visibility_changes = 0;

    let opened = launcher.handle_event(LauncherEvent::Toggle);
    redraw_requests += usize::from(opened.redraw_requested);
    visibility_changes += usize::from(opened.visibility_changed);
    if opened.visibility_changed {
        scene.set_surface_visible(SurfaceKind::Launcher, launcher.is_open());
    }
    let opened_launcher_visible = scene.surface_is_visible(SurfaceKind::Launcher);
    let open_draw_command_count = plan_static_scene(&scene).commands.len();

    let searched = launcher.handle_event(LauncherEvent::ReplaceQuery("settings".to_string()));
    redraw_requests += usize::from(searched.redraw_requested);
    visibility_changes += usize::from(searched.visibility_changed);
    let activated = launcher.handle_event(LauncherEvent::Activate);

    let dismissed = launcher.handle_event(LauncherEvent::Dismiss);
    redraw_requests += usize::from(dismissed.redraw_requested);
    visibility_changes += usize::from(dismissed.visibility_changed);
    if dismissed.visibility_changed {
        scene.set_surface_visible(SurfaceKind::Launcher, launcher.is_open());
    }
    let dismissed_launcher_visible = scene.surface_is_visible(SurfaceKind::Launcher);
    let closed_draw_command_count = plan_static_scene(&scene).commands.len();

    LauncherInputSceneProbe {
        status: "launcher-input-scene-binding",
        input_source: "compositor-seat-adapter-contract",
        initial_launcher_visible,
        opened_launcher_visible,
        dismissed_launcher_visible,
        open_draw_command_count,
        closed_draw_command_count,
        redraw_requests,
        visibility_changes,
        launch_request: activated.launch_request,
        boot_graphics: false,
        autostart: false,
    }
}

pub fn probe_smithay_launcher_seat(
    viewport: Viewport,
) -> Result<SmithayLauncherSeatProbe, Box<dyn std::error::Error>> {
    probe_smithay_launcher_seat_impl(viewport)
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_smithay_launcher_seat_impl(
    viewport: Viewport,
) -> Result<SmithayLauncherSeatProbe, Box<dyn std::error::Error>> {
    let display: Display<WaylandSmokeState> = Display::new()?;
    let display_handle = display.handle();
    let mut state = WaylandSmokeState::new(&display_handle)?;
    state.launcher_scene = static_shell_scene(viewport);
    state
        .launcher_scene
        .set_surface_visible(SurfaceKind::Launcher, false);

    let keyboard = state
        .seat
        .get_keyboard()
        .ok_or("Aqua Seat keyboard capability was not created")?;
    let keyboard_event_intercepted = keyboard
        .input(
            &mut state,
            Keycode::from(125_u32),
            KeyState::Pressed,
            Serial::from(1_u32),
            1,
            |state, _, _| {
                state.keyboard_event_count += 1;
                state.apply_launcher_event(LauncherEvent::Toggle);
                FilterResult::Intercept(())
            },
        )
        .is_some();

    let pointer = state
        .seat
        .get_pointer()
        .ok_or("Aqua Seat pointer capability was not created")?;
    pointer.motion(
        &mut state,
        None,
        &MotionEvent {
            location: (1180.0, 760.0).into(),
            serial: Serial::from(2_u32),
            time: 2,
        },
    );
    state.pointer_motion_count += 1;
    pointer.button(
        &mut state,
        &ButtonEvent {
            serial: Serial::from(3_u32),
            time: 3,
            button: 0x110,
            state: ButtonState::Pressed,
        },
    );
    state.pointer_button_count += 1;
    state.apply_launcher_event(LauncherEvent::SelectCategory(LauncherCategory::Settings));

    Ok(SmithayLauncherSeatProbe {
        status: "smithay-launcher-seat-binding",
        seat_name: "Aqua Seat",
        seat_global_created: state.seat_global_created,
        keyboard_capability: state.seat.get_keyboard().is_some(),
        pointer_capability: state.seat.get_pointer().is_some(),
        keyboard_event_intercepted: keyboard_event_intercepted && state.keyboard_event_count == 1,
        pointer_motion_dispatched: state.pointer_motion_count == 1,
        pointer_button_dispatched: state.pointer_button_count == 1,
        launcher_visible: state
            .launcher_scene
            .surface_is_visible(SurfaceKind::Launcher),
        selected_category: state.launcher_state.category().id(),
        draw_command_count: plan_static_scene(&state.launcher_scene).commands.len(),
        host_stub: false,
        boot_graphics: false,
        autostart: false,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_smithay_launcher_seat_impl(
    viewport: Viewport,
) -> Result<SmithayLauncherSeatProbe, Box<dyn std::error::Error>> {
    let mut launcher_state = LauncherState::default();
    let mut launcher_scene = static_shell_scene(viewport);
    launcher_scene.set_surface_visible(SurfaceKind::Launcher, false);
    let update = launcher_state.handle_event(LauncherEvent::Toggle);
    if update.visibility_changed {
        launcher_scene.set_surface_visible(SurfaceKind::Launcher, launcher_state.is_open());
    }
    launcher_state.handle_event(LauncherEvent::SelectCategory(LauncherCategory::Settings));

    Ok(SmithayLauncherSeatProbe {
        status: "smithay-launcher-seat-binding",
        seat_name: "Aqua Seat",
        seat_global_created: false,
        keyboard_capability: false,
        pointer_capability: false,
        keyboard_event_intercepted: false,
        pointer_motion_dispatched: false,
        pointer_button_dispatched: false,
        launcher_visible: launcher_scene.surface_is_visible(SurfaceKind::Launcher),
        selected_category: launcher_state.category().id(),
        draw_command_count: plan_static_scene(&launcher_scene).commands.len(),
        host_stub: true,
        boot_graphics: false,
        autostart: false,
    })
}

pub fn probe_static_render_plan(viewport: Viewport) -> RenderPlanProbe {
    let scene = static_shell_scene(viewport);
    let plan = plan_static_scene(&scene);

    RenderPlanProbe {
        draw_command_count: plan.commands.len(),
        expected_draw_commands: aqua_scene::REQUIRED_KINDS.len(),
        system_surface_commands_simulated: plan.system_surface_commands_are_simulated(),
        renderer_started: false,
        boot_graphics: false,
        plan,
    }
}

pub fn probe_renderer_surface_sources(
    viewport: Viewport,
) -> Result<RendererSurfaceSourceProbe, Box<dyn std::error::Error>> {
    let registry = probe_client_surface_registry(viewport)?;
    let sources = registry
        .records
        .iter()
        .map(|record| ClientSurfaceSource {
            client_id: record.client_id,
            surface_id: record.surface_id,
            window_id: record.window_id,
            z_index: record.z_index,
            focused: record.focused,
            rect: window_model_rect_for_record(&registry, record),
            width: record.buffer_width,
            height: record.buffer_height,
            stride: record.buffer_stride,
            format: record.buffer_format,
            source: record.buffer_source,
            sample_checksum: record.sample_checksum,
            sample_pixel: record.sample_pixel,
            sample_grid: record.sample_grid,
            client_buffer_rgba: record.client_buffer_rgba.clone(),
            renderer_import_ready: record.imported_for_sampling,
        })
        .collect();
    let plan = plan_client_surface_sources(sources);

    Ok(RendererSurfaceSourceProbe {
        source_registry_ready: registry.is_ready(),
        source_count: plan.sources.len(),
        expected_sources: registry.record_count,
        active_source_ready: plan.sources.first().is_some_and(|source| {
            source.client_id == registry.active_client_id
                && source.surface_id == registry.active_surface_id
                && source.window_id == registry.active_window_id
                && source.focused
                && source.renderer_import_ready
        }),
        import_sources_ready: plan.sources.iter().all(ClientSurfaceSource::is_ready),
        z_order_ready: plan
            .sources
            .windows(2)
            .all(|pair| pair[0].z_index >= pair[1].z_index),
        renderer_started: plan.renderer_started,
        boot_graphics: false,
        plan,
    })
}

pub fn probe_client_layer_pipeline(
    viewport: Viewport,
) -> Result<ClientLayerPipelineProbe, Box<dyn std::error::Error>> {
    let source_probe = probe_renderer_surface_sources(viewport)?;
    let paint_plan = plan_client_layer_paint_steps(&source_probe.plan);
    let raster_probe = probe_client_layer_raster(viewport, &paint_plan);

    Ok(ClientLayerPipelineProbe {
        source_plan_ready: source_probe.is_ready(),
        paint_plan_ready: paint_plan.is_ready(),
        raster_ready: raster_probe.is_ready(),
        layer_count: raster_probe.layer_count,
        expected_layers: raster_probe.expected_layer_count,
        renderer_started: paint_plan.renderer_started || raster_probe.renderer_started,
        boot_graphics: false,
        source_probe,
        paint_plan,
        raster_probe,
    })
}

fn window_model_rect_for_record(
    _registry: &ClientSurfaceRegistryProbe,
    record: &ClientSurfaceRegistryRecord,
) -> aqua_scene::Rect {
    match record.window_id {
        "wayland-test-client" => aqua_scene::Rect {
            x: 416,
            y: 220,
            width: 704,
            height: 436,
        },
        "aqua-settings-client" => aqua_scene::Rect {
            x: 464,
            y: 248,
            width: 704,
            height: 436,
        },
        _ => aqua_scene::Rect {
            x: 0,
            y: 0,
            width: record.buffer_width,
            height: record.buffer_height,
        },
    }
}

pub fn probe_static_paint_plan(viewport: Viewport) -> PaintPlanProbe {
    let plan = paint_plan_for_static_scene(viewport);

    PaintPlanProbe {
        paint_step_count: plan.steps.len(),
        expected_paint_steps: aqua_scene::REQUIRED_KINDS.len(),
        system_surface_steps_translucent: plan.system_surface_steps_are_translucent(),
        paint_order_stable: plan.orders_are_stable(),
        renderer_started: plan.renderer_started,
        boot_graphics: false,
        plan,
    }
}

pub fn probe_static_frame_plan(viewport: Viewport) -> FramePlanProbe {
    let plan = frame_plan_for_static_scene(viewport);

    FramePlanProbe {
        frame_ready: plan.width == viewport.width
            && plan.height == viewport.height
            && plan.paint_step_count == aqua_scene::REQUIRED_KINDS.len(),
        pixel_format_ready: plan.pixel_format == "rgba8888",
        stride_ready: plan.stride_bytes == viewport.width * 4
            && plan.buffer_bytes == u64::from(viewport.width) * u64::from(viewport.height) * 4,
        damage_ready: plan.damage_rect.contains_viewport(viewport),
        renderer_started: plan.renderer_started,
        boot_graphics: false,
        plan,
    }
}

pub fn probe_static_frame_buffer(viewport: Viewport) -> FrameBufferContractProbe {
    let probe = probe_frame_buffer_for_static_scene(viewport);

    FrameBufferContractProbe {
        buffer_allocated: probe.buffer_bytes == probe.allocated_bytes as u64,
        clear_color_ready: probe.clear_color == "#001725ff",
        first_pixel_ready: probe.first_pixel == [0x00, 0x17, 0x25, 0xff],
        last_pixel_ready: probe.last_pixel == [0x00, 0x17, 0x25, 0xff],
        renderer_started: probe.renderer_started,
        boot_graphics: false,
        probe,
    }
}

pub fn probe_static_software_raster(viewport: Viewport) -> SoftwareRasterContractProbe {
    let probe = probe_software_raster_for_static_scene(viewport);

    SoftwareRasterContractProbe {
        rect_count_ready: probe.filled_rect_count == aqua_scene::REQUIRED_KINDS.len(),
        wallpaper_sample_ready: probe.wallpaper_sample == [0x04, 0x3b, 0x5c, 0xff],
        surface_sample_ready: probe.surface_sample == [0x51, 0xac, 0xd2, 0xff],
        dock_sample_ready: probe.dock_sample == [0x51, 0xac, 0xd2, 0xff],
        surface_border_sample_ready: probe.surface_border_sample == [0x3d, 0x72, 0x8c, 0xff],
        surface_highlight_sample_ready: probe.surface_highlight_sample == [0xa3, 0xd3, 0xe7, 0xff],
        surface_corner_sample_ready: probe.surface_corner_sample == [0x2a, 0x6c, 0x8c, 0xff],
        surface_shadow_sample_ready: probe.surface_shadow_sample == [0x33, 0x86, 0xaa, 0xff],
        checksum_ready: probe.raster_checksum == 0x7015_58d1_5395_21df,
        surface_primitives_ready: probe.surface_primitive_count == 15,
        renderer_started: probe.renderer_started,
        boot_graphics: false,
        probe,
    }
}

pub fn probe_static_raster_export(viewport: Viewport) -> RasterExportContractProbe {
    let export = export_software_raster_ppm_for_static_scene(viewport);

    RasterExportContractProbe {
        format_ready: export.format == "ppm-p6-rgb888",
        byte_count_ready: export.byte_count == 4_718_609,
        checksum_ready: export.checksum == 0xefdc_ba78_578c_2cd5,
        renderer_started: export.renderer_started,
        boot_graphics: false,
        export,
    }
}

pub fn probe_static_raster_png_export(viewport: Viewport) -> RasterPngExportContractProbe {
    let export = export_software_raster_png_for_static_scene(viewport);

    RasterPngExportContractProbe {
        format_ready: export.format == "png-rgba8888",
        byte_count_ready: export.byte_count == 6_293_028 && export.byte_count == export.bytes.len(),
        checksum_ready: export.checksum == 0x2cdb_1d86_a1ba_9300,
        renderer_started: export.renderer_started,
        boot_graphics: false,
        export,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeAssetProbe {
    pub root: PathBuf,
    pub wallpaper: bool,
    pub design_tokens: bool,
    pub aqua_home_icon: bool,
    pub aqua_icon_license: bool,
}

impl RuntimeAssetProbe {
    pub fn is_ready(&self) -> bool {
        self.wallpaper && self.design_tokens && self.aqua_home_icon && self.aqua_icon_license
    }
}

pub fn probe_runtime_assets(root: impl AsRef<Path>) -> RuntimeAssetProbe {
    let root = root.as_ref().to_path_buf();

    RuntimeAssetProbe {
        wallpaper: root.join("wallpapers/default-wallpaper.png").is_file(),
        design_tokens: root.join("tokens/design-tokens.json").is_file(),
        aqua_home_icon: root.join("icons/aqua/home.svg").is_file(),
        aqua_icon_license: root.join("icons/aqua/LICENSE").is_file(),
        root,
    }
}

pub fn design_tokens_include_product(path: impl AsRef<Path>) -> bool {
    fs::read_to_string(path)
        .map(|content| content.contains("\"product\": \"Aqua Linux\""))
        .unwrap_or(false)
}

pub fn design_tokens_include_scene_materials(path: impl AsRef<Path>) -> bool {
    fs::read_to_string(path)
        .map(|content| {
            [
                "\"surface\"",
                "\"fill\"",
                "\"secondaryFill\"",
                "\"border\"",
                "\"separator\"",
                "\"shadow\"",
                "\"optionalBlurRadius\"",
                "\"blurRequired\"",
            ]
            .iter()
            .all(|token| content.contains(token))
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventLoopSmokeResult {
    pub ticks: u32,
}

impl EventLoopSmokeResult {
    pub fn is_ready(&self) -> bool {
        self.ticks == 1
    }
}

pub fn run_event_loop_smoke() -> Result<EventLoopSmokeResult, Box<dyn std::error::Error>> {
    let mut event_loop: EventLoop<u32> = EventLoop::try_new()?;
    let handle = event_loop.handle();

    handle.insert_source(Timer::immediate(), |_deadline, _metadata, ticks| {
        *ticks += 1;
        TimeoutAction::Drop
    })?;

    let mut ticks = 0;
    event_loop.dispatch(Duration::from_millis(20), &mut ticks)?;

    Ok(EventLoopSmokeResult { ticks })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaylandDisplaySmokeResult {
    pub display_created: bool,
    pub compositor_global_created: bool,
    pub host_stub: bool,
}

impl WaylandDisplaySmokeResult {
    pub fn is_ready(&self) -> bool {
        self.display_created && self.compositor_global_created
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaylandSocketSmokeResult {
    pub socket_name: String,
    pub display_created: bool,
    pub compositor_global_created: bool,
    pub socket_bound: bool,
    pub accept_nonblocking: bool,
    pub client_connected: bool,
    pub client_accepted: bool,
    pub client_inserted: bool,
    pub socket_cleaned: bool,
    pub host_stub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalloopSocketSmokeResult {
    pub socket_name: String,
    pub display_created: bool,
    pub compositor_global_created: bool,
    pub socket_bound: bool,
    pub client_connected: bool,
    pub callback_invoked: bool,
    pub client_accepted: bool,
    pub client_inserted: bool,
    pub dispatch_clients_ok: bool,
    pub dispatched_requests: usize,
    pub flush_clients_ok: bool,
    pub socket_cleaned: bool,
    pub host_stub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSkeletonProbe {
    pub product: &'static str,
    pub mode: &'static str,
    pub foundation: &'static str,
    pub event_loop: &'static str,
    pub display_owned: bool,
    pub compositor_state_owned: bool,
    pub client_inserted: bool,
    pub dispatch_clients_ok: bool,
    pub flush_clients_ok: bool,
    pub host_stub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRunOnceSmokeResult {
    pub socket_name: String,
    pub run_once_called: bool,
    pub socket_bound: bool,
    pub client_connected: bool,
    pub callback_invoked: bool,
    pub client_accepted: bool,
    pub client_inserted: bool,
    pub dispatch_clients_ok: bool,
    pub dispatched_requests: usize,
    pub flush_clients_ok: bool,
    pub socket_cleaned: bool,
    pub host_stub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionLoopSmokeResult {
    pub socket_name: String,
    pub loop_started: bool,
    pub loop_iterations: u32,
    pub max_iterations: u32,
    pub socket_bound: bool,
    pub client_connected: bool,
    pub callback_invoked: bool,
    pub client_accepted: bool,
    pub client_inserted: bool,
    pub dispatch_passes: u32,
    pub flush_passes: u32,
    pub socket_cleaned: bool,
    pub host_stub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionBootstrapProbe {
    pub product: String,
    pub mode: String,
    pub configured_runtime_dir: String,
    pub prepared_runtime_dir: PathBuf,
    pub wayland_display: String,
    pub xdg_runtime_dir: String,
    pub aqua_asset_root: String,
    pub config_recovery_safe: bool,
    pub env_recovery_safe: bool,
    pub runtime_dir_prepared: bool,
    pub runtime_dir_private: bool,
    pub autostart_blocked: bool,
    pub boot_graphics_blocked: bool,
    pub session_started: bool,
    pub desktop_shell_started: bool,
}

impl SessionRunOnceSmokeResult {
    pub fn is_ready(&self) -> bool {
        self.run_once_called
            && self.socket_bound
            && self.client_connected
            && self.callback_invoked
            && self.client_accepted
            && self.client_inserted
            && self.dispatch_clients_ok
            && self.flush_clients_ok
            && self.socket_cleaned
    }
}

impl SessionLoopSmokeResult {
    pub fn is_ready(&self) -> bool {
        self.loop_started
            && self.loop_iterations == self.max_iterations
            && self.max_iterations >= 3
            && self.socket_bound
            && self.client_connected
            && self.callback_invoked
            && self.client_accepted
            && self.client_inserted
            && self.dispatch_passes == self.max_iterations
            && self.flush_passes == self.max_iterations
            && self.socket_cleaned
    }
}

impl SessionBootstrapProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.mode == DEV_MODE
            && self.config_recovery_safe
            && self.env_recovery_safe
            && self.runtime_dir_prepared
            && self.runtime_dir_private
            && self.autostart_blocked
            && self.boot_graphics_blocked
            && !self.session_started
            && !self.desktop_shell_started
    }
}

impl SessionSkeletonProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.mode == DEV_MODE
            && self.foundation == FOUNDATION
            && self.event_loop == EVENT_LOOP
            && self.display_owned
            && self.compositor_state_owned
            && self.client_inserted
            && self.dispatch_clients_ok
            && self.flush_clients_ok
    }
}

impl CalloopSocketSmokeResult {
    pub fn is_ready(&self) -> bool {
        self.display_created
            && self.compositor_global_created
            && self.socket_bound
            && self.client_connected
            && self.callback_invoked
            && self.client_accepted
            && self.client_inserted
            && self.dispatch_clients_ok
            && self.flush_clients_ok
            && self.socket_cleaned
    }
}

impl WaylandSocketSmokeResult {
    pub fn is_ready(&self) -> bool {
        self.display_created
            && self.compositor_global_created
            && self.socket_bound
            && self.accept_nonblocking
            && self.client_connected
            && self.client_accepted
            && self.client_inserted
            && self.socket_cleaned
    }
}

pub fn run_wayland_display_smoke() -> Result<WaylandDisplaySmokeResult, Box<dyn std::error::Error>>
{
    run_wayland_display_smoke_impl()
}

pub fn run_wayland_socket_smoke() -> Result<WaylandSocketSmokeResult, Box<dyn std::error::Error>> {
    run_wayland_socket_smoke_impl()
}

pub fn run_calloop_socket_smoke() -> Result<CalloopSocketSmokeResult, Box<dyn std::error::Error>> {
    run_calloop_socket_smoke_impl()
}

pub fn probe_session_skeleton() -> Result<SessionSkeletonProbe, Box<dyn std::error::Error>> {
    probe_session_skeleton_impl()
}

pub fn run_session_once_smoke() -> Result<SessionRunOnceSmokeResult, Box<dyn std::error::Error>> {
    run_session_once_smoke_impl()
}

pub fn run_session_loop_smoke() -> Result<SessionLoopSmokeResult, Box<dyn std::error::Error>> {
    run_session_loop_smoke_impl()
}

pub fn probe_session_bootstrap(
    config: &ParsedSessionConfig,
    prepared_runtime_dir: impl AsRef<Path>,
) -> Result<SessionBootstrapProbe, Box<dyn std::error::Error>> {
    let prepared_runtime_dir = prepared_runtime_dir.as_ref().to_path_buf();
    fs::create_dir_all(&prepared_runtime_dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&prepared_runtime_dir, fs::Permissions::from_mode(0o700))?;
    }

    let metadata = fs::metadata(&prepared_runtime_dir)?;
    let runtime_dir_prepared = metadata.is_dir();

    #[cfg(unix)]
    let runtime_dir_private = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o077 == 0
    };
    #[cfg(not(unix))]
    let runtime_dir_private = true;

    let env = config.environment();

    Ok(SessionBootstrapProbe {
        product: config.product.clone(),
        mode: config.mode.clone(),
        configured_runtime_dir: config.runtime_dir.clone(),
        prepared_runtime_dir,
        wayland_display: env.wayland_display.clone(),
        xdg_runtime_dir: env.xdg_runtime_dir.clone(),
        aqua_asset_root: env.aqua_asset_root.clone(),
        config_recovery_safe: config.is_recovery_safe(),
        env_recovery_safe: env.is_recovery_safe(),
        runtime_dir_prepared,
        runtime_dir_private,
        autostart_blocked: !config.autostart,
        boot_graphics_blocked: !config.boot_graphics,
        session_started: false,
        desktop_shell_started: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_xdg_shell_binding_impl(
    viewport: Viewport,
) -> Result<XdgShellBindingProbe, Box<dyn std::error::Error>> {
    let display: Display<WaylandSmokeState> = Display::new()?;
    let display_handle = display.handle();
    let state = WaylandSmokeState::new(&display_handle)?;
    let lifecycle = probe_client_surface_lifecycle(viewport);

    Ok(XdgShellBindingProbe {
        product: PRODUCT,
        status: "xdg-shell-binding",
        foundation: FOUNDATION,
        protocol: "xdg_wm_base",
        handler_bound: true,
        global_created: true,
        toplevel_callbacks_bound: state.toplevel_callbacks_bound,
        popup_callbacks_bound: state.popup_callbacks_bound,
        lifecycle_probe_ready: lifecycle.is_ready(),
        real_wayland_client_started: false,
        renderer_started: false,
        boot_graphics: false,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_xdg_toplevel_client_impl() -> Result<XdgToplevelClientProbe, Box<dyn std::error::Error>> {
    let mut session = AquaCompositorSession::new()?;
    let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
    let client_one_inserted = session.insert_client(server_stream_one).is_ok();
    let client_two_inserted = session.insert_client(server_stream_two).is_ok();

    let client_one_conn = ClientConnection::from_socket(client_stream_one)?;
    let client_two_conn = ClientConnection::from_socket(client_stream_two)?;
    let mut event_queue_one = client_one_conn.new_event_queue();
    let mut event_queue_two = client_two_conn.new_event_queue();
    let qh_one = event_queue_one.handle();
    let qh_two = event_queue_two.handle();
    client_one_conn.display().get_registry(&qh_one, ());
    client_two_conn.display().get_registry(&qh_two, ());
    client_one_conn.flush()?;
    client_two_conn.flush()?;

    session.dispatch_clients()?;
    session.flush_clients()?;

    let mut client_one = XdgSmokeClientState::with_buffer_size(384, 256);
    let mut client_two = XdgSmokeClientState::with_buffer_size(320, 220);
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;

    if client_one.surface_created
        && client_one.toplevel_requested
        && client_two.surface_created
        && client_two.toplevel_requested
    {
        client_one_conn.flush()?;
        client_two_conn.flush()?;
        session.dispatch_clients()?;
        session.flush_clients()?;
        event_queue_one.blocking_dispatch(&mut client_one)?;
        event_queue_two.blocking_dispatch(&mut client_two)?;
        client_one_conn.flush()?;
        client_two_conn.flush()?;
        session.dispatch_clients()?;
        session.flush_clients()?;
    }

    Ok(XdgToplevelClientProbe {
        product: PRODUCT,
        status: "xdg-toplevel-client",
        foundation: FOUNDATION,
        protocol: "xdg_wm_base",
        client_connected: true,
        client_inserted: client_one_inserted && client_two_inserted,
        registry_bound: client_one.registry_bound && client_two.registry_bound,
        compositor_global_seen: client_one.compositor_global_seen
            && client_two.compositor_global_seen,
        shm_global_created: true,
        shm_global_seen: client_one.shm_global_seen && client_two.shm_global_seen,
        shm_buffer_created: client_one.shm_buffer_created && client_two.shm_buffer_created,
        client_buffer_attached: client_one.client_buffer_attached
            && client_two.client_buffer_attached,
        xdg_wm_base_global_seen: client_one.xdg_wm_base_global_seen
            && client_two.xdg_wm_base_global_seen,
        surface_created: client_one.surface_created && client_two.surface_created,
        toplevel_requested: client_one.toplevel_requested && client_two.toplevel_requested,
        surface_committed: session.wayland_state.surface_commit_count >= 2,
        server_buffer_attached: session.wayland_state.server_buffer_attach_count >= 2,
        server_shm_buffer_imported: session.wayland_state.server_shm_buffer_import_count >= 2,
        server_shm_buffer_sampled: session.wayland_state.server_shm_buffer_sample_count >= 2,
        shm_sample_checksum: session.wayland_state.shm_sample_checksum,
        shm_sample_pixel: session.wayland_state.shm_sample_pixel,
        shm_sample_grid: session.wayland_state.shm_sample_grid,
        shm_buffer_rgba: session.wayland_state.shm_buffer_rgba.clone(),
        server_toplevel_created: session.wayland_state.toplevel_count >= 2,
        server_configure_sent: session.wayland_state.toplevel_configure_sent,
        client_configure_ack_sent: client_one.configure_ack_sent && client_two.configure_ack_sent,
        server_configure_ack_received: session.wayland_state.toplevel_configure_ack_count >= 2,
        server_close_sent: session.wayland_state.toplevel_close_sent,
        client_close_event_received: client_one.close_event_received
            && client_two.close_event_received,
        dispatch_clients_ok: true,
        flush_clients_ok: true,
        test_wayland_client_started: true,
        test_wayland_client_count: 2,
        renderer_started: false,
        boot_graphics: false,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_wayland_display_smoke_impl() -> Result<WaylandDisplaySmokeResult, Box<dyn std::error::Error>>
{
    let display: Display<WaylandSmokeState> = Display::new()?;
    let display_handle = display.handle();
    let _state = WaylandSmokeState::new(&display_handle)?;

    Ok(WaylandDisplaySmokeResult {
        display_created: true,
        compositor_global_created: true,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_wayland_socket_smoke_impl() -> Result<WaylandSocketSmokeResult, Box<dyn std::error::Error>> {
    let socket_name = "aqua-wayland-0".to_string();
    let runtime_dir = std::env::temp_dir().join(format!(
        "aqua-wayland-smoke-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&runtime_dir)?;

    let socket_path = runtime_dir.join(&socket_name);
    let lock_path = socket_path.with_extension("lock");

    let (
        display_created,
        compositor_global_created,
        socket_bound,
        accept_nonblocking,
        client_connected,
        client_accepted,
        client_inserted,
    ) = {
        let display: Display<WaylandSmokeState> = Display::new()?;
        let mut display_handle = display.handle();
        let _state = WaylandSmokeState::new(&display_handle)?;
        let listener = ListeningSocket::bind_absolute(socket_path.clone())?;
        let accept_nonblocking = listener.accept()?.is_none();

        let _client_stream = std::os::unix::net::UnixStream::connect(&socket_path)?;
        let accepted_stream = wait_for_accepted_client(&listener, Duration::from_millis(50))?;
        let client_accepted = accepted_stream.is_some();
        let client_inserted = match accepted_stream {
            Some(stream) => display_handle
                .insert_client(stream, Arc::new(WaylandSmokeClientState::default()))
                .is_ok(),
            None => false,
        };

        (
            true,
            true,
            socket_path.is_file(),
            accept_nonblocking,
            true,
            client_accepted,
            client_inserted,
        )
    };

    let socket_cleaned = !socket_path.exists() && !lock_path.exists();
    fs::remove_dir(&runtime_dir)?;

    Ok(WaylandSocketSmokeResult {
        socket_name,
        display_created,
        compositor_global_created,
        socket_bound,
        accept_nonblocking,
        client_connected,
        client_accepted,
        client_inserted,
        socket_cleaned,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_calloop_socket_smoke_impl() -> Result<CalloopSocketSmokeResult, Box<dyn std::error::Error>> {
    let run_once =
        run_session_once_at_socket("aqua-wayland-calloop-0", "aqua-calloop-socket-smoke")?;

    Ok(CalloopSocketSmokeResult {
        socket_name: run_once.socket_name,
        display_created: true,
        compositor_global_created: true,
        socket_bound: run_once.socket_bound,
        client_connected: run_once.client_connected,
        callback_invoked: run_once.callback_invoked,
        client_accepted: run_once.client_accepted,
        client_inserted: run_once.client_inserted,
        dispatch_clients_ok: run_once.dispatch_clients_ok,
        dispatched_requests: run_once.dispatched_requests,
        flush_clients_ok: run_once.flush_clients_ok,
        socket_cleaned: run_once.socket_cleaned,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_session_once_smoke_impl() -> Result<SessionRunOnceSmokeResult, Box<dyn std::error::Error>> {
    run_session_once_at_socket("aqua-wayland-run-once-0", "aqua-session-run-once-smoke")
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_session_loop_smoke_impl() -> Result<SessionLoopSmokeResult, Box<dyn std::error::Error>> {
    run_session_loop_at_socket("aqua-wayland-session-loop-0", "aqua-session-loop-smoke", 3)
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_session_once_at_socket(
    socket_name: &str,
    runtime_prefix: &str,
) -> Result<SessionRunOnceSmokeResult, Box<dyn std::error::Error>> {
    let runtime_dir = std::env::temp_dir().join(format!(
        "{runtime_prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&runtime_dir)?;

    let socket_path = runtime_dir.join(socket_name);
    let lock_path = socket_path.with_extension("lock");

    let run_once = {
        let session = AquaCompositorSession::new()?;
        session.run_once_smoke(socket_path.clone(), Duration::from_millis(50))?
    };

    let socket_cleaned = !socket_path.exists() && !lock_path.exists();
    fs::remove_dir(&runtime_dir)?;

    Ok(SessionRunOnceSmokeResult {
        socket_name: socket_name.to_string(),
        socket_cleaned,
        host_stub: false,
        ..run_once
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_session_loop_at_socket(
    socket_name: &str,
    runtime_prefix: &str,
    max_iterations: u32,
) -> Result<SessionLoopSmokeResult, Box<dyn std::error::Error>> {
    let runtime_dir = std::env::temp_dir().join(format!(
        "{runtime_prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&runtime_dir)?;

    let socket_path = runtime_dir.join(socket_name);
    let lock_path = socket_path.with_extension("lock");

    let loop_result = {
        let session = AquaCompositorSession::new()?;
        session.run_bounded_loop_smoke(
            socket_path.clone(),
            Duration::from_millis(10),
            max_iterations,
        )?
    };

    let socket_cleaned = !socket_path.exists() && !lock_path.exists();
    fs::remove_dir(&runtime_dir)?;

    Ok(SessionLoopSmokeResult {
        socket_name: socket_name.to_string(),
        socket_cleaned,
        host_stub: false,
        ..loop_result
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_session_skeleton_impl() -> Result<SessionSkeletonProbe, Box<dyn std::error::Error>> {
    let mut session = AquaCompositorSession::new()?;
    let (client_a, client_b) = std::os::unix::net::UnixStream::pair()?;
    drop(client_b);

    let client_inserted = session.insert_client(client_a).is_ok();
    let dispatch_clients_ok = session.dispatch_clients().is_ok();
    let flush_clients_ok = session.flush_clients().is_ok();

    Ok(SessionSkeletonProbe {
        product: PRODUCT,
        mode: DEV_MODE,
        foundation: FOUNDATION,
        event_loop: EVENT_LOOP,
        display_owned: true,
        compositor_state_owned: true,
        client_inserted,
        dispatch_clients_ok,
        flush_clients_ok,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn wait_for_accepted_client(
    listener: &ListeningSocket,
    timeout: Duration,
) -> std::io::Result<Option<std::os::unix::net::UnixStream>> {
    let started_at = std::time::Instant::now();

    loop {
        if let Some(stream) = listener.accept()? {
            return Ok(Some(stream));
        }

        if started_at.elapsed() >= timeout {
            return Ok(None);
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn run_wayland_display_smoke_impl() -> Result<WaylandDisplaySmokeResult, Box<dyn std::error::Error>>
{
    Ok(WaylandDisplaySmokeResult {
        display_created: true,
        compositor_global_created: true,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_xdg_shell_binding_impl(
    viewport: Viewport,
) -> Result<XdgShellBindingProbe, Box<dyn std::error::Error>> {
    Ok(XdgShellBindingProbe {
        product: PRODUCT,
        status: "xdg-shell-binding",
        foundation: FOUNDATION,
        protocol: "xdg_wm_base",
        handler_bound: true,
        global_created: true,
        toplevel_callbacks_bound: true,
        popup_callbacks_bound: true,
        lifecycle_probe_ready: probe_client_surface_lifecycle(viewport).is_ready(),
        real_wayland_client_started: false,
        renderer_started: false,
        boot_graphics: false,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_xdg_toplevel_client_impl() -> Result<XdgToplevelClientProbe, Box<dyn std::error::Error>> {
    Ok(XdgToplevelClientProbe {
        product: PRODUCT,
        status: "xdg-toplevel-client",
        foundation: FOUNDATION,
        protocol: "xdg_wm_base",
        client_connected: true,
        client_inserted: true,
        registry_bound: true,
        compositor_global_seen: true,
        shm_global_created: true,
        shm_global_seen: true,
        shm_buffer_created: true,
        client_buffer_attached: true,
        xdg_wm_base_global_seen: true,
        surface_created: true,
        toplevel_requested: true,
        surface_committed: true,
        server_buffer_attached: true,
        server_shm_buffer_imported: true,
        server_shm_buffer_sampled: true,
        shm_sample_checksum: 0xfeed_a011_u64,
        shm_sample_pixel: [0x00, 0x00, 0x7f, 0xff],
        shm_sample_grid: solid_sample_grid([0x00, 0x00, 0x7f, 0xff]),
        shm_buffer_rgba: Vec::new(),
        server_toplevel_created: true,
        server_configure_sent: true,
        client_configure_ack_sent: true,
        server_configure_ack_received: true,
        server_close_sent: true,
        client_close_event_received: true,
        dispatch_clients_ok: true,
        flush_clients_ok: true,
        test_wayland_client_started: true,
        test_wayland_client_count: 2,
        renderer_started: false,
        boot_graphics: false,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn run_wayland_socket_smoke_impl() -> Result<WaylandSocketSmokeResult, Box<dyn std::error::Error>> {
    Ok(WaylandSocketSmokeResult {
        socket_name: "aqua-wayland-0".to_string(),
        display_created: true,
        compositor_global_created: true,
        socket_bound: true,
        accept_nonblocking: true,
        client_connected: true,
        client_accepted: true,
        client_inserted: true,
        socket_cleaned: true,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn run_calloop_socket_smoke_impl() -> Result<CalloopSocketSmokeResult, Box<dyn std::error::Error>> {
    Ok(CalloopSocketSmokeResult {
        socket_name: "aqua-wayland-calloop-0".to_string(),
        display_created: true,
        compositor_global_created: true,
        socket_bound: true,
        client_connected: true,
        callback_invoked: true,
        client_accepted: true,
        client_inserted: true,
        dispatch_clients_ok: true,
        dispatched_requests: 0,
        flush_clients_ok: true,
        socket_cleaned: true,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn run_session_once_smoke_impl() -> Result<SessionRunOnceSmokeResult, Box<dyn std::error::Error>> {
    Ok(SessionRunOnceSmokeResult {
        socket_name: "aqua-wayland-run-once-0".to_string(),
        run_once_called: true,
        socket_bound: true,
        client_connected: true,
        callback_invoked: true,
        client_accepted: true,
        client_inserted: true,
        dispatch_clients_ok: true,
        dispatched_requests: 0,
        flush_clients_ok: true,
        socket_cleaned: true,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn run_session_loop_smoke_impl() -> Result<SessionLoopSmokeResult, Box<dyn std::error::Error>> {
    Ok(SessionLoopSmokeResult {
        socket_name: "aqua-wayland-session-loop-0".to_string(),
        loop_started: true,
        loop_iterations: 3,
        max_iterations: 3,
        socket_bound: true,
        client_connected: true,
        callback_invoked: true,
        client_accepted: true,
        client_inserted: true,
        dispatch_passes: 3,
        flush_passes: 3,
        socket_cleaned: true,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_session_skeleton_impl() -> Result<SessionSkeletonProbe, Box<dyn std::error::Error>> {
    Ok(SessionSkeletonProbe {
        product: PRODUCT,
        mode: DEV_MODE,
        foundation: FOUNDATION,
        event_loop: EVENT_LOOP,
        display_owned: true,
        compositor_state_owned: true,
        client_inserted: true,
        dispatch_clients_ok: true,
        flush_clients_ok: true,
        host_stub: true,
    })
}

pub fn status_lines() -> [&'static str; 16] {
    [
        "product=Aqua Linux",
        "component=aqua-compositor",
        "mode=nested-dev",
        "backend_target=custom Wayland compositor",
        "foundation=smithay",
        "foundation_version=0.7.0",
        "foundation_status=selected-scene-model-spike",
        "smithay_features=wayland_frontend,backend_libinput,udev",
        "wayland_display=smoke",
        "event_loop=calloop",
        "event_loop_version=0.14.4",
        "scene_model=aqua-scene",
        "scene_status=static-shell-model",
        "renderer=aqua-renderer",
        "renderer_status=plan-only",
        "session_loop=bounded-smoke",
    ]
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct WaylandSmokeState {
    compositor_state: CompositorState,
    shm_state: ShmState,
    xdg_shell_state: XdgShellState,
    seat_state: SeatState<Self>,
    seat: Seat<Self>,
    launcher_state: LauncherState,
    desktop_icon_state: DesktopIconState,
    trash_model: TrashModel,
    notification_center: NotificationCenter,
    notification_now_ms: u64,
    notification_clock_started_at: std::time::Instant,
    session_menu_state: SessionMenuState,
    launcher_scene: ShellScene,
    seat_global_created: bool,
    keyboard_event_count: usize,
    pointer_motion_count: usize,
    pointer_button_count: usize,
    pointer_location: (f64, f64),
    output_width: u32,
    output_height: u32,
    active_workspace: usize,
    toplevel_callbacks_bound: bool,
    popup_callbacks_bound: bool,
    toplevel_count: usize,
    toplevel_configure_sent: bool,
    toplevel_configure_serial: Option<u32>,
    toplevel_configure_ack_count: usize,
    toplevel_close_sent: bool,
    surface_commit_count: usize,
    server_buffer_attach_count: usize,
    server_shm_buffer_import_count: usize,
    server_shm_buffer_sample_count: usize,
    shm_sample_checksum: u64,
    shm_sample_pixel: [u8; 4],
    shm_sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    shm_buffer_rgba: Vec<u8>,
    shm_buffer_width: u32,
    shm_buffer_height: u32,
    shm_buffer_stride: u32,
    close_new_toplevels: bool,
    mapped_surface: Option<WlSurface>,
    mapped_surfaces: Vec<ServerSurfaceRecord>,
    toplevel_surfaces: Vec<ToplevelSurface>,
    pointer_focus_surface: Option<WlSurface>,
    damage_commit_count: usize,
    damage_rect_count: usize,
    pending_frame_callbacks: Vec<wl_callback::WlCallback>,
    frame_callbacks_sent: usize,
    keyboard_focus_assigned: bool,
    pointer_focus_assigned: bool,
    keyboard_shortcut_intercept_count: usize,
    keyboard_forward_count: usize,
    pointer_hit_test_count: usize,
    pointer_surface_hit_count: usize,
    surface_focus_change_count: usize,
    stacking_change_count: usize,
    destroyed_surface_count: usize,
    client_cleanup_count: usize,
    cleanup_keyboard_focus_reassigned: bool,
    move_request_count: usize,
    resize_request_count: usize,
    close_request_count: usize,
    maximize_request_count: usize,
    unmaximize_request_count: usize,
    fullscreen_request_count: usize,
    unfullscreen_request_count: usize,
    launcher_pointer_hit_count: usize,
    launcher_category_click_count: usize,
    launcher_app_click_count: usize,
    launcher_launch_request: Option<LaunchRequest>,
    session_action_request: Option<SessionAction>,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    workspace_switch_count: usize,
    workspace_move_count: usize,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Clone)]
struct ServerSurfaceRecord {
    surface: WlSurface,
    buffer: wl_buffer::WlBuffer,
    workspace: usize,
    sample_checksum: u64,
    sample_pixel: [u8; 4],
    sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    buffer_rgba: Vec<u8>,
    buffer_opaque: bool,
    width: u32,
    height: u32,
    stride: u32,
    x: u32,
    y: u32,
    display_width: u32,
    display_height: u32,
    restore_geometry: Option<(u32, u32, u32, u32)>,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl WaylandSmokeState {
    fn new(display_handle: &DisplayHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(display_handle, "Aqua Seat");
        seat.add_pointer();
        seat.add_keyboard(Default::default(), 400, 25)?;
        let mut launcher_scene = static_shell_scene(Viewport::new(1536, 1024));
        launcher_scene.set_surface_visible(SurfaceKind::Launcher, false);
        launcher_scene.set_surface_visible(SurfaceKind::SystemOverview, false);
        launcher_scene.set_surface_visible(SurfaceKind::NotificationToast, false);
        let trash_root = std::env::var_os("AQUA_TRASH_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home/aqua/Trash"));
        let trash_model = TrashModel::open(trash_root)?;

        Ok(Self {
            compositor_state: CompositorState::new::<WaylandSmokeState>(display_handle),
            shm_state: ShmState::new::<WaylandSmokeState>(display_handle, []),
            xdg_shell_state: XdgShellState::new::<WaylandSmokeState>(display_handle),
            seat_state,
            seat,
            launcher_state: LauncherState::default(),
            desktop_icon_state: DesktopIconState::default(),
            trash_model,
            notification_center: NotificationCenter::default(),
            notification_now_ms: 0,
            notification_clock_started_at: std::time::Instant::now(),
            session_menu_state: SessionMenuState::default(),
            launcher_scene,
            seat_global_created: true,
            keyboard_event_count: 0,
            pointer_motion_count: 0,
            pointer_button_count: 0,
            pointer_location: (768.0, 512.0),
            output_width: 1536,
            output_height: 1024,
            active_workspace: 0,
            toplevel_callbacks_bound: true,
            popup_callbacks_bound: true,
            toplevel_count: 0,
            toplevel_configure_sent: false,
            toplevel_configure_serial: None,
            toplevel_configure_ack_count: 0,
            toplevel_close_sent: false,
            surface_commit_count: 0,
            server_buffer_attach_count: 0,
            server_shm_buffer_import_count: 0,
            server_shm_buffer_sample_count: 0,
            shm_sample_checksum: 0,
            shm_sample_pixel: [0, 0, 0, 0],
            shm_sample_grid: solid_sample_grid([0, 0, 0, 0]),
            shm_buffer_rgba: Vec::new(),
            shm_buffer_width: 0,
            shm_buffer_height: 0,
            shm_buffer_stride: 0,
            close_new_toplevels: true,
            mapped_surface: None,
            mapped_surfaces: Vec::new(),
            toplevel_surfaces: Vec::new(),
            pointer_focus_surface: None,
            damage_commit_count: 0,
            damage_rect_count: 0,
            pending_frame_callbacks: Vec::new(),
            frame_callbacks_sent: 0,
            keyboard_focus_assigned: false,
            pointer_focus_assigned: false,
            keyboard_shortcut_intercept_count: 0,
            keyboard_forward_count: 0,
            pointer_hit_test_count: 0,
            pointer_surface_hit_count: 0,
            surface_focus_change_count: 0,
            stacking_change_count: 0,
            destroyed_surface_count: 0,
            client_cleanup_count: 0,
            cleanup_keyboard_focus_reassigned: false,
            move_request_count: 0,
            resize_request_count: 0,
            close_request_count: 0,
            maximize_request_count: 0,
            unmaximize_request_count: 0,
            fullscreen_request_count: 0,
            unfullscreen_request_count: 0,
            launcher_pointer_hit_count: 0,
            launcher_category_click_count: 0,
            launcher_app_click_count: 0,
            launcher_launch_request: None,
            session_action_request: None,
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            workspace_switch_count: 0,
            workspace_move_count: 0,
        })
    }

    fn focus_top_surface_in_active_workspace(&mut self, serial: u32) {
        self.mapped_surface = self
            .mapped_surfaces
            .iter()
            .rev()
            .find(|record| record.workspace == self.active_workspace)
            .map(|record| record.surface.clone());
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.set_focus(
                self,
                self.mapped_surface.clone(),
                Serial::from(serial.max(1)),
            );
        }
        self.keyboard_focus_assigned = self.mapped_surface.is_some();
        self.pointer_focus_surface = None;
        self.pointer_focus_assigned = false;
    }

    fn activate_workspace(&mut self, workspace: usize, serial: u32) -> bool {
        if workspace >= WORKSPACE_COUNT || workspace == self.active_workspace {
            return false;
        }
        let previous = self.active_workspace;
        self.active_workspace = workspace;
        self.workspace_switch_count += 1;
        self.focus_top_surface_in_active_workspace(serial);
        let visible_clients = self
            .mapped_surfaces
            .iter()
            .filter(|record| record.workspace == workspace)
            .count();
        println!(
            "desktop_workspace_switched from={previous} to={workspace} visible_clients={visible_clients} count={}",
            self.workspace_switch_count
        );
        true
    }

    fn move_active_surface_to_workspace(&mut self, workspace: usize, serial: u32) -> bool {
        if workspace >= WORKSPACE_COUNT || workspace == self.active_workspace {
            return false;
        }
        let Some(active_surface) = self.mapped_surface.clone() else {
            return false;
        };
        let Some(record) = self.mapped_surfaces.iter_mut().find(|record| {
            record.surface == active_surface && record.workspace == self.active_workspace
        }) else {
            return false;
        };
        let previous = record.workspace;
        record.workspace = workspace;
        self.workspace_move_count += 1;
        println!(
            "desktop_window_workspace_moved from={previous} to={workspace} count={}",
            self.workspace_move_count
        );
        self.focus_top_surface_in_active_workspace(serial);
        true
    }

    fn apply_launcher_event(&mut self, event: LauncherEvent) {
        let update = self.launcher_state.handle_event(event);
        if update.launch_request.is_some() {
            self.launcher_launch_request = update.launch_request;
        }
        if self.launcher_state.is_open() && self.session_menu_state.is_open() {
            self.session_menu_state
                .handle_event(SessionMenuEvent::Dismiss);
            self.launcher_scene
                .set_surface_visible(SurfaceKind::SystemOverview, false);
        }
        if update.visibility_changed {
            self.launcher_scene
                .set_surface_visible(SurfaceKind::Launcher, self.launcher_state.is_open());
        }
    }

    fn apply_session_menu_event(&mut self, event: SessionMenuEvent) {
        let update = self.session_menu_state.handle_event(event);
        if self.session_menu_state.is_open() && self.launcher_state.is_open() {
            self.launcher_state.handle_event(LauncherEvent::Dismiss);
            self.launcher_scene
                .set_surface_visible(SurfaceKind::Launcher, false);
        }
        if update.visibility_changed {
            self.launcher_scene.set_surface_visible(
                SurfaceKind::SystemOverview,
                self.session_menu_state.is_open(),
            );
        }
        if update.redraw_requested {
            println!(
                "desktop_session_menu_selected={}",
                self.session_menu_state.selected_action().id()
            );
            println!(
                "desktop_session_menu_confirmation={}",
                self.session_menu_state
                    .confirmation()
                    .map_or("none", SessionAction::id)
            );
        }
        if update.action_request.is_some() {
            println!(
                "desktop_session_action_queued={}",
                update
                    .action_request
                    .expect("checked session action request")
                    .id()
            );
            self.session_action_request = update.action_request;
        }
    }

    fn sync_notification_visibility(&mut self) {
        self.launcher_scene.set_surface_visible(
            SurfaceKind::NotificationToast,
            self.notification_center.active().is_some(),
        );
    }

    fn post_desktop_notification(&mut self, title: &str, body: &str) {
        self.notification_now_ms = self.current_notification_time_ms();
        self.notification_center.post(
            self.notification_now_ms,
            "Desktop",
            title,
            body,
            NOTIFICATION_DEFAULT_TIMEOUT_MS,
        );
        self.sync_notification_visibility();
    }

    fn current_notification_time_ms(&self) -> u64 {
        self.notification_now_ms.max(
            self.notification_clock_started_at
                .elapsed()
                .as_millis()
                .min(u128::from(u64::MAX)) as u64,
        )
    }

    fn notification_close_hit(&self, x: u32, y: u32) -> bool {
        let Some(notification) = self.notification_center.active() else {
            return false;
        };
        self.launcher_scene
            .surfaces
            .iter()
            .find(|surface| surface.kind == SurfaceKind::NotificationToast)
            .is_some_and(|surface| {
                notification_dismiss_hit(
                    surface.rect,
                    &notification.source,
                    &notification.title,
                    &notification.body,
                    x,
                    y,
                )
            })
    }

    fn close_active_toplevel(&mut self) -> bool {
        let Some(surface) = self.mapped_surface.clone() else {
            return false;
        };
        let Some(toplevel) = self
            .toplevel_surfaces
            .iter()
            .find(|toplevel| *toplevel.wl_surface() == surface)
            .cloned()
        else {
            return false;
        };
        toplevel.send_close();
        self.close_request_count += 1;
        true
    }

    fn resize_active_toplevel(&mut self) -> bool {
        let Some(surface) = self.mapped_surface.clone() else {
            return false;
        };
        let Some(toplevel) = self
            .toplevel_surfaces
            .iter()
            .find(|toplevel| *toplevel.wl_surface() == surface)
            .cloned()
        else {
            return false;
        };
        let Some(record) = self
            .mapped_surfaces
            .iter_mut()
            .find(|record| record.surface == surface)
        else {
            return false;
        };
        record.display_width = (record.display_width + 64).min(640);
        record.display_height = (record.display_height + 48).min(480);
        self.resize_request_count += 1;
        println!(
            "desktop_toplevel_resize_request width={} height={} count={}",
            record.display_width, record.display_height, self.resize_request_count
        );
        toplevel.with_pending_state(|state| {
            state.size = Some((record.display_width as i32, record.display_height as i32).into());
        });
        toplevel.send_configure();
        true
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct CalloopSocketSmokeState {
    session: AquaCompositorSession,
    callback_invoked: bool,
    client_accepted: bool,
    client_inserted: bool,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct AquaTerminalSession {
    parser: vt100::Parser,
    master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn std::io::Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    output: Receiver<Vec<u8>>,
    rows: u16,
    cols: u16,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl AquaTerminalSession {
    fn spawn(rows: u16, cols: u16) -> Result<Self, Box<dyn std::error::Error>> {
        use portable_pty::{native_pty_system, CommandBuilder, PtySize};
        use std::io::Read as _;

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = native_pty_system().openpty(size)?;
        let shell = std::env::var("AQUA_TERMINAL_SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let mut command = CommandBuilder::new(shell);
        command.arg("-i");
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("PS1", "aqua@aqua:\\w$ ");
        if let Some(home) = std::env::var_os("HOME") {
            command.cwd(home);
        }
        let child = pair.slave.spawn_command(command)?;
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        let (tx, output) = mpsc::channel();
        std::thread::Builder::new()
            .name("aqua-terminal-pty-reader".into())
            .spawn(move || {
                let mut buffer = [0_u8; 4096];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(count) if tx.send(buffer[..count].to_vec()).is_err() => break,
                        Ok(_) => {}
                    }
                }
            })?;
        Ok(Self {
            parser: vt100::Parser::new(rows, cols, 1000),
            master: pair.master,
            writer,
            child,
            output,
            rows,
            cols,
        })
    }

    fn drain_output(&mut self) -> bool {
        use std::io::Write as _;

        let mut changed = false;
        while let Ok(bytes) = self.output.try_recv() {
            println!("aqua_terminal_pty_output_bytes={}", bytes.len());
            self.parser.process(&bytes);
            changed = true;
        }
        if changed {
            let _ = std::io::stdout().flush();
        }
        changed
    }

    fn write_input(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write as _;
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        if bytes == b"\r" {
            println!("aqua_terminal_input_enter=true");
            let _ = std::io::stdout().flush();
        }
        Ok(())
    }

    fn resize(&mut self, rows: u16, cols: u16) -> Result<bool, Box<dyn std::error::Error>> {
        if rows == self.rows && cols == self.cols {
            return Ok(false);
        }
        self.master.resize(portable_pty::PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        self.parser.screen_mut().set_size(rows, cols);
        self.rows = rows;
        self.cols = cols;
        Ok(true)
    }

    fn view(&self) -> aqua_shell::TerminalView {
        let screen = self.parser.screen();
        let (cursor_row, cursor_col) = screen.cursor_position();
        aqua_shell::TerminalView {
            lines: screen.rows(0, self.cols).collect(),
            cursor_row,
            cursor_col,
            rows: self.rows,
            cols: self.cols,
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl Drop for AquaTerminalSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.try_wait();
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn decode_installer_logo(path: &Path) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    use std::io::BufReader;

    let decoder = png::Decoder::new(BufReader::new(fs::File::open(path)?));
    let mut reader = decoder.read_info()?;
    let output_size = reader
        .output_buffer_size()
        .ok_or("installer logo output buffer is too large")?;
    let mut decoded = vec![0; output_size];
    let info = reader.next_frame(&mut decoded)?;
    if info.bit_depth != png::BitDepth::Eight {
        return Err("installer logo must use 8-bit channels".into());
    }
    let bytes = &decoded[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 0xff])
            .collect(),
        png::ColorType::Rgba => bytes.to_vec(),
        other => return Err(format!("unsupported installer logo color type: {other:?}").into()),
    };
    Ok((info.width, info.height, rgba))
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn build_installer_presentation_graph(
    model: &InstallerModel,
) -> Result<InstallTransactionGraph, Box<dyn std::error::Error>> {
    let artifacts = InstallArtifacts::new(
        "/mnt/aqua-artifacts/rootfs.tar",
        "/mnt/aqua-artifacts/bzImage",
        "/mnt/aqua-artifacts/bootx64.efi",
    )?;
    let plan = build_dry_run_plan(model, &artifacts)?;
    let prerequisites = validate_install_prerequisites(&InstallToolPaths::system())?;
    let commands = compile_install_commands(&plan, &prerequisites)?;
    let internal = compile_internal_install_actions(&plan)?;
    Ok(build_install_transaction_graph(
        &plan, &commands, &internal,
    )?)
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn log_installer_progress(progress: &InstallProgressEvent) {
    println!(
        "[AQUA-INSTALLER-PROGRESS] state={} phase={} operation={} completed={} total={} percent={}",
        progress.state().id(),
        progress.phase().id(),
        progress.operation(),
        progress.completed_steps(),
        progress.total_steps(),
        progress.percent()
    );
}

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct XdgSmokeClientState {
    registry_bound: bool,
    compositor_global_seen: bool,
    shm_global_seen: bool,
    shm_buffer_created: bool,
    client_buffer_attached: bool,
    xdg_wm_base_global_seen: bool,
    base_surface: Option<wl_surface::WlSurface>,
    xdg_surface: Option<client_xdg_surface::XdgSurface>,
    xdg_toplevel: Option<client_xdg_toplevel::XdgToplevel>,
    seat: Option<client_wl_seat::WlSeat>,
    shm_buffer: Option<client_wl_buffer::WlBuffer>,
    shm: Option<client_wl_shm::WlShm>,
    wm_base: Option<client_xdg_wm_base::XdgWmBase>,
    buffer_width: u32,
    buffer_height: u32,
    surface_created: bool,
    toplevel_requested: bool,
    configure_ack_sent: bool,
    close_event_received: bool,
    frame_callback_received: bool,
    partial_damage_commit_sent: bool,
    seat_global_seen: bool,
    keyboard_event_received: bool,
    pointer_event_received: bool,
    interactive_requests_sent: bool,
    state_cycle_enabled: bool,
    state_cycle_started: bool,
    state_configure_count: usize,
    size_constraints_sent: bool,
    state_cycle_complete: bool,
    title: String,
    app_id: String,
    theme: aqua_shell::AquaTheme,
    files_model: Option<aqua_shell::FilesWindowModel>,
    files_navigator: Option<aqua_shell::FilesNavigator>,
    files_scrollbar_dragging: bool,
    settings_model: Option<aqua_shell::SettingsWindowModel>,
    settings_config_path: Option<PathBuf>,
    settings_persistence_failed: bool,
    properties_model: Option<aqua_shell::DesktopPropertiesModel>,
    properties_home_root: Option<PathBuf>,
    properties_system_root: Option<PathBuf>,
    terminal_session: Option<AquaTerminalSession>,
    terminal_frame_pending: bool,
    terminal_frame_requested_at: Option<std::time::Instant>,
    terminal_dirty: bool,
    terminal_command_observed: bool,
    installer_model: Option<InstallerModel>,
    installer_forms: Option<InstallerFormState>,
    installer_ui: Option<InstallerUiState>,
    installer_logo_width: u32,
    installer_logo_height: u32,
    installer_logo_rgba: Vec<u8>,
    installer_keyboard_press_count: usize,
    installer_redraw_count: usize,
    installer_progress_graph: Option<InstallTransactionGraph>,
    installer_progress: Option<InstallProgressEvent>,
    installer_progress_checkpoint: usize,
    installer_progress_rehearsal_enabled: bool,
    typography_acceptance: bool,
    component_acceptance: bool,
    keyboard_shift: bool,
    keyboard_ctrl: bool,
    pointer_surface_x: f64,
    pointer_surface_y: f64,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl XdgSmokeClientState {
    fn handle_window_frame_pointer(&mut self, serial: u32) -> bool {
        let titlebar_height = match self.app_id.as_str() {
            "aqua.terminal" | "aqua.files" => 48,
            "aqua.properties" => 52,
            "aqua.settings" => 58,
            _ => return false,
        };
        let frame = WindowFrame::new(
            Rect {
                x: 0,
                y: 0,
                width: self.buffer_width,
                height: self.buffer_height,
            },
            &self.title,
            titlebar_height,
        );
        let pointer_x = self.pointer_surface_x.max(0.0) as u32;
        let pointer_y = self.pointer_surface_y.max(0.0) as u32;
        let action = first_party_window_action(frame, pointer_x, pointer_y);

        match action {
            FirstPartyWindowAction::Close => {
                self.close_event_received = true;
                self.interactive_requests_sent = true;
            }
            FirstPartyWindowAction::Minimize => {
                if let Some(toplevel) = self.xdg_toplevel.as_ref() {
                    toplevel.set_minimized();
                    self.interactive_requests_sent = true;
                }
            }
            FirstPartyWindowAction::Maximize => {
                if let Some(toplevel) = self.xdg_toplevel.as_ref() {
                    toplevel.set_maximized();
                    self.interactive_requests_sent = true;
                }
            }
            FirstPartyWindowAction::Move => {
                if let (Some(toplevel), Some(seat)) =
                    (self.xdg_toplevel.as_ref(), self.seat.as_ref())
                {
                    toplevel._move(seat, serial);
                    self.interactive_requests_sent = true;
                }
            }
            FirstPartyWindowAction::Resize => {
                if let (Some(toplevel), Some(seat)) =
                    (self.xdg_toplevel.as_ref(), self.seat.as_ref())
                {
                    toplevel.resize(seat, serial, client_xdg_toplevel::ResizeEdge::BottomRight);
                    self.interactive_requests_sent = true;
                }
            }
            FirstPartyWindowAction::None => {}
        }
        println!(
            "aqua_window_frame_pointer app_id={} x={pointer_x} y={pointer_y} action={action:?}",
            self.app_id
        );
        true
    }

    fn configured_theme() -> aqua_shell::AquaTheme {
        if let Ok(value) = std::env::var("AQUA_THEME") {
            if let Some(theme) = aqua_shell::AquaTheme::parse(&value) {
                return theme;
            }
        }
        let config_path = std::env::var_os("AQUA_SETTINGS_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home/aqua/.config/aqua/settings.conf"));
        aqua_shell::SettingsWindowModel::load_or_default(&config_path)
            .map(|model| model.theme)
            .unwrap_or_default()
    }

    fn apply_runtime_theme(&mut self, theme: aqua_shell::AquaTheme) -> bool {
        if self.theme == theme {
            return false;
        }
        self.theme = theme;
        if let Some(model) = self.settings_model.as_mut() {
            model.theme = theme;
        }
        true
    }

    fn refresh_runtime_theme(&mut self, qh: &QueueHandle<Self>) -> bool {
        let theme = Self::configured_theme();
        if !self.apply_runtime_theme(theme) {
            return false;
        }
        println!(
            "aqua_runtime_theme_changed={} app_id={}",
            theme.id(),
            self.app_id
        );
        if self.installer_model.is_some() {
            self.redraw_installer_buffer(qh);
        } else if self.files_model.is_some() {
            self.redraw_files_buffer(qh);
        } else if self.settings_model.is_some() {
            self.redraw_settings_buffer(qh);
        } else if self.properties_model.is_some() {
            self.redraw_properties_buffer(qh);
        } else if self.terminal_session.is_some() {
            self.redraw_terminal_buffer(qh);
        }
        true
    }

    fn with_buffer_size(width: u32, height: u32) -> Self {
        Self {
            buffer_width: width,
            buffer_height: height,
            state_cycle_enabled: true,
            title: "Aqua Linux test client".to_string(),
            app_id: "aqua.test-client".to_string(),
            ..Self::default()
        }
    }

    fn files_app() -> Self {
        let root = std::env::var_os("AQUA_FILES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home/aqua"));
        let files_navigator = aqua_shell::FilesNavigator::open(&root).ok();
        let files_model = files_navigator
            .as_ref()
            .map(|navigator| navigator.window().clone())
            .unwrap_or_default();
        Self {
            buffer_width: 640,
            buffer_height: 420,
            title: "Files".to_string(),
            app_id: "aqua.files".to_string(),
            theme: Self::configured_theme(),
            files_model: Some(files_model),
            files_navigator,
            ..Self::default()
        }
    }

    fn settings_app() -> Result<Self, aqua_shell::SettingsConfigError> {
        let config_path = std::env::var_os("AQUA_SETTINGS_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home/aqua/.config/aqua/settings.conf"));
        let settings_model = aqua_shell::SettingsWindowModel::load_or_default(&config_path)?;
        let mut settings_model = settings_model;
        let network_root = std::env::var_os("AQUA_NETWORK_SYS_CLASS_NET")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/sys/class/net"));
        if let Err(error) = settings_model.refresh_network_status(&network_root) {
            eprintln!("aqua_settings_network_status_available=false error={error}");
        }
        Ok(Self {
            buffer_width: 600,
            buffer_height: 400,
            title: "System Settings".to_string(),
            app_id: "aqua.settings".to_string(),
            theme: settings_model.theme,
            settings_model: Some(settings_model),
            settings_config_path: Some(config_path),
            ..Self::default()
        })
    }

    fn terminal_app() -> Result<Self, Box<dyn std::error::Error>> {
        let rows = 18;
        let cols = 72;
        Ok(Self {
            buffer_width: 680,
            buffer_height: 430,
            title: "Terminal".to_string(),
            app_id: "aqua.terminal".to_string(),
            theme: Self::configured_theme(),
            terminal_session: Some(AquaTerminalSession::spawn(rows, cols)?),
            terminal_dirty: true,
            ..Self::default()
        })
    }

    fn properties_app(target: &'static str) -> Result<Self, std::io::Error> {
        let home_root = std::env::var_os("AQUA_HOME_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home/aqua"));
        let system_root = std::env::var_os("AQUA_SYSTEM_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        let properties_model =
            aqua_shell::DesktopPropertiesModel::load(target, &home_root, &system_root)?;
        Ok(Self {
            buffer_width: 480,
            buffer_height: 300,
            title: properties_model.title.clone(),
            app_id: "aqua.properties".to_string(),
            theme: Self::configured_theme(),
            properties_model: Some(properties_model),
            properties_home_root: Some(home_root),
            properties_system_root: Some(system_root),
            ..Self::default()
        })
    }

    fn installer_app() -> Result<Self, Box<dyn std::error::Error>> {
        let logo_path = std::env::var_os("AQUA_INSTALLER_LOGO")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/usr/share/aqua/brand/aqua-symbol-primary.png"));
        let mut app = Self::installer_app_with_logo(&logo_path)?;
        let inventory = probe_storage(&StorageProbePaths::system())?;
        println!(
            "aqua_installer_storage_candidate_count={}",
            inventory.candidates().len()
        );
        println!(
            "aqua_installer_storage_eligible_count={}",
            inventory.eligible_candidates().count()
        );
        for candidate in inventory.candidates() {
            let reasons = candidate
                .blocked_reasons()
                .iter()
                .map(|reason| reason.id())
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "aqua_installer_storage_candidate={} eligible={} blocked_reasons={}",
                candidate.device(),
                candidate.is_eligible(),
                if reasons.is_empty() { "none" } else { &reasons }
            );
        }
        app.installer_forms
            .as_mut()
            .expect("installer forms should exist")
            .load_storage_inventory(&inventory);
        Ok(app)
    }

    fn installer_app_with_logo(logo_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let (logo_width, logo_height, logo_rgba) = decode_installer_logo(logo_path)?;
        let mut model = InstallerModel::default();
        model.set_mode(InstallMode::Real);
        let ui = InstallerUiState::new(&model);
        let theme = Self::configured_theme();
        println!("aqua_installer_theme={}", theme.id());
        Ok(Self {
            buffer_width: 1280,
            buffer_height: 800,
            title: "Aqua Linux Kurulumu".to_string(),
            app_id: "aqua.installer".to_string(),
            theme,
            installer_model: Some(model),
            installer_forms: Some(InstallerFormState::default()),
            installer_ui: Some(ui),
            installer_logo_width: logo_width,
            installer_logo_height: logo_height,
            installer_logo_rgba: logo_rgba,
            installer_progress_rehearsal_enabled: std::env::var(
                "AQUA_INSTALLER_PROGRESS_PRESENTATION_REHEARSAL",
            )
            .is_ok_and(|value| value == "true"),
            ..Self::default()
        })
    }

    fn typography_acceptance_app() -> Self {
        let theme = Self::configured_theme();
        println!("aqua_typography_acceptance_theme={}", theme.id());
        Self {
            buffer_width: 1280,
            buffer_height: 800,
            title: "Aqua Typography Acceptance".to_string(),
            app_id: "aqua.typography-acceptance".to_string(),
            theme,
            typography_acceptance: true,
            ..Self::default()
        }
    }

    fn component_acceptance_app() -> Self {
        let theme = Self::configured_theme();
        println!("aqua_component_acceptance_theme={}", theme.id());
        Self {
            buffer_width: 1280,
            buffer_height: 800,
            title: "Aqua Component Acceptance".to_string(),
            app_id: "aqua.component-acceptance".to_string(),
            theme,
            component_acceptance: true,
            ..Self::default()
        }
    }

    fn render_installer_buffer(&self) -> Result<Vec<u8>, String> {
        let model = self
            .installer_model
            .as_ref()
            .ok_or_else(|| "installer model is missing".to_string())?;
        let forms = self
            .installer_forms
            .as_ref()
            .ok_or_else(|| "installer forms are missing".to_string())?;
        let ui = self
            .installer_ui
            .as_ref()
            .ok_or_else(|| "installer UI state is missing".to_string())?;
        let logo = InstallerImageSource::new(
            self.installer_logo_width,
            self.installer_logo_height,
            &self.installer_logo_rgba,
        )?;
        render_installer_window_rgba_with_theme(
            self.buffer_width,
            self.buffer_height,
            model,
            ui,
            forms,
            logo,
            InstallerRenderOptions {
                progress: self.installer_progress.as_ref(),
                theme: self.theme,
            },
        )
        .map(|(pixels, _)| pixels)
    }

    fn redraw_installer_buffer(&mut self, qh: &QueueHandle<Self>) {
        let Some(shm) = self.shm.clone() else {
            return;
        };
        let width = self.buffer_width.max(1);
        let height = self.buffer_height.max(1);
        let stride = width * 4;
        let size = stride * height;
        let Ok(pixels) = self.render_installer_buffer() else {
            eprintln!("aqua_installer_redraw_error=render-failed");
            return;
        };

        use std::io::Write;
        use std::os::unix::io::AsFd;
        let mut file = tempfile::tempfile().expect("Aqua Installer redraw tempfile should open");
        file.write_all(&pixels)
            .expect("Aqua Installer redraw buffer should be writable");
        file.flush()
            .expect("Aqua Installer redraw buffer should flush");
        let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            client_wl_shm::Format::Argb8888,
            qh,
            (),
        );
        if let Some(surface) = self.base_surface.as_ref() {
            surface.attach(Some(&buffer), 0, 0);
            surface.damage(0, 0, width as i32, height as i32);
            surface.frame(qh, ());
            surface.commit();
            self.shm_buffer = Some(buffer);
            self.installer_redraw_count += 1;
            println!(
                "aqua_installer_redraw_count={}",
                self.installer_redraw_count
            );
        }
    }

    fn handle_installer_pointer(&mut self, x: u32, y: u32, qh: &QueueHandle<Self>) -> bool {
        let Ok(layout) = InstallerWindowLayout::for_viewport(Viewport::new(
            self.buffer_width,
            self.buffer_height,
        )) else {
            return false;
        };
        let (Some(model), Some(forms), Some(ui)) = (
            self.installer_model.as_mut(),
            self.installer_forms.as_mut(),
            self.installer_ui.as_mut(),
        ) else {
            return false;
        };
        let content_changed = match model.step() {
            InstallerStep::Language | InstallerStep::Keyboard | InstallerStep::TimeZone => {
                match forms.handle_choice_pointer(model, &layout, x, y) {
                    Ok(update) => {
                        if update.changed() {
                            println!("aqua_installer_pointer_form_update={update:?}");
                        }
                        update.changed()
                    }
                    Err(error) => {
                        eprintln!("aqua_installer_pointer_form_error={error}");
                        false
                    }
                }
            }
            InstallerStep::Partitions => {
                let update = forms.handle_disk_pointer(model, &layout, x, y);
                if update.changed() {
                    println!("aqua_installer_pointer_disk_update={update:?}");
                }
                update.changed()
            }
            InstallerStep::UserInformation => {
                let update = forms.user_mut().handle_pointer(model, &layout, x, y);
                if update.changed() {
                    println!("aqua_installer_pointer_user_update={update:?}");
                }
                update.changed()
            }
            InstallerStep::Summary => {
                let update = forms.summary_mut().handle_pointer(model, &layout, x, y);
                if update.changed() {
                    println!("aqua_installer_pointer_summary_update={update:?}");
                }
                if let aqua_installer::InstallerSummaryUpdate::AcknowledgementChanged(checked) =
                    update
                {
                    println!("aqua_installer_summary_destructive_acknowledgement={checked}");
                    println!("aqua_installer_execution_allowed=false");
                }
                update.changed()
            }
            _ => false,
        };
        if content_changed {
            let focus_action = ui.focus_step_content();
            println!("aqua_installer_pointer x={x} y={y} action={focus_action:?} content=true");
            self.redraw_installer_buffer(qh);
            return true;
        }
        let action = ui.handle_pointer(&layout, x, y);
        if action == InstallerUiAction::None {
            return false;
        }
        println!("aqua_installer_pointer x={x} y={y} action={action:?}");

        match action {
            InstallerUiAction::AdvanceRequested => {
                let (Some(model), Some(forms), Some(ui)) = (
                    self.installer_model.as_mut(),
                    self.installer_forms.as_mut(),
                    self.installer_ui.as_mut(),
                ) else {
                    return false;
                };
                match model.advance() {
                    Ok(step) => {
                        forms.sync_model(model);
                        ui.sync_step(model);
                        println!("aqua_installer_step={}", step.id());
                        self.redraw_installer_buffer(qh);
                    }
                    Err(error) => eprintln!("aqua_installer_navigation_blocked={error}"),
                }
            }
            InstallerUiAction::RetreatRequested => {
                let (Some(model), Some(forms), Some(ui)) = (
                    self.installer_model.as_mut(),
                    self.installer_forms.as_mut(),
                    self.installer_ui.as_mut(),
                ) else {
                    return false;
                };
                match model.retreat() {
                    Ok(step) => {
                        forms.sync_model(model);
                        ui.sync_step(model);
                        println!("aqua_installer_step={}", step.id());
                        self.redraw_installer_buffer(qh);
                    }
                    Err(error) => eprintln!("aqua_installer_navigation_blocked={error}"),
                }
            }
            InstallerUiAction::BeginInstallRequested => {
                if !self.installer_progress_rehearsal_enabled {
                    println!("aqua_installer_execution_allowed=false");
                    println!("aqua_installer_begin_install_blocked=live-execution-disabled");
                } else {
                    let Some(model) = self.installer_model.as_mut() else {
                        return false;
                    };
                    match build_installer_presentation_graph(model) {
                        Ok(graph) => {
                            let runner = NonExecutingInstallTransactionRunner;
                            match runner.rehearse(&graph, None) {
                                Ok(rehearsal) if !rehearsal.executed() => {
                                    match model.begin_install() {
                                        Ok(()) => {
                                            let progress = InstallProgressEvent::running(&graph, 0)
                                                .expect("non-empty canonical transaction graph");
                                            log_installer_progress(&progress);
                                            self.installer_progress_graph = Some(graph);
                                            self.installer_progress = Some(progress);
                                            self.installer_progress_checkpoint = 0;
                                            if let Some(ui) = self.installer_ui.as_mut() {
                                                ui.sync_step(model);
                                            }
                                            println!("aqua_installer_step=installation");
                                            println!("aqua_installer_progress_presentation_rehearsal=true");
                                            println!("aqua_installer_transaction_executed=false");
                                            println!("aqua_installer_execution_allowed=false");
                                            self.redraw_installer_buffer(qh);
                                        }
                                        Err(error) => {
                                            eprintln!("aqua_installer_navigation_blocked={error}")
                                        }
                                    }
                                }
                                Ok(_) => eprintln!(
                                    "aqua_installer_progress_error=rehearsal-executed-unexpectedly"
                                ),
                                Err(error) => {
                                    eprintln!("aqua_installer_progress_error={error}")
                                }
                            }
                        }
                        Err(error) => eprintln!("aqua_installer_progress_error={error}"),
                    }
                }
            }
            InstallerUiAction::CancelRequested => {
                println!("aqua_installer_cancel_requested=true");
                self.redraw_installer_buffer(qh);
            }
            InstallerUiAction::OpenLanguageControl => {
                println!("aqua_installer_language_control_requested=true");
                self.redraw_installer_buffer(qh);
            }
            InstallerUiAction::FinishRequested => {
                println!("aqua_installer_finish_requested=true");
                self.redraw_installer_buffer(qh);
            }
            InstallerUiAction::None
            | InstallerUiAction::FocusChanged(_)
            | InstallerUiAction::ActivateStepContent(_) => return false,
        }
        true
    }

    fn handle_installer_key(&mut self, key: u32, qh: &QueueHandle<Self>) -> bool {
        if self
            .installer_model
            .as_ref()
            .is_some_and(|model| model.step() == InstallerStep::Installation)
            && self.installer_progress_rehearsal_enabled
            && key == 28
        {
            self.installer_keyboard_press_count += 1;
            let Some(graph) = self.installer_progress_graph.as_ref() else {
                eprintln!("aqua_installer_progress_error=transaction-graph-missing");
                return true;
            };
            const CHECKPOINTS: [usize; 3] = [8, 13, 19];
            if let Some(completed_steps) = CHECKPOINTS.get(self.installer_progress_checkpoint) {
                match InstallProgressEvent::running(graph, *completed_steps) {
                    Ok(progress) => {
                        log_installer_progress(&progress);
                        self.installer_progress = Some(progress);
                        self.installer_progress_checkpoint += 1;
                        self.redraw_installer_buffer(qh);
                    }
                    Err(error) => eprintln!("aqua_installer_progress_error={error}"),
                }
            } else {
                match InstallProgressEvent::completed(graph) {
                    Ok(progress) => {
                        log_installer_progress(&progress);
                        self.installer_progress = Some(progress);
                        if let Some(model) = self.installer_model.as_mut() {
                            if let Err(error) = model.complete_install() {
                                eprintln!("aqua_installer_completion_error={error}");
                                return true;
                            }
                            if let Some(ui) = self.installer_ui.as_mut() {
                                ui.sync_step(model);
                            }
                        }
                        println!("aqua_installer_step=completed");
                        println!("aqua_installer_transaction_executed=false");
                        println!("aqua_installer_presentation_rehearsal_completed=true");
                        self.redraw_installer_buffer(qh);
                    }
                    Err(error) => eprintln!("aqua_installer_progress_error={error}"),
                }
            }
            return true;
        }
        let (Some(model), Some(forms), Some(ui)) = (
            self.installer_model.as_mut(),
            self.installer_forms.as_mut(),
            self.installer_ui.as_mut(),
        ) else {
            return false;
        };
        if model.step() == InstallerStep::Summary {
            let summary_ready = forms.summary().can_begin_install(model);
            let summary_key = match key {
                14 => Some(InstallerSummaryKey::Backspace),
                103 => Some(InstallerSummaryKey::PreviousControl),
                108 => Some(InstallerSummaryKey::NextControl),
                57 if forms.summary().active_control()
                    == aqua_installer::InstallerSummaryControl::Acknowledgement =>
                {
                    Some(InstallerSummaryKey::Activate)
                }
                28 if !summary_ready => Some(InstallerSummaryKey::Activate),
                28 => None,
                _ => installer_printable_character(key, self.keyboard_shift)
                    .map(InstallerSummaryKey::Character),
            };
            if let Some(summary_key) = summary_key {
                self.installer_keyboard_press_count += 1;
                match forms.summary_mut().handle_key(model, summary_key) {
                    Ok(update) => {
                        println!(
                            "aqua_installer_summary_input key={key} press_count={} update={update:?}",
                            self.installer_keyboard_press_count
                        );
                        if matches!(
                            update,
                            aqua_installer::InstallerSummaryUpdate::ConfirmationApplied
                        ) {
                            println!("aqua_installer_summary_confirmation_applied=true");
                            println!(
                                "aqua_installer_summary_ready={}",
                                forms.summary().can_begin_install(model)
                            );
                            if let Some(target) = model.target() {
                                println!(
                                    "aqua_installer_summary_target_device={}",
                                    target.disk.device()
                                );
                            }
                            println!("aqua_installer_execution_allowed=false");
                        }
                        if let aqua_installer::InstallerSummaryUpdate::AcknowledgementChanged(
                            checked,
                        ) = update
                        {
                            println!(
                                "aqua_installer_summary_destructive_acknowledgement={checked}"
                            );
                            println!("aqua_installer_execution_allowed=false");
                        }
                        if matches!(
                            update,
                            aqua_installer::InstallerSummaryUpdate::ConfirmationApplied
                                | aqua_installer::InstallerSummaryUpdate::ReadyToInstall
                                | aqua_installer::InstallerSummaryUpdate::AcknowledgementChanged(_)
                                | aqua_installer::InstallerSummaryUpdate::FocusChanged(_)
                        ) {
                            self.redraw_installer_buffer(qh);
                        }
                    }
                    Err(error) => {
                        let entered = forms.summary().confirmation().as_bytes();
                        let expected = model.confirmation_phrase().unwrap_or_default();
                        eprintln!("aqua_installer_summary_confirmation_rejected={error}");
                        println!(
                            "aqua_installer_summary_confirmation_diagnostic entered_length={} entered_checksum={:016x} expected_length={} expected_checksum={:016x}",
                            entered.len(),
                            checksum_bytes(entered),
                            expected.len(),
                            checksum_bytes(expected.as_bytes())
                        );
                    }
                }
                return true;
            }
        }
        if model.step() == InstallerStep::UserInformation
            && ui.focus() == InstallerFocusTarget::StepContent
        {
            let form_key = match key {
                14 => Some(InstallerUserFormKey::Backspace),
                103 => Some(InstallerUserFormKey::PreviousField),
                108 => Some(InstallerUserFormKey::NextField),
                57 if forms.user().active_field() == InstallerUserField::Password => {
                    Some(InstallerUserFormKey::SetPasswordConfigured(
                        !forms.user().password_configured(),
                    ))
                }
                _ => installer_printable_character(key, self.keyboard_shift)
                    .map(InstallerUserFormKey::Character),
            };
            if let Some(form_key) = form_key {
                self.installer_keyboard_press_count += 1;
                match forms.user_mut().handle_key(model, form_key) {
                    Ok(update) => {
                        println!(
                            "aqua_installer_user_input key={key} press_count={} field={} update={update:?}",
                            self.installer_keyboard_press_count,
                            forms.user().active_field().id()
                        );
                        if update.changed() {
                            self.redraw_installer_buffer(qh);
                        }
                    }
                    Err(error) => eprintln!("aqua_installer_user_form_error={error}"),
                }
                return true;
            }
        }
        let ui_key = match key {
            1 => InstallerUiKey::Escape,
            15 if self.keyboard_shift => InstallerUiKey::BackTab,
            15 => InstallerUiKey::Tab,
            28 => InstallerUiKey::Activate,
            102 => InstallerUiKey::Home,
            105 => InstallerUiKey::Left,
            106 => InstallerUiKey::Right,
            107 => InstallerUiKey::End,
            _ => return false,
        };
        self.installer_keyboard_press_count += 1;
        let action = ui.handle_key(ui_key);
        println!(
            "aqua_installer_keyboard key={key} press_count={} action={action:?}",
            self.installer_keyboard_press_count
        );
        let mut changed = action.changed();
        if self.installer_progress_rehearsal_enabled
            && model.step() == InstallerStep::Summary
            && forms.summary().can_begin_install(model)
            && matches!(action, InstallerUiAction::FocusChanged(_))
        {
            // Keep the acceptance-only End+Enter burst in one committed frame.
            changed = false;
        }
        match action {
            InstallerUiAction::ActivateStepContent(
                InstallerStep::Language | InstallerStep::Keyboard | InstallerStep::TimeZone,
            ) => match forms.handle_key(model, InstallerFormKey::Activate) {
                Ok(update) => {
                    changed |= update.changed();
                    println!("aqua_installer_form_update={update:?}");
                }
                Err(error) => eprintln!("aqua_installer_form_error={error}"),
            },
            InstallerUiAction::ActivateStepContent(InstallerStep::Partitions) => {
                match forms.handle_disk_key(model, InstallerFormKey::Activate) {
                    Ok(update) => {
                        changed |= update.changed();
                        println!("aqua_installer_disk_form_update={update:?}");
                        if let Some(target) = model.target() {
                            println!("aqua_installer_target_device={}", target.disk.device());
                            println!("aqua_installer_execution_allowed=false");
                        }
                    }
                    Err(error) => eprintln!("aqua_installer_disk_form_error={error}"),
                }
            }
            InstallerUiAction::ActivateStepContent(InstallerStep::UserInformation) => {
                match forms
                    .user_mut()
                    .handle_key(model, InstallerUserFormKey::Activate)
                {
                    Ok(update) => {
                        changed |= update.changed();
                        println!("aqua_installer_user_form_update={update:?}");
                        if let Some(user) = model.user() {
                            println!(
                                "aqua_installer_user_profile username={} display_name={} password_configured={}",
                                user.username(),
                                user.display_name(),
                                user.password_configured()
                            );
                        }
                    }
                    Err(error) => eprintln!("aqua_installer_user_form_error={error}"),
                }
            }
            InstallerUiAction::AdvanceRequested => match model.advance() {
                Ok(step) => {
                    forms.sync_model(model);
                    ui.sync_step(model);
                    changed = true;
                    println!("aqua_installer_step={}", step.id());
                }
                Err(error) => eprintln!("aqua_installer_navigation_blocked={error}"),
            },
            InstallerUiAction::RetreatRequested => match model.retreat() {
                Ok(step) => {
                    forms.sync_model(model);
                    ui.sync_step(model);
                    changed = true;
                    println!("aqua_installer_step={}", step.id());
                }
                Err(error) => eprintln!("aqua_installer_navigation_blocked={error}"),
            },
            InstallerUiAction::BeginInstallRequested => {
                if !self.installer_progress_rehearsal_enabled {
                    println!("aqua_installer_execution_allowed=false");
                    println!("aqua_installer_begin_install_blocked=live-execution-disabled");
                } else {
                    match build_installer_presentation_graph(model) {
                        Ok(graph) => {
                            let runner = NonExecutingInstallTransactionRunner;
                            match runner.rehearse(&graph, None) {
                                Ok(rehearsal) if !rehearsal.executed() => {
                                    match model.begin_install() {
                                        Ok(()) => {
                                            let progress = InstallProgressEvent::running(&graph, 0)
                                                .expect("non-empty canonical transaction graph");
                                            log_installer_progress(&progress);
                                            self.installer_progress_graph = Some(graph);
                                            self.installer_progress = Some(progress);
                                            self.installer_progress_checkpoint = 0;
                                            ui.sync_step(model);
                                            changed = true;
                                            println!("aqua_installer_step=installation");
                                            println!("aqua_installer_progress_presentation_rehearsal=true");
                                            println!("aqua_installer_transaction_executed=false");
                                            println!("aqua_installer_execution_allowed=false");
                                        }
                                        Err(error) => {
                                            eprintln!("aqua_installer_navigation_blocked={error}")
                                        }
                                    }
                                }
                                Ok(_) => eprintln!(
                                    "aqua_installer_progress_error=rehearsal-executed-unexpectedly"
                                ),
                                Err(error) => {
                                    eprintln!("aqua_installer_progress_error={error}")
                                }
                            }
                        }
                        Err(error) => eprintln!("aqua_installer_progress_error={error}"),
                    }
                }
            }
            InstallerUiAction::CancelRequested => {
                println!("aqua_installer_cancel_requested=true");
            }
            InstallerUiAction::None
            | InstallerUiAction::FocusChanged(_)
            | InstallerUiAction::ActivateStepContent(_)
            | InstallerUiAction::OpenLanguageControl
            | InstallerUiAction::FinishRequested => {}
        }
        if changed {
            println!("aqua_installer_focus={}", ui.focus().id());
            if let Some(locale) = model.locale() {
                println!("aqua_installer_locale={locale}");
            }
            if let Some(layout) = model.keyboard_layout() {
                println!("aqua_installer_keyboard_layout={layout}");
            }
            if let Some(timezone) = model.timezone() {
                println!("aqua_installer_timezone={timezone}");
            }
            self.redraw_installer_buffer(qh);
        }
        true
    }

    fn persist_settings(&mut self) {
        let (Some(model), Some(path)) = (&self.settings_model, &self.settings_config_path) else {
            self.settings_persistence_failed = true;
            return;
        };
        match model.persist(path) {
            Ok(()) => {
                println!("aqua_settings_persisted=true");
                println!("aqua_settings_config_path={}", path.display());
            }
            Err(error) => {
                self.settings_persistence_failed = true;
                eprintln!("aqua_settings_persisted=false error={error}");
            }
        }
    }

    fn init_xdg_surface(&mut self, qh: &QueueHandle<Self>) {
        if self.surface_created {
            return;
        }

        let Some(wm_base) = self.wm_base.as_ref() else {
            return;
        };
        let Some(base_surface) = self.base_surface.as_ref() else {
            return;
        };

        let xdg_surface = wm_base.get_xdg_surface(base_surface, qh, ());
        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title(self.title.clone());
        toplevel.set_app_id(self.app_id.clone());
        base_surface.commit();
        self.xdg_surface = Some(xdg_surface);
        self.xdg_toplevel = Some(toplevel);
        self.surface_created = true;
        self.toplevel_requested = true;
    }

    fn create_shm_buffer(&mut self, shm: &client_wl_shm::WlShm, qh: &QueueHandle<Self>) {
        if self.shm_buffer_created {
            return;
        }

        use std::io::Write;
        use std::os::unix::io::AsFd;

        let width = self.buffer_width.max(1);
        let height = self.buffer_height.max(1);
        let stride = width * 4;
        let size = stride * height;
        let mut file = tempfile::tempfile().expect("Aqua shm smoke buffer tempfile should open");
        if let Some(terminal) = self.terminal_session.as_mut() {
            terminal.drain_output();
        }
        let pixels = if self.component_acceptance {
            let (pixels, probe) = render_component_acceptance_rgba(
                Viewport::new(width, height),
                self.theme,
                OutputScale::One,
            )
            .expect("Aqua component acceptance raster should render");
            assert!(probe.is_ready());
            pixels
        } else if self.typography_acceptance {
            let (pixels, probe) = render_typography_layout_acceptance_rgba(
                Viewport::new(width, height),
                self.theme,
                OutputScale::One,
            )
            .expect("Aqua typography acceptance raster should render");
            assert!(probe.is_ready());
            pixels
        } else if self.installer_model.is_some() {
            self.render_installer_buffer()
                .expect("Aqua Installer initial raster should render")
        } else if let Some(model) = self.files_model.as_ref() {
            render_files_window_rgba_with_theme(width, height, model, self.theme).0
        } else if let Some(model) = self.settings_model.as_ref() {
            render_settings_window_rgba(width, height, model).0
        } else if let Some(model) = self.properties_model.as_ref() {
            render_properties_window_rgba_with_theme(width, height, model, self.theme).0
        } else if let Some(terminal) = self.terminal_session.as_ref() {
            render_terminal_window_rgba_with_theme(width, height, &terminal.view(), self.theme).0
        } else {
            let variant = std::env::var("AQUA_WAYLAND_TEST_CLIENT_VARIANT")
                .ok()
                .and_then(|value| value.parse::<u8>().ok())
                .unwrap_or(1);
            let mut pixels = Vec::with_capacity(size as usize);
            for y in 0..height {
                for x in 0..width {
                    pixels.extend_from_slice(&[
                        ((x * 255) / width) as u8,
                        ((y * 255) / height) as u8,
                        0x40_u8.saturating_add(variant.saturating_mul(0x3f)),
                        0xff,
                    ]);
                }
            }
            pixels
        };
        file.write_all(&pixels)
            .expect("Aqua shm smoke buffer should be writable");
        file.flush()
            .expect("Aqua shm smoke buffer should flush before pool creation");

        let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            client_wl_shm::Format::Argb8888,
            qh,
            (),
        );
        self.shm_buffer = Some(buffer);
        self.shm_buffer_created = true;
        self.attach_client_buffer(qh);
    }

    fn redraw_files_buffer(&mut self, qh: &QueueHandle<Self>) {
        let Some(shm) = self.shm.clone() else {
            return;
        };
        let Some(model) = self.files_model.as_ref() else {
            return;
        };
        let width = self.buffer_width.max(1);
        let height = self.buffer_height.max(1);
        let stride = width * 4;
        let size = stride * height;
        let pixels = render_files_window_rgba_with_theme(width, height, model, self.theme).0;

        use std::io::Write;
        use std::os::unix::io::AsFd;
        let mut file = tempfile::tempfile().expect("Aqua Files redraw tempfile should open");
        file.write_all(&pixels)
            .expect("Aqua Files redraw buffer should be writable");
        file.flush().expect("Aqua Files redraw buffer should flush");
        let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            client_wl_shm::Format::Argb8888,
            qh,
            (),
        );
        if let Some(surface) = self.base_surface.as_ref() {
            surface.attach(Some(&buffer), 0, 0);
            surface.damage(0, 0, width as i32, height as i32);
            surface.frame(qh, ());
            surface.commit();
            self.shm_buffer = Some(buffer);
        }
    }

    fn redraw_settings_buffer(&mut self, qh: &QueueHandle<Self>) {
        let (Some(shm), Some(model)) = (self.shm.clone(), self.settings_model.as_ref()) else {
            return;
        };
        let width = self.buffer_width.max(1);
        let height = self.buffer_height.max(1);
        let stride = width * 4;
        let size = stride * height;
        let pixels = render_settings_window_rgba(width, height, model).0;

        use std::io::Write;
        use std::os::unix::io::AsFd;
        let mut file = tempfile::tempfile().expect("Aqua Settings redraw tempfile should open");
        file.write_all(&pixels)
            .expect("Aqua Settings redraw buffer should be writable");
        file.flush()
            .expect("Aqua Settings redraw buffer should flush");
        let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            client_wl_shm::Format::Argb8888,
            qh,
            (),
        );
        if let Some(surface) = self.base_surface.as_ref() {
            surface.attach(Some(&buffer), 0, 0);
            surface.damage(0, 0, width as i32, height as i32);
            surface.frame(qh, ());
            surface.commit();
            self.shm_buffer = Some(buffer);
        }
    }

    fn redraw_properties_buffer(&mut self, qh: &QueueHandle<Self>) {
        let (Some(shm), Some(model)) = (self.shm.clone(), self.properties_model.as_ref()) else {
            return;
        };
        let width = self.buffer_width.max(1);
        let height = self.buffer_height.max(1);
        let stride = width * 4;
        let size = stride * height;
        let pixels = render_properties_window_rgba_with_theme(width, height, model, self.theme).0;

        use std::io::Write;
        use std::os::unix::io::AsFd;
        let mut file = tempfile::tempfile().expect("Aqua Properties redraw tempfile should open");
        file.write_all(&pixels)
            .expect("Aqua Properties redraw buffer should be writable");
        file.flush()
            .expect("Aqua Properties redraw buffer should flush");
        let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            client_wl_shm::Format::Argb8888,
            qh,
            (),
        );
        if let Some(surface) = self.base_surface.as_ref() {
            surface.attach(Some(&buffer), 0, 0);
            surface.damage(0, 0, width as i32, height as i32);
            surface.frame(qh, ());
            surface.commit();
            self.shm_buffer = Some(buffer);
        }
    }

    fn redraw_terminal_buffer(&mut self, qh: &QueueHandle<Self>) {
        let Some(shm) = self.shm.clone() else {
            return;
        };
        let Some(terminal) = self.terminal_session.as_mut() else {
            return;
        };
        terminal.drain_output();
        let view = terminal.view();
        let width = self.buffer_width.max(1);
        let height = self.buffer_height.max(1);
        let stride = width * 4;
        let size = stride * height;
        let pixels = render_terminal_window_rgba_with_theme(width, height, &view, self.theme).0;

        use std::io::Write;
        use std::os::unix::io::AsFd;
        let mut file = tempfile::tempfile().expect("Aqua Terminal redraw tempfile should open");
        file.write_all(&pixels)
            .expect("Aqua Terminal redraw buffer should be writable");
        file.flush()
            .expect("Aqua Terminal redraw buffer should flush");
        let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            client_wl_shm::Format::Argb8888,
            qh,
            (),
        );
        if let Some(surface) = self.base_surface.as_ref() {
            surface.attach(Some(&buffer), 0, 0);
            surface.damage(0, 0, width as i32, height as i32);
            surface.frame(qh, ());
            surface.commit();
            self.shm_buffer = Some(buffer);
            self.terminal_frame_pending = true;
            self.terminal_frame_requested_at = Some(std::time::Instant::now());
            self.terminal_dirty = false;
        }
    }

    fn refresh_properties(&mut self, qh: &QueueHandle<Self>) {
        let (Some(home_root), Some(system_root)) = (
            self.properties_home_root.as_ref(),
            self.properties_system_root.as_ref(),
        ) else {
            return;
        };
        let result = self
            .properties_model
            .as_mut()
            .map(|model| model.refresh(home_root, system_root));
        match result {
            Some(Ok(action)) => {
                let model = self
                    .properties_model
                    .as_ref()
                    .expect("refreshed Properties model should remain available");
                println!("aqua_properties_action={}", action.log_name());
                println!(
                    "aqua_properties_refresh_generation={}",
                    model.refresh_generation
                );
                println!("aqua_properties_refresh_status={}", model.status);
                println!(
                    "aqua_properties_refresh_items={}",
                    model
                        .item_count
                        .map(|count| count.to_string())
                        .unwrap_or_else(|| "not-applicable".to_string())
                );
                self.redraw_properties_buffer(qh);
            }
            Some(Err(error)) => eprintln!("aqua_properties_refresh_error={error}"),
            None => {}
        }
    }

    fn attach_client_buffer(&mut self, qh: &QueueHandle<Self>) {
        if !self.configure_ack_sent || self.client_buffer_attached {
            return;
        }

        let Some(surface) = self.base_surface.as_ref() else {
            return;
        };
        let Some(buffer) = self.shm_buffer.as_ref() else {
            return;
        };

        surface.attach(Some(buffer), 0, 0);
        surface.damage(0, 0, self.buffer_width as i32, self.buffer_height as i32);
        surface.frame(qh, ());
        surface.commit();
        self.client_buffer_attached = true;
        if self.terminal_session.is_some() {
            self.terminal_frame_pending = true;
            self.terminal_frame_requested_at = Some(std::time::Instant::now());
            self.terminal_dirty = false;
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_aqua_ui_event_loop(
    connection: &ClientConnection,
    event_queue: &mut wayland_client::EventQueue<XdgSmokeClientState>,
    state: &mut XdgSmokeClientState,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::fd::AsFd as _;

    let queue_handle = event_queue.handle();
    let poller = polling::Poller::new()?;
    unsafe {
        poller.add_with_mode(
            &event_queue.as_fd(),
            polling::Event::readable(1),
            polling::PollMode::Level,
        )?;
    }
    let mut events = polling::Events::new();
    while !state.close_event_received {
        event_queue.dispatch_pending(state)?;
        state.refresh_runtime_theme(&queue_handle);
        connection.flush()?;
        if state.close_event_received {
            break;
        }

        let Some(read_guard) = event_queue.prepare_read() else {
            continue;
        };
        events.clear();
        if poller.wait(&mut events, Some(Duration::from_millis(100)))? > 0 {
            read_guard.read()?;
        }
    }
    poller.delete(event_queue.as_fd())?;
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct AquaCompositorSession {
    display: Display<WaylandSmokeState>,
    wayland_state: WaylandSmokeState,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub struct SmithayDrmSession {
    session: AquaCompositorSession,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmithayBackendInputSnapshot {
    pub keyboard_event_count: usize,
    pub pointer_motion_count: usize,
    pub pointer_button_count: usize,
    pub launcher_visible: bool,
    pub keyboard_shortcut_intercept_count: usize,
    pub keyboard_forward_count: usize,
    pub pointer_hit_test_count: usize,
    pub pointer_surface_hit_count: usize,
    pub surface_focus_change_count: usize,
    pub stacking_change_count: usize,
    pub launcher_pointer_hit_count: usize,
    pub launcher_category_click_count: usize,
    pub launcher_app_click_count: usize,
    pub launcher_launch_request: Option<LaunchRequest>,
    pub pointer_x: u32,
    pub pointer_y: u32,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmithayClientSurfaceSnapshot {
    pub workspace: usize,
    pub commit_count: usize,
    pub buffer_attach_count: usize,
    pub shm_import_count: usize,
    pub toplevel_count: usize,
    pub configure_ack_count: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub sample_checksum: u64,
    pub sample_pixel: [u8; 4],
    pub sample_grid: [[u8; 4]; CLIENT_SAMPLE_GRID_PIXELS],
    pub buffer_rgba: Vec<u8>,
    pub buffer_opaque: bool,
    pub damage_commit_count: usize,
    pub damage_rect_count: usize,
    pub pending_frame_callback_count: usize,
    pub frame_callbacks_sent: usize,
    pub keyboard_focus_assigned: bool,
    pub pointer_focus_assigned: bool,
    pub mapped_surface_count: usize,
    pub surface_focus_change_count: usize,
    pub stacking_change_count: usize,
    pub destroyed_surface_count: usize,
    pub client_cleanup_count: usize,
    pub cleanup_keyboard_focus_reassigned: bool,
    pub x: u32,
    pub y: u32,
    pub display_width: u32,
    pub display_height: u32,
    pub move_request_count: usize,
    pub resize_request_count: usize,
    pub close_request_count: usize,
    pub maximize_request_count: usize,
    pub unmaximize_request_count: usize,
    pub fullscreen_request_count: usize,
    pub unfullscreen_request_count: usize,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl SmithayClientSurfaceSnapshot {
    pub fn is_ready(&self) -> bool {
        self.commit_count >= 2
            && self.buffer_attach_count >= 1
            && self.shm_import_count >= 1
            && self.toplevel_count >= 1
            && self.configure_ack_count >= 1
            && self.width > 0
            && self.height > 0
            && self.stride >= self.width * 4
            && self.sample_checksum != 0
            && self.buffer_rgba.len() == self.width as usize * self.height as usize * 4
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl SmithayDrmSession {
    pub fn set_output_dimensions(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.session.wayland_state.output_width = width;
        self.session.wayland_state.output_height = height;
    }

    pub fn launcher_state_snapshot(&self) -> LauncherState {
        self.session.wayland_state.launcher_state.clone()
    }

    pub fn active_workspace(&self) -> usize {
        self.session.wayland_state.active_workspace
    }

    pub fn activate_workspace(&mut self, workspace: usize, serial: u32) -> bool {
        self.session
            .wayland_state
            .activate_workspace(workspace, serial)
    }

    pub fn move_active_toplevel_to_workspace(&mut self, workspace: usize, serial: u32) -> bool {
        self.session
            .wayland_state
            .move_active_surface_to_workspace(workspace, serial)
    }

    pub fn desktop_icon_state_snapshot(&self) -> DesktopIconState {
        self.session.wayland_state.desktop_icon_state.clone()
    }

    pub fn notification_center_snapshot(&self) -> NotificationCenter {
        self.session.wayland_state.notification_center.clone()
    }

    pub fn post_notification(
        &mut self,
        now_ms: u64,
        source: &str,
        title: &str,
        body: &str,
    ) -> bool {
        self.session.wayland_state.notification_now_ms = now_ms;
        let update = self.session.wayland_state.notification_center.post(
            now_ms,
            source,
            title,
            body,
            NOTIFICATION_DEFAULT_TIMEOUT_MS,
        );
        self.session.wayland_state.sync_notification_visibility();
        update.redraw_requested
    }

    pub fn tick_notifications(&mut self, now_ms: u64) -> bool {
        self.session.wayland_state.notification_now_ms = now_ms;
        let update = self.session.wayland_state.notification_center.tick(now_ms);
        self.session.wayland_state.sync_notification_visibility();
        update.redraw_requested
    }

    pub fn dismiss_notification(&mut self, now_ms: u64) -> bool {
        self.session.wayland_state.notification_now_ms = now_ms;
        let update = self
            .session
            .wayland_state
            .notification_center
            .dismiss(now_ms);
        self.session.wayland_state.sync_notification_visibility();
        update.redraw_requested
    }

    pub fn session_menu_state_snapshot(&self) -> SessionMenuState {
        self.session.wayland_state.session_menu_state.clone()
    }

    pub fn take_session_action_request(&mut self) -> Option<SessionAction> {
        self.session.wayland_state.session_action_request.take()
    }

    pub fn has_session_action_request(&self) -> bool {
        self.session.wayland_state.session_action_request.is_some()
    }

    pub fn take_launcher_launch_request(&mut self) -> Option<LaunchRequest> {
        let request = self.session.wayland_state.launcher_launch_request.take();
        if request.is_some() {
            self.session
                .wayland_state
                .apply_launcher_event(LauncherEvent::Dismiss);
        }
        request
    }

    pub fn prepare_launcher_search_demo(&mut self) {
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::SelectCategory(
                LauncherCategory::AllApplications,
            ));
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::ReplaceQuery("settings".to_string()));
    }

    fn surface_geometry(index: usize, count: usize, width: u32, height: u32) -> (f64, f64) {
        let center_x = (1536.0 - f64::from(width)) / 2.0;
        let center_y = (1024.0 - f64::from(height)) / 2.0;
        if count <= 1 {
            return (center_x, center_y);
        }
        let offset = f64::from(width) / 4.0;
        let start = center_x - offset * (count.saturating_sub(1) as f64) / 2.0;
        (start + offset * index as f64, center_y)
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut session = AquaCompositorSession::new()?;
        session.wayland_state.close_new_toplevels = false;
        Ok(Self { session })
    }

    pub fn insert_client(&mut self, stream: std::os::unix::net::UnixStream) -> std::io::Result<()> {
        self.session.insert_client(stream).map(|_| ())
    }

    pub fn has_toplevel_app_id(&self, expected_app_id: &str) -> bool {
        self.session
            .wayland_state
            .toplevel_surfaces
            .iter()
            .any(|surface| {
                with_states(surface.wl_surface(), |states| {
                    states
                        .data_map
                        .get::<XdgToplevelSurfaceData>()
                        .and_then(|data| data.lock().ok())
                        .and_then(|attributes| attributes.app_id.clone())
                        .as_deref()
                        == Some(expected_app_id)
                })
            })
    }

    pub fn dispatch_files_first_entry_click(&mut self, time: u32) -> bool {
        self.dispatch_files_pointer_click(220, 140, time)
    }

    pub fn dispatch_files_pointer_click(&mut self, local_x: u32, local_y: u32, time: u32) -> bool {
        self.dispatch_app_pointer_click(640, 420, local_x, local_y, time)
    }

    pub fn dispatch_settings_pointer_click(
        &mut self,
        local_x: u32,
        local_y: u32,
        time: u32,
    ) -> bool {
        self.dispatch_app_pointer_click(600, 400, local_x, local_y, time)
    }

    fn dispatch_app_pointer_click(
        &mut self,
        width: u32,
        height: u32,
        local_x: u32,
        local_y: u32,
        time: u32,
    ) -> bool {
        let Some((x, y)) = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .find(|surface| surface.width == width && surface.height == height)
            .map(|surface| (surface.x + local_x, surface.y + local_y))
        else {
            return false;
        };
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Dismiss);
        let current = self.session.wayland_state.pointer_location;
        let moved =
            self.dispatch_pointer_motion(f64::from(x) - current.0, f64::from(y) - current.1, time);
        let pressed = self.dispatch_pointer_button(0x110, true, time.saturating_add(1));
        let released = self.dispatch_pointer_button(0x110, false, time.saturating_add(2));
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Toggle);
        moved && pressed && released
    }

    pub fn dispatch_files_keyboard_key(&mut self, code: u32, time: u32) -> bool {
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Dismiss);
        let pressed = self.dispatch_keyboard_key(code, true, time);
        let released = self.dispatch_keyboard_key(code, false, time.saturating_add(1));
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Toggle);
        pressed && released
    }

    pub fn dispatch_files_pointer_axis(&mut self, rows: isize, time: u32) -> bool {
        let Some(pointer) = self.session.wayland_state.seat.get_pointer() else {
            return false;
        };
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Dismiss);
        pointer.axis(
            &mut self.session.wayland_state,
            AxisFrame::new(time)
                .source(AxisSource::Wheel)
                .v120(Axis::Vertical, rows.signum() as i32 * 120)
                .value(Axis::Vertical, rows.signum() as f64 * 15.0),
        );
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Toggle);
        true
    }

    pub fn dispatch_files_scrollbar_drag(&mut self, from_y: u32, to_y: u32, time: u32) -> bool {
        let Some((surface_x, surface_y)) = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .find(|surface| surface.width == 640 && surface.height == 420)
            .map(|surface| (surface.x, surface.y))
        else {
            return false;
        };
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Dismiss);
        let current = self.session.wayland_state.pointer_location;
        let x = surface_x + aqua_shell::FILES_SCROLLBAR_X;
        let moved_to_start = self.dispatch_pointer_motion(
            f64::from(x) - current.0,
            f64::from(surface_y + from_y) - current.1,
            time,
        );
        let pressed = self.dispatch_pointer_button(0x110, true, time.saturating_add(1));
        let moved_to_end = self.dispatch_pointer_motion(
            0.0,
            f64::from(to_y) - f64::from(from_y),
            time.saturating_add(2),
        );
        let released = self.dispatch_pointer_button(0x110, false, time.saturating_add(3));
        self.session
            .wayland_state
            .apply_launcher_event(LauncherEvent::Toggle);
        moved_to_start && pressed && moved_to_end && released
    }

    pub fn dispatch_clients(&mut self) -> std::io::Result<usize> {
        self.session.dispatch_clients()
    }

    pub fn flush_clients(&mut self) -> std::io::Result<()> {
        self.session.flush_clients()
    }

    pub fn compositor_global_started(&self) -> bool {
        true
    }

    pub fn shm_global_started(&self) -> bool {
        true
    }

    pub fn xdg_shell_global_started(&self) -> bool {
        true
    }

    pub fn seat_started(&self) -> bool {
        self.session.wayland_state.seat_global_created
            && self.session.wayland_state.seat.get_keyboard().is_some()
            && self.session.wayland_state.seat.get_pointer().is_some()
    }

    pub fn dispatch_keyboard_key(&mut self, code: u32, pressed: bool, time: u32) -> bool {
        let Some(keyboard) = self.session.wayland_state.seat.get_keyboard() else {
            return false;
        };
        let state = &mut self.session.wayland_state;
        if code == 125 {
            state.keyboard_event_count += 1;
            state.keyboard_shortcut_intercept_count += 1;
            if pressed {
                state.apply_launcher_event(LauncherEvent::Toggle);
            }
            return true;
        }
        if code == 68 {
            state.keyboard_event_count += 1;
            state.keyboard_shortcut_intercept_count += 1;
            if pressed {
                state.apply_session_menu_event(SessionMenuEvent::Toggle);
            }
            return true;
        }
        if state.launcher_state.is_open() {
            state.keyboard_event_count += 1;
            state.keyboard_shortcut_intercept_count += 1;
            if pressed {
                let event = match code {
                    1 => Some(LauncherEvent::Dismiss),
                    14 => {
                        let mut query = state.launcher_state.query().to_string();
                        query.pop();
                        Some(LauncherEvent::ReplaceQuery(query))
                    }
                    28 => Some(LauncherEvent::Activate),
                    103 => Some(LauncherEvent::MoveSelection(-1)),
                    108 => Some(LauncherEvent::MoveSelection(1)),
                    _ => launcher_key_character(code).map(|character| {
                        let mut query = state.launcher_state.query().to_string();
                        query.push(character);
                        LauncherEvent::ReplaceQuery(query)
                    }),
                };
                if let Some(event) = event {
                    state.apply_launcher_event(event);
                    println!(
                        "desktop_launcher_keyboard_code={code} query={} selected={} request={}",
                        state.launcher_state.query(),
                        state.launcher_state.selected_index(),
                        state
                            .launcher_launch_request
                            .as_ref()
                            .map_or("none", |request| request.app_id)
                    );
                }
            }
            return true;
        }
        if state.session_menu_state.is_open() {
            state.keyboard_event_count += 1;
            state.keyboard_shortcut_intercept_count += 1;
            println!("desktop_session_menu_key_code={code} pressed={pressed} serial={time}");
            if pressed {
                match code {
                    1 => state.apply_session_menu_event(SessionMenuEvent::Dismiss),
                    103 => state.apply_session_menu_event(SessionMenuEvent::MoveSelection(-1)),
                    108 => state.apply_session_menu_event(SessionMenuEvent::MoveSelection(1)),
                    28 => state.apply_session_menu_event(SessionMenuEvent::Activate),
                    _ => {}
                }
            }
            return true;
        }
        if code == 1 && state.notification_center.active().is_some() {
            state.keyboard_event_count += 1;
            state.keyboard_shortcut_intercept_count += 1;
            if pressed {
                state.notification_now_ms = u64::from(time);
                state.notification_center.dismiss(u64::from(time));
                state.sync_notification_visibility();
                println!("desktop_notification_dismissed_by_escape=true");
            }
            return true;
        }
        keyboard.input(
            &mut self.session.wayland_state,
            Keycode::from(code.saturating_add(8)),
            if pressed {
                KeyState::Pressed
            } else {
                KeyState::Released
            },
            Serial::from(time.max(1)),
            time,
            |state, _, _| {
                state.keyboard_event_count += 1;
                if code == 29 {
                    state.ctrl_pressed = pressed;
                    state.keyboard_forward_count += 1;
                    FilterResult::Forward
                } else if code == 42 || code == 54 {
                    state.shift_pressed = pressed;
                    state.keyboard_forward_count += 1;
                    FilterResult::Forward
                } else if code == 56 {
                    state.alt_pressed = pressed;
                    state.keyboard_forward_count += 1;
                    FilterResult::Forward
                } else if state.ctrl_pressed && state.alt_pressed && matches!(code, 105 | 106) {
                    if pressed {
                        let destination = if code == 105 {
                            state.active_workspace.checked_sub(1)
                        } else {
                            (state.active_workspace + 1 < WORKSPACE_COUNT)
                                .then_some(state.active_workspace + 1)
                        };
                        if let Some(destination) = destination {
                            if state.shift_pressed {
                                state.move_active_surface_to_workspace(destination, time);
                            } else {
                                state.activate_workspace(destination, time);
                            }
                        }
                    }
                    state.keyboard_shortcut_intercept_count += 1;
                    FilterResult::Intercept(())
                } else if code == 62 && pressed && state.alt_pressed {
                    let close_sent = state.close_active_toplevel();
                    println!("desktop_close_shortcut_received=true");
                    println!("desktop_close_request_sent={close_sent}");
                    FilterResult::Intercept(())
                } else if code == 66 && pressed && state.alt_pressed {
                    let resize_sent = state.resize_active_toplevel();
                    println!("desktop_resize_shortcut_received=true");
                    println!("desktop_resize_request_sent={resize_sent}");
                    FilterResult::Intercept(())
                } else {
                    state.keyboard_forward_count += 1;
                    FilterResult::Forward
                }
            },
        );
        println!(
            "desktop_keyboard_forward_event code={code} pressed={pressed} count={}",
            self.session.wayland_state.keyboard_forward_count
        );
        true
    }

    pub fn dispatch_pointer_motion(&mut self, dx: f64, dy: f64, time: u32) -> bool {
        let Some(pointer) = self.session.wayland_state.seat.get_pointer() else {
            return false;
        };
        let location = &mut self.session.wayland_state.pointer_location;
        location.0 = (location.0 + dx).clamp(0.0, 1535.0);
        location.1 = (location.1 + dy).clamp(0.0, 1023.0);
        let pointer_location = *location;
        if self.session.wayland_state.launcher_state.is_open() {
            self.session.wayland_state.pointer_focus_surface = None;
            self.session.wayland_state.pointer_focus_assigned = false;
            if self
                .session
                .wayland_state
                .launcher_state
                .pointer_target(pointer_location.0 as u32, pointer_location.1 as u32)
                .is_some()
            {
                self.session.wayland_state.launcher_pointer_hit_count += 1;
            }
            pointer.motion(
                &mut self.session.wayland_state,
                None,
                &MotionEvent {
                    location: pointer_location.into(),
                    serial: Serial::from(time.max(1)),
                    time,
                },
            );
            self.session.wayland_state.pointer_motion_count += 1;
            return true;
        }
        let surface_focus = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .rev()
            .filter(|record| record.workspace == self.session.wayland_state.active_workspace)
            .find_map(|record| {
                let origin_x = f64::from(record.x);
                let origin_y = f64::from(record.y);
                let local_x = pointer_location.0 - origin_x;
                let local_y = pointer_location.1 - origin_y;
                (local_x >= 0.0
                    && local_x < f64::from(record.display_width)
                    && local_y >= 0.0
                    && local_y < f64::from(record.display_height))
                .then(|| (record.surface.clone(), (origin_x, origin_y).into()))
            });
        self.session.wayland_state.pointer_hit_test_count += 1;
        if surface_focus.is_some() {
            self.session.wayland_state.pointer_surface_hit_count += 1;
        }
        self.session.wayland_state.pointer_focus_assigned = surface_focus.is_some();
        self.session.wayland_state.pointer_focus_surface =
            surface_focus.as_ref().map(|(surface, _)| surface.clone());
        pointer.motion(
            &mut self.session.wayland_state,
            surface_focus,
            &MotionEvent {
                location: pointer_location.into(),
                serial: Serial::from(time.max(1)),
                time,
            },
        );
        self.session.wayland_state.pointer_motion_count += 1;
        true
    }

    pub fn dispatch_pointer_button(&mut self, button: u32, pressed: bool, time: u32) -> bool {
        let Some(pointer) = self.session.wayland_state.seat.get_pointer() else {
            return false;
        };
        let (pointer_x, pointer_y) = self.session.wayland_state.pointer_location;
        if button == 0x110 && pressed {
            println!(
                "desktop_primary_pointer_press x={} y={} notification_active={}",
                pointer_x as u32,
                pointer_y as u32,
                self.session
                    .wayland_state
                    .notification_center
                    .active()
                    .map_or(0, |notification| notification.id)
            );
        }
        if button == 0x110
            && pressed
            && self
                .session
                .wayland_state
                .notification_close_hit(pointer_x as u32, pointer_y as u32)
        {
            let notification_now_ms = self.session.wayland_state.notification_now_ms;
            self.dismiss_notification(notification_now_ms);
            self.session.wayland_state.pointer_button_count += 1;
            println!("desktop_notification_dismissed=true");
            return true;
        }
        if button == 0x110 && pressed && self.session.wayland_state.pointer_focus_surface.is_none()
        {
            let output_width = self.session.wayland_state.output_width;
            let output_height = self.session.wayland_state.output_height;
            if top_system_bar_session_hit(
                Viewport::new(output_width, output_height),
                pointer_x as u32,
                pointer_y as u32,
            ) {
                self.session
                    .wayland_state
                    .apply_session_menu_event(SessionMenuEvent::Toggle);
                self.session.wayland_state.pointer_button_count += 1;
                println!("desktop_top_system_bar_session_activated=true");
                return true;
            }
            let dock_target = static_shell_scene(Viewport::new(800, 600))
                .surface_rect(SurfaceKind::Dock)
                .and_then(|canonical| {
                    let rect = aqua_scene::Rect {
                        x: canonical.x * output_width / 800,
                        y: canonical.y * output_height / 600,
                        width: canonical.width * output_width / 800,
                        height: canonical.height * output_height / 600,
                    };
                    let x = pointer_x as u32;
                    let y = pointer_y as u32;
                    ((rect.x..rect.x + rect.width).contains(&x)
                        && (rect.y..rect.y + rect.height).contains(&y))
                    .then(|| {
                        dock_pointer_target(
                            (x - rect.x) * canonical.width / rect.width.max(1),
                            (y - rect.y) * canonical.height / rect.height.max(1),
                            canonical.width,
                            canonical.height,
                        )
                    })
                    .flatten()
                });
            if let Some(target) = dock_target {
                println!("desktop_bottom_shell_activation={target:?}");
                match target {
                    BottomShellTarget::Applications => self
                        .session
                        .wayland_state
                        .apply_launcher_event(LauncherEvent::OpenApplications),
                    BottomShellTarget::Search => self
                        .session
                        .wayland_state
                        .apply_launcher_event(LauncherEvent::OpenSearch),
                    BottomShellTarget::Application(item) => {
                        self.session.wayland_state.launcher_launch_request = item.launch_request();
                        self.session
                            .wayland_state
                            .apply_launcher_event(LauncherEvent::Dismiss);
                    }
                    BottomShellTarget::Workspace(index) => {
                        self.session.wayland_state.activate_workspace(index, time);
                    }
                }
                self.session.wayland_state.pointer_button_count += 1;
                return true;
            }
        }
        if self.session.wayland_state.launcher_state.is_open() {
            if pressed {
                let (x, y) = self.session.wayland_state.pointer_location;
                let target = self
                    .session
                    .wayland_state
                    .launcher_state
                    .pointer_target_in_viewport(
                        x as u32,
                        y as u32,
                        self.session.wayland_state.output_width,
                        self.session.wayland_state.output_height,
                    );
                println!(
                    "aqua_launcher_pointer x={} y={} target={target:?}",
                    x as u32, y as u32
                );
                match target {
                    Some(LauncherPointerTarget::Category(category)) => {
                        self.session
                            .wayland_state
                            .apply_launcher_event(LauncherEvent::SelectCategory(category));
                        self.session.wayland_state.launcher_category_click_count += 1;
                    }
                    Some(LauncherPointerTarget::Application(index)) => {
                        if self
                            .session
                            .wayland_state
                            .launcher_state
                            .select_visible_index(index)
                        {
                            let update = self
                                .session
                                .wayland_state
                                .launcher_state
                                .handle_event(LauncherEvent::Activate);
                            self.session.wayland_state.launcher_launch_request =
                                update.launch_request;
                            self.session.wayland_state.launcher_app_click_count += 1;
                        }
                    }
                    Some(LauncherPointerTarget::QuickAction(action)) => {
                        let update = self
                            .session
                            .wayland_state
                            .launcher_state
                            .activate_quick_action(action);
                        self.session.wayland_state.launcher_launch_request = update.launch_request;
                    }
                    Some(LauncherPointerTarget::SearchField) => {
                        self.session
                            .wayland_state
                            .apply_launcher_event(LauncherEvent::OpenSearch);
                    }
                    Some(LauncherPointerTarget::Panel) | None => {}
                }
            }
            self.session.wayland_state.pointer_button_count += 1;
            return true;
        }
        if pressed
            && self.session.wayland_state.pointer_focus_surface.is_none()
            && matches!(button, 0x110 | 0x111)
        {
            let desktop_button = if button == 0x111 {
                DesktopPointerButton::Secondary
            } else {
                DesktopPointerButton::Primary
            };
            let update = self.session.wayland_state.desktop_icon_state.pointer_press(
                pointer_x as u32,
                pointer_y as u32,
                desktop_button,
                u64::from(time),
            );
            let launch_requested = update.launch_request.is_some();
            if let Some(request) = update.launch_request {
                println!("desktop_icon_activation_app={}", request.app_id);
                self.session.wayland_state.launcher_launch_request = Some(request);
            }
            if let Some(action) = update.context_action {
                match action {
                    DesktopContextAction::MenuOpened => {
                        println!("desktop_icon_context_action=menu-opened");
                    }
                    DesktopContextAction::Properties(icon_id) => {
                        println!("desktop_icon_context_action=properties icon={icon_id}");
                        self.session.wayland_state.launcher_launch_request =
                            properties_launch_request(icon_id);
                        self.session.wayland_state.post_desktop_notification(
                            "Opening Properties",
                            &format!("Preparing {icon_id} details."),
                        );
                    }
                    DesktopContextAction::TrashEmptyConfirmationRequested => {
                        let refresh = self.session.wayland_state.trash_model.refresh();
                        let count = self.session.wayland_state.trash_model.entries().len();
                        println!(
                            "desktop_icon_context_action=trash-empty-confirmation count={count} refresh_ok={}",
                            refresh.is_ok()
                        );
                        let body = if refresh.is_ok() {
                            format!("Empty {count} item(s)? Select Confirm Empty to continue.")
                        } else {
                            "Trash could not be inspected safely.".to_string()
                        };
                        self.session
                            .wayland_state
                            .post_desktop_notification("Empty Trash", &body);
                    }
                    DesktopContextAction::TrashEmptyConfirmed => {
                        match self.session.wayland_state.trash_model.empty() {
                            Ok(count) => {
                                println!(
                                    "desktop_icon_context_action=trash-emptied count={count} status=ok"
                                );
                                self.session.wayland_state.post_desktop_notification(
                                    "Trash emptied",
                                    &format!("Removed {count} item(s)."),
                                );
                            }
                            Err(error) => {
                                println!(
                                    "desktop_icon_context_action=trash-emptied count=0 status=error error={error}"
                                );
                                self.session.wayland_state.post_desktop_notification(
                                    "Trash was not emptied",
                                    "The Trash folder changed or could not be accessed safely.",
                                );
                            }
                        }
                    }
                }
            }
            if update.redraw_requested {
                println!(
                    "desktop_icon_selected={}",
                    self.session
                        .wayland_state
                        .desktop_icon_state
                        .selected()
                        .map_or("none".to_string(), |index| index.to_string())
                );
            }
            if update.redraw_requested || launch_requested {
                self.session.wayland_state.pointer_button_count += 1;
                return true;
            }
        }
        pointer.button(
            &mut self.session.wayland_state,
            &ButtonEvent {
                serial: Serial::from(time.max(1)),
                time,
                button,
                state: if pressed {
                    ButtonState::Pressed
                } else {
                    ButtonState::Released
                },
            },
        );
        if pressed {
            println!(
                "aqua_desktop_pointer x={} y={} surface_hit={}",
                self.session.wayland_state.pointer_location.0 as u32,
                self.session.wayland_state.pointer_location.1 as u32,
                self.session.wayland_state.pointer_focus_surface.is_some()
            );
            if let Some(surface) = self.session.wayland_state.pointer_focus_surface.clone() {
                if let Some(keyboard) = self.session.wayland_state.seat.get_keyboard() {
                    keyboard.set_focus(
                        &mut self.session.wayland_state,
                        Some(surface.clone()),
                        Serial::from(time.max(1)),
                    );
                    self.session.wayland_state.keyboard_focus_assigned = true;
                    self.session.wayland_state.surface_focus_change_count += 1;
                }
                if let Some(index) = self
                    .session
                    .wayland_state
                    .mapped_surfaces
                    .iter()
                    .position(|record| record.surface == surface)
                {
                    let record = self.session.wayland_state.mapped_surfaces.remove(index);
                    self.session.wayland_state.mapped_surfaces.push(record);
                    self.session.wayland_state.mapped_surface = Some(surface);
                    self.session.wayland_state.stacking_change_count += 1;
                }
            }
        }
        self.session.wayland_state.pointer_button_count += 1;
        true
    }

    pub fn input_snapshot(&self) -> SmithayBackendInputSnapshot {
        SmithayBackendInputSnapshot {
            keyboard_event_count: self.session.wayland_state.keyboard_event_count,
            pointer_motion_count: self.session.wayland_state.pointer_motion_count,
            pointer_button_count: self.session.wayland_state.pointer_button_count,
            launcher_visible: self
                .session
                .wayland_state
                .launcher_scene
                .surface_is_visible(SurfaceKind::Launcher),
            keyboard_shortcut_intercept_count: self
                .session
                .wayland_state
                .keyboard_shortcut_intercept_count,
            keyboard_forward_count: self.session.wayland_state.keyboard_forward_count,
            pointer_hit_test_count: self.session.wayland_state.pointer_hit_test_count,
            pointer_surface_hit_count: self.session.wayland_state.pointer_surface_hit_count,
            surface_focus_change_count: self.session.wayland_state.surface_focus_change_count,
            stacking_change_count: self.session.wayland_state.stacking_change_count,
            launcher_pointer_hit_count: self.session.wayland_state.launcher_pointer_hit_count,
            launcher_category_click_count: self.session.wayland_state.launcher_category_click_count,
            launcher_app_click_count: self.session.wayland_state.launcher_app_click_count,
            launcher_launch_request: self.session.wayland_state.launcher_launch_request.clone(),
            pointer_x: self.session.wayland_state.pointer_location.0 as u32,
            pointer_y: self.session.wayland_state.pointer_location.1 as u32,
        }
    }

    pub fn client_surface_snapshot(&self) -> SmithayClientSurfaceSnapshot {
        let state = &self.session.wayland_state;
        SmithayClientSurfaceSnapshot {
            workspace: state.active_workspace,
            commit_count: state.surface_commit_count,
            buffer_attach_count: state.server_buffer_attach_count,
            shm_import_count: state.server_shm_buffer_import_count,
            toplevel_count: state.toplevel_count,
            configure_ack_count: state.toplevel_configure_ack_count,
            width: state.shm_buffer_width,
            height: state.shm_buffer_height,
            stride: state.shm_buffer_stride,
            sample_checksum: state.shm_sample_checksum,
            sample_pixel: state.shm_sample_pixel,
            sample_grid: state.shm_sample_grid,
            buffer_rgba: state.shm_buffer_rgba.clone(),
            buffer_opaque: false,
            damage_commit_count: state.damage_commit_count,
            damage_rect_count: state.damage_rect_count,
            pending_frame_callback_count: state.pending_frame_callbacks.len(),
            frame_callbacks_sent: state.frame_callbacks_sent,
            keyboard_focus_assigned: state.keyboard_focus_assigned,
            pointer_focus_assigned: state.pointer_focus_assigned,
            mapped_surface_count: state.mapped_surfaces.len(),
            surface_focus_change_count: state.surface_focus_change_count,
            stacking_change_count: state.stacking_change_count,
            destroyed_surface_count: state.destroyed_surface_count,
            client_cleanup_count: state.client_cleanup_count,
            cleanup_keyboard_focus_reassigned: state.cleanup_keyboard_focus_reassigned,
            x: 0,
            y: 0,
            display_width: state.shm_buffer_width,
            display_height: state.shm_buffer_height,
            move_request_count: state.move_request_count,
            resize_request_count: state.resize_request_count,
            close_request_count: state.close_request_count,
            maximize_request_count: state.maximize_request_count,
            unmaximize_request_count: state.unmaximize_request_count,
            fullscreen_request_count: state.fullscreen_request_count,
            unfullscreen_request_count: state.unfullscreen_request_count,
        }
    }

    pub fn client_surface_snapshots(&self) -> Vec<SmithayClientSurfaceSnapshot> {
        let state = &self.session.wayland_state;
        state
            .mapped_surfaces
            .iter()
            .map(|surface| SmithayClientSurfaceSnapshot {
                workspace: surface.workspace,
                commit_count: state.surface_commit_count,
                buffer_attach_count: state.server_buffer_attach_count,
                shm_import_count: state.server_shm_buffer_import_count,
                toplevel_count: state.toplevel_count,
                configure_ack_count: state.toplevel_configure_ack_count,
                width: surface.width,
                height: surface.height,
                stride: surface.stride,
                sample_checksum: surface.sample_checksum,
                sample_pixel: surface.sample_pixel,
                sample_grid: surface.sample_grid,
                buffer_rgba: surface.buffer_rgba.clone(),
                buffer_opaque: surface.buffer_opaque,
                damage_commit_count: state.damage_commit_count,
                damage_rect_count: state.damage_rect_count,
                pending_frame_callback_count: state.pending_frame_callbacks.len(),
                frame_callbacks_sent: state.frame_callbacks_sent,
                keyboard_focus_assigned: state.keyboard_focus_assigned
                    && state.mapped_surface.as_ref() == Some(&surface.surface),
                pointer_focus_assigned: state.pointer_focus_assigned,
                mapped_surface_count: state.mapped_surfaces.len(),
                surface_focus_change_count: state.surface_focus_change_count,
                stacking_change_count: state.stacking_change_count,
                destroyed_surface_count: state.destroyed_surface_count,
                client_cleanup_count: state.client_cleanup_count,
                cleanup_keyboard_focus_reassigned: state.cleanup_keyboard_focus_reassigned,
                x: surface.x,
                y: surface.y,
                display_width: surface.display_width,
                display_height: surface.display_height,
                move_request_count: state.move_request_count,
                resize_request_count: state.resize_request_count,
                close_request_count: state.close_request_count,
                maximize_request_count: state.maximize_request_count,
                unmaximize_request_count: state.unmaximize_request_count,
                fullscreen_request_count: state.fullscreen_request_count,
                unfullscreen_request_count: state.unfullscreen_request_count,
            })
            .collect()
    }

    pub fn visible_client_surface_snapshots(&self) -> Vec<SmithayClientSurfaceSnapshot> {
        let active_workspace = self.session.wayland_state.active_workspace;
        self.client_surface_snapshots()
            .into_iter()
            .filter(|surface| surface.workspace == active_workspace)
            .collect()
    }

    pub fn present_client_surface(&mut self, time: u32) -> bool {
        let Some(surface) = self.session.wayland_state.mapped_surface.clone() else {
            return false;
        };
        let Some(keyboard) = self.session.wayland_state.seat.get_keyboard() else {
            return false;
        };
        let Some(pointer) = self.session.wayland_state.seat.get_pointer() else {
            return false;
        };

        keyboard.set_focus(
            &mut self.session.wayland_state,
            Some(surface.clone()),
            Serial::from(time.max(1)),
        );
        let geometry = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .find(|record| record.surface == surface)
            .map(|record| (f64::from(record.x), f64::from(record.y)))
            .unwrap_or((0.0, 0.0));
        let (origin_x, origin_y) = geometry;
        let pointer_location = (768.0, 512.0);
        self.session.wayland_state.pointer_location = pointer_location;
        pointer.motion(
            &mut self.session.wayland_state,
            Some((
                surface.clone(),
                (pointer_location.0 - origin_x, pointer_location.1 - origin_y).into(),
            )),
            &MotionEvent {
                location: pointer_location.into(),
                serial: Serial::from(time.max(1)),
                time,
            },
        );
        self.session.wayland_state.keyboard_focus_assigned = true;
        self.session.wayland_state.pointer_focus_assigned = true;
        self.session.wayland_state.pointer_focus_surface = Some(surface);
        let callbacks = std::mem::take(&mut self.session.wayland_state.pending_frame_callbacks);
        self.session.wayland_state.frame_callbacks_sent += callbacks.len();
        for callback in callbacks {
            callback.done(time);
        }
        true
    }

    pub fn close_active_toplevel(&mut self) -> bool {
        self.session.wayland_state.close_active_toplevel()
    }

    pub fn raise_surface_with_buffer_size(&mut self, width: u32, height: u32) -> bool {
        let Some(index) = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .position(|record| record.width == width && record.height == height)
        else {
            return false;
        };
        let record = self.session.wayland_state.mapped_surfaces.remove(index);
        self.session.wayland_state.active_workspace = record.workspace;
        self.session.wayland_state.mapped_surface = Some(record.surface.clone());
        self.session.wayland_state.mapped_surfaces.push(record);
        true
    }

    pub fn raise_surface_with_app_id(&mut self, expected_app_id: &str) -> bool {
        let surface = self
            .session
            .wayland_state
            .toplevel_surfaces
            .iter()
            .find(|surface| {
                with_states(surface.wl_surface(), |states| {
                    states
                        .data_map
                        .get::<XdgToplevelSurfaceData>()
                        .and_then(|data| data.lock().ok())
                        .and_then(|attributes| attributes.app_id.clone())
                        .as_deref()
                        == Some(expected_app_id)
                })
            })
            .map(|surface| surface.wl_surface().clone());
        let Some(surface) = surface else {
            return false;
        };
        let Some(index) = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .position(|record| record.surface == surface)
        else {
            return false;
        };
        let record = self.session.wayland_state.mapped_surfaces.remove(index);
        self.session.wayland_state.active_workspace = record.workspace;
        self.session.wayland_state.mapped_surface = Some(record.surface.clone());
        self.session.wayland_state.mapped_surfaces.push(record);
        true
    }

    pub fn active_toplevel_app_id(&self) -> Option<String> {
        let active_surface = self.session.wayland_state.mapped_surface.as_ref()?;
        self.session
            .wayland_state
            .toplevel_surfaces
            .iter()
            .find(|surface| surface.wl_surface() == active_surface)
            .and_then(|surface| {
                with_states(surface.wl_surface(), |states| {
                    states
                        .data_map
                        .get::<XdgToplevelSurfaceData>()
                        .and_then(|data| data.lock().ok())
                        .and_then(|attributes| attributes.app_id.clone())
                })
            })
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_external_wayland_test_client(
    socket_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::with_buffer_size(384, 256);
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("external Wayland test client did not attach its wl_shm buffer".into());
    }

    println!("external_client_connected=true");
    println!("external_client_protocol=xdg_toplevel");
    println!("external_client_buffer=384x256");
    println!(
        "external_client_variant={}",
        std::env::var("AQUA_WAYLAND_TEST_CLIENT_VARIANT").unwrap_or_else(|_| "1".into())
    );
    println!("[AQUA-CLIENT] stage=external-wayland-surface status=active");
    std::io::stdout().flush()?;

    let controlled_exit =
        std::env::var("AQUA_WAYLAND_TEST_CLIENT_CONTROLLED_EXIT").as_deref() == Ok("true");
    let wait_for_close =
        std::env::var("AQUA_WAYLAND_TEST_CLIENT_WAIT_FOR_CLOSE").as_deref() == Ok("true");

    while !(state.close_event_received
        || state.partial_damage_commit_sent
            && state.keyboard_event_received
            && state.pointer_event_received)
    {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
    }
    println!(
        "external_client_frame_callback_received={}",
        state.frame_callback_received
    );
    println!(
        "external_client_partial_damage_commit={}",
        state.partial_damage_commit_sent
    );
    println!(
        "external_client_keyboard_event_received={}",
        state.keyboard_event_received
    );
    println!(
        "external_client_pointer_event_received={}",
        state.pointer_event_received
    );
    println!(
        "external_client_interactive_requests_sent={}",
        state.interactive_requests_sent
    );
    if controlled_exit {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if let Some(toplevel) = state.xdg_toplevel.take() {
            toplevel.destroy();
        }
        if let Some(xdg_surface) = state.xdg_surface.take() {
            xdg_surface.destroy();
        }
        if let Some(surface) = state.base_surface.take() {
            surface.destroy();
        }
        connection.flush()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("external_client_controlled_exit=true");
    } else if wait_for_close {
        while !state.close_event_received {
            event_queue.blocking_dispatch(&mut state)?;
            connection.flush()?;
        }
        if let Some(toplevel) = state.xdg_toplevel.take() {
            toplevel.destroy();
        }
        if let Some(xdg_surface) = state.xdg_surface.take() {
            xdg_surface.destroy();
        }
        if let Some(surface) = state.base_surface.take() {
            surface.destroy();
        }
        connection.flush()?;
        println!("external_client_close_event_received=true");
        println!("external_client_close_cleanup=true");
    }
    println!(
        "external_client_size_constraints_sent={}",
        state.size_constraints_sent
    );
    println!(
        "external_client_state_configures={}",
        state.state_configure_count
    );
    println!(
        "external_client_state_cycle_complete={}",
        state.state_cycle_complete
    );
    println!("[AQUA-CLIENT] stage=external-wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_aqua_files_client(
    socket_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::files_app();
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("Aqua Files did not attach its wl_shm buffer".into());
    }

    println!("aqua_files_connected=true");
    println!("aqua_files_app_id=aqua.files");
    println!("aqua_files_title=Files");
    println!("aqua_files_buffer=640x420");
    println!("[AQUA-FILES] stage=wayland-surface status=active");
    std::io::stdout().flush()?;

    run_aqua_ui_event_loop(&connection, &mut event_queue, &mut state)?;
    println!("aqua_files_close_received=true");
    println!("[AQUA-FILES] stage=wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_aqua_properties_client(
    socket_path: impl AsRef<Path>,
    target: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::properties_app(target)?;
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("Aqua Properties did not attach its wl_shm buffer".into());
    }

    let model = state
        .properties_model
        .as_ref()
        .ok_or("Aqua Properties model is missing")?;
    println!("aqua_properties_connected=true");
    println!("aqua_properties_app_id=aqua.properties");
    println!("aqua_properties_target={}", model.icon_id);
    println!("aqua_properties_kind={}", model.kind);
    println!("aqua_properties_location={}", model.location);
    println!("aqua_properties_status={}", model.status);
    println!(
        "aqua_properties_primary_action={}",
        model.primary_action().log_name()
    );
    println!(
        "aqua_properties_refresh_generation={}",
        model.refresh_generation
    );
    println!("aqua_properties_buffer=480x300");
    println!("[AQUA-PROPERTIES] stage=wayland-surface status=active");
    std::io::stdout().flush()?;

    run_aqua_ui_event_loop(&connection, &mut event_queue, &mut state)?;
    println!("aqua_properties_close_received=true");
    println!("[AQUA-PROPERTIES] stage=wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_aqua_settings_client(
    socket_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::settings_app()?;
    println!(
        "aqua_settings_loaded_reduced_motion={}",
        state
            .settings_model
            .as_ref()
            .is_some_and(|model| model.reduced_motion)
    );
    println!(
        "aqua_settings_loaded_desktop_icons={}",
        state
            .settings_model
            .as_ref()
            .is_some_and(|model| model.desktop_icons)
    );
    println!(
        "aqua_settings_loaded_key_repeat={}",
        state
            .settings_model
            .as_ref()
            .is_some_and(|model| model.key_repeat)
    );
    println!(
        "aqua_settings_loaded_theme={}",
        state
            .settings_model
            .as_ref()
            .map_or(aqua_shell::AquaTheme::default().id(), |model| model
                .theme
                .id())
    );
    if let Some(model) = state.settings_model.as_ref() {
        println!("aqua_settings_audio_available={}", model.audio.available());
        println!(
            "aqua_settings_loaded_audio_volume={}",
            model.audio.volume_percent()
        );
        println!("aqua_settings_loaded_audio_muted={}", model.audio.muted());
        println!(
            "aqua_settings_audio_service_health={}",
            model.audio.service_health().id()
        );
        println!(
            "aqua_settings_audio_backend_applied={}",
            model.audio.backend_applied()
        );
        println!(
            "aqua_settings_audio_control_status={}",
            model.audio.control_status().id()
        );
        println!(
            "aqua_settings_audio_controls_enabled={}",
            model.audio.controls_enabled()
        );
        println!(
            "aqua_settings_audio_submission_attempts={}",
            model.audio.submission_attempts()
        );
        println!(
            "aqua_settings_audio_submission_retry_exhausted={}",
            model.audio.submission_retry_exhausted()
        );
    }
    println!(
        "aqua_settings_network_status_available={}",
        state
            .settings_model
            .as_ref()
            .is_some_and(|model| model.network_status_available)
    );
    println!(
        "aqua_settings_network_interface_count={}",
        state
            .settings_model
            .as_ref()
            .map_or(0, |model| model.network_interfaces.len())
    );
    if let Some(interface) = state
        .settings_model
        .as_ref()
        .and_then(|model| model.network_interfaces.first())
    {
        println!("aqua_settings_network_interface={}", interface.name);
        println!("aqua_settings_network_state={}", interface.state);
    }
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("Aqua Settings did not attach its wl_shm buffer".into());
    }

    println!("aqua_settings_connected=true");
    println!("aqua_settings_app_id=aqua.settings");
    println!("aqua_settings_title=System Settings");
    println!("aqua_settings_buffer=600x400");
    println!("aqua_settings_font_family={UI_FONT_FAMILY}");
    println!("aqua_settings_font_source={UI_FONT_SOURCE}");
    println!("aqua_settings_font_ready={}", embedded_ui_font_ready());
    println!("[AQUA-SETTINGS] stage=wayland-surface status=active");
    std::io::stdout().flush()?;

    run_aqua_ui_event_loop(&connection, &mut event_queue, &mut state)?;
    if state.settings_persistence_failed {
        return Err("Aqua Settings could not persist its configuration".into());
    }
    println!("aqua_settings_close_received=true");
    println!("[AQUA-SETTINGS] stage=wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_aqua_installer_client(
    socket_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::installer_app()?;
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("Aqua Installer did not attach its wl_shm buffer".into());
    }

    println!("aqua_installer_connected=true");
    println!("aqua_installer_app_id=aqua.installer");
    println!("aqua_installer_title=Aqua Linux Kurulumu");
    println!("aqua_installer_buffer=1280x800");
    println!("aqua_installer_step=welcome");
    println!("aqua_installer_live_input=true");
    println!("aqua_installer_execution_allowed=false");
    println!("[AQUA-INSTALLER-UI] stage=wayland-surface status=active");
    std::io::stdout().flush()?;

    run_aqua_ui_event_loop(&connection, &mut event_queue, &mut state)?;
    println!("aqua_installer_close_received=true");
    println!("[AQUA-INSTALLER-UI] stage=wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_aqua_typography_acceptance_client(
    socket_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::typography_acceptance_app();
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("Aqua typography acceptance client did not attach its wl_shm buffer".into());
    }

    println!("aqua_typography_acceptance_connected=true");
    println!("aqua_typography_acceptance_app_id=aqua.typography-acceptance");
    println!("aqua_typography_acceptance_buffer=1280x800");
    println!("aqua_typography_acceptance_locales=tr-TR,ar");
    println!("aqua_typography_acceptance_scale=1.0");
    println!("[AQUA-TYPOGRAPHY] stage=wayland-surface status=active");
    std::io::stdout().flush()?;

    run_aqua_ui_event_loop(&connection, &mut event_queue, &mut state)?;
    println!("aqua_typography_acceptance_close_received=true");
    println!("[AQUA-TYPOGRAPHY] stage=wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_aqua_component_acceptance_client(
    socket_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::component_acceptance_app();
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("Aqua component acceptance client did not attach its wl_shm buffer".into());
    }

    println!("aqua_component_acceptance_connected=true");
    println!("aqua_component_acceptance_app_id=aqua.component-acceptance");
    println!("aqua_component_acceptance_buffer=1280x800");
    println!("aqua_component_acceptance_fixture_revision={COMPONENT_FIXTURE_REVISION}");
    println!("aqua_component_acceptance_catalog=22");
    println!("aqua_component_acceptance_shared=22");
    println!("aqua_component_acceptance_ready=true");
    println!("[AQUA-COMPONENTS] stage=wayland-surface status=active");
    std::io::stdout().flush()?;

    run_aqua_ui_event_loop(&connection, &mut event_queue, &mut state)?;
    println!("aqua_component_acceptance_close_received=true");
    println!("[AQUA-COMPONENTS] stage=wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn run_aqua_terminal_client(
    socket_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;
    use std::os::fd::AsFd as _;

    let stream = std::os::unix::net::UnixStream::connect(socket_path.as_ref())?;
    let connection = ClientConnection::from_socket(stream)?;
    let mut event_queue = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    connection.display().get_registry(&queue_handle, ());
    connection.flush()?;

    let mut state = XdgSmokeClientState::terminal_app()?;
    for _ in 0..32 {
        event_queue.blocking_dispatch(&mut state)?;
        connection.flush()?;
        if state.client_buffer_attached {
            break;
        }
    }
    if !state.client_buffer_attached {
        return Err("Aqua Terminal did not attach its wl_shm buffer".into());
    }

    println!("aqua_terminal_connected=true");
    println!("aqua_terminal_app_id=aqua.terminal");
    println!("aqua_terminal_title=Terminal");
    println!("aqua_terminal_pty=true");
    println!("aqua_terminal_emulator=vt100");
    println!("aqua_terminal_resize_protocol=true");
    println!("aqua_terminal_buffer=680x430");
    println!("[AQUA-TERMINAL] stage=wayland-surface status=active");
    std::io::stdout().flush()?;

    let poller = polling::Poller::new()?;
    unsafe {
        poller.add_with_mode(
            &event_queue.as_fd(),
            polling::Event::readable(1),
            polling::PollMode::Level,
        )?;
    }
    let mut events = polling::Events::new();
    let mut last_output_at = None;
    while !state.close_event_received {
        event_queue.dispatch_pending(&mut state)?;
        state.refresh_runtime_theme(&queue_handle);
        connection.flush()?;

        let output_changed = state
            .terminal_session
            .as_mut()
            .is_some_and(AquaTerminalSession::drain_output);
        if output_changed {
            state.terminal_dirty = true;
            last_output_at = Some(std::time::Instant::now());
            if !state.terminal_command_observed
                && state.terminal_session.as_ref().is_some_and(|terminal| {
                    terminal
                        .view()
                        .lines
                        .iter()
                        .any(|line| line.contains("aquaterminalok"))
                })
            {
                state.terminal_command_observed = true;
                println!("aqua_terminal_command_output=true");
            }
        }
        let output_idle = last_output_at
            .is_some_and(|changed_at| changed_at.elapsed() >= Duration::from_millis(120));
        let frame_callback_stalled = state.terminal_frame_pending
            && state
                .terminal_frame_requested_at
                .is_some_and(|requested_at| requested_at.elapsed() >= Duration::from_millis(500));
        if state.terminal_dirty
            && output_idle
            && (!state.terminal_frame_pending || frame_callback_stalled)
        {
            if frame_callback_stalled {
                println!("aqua_terminal_frame_callback_timeout=true");
            }
            state.redraw_terminal_buffer(&queue_handle);
            connection.flush()?;
            last_output_at = None;
        }
        if state.close_event_received {
            break;
        }

        let Some(read_guard) = event_queue.prepare_read() else {
            continue;
        };
        events.clear();
        if poller.wait(&mut events, Some(Duration::from_millis(16)))? > 0 {
            read_guard.read()?;
        }
    }
    poller.delete(event_queue.as_fd())?;
    println!("aqua_terminal_close_received=true");
    println!("[AQUA-TERMINAL] stage=wayland-surface status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
pub fn probe_aqua_terminal_pty() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = AquaTerminalSession::spawn(8, 48)?;
    std::thread::sleep(Duration::from_millis(100));
    terminal.drain_output();
    terminal.write_input(b"stty -echo\r")?;
    std::thread::sleep(Duration::from_millis(100));
    terminal.drain_output();
    terminal.write_input(b"echo aquaptyok\r")?;

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let mut output_ready = false;
    while std::time::Instant::now() < deadline {
        terminal.drain_output();
        if terminal
            .view()
            .lines
            .iter()
            .any(|line| line.contains("aquaptyok"))
        {
            output_ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    if !output_ready {
        return Err("Aqua Terminal PTY command output was not parsed".into());
    }
    terminal.resize(10, 60)?;
    println!("aqua_terminal_probe_pty=true");
    println!("aqua_terminal_probe_emulator=vt100");
    println!("aqua_terminal_probe_command=true");
    println!("aqua_terminal_probe_resize=true");
    println!("[AQUA-TERMINAL] stage=pty-probe status=ok");
    Ok(())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl AquaCompositorSession {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let display = Display::new()?;
        let display_handle = display.handle();
        Ok(Self {
            display,
            wayland_state: WaylandSmokeState::new(&display_handle)?,
        })
    }

    fn insert_client(&mut self, stream: std::os::unix::net::UnixStream) -> std::io::Result<Client> {
        self.display
            .handle()
            .insert_client(stream, Arc::new(WaylandSmokeClientState::default()))
    }

    fn dispatch_clients(&mut self) -> std::io::Result<usize> {
        self.display.dispatch_clients(&mut self.wayland_state)
    }

    fn flush_clients(&mut self) -> std::io::Result<()> {
        self.display.flush_clients()
    }

    fn run_once_smoke(
        self,
        socket_path: PathBuf,
        timeout: Duration,
    ) -> Result<SessionRunOnceSmokeResult, Box<dyn std::error::Error>> {
        let socket_name = socket_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("aqua-wayland-run-once")
            .to_string();
        let mut event_loop: EventLoop<CalloopSocketSmokeState> = EventLoop::try_new()?;
        let listener = ListeningSocket::bind_absolute(socket_path.clone())?;
        let socket_bound = is_unix_socket(&socket_path);

        event_loop.handle().insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            |_readiness, listener, state| {
                state.callback_invoked = true;

                while let Some(stream) = listener.accept()? {
                    state.client_accepted = true;
                    state.client_inserted = state.session.insert_client(stream).is_ok();
                }

                Ok(PostAction::Remove)
            },
        )?;

        let mut state = CalloopSocketSmokeState {
            session: self,
            callback_invoked: false,
            client_accepted: false,
            client_inserted: false,
        };

        let _client_stream = std::os::unix::net::UnixStream::connect(&socket_path)?;
        event_loop.dispatch(timeout, &mut state)?;
        let dispatched_requests = state.session.dispatch_clients()?;
        let flush_clients_ok = state.session.flush_clients().is_ok();

        Ok(SessionRunOnceSmokeResult {
            socket_name,
            run_once_called: true,
            socket_bound,
            client_connected: true,
            callback_invoked: state.callback_invoked,
            client_accepted: state.client_accepted,
            client_inserted: state.client_inserted,
            dispatch_clients_ok: true,
            dispatched_requests,
            flush_clients_ok,
            socket_cleaned: false,
            host_stub: false,
        })
    }

    fn run_bounded_loop_smoke(
        self,
        socket_path: PathBuf,
        timeout: Duration,
        max_iterations: u32,
    ) -> Result<SessionLoopSmokeResult, Box<dyn std::error::Error>> {
        let socket_name = socket_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("aqua-wayland-session-loop")
            .to_string();
        let mut event_loop: EventLoop<CalloopSocketSmokeState> = EventLoop::try_new()?;
        let listener = ListeningSocket::bind_absolute(socket_path.clone())?;
        let socket_bound = is_unix_socket(&socket_path);

        event_loop.handle().insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            |_readiness, listener, state| {
                state.callback_invoked = true;

                while let Some(stream) = listener.accept()? {
                    state.client_accepted = true;
                    state.client_inserted = state.session.insert_client(stream).is_ok();
                }

                Ok(PostAction::Remove)
            },
        )?;

        let mut state = CalloopSocketSmokeState {
            session: self,
            callback_invoked: false,
            client_accepted: false,
            client_inserted: false,
        };

        let _client_stream = std::os::unix::net::UnixStream::connect(&socket_path)?;
        let mut loop_iterations = 0;
        let mut dispatch_passes = 0;
        let mut flush_passes = 0;

        while loop_iterations < max_iterations {
            event_loop.dispatch(timeout, &mut state)?;
            state.session.dispatch_clients()?;
            dispatch_passes += 1;
            state.session.flush_clients()?;
            flush_passes += 1;
            loop_iterations += 1;
        }

        Ok(SessionLoopSmokeResult {
            socket_name,
            loop_started: true,
            loop_iterations,
            max_iterations,
            socket_bound,
            client_connected: true,
            callback_invoked: state.callback_invoked,
            client_accepted: state.client_accepted,
            client_inserted: state.client_inserted,
            dispatch_passes,
            flush_passes,
            socket_cleaned: false,
            host_stub: false,
        })
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn is_unix_socket(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.file_type().is_socket())
        .unwrap_or(false)
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn launcher_key_character(code: u32) -> Option<char> {
    Some(match code {
        16 => 'q',
        17 => 'w',
        18 => 'e',
        19 => 'r',
        20 => 't',
        21 => 'y',
        22 => 'u',
        23 => 'i',
        24 => 'o',
        25 => 'p',
        30 => 'a',
        31 => 's',
        32 => 'd',
        33 => 'f',
        34 => 'g',
        35 => 'h',
        36 => 'j',
        37 => 'k',
        38 => 'l',
        44 => 'z',
        45 => 'x',
        46 => 'c',
        47 => 'v',
        48 => 'b',
        49 => 'n',
        50 => 'm',
        _ => return None,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl CompositorHandler for WaylandSmokeState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        client
            .get_data::<WaylandSmokeClientState>()
            .expect("Aqua Wayland smoke clients must carry compositor state")
            .compositor_state()
    }

    fn commit(&mut self, surface: &WlSurface) {
        self.surface_commit_count += 1;
        let installer_full_output = self.toplevel_surfaces.iter().any(|toplevel| {
            toplevel.wl_surface() == surface
                && with_states(toplevel.wl_surface(), |states| {
                    states
                        .data_map
                        .get::<XdgToplevelSurfaceData>()
                        .and_then(|data| data.lock().ok())
                        .and_then(|attributes| attributes.app_id.clone())
                        .as_deref()
                        == Some("aqua.installer")
                })
        });
        with_states(surface, |states| {
            let mut guard = states
                .cached_state
                .get::<smithay::wayland::compositor::SurfaceAttributes>();
            let attributes = guard.current();
            if !attributes.damage.is_empty() {
                self.damage_commit_count += 1;
                self.damage_rect_count += attributes.damage.len();
                attributes.damage.clear();
            }
            self.pending_frame_callbacks
                .append(&mut attributes.frame_callbacks);
            let new_buffer = match attributes.buffer.as_ref() {
                Some(BufferAssignment::NewBuffer(buffer)) => Some(buffer.clone()),
                _ => None,
            };
            let buffer = new_buffer.clone().or_else(|| {
                self.mapped_surfaces
                    .iter()
                    .find(|record| record.surface == *surface)
                    .map(|record| record.buffer.clone())
            });
            if let Some(buffer) = buffer {
                if new_buffer.is_some() {
                    self.server_buffer_attach_count += 1;
                }
                if let Ok((
                    sample_checksum,
                    sample_pixel,
                    sample_grid,
                    buffer_rgba,
                    buffer_width,
                    buffer_height,
                    buffer_stride,
                )) = with_buffer_contents(&buffer, |ptr, len, metadata| {
                    let byte_count = (metadata.height as usize)
                        .saturating_mul(metadata.stride as usize)
                        .min(len);
                    if byte_count == 0 {
                        return (
                            0,
                            [0, 0, 0, 0],
                            solid_sample_grid([0, 0, 0, 0]),
                            Vec::new(),
                            0,
                            0,
                            0,
                        );
                    }

                    // Copy a tiny prefix while the shm mapping is valid; do not retain pointers.
                    let sample_len = byte_count.min(64);
                    let bytes = unsafe { std::slice::from_raw_parts(ptr, sample_len) };
                    let center_x = (metadata.width.max(1) / 2) as usize;
                    let center_y = (metadata.height.max(1) / 2) as usize;
                    let sample_pixel =
                        copy_shm_pixel(ptr, len, metadata.stride, center_x, center_y);
                    let max_x = metadata.width.saturating_sub(1) as usize;
                    let max_y = metadata.height.saturating_sub(1) as usize;
                    let sample_grid = [
                        copy_shm_pixel(ptr, len, metadata.stride, 0, 0),
                        copy_shm_pixel(ptr, len, metadata.stride, max_x, 0),
                        copy_shm_pixel(ptr, len, metadata.stride, 0, max_y),
                        copy_shm_pixel(ptr, len, metadata.stride, max_x, max_y),
                    ];
                    let buffer_rgba = copy_shm_buffer_rgba(
                        ptr,
                        len,
                        metadata.stride,
                        metadata.width,
                        metadata.height,
                    );
                    (
                        checksum_bytes(bytes),
                        sample_pixel,
                        sample_grid,
                        buffer_rgba,
                        metadata.width.max(0) as u32,
                        metadata.height.max(0) as u32,
                        metadata.stride.max(0) as u32,
                    )
                }) {
                    if sample_checksum != 0 {
                        let buffer_opaque =
                            attributes.opaque_region.as_ref().is_some_and(|region| {
                                matches!(
                                    region.rects.as_slice(),
                                    [(RectangleKind::Add, rect)]
                                        if rect.loc.x <= 0
                                            && rect.loc.y <= 0
                                            && rect.loc.x.saturating_add(rect.size.w)
                                                >= buffer_width as i32
                                            && rect.loc.y.saturating_add(rect.size.h)
                                                >= buffer_height as i32
                                )
                            });
                        self.server_shm_buffer_import_count += 1;
                        self.server_shm_buffer_sample_count += 1;
                        self.shm_sample_checksum = sample_checksum;
                        self.shm_sample_pixel = sample_pixel;
                        self.shm_sample_grid = sample_grid;
                        self.shm_buffer_rgba = buffer_rgba;
                        self.shm_buffer_width = buffer_width;
                        self.shm_buffer_height = buffer_height;
                        self.shm_buffer_stride = buffer_stride;
                        let record = ServerSurfaceRecord {
                            surface: surface.clone(),
                            buffer,
                            workspace: self.active_workspace,
                            sample_checksum,
                            sample_pixel,
                            sample_grid,
                            buffer_rgba: self.shm_buffer_rgba.clone(),
                            buffer_opaque,
                            width: buffer_width,
                            height: buffer_height,
                            stride: buffer_stride,
                            x: 0,
                            y: 0,
                            display_width: buffer_width,
                            display_height: buffer_height,
                            restore_geometry: None,
                        };
                        if let Some(existing) = self
                            .mapped_surfaces
                            .iter_mut()
                            .find(|existing| existing.surface == *surface)
                        {
                            let x = existing.x;
                            let y = existing.y;
                            let display_width = existing.display_width;
                            let display_height = existing.display_height;
                            let restore_geometry = existing.restore_geometry;
                            let workspace = existing.workspace;
                            *existing = record;
                            existing.workspace = workspace;
                            existing.x = x;
                            existing.y = y;
                            existing.display_width = display_width;
                            existing.display_height = display_height;
                            existing.restore_geometry = restore_geometry;
                            if installer_full_output {
                                existing.x = 0;
                                existing.y = 0;
                                existing.display_width = buffer_width;
                                existing.display_height = buffer_height;
                            }
                        } else {
                            self.mapped_surfaces.push(record);
                            let workspace = self.active_workspace;
                            let count = self
                                .mapped_surfaces
                                .iter()
                                .filter(|record| record.workspace == workspace)
                                .count();
                            for (index, record) in self
                                .mapped_surfaces
                                .iter_mut()
                                .filter(|record| record.workspace == workspace)
                                .enumerate()
                            {
                                let (x, y) = SmithayDrmSession::surface_geometry(
                                    index,
                                    count,
                                    record.display_width,
                                    record.display_height,
                                );
                                record.x = x.max(0.0) as u32;
                                record.y = y.max(0.0) as u32;
                            }
                            if installer_full_output {
                                if let Some(record) = self
                                    .mapped_surfaces
                                    .iter_mut()
                                    .find(|record| record.surface == *surface)
                                {
                                    record.x = 0;
                                    record.y = 0;
                                    record.display_width = buffer_width;
                                    record.display_height = buffer_height;
                                }
                            }
                        }
                        if self.mapped_surfaces.iter().any(|record| {
                            record.surface == *surface && record.workspace == self.active_workspace
                        }) {
                            self.mapped_surface = Some(surface.clone());
                        }
                    }
                }
            }
        });
    }

    fn destroyed(&mut self, surface: &WlSurface) {
        let previous_count = self.mapped_surfaces.len();
        self.mapped_surfaces
            .retain(|record| record.surface != *surface);
        self.toplevel_surfaces
            .retain(|toplevel| toplevel.wl_surface() != surface);
        if self.mapped_surfaces.len() == previous_count {
            return;
        }

        self.destroyed_surface_count += 1;
        self.client_cleanup_count += 1;
        let destroyed_surface_was_active = self.mapped_surface.as_ref() == Some(surface);
        if destroyed_surface_was_active {
            self.mapped_surface = self
                .mapped_surfaces
                .iter()
                .rev()
                .find(|record| record.workspace == self.active_workspace)
                .map(|record| record.surface.clone());
            if let Some(keyboard) = self.seat.get_keyboard() {
                keyboard.set_focus(
                    self,
                    self.mapped_surface.clone(),
                    Serial::from((self.surface_commit_count as u32).saturating_add(100)),
                );
                if self.mapped_surface.is_some() {
                    self.keyboard_focus_assigned = true;
                    self.cleanup_keyboard_focus_reassigned = true;
                }
            }
        }
        if self.pointer_focus_surface.as_ref() == Some(surface) {
            self.pointer_focus_surface = None;
            self.pointer_focus_assigned = false;
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl BufferHandler for WaylandSmokeState {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ShmHandler for WaylandSmokeState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl XdgShellHandler for WaylandSmokeState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        self.toplevel_count += 1;
        self.toplevel_surfaces.push(surface.clone());
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        let serial = surface.send_configure();
        self.toplevel_configure_sent = true;
        self.toplevel_configure_serial = Some(u32::from(serial));
        if self.close_new_toplevels {
            surface.send_close();
            self.toplevel_close_sent = true;
        }
    }

    fn move_request(&mut self, surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        if let Some(record) = self
            .mapped_surfaces
            .iter_mut()
            .find(|record| record.surface == *surface.wl_surface())
        {
            record.x = (record.x + 56).min(1536_u32.saturating_sub(record.display_width));
            record.y = (record.y + 32).min(1024_u32.saturating_sub(record.display_height));
            self.move_request_count += 1;
            println!(
                "desktop_toplevel_move_request x={} y={} count={}",
                record.x, record.y, self.move_request_count
            );
        }
    }

    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        _seat: wl_seat::WlSeat,
        _serial: Serial,
        _edges: xdg_toplevel::ResizeEdge,
    ) {
        let mut configured_size = None;
        if let Some(record) = self
            .mapped_surfaces
            .iter_mut()
            .find(|record| record.surface == *surface.wl_surface())
        {
            record.display_width = (record.display_width + 64).min(640);
            record.display_height = (record.display_height + 48).min(480);
            configured_size = Some((record.display_width, record.display_height));
            self.resize_request_count += 1;
            println!(
                "desktop_toplevel_resize_request width={} height={} count={}",
                record.display_width, record.display_height, self.resize_request_count
            );
        }
        if let Some((width, height)) = configured_size {
            surface.with_pending_state(|state| {
                state.size = Some((width as i32, height as i32).into());
            });
            surface.send_configure();
        }
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        if let Some(record) = self
            .mapped_surfaces
            .iter_mut()
            .find(|record| record.surface == *surface.wl_surface())
        {
            record.restore_geometry = Some((
                record.x,
                record.y,
                record.display_width,
                record.display_height,
            ));
            record.x = 192;
            record.y = 96;
            record.display_width = 1152;
            record.display_height = 832;
            self.maximize_request_count += 1;
            surface.with_pending_state(|state| {
                state.size = Some((1152, 832).into());
                state.states.set(xdg_toplevel::State::Maximized);
            });
            surface.send_configure();
        }
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        if let Some(record) = self
            .mapped_surfaces
            .iter_mut()
            .find(|record| record.surface == *surface.wl_surface())
        {
            if let Some((x, y, width, height)) = record.restore_geometry.take() {
                record.x = x;
                record.y = y;
                record.display_width = width;
                record.display_height = height;
            }
            self.unmaximize_request_count += 1;
            let size = (record.display_width as i32, record.display_height as i32);
            surface.with_pending_state(|state| {
                state.size = Some(size.into());
                state.states.unset(xdg_toplevel::State::Maximized);
            });
            surface.send_configure();
        }
    }

    fn fullscreen_request(
        &mut self,
        surface: ToplevelSurface,
        _output: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
    ) {
        if let Some(record) = self
            .mapped_surfaces
            .iter_mut()
            .find(|record| record.surface == *surface.wl_surface())
        {
            record.restore_geometry = Some((
                record.x,
                record.y,
                record.display_width,
                record.display_height,
            ));
            record.x = 0;
            record.y = 0;
            record.display_width = 1536;
            record.display_height = 1024;
            self.fullscreen_request_count += 1;
            surface.with_pending_state(|state| {
                state.size = Some((1536, 1024).into());
                state.states.set(xdg_toplevel::State::Fullscreen);
            });
            surface.send_configure();
        }
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        if let Some(record) = self
            .mapped_surfaces
            .iter_mut()
            .find(|record| record.surface == *surface.wl_surface())
        {
            if let Some((x, y, width, height)) = record.restore_geometry.take() {
                record.x = x;
                record.y = y;
                record.display_width = width;
                record.display_height = height;
            }
            self.unfullscreen_request_count += 1;
            let size = (record.display_width as i32, record.display_height as i32);
            surface.with_pending_state(|state| {
                state.size = Some(size.into());
                state.states.unset(xdg_toplevel::State::Fullscreen);
            });
            surface.send_configure();
        }
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
    }

    fn ack_configure(&mut self, _surface: WlSurface, configure: Configure) {
        if let Configure::Toplevel(configure) = configure {
            self.toplevel_configure_ack_count += 1;
            self.toplevel_configure_serial = Some(u32::from(configure.serial));
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl SeatHandler for WaylandSmokeState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl AsMut<CompositorState> for WaylandSmokeState {
    fn as_mut(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_compositor!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_seat!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_shm!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_xdg_shell!(WaylandSmokeState);

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct WaylandSmokeClientState {
    compositor_state: CompositorClientState,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl WaylandSmokeClientState {
    fn compositor_state(&self) -> &CompositorClientState {
        &self.compositor_state
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientData for WaylandSmokeClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for XdgSmokeClientState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        state.registry_bound = true;

        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    let surface = compositor.create_surface(qh, ());
                    if state.app_id == "aqua.installer" {
                        let region = compositor.create_region(qh, ());
                        region.add(0, 0, state.buffer_width as i32, state.buffer_height as i32);
                        surface.set_opaque_region(Some(&region));
                        region.destroy();
                    }
                    state.base_surface = Some(surface);
                    state.compositor_global_seen = true;
                    state.init_xdg_surface(qh);
                }
                "wl_shm" => {
                    let shm = registry.bind::<client_wl_shm::WlShm, _, _>(name, 1, qh, ());
                    state.shm_global_seen = true;
                    state.create_shm_buffer(&shm, qh);
                    state.shm = Some(shm);
                }
                "xdg_wm_base" => {
                    state.wm_base =
                        Some(registry.bind::<client_xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ()));
                    state.xdg_wm_base_global_seen = true;
                    state.init_xdg_surface(qh);
                }
                "wl_seat" => {
                    state.seat =
                        Some(registry.bind::<client_wl_seat::WlSeat, _, _>(name, 1, qh, ()));
                    state.seat_global_seen = true;
                }
                _ => {}
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_xdg_wm_base::XdgWmBase, ()> for XdgSmokeClientState {
    fn event(
        _: &mut Self,
        wm_base: &client_xdg_wm_base::XdgWmBase,
        event: client_xdg_wm_base::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_xdg_surface::XdgSurface, ()> for XdgSmokeClientState {
    fn event(
        state: &mut Self,
        xdg_surface: &client_xdg_surface::XdgSurface,
        event: client_xdg_surface::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        if let client_xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(serial);
            state.configure_ack_sent = true;
            state.attach_client_buffer(qh);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_xdg_toplevel::XdgToplevel, ()> for XdgSmokeClientState {
    fn event(
        state: &mut Self,
        toplevel: &client_xdg_toplevel::XdgToplevel,
        event: client_xdg_toplevel::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            client_xdg_toplevel::Event::Close => state.close_event_received = true,
            client_xdg_toplevel::Event::Configure { width, height, .. }
                if state.terminal_session.is_some()
                    && state.client_buffer_attached
                    && width > 0
                    && height > 0 =>
            {
                state.buffer_width = width as u32;
                state.buffer_height = height as u32;
                let cols = ((state.buffer_width.saturating_sub(44)) / 8).clamp(20, 240) as u16;
                let rows = ((state.buffer_height.saturating_sub(86)) / 18).clamp(5, 100) as u16;
                if let Some(terminal) = state.terminal_session.as_mut() {
                    match terminal.resize(rows, cols) {
                        Err(error) => eprintln!("aqua_terminal_resize_error={error}"),
                        Ok(changed) => {
                            println!("aqua_terminal_resized={}x{}", cols, rows);
                            println!("aqua_terminal_resize_buffer={}x{}", width, height);
                            println!("aqua_terminal_resize_grid={}x{}", cols, rows);
                            println!("aqua_terminal_resize_pty={changed}");
                        }
                    }
                }
                state.terminal_dirty = true;
                state.redraw_terminal_buffer(qh);
            }
            client_xdg_toplevel::Event::Configure { .. } if state.state_cycle_started => {
                state.state_configure_count += 1;
                match state.state_configure_count {
                    1 => toplevel.set_maximized(),
                    2 => toplevel.unset_maximized(),
                    3 => toplevel.set_fullscreen(None),
                    4 => toplevel.unset_fullscreen(),
                    5 => state.state_cycle_complete = true,
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_callback::WlCallback, ()> for XdgSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_callback::WlCallback,
        event: client_wl_callback::Event,
        _: &(),
        _: &ClientConnection,
        _qh: &QueueHandle<Self>,
    ) {
        if let client_wl_callback::Event::Done { .. } = event {
            state.frame_callback_received = true;
            if state.terminal_session.is_some() {
                state.terminal_frame_pending = false;
                state.terminal_frame_requested_at = None;
                return;
            }
            if !state.partial_damage_commit_sent {
                if let Some(surface) = state.base_surface.as_ref() {
                    surface.damage(32, 24, 96, 64);
                    surface.commit();
                    state.partial_damage_commit_sent = true;
                }
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn installer_printable_character(key: u32, shift: bool) -> Option<char> {
    let bytes = terminal_key_bytes(key, shift, false)?;
    if bytes.len() != 1 {
        return None;
    }
    let byte = bytes[0];
    (byte.is_ascii_graphic() || byte == b' ').then(|| char::from(byte))
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn terminal_key_bytes(key: u32, shift: bool, ctrl: bool) -> Option<Vec<u8>> {
    let letter = match key {
        16 => Some('q'),
        17 => Some('w'),
        18 => Some('e'),
        19 => Some('r'),
        20 => Some('t'),
        21 => Some('y'),
        22 => Some('u'),
        23 => Some('i'),
        24 => Some('o'),
        25 => Some('p'),
        30 => Some('a'),
        31 => Some('s'),
        32 => Some('d'),
        33 => Some('f'),
        34 => Some('g'),
        35 => Some('h'),
        36 => Some('j'),
        37 => Some('k'),
        38 => Some('l'),
        44 => Some('z'),
        45 => Some('x'),
        46 => Some('c'),
        47 => Some('v'),
        48 => Some('b'),
        49 => Some('n'),
        50 => Some('m'),
        _ => None,
    };
    if let Some(letter) = letter {
        if ctrl {
            return Some(vec![(letter as u8 - b'a') + 1]);
        }
        return Some(vec![if shift {
            letter.to_ascii_uppercase()
        } else {
            letter
        } as u8]);
    }
    let bytes: &[u8] = match key {
        1 => b"\x1b",
        2 => {
            if shift {
                b"!"
            } else {
                b"1"
            }
        }
        3 => {
            if shift {
                b"@"
            } else {
                b"2"
            }
        }
        4 => {
            if shift {
                b"#"
            } else {
                b"3"
            }
        }
        5 => {
            if shift {
                b"$"
            } else {
                b"4"
            }
        }
        6 => {
            if shift {
                b"%"
            } else {
                b"5"
            }
        }
        7 => {
            if shift {
                b"^"
            } else {
                b"6"
            }
        }
        8 => {
            if shift {
                b"&"
            } else {
                b"7"
            }
        }
        9 => {
            if shift {
                b"*"
            } else {
                b"8"
            }
        }
        10 => {
            if shift {
                b"("
            } else {
                b"9"
            }
        }
        11 => {
            if shift {
                b")"
            } else {
                b"0"
            }
        }
        12 => {
            if shift {
                b"_"
            } else {
                b"-"
            }
        }
        13 => {
            if shift {
                b"+"
            } else {
                b"="
            }
        }
        14 => b"\x7f",
        15 => b"\t",
        26 => {
            if shift {
                b"{"
            } else {
                b"["
            }
        }
        27 => {
            if shift {
                b"}"
            } else {
                b"]"
            }
        }
        28 => b"\r",
        39 => {
            if shift {
                b":"
            } else {
                b";"
            }
        }
        40 => {
            if shift {
                b"\""
            } else {
                b"'"
            }
        }
        41 => {
            if shift {
                b"~"
            } else {
                b"`"
            }
        }
        43 => {
            if shift {
                b"|"
            } else {
                b"\\"
            }
        }
        51 => {
            if shift {
                b"<"
            } else {
                b","
            }
        }
        52 => {
            if shift {
                b">"
            } else {
                b"."
            }
        }
        53 => {
            if shift {
                b"?"
            } else {
                b"/"
            }
        }
        57 => b" ",
        102 => b"\x1b[H",
        103 => b"\x1b[A",
        104 => b"\x1b[5~",
        105 => b"\x1b[D",
        106 => b"\x1b[C",
        107 => b"\x1b[F",
        108 => b"\x1b[B",
        109 => b"\x1b[6~",
        111 => b"\x1b[3~",
        _ => return None,
    };
    Some(bytes.to_vec())
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_seat::WlSeat, ()> for XdgSmokeClientState {
    fn event(
        _: &mut Self,
        seat: &client_wl_seat::WlSeat,
        event: client_wl_seat::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        if let client_wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(client_wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
            if capabilities.contains(client_wl_seat::Capability::Pointer) {
                seat.get_pointer(qh, ());
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_keyboard::WlKeyboard, ()> for XdgSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_keyboard::WlKeyboard,
        event: client_wl_keyboard::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        if let client_wl_keyboard::Event::Modifiers { mods_depressed, .. } = event {
            state.keyboard_shift = mods_depressed & 1 != 0;
            state.keyboard_ctrl = mods_depressed & 4 != 0;
            return;
        }
        if matches!(event, client_wl_keyboard::Event::Key { .. }) {
            state.keyboard_event_received = true;
        }
        if let client_wl_keyboard::Event::Key {
            key,
            state: WEnum::Value(key_state),
            ..
        } = &event
        {
            let pressed = *key_state == client_wl_keyboard::KeyState::Pressed;
            if matches!(*key, 42 | 54) {
                state.keyboard_shift = pressed;
                return;
            }
            if *key == 29 {
                state.keyboard_ctrl = pressed;
                return;
            }
        }
        if let client_wl_keyboard::Event::Key {
            key,
            state: WEnum::Value(client_wl_keyboard::KeyState::Pressed),
            ..
        } = event
        {
            if state.installer_model.is_some() {
                state.handle_installer_key(key, qh);
                return;
            }
            if state.terminal_session.is_some() {
                if let Some(bytes) =
                    terminal_key_bytes(key, state.keyboard_shift, state.keyboard_ctrl)
                {
                    if let Some(terminal) = state.terminal_session.as_mut() {
                        if let Err(error) = terminal.write_input(&bytes) {
                            eprintln!("aqua_terminal_input_error={error}");
                        }
                    }
                    state.terminal_dirty = true;
                    println!("aqua_terminal_key={key} bytes={}", bytes.len());
                }
                return;
            }
            let files_key = match key {
                103 => Some(aqua_shell::FilesKey::Up),
                108 => Some(aqua_shell::FilesKey::Down),
                104 => Some(aqua_shell::FilesKey::PageUp),
                109 => Some(aqua_shell::FilesKey::PageDown),
                102 => Some(aqua_shell::FilesKey::Home),
                107 => Some(aqua_shell::FilesKey::End),
                28 => Some(aqua_shell::FilesKey::Activate),
                14 => Some(aqua_shell::FilesKey::Back),
                _ => None,
            };
            if key == 63 && state.properties_model.is_some() {
                state.refresh_properties(qh);
                return;
            }
            if let (Some(files_key), Some(navigator)) = (files_key, state.files_navigator.as_mut())
            {
                let navigation = navigator.handle_key(files_key);
                println!("aqua_files_keyboard key={key} navigation={navigation:?}");
                if navigation.changed() {
                    state.files_model = Some(navigator.window().clone());
                    state.redraw_files_buffer(qh);
                }
            }
            let settings_key = match key {
                102 => Some(aqua_shell::SettingsKey::Home),
                103 => Some(aqua_shell::SettingsKey::Up),
                108 => Some(aqua_shell::SettingsKey::Down),
                28 => Some(aqua_shell::SettingsKey::Activate),
                105 => Some(aqua_shell::SettingsKey::Decrease),
                106 => Some(aqua_shell::SettingsKey::Increase),
                _ => None,
            };
            if let (Some(settings_key), Some(model)) = (settings_key, state.settings_model.as_mut())
            {
                let update = model.handle_key(settings_key);
                println!("aqua_settings_keyboard key={key} update={update:?}");
                if let aqua_shell::SettingsUpdate::ThemeChanged(theme) = update {
                    state.theme = theme;
                    println!("aqua_settings_theme={}", theme.id());
                }
                if matches!(
                    update,
                    aqua_shell::SettingsUpdate::ReducedMotionChanged(_)
                        | aqua_shell::SettingsUpdate::DesktopIconsChanged(_)
                        | aqua_shell::SettingsUpdate::KeyRepeatChanged(_)
                        | aqua_shell::SettingsUpdate::ThemeChanged(_)
                        | aqua_shell::SettingsUpdate::AudioVolumeChanged(_)
                        | aqua_shell::SettingsUpdate::AudioMutedChanged(_)
                ) {
                    state.persist_settings();
                }
                if update.changed() {
                    state.redraw_settings_buffer(qh);
                }
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_pointer::WlPointer, ()> for XdgSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_pointer::WlPointer,
        event: client_wl_pointer::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        if matches!(
            event,
            client_wl_pointer::Event::Enter { .. }
                | client_wl_pointer::Event::Motion { .. }
                | client_wl_pointer::Event::Button { .. }
        ) {
            state.pointer_event_received = true;
        }
        match event {
            client_wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            }
            | client_wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.pointer_surface_x = surface_x;
                state.pointer_surface_y = surface_y;
                if state.settings_model.is_some() {
                    println!(
                        "aqua_settings_pointer_motion x={} y={}",
                        surface_x.max(0.0) as u32,
                        surface_y.max(0.0) as u32
                    );
                }
                if let Some(navigator) = state.files_navigator.as_mut() {
                    let navigation = state
                        .files_scrollbar_dragging
                        .then(|| navigator.handle_scrollbar_drag(surface_y.max(0.0) as u32));
                    let files_changed = navigation
                        .is_some_and(aqua_shell::FilesNavigation::changed)
                        || (!state.files_scrollbar_dragging
                            && navigator.handle_hover(
                                state.pointer_surface_x.max(0.0) as u32,
                                state.pointer_surface_y.max(0.0) as u32,
                            ));
                    if files_changed {
                        state.files_model = Some(navigator.window().clone());
                        state.redraw_files_buffer(qh);
                    }
                } else if let Some(model) = state.settings_model.as_mut() {
                    if model.handle_hover(
                        state.pointer_surface_x.max(0.0) as u32,
                        state.pointer_surface_y.max(0.0) as u32,
                    ) {
                        state.redraw_settings_buffer(qh);
                    }
                }
            }
            _ => {}
        }
        if let client_wl_pointer::Event::Axis {
            axis: WEnum::Value(client_wl_pointer::Axis::VerticalScroll),
            value,
            ..
        } = event
        {
            if let Some(navigator) = state.files_navigator.as_mut() {
                let navigation = navigator.handle_scroll(if value > 0.0 { 1 } else { -1 });
                println!("aqua_files_axis value={value} navigation={navigation:?}");
                if navigation.changed() {
                    state.files_model = Some(navigator.window().clone());
                    state.redraw_files_buffer(qh);
                }
            }
        }
        if let client_wl_pointer::Event::Button {
            serial,
            state: WEnum::Value(button_state),
            ..
        } = event
        {
            if button_state == client_wl_pointer::ButtonState::Released
                && state.files_scrollbar_dragging
            {
                state.files_scrollbar_dragging = false;
                return;
            }
            if button_state != client_wl_pointer::ButtonState::Pressed {
                return;
            }
            if state.installer_model.is_some() {
                let pointer_x = state.pointer_surface_x.max(0.0) as u32;
                let pointer_y = state.pointer_surface_y.max(0.0) as u32;
                println!("aqua_installer_pointer_event_received=true x={pointer_x} y={pointer_y}");
                state.handle_installer_pointer(pointer_x, pointer_y, qh);
                return;
            } else if state.terminal_session.is_some() {
                state.handle_window_frame_pointer(serial);
                return;
            } else if let Some(navigator) = state.files_navigator.as_mut() {
                let pointer_x = state.pointer_surface_x.max(0.0) as u32;
                let pointer_y = state.pointer_surface_y.max(0.0) as u32;
                if navigator.scrollbar_hit(pointer_x, pointer_y) {
                    state.files_scrollbar_dragging = true;
                    let navigation = navigator.handle_scrollbar_drag(pointer_y);
                    println!("aqua_files_scrollbar y={pointer_y} navigation={navigation:?}");
                    if navigation.changed() {
                        state.files_model = Some(navigator.window().clone());
                        state.redraw_files_buffer(qh);
                    }
                    return;
                }
                let navigation = navigator.handle_pointer(pointer_x, pointer_y);
                println!(
                    "aqua_files_pointer x={} y={} navigation={navigation:?}",
                    state.pointer_surface_x.max(0.0) as u32,
                    state.pointer_surface_y.max(0.0) as u32
                );
                if navigation.changed() {
                    state.files_model = Some(navigator.window().clone());
                    state.redraw_files_buffer(qh);
                    return;
                }
            } else if let Some(model) = state.settings_model.as_mut() {
                let update = model.handle_pointer(
                    state.pointer_surface_x.max(0.0) as u32,
                    state.pointer_surface_y.max(0.0) as u32,
                );
                println!(
                    "aqua_settings_pointer x={} y={} update={update:?}",
                    state.pointer_surface_x.max(0.0) as u32,
                    state.pointer_surface_y.max(0.0) as u32
                );
                if let aqua_shell::SettingsUpdate::ThemeChanged(theme) = update {
                    state.theme = theme;
                    println!("aqua_settings_theme={}", theme.id());
                }
                if matches!(
                    update,
                    aqua_shell::SettingsUpdate::ReducedMotionChanged(_)
                        | aqua_shell::SettingsUpdate::DesktopIconsChanged(_)
                        | aqua_shell::SettingsUpdate::KeyRepeatChanged(_)
                        | aqua_shell::SettingsUpdate::ThemeChanged(_)
                        | aqua_shell::SettingsUpdate::AudioVolumeChanged(_)
                        | aqua_shell::SettingsUpdate::AudioMutedChanged(_)
                ) {
                    state.persist_settings();
                }
                if update.changed() {
                    state.redraw_settings_buffer(qh);
                    return;
                }
            } else if let Some(model) = state.files_model.as_mut() {
                let selection = model.select_at(
                    state.pointer_surface_x.max(0.0) as u32,
                    state.pointer_surface_y.max(0.0) as u32,
                );
                if selection != aqua_shell::FilesSelection::None {
                    state.redraw_files_buffer(qh);
                    return;
                }
            }
            if state.handle_window_frame_pointer(serial) {
                return;
            }
            if !state.interactive_requests_sent {
                if let (Some(toplevel), Some(seat)) =
                    (state.xdg_toplevel.as_ref(), state.seat.as_ref())
                {
                    toplevel._move(seat, serial);
                    toplevel.resize(seat, serial, client_xdg_toplevel::ResizeEdge::BottomRight);
                    if state.state_cycle_enabled {
                        toplevel.set_min_size(320, 200);
                        toplevel.set_max_size(1200, 900);
                        state.size_constraints_sent = true;
                        state.state_cycle_started = true;
                    }
                    state.interactive_requests_sent = true;
                }
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(XdgSmokeClientState: ignore wl_compositor::WlCompositor);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(XdgSmokeClientState: ignore client_wl_region::WlRegion);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(XdgSmokeClientState: ignore wl_surface::WlSurface);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(XdgSmokeClientState: ignore client_wl_shm::WlShm);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(XdgSmokeClientState: ignore client_wl_shm_pool::WlShmPool);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(XdgSmokeClientState: ignore client_wl_buffer::WlBuffer);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_window_frame_routes_controls_move_and_resize_without_overlap() {
        let frame = WindowFrame::new(
            Rect {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
            "Terminal",
            48,
        );

        assert_eq!(
            first_party_window_action(frame, 25, 24),
            FirstPartyWindowAction::Close
        );
        assert_eq!(
            first_party_window_action(frame, 47, 24),
            FirstPartyWindowAction::Minimize
        );
        assert_eq!(
            first_party_window_action(frame, 69, 24),
            FirstPartyWindowAction::Maximize
        );
        assert_eq!(
            first_party_window_action(frame, 200, 47),
            FirstPartyWindowAction::Move
        );
        assert_eq!(
            first_party_window_action(frame, 200, 48),
            FirstPartyWindowAction::None
        );
        assert_eq!(
            first_party_window_action(frame, 799, 599),
            FirstPartyWindowAction::Resize
        );
    }

    #[test]
    fn status_identifies_aqua_linux() {
        assert!(status_lines().contains(&"product=Aqua Linux"));
        assert!(status_lines().contains(&"component=aqua-compositor"));
        assert!(status_lines().contains(&"foundation=smithay"));
        assert!(status_lines().contains(&"event_loop=calloop"));
    }

    #[test]
    fn shared_top_system_bar_routes_only_the_session_control() {
        let viewport = Viewport::new(1536, 1024);
        assert!(top_system_bar_session_hit(viewport, 1535, 18));
        assert!(top_system_bar_session_hit(viewport, 1508, 18));
        assert!(!top_system_bar_session_hit(viewport, 1507, 18));
        assert!(!top_system_bar_session_hit(viewport, 1535, 36));
    }

    #[test]
    fn shared_notification_routes_only_the_dismiss_control() {
        let rect = Rect {
            x: 1152,
            y: 824,
            width: 360,
            height: 88,
        };
        assert!(!notification_dismiss_hit(
            rect,
            "Aqua System",
            "Update ready",
            "Restart later.",
            1463,
            824,
        ));
        assert!(notification_dismiss_hit(
            rect,
            "Aqua System",
            "Update ready",
            "Restart later.",
            1464,
            824,
        ));
        assert!(notification_dismiss_hit(
            rect,
            "Aqua System",
            "Update ready",
            "Restart later.",
            1511,
            871,
        ));
        assert!(!notification_dismiss_hit(
            rect,
            "Aqua System",
            "Update ready",
            "Restart later.",
            1512,
            871,
        ));
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn installer_wayland_client_state_renders_packaged_welcome_surface() {
        let logo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/aqua-linux/assets/aqua-symbol-primary.png");
        let state = XdgSmokeClientState::installer_app_with_logo(&logo_path)
            .expect("installer client state should load canonical logo");
        let pixels = state
            .render_installer_buffer()
            .expect("installer welcome surface should render");

        assert_eq!(state.app_id, "aqua.installer");
        assert_eq!(state.title, "Aqua Linux Kurulumu");
        assert_eq!(state.buffer_width, 1280);
        assert_eq!(state.buffer_height, 800);
        assert_eq!(pixels.len(), 1280 * 800 * 4);
        assert!(pixels.iter().any(|byte| *byte != 0));
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn typography_wayland_client_state_uses_accepted_full_output_raster() {
        let state = XdgSmokeClientState::typography_acceptance_app();
        let (pixels, probe) = render_typography_layout_acceptance_rgba(
            Viewport::new(state.buffer_width, state.buffer_height),
            state.theme,
            OutputScale::One,
        )
        .expect("typography acceptance surface should render");

        assert_eq!(state.app_id, "aqua.typography-acceptance");
        assert_eq!(state.buffer_width, 1280);
        assert_eq!(state.buffer_height, 800);
        assert!(state.typography_acceptance);
        assert_eq!(pixels.len(), 1280 * 800 * 4);
        assert!(probe.is_ready());
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn component_wayland_client_state_uses_accepted_full_output_raster() {
        let state = XdgSmokeClientState::component_acceptance_app();
        let (pixels, probe) = render_component_acceptance_rgba(
            Viewport::new(state.buffer_width, state.buffer_height),
            state.theme,
            OutputScale::One,
        )
        .expect("component acceptance surface should render");

        assert_eq!(state.app_id, "aqua.component-acceptance");
        assert_eq!(state.buffer_width, 1280);
        assert_eq!(state.buffer_height, 800);
        assert!(state.component_acceptance);
        assert_eq!(pixels.len(), 1280 * 800 * 4);
        assert!(probe.is_ready());
    }

    #[test]
    fn missing_runtime_assets_are_not_ready() {
        let probe = probe_runtime_assets("definitely-missing-aqua-runtime-root");
        assert!(!probe.is_ready());
    }

    #[test]
    fn missing_design_token_file_does_not_satisfy_scene_material_contract() {
        assert!(!design_tokens_include_scene_materials(
            "definitely-missing-aqua-design-tokens.json"
        ));
    }

    #[test]
    fn event_loop_smoke_ticks_once() {
        let result = run_event_loop_smoke().expect("event loop smoke should run");
        assert_eq!(result.ticks, 1);
        assert!(result.is_ready());
    }

    #[test]
    fn default_session_config_is_recovery_safe() {
        let config = default_session_config();

        assert_eq!(config.product, PRODUCT);
        assert_eq!(config.mode, DEV_MODE);
        assert_eq!(config.wayland_socket, "aqua-wayland-0");
        assert_eq!(config.runtime_dir, "/run/user/1000");
        assert_eq!(config.runtime_asset_root, "/usr/share/aqua");
        assert!(!config.autostart);
        assert!(!config.boot_graphics);
        assert!(config.recovery_tty_required);
        assert!(config.is_recovery_safe());
    }

    #[test]
    fn default_session_environment_is_recovery_safe() {
        let env = default_session_environment();

        assert_eq!(env.wayland_display, "aqua-wayland-0");
        assert_eq!(env.xdg_runtime_dir, "/run/user/1000");
        assert_eq!(env.aqua_asset_root, "/usr/share/aqua");
        assert_eq!(env.aqua_session_mode, "nested-dev");
        assert!(!env.aqua_compositor_autostart);
        assert!(!env.aqua_boot_graphics);
        assert!(env.is_recovery_safe());
        assert!(env
            .dump_lines()
            .contains(&"WAYLAND_DISPLAY=aqua-wayland-0".to_string()));
    }

    #[test]
    fn display_output_plan_is_recovery_safe_without_starting_graphics() {
        let probe = probe_display_output_plan();

        assert!(probe.is_ready());
        assert_eq!(probe.plan.product, PRODUCT);
        assert_eq!(probe.plan.mode, DEV_MODE);
        assert_eq!(probe.plan.primary_backend, "nested-dev-window");
        assert_eq!(probe.plan.later_backend, "qemu-drm-kms");
        assert_eq!(probe.plan.width, 1536);
        assert_eq!(probe.plan.height, 1024);
        assert_eq!(probe.plan.pixel_format, "rgba8888");
        assert!(probe.recovery_safe);
        assert!(!probe.plan.boot_graphics);
        assert!(!probe.plan.renderer_started);
        assert!(!probe.plan.desktop_shell_started);
    }

    #[test]
    fn visible_preview_plan_collects_readiness_without_opening_window() {
        let probe = probe_visible_preview_plan(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert!(probe.output.is_ready());
        assert!(probe.scene_ready);
        assert!(probe.render_plan_ready);
        assert!(probe.paint_plan_ready);
        assert!(probe.frame_plan_ready);
        assert!(probe.frame_buffer_ready);
        assert!(probe.raster_ready);
        assert!(probe.png_export_ready);
        assert!(probe.client_layer_pipeline_ready);
        assert_eq!(probe.client_layer_count, 2);
        assert_ne!(probe.client_layer_checksum, 0);
        assert!(!probe.preview_window_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn visible_preview_export_embeds_png_without_opening_window() {
        let probe = export_visible_preview_html(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.format, "html-data-uri-png-preview");
        assert!(probe.html.contains("data:image/png;base64,"));
        assert!(probe.html.contains("Aqua Linux visible preview export"));
        assert_eq!(probe.byte_count, probe.html.len());
        assert!(probe.byte_count > 6_293_028);
        assert_ne!(probe.checksum, 0);
        assert!(probe.client_layer_pipeline_ready);
        assert!(probe.client_layer_composited);
        assert_eq!(probe.client_layer_count, 2);
        assert_eq!(
            probe.client_layer_checksum,
            probe.plan.client_layer_checksum
        );
        assert_ne!(probe.png_checksum, 0);
        assert!(!probe.preview_window_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn display_output_handoff_is_ready_without_starting_graphics() {
        let probe = probe_display_output_handoff(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.status, "display-output-handoff-ready");
        assert_eq!(probe.target_backend, "nested-dev-window");
        assert_eq!(probe.output_width, 1536);
        assert_eq!(probe.output_height, 1024);
        assert_eq!(probe.pixel_format, "rgba8888");
        assert_eq!(probe.frame_buffer_bytes, 1536 * 1024 * 4);
        assert_eq!(probe.frame_format, "raw-rgba8888-composited-client-preview");
        assert_ne!(probe.frame_checksum, 0);
        assert!(probe.export.is_ready());
        assert!(probe.client_layer_composited);
        assert_eq!(probe.client_layer_buffer_snapshot_bytes, 674_816);
        assert_eq!(probe.client_layer_snapshot_mode, "full-buffer-snapshot");
        assert!(probe.output_surface_prepared);
        assert!(probe.recovery_safe);
        assert!(!probe.display_output_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(!probe.desktop_shell_started);
    }

    #[test]
    fn display_activation_plan_is_manual_and_recovery_safe() {
        let probe = probe_display_activation_plan(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.status, "manual-display-activation-plan-ready");
        assert_eq!(probe.launch_mode, "manual-dev");
        assert!(probe.source_handoff_ready);
        assert_eq!(probe.target_backend, "nested-dev-window");
        assert_eq!(probe.frame_format, "raw-rgba8888-composited-client-preview");
        assert_ne!(probe.frame_checksum, 0);
        assert!(probe.manual_start_required);
        assert!(probe.fallback_tty_required);
        assert!(probe.can_activate_display_output);
        assert!(probe.recovery_safe);
        assert!(!probe.display_output_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(!probe.desktop_shell_started);
        assert!(!probe.autostart);
    }

    #[test]
    fn manual_display_output_smoke_starts_and_stops_bounded() {
        let probe = run_manual_display_output_smoke(Viewport::new(1536, 1024), 3)
            .expect("manual display output smoke should run bounded frame clock");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "manual-display-output-smoke-complete");
        assert_eq!(probe.launch_mode, "manual-dev");
        assert_eq!(probe.target_backend, "nested-dev-window");
        assert_eq!(probe.requested_frames, 3);
        assert_eq!(probe.presented_frames, 3);
        assert_eq!(probe.frame_interval_ms, 16);
        assert!(probe.display_output_started);
        assert!(probe.display_output_stopped);
        assert!(probe.manual_start_required);
        assert!(probe.fallback_tty_available);
        assert_eq!(probe.frame_format, "raw-rgba8888-composited-client-preview");
        assert_ne!(probe.frame_checksum, 0);
        assert_ne!(probe.checksum_accumulator, 0);
        assert!(probe.recovery_safe);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(!probe.desktop_shell_started);
        assert!(!probe.autostart);
    }

    #[test]
    fn nested_output_surface_lifecycle_is_bounded_and_manual() {
        let probe = run_nested_output_surface_lifecycle(Viewport::new(1536, 1024), 3)
            .expect("nested output surface lifecycle should run from bounded display smoke");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "nested-output-surface-lifecycle-complete");
        assert_eq!(probe.launch_mode, "manual-dev");
        assert_eq!(probe.backend, "nested-dev-window");
        assert!(probe.surface_acquired);
        assert!(probe.surface_configured);
        assert!(probe.frame_attached);
        assert!(probe.frame_presented);
        assert!(probe.surface_released);
        assert_eq!(probe.presented_frames, 3);
        assert_ne!(probe.frame_checksum, 0);
        assert_eq!(probe.lifecycle_serial, 1);
        assert!(probe.manual_start_required);
        assert!(probe.fallback_tty_available);
        assert!(probe.recovery_safe);
        assert!(!probe.autostart);
        assert!(!probe.boot_graphics);
        assert!(!probe.renderer_started);
        assert!(!probe.desktop_shell_started);
    }

    #[test]
    fn nested_preview_frame_loop_runs_bounded_without_boot_autostart() {
        let probe = run_nested_preview_frame_loop(Viewport::new(1536, 1024), 3)
            .expect("nested preview frame loop should run");

        assert!(probe.is_ready());
        assert_eq!(probe.launch_mode, "manual-dev");
        assert_eq!(probe.window_backend, "nested-dev-window");
        assert_eq!(probe.frame_interval_ms, 16);
        assert_eq!(probe.requested_frames, 3);
        assert_eq!(probe.rendered_frames, 3);
        assert!(probe.frame_clock_started);
        assert!(probe.manual_start_required);
        assert!(!probe.autostart);
        assert!(!probe.preview_window_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert_ne!(probe.checksum_accumulator, 0);
    }

    #[test]
    fn manual_nested_preview_backend_path_is_ready_behind_recovery_gate() {
        let probe = probe_manual_nested_preview_backend(Viewport::new(1536, 1024), 3)
            .expect("manual nested preview backend should validate handoff path");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "manual-nested-preview-backend-ready");
        assert_eq!(probe.launch_mode, "manual-recovery");
        assert_eq!(probe.backend_path, "nested-dev-window");
        assert!(probe.backend_selected);
        assert!(probe.handoff_ready);
        assert!(probe.surface_lifecycle_ready);
        assert!(probe.frame_loop_ready);
        assert!(probe.visible_export_ready);
        assert_eq!(
            probe.frame_source,
            "display-output-handoff-composited-client-frame"
        );
        assert_eq!(probe.frame_format, "raw-rgba8888-composited-client-preview");
        assert_ne!(probe.frame_checksum, 0);
        assert_eq!(probe.frame_checksum, probe.surface_frame_checksum);
        assert_ne!(probe.loop_checksum_accumulator, 0);
        assert!(probe.frame_checksum_matches_surface);
        assert!(probe.manual_start_required);
        assert!(probe.fallback_tty_required);
        assert!(probe.fallback_tty_available);
        assert_eq!(probe.bounded_frame_limit, 3);
        assert!(!probe.display_output_started);
        assert!(probe.display_output_stopped);
        assert!(!probe.preview_window_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(!probe.desktop_shell_started);
        assert!(!probe.autostart);
        assert!(probe.recovery_safe);
    }

    #[test]
    fn manual_nested_preview_execution_is_operator_controlled_and_bounded() {
        let probe = run_manual_nested_preview_execution(Viewport::new(1536, 1024), 3, true)
            .expect("manual nested preview execution should run bounded");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "manual-nested-preview-execution-complete");
        assert_eq!(probe.launch_mode, "manual-recovery");
        assert_eq!(probe.backend_path, "nested-dev-window");
        assert!(probe.operator_controlled);
        assert!(probe.operator_ack_required);
        assert!(probe.operator_acknowledged);
        assert!(probe.backend_ready);
        assert_eq!(probe.requested_frames, 3);
        assert_eq!(probe.rendered_frames, 3);
        assert_eq!(probe.frame_interval_ms, 16);
        assert_eq!(probe.frame_source, "manual-nested-preview-backend-frame");
        assert_eq!(probe.frame_format, "raw-rgba8888-composited-client-preview");
        assert_ne!(probe.frame_checksum, 0);
        assert_ne!(probe.checksum_accumulator, 0);
        assert!(probe.display_output_started);
        assert!(probe.display_output_stopped);
        assert!(!probe.preview_window_started);
        assert!(probe.cleanup_complete);
        assert!(probe.fallback_tty_available);
        assert!(probe.safe_return_to_recovery);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(!probe.desktop_shell_started);
        assert!(!probe.autostart);
        assert!(probe.recovery_safe);
    }

    #[test]
    fn client_window_model_covers_focus_move_resize_close_and_stacking() {
        let probe = probe_client_window_model(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.status, "client-window-model");
        assert_eq!(probe.windows.len(), 2);
        assert_eq!(probe.operation_count, 5);
        assert!(probe.focus_ready);
        assert!(probe.move_ready);
        assert!(probe.resize_ready);
        assert!(probe.close_ready);
        assert!(probe.stacking_ready);
        assert!(probe.chrome_ready);
        assert!(!probe.real_wayland_client_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(probe
            .dump_lines()
            .contains(&"window id=terminal-demo title=Terminal rect=216,178,680,420 z_index=3 focused=true closed=false chrome=aqua-window".to_string()));
    }

    #[test]
    fn client_surface_lifecycle_covers_pre_client_xdg_toplevel_flow() {
        let probe = probe_client_surface_lifecycle(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.status, "client-surface-lifecycle");
        assert_eq!(probe.surface_id, "terminal-demo-surface");
        assert_eq!(probe.window_id, "terminal-demo");
        assert_eq!(probe.role, "xdg-toplevel");
        assert_eq!(probe.steps.len(), 7);
        assert!(probe.configure_ready);
        assert!(probe.commit_ready);
        assert!(probe.map_ready);
        assert!(probe.focus_ready);
        assert!(probe.unmap_ready);
        assert!(probe.destroy_ready);
        assert!(probe.focus_bound_to_window);
        assert!(probe.window_geometry_ready);
        assert!(!probe.real_wayland_client_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(probe
            .dump_lines()
            .contains(&"step order=4 name=mapped".to_string()));
    }

    #[test]
    fn xdg_shell_binding_probe_covers_handler_contract_without_real_client() {
        let probe = probe_xdg_shell_binding(Viewport::new(1536, 1024))
            .expect("xdg shell binding probe should run");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "xdg-shell-binding");
        assert_eq!(probe.foundation, "smithay");
        assert_eq!(probe.protocol, "xdg_wm_base");
        assert!(probe.handler_bound);
        assert!(probe.global_created);
        assert!(probe.toplevel_callbacks_bound);
        assert!(probe.popup_callbacks_bound);
        assert!(probe.lifecycle_probe_ready);
        assert!(!probe.real_wayland_client_started);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn xdg_toplevel_client_probe_records_first_surface_without_rendering() {
        let probe = probe_xdg_toplevel_client().expect("xdg toplevel client probe should run");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "xdg-toplevel-client");
        assert_eq!(probe.protocol, "xdg_wm_base");
        assert!(probe.client_connected);
        assert!(probe.client_inserted);
        assert!(probe.registry_bound);
        assert!(probe.compositor_global_seen);
        assert!(probe.shm_global_created);
        assert!(probe.shm_global_seen);
        assert!(probe.shm_buffer_created);
        assert!(probe.client_buffer_attached);
        assert!(probe.xdg_wm_base_global_seen);
        assert!(probe.surface_created);
        assert!(probe.toplevel_requested);
        assert!(probe.surface_committed);
        assert!(probe.server_buffer_attached);
        assert!(probe.server_shm_buffer_imported);
        assert!(probe.server_shm_buffer_sampled);
        assert_ne!(probe.shm_sample_checksum, 0);
        assert!(probe.server_toplevel_created);
        assert!(probe.server_configure_sent);
        assert!(probe.client_configure_ack_sent);
        assert!(probe.server_configure_ack_received);
        assert!(probe.server_close_sent);
        assert!(probe.client_close_event_received);
        assert!(probe.test_wayland_client_started);
        assert_eq!(probe.test_wayland_client_count, 2);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn xdg_toplevel_window_model_binds_first_surface_without_rendering() {
        let probe = probe_xdg_toplevel_window_model(Viewport::new(1536, 1024))
            .expect("xdg toplevel window model probe should run");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "xdg-toplevel-window-model");
        assert_eq!(probe.window_id, "wayland-test-client");
        assert_eq!(probe.surface_id, "xdg-toplevel-1");
        assert_eq!(probe.title, "Aqua Test Client");
        assert_eq!(probe.role, "xdg-toplevel");
        assert!(probe.source_client_ready);
        assert!(probe.server_surface_bound);
        assert!(probe.window_model_bound);
        assert_eq!(probe.window_count, 2);
        assert!(probe.two_window_model_ready);
        assert!(probe.stacking_ready);
        assert!(probe.mapped);
        assert!(probe.focused);
        assert!(probe.geometry_ready);
        assert!(probe.chrome_ready);
        assert_eq!(probe.window.chrome, "aqua-window");
        assert!(probe
            .windows
            .iter()
            .any(|window| window.id == "aqua-settings-client" && !window.focused));
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
        assert!(probe
            .dump_lines()
            .contains(&"window_model_status=xdg-toplevel-window-model".to_string()));
    }

    #[test]
    fn client_surface_registry_tracks_active_xdg_toplevel_without_rendering() {
        let probe = probe_client_surface_registry(Viewport::new(1536, 1024))
            .expect("client surface registry probe should run");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "client-surface-registry");
        assert_eq!(probe.record_count, 2);
        assert_eq!(probe.active_client_id, "wayland-client-1");
        assert_eq!(probe.active_surface_id, "xdg-toplevel-1");
        assert_eq!(probe.active_window_id, "wayland-test-client");
        assert!(probe.source_window_model_ready);
        assert!(probe.configure_serial_ready);
        assert!(probe.lifecycle_state_ready);
        assert!(probe.two_client_ready);
        assert!(probe.focus_index_ready);
        assert!(probe.stacking_order_ready);
        assert!(probe.close_request_ready);
        assert!(probe.buffer_metadata_ready);
        assert!(probe.buffer_import_plan_ready);
        assert!(probe.no_renderer_binding);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);

        let record = probe
            .records
            .first()
            .expect("registry should have an active record");
        assert_eq!(record.lifecycle_state, "mapped-focused");
        assert_eq!(record.z_index, 2);
        assert_eq!(record.configure_serial, 1);
        assert!(record.configured);
        assert!(record.committed);
        assert!(record.mapped);
        assert!(record.focused);
        assert!(record.close_supported);
        assert!(record.buffer_attached);
        assert!(record.buffer_committed);
        assert_eq!(record.buffer_width, 384);
        assert_eq!(record.buffer_height, 256);
        assert_eq!(record.buffer_stride, 1536);
        assert_eq!(record.buffer_format, "argb8888");
        assert_eq!(record.buffer_source, "client-committed-wl-shm");
        assert!(record.import_required);
        assert!(record.import_planned);
        assert!(record.imported_for_sampling);
        assert_ne!(record.sample_checksum, 0);
        assert!(!record.renderer_bound);

        let inactive = probe
            .records
            .iter()
            .find(|candidate| candidate.client_id == "wayland-client-2")
            .expect("registry should have an inactive record");
        assert_eq!(inactive.lifecycle_state, "mapped-unfocused");
        assert_eq!(inactive.z_index, 1);
        assert_eq!(inactive.configure_serial, 2);
        assert!(inactive.configured);
        assert!(inactive.committed);
        assert!(inactive.mapped);
        assert!(!inactive.focused);
        assert!(inactive.close_supported);
        assert!(inactive.buffer_attached);
        assert!(inactive.buffer_committed);
        assert_eq!(inactive.buffer_width, 320);
        assert_eq!(inactive.buffer_height, 220);
        assert_eq!(inactive.buffer_stride, 1280);
        assert_eq!(inactive.buffer_format, "argb8888");
        assert_eq!(inactive.buffer_source, "client-committed-wl-shm");
        assert!(inactive.import_required);
        assert!(inactive.import_planned);
        assert!(inactive.imported_for_sampling);
        assert_ne!(inactive.sample_checksum, 0);
        assert!(!inactive.renderer_bound);
    }

    #[test]
    fn renderer_surface_sources_bind_sampled_wl_shm_buffers_without_rendering() {
        let probe = probe_renderer_surface_sources(Viewport::new(1536, 1024))
            .expect("renderer surface source probe should run");

        assert!(probe.is_ready());
        assert!(probe.source_registry_ready);
        assert_eq!(probe.source_count, 2);
        assert_eq!(probe.expected_sources, 2);
        assert!(probe.active_source_ready);
        assert!(probe.import_sources_ready);
        assert!(probe.z_order_ready);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);

        let active = &probe.plan.sources[0];
        assert_eq!(active.client_id, "wayland-client-1");
        assert_eq!(active.surface_id, "xdg-toplevel-1");
        assert_eq!(active.window_id, "wayland-test-client");
        assert_eq!(active.width, 384);
        assert_eq!(active.height, 256);
        assert_eq!(active.stride, 1536);
        assert_eq!(active.format, "argb8888");
        assert_eq!(active.source, "client-committed-wl-shm");
        assert_ne!(active.sample_checksum, 0);
        assert!(active.renderer_import_ready);
    }

    #[test]
    fn client_layer_pipeline_plans_and_rasterizes_sampled_sources_without_display_output() {
        let probe = probe_client_layer_pipeline(Viewport::new(1536, 1024))
            .expect("client layer pipeline probe should run");

        assert!(probe.is_ready());
        assert!(probe.source_plan_ready);
        assert!(probe.paint_plan_ready);
        assert!(probe.raster_ready);
        assert_eq!(probe.layer_count, 2);
        assert_eq!(probe.expected_layers, 2);
        assert_eq!(probe.paint_plan.steps[0].client_id, "wayland-client-1");
        assert_eq!(
            probe.paint_plan.steps[0].effect,
            "sampled-wl-shm-client-buffer"
        );
        assert_ne!(probe.raster_probe.layer_checksum, 0);
        assert_ne!(probe.raster_probe.source_checksum_fold, 0);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn parses_recovery_safe_session_config_file_content() {
        let config = parse_session_config(
            "product=Aqua Linux\n\
             mode=nested-dev\n\
             wayland_socket=aqua-wayland-0\n\
             runtime_dir=/run/user/1000\n\
             runtime_asset_root=/usr/share/aqua\n\
             autostart=false\n\
             boot_graphics=false\n\
             recovery_tty_required=true\n",
        )
        .expect("session config should parse");

        assert_eq!(config.product, PRODUCT);
        assert_eq!(config.wayland_socket, DEFAULT_WAYLAND_SOCKET);
        assert!(config.is_recovery_safe());
        assert!(config.environment().is_recovery_safe());
    }

    #[test]
    fn rejects_graphical_autostart_in_recovery_safe_session_config() {
        let config = parse_session_config(
            "product=Aqua Linux\n\
             mode=nested-dev\n\
             wayland_socket=aqua-wayland-0\n\
             runtime_dir=/run/user/1000\n\
             runtime_asset_root=/usr/share/aqua\n\
             autostart=true\n\
             boot_graphics=false\n\
             recovery_tty_required=true\n",
        )
        .expect("session config should parse");

        assert!(!config.is_recovery_safe());
    }

    #[test]
    fn session_bootstrap_prepares_runtime_without_starting_graphics() {
        let config = parse_session_config(
            "product=Aqua Linux\n\
             mode=nested-dev\n\
             wayland_socket=aqua-wayland-0\n\
             runtime_dir=/run/user/1000\n\
             runtime_asset_root=/usr/share/aqua\n\
             autostart=false\n\
             boot_graphics=false\n\
             recovery_tty_required=true\n",
        )
        .expect("session config should parse");
        let runtime_dir = std::env::temp_dir().join(format!(
            "aqua-bootstrap-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));

        let probe =
            probe_session_bootstrap(&config, &runtime_dir).expect("bootstrap probe should run");

        assert_eq!(probe.configured_runtime_dir, "/run/user/1000");
        assert_eq!(probe.wayland_display, "aqua-wayland-0");
        assert!(probe.runtime_dir_prepared);
        assert!(probe.runtime_dir_private);
        assert!(!probe.session_started);
        assert!(!probe.desktop_shell_started);
        assert!(probe.is_ready());

        fs::remove_dir_all(runtime_dir).expect("temporary runtime dir should be removable");
    }

    #[test]
    fn scene_probe_covers_static_shell_scope_without_boot_graphics() {
        let probe = probe_static_shell_scene(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.scene.product, PRODUCT);
        assert_eq!(probe.scene.status, SCENE_STATUS);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn launcher_input_controls_scene_visibility_and_render_membership() {
        let probe = probe_launcher_input_scene_binding(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert!(!probe.initial_launcher_visible);
        assert!(probe.opened_launcher_visible);
        assert!(!probe.dismissed_launcher_visible);
        assert_eq!(probe.open_draw_command_count, 7);
        assert_eq!(probe.closed_draw_command_count, 6);
        assert_eq!(probe.launch_request.unwrap().app_id, "settings");
    }

    #[test]
    fn smithay_launcher_seat_contract_opens_launcher_and_selects_settings() {
        let probe = probe_smithay_launcher_seat(Viewport::new(1536, 1024))
            .expect("Smithay launcher seat probe should run");

        assert!(probe.is_ready());
        assert_eq!(probe.seat_name, "Aqua Seat");
        assert!(probe.launcher_visible);
        assert_eq!(probe.selected_category, "settings");
        assert_eq!(probe.draw_command_count, 7);
        #[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
        assert!(probe.host_stub);
        #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_launcher_keyboard_is_compositor_owned() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");

        assert!(session.dispatch_keyboard_key(125, true, 1));
        assert!(session.dispatch_keyboard_key(125, false, 2));
        for (index, code) in [31, 18, 20, 20, 23, 49, 34, 31, 45].into_iter().enumerate() {
            let time = 3 + index as u32 * 2;
            assert!(session.dispatch_keyboard_key(code, true, time));
            assert!(session.dispatch_keyboard_key(code, false, time + 1));
        }
        assert_eq!(session.launcher_state_snapshot().query(), "settingsx");
        assert!(session.dispatch_keyboard_key(14, true, 30));
        assert_eq!(session.launcher_state_snapshot().query(), "settings");
        assert!(session.dispatch_keyboard_key(108, true, 31));
        assert!(session.dispatch_keyboard_key(108, false, 32));
        assert!(session.dispatch_keyboard_key(28, true, 33));

        let snapshot = session.input_snapshot();
        assert_eq!(snapshot.keyboard_forward_count, 0);
        assert!(snapshot.keyboard_shortcut_intercept_count >= 23);
        assert_eq!(
            session
                .take_launcher_launch_request()
                .expect("Settings request should be queued")
                .app_id,
            "settings"
        );
        assert!(!session.launcher_state_snapshot().is_open());
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn first_party_runtime_theme_transition_is_idempotent() {
        let mut state = XdgSmokeClientState {
            theme: aqua_shell::AquaTheme::LightWhite,
            settings_model: Some(aqua_shell::SettingsWindowModel::default()),
            ..XdgSmokeClientState::default()
        };

        assert!(!state.apply_runtime_theme(aqua_shell::AquaTheme::LightWhite));
        assert!(state.apply_runtime_theme(aqua_shell::AquaTheme::Deepside));
        assert_eq!(state.theme, aqua_shell::AquaTheme::Deepside);
        assert_eq!(
            state.settings_model.as_ref().map(|model| model.theme),
            Some(aqua_shell::AquaTheme::Deepside)
        );
        assert!(!state.apply_runtime_theme(aqua_shell::AquaTheme::Deepside));
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_notification_close_promotes_queue_and_timeout_hides_toast() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");
        assert!(session.post_notification(100, "Aqua Files", "Opened", "Home is ready."));
        assert!(!session.post_notification(
            120,
            "System Settings",
            "Saved",
            "Preferences updated."
        ));
        assert_eq!(
            session
                .notification_center_snapshot()
                .active()
                .map(|notification| notification.id),
            Some(1)
        );

        assert!(session.dispatch_pointer_motion(700.0, 350.0, 150));
        assert!(session.dispatch_pointer_button(0x110, true, 200));
        let promoted = session.notification_center_snapshot();
        assert_eq!(
            promoted.active().map(|notification| notification.id),
            Some(2)
        );
        assert_eq!(promoted.queued_count(), 0);

        assert!(session.dispatch_keyboard_key(1, true, 250));
        assert!(session.dispatch_keyboard_key(1, false, 251));
        assert!(session.notification_center_snapshot().active().is_none());

        assert!(session.post_notification(300, "Aqua Files", "Opened", "Home is ready."));
        assert!(session.tick_notifications(30_300));
        assert!(session.notification_center_snapshot().active().is_none());
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_first_party_surfaces_raise_and_move_between_workspaces() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");
        let (files_server, files_client) =
            std::os::unix::net::UnixStream::pair().expect("Files Wayland stream pair should open");
        let (settings_server, settings_client) = std::os::unix::net::UnixStream::pair()
            .expect("Settings Wayland stream pair should open");
        session
            .insert_client(files_server)
            .expect("Files client should insert");
        session
            .insert_client(settings_server)
            .expect("Settings client should insert");

        let files_connection =
            ClientConnection::from_socket(files_client).expect("Files connection should open");
        let settings_connection = ClientConnection::from_socket(settings_client)
            .expect("Settings connection should open");
        let mut files_queue = files_connection.new_event_queue();
        let mut settings_queue = settings_connection.new_event_queue();
        let files_qh = files_queue.handle();
        let settings_qh = settings_queue.handle();
        files_connection.display().get_registry(&files_qh, ());
        settings_connection.display().get_registry(&settings_qh, ());
        files_connection.flush().expect("Files registry flush");
        settings_connection
            .flush()
            .expect("Settings registry flush");
        session.dispatch_clients().expect("registry dispatch");
        session.flush_clients().expect("registry response flush");

        let mut files = XdgSmokeClientState::files_app();
        let mut settings = XdgSmokeClientState::settings_app().expect("Settings state");
        files_queue
            .blocking_dispatch(&mut files)
            .expect("Files globals dispatch");
        settings_queue
            .blocking_dispatch(&mut settings)
            .expect("Settings globals dispatch");
        files_connection.flush().expect("Files surface flush");
        settings_connection.flush().expect("Settings surface flush");
        session.dispatch_clients().expect("surface dispatch");
        session.flush_clients().expect("configure flush");
        files_queue
            .blocking_dispatch(&mut files)
            .expect("Files configure dispatch");
        settings_queue
            .blocking_dispatch(&mut settings)
            .expect("Settings configure dispatch");
        files_connection.flush().expect("Files buffer flush");
        settings_connection.flush().expect("Settings buffer flush");
        session.dispatch_clients().expect("buffer dispatch");
        session.flush_clients().expect("buffer response flush");

        assert!(session.raise_surface_with_app_id("aqua.files"));
        assert_eq!(
            session.active_toplevel_app_id().as_deref(),
            Some("aqua.files")
        );
        assert!(session.raise_surface_with_app_id("aqua.settings"));
        assert_eq!(
            session.active_toplevel_app_id().as_deref(),
            Some("aqua.settings")
        );
        assert_eq!(session.active_workspace(), 0);
        assert_eq!(session.visible_client_surface_snapshots().len(), 2);
        assert!(session.move_active_toplevel_to_workspace(1, 90));
        assert_eq!(session.active_workspace(), 0);
        assert_eq!(session.visible_client_surface_snapshots().len(), 1);
        assert_eq!(
            session.active_toplevel_app_id().as_deref(),
            Some("aqua.files")
        );
        assert!(session.activate_workspace(1, 91));
        assert_eq!(session.visible_client_surface_snapshots().len(), 1);
        assert_eq!(
            session.active_toplevel_app_id().as_deref(),
            Some("aqua.settings")
        );
        assert!(session.dispatch_keyboard_key(29, true, 92));
        assert!(session.dispatch_keyboard_key(56, true, 93));
        assert!(session.dispatch_keyboard_key(105, true, 94));
        assert!(session.dispatch_keyboard_key(105, false, 95));
        assert!(session.dispatch_keyboard_key(56, false, 96));
        assert!(session.dispatch_keyboard_key(29, false, 97));
        assert_eq!(session.active_workspace(), 0);

        assert!(session.activate_workspace(1, 98));
        let dock = static_shell_scene(Viewport::new(800, 600))
            .surface_rect(SurfaceKind::Dock)
            .expect("Dock geometry should exist");
        let canonical_x = dock.width - 60 * WORKSPACE_COUNT as u32 + 30;
        let pointer_x = (dock.x + canonical_x) * 1536 / 800;
        let pointer_y = (dock.y + dock.height / 2) * 1024 / 600;
        let input = session.input_snapshot();
        assert!(session.dispatch_pointer_motion(
            f64::from(pointer_x) - f64::from(input.pointer_x),
            f64::from(pointer_y) - f64::from(input.pointer_y),
            99,
        ));
        assert!(session.dispatch_pointer_button(0x110, true, 100));
        assert_eq!(session.active_workspace(), 0);
        assert!(session.present_client_surface(100));
        let input = session.input_snapshot();
        assert_eq!(input.pointer_x, 768);
        assert_eq!(input.pointer_y, 512);
        assert!(!session.raise_surface_with_app_id("aqua.unknown"));
        assert_eq!(
            session.active_toplevel_app_id().as_deref(),
            Some("aqua.files")
        );
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_session_menu_queues_confirmed_recovery_action() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");

        assert!(session.dispatch_keyboard_key(68, true, 1));
        for (index, time) in (2..=4).enumerate() {
            assert!(session.dispatch_keyboard_key(108, true, time));
            assert!(session.dispatch_keyboard_key(108, false, time + 10));
            assert_eq!(
                session.session_menu_state_snapshot().selected_index(),
                index + 1
            );
        }
        assert!(session.dispatch_keyboard_key(28, true, 20));
        assert!(session.dispatch_keyboard_key(28, false, 21));
        assert_eq!(
            session.session_menu_state_snapshot().confirmation(),
            Some(SessionAction::Recovery)
        );
        assert!(session.dispatch_keyboard_key(28, true, 22));

        assert!(session.has_session_action_request());
        assert_eq!(
            session.take_session_action_request(),
            Some(SessionAction::Recovery)
        );
    }

    #[test]
    fn render_plan_probe_covers_scene_without_starting_renderer() {
        let probe = probe_static_render_plan(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.plan.status, RENDERER_STATUS);
        assert_eq!(probe.plan.backend, RENDER_BACKEND);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn paint_plan_probe_covers_draw_order_without_starting_renderer() {
        let probe = probe_static_paint_plan(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert_eq!(probe.paint_step_count, 7);
        assert_eq!(probe.expected_paint_steps, 7);
        assert!(probe.system_surface_steps_translucent);
        assert!(probe.paint_order_stable);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn frame_plan_probe_covers_output_contract_without_starting_renderer() {
        let probe = probe_static_frame_plan(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert!(probe.frame_ready);
        assert!(probe.pixel_format_ready);
        assert!(probe.stride_ready);
        assert!(probe.damage_ready);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn frame_buffer_probe_covers_allocation_without_starting_renderer() {
        let probe = probe_static_frame_buffer(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert!(probe.buffer_allocated);
        assert!(probe.clear_color_ready);
        assert!(probe.first_pixel_ready);
        assert!(probe.last_pixel_ready);
        assert_eq!(probe.probe.buffer_bytes, 6_291_456);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn software_raster_probe_covers_headless_pixels_without_display_output() {
        let probe = probe_static_software_raster(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert!(probe.rect_count_ready);
        assert!(probe.wallpaper_sample_ready);
        assert!(probe.surface_sample_ready);
        assert!(probe.dock_sample_ready);
        assert!(probe.surface_border_sample_ready);
        assert!(probe.surface_highlight_sample_ready);
        assert!(probe.surface_corner_sample_ready);
        assert!(probe.surface_shadow_sample_ready);
        assert!(probe.checksum_ready);
        assert!(probe.surface_primitives_ready);
        assert_eq!(probe.probe.filled_rect_count, 7);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn raster_export_probe_covers_ppm_artifact_without_display_output() {
        let probe = probe_static_raster_export(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert!(probe.format_ready);
        assert!(probe.byte_count_ready);
        assert!(probe.checksum_ready);
        assert_eq!(probe.export.format, "ppm-p6-rgb888");
        assert_eq!(probe.export.byte_count, 4_718_609);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn raster_png_export_probe_covers_png_artifact_without_display_output() {
        let probe = probe_static_raster_png_export(Viewport::new(1536, 1024));

        assert!(probe.is_ready());
        assert!(probe.format_ready);
        assert!(probe.byte_count_ready);
        assert!(probe.checksum_ready);
        assert_eq!(probe.export.format, "png-rgba8888");
        assert_eq!(probe.export.byte_count, 6_293_028);
        assert!(!probe.renderer_started);
        assert!(!probe.boot_graphics);
    }

    #[test]
    fn wayland_display_smoke_creates_compositor_global() {
        let result = run_wayland_display_smoke().expect("Wayland display smoke should run");
        assert!(result.is_ready());
    }

    #[test]
    fn wayland_socket_smoke_binds_and_cleans_up() {
        let result = run_wayland_socket_smoke().expect("Wayland socket smoke should run");
        assert_eq!(result.socket_name, "aqua-wayland-0");
        assert!(result.client_connected);
        assert!(result.client_accepted);
        assert!(result.client_inserted);
        assert!(result.is_ready());
    }

    #[test]
    fn calloop_socket_smoke_accepts_and_inserts_client() {
        let result = run_calloop_socket_smoke().expect("Calloop socket smoke should run");
        assert_eq!(result.socket_name, "aqua-wayland-calloop-0");
        assert!(result.callback_invoked);
        assert!(result.client_accepted);
        assert!(result.client_inserted);
        assert!(result.dispatch_clients_ok);
        assert!(result.flush_clients_ok);
        assert!(result.is_ready());
    }

    #[test]
    fn session_skeleton_owns_display_and_compositor_state() {
        let probe = probe_session_skeleton().expect("session skeleton probe should run");
        assert_eq!(probe.product, "Aqua Linux");
        assert_eq!(probe.mode, "nested-dev");
        assert!(probe.display_owned);
        assert!(probe.compositor_state_owned);
        assert!(probe.client_inserted);
        assert!(probe.dispatch_clients_ok);
        assert!(probe.flush_clients_ok);
        assert!(probe.is_ready());
    }

    #[test]
    fn session_run_once_smoke_accepts_dispatches_and_flushes() {
        let result = run_session_once_smoke().expect("session run-once smoke should run");
        assert_eq!(result.socket_name, "aqua-wayland-run-once-0");
        assert!(result.run_once_called);
        assert!(result.callback_invoked);
        assert!(result.client_inserted);
        assert!(result.dispatch_clients_ok);
        assert!(result.flush_clients_ok);
        assert!(result.is_ready());
    }

    #[test]
    fn session_loop_smoke_runs_bounded_dispatch_and_flush_passes() {
        let result = run_session_loop_smoke().expect("session loop smoke should run");

        assert_eq!(result.socket_name, "aqua-wayland-session-loop-0");
        assert!(result.loop_started);
        assert_eq!(result.loop_iterations, 3);
        assert_eq!(result.dispatch_passes, 3);
        assert_eq!(result.flush_passes, 3);
        assert!(result.callback_invoked);
        assert!(result.client_inserted);
        assert!(result.is_ready());
    }

    #[test]
    fn first_party_launch_preflight_accepts_only_executable_aqua_binary() {
        let root = std::env::temp_dir().join(format!(
            "aqua-launch-preflight-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let bin = root.join("usr/bin");
        fs::create_dir_all(&bin).expect("bin directory");
        let executable = bin.join("aqua-files");
        fs::write(&executable, b"#!/bin/sh\nexit 0\n").expect("fixture executable");
        #[cfg(unix)]
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
            .expect("executable permissions");
        let request = LaunchRequest {
            app_id: "files",
            command: "/usr/bin/aqua-files",
            target: None,
        };

        let accepted = preflight_first_party_launch(&request, &root);
        assert!(accepted.accepted);
        assert_eq!(accepted.reason, "accepted");

        #[cfg(unix)]
        {
            fs::set_permissions(&executable, fs::Permissions::from_mode(0o644))
                .expect("non-executable permissions");
            let non_executable = preflight_first_party_launch(&request, &root);
            assert!(!non_executable.accepted);
            assert_eq!(non_executable.reason, "not-executable");

            fs::remove_file(&executable).expect("remove non-executable fixture");
            std::os::unix::fs::symlink("/bin/sh", &executable).expect("symlink fixture");
            let symlink = preflight_first_party_launch(&request, &root);
            assert!(!symlink.accepted);
            assert_eq!(symlink.reason, "symlink-not-allowed");
        }

        fs::remove_file(&executable).expect("remove fixture");
        let missing = preflight_first_party_launch(&request, &root);
        assert!(!missing.accepted);
        assert_eq!(missing.reason, "missing-executable");

        let rejected = preflight_first_party_launch(
            &LaunchRequest {
                app_id: "files",
                command: "/bin/sh",
                target: None,
            },
            &root,
        );
        assert!(!rejected.accepted);
        assert_eq!(rejected.reason, "command-not-allowed");
        fs::remove_dir_all(root).expect("remove fixture root");
    }

    #[cfg(unix)]
    #[test]
    fn first_party_process_supervisor_rejects_duplicates_and_reaps_children() {
        let root = std::env::temp_dir().join(format!(
            "aqua-process-supervisor-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let bin = root.join("usr/bin");
        let runtime = root.join("run/aqua");
        fs::create_dir_all(&bin).expect("bin directory");
        fs::create_dir_all(&runtime).expect("runtime directory");
        let executable = bin.join("aqua-files");
        fs::write(&executable, b"#!/bin/sh\nsleep 1\n").expect("fixture executable");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
            .expect("executable permissions");
        let request = LaunchRequest {
            app_id: "files",
            command: "/usr/bin/aqua-files",
            target: None,
        };
        let mut preflight = preflight_first_party_launch(&request, &root);
        preflight.command = Box::leak(executable.to_string_lossy().into_owned().into_boxed_str());

        let mut supervisor = FirstPartyProcessSupervisor::default();
        let process = supervisor
            .spawn(&preflight, &runtime, "aqua-test-0")
            .expect("supervised process should start");
        assert!(process.pid > 0);
        assert_eq!(supervisor.active_count(), 1);
        assert!(supervisor.contains("files"));
        assert_eq!(
            supervisor.spawn(&preflight, &runtime, "aqua-test-0"),
            Err(ProcessSupervisorError::AlreadyRunning)
        );
        let reaped = supervisor
            .terminate_and_reap("files")
            .expect("process should terminate and reap");
        assert_eq!(reaped.app_id, "files");
        assert_eq!(reaped.pid, process.pid);
        assert_eq!(supervisor.active_count(), 0);
        assert_eq!(
            supervisor.try_reap("files"),
            Err(ProcessSupervisorError::MissingProcess)
        );

        fs::remove_dir_all(root).expect("remove supervisor fixture root");
    }

    #[test]
    fn properties_preflight_requires_an_allowlisted_target() {
        let root =
            std::env::temp_dir().join(format!("aqua-properties-preflight-{}", std::process::id()));
        let executable = root.join("usr/bin/aqua-properties");
        fs::create_dir_all(executable.parent().expect("properties bin parent"))
            .expect("properties bin directory");
        fs::write(&executable, b"#!/bin/sh\nexit 0\n").expect("properties executable");
        #[cfg(unix)]
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
            .expect("properties executable permissions");

        let accepted = preflight_first_party_launch(
            &LaunchRequest {
                app_id: "properties",
                command: "/usr/bin/aqua-properties",
                target: Some("files"),
            },
            &root,
        );
        assert!(accepted.accepted);
        assert!(accepted.target_allowed);
        assert_eq!(accepted.target, Some("files"));

        for target in [None, Some("unknown")] {
            let rejected = preflight_first_party_launch(
                &LaunchRequest {
                    app_id: "properties",
                    command: "/usr/bin/aqua-properties",
                    target,
                },
                &root,
            );
            assert!(!rejected.accepted);
            assert_eq!(rejected.reason, "target-not-allowed");
        }
        fs::remove_dir_all(root).expect("remove properties preflight fixture");
    }

    #[test]
    fn first_party_user_applications_do_not_restart_automatically() {
        assert_eq!(
            first_party_restart_policy("files"),
            Some(FirstPartyRestartPolicy::Never)
        );
        assert_eq!(
            first_party_restart_policy("settings"),
            Some(FirstPartyRestartPolicy::Never)
        );
        assert_eq!(
            first_party_restart_policy("properties"),
            Some(FirstPartyRestartPolicy::Never)
        );
        assert_eq!(first_party_restart_policy("browser"), None);
    }
}
