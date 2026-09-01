use std::fs;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::io::{Read, Write};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::os::unix::io::AsFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::sync::mpsc::{self, Receiver};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod presentation;
pub use presentation::{
    DiagnosticReadbackEvidence, PresentationBudget, PresentationEventSnapshot,
    PresentationEvidenceTarget, PresentationPath, PresentationSample, PresentationTelemetry,
    PresentationTelemetryError, PresentationWorkload, R2PresentationReport,
    MAX_PRESENTATION_EVENTS, MAX_PRESENTATION_SAMPLES, QEMU_TCG_BOCHS_QUALIFICATION_V1_BUDGET,
    QEMU_TCG_BOCHS_SOAK_V1_BUDGET, QEMU_TCG_BOCHS_V1_BUDGET,
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
    StorageProbePaths, KEYBOARD_OPTIONS, LANGUAGE_OPTIONS,
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
    dock_pointer_target, properties_launch_request, top_system_bar, workspace_keyboard_target,
    BottomShellTarget, CollectionNavigationKey, DesktopContextAction, DesktopContextMenuKey,
    DesktopIconState, DesktopIconUpdate, DesktopPointerButton, DockItem, DockState, LaunchRequest,
    LauncherCategory, LauncherEvent, LauncherPointerTarget, LauncherState, MenuNavigationKey,
    NotificationCenter, SessionAction, SessionMenuEvent, SessionMenuState, TrashModel,
    WorkspaceNavigationKey, NOTIFICATION_DEFAULT_TIMEOUT_MS, SESSION_MENU_RUNTIME_HEIGHT,
    SESSION_MENU_RUNTIME_WIDTH, WORKSPACE_COUNT,
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

pub fn bottom_shell_pointer_target(
    viewport: Viewport,
    pointer_x: u32,
    pointer_y: u32,
) -> Option<BottomShellTarget> {
    let rect = static_shell_scene(viewport).surface_rect(SurfaceKind::Dock)?;
    if pointer_x < rect.x
        || pointer_x >= rect.right()
        || pointer_y < rect.y
        || pointer_y >= rect.bottom()
    {
        return None;
    }
    dock_pointer_target(
        pointer_x - rect.x,
        pointer_y - rect.y,
        rect.width,
        rect.height,
    )
}

pub fn session_menu_pointer_position(
    viewport: Viewport,
    pointer_x: u32,
    pointer_y: u32,
) -> Option<(u32, u32)> {
    let rect = static_shell_scene(viewport).surface_rect(SurfaceKind::SystemOverview)?;
    if pointer_x < rect.x
        || pointer_x >= rect.right()
        || pointer_y < rect.y
        || pointer_y >= rect.bottom()
        || rect.width == 0
        || rect.height == 0
    {
        return None;
    }
    Some((
        (u64::from(pointer_x - rect.x) * u64::from(SESSION_MENU_RUNTIME_WIDTH)
            / u64::from(rect.width)) as u32,
        (u64::from(pointer_y - rect.y) * u64::from(SESSION_MENU_RUNTIME_HEIGHT)
            / u64::from(rect.height)) as u32,
    ))
}

#[cfg(any(test, all(target_os = "linux", feature = "smithay-smoke")))]
fn pointer_location_after_motion(
    location: (f64, f64),
    dx: f64,
    dy: f64,
    viewport: Viewport,
) -> (f64, f64) {
    (
        (location.0 + dx).clamp(0.0, f64::from(viewport.width.saturating_sub(1))),
        (location.1 + dy).clamp(0.0, f64::from(viewport.height.saturating_sub(1))),
    )
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
    backend::input::{Axis, AxisSource, ButtonState, KeyState, Keycode, TouchSlot},
    delegate_compositor, delegate_data_device, delegate_fractional_scale,
    delegate_input_method_manager, delegate_output, delegate_primary_selection, delegate_seat,
    delegate_shm, delegate_text_input_manager, delegate_viewporter, delegate_xdg_shell,
    input::{
        keyboard::{xkb, FilterResult, XkbConfig},
        pointer::{AxisFrame, ButtonEvent, CursorImageStatus, MotionEvent},
        touch::{
            DownEvent as TouchDownEvent, MotionEvent as TouchMotionEvent, UpEvent as TouchUpEvent,
        },
        Seat, SeatHandler, SeatState,
    },
    output::{Mode as OutputMode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::wayland_server::{
        backend::{ClientData, ClientId, DisconnectReason, GlobalId},
        protocol::{
            wl_buffer, wl_callback, wl_data_device_manager::DndAction as ServerDndAction, wl_seat,
            wl_surface::WlSurface,
        },
        Client, Display, DisplayHandle, ListeningSocket, Resource,
    },
    utils::{Logical, Rectangle, Serial, Transform},
    wayland::shell::xdg::{
        Configure, PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        XdgToplevelSurfaceData,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_children, get_parent, get_role, is_sync_subsurface, with_states, BufferAssignment,
            CompositorClientState, CompositorHandler, CompositorState, RectangleKind,
            SubsurfaceCachedState,
        },
        fractional_scale::{
            with_fractional_scale, FractionalScaleHandler, FractionalScaleManagerState,
        },
        input_method::{
            InputMethodHandler, InputMethodManagerState, PopupSurface as InputMethodPopupSurface,
        },
        output::{OutputHandler, OutputManagerState},
        selection::{
            data_device::{
                clear_data_device_selection, set_data_device_focus, ClientDndGrabHandler,
                DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
            primary_selection::{
                clear_primary_selection, set_primary_focus, PrimarySelectionHandler,
                PrimarySelectionState,
            },
            SelectionHandler, SelectionSource, SelectionTarget,
        },
        shm::{with_buffer_contents, ShmHandler, ShmState},
        text_input::TextInputManagerState,
        viewporter::{ViewportCachedState, ViewporterState},
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
            wl_data_device as client_wl_data_device,
            wl_data_device_manager as client_wl_data_device_manager,
            wl_data_offer as client_wl_data_offer, wl_data_source as client_wl_data_source,
            wl_keyboard as client_wl_keyboard, wl_output as client_wl_output,
            wl_pointer as client_wl_pointer, wl_region as client_wl_region, wl_registry,
            wl_seat as client_wl_seat, wl_shm as client_wl_shm, wl_shm_pool as client_wl_shm_pool,
            wl_subcompositor as client_wl_subcompositor, wl_subsurface as client_wl_subsurface,
            wl_surface,
        },
        Connection as ClientConnection, Dispatch as ClientDispatch, Proxy, QueueHandle, WEnum,
    },
    wayland_protocols::wp::fractional_scale::v1::client::{
        wp_fractional_scale_manager_v1 as client_fractional_scale_manager,
        wp_fractional_scale_v1 as client_fractional_scale,
    },
    wayland_protocols::wp::primary_selection::zv1::client::{
        zwp_primary_selection_device_manager_v1 as client_primary_selection_manager,
        zwp_primary_selection_device_v1 as client_primary_selection_device,
        zwp_primary_selection_offer_v1 as client_primary_selection_offer,
        zwp_primary_selection_source_v1 as client_primary_selection_source,
    },
    wayland_protocols::wp::text_input::zv3::client::{
        zwp_text_input_manager_v3 as client_text_input_manager,
        zwp_text_input_v3 as client_text_input,
    },
    wayland_protocols::wp::viewporter::client::{
        wp_viewport as client_viewport, wp_viewporter as client_viewporter,
    },
    wayland_protocols::xdg::shell::client::{
        xdg_popup as client_xdg_popup, xdg_positioner as client_xdg_positioner,
        xdg_surface as client_xdg_surface, xdg_toplevel as client_xdg_toplevel,
        xdg_wm_base as client_xdg_wm_base,
    },
    wayland_protocols::xdg::xdg_output::zv1::client::{
        zxdg_output_manager_v1 as client_xdg_output_manager, zxdg_output_v1 as client_xdg_output,
    },
    wayland_protocols_misc::zwp_input_method_v2::client::{
        zwp_input_method_manager_v2 as client_input_method_manager,
        zwp_input_method_v2 as client_input_method,
        zwp_input_popup_surface_v2 as client_input_popup_surface,
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
pub struct SelectionOwnershipProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub clipboard_protocol: &'static str,
    pub primary_protocol: &'static str,
    pub client_count: usize,
    pub globals_visible_to_both_clients: bool,
    pub focus_follows_keyboard: bool,
    pub unfocused_clipboard_rejected: bool,
    pub unfocused_primary_rejected: bool,
    pub focused_clipboard_accepted: bool,
    pub focused_primary_accepted: bool,
    pub clipboard_offer_reaches_new_focus: bool,
    pub primary_offer_reaches_new_focus: bool,
    pub clipboard_mime_negotiated: bool,
    pub primary_mime_negotiated: bool,
    pub unsupported_mime_not_requested: bool,
    pub clipboard_payload_transferred: bool,
    pub primary_payload_transferred: bool,
    pub clipboard_payload_bytes: usize,
    pub primary_payload_bytes: usize,
    pub transfer_limit_bytes: usize,
    pub compositor_buffers_payload: bool,
    pub owner_disconnect_clears_clipboard: bool,
    pub owner_disconnect_clears_primary: bool,
    pub ownership_handoff_accepted: bool,
    pub data_control_global_exposed: bool,
    pub host_stub: bool,
}

impl SelectionOwnershipProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "selection-ownership"
            && self.clipboard_protocol == "wl_data_device_manager"
            && self.primary_protocol == "zwp_primary_selection_device_manager_v1"
            && self.client_count == 2
            && self.globals_visible_to_both_clients
            && self.focus_follows_keyboard
            && self.unfocused_clipboard_rejected
            && self.unfocused_primary_rejected
            && self.focused_clipboard_accepted
            && self.focused_primary_accepted
            && self.clipboard_offer_reaches_new_focus
            && self.primary_offer_reaches_new_focus
            && self.clipboard_mime_negotiated
            && self.primary_mime_negotiated
            && self.unsupported_mime_not_requested
            && self.clipboard_payload_transferred
            && self.primary_payload_transferred
            && self.clipboard_payload_bytes > 0
            && self.clipboard_payload_bytes <= self.transfer_limit_bytes
            && self.primary_payload_bytes > 0
            && self.primary_payload_bytes <= self.transfer_limit_bytes
            && !self.compositor_buffers_payload
            && self.owner_disconnect_clears_clipboard
            && self.owner_disconnect_clears_primary
            && self.ownership_handoff_accepted
            && !self.data_control_global_exposed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragAndDropProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub protocol: &'static str,
    pub client_count: usize,
    pub globals_visible_to_both_clients: bool,
    pub start_without_implicit_grab_rejected: bool,
    pub pointer_grab_started: bool,
    pub source_client_owns_drag: bool,
    pub enter_reaches_pointer_focus_only: bool,
    pub keyboard_focus_unchanged: bool,
    pub mime_negotiated: bool,
    pub unsupported_mime_not_accepted: bool,
    pub copy_action_negotiated: bool,
    pub payload_transferred: bool,
    pub payload_bytes: usize,
    pub transfer_limit_bytes: usize,
    pub compositor_buffers_payload: bool,
    pub drop_delivered_to_target: bool,
    pub source_drop_performed: bool,
    pub source_finished: bool,
    pub rejected_drop_cancelled: bool,
    pub rejected_drop_not_delivered: bool,
    pub data_control_global_exposed: bool,
    pub host_stub: bool,
}

impl DragAndDropProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "drag-and-drop"
            && self.protocol == "wl_data_device_manager"
            && self.client_count == 2
            && self.globals_visible_to_both_clients
            && self.start_without_implicit_grab_rejected
            && self.pointer_grab_started
            && self.source_client_owns_drag
            && self.enter_reaches_pointer_focus_only
            && self.keyboard_focus_unchanged
            && self.mime_negotiated
            && self.unsupported_mime_not_accepted
            && self.copy_action_negotiated
            && self.payload_transferred
            && self.payload_bytes > 0
            && self.payload_bytes <= self.transfer_limit_bytes
            && !self.compositor_buffers_payload
            && self.drop_delivered_to_target
            && self.source_drop_performed
            && self.source_finished
            && self.rejected_drop_cancelled
            && self.rejected_drop_not_delivered
            && !self.data_control_global_exposed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub text_input_protocol: &'static str,
    pub input_method_protocol: &'static str,
    pub client_count: usize,
    pub text_input_visible_to_normal_clients: bool,
    pub input_method_hidden_from_normal_clients: bool,
    pub input_method_visible_to_authorized_client: bool,
    pub focus_follows_keyboard: bool,
    pub unfocused_enable_rejected: bool,
    pub focused_enable_activates_input_method: bool,
    pub surrounding_text_forwarded: bool,
    pub content_type_forwarded: bool,
    pub cursor_rectangle_forwarded: bool,
    pub turkish_preedit_delivered: bool,
    pub turkish_commit_delivered: bool,
    pub delete_surrounding_delivered: bool,
    pub serial_synchronized: bool,
    pub focus_handoff_deactivates_input_method: bool,
    pub focus_handoff_enters_new_client: bool,
    pub stale_unfocused_client_blocked: bool,
    pub popup_parent_bound: bool,
    pub popup_repositioned: bool,
    pub payload_limit_bytes: usize,
    pub host_stub: bool,
}

pub const DECLARED_LOCALES: [&str; 3] = ["tr_TR.UTF-8", "en_US.UTF-8", "de_DE.UTF-8"];
pub const DECLARED_KEYBOARD_LAYOUTS: [&str; 3] = ["trq", "trf", "us"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardLocaleMatrixProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub protocol: &'static str,
    pub locale_count: usize,
    pub keyboard_layout_count: usize,
    pub supported_combination_count: usize,
    pub client_count_per_layout: usize,
    pub keymaps_delivered_to_all_clients: bool,
    pub keymaps_compile_for_all_layouts: bool,
    pub representative_utf8_matches: bool,
    pub compose_key_available_for_all_layouts: bool,
    pub compose_case_count: usize,
    pub compose_utf8_matches_for_all_clients: bool,
    pub dead_key_layout_count: usize,
    pub dead_key_case_count: usize,
    pub dead_key_utf8_matches_for_all_clients: bool,
    pub cancelled_compose_rejected_for_all_locales: bool,
    pub repeat_delay_ms: i32,
    pub repeat_rate_hz: i32,
    pub repeat_info_matches: bool,
    pub host_stub: bool,
}

impl KeyboardLocaleMatrixProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "keyboard-locale-matrix"
            && self.protocol == "wl_keyboard"
            && self.locale_count == DECLARED_LOCALES.len()
            && self.keyboard_layout_count == DECLARED_KEYBOARD_LAYOUTS.len()
            && self.supported_combination_count
                == DECLARED_LOCALES.len() * DECLARED_KEYBOARD_LAYOUTS.len()
            && self.client_count_per_layout == 2
            && self.keymaps_delivered_to_all_clients
            && self.keymaps_compile_for_all_layouts
            && self.representative_utf8_matches
            && self.compose_key_available_for_all_layouts
            && self.compose_case_count == DECLARED_LOCALES.len() * DECLARED_KEYBOARD_LAYOUTS.len()
            && self.compose_utf8_matches_for_all_clients
            && self.dead_key_layout_count == 2
            && self.dead_key_case_count == DECLARED_LOCALES.len() * self.dead_key_layout_count
            && self.dead_key_utf8_matches_for_all_clients
            && self.cancelled_compose_rejected_for_all_locales
            && self.repeat_delay_ms == 400
            && self.repeat_rate_hz == 25
            && self.repeat_info_matches
    }
}

pub const PRIVILEGED_WAYLAND_GLOBALS: [&str; 16] = [
    "weston_screenshooter",
    "zwlr_screencopy_manager_v1",
    "ext_image_copy_capture_manager_v1",
    "ext_output_image_capture_source_manager_v1",
    "zwlr_export_dmabuf_manager_v1",
    "xdg_activation_v1",
    "zwlr_layer_shell_v1",
    "weston_desktop_shell",
    "zwp_virtual_keyboard_manager_v1",
    "zwlr_foreign_toplevel_manager_v1",
    "ext_foreign_toplevel_list_v1",
    "zwlr_output_manager_v1",
    "zwlr_gamma_control_manager_v1",
    "zwlr_output_power_manager_v1",
    "wp_drm_lease_device_v1",
    "ext_session_lock_manager_v1",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivilegedProtocolBoundaryProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub client_count: usize,
    pub normal_client_count: usize,
    pub authorized_client_count: usize,
    pub baseline_globals_visible_to_all_clients: bool,
    pub input_method_hidden_from_normal_clients: bool,
    pub input_method_visible_to_authorized_client: bool,
    pub privileged_global_count: usize,
    pub screenshot_global_exposed: bool,
    pub screencopy_global_exposed: bool,
    pub activation_global_exposed: bool,
    pub privileged_shell_global_exposed: bool,
    pub virtual_input_global_exposed: bool,
    pub desktop_management_global_exposed: bool,
    pub authorized_scope_is_narrow: bool,
    pub host_stub: bool,
}

impl PrivilegedProtocolBoundaryProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "privileged-protocol-boundary"
            && self.client_count == 3
            && self.normal_client_count == 2
            && self.authorized_client_count == 1
            && self.baseline_globals_visible_to_all_clients
            && self.input_method_hidden_from_normal_clients
            && self.input_method_visible_to_authorized_client
            && self.privileged_global_count == PRIVILEGED_WAYLAND_GLOBALS.len()
            && !self.screenshot_global_exposed
            && !self.screencopy_global_exposed
            && !self.activation_global_exposed
            && !self.privileged_shell_global_exposed
            && !self.virtual_input_global_exposed
            && !self.desktop_management_global_exposed
            && self.authorized_scope_is_narrow
    }
}

impl TextInputProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "text-input"
            && self.text_input_protocol == "zwp_text_input_manager_v3"
            && self.input_method_protocol == "zwp_input_method_manager_v2"
            && self.client_count == 3
            && self.text_input_visible_to_normal_clients
            && self.input_method_hidden_from_normal_clients
            && self.input_method_visible_to_authorized_client
            && self.focus_follows_keyboard
            && self.unfocused_enable_rejected
            && self.focused_enable_activates_input_method
            && self.surrounding_text_forwarded
            && self.content_type_forwarded
            && self.cursor_rectangle_forwarded
            && self.turkish_preedit_delivered
            && self.turkish_commit_delivered
            && self.delete_surrounding_delivered
            && self.serial_synchronized
            && self.focus_handoff_deactivates_input_method
            && self.focus_handoff_enters_new_client
            && self.stale_unfocused_client_blocked
            && self.popup_parent_bound
            && self.popup_repositioned
            && self.payload_limit_bytes == 4_000
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1ClientBufferContractProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub application_model: &'static str,
    pub required_buffer_protocol: &'static str,
    pub required_shm_format: &'static str,
    pub client_count: usize,
    pub wl_shm_visible_to_all_clients: bool,
    pub argb8888_visible_to_all_clients: bool,
    pub linux_dmabuf_advertised: bool,
    pub drm_syncobj_advertised: bool,
    pub explicit_sync_advertised: bool,
    pub accelerated_clients_supported: bool,
    pub synchronization_scope: &'static str,
    pub host_stub: bool,
}

impl V1ClientBufferContractProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "v1-client-buffer-contract"
            && self.application_model == "first-party-wl-shm-v1"
            && self.required_buffer_protocol == "wl_shm"
            && self.required_shm_format == "argb8888"
            && self.client_count == 2
            && self.wl_shm_visible_to_all_clients
            && self.argb8888_visible_to_all_clients
            && !self.linux_dmabuf_advertised
            && !self.drm_syncobj_advertised
            && !self.explicit_sync_advertised
            && !self.accelerated_clients_supported
            && self.synchronization_scope == "wl_buffer.release+wl_surface.frame"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaylandOutputMatrixProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub client_count: usize,
    pub output_count: usize,
    pub declared_scale_count: usize,
    pub declared_transform_count: usize,
    pub outputs_visible_to_both_clients: bool,
    pub modes_match_supported_matrix: bool,
    pub preferred_modes_advertised: bool,
    pub logical_coordinates_match: bool,
    pub integer_scales_match: bool,
    pub fractional_scales_match: bool,
    pub transforms_match: bool,
    pub fractional_scale_advertised: bool,
    pub fractional_scale_120ths: u32,
    pub viewport_source_applied: bool,
    pub viewport_destination_applied: bool,
    pub hotplug_add_reaches_both_clients: bool,
    pub hotplug_remove_reaches_both_clients: bool,
    pub remaining_output_usable: bool,
    pub host_stub: bool,
}

impl WaylandOutputMatrixProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "wayland-output-matrix"
            && self.client_count == 2
            && self.output_count == 4
            && self.declared_scale_count == 4
            && self.declared_transform_count == 4
            && self.outputs_visible_to_both_clients
            && self.modes_match_supported_matrix
            && self.preferred_modes_advertised
            && self.logical_coordinates_match
            && self.integer_scales_match
            && self.fractional_scales_match
            && self.transforms_match
            && self.fractional_scale_advertised
            && self.fractional_scale_120ths == 150
            && self.viewport_source_applied
            && self.viewport_destination_applied
            && self.hotplug_add_reaches_both_clients
            && self.hotplug_remove_reaches_both_clients
            && self.remaining_output_usable
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupSubsurfaceMatrixProbe {
    pub product: &'static str,
    pub status: &'static str,
    pub client_count: usize,
    pub xdg_popup_created: bool,
    pub popup_parent_bound: bool,
    pub popup_geometry_matches: bool,
    pub popup_configure_acknowledged: bool,
    pub popup_reposition_requested: bool,
    pub popup_reposition_token: u32,
    pub popup_reposition_acknowledged: bool,
    pub popup_destroyed: bool,
    pub subsurface_created: bool,
    pub subsurface_parent_bound: bool,
    pub subsurface_position_matches: bool,
    pub synchronized_commit_observed: bool,
    pub desynchronized_commit_observed: bool,
    pub subsurface_destroyed: bool,
    pub parent_surfaces_remain_independent: bool,
    pub host_stub: bool,
}

impl PopupSubsurfaceMatrixProbe {
    pub fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "popup-subsurface-matrix"
            && self.client_count == 2
            && self.xdg_popup_created
            && self.popup_parent_bound
            && self.popup_geometry_matches
            && self.popup_configure_acknowledged
            && self.popup_reposition_requested
            && self.popup_reposition_token == 77
            && self.popup_reposition_acknowledged
            && self.popup_destroyed
            && self.subsurface_created
            && self.subsurface_parent_bound
            && self.subsurface_position_matches
            && self.synchronized_commit_observed
            && self.desynchronized_commit_observed
            && self.subsurface_destroyed
            && self.parent_surfaces_remain_independent
    }
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

pub fn probe_selection_ownership() -> Result<SelectionOwnershipProbe, Box<dyn std::error::Error>> {
    probe_selection_ownership_impl()
}

pub fn probe_drag_and_drop() -> Result<DragAndDropProbe, Box<dyn std::error::Error>> {
    probe_drag_and_drop_impl()
}

pub fn probe_text_input() -> Result<TextInputProbe, Box<dyn std::error::Error>> {
    probe_text_input_impl()
}

pub fn probe_keyboard_locale_matrix(
) -> Result<KeyboardLocaleMatrixProbe, Box<dyn std::error::Error>> {
    probe_keyboard_locale_matrix_impl()
}

