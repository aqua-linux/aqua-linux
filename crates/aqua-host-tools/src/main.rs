use std::{env, fs, path::PathBuf};

use aqua_compositor::{
    probe_client_layer_pipeline, probe_display_output_handoff, run_manual_nested_preview_execution,
    run_nested_output_surface_lifecycle,
};
#[cfg(test)]
use aqua_renderer::export_software_raster_rgba_for_static_scene;
use aqua_renderer::{export_composited_preview_rgba_with_client_layers, RasterRgbaExport};
use aqua_scene::Viewport;

const PRODUCT: &str = "Aqua Linux";
const OS_BASE: &str = "Buildroot";
const DEV_TARGET: &str = "QEMU x86_64";
const GRAPHICS_TARGET: &str = "custom Wayland compositor";
const PREVIEW_WIDTH: u32 = 1536;
const PREVIEW_HEIGHT: u32 = 1024;
const HOST_WINDOW_FRAME_LIMIT: u32 = 600;
const HOST_WINDOW_SMOKE_FRAME_LIMIT: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostWindowLifecycleProbe {
    product: &'static str,
    status: &'static str,
    launch_mode: &'static str,
    window_backend: &'static str,
    feature_gate: &'static str,
    feature_gate_required: bool,
    feature_compiled: bool,
    source_presenter_ready: bool,
    source_surface_lifecycle_ready: bool,
    frame_ready: bool,
    frame_format: &'static str,
    frame_checksum: u64,
    bounded_frame_limit: u32,
    window_opened: bool,
    window_closed: bool,
    manual_start_required: bool,
    autostart: bool,
    boot_graphics: bool,
    rootfs_packaged: bool,
    recovery_safe: bool,
}

impl HostWindowLifecycleProbe {
    fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "manual-host-window-lifecycle-ready"
            && self.launch_mode == "manual-dev"
            && self.window_backend == "minifb"
            && self.feature_gate == "host-window-preview"
            && self.feature_gate_required
            && self.source_presenter_ready
            && self.source_surface_lifecycle_ready
            && self.frame_ready
            && self.frame_format == "raw-rgba8888-composited-client-preview"
            && self.frame_checksum != 0
            && self.bounded_frame_limit == HOST_WINDOW_FRAME_LIMIT
            && !self.window_opened
            && !self.window_closed
            && self.manual_start_required
            && !self.autostart
            && !self.boot_graphics
            && !self.rootfs_packaged
            && self.recovery_safe
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManualExecutionWindowBridgeProbe {
    product: &'static str,
    status: &'static str,
    launch_mode: &'static str,
    source_execution_ready: bool,
    source_execution_status: &'static str,
    source_backend_path: &'static str,
    source_display_started: bool,
    source_display_stopped: bool,
    source_safe_return_to_recovery: bool,
    window_backend: &'static str,
    feature_gate: &'static str,
    feature_gate_required: bool,
    feature_compiled: bool,
    host_window_ready: bool,
    frame_format: &'static str,
    execution_frame_checksum: u64,
    host_frame_checksum: u64,
    frame_checksum_matches: bool,
    bounded_frame_limit: u32,
    visible_window_bound: bool,
    window_opened: bool,
    window_closed: bool,
    manual_start_required: bool,
    autostart: bool,
    boot_graphics: bool,
    rootfs_packaged: bool,
    recovery_safe: bool,
}

impl ManualExecutionWindowBridgeProbe {
    fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "manual-execution-window-bridge-ready"
            && self.launch_mode == "manual-dev"
            && self.source_execution_ready
            && self.source_execution_status == "manual-nested-preview-execution-complete"
            && self.source_backend_path == "nested-dev-window"
            && self.source_display_started
            && self.source_display_stopped
            && self.source_safe_return_to_recovery
            && self.window_backend == "minifb"
            && self.feature_gate == "host-window-preview"
            && self.feature_gate_required
            && self.host_window_ready
            && self.frame_format == "raw-rgba8888-composited-client-preview"
            && self.execution_frame_checksum != 0
            && self.execution_frame_checksum == self.host_frame_checksum
            && self.frame_checksum_matches
            && self.bounded_frame_limit == HOST_WINDOW_SMOKE_FRAME_LIMIT
            && self.visible_window_bound
            && !self.window_opened
            && !self.window_closed
            && self.manual_start_required
            && !self.autostart
            && !self.boot_graphics
            && !self.rootfs_packaged
            && self.recovery_safe
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostDevHandoffSummary {
    product: &'static str,
    status: &'static str,
    launch_mode: &'static str,
    recovery_launcher_ready: bool,
    recovery_launcher_status: String,
    recovery_launcher_path: PathBuf,
    recovery_request_ready: bool,
    recovery_launch_plan_ready: bool,
    host_bridge_ready: bool,
    host_window_backend: &'static str,
    host_feature_gate: &'static str,
    next_qemu_command: &'static str,
    next_host_command: &'static str,
    host_tool_packaged: bool,
    rootfs_graphical_boot: bool,
    rootfs_autostart: bool,
    qemu_window_started: bool,
    preview_window_started: bool,
    fallback_tty_available: bool,
    recovery_safe: bool,
}

impl HostDevHandoffSummary {
    fn is_ready(&self) -> bool {
        self.product == PRODUCT
            && self.status == "host-dev-handoff-summary-ready"
            && self.launch_mode == "manual-dev"
            && self.recovery_launcher_ready
            && self.recovery_launcher_status == "qemu-safe-visible-nested-preview-launch-ready"
            && self.recovery_request_ready
            && self.recovery_launch_plan_ready
            && self.host_bridge_ready
            && self.host_window_backend == "minifb"
            && self.host_feature_gate == "host-window-preview"
            && !self.host_tool_packaged
            && !self.rootfs_graphical_boot
            && !self.rootfs_autostart
            && !self.qemu_window_started
            && !self.preview_window_started
            && self.fallback_tty_available
            && self.recovery_safe
    }
}

fn main() {
    let command = env::args().nth(1).unwrap_or_else(|| "status".to_string());

    match command.as_str() {
        "status" => print_status(),
        "boot-markers" => print_boot_markers(),
        "probe-preview-window" => probe_preview_window(),
        "probe-nested-output-presenter" => probe_nested_output_presenter(),
        "probe-host-window-lifecycle" => probe_host_window_lifecycle_cli(),
        "probe-manual-execution-window-bridge" => probe_manual_execution_window_bridge_cli(),
        "handoff-summary" => host_dev_handoff_summary_cli(),
        "smoke-host-window-lifecycle" => smoke_host_window_lifecycle(),
        "smoke-manual-execution-window" => smoke_manual_execution_window(),
        "preview-window" => preview_window(),
        _ => {
            eprintln!("unknown command: {command}");
            eprintln!(
                "usage: aqua-host-tools [status|boot-markers|probe-preview-window|probe-nested-output-presenter|probe-host-window-lifecycle|probe-manual-execution-window-bridge|handoff-summary|smoke-host-window-lifecycle|smoke-manual-execution-window|preview-window]"
            );
            std::process::exit(2);
        }
    }
}

fn print_status() {
    println!("{PRODUCT}");
    println!("base: {OS_BASE}");
    println!("first-dev-target: {DEV_TARGET}");
    println!("graphics-target: {GRAPHICS_TARGET}");
    println!("milestone: 0/1 bootstrap");
}

fn print_boot_markers() {
    for marker in [
        "[AQUA-BOOT] stage=rcS-start product=\"Aqua Linux\"",
        "[AQUA-BOOT] stage=filesystems-mounted status=ok",
        "[AQUA-BOOT] stage=os-release id=aqua pretty=\"Aqua Linux Milestone 1\"",
        "[AQUA-BOOT] stage=runtime-assets-ready milestone=2 status=ok",
        "[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh",
    ] {
        println!("{marker}");
    }
}

fn probe_preview_window() {
    let handoff = probe_display_output_handoff(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT));
    let frame = export_composited_presenter_frame();
    let frame_ready = frame
        .as_ref()
        .map(|frame| frame.is_ready())
        .unwrap_or(false);

    println!("product={PRODUCT}");
    println!("tool=aqua-host-tools");
    println!("launch_mode=manual-dev");
    println!("window_backend=minifb");
    println!(
        "frame_size={}x{}",
        handoff.output_width, handoff.output_height
    );
    println!("pixel_format={}", handoff.pixel_format);
    println!("frame_bytes={}", handoff.frame_buffer_bytes);
    println!(
        "client_layer_buffer_snapshot_bytes={}",
        handoff.client_layer_buffer_snapshot_bytes
    );
    println!(
        "client_layer_snapshot_mode={}",
        handoff.client_layer_snapshot_mode
    );
    println!("frame_source=display-output-handoff-composited-client-frame");
    if let Ok(frame) = &frame {
        println!("frame_format={}", frame.format);
        println!("frame_checksum={:016x}", frame.checksum);
    } else {
        println!("frame_format=missing");
        println!("frame_checksum=0000000000000000");
    }
    println!("handoff_ready={}", state(handoff.is_ready()));
    println!("frame_ready={}", state(frame_ready));
    println!("display_output_started={}", handoff.display_output_started);
    println!("preview_window_feature=host-window-preview");
    println!("autostart=false");
    println!("boot_graphics=false");
    println!("rootfs_packaged=false");
    println!(
        "[AQUA-HOST] stage=preview-window-probe status={}",
        if handoff.is_ready() && frame_ready {
            "ok"
        } else {
            "error"
        }
    );
}

fn probe_host_window_lifecycle_cli() {
    let probe = probe_host_window_lifecycle();

    println!("product={}", probe.product);
    println!("tool=aqua-host-tools");
    println!("window_status={}", probe.status);
    println!("launch_mode={}", probe.launch_mode);
    println!("window_backend={}", probe.window_backend);
    println!("feature_gate={}", probe.feature_gate);
    println!("feature_gate_required={}", probe.feature_gate_required);
    println!("feature_compiled={}", probe.feature_compiled);
    println!(
        "source_presenter_ready={}",
        state(probe.source_presenter_ready)
    );
    println!(
        "source_surface_lifecycle_ready={}",
        state(probe.source_surface_lifecycle_ready)
    );
    println!("frame_ready={}", state(probe.frame_ready));
    println!("frame_format={}", probe.frame_format);
    println!("frame_checksum={:016x}", probe.frame_checksum);
    println!("bounded_frame_limit={}", probe.bounded_frame_limit);
    println!("window_opened={}", probe.window_opened);
    println!("window_closed={}", probe.window_closed);
    println!("manual_start_required={}", probe.manual_start_required);
    println!("autostart={}", probe.autostart);
    println!("boot_graphics={}", probe.boot_graphics);
    println!("rootfs_packaged={}", probe.rootfs_packaged);
    println!("recovery_safe={}", state(probe.recovery_safe));
    println!(
        "[AQUA-HOST] stage=host-window-lifecycle-probe status={}",
        if probe.is_ready() { "ok" } else { "error" }
    );
}

fn probe_host_window_lifecycle() -> HostWindowLifecycleProbe {
    let handoff = probe_display_output_handoff(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT));
    let lifecycle =
        run_nested_output_surface_lifecycle(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT), 3);
    let frame = export_composited_presenter_frame();
    let frame_ready = frame
        .as_ref()
        .map(|frame| frame.is_ready())
        .unwrap_or(false);
    let lifecycle_ready = lifecycle
        .as_ref()
        .map(|lifecycle| lifecycle.is_ready())
        .unwrap_or(false);
    let lifecycle_checksum_matches = match (&lifecycle, &frame) {
        (Ok(lifecycle), Ok(frame)) => lifecycle.frame_checksum == frame.checksum,
        _ => false,
    };
    let presenter_ready = handoff.is_ready()
        && frame_ready
        && lifecycle_ready
        && lifecycle_checksum_matches
        && handoff.target_backend == "nested-dev-window"
        && handoff.output_surface_prepared
        && !handoff.display_output_started
        && !handoff.renderer_started
        && !handoff.boot_graphics
        && !handoff.desktop_shell_started;
    let (frame_format, frame_checksum) = frame
        .as_ref()
        .map(|frame| (frame.format, frame.checksum))
        .unwrap_or(("missing", 0));

    HostWindowLifecycleProbe {
        product: PRODUCT,
        status: "manual-host-window-lifecycle-ready",
        launch_mode: "manual-dev",
        window_backend: "minifb",
        feature_gate: "host-window-preview",
        feature_gate_required: true,
        feature_compiled: cfg!(feature = "host-window-preview"),
        source_presenter_ready: presenter_ready,
        source_surface_lifecycle_ready: lifecycle_ready,
        frame_ready,
        frame_format,
        frame_checksum,
        bounded_frame_limit: HOST_WINDOW_FRAME_LIMIT,
        window_opened: false,
        window_closed: false,
        manual_start_required: true,
        autostart: false,
        boot_graphics: false,
        rootfs_packaged: false,
        recovery_safe: handoff.recovery_safe,
    }
}