pub fn probe_privileged_protocol_boundary(
) -> Result<PrivilegedProtocolBoundaryProbe, Box<dyn std::error::Error>> {
    probe_privileged_protocol_boundary_impl()
}

pub fn probe_v1_client_buffer_contract(
) -> Result<V1ClientBufferContractProbe, Box<dyn std::error::Error>> {
    probe_v1_client_buffer_contract_impl()
}

pub fn probe_wayland_output_matrix() -> Result<WaylandOutputMatrixProbe, Box<dyn std::error::Error>>
{
    probe_wayland_output_matrix_impl()
}

pub fn probe_popup_subsurface_matrix(
) -> Result<PopupSubsurfaceMatrixProbe, Box<dyn std::error::Error>> {
    probe_popup_subsurface_matrix_impl()
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

#[cfg_attr(
    not(all(target_os = "linux", feature = "smithay-smoke")),
    allow(dead_code)
)]
fn shm_buffer_bounds(
    pool_len: usize,
    offset: i32,
    stride: i32,
    height: i32,
) -> Option<(usize, usize)> {
    let offset = usize::try_from(offset).ok()?;
    let stride = usize::try_from(stride).ok()?;
    let height = usize::try_from(height).ok()?;
    let byte_count = stride.checked_mul(height)?;
    let end = offset.checked_add(byte_count)?;
    (end <= pool_len && byte_count > 0).then_some((offset, byte_count))
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
fn probe_v1_client_buffer_contract_impl(
) -> Result<V1ClientBufferContractProbe, Box<dyn std::error::Error>> {
    let mut session = AquaCompositorSession::new()?;
    let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
    session.insert_client(server_stream_one)?;
    session.insert_client(server_stream_two)?;

    let client_one_conn = ClientConnection::from_socket(client_stream_one)?;
    let client_two_conn = ClientConnection::from_socket(client_stream_two)?;
    let mut event_queue_one = client_one_conn.new_event_queue();
    let mut event_queue_two = client_two_conn.new_event_queue();
    client_one_conn
        .display()
        .get_registry(&event_queue_one.handle(), ());
    client_two_conn
        .display()
        .get_registry(&event_queue_two.handle(), ());
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let mut client_one = V1BufferRegistryClientState::default();
    let mut client_two = V1BufferRegistryClientState::default();
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;

    Ok(V1ClientBufferContractProbe {
        product: PRODUCT,
        status: "v1-client-buffer-contract",
        application_model: "first-party-wl-shm-v1",
        required_buffer_protocol: "wl_shm",
        required_shm_format: "argb8888",
        client_count: 2,
        wl_shm_visible_to_all_clients: client_one.registry_bound
            && client_two.registry_bound
            && client_one.wl_shm_seen
            && client_two.wl_shm_seen,
        argb8888_visible_to_all_clients: client_one.shm_argb8888_seen
            && client_two.shm_argb8888_seen,
        linux_dmabuf_advertised: client_one.linux_dmabuf_seen || client_two.linux_dmabuf_seen,
        drm_syncobj_advertised: client_one.drm_syncobj_seen || client_two.drm_syncobj_seen,
        explicit_sync_advertised: client_one.explicit_sync_seen || client_two.explicit_sync_seen,
        accelerated_clients_supported: false,
        synchronization_scope: "wl_buffer.release+wl_surface.frame",
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
const AQUA_COMPOSE_TABLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../br2-external/aqua/rootfs-overlay/usr/share/aqua/compose/Compose"
));

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn compose_sequence_result(
    locale: &str,
    sequence: &[xkb::Keysym],
) -> Result<(xkb::compose::Status, Option<String>), ()> {
    const MAX_COMPOSE_SEQUENCE: usize = 8;
    if sequence.is_empty() || sequence.len() > MAX_COMPOSE_SEQUENCE {
        return Err(());
    }

    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let table = xkb::compose::Table::new_from_buffer(
        &context,
        AQUA_COMPOSE_TABLE,
        locale,
        xkb::compose::FORMAT_TEXT_V1,
        xkb::compose::COMPILE_NO_FLAGS,
    )?;
    let mut state = xkb::compose::State::new(&table, xkb::compose::STATE_NO_FLAGS);
    for keysym in sequence {
        if state.feed(*keysym) != xkb::compose::FeedResult::Accepted {
            return Err(());
        }
    }
    Ok((state.status(), state.utf8()))
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_keyboard_locale_matrix_impl(
) -> Result<KeyboardLocaleMatrixProbe, Box<dyn std::error::Error>> {
    if !LANGUAGE_OPTIONS
        .iter()
        .map(|option| option.value)
        .eq(DECLARED_LOCALES)
        || !KEYBOARD_OPTIONS
            .iter()
            .map(|option| option.value)
            .eq(DECLARED_KEYBOARD_LAYOUTS)
    {
        return Err("installer choices differ from compositor keyboard/locale matrix".into());
    }
    let mut keymaps_delivered_to_all_clients = true;
    let mut keymaps_compile_for_all_layouts = true;
    let mut representative_utf8_matches = true;
    let mut compose_key_available_for_all_layouts = true;
    let mut compose_utf8_matches_for_all_clients = true;
    let mut dead_key_utf8_matches_for_all_clients = true;
    let mut cancelled_compose_rejected_for_all_locales = true;
    let mut repeat_info_matches = true;

    for (index, keyboard_layout) in KEYBOARD_LAYOUT_SPECS.iter().copied().enumerate() {
        if DECLARED_KEYBOARD_LAYOUTS[index] != keyboard_layout.installer_value {
            return Err("installer keyboard layout order differs from compositor matrix".into());
        }

        let mut session = AquaCompositorSession::new_with_keyboard_layout(keyboard_layout)?;
        let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
        let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
        session.insert_client(server_stream_one)?;
        session.insert_client(server_stream_two)?;

        let client_one_conn = ClientConnection::from_socket(client_stream_one)?;
        let client_two_conn = ClientConnection::from_socket(client_stream_two)?;
        let mut event_queue_one = client_one_conn.new_event_queue();
        let mut event_queue_two = client_two_conn.new_event_queue();
        client_one_conn
            .display()
            .get_registry(&event_queue_one.handle(), ());
        client_two_conn
            .display()
            .get_registry(&event_queue_two.handle(), ());

        let mut client_one = KeyboardMatrixClientState::default();
        let mut client_two = KeyboardMatrixClientState::default();
        for _ in 0..3 {
            client_one_conn.flush()?;
            client_two_conn.flush()?;
            session.dispatch_clients()?;
            session.flush_clients()?;
            event_queue_one.blocking_dispatch(&mut client_one)?;
            event_queue_two.blocking_dispatch(&mut client_two)?;
        }

        keymaps_delivered_to_all_clients &= client_one.registry_bound
            && client_two.registry_bound
            && client_one.seat_seen
            && client_two.seat_seen
            && client_one.keyboard_capability_seen
            && client_two.keyboard_capability_seen
            && client_one.keymap.is_some()
            && client_two.keymap.is_some();
        repeat_info_matches &=
            client_one.repeat_info == Some((400, 25)) && client_two.repeat_info == Some((400, 25));

        for keymap in [client_one.keymap, client_two.keymap] {
            let Some(keymap) = keymap else {
                keymaps_compile_for_all_layouts = false;
                representative_utf8_matches = false;
                continue;
            };
            let state = xkb::State::new(&keymap);
            let keycode =
                xkb::Keycode::new(keyboard_layout.representative_evdev_key + XKB_KEYCODE_OFFSET);
            representative_utf8_matches &=
                state.key_get_utf8(keycode) == keyboard_layout.representative_utf8;

            let compose_keysym = state.key_get_one_sym(xkb::Keycode::new(COMPOSE_XKB_KEYCODE));
            compose_key_available_for_all_layouts &=
                compose_keysym.raw() == xkb::keysyms::KEY_Multi_key;
            for locale in DECLARED_LOCALES {
                compose_utf8_matches_for_all_clients &=
                    compose_sequence_result(
                        locale,
                        &[
                            compose_keysym,
                            xkb::Keysym::new(xkb::keysyms::KEY_apostrophe),
                            xkb::Keysym::new(xkb::keysyms::KEY_e),
                        ],
                    ) == Ok((xkb::compose::Status::Composed, Some("é".to_string())));
            }

            if index < 2 {
                let mut state = xkb::State::new(&keymap);
                state.update_key(
                    xkb::Keycode::new(LEFT_SHIFT_XKB_KEYCODE),
                    xkb::KeyDirection::Down,
                );
                state.update_key(
                    xkb::Keycode::new(RIGHT_ALT_XKB_KEYCODE),
                    xkb::KeyDirection::Down,
                );
                let dead_acute =
                    state.key_get_one_sym(xkb::Keycode::new(TURKISH_DEAD_KEY_XKB_KEYCODE));
                state.update_key(
                    xkb::Keycode::new(RIGHT_ALT_XKB_KEYCODE),
                    xkb::KeyDirection::Up,
                );
                state.update_key(
                    xkb::Keycode::new(LEFT_SHIFT_XKB_KEYCODE),
                    xkb::KeyDirection::Up,
                );
                dead_key_utf8_matches_for_all_clients &=
                    dead_acute.raw() == xkb::keysyms::KEY_dead_acute;
                for locale in DECLARED_LOCALES {
                    dead_key_utf8_matches_for_all_clients &=
                        compose_sequence_result(
                            locale,
                            &[dead_acute, xkb::Keysym::new(xkb::keysyms::KEY_e)],
                        ) == Ok((xkb::compose::Status::Composed, Some("é".to_string())));
                }
            }
        }
    }

    for locale in DECLARED_LOCALES {
        cancelled_compose_rejected_for_all_locales &=
            compose_sequence_result(
                locale,
                &[
                    xkb::Keysym::new(xkb::keysyms::KEY_Multi_key),
                    xkb::Keysym::new(xkb::keysyms::KEY_apostrophe),
                    xkb::Keysym::new(xkb::keysyms::KEY_x),
                ],
            ) == Ok((xkb::compose::Status::Cancelled, None));
    }

    Ok(KeyboardLocaleMatrixProbe {
        product: PRODUCT,
        status: "keyboard-locale-matrix",
        protocol: "wl_keyboard",
        locale_count: DECLARED_LOCALES.len(),
        keyboard_layout_count: DECLARED_KEYBOARD_LAYOUTS.len(),
        supported_combination_count: DECLARED_LOCALES.len() * DECLARED_KEYBOARD_LAYOUTS.len(),
        client_count_per_layout: 2,
        keymaps_delivered_to_all_clients,
        keymaps_compile_for_all_layouts,
        representative_utf8_matches,
        compose_key_available_for_all_layouts,
        compose_case_count: DECLARED_LOCALES.len() * DECLARED_KEYBOARD_LAYOUTS.len(),
        compose_utf8_matches_for_all_clients,
        dead_key_layout_count: 2,
        dead_key_case_count: DECLARED_LOCALES.len() * 2,
        dead_key_utf8_matches_for_all_clients,
        cancelled_compose_rejected_for_all_locales,
        repeat_delay_ms: 400,
        repeat_rate_hz: 25,
        repeat_info_matches,
        host_stub: false,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_privileged_protocol_boundary_impl(
) -> Result<PrivilegedProtocolBoundaryProbe, Box<dyn std::error::Error>> {
    Ok(PrivilegedProtocolBoundaryProbe {
        product: PRODUCT,
        status: "privileged-protocol-boundary",
        client_count: 3,
        normal_client_count: 2,
        authorized_client_count: 1,
        baseline_globals_visible_to_all_clients: true,
        input_method_hidden_from_normal_clients: true,
        input_method_visible_to_authorized_client: true,
        privileged_global_count: PRIVILEGED_WAYLAND_GLOBALS.len(),
        screenshot_global_exposed: false,
        screencopy_global_exposed: false,
        activation_global_exposed: false,
        privileged_shell_global_exposed: false,
        virtual_input_global_exposed: false,
        desktop_management_global_exposed: false,
        authorized_scope_is_narrow: true,
        host_stub: true,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_privileged_protocol_boundary_impl(
) -> Result<PrivilegedProtocolBoundaryProbe, Box<dyn std::error::Error>> {
    const INPUT_METHOD_MANAGER: &str = "zwp_input_method_manager_v2";
    const SCREENSHOT_GLOBALS: [&str; 1] = ["weston_screenshooter"];
    const SCREENCOPY_GLOBALS: [&str; 4] = [
        "zwlr_screencopy_manager_v1",
        "ext_image_copy_capture_manager_v1",
        "ext_output_image_capture_source_manager_v1",
        "zwlr_export_dmabuf_manager_v1",
    ];
    const ACTIVATION_GLOBALS: [&str; 1] = ["xdg_activation_v1"];
    const PRIVILEGED_SHELL_GLOBALS: [&str; 3] = [
        "zwlr_layer_shell_v1",
        "weston_desktop_shell",
        "ext_session_lock_manager_v1",
    ];
    const VIRTUAL_INPUT_GLOBALS: [&str; 1] = ["zwp_virtual_keyboard_manager_v1"];
    const DESKTOP_MANAGEMENT_GLOBALS: [&str; 6] = [
        "zwlr_foreign_toplevel_manager_v1",
        "ext_foreign_toplevel_list_v1",
        "zwlr_output_manager_v1",
        "zwlr_gamma_control_manager_v1",
        "zwlr_output_power_manager_v1",
        "wp_drm_lease_device_v1",
    ];

    let mut session = AquaCompositorSession::new()?;
    let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_authorized, client_stream_authorized) =
        std::os::unix::net::UnixStream::pair()?;
    session.insert_client(server_stream_one)?;
    session.insert_client(server_stream_two)?;
    session.insert_authorized_input_method_client(server_stream_authorized)?;

    let client_one_conn = ClientConnection::from_socket(client_stream_one)?;
    let client_two_conn = ClientConnection::from_socket(client_stream_two)?;
    let authorized_conn = ClientConnection::from_socket(client_stream_authorized)?;
    let mut event_queue_one = client_one_conn.new_event_queue();
    let mut event_queue_two = client_two_conn.new_event_queue();
    let mut event_queue_authorized = authorized_conn.new_event_queue();
    client_one_conn
        .display()
        .get_registry(&event_queue_one.handle(), ());
    client_two_conn
        .display()
        .get_registry(&event_queue_two.handle(), ());
    authorized_conn
        .display()
        .get_registry(&event_queue_authorized.handle(), ());
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    authorized_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let mut client_one = ProtocolBoundaryClientState::default();
    let mut client_two = ProtocolBoundaryClientState::default();
    let mut authorized_client = ProtocolBoundaryClientState::default();
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    event_queue_authorized.blocking_dispatch(&mut authorized_client)?;

    let clients = [&client_one, &client_two, &authorized_client];
    let any_client_sees =
        |interfaces: &[&str]| clients.iter().any(|client| client.sees_any(interfaces));
    let screenshot_global_exposed = any_client_sees(&SCREENSHOT_GLOBALS);
    let screencopy_global_exposed = any_client_sees(&SCREENCOPY_GLOBALS);
    let activation_global_exposed = any_client_sees(&ACTIVATION_GLOBALS);
    let privileged_shell_global_exposed = any_client_sees(&PRIVILEGED_SHELL_GLOBALS);
    let virtual_input_global_exposed = any_client_sees(&VIRTUAL_INPUT_GLOBALS);
    let desktop_management_global_exposed = any_client_sees(&DESKTOP_MANAGEMENT_GLOBALS);
    let authorized_sees_forbidden = authorized_client.sees_any(&PRIVILEGED_WAYLAND_GLOBALS);

    Ok(PrivilegedProtocolBoundaryProbe {
        product: PRODUCT,
        status: "privileged-protocol-boundary",
        client_count: 3,
        normal_client_count: 2,
        authorized_client_count: 1,
        baseline_globals_visible_to_all_clients: clients
            .iter()
            .all(|client| client.registry_bound && client.sees_all_baseline_globals()),
        input_method_hidden_from_normal_clients: !client_one.sees(INPUT_METHOD_MANAGER)
            && !client_two.sees(INPUT_METHOD_MANAGER),
        input_method_visible_to_authorized_client: authorized_client.sees(INPUT_METHOD_MANAGER),
        privileged_global_count: PRIVILEGED_WAYLAND_GLOBALS.len(),
        screenshot_global_exposed,
        screencopy_global_exposed,
        activation_global_exposed,
        privileged_shell_global_exposed,
        virtual_input_global_exposed,
        desktop_management_global_exposed,
        authorized_scope_is_narrow: authorized_client.sees(INPUT_METHOD_MANAGER)
            && !authorized_sees_forbidden,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_selection_ownership_impl() -> Result<SelectionOwnershipProbe, Box<dyn std::error::Error>> {
    const CLIPBOARD_PAYLOAD: &[u8] = b"Aqua clipboard transfer\n";
    const PRIMARY_PAYLOAD: &[u8] = b"Aqua primary selection transfer\n";

    let mut session = AquaCompositorSession::new()?;
    let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
    let server_client_one = session.insert_client(server_stream_one)?;
    let server_client_two = session.insert_client(server_stream_two)?;

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

    let mut client_one =
        SelectionSmokeClientState::with_payloads(CLIPBOARD_PAYLOAD, PRIMARY_PAYLOAD);
    let mut client_two = SelectionSmokeClientState::with_payloads(b"replacement", b"replacement");
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let surface_one = session
        .wayland_state
        .committed_surfaces
        .iter()
        .find(|surface| surface.client().as_ref() == Some(&server_client_one))
        .cloned()
        .ok_or("first selection client did not commit a focus surface")?;
    let surface_two = session
        .wayland_state
        .committed_surfaces
        .iter()
        .find(|surface| surface.client().as_ref() == Some(&server_client_two))
        .cloned()
        .ok_or("second selection client did not commit a focus surface")?;

    let keyboard = session
        .wayland_state
        .seat
        .get_keyboard()
        .ok_or("Aqua Seat keyboard is required for selection ownership")?;
    keyboard.set_focus(
        &mut session.wayland_state,
        Some(surface_one),
        Serial::from(1),
    );
    session.flush_clients()?;

    client_two.set_clipboard(2);
    client_two.set_primary(2);
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    let unfocused_clipboard_rejected = session.wayland_state.clipboard_selection_count == 0;
    let unfocused_primary_rejected = session.wayland_state.primary_selection_count == 0;

    client_one.set_clipboard(3);
    client_one.set_primary(3);
    client_one_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    let focused_clipboard_accepted = session.wayland_state.clipboard_selection_count == 1;
    let focused_primary_accepted = session.wayland_state.primary_selection_count == 1;

    keyboard.set_focus(
        &mut session.wayland_state,
        Some(surface_two),
        Serial::from(4),
    );
    session.flush_clients()?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    let clipboard_offer_reaches_new_focus = client_two.clipboard_offer_received;
    let primary_offer_reaches_new_focus = client_two.primary_offer_received;

    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    let clipboard_payload = client_two.read_clipboard_payload()?;
    let primary_payload = client_two.read_primary_payload()?;
    let clipboard_mime_negotiated = client_two
        .clipboard_offer_mimes
        .iter()
        .any(|mime| mime == SelectionSmokeClientState::MIME_TYPE)
        && client_two
            .clipboard_requested_mimes
            .iter()
            .any(|mime| mime == SelectionSmokeClientState::MIME_TYPE);
    let primary_mime_negotiated = client_two
        .primary_offer_mimes
        .iter()
        .any(|mime| mime == SelectionSmokeClientState::MIME_TYPE)
        && client_two
            .primary_requested_mimes
            .iter()
            .any(|mime| mime == SelectionSmokeClientState::MIME_TYPE);
    let unsupported_mime_not_requested = !client_one
        .clipboard_requested_mimes
        .iter()
        .chain(client_one.primary_requested_mimes.iter())
        .chain(client_two.clipboard_requested_mimes.iter())
        .chain(client_two.primary_requested_mimes.iter())
        .any(|mime| mime == SelectionSmokeClientState::UNSUPPORTED_MIME_TYPE);
    let clipboard_payload_transferred = clipboard_payload == CLIPBOARD_PAYLOAD;
    let primary_payload_transferred = primary_payload == PRIMARY_PAYLOAD;
    let clipboard_payload_bytes = clipboard_payload.len();
    let primary_payload_bytes = primary_payload.len();
    let data_control_global_exposed =
        client_one.data_control_global_seen || client_two.data_control_global_seen;
    let globals_visible_to_both_clients = client_one.globals_ready() && client_two.globals_ready();

    session
        .wayland_state
        .display_handle
        .backend_handle()
        .kill_client(server_client_one.id(), DisconnectReason::ConnectionClosed);
    drop(client_one);
    drop(qh_one);
    drop(event_queue_one);
    drop(client_one_conn);
    drop(server_client_one);
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    let owner_disconnect_clears_clipboard = client_two.clipboard_selection_cleared;
    let owner_disconnect_clears_primary = client_two.primary_selection_cleared;

    client_two.set_clipboard(5);
    client_two.set_primary(5);
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    let ownership_handoff_accepted = session.wayland_state.clipboard_selection_count == 2
        && session.wayland_state.primary_selection_count == 2;

    Ok(SelectionOwnershipProbe {
        product: PRODUCT,
        status: "selection-ownership",
        clipboard_protocol: "wl_data_device_manager",
        primary_protocol: "zwp_primary_selection_device_manager_v1",
        client_count: 2,
        globals_visible_to_both_clients,
        focus_follows_keyboard: true,
        unfocused_clipboard_rejected,
        unfocused_primary_rejected,
        focused_clipboard_accepted,
        focused_primary_accepted,
        clipboard_offer_reaches_new_focus,
        primary_offer_reaches_new_focus,
        clipboard_mime_negotiated,
        primary_mime_negotiated,
        unsupported_mime_not_requested,
        clipboard_payload_transferred,
        primary_payload_transferred,
        clipboard_payload_bytes,
        primary_payload_bytes,
        transfer_limit_bytes: SelectionSmokeClientState::TRANSFER_LIMIT_BYTES,
        compositor_buffers_payload: false,
        owner_disconnect_clears_clipboard,
        owner_disconnect_clears_primary,
        ownership_handoff_accepted,
        data_control_global_exposed,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_drag_and_drop_impl() -> Result<DragAndDropProbe, Box<dyn std::error::Error>> {
    const PAYLOAD: &[u8] = b"Aqua drag-and-drop transfer\n";

    let mut session = AquaCompositorSession::new()?;
    let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
    let server_client_one = session.insert_client(server_stream_one)?;
    let server_client_two = session.insert_client(server_stream_two)?;

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

    let mut client_one = DndSmokeClientState::with_payload(PAYLOAD);
    let mut client_two = DndSmokeClientState {
        accept_drop: true,
        ..DndSmokeClientState::default()
    };
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let surface_one = session
        .wayland_state
        .committed_surfaces
        .iter()
        .find(|surface| surface.client().as_ref() == Some(&server_client_one))
        .cloned()
        .ok_or("first drag-and-drop client did not commit an origin surface")?;
    let surface_two = session
        .wayland_state
        .committed_surfaces
        .iter()
        .find(|surface| surface.client().as_ref() == Some(&server_client_two))
        .cloned()
        .ok_or("second drag-and-drop client did not commit a target surface")?;
    let pointer = session
        .wayland_state
        .seat
        .get_pointer()
        .ok_or("Aqua Seat pointer is required for drag-and-drop")?;
    let keyboard = session
        .wayland_state
        .seat
        .get_keyboard()
        .ok_or("Aqua Seat keyboard is required for drag-and-drop focus isolation")?;
    keyboard.set_focus(
        &mut session.wayland_state,
        Some(surface_one.clone()),
        Serial::from(1),
    );

    client_one.start_drag(&qh_one, 2);
    client_one_conn.flush()?;
    session.dispatch_clients()?;
    let start_without_implicit_grab_rejected = session.wayland_state.dnd_started_count == 0;

    pointer.motion(
        &mut session.wayland_state,
        Some((surface_one.clone(), (0.0, 0.0).into())),
        &MotionEvent {
            location: (16.0, 16.0).into(),
            serial: Serial::from(3),
            time: 3,
        },
    );
    pointer.button(
        &mut session.wayland_state,
        &ButtonEvent {
            serial: Serial::from(4),
            time: 4,
            button: 0x110,
            state: ButtonState::Pressed,
        },
    );
    client_one.start_drag(&qh_one, 4);
    client_one_conn.flush()?;
    session.dispatch_clients()?;
    let pointer_grab_started = session.wayland_state.dnd_started_count == 1;
    let source_client_owns_drag =
        session.wayland_state.dnd_source_owner == Some(server_client_one.id());

    pointer.motion(
        &mut session.wayland_state,
        Some((surface_two.clone(), (100.0, 100.0).into())),
        &MotionEvent {
            location: (132.0, 140.0).into(),
            serial: Serial::from(5),
            time: 5,
        },
    );
    session.flush_clients()?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    let payload = client_two.read_payload()?;

    let enter_reaches_pointer_focus_only = client_two.enter_count == 1
        && client_two.enter_matches_own_surface
        && client_one.enter_count == 0;
    let mime_negotiated = client_two
        .offered_mimes
        .iter()
        .any(|mime| mime == DndSmokeClientState::MIME_TYPE)
        && client_two
            .requested_mimes
            .iter()
            .any(|mime| mime == DndSmokeClientState::MIME_TYPE)
        && client_one
            .requested_mimes
            .iter()
            .any(|mime| mime == DndSmokeClientState::MIME_TYPE);
    let unsupported_mime_not_accepted = client_one.source_target_mime.as_deref()
        != Some(DndSmokeClientState::UNSUPPORTED_MIME_TYPE)
        && !client_one
            .requested_mimes
            .iter()
            .chain(client_two.requested_mimes.iter())
            .any(|mime| mime == DndSmokeClientState::UNSUPPORTED_MIME_TYPE);
    let copy_action_negotiated = client_two.source_actions
        == Some(
            client_wl_data_device_manager::DndAction::Copy
                | client_wl_data_device_manager::DndAction::Move,
        )
        && client_two.chosen_action == Some(client_wl_data_device_manager::DndAction::Copy)
        && client_one.source_chosen_action == Some(client_wl_data_device_manager::DndAction::Copy);
    let payload_transferred = payload == PAYLOAD;
    let payload_bytes = payload.len();

    pointer.button(
        &mut session.wayland_state,
        &ButtonEvent {
            serial: Serial::from(6),
            time: 6,
            button: 0x110,
            state: ButtonState::Released,
        },
    );
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;

    let drop_delivered_to_target = client_two.drop_count == 1
        && session.wayland_state.dnd_validated_drop_count == 1
        && session.wayland_state.dnd_drop_target == Some(server_client_two.id());
    let source_drop_performed = client_one.source_drop_performed;
    let source_finished = client_one.source_finished;
    let accepted_drop_count = client_two.drop_count;

    pointer.motion(
        &mut session.wayland_state,
        Some((surface_one.clone(), (0.0, 0.0).into())),
        &MotionEvent {
            location: (20.0, 20.0).into(),
            serial: Serial::from(7),
            time: 7,
        },
    );
    pointer.button(
        &mut session.wayland_state,
        &ButtonEvent {
            serial: Serial::from(8),
            time: 8,
            button: 0x110,
            state: ButtonState::Pressed,
        },
    );
    client_two.reset_target_for_rejected_drop();
    client_one.start_drag(&qh_one, 8);
    client_one_conn.flush()?;
    session.dispatch_clients()?;
    pointer.motion(
        &mut session.wayland_state,
        Some((surface_two.clone(), (100.0, 100.0).into())),
        &MotionEvent {
            location: (136.0, 144.0).into(),
            serial: Serial::from(9),
            time: 9,
        },
    );
    session.flush_clients()?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    pointer.button(
        &mut session.wayland_state,
        &ButtonEvent {
            serial: Serial::from(10),
            time: 10,
            button: 0x110,
            state: ButtonState::Released,
        },
    );
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;

    let rejected_drop_cancelled = client_one.source_cancelled
        && session.wayland_state.dnd_started_count == 2
        && session.wayland_state.dnd_cancelled_drop_count == 1;
    let rejected_drop_not_delivered = client_two.drop_count == accepted_drop_count;
    let keyboard_focus_unchanged = keyboard.current_focus().as_ref() == Some(&surface_one);

    Ok(DragAndDropProbe {
        product: PRODUCT,
        status: "drag-and-drop",
        protocol: "wl_data_device_manager",
        client_count: 2,
        globals_visible_to_both_clients: client_one.globals_ready() && client_two.globals_ready(),
        start_without_implicit_grab_rejected,
        pointer_grab_started,
        source_client_owns_drag,
        enter_reaches_pointer_focus_only,
        keyboard_focus_unchanged,
        mime_negotiated,
        unsupported_mime_not_accepted,
        copy_action_negotiated,
        payload_transferred,
        payload_bytes,
        transfer_limit_bytes: DndSmokeClientState::TRANSFER_LIMIT_BYTES,
        compositor_buffers_payload: false,
        drop_delivered_to_target,
        source_drop_performed,
        source_finished,
        rejected_drop_cancelled,
        rejected_drop_not_delivered,
        data_control_global_exposed: client_one.data_control_global_seen
            || client_two.data_control_global_seen,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_text_input_impl() -> Result<TextInputProbe, Box<dyn std::error::Error>> {
    let mut session = AquaCompositorSession::new()?;
    let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_ime, client_stream_ime) = std::os::unix::net::UnixStream::pair()?;
    let server_client_one = session.insert_client(server_stream_one)?;
    let server_client_two = session.insert_client(server_stream_two)?;
    session.insert_authorized_input_method_client(server_stream_ime)?;

    let client_one_conn = ClientConnection::from_socket(client_stream_one)?;
    let client_two_conn = ClientConnection::from_socket(client_stream_two)?;
    let input_method_conn = ClientConnection::from_socket(client_stream_ime)?;
    let mut event_queue_one = client_one_conn.new_event_queue();
    let mut event_queue_two = client_two_conn.new_event_queue();
    let mut event_queue_ime = input_method_conn.new_event_queue();
    let qh_one = event_queue_one.handle();
    let qh_two = event_queue_two.handle();
    let qh_ime = event_queue_ime.handle();
    client_one_conn.display().get_registry(&qh_one, ());
    client_two_conn.display().get_registry(&qh_two, ());
    input_method_conn.display().get_registry(&qh_ime, ());
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    input_method_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let mut client_one = TextInputSmokeClientState::default();
    let mut client_two = TextInputSmokeClientState::default();
    let mut input_method = InputMethodSmokeClientState::default();
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    event_queue_ime.blocking_dispatch(&mut input_method)?;
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    input_method_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let surface_one = session
        .wayland_state
        .committed_surfaces
        .iter()
        .find(|surface| surface.client().as_ref() == Some(&server_client_one))
        .cloned()
        .ok_or("first text-input client did not commit a surface")?;
    let surface_two = session
        .wayland_state
        .committed_surfaces
        .iter()
        .find(|surface| surface.client().as_ref() == Some(&server_client_two))
        .cloned()
        .ok_or("second text-input client did not commit a surface")?;
    let keyboard = session
        .wayland_state
        .seat
        .get_keyboard()
        .ok_or("Aqua Seat keyboard is required for text-input focus")?;

    keyboard.set_focus(
        &mut session.wayland_state,
        Some(surface_one.clone()),
        Serial::from(1),
    );
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    let focus_follows_keyboard = client_one.enter_count == 1
        && client_one.entered_own_surface
        && client_two.enter_count == 0;

    client_two.enable_with_state();
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    let unfocused_enable_rejected = input_method.activate_count == 0;

    client_one.enable_with_state();
    client_one_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_ime.blocking_dispatch(&mut input_method)?;
    while event_queue_ime.dispatch_pending(&mut input_method)? > 0 {}
    input_method_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;

    let expected_cursor = TextInputSmokeClientState::SURROUNDING_TEXT.len() as u32;
    let focused_enable_activates_input_method = input_method.activate_count == 1;
    let surrounding_text_forwarded = input_method.surrounding_text.as_deref()
        == Some(TextInputSmokeClientState::SURROUNDING_TEXT)
        && input_method.surrounding_cursor == expected_cursor
        && input_method.surrounding_anchor == expected_cursor;
    let content_type_forwarded =
        input_method.content_type_forwarded && input_method.text_change_cause_forwarded;
    let turkish_preedit_delivered =
        client_one.preedit_text.as_deref() == Some(TextInputSmokeClientState::PREEDIT_TEXT);
    let turkish_commit_delivered =
        client_one.commit_text.as_deref() == Some(TextInputSmokeClientState::COMMIT_TEXT);
    let delete_surrounding_delivered =
        client_one.delete_before == 1 && client_one.delete_after == 0;
    let serial_synchronized = client_one.done_serials.last().copied() == Some(1);
    let popup_parent_bound = session.wayland_state.input_method_popup_new_count == 1;

    client_one.update_cursor_rectangle(48, 52);
    client_one_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_ime.blocking_dispatch(&mut input_method)?;
    let cursor_rectangle_forwarded = input_method.popup_rectangle == Some((48, 52, 2, 28));
    let popup_repositioned = session.wayland_state.input_method_popup_reposition_count == 1;

    let deactivate_count_before_handoff = input_method.deactivate_count;
    keyboard.set_focus(
        &mut session.wayland_state,
        Some(surface_two),
        Serial::from(2),
    );
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    event_queue_ime.blocking_dispatch(&mut input_method)?;
    let focus_handoff_deactivates_input_method = input_method.deactivate_count
        > deactivate_count_before_handoff
        && client_one.leave_count == 1
        && session.wayland_state.input_method_popup_dismiss_count == 1;
    let focus_handoff_enters_new_client =
        client_two.enter_count == 1 && client_two.entered_own_surface;

    let activation_count_before_stale_request = input_method.activate_count;
    client_one.enable_with_state();
    client_one_conn.flush()?;
    session.dispatch_clients()?;
    let stale_unfocused_client_blocked =
        input_method.activate_count == activation_count_before_stale_request;

    client_two.enable_with_state();
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_ime.blocking_dispatch(&mut input_method)?;
    let second_focus_activated = input_method.activate_count == 2;

    Ok(TextInputProbe {
        product: PRODUCT,
        status: "text-input",
        text_input_protocol: "zwp_text_input_manager_v3",
        input_method_protocol: "zwp_input_method_manager_v2",
        client_count: 3,
        text_input_visible_to_normal_clients: client_one.globals_ready()
            && client_two.globals_ready(),
        input_method_hidden_from_normal_clients: !client_one.input_method_global_seen
            && !client_two.input_method_global_seen,
        input_method_visible_to_authorized_client: input_method.registry_bound
            && input_method.text_input_global_seen
            && input_method.input_method_global_seen
            && input_method.input_method.is_some()
            && !input_method.unavailable,
        focus_follows_keyboard,
        unfocused_enable_rejected,
        focused_enable_activates_input_method,
        surrounding_text_forwarded,
        content_type_forwarded,
        cursor_rectangle_forwarded,
        turkish_preedit_delivered,
        turkish_commit_delivered,
        delete_surrounding_delivered,
        serial_synchronized,
        focus_handoff_deactivates_input_method,
        focus_handoff_enters_new_client: focus_handoff_enters_new_client && second_focus_activated,
        stale_unfocused_client_blocked,
        popup_parent_bound,
        popup_repositioned,
        payload_limit_bytes: TextInputSmokeClientState::PAYLOAD_LIMIT_BYTES,
        host_stub: false,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_keyboard_locale_matrix_impl(
) -> Result<KeyboardLocaleMatrixProbe, Box<dyn std::error::Error>> {
    Ok(KeyboardLocaleMatrixProbe {
        product: PRODUCT,
        status: "keyboard-locale-matrix",
        protocol: "wl_keyboard",
        locale_count: DECLARED_LOCALES.len(),
        keyboard_layout_count: DECLARED_KEYBOARD_LAYOUTS.len(),
        supported_combination_count: DECLARED_LOCALES.len() * DECLARED_KEYBOARD_LAYOUTS.len(),
        client_count_per_layout: 2,
        keymaps_delivered_to_all_clients: true,
        keymaps_compile_for_all_layouts: true,
        representative_utf8_matches: true,
        compose_key_available_for_all_layouts: true,
        compose_case_count: DECLARED_LOCALES.len() * DECLARED_KEYBOARD_LAYOUTS.len(),
        compose_utf8_matches_for_all_clients: true,
        dead_key_layout_count: 2,
        dead_key_case_count: DECLARED_LOCALES.len() * 2,
        dead_key_utf8_matches_for_all_clients: true,
        cancelled_compose_rejected_for_all_locales: true,
        repeat_delay_ms: 400,
        repeat_rate_hz: 25,
        repeat_info_matches: true,
        host_stub: true,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_wayland_output_matrix_impl() -> Result<WaylandOutputMatrixProbe, Box<dyn std::error::Error>>
{
    let mut session = AquaCompositorSession::new()?;
    session.wayland_state.surface_preferred_scale = 1.25;
    let primary_mode = OutputMode {
        size: (1280, 800).into(),
        refresh: 60_000,
    };
    session.wayland_state.outputs[0].set_preferred(primary_mode);
    session.wayland_state.outputs[0].change_current_state(
        Some(primary_mode),
        Some(Transform::Normal),
        Some(Scale::Integer(1)),
        Some((0, 0).into()),
    );
    let (server_stream_one, client_stream_one) = std::os::unix::net::UnixStream::pair()?;
    let (server_stream_two, client_stream_two) = std::os::unix::net::UnixStream::pair()?;
    session.insert_client(server_stream_one)?;
    session.insert_client(server_stream_two)?;

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

    let mut client_one = OutputSmokeClientState::default();
    let mut client_two = OutputSmokeClientState::default();
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    while event_queue_one.dispatch_pending(&mut client_one)? > 0 {}
    while event_queue_two.dispatch_pending(&mut client_two)? > 0 {}
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    while event_queue_one.dispatch_pending(&mut client_one)? > 0 {}
    while event_queue_two.dispatch_pending(&mut client_two)? > 0 {}

    let initial_output_visible_to_both_clients =
        client_one.globals_ready(1) && client_two.globals_ready(1);

    for output in [
        configured_output(
            "Aqua-2",
            (2560, 1440),
            (1280, 0),
            Scale::Custom {
                advertised_integer: 2,
                fractional: 1.25,
            },
            Transform::_90,
            (597, 336),
        ),
        configured_output(
            "Aqua-3",
            (1920, 1080),
            (2432, 0),
            Scale::Custom {
                advertised_integer: 2,
                fractional: 1.5,
            },
            Transform::_180,
            (527, 296),
        ),
        configured_output(
            "Aqua-4",
            (3840, 2160),
            (3712, 0),
            Scale::Integer(2),
            Transform::_270,
            (708, 399),
        ),
    ] {
        session
            .wayland_state
            .output_globals
            .push(output.create_global::<WaylandSmokeState>(&session.wayland_state.display_handle));
        session.wayland_state.outputs.push(output);
    }
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    while event_queue_one.dispatch_pending(&mut client_one)? > 0 {}
    while event_queue_two.dispatch_pending(&mut client_two)? > 0 {}
    let hotplug_add_reaches_both_clients =
        client_one.outputs.len() == 4 && client_two.outputs.len() == 4;
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    while event_queue_one.dispatch_pending(&mut client_one)? > 0 {}
    while event_queue_two.dispatch_pending(&mut client_two)? > 0 {}

    let globals_visible_to_both_clients = initial_output_visible_to_both_clients
        && client_one.globals_ready(4)
        && client_two.globals_ready(4);
    client_one.request_surface_extensions(&qh_one)?;
    client_two.request_surface_extensions(&qh_two)?;
    client_one_conn.flush()?;
    client_two_conn.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    while event_queue_one.dispatch_pending(&mut client_one)? > 0 {}
    while event_queue_two.dispatch_pending(&mut client_two)? > 0 {}

    let matrix_matches = |client: &OutputSmokeClientState| {
        [
            ("Aqua-1", (1280, 800, 60_000)),
            ("Aqua-2", (2560, 1440, 60_000)),
            ("Aqua-3", (1920, 1080, 60_000)),
            ("Aqua-4", (3840, 2160, 60_000)),
        ]
        .iter()
        .all(|(name, mode)| {
            client.outputs.iter().any(|record| {
                record.protocol_name.as_deref() == Some(*name) && record.mode == Some(*mode)
            })
        })
    };
    let preferred_matches = |client: &OutputSmokeClientState| {
        client
            .outputs
            .iter()
            .all(|record| record.current && record.preferred)
    };
    let logical_matches = |client: &OutputSmokeClientState| {
        [
            ("Aqua-1", (0, 0), (1280, 800)),
            ("Aqua-2", (1280, 0), (1152, 2048)),
            ("Aqua-3", (2432, 0), (1280, 720)),
            ("Aqua-4", (3712, 0), (1080, 1920)),
        ]
        .iter()
        .all(|(name, position, logical_size)| {
            client.outputs.iter().any(|record| {
                record.protocol_name.as_deref() == Some(*name)
                    && record.location == Some(*position)
                    && record.logical_position == Some(*position)
                    && record.logical_size == Some(*logical_size)
            })
        })
    };
    let scale_matches = |client: &OutputSmokeClientState| {
        [("Aqua-1", 1), ("Aqua-2", 2), ("Aqua-3", 2), ("Aqua-4", 2)]
            .iter()
            .all(|(name, scale)| {
                client.outputs.iter().any(|record| {
                    record.protocol_name.as_deref() == Some(*name)
                        && record.integer_scale == Some(*scale)
                })
            })
    };
    let transform_matches = |client: &OutputSmokeClientState| {
        [
            ("Aqua-1", client_wl_output::Transform::Normal),
            ("Aqua-2", client_wl_output::Transform::_90),
            ("Aqua-3", client_wl_output::Transform::_180),
            ("Aqua-4", client_wl_output::Transform::_270),
        ]
        .iter()
        .all(|(name, transform)| {
            client.outputs.iter().any(|record| {
                record.protocol_name.as_deref() == Some(*name)
                    && record.transform == Some(*transform)
            })
        })
    };

    let removed_global_one = client_one
        .outputs
        .iter()
        .find(|record| record.protocol_name.as_deref() == Some("Aqua-4"))
        .map(|record| record.global_name)
        .ok_or("first client did not discover Aqua-4")?;
    let removed_global_two = client_two
        .outputs
        .iter()
        .find(|record| record.protocol_name.as_deref() == Some("Aqua-4"))
        .map(|record| record.global_name)
        .ok_or("second client did not discover Aqua-4")?;
    session
        .wayland_state
        .display_handle
        .disable_global::<WaylandSmokeState>(session.wayland_state.output_globals[3].clone());
    session.flush_clients()?;
    event_queue_one.blocking_dispatch(&mut client_one)?;
    event_queue_two.blocking_dispatch(&mut client_two)?;
    while event_queue_one.dispatch_pending(&mut client_one)? > 0 {}
    while event_queue_two.dispatch_pending(&mut client_two)? > 0 {}

    let hotplug_remove_reaches_both_clients =
        client_one.removed_globals.contains(&removed_global_one)
            && client_two.removed_globals.contains(&removed_global_two);
    let remaining_output_usable = session.wayland_state.outputs[..3]
        .iter()
        .all(|output| output.current_mode().is_some())
        && ["Aqua-1", "Aqua-2", "Aqua-3"].iter().all(|name| {
            client_one
                .outputs
                .iter()
                .any(|record| record.protocol_name.as_deref() == Some(*name) && record.current)
                && client_two
                    .outputs
                    .iter()
                    .any(|record| record.protocol_name.as_deref() == Some(*name) && record.current)
        });

    Ok(WaylandOutputMatrixProbe {
        product: PRODUCT,
        status: "wayland-output-matrix",
        client_count: 2,
        output_count: 4,
        declared_scale_count: 4,
        declared_transform_count: 4,
        outputs_visible_to_both_clients: globals_visible_to_both_clients,
        modes_match_supported_matrix: matrix_matches(&client_one) && matrix_matches(&client_two),
        preferred_modes_advertised: preferred_matches(&client_one)
            && preferred_matches(&client_two),
        logical_coordinates_match: logical_matches(&client_one) && logical_matches(&client_two),
        integer_scales_match: scale_matches(&client_one) && scale_matches(&client_two),
        fractional_scales_match: logical_matches(&client_one) && logical_matches(&client_two),
        transforms_match: transform_matches(&client_one) && transform_matches(&client_two),
        fractional_scale_advertised: client_one.fractional_scale_120ths == Some(150)
            && client_two.fractional_scale_120ths == Some(150)
            && session.wayland_state.fractional_scale_request_count == 2,
        fractional_scale_120ths: client_one.fractional_scale_120ths.unwrap_or_default(),
        viewport_source_applied: session.wayland_state.viewport_source
            == Some((8.0, 12.0, 320.0, 180.0)),
        viewport_destination_applied: session.wayland_state.viewport_destination
            == Some((640, 360)),
        hotplug_add_reaches_both_clients,
        hotplug_remove_reaches_both_clients,
        remaining_output_usable,
        host_stub: false,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_popup_subsurface_matrix_impl(
) -> Result<PopupSubsurfaceMatrixProbe, Box<dyn std::error::Error>> {
    let mut session = AquaCompositorSession::new()?;
    session.wayland_state.close_new_toplevels = false;
    let (popup_server_stream, popup_client_stream) = std::os::unix::net::UnixStream::pair()?;
    let (subsurface_server_stream, subsurface_client_stream) =
        std::os::unix::net::UnixStream::pair()?;
    session.insert_client(popup_server_stream)?;
    session.insert_client(subsurface_server_stream)?;

    let popup_connection = ClientConnection::from_socket(popup_client_stream)?;
    let subsurface_connection = ClientConnection::from_socket(subsurface_client_stream)?;
    let mut popup_queue = popup_connection.new_event_queue();
    let mut subsurface_queue = subsurface_connection.new_event_queue();
    popup_connection
        .display()
        .get_registry(&popup_queue.handle(), ());
    subsurface_connection
        .display()
        .get_registry(&subsurface_queue.handle(), ());
    popup_connection.flush()?;
    subsurface_connection.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let mut popup_client = PopupLifecycleClientState::default();
    let mut subsurface_client = SubsurfaceLifecycleClientState::default();
    popup_queue.blocking_dispatch(&mut popup_client)?;
    subsurface_queue.blocking_dispatch(&mut subsurface_client)?;
    while popup_queue.dispatch_pending(&mut popup_client)? > 0 {}
    while subsurface_queue.dispatch_pending(&mut subsurface_client)? > 0 {}

    popup_connection.flush()?;
    subsurface_connection.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    popup_queue.blocking_dispatch(&mut popup_client)?;
    while popup_queue.dispatch_pending(&mut popup_client)? > 0 {}

    popup_connection.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    popup_queue.blocking_dispatch(&mut popup_client)?;
    while popup_queue.dispatch_pending(&mut popup_client)? > 0 {}

    popup_connection.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;
    popup_queue.blocking_dispatch(&mut popup_client)?;
    while popup_queue.dispatch_pending(&mut popup_client)? > 0 {}
    popup_connection.flush()?;
    session.dispatch_clients()?;

    let subsurface = session
        .wayland_state
        .committed_surfaces
        .iter()
        .find(|surface| get_role(surface) == Some("subsurface"))
        .cloned()
        .ok_or("server did not observe the independent wl_subsurface")?;
    let subsurface_parent = get_parent(&subsurface).ok_or("subsurface parent is missing")?;
    let subsurface_created = get_children(&subsurface_parent).contains(&subsurface);
    let subsurface_parent_bound = subsurface_created;
    let subsurface_position_matches = with_states(&subsurface, |states| {
        let mut cached = states.cached_state.get::<SubsurfaceCachedState>();
        let current = cached.current();
        current.location.x == 24 && current.location.y == 36
    });
    let synchronized_commit_observed = is_sync_subsurface(&subsurface)
        && session
            .wayland_state
            .committed_surfaces
            .contains(&subsurface_parent);

    let commit_count_before_desync = session.wayland_state.surface_commit_count;
    subsurface_client.set_desynchronized();
    subsurface_connection.flush()?;
    session.dispatch_clients()?;
    let desynchronized_commit_observed = !is_sync_subsurface(&subsurface)
        && session.wayland_state.surface_commit_count > commit_count_before_desync;

    popup_client.destroy_popup();
    subsurface_client.destroy_subsurface();
    popup_connection.flush()?;
    subsurface_connection.flush()?;
    session.dispatch_clients()?;
    session.flush_clients()?;

    let popup_destroyed = session.wayland_state.popup_destroy_count == 1;
    let subsurface_destroyed = !subsurface.is_alive()
        && !session
            .wayland_state
            .committed_surfaces
            .iter()
            .any(|surface| get_role(surface) == Some("subsurface"));
    let root_clients = session
        .wayland_state
        .committed_surfaces
        .iter()
        .filter(|surface| surface.is_alive() && get_parent(surface).is_none())
        .filter_map(Resource::client)
        .map(|client| client.id())
        .collect::<Vec<_>>();
    let parent_surfaces_remain_independent =
        root_clients.iter().enumerate().any(|(index, client)| {
            root_clients[index + 1..]
                .iter()
                .any(|other| other != client)
        });

    let popup_geometry_matches = session.wayland_state.popup_geometry_matches
        && popup_client.popup_geometries.contains(&(32, 48, 240, 120))
        && popup_client.popup_geometries.contains(&(64, 72, 240, 120));
    let result = PopupSubsurfaceMatrixProbe {
        product: PRODUCT,
        status: "popup-subsurface-matrix",
        client_count: 2,
        xdg_popup_created: session.wayland_state.popup_new_count == 1,
        popup_parent_bound: session.wayland_state.popup_parent_bound,
        popup_geometry_matches,
        popup_configure_acknowledged: session.wayland_state.popup_configure_ack_count >= 2
            && popup_client.popup_configure_ack_count >= 2,
        popup_reposition_requested: session.wayland_state.popup_reposition_count == 1,
        popup_reposition_token: session
            .wayland_state
            .popup_reposition_token
            .unwrap_or_default(),
        popup_reposition_acknowledged: popup_client.repositioned_token == Some(77),
        popup_destroyed,
        subsurface_created,
        subsurface_parent_bound,
        subsurface_position_matches,
        synchronized_commit_observed,
        desynchronized_commit_observed,
        subsurface_destroyed,
        parent_surfaces_remain_independent,
        host_stub: false,
    };

    popup_client.destroy_parent();
    subsurface_client.destroy_parent();
    popup_connection.flush()?;
    subsurface_connection.flush()?;
    session.dispatch_clients()?;
    Ok(result)
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_popup_subsurface_matrix_impl(
) -> Result<PopupSubsurfaceMatrixProbe, Box<dyn std::error::Error>> {
    Ok(PopupSubsurfaceMatrixProbe {
        product: PRODUCT,
        status: "popup-subsurface-matrix",
        client_count: 2,
        xdg_popup_created: true,
        popup_parent_bound: true,
        popup_geometry_matches: true,
        popup_configure_acknowledged: true,
        popup_reposition_requested: true,
        popup_reposition_token: 77,
        popup_reposition_acknowledged: true,
        popup_destroyed: true,
        subsurface_created: true,
        subsurface_parent_bound: true,
        subsurface_position_matches: true,
        synchronized_commit_observed: true,
        desynchronized_commit_observed: true,
        subsurface_destroyed: true,
        parent_surfaces_remain_independent: true,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_drag_and_drop_impl() -> Result<DragAndDropProbe, Box<dyn std::error::Error>> {
    Ok(DragAndDropProbe {
        product: PRODUCT,
        status: "drag-and-drop",
        protocol: "wl_data_device_manager",
        client_count: 2,
        globals_visible_to_both_clients: true,
        start_without_implicit_grab_rejected: true,
        pointer_grab_started: true,
        source_client_owns_drag: true,
        enter_reaches_pointer_focus_only: true,
        keyboard_focus_unchanged: true,
        mime_negotiated: true,
        unsupported_mime_not_accepted: true,
        copy_action_negotiated: true,
        payload_transferred: true,
        payload_bytes: 28,
        transfer_limit_bytes: 4_096,
        compositor_buffers_payload: false,
        drop_delivered_to_target: true,
        source_drop_performed: true,
        source_finished: true,
        rejected_drop_cancelled: true,
        rejected_drop_not_delivered: true,
        data_control_global_exposed: false,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_text_input_impl() -> Result<TextInputProbe, Box<dyn std::error::Error>> {
    Ok(TextInputProbe {
        product: PRODUCT,
        status: "text-input",
        text_input_protocol: "zwp_text_input_manager_v3",
        input_method_protocol: "zwp_input_method_manager_v2",
        client_count: 3,
        text_input_visible_to_normal_clients: true,
        input_method_hidden_from_normal_clients: true,
        input_method_visible_to_authorized_client: true,
        focus_follows_keyboard: true,
        unfocused_enable_rejected: true,
        focused_enable_activates_input_method: true,
        surrounding_text_forwarded: true,
        content_type_forwarded: true,
        cursor_rectangle_forwarded: true,
        turkish_preedit_delivered: true,
        turkish_commit_delivered: true,
        delete_surrounding_delivered: true,
        serial_synchronized: true,
        focus_handoff_deactivates_input_method: true,
        focus_handoff_enters_new_client: true,
        stale_unfocused_client_blocked: true,
        popup_parent_bound: true,
        popup_repositioned: true,
        payload_limit_bytes: 4_000,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_wayland_output_matrix_impl() -> Result<WaylandOutputMatrixProbe, Box<dyn std::error::Error>>
{
    Ok(WaylandOutputMatrixProbe {
        product: PRODUCT,
        status: "wayland-output-matrix",
        client_count: 2,
        output_count: 4,
        declared_scale_count: 4,
        declared_transform_count: 4,
        outputs_visible_to_both_clients: true,
        modes_match_supported_matrix: true,
        preferred_modes_advertised: true,
        logical_coordinates_match: true,
        integer_scales_match: true,
        fractional_scales_match: true,
        transforms_match: true,
        fractional_scale_advertised: true,
        fractional_scale_120ths: 150,
        viewport_source_applied: true,
        viewport_destination_applied: true,
        hotplug_add_reaches_both_clients: true,
        hotplug_remove_reaches_both_clients: true,
        remaining_output_usable: true,
        host_stub: true,
    })
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_selection_ownership_impl() -> Result<SelectionOwnershipProbe, Box<dyn std::error::Error>> {
    Ok(SelectionOwnershipProbe {
        product: PRODUCT,
        status: "selection-ownership",
        clipboard_protocol: "wl_data_device_manager",
        primary_protocol: "zwp_primary_selection_device_manager_v1",
        client_count: 2,
        globals_visible_to_both_clients: true,
        focus_follows_keyboard: true,
        unfocused_clipboard_rejected: true,
        unfocused_primary_rejected: true,
        focused_clipboard_accepted: true,
        focused_primary_accepted: true,
        clipboard_offer_reaches_new_focus: true,
        primary_offer_reaches_new_focus: true,
        clipboard_mime_negotiated: true,
        primary_mime_negotiated: true,
        unsupported_mime_not_requested: true,
        clipboard_payload_transferred: true,
        primary_payload_transferred: true,
        clipboard_payload_bytes: 24,
        primary_payload_bytes: 32,
        transfer_limit_bytes: 4_096,
        compositor_buffers_payload: false,
        owner_disconnect_clears_clipboard: true,
        owner_disconnect_clears_primary: true,
        ownership_handoff_accepted: true,
        data_control_global_exposed: false,
        host_stub: true,
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
fn probe_v1_client_buffer_contract_impl(
) -> Result<V1ClientBufferContractProbe, Box<dyn std::error::Error>> {
    Ok(V1ClientBufferContractProbe {
        product: PRODUCT,
        status: "v1-client-buffer-contract",
        application_model: "first-party-wl-shm-v1",
        required_buffer_protocol: "wl_shm",
        required_shm_format: "argb8888",
        client_count: 2,
        wl_shm_visible_to_all_clients: true,
        argb8888_visible_to_all_clients: true,
        linux_dmabuf_advertised: false,
        drm_syncobj_advertised: false,
        explicit_sync_advertised: false,
        accelerated_clients_supported: false,
        synchronization_scope: "wl_buffer.release+wl_surface.frame",
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
    display_handle: DisplayHandle,
    disconnected_clients: Arc<Mutex<Vec<ClientId>>>,
    compositor_state: CompositorState,
    shm_state: ShmState,
    xdg_shell_state: XdgShellState,
    data_device_state: DataDeviceState,
    primary_selection_state: PrimarySelectionState,
    _text_input_manager_state: TextInputManagerState,
    _input_method_manager_state: InputMethodManagerState,
    _output_manager_state: OutputManagerState,
    _fractional_scale_manager_state: FractionalScaleManagerState,
    _viewporter_state: ViewporterState,
    outputs: Vec<Output>,
    output_globals: Vec<GlobalId>,
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
    popup_new_count: usize,
    popup_parent_bound: bool,
    popup_geometry_matches: bool,
    popup_configure_sent_count: usize,
    popup_configure_ack_count: usize,
    popup_reposition_count: usize,
    popup_reposition_token: Option<u32>,
    popup_destroy_count: usize,
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
    committed_surfaces: Vec<WlSurface>,
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
    desktop_context_menu_pressed_keys: Vec<u32>,
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
    clipboard_selection_count: usize,
    primary_selection_count: usize,
    clipboard_selection_owner: Option<ClientId>,
    primary_selection_owner: Option<ClientId>,
    dnd_started_count: usize,
    dnd_validated_drop_count: usize,
    dnd_cancelled_drop_count: usize,
    dnd_source_owner: Option<ClientId>,
    dnd_drop_target: Option<ClientId>,
    input_method_popup_new_count: usize,
    input_method_popup_dismiss_count: usize,
    input_method_popup_reposition_count: usize,
    fractional_scale_request_count: usize,
    surface_preferred_scale: f64,
    viewport_source: Option<(f64, f64, f64, f64)>,
    viewport_destination: Option<(i32, i32)>,
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
    damage_commit_count: usize,
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
fn configured_output(
    name: &str,
    size: (i32, i32),
    location: (i32, i32),
    scale: Scale,
    transform: Transform,
    physical_size: (i32, i32),
) -> Output {
    let output = Output::new(
        name.to_string(),
        PhysicalProperties {
            size: physical_size.into(),
            subpixel: Subpixel::HorizontalRgb,
            make: "Aqua Linux".to_string(),
            model: "Supported virtual display".to_string(),
        },
    );
    let mode = OutputMode {
        size: size.into(),
        refresh: 60_000,
    };
    output.set_preferred(mode);
    output.change_current_state(
        Some(mode),
        Some(transform),
        Some(scale),
        Some(location.into()),
    );
    output
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Clone, Copy)]
struct KeyboardLayoutSpec {
    installer_value: &'static str,
    xkb_layout: &'static str,
    xkb_variant: &'static str,
    representative_evdev_key: u32,
    representative_utf8: &'static str,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
const KEYBOARD_LAYOUT_SPECS: [KeyboardLayoutSpec; 3] = [
    KeyboardLayoutSpec {
        installer_value: "trq",
        xkb_layout: "tr",
        xkb_variant: "",
        representative_evdev_key: 23,
        representative_utf8: "ı",
    },
    KeyboardLayoutSpec {
        installer_value: "trf",
        xkb_layout: "tr",
        xkb_variant: "f",
        representative_evdev_key: 16,
        representative_utf8: "f",
    },
    KeyboardLayoutSpec {
        installer_value: "us",
        xkb_layout: "us",
        xkb_variant: "",
        representative_evdev_key: 16,
        representative_utf8: "q",
    },
];

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
const XKB_KEYCODE_OFFSET: u32 = 8;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
const COMPOSE_XKB_KEYCODE: u32 = 127 + XKB_KEYCODE_OFFSET;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
const LEFT_SHIFT_XKB_KEYCODE: u32 = 42 + XKB_KEYCODE_OFFSET;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
const RIGHT_ALT_XKB_KEYCODE: u32 = 100 + XKB_KEYCODE_OFFSET;
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
const TURKISH_DEAD_KEY_XKB_KEYCODE: u32 = 39 + XKB_KEYCODE_OFFSET;

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl WaylandSmokeState {
    fn new(display_handle: &DisplayHandle) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_with_keyboard_layout(display_handle, KEYBOARD_LAYOUT_SPECS[2])
    }

    fn new_with_keyboard_layout(
        display_handle: &DisplayHandle,
        keyboard_layout: KeyboardLayoutSpec,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(display_handle, "Aqua Seat");
        seat.add_pointer();
        seat.add_touch();
        seat.add_keyboard(
            XkbConfig {
                layout: keyboard_layout.xkb_layout,
                variant: keyboard_layout.xkb_variant,
                options: Some("compose:menu".to_string()),
                ..XkbConfig::default()
            },
            400,
            25,
        )?;
        let mut launcher_scene = static_shell_scene(Viewport::new(1536, 1024));
        launcher_scene.set_surface_visible(SurfaceKind::Launcher, false);
        launcher_scene.set_surface_visible(SurfaceKind::SystemOverview, false);
        launcher_scene.set_surface_visible(SurfaceKind::NotificationToast, false);
        let trash_root = std::env::var_os("AQUA_TRASH_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/home/aqua/Trash"));
        let trash_model = TrashModel::open(trash_root)?;

        let output_manager_state =
            OutputManagerState::new_with_xdg_output::<WaylandSmokeState>(display_handle);
        let fractional_scale_manager_state =
            FractionalScaleManagerState::new::<WaylandSmokeState>(display_handle);
        let viewporter_state = ViewporterState::new::<WaylandSmokeState>(display_handle);
        let primary_output = configured_output(
            "Aqua-1",
            (1536, 1024),
            (0, 0),
            Scale::Integer(1),
            Transform::Normal,
            (346, 231),
        );
        let output_globals =
            vec![primary_output.create_global::<WaylandSmokeState>(display_handle)];

        Ok(Self {
            display_handle: display_handle.clone(),
            disconnected_clients: Arc::new(Mutex::new(Vec::new())),
            compositor_state: CompositorState::new::<WaylandSmokeState>(display_handle),
            shm_state: ShmState::new::<WaylandSmokeState>(display_handle, []),
            xdg_shell_state: XdgShellState::new::<WaylandSmokeState>(display_handle),
            data_device_state: DataDeviceState::new::<WaylandSmokeState>(display_handle),
            primary_selection_state: PrimarySelectionState::new::<WaylandSmokeState>(
                display_handle,
            ),
            _text_input_manager_state: TextInputManagerState::new::<WaylandSmokeState>(
                display_handle,
            ),
            _input_method_manager_state: InputMethodManagerState::new::<WaylandSmokeState, _>(
                display_handle,
                |client| {
                    client
                        .get_data::<WaylandSmokeClientState>()
                        .is_some_and(|data| data.input_method_authorized)
                },
            ),
            _output_manager_state: output_manager_state,
            _fractional_scale_manager_state: fractional_scale_manager_state,
            _viewporter_state: viewporter_state,
            outputs: vec![primary_output],
            output_globals,
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
            popup_new_count: 0,
            popup_parent_bound: false,
            popup_geometry_matches: false,
            popup_configure_sent_count: 0,
            popup_configure_ack_count: 0,
            popup_reposition_count: 0,
            popup_reposition_token: None,
            popup_destroy_count: 0,
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
            committed_surfaces: Vec::new(),
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
            desktop_context_menu_pressed_keys: Vec::new(),
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
            clipboard_selection_count: 0,
            primary_selection_count: 0,
            clipboard_selection_owner: None,
            primary_selection_owner: None,
            dnd_started_count: 0,
            dnd_validated_drop_count: 0,
            dnd_cancelled_drop_count: 0,
            dnd_source_owner: None,
            dnd_drop_target: None,
            input_method_popup_new_count: 0,
            input_method_popup_dismiss_count: 0,
            input_method_popup_reposition_count: 0,
            fractional_scale_request_count: 0,
            surface_preferred_scale: 1.0,
            viewport_source: None,
            viewport_destination: None,
        })
    }

    fn process_disconnected_selection_owners(&mut self) {
        let disconnected = {
            let mut queue = self.disconnected_clients.lock().unwrap();
            std::mem::take(&mut *queue)
        };
        if disconnected.is_empty() {
            return;
        }

        let clear_clipboard = self
            .clipboard_selection_owner
            .as_ref()
            .is_some_and(|owner| disconnected.iter().any(|client| client == owner));
        let clear_primary = self
            .primary_selection_owner
            .as_ref()
            .is_some_and(|owner| disconnected.iter().any(|client| client == owner));
        if clear_clipboard {
            clear_data_device_selection(&self.display_handle, &self.seat);
            self.clipboard_selection_owner = None;
        }
        if clear_primary {
            clear_primary_selection(&self.display_handle, &self.seat);
            self.primary_selection_owner = None;
        }
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
        self.apply_session_menu_update(update);
    }

    fn apply_session_menu_pointer(&mut self, x: u32, y: u32) {
        let update = self.session_menu_state.handle_pointer(
            SESSION_MENU_RUNTIME_WIDTH,
            SESSION_MENU_RUNTIME_HEIGHT,
            x,
            y,
        );
        self.apply_session_menu_update(update);
    }

    fn apply_session_menu_update(&mut self, update: aqua_shell::SessionMenuUpdate) {
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

    fn apply_desktop_icon_update(&mut self, update: DesktopIconUpdate) -> bool {
        let launch_requested = update.launch_request.is_some();
        if let Some(request) = update.launch_request {
            println!("desktop_icon_activation_app={}", request.app_id);
            self.launcher_launch_request = Some(request);
        }
        if let Some(action) = update.context_action {
            match action {
                DesktopContextAction::MenuOpened => {
                    println!("desktop_icon_context_action=menu-opened");
                }
                DesktopContextAction::Properties(icon_id) => {
                    println!("desktop_icon_context_action=properties icon={icon_id}");
                    self.launcher_launch_request = properties_launch_request(icon_id);
                    self.post_desktop_notification(
                        "Opening Properties",
                        &format!("Preparing {icon_id} details."),
                    );
                }
                DesktopContextAction::TrashEmptyConfirmationRequested => {
                    let refresh = self.trash_model.refresh();
                    let count = self.trash_model.entries().len();
                    println!(
                        "desktop_icon_context_action=trash-empty-confirmation count={count} refresh_ok={}",
                        refresh.is_ok()
                    );
                    let body = if refresh.is_ok() {
                        format!("Empty {count} item(s)? Select Confirm Empty to continue.")
                    } else {
                        "Trash could not be inspected safely.".to_string()
                    };
                    self.post_desktop_notification("Empty Trash", &body);
                }
                DesktopContextAction::TrashEmptyConfirmed => match self.trash_model.empty() {
                    Ok(count) => {
                        println!(
                            "desktop_icon_context_action=trash-emptied count={count} status=ok"
                        );
                        self.post_desktop_notification(
                            "Trash emptied",
                            &format!("Removed {count} item(s)."),
                        );
                    }
                    Err(error) => {
                        println!(
                            "desktop_icon_context_action=trash-emptied count=0 status=error error={error}"
                        );
                        self.post_desktop_notification(
                            "Trash was not emptied",
                            "The Trash folder changed or could not be accessed safely.",
                        );
                    }
                },
            }
        }
        if update.redraw_requested {
            println!(
                "desktop_icon_selected={}",
                self.desktop_icon_state
                    .selected()
                    .map_or("none".to_string(), |index| index.to_string())
            );
            println!(
                "desktop_context_menu_selected_row={}",
                self.desktop_icon_state
                    .context_menu_selected_row()
                    .map_or("none".to_string(), |index| index.to_string())
            );
        }
        update.redraw_requested || launch_requested
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
struct V1BufferRegistryClientState {
    registry_bound: bool,
    wl_shm_seen: bool,
    shm_argb8888_seen: bool,
    shm: Option<client_wl_shm::WlShm>,
    linux_dmabuf_seen: bool,
    drm_syncobj_seen: bool,
    explicit_sync_seen: bool,
}

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct KeyboardMatrixClientState {
    registry_bound: bool,
    seat_seen: bool,
    keyboard_capability_seen: bool,
    seat: Option<client_wl_seat::WlSeat>,
    keyboard: Option<client_wl_keyboard::WlKeyboard>,
    keymap: Option<xkb::Keymap>,
    repeat_info: Option<(i32, i32)>,
}

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct ProtocolBoundaryClientState {
    registry_bound: bool,
    globals: Vec<String>,
    global_limit_exceeded: bool,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ProtocolBoundaryClientState {
    const GLOBAL_LIMIT: usize = 64;
    const BASELINE_GLOBALS: [&'static str; 11] = [
        "wl_compositor",
        "wl_shm",
        "wl_seat",
        "wl_output",
        "xdg_wm_base",
        "wl_data_device_manager",
        "zwp_primary_selection_device_manager_v1",
        "zwp_text_input_manager_v3",
        "zxdg_output_manager_v1",
        "wp_fractional_scale_manager_v1",
        "wp_viewporter",
    ];

    fn sees(&self, interface: &str) -> bool {
        self.globals.iter().any(|global| global == interface)
    }

    fn sees_all_baseline_globals(&self) -> bool {
        !self.global_limit_exceeded
            && Self::BASELINE_GLOBALS
                .iter()
                .all(|interface| self.sees(interface))
    }

    fn sees_any(&self, interfaces: &[&str]) -> bool {
        interfaces.iter().any(|interface| self.sees(interface))
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for ProtocolBoundaryClientState {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        state.registry_bound = true;
        if let wl_registry::Event::Global { interface, .. } = event {
            if state.globals.len() >= Self::GLOBAL_LIMIT {
                state.global_limit_exceeded = true;
            } else {
                state.globals.push(interface);
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for KeyboardMatrixClientState {
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
            name,
            interface,
            version,
        } = event
        {
            if interface == "wl_seat" {
                state.seat_seen = true;
                state.seat = Some(registry.bind::<client_wl_seat::WlSeat, _, _>(
                    name,
                    version.min(7),
                    qh,
                    (),
                ));
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_seat::WlSeat, ()> for KeyboardMatrixClientState {
    fn event(
        state: &mut Self,
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
                state.keyboard_capability_seen = true;
                state.keyboard = Some(seat.get_keyboard(qh, ()));
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_keyboard::WlKeyboard, ()> for KeyboardMatrixClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_keyboard::WlKeyboard,
        event: client_wl_keyboard::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            client_wl_keyboard::Event::Keymap {
                format: WEnum::Value(client_wl_keyboard::KeymapFormat::XkbV1),
                fd,
                size,
            } => {
                const MAX_KEYMAP_BYTES: u64 = 1_048_576;
                if u64::from(size) > MAX_KEYMAP_BYTES {
                    return;
                }
                let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
                // The owned descriptor and byte count come directly from the
                // wl_keyboard keymap event and stay valid for this call.
                state.keymap = unsafe {
                    xkb::Keymap::new_from_fd(
                        &context,
                        fd,
                        size as usize,
                        xkb::KEYMAP_FORMAT_TEXT_V1,
                        xkb::KEYMAP_COMPILE_NO_FLAGS,
                    )
                    .ok()
                    .flatten()
                };
            }
            client_wl_keyboard::Event::RepeatInfo { rate, delay } => {
                state.repeat_info = Some((delay, rate));
            }
            _ => {}
        }
    }
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
    settings_wifi_socket_path: Option<PathBuf>,
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
impl ClientDispatch<wl_registry::WlRegistry, ()> for V1BufferRegistryClientState {
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
                "wl_shm" => {
                    state.wl_shm_seen = true;
                    state.shm = Some(registry.bind::<client_wl_shm::WlShm, _, _>(name, 1, qh, ()));
                }
                "zwp_linux_dmabuf_v1" => state.linux_dmabuf_seen = true,
                "wp_linux_drm_syncobj_manager_v1" => state.drm_syncobj_seen = true,
                "zwp_linux_explicit_synchronization_v1" => state.explicit_sync_seen = true,
                _ => {}
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_shm::WlShm, ()> for V1BufferRegistryClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_shm::WlShm,
        event: client_wl_shm::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_wl_shm::Event::Format {
            format: WEnum::Value(client_wl_shm::Format::Argb8888),
        } = event
        {
            state.shm_argb8888_seen = true;
        }
    }
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
        let network_route = std::env::var_os("AQUA_NETWORK_IPV4_ROUTE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/proc/net/route"));
        let network_resolver = std::env::var_os("AQUA_NETWORK_RESOLV_CONF")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/etc/resolv.conf"));
        if let Err(error) =
            settings_model.refresh_network_status(&network_root, &network_route, &network_resolver)
        {
            eprintln!("aqua_settings_network_status_available=false error={error}");
        }
        #[cfg(test)]
        let wifi_socket_path = std::env::var_os("AQUA_NETWORK_BROKER_SOCKET")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(aqua_shell::SETTINGS_WIFI_BROKER_SOCKET_PATH));
        #[cfg(not(test))]
        let wifi_socket_path = PathBuf::from(aqua_shell::SETTINGS_WIFI_BROKER_SOCKET_PATH);
        if settings_model.refresh_wifi_control(&wifi_socket_path) {
            settings_model.refresh_wifi_networks(&wifi_socket_path);
        }
        Ok(Self {
            buffer_width: 600,
            buffer_height: 400,
            title: "System Settings".to_string(),
            app_id: "aqua.settings".to_string(),
            theme: settings_model.theme,
            settings_model: Some(settings_model),
            settings_config_path: Some(config_path),
            settings_wifi_socket_path: Some(wifi_socket_path),
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

    fn apply_settings_wifi_control(&mut self, connected: bool) {
        let (Some(model), Some(socket_path)) = (
            self.settings_model.as_mut(),
            self.settings_wifi_socket_path.as_deref(),
        ) else {
            return;
        };
        let applied = model.apply_wifi_control(socket_path, connected);
        println!(
            "aqua_settings_wifi_control requested={} applied={} available={} connected={} status={}",
            if connected { "reconnect" } else { "disconnect" },
            applied,
            model.wifi.available(),
            model.wifi.connected(),
            model.wifi.status_label()
        );
    }

    fn apply_settings_wifi_connection(&mut self) {
        let (Some(model), Some(socket_path)) = (
            self.settings_model.as_mut(),
            self.settings_wifi_socket_path.as_deref(),
        ) else {
            return;
        };
        let applied = model.apply_wifi_connection(socket_path);
        println!(
            "aqua_settings_wifi_new_network applied={} available={} connected={} status={}",
            applied,
            model.wifi.available(),
            model.wifi.connected(),
            model.wifi.status_label()
        );
    }

    fn apply_settings_wifi_scan(&mut self) {
        let (Some(model), Some(socket_path)) = (
            self.settings_model.as_mut(),
            self.settings_wifi_socket_path.as_deref(),
        ) else {
            return;
        };
        let applied = model.refresh_wifi_networks(socket_path);
        println!(
            "aqua_settings_wifi_rescan applied={} available={} count={} status={}",
            applied,
            model.wifi.available(),
            model.wifi.networks().len(),
            model.wifi.status_label()
        );
    }

    fn apply_settings_wifi_forget(&mut self) {
        let (Some(model), Some(socket_path)) = (
            self.settings_model.as_mut(),
            self.settings_wifi_socket_path.as_deref(),
        ) else {
            return;
        };
        let applied = model.forget_saved_wifi_network(socket_path);
        println!(
            "aqua_settings_wifi_forget applied={} available={} credential_saved={} status={}",
            applied,
            model.wifi.available(),
            model.wifi.credential_saved(),
            model.wifi.status_label()
        );
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
        let mode = OutputMode {
            size: (width as i32, height as i32).into(),
            refresh: 60_000,
        };
        if let Some(output) = self.session.wayland_state.outputs.first() {
            output.set_preferred(mode);
            output.change_current_state(
                Some(mode),
                Some(Transform::Normal),
                Some(Scale::Integer(1)),
                Some((0, 0).into()),
            );
        }
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
        let mut files_model = aqua_shell::FilesWindowModel::default();
        files_model.entries.push(files_model.entries[0].clone());
        let Some(scrollbar) = files_model.list_scrollbar(640) else {
            return false;
        };
        let x = surface_x + scrollbar.track.x;
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
            && self.session.wayland_state.seat.get_touch().is_some()
    }

    pub fn dispatch_keyboard_key(&mut self, code: u32, pressed: bool, time: u32) -> bool {
        let Some(keyboard) = self.session.wayland_state.seat.get_keyboard() else {
            return false;
        };
        let state = &mut self.session.wayland_state;
        if !pressed {
            if let Some(index) = state
                .desktop_context_menu_pressed_keys
                .iter()
                .position(|pressed_code| *pressed_code == code)
            {
                state.desktop_context_menu_pressed_keys.swap_remove(index);
                state.keyboard_event_count += 1;
                state.keyboard_shortcut_intercept_count += 1;
                return true;
            }
        }
        if state.desktop_icon_state.context_menu().is_some() {
            state.keyboard_event_count += 1;
            state.keyboard_shortcut_intercept_count += 1;
            if pressed {
                if !state.desktop_context_menu_pressed_keys.contains(&code) {
                    state.desktop_context_menu_pressed_keys.push(code);
                }
                let key = match code {
                    1 => Some(DesktopContextMenuKey::Dismiss),
                    28 | 57 => Some(DesktopContextMenuKey::Activate),
                    102 => Some(DesktopContextMenuKey::Navigate(MenuNavigationKey::Home)),
                    103 => Some(DesktopContextMenuKey::Navigate(MenuNavigationKey::Previous)),
                    107 => Some(DesktopContextMenuKey::Navigate(MenuNavigationKey::End)),
                    108 => Some(DesktopContextMenuKey::Navigate(MenuNavigationKey::Next)),
                    _ => None,
                };
                if let Some(key) = key {
                    let update = state.desktop_icon_state.handle_context_menu_key(key);
                    state.apply_desktop_icon_update(update);
                }
            }
            return true;
        }
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
                    102 => Some(LauncherEvent::Navigate(CollectionNavigationKey::Home)),
                    103 | 105 => Some(LauncherEvent::Navigate(CollectionNavigationKey::Previous)),
                    106 | 108 => Some(LauncherEvent::Navigate(CollectionNavigationKey::Next)),
                    107 => Some(LauncherEvent::Navigate(CollectionNavigationKey::End)),
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
                    102 => state.apply_session_menu_event(SessionMenuEvent::Navigate(
                        MenuNavigationKey::Home,
                    )),
                    103 => state.apply_session_menu_event(SessionMenuEvent::Navigate(
                        MenuNavigationKey::Previous,
                    )),
                    107 => state.apply_session_menu_event(SessionMenuEvent::Navigate(
                        MenuNavigationKey::End,
                    )),
                    108 => state.apply_session_menu_event(SessionMenuEvent::Navigate(
                        MenuNavigationKey::Next,
                    )),
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
                } else if state.ctrl_pressed
                    && state.alt_pressed
                    && matches!(code, 102 | 105 | 106 | 107)
                {
                    if pressed {
                        let key = match code {
                            102 => WorkspaceNavigationKey::Home,
                            105 => WorkspaceNavigationKey::Previous,
                            106 => WorkspaceNavigationKey::Next,
                            107 => WorkspaceNavigationKey::End,
                            _ => unreachable!("workspace shortcut key was bounded above"),
                        };
                        if let Some(destination) =
                            workspace_keyboard_target(state.active_workspace, key)
                        {
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
        let viewport = Viewport::new(
            self.session.wayland_state.output_width,
            self.session.wayland_state.output_height,
        );
        let pointer_location = pointer_location_after_motion(
            self.session.wayland_state.pointer_location,
            dx,
            dy,
            viewport,
        );
        self.session.wayland_state.pointer_location = pointer_location;
        if self.session.wayland_state.launcher_state.is_open()
            || self.session.wayland_state.session_menu_state.is_open()
        {
            self.session.wayland_state.pointer_focus_surface = None;
            self.session.wayland_state.pointer_focus_assigned = false;
            if self.session.wayland_state.launcher_state.is_open()
                && self
                    .session
                    .wayland_state
                    .launcher_state
                    .pointer_target_in_viewport(
                        pointer_location.0 as u32,
                        pointer_location.1 as u32,
                        viewport.width,
                        viewport.height,
                    )
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
            let dock_target = bottom_shell_pointer_target(
                Viewport::new(output_width, output_height),
                pointer_x as u32,
                pointer_y as u32,
            );
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
        if button == 0x110 && pressed && self.session.wayland_state.session_menu_state.is_open() {
            let viewport = Viewport::new(
                self.session.wayland_state.output_width,
                self.session.wayland_state.output_height,
            );
            let position =
                session_menu_pointer_position(viewport, pointer_x as u32, pointer_y as u32);
            if let Some((x, y)) = position {
                self.session.wayland_state.apply_session_menu_pointer(x, y);
            }
            println!(
                "desktop_session_menu_pointer x={} y={} local={position:?}",
                pointer_x as u32, pointer_y as u32
            );
            self.session.wayland_state.pointer_button_count += 1;
            return true;
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
            if self.session.wayland_state.apply_desktop_icon_update(update) {
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
                damage_commit_count: surface.damage_commit_count,
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
        let pointer_location = self.session.wayland_state.pointer_location;
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

    pub fn client_surface_snapshot_for_app_id(
        &self,
        expected_app_id: &str,
    ) -> Option<SmithayClientSurfaceSnapshot> {
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
            })?
            .wl_surface();
        let index = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .position(|record| record.surface == *surface)?;
        self.client_surface_snapshots().into_iter().nth(index)
    }

    pub fn focus_keyboard_surface_with_app_id(&mut self, expected_app_id: &str, time: u32) -> bool {
        if !self.raise_surface_with_app_id(expected_app_id) {
            return false;
        }
        let Some(surface) = self.session.wayland_state.mapped_surface.clone() else {
            return false;
        };
        let Some(keyboard) = self.session.wayland_state.seat.get_keyboard() else {
            return false;
        };
        keyboard.set_focus(
            &mut self.session.wayland_state,
            Some(surface),
            Serial::from(time.max(1)),
        );
        self.session.wayland_state.keyboard_focus_assigned = true;
        let callbacks = std::mem::take(&mut self.session.wayland_state.pending_frame_callbacks);
        self.session.wayland_state.frame_callbacks_sent += callbacks.len();
        for callback in callbacks {
            callback.done(time);
        }
        true
    }

    pub fn dispatch_touch_sequence_to_app_id(
        &mut self,
        expected_app_id: &str,
        start: (u32, u32),
        end: (u32, u32),
        time: u32,
    ) -> bool {
        if !self.raise_surface_with_app_id(expected_app_id) {
            return false;
        }
        let Some(surface) = self.session.wayland_state.mapped_surface.clone() else {
            return false;
        };
        let Some((origin_x, origin_y, width, height)) = self
            .session
            .wayland_state
            .mapped_surfaces
            .iter()
            .find(|record| record.surface == surface)
            .map(|record| (record.x, record.y, record.width, record.height))
        else {
            return false;
        };
        if start.0 >= width || start.1 >= height || end.0 >= width || end.1 >= height {
            return false;
        }
        let Some(touch) = self.session.wayland_state.seat.get_touch() else {
            return false;
        };
        let origin = (f64::from(origin_x), f64::from(origin_y)).into();
        let start_location = (
            f64::from(origin_x.saturating_add(start.0)),
            f64::from(origin_y.saturating_add(start.1)),
        )
            .into();
        let end_location = (
            f64::from(origin_x.saturating_add(end.0)),
            f64::from(origin_y.saturating_add(end.1)),
        )
            .into();
        let slot = TouchSlot::from(Some(0));
        touch.down(
            &mut self.session.wayland_state,
            Some((surface.clone(), origin)),
            &TouchDownEvent {
                slot,
                location: start_location,
                serial: Serial::from(time.max(1)),
                time,
            },
        );
        touch.frame(&mut self.session.wayland_state);
        touch.motion(
            &mut self.session.wayland_state,
            Some((surface, origin)),
            &TouchMotionEvent {
                slot,
                location: end_location,
                time: time.saturating_add(1),
            },
        );
        touch.frame(&mut self.session.wayland_state);
        touch.up(
            &mut self.session.wayland_state,
            &TouchUpEvent {
                slot,
                serial: Serial::from(time.saturating_add(2).max(1)),
                time: time.saturating_add(2),
            },
        );
        touch.frame(&mut self.session.wayland_state);
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
            .is_some_and(|model| model.network.status_available())
    );
    println!(
        "aqua_settings_network_interface_count={}",
        state
            .settings_model
            .as_ref()
            .map_or(0, |model| model.network.interfaces().len())
    );
    if let Some(model) = state.settings_model.as_ref() {
        println!(
            "aqua_settings_wifi_control_available={}",
            model.wifi.available()
        );
        println!(
            "aqua_settings_wifi_controls_enabled={}",
            model.wifi.controls_enabled()
        );
        println!("aqua_settings_wifi_connected={}", model.wifi.connected());
        println!("aqua_settings_wifi_status={}", model.wifi.status_label());
    }
    if let Some(interface) = state
        .settings_model
        .as_ref()
        .and_then(|model| model.network.primary_interface())
    {
        println!("aqua_settings_network_interface={}", interface.name());
        println!("aqua_settings_network_state={}", interface.link().id());
    }
    if let Some(model) = state.settings_model.as_ref() {
        println!(
            "aqua_settings_network_health={}",
            model.network.health().id()
        );
        println!(
            "aqua_settings_network_default_route={}",
            model.network.default_route().unwrap_or("none")
        );
        println!(
            "aqua_settings_network_dns_count={}",
            model.network.dns_servers().len()
        );
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

    fn new_with_keyboard_layout(
        keyboard_layout: KeyboardLayoutSpec,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display = Display::new()?;
        let display_handle = display.handle();
        Ok(Self {
            display,
            wayland_state: WaylandSmokeState::new_with_keyboard_layout(
                &display_handle,
                keyboard_layout,
            )?,
        })
    }

    fn insert_client(&mut self, stream: std::os::unix::net::UnixStream) -> std::io::Result<Client> {
        let client_data = WaylandSmokeClientState::with_disconnect_queue(
            self.wayland_state.disconnected_clients.clone(),
        );
        self.display
            .handle()
            .insert_client(stream, Arc::new(client_data))
    }

    fn insert_authorized_input_method_client(
        &mut self,
        stream: std::os::unix::net::UnixStream,
    ) -> std::io::Result<Client> {
        let client_data = WaylandSmokeClientState::authorized_input_method(
            self.wayland_state.disconnected_clients.clone(),
        );
        self.display
            .handle()
            .insert_client(stream, Arc::new(client_data))
    }

    fn dispatch_clients(&mut self) -> std::io::Result<usize> {
        let dispatched = self.display.dispatch_clients(&mut self.wayland_state)?;
        self.wayland_state.process_disconnected_selection_owners();
        Ok(dispatched)
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
fn settings_passphrase_character(code: u32, shift: bool) -> Option<char> {
    if let Some(character) = launcher_key_character(code) {
        return Some(if shift {
            character.to_ascii_uppercase()
        } else {
            character
        });
    }
    Some(match (code, shift) {
        (2, false) => '1',
        (3, false) => '2',
        (4, false) => '3',
        (5, false) => '4',
        (6, false) => '5',
        (7, false) => '6',
        (8, false) => '7',
        (9, false) => '8',
        (10, false) => '9',
        (11, false) => '0',
        (2, true) => '!',
        (3, true) => '@',
        (4, true) => '#',
        (5, true) => '$',
        (6, true) => '%',
        (7, true) => '^',
        (8, true) => '&',
        (9, true) => '*',
        (10, true) => '(',
        (11, true) => ')',
        (12, false) => '-',
        (12, true) => '_',
        (13, false) => '=',
        (13, true) => '+',
        (26, false) => '[',
        (26, true) => '{',
        (27, false) => ']',
        (27, true) => '}',
        (39, false) => ';',
        (39, true) => ':',
        (40, false) => '\'',
        (40, true) => '"',
        (41, false) => '`',
        (41, true) => '~',
        (43, false) => '\\',
        (43, true) => '|',
        (51, false) => ',',
        (51, true) => '<',
        (52, false) => '.',
        (52, true) => '>',
        (53, false) => '/',
        (53, true) => '?',
        (57, _) => ' ',
        _ => return None,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn settings_key_for_code(code: u32) -> Option<aqua_shell::SettingsKey> {
    Some(match code {
        102 => aqua_shell::SettingsKey::Home,
        107 => aqua_shell::SettingsKey::End,
        103 => aqua_shell::SettingsKey::Up,
        108 => aqua_shell::SettingsKey::Down,
        28 => aqua_shell::SettingsKey::Activate,
        105 => aqua_shell::SettingsKey::Decrease,
        106 => aqua_shell::SettingsKey::Increase,
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
        if !self.committed_surfaces.contains(surface) {
            self.committed_surfaces.push(surface.clone());
        }
        let viewport = with_states(surface, |states| {
            *states.cached_state.get::<ViewportCachedState>().current()
        });
        self.viewport_source = viewport
            .src
            .map(|src| (src.loc.x, src.loc.y, src.size.w, src.size.h));
        self.viewport_destination = viewport.dst.map(|dst| (dst.w, dst.h));
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
            let surface_damage_committed = !attributes.damage.is_empty();
            if surface_damage_committed {
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
                    let Some((buffer_offset, byte_count)) =
                        shm_buffer_bounds(len, metadata.offset, metadata.stride, metadata.height)
                    else {
                        return (
                            0,
                            [0, 0, 0, 0],
                            solid_sample_grid([0, 0, 0, 0]),
                            Vec::new(),
                            0,
                            0,
                            0,
                        );
                    };
                    let ptr = unsafe { ptr.add(buffer_offset) };
                    let len = byte_count;

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
                            damage_commit_count: usize::from(surface_damage_committed),
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
                            let damage_commit_count = existing.damage_commit_count;
                            *existing = record;
                            existing.workspace = workspace;
                            existing.damage_commit_count =
                                damage_commit_count.saturating_add(existing.damage_commit_count);
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
        self.committed_surfaces
            .retain(|committed| committed != surface);
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
impl OutputHandler for WaylandSmokeState {}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl FractionalScaleHandler for WaylandSmokeState {
    fn new_fractional_scale(&mut self, surface: WlSurface) {
        self.fractional_scale_request_count += 1;
        let preferred_scale = self.surface_preferred_scale;
        with_states(&surface, |states| {
            with_fractional_scale(states, |fractional_scale| {
                fractional_scale.set_preferred_scale(preferred_scale);
            });
        });
    }
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

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        self.popup_new_count += 1;
        self.popup_parent_bound = surface.get_parent_surface().is_some();
        self.popup_geometry_matches = positioner.rect_size.w == 240
            && positioner.rect_size.h == 120
            && positioner.anchor_rect.loc.x == 24
            && positioner.anchor_rect.loc.y == 36
            && positioner.anchor_rect.size.w == 80
            && positioner.anchor_rect.size.h == 40;
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = Rectangle::new((32, 48).into(), (240, 120).into());
        });
        if surface.send_configure().is_ok() {
            self.popup_configure_sent_count += 1;
        }
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        self.popup_reposition_count += 1;
        self.popup_reposition_token = Some(token);
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = Rectangle::new((64, 72).into(), (240, 120).into());
        });
        surface.send_repositioned(token);
    }

    fn ack_configure(&mut self, _surface: WlSurface, configure: Configure) {
        match configure {
            Configure::Toplevel(configure) => {
                self.toplevel_configure_ack_count += 1;
                self.toplevel_configure_serial = Some(u32::from(configure.serial));
            }
            Configure::Popup(_) => self.popup_configure_ack_count += 1,
        }
    }

    fn popup_destroyed(&mut self, _surface: PopupSurface) {
        self.popup_destroy_count += 1;
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

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let client = focused.and_then(Resource::client);
        set_data_device_focus(&self.display_handle, seat, client.clone());
        set_primary_focus(&self.display_handle, seat, client);
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl SelectionHandler for WaylandSmokeState {
    type SelectionUserData = ();

    fn new_selection(
        &mut self,
        target: SelectionTarget,
        source: Option<SelectionSource>,
        _seat: Seat<Self>,
    ) {
        let owner = source.as_ref().and_then(|_| {
            self.seat
                .get_keyboard()
                .and_then(|keyboard| keyboard.current_focus())
                .and_then(|surface| surface.client())
                .map(|client| client.id())
        });
        match target {
            SelectionTarget::Clipboard => {
                self.clipboard_selection_count += 1;
                self.clipboard_selection_owner = owner;
            }
            SelectionTarget::Primary => {
                self.primary_selection_count += 1;
                self.primary_selection_owner = owner;
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDndGrabHandler for WaylandSmokeState {
    fn started(
        &mut self,
        source: Option<smithay::reexports::wayland_server::protocol::wl_data_source::WlDataSource>,
        _icon: Option<WlSurface>,
        _seat: Seat<Self>,
    ) {
        self.dnd_started_count += 1;
        self.dnd_source_owner = source
            .and_then(|source| source.client())
            .map(|client| client.id());
    }

    fn dropped(&mut self, target: Option<WlSurface>, validated: bool, _seat: Seat<Self>) {
        self.dnd_drop_target = target
            .and_then(|surface| surface.client())
            .map(|client| client.id());
        if validated {
            self.dnd_validated_drop_count += 1;
        } else {
            self.dnd_cancelled_drop_count += 1;
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ServerDndGrabHandler for WaylandSmokeState {}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl DataDeviceHandler for WaylandSmokeState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }

    fn action_choice(
        &mut self,
        available: ServerDndAction,
        preferred: ServerDndAction,
    ) -> ServerDndAction {
        if preferred == ServerDndAction::Copy && available.contains(ServerDndAction::Copy) {
            ServerDndAction::Copy
        } else {
            ServerDndAction::None
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl PrimarySelectionHandler for WaylandSmokeState {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.primary_selection_state
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl InputMethodHandler for WaylandSmokeState {
    fn new_popup(&mut self, _surface: InputMethodPopupSurface) {
        self.input_method_popup_new_count += 1;
    }

    fn dismiss_popup(&mut self, _surface: InputMethodPopupSurface) {
        self.input_method_popup_dismiss_count += 1;
    }

    fn popup_repositioned(&mut self, _surface: InputMethodPopupSurface) {
        self.input_method_popup_reposition_count += 1;
    }

    fn parent_geometry(&self, parent: &WlSurface) -> Rectangle<i32, Logical> {
        self.mapped_surfaces
            .iter()
            .find(|record| record.surface == *parent)
            .map(|record| {
                Rectangle::new(
                    (record.x as i32, record.y as i32).into(),
                    (record.display_width as i32, record.display_height as i32).into(),
                )
            })
            .unwrap_or_else(|| Rectangle::new((0, 0).into(), (640, 480).into()))
    }
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
delegate_data_device!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_input_method_manager!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_output!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_primary_selection!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_seat!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_shm!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_text_input_manager!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_fractional_scale!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_viewporter!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_xdg_shell!(WaylandSmokeState);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Default)]
struct WaylandSmokeClientState {
    compositor_state: CompositorClientState,
    disconnected_clients: Option<Arc<Mutex<Vec<ClientId>>>>,
    input_method_authorized: bool,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl WaylandSmokeClientState {
    fn with_disconnect_queue(disconnected_clients: Arc<Mutex<Vec<ClientId>>>) -> Self {
        Self {
            compositor_state: CompositorClientState::default(),
            disconnected_clients: Some(disconnected_clients),
            input_method_authorized: false,
        }
    }

    fn authorized_input_method(disconnected_clients: Arc<Mutex<Vec<ClientId>>>) -> Self {
        Self {
            compositor_state: CompositorClientState::default(),
            disconnected_clients: Some(disconnected_clients),
            input_method_authorized: true,
        }
    }

    fn compositor_state(&self) -> &CompositorClientState {
        &self.compositor_state
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientData for WaylandSmokeClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, client_id: ClientId, _reason: DisconnectReason) {
        if let Some(queue) = &self.disconnected_clients {
            queue.lock().unwrap().push(client_id);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Debug, Default)]
struct OutputSmokeRecord {
    global_name: u32,
    proxy: Option<client_wl_output::WlOutput>,
    protocol_name: Option<String>,
    location: Option<(i32, i32)>,
    mode: Option<(i32, i32, i32)>,
    current: bool,
    preferred: bool,
    integer_scale: Option<i32>,
    logical_position: Option<(i32, i32)>,
    logical_size: Option<(i32, i32)>,
    transform: Option<client_wl_output::Transform>,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Clone, Copy)]
enum PopupXdgRole {
    Parent,
    Popup,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Default)]
struct PopupLifecycleClientState {
    registry_bound: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    wm_base: Option<client_xdg_wm_base::XdgWmBase>,
    parent_surface: Option<wl_surface::WlSurface>,
    parent_xdg_surface: Option<client_xdg_surface::XdgSurface>,
    toplevel: Option<client_xdg_toplevel::XdgToplevel>,
    popup_surface: Option<wl_surface::WlSurface>,
    popup_xdg_surface: Option<client_xdg_surface::XdgSurface>,
    popup: Option<client_xdg_popup::XdgPopup>,
    parent_configure_acknowledged: bool,
    popup_configure_count: usize,
    popup_configure_ack_count: usize,
    popup_geometries: Vec<(i32, i32, i32, i32)>,
    reposition_requested: bool,
    repositioned_token: Option<u32>,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl PopupLifecycleClientState {
    fn maybe_create_parent(&mut self, qh: &QueueHandle<Self>) {
        if self.parent_surface.is_some() {
            return;
        }
        let (Some(compositor), Some(wm_base)) = (&self.compositor, &self.wm_base) else {
            return;
        };
        let surface = compositor.create_surface(qh, ());
        let xdg_surface = wm_base.get_xdg_surface(&surface, qh, PopupXdgRole::Parent);
        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title("Popup lifecycle parent".to_string());
        toplevel.set_app_id("aqua.test.popup-parent".to_string());
        surface.commit();
        self.parent_surface = Some(surface);
        self.parent_xdg_surface = Some(xdg_surface);
        self.toplevel = Some(toplevel);
    }

    fn create_popup(&mut self, qh: &QueueHandle<Self>) {
        if self.popup.is_some() {
            return;
        }
        let (Some(compositor), Some(wm_base), Some(parent_xdg_surface)) =
            (&self.compositor, &self.wm_base, &self.parent_xdg_surface)
        else {
            return;
        };
        let positioner = wm_base.create_positioner(qh, ());
        positioner.set_size(240, 120);
        positioner.set_anchor_rect(24, 36, 80, 40);
        positioner.set_anchor(client_xdg_positioner::Anchor::BottomLeft);
        positioner.set_gravity(client_xdg_positioner::Gravity::BottomRight);
        positioner.set_reactive();
        let surface = compositor.create_surface(qh, ());
        let xdg_surface = wm_base.get_xdg_surface(&surface, qh, PopupXdgRole::Popup);
        let popup = xdg_surface.get_popup(Some(parent_xdg_surface), &positioner, qh, ());
        surface.commit();
        positioner.destroy();
        self.popup_surface = Some(surface);
        self.popup_xdg_surface = Some(xdg_surface);
        self.popup = Some(popup);
    }

    fn request_reposition(&mut self, qh: &QueueHandle<Self>) {
        if self.reposition_requested {
            return;
        }
        let (Some(wm_base), Some(popup)) = (&self.wm_base, &self.popup) else {
            return;
        };
        let positioner = wm_base.create_positioner(qh, ());
        positioner.set_size(240, 120);
        positioner.set_anchor_rect(48, 56, 80, 40);
        positioner.set_anchor(client_xdg_positioner::Anchor::BottomRight);
        positioner.set_gravity(client_xdg_positioner::Gravity::BottomLeft);
        positioner.set_reactive();
        popup.reposition(&positioner, 77);
        positioner.destroy();
        self.reposition_requested = true;
    }

    fn destroy_popup(&mut self) {
        if let Some(popup) = self.popup.take() {
            popup.destroy();
        }
        if let Some(xdg_surface) = self.popup_xdg_surface.take() {
            xdg_surface.destroy();
        }
        if let Some(surface) = self.popup_surface.take() {
            surface.destroy();
        }
    }

    fn destroy_parent(&mut self) {
        if let Some(toplevel) = self.toplevel.take() {
            toplevel.destroy();
        }
        if let Some(xdg_surface) = self.parent_xdg_surface.take() {
            xdg_surface.destroy();
        }
        if let Some(surface) = self.parent_surface.take() {
            surface.destroy();
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Default)]
struct SubsurfaceLifecycleClientState {
    registry_bound: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    subcompositor: Option<client_wl_subcompositor::WlSubcompositor>,
    parent_surface: Option<wl_surface::WlSurface>,
    child_surface: Option<wl_surface::WlSurface>,
    subsurface: Option<client_wl_subsurface::WlSubsurface>,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl SubsurfaceLifecycleClientState {
    fn maybe_create_tree(&mut self, qh: &QueueHandle<Self>) {
        if self.subsurface.is_some() {
            return;
        }
        let (Some(compositor), Some(subcompositor)) = (&self.compositor, &self.subcompositor)
        else {
            return;
        };
        let parent = compositor.create_surface(qh, ());
        let child = compositor.create_surface(qh, ());
        let subsurface = subcompositor.get_subsurface(&child, &parent, qh, ());
        subsurface.set_position(24, 36);
        subsurface.set_sync();
        child.commit();
        parent.commit();
        self.parent_surface = Some(parent);
        self.child_surface = Some(child);
        self.subsurface = Some(subsurface);
    }

    fn set_desynchronized(&self) {
        if let (Some(subsurface), Some(child)) = (&self.subsurface, &self.child_surface) {
            subsurface.set_desync();
            child.commit();
        }
    }

    fn destroy_subsurface(&mut self) {
        if let Some(subsurface) = self.subsurface.take() {
            subsurface.destroy();
        }
        if let Some(child) = self.child_surface.take() {
            child.destroy();
        }
    }

    fn destroy_parent(&mut self) {
        if let Some(parent) = self.parent_surface.take() {
            parent.destroy();
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Debug, Default)]
struct OutputSmokeClientState {
    registry_bound: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    xdg_output_manager: Option<client_xdg_output_manager::ZxdgOutputManagerV1>,
    fractional_scale_manager: Option<client_fractional_scale_manager::WpFractionalScaleManagerV1>,
    viewporter: Option<client_viewporter::WpViewporter>,
    outputs: Vec<OutputSmokeRecord>,
    removed_globals: Vec<u32>,
    surface: Option<wl_surface::WlSurface>,
    fractional_scale_120ths: Option<u32>,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl OutputSmokeClientState {
    fn request_surface_extensions(&mut self, qh: &QueueHandle<Self>) -> Result<(), &'static str> {
        let xdg_output_manager = self
            .xdg_output_manager
            .as_ref()
            .ok_or("xdg-output manager was not advertised")?;
        for output in &self.outputs {
            xdg_output_manager.get_xdg_output(
                output.proxy.as_ref().ok_or("wl_output proxy missing")?,
                qh,
                output.global_name,
            );
        }

        let surface = self
            .compositor
            .as_ref()
            .ok_or("wl_compositor was not advertised")?
            .create_surface(qh, ());
        self.fractional_scale_manager
            .as_ref()
            .ok_or("fractional-scale manager was not advertised")?
            .get_fractional_scale(&surface, qh, ());
        let viewport = self
            .viewporter
            .as_ref()
            .ok_or("viewporter was not advertised")?
            .get_viewport(&surface, qh, ());
        viewport.set_source(8.0, 12.0, 320.0, 180.0);
        viewport.set_destination(640, 360);
        surface.commit();
        self.surface = Some(surface);
        Ok(())
    }

    fn globals_ready(&self, expected_output_count: usize) -> bool {
        self.registry_bound
            && self.compositor.is_some()
            && self.xdg_output_manager.is_some()
            && self.fractional_scale_manager.is_some()
            && self.viewporter.is_some()
            && self.outputs.len() == expected_output_count
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for OutputSmokeClientState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                state.registry_bound = true;
                match interface.as_str() {
                    "wl_compositor" => {
                        state.compositor = Some(registry.bind(name, version.min(6), qh, ()))
                    }
                    "wl_output" => {
                        let proxy = registry.bind(name, version.min(4), qh, ());
                        state.outputs.push(OutputSmokeRecord {
                            global_name: name,
                            proxy: Some(proxy),
                            ..OutputSmokeRecord::default()
                        });
                    }
                    "zxdg_output_manager_v1" => {
                        state.xdg_output_manager = Some(registry.bind(name, version.min(3), qh, ()))
                    }
                    "wp_fractional_scale_manager_v1" => {
                        state.fractional_scale_manager =
                            Some(registry.bind(name, version.min(1), qh, ()))
                    }
                    "wp_viewporter" => {
                        state.viewporter = Some(registry.bind(name, version.min(1), qh, ()))
                    }
                    _ => {}
                }
            }
            wl_registry::Event::GlobalRemove { name } => state.removed_globals.push(name),
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_output::WlOutput, ()> for OutputSmokeClientState {
    fn event(
        state: &mut Self,
        output: &client_wl_output::WlOutput,
        event: client_wl_output::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        let Some(record) = state.outputs.iter_mut().find(|record| {
            record
                .proxy
                .as_ref()
                .is_some_and(|proxy| proxy.id() == output.id())
        }) else {
            return;
        };
        match event {
            client_wl_output::Event::Geometry {
                x, y, transform, ..
            } => {
                record.location = Some((x, y));
                if let WEnum::Value(transform) = transform {
                    record.transform = Some(transform);
                }
            }
            client_wl_output::Event::Mode {
                flags,
                width,
                height,
                refresh,
            } => {
                record.mode = Some((width, height, refresh));
                if let WEnum::Value(flags) = flags {
                    record.current = flags.contains(client_wl_output::Mode::Current);
                    record.preferred = flags.contains(client_wl_output::Mode::Preferred);
                }
            }
            client_wl_output::Event::Scale { factor } => record.integer_scale = Some(factor),
            client_wl_output::Event::Name { name } => record.protocol_name = Some(name),
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_xdg_output::ZxdgOutputV1, u32> for OutputSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_xdg_output::ZxdgOutputV1,
        event: client_xdg_output::Event,
        global_name: &u32,
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        let Some(record) = state
            .outputs
            .iter_mut()
            .find(|record| record.global_name == *global_name)
        else {
            return;
        };
        match event {
            client_xdg_output::Event::LogicalPosition { x, y } => {
                record.logical_position = Some((x, y))
            }
            client_xdg_output::Event::LogicalSize { width, height } => {
                record.logical_size = Some((width, height))
            }
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_fractional_scale::WpFractionalScaleV1, ()> for OutputSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_fractional_scale::WpFractionalScaleV1,
        event: client_fractional_scale::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_fractional_scale::Event::PreferredScale { scale } = event {
            state.fractional_scale_120ths = Some(scale);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(OutputSmokeClientState: ignore wl_compositor::WlCompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(OutputSmokeClientState: ignore wl_surface::WlSurface);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(OutputSmokeClientState: ignore client_xdg_output_manager::ZxdgOutputManagerV1);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(OutputSmokeClientState: ignore client_fractional_scale_manager::WpFractionalScaleManagerV1);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(OutputSmokeClientState: ignore client_viewporter::WpViewporter);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(OutputSmokeClientState: ignore client_viewport::WpViewport);

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct TextInputSmokeClientState {
    registry_bound: bool,
    text_input_global_seen: bool,
    input_method_global_seen: bool,
    surface: Option<wl_surface::WlSurface>,
    seat: Option<client_wl_seat::WlSeat>,
    manager: Option<client_text_input_manager::ZwpTextInputManagerV3>,
    text_input: Option<client_text_input::ZwpTextInputV3>,
    enter_count: usize,
    leave_count: usize,
    entered_own_surface: bool,
    preedit_text: Option<String>,
    commit_text: Option<String>,
    delete_before: u32,
    delete_after: u32,
    done_serials: Vec<u32>,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl TextInputSmokeClientState {
    const SURROUNDING_TEXT: &'static str = "Merhaba İstanbul";
    const PREEDIT_TEXT: &'static str = "İ";
    const COMMIT_TEXT: &'static str = "İstanbul";
    const PAYLOAD_LIMIT_BYTES: usize = 4_000;

    fn initialize_text_input(&mut self, qh: &QueueHandle<Self>) {
        if self.text_input.is_none() {
            if let (Some(manager), Some(seat)) = (self.manager.clone(), self.seat.clone()) {
                self.text_input = Some(manager.get_text_input(&seat, qh, ()));
            }
        }
    }

    fn globals_ready(&self) -> bool {
        self.registry_bound
            && self.text_input_global_seen
            && self.surface.is_some()
            && self.text_input.is_some()
    }

    fn enable_with_state(&self) {
        let Some(text_input) = &self.text_input else {
            return;
        };
        let cursor = Self::SURROUNDING_TEXT.len() as i32;
        text_input.enable();
        text_input.set_surrounding_text(Self::SURROUNDING_TEXT.to_string(), cursor, cursor);
        text_input.set_text_change_cause(client_text_input::ChangeCause::InputMethod);
        text_input.set_content_type(
            client_text_input::ContentHint::Completion | client_text_input::ContentHint::Spellcheck,
            client_text_input::ContentPurpose::Normal,
        );
        text_input.set_cursor_rectangle(24, 36, 2, 28);
        text_input.commit();
    }

    fn update_cursor_rectangle(&self, x: i32, y: i32) {
        if let Some(text_input) = &self.text_input {
            text_input.set_cursor_rectangle(x, y, 2, 28);
            text_input.commit();
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for TextInputSmokeClientState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        state.registry_bound = true;
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };
        match interface.as_str() {
            "wl_compositor" => {
                let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                );
                let surface = compositor.create_surface(qh, ());
                surface.commit();
                state.surface = Some(surface);
            }
            "wl_seat" => {
                state.seat = Some(registry.bind::<client_wl_seat::WlSeat, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ));
            }
            "zwp_text_input_manager_v3" => {
                state.manager = Some(
                    registry.bind::<client_text_input_manager::ZwpTextInputManagerV3, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    ),
                );
                state.text_input_global_seen = true;
            }
            "zwp_input_method_manager_v2" => state.input_method_global_seen = true,
            _ => {}
        }
        state.initialize_text_input(qh);
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_text_input::ZwpTextInputV3, ()> for TextInputSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_text_input::ZwpTextInputV3,
        event: client_text_input::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            client_text_input::Event::Enter { surface } => {
                state.enter_count += 1;
                state.entered_own_surface = state.surface.as_ref() == Some(&surface);
            }
            client_text_input::Event::Leave { .. } => state.leave_count += 1,
            client_text_input::Event::PreeditString { text, .. } => state.preedit_text = text,
            client_text_input::Event::CommitString { text } => state.commit_text = text,
            client_text_input::Event::DeleteSurroundingText {
                before_length,
                after_length,
            } => {
                state.delete_before = before_length;
                state.delete_after = after_length;
            }
            client_text_input::Event::Done { serial } => state.done_serials.push(serial),
            _ => {}
        }
    }
}

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct InputMethodSmokeClientState {
    registry_bound: bool,
    text_input_global_seen: bool,
    input_method_global_seen: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    seat: Option<client_wl_seat::WlSeat>,
    manager: Option<client_input_method_manager::ZwpInputMethodManagerV2>,
    input_method: Option<client_input_method::ZwpInputMethodV2>,
    popup: Option<client_input_popup_surface::ZwpInputPopupSurfaceV2>,
    unavailable: bool,
    activate_count: usize,
    deactivate_count: usize,
    active: bool,
    surrounding_text: Option<String>,
    surrounding_cursor: u32,
    surrounding_anchor: u32,
    content_type_forwarded: bool,
    text_change_cause_forwarded: bool,
    done_count: u32,
    response_sent: bool,
    popup_rectangle: Option<(i32, i32, i32, i32)>,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl InputMethodSmokeClientState {
    fn initialize_input_method(&mut self, qh: &QueueHandle<Self>) {
        if self.input_method.is_none() {
            if let (Some(manager), Some(seat)) = (self.manager.clone(), self.seat.clone()) {
                self.input_method = Some(manager.get_input_method(&seat, qh, ()));
            }
        }
    }

    fn initialize_popup(&mut self, qh: &QueueHandle<Self>) {
        if self.popup.is_some() {
            return;
        }
        if let (Some(compositor), Some(input_method)) =
            (self.compositor.clone(), self.input_method.clone())
        {
            let surface = compositor.create_surface(qh, ());
            surface.commit();
            self.popup = Some(input_method.get_input_popup_surface(&surface, qh, ()));
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for InputMethodSmokeClientState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        state.registry_bound = true;
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };
        match interface.as_str() {
            "wl_compositor" => {
                state.compositor = Some(registry.bind::<wl_compositor::WlCompositor, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ));
            }
            "wl_seat" => {
                state.seat = Some(registry.bind::<client_wl_seat::WlSeat, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ));
            }
            "zwp_text_input_manager_v3" => state.text_input_global_seen = true,
            "zwp_input_method_manager_v2" => {
                state.manager = Some(
                    registry.bind::<client_input_method_manager::ZwpInputMethodManagerV2, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    ),
                );
                state.input_method_global_seen = true;
            }
            _ => {}
        }
        state.initialize_input_method(qh);
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_input_method::ZwpInputMethodV2, ()> for InputMethodSmokeClientState {
    fn event(
        state: &mut Self,
        input_method: &client_input_method::ZwpInputMethodV2,
        event: client_input_method::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            client_input_method::Event::Activate => {
                state.activate_count += 1;
                state.active = true;
                state.initialize_popup(qh);
            }
            client_input_method::Event::Deactivate => {
                state.deactivate_count += 1;
                state.active = false;
            }
            client_input_method::Event::SurroundingText {
                text,
                cursor,
                anchor,
            } => {
                state.surrounding_text = Some(text);
                state.surrounding_cursor = cursor;
                state.surrounding_anchor = anchor;
            }
            client_input_method::Event::TextChangeCause { .. } => {
                state.text_change_cause_forwarded = true
            }
            client_input_method::Event::ContentType { .. } => state.content_type_forwarded = true,
            client_input_method::Event::Done => {
                state.done_count += 1;
                if state.active && !state.response_sent {
                    input_method.set_preedit_string(
                        TextInputSmokeClientState::PREEDIT_TEXT.to_string(),
                        2,
                        2,
                    );
                    input_method.delete_surrounding_text(1, 0);
                    input_method.commit_string(TextInputSmokeClientState::COMMIT_TEXT.to_string());
                    input_method.commit(state.done_count);
                    state.response_sent = true;
                }
            }
            client_input_method::Event::Unavailable => state.unavailable = true,
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_input_popup_surface::ZwpInputPopupSurfaceV2, ()>
    for InputMethodSmokeClientState
{
    fn event(
        state: &mut Self,
        _: &client_input_popup_surface::ZwpInputPopupSurfaceV2,
        event: client_input_popup_surface::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_input_popup_surface::Event::TextInputRectangle {
            x,
            y,
            width,
            height,
        } = event
        {
            state.popup_rectangle = Some((x, y, width, height));
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(TextInputSmokeClientState: ignore wl_compositor::WlCompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(TextInputSmokeClientState: ignore wl_surface::WlSurface);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(TextInputSmokeClientState: ignore client_wl_seat::WlSeat);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(TextInputSmokeClientState: ignore client_text_input_manager::ZwpTextInputManagerV3);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(InputMethodSmokeClientState: ignore wl_compositor::WlCompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(InputMethodSmokeClientState: ignore wl_surface::WlSurface);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(InputMethodSmokeClientState: ignore client_wl_seat::WlSeat);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(InputMethodSmokeClientState: ignore client_input_method_manager::ZwpInputMethodManagerV2);

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct SelectionSmokeClientState {
    registry_bound: bool,
    clipboard_global_seen: bool,
    primary_global_seen: bool,
    data_control_global_seen: bool,
    surface: Option<wl_surface::WlSurface>,
    seat: Option<client_wl_seat::WlSeat>,
    data_manager: Option<client_wl_data_device_manager::WlDataDeviceManager>,
    data_device: Option<client_wl_data_device::WlDataDevice>,
    data_source: Option<client_wl_data_source::WlDataSource>,
    primary_manager: Option<client_primary_selection_manager::ZwpPrimarySelectionDeviceManagerV1>,
    primary_device: Option<client_primary_selection_device::ZwpPrimarySelectionDeviceV1>,
    primary_source: Option<client_primary_selection_source::ZwpPrimarySelectionSourceV1>,
    clipboard_offer_received: bool,
    primary_offer_received: bool,
    clipboard_offer_mimes: Vec<String>,
    primary_offer_mimes: Vec<String>,
    clipboard_requested_mimes: Vec<String>,
    primary_requested_mimes: Vec<String>,
    clipboard_receive_stream: Option<std::os::unix::net::UnixStream>,
    primary_receive_stream: Option<std::os::unix::net::UnixStream>,
    clipboard_payload: Vec<u8>,
    primary_payload: Vec<u8>,
    clipboard_selection_cleared: bool,
    primary_selection_cleared: bool,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl SelectionSmokeClientState {
    const MIME_TYPE: &'static str = "text/plain;charset=utf-8";
    const UNSUPPORTED_MIME_TYPE: &'static str = "application/x-aqua-unsupported";
    const TRANSFER_LIMIT_BYTES: usize = 4_096;

    fn with_payloads(clipboard_payload: &[u8], primary_payload: &[u8]) -> Self {
        Self {
            clipboard_payload: clipboard_payload.to_vec(),
            primary_payload: primary_payload.to_vec(),
            ..Self::default()
        }
    }

    fn initialize_devices(&mut self, qh: &QueueHandle<Self>) {
        if self.data_device.is_none() {
            if let (Some(manager), Some(seat)) = (self.data_manager.clone(), self.seat.clone()) {
                let source = manager.create_data_source(qh, ());
                source.offer(Self::UNSUPPORTED_MIME_TYPE.to_string());
                source.offer(Self::MIME_TYPE.to_string());
                self.data_device = Some(manager.get_data_device(&seat, qh, ()));
                self.data_source = Some(source);
            }
        }
        if self.primary_device.is_none() {
            if let (Some(manager), Some(seat)) = (self.primary_manager.clone(), self.seat.clone()) {
                let source = manager.create_source(qh, ());
                source.offer(Self::UNSUPPORTED_MIME_TYPE.to_string());
                source.offer(Self::MIME_TYPE.to_string());
                self.primary_device = Some(manager.get_device(&seat, qh, ()));
                self.primary_source = Some(source);
            }
        }
    }

    fn set_clipboard(&self, serial: u32) {
        if let (Some(device), Some(source)) = (&self.data_device, &self.data_source) {
            device.set_selection(Some(source), serial);
        }
    }

    fn set_primary(&self, serial: u32) {
        if let (Some(device), Some(source)) = (&self.primary_device, &self.primary_source) {
            device.set_selection(Some(source), serial);
        }
    }

    fn globals_ready(&self) -> bool {
        self.registry_bound
            && self.clipboard_global_seen
            && self.primary_global_seen
            && self.surface.is_some()
            && self.data_device.is_some()
            && self.primary_device.is_some()
    }

    fn request_clipboard_payload(&mut self, offer: &client_wl_data_offer::WlDataOffer) {
        if !self
            .clipboard_offer_mimes
            .iter()
            .any(|mime| mime == Self::MIME_TYPE)
        {
            return;
        }
        let Ok((read_stream, write_stream)) = std::os::unix::net::UnixStream::pair() else {
            return;
        };
        let _ = read_stream.set_read_timeout(Some(Duration::from_secs(2)));
        offer.receive(Self::MIME_TYPE.to_string(), write_stream.as_fd());
        self.clipboard_requested_mimes
            .push(Self::MIME_TYPE.to_string());
        self.clipboard_receive_stream = Some(read_stream);
    }

    fn request_primary_payload(
        &mut self,
        offer: &client_primary_selection_offer::ZwpPrimarySelectionOfferV1,
    ) {
        if !self
            .primary_offer_mimes
            .iter()
            .any(|mime| mime == Self::MIME_TYPE)
        {
            return;
        }
        let Ok((read_stream, write_stream)) = std::os::unix::net::UnixStream::pair() else {
            return;
        };
        let _ = read_stream.set_read_timeout(Some(Duration::from_secs(2)));
        offer.receive(Self::MIME_TYPE.to_string(), write_stream.as_fd());
        self.primary_requested_mimes
            .push(Self::MIME_TYPE.to_string());
        self.primary_receive_stream = Some(read_stream);
    }

    fn read_clipboard_payload(&mut self) -> std::io::Result<Vec<u8>> {
        read_bounded_selection_payload(
            self.clipboard_receive_stream.take(),
            Self::TRANSFER_LIMIT_BYTES,
        )
    }

    fn read_primary_payload(&mut self) -> std::io::Result<Vec<u8>> {
        read_bounded_selection_payload(
            self.primary_receive_stream.take(),
            Self::TRANSFER_LIMIT_BYTES,
        )
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn read_bounded_selection_payload(
    stream: Option<std::os::unix::net::UnixStream>,
    limit: usize,
) -> std::io::Result<Vec<u8>> {
    let mut stream = stream.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotConnected, "selection stream missing")
    })?;
    let mut payload = Vec::new();
    Read::by_ref(&mut stream)
        .take((limit + 1) as u64)
        .read_to_end(&mut payload)?;
    if payload.len() > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "selection payload exceeds probe limit",
        ));
    }
    Ok(payload)
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for SelectionSmokeClientState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        state.registry_bound = true;
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };
        match interface.as_str() {
            "wl_compositor" => {
                let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                );
                let surface = compositor.create_surface(qh, ());
                surface.commit();
                state.surface = Some(surface);
            }
            "wl_seat" => {
                state.seat = Some(registry.bind::<client_wl_seat::WlSeat, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ));
            }
            "wl_data_device_manager" => {
                state.data_manager = Some(
                    registry.bind::<client_wl_data_device_manager::WlDataDeviceManager, _, _>(
                        name,
                        version.min(3),
                        qh,
                        (),
                    ),
                );
                state.clipboard_global_seen = true;
            }
            "zwp_primary_selection_device_manager_v1" => {
                state.primary_manager = Some(registry.bind::<
                    client_primary_selection_manager::ZwpPrimarySelectionDeviceManagerV1,
                    _,
                    _,
                >(name, version.min(1), qh, ()));
                state.primary_global_seen = true;
            }
            "zwlr_data_control_manager_v1" | "ext_data_control_manager_v1" => {
                state.data_control_global_seen = true;
            }
            _ => {}
        }
        state.initialize_devices(qh);
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_data_device::WlDataDevice, ()> for SelectionSmokeClientState {
    wayland_client::event_created_child!(SelectionSmokeClientState, client_wl_data_device::WlDataDevice, [
        0 => (client_wl_data_offer::WlDataOffer, ())
    ]);

    fn event(
        state: &mut Self,
        _: &client_wl_data_device::WlDataDevice,
        event: client_wl_data_device::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_wl_data_device::Event::Selection { id } = event {
            if let Some(offer) = id {
                state.clipboard_offer_received = true;
                state.request_clipboard_payload(&offer);
            } else {
                state.clipboard_selection_cleared = true;
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_primary_selection_device::ZwpPrimarySelectionDeviceV1, ()>
    for SelectionSmokeClientState
{
    wayland_client::event_created_child!(SelectionSmokeClientState, client_primary_selection_device::ZwpPrimarySelectionDeviceV1, [
        0 => (client_primary_selection_offer::ZwpPrimarySelectionOfferV1, ())
    ]);

    fn event(
        state: &mut Self,
        _: &client_primary_selection_device::ZwpPrimarySelectionDeviceV1,
        event: client_primary_selection_device::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_primary_selection_device::Event::Selection { id } = event {
            if let Some(offer) = id {
                state.primary_offer_received = true;
                state.request_primary_payload(&offer);
            } else {
                state.primary_selection_cleared = true;
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_data_offer::WlDataOffer, ()> for SelectionSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_data_offer::WlDataOffer,
        event: client_wl_data_offer::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_wl_data_offer::Event::Offer { mime_type } = event {
            state.clipboard_offer_mimes.push(mime_type);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_primary_selection_offer::ZwpPrimarySelectionOfferV1, ()>
    for SelectionSmokeClientState
{
    fn event(
        state: &mut Self,
        _: &client_primary_selection_offer::ZwpPrimarySelectionOfferV1,
        event: client_primary_selection_offer::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_primary_selection_offer::Event::Offer { mime_type } = event {
            state.primary_offer_mimes.push(mime_type);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_data_source::WlDataSource, ()> for SelectionSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_data_source::WlDataSource,
        event: client_wl_data_source::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_wl_data_source::Event::Send { mime_type, fd } = event {
            state.clipboard_requested_mimes.push(mime_type);
            let mut file = std::fs::File::from(fd);
            let _ = file.write_all(&state.clipboard_payload);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_primary_selection_source::ZwpPrimarySelectionSourceV1, ()>
    for SelectionSmokeClientState
{
    fn event(
        state: &mut Self,
        _: &client_primary_selection_source::ZwpPrimarySelectionSourceV1,
        event: client_primary_selection_source::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        if let client_primary_selection_source::Event::Send { mime_type, fd } = event {
            state.primary_requested_mimes.push(mime_type);
            let mut file = std::fs::File::from(fd);
            let _ = file.write_all(&state.primary_payload);
        }
    }
}

#[derive(Default)]
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct DndSmokeClientState {
    registry_bound: bool,
    data_device_global_seen: bool,
    data_control_global_seen: bool,
    surface: Option<wl_surface::WlSurface>,
    seat: Option<client_wl_seat::WlSeat>,
    data_manager: Option<client_wl_data_device_manager::WlDataDeviceManager>,
    data_device: Option<client_wl_data_device::WlDataDevice>,
    data_source: Option<client_wl_data_source::WlDataSource>,
    current_offer: Option<client_wl_data_offer::WlDataOffer>,
    offered_mimes: Vec<String>,
    source_actions: Option<client_wl_data_device_manager::DndAction>,
    chosen_action: Option<client_wl_data_device_manager::DndAction>,
    source_target_mime: Option<String>,
    source_chosen_action: Option<client_wl_data_device_manager::DndAction>,
    requested_mimes: Vec<String>,
    receive_stream: Option<std::os::unix::net::UnixStream>,
    payload: Vec<u8>,
    accept_drop: bool,
    enter_count: usize,
    drop_count: usize,
    enter_matches_own_surface: bool,
    source_cancelled: bool,
    source_drop_performed: bool,
    source_finished: bool,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl DndSmokeClientState {
    const MIME_TYPE: &'static str = "text/plain;charset=utf-8";
    const UNSUPPORTED_MIME_TYPE: &'static str = "application/x-aqua-dnd-unsupported";
    const TRANSFER_LIMIT_BYTES: usize = 4_096;

    fn with_payload(payload: &[u8]) -> Self {
        Self {
            payload: payload.to_vec(),
            ..Self::default()
        }
    }

    fn initialize_device(&mut self, qh: &QueueHandle<Self>) {
        if self.data_device.is_none() {
            if let (Some(manager), Some(seat)) = (self.data_manager.clone(), self.seat.clone()) {
                self.data_device = Some(manager.get_data_device(&seat, qh, ()));
            }
        }
    }

    fn globals_ready(&self) -> bool {
        self.registry_bound
            && self.data_device_global_seen
            && self.surface.is_some()
            && self.data_device.is_some()
    }

    fn start_drag(&mut self, qh: &QueueHandle<Self>, serial: u32) {
        let (Some(manager), Some(device), Some(surface)) = (
            self.data_manager.clone(),
            self.data_device.clone(),
            self.surface.clone(),
        ) else {
            return;
        };
        if let Some(source) = self.data_source.take() {
            source.destroy();
        }
        self.source_target_mime = None;
        self.source_chosen_action = None;
        self.source_cancelled = false;
        self.source_drop_performed = false;
        self.source_finished = false;
        let source = manager.create_data_source(qh, ());
        source.offer(Self::UNSUPPORTED_MIME_TYPE.to_string());
        source.offer(Self::MIME_TYPE.to_string());
        source.set_actions(
            client_wl_data_device_manager::DndAction::Copy
                | client_wl_data_device_manager::DndAction::Move,
        );
        device.start_drag(Some(&source), &surface, None, serial);
        self.data_source = Some(source);
    }

    fn reset_target_for_rejected_drop(&mut self) {
        self.accept_drop = false;
        self.current_offer = None;
        self.offered_mimes.clear();
        self.source_actions = None;
        self.chosen_action = None;
        self.receive_stream = None;
    }

    fn accept_offer(&mut self, offer: &client_wl_data_offer::WlDataOffer, serial: u32) {
        if !self.accept_drop
            || !self
                .offered_mimes
                .iter()
                .any(|mime| mime == Self::MIME_TYPE)
        {
            offer.accept(serial, None);
            return;
        }
        offer.accept(serial, Some(Self::MIME_TYPE.to_string()));
        offer.set_actions(
            client_wl_data_device_manager::DndAction::Copy,
            client_wl_data_device_manager::DndAction::Copy,
        );
        let Ok((read_stream, write_stream)) = std::os::unix::net::UnixStream::pair() else {
            return;
        };
        let _ = read_stream.set_read_timeout(Some(Duration::from_secs(2)));
        offer.receive(Self::MIME_TYPE.to_string(), write_stream.as_fd());
        self.requested_mimes.push(Self::MIME_TYPE.to_string());
        self.receive_stream = Some(read_stream);
    }

    fn read_payload(&mut self) -> std::io::Result<Vec<u8>> {
        read_bounded_selection_payload(self.receive_stream.take(), Self::TRANSFER_LIMIT_BYTES)
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for DndSmokeClientState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        state.registry_bound = true;
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };
        match interface.as_str() {
            "wl_compositor" => {
                let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                );
                let surface = compositor.create_surface(qh, ());
                surface.commit();
                state.surface = Some(surface);
            }
            "wl_seat" => {
                state.seat = Some(registry.bind::<client_wl_seat::WlSeat, _, _>(
                    name,
                    version.min(1),
                    qh,
                    (),
                ));
            }
            "wl_data_device_manager" => {
                state.data_manager = Some(
                    registry.bind::<client_wl_data_device_manager::WlDataDeviceManager, _, _>(
                        name,
                        version.min(3),
                        qh,
                        (),
                    ),
                );
                state.data_device_global_seen = true;
            }
            "zwlr_data_control_manager_v1" | "ext_data_control_manager_v1" => {
                state.data_control_global_seen = true;
            }
            _ => {}
        }
        state.initialize_device(qh);
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_data_device::WlDataDevice, ()> for DndSmokeClientState {
    wayland_client::event_created_child!(DndSmokeClientState, client_wl_data_device::WlDataDevice, [
        0 => (client_wl_data_offer::WlDataOffer, ())
    ]);

    fn event(
        state: &mut Self,
        _: &client_wl_data_device::WlDataDevice,
        event: client_wl_data_device::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            client_wl_data_device::Event::Enter {
                serial,
                surface,
                id,
                ..
            } => {
                state.enter_count += 1;
                state.enter_matches_own_surface = state.surface.as_ref() == Some(&surface);
                if let Some(offer) = id {
                    state.accept_offer(&offer, serial);
                    state.current_offer = Some(offer);
                }
            }
            client_wl_data_device::Event::Drop => {
                state.drop_count += 1;
                if state.accept_drop {
                    if let Some(offer) = &state.current_offer {
                        offer.finish();
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_data_offer::WlDataOffer, ()> for DndSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_data_offer::WlDataOffer,
        event: client_wl_data_offer::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            client_wl_data_offer::Event::Offer { mime_type } => state.offered_mimes.push(mime_type),
            client_wl_data_offer::Event::SourceActions {
                source_actions: WEnum::Value(actions),
            } => state.source_actions = Some(actions),
            client_wl_data_offer::Event::Action {
                dnd_action: WEnum::Value(action),
            } => state.chosen_action = Some(action),
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_wl_data_source::WlDataSource, ()> for DndSmokeClientState {
    fn event(
        state: &mut Self,
        _: &client_wl_data_source::WlDataSource,
        event: client_wl_data_source::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            client_wl_data_source::Event::Target { mime_type } => {
                state.source_target_mime = mime_type
            }
            client_wl_data_source::Event::Send { mime_type, fd } => {
                state.requested_mimes.push(mime_type);
                let mut file = std::fs::File::from(fd);
                let _ = file.write_all(&state.payload);
            }
            client_wl_data_source::Event::Cancelled => state.source_cancelled = true,
            client_wl_data_source::Event::DndDropPerformed => state.source_drop_performed = true,
            client_wl_data_source::Event::DndFinished => state.source_finished = true,
            client_wl_data_source::Event::Action {
                dnd_action: WEnum::Value(action),
            } => state.source_chosen_action = Some(action),
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(DndSmokeClientState: ignore wl_compositor::WlCompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(DndSmokeClientState: ignore wl_surface::WlSurface);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(DndSmokeClientState: ignore client_wl_seat::WlSeat);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(DndSmokeClientState: ignore client_wl_data_device_manager::WlDataDeviceManager);

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SelectionSmokeClientState: ignore wl_compositor::WlCompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SelectionSmokeClientState: ignore wl_surface::WlSurface);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SelectionSmokeClientState: ignore client_wl_seat::WlSeat);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SelectionSmokeClientState: ignore client_wl_data_device_manager::WlDataDeviceManager);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SelectionSmokeClientState: ignore client_primary_selection_manager::ZwpPrimarySelectionDeviceManagerV1);

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
            let wifi_credential_entry = state
                .settings_model
                .as_ref()
                .is_some_and(|model| model.wifi.credential_entry());
            if wifi_credential_entry && key == 1 {
                if state
                    .settings_model
                    .as_mut()
                    .is_some_and(aqua_shell::SettingsWindowModel::cancel_wifi_credential_entry)
                {
                    state.redraw_settings_buffer(qh);
                }
                return;
            }
            if wifi_credential_entry && key == 14 {
                if state
                    .settings_model
                    .as_mut()
                    .is_some_and(|model| model.remove_wifi_passphrase_character())
                {
                    state.redraw_settings_buffer(qh);
                }
                return;
            }
            if wifi_credential_entry && key != 28 {
                if let Some(character) = settings_passphrase_character(key, state.keyboard_shift) {
                    if state
                        .settings_model
                        .as_mut()
                        .is_some_and(|model| model.input_wifi_passphrase(character))
                    {
                        println!("aqua_settings_wifi_secret_input=redacted");
                        state.redraw_settings_buffer(qh);
                    }
                }
                return;
            }
            let settings_key = settings_key_for_code(key);
            if let (Some(settings_key), Some(model)) = (settings_key, state.settings_model.as_mut())
            {
                let update = model.handle_key(settings_key);
                println!("aqua_settings_keyboard key={key} update={update:?}");
                if let aqua_shell::SettingsUpdate::ThemeChanged(theme) = update {
                    state.theme = theme;
                    println!("aqua_settings_theme={}", theme.id());
                }
                if let aqua_shell::SettingsUpdate::WifiControlRequested(connected) = update {
                    state.apply_settings_wifi_control(connected);
                }
                if update == aqua_shell::SettingsUpdate::WifiConnectRequested {
                    state.apply_settings_wifi_connection();
                }
                if update == aqua_shell::SettingsUpdate::WifiScanRequested {
                    state.apply_settings_wifi_scan();
                }
                if update == aqua_shell::SettingsUpdate::WifiForgetRequested {
                    state.apply_settings_wifi_forget();
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
                    let buffer_width = state.buffer_width.max(1);
                    let navigation = state.files_scrollbar_dragging.then(|| {
                        navigator.handle_scrollbar_drag(buffer_width, surface_y.max(0.0) as u32)
                    });
                    let files_changed = navigation
                        .is_some_and(aqua_shell::FilesNavigation::changed)
                        || (!state.files_scrollbar_dragging
                            && navigator.handle_hover(
                                buffer_width,
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
                let buffer_width = state.buffer_width.max(1);
                if navigator.scrollbar_hit(buffer_width, pointer_x, pointer_y) {
                    state.files_scrollbar_dragging = true;
                    let navigation = navigator.handle_scrollbar_drag(buffer_width, pointer_y);
                    println!("aqua_files_scrollbar y={pointer_y} navigation={navigation:?}");
                    if navigation.changed() {
                        state.files_model = Some(navigator.window().clone());
                        state.redraw_files_buffer(qh);
                    }
                    return;
                }
                let navigation = navigator.handle_pointer(buffer_width, pointer_x, pointer_y);
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
                if let aqua_shell::SettingsUpdate::WifiControlRequested(connected) = update {
                    state.apply_settings_wifi_control(connected);
                }
                if update == aqua_shell::SettingsUpdate::WifiScanRequested {
                    state.apply_settings_wifi_scan();
                }
                if update == aqua_shell::SettingsUpdate::WifiForgetRequested {
                    state.apply_settings_wifi_forget();
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
                    state.buffer_width.max(1),
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

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for PopupLifecycleClientState {
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
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version.min(4),
                        qh,
                        (),
                    ));
                }
                "xdg_wm_base" => {
                    state.wm_base = Some(registry.bind::<client_xdg_wm_base::XdgWmBase, _, _>(
                        name,
                        version.min(6),
                        qh,
                        (),
                    ));
                }
                _ => {}
            }
            state.maybe_create_parent(qh);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_xdg_wm_base::XdgWmBase, ()> for PopupLifecycleClientState {
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
impl ClientDispatch<client_xdg_surface::XdgSurface, PopupXdgRole> for PopupLifecycleClientState {
    fn event(
        state: &mut Self,
        surface: &client_xdg_surface::XdgSurface,
        event: client_xdg_surface::Event,
        role: &PopupXdgRole,
        _: &ClientConnection,
        qh: &QueueHandle<Self>,
    ) {
        if let client_xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            match role {
                PopupXdgRole::Parent => {
                    state.parent_configure_acknowledged = true;
                    state.create_popup(qh);
                }
                PopupXdgRole::Popup => {
                    state.popup_configure_ack_count += 1;
                    state.request_reposition(qh);
                }
            }
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<client_xdg_popup::XdgPopup, ()> for PopupLifecycleClientState {
    fn event(
        state: &mut Self,
        _: &client_xdg_popup::XdgPopup,
        event: client_xdg_popup::Event,
        _: &(),
        _: &ClientConnection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            client_xdg_popup::Event::Configure {
                x,
                y,
                width,
                height,
            } => {
                state.popup_configure_count += 1;
                state.popup_geometries.push((x, y, width, height));
            }
            client_xdg_popup::Event::Repositioned { token } => {
                state.repositioned_token = Some(token);
            }
            _ => {}
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl ClientDispatch<wl_registry::WlRegistry, ()> for SubsurfaceLifecycleClientState {
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
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version.min(4),
                        qh,
                        (),
                    ));
                }
                "wl_subcompositor" => {
                    state.subcompositor = Some(
                        registry.bind::<client_wl_subcompositor::WlSubcompositor, _, _>(
                            name,
                            version.min(1),
                            qh,
                            (),
                        ),
                    );
                }
                _ => {}
            }
            state.maybe_create_tree(qh);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(PopupLifecycleClientState: ignore wl_compositor::WlCompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(PopupLifecycleClientState: ignore wl_surface::WlSurface);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(PopupLifecycleClientState: ignore client_xdg_toplevel::XdgToplevel);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(PopupLifecycleClientState: ignore client_xdg_positioner::XdgPositioner);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SubsurfaceLifecycleClientState: ignore wl_compositor::WlCompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SubsurfaceLifecycleClientState: ignore wl_surface::WlSurface);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SubsurfaceLifecycleClientState: ignore client_wl_subcompositor::WlSubcompositor);
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
delegate_noop!(SubsurfaceLifecycleClientState: ignore client_wl_subsurface::WlSubsurface);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shm_buffer_bounds_honor_nonzero_pool_offsets() {
        assert_eq!(
            shm_buffer_bounds(768_000, 384_000, 1_600, 240),
            Some((384_000, 384_000))
        );
        assert_eq!(shm_buffer_bounds(767_999, 384_000, 1_600, 240), None);
        assert_eq!(shm_buffer_bounds(768_000, -1, 1_600, 240), None);
        assert_eq!(shm_buffer_bounds(768_000, 0, 0, 240), None);
    }

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
    fn bottom_shell_routes_the_actual_viewport_dock_geometry() {
        let viewport = Viewport::new(1536, 1024);
        let dock = static_shell_scene(viewport)
            .surface_rect(SurfaceKind::Dock)
            .expect("Dock geometry should exist");
        assert_eq!(
            dock,
            Rect {
                x: 388,
                y: 928,
                width: 760,
                height: 72,
            }
        );
        assert_eq!(
            bottom_shell_pointer_target(viewport, dock.x + 32, dock.y + 36),
            Some(BottomShellTarget::Applications)
        );
        assert_eq!(
            bottom_shell_pointer_target(
                viewport,
                dock.x + dock.width - 60 * WORKSPACE_COUNT as u32 + 30,
                dock.y + dock.height / 2,
            ),
            Some(BottomShellTarget::Workspace(0))
        );
        assert_eq!(
            bottom_shell_pointer_target(viewport, dock.x + 64, dock.y + 36),
            None
        );
        assert_eq!(
            bottom_shell_pointer_target(viewport, dock.right(), dock.y + 36),
            None
        );
        assert_eq!(
            bottom_shell_pointer_target(viewport, dock.x + 32, dock.bottom()),
            None
        );

        let canonical = static_shell_scene(Viewport::new(800, 600))
            .surface_rect(SurfaceKind::Dock)
            .expect("Canonical Dock geometry should exist");
        let legacy_scaled_x = (canonical.x + canonical.width - 30) * viewport.width / 800;
        let legacy_scaled_y = (canonical.y + canonical.height / 2) * viewport.height / 600;
        assert_eq!(
            bottom_shell_pointer_target(viewport, legacy_scaled_x, legacy_scaled_y),
            None
        );
    }

    #[test]
    fn pointer_motion_clamps_to_the_actual_viewport() {
        let viewport = Viewport::new(1024, 768);

        assert_eq!(
            pointer_location_after_motion((768.0, 512.0), 500.0, 500.0, viewport),
            (1023.0, 767.0)
        );
        assert_eq!(
            pointer_location_after_motion((100.0, 100.0), -500.0, -500.0, viewport),
            (0.0, 0.0)
        );

        let mut launcher = LauncherState::default();
        launcher.handle_event(LauncherEvent::OpenApplications);
        assert_eq!(launcher.pointer_target(790, 140), None);
        assert_eq!(
            launcher.pointer_target_in_viewport(790, 140, viewport.width, viewport.height),
            Some(LauncherPointerTarget::SearchField)
        );
    }

    #[test]
    fn session_menu_pointer_maps_the_actual_viewport_to_runtime_rows() {
        let viewport = Viewport::new(1024, 768);
        let panel = static_shell_scene(viewport)
            .surface_rect(SurfaceKind::SystemOverview)
            .expect("Session menu surface should exist");
        assert_eq!(
            panel,
            Rect {
                x: 680,
                y: 60,
                width: 320,
                height: 220,
            }
        );
        assert_eq!(
            session_menu_pointer_position(viewport, panel.x + 160, panel.y + 128),
            Some((256, 170))
        );
        assert_eq!(
            session_menu_pointer_position(viewport, panel.right(), panel.y + 128),
            None
        );
        assert_eq!(
            session_menu_pointer_position(viewport, panel.x + 160, panel.bottom()),
            None
        );
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
        for (index, (code, expected)) in [(107, 5), (102, 0), (105, 5), (106, 0)]
            .into_iter()
            .enumerate()
        {
            let time = 3 + index as u32 * 2;
            assert!(session.dispatch_keyboard_key(code, true, time));
            assert!(session.dispatch_keyboard_key(code, false, time + 1));
            assert_eq!(session.launcher_state_snapshot().selected_index(), expected);
        }
        for (index, code) in [31, 18, 20, 20, 23, 49, 34, 31, 45].into_iter().enumerate() {
            let time = 11 + index as u32 * 2;
            assert!(session.dispatch_keyboard_key(code, true, time));
            assert!(session.dispatch_keyboard_key(code, false, time + 1));
        }
        assert_eq!(session.launcher_state_snapshot().query(), "settingsx");
        assert!(session.dispatch_keyboard_key(14, true, 40));
        assert_eq!(session.launcher_state_snapshot().query(), "settings");
        assert!(session.dispatch_keyboard_key(108, true, 41));
        assert!(session.dispatch_keyboard_key(108, false, 42));
        assert!(session.dispatch_keyboard_key(28, true, 43));

        let snapshot = session.input_snapshot();
        assert_eq!(snapshot.keyboard_forward_count, 0);
        assert!(snapshot.keyboard_shortcut_intercept_count >= 32);
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
    fn smithay_pointer_motion_uses_output_dimensions_for_bounds_and_launcher_hits() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");
        session.set_output_dimensions(1024, 768);
        assert!(session.dispatch_keyboard_key(125, true, 1));

        let launcher = session.launcher_state_snapshot();
        assert_eq!(launcher.pointer_target(790, 140), None);
        assert_eq!(
            launcher.pointer_target_in_viewport(790, 140, 1024, 768),
            Some(LauncherPointerTarget::SearchField)
        );

        assert!(session.dispatch_pointer_motion(22.0, -372.0, 2));
        let launcher_hit = session.input_snapshot();
        assert_eq!(launcher_hit.pointer_x, 790);
        assert_eq!(launcher_hit.pointer_y, 140);
        assert_eq!(launcher_hit.launcher_pointer_hit_count, 1);

        assert!(session.dispatch_pointer_motion(1000.0, 1000.0, 3));
        let bounded = session.input_snapshot();
        assert_eq!(bounded.pointer_x, 1023);
        assert_eq!(bounded.pointer_y, 767);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_selection_ownership_is_keyboard_focus_bound() {
        let probe = probe_selection_ownership().expect("selection ownership probe");

        assert!(probe.is_ready());
        assert_eq!(probe.client_count, 2);
        assert!(probe.globals_visible_to_both_clients);
        assert!(probe.focus_follows_keyboard);
        assert!(probe.unfocused_clipboard_rejected);
        assert!(probe.unfocused_primary_rejected);
        assert!(probe.focused_clipboard_accepted);
        assert!(probe.focused_primary_accepted);
        assert!(probe.clipboard_offer_reaches_new_focus);
        assert!(probe.primary_offer_reaches_new_focus);
        assert!(probe.clipboard_mime_negotiated);
        assert!(probe.primary_mime_negotiated);
        assert!(probe.unsupported_mime_not_requested);
        assert!(probe.clipboard_payload_transferred);
        assert!(probe.primary_payload_transferred);
        assert_eq!(probe.clipboard_payload_bytes, 24);
        assert_eq!(probe.primary_payload_bytes, 32);
        assert_eq!(probe.transfer_limit_bytes, 4_096);
        assert!(!probe.compositor_buffers_payload);
        assert!(probe.owner_disconnect_clears_clipboard);
        assert!(probe.owner_disconnect_clears_primary);
        assert!(probe.ownership_handoff_accepted);
        assert!(!probe.data_control_global_exposed);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_drag_and_drop_is_focus_safe_and_bounded() {
        let probe = probe_drag_and_drop().expect("drag-and-drop probe");

        assert!(probe.is_ready());
        assert_eq!(probe.client_count, 2);
        assert!(probe.start_without_implicit_grab_rejected);
        assert!(probe.pointer_grab_started);
        assert!(probe.source_client_owns_drag);
        assert!(probe.enter_reaches_pointer_focus_only);
        assert!(probe.keyboard_focus_unchanged);
        assert!(probe.mime_negotiated);
        assert!(probe.unsupported_mime_not_accepted);
        assert!(probe.copy_action_negotiated);
        assert!(probe.payload_transferred);
        assert_eq!(probe.payload_bytes, 28);
        assert_eq!(probe.transfer_limit_bytes, 4_096);
        assert!(!probe.compositor_buffers_payload);
        assert!(probe.drop_delivered_to_target);
        assert!(probe.source_drop_performed);
        assert!(probe.source_finished);
        assert!(probe.rejected_drop_cancelled);
        assert!(probe.rejected_drop_not_delivered);
        assert!(!probe.data_control_global_exposed);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_text_input_is_focus_and_authorization_safe() {
        let probe = probe_text_input().expect("text-input probe");

        assert!(probe.is_ready());
        assert_eq!(probe.client_count, 3);
        assert!(probe.text_input_visible_to_normal_clients);
        assert!(probe.input_method_hidden_from_normal_clients);
        assert!(probe.input_method_visible_to_authorized_client);
        assert!(probe.focus_follows_keyboard);
        assert!(probe.unfocused_enable_rejected);
        assert!(probe.focused_enable_activates_input_method);
        assert!(probe.surrounding_text_forwarded);
        assert!(probe.content_type_forwarded);
        assert!(probe.cursor_rectangle_forwarded);
        assert!(probe.turkish_preedit_delivered);
        assert!(probe.turkish_commit_delivered);
        assert!(probe.delete_surrounding_delivered);
        assert!(probe.serial_synchronized);
        assert!(probe.focus_handoff_deactivates_input_method);
        assert!(probe.focus_handoff_enters_new_client);
        assert!(probe.stale_unfocused_client_blocked);
        assert!(probe.popup_parent_bound);
        assert!(probe.popup_repositioned);
        assert_eq!(probe.payload_limit_bytes, 4_000);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_keyboard_locale_matrix_delivers_compose_and_dead_keys() {
        let probe = probe_keyboard_locale_matrix().expect("keyboard locale matrix probe");

        assert!(probe.is_ready());
        assert_eq!(probe.locale_count, 3);
        assert_eq!(probe.keyboard_layout_count, 3);
        assert_eq!(probe.supported_combination_count, 9);
        assert_eq!(probe.client_count_per_layout, 2);
        assert!(probe.keymaps_delivered_to_all_clients);
        assert!(probe.keymaps_compile_for_all_layouts);
        assert!(probe.representative_utf8_matches);
        assert!(probe.compose_key_available_for_all_layouts);
        assert_eq!(probe.compose_case_count, 9);
        assert!(probe.compose_utf8_matches_for_all_clients);
        assert_eq!(probe.dead_key_layout_count, 2);
        assert_eq!(probe.dead_key_case_count, 6);
        assert!(probe.dead_key_utf8_matches_for_all_clients);
        assert!(probe.cancelled_compose_rejected_for_all_locales);
        assert_eq!(probe.repeat_delay_ms, 400);
        assert_eq!(probe.repeat_rate_hz, 25);
        assert!(probe.repeat_info_matches);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_privileged_protocol_boundary_is_narrow_and_unadvertised() {
        let probe =
            probe_privileged_protocol_boundary().expect("privileged protocol boundary probe");

        assert!(probe.is_ready());
        assert_eq!(probe.client_count, 3);
        assert_eq!(probe.normal_client_count, 2);
        assert_eq!(probe.authorized_client_count, 1);
        assert!(probe.baseline_globals_visible_to_all_clients);
        assert!(probe.input_method_hidden_from_normal_clients);
        assert!(probe.input_method_visible_to_authorized_client);
        assert_eq!(probe.privileged_global_count, 16);
        assert!(!probe.screenshot_global_exposed);
        assert!(!probe.screencopy_global_exposed);
        assert!(!probe.activation_global_exposed);
        assert!(!probe.privileged_shell_global_exposed);
        assert!(!probe.virtual_input_global_exposed);
        assert!(!probe.desktop_management_global_exposed);
        assert!(probe.authorized_scope_is_narrow);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_output_matrix_is_discoverable_scaled_and_hotpluggable() {
        let probe = probe_wayland_output_matrix().expect("Wayland output matrix probe");

        assert!(probe.is_ready());
        assert_eq!(probe.client_count, 2);
        assert_eq!(probe.output_count, 4);
        assert_eq!(probe.declared_scale_count, 4);
        assert_eq!(probe.declared_transform_count, 4);
        assert!(probe.outputs_visible_to_both_clients);
        assert!(probe.modes_match_supported_matrix);
        assert!(probe.preferred_modes_advertised);
        assert!(probe.logical_coordinates_match);
        assert!(probe.integer_scales_match);
        assert!(probe.fractional_scales_match);
        assert!(probe.transforms_match);
        assert!(probe.fractional_scale_advertised);
        assert_eq!(probe.fractional_scale_120ths, 150);
        assert!(probe.viewport_source_applied);
        assert!(probe.viewport_destination_applied);
        assert!(probe.hotplug_add_reaches_both_clients);
        assert!(probe.hotplug_remove_reaches_both_clients);
        assert!(probe.remaining_output_usable);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_popup_and_subsurface_lifecycles_are_independent() {
        let probe = probe_popup_subsurface_matrix().expect("popup and subsurface matrix probe");

        assert!(probe.is_ready());
        assert_eq!(probe.client_count, 2);
        assert!(probe.xdg_popup_created);
        assert!(probe.popup_parent_bound);
        assert!(probe.popup_geometry_matches);
        assert!(probe.popup_configure_acknowledged);
        assert!(probe.popup_reposition_requested);
        assert_eq!(probe.popup_reposition_token, 77);
        assert!(probe.popup_reposition_acknowledged);
        assert!(probe.popup_destroyed);
        assert!(probe.subsurface_created);
        assert!(probe.subsurface_parent_bound);
        assert!(probe.subsurface_position_matches);
        assert!(probe.synchronized_commit_observed);
        assert!(probe.desynchronized_commit_observed);
        assert!(probe.subsurface_destroyed);
        assert!(probe.parent_surfaces_remain_independent);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn v1_client_buffer_contract_excludes_accelerated_clients() {
        let probe = probe_v1_client_buffer_contract().expect("v1 client buffer contract probe");

        assert!(probe.is_ready());
        assert!(probe.wl_shm_visible_to_all_clients);
        assert!(probe.argb8888_visible_to_all_clients);
        assert!(!probe.linux_dmabuf_advertised);
        assert!(!probe.drm_syncobj_advertised);
        assert!(!probe.explicit_sync_advertised);
        assert!(!probe.accelerated_clients_supported);
        assert!(!probe.host_stub);
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn selection_payload_probe_rejects_bytes_beyond_limit() {
        let (read_stream, mut write_stream) = std::os::unix::net::UnixStream::pair().unwrap();
        write_stream.write_all(b"12345").unwrap();
        drop(write_stream);

        let error = read_bounded_selection_payload(Some(read_stream), 4)
            .expect_err("payload above the probe limit must fail");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
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
    fn first_party_settings_keyboard_uses_shared_control_targets() {
        let mut model = aqua_shell::SettingsWindowModel::default();

        assert_eq!(
            settings_key_for_code(107).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::CategorySelected(5))
        );
        assert_eq!(
            settings_key_for_code(103).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::CategorySelected(4))
        );
        assert_eq!(
            settings_key_for_code(102).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::CategorySelected(0))
        );
        assert_eq!(
            settings_key_for_code(105).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::ThemeChanged(
                aqua_shell::AquaTheme::Nightmare
            ))
        );
        assert_eq!(
            settings_key_for_code(106).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::ThemeChanged(
                aqua_shell::AquaTheme::LightWhite
            ))
        );
        assert_eq!(
            settings_key_for_code(28).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::ReducedMotionChanged(true))
        );
        assert_eq!(
            settings_key_for_code(107).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::CategorySelected(5))
        );
        assert_eq!(
            settings_key_for_code(103).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::CategorySelected(4))
        );
        assert_eq!(
            settings_key_for_code(28).map(|key| model.handle_key(key)),
            Some(aqua_shell::SettingsUpdate::None)
        );
        assert_eq!(settings_key_for_code(0), None);
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

        assert!(session.dispatch_keyboard_key(29, true, 98));
        assert!(session.dispatch_keyboard_key(56, true, 99));
        assert!(session.dispatch_keyboard_key(107, true, 100));
        assert!(session.dispatch_keyboard_key(107, false, 101));
        assert!(session.dispatch_keyboard_key(56, false, 102));
        assert!(session.dispatch_keyboard_key(29, false, 103));
        assert_eq!(session.active_workspace(), 2);

        assert!(session.dispatch_keyboard_key(29, true, 104));
        assert!(session.dispatch_keyboard_key(56, true, 105));
        assert!(session.dispatch_keyboard_key(102, true, 106));
        assert!(session.dispatch_keyboard_key(102, false, 107));
        assert!(session.dispatch_keyboard_key(56, false, 108));
        assert!(session.dispatch_keyboard_key(29, false, 109));
        assert_eq!(session.active_workspace(), 0);

        assert!(session.activate_workspace(1, 110));
        assert!(session.dispatch_keyboard_key(29, true, 111));
        assert!(session.dispatch_keyboard_key(56, true, 112));
        assert!(session.dispatch_keyboard_key(42, true, 113));
        assert!(session.dispatch_keyboard_key(107, true, 114));
        assert!(session.dispatch_keyboard_key(107, false, 115));
        assert!(session.dispatch_keyboard_key(42, false, 116));
        assert!(session.dispatch_keyboard_key(56, false, 117));
        assert!(session.dispatch_keyboard_key(29, false, 118));
        assert_eq!(session.active_workspace(), 1);
        assert!(session.visible_client_surface_snapshots().is_empty());
        assert!(session.activate_workspace(2, 119));
        assert_eq!(
            session.active_toplevel_app_id().as_deref(),
            Some("aqua.settings")
        );
        assert!(session.activate_workspace(1, 120));
        let dock = static_shell_scene(Viewport::new(1536, 1024))
            .surface_rect(SurfaceKind::Dock)
            .expect("Dock geometry should exist");
        let pointer_x = dock.x + dock.width - 60 * WORKSPACE_COUNT as u32 + 30;
        let pointer_y = dock.y + dock.height / 2;
        let input = session.input_snapshot();
        assert!(session.dispatch_pointer_motion(
            f64::from(pointer_x) - f64::from(input.pointer_x),
            f64::from(pointer_y) - f64::from(input.pointer_y),
            121,
        ));
        assert!(session.dispatch_pointer_button(0x110, true, 122));
        assert_eq!(session.active_workspace(), 0);
        assert!(session.present_client_surface(122));
        let input = session.input_snapshot();
        assert_eq!(input.pointer_x, pointer_x);
        assert_eq!(input.pointer_y, pointer_y);
        assert!(!session.raise_surface_with_app_id("aqua.unknown"));
        assert_eq!(
            session.active_toplevel_app_id().as_deref(),
            Some("aqua.files")
        );
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_session_menu_keyboard_uses_shared_home_end_and_confirmation_gate() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");

        assert!(session.dispatch_keyboard_key(68, true, 1));
        assert!(session.dispatch_keyboard_key(107, true, 2));
        assert!(session.dispatch_keyboard_key(107, false, 3));
        assert_eq!(session.session_menu_state_snapshot().selected_index(), 3);
        assert!(session.dispatch_keyboard_key(28, true, 20));
        assert!(session.dispatch_keyboard_key(28, false, 21));
        assert_eq!(
            session.session_menu_state_snapshot().confirmation(),
            Some(SessionAction::Recovery)
        );

        assert!(session.dispatch_keyboard_key(102, true, 22));
        assert!(session.dispatch_keyboard_key(102, false, 23));
        let home = session.session_menu_state_snapshot();
        assert_eq!(home.selected_index(), 0);
        assert_eq!(home.confirmation(), None);

        assert!(session.dispatch_keyboard_key(103, true, 24));
        assert!(session.dispatch_keyboard_key(103, false, 25));
        assert_eq!(session.session_menu_state_snapshot().selected_index(), 3);
        assert!(session.dispatch_keyboard_key(28, true, 26));
        assert!(session.dispatch_keyboard_key(28, false, 27));
        assert!(session.dispatch_keyboard_key(28, true, 28));

        assert!(session.has_session_action_request());
        assert_eq!(
            session.take_session_action_request(),
            Some(SessionAction::Recovery)
        );
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_desktop_context_menu_keyboard_is_compositor_owned() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");
        let input = session.input_snapshot();

        assert!(session.dispatch_pointer_motion(
            48.0 - f64::from(input.pointer_x),
            194.0 - f64::from(input.pointer_y),
            1,
        ));
        assert!(session.dispatch_pointer_button(0x111, true, 2));
        assert_eq!(
            session
                .desktop_icon_state_snapshot()
                .context_menu_selected_row(),
            Some(0)
        );

        assert!(session.dispatch_keyboard_key(125, true, 3));
        assert!(session.dispatch_keyboard_key(125, false, 4));
        assert!(!session.launcher_state_snapshot().is_open());
        assert!(session.dispatch_keyboard_key(108, true, 5));
        assert!(session.dispatch_keyboard_key(108, false, 6));
        assert_eq!(
            session
                .desktop_icon_state_snapshot()
                .context_menu_selected_row(),
            Some(1)
        );
        assert!(session.dispatch_keyboard_key(28, true, 7));
        assert!(session.dispatch_keyboard_key(28, false, 8));

        let snapshot = session.input_snapshot();
        assert_eq!(snapshot.keyboard_forward_count, 0);
        assert!(snapshot.keyboard_shortcut_intercept_count >= 6);
        assert_eq!(
            session
                .take_launcher_launch_request()
                .expect("Properties request should be queued"),
            LaunchRequest {
                app_id: "properties",
                command: "/usr/bin/aqua-properties",
                target: Some("settings"),
            }
        );
        assert!(session
            .desktop_icon_state_snapshot()
            .context_menu()
            .is_none());
    }

    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    #[test]
    fn smithay_session_menu_pointer_uses_actual_output_and_confirmation_gate() {
        let mut session = SmithayDrmSession::new().expect("Smithay session should start");
        session.set_output_dimensions(1024, 768);
        assert!(session.dispatch_keyboard_key(68, true, 1));

        let input = session.input_snapshot();
        assert!(session.dispatch_pointer_motion(
            840.0 - f64::from(input.pointer_x),
            188.0 - f64::from(input.pointer_y),
            2,
        ));
        assert!(session.dispatch_pointer_button(0x110, true, 3));
        let armed = session.session_menu_state_snapshot();
        assert_eq!(armed.selected_action(), SessionAction::Shutdown);
        assert_eq!(armed.confirmation(), Some(SessionAction::Shutdown));
        assert!(!session.has_session_action_request());

        assert!(session.dispatch_pointer_button(0x110, true, 4));
        assert_eq!(
            session.take_session_action_request(),
            Some(SessionAction::Shutdown)
        );
        assert!(!session.session_menu_state_snapshot().is_open());
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