fn probe_manual_execution_window_bridge_cli() {
    let probe = probe_manual_execution_window_bridge().unwrap_or_else(|error| {
        eprintln!("manual execution window bridge probe failed: {error}");
        std::process::exit(1);
    });

    println!("product={}", probe.product);
    println!("tool=aqua-host-tools");
    println!("bridge_status={}", probe.status);
    println!("launch_mode={}", probe.launch_mode);
    println!(
        "source_execution_ready={}",
        state(probe.source_execution_ready)
    );
    println!("source_execution_status={}", probe.source_execution_status);
    println!("source_backend_path={}", probe.source_backend_path);
    println!("source_display_started={}", probe.source_display_started);
    println!("source_display_stopped={}", probe.source_display_stopped);
    println!(
        "source_safe_return_to_recovery={}",
        state(probe.source_safe_return_to_recovery)
    );
    println!("window_backend={}", probe.window_backend);
    println!("feature_gate={}", probe.feature_gate);
    println!("feature_gate_required={}", probe.feature_gate_required);
    println!("feature_compiled={}", probe.feature_compiled);
    println!("host_window_ready={}", state(probe.host_window_ready));
    println!("frame_format={}", probe.frame_format);
    println!(
        "execution_frame_checksum={:016x}",
        probe.execution_frame_checksum
    );
    println!("host_frame_checksum={:016x}", probe.host_frame_checksum);
    println!(
        "frame_checksum_matches={}",
        state(probe.frame_checksum_matches)
    );
    println!("bounded_frame_limit={}", probe.bounded_frame_limit);
    println!("visible_window_bound={}", state(probe.visible_window_bound));
    println!("window_opened={}", probe.window_opened);
    println!("window_closed={}", probe.window_closed);
    println!("manual_start_required={}", probe.manual_start_required);
    println!("autostart={}", probe.autostart);
    println!("boot_graphics={}", probe.boot_graphics);
    println!("rootfs_packaged={}", probe.rootfs_packaged);
    println!("recovery_safe={}", state(probe.recovery_safe));
    println!(
        "[AQUA-HOST] stage=manual-execution-window-bridge status={}",
        if probe.is_ready() { "ok" } else { "error" }
    );
}

fn probe_manual_execution_window_bridge(
) -> Result<ManualExecutionWindowBridgeProbe, Box<dyn std::error::Error>> {
    let execution =
        run_manual_nested_preview_execution(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT), 3, true)?;
    let host_window = probe_host_window_lifecycle();

    let frame_checksum_matches =
        execution.frame_checksum != 0 && execution.frame_checksum == host_window.frame_checksum;
    let host_window_ready = host_window.is_ready()
        && host_window.window_backend == "minifb"
        && host_window.frame_format == execution.frame_format
        && frame_checksum_matches;

    Ok(ManualExecutionWindowBridgeProbe {
        product: PRODUCT,
        status: "manual-execution-window-bridge-ready",
        launch_mode: "manual-dev",
        source_execution_ready: execution.is_ready(),
        source_execution_status: execution.status,
        source_backend_path: execution.backend_path,
        source_display_started: execution.display_output_started,
        source_display_stopped: execution.display_output_stopped,
        source_safe_return_to_recovery: execution.safe_return_to_recovery,
        window_backend: host_window.window_backend,
        feature_gate: host_window.feature_gate,
        feature_gate_required: true,
        feature_compiled: cfg!(feature = "host-window-preview"),
        host_window_ready,
        frame_format: execution.frame_format,
        execution_frame_checksum: execution.frame_checksum,
        host_frame_checksum: host_window.frame_checksum,
        frame_checksum_matches,
        bounded_frame_limit: HOST_WINDOW_SMOKE_FRAME_LIMIT,
        visible_window_bound: true,
        window_opened: false,
        window_closed: false,
        manual_start_required: true,
        autostart: false,
        boot_graphics: false,
        rootfs_packaged: false,
        recovery_safe: execution.recovery_safe && host_window.recovery_safe,
    })
}

fn host_dev_handoff_summary_cli() {
    let summary = host_dev_handoff_summary().unwrap_or_else(|error| {
        eprintln!("host/dev handoff summary failed: {error}");
        std::process::exit(1);
    });

    println!("product={}", summary.product);
    println!("tool=aqua-host-tools");
    println!("handoff_status={}", summary.status);
    println!("launch_mode={}", summary.launch_mode);
    println!(
        "recovery_launcher_ready={}",
        state(summary.recovery_launcher_ready)
    );
    println!(
        "recovery_launcher_status={}",
        summary.recovery_launcher_status
    );
    println!(
        "recovery_launcher_path={}",
        summary.recovery_launcher_path.display()
    );
    println!(
        "recovery_request_ready={}",
        state(summary.recovery_request_ready)
    );
    println!(
        "recovery_launch_plan_ready={}",
        state(summary.recovery_launch_plan_ready)
    );
    println!("host_bridge_ready={}", state(summary.host_bridge_ready));
    println!("host_window_backend={}", summary.host_window_backend);
    println!("host_feature_gate={}", summary.host_feature_gate);
    println!("next_qemu_command={}", summary.next_qemu_command);
    println!("next_host_command={}", summary.next_host_command);
    println!("host_tool_packaged={}", summary.host_tool_packaged);
    println!("rootfs_graphical_boot={}", summary.rootfs_graphical_boot);
    println!("rootfs_autostart={}", summary.rootfs_autostart);
    println!("qemu_window_started={}", summary.qemu_window_started);
    println!("preview_window_started={}", summary.preview_window_started);
    println!("fallback_tty_available={}", summary.fallback_tty_available);
    println!("recovery_safe={}", state(summary.recovery_safe));
    println!(
        "[AQUA-HOST] stage=host-dev-handoff-summary status={}",
        if summary.is_ready() { "ok" } else { "error" }
    );

    if !summary.is_ready() {
        std::process::exit(1);
    }
}

fn host_dev_handoff_summary() -> Result<HostDevHandoffSummary, Box<dyn std::error::Error>> {
    let launcher_path = recovery_launcher_artifact_path();
    let launcher = fs::read_to_string(&launcher_path)?;
    let bridge = probe_manual_execution_window_bridge()?;

    let recovery_launcher_status =
        value_for_key(&launcher, "launch_status").unwrap_or_else(|| "missing".to_string());
    let recovery_launcher_ready =
        launcher.contains("[AQUA-PREVIEW] stage=visible-nested-preview-launch status=ok");
    let recovery_request_ready = launcher.contains("launch_request_ready=ok")
        && launcher.contains("request_command_ready=ok");
    let recovery_launch_plan_ready = launcher.contains("launch_plan_written=ok")
        && launcher.contains("launch_window_backend=minifb")
        && launcher.contains("launch_feature_gate=host-window-preview");
    let rootfs_graphical_boot = !launcher.contains("launch_boot_graphics=false");
    let rootfs_autostart = !launcher.contains("launch_autostart=false");
    let qemu_window_started = !launcher.contains("launch_qemu_window_started=false");
    let preview_window_started = !launcher.contains("launch_preview_window_started=false");
    let fallback_tty_available = launcher.contains("fallback_tty_available=true");
    let host_tool_packaged = !launcher.contains("launch_host_tool_packaged=false");

    Ok(HostDevHandoffSummary {
        product: PRODUCT,
        status: "host-dev-handoff-summary-ready",
        launch_mode: "manual-dev",
        recovery_launcher_ready,
        recovery_launcher_status,
        recovery_launcher_path: launcher_path,
        recovery_request_ready,
        recovery_launch_plan_ready,
        host_bridge_ready: bridge.is_ready(),
        host_window_backend: "minifb",
        host_feature_gate: "host-window-preview",
        next_qemu_command: "/usr/bin/aqua-visible-preview-launch",
        next_host_command:
            "aqua-host-tools --features host-window-preview -- smoke-manual-execution-window",
        host_tool_packaged,
        rootfs_graphical_boot,
        rootfs_autostart,
        qemu_window_started,
        preview_window_started,
        fallback_tty_available,
        recovery_safe: bridge.recovery_safe && launcher.contains("safe_return_to_recovery=ok"),
    })
}

fn recovery_launcher_artifact_path() -> PathBuf {
    env::var_os("AQUA_VISIBLE_PREVIEW_LAUNCH_ARTIFACT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("build/rootfs-compositor-contract/visible-preview-launch.txt")
        })
}

fn value_for_key(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let (line_key, value) = line.split_once('=')?;
        if line_key == key {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn probe_nested_output_presenter() {
    let handoff = probe_display_output_handoff(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT));
    let lifecycle =
        run_nested_output_surface_lifecycle(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT), 3);
    let frame = export_composited_presenter_frame();
    let frame_ready = frame
        .as_ref()
        .map(|frame| frame.is_ready())
        .unwrap_or(false);
    let lifecycle_ready = lifecycle
        .as_ref()
        .map(|lifecycle| lifecycle.is_ready())
        .unwrap_or(false);
    let lifecycle_checksum_matches = match (&lifecycle, &frame) {
        (Ok(lifecycle), Ok(frame)) => lifecycle.frame_checksum == frame.checksum,
        _ => false,
    };
    let presenter_ready = handoff.is_ready()
        && frame_ready
        && lifecycle_ready
        && lifecycle_checksum_matches
        && handoff.target_backend == "nested-dev-window"
        && handoff.output_surface_prepared
        && !handoff.display_output_started
        && !handoff.renderer_started
        && !handoff.boot_graphics
        && !handoff.desktop_shell_started;

    println!("product={PRODUCT}");
    println!("tool=aqua-host-tools");
    println!("presenter_status=manual-nested-output-presenter-ready");
    println!("source_handoff_ready={}", state(handoff.is_ready()));
    println!("source_surface_lifecycle_ready={}", state(lifecycle_ready));
    println!("target_backend={}", handoff.target_backend);
    println!(
        "frame_size={}x{}",
        handoff.output_width, handoff.output_height
    );
    println!("pixel_format={}", handoff.pixel_format);
    println!("frame_buffer_bytes={}", handoff.frame_buffer_bytes);
    println!(
        "client_layer_buffer_snapshot_bytes={}",
        handoff.client_layer_buffer_snapshot_bytes
    );
    println!(
        "client_layer_snapshot_mode={}",
        handoff.client_layer_snapshot_mode
    );
    println!("frame_source=display-output-handoff-composited-client-frame");
    if let Ok(frame) = &frame {
        println!("frame_format={}", frame.format);
        println!("frame_checksum={:016x}", frame.checksum);
    } else {
        println!("frame_format=missing");
        println!("frame_checksum=0000000000000000");
    }
    println!("frame_ready={}", state(frame_ready));
    if let Ok(lifecycle) = &lifecycle {
        println!("surface_status={}", lifecycle.status);
        println!("surface_acquired={}", state(lifecycle.surface_acquired));
        println!("surface_configured={}", state(lifecycle.surface_configured));
        println!("frame_attached={}", state(lifecycle.frame_attached));
        println!("frame_presented={}", state(lifecycle.frame_presented));
        println!("surface_released={}", state(lifecycle.surface_released));
        println!("presented_frames={}", lifecycle.presented_frames);
        println!("surface_frame_checksum={:016x}", lifecycle.frame_checksum);
        println!("lifecycle_serial={}", lifecycle.lifecycle_serial);
    } else {
        println!("surface_status=missing");
        println!("surface_acquired=missing");
        println!("surface_configured=missing");
        println!("frame_attached=missing");
        println!("frame_presented=missing");
        println!("surface_released=missing");
        println!("presented_frames=0");
        println!("surface_frame_checksum=0000000000000000");
        println!("lifecycle_serial=0");
    }
    println!(
        "surface_frame_matches_presenter_frame={}",
        state(lifecycle_checksum_matches)
    );
    println!(
        "output_surface_prepared={}",
        state(handoff.output_surface_prepared)
    );
    println!("manual_start_required=true");
    println!("display_output_started={}", handoff.display_output_started);
    println!("renderer_started={}", handoff.renderer_started);
    println!("boot_graphics={}", handoff.boot_graphics);
    println!("desktop_shell_started={}", handoff.desktop_shell_started);
    println!("rootfs_packaged=false");
    println!(
        "[AQUA-HOST] stage=nested-output-presenter-probe status={}",
        if presenter_ready { "ok" } else { "error" }
    );
}

#[cfg(feature = "host-window-preview")]
fn preview_window() {
    let run = run_minifb_preview_window(HOST_WINDOW_FRAME_LIMIT).unwrap_or_else(|error| {
        eprintln!("preview window failed: {error}");
        println!("[AQUA-HOST] stage=preview-window status=error");
        std::process::exit(1);
    });

    println!("product={PRODUCT}");
    println!("window_backend=minifb");
    println!("rendered_frames={}", run.rendered_frames);
    println!("bounded_frame_limit={HOST_WINDOW_FRAME_LIMIT}");
    println!("frame_checksum={:016x}", run.frame_checksum);
    println!("autostart=false");
    println!("boot_graphics=false");
    println!("rootfs_packaged=false");
    println!("[AQUA-HOST] stage=preview-window status=ok");
}

#[cfg(feature = "host-window-preview")]
fn smoke_host_window_lifecycle() {
    let run = run_minifb_preview_window(HOST_WINDOW_SMOKE_FRAME_LIMIT).unwrap_or_else(|error| {
        eprintln!("host window lifecycle smoke failed: {error}");
        println!("[AQUA-HOST] stage=host-window-lifecycle-smoke status=error");
        std::process::exit(1);
    });

    let ready = run.window_opened
        && run.window_closed
        && run.rendered_frames <= HOST_WINDOW_SMOKE_FRAME_LIMIT
        && run.rendered_frames > 0
        && run.frame_checksum != 0;

    println!("product={PRODUCT}");
    println!("tool=aqua-host-tools");
    println!("window_status=manual-host-window-lifecycle-complete");
    println!("launch_mode=manual-dev");
    println!("window_backend=minifb");
    println!("feature_gate=host-window-preview");
    println!("feature_compiled=true");
    println!("window_opened={}", run.window_opened);
    println!("rendered_frames={}", run.rendered_frames);
    println!("window_closed={}", run.window_closed);
    println!("bounded_frame_limit={HOST_WINDOW_SMOKE_FRAME_LIMIT}");
    println!("frame_checksum={:016x}", run.frame_checksum);
    println!("manual_start_required=true");
    println!("autostart=false");
    println!("boot_graphics=false");
    println!("rootfs_packaged=false");
    println!(
        "[AQUA-HOST] stage=host-window-lifecycle-smoke status={}",
        if ready { "ok" } else { "error" }
    );

    if !ready {
        std::process::exit(1);
    }
}

#[cfg(feature = "host-window-preview")]
fn smoke_manual_execution_window() {
    let probe = probe_manual_execution_window_bridge().unwrap_or_else(|error| {
        eprintln!("manual execution window bridge failed: {error}");
        println!("[AQUA-HOST] stage=manual-execution-window-smoke status=error");
        std::process::exit(1);
    });
    if !probe.is_ready() {
        eprintln!("manual execution window bridge is not ready");
        println!("[AQUA-HOST] stage=manual-execution-window-smoke status=error");
        std::process::exit(1);
    }

    let run = run_minifb_preview_window(HOST_WINDOW_SMOKE_FRAME_LIMIT).unwrap_or_else(|error| {
        eprintln!("manual execution window smoke failed: {error}");
        println!("[AQUA-HOST] stage=manual-execution-window-smoke status=error");
        std::process::exit(1);
    });

    let ready = run.window_opened
        && run.window_closed
        && run.rendered_frames <= HOST_WINDOW_SMOKE_FRAME_LIMIT
        && run.rendered_frames > 0
        && run.frame_checksum == probe.execution_frame_checksum;

    println!("product={PRODUCT}");
    println!("tool=aqua-host-tools");
    println!("window_status=manual-execution-window-complete");
    println!("launch_mode=manual-dev");
    println!("source_execution_ready=ok");
    println!("window_backend=minifb");
    println!("feature_gate=host-window-preview");
    println!("feature_compiled=true");
    println!("window_opened={}", run.window_opened);
    println!("rendered_frames={}", run.rendered_frames);
    println!("window_closed={}", run.window_closed);
    println!("bounded_frame_limit={HOST_WINDOW_SMOKE_FRAME_LIMIT}");
    println!("frame_checksum={:016x}", run.frame_checksum);
    println!(
        "frame_checksum_matches_execution={}",
        state(run.frame_checksum == probe.execution_frame_checksum)
    );
    println!("manual_start_required=true");
    println!("autostart=false");
    println!("boot_graphics=false");
    println!("rootfs_packaged=false");
    println!(
        "[AQUA-HOST] stage=manual-execution-window-smoke status={}",
        if ready { "ok" } else { "error" }
    );

    if !ready {
        std::process::exit(1);
    }
}

fn export_composited_presenter_frame() -> Result<RasterRgbaExport, Box<dyn std::error::Error>> {
    let viewport = Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT);
    let pipeline = probe_client_layer_pipeline(viewport)?;

    Ok(export_composited_preview_rgba_with_client_layers(
        viewport,
        &pipeline.paint_plan,
    ))
}

fn state(ready: bool) -> &'static str {
    if ready {
        "ok"
    } else {
        "missing"
    }
}

#[cfg(not(feature = "host-window-preview"))]
fn preview_window() {
    eprintln!("preview-window requires: cargo run -p aqua-host-tools --features host-window-preview -- preview-window");
    println!("[AQUA-HOST] stage=preview-window status=feature-disabled");
    std::process::exit(2);
}

#[cfg(not(feature = "host-window-preview"))]
fn smoke_host_window_lifecycle() {
    eprintln!(
        "smoke-host-window-lifecycle requires: cargo run -p aqua-host-tools --features host-window-preview -- smoke-host-window-lifecycle"
    );
    println!("[AQUA-HOST] stage=host-window-lifecycle-smoke status=feature-disabled");
    std::process::exit(2);
}

#[cfg(not(feature = "host-window-preview"))]
fn smoke_manual_execution_window() {
    eprintln!(
        "smoke-manual-execution-window requires: cargo run -p aqua-host-tools --features host-window-preview -- smoke-manual-execution-window"
    );
    println!("[AQUA-HOST] stage=manual-execution-window-smoke status=feature-disabled");
    std::process::exit(2);
}

#[cfg(feature = "host-window-preview")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HostWindowRunResult {
    window_opened: bool,
    rendered_frames: u32,
    window_closed: bool,
    frame_checksum: u64,
}

#[cfg(feature = "host-window-preview")]
fn run_minifb_preview_window(
    max_frames: u32,
) -> Result<HostWindowRunResult, Box<dyn std::error::Error>> {
    use minifb::{Key, Window, WindowOptions};

    let export = export_composited_presenter_frame()?;
    if !export.is_ready() {
        return Err("preview frame is not ready".into());
    }

    let frame = rgba_to_minifb_buffer(&export.bytes);
    let mut window = Window::new(
        "Aqua Linux compositor preview",
        export.width as usize,
        export.height as usize,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )?;

    window.set_target_fps(60);

    let mut frames = 0_u32;
    while window.is_open() && !window.is_key_down(Key::Escape) && frames < max_frames {
        window.update_with_buffer(&frame, export.width as usize, export.height as usize)?;
        frames += 1;
    }

    Ok(HostWindowRunResult {
        window_opened: true,
        rendered_frames: frames,
        window_closed: true,
        frame_checksum: export.checksum,
    })
}

#[cfg(feature = "host-window-preview")]
fn rgba_to_minifb_buffer(rgba: &[u8]) -> Vec<u32> {
    rgba.chunks_exact(4)
        .map(|pixel| (u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_frame_contract_is_host_window_ready() {
        let handoff = probe_display_output_handoff(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT));

        assert!(handoff.is_ready());
        assert_eq!(handoff.frame_buffer_bytes, 6_291_456);
        assert_eq!(handoff.client_layer_buffer_snapshot_bytes, 674_816);
        assert_eq!(handoff.client_layer_snapshot_mode, "full-buffer-snapshot");
    }

    #[test]
    fn static_preview_frame_export_is_still_available_for_manual_window() {
        let export = export_software_raster_rgba_for_static_scene(Viewport::new(
            PREVIEW_WIDTH,
            PREVIEW_HEIGHT,
        ));

        assert!(export.is_ready());
        assert_eq!(export.format, "raw-rgba8888");
        assert_eq!(export.byte_count, 6_291_456);
        assert_eq!(export.checksum, 0x717b_7e2c_50c3_29f1);
    }

    #[test]
    fn composited_presenter_frame_uses_client_layer_output() {
        let export = export_composited_presenter_frame()
            .expect("manual presenter should export composited client-layer frame");

        assert!(export.is_ready());
        assert_eq!(export.format, "raw-rgba8888-composited-client-preview");
        assert_eq!(export.byte_count, 6_291_456);
        assert_eq!(export.bytes.len(), 6_291_456);
        assert_ne!(export.checksum, 0);
        assert_ne!(export.checksum, 0x717b_7e2c_50c3_29f1);
    }

    #[test]
    fn nested_output_presenter_consumes_surface_lifecycle_frame() {
        let lifecycle =
            run_nested_output_surface_lifecycle(Viewport::new(PREVIEW_WIDTH, PREVIEW_HEIGHT), 3)
                .expect("manual presenter should read nested output surface lifecycle");
        let export = export_composited_presenter_frame()
            .expect("manual presenter should export composited client-layer frame");

        assert!(lifecycle.is_ready());
        assert!(export.is_ready());
        assert_eq!(lifecycle.status, "nested-output-surface-lifecycle-complete");
        assert_eq!(lifecycle.backend, "nested-dev-window");
        assert!(lifecycle.surface_acquired);
        assert!(lifecycle.surface_configured);
        assert!(lifecycle.frame_attached);
        assert!(lifecycle.frame_presented);
        assert!(lifecycle.surface_released);
        assert_eq!(lifecycle.presented_frames, 3);
        assert_eq!(lifecycle.frame_checksum, export.checksum);
        assert!(!lifecycle.autostart);
        assert!(!lifecycle.boot_graphics);
        assert!(!lifecycle.renderer_started);
        assert!(!lifecycle.desktop_shell_started);
    }

    #[test]
    fn host_window_lifecycle_probe_is_bounded_and_feature_gated() {
        let probe = probe_host_window_lifecycle();

        assert!(probe.is_ready());
        assert_eq!(probe.status, "manual-host-window-lifecycle-ready");
        assert_eq!(probe.launch_mode, "manual-dev");
        assert_eq!(probe.window_backend, "minifb");
        assert_eq!(probe.feature_gate, "host-window-preview");
        assert!(probe.feature_gate_required);
        assert!(probe.source_presenter_ready);
        assert!(probe.source_surface_lifecycle_ready);
        assert!(probe.frame_ready);
        assert_eq!(probe.frame_format, "raw-rgba8888-composited-client-preview");
        assert_ne!(probe.frame_checksum, 0);
        assert_eq!(probe.bounded_frame_limit, HOST_WINDOW_FRAME_LIMIT);
        assert!(!probe.window_opened);
        assert!(!probe.window_closed);
        assert!(probe.manual_start_required);
        assert!(!probe.autostart);
        assert!(!probe.boot_graphics);
        assert!(!probe.rootfs_packaged);
        assert!(probe.recovery_safe);
    }

    #[test]
    fn manual_execution_window_bridge_matches_execution_frame() {
        let probe = probe_manual_execution_window_bridge()
            .expect("manual execution should bridge to the host window frame");

        assert!(probe.is_ready());
        assert_eq!(probe.status, "manual-execution-window-bridge-ready");
        assert_eq!(probe.launch_mode, "manual-dev");
        assert!(probe.source_execution_ready);
        assert_eq!(
            probe.source_execution_status,
            "manual-nested-preview-execution-complete"
        );
        assert_eq!(probe.source_backend_path, "nested-dev-window");
        assert!(probe.source_display_started);
        assert!(probe.source_display_stopped);
        assert!(probe.source_safe_return_to_recovery);
        assert_eq!(probe.window_backend, "minifb");
        assert_eq!(probe.feature_gate, "host-window-preview");
        assert!(probe.feature_gate_required);
        assert!(probe.host_window_ready);
        assert_eq!(probe.frame_format, "raw-rgba8888-composited-client-preview");
        assert_ne!(probe.execution_frame_checksum, 0);
        assert_eq!(probe.execution_frame_checksum, probe.host_frame_checksum);
        assert!(probe.frame_checksum_matches);
        assert_eq!(probe.bounded_frame_limit, HOST_WINDOW_SMOKE_FRAME_LIMIT);
        assert!(probe.visible_window_bound);
        assert!(!probe.window_opened);
        assert!(!probe.window_closed);
        assert!(probe.manual_start_required);
        assert!(!probe.autostart);
        assert!(!probe.boot_graphics);
        assert!(!probe.rootfs_packaged);
        assert!(probe.recovery_safe);
    }

    #[test]
    fn host_dev_handoff_summary_pairs_recovery_launcher_with_host_bridge() {
        let temp_path = env::temp_dir().join(format!(
            "aqua-visible-preview-launch-test-{}.txt",
            std::process::id()
        ));
        fs::write(
            &temp_path,
            [
                "launch_status=qemu-safe-visible-nested-preview-launch-ready",
                "launch_request_ready=ok",
                "request_command_ready=ok",
                "launch_plan_written=ok",
                "launch_window_backend=minifb",
                "launch_feature_gate=host-window-preview",
                "launch_host_tool_packaged=false",
                "launch_qemu_window_started=false",
                "launch_preview_window_started=false",
                "launch_autostart=false",
                "launch_boot_graphics=false",
                "fallback_tty_available=true",
                "safe_return_to_recovery=ok",
                "[AQUA-PREVIEW] stage=visible-nested-preview-launch status=ok",
            ]
            .join("\n"),
        )
        .expect("test should write recovery launcher artifact");

        let previous = env::var_os("AQUA_VISIBLE_PREVIEW_LAUNCH_ARTIFACT");
        env::set_var("AQUA_VISIBLE_PREVIEW_LAUNCH_ARTIFACT", &temp_path);
        let summary = host_dev_handoff_summary()
            .expect("host/dev handoff summary should read launcher artifact");
        if let Some(previous) = previous {
            env::set_var("AQUA_VISIBLE_PREVIEW_LAUNCH_ARTIFACT", previous);
        } else {
            env::remove_var("AQUA_VISIBLE_PREVIEW_LAUNCH_ARTIFACT");
        }
        let _ = fs::remove_file(&temp_path);

        assert!(summary.is_ready());
        assert_eq!(summary.status, "host-dev-handoff-summary-ready");
        assert_eq!(
            summary.recovery_launcher_status,
            "qemu-safe-visible-nested-preview-launch-ready"
        );
        assert!(summary.recovery_launcher_ready);
        assert!(summary.recovery_request_ready);
        assert!(summary.recovery_launch_plan_ready);
        assert!(summary.host_bridge_ready);
        assert_eq!(summary.host_window_backend, "minifb");
        assert_eq!(summary.host_feature_gate, "host-window-preview");
        assert_eq!(
            summary.next_qemu_command,
            "/usr/bin/aqua-visible-preview-launch"
        );
        assert_eq!(
            summary.next_host_command,
            "aqua-host-tools --features host-window-preview -- smoke-manual-execution-window"
        );
        assert!(!summary.host_tool_packaged);
        assert!(!summary.rootfs_graphical_boot);
        assert!(!summary.rootfs_autostart);
        assert!(!summary.qemu_window_started);
        assert!(!summary.preview_window_started);
        assert!(summary.fallback_tty_available);
        assert!(summary.recovery_safe);
    }

    #[cfg(feature = "host-window-preview")]
    #[test]
    fn rgba_to_minifb_buffer_preserves_rgb_channels() {
        let buffer = rgba_to_minifb_buffer(&[0x04, 0x3b, 0x5c, 0xff]);

        assert_eq!(buffer, vec![0x0004_3b5c]);
    }
}
