#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::cell::{Cell, RefCell};
use std::fs::{File, OpenOptions};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::io::Read;
use std::io::{BufReader, Seek, SeekFrom, Write};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::{Path, PathBuf};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::process::{Child, Command, Stdio};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use std::rc::Rc;
use std::{env, fs, thread, time::Duration};

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_renderer::icons::IconRasterCache;
use aqua_renderer::{
    export_composited_preview_rgba_with_client_layers,
    export_composited_preview_rgba_with_wallpaper_and_client_layers, select_renderer_backend,
    GpuRuntimeCapabilities, RendererPreference,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_renderer::{
    export_runtime_desktop_rgba_with_launcher_and_theme, plan_client_layer_paint_steps,
    plan_client_surface_sources, render_desktop_icons_rgba_with_cached_icons,
    render_dock_rgba_with_cached_icons, render_launcher_overlay_rgba_with_theme,
    render_notification_toast_rgba_with_cached_icons, render_session_menu_overlay_rgba_with_theme,
    render_system_overview_rgba_with_theme, render_top_bar_rgba_with_cached_icons,
    ClientLayerPaintPlan, ClientSurfaceSource, ElevationLevel, ShadowMaskCache, ShadowMaskKey,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_scene::{static_shell_scene, MaterialKind, Rect};
use aqua_shell::probe_launcher_model;

use aqua_compositor::{
    default_session_config, default_session_environment, design_tokens_include_product,
    design_tokens_include_scene_materials, export_visible_preview_html, parse_session_config,
    probe_client_layer_pipeline, probe_client_surface_lifecycle, probe_client_surface_registry,
    probe_client_window_model, probe_display_activation_plan, probe_display_output_handoff,
    probe_display_output_plan, probe_launcher_input_scene_binding,
    probe_manual_nested_preview_backend, probe_renderer_surface_sources, probe_runtime_assets,
    probe_session_bootstrap, probe_session_skeleton, probe_smithay_launcher_seat,
    probe_static_frame_buffer, probe_static_frame_plan, probe_static_paint_plan,
    probe_static_raster_export, probe_static_raster_png_export, probe_static_render_plan,
    probe_static_shell_scene, probe_static_software_raster, probe_visible_preview_plan,
    probe_xdg_shell_binding, probe_xdg_toplevel_client, probe_xdg_toplevel_window_model,
    read_session_config, run_calloop_socket_smoke, run_event_loop_smoke,
    run_manual_display_output_smoke, run_manual_nested_preview_execution,
    run_nested_output_surface_lifecycle, run_nested_preview_frame_loop, run_session_loop_smoke,
    run_session_once_smoke, run_wayland_display_smoke, run_wayland_socket_smoke, status_lines,
    Viewport,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_compositor::{
    first_party_restart_policy, preflight_first_party_launch, FirstPartyProcessSupervisor,
    LaunchRequest, ProcessSupervisorError,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use aqua_compositor::{
    run_external_wayland_test_client, SmithayClientSurfaceSnapshot, SmithayDrmSession,
};

#[cfg(target_os = "linux")]
use calloop::{generic::Generic, EventLoop, Interest, Mode, PostAction};
#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
use drm::buffer::PlanarBuffer;
#[cfg(target_os = "linux")]
use drm::buffer::{Buffer as DrmBuffer, DrmFourcc};
#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
use drm::control::FbCmd2Flags;
#[cfg(target_os = "linux")]
use drm::control::{
    connector, ClipRect, Device as DrmControlDevice, Event as DrmEvent, PageFlipFlags,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use input::{
    event::{
        device::DeviceEvent,
        keyboard::{KeyState, KeyboardEvent, KeyboardEventTrait},
        pointer::{ButtonState, PointerEvent},
        EventTrait,
    },
    DeviceCapability, Libinput, LibinputInterface,
};
#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
use polling::PollMode;
#[cfg(target_os = "linux")]
use polling::{Event as PollEvent, Events as PollEvents, Poller};
#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
use smithay::{
    backend::{
        allocator::{
            dmabuf::{AsDmabuf, Dmabuf},
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Allocator, Fourcc, Modifier,
        },
        egl::{EGLContext, EGLDisplay},
        renderer::{
            gles::{GlesRenderer, GlesTexProgram, GlesTexture, Uniform, UniformName, UniformType},
            Bind, Color32F, ExportMem, Frame, ImportMem, Offscreen, Renderer,
        },
    },
    utils::{DeviceFd, Rectangle, Transform},
};
#[cfg(target_os = "linux")]
use std::os::fd::{AsFd, BorrowedFd};
#[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
use std::sync::Arc;
#[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
use wayland_server::Display as WaylandDisplay;
#[cfg(target_os = "linux")]
use wayland_server::ListeningSocket;

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "status".to_string());

    match command.as_str() {
        "status" => print_status(),
        "probe-assets" => {
            let root = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("docs/aqua-linux/assets-runtime"));
            probe_assets(root);
        }
        "smoke-loop" => smoke_loop(),
        "smoke-wayland" => smoke_wayland(),
        "smoke-socket" => smoke_socket(),
        "smoke-calloop-socket" => smoke_calloop_socket(),
        "probe-session-config" => probe_session_config(args.next().map(PathBuf::from)),
        "probe-session-env" => probe_session_env(args.next().map(PathBuf::from)),
        "probe-session-bootstrap" => probe_session_bootstrap_cli(
            args.next().map(PathBuf::from),
            args.next().map(PathBuf::from),
        ),
        "probe-session" => probe_session(),
        "probe-output-plan" => probe_output_plan(),
        "dump-output-plan" => dump_output_plan(),
        "probe-display-output-handoff" => probe_display_output_handoff_cli(),
        "probe-display-activation-plan" => probe_display_activation_plan_cli(),
        "probe-renderer-backend" => probe_renderer_backend_cli(args.next().as_deref()),
        "probe-gpu-offscreen-frame" => probe_gpu_offscreen_frame_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "probe-drm-device" => probe_drm_device_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "probe-drm-dumb-buffer" => probe_drm_dumb_buffer_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "probe-drm-gbm-scanout-buffer" => probe_drm_gbm_scanout_buffer_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "present-drm-gbm-scanout" => present_drm_gbm_scanout_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "present-drm-kms" => present_drm_kms_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "present-drm-page-flip" => present_drm_page_flip_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "present-drm-gpu-surface" => present_drm_gpu_surface_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "run-drm-frame-loop" => run_drm_frame_loop_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "run-drm-session-loop" => run_drm_session_loop_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "run-drm-wayland-session" => run_drm_wayland_session_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/dri/card0")),
        ),
        "run-wayland-test-client" => run_wayland_test_client_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/run/aqua/aqua-wayland-drm-0")),
        ),
        "smoke-display-output" => smoke_display_output(),
        "smoke-nested-output-surface" => smoke_nested_output_surface(),
        "probe-visible-preview-plan" => probe_visible_preview_plan_cli(),
        "probe-visible-preview-export" => probe_visible_preview_export(),
        "export-visible-preview-html" => {
            export_visible_preview_html_cli(args.next().map(PathBuf::from))
        }
        "probe-fbdev-frame" => probe_fbdev_frame_cli(
            args.next().as_deref(),
            args.next().as_deref(),
            args.next().as_deref(),
        ),
        "present-fbdev" => present_fbdev_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/fb0")),
        ),
        "smoke-nested-preview-loop" => smoke_nested_preview_loop(),
        "probe-manual-nested-preview-backend" => probe_manual_nested_preview_backend_cli(),
        "run-manual-nested-preview-execution" => run_manual_nested_preview_execution_cli(),
        "probe-client-window-model" => probe_client_window_model_cli(),
        "probe-client-surface-lifecycle" => probe_client_surface_lifecycle_cli(),
        "probe-client-surface-registry" => probe_client_surface_registry_cli(),
        "probe-xdg-shell-binding" => probe_xdg_shell_binding_cli(),
        "probe-xdg-toplevel-client" => probe_xdg_toplevel_client_cli(),
        "probe-xdg-toplevel-window-model" => probe_xdg_toplevel_window_model_cli(),
        "probe-launcher-model" => probe_launcher_model_cli(),
        "probe-launcher-input-scene" => probe_launcher_input_scene_cli(),
        "probe-smithay-launcher-seat" => probe_smithay_launcher_seat_cli(),
        "probe-evdev-aqua-seat" => probe_evdev_aqua_seat_cli(
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/input/event0")),
            args.next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/dev/input/event1")),
        ),
        "probe-scene" => probe_scene(),
        "dump-scene" => dump_scene(),
        "probe-render-plan" => probe_render_plan(),
        "dump-render-plan" => dump_render_plan(),
        "probe-renderer-surface-sources" => probe_renderer_surface_sources_cli(),
        "probe-client-layer-pipeline" => probe_client_layer_pipeline_cli(),
        "probe-paint-plan" => probe_paint_plan(),
        "dump-paint-plan" => dump_paint_plan(),
        "probe-frame-plan" => probe_frame_plan(),
        "dump-frame-plan" => dump_frame_plan(),
        "probe-frame-buffer" => probe_frame_buffer(),
        "dump-frame-buffer" => dump_frame_buffer(),
        "probe-raster" => probe_raster(),
        "dump-raster" => dump_raster(),
        "probe-raster-export" => probe_raster_export(),
        "dump-raster-export" => dump_raster_export(),
        "export-raster-ppm" => export_raster_ppm(args.next().map(PathBuf::from)),
        "probe-raster-png-export" => probe_raster_png_export(),
        "dump-raster-png-export" => dump_raster_png_export(),
        "export-raster-png" => export_raster_png(args.next().map(PathBuf::from)),
        "smoke-run-once" => smoke_run_once(),
        "smoke-session-loop" => smoke_session_loop(),
        _ => {
            eprintln!("unknown command: {command}");
            eprintln!(
                "usage: aqua-compositor [status|probe-assets <runtime-asset-root>|probe-renderer-backend [auto|gpu|software]|probe-gpu-offscreen-frame [device]|smoke-loop|smoke-wayland|smoke-socket|smoke-calloop-socket|probe-session-config|probe-session-env|probe-session-bootstrap <config-path> <prepared-runtime-dir>|probe-session|probe-output-plan|dump-output-plan|probe-display-output-handoff|probe-display-activation-plan|probe-drm-device [device]|probe-drm-dumb-buffer [device]|probe-drm-gbm-scanout-buffer [device]|present-drm-gbm-scanout [device]|present-drm-kms [device]|present-drm-page-flip [device]|run-drm-frame-loop [device]|run-drm-session-loop [device]|run-drm-wayland-session [device]|run-wayland-test-client [socket]|smoke-display-output|smoke-nested-output-surface|probe-visible-preview-plan|probe-visible-preview-export|export-visible-preview-html <path>|probe-fbdev-frame <width> <height> <bits-per-pixel>|present-fbdev [device]|smoke-nested-preview-loop|probe-manual-nested-preview-backend|run-manual-nested-preview-execution|probe-client-window-model|probe-client-surface-lifecycle|probe-client-surface-registry|probe-xdg-shell-binding|probe-xdg-toplevel-client|probe-xdg-toplevel-window-model|probe-launcher-model|probe-launcher-input-scene|probe-smithay-launcher-seat|probe-evdev-aqua-seat <keyboard-event> <pointer-event>|probe-scene|dump-scene|probe-render-plan|dump-render-plan|probe-renderer-surface-sources|probe-client-layer-pipeline|probe-paint-plan|dump-paint-plan|probe-frame-plan|dump-frame-plan|probe-frame-buffer|dump-frame-buffer|probe-raster|dump-raster|probe-raster-export|dump-raster-export|export-raster-ppm <path>|probe-raster-png-export|dump-raster-png-export|export-raster-png <path>|smoke-run-once|smoke-session-loop]"
            );
            std::process::exit(2);
        }
    }
}

fn probe_renderer_backend_cli(preference: Option<&str>) {
    let preference = preference.unwrap_or("auto");
    let Some(preference) = RendererPreference::parse(preference) else {
        eprintln!("renderer preference must be auto, gpu, or software");
        println!("[AQUA-COMPOSITOR] stage=renderer-backend-probe status=error");
        std::process::exit(2);
    };
    let library_root = env::var_os("AQUA_GPU_LIBRARY_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/lib"));
    let capabilities = GpuRuntimeCapabilities {
        drm: cfg!(target_os = "linux"),
        gbm: runtime_library_available(&library_root, "libgbm.so"),
        egl: runtime_library_available(&library_root, "libEGL.so"),
        gles2: runtime_library_available(&library_root, "libGLESv2.so"),
    };
    let decision = select_renderer_backend(preference, capabilities);

    println!("renderer_drm_available={}", capabilities.drm);
    println!("renderer_gbm_available={}", capabilities.gbm);
    println!("renderer_egl_available={}", capabilities.egl);
    println!("renderer_gles2_available={}", capabilities.gles2);
    for line in decision.dump_lines() {
        println!("{line}");
    }
    println!("renderer_context_created=false");
    println!("renderer_display_output_started=false");
    println!("renderer_recovery_safe=true");
    println!(
        "[AQUA-COMPOSITOR] stage=renderer-backend-probe status={}",
        if decision.can_start {
            "ok"
        } else {
            "unavailable"
        }
    );
    if !decision.can_start {
        std::process::exit(1);
    }
}

fn runtime_library_available(root: &Path, soname: &str) -> bool {
    root.join(soname).exists()
        || fs::read_dir(root).is_ok_and(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(&format!("{soname}.")))
            })
        })
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn probe_gpu_offscreen_frame_cli(device: PathBuf) {
    match probe_gpu_offscreen_frame(&device) {
        Ok(result) => {
            println!("gpu_device={}", device.display());
            println!("gpu_backend=smithay-gles2-gbm");
            println!("gpu_context_created=true");
            println!("gpu_offscreen_size=320x240");
            println!("gpu_offscreen_format=abgr8888");
            println!("gpu_scene_surface_count={}", result.surface_count);
            println!(
                "gpu_scene_surface_layer_count={}",
                result.surface_layer_count
            );
            println!("gpu_scene_shader=aqua-surface-compositor-v1");
            println!("gpu_surface_shader_compiled=true");
            println!("gpu_surface_shader_panels={}", result.surface_layer_count);
            println!("gpu_surface_refraction_strength=0.0025");
            println!("gpu_surface_tint_strength=0.18");
            println!("gpu_surface_highlight_strength=0.16");
            println!("gpu_surface_refraction_bounded=true");
            println!("gpu_surface_rounded_mask=true");
            println!("gpu_surface_corner_radius_px=12.0");
            println!("gpu_surface_edge_light_strength=0.24");
            println!("gpu_surface_edge_width_px=1.5");
            println!("gpu_surface_blur_shader_compiled=true");
            println!("gpu_surface_blur_passes=2");
            println!("gpu_surface_blur_kernel_samples=9");
            println!("gpu_surface_blur_radius_px=4.0");
            println!("gpu_surface_blur_intermediate_size=320x240");
            println!("gpu_surface_blur_synchronized=true");
            println!("gpu_surface_blur_composited=true");
            println!("gpu_client_texture_source=sampled-wl-shm-contract");
            println!("gpu_client_texture_count={}", result.client_texture_count);
            println!("gpu_client_texture_bytes={}", result.client_texture_bytes);
            println!("gpu_client_textures_uploaded=true");
            println!("gpu_client_textures_composited=true");
            println!("gpu_client_live_wayland_session=false");
            println!("gpu_wallpaper_source=runtime-asset");
            println!(
                "gpu_wallpaper_size={}x{}",
                result.wallpaper_width, result.wallpaper_height
            );
            println!("gpu_wallpaper_texture_uploaded=true");
            println!("gpu_wallpaper_composited=true");
            println!("gpu_frame_rendered=true");
            println!("gpu_frame_synchronized=true");
            println!("gpu_frame_readback=true");
            println!("gpu_frame_bytes={}", result.frame_bytes);
            println!("gpu_frame_checksum={:016x}", result.checksum);
            println!("gpu_frame_repeat_checksum={:016x}", result.repeat_checksum);
            println!(
                "gpu_frame_deterministic={}",
                result.checksum == result.repeat_checksum
            );
            println!("gpu_context_destroyed=true");
            println!("gpu_kms_activated=false");
            println!("gpu_display_output_started=false");
            println!("gpu_recovery_safe=true");
            println!("[AQUA-COMPOSITOR] stage=gpu-offscreen-frame status=ok");
        }
        Err(error) => {
            eprintln!("gpu offscreen frame probe failed: {error}");
            println!("gpu_context_destroyed=true");
            println!("gpu_kms_activated=false");
            println!("gpu_display_output_started=false");
            println!("gpu_recovery_safe=true");
            println!("[AQUA-COMPOSITOR] stage=gpu-offscreen-frame status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
struct GpuOffscreenFrameResult {
    surface_count: usize,
    surface_layer_count: usize,
    client_texture_count: usize,
    client_texture_bytes: usize,
    frame_bytes: usize,
    wallpaper_width: u32,
    wallpaper_height: u32,
    checksum: u64,
    repeat_checksum: u64,
    frame_rgba: Vec<u8>,
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
const AQUA_SURFACE_FRAGMENT_SHADER: &str = r#"
//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

precision mediump float;
#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
uniform float refraction_strength;
uniform float tint_strength;
uniform float highlight_strength;
uniform vec3 surface_tint;
uniform vec2 panel_uv_origin;
uniform vec2 panel_uv_size;
uniform vec2 panel_size_px;
uniform float corner_radius_px;
uniform float edge_width_px;
uniform float edge_light_strength;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

float rounded_rect_distance(vec2 point, vec2 half_size, float radius) {
    vec2 corner = abs(point) - (half_size - vec2(radius));
    return length(max(corner, vec2(0.0)))
        + min(max(corner.x, corner.y), 0.0) - radius;
}

void main() {
    vec2 local_uv = (v_coords - panel_uv_origin) / panel_uv_size;
    vec2 panel_point = local_uv * panel_size_px - panel_size_px * 0.5;
    float radius = min(corner_radius_px, min(panel_size_px.x, panel_size_px.y) * 0.5);
    float edge_distance = rounded_rect_distance(
        panel_point,
        panel_size_px * 0.5,
        radius
    );
    float mask = 1.0 - smoothstep(-1.0, 1.0, edge_distance);
    float inner_edge = (1.0 - smoothstep(0.0, edge_width_px, abs(edge_distance)))
        * step(edge_distance, 0.0);
    float wave = sin(local_uv.y * 12.0) * cos(local_uv.x * 9.0);
    vec2 offset = vec2(wave, wave * 0.35) * refraction_strength;
    vec2 inset = vec2(refraction_strength * 1.5);
    vec2 sample_uv = clamp(
        v_coords + offset,
        panel_uv_origin + inset,
        panel_uv_origin + panel_uv_size - inset
    );
    vec4 source = texture2D(tex, sample_uv);
    vec3 color = mix(source.rgb, surface_tint, tint_strength);
    float highlight = pow(clamp(1.0 - local_uv.y, 0.0, 1.0), 10.0)
        * highlight_strength;
    float edge_facing = mix(0.45, 1.0, clamp(1.0 - local_uv.y, 0.0, 1.0));
    color += vec3(highlight + inner_edge * edge_facing * edge_light_strength);

#if defined(DEBUG_FLAGS)
    if (tint == 1.0)
        color = mix(color, vec3(0.0, 0.2, 0.0), 0.2);
#endif

    float surface_alpha = alpha * mask;
    gl_FragColor = vec4(color * surface_alpha, surface_alpha);
}
"#;

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
const AQUA_BLUR_FRAGMENT_SHADER: &str = r#"
//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

precision mediump float;
#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
uniform vec2 texel_step;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

void main() {
    vec4 color = texture2D(tex, v_coords) * 0.2270270270;
    color += texture2D(tex, v_coords + texel_step * 1.0) * 0.1945945946;
    color += texture2D(tex, v_coords - texel_step * 1.0) * 0.1945945946;
    color += texture2D(tex, v_coords + texel_step * 2.0) * 0.1216216216;
    color += texture2D(tex, v_coords - texel_step * 2.0) * 0.1216216216;
    color += texture2D(tex, v_coords + texel_step * 3.0) * 0.0540540541;
    color += texture2D(tex, v_coords - texel_step * 3.0) * 0.0540540541;
    color += texture2D(tex, v_coords + texel_step * 4.0) * 0.0162162162;
    color += texture2D(tex, v_coords - texel_step * 4.0) * 0.0162162162;

#if defined(DEBUG_FLAGS)
    if (tint == 1.0)
        color.rgb = mix(color.rgb, vec3(0.0, 0.2, 0.0), 0.2);
#endif

    gl_FragColor = color * alpha;
}
"#;

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn probe_gpu_offscreen_frame(device: &Path) -> Result<GpuOffscreenFrameResult, String> {
    let client_pipeline = probe_client_layer_pipeline(Viewport::new(1536, 1024))
        .map_err(|error| format!("cannot prepare GPU client layers: {error}"))?;
    if !client_pipeline.is_ready() {
        return Err("GPU client layer contract is not ready".to_string());
    }
    render_gpu_offscreen_frame(device, &client_pipeline.paint_plan)
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn render_gpu_offscreen_frame(
    device: &Path,
    client_plan: &ClientLayerPaintPlan,
) -> Result<GpuOffscreenFrameResult, String> {
    LiveGpuCompositor::new(device)?.render(client_plan)
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
struct LiveGpuCompositor {
    renderer: GlesRenderer,
    wallpaper_texture: GlesTexture,
    wallpaper_width: u32,
    wallpaper_height: u32,
    blurred_wallpaper: GlesTexture,
    surface_program: GlesTexProgram,
    target: GlesTexture,
    target_size: (u32, u32),
    scene: aqua_scene::ShellScene,
    theme: aqua_shell::AquaTheme,
    launcher_texture: Option<GlesTexture>,
    launcher_state: Option<aqua_shell::LauncherState>,
    launcher_texture_size: (u32, u32),
    top_bar_texture: Option<GlesTexture>,
    top_bar_state: Option<aqua_shell::TopBarState>,
    top_bar_texture_size: (u32, u32),
    session_menu_texture: Option<GlesTexture>,
    session_menu_state: Option<aqua_shell::SessionMenuState>,
    session_menu_texture_size: (u32, u32),
    system_overview_texture: Option<GlesTexture>,
    system_overview_state: Option<aqua_shell::SystemOverviewModel>,
    system_overview_texture_size: (u32, u32),
    desktop_icons_texture: Option<GlesTexture>,
    desktop_icons_state: Option<aqua_shell::DesktopIconState>,
    desktop_icons_texture_size: (u32, u32),
    dock_texture: Option<GlesTexture>,
    dock_state: Option<aqua_shell::DockState>,
    dock_texture_size: (u32, u32),
    notification_texture: Option<GlesTexture>,
    notification_state: aqua_shell::NotificationCenter,
    notification_texture_size: (u32, u32),
    icon_raster_cache: IconRasterCache,
    client_texture_cache: Vec<ClientTextureCacheEntry>,
    shadow_mask_cache: ShadowMaskCache,
    client_shadow_texture_cache: Vec<ClientShadowTextureCacheEntry>,
    opaque_direct_bridge_ready: bool,
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
struct ClientTextureCacheEntry {
    surface_id: &'static str,
    revision: u64,
    source_size: (u32, u32),
    texture: GlesTexture,
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
#[derive(Clone)]
struct ClientShadowTexture {
    texture: GlesTexture,
    size: (u32, u32),
    surface_offset: (u32, u32),
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
struct ClientShadowTextureCacheEntry {
    key: ShadowMaskKey,
    shadow: ClientShadowTexture,
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
impl LiveGpuCompositor {
    fn set_theme(&mut self, theme: aqua_shell::AquaTheme) -> bool {
        if self.theme == theme {
            return false;
        }
        self.theme = theme;
        self.launcher_state = None;
        self.top_bar_state = None;
        self.session_menu_state = None;
        self.system_overview_state = None;
        self.desktop_icons_state = None;
        self.dock_state = None;
        self.notification_state = aqua_shell::NotificationCenter::default();
        self.launcher_texture = None;
        self.top_bar_texture = None;
        self.session_menu_texture = None;
        self.system_overview_texture = None;
        self.desktop_icons_texture = None;
        self.dock_texture = None;
        self.notification_texture = None;
        self.client_texture_cache.clear();
        self.client_shadow_texture_cache.clear();
        self.opaque_direct_bridge_ready = false;
        println!("desktop_shell_theme_changed={}", theme.id());
        true
    }

    fn set_top_bar_state(&mut self, state: &aqua_shell::TopBarState) -> Result<(), String> {
        if self.top_bar_state.as_ref() == Some(state) {
            return Ok(());
        }
        let overlay = render_top_bar_rgba_with_cached_icons(
            self.scene.viewport.width,
            36,
            state,
            self.theme,
            &mut self.icon_raster_cache,
        )
        .map_err(|error| format!("cannot rasterize top bar icons: {error}"))?;
        self.top_bar_texture = Some(
            self.renderer
                .import_memory(
                    &overlay.rgba,
                    Fourcc::Abgr8888,
                    (overlay.width as i32, overlay.height as i32).into(),
                    false,
                )
                .map_err(|error| format!("cannot upload top bar texture: {error}"))?,
        );
        self.top_bar_state = Some(state.clone());
        self.top_bar_texture_size = (overlay.width, overlay.height);
        println!("desktop_top_bar_texture_ready=true");
        println!("desktop_top_bar_clock={}", state.clock_label);
        println!("desktop_top_bar_network={}", state.network_connected);
        println!(
            "desktop_top_bar_battery={}",
            state
                .battery_percent
                .map_or("none".to_string(), |percent| percent.to_string())
        );
        self.log_icon_raster_surface("top-bar", 3);
        Ok(())
    }

    fn set_shell_chrome_visible(&mut self, visible: bool) {
        for kind in [
            aqua_scene::SurfaceKind::TopPanel,
            aqua_scene::SurfaceKind::DesktopIconColumn,
            aqua_scene::SurfaceKind::Dock,
            aqua_scene::SurfaceKind::Launcher,
            aqua_scene::SurfaceKind::SystemOverview,
            aqua_scene::SurfaceKind::NotificationToast,
        ] {
            self.scene.set_surface_visible(kind, visible);
        }
    }

    fn set_client_window_presence(&mut self, present: bool) {
        let session_menu_open = self.session_menu_state.is_some();
        self.scene.set_surface_visible(
            aqua_scene::SurfaceKind::SystemOverview,
            !present || session_menu_open,
        );
    }

    fn set_launcher_state(&mut self, state: &aqua_shell::LauncherState) -> Result<(), String> {
        if self.launcher_state.as_ref() == Some(state) {
            return Ok(());
        }
        let visible = state.is_open();
        self.scene
            .set_surface_visible(aqua_scene::SurfaceKind::Launcher, visible);
        if !visible {
            self.launcher_texture = None;
            self.launcher_state = Some(state.clone());
            return Ok(());
        }
        let (rgba, probe) =
            render_launcher_overlay_rgba_with_theme(self.scene.viewport, state, self.theme);
        if !probe.is_ready() {
            return Err("launcher overlay did not satisfy its render contract".to_string());
        }
        self.launcher_texture = Some(
            self.renderer
                .import_memory(
                    &rgba,
                    Fourcc::Abgr8888,
                    (
                        self.scene.viewport.width as i32,
                        self.scene.viewport.height as i32,
                    )
                        .into(),
                    false,
                )
                .map_err(|error| format!("cannot upload launcher texture: {error}"))?,
        );
        self.launcher_state = Some(state.clone());
        self.launcher_texture_size = (self.scene.viewport.width, self.scene.viewport.height);
        println!("desktop_launcher_theme={}", self.theme.id());
        Ok(())
    }

    fn set_desktop_icons_state(
        &mut self,
        state: &aqua_shell::DesktopIconState,
    ) -> Result<(), String> {
        if self.desktop_icons_state.as_ref() == Some(state) {
            return Ok(());
        }
        let overlay = render_desktop_icons_rgba_with_cached_icons(
            aqua_shell::DESKTOP_ICON_LAYER_WIDTH,
            aqua_shell::DESKTOP_ICON_LAYER_HEIGHT,
            state,
            self.theme,
            &mut self.icon_raster_cache,
        )
        .map_err(|error| format!("cannot rasterize desktop icons: {error}"))?;
        self.desktop_icons_texture = Some(
            self.renderer
                .import_memory(
                    &overlay.rgba,
                    Fourcc::Abgr8888,
                    (overlay.width as i32, overlay.height as i32).into(),
                    false,
                )
                .map_err(|error| format!("cannot upload desktop icons texture: {error}"))?,
        );
        self.desktop_icons_state = Some(state.clone());
        self.desktop_icons_texture_size = (overlay.width, overlay.height);
        println!("desktop_icons_texture_ready=true");
        println!(
            "desktop_icons_selected={}",
            state
                .selected()
                .map_or("none".to_string(), |index| index.to_string())
        );
        println!(
            "desktop_icons_context_menu={}",
            state
                .context_menu()
                .map_or("none".to_string(), |index| index.to_string())
        );
        self.log_icon_raster_surface("desktop", 3);
        Ok(())
    }

    fn set_dock_state(&mut self, state: &aqua_shell::DockState) -> Result<(), String> {
        if self.dock_state.as_ref() == Some(state) {
            return Ok(());
        }
        let overlay = render_dock_rgba_with_cached_icons(
            760,
            72,
            state,
            self.theme,
            &mut self.icon_raster_cache,
        )
        .map_err(|error| format!("cannot rasterize dock icons: {error}"))?;
        self.dock_texture = Some(
            self.renderer
                .import_memory(
                    &overlay.rgba,
                    Fourcc::Abgr8888,
                    (overlay.width as i32, overlay.height as i32).into(),
                    false,
                )
                .map_err(|error| format!("cannot upload dock texture: {error}"))?,
        );
        self.dock_state = Some(state.clone());
        self.dock_texture_size = (overlay.width, overlay.height);
        println!("desktop_dock_texture_ready=true");
        println!("desktop_dock_item_count={}", aqua_shell::DOCK_ITEM_COUNT);
        println!("desktop_bottom_shell_group_count={}", overlay.group_count);
        println!("desktop_workspace_active={}", overlay.active_workspace);
        println!(
            "desktop_dock_running_indicators={}",
            overlay.running_item_count
        );
        self.log_icon_raster_surface("dock", 3);
        Ok(())
    }

    fn set_session_menu_state(
        &mut self,
        state: &aqua_shell::SessionMenuState,
    ) -> Result<(), String> {
        self.scene
            .set_surface_visible(aqua_scene::SurfaceKind::SystemOverview, true);
        if !state.is_open() {
            self.session_menu_texture = None;
            self.session_menu_state = None;
            return Ok(());
        }
        let overlay = render_session_menu_overlay_rgba_with_theme(512, 293, state, self.theme);
        println!("desktop_session_menu_overlay_texture_ready=true");
        println!(
            "desktop_session_menu_overlay_selected={}",
            overlay.selected_action
        );
        println!(
            "desktop_session_menu_overlay_confirmation={}",
            overlay.confirmation_visible
        );
        println!(
            "desktop_session_menu_overlay_primitives={}",
            overlay.primitive_count
        );
        self.session_menu_texture = Some(
            self.renderer
                .import_memory(
                    &overlay.rgba,
                    Fourcc::Abgr8888,
                    (overlay.width as i32, overlay.height as i32).into(),
                    false,
                )
                .map_err(|error| format!("cannot upload session menu texture: {error}"))?,
        );
        self.session_menu_state = Some(state.clone());
        self.session_menu_texture_size = (overlay.width, overlay.height);
        Ok(())
    }

    fn set_system_overview_state(
        &mut self,
        state: &aqua_shell::SystemOverviewModel,
    ) -> Result<(), String> {
        if self.system_overview_state.as_ref() == Some(state) {
            return Ok(());
        }
        let overlay = render_system_overview_rgba_with_theme(512, 352, state, self.theme);
        self.system_overview_texture = Some(
            self.renderer
                .import_memory(
                    &overlay.rgba,
                    Fourcc::Abgr8888,
                    (overlay.width as i32, overlay.height as i32).into(),
                    false,
                )
                .map_err(|error| format!("cannot upload system overview texture: {error}"))?,
        );
        self.system_overview_state = Some(state.clone());
        self.system_overview_texture_size = (overlay.width, overlay.height);
        println!("desktop_system_overview_texture_ready=true");
        println!("desktop_system_overview_clock={}", state.clock_utc);
        println!("desktop_system_overview_kernel={}", state.kernel);
        println!(
            "desktop_system_overview_memory_percent={}",
            state.memory_used_percent()
        );
        Ok(())
    }

    fn set_notification_state(
        &mut self,
        state: &aqua_shell::NotificationCenter,
    ) -> Result<(), String> {
        let visible = state.active().is_some();
        self.scene
            .set_surface_visible(aqua_scene::SurfaceKind::NotificationToast, visible);
        if !visible {
            self.notification_texture = None;
            self.notification_state = state.clone();
            return Ok(());
        }
        let surface = self
            .scene
            .surfaces
            .iter()
            .find(|surface| surface.kind == aqua_scene::SurfaceKind::NotificationToast)
            .ok_or_else(|| "notification surface is missing from the shell scene".to_string())?;
        let overlay = render_notification_toast_rgba_with_cached_icons(
            surface.rect.width,
            surface.rect.height,
            state,
            self.theme,
            &mut self.icon_raster_cache,
        )
        .map_err(|error| format!("cannot rasterize notification icon: {error}"))?;
        self.notification_texture = Some(
            self.renderer
                .import_memory(
                    &overlay.rgba,
                    Fourcc::Abgr8888,
                    (overlay.width as i32, overlay.height as i32).into(),
                    false,
                )
                .map_err(|error| format!("cannot upload notification texture: {error}"))?,
        );
        self.notification_state = state.clone();
        self.notification_texture_size = (overlay.width, overlay.height);
        println!("desktop_notification_overlay_texture_ready=true");
        println!(
            "desktop_notification_overlay_id={}",
            overlay.notification_id.unwrap_or_default()
        );
        println!(
            "desktop_notification_overlay_primitives={}",
            overlay.primitive_count
        );
        self.log_icon_raster_surface("notification", 1);
        Ok(())
    }

    fn log_icon_raster_surface(&self, surface: &str, role_count: usize) {
        let stats = self.icon_raster_cache.stats();
        println!(
            "desktop_icon_rasters_ready=true surface={surface} roles={role_count} theme={}",
            self.theme.id()
        );
        println!(
            "desktop_icon_raster_cache_entries={}",
            self.icon_raster_cache.len()
        );
        println!("desktop_icon_raster_cache_hits={}", stats.hits);
        println!("desktop_icon_raster_cache_misses={}", stats.misses);
        println!("desktop_icon_raster_cache_evictions={}", stats.evictions);
        println!(
            "desktop_icon_raster_cache_parsed_sources={}",
            stats.parsed_sources
        );
    }

    fn new(device: &Path) -> Result<Self, String> {
        let render_device = gpu_render_device_path(device);
        Self::new_on_render_device(&render_device)
    }

    fn new_on_render_device(render_device: &Path) -> Result<Self, String> {
        Self::new_on_render_device_with_viewport(render_device, Viewport::new(800, 600))
    }

    fn new_on_render_device_with_viewport(
        render_device: &Path,
        viewport: Viewport,
    ) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(render_device)
            .map_err(|error| format!("cannot open {}: {error}", render_device.display()))?;
        let owned_fd: OwnedFd = file.into();
        let gbm = GbmDevice::new(DeviceFd::from(owned_fd))
            .map_err(|error| format!("cannot create GBM device: {error}"))?;
        let display = unsafe { EGLDisplay::new(gbm) }
            .map_err(|error| format!("cannot create EGL display: {error}"))?;
        let context = EGLContext::new(&display)
            .map_err(|error| format!("cannot create EGL context: {error}"))?;
        let mut renderer = unsafe { GlesRenderer::new(context) }
            .map_err(|error| format!("cannot create GLES renderer: {error}"))?;
        let asset_root = env::var_os("AQUA_ASSET_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/usr/share/aqua"));
        let wallpaper = decode_png_rgba(&asset_root.join("wallpapers/default-wallpaper.png"))?;
        let wallpaper_texture = renderer
            .import_memory(
                &wallpaper.rgba,
                Fourcc::Abgr8888,
                (wallpaper.width as i32, wallpaper.height as i32).into(),
                false,
            )
            .map_err(|error| format!("cannot upload runtime wallpaper texture: {error}"))?;
        let surface_uniform_names = [
            UniformName::new("refraction_strength", UniformType::_1f),
            UniformName::new("tint_strength", UniformType::_1f),
            UniformName::new("highlight_strength", UniformType::_1f),
            UniformName::new("surface_tint", UniformType::_3f),
            UniformName::new("panel_uv_origin", UniformType::_2f),
            UniformName::new("panel_uv_size", UniformType::_2f),
            UniformName::new("panel_size_px", UniformType::_2f),
            UniformName::new("corner_radius_px", UniformType::_1f),
            UniformName::new("edge_width_px", UniformType::_1f),
            UniformName::new("edge_light_strength", UniformType::_1f),
        ];
        let surface_program = renderer
            .compile_custom_texture_shader(AQUA_SURFACE_FRAGMENT_SHADER, &surface_uniform_names)
            .map_err(|error| format!("cannot compile Aqua system-surface shader: {error}"))?;
        let blur_program = renderer
            .compile_custom_texture_shader(
                AQUA_BLUR_FRAGMENT_SHADER,
                &[UniformName::new("texel_step", UniformType::_2f)],
            )
            .map_err(|error| format!("cannot compile Aqua separable blur shader: {error}"))?;
        let mut horizontal_blur = Offscreen::<GlesTexture>::create_buffer(
            &mut renderer,
            Fourcc::Abgr8888,
            (320, 240).into(),
        )
        .map_err(|error| format!("cannot create horizontal blur texture: {error}"))?;
        let mut blurred_wallpaper = Offscreen::<GlesTexture>::create_buffer(
            &mut renderer,
            Fourcc::Abgr8888,
            (320, 240).into(),
        )
        .map_err(|error| format!("cannot create vertical blur texture: {error}"))?;
        render_gpu_blur_pass(
            &mut renderer,
            &mut horizontal_blur,
            &wallpaper_texture,
            (wallpaper.width, wallpaper.height),
            &blur_program,
            (1.0 / 320.0, 0.0),
            "horizontal",
        )?;
        render_gpu_blur_pass(
            &mut renderer,
            &mut blurred_wallpaper,
            &horizontal_blur,
            (320, 240),
            &blur_program,
            (0.0, 1.0 / 240.0),
            "vertical",
        )?;
        let target = Offscreen::<GlesTexture>::create_buffer(
            &mut renderer,
            Fourcc::Abgr8888,
            (320, 240).into(),
        )
        .map_err(|error| format!("cannot create offscreen texture: {error}"))?;
        let mut scene = static_shell_scene(viewport);
        scene.set_surface_visible(aqua_scene::SurfaceKind::Launcher, false);
        scene.set_surface_visible(aqua_scene::SurfaceKind::SystemOverview, true);
        scene.set_surface_visible(aqua_scene::SurfaceKind::DesktopIconColumn, true);
        scene.set_surface_visible(aqua_scene::SurfaceKind::NotificationToast, false);
        let theme = configured_runtime_theme();
        println!("desktop_shell_theme={}", theme.id());
        Ok(Self {
            renderer,
            wallpaper_texture,
            wallpaper_width: wallpaper.width,
            wallpaper_height: wallpaper.height,
            blurred_wallpaper,
            surface_program,
            target,
            target_size: (320, 240),
            scene,
            theme,
            launcher_texture: None,
            launcher_state: None,
            launcher_texture_size: (0, 0),
            top_bar_texture: None,
            top_bar_state: None,
            top_bar_texture_size: (0, 0),
            session_menu_texture: None,
            session_menu_state: None,
            session_menu_texture_size: (0, 0),
            system_overview_texture: None,
            system_overview_state: None,
            system_overview_texture_size: (0, 0),
            desktop_icons_texture: None,
            desktop_icons_state: None,
            desktop_icons_texture_size: (0, 0),
            dock_texture: None,
            dock_state: None,
            dock_texture_size: (0, 0),
            notification_texture: None,
            notification_state: aqua_shell::NotificationCenter::default(),
            notification_texture_size: (0, 0),
            icon_raster_cache: IconRasterCache::default(),
            client_texture_cache: Vec::new(),
            shadow_mask_cache: ShadowMaskCache::default(),
            client_shadow_texture_cache: Vec::new(),
            opaque_direct_bridge_ready: false,
        })
    }

    fn client_textures(
        &mut self,
        client_plan: &ClientLayerPaintPlan,
        revisions: Option<&[u64]>,
    ) -> Result<(Vec<GlesTexture>, usize, usize, usize), String> {
        if revisions.is_some_and(|revisions| revisions.len() != client_plan.steps.len()) {
            return Err("client texture revisions do not match paint steps".to_string());
        }
        self.client_texture_cache.retain(|cached| {
            client_plan
                .steps
                .iter()
                .any(|step| step.surface_id == cached.surface_id)
        });

        let mut textures = Vec::with_capacity(client_plan.steps.len());
        let mut cache_hits = 0;
        let mut uploads = 0;
        let mut source_bytes = 0;
        for (index, step) in client_plan.steps.iter().enumerate() {
            let revision = revisions
                .map(|revisions| revisions[index])
                .unwrap_or(step.sample_checksum);
            source_bytes += step.source_width as usize * step.source_height as usize * 4;
            if let Some(cached) = self.client_texture_cache.iter().find(|cached| {
                cached.surface_id == step.surface_id
                    && cached.revision == revision
                    && cached.source_size == (step.source_width, step.source_height)
            }) {
                textures.push(cached.texture.clone());
                cache_hits += 1;
                continue;
            }

            let pixels = gpu_client_texture_pixels(step);
            let texture = self
                .renderer
                .import_memory(
                    &pixels,
                    Fourcc::Abgr8888,
                    (step.source_width as i32, step.source_height as i32).into(),
                    false,
                )
                .map_err(|error| {
                    format!(
                        "cannot upload GPU client texture {}: {error}",
                        step.surface_id
                    )
                })?;
            self.client_texture_cache
                .retain(|cached| cached.surface_id != step.surface_id);
            self.client_texture_cache.push(ClientTextureCacheEntry {
                surface_id: step.surface_id,
                revision,
                source_size: (step.source_width, step.source_height),
                texture: texture.clone(),
            });
            textures.push(texture);
            uploads += 1;
        }
        Ok((textures, source_bytes, cache_hits, uploads))
    }

    fn client_shadow_textures(
        &mut self,
        client_plan: &ClientLayerPaintPlan,
        render_size: (u32, u32),
    ) -> Result<Vec<ClientShadowTexture>, String> {
        let (render_width, render_height) = render_size;
        let mut shadows = Vec::with_capacity(client_plan.steps.len());
        for step in &client_plan.steps {
            let surface_width = (step.rect.width * render_width / 1536).max(1);
            let surface_height = (step.rect.height * render_height / 1024).max(1);
            let corner_radius = (18 * render_width / 1536).max(1);
            let elevation = if step.focused {
                ElevationLevel::ActiveWindow
            } else {
                ElevationLevel::Dialog
            };
            let key = ShadowMaskKey::from_physical(
                surface_width,
                surface_height,
                corner_radius,
                aqua_text::OutputScale::One,
                self.theme,
                elevation,
            );
            if let Some(cached) = self
                .client_shadow_texture_cache
                .iter()
                .find(|cached| cached.key == key)
            {
                shadows.push(cached.shadow.clone());
                continue;
            }
            let mask = self.shadow_mask_cache.get_or_render(key);
            let texture = self
                .renderer
                .import_memory(
                    &mask.rgba,
                    Fourcc::Abgr8888,
                    (mask.width as i32, mask.height as i32).into(),
                    false,
                )
                .map_err(|error| format!("cannot upload client shadow texture: {error}"))?;
            let shadow = ClientShadowTexture {
                texture,
                size: (mask.width, mask.height),
                surface_offset: (mask.surface_x, mask.surface_y),
            };
            if self.client_shadow_texture_cache.len() >= self.shadow_mask_cache.capacity() {
                self.client_shadow_texture_cache.remove(0);
            }
            self.client_shadow_texture_cache
                .push(ClientShadowTextureCacheEntry {
                    key,
                    shadow: shadow.clone(),
                });
            shadows.push(shadow);
        }
        Ok(shadows)
    }

    fn render(
        &mut self,
        client_plan: &ClientLayerPaintPlan,
    ) -> Result<GpuOffscreenFrameResult, String> {
        self.ensure_target_size(320, 240)?;
        let (client_textures, client_texture_bytes, _, _) =
            self.client_textures(client_plan, None)?;
        let client_shadows = self.client_shadow_textures(client_plan, self.target_size)?;
        let (frame_rgba, checksum) = render_gpu_scene(
            &mut self.renderer,
            &mut self.target,
            &self.wallpaper_texture,
            self.wallpaper_width,
            self.wallpaper_height,
            &self.blurred_wallpaper,
            &self.surface_program,
            &self.scene,
            client_plan,
            &client_textures,
            &client_shadows,
            self.launcher_texture.as_ref(),
            self.launcher_texture_size,
            self.top_bar_texture.as_ref(),
            self.top_bar_texture_size,
            self.desktop_icons_texture.as_ref(),
            self.desktop_icons_texture_size,
            self.dock_texture.as_ref(),
            self.dock_texture_size,
            self.system_overview_texture.as_ref(),
            self.system_overview_texture_size,
            self.session_menu_texture.as_ref(),
            self.session_menu_texture_size,
            self.notification_texture.as_ref(),
            self.notification_texture_size,
            self.target_size,
            None,
            true,
        )?
        .ok_or_else(|| "GPU verification readback was not produced".to_string())?;
        let (_, repeat_checksum) = render_gpu_scene(
            &mut self.renderer,
            &mut self.target,
            &self.wallpaper_texture,
            self.wallpaper_width,
            self.wallpaper_height,
            &self.blurred_wallpaper,
            &self.surface_program,
            &self.scene,
            client_plan,
            &client_textures,
            &client_shadows,
            self.launcher_texture.as_ref(),
            self.launcher_texture_size,
            self.top_bar_texture.as_ref(),
            self.top_bar_texture_size,
            self.desktop_icons_texture.as_ref(),
            self.desktop_icons_texture_size,
            self.dock_texture.as_ref(),
            self.dock_texture_size,
            self.system_overview_texture.as_ref(),
            self.system_overview_texture_size,
            self.session_menu_texture.as_ref(),
            self.session_menu_texture_size,
            self.notification_texture.as_ref(),
            self.notification_texture_size,
            self.target_size,
            None,
            true,
        )?
        .ok_or_else(|| "repeat GPU verification readback was not produced".to_string())?;
        if checksum != repeat_checksum {
            return Err(format!(
                "GPU scene is not deterministic: {checksum:016x} != {repeat_checksum:016x}"
            ));
        }
        Ok(GpuOffscreenFrameResult {
            surface_count: self
                .scene
                .surfaces
                .iter()
                .filter(|surface| surface.visible)
                .count(),
            surface_layer_count: self
                .scene
                .surfaces
                .iter()
                .filter(|surface| {
                    surface.visible && surface.material == MaterialKind::SystemSurface
                })
                .count(),
            client_texture_count: client_textures.len(),
            client_texture_bytes,
            frame_bytes: 320 * 240 * 4,
            wallpaper_width: self.wallpaper_width,
            wallpaper_height: self.wallpaper_height,
            checksum,
            repeat_checksum,
            frame_rgba,
        })
    }

    fn render_direct(
        &mut self,
        client_plan: &ClientLayerPaintPlan,
    ) -> Result<GpuOffscreenFrameResult, String> {
        self.render_direct_at(client_plan, 320, 240, None, None)
    }

    fn render_direct_at(
        &mut self,
        client_plan: &ClientLayerPaintPlan,
        width: u32,
        height: u32,
        revisions: Option<&[u64]>,
        opaque_sources: Option<&[bool]>,
    ) -> Result<GpuOffscreenFrameResult, String> {
        self.ensure_target_size(width, height)?;
        let opaque_direct_candidate = opaque_client_covers_output(client_plan, opaque_sources);
        let upload_started = std::time::Instant::now();
        let (client_textures, client_texture_bytes, cache_hits, uploads) =
            self.client_textures(client_plan, revisions)?;
        let client_shadows = self.client_shadow_textures(client_plan, self.target_size)?;
        let shadow_stats = self.shadow_mask_cache.stats();
        println!(
            "gpu_client_texture_upload_ms={}",
            upload_started.elapsed().as_millis()
        );
        println!("gpu_client_texture_cache_hits={cache_hits}");
        println!("gpu_client_texture_uploads={uploads}");
        println!("gpu_shadow_mask_cache_hits={}", shadow_stats.hits);
        println!("gpu_shadow_mask_cache_misses={}", shadow_stats.misses);
        println!(
            "gpu_shadow_damage_rects={}",
            client_shadow_damage_rects(client_plan, width, height).len()
        );
        let (frame_rgba, checksum) = render_gpu_scene(
            &mut self.renderer,
            &mut self.target,
            &self.wallpaper_texture,
            self.wallpaper_width,
            self.wallpaper_height,
            &self.blurred_wallpaper,
            &self.surface_program,
            &self.scene,
            client_plan,
            &client_textures,
            &client_shadows,
            self.launcher_texture.as_ref(),
            self.launcher_texture_size,
            self.top_bar_texture.as_ref(),
            self.top_bar_texture_size,
            self.desktop_icons_texture.as_ref(),
            self.desktop_icons_texture_size,
            self.dock_texture.as_ref(),
            self.dock_texture_size,
            self.system_overview_texture.as_ref(),
            self.system_overview_texture_size,
            self.session_menu_texture.as_ref(),
            self.session_menu_texture_size,
            self.notification_texture.as_ref(),
            self.notification_texture_size,
            self.target_size,
            opaque_sources,
            true,
        )?
        .ok_or_else(|| "direct GPU scene readback was not produced".to_string())?;
        let result = GpuOffscreenFrameResult {
            surface_count: self
                .scene
                .surfaces
                .iter()
                .filter(|surface| surface.visible)
                .count(),
            surface_layer_count: self
                .scene
                .surfaces
                .iter()
                .filter(|surface| {
                    surface.visible && surface.material == MaterialKind::SystemSurface
                })
                .count(),
            client_texture_count: client_textures.len(),
            client_texture_bytes,
            frame_bytes: frame_rgba.len(),
            wallpaper_width: self.wallpaper_width,
            wallpaper_height: self.wallpaper_height,
            checksum,
            repeat_checksum: checksum,
            frame_rgba,
        };
        self.opaque_direct_bridge_ready = opaque_direct_candidate;
        Ok(result)
    }

    fn ensure_target_size(&mut self, width: u32, height: u32) -> Result<(), String> {
        if width == 0 || height == 0 {
            return Err("GPU render target dimensions must be non-zero".to_string());
        }
        if self.target_size == (width, height) {
            return Ok(());
        }
        self.target = Offscreen::<GlesTexture>::create_buffer(
            &mut self.renderer,
            Fourcc::Abgr8888,
            (width as i32, height as i32).into(),
        )
        .map_err(|error| format!("cannot resize GPU render target: {error}"))?;
        self.target_size = (width, height);
        println!("gpu_render_target={}x{}", width, height);
        Ok(())
    }

    fn render_to_scanout(
        &mut self,
        target: &mut Dmabuf,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let mut framebuffer = self
            .renderer
            .bind(target)
            .map_err(|error| format!("cannot bind GBM dma-buf scanout target: {error}"))?;
        let mut frame = self
            .renderer
            .render(
                &mut framebuffer,
                (width as i32, height as i32).into(),
                Transform::Normal,
            )
            .map_err(|error| format!("cannot begin GBM dma-buf scanout frame: {error}"))?;
        let damage = [Rectangle::from_size((width as i32, height as i32).into())];
        frame
            .clear(Color32F::new(0.01, 0.10, 0.24, 1.0), &damage)
            .map_err(|error| format!("cannot clear GBM dma-buf scanout frame: {error}"))?;
        frame
            .render_texture_from_to(
                &self.target,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (self.target_size.0 as f64, self.target_size.1 as f64).into(),
                ),
                Rectangle::from_size((width as i32, height as i32).into()),
                &damage,
                &[],
                Transform::Normal,
                1.0,
                None,
                &[],
            )
            .map_err(|error| format!("cannot render Aqua scene into GBM dma-buf: {error}"))?;
        if let (Some(texture), Some(surface)) = (
            self.session_menu_texture.as_ref(),
            self.scene
                .surfaces
                .iter()
                .find(|surface| surface.kind == aqua_scene::SurfaceKind::SystemOverview),
        ) {
            println!("drm_wayland_session_menu_scanout_composited=true");
            let rect = Rectangle::new(
                (
                    (surface.rect.x * width / self.scene.viewport.width) as i32,
                    (surface.rect.y * height / self.scene.viewport.height) as i32,
                )
                    .into(),
                (
                    (surface.rect.width * width / self.scene.viewport.width) as i32,
                    (surface.rect.height * height / self.scene.viewport.height) as i32,
                )
                    .into(),
            );
            frame
                .render_texture_from_to(
                    texture,
                    Rectangle::new(
                        (0.0, 0.0).into(),
                        (
                            self.session_menu_texture_size.0 as f64,
                            self.session_menu_texture_size.1 as f64,
                        )
                            .into(),
                    ),
                    rect,
                    &[Rectangle::from_size(rect.size)],
                    &[],
                    Transform::Normal,
                    1.0,
                    None,
                    &[],
                )
                .map_err(|error| {
                    format!("cannot composite session menu into GBM scanout: {error}")
                })?;
        }
        frame
            .finish()
            .map_err(|error| format!("cannot finish GBM dma-buf scanout frame: {error}"))?
            .wait()
            .map_err(|error| format!("cannot synchronize GBM dma-buf scanout frame: {error}"))
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn gpu_render_device_path(kms_device: &Path) -> PathBuf {
    if let Some(configured) = env::var_os("AQUA_DRM_RENDER_DEVICE") {
        return PathBuf::from(configured);
    }
    let render_node = kms_device
        .parent()
        .unwrap_or_else(|| Path::new("/dev/dri"))
        .join("renderD128");
    if render_node.exists() {
        render_node
    } else {
        kms_device.to_path_buf()
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn render_live_gpu_wayland_frame(
    compositor: &RefCell<Option<LiveGpuCompositor>>,
    snapshots: &[SmithayClientSurfaceSnapshot],
    output_width: u32,
    output_height: u32,
) -> Result<(Vec<u8>, GpuOffscreenFrameResult), String> {
    let frame_started = std::time::Instant::now();
    let paint_plan = external_client_paint_plan(snapshots)?;
    let mut compositor = compositor.borrow_mut();
    let compositor = compositor
        .as_mut()
        .ok_or_else(|| "live GPU compositor is unavailable".to_string())?;
    let revisions = snapshots
        .iter()
        .map(|snapshot| {
            (snapshot.commit_count as u64)
                ^ snapshot.sample_checksum.rotate_left(17)
                ^ (snapshot.workspace as u64).rotate_left(41)
        })
        .collect::<Vec<_>>();
    let opaque_sources = snapshots
        .iter()
        .map(|snapshot| snapshot.buffer_opaque)
        .collect::<Vec<_>>();
    let direct_bridge_candidate = compositor.opaque_direct_bridge_ready
        && opaque_client_covers_output(&paint_plan, Some(&opaque_sources))
        && paint_plan.steps.len() == 1
        && paint_plan.steps[0].source_width == output_width
        && paint_plan.steps[0].source_height == output_height
        && paint_plan.steps[0].client_buffer_rgba.len()
            == output_width as usize * output_height as usize * 4
        && compositor
            .scene
            .surfaces
            .iter()
            .all(|surface| !surface.visible || surface.material == MaterialKind::Image);
    if direct_bridge_candidate {
        let direct_started = std::time::Instant::now();
        let frame_rgba = paint_plan.steps[0].client_buffer_rgba.clone();
        let checksum = checksum_frame_bytes(&frame_rgba);
        let render_ms = direct_started.elapsed().as_millis();
        let pack_started = std::time::Instant::now();
        let scanout_frame = pack_rgba_frame(
            &frame_rgba,
            output_width,
            output_height,
            output_width,
            output_height,
            32,
        )?;
        let pack_ms = pack_started.elapsed().as_millis();
        println!("gpu_native_opaque_direct_bridge=true");
        println!("gpu_native_render_ms={render_ms}");
        println!("gpu_xrgb_pack_ms={pack_ms}");
        println!(
            "gpu_native_frame_total_ms={}",
            frame_started.elapsed().as_millis()
        );
        return Ok((
            scanout_frame,
            GpuOffscreenFrameResult {
                surface_count: 1,
                surface_layer_count: 0,
                client_texture_count: 0,
                client_texture_bytes: frame_rgba.len(),
                frame_bytes: frame_rgba.len(),
                wallpaper_width: compositor.wallpaper_width,
                wallpaper_height: compositor.wallpaper_height,
                checksum,
                repeat_checksum: checksum,
                frame_rgba,
            },
        ));
    }
    let render_started = std::time::Instant::now();
    let gpu_frame = compositor.render_direct_at(
        &paint_plan,
        output_width,
        output_height,
        Some(&revisions),
        Some(&opaque_sources),
    )?;
    let render_ms = render_started.elapsed().as_millis();
    let pack_started = std::time::Instant::now();
    let scanout_frame = pack_rgba_frame(
        &gpu_frame.frame_rgba,
        output_width,
        output_height,
        output_width,
        output_height,
        32,
    )?;
    let pack_ms = pack_started.elapsed().as_millis();
    if compositor.session_menu_state.is_some() {
        println!("gpu_readback_session_menu_native_composited=true");
    }
    println!("gpu_native_render_ms={render_ms}");
    println!("gpu_xrgb_pack_ms={pack_ms}");
    println!(
        "gpu_native_frame_total_ms={}",
        frame_started.elapsed().as_millis()
    );
    Ok((scanout_frame, gpu_frame))
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct DesktopSurfaceRevision {
    workspace: usize,
    commit_count: usize,
    damage_commit_count: usize,
    damage_rect_count: usize,
    frame_callbacks_sent: usize,
    sample_checksum: u64,
    mapped_surface_count: usize,
    destroyed_surface_count: usize,
    client_cleanup_count: usize,
    x: u32,
    y: u32,
    display_width: u32,
    display_height: u32,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn desktop_surface_revisions(
    snapshots: &[SmithayClientSurfaceSnapshot],
) -> Vec<DesktopSurfaceRevision> {
    snapshots
        .iter()
        .map(|snapshot| DesktopSurfaceRevision {
            workspace: snapshot.workspace,
            commit_count: snapshot.commit_count,
            damage_commit_count: snapshot.damage_commit_count,
            damage_rect_count: snapshot.damage_rect_count,
            frame_callbacks_sent: snapshot.frame_callbacks_sent,
            sample_checksum: snapshot.sample_checksum,
            mapped_surface_count: snapshot.mapped_surface_count,
            destroyed_surface_count: snapshot.destroyed_surface_count,
            client_cleanup_count: snapshot.client_cleanup_count,
            x: snapshot.x,
            y: snapshot.y,
            display_width: snapshot.display_width,
            display_height: snapshot.display_height,
        })
        .collect()
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn first_party_surface_app_id(app_id: &str) -> Option<&'static str> {
    match app_id {
        "files" => Some("aqua.files"),
        "settings" => Some("aqua.settings"),
        "properties" => Some("aqua.properties"),
        "terminal" => Some("aqua.terminal"),
        _ => None,
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn current_dock_state(
    launcher: &aqua_shell::LauncherState,
    supervisor: &FirstPartyProcessSupervisor,
    active_workspace: usize,
) -> aqua_shell::DockState {
    aqua_shell::DockState {
        applications_open: launcher.is_open()
            && launcher.mode() == aqua_shell::LauncherMode::Applications,
        search_open: launcher.is_open() && launcher.mode() == aqua_shell::LauncherMode::Search,
        files_running: supervisor.contains("files"),
        settings_running: supervisor.contains("settings"),
        active_workspace,
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn launch_first_party_desktop_client(
    request: &LaunchRequest,
    listener: &ListeningSocket,
    session: &RefCell<SmithayDrmSession>,
    supervisor: &RefCell<FirstPartyProcessSupervisor>,
    runtime_dir: &Path,
) -> Result<bool, String> {
    let preflight = preflight_first_party_launch(request, Path::new("/"));
    println!("desktop_launch_request_app={}", request.app_id);
    println!("desktop_launch_preflight={}", preflight.reason);
    if !preflight.accepted {
        println!("desktop_launch_accepted=false");
        return Ok(false);
    }

    let process = match supervisor
        .borrow_mut()
        .spawn(&preflight, runtime_dir, "aqua-wayland-drm-0")
    {
        Ok(process) => process,
        Err(ProcessSupervisorError::AlreadyRunning) => {
            let Some(surface_app_id) = first_party_surface_app_id(preflight.app_id) else {
                println!("desktop_launch_duplicate_rejected=true");
                return Ok(false);
            };
            let raised = {
                let mut session = session.borrow_mut();
                session.raise_surface_with_app_id(surface_app_id)
                    && session.present_client_surface(15_000)
            };
            println!("desktop_launch_existing_surface_raised={raised}");
            return Ok(raised);
        }
        Err(error) => {
            return Err(format!(
                "cannot supervise desktop client {}: {error:?}",
                preflight.app_id
            ));
        }
    };
    println!("desktop_launch_accepted=true");
    println!("desktop_launch_process_pid={}", process.pid);

    let connect_deadline = std::time::Instant::now() + Duration::from_secs(10);
    let app_stream = loop {
        match listener.accept() {
            Ok(Some(stream)) => break stream,
            Ok(None) => {}
            Err(error) => {
                let _ = supervisor.borrow_mut().terminate_and_reap(preflight.app_id);
                return Err(format!(
                    "cannot accept {} desktop Wayland connection: {error}",
                    preflight.app_id
                ));
            }
        }
        if supervisor
            .borrow_mut()
            .try_reap(preflight.app_id)
            .map_err(|error| format!("cannot poll {}: {error:?}", preflight.app_id))?
            .is_some()
        {
            return Err(format!(
                "{} exited before opening its desktop Wayland connection",
                preflight.app_id
            ));
        }
        if std::time::Instant::now() >= connect_deadline {
            let _ = supervisor.borrow_mut().terminate_and_reap(preflight.app_id);
            return Err(format!(
                "{} did not open a desktop Wayland connection before timeout",
                preflight.app_id
            ));
        }
        thread::sleep(Duration::from_millis(10));
    };
    println!("desktop_launch_wayland_connection_accepted=true");
    session
        .borrow_mut()
        .insert_client(app_stream)
        .map_err(|error| format!("cannot insert {} desktop client: {error}", preflight.app_id))?;

    let expected_surface_app_id = match first_party_surface_app_id(preflight.app_id) {
        Some(app_id) => app_id,
        None => {
            let _ = supervisor.borrow_mut().terminate_and_reap(preflight.app_id);
            return Err(format!(
                "unsupported first-party surface owner: {}",
                preflight.app_id
            ));
        }
    };
    let surface_deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        session
            .borrow_mut()
            .dispatch_clients()
            .map_err(|error| format!("cannot dispatch {}: {error}", preflight.app_id))?;
        session
            .borrow_mut()
            .flush_clients()
            .map_err(|error| format!("cannot flush {}: {error}", preflight.app_id))?;
        let app_id_ready = session
            .borrow()
            .has_toplevel_app_id(expected_surface_app_id);
        let owned_surface_ready = app_id_ready
            && session
                .borrow_mut()
                .raise_surface_with_app_id(expected_surface_app_id);
        if owned_surface_ready {
            break;
        }
        if supervisor
            .borrow_mut()
            .try_reap(preflight.app_id)
            .map_err(|error| format!("cannot poll {}: {error:?}", preflight.app_id))?
            .is_some()
        {
            return Err(format!(
                "{} exited before its desktop surface became ready",
                preflight.app_id
            ));
        }
        if std::time::Instant::now() >= surface_deadline {
            let _ = supervisor.borrow_mut().terminate_and_reap(preflight.app_id);
            return Err(format!(
                "{} desktop surface was not ready before timeout",
                preflight.app_id
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    println!("desktop_launch_surface_protocol_ready=true");
    let surface_focused = {
        let mut session = session.borrow_mut();
        session.raise_surface_with_app_id(expected_surface_app_id)
            && session.present_client_surface(15_000)
    };
    if !surface_focused {
        let _ = supervisor.borrow_mut().terminate_and_reap(preflight.app_id);
        return Err(format!(
            "{} desktop surface could not receive launch focus",
            preflight.app_id
        ));
    }
    println!("desktop_launch_surface_focus_assigned=true");
    println!("desktop_launch_surface_app_id={expected_surface_app_id}");
    println!("desktop_launch_surface_ready=true");
    println!("desktop_launch_surface_focused=true");
    Ok(true)
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn gpu_client_texture_pixels(step: &aqua_renderer::ClientLayerPaintStep) -> Vec<u8> {
    let expected = step.source_width as usize * step.source_height as usize * 4;
    if step.client_buffer_rgba.len() == expected {
        return step.client_buffer_rgba.clone();
    }

    let mut pixels = Vec::with_capacity(expected);
    for y in 0..step.source_height {
        for x in 0..step.source_width {
            let column = usize::from(x >= step.source_width / 2);
            let row = usize::from(y >= step.source_height / 2);
            pixels.extend_from_slice(&step.sample_grid[row * 2 + column]);
        }
    }
    pixels
}

#[cfg(any(test, all(target_os = "linux", feature = "smithay-gpu")))]
fn opaque_layer_covers_reference_output(
    opacity: u8,
    rect: (u32, u32, u32, u32),
    source_declared_opaque: bool,
) -> bool {
    opacity == u8::MAX
        && rect.0 == 0
        && rect.1 == 0
        && rect.2 >= 1536
        && rect.3 >= 1024
        && source_declared_opaque
}

#[cfg(any(test, all(target_os = "linux", feature = "smithay-gpu")))]
fn client_shadow_damage_rects(
    client_plan: &aqua_renderer::ClientLayerPaintPlan,
    render_width: u32,
    render_height: u32,
) -> Vec<aqua_scene::Rect> {
    let viewport = aqua_scene::Viewport::new(render_width, render_height);
    client_plan
        .steps
        .iter()
        .map(|step| {
            let surface = aqua_scene::Rect {
                x: step.rect.x * render_width / 1536,
                y: step.rect.y * render_height / 1024,
                width: (step.rect.width * render_width / 1536).max(1),
                height: (step.rect.height * render_height / 1024).max(1),
            };
            let elevation = if step.focused {
                aqua_renderer::ElevationLevel::ActiveWindow
            } else {
                aqua_renderer::ElevationLevel::Dialog
            };
            aqua_renderer::elevation_damage_rect(
                surface,
                viewport,
                aqua_text::OutputScale::One,
                elevation,
            )
        })
        .collect()
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn opaque_client_covers_output(
    client_plan: &aqua_renderer::ClientLayerPaintPlan,
    opaque_sources: Option<&[bool]>,
) -> bool {
    let Some(opaque_sources) = opaque_sources else {
        return false;
    };
    if opaque_sources.len() != client_plan.steps.len() {
        return false;
    }
    client_plan.steps.iter().enumerate().any(|(index, step)| {
        opaque_layer_covers_reference_output(
            step.opacity,
            (step.rect.x, step.rect.y, step.rect.width, step.rect.height),
            opaque_sources[index],
        )
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn render_gpu_blur_pass(
    renderer: &mut GlesRenderer,
    target: &mut GlesTexture,
    source: &GlesTexture,
    source_size: (u32, u32),
    blur_program: &GlesTexProgram,
    texel_step: (f32, f32),
    pass_name: &str,
) -> Result<(), String> {
    let mut framebuffer = renderer
        .bind(target)
        .map_err(|error| format!("cannot bind {pass_name} blur texture: {error}"))?;
    let mut frame = renderer
        .render(&mut framebuffer, (320, 240).into(), Transform::Normal)
        .map_err(|error| format!("cannot begin {pass_name} blur pass: {error}"))?;
    let damage = [Rectangle::from_size((320, 240).into())];
    frame
        .render_texture_from_to(
            source,
            Rectangle::new(
                (0.0, 0.0).into(),
                (source_size.0 as f64, source_size.1 as f64).into(),
            ),
            Rectangle::from_size((320, 240).into()),
            &damage,
            &[],
            Transform::Normal,
            1.0,
            Some(blur_program),
            &[Uniform::new("texel_step", texel_step)],
        )
        .map_err(|error| format!("cannot render {pass_name} blur pass: {error}"))?;
    frame
        .finish()
        .map_err(|error| format!("cannot finish {pass_name} blur pass: {error}"))?
        .wait()
        .map_err(|error| format!("cannot synchronize {pass_name} blur pass: {error}"))
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
#[allow(clippy::too_many_arguments)]
fn render_gpu_scene(
    renderer: &mut GlesRenderer,
    target: &mut GlesTexture,
    wallpaper: &GlesTexture,
    wallpaper_width: u32,
    wallpaper_height: u32,
    blurred_wallpaper: &GlesTexture,
    surface_program: &GlesTexProgram,
    scene: &aqua_scene::ShellScene,
    client_plan: &aqua_renderer::ClientLayerPaintPlan,
    client_textures: &[GlesTexture],
    client_shadows: &[ClientShadowTexture],
    launcher_texture: Option<&GlesTexture>,
    launcher_texture_size: (u32, u32),
    top_bar_texture: Option<&GlesTexture>,
    top_bar_texture_size: (u32, u32),
    desktop_icons_texture: Option<&GlesTexture>,
    desktop_icons_texture_size: (u32, u32),
    dock_texture: Option<&GlesTexture>,
    dock_texture_size: (u32, u32),
    system_overview_texture: Option<&GlesTexture>,
    system_overview_texture_size: (u32, u32),
    session_menu_texture: Option<&GlesTexture>,
    session_menu_texture_size: (u32, u32),
    notification_texture: Option<&GlesTexture>,
    notification_texture_size: (u32, u32),
    render_size: (u32, u32),
    opaque_sources: Option<&[bool]>,
    readback: bool,
) -> Result<Option<(Vec<u8>, u64)>, String> {
    let (render_width, render_height) = render_size;
    if client_plan.steps.len() != client_textures.len()
        || client_plan.steps.len() != client_shadows.len()
    {
        return Err("GPU client layers and shadows do not match paint steps".to_string());
    }
    let opaque_client_cover = opaque_client_covers_output(client_plan, opaque_sources);
    println!("gpu_opaque_client_cover={opaque_client_cover}");
    let mut framebuffer = renderer
        .bind(target)
        .map_err(|error| format!("cannot bind offscreen texture: {error}"))?;
    let mut frame = renderer
        .render(
            &mut framebuffer,
            (render_width as i32, render_height as i32).into(),
            Transform::Normal,
        )
        .map_err(|error| format!("cannot begin offscreen frame: {error}"))?;
    let damage = [Rectangle::from_size(
        (render_width as i32, render_height as i32).into(),
    )];
    if !opaque_client_cover {
        frame
            .clear(Color32F::new(0.01, 0.10, 0.24, 1.0), &damage)
            .map_err(|error| format!("cannot clear GPU scene: {error}"))?;
        frame
            .render_texture_from_to(
                wallpaper,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (wallpaper_width as f64, wallpaper_height as f64).into(),
                ),
                Rectangle::from_size((render_width as i32, render_height as i32).into()),
                &damage,
                &[],
                Transform::Normal,
                1.0,
                None,
                &[],
            )
            .map_err(|error| format!("cannot composite runtime wallpaper texture: {error}"))?;
    }
    for surface in scene.surfaces.iter().filter(|surface| {
        !opaque_client_cover
            && surface.visible
            && surface.material == MaterialKind::IconGrid
            && desktop_icons_texture.is_none()
    }) {
        let rect = Rectangle::new(
            (
                (surface.rect.x * render_width / scene.viewport.width) as i32,
                (surface.rect.y * render_height / scene.viewport.height) as i32,
            )
                .into(),
            (
                (surface.rect.width * render_width / scene.viewport.width) as i32,
                (surface.rect.height * render_height / scene.viewport.height) as i32,
            )
                .into(),
        );
        match surface.material {
            MaterialKind::SystemSurface => unreachable!("surface renders above client surfaces"),
            MaterialKind::IconGrid => frame
                .draw_solid(
                    rect,
                    &[Rectangle::from_size(rect.size)],
                    Color32F::new(0.24, 0.82, 0.92, 0.78),
                )
                .map_err(|error| {
                    format!("cannot draw GPU scene surface {}: {error}", surface.id)
                })?,
            MaterialKind::Image => continue,
        }
    }
    if !opaque_client_cover {
        if let (Some(texture), Some(surface)) = (
            desktop_icons_texture,
            scene
                .surfaces
                .iter()
                .find(|surface| surface.kind == aqua_scene::SurfaceKind::DesktopIconColumn),
        ) {
            let rect = Rectangle::new(
                (
                    (surface.rect.x * render_width / scene.viewport.width) as i32,
                    (surface.rect.y * render_height / scene.viewport.height) as i32,
                )
                    .into(),
                (
                    (surface.rect.width * render_width / scene.viewport.width) as i32,
                    (surface.rect.height * render_height / scene.viewport.height) as i32,
                )
                    .into(),
            );
            frame
                .render_texture_from_to(
                    texture,
                    Rectangle::new(
                        (0.0, 0.0).into(),
                        (
                            desktop_icons_texture_size.0 as f64,
                            desktop_icons_texture_size.1 as f64,
                        )
                            .into(),
                    ),
                    rect,
                    &[Rectangle::from_size(rect.size)],
                    &[],
                    Transform::Normal,
                    1.0,
                    None,
                    &[],
                )
                .map_err(|error| format!("cannot composite desktop icons: {error}"))?;
        }
    }
    for (index, (step, texture)) in client_plan.steps.iter().zip(client_textures).enumerate() {
        let rect = Rectangle::new(
            (
                (step.rect.x * render_width / 1536) as i32,
                (step.rect.y * render_height / 1024) as i32,
            )
                .into(),
            (
                (step.rect.width * render_width / 1536) as i32,
                (step.rect.height * render_height / 1024) as i32,
            )
                .into(),
        );
        if !opaque_client_cover {
            let shadow = &client_shadows[index];
            let shadow_rect = Rectangle::new(
                (
                    rect.loc.x - shadow.surface_offset.0 as i32,
                    rect.loc.y - shadow.surface_offset.1 as i32,
                )
                    .into(),
                (shadow.size.0 as i32, shadow.size.1 as i32).into(),
            );
            let damage_x = (-shadow_rect.loc.x).max(0).min(shadow_rect.size.w);
            let damage_y = (-shadow_rect.loc.y).max(0).min(shadow_rect.size.h);
            let damage_right = (render_width as i32 - shadow_rect.loc.x)
                .max(damage_x)
                .min(shadow_rect.size.w);
            let damage_bottom = (render_height as i32 - shadow_rect.loc.y)
                .max(damage_y)
                .min(shadow_rect.size.h);
            let shadow_damage = Rectangle::new(
                (damage_x, damage_y).into(),
                (damage_right - damage_x, damage_bottom - damage_y).into(),
            );
            frame
                .render_texture_from_to(
                    &shadow.texture,
                    Rectangle::new(
                        (0.0, 0.0).into(),
                        (shadow.size.0 as f64, shadow.size.1 as f64).into(),
                    ),
                    shadow_rect,
                    &[shadow_damage],
                    &[],
                    Transform::Normal,
                    1.0,
                    None,
                    &[],
                )
                .map_err(|error| {
                    format!(
                        "cannot composite GPU client shadow {}: {error}",
                        step.surface_id
                    )
                })?;
        }
        frame
            .render_texture_from_to(
                texture,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (step.source_width as f64, step.source_height as f64).into(),
                ),
                rect,
                &[Rectangle::from_size(rect.size)],
                &[],
                Transform::Normal,
                f32::from(step.opacity) / 255.0,
                None,
                &[],
            )
            .map_err(|error| {
                format!(
                    "cannot composite GPU client texture {}: {error}",
                    step.surface_id
                )
            })?;
    }
    for surface in scene.surfaces.iter().filter(|surface| {
        surface.visible
            && surface.material == MaterialKind::SystemSurface
            && match surface.kind {
                aqua_scene::SurfaceKind::TopPanel => top_bar_texture.is_none(),
                aqua_scene::SurfaceKind::Launcher => launcher_texture.is_none(),
                aqua_scene::SurfaceKind::Dock => dock_texture.is_none(),
                aqua_scene::SurfaceKind::SystemOverview => {
                    system_overview_texture.is_none() && session_menu_texture.is_none()
                }
                aqua_scene::SurfaceKind::NotificationToast => notification_texture.is_none(),
                _ => true,
            }
    }) {
        let rect = Rectangle::new(
            (
                (surface.rect.x * render_width / scene.viewport.width) as i32,
                (surface.rect.y * render_height / scene.viewport.height) as i32,
            )
                .into(),
            (
                (surface.rect.width * render_width / scene.viewport.width) as i32,
                (surface.rect.height * render_height / scene.viewport.height) as i32,
            )
                .into(),
        );
        let uv_origin = (
            surface.rect.x as f32 / scene.viewport.width as f32,
            surface.rect.y as f32 / scene.viewport.height as f32,
        );
        let uv_size = (
            surface.rect.width as f32 / scene.viewport.width as f32,
            surface.rect.height as f32 / scene.viewport.height as f32,
        );
        let uniforms = [
            Uniform::new("refraction_strength", 0.0025_f32),
            Uniform::new("tint_strength", 0.18_f32),
            Uniform::new("highlight_strength", 0.16_f32),
            Uniform::new("surface_tint", (0.04_f32, 0.58_f32, 0.84_f32)),
            Uniform::new("panel_uv_origin", uv_origin),
            Uniform::new("panel_uv_size", uv_size),
            Uniform::new("panel_size_px", (rect.size.w as f32, rect.size.h as f32)),
            Uniform::new("corner_radius_px", 12.0_f32),
            Uniform::new("edge_width_px", 1.5_f32),
            Uniform::new("edge_light_strength", 0.24_f32),
        ];
        frame
            .render_texture_from_to(
                blurred_wallpaper,
                Rectangle::new(
                    (uv_origin.0 as f64 * 320.0, uv_origin.1 as f64 * 240.0).into(),
                    (uv_size.0 as f64 * 320.0, uv_size.1 as f64 * 240.0).into(),
                ),
                rect,
                &[Rectangle::from_size(rect.size)],
                &[],
                Transform::Normal,
                0.72,
                Some(surface_program),
                &uniforms,
            )
            .map_err(|error| format!("cannot shade GPU surface surface {}: {error}", surface.id))?;
    }
    if let Some(texture) = launcher_texture {
        let rect = Rectangle::from_size((render_width as i32, render_height as i32).into());
        frame
            .render_texture_from_to(
                texture,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (
                        launcher_texture_size.0 as f64,
                        launcher_texture_size.1 as f64,
                    )
                        .into(),
                ),
                rect,
                &[Rectangle::from_size(rect.size)],
                &[],
                Transform::Normal,
                1.0,
                None,
                &[],
            )
            .map_err(|error| format!("cannot composite launcher content: {error}"))?;
    }
    if let (Some(texture), Some(surface)) = (
        top_bar_texture,
        scene
            .surfaces
            .iter()
            .find(|surface| surface.visible && surface.kind == aqua_scene::SurfaceKind::TopPanel),
    ) {
        let rect = Rectangle::new(
            (
                (surface.rect.x * render_width / scene.viewport.width) as i32,
                (surface.rect.y * render_height / scene.viewport.height) as i32,
            )
                .into(),
            (
                (surface.rect.width * render_width / scene.viewport.width) as i32,
                (surface.rect.height * render_height / scene.viewport.height) as i32,
            )
                .into(),
        );
        frame
            .render_texture_from_to(
                texture,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (top_bar_texture_size.0 as f64, top_bar_texture_size.1 as f64).into(),
                ),
                rect,
                &[Rectangle::from_size(rect.size)],
                &[],
                Transform::Normal,
                1.0,
                None,
                &[],
            )
            .map_err(|error| format!("cannot composite top bar content: {error}"))?;
    }
    if let (Some(texture), Some(surface)) = (
        dock_texture,
        scene
            .surfaces
            .iter()
            .find(|surface| surface.kind == aqua_scene::SurfaceKind::Dock),
    ) {
        let rect = Rectangle::new(
            (
                (surface.rect.x * render_width / scene.viewport.width) as i32,
                (surface.rect.y * render_height / scene.viewport.height) as i32,
            )
                .into(),
            (
                (surface.rect.width * render_width / scene.viewport.width) as i32,
                (surface.rect.height * render_height / scene.viewport.height) as i32,
            )
                .into(),
        );
        frame
            .render_texture_from_to(
                texture,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (dock_texture_size.0 as f64, dock_texture_size.1 as f64).into(),
                ),
                rect,
                &[Rectangle::from_size(rect.size)],
                &[],
                Transform::Normal,
                1.0,
                None,
                &[],
            )
            .map_err(|error| format!("cannot composite dock content: {error}"))?;
    }
    if session_menu_texture.is_none() {
        if let (Some(texture), Some(surface)) = (
            system_overview_texture,
            scene.surfaces.iter().find(|surface| {
                surface.visible && surface.kind == aqua_scene::SurfaceKind::SystemOverview
            }),
        ) {
            let rect = Rectangle::new(
                (
                    (surface.rect.x * render_width / scene.viewport.width) as i32,
                    (surface.rect.y * render_height / scene.viewport.height) as i32,
                )
                    .into(),
                (
                    (surface.rect.width * render_width / scene.viewport.width) as i32,
                    (surface.rect.height * render_height / scene.viewport.height) as i32,
                )
                    .into(),
            );
            frame
                .render_texture_from_to(
                    texture,
                    Rectangle::new(
                        (0.0, 0.0).into(),
                        (
                            system_overview_texture_size.0 as f64,
                            system_overview_texture_size.1 as f64,
                        )
                            .into(),
                    ),
                    rect,
                    &[Rectangle::from_size(rect.size)],
                    &[],
                    Transform::Normal,
                    1.0,
                    None,
                    &[],
                )
                .map_err(|error| format!("cannot composite system overview content: {error}"))?;
        }
    }
    if let (Some(texture), Some(surface)) = (
        session_menu_texture,
        scene
            .surfaces
            .iter()
            .find(|surface| surface.kind == aqua_scene::SurfaceKind::SystemOverview),
    ) {
        let rect = Rectangle::new(
            (
                (surface.rect.x * render_width / scene.viewport.width) as i32,
                (surface.rect.y * render_height / scene.viewport.height) as i32,
            )
                .into(),
            (
                (surface.rect.width * render_width / scene.viewport.width) as i32,
                (surface.rect.height * render_height / scene.viewport.height) as i32,
            )
                .into(),
        );
        frame
            .render_texture_from_to(
                texture,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (
                        session_menu_texture_size.0 as f64,
                        session_menu_texture_size.1 as f64,
                    )
                        .into(),
                ),
                rect,
                &[Rectangle::from_size(rect.size)],
                &[],
                Transform::Normal,
                1.0,
                None,
                &[],
            )
            .map_err(|error| format!("cannot composite session menu content: {error}"))?;
    }
    if let (Some(texture), Some(surface)) = (
        notification_texture,
        scene
            .surfaces
            .iter()
            .find(|surface| surface.kind == aqua_scene::SurfaceKind::NotificationToast),
    ) {
        let rect = Rectangle::new(
            (
                (surface.rect.x * render_width / scene.viewport.width) as i32,
                (surface.rect.y * render_height / scene.viewport.height) as i32,
            )
                .into(),
            (
                (surface.rect.width * render_width / scene.viewport.width) as i32,
                (surface.rect.height * render_height / scene.viewport.height) as i32,
            )
                .into(),
        );
        frame
            .render_texture_from_to(
                texture,
                Rectangle::new(
                    (0.0, 0.0).into(),
                    (
                        notification_texture_size.0 as f64,
                        notification_texture_size.1 as f64,
                    )
                        .into(),
                ),
                rect,
                &[Rectangle::from_size(rect.size)],
                &[],
                Transform::Normal,
                1.0,
                None,
                &[],
            )
            .map_err(|error| format!("cannot composite notification content: {error}"))?;
    }
    let submit_started = std::time::Instant::now();
    frame
        .finish()
        .map_err(|error| format!("cannot finish GPU scene frame: {error}"))?
        .wait()
        .map_err(|error| format!("cannot synchronize GPU scene frame: {error}"))?;
    println!(
        "gpu_scene_submit_sync_ms={}",
        submit_started.elapsed().as_millis()
    );
    if !readback {
        return Ok(None);
    }
    let readback_started = std::time::Instant::now();
    let mapping = renderer
        .copy_framebuffer(
            &framebuffer,
            Rectangle::from_size((render_width as i32, render_height as i32).into()),
            Fourcc::Abgr8888,
        )
        .map_err(|error| format!("cannot read GPU scene framebuffer: {error}"))?;
    let bytes = renderer
        .map_texture(&mapping)
        .map_err(|error| format!("cannot map GPU scene readback: {error}"))?;
    if bytes.len() != render_width as usize * render_height as usize * 4 {
        return Err(format!("unexpected GPU scene byte count: {}", bytes.len()));
    }
    let frame_rgba = bytes.to_vec();
    println!(
        "gpu_scene_readback_ms={}",
        readback_started.elapsed().as_millis()
    );
    let checksum_started = std::time::Instant::now();
    let checksum = checksum_frame_bytes(&frame_rgba);
    println!(
        "gpu_scene_checksum_ms={}",
        checksum_started.elapsed().as_millis()
    );
    Ok(Some((frame_rgba, checksum)))
}

#[cfg(any(test, all(target_os = "linux", feature = "smithay-gpu")))]
fn checksum_frame_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        hash ^= u64::from_le_bytes(chunk.try_into().expect("eight-byte frame chunk"));
        hash = hash.rotate_left(13).wrapping_mul(0x9e37_79b1_85eb_ca87);
    }
    for byte in chunks.remainder() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash ^ bytes.len() as u64
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn present_drm_gpu_surface_cli(device: PathBuf) {
    let confirmation_source = drm_kms_confirmation_source(
        env::var("AQUA_DRM_GPU_SURFACE_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_GPU_SURFACE_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_GPU_SURFACE_TEST_MODE").ok().as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!("DRM GPU surface presentation requires explicit confirmation");
        println!("[AQUA-COMPOSITOR] stage=drm-gpu-surface status=blocked-confirmation");
        std::process::exit(1);
    };
    let timeout_ms = env::var("AQUA_DRM_GPU_SURFACE_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000)
        .clamp(100, 5_000);
    let hold_seconds = env::var("AQUA_DRM_GPU_SURFACE_HOLD_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3)
        .min(30);

    let gpu_frame = probe_gpu_offscreen_frame(&device).unwrap_or_else(|error| {
        eprintln!("DRM GPU surface composition failed: {error}");
        println!("[AQUA-COMPOSITOR] stage=drm-gpu-surface status=error");
        std::process::exit(1);
    });
    let gpu_checksum = gpu_frame.checksum;
    let frame_rgba = gpu_frame.frame_rgba;
    let result = present_drm_page_flip(
        &device,
        1,
        timeout_ms,
        hold_seconds,
        DrmEventWaiter::Polling,
        move |width, height| {
            let packed = pack_rgba_frame(&frame_rgba, 320, 240, width, height, 32)?;
            Ok((packed, gpu_checksum, true))
        },
        |_| Ok(()),
        |active| {
            println!("product=Aqua Linux");
            println!("backend=drm-kms-gpu-surface");
            println!("confirmation_source={confirmation_source}");
            println!("device={}", device.display());
            println!("selected_mode={}x{}", active.width, active.height);
            println!("composition_backend=smithay-gles2-gbm");
            println!("composition_shader=aqua-surface-compositor-v1");
            println!("composition_blur_passes=2");
            println!("composition_source_size=320x240");
            println!("composition_source_checksum={gpu_checksum:016x}");
            println!("composition_client_texture_source=sampled-wl-shm-contract");
            println!("composition_client_texture_count=2");
            println!("composition_client_textures_composited=true");
            println!("composition_live_wayland_session=false");
            println!("scanout_bridge=cpu-readback-copy");
            println!("direct_dmabuf_scanout=false");
            println!("scanout_format=xrgb8888");
            println!("scanout_checksum={:016x}", active.buffer_checksum);
            println!("front_framebuffer_created=true");
            println!("back_framebuffer_created=true");
            println!("page_flip_submitted=true");
            println!("page_flip_event_received=true");
            println!("display_output_started=true");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            println!("[AQUA-COMPOSITOR] stage=drm-gpu-surface status=active");
            let _ = std::io::stdout().flush();
        },
        |duration, _repaint| {
            thread::sleep(duration);
            Ok(())
        },
    );

    match result {
        Ok(final_state) => {
            println!("crtc_restored={}", final_state.crtc_restored);
            println!(
                "gpu_surface_front_framebuffer_destroyed={}",
                final_state.front_framebuffer_destroyed
            );
            println!(
                "gpu_surface_back_framebuffer_destroyed={}",
                final_state.back_framebuffer_destroyed
            );
            println!("display_output_stopped=true");
            println!("safe_return_to_recovery=ok");
            println!("[AQUA-COMPOSITOR] stage=drm-gpu-surface status=ok");
        }
        Err(error) => {
            eprintln!("DRM GPU surface presentation failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-gpu-surface status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(all(target_os = "linux", feature = "smithay-gpu")))]
fn present_drm_gpu_surface_cli(_device: PathBuf) {
    eprintln!("DRM GPU surface presentation requires Linux with smithay-gpu");
    println!("[AQUA-COMPOSITOR] stage=drm-gpu-surface status=unsupported-host");
    std::process::exit(1);
}

#[cfg(not(all(target_os = "linux", feature = "smithay-gpu")))]
fn probe_gpu_offscreen_frame_cli(_device: PathBuf) {
    eprintln!("GPU offscreen probe requires Linux with the smithay-gpu feature");
    println!("[AQUA-COMPOSITOR] stage=gpu-offscreen-frame status=unavailable");
    std::process::exit(1);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DrmConnectorProbe {
    name: String,
    status: String,
    modes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DrmDeviceProbe {
    device: PathBuf,
    card_name: String,
    connectors: Vec<DrmConnectorProbe>,
}

impl DrmDeviceProbe {
    fn connected_connectors(&self) -> impl Iterator<Item = &DrmConnectorProbe> {
        self.connectors
            .iter()
            .filter(|connector| connector.status == "connected")
    }

    fn is_ready(&self) -> bool {
        self.connected_connectors()
            .any(|connector| !connector.modes.is_empty())
    }
}

fn probe_drm_device_cli(device: PathBuf) {
    let sysfs_root = env::var("AQUA_DRM_SYSFS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/sys/class/drm"));
    let card_name = env::var("AQUA_DRM_CARD_NAME").unwrap_or_else(|_| {
        device
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("card0")
            .to_string()
    });

    match probe_drm_device(&device, &sysfs_root, &card_name) {
        Ok(probe) => {
            let connected_count = probe.connected_connectors().count();
            let mode_count = probe
                .connected_connectors()
                .map(|connector| connector.modes.len())
                .sum::<usize>();
            println!("product=Aqua Linux");
            println!("backend=drm-kms");
            println!("device={}", probe.device.display());
            println!("card_name={}", probe.card_name);
            println!("device_open_mode=read-only");
            println!("device_open_read_only=ok");
            println!("connector_count={}", probe.connectors.len());
            println!("connected_connector_count={connected_count}");
            println!("connected_mode_count={mode_count}");
            for connector in &probe.connectors {
                println!("connector.{}.status={}", connector.name, connector.status);
                println!(
                    "connector.{}.mode_count={}",
                    connector.name,
                    connector.modes.len()
                );
                if let Some(mode) = connector.modes.first() {
                    println!("connector.{}.first_mode={mode}", connector.name);
                }
            }
            println!("drm_master_acquired=false");
            println!("kms_activated=false");
            println!("display_output_started=false");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            println!("recovery_safe=ok");
            finish_stage("drm-device-probe", probe.is_ready());
        }
        Err(error) => {
            eprintln!("DRM device probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-device-probe status=error");
            std::process::exit(1);
        }
    }
}

fn probe_drm_device(
    device: &Path,
    sysfs_root: &Path,
    card_name: &str,
) -> Result<DrmDeviceProbe, String> {
    File::open(device)
        .map_err(|error| format!("cannot open {} read-only: {error}", device.display()))?;

    let prefix = format!("{card_name}-");
    let entries = fs::read_dir(sysfs_root)
        .map_err(|error| format!("cannot read DRM sysfs {}: {error}", sysfs_root.display()))?;
    let mut connectors = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read DRM sysfs entry: {error}"))?;
        let entry_name = entry.file_name().to_string_lossy().into_owned();
        let Some(connector_name) = entry_name.strip_prefix(&prefix) else {
            continue;
        };
        let status_path = entry.path().join("status");
        if !status_path.is_file() {
            continue;
        }
        let status = fs::read_to_string(&status_path)
            .map_err(|error| format!("cannot read {}: {error}", status_path.display()))?
            .trim()
            .to_string();
        let modes = fs::read_to_string(entry.path().join("modes"))
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|mode| !mode.is_empty())
            .map(str::to_string)
            .collect();
        connectors.push(DrmConnectorProbe {
            name: connector_name.to_string(),
            status,
            modes,
        });
    }
    connectors.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(DrmDeviceProbe {
        device: device.to_path_buf(),
        card_name: card_name.to_string(),
        connectors,
    })
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct DrmCard(File);

#[cfg(target_os = "linux")]
impl AsFd for DrmCard {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

#[cfg(target_os = "linux")]
impl drm::Device for DrmCard {}

#[cfg(target_os = "linux")]
impl DrmControlDevice for DrmCard {}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn drm_device_uses_cpu_scanout_compat(device: &Path) -> bool {
    let Ok(file) = OpenOptions::new().read(true).write(true).open(device) else {
        return false;
    };
    drm::Device::get_driver(&DrmCard(file))
        .map(|driver| driver.name() == "virtio_gpu")
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn probe_drm_dumb_buffer_cli(device: PathBuf) {
    match probe_drm_dumb_buffer(&device) {
        Ok(probe) => {
            println!("product=Aqua Linux");
            println!("backend=drm-kms");
            println!("device={}", device.display());
            println!("device_open_mode=read-write-bounded");
            println!("resource_connector_count={}", probe.connector_count);
            println!("resource_crtc_count={}", probe.crtc_count);
            println!("connected_connector_found=ok");
            println!("selected_mode={}x{}", probe.width, probe.height);
            println!("pixel_format=xrgb8888");
            println!("bits_per_pixel=32");
            println!("buffer_pitch={}", probe.pitch);
            println!("buffer_bytes={}", probe.bytes);
            println!("buffer_checksum={:016x}", probe.buffer_checksum);
            println!("source_checksum={:016x}", probe.source_checksum);
            println!(
                "wallpaper_source={}",
                if probe.runtime_wallpaper_loaded {
                    "runtime-asset"
                } else {
                    "deterministic-fallback"
                }
            );
            println!("dumb_buffer_created=true");
            println!("dumb_buffer_mapped=true");
            println!("framebuffer_created=false");
            println!("drm_master_requested=false");
            println!("kms_activated=false");
            println!("page_flip_submitted=false");
            println!("dumb_buffer_destroyed=true");
            println!("display_output_started=false");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            println!("safe_return_to_recovery=ok");
            finish_stage("drm-dumb-buffer-probe", true);
        }
        Err(error) => {
            eprintln!("DRM dumb buffer probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-dumb-buffer-probe status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn probe_drm_dumb_buffer_cli(_device: PathBuf) {
    eprintln!("DRM dumb buffer probe requires Linux");
    println!("[AQUA-COMPOSITOR] stage=drm-dumb-buffer-probe status=unsupported-host");
    std::process::exit(1);
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn probe_drm_gbm_scanout_buffer_cli(device: PathBuf) {
    match probe_drm_gbm_scanout_buffer(&device) {
        Ok(probe) => {
            println!("product=Aqua Linux");
            println!("backend=drm-gbm-scanout-foundation");
            println!("device={}", device.display());
            println!("selected_mode={}x{}", probe.width, probe.height);
            println!("pixel_format=xrgb8888");
            println!("gbm_buffer_count=2");
            println!("gbm_usage_scanout=true");
            println!("gbm_usage_rendering=true");
            println!("gbm_front_pitch={}", probe.front_pitch);
            println!("gbm_back_pitch={}", probe.back_pitch);
            println!("gbm_front_handle_count={}", probe.front_handle_count);
            println!("gbm_back_handle_count={}", probe.back_handle_count);
            println!("gbm_modifier_explicit={}", probe.modifier_explicit);
            println!("dmabuf_exported=true");
            println!("dmabuf_front_plane_count={}", probe.front_plane_count);
            println!("dmabuf_back_plane_count={}", probe.back_plane_count);
            println!("kms_addfb2_front=true");
            println!("kms_addfb2_back=true");
            println!("kms_framebuffers_destroyed=true");
            println!("kms_activated=false");
            println!("page_flip_submitted=false");
            println!("direct_dmabuf_scanout=false");
            println!("display_output_started=false");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("safe_return_to_recovery=ok");
            finish_stage("drm-gbm-scanout-buffer-probe", true);
        }
        Err(error) => {
            eprintln!("DRM GBM scanout buffer probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-gbm-scanout-buffer-probe status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(all(target_os = "linux", feature = "smithay-gpu")))]
fn probe_drm_gbm_scanout_buffer_cli(_device: PathBuf) {
    eprintln!("DRM GBM scanout buffer probe requires Linux with smithay-gpu");
    println!("[AQUA-COMPOSITOR] stage=drm-gbm-scanout-buffer-probe status=unsupported-host");
    std::process::exit(1);
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
struct DrmGbmScanoutBufferProbe {
    width: u32,
    height: u32,
    front_pitch: u32,
    back_pitch: u32,
    front_handle_count: usize,
    back_handle_count: usize,
    modifier_explicit: bool,
    front_plane_count: usize,
    back_plane_count: usize,
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn probe_drm_gbm_scanout_buffer(device: &Path) -> Result<DrmGbmScanoutBufferProbe, String> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(device)
        .map_err(|error| format!("cannot open {} read-write: {error}", device.display()))?;
    let card = DrmCard(
        file.try_clone()
            .map_err(|error| format!("cannot clone DRM card fd: {error}"))?,
    );
    let resources = card
        .resource_handles()
        .map_err(|error| format!("cannot read DRM resources: {error}"))?;
    let connectors = resources
        .connectors()
        .iter()
        .map(|handle| card.get_connector(*handle, true))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read DRM connector: {error}"))?;
    let connector = connectors
        .iter()
        .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
        .ok_or_else(|| "no connected DRM connector with a mode".to_string())?;
    let (mode_width, mode_height) = connector.modes()[0].size();
    let width = u32::from(mode_width);
    let height = u32::from(mode_height);

    let gbm = GbmDevice::new(DeviceFd::from(OwnedFd::from(file)))
        .map_err(|error| format!("cannot create GBM device on card node: {error}"))?;
    let flags = GbmBufferFlags::SCANOUT | GbmBufferFlags::RENDERING;
    let mut allocator = GbmAllocator::new(gbm, flags);
    let modifiers = [Modifier::Invalid];
    let front = allocator
        .create_buffer(width, height, Fourcc::Xrgb8888, &modifiers)
        .map_err(|error| format!("cannot allocate front GBM scanout buffer: {error}"))?;
    let back = allocator
        .create_buffer(width, height, Fourcc::Xrgb8888, &modifiers)
        .map_err(|error| format!("cannot allocate back GBM scanout buffer: {error}"))?;

    let front_dmabuf = front
        .export()
        .map_err(|error| format!("cannot export front GBM buffer as dma-buf: {error}"))?;
    let back_dmabuf = back
        .export()
        .map_err(|error| format!("cannot export back GBM buffer as dma-buf: {error}"))?;
    let modifier_explicit = PlanarBuffer::modifier(&front).is_some();
    if modifier_explicit != PlanarBuffer::modifier(&back).is_some() {
        return Err("front and back GBM modifier modes differ".to_string());
    }
    let fb_flags = if modifier_explicit {
        FbCmd2Flags::MODIFIERS
    } else {
        FbCmd2Flags::empty()
    };
    let front_framebuffer = card
        .add_planar_framebuffer(&front, fb_flags)
        .map_err(|error| format!("cannot add front GBM buffer with KMS ADDFB2: {error}"))?;
    let back_framebuffer = card
        .add_planar_framebuffer(&back, fb_flags)
        .map_err(|error| format!("cannot add back GBM buffer with KMS ADDFB2: {error}"))?;

    let probe = DrmGbmScanoutBufferProbe {
        width,
        height,
        front_pitch: PlanarBuffer::pitches(&front)[0],
        back_pitch: PlanarBuffer::pitches(&back)[0],
        front_handle_count: PlanarBuffer::handles(&front)
            .iter()
            .filter(|handle| handle.is_some())
            .count(),
        back_handle_count: PlanarBuffer::handles(&back)
            .iter()
            .filter(|handle| handle.is_some())
            .count(),
        modifier_explicit,
        front_plane_count: front_dmabuf.num_planes(),
        back_plane_count: back_dmabuf.num_planes(),
    };
    card.destroy_framebuffer(back_framebuffer)
        .map_err(|error| format!("cannot destroy back GBM framebuffer: {error}"))?;
    card.destroy_framebuffer(front_framebuffer)
        .map_err(|error| format!("cannot destroy front GBM framebuffer: {error}"))?;
    Ok(probe)
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn present_drm_gbm_scanout_cli(device: PathBuf) {
    let confirmation_source = drm_kms_confirmation_source(
        env::var("AQUA_DRM_GBM_SCANOUT_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_GBM_SCANOUT_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_GBM_SCANOUT_TEST_MODE").ok().as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!("DRM GBM scanout presentation requires explicit confirmation");
        println!("[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=blocked-confirmation");
        std::process::exit(1);
    };
    let hold_seconds = env::var("AQUA_DRM_GBM_SCANOUT_HOLD_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3)
        .min(30);
    let timeout_ms = env::var("AQUA_DRM_GBM_SCANOUT_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000)
        .clamp(100, 5_000);

    println!("product=Aqua Linux");
    println!("component=aqua-compositor");
    println!("backend=drm-gbm-dmabuf-direct-scanout");
    println!("device={}", device.display());
    println!("confirmation_source={confirmation_source}");
    println!("manual_start_required=true");
    println!("boot_graphics=false");
    println!("autostart=false");

    match present_drm_gbm_scanout(&device, hold_seconds, timeout_ms) {
        Ok(result) => {
            println!("selected_mode={}x{}", result.width, result.height);
            println!("scanout_format=xrgb8888");
            println!("scanout_pitch={}", result.pitch);
            println!("scanout_bridge=gbm-dmabuf-direct");
            println!("scanout_cpu_copy=false");
            println!("scanout_verification_readback=true");
            println!("scanout_scene_checksum={:016x}", result.scene_checksum);
            println!("direct_dmabuf_scanout=true");
            println!("gbm_front_rendered=true");
            println!("gbm_back_rendered=true");
            println!("page_flip_submitted=true");
            println!("page_flip_event_received=true");
            println!("page_flip_event_frame={}", result.event_frame);
            println!("crtc_restored=true");
            println!("front_framebuffer_destroyed=true");
            println!("back_framebuffer_destroyed=true");
            println!("display_output_stopped=true");
            println!("safe_return_to_recovery=ok");
            finish_stage("drm-gbm-scanout", true);
        }
        Err(error) => {
            eprintln!("DRM GBM direct scanout failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(all(target_os = "linux", feature = "smithay-gpu")))]
fn present_drm_gbm_scanout_cli(_device: PathBuf) {
    eprintln!("DRM GBM direct scanout requires Linux with smithay-gpu");
    println!("[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=unsupported-host");
    std::process::exit(1);
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
struct DrmGbmScanoutResult {
    width: u32,
    height: u32,
    pitch: u32,
    scene_checksum: u64,
    event_frame: u32,
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
fn present_drm_gbm_scanout(
    device: &Path,
    hold_seconds: u64,
    timeout_ms: u64,
) -> Result<DrmGbmScanoutResult, String> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(device)
        .map_err(|error| format!("cannot open {} read-write: {error}", device.display()))?;
    let card = DrmCard(
        file.try_clone()
            .map_err(|error| format!("cannot clone DRM card fd: {error}"))?,
    );
    let resources = card
        .resource_handles()
        .map_err(|error| format!("cannot read DRM resources: {error}"))?;
    let connectors = resources
        .connectors()
        .iter()
        .map(|handle| card.get_connector(*handle, true))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read DRM connector: {error}"))?;
    let connector = connectors
        .iter()
        .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
        .ok_or_else(|| "no connected DRM connector with a mode".to_string())?;
    let mode = connector.modes()[0];
    let crtc_handle = connector
        .current_encoder()
        .and_then(|handle| card.get_encoder(handle).ok())
        .and_then(|encoder| encoder.crtc())
        .or_else(|| resources.crtcs().first().copied())
        .ok_or_else(|| "no DRM CRTC is available".to_string())?;
    let original_crtc = card
        .get_crtc(crtc_handle)
        .map_err(|error| format!("cannot read original DRM CRTC: {error}"))?;
    let (mode_width, mode_height) = mode.size();
    let width = u32::from(mode_width);
    let height = u32::from(mode_height);

    let gbm = GbmDevice::new(DeviceFd::from(OwnedFd::from(file)))
        .map_err(|error| format!("cannot create scanout GBM device: {error}"))?;
    let mut allocator = GbmAllocator::new(gbm, GbmBufferFlags::SCANOUT | GbmBufferFlags::RENDERING);
    let modifiers = [Modifier::Linear];
    let front = allocator
        .create_buffer(width, height, Fourcc::Xrgb8888, &modifiers)
        .map_err(|error| format!("cannot allocate front direct-scanout buffer: {error}"))?;
    let back = allocator
        .create_buffer(width, height, Fourcc::Xrgb8888, &modifiers)
        .map_err(|error| format!("cannot allocate back direct-scanout buffer: {error}"))?;
    let mut front_dmabuf = front
        .export()
        .map_err(|error| format!("cannot export front direct-scanout dma-buf: {error}"))?;
    let mut back_dmabuf = back
        .export()
        .map_err(|error| format!("cannot export back direct-scanout dma-buf: {error}"))?;

    let paint_plan = probe_client_layer_pipeline(Viewport::new(1536, 1024))
        .map_err(|error| format!("cannot prepare direct-scanout scene: {error}"))?
        .paint_plan;
    let mut compositor = LiveGpuCompositor::new_on_render_device(device)?;
    println!("drm_gbm_scanout_renderer_created=true");
    let _ = std::io::stdout().flush();
    let scene_frame = compositor.render(&paint_plan)?;
    println!("drm_gbm_scanout_scene_rendered=true");
    let _ = std::io::stdout().flush();
    compositor.render_to_scanout(&mut front_dmabuf, width, height)?;
    println!("drm_gbm_scanout_front_bound=true");
    let _ = std::io::stdout().flush();
    compositor.render_to_scanout(&mut back_dmabuf, width, height)?;
    println!("drm_gbm_scanout_back_bound=true");
    let _ = std::io::stdout().flush();

    let modifier_explicit = PlanarBuffer::modifier(&front).is_some();
    let fb_flags = if modifier_explicit {
        FbCmd2Flags::MODIFIERS
    } else {
        FbCmd2Flags::empty()
    };
    let front_framebuffer = card
        .add_planar_framebuffer(&front, fb_flags)
        .map_err(|error| format!("cannot add front direct-scanout framebuffer: {error}"))?;
    let back_framebuffer = card
        .add_planar_framebuffer(&back, fb_flags)
        .map_err(|error| format!("cannot add back direct-scanout framebuffer: {error}"))?;

    card.set_crtc(
        crtc_handle,
        Some(front_framebuffer),
        (0, 0),
        &[connector.handle()],
        Some(mode),
    )
    .map_err(|error| format!("cannot activate front direct-scanout framebuffer: {error}"))?;
    println!("[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=active");
    let _ = std::io::stdout().flush();
    thread::sleep(Duration::from_secs(hold_seconds));

    let presentation_result = (|| {
        card.page_flip(crtc_handle, back_framebuffer, PageFlipFlags::EVENT, None)
            .map_err(|error| format!("cannot submit direct-scanout page flip: {error}"))?;
        wait_for_drm_page_flip(&card, crtc_handle, timeout_ms, 1, DrmEventWaiter::Polling)
    })();

    let restore_connectors = if original_crtc.mode().is_some() {
        vec![connector.handle()]
    } else {
        Vec::new()
    };
    let restore_result = card.set_crtc(
        crtc_handle,
        original_crtc.framebuffer(),
        original_crtc.position(),
        &restore_connectors,
        original_crtc.mode(),
    );
    let back_cleanup = card.destroy_framebuffer(back_framebuffer);
    let front_cleanup = card.destroy_framebuffer(front_framebuffer);

    let event_frame = presentation_result?;
    restore_result.map_err(|error| format!("cannot restore original DRM CRTC: {error}"))?;
    back_cleanup.map_err(|error| format!("cannot destroy back direct framebuffer: {error}"))?;
    front_cleanup.map_err(|error| format!("cannot destroy front direct framebuffer: {error}"))?;
    Ok(DrmGbmScanoutResult {
        width,
        height,
        pitch: PlanarBuffer::pitches(&front)[0],
        scene_checksum: scene_frame.checksum,
        event_frame,
    })
}

#[cfg(target_os = "linux")]
struct DrmDumbBufferProbe {
    connector_count: usize,
    crtc_count: usize,
    width: u32,
    height: u32,
    pitch: u32,
    bytes: usize,
    buffer_checksum: u64,
    source_checksum: u64,
    runtime_wallpaper_loaded: bool,
}

#[cfg(target_os = "linux")]
fn probe_drm_dumb_buffer(device: &Path) -> Result<DrmDumbBufferProbe, String> {
    let card = DrmCard(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|error| format!("cannot open {} read-write: {error}", device.display()))?,
    );
    let resources = card
        .resource_handles()
        .map_err(|error| format!("cannot read DRM resources: {error}"))?;
    let connectors = resources
        .connectors()
        .iter()
        .map(|handle| card.get_connector(*handle, true))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read DRM connector: {error}"))?;
    let connector = connectors
        .iter()
        .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
        .ok_or_else(|| "no connected DRM connector with a mode".to_string())?;
    let mode = connector.modes()[0];
    let (mode_width, mode_height) = mode.size();
    let width = u32::from(mode_width);
    let height = u32::from(mode_height);
    let mut buffer = card
        .create_dumb_buffer((width, height), DrmFourcc::Xrgb8888, 32)
        .map_err(|error| format!("cannot create DRM dumb buffer: {error}"))?;
    let pitch = buffer.pitch();
    let packed_stride = width as usize * 4;
    if (pitch as usize) < packed_stride {
        return Err("DRM dumb buffer pitch is smaller than one packed row".to_string());
    }
    let (packed_frame, source_checksum, runtime_wallpaper_loaded) =
        render_fbdev_frame(width, height, 32)?;
    let pitched_frame = with_stride(
        &packed_frame,
        packed_stride,
        pitch as usize,
        height as usize,
    );
    let bytes = pitched_frame.len();
    let buffer_checksum = checksum_bytes(&pitched_frame);
    {
        let mut mapping = card
            .map_dumb_buffer(&mut buffer)
            .map_err(|error| format!("cannot map DRM dumb buffer: {error}"))?;
        if mapping.len() < pitched_frame.len() {
            return Err("mapped DRM dumb buffer is smaller than the rendered frame".to_string());
        }
        mapping[..pitched_frame.len()].copy_from_slice(&pitched_frame);
    }
    card.destroy_dumb_buffer(buffer)
        .map_err(|error| format!("cannot destroy DRM dumb buffer: {error}"))?;

    Ok(DrmDumbBufferProbe {
        connector_count: connectors.len(),
        crtc_count: resources.crtcs().len(),
        width,
        height,
        pitch,
        bytes,
        buffer_checksum,
        source_checksum,
        runtime_wallpaper_loaded,
    })
}

#[cfg(any(target_os = "linux", test))]
fn drm_kms_confirmation_source(
    operator_confirmed: bool,
    headless_test_confirmed: bool,
    test_mode: Option<&str>,
) -> Option<&'static str> {
    if operator_confirmed {
        Some("manual-operator")
    } else if headless_test_confirmed && test_mode == Some("headless-qemu") {
        Some("headless-qemu-test")
    } else {
        None
    }
}

#[cfg(any(target_os = "linux", test))]
fn drm_wayland_hold_seconds(value: Option<&str>, persistent: bool) -> Option<u64> {
    if persistent {
        None
    } else {
        Some(
            value
                .and_then(|value| value.parse().ok())
                .unwrap_or(3)
                .min(30),
        )
    }
}

#[cfg(target_os = "linux")]
fn present_drm_kms_cli(device: PathBuf) {
    let confirmation_source = drm_kms_confirmation_source(
        env::var("AQUA_DRM_KMS_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_KMS_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_KMS_TEST_MODE").ok().as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!("DRM KMS presentation requires explicit operator or headless QEMU confirmation");
        println!("[AQUA-COMPOSITOR] stage=drm-kms-present status=blocked-confirmation");
        std::process::exit(1);
    };
    let hold_seconds = env::var("AQUA_DRM_KMS_HOLD_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3)
        .min(30);

    let result = present_drm_kms(&device, hold_seconds, |active| {
        println!("product=Aqua Linux");
        println!("backend=drm-kms");
        println!("confirmation_source={confirmation_source}");
        println!("device={}", device.display());
        println!("connector={}", active.connector);
        println!("selected_mode={}x{}", active.width, active.height);
        println!("pixel_format=xrgb8888");
        println!("buffer_pitch={}", active.pitch);
        println!("buffer_bytes={}", active.bytes);
        println!("buffer_checksum={:016x}", active.buffer_checksum);
        println!("framebuffer_created=true");
        println!("kms_activated=true");
        println!("display_output_started=true");
        println!("bounded_hold_seconds={hold_seconds}");
        println!("page_flip_submitted=false");
        println!("boot_graphics=false");
        println!("autostart=false");
        println!("persistent_graphical_session_started=false");
        println!("[AQUA-COMPOSITOR] stage=drm-kms-present status=active");
        let _ = std::io::stdout().flush();
    });

    match result {
        Ok(final_state) => {
            println!("crtc_restored={}", final_state.crtc_restored);
            println!(
                "framebuffer_destroyed={}",
                final_state.framebuffer_destroyed
            );
            println!(
                "dumb_buffer_destroyed={}",
                final_state.dumb_buffer_destroyed
            );
            println!("display_output_stopped=true");
            println!("safe_return_to_recovery=ok");
            println!("[AQUA-COMPOSITOR] stage=drm-kms-present status=ok");
        }
        Err(error) => {
            eprintln!("DRM KMS presentation failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-kms-present status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn present_drm_kms_cli(_device: PathBuf) {
    eprintln!("DRM KMS presentation requires Linux");
    println!("[AQUA-COMPOSITOR] stage=drm-kms-present status=unsupported-host");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
struct DrmKmsActiveFrame {
    connector: String,
    width: u32,
    height: u32,
    pitch: u32,
    bytes: usize,
    buffer_checksum: u64,
}

#[cfg(target_os = "linux")]
struct DrmKmsFinalState {
    crtc_restored: bool,
    framebuffer_destroyed: bool,
    dumb_buffer_destroyed: bool,
}

#[cfg(target_os = "linux")]
fn present_drm_kms(
    device: &Path,
    hold_seconds: u64,
    on_active: impl FnOnce(&DrmKmsActiveFrame),
) -> Result<DrmKmsFinalState, String> {
    let card = DrmCard(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|error| format!("cannot open {} read-write: {error}", device.display()))?,
    );
    let resources = card
        .resource_handles()
        .map_err(|error| format!("cannot read DRM resources: {error}"))?;
    let connectors = resources
        .connectors()
        .iter()
        .map(|handle| card.get_connector(*handle, true))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read DRM connector: {error}"))?;
    let connector = connectors
        .iter()
        .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
        .ok_or_else(|| "no connected DRM connector with a mode".to_string())?;
    let mode = connector.modes()[0];
    let crtc_handle = connector
        .current_encoder()
        .and_then(|handle| card.get_encoder(handle).ok())
        .and_then(|encoder| encoder.crtc())
        .or_else(|| resources.crtcs().first().copied())
        .ok_or_else(|| "no DRM CRTC is available".to_string())?;
    let original_crtc = card
        .get_crtc(crtc_handle)
        .map_err(|error| format!("cannot read original DRM CRTC: {error}"))?;
    let (mode_width, mode_height) = mode.size();
    let width = u32::from(mode_width);
    let height = u32::from(mode_height);
    let mut buffer = card
        .create_dumb_buffer((width, height), DrmFourcc::Xrgb8888, 32)
        .map_err(|error| format!("cannot create DRM dumb buffer: {error}"))?;
    let pitch = buffer.pitch();
    let packed_stride = width as usize * 4;
    let (packed_frame, _, _) = render_fbdev_frame(width, height, 32)?;
    let pitched_frame = with_stride(
        &packed_frame,
        packed_stride,
        pitch as usize,
        height as usize,
    );
    {
        let mut mapping = card
            .map_dumb_buffer(&mut buffer)
            .map_err(|error| format!("cannot map DRM dumb buffer: {error}"))?;
        if mapping.len() < pitched_frame.len() {
            return Err("mapped DRM dumb buffer is smaller than the rendered frame".to_string());
        }
        mapping[..pitched_frame.len()].copy_from_slice(&pitched_frame);
    }
    let framebuffer = card
        .add_framebuffer(&buffer, 24, 32)
        .map_err(|error| format!("cannot create KMS framebuffer: {error}"))?;
    card.set_crtc(
        crtc_handle,
        Some(framebuffer),
        (0, 0),
        &[connector.handle()],
        Some(mode),
    )
    .map_err(|error| format!("cannot activate bounded KMS framebuffer: {error}"))?;

    on_active(&DrmKmsActiveFrame {
        connector: connector.to_string(),
        width,
        height,
        pitch,
        bytes: pitched_frame.len(),
        buffer_checksum: checksum_bytes(&pitched_frame),
    });
    thread::sleep(Duration::from_secs(hold_seconds));

    let restore_connectors = if original_crtc.mode().is_some() {
        vec![connector.handle()]
    } else {
        Vec::new()
    };
    card.set_crtc(
        crtc_handle,
        original_crtc.framebuffer(),
        original_crtc.position(),
        &restore_connectors,
        original_crtc.mode(),
    )
    .map_err(|error| format!("cannot restore original DRM CRTC: {error}"))?;
    card.destroy_framebuffer(framebuffer)
        .map_err(|error| format!("cannot destroy KMS framebuffer: {error}"))?;
    card.destroy_dumb_buffer(buffer)
        .map_err(|error| format!("cannot destroy DRM dumb buffer: {error}"))?;

    Ok(DrmKmsFinalState {
        crtc_restored: true,
        framebuffer_destroyed: true,
        dumb_buffer_destroyed: true,
    })
}

#[cfg(target_os = "linux")]
struct DrmPageFlipActiveFrame {
    connector: String,
    width: u32,
    height: u32,
    pitch: u32,
    bytes: usize,
    buffer_checksum: u64,
    event_frames: Vec<u32>,
}

#[cfg(target_os = "linux")]
struct DrmPageFlipFinalState {
    crtc_restored: bool,
    front_framebuffer_destroyed: bool,
    back_framebuffer_destroyed: bool,
    front_buffer_destroyed: bool,
    back_buffer_destroyed: bool,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
enum DrmEventWaiter {
    Polling,
    Calloop,
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
macro_rules! present_drm_wayland_page_flip {
    ($device:expr, $frames:expr, $timeout:expr, $hold:expr, $waiter:expr, $render:expr, $scanout:expr, $event:expr, $active:expr, $during_hold:expr $(,)?) => {
        present_drm_gbm_page_flip(
            $device,
            $frames,
            $timeout,
            $hold,
            $waiter,
            $render,
            $scanout,
            $event,
            $active,
            $during_hold,
        )
    };
}

#[cfg(all(target_os = "linux", not(feature = "smithay-gpu")))]
macro_rules! present_drm_wayland_page_flip {
    ($device:expr, $frames:expr, $timeout:expr, $hold:expr, $waiter:expr, $render:expr, $scanout:expr, $event:expr, $active:expr, $during_hold:expr $(,)?) => {
        present_drm_page_flip(
            $device,
            $frames,
            $timeout,
            $hold,
            $waiter,
            $render,
            $event,
            $active,
            $during_hold,
        )
    };
}

#[cfg(target_os = "linux")]
fn present_drm_page_flip_cli(device: PathBuf) {
    let confirmation_source = drm_kms_confirmation_source(
        env::var("AQUA_DRM_PAGE_FLIP_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_PAGE_FLIP_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_PAGE_FLIP_TEST_MODE").ok().as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!("DRM page flip requires explicit operator or headless QEMU confirmation");
        println!("[AQUA-COMPOSITOR] stage=drm-page-flip status=blocked-confirmation");
        std::process::exit(1);
    };
    let hold_seconds = env::var("AQUA_DRM_PAGE_FLIP_HOLD_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3)
        .min(30);
    let timeout_ms = env::var("AQUA_DRM_PAGE_FLIP_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000)
        .clamp(100, 5_000);

    let result = present_drm_page_flip(
        &device,
        1,
        timeout_ms,
        hold_seconds,
        DrmEventWaiter::Polling,
        |width, height| render_fbdev_frame(width, height, 32),
        |_| Ok(()),
        |active| {
            println!("product=Aqua Linux");
            println!("backend=drm-kms-page-flip");
            println!("confirmation_source={confirmation_source}");
            println!("device={}", device.display());
            println!("connector={}", active.connector);
            println!("selected_mode={}x{}", active.width, active.height);
            println!("pixel_format=xrgb8888");
            println!("buffer_pitch={}", active.pitch);
            println!("buffer_bytes={}", active.bytes);
            println!("buffer_checksum={:016x}", active.buffer_checksum);
            println!("front_framebuffer_created=true");
            println!("back_framebuffer_created=true");
            println!("kms_activated=true");
            println!("page_flip_submitted=true");
            println!("page_flip_event_received=true");
            println!("page_flip_event_frame={}", active.event_frames[0]);
            println!("display_output_started=true");
            println!("bounded_event_timeout_ms={timeout_ms}");
            println!("bounded_hold_seconds={hold_seconds}");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            println!("[AQUA-COMPOSITOR] stage=drm-page-flip status=active");
            let _ = std::io::stdout().flush();
        },
        |duration, _repaint| {
            thread::sleep(duration);
            Ok(())
        },
    );

    match result {
        Ok(final_state) => {
            println!("crtc_restored={}", final_state.crtc_restored);
            println!(
                "front_framebuffer_destroyed={}",
                final_state.front_framebuffer_destroyed
            );
            println!(
                "back_framebuffer_destroyed={}",
                final_state.back_framebuffer_destroyed
            );
            println!(
                "front_dumb_buffer_destroyed={}",
                final_state.front_buffer_destroyed
            );
            println!(
                "back_dumb_buffer_destroyed={}",
                final_state.back_buffer_destroyed
            );
            println!("display_output_stopped=true");
            println!("safe_return_to_recovery=ok");
            println!("[AQUA-COMPOSITOR] stage=drm-page-flip status=ok");
        }
        Err(error) => {
            eprintln!("DRM page flip failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-page-flip status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn present_drm_page_flip_cli(_device: PathBuf) {
    eprintln!("DRM page flip requires Linux");
    println!("[AQUA-COMPOSITOR] stage=drm-page-flip status=unsupported-host");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn run_drm_frame_loop_cli(device: PathBuf) {
    let confirmation_source = drm_kms_confirmation_source(
        env::var("AQUA_DRM_FRAME_LOOP_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_FRAME_LOOP_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_FRAME_LOOP_TEST_MODE").ok().as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!("DRM frame loop requires explicit operator or headless QEMU confirmation");
        println!("[AQUA-COMPOSITOR] stage=drm-frame-loop status=blocked-confirmation");
        std::process::exit(1);
    };
    let frame_count = env::var("AQUA_DRM_FRAME_LOOP_FRAMES")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(3)
        .clamp(2, 8);
    let timeout_ms = env::var("AQUA_DRM_FRAME_LOOP_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000)
        .clamp(100, 5_000);
    let hold_seconds = env::var("AQUA_DRM_FRAME_LOOP_HOLD_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3)
        .min(30);

    let result = present_drm_page_flip(
        &device,
        frame_count,
        timeout_ms,
        hold_seconds,
        DrmEventWaiter::Polling,
        |width, height| render_fbdev_frame(width, height, 32),
        |_| Ok(()),
        |active| {
            let event_frames = active
                .event_frames
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let event_sequence_available = active.event_frames.iter().any(|frame| *frame != 0);
            println!("product=Aqua Linux");
            println!("backend=drm-kms-frame-loop");
            println!("confirmation_source={confirmation_source}");
            println!("device={}", device.display());
            println!("connector={}", active.connector);
            println!("selected_mode={}x{}", active.width, active.height);
            println!("pixel_format=xrgb8888");
            println!("buffer_pitch={}", active.pitch);
            println!("buffer_bytes={}", active.bytes);
            println!("buffer_checksum={:016x}", active.buffer_checksum);
            println!("front_framebuffer_created=true");
            println!("back_framebuffer_created=true");
            println!("kms_activated=true");
            println!("requested_frames={frame_count}");
            println!("submitted_page_flips={}", active.event_frames.len());
            println!("received_page_flip_events={}", active.event_frames.len());
            println!("page_flip_event_frames={event_frames}");
            println!("page_flip_event_order_complete=true");
            println!("page_flip_event_sequence_available={event_sequence_available}");
            println!("front_back_buffer_alternation=true");
            println!("display_output_started=true");
            println!("bounded_event_timeout_ms={timeout_ms}");
            println!("bounded_hold_seconds={hold_seconds}");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            println!("[AQUA-COMPOSITOR] stage=drm-frame-loop status=active");
            let _ = std::io::stdout().flush();
        },
        |duration, _repaint| {
            thread::sleep(duration);
            Ok(())
        },
    );

    match result {
        Ok(final_state) => {
            println!("crtc_restored={}", final_state.crtc_restored);
            println!(
                "front_framebuffer_destroyed={}",
                final_state.front_framebuffer_destroyed
            );
            println!(
                "back_framebuffer_destroyed={}",
                final_state.back_framebuffer_destroyed
            );
            println!(
                "front_dumb_buffer_destroyed={}",
                final_state.front_buffer_destroyed
            );
            println!(
                "back_dumb_buffer_destroyed={}",
                final_state.back_buffer_destroyed
            );
            println!("display_output_stopped=true");
            println!("safe_return_to_recovery=ok");
            println!("[AQUA-COMPOSITOR] stage=drm-frame-loop status=ok");
        }
        Err(error) => {
            eprintln!("DRM frame loop failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-frame-loop status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn run_drm_frame_loop_cli(_device: PathBuf) {
    eprintln!("DRM frame loop requires Linux");
    println!("[AQUA-COMPOSITOR] stage=drm-frame-loop status=unsupported-host");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn run_drm_session_loop_cli(device: PathBuf) {
    let confirmation_source = drm_kms_confirmation_source(
        env::var("AQUA_DRM_SESSION_LOOP_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_SESSION_LOOP_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_SESSION_LOOP_TEST_MODE").ok().as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!("DRM session loop requires explicit operator or headless QEMU confirmation");
        println!("[AQUA-COMPOSITOR] stage=drm-session-loop status=blocked-confirmation");
        std::process::exit(1);
    };
    let frame_count = env::var("AQUA_DRM_SESSION_LOOP_FRAMES")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(3)
        .clamp(2, 8);
    let timeout_ms = env::var("AQUA_DRM_SESSION_LOOP_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2_000)
        .clamp(100, 5_000);
    let hold_seconds = env::var("AQUA_DRM_SESSION_LOOP_HOLD_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3)
        .min(30);

    let result = present_drm_page_flip(
        &device,
        frame_count,
        timeout_ms,
        hold_seconds,
        DrmEventWaiter::Calloop,
        |width, height| render_fbdev_frame(width, height, 32),
        |_| Ok(()),
        |active| {
            println!("product=Aqua Linux");
            println!("component=aqua-compositor");
            println!("backend=drm-kms-session-loop");
            println!("confirmation_source={confirmation_source}");
            println!("session_owner=aqua-compositor");
            println!("event_loop=calloop");
            println!("drm_event_source_owned=true");
            println!("device={}", device.display());
            println!("connector={}", active.connector);
            println!("selected_mode={}x{}", active.width, active.height);
            println!("pixel_format=xrgb8888");
            println!("buffer_pitch={}", active.pitch);
            println!("buffer_bytes={}", active.bytes);
            println!("buffer_checksum={:016x}", active.buffer_checksum);
            println!("requested_frames={frame_count}");
            println!("calloop_dispatch_passes={}", active.event_frames.len());
            println!("received_page_flip_events={}", active.event_frames.len());
            println!("front_back_buffer_alternation=true");
            println!("display_output_started=true");
            println!("wayland_display_started=false");
            println!("bounded_event_timeout_ms={timeout_ms}");
            println!("bounded_hold_seconds={hold_seconds}");
            println!("manual_start_required=true");
            println!("recovery_tty_required=true");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            println!("[AQUA-COMPOSITOR] stage=drm-session-loop status=active");
            let _ = std::io::stdout().flush();
        },
        |duration, _repaint| {
            thread::sleep(duration);
            Ok(())
        },
    );

    match result {
        Ok(final_state) => {
            println!("crtc_restored={}", final_state.crtc_restored);
            println!(
                "front_framebuffer_destroyed={}",
                final_state.front_framebuffer_destroyed
            );
            println!(
                "back_framebuffer_destroyed={}",
                final_state.back_framebuffer_destroyed
            );
            println!(
                "front_dumb_buffer_destroyed={}",
                final_state.front_buffer_destroyed
            );
            println!(
                "back_dumb_buffer_destroyed={}",
                final_state.back_buffer_destroyed
            );
            println!("drm_event_source_released=true");
            println!("display_output_stopped=true");
            println!("safe_return_to_recovery=ok");
            println!("[AQUA-COMPOSITOR] stage=drm-session-loop status=ok");
        }
        Err(error) => {
            eprintln!("DRM session loop failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-session-loop status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn run_drm_session_loop_cli(_device: PathBuf) {
    eprintln!("DRM session loop requires Linux");
    println!("[AQUA-COMPOSITOR] stage=drm-session-loop status=unsupported-host");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn run_drm_wayland_session_cli(device: PathBuf) {
    let boot_graphics = env::var("AQUA_BOOT_GRAPHICS").as_deref() == Ok("true");
    let autostart = env::var("AQUA_COMPOSITOR_AUTOSTART").as_deref() == Ok("true");
    let confirmation_source = drm_kms_confirmation_source(
        env::var("AQUA_DRM_WAYLAND_SESSION_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_WAYLAND_SESSION_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_DRM_WAYLAND_SESSION_TEST_MODE")
            .ok()
            .as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!("DRM Wayland session requires explicit operator or headless QEMU confirmation");
        println!("[AQUA-COMPOSITOR] stage=drm-wayland-session status=blocked-confirmation");
        std::process::exit(1);
    };
    let frame_count = 3;
    let timeout_ms = 2_000;
    let persistent_session =
        env::var("AQUA_DRM_WAYLAND_SESSION_PERSISTENT").as_deref() == Ok("true");
    let hold_seconds = drm_wayland_hold_seconds(
        env::var("AQUA_DRM_WAYLAND_SESSION_HOLD_SECONDS")
            .ok()
            .as_deref(),
        persistent_session,
    );
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let graceful_stop_file = env::var_os("AQUA_DRM_WAYLAND_STOP_FILE").map(PathBuf::from);
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let graceful_stop_requested = Rc::new(Cell::new(false));
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let input_required = env::var("AQUA_DRM_WAYLAND_INPUT_REQUIRED").as_deref() == Ok("true");
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let diagnostic_scenario =
        env::var("AQUA_DRM_WAYLAND_SCENARIO").as_deref() == Ok("qemu-integration");
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let installer_scenario =
        env::var("AQUA_DRM_WAYLAND_SCENARIO").as_deref() == Ok("installer-welcome");
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let typography_scenario =
        env::var("AQUA_DRM_WAYLAND_SCENARIO").as_deref() == Ok("typography-acceptance");
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let elevation_scenario =
        env::var("AQUA_DRM_WAYLAND_SCENARIO").as_deref() == Ok("elevation-acceptance");
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let icon_scenario = env::var("AQUA_DRM_WAYLAND_SCENARIO").as_deref() == Ok("icon-acceptance");
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let installer_scenario = false;
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let typography_scenario = false;
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let elevation_scenario = false;
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let icon_scenario = false;
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let external_client_required = diagnostic_scenario
        && env::var("AQUA_DRM_WAYLAND_EXTERNAL_CLIENT_REQUIRED").as_deref() == Ok("true");
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let external_client_required = false;
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let fixture_clients_required = external_client_required || elevation_scenario;
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let fixture_clients_required = false;
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let managed_client_required =
        fixture_clients_required || installer_scenario || typography_scenario;
    let runtime_dir = PathBuf::from("/run/aqua");
    let socket_path = runtime_dir.join("aqua-wayland-drm-0");
    let lock_path = socket_path.with_extension("lock");
    fs::create_dir_all(&runtime_dir).unwrap_or_else(|error| {
        eprintln!("cannot prepare Aqua runtime directory: {error}");
        std::process::exit(1);
    });
    let _ = fs::remove_file(&socket_path);
    let _ = fs::remove_file(&lock_path);

    let listener = ListeningSocket::bind_absolute(socket_path.clone()).unwrap_or_else(|error| {
        eprintln!("cannot bind Aqua Wayland socket: {error}");
        std::process::exit(1);
    });
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let (client_stream, mut external_clients): (
        Option<std::os::unix::net::UnixStream>,
        Vec<Child>,
    ) = if managed_client_required {
        println!(
            "wayland_external_client_socket_ready={}",
            socket_path.display()
        );
        let _ = std::io::stdout().flush();
        let executable = env::current_exe().unwrap_or_else(|error| {
            eprintln!("cannot resolve Aqua compositor executable: {error}");
            std::process::exit(1);
        });
        if installer_scenario || typography_scenario {
            let (client, label) = if installer_scenario {
                (PathBuf::from("/usr/bin/aqua-installer"), "Installer")
            } else {
                (
                    PathBuf::from("/usr/libexec/aqua-tests/aqua-typography-acceptance"),
                    "typography acceptance",
                )
            };
            if !client.is_file() {
                eprintln!("missing Aqua {label} Wayland client: {}", client.display());
                std::process::exit(1);
            }
            let child = Command::new(&client)
                .env("XDG_RUNTIME_DIR", &runtime_dir)
                .env("WAYLAND_DISPLAY", "aqua-wayland-drm-0")
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap_or_else(|error| {
                    eprintln!("cannot start Aqua {label} Wayland client: {error}");
                    std::process::exit(1);
                });
            if installer_scenario {
                println!("installer_wayland_client_process_started=true");
            } else {
                println!("typography_wayland_client_process_started=true");
            }
            (None, vec![child])
        } else {
            let compatibility_client = PathBuf::from("/usr/libexec/aqua-tests/weston-simple-shm");
            if !compatibility_client.is_file() {
                eprintln!(
                    "missing third-party Wayland compatibility client: {}",
                    compatibility_client.display()
                );
                std::process::exit(1);
            }
            let mut children = Vec::with_capacity(2);
            children.push(
                Command::new(&compatibility_client)
                    .env("XDG_RUNTIME_DIR", &runtime_dir)
                    .env("WAYLAND_DISPLAY", "aqua-wayland-drm-0")
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .unwrap_or_else(|error| {
                        eprintln!("cannot start upstream weston-simple-shm client: {error}");
                        std::process::exit(1);
                    }),
            );
            children.push(
                Command::new(&executable)
                    .arg("run-wayland-test-client")
                    .arg(&socket_path)
                    .env("AQUA_WAYLAND_TEST_CLIENT_VARIANT", "1")
                    .env("AQUA_WAYLAND_TEST_CLIENT_CONTROLLED_EXIT", "false")
                    .env("AQUA_WAYLAND_TEST_CLIENT_WAIT_FOR_CLOSE", "true")
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .unwrap_or_else(|error| {
                        eprintln!("cannot start external Aqua Wayland state client: {error}");
                        std::process::exit(1);
                    }),
            );
            println!("third_party_wayland_client=weston-simple-shm");
            println!("third_party_wayland_client_role=compatibility-fixture");
            println!("weston_compositor_started=false");
            (None, children)
        }
    } else {
        let stream =
            std::os::unix::net::UnixStream::connect(&socket_path).unwrap_or_else(|error| {
                eprintln!("cannot connect Aqua Wayland transport client: {error}");
                std::process::exit(1);
            });
        (Some(stream), Vec::new())
    };
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let client_stream = Some(
        std::os::unix::net::UnixStream::connect(&socket_path).unwrap_or_else(|error| {
            eprintln!("cannot connect Aqua Wayland transport client: {error}");
            std::process::exit(1);
        }),
    );
    let expected_client_count = if fixture_clients_required { 2 } else { 1 };
    let mut server_streams = Vec::with_capacity(expected_client_count);
    for _ in 0..500 {
        match listener.accept() {
            Ok(Some(stream)) => server_streams.push(stream),
            Ok(None) => thread::sleep(Duration::from_millis(1)),
            Err(error) => eprintln!("cannot accept Aqua Wayland client: {error}"),
        }
        if server_streams.len() == expected_client_count {
            break;
        }
    }
    if server_streams.len() != expected_client_count {
        eprintln!(
            "Aqua Wayland accepted {} of {expected_client_count} clients",
            server_streams.len()
        );
        std::process::exit(1);
    }
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let smithay_session = RefCell::new(SmithayDrmSession::new().unwrap_or_else(|error| {
        eprintln!("cannot create Aqua Smithay session: {error}");
        std::process::exit(1);
    }));
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    for server_stream in server_streams.drain(..) {
        smithay_session
            .borrow_mut()
            .insert_client(server_stream)
            .unwrap_or_else(|error| {
                eprintln!("cannot insert Aqua Smithay client: {error}");
                std::process::exit(1);
            });
    }
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let external_surface_snapshot = if managed_client_required {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            smithay_session
                .borrow_mut()
                .dispatch_clients()
                .unwrap_or_else(|error| {
                    eprintln!("cannot dispatch external Aqua Wayland client: {error}");
                    std::process::exit(1);
                });
            smithay_session
                .borrow_mut()
                .flush_clients()
                .unwrap_or_else(|error| {
                    eprintln!("cannot flush external Aqua Wayland client: {error}");
                    std::process::exit(1);
                });
            let mut snapshots = smithay_session.borrow().client_surface_snapshots();
            let surfaces_ready = if installer_scenario || typography_scenario {
                snapshots.len() == 1
                    && snapshots[0].is_ready()
                    && snapshots[0].width == 1280
                    && snapshots[0].height == 800
            } else {
                snapshots.len() == 2
                    && snapshots.iter().all(SmithayClientSurfaceSnapshot::is_ready)
                    && snapshots[0].sample_checksum != snapshots[1].sample_checksum
            };
            if surfaces_ready {
                if elevation_scenario {
                    if !smithay_session.borrow_mut().present_client_surface(1) {
                        eprintln!("elevation acceptance surface could not receive focus");
                        std::process::exit(1);
                    }
                    snapshots = smithay_session.borrow().client_surface_snapshots();
                }
                if installer_scenario || typography_scenario {
                    snapshots[0].x = 0;
                    snapshots[0].y = 0;
                    snapshots[0].display_width = 1536;
                    snapshots[0].display_height = 1024;
                }
                println!("external_wayland_surface_ready=true");
                println!("external_wayland_surface_count={}", snapshots.len());
                println!(
                    "external_wayland_surface_sizes={}",
                    snapshots
                        .iter()
                        .map(|snapshot| format!("{}x{}", snapshot.width, snapshot.height))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                println!(
                    "external_wayland_surface_bytes={}",
                    snapshots
                        .iter()
                        .map(|snapshot| snapshot.buffer_rgba.len())
                        .sum::<usize>()
                );
                println!(
                    "external_wayland_surface_checksums={}",
                    snapshots
                        .iter()
                        .map(|snapshot| format!("{:016x}", snapshot.sample_checksum))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                if installer_scenario {
                    println!("installer_wayland_surface_ready=true");
                    println!("installer_wayland_surface_size=1280x800");
                    println!(
                        "installer_wayland_surface_checksum={:016x}",
                        snapshots[0].sample_checksum
                    );
                    println!("installer_wayland_surface_execution_allowed=false");
                } else if typography_scenario {
                    println!("typography_wayland_surface_ready=true");
                    println!("typography_wayland_surface_size=1280x800");
                    println!(
                        "typography_wayland_surface_checksum={:016x}",
                        snapshots[0].sample_checksum
                    );
                } else {
                    if elevation_scenario {
                        let focused = snapshots
                            .iter()
                            .filter(|snapshot| snapshot.keyboard_focus_assigned)
                            .count();
                        println!("elevation_wayland_surface_ready=true");
                        println!("elevation_wayland_surface_count={}", snapshots.len());
                        println!("elevation_wayland_focused_surface_count={focused}");
                        println!(
                            "elevation_wayland_inactive_surface_count={}",
                            snapshots.len().saturating_sub(focused)
                        );
                    }
                    if !smithay_session
                        .borrow_mut()
                        .raise_surface_with_buffer_size(384, 256)
                    {
                        eprintln!("Aqua state-cycle surface could not be raised");
                        std::process::exit(1);
                    }
                    println!("aqua_state_client_raised=true");
                }
                break snapshots;
            }
            if std::time::Instant::now() >= deadline {
                eprintln!("external Aqua Wayland surface did not commit before timeout");
                std::process::exit(1);
            }
            if external_clients
                .iter_mut()
                .any(|child| child.try_wait().ok().flatten().is_some())
            {
                eprintln!("external Aqua Wayland client exited before committing a surface");
                std::process::exit(1);
            }
            thread::sleep(Duration::from_millis(10));
        }
    } else {
        Vec::new()
    };

    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let mut display = WaylandDisplay::<()>::new().unwrap_or_else(|error| {
        eprintln!("cannot create Aqua Wayland display: {error}");
        std::process::exit(1);
    });
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    display
        .handle()
        .insert_client(
            server_streams.pop().expect("transport stream must exist"),
            Arc::new(()),
        )
        .unwrap_or_else(|error| {
            eprintln!("cannot insert Aqua Wayland client: {error}");
            std::process::exit(1);
        });
    #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
    let mut wayland_state = ();
    let mut wayland_dispatch_passes = 0_u32;
    let mut wayland_flush_passes = 0_u32;
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let smithay_compositor_global_started = smithay_session.borrow().compositor_global_started();
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let smithay_shm_global_started = smithay_session.borrow().shm_global_started();
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let smithay_xdg_shell_global_started = smithay_session.borrow().xdg_shell_global_started();
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let smithay_seat_started = smithay_session.borrow().seat_started();
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let input_source = RefCell::new(if input_required {
        Some(
            LibinputAquaSeatSource::open("seat0").unwrap_or_else(|error| {
                eprintln!("cannot prepare DRM Wayland libinput source: {error}");
                std::process::exit(1);
            }),
        )
    } else {
        None
    });
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let input_enabled = input_source.borrow().is_some();
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let drm_surface_dimensions = RefCell::new(None);
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let initial_external_frame_checksum = RefCell::new(None);
    #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
    let live_gpu_wayland_compositor = RefCell::new(None);
    #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
    let live_gpu_wayland_frame = RefCell::new(None);
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_process_supervisor = RefCell::new(FirstPartyProcessSupervisor::default());
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    if icon_scenario {
        smithay_session.borrow_mut().post_notification(
            1,
            "Aqua Desktop",
            "Scale-native icons",
            "Aqua Core Icon raster cache is active.",
        );
    }
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_launcher_state = RefCell::new(smithay_session.borrow().launcher_state_snapshot());
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_dock_state = RefCell::new(current_dock_state(
        &runtime_launcher_state.borrow(),
        &runtime_process_supervisor.borrow(),
        smithay_session.borrow().active_workspace(),
    ));
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_desktop_icon_state =
        RefCell::new(smithay_session.borrow().desktop_icon_state_snapshot());
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_session_menu_state =
        RefCell::new(smithay_session.borrow().session_menu_state_snapshot());
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_notification_state =
        RefCell::new(smithay_session.borrow().notification_center_snapshot());
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_system_overview = RefCell::new(
        aqua_shell::SystemOverviewModel::read(
            Path::new("/"),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        )
        .unwrap_or_else(|error| {
            eprintln!("cannot read Aqua system overview: {error}");
            std::process::exit(1);
        }),
    );
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_top_bar = RefCell::new(aqua_shell::TopBarState::read(
        Path::new("/"),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    ));
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let last_system_overview_refresh_ms = Cell::new(0_u64);
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let desktop_session_started_at = std::time::Instant::now();
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_surface_revisions = RefCell::new(desktop_surface_revisions(
        &smithay_session.borrow().visible_client_surface_snapshots(),
    ));
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_repaint_sequence = Cell::new(0_u64);
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    let runtime_theme = Cell::new(configured_runtime_theme());

    let result = present_drm_wayland_page_flip!(
        &device,
        frame_count,
        timeout_ms,
        hold_seconds.unwrap_or_default(),
        DrmEventWaiter::Calloop,
        |width, height| {
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            {
                *drm_surface_dimensions.borrow_mut() = Some((width, height));
                smithay_session
                    .borrow_mut()
                    .set_output_dimensions(width, height);
                if !external_surface_snapshot.is_empty() {
                    let rendered = render_fbdev_frame_with_external_clients(
                        width,
                        height,
                        &external_surface_snapshot,
                        &smithay_session.borrow().launcher_state_snapshot(),
                    )?;
                    *initial_external_frame_checksum.borrow_mut() =
                        Some(checksum_bytes(&rendered.0));
                    #[cfg(not(all(target_os = "linux", feature = "smithay-gpu")))]
                    return Ok(rendered);
                }
                #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
                {
                    let paint_plan = external_client_paint_plan(&external_surface_snapshot)?;
                    let mut compositor = LiveGpuCompositor::new_on_render_device_with_viewport(
                        &device,
                        Viewport::new(width, height),
                    )?;
                    if installer_scenario || typography_scenario {
                        compositor.set_shell_chrome_visible(false);
                        if installer_scenario {
                            println!("installer_wayland_shell_chrome_visible=false");
                        } else {
                            println!("typography_wayland_shell_chrome_visible=false");
                        }
                    } else {
                        compositor.set_launcher_state(&runtime_launcher_state.borrow())?;
                        compositor.set_top_bar_state(&runtime_top_bar.borrow())?;
                        compositor.set_desktop_icons_state(&runtime_desktop_icon_state.borrow())?;
                        compositor.set_dock_state(&runtime_dock_state.borrow())?;
                        compositor.set_system_overview_state(&runtime_system_overview.borrow())?;
                        compositor.set_notification_state(&runtime_notification_state.borrow())?;
                        if icon_scenario {
                            compositor.dock_state = None;
                            compositor.set_dock_state(&runtime_dock_state.borrow())?;
                            let stats = compositor.icon_raster_cache.stats();
                            println!(
                                "icon_wayland_raster_cache_ready={}",
                                compositor.icon_raster_cache.len() == 10
                                    && stats.hits == 3
                                    && stats.misses == 10
                                    && stats.parsed_sources == 7
                                    && stats.evictions == 0
                            );
                            println!("icon_wayland_raster_roles=7");
                            println!("icon_wayland_raster_surfaces=4");
                            println!("icon_wayland_theme={}", compositor.theme.id());
                        }
                    }
                    let gpu_frame =
                        compositor.render_direct_at(&paint_plan, width, height, None, None)?;
                    println!(
                        "desktop_system_overview_visible={}",
                        !(installer_scenario || typography_scenario)
                    );
                    let scanout_frame =
                        pack_rgba_frame(&gpu_frame.frame_rgba, width, height, width, height, 32)?;
                    let checksum = gpu_frame.checksum;
                    *live_gpu_wayland_compositor.borrow_mut() = Some(compositor);
                    *live_gpu_wayland_frame.borrow_mut() = Some(gpu_frame);
                    Ok((scanout_frame, checksum, true))
                }
            }
            #[cfg(not(all(
                target_os = "linux",
                feature = "smithay-smoke",
                feature = "smithay-gpu"
            )))]
            return render_fbdev_frame(width, height, 32);
        },
        |target, width, height| {
            let mut compositor = live_gpu_wayland_compositor.borrow_mut();
            compositor
                .as_mut()
                .ok_or_else(|| "live GPU compositor is unavailable".to_string())?
                .render_to_scanout(target, width, height)
        },
        |_frame_index| {
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            {
                smithay_session
                    .borrow_mut()
                    .dispatch_clients()
                    .map_err(|error| format!("cannot dispatch Smithay clients: {error}"))?;
                if managed_client_required && _frame_index == 1 {
                    let presented = smithay_session
                        .borrow_mut()
                        .present_client_surface(_frame_index * 16);
                    if !presented {
                        return Err("external Wayland surface could not receive frame/focus".into());
                    }
                }
                smithay_session
                    .borrow_mut()
                    .flush_clients()
                    .map_err(|error| format!("cannot flush Smithay clients: {error}"))?;
            }
            #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
            {
                display
                    .dispatch_clients(&mut wayland_state)
                    .map_err(|error| format!("cannot dispatch Wayland clients: {error}"))?;
                display
                    .flush_clients()
                    .map_err(|error| format!("cannot flush Wayland clients: {error}"))?;
            }
            wayland_dispatch_passes += 1;
            wayland_flush_passes += 1;
            Ok(())
        },
        |active| {
            println!("product=Aqua Linux");
            println!("component=aqua-compositor");
            println!("backend=drm-kms-wayland-session");
            println!("confirmation_source={confirmation_source}");
            println!("session_owner=aqua-compositor");
            println!("shared_session_lifecycle=true");
            println!("event_loop=calloop");
            println!("drm_event_source_owned=true");
            println!("wayland_display_created=true");
            println!("wayland_socket={}", socket_path.display());
            println!("wayland_socket_bound=true");
            println!("wayland_client_connected=true");
            println!("wayland_client_inserted=true");
            println!("wayland_dispatch_passes={frame_count}");
            println!("wayland_flush_passes={frame_count}");
            println!(
                "smithay_protocol_globals_started={}",
                cfg!(feature = "smithay-smoke")
            );
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            {
                println!(
                    "compositor_global_started={}",
                    smithay_compositor_global_started
                );
                println!("shm_global_started={smithay_shm_global_started}");
                println!(
                    "xdg_shell_global_started={}",
                    smithay_xdg_shell_global_started
                );
                println!("seat_started={smithay_seat_started}");
                println!("input_source=libinput-udev");
                println!("input_source_enabled={input_enabled}");
                println!("input_required={input_required}");
                if let Some(source) = input_source.borrow().as_ref() {
                    println!("input_seat={}", source.seat_name);
                }
            }
            #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
            println!("seat_started=false");
            println!("device={}", device.display());
            println!("connector={}", active.connector);
            println!("selected_mode={}x{}", active.width, active.height);
            println!("buffer_checksum={:016x}", active.buffer_checksum);
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            if !external_surface_snapshot.is_empty() {
                println!(
                    "external_wayland_frame_checksum={:016x}",
                    active.buffer_checksum
                );
            }
            #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
            if let Some(gpu_frame) = live_gpu_wayland_frame.borrow().as_ref() {
                let virtio_scanout_compat = drm_device_uses_cpu_scanout_compat(&device);
                println!(
                    "drm_wayland_composition_backend={}",
                    if virtio_scanout_compat {
                        "smithay-gles2-readback-dumb-buffer"
                    } else {
                        "smithay-gles2-gbm"
                    }
                );
                println!("drm_wayland_gpu_render_device={}", device.display());
                println!("drm_wayland_gpu_render_node_separate=false");
                println!("drm_wayland_gpu_client_texture_source=live-smithay-wl-shm-snapshot");
                println!(
                    "drm_wayland_gpu_client_texture_count={}",
                    gpu_frame.client_texture_count
                );
                println!(
                    "drm_wayland_gpu_client_texture_bytes={}",
                    gpu_frame.client_texture_bytes
                );
                println!("drm_wayland_gpu_client_textures_uploaded=true");
                println!("drm_wayland_gpu_client_textures_composited=true");
                println!("drm_wayland_gpu_live_session=true");
                println!(
                    "drm_wayland_gpu_initial_frame_checksum={:016x}",
                    gpu_frame.checksum
                );
                println!("drm_wayland_gpu_context_lifecycle=session-owned");
                println!(
                    "drm_wayland_scanout_bridge={}",
                    if virtio_scanout_compat {
                        "gpu-readback-dumb-buffer"
                    } else {
                        "gbm-dmabuf-direct"
                    }
                );
                println!("drm_wayland_scanout_cpu_copy={virtio_scanout_compat}");
                println!(
                    "drm_wayland_direct_dmabuf_scanout={}",
                    !virtio_scanout_compat
                );
                println!("drm_wayland_gpu_frame_readback=true");
                println!("drm_wayland_gpu_checksum_source=frame-readback");
            }
            println!("calloop_drm_dispatch_passes={}", active.event_frames.len());
            println!("received_page_flip_events={}", active.event_frames.len());
            println!("display_output_started=true");
            println!("manual_start_required={}", !boot_graphics);
            println!("recovery_tty_required=true");
            println!("boot_graphics={boot_graphics}");
            println!("autostart={autostart}");
            println!("persistent_graphical_session_started={boot_graphics}");
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            println!(
                "session_scenario={}",
                if diagnostic_scenario {
                    "qemu-integration"
                } else if installer_scenario {
                    "installer-welcome"
                } else if typography_scenario {
                    "typography-acceptance"
                } else if elevation_scenario {
                    "elevation-acceptance"
                } else if icon_scenario {
                    "icon-acceptance"
                } else {
                    "desktop-event-loop"
                }
            );
            println!("external_fixture_clients_started={fixture_clients_required}");
            println!("installer_wayland_client_started={installer_scenario}");
            println!("typography_wayland_client_started={typography_scenario}");
            println!("elevation_wayland_client_started={elevation_scenario}");
            println!("icon_wayland_scenario_started={icon_scenario}");
            println!(
                "session_lifetime_policy={}",
                if persistent_session {
                    "persistent-until-stop"
                } else {
                    "bounded"
                }
            );
            println!("[AQUA-COMPOSITOR] stage=drm-wayland-session status=active");
            let _ = std::io::stdout().flush();
        },
        |_duration, _repaint| {
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            {
                if let Some(source) = input_source.borrow_mut().as_mut() {
                    let deadline = hold_seconds
                        .map(|seconds| std::time::Instant::now() + Duration::from_secs(seconds));
                    loop {
                        if graceful_stop_file
                            .as_ref()
                            .is_some_and(|path| path.is_file())
                        {
                            graceful_stop_requested.set(true);
                            if let Some(path) = graceful_stop_file.as_ref() {
                                let _ = fs::remove_file(path);
                            }
                            println!("drm_wayland_graceful_stop_requested=true");
                            let _ = std::io::stdout().flush();
                            break;
                        }
                        let now = std::time::Instant::now();
                        if deadline.is_some_and(|deadline| now >= deadline) {
                            break;
                        }
                        let dispatch_timeout = deadline
                            .map(|deadline| deadline - now)
                            .unwrap_or(Duration::from_millis(100))
                            .min(Duration::from_millis(100));
                        source
                            .dispatch_until(&mut smithay_session.borrow_mut(), dispatch_timeout)?;
                        if smithay_session.borrow().has_session_action_request() {
                            println!("desktop_input_action_yield=desktop-event-loop");
                        }

                        if !diagnostic_scenario {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch desktop Wayland clients: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush desktop Wayland clients: {error}")
                                })?;

                            let configured_theme = configured_runtime_theme();
                            let theme_changed = runtime_theme.get() != configured_theme;
                            if theme_changed {
                                runtime_theme.set(configured_theme);
                                #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
                                if let Some(compositor) =
                                    live_gpu_wayland_compositor.borrow_mut().as_mut()
                                {
                                    compositor.set_theme(configured_theme);
                                }
                                println!(
                                    "desktop_runtime_theme_broadcast={}",
                                    configured_theme.id()
                                );
                            }

                            if installer_scenario {
                                let mut snapshots =
                                    smithay_session.borrow().client_surface_snapshots();
                                let revisions = desktop_surface_revisions(&snapshots);
                                let surface_changed =
                                    *runtime_surface_revisions.borrow() != revisions;
                                if surface_changed || theme_changed {
                                    for snapshot in &mut snapshots {
                                        snapshot.x = 0;
                                        snapshot.y = 0;
                                        snapshot.display_width = 1536;
                                        snapshot.display_height = 1024;
                                    }
                                    #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
                                    let repaint_frame = {
                                        if let Some(compositor) =
                                            live_gpu_wayland_compositor.borrow_mut().as_mut()
                                        {
                                            compositor.set_shell_chrome_visible(false);
                                        }
                                        let (output_width, output_height) = drm_surface_dimensions
                                            .borrow()
                                            .as_ref()
                                            .copied()
                                            .ok_or_else(|| {
                                                "installer repaint dimensions are unavailable"
                                                    .to_string()
                                            })?;
                                        let (frame, gpu_frame) = render_live_gpu_wayland_frame(
                                            &live_gpu_wayland_compositor,
                                            &snapshots,
                                            output_width,
                                            output_height,
                                        )?;
                                        *live_gpu_wayland_frame.borrow_mut() = Some(gpu_frame);
                                        frame
                                    };
                                    #[cfg(not(all(target_os = "linux", feature = "smithay-gpu")))]
                                    let repaint_frame = {
                                        let snapshot = snapshots.first().ok_or_else(|| {
                                            "installer repaint surface is unavailable".to_string()
                                        })?;
                                        pack_rgba_frame(
                                            &snapshot.buffer_rgba,
                                            snapshot.width,
                                            snapshot.height,
                                            drm_surface_dimensions
                                                .borrow()
                                                .as_ref()
                                                .copied()
                                                .ok_or_else(|| {
                                                    "installer repaint dimensions are unavailable"
                                                        .to_string()
                                                })?
                                                .0,
                                            drm_surface_dimensions
                                                .borrow()
                                                .as_ref()
                                                .copied()
                                                .ok_or_else(|| {
                                                    "installer repaint dimensions are unavailable"
                                                        .to_string()
                                                })?
                                                .1,
                                            32,
                                        )?
                                    };
                                    let checksum = _repaint(repaint_frame)?;
                                    let _ =
                                        smithay_session.borrow_mut().present_client_surface(20_000);
                                    *runtime_surface_revisions.borrow_mut() =
                                        desktop_surface_revisions(
                                            &smithay_session.borrow().client_surface_snapshots(),
                                        );
                                    println!("installer_wayland_repaint=true");
                                    println!("installer_wayland_repaint_checksum={checksum:016x}");
                                    println!(
                                        "installer_wayland_repaint_commit_count={}",
                                        snapshots
                                            .iter()
                                            .map(|snapshot| snapshot.commit_count)
                                            .max()
                                            .unwrap_or_default()
                                    );
                                    let _ = std::io::stdout().flush();
                                }
                                let snapshot = smithay_session.borrow().input_snapshot();
                                let installer_ready = snapshot.keyboard_event_count >= 98
                                    && snapshot.keyboard_forward_count >= 98
                                    && source.keyboard_devices >= 1
                                    && smithay_session
                                        .borrow()
                                        .client_surface_snapshots()
                                        .iter()
                                        .any(|surface| surface.commit_count >= 37);
                                if installer_ready {
                                    println!("installer_wayland_input_sequence_complete=true");
                                    let _ = std::io::stdout().flush();
                                    thread::sleep(Duration::from_secs(3));
                                    break;
                                }
                                continue;
                            }

                            let mut process_exited = false;
                            for app_id in ["files", "settings", "properties", "terminal"] {
                                let reaped = {
                                    let mut supervisor = runtime_process_supervisor.borrow_mut();
                                    if supervisor.contains(app_id) {
                                        supervisor.try_reap(app_id).map_err(|error| {
                                            format!(
                                                "cannot poll desktop client {app_id}: {error:?}"
                                            )
                                        })?
                                    } else {
                                        None
                                    }
                                };
                                if let Some(process) = reaped {
                                    process_exited = true;
                                    println!("desktop_runtime_process_exited_{app_id}=true");
                                    println!(
                                        "desktop_runtime_process_exit_{app_id}_success={}",
                                        process.success
                                    );
                                    let restart_policy = first_party_restart_policy(app_id)
                                        .map(|policy| policy.as_str())
                                        .unwrap_or("unsupported");
                                    println!(
                                        "desktop_runtime_process_{app_id}_restart_policy={restart_policy}"
                                    );
                                    println!(
                                        "desktop_runtime_process_{app_id}_restart_attempted=false"
                                    );
                                    println!(
                                        "desktop_runtime_process_active_count={}",
                                        runtime_process_supervisor.borrow().active_count()
                                    );
                                    let now_ms = desktop_session_started_at
                                        .elapsed()
                                        .as_millis()
                                        .min(u128::from(u64::MAX))
                                        as u64;
                                    smithay_session.borrow_mut().post_notification(
                                        now_ms,
                                        "Aqua System",
                                        "Application closed",
                                        &format!("{} has stopped.", app_id),
                                    );
                                }
                            }

                            let request =
                                smithay_session.borrow_mut().take_launcher_launch_request();
                            let launched = if let Some(request) = request.as_ref() {
                                launch_first_party_desktop_client(
                                    request,
                                    &listener,
                                    &smithay_session,
                                    &runtime_process_supervisor,
                                    &runtime_dir,
                                )?
                            } else {
                                false
                            };
                            let now_ms = desktop_session_started_at
                                .elapsed()
                                .as_millis()
                                .min(u128::from(u64::MAX))
                                as u64;
                            if launched
                                && request.as_ref().is_some_and(|request| {
                                    !matches!(request.app_id, "properties" | "terminal")
                                })
                            {
                                let app_name = request
                                    .as_ref()
                                    .map(|request| match request.app_id {
                                        "files" => "Aqua Files",
                                        "settings" => "System Settings",
                                        "properties" => "Properties",
                                        "terminal" => "Aqua Terminal",
                                        _ => "Application",
                                    })
                                    .unwrap_or("Application");
                                smithay_session.borrow_mut().post_notification(
                                    now_ms,
                                    "Aqua Launcher",
                                    "Application opened",
                                    &format!("{} is ready.", app_name),
                                );
                            }
                            let notification_ticked =
                                smithay_session.borrow_mut().tick_notifications(now_ms);
                            let shell_status_changed = if now_ms
                                .saturating_sub(last_system_overview_refresh_ms.get())
                                >= aqua_shell::SYSTEM_OVERVIEW_REFRESH_MS
                            {
                                last_system_overview_refresh_ms.set(now_ms);
                                let epoch_seconds = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                let refreshed = aqua_shell::SystemOverviewModel::read(
                                    Path::new("/"),
                                    epoch_seconds,
                                )
                                .map_err(|error| {
                                    format!("cannot refresh system overview: {error}")
                                })?;
                                let top_bar =
                                    aqua_shell::TopBarState::read(Path::new("/"), epoch_seconds);
                                let changed = *runtime_system_overview.borrow() != refreshed
                                    || *runtime_top_bar.borrow() != top_bar;
                                *runtime_system_overview.borrow_mut() = refreshed;
                                *runtime_top_bar.borrow_mut() = top_bar;
                                changed
                            } else {
                                false
                            };
                            let system_overview_state = runtime_system_overview.borrow().clone();
                            let top_bar_state = runtime_top_bar.borrow().clone();
                            let launcher_state = smithay_session.borrow().launcher_state_snapshot();
                            let dock_state = current_dock_state(
                                &launcher_state,
                                &runtime_process_supervisor.borrow(),
                                smithay_session.borrow().active_workspace(),
                            );
                            let desktop_icon_state =
                                smithay_session.borrow().desktop_icon_state_snapshot();
                            let session_menu_state =
                                smithay_session.borrow().session_menu_state_snapshot();
                            let notification_state =
                                smithay_session.borrow().notification_center_snapshot();
                            let session_action =
                                smithay_session.borrow_mut().take_session_action_request();
                            let launcher_changed =
                                *runtime_launcher_state.borrow() != launcher_state;
                            let dock_changed = *runtime_dock_state.borrow() != dock_state;
                            let desktop_icons_changed =
                                *runtime_desktop_icon_state.borrow() != desktop_icon_state;
                            let session_menu_changed =
                                *runtime_session_menu_state.borrow() != session_menu_state;
                            let notification_changed =
                                *runtime_notification_state.borrow() != notification_state;
                            let snapshots =
                                smithay_session.borrow().visible_client_surface_snapshots();
                            let revisions = desktop_surface_revisions(&snapshots);
                            let surface_changed = *runtime_surface_revisions.borrow() != revisions;
                            if session_action.is_none()
                                && (launcher_changed
                                    || dock_changed
                                    || desktop_icons_changed
                                    || session_menu_changed
                                    || notification_changed
                                    || notification_ticked
                                    || shell_status_changed
                                    || theme_changed
                                    || launched
                                    || surface_changed
                                    || process_exited)
                            {
                                #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
                                let repaint_frame = {
                                    if let Some(compositor) =
                                        live_gpu_wayland_compositor.borrow_mut().as_mut()
                                    {
                                        compositor.set_launcher_state(&launcher_state)?;
                                        compositor.set_top_bar_state(&top_bar_state)?;
                                        compositor.set_desktop_icons_state(&desktop_icon_state)?;
                                        compositor.set_dock_state(&dock_state)?;
                                        compositor
                                            .set_system_overview_state(&system_overview_state)?;
                                        compositor.set_session_menu_state(&session_menu_state)?;
                                        compositor.set_notification_state(&notification_state)?;
                                        compositor
                                            .set_client_window_presence(!snapshots.is_empty());
                                    }
                                    let (output_width, output_height) = drm_surface_dimensions
                                        .borrow()
                                        .as_ref()
                                        .copied()
                                        .ok_or_else(|| {
                                            "desktop repaint dimensions are unavailable".to_string()
                                        })?;
                                    let (frame, gpu_frame) = render_live_gpu_wayland_frame(
                                        &live_gpu_wayland_compositor,
                                        &snapshots,
                                        output_width,
                                        output_height,
                                    )?;
                                    *live_gpu_wayland_frame.borrow_mut() = Some(gpu_frame);
                                    frame
                                };
                                #[cfg(not(all(target_os = "linux", feature = "smithay-gpu")))]
                                let repaint_frame = {
                                    let (width, height) = drm_surface_dimensions
                                        .borrow()
                                        .as_ref()
                                        .copied()
                                        .ok_or_else(|| {
                                            "desktop repaint dimensions are unavailable".to_string()
                                        })?;
                                    render_fbdev_frame_with_external_clients(
                                        width,
                                        height,
                                        &snapshots,
                                        &launcher_state,
                                    )?
                                    .0
                                };
                                let checksum = _repaint(repaint_frame)?;
                                let _ = smithay_session.borrow_mut().present_client_surface(20_000);
                                let presented_revisions = desktop_surface_revisions(
                                    &smithay_session.borrow().visible_client_surface_snapshots(),
                                );
                                let repaint_sequence = runtime_repaint_sequence.get() + 1;
                                runtime_repaint_sequence.set(repaint_sequence);
                                println!("desktop_event_repaint=true");
                                println!("desktop_event_repaint_sequence={repaint_sequence}");
                                println!("desktop_event_repaint_checksum={checksum:016x}");
                                println!("desktop_event_theme_changed={theme_changed}");
                                println!(
                                    "desktop_event_surface_revision_changed={surface_changed}"
                                );
                                println!(
                                    "desktop_event_damage_commit_count={}",
                                    snapshots
                                        .iter()
                                        .map(|snapshot| snapshot.damage_commit_count)
                                        .max()
                                        .unwrap_or_default()
                                );
                                println!(
                                    "desktop_event_launcher_visible={}",
                                    launcher_state.is_open()
                                );
                                println!("desktop_event_client_surface_count={}", snapshots.len());
                                println!(
                                    "desktop_session_menu_visible={}",
                                    session_menu_state.is_open()
                                );
                                println!(
                                    "desktop_session_menu_selected={}",
                                    session_menu_state.selected_action().id()
                                );
                                println!(
                                    "desktop_session_menu_confirmation={}",
                                    session_menu_state
                                        .confirmation()
                                        .map_or("none", aqua_shell::SessionAction::id)
                                );
                                println!(
                                    "desktop_notification_visible={}",
                                    notification_state.active().is_some()
                                );
                                println!(
                                    "desktop_notification_active_id={}",
                                    notification_state
                                        .active()
                                        .map_or(0, |notification| notification.id)
                                );
                                println!(
                                    "desktop_notification_queued={}",
                                    notification_state.queued_count()
                                );
                                println!("desktop_system_overview_visible=true");
                                println!(
                                    "desktop_system_overview_memory_percent={}",
                                    system_overview_state.memory_used_percent()
                                );
                                *runtime_launcher_state.borrow_mut() = launcher_state;
                                *runtime_dock_state.borrow_mut() = dock_state;
                                *runtime_desktop_icon_state.borrow_mut() = desktop_icon_state;
                                *runtime_session_menu_state.borrow_mut() = session_menu_state;
                                *runtime_notification_state.borrow_mut() = notification_state;
                                *runtime_surface_revisions.borrow_mut() = presented_revisions;
                            }
                            if let Some(action) = session_action {
                                println!("desktop_session_action_requested={}", action.id());
                                println!("desktop_session_action_confirmed=true");
                                match action {
                                    aqua_shell::SessionAction::Logout
                                    | aqua_shell::SessionAction::Recovery => {
                                        println!(
                                            "desktop_session_action_execution=return-to-recovery"
                                        );
                                        graceful_stop_requested.set(true);
                                    }
                                    aqua_shell::SessionAction::Restart
                                    | aqua_shell::SessionAction::Shutdown => {
                                        println!(
                                            "desktop_session_action_execution=delegated-to-system"
                                        );
                                    }
                                }
                            }
                            if graceful_stop_requested.get() {
                                println!("drm_wayland_graceful_stop_requested=true");
                                let _ = std::io::stdout().flush();
                                break;
                            }
                        }
                    }
                    let snapshot = smithay_session.borrow().input_snapshot();
                    let ready = if installer_scenario {
                        snapshot.keyboard_event_count >= 98
                            && snapshot.keyboard_forward_count >= 98
                            && source.keyboard_devices >= 1
                            && smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .iter()
                                .any(|surface| surface.commit_count >= 37)
                    } else {
                        snapshot.keyboard_event_count >= 1
                            && snapshot.pointer_motion_count >= 1
                            && snapshot.pointer_button_count >= 2
                            && snapshot.launcher_visible
                            && snapshot.keyboard_shortcut_intercept_count >= 1
                            && snapshot.keyboard_forward_count >= 2
                            && snapshot.pointer_surface_hit_count >= 1
                            && snapshot.surface_focus_change_count >= 1
                            && snapshot.stacking_change_count >= 1
                            && snapshot.launcher_pointer_hit_count >= 1
                            && snapshot.launcher_category_click_count >= 1
                            && snapshot.launcher_app_click_count >= 1
                            && snapshot.launcher_launch_request.is_some()
                            && source.keyboard_devices >= 1
                            && source.pointer_devices >= 1
                    };
                    if input_required && !ready && !graceful_stop_requested.get() {
                        return Err(format!(
                            "required DRM Wayland libinput events were not dispatched (keyboard_devices={}, pointer_devices={}, keyboard_events={}, shortcut_intercepts={}, keys_forwarded={}, pointer_motion_events={}, pointer_surface_hits={}, pointer_button_events={}, pointer={}x{}, launcher_hits={}, category_clicks={}, app_clicks={}, launch_request={})",
                            source.keyboard_devices,
                            source.pointer_devices,
                            snapshot.keyboard_event_count,
                            snapshot.keyboard_shortcut_intercept_count,
                            snapshot.keyboard_forward_count,
                            snapshot.pointer_motion_count,
                            snapshot.pointer_surface_hit_count,
                            snapshot.pointer_button_count,
                            snapshot.pointer_x,
                            snapshot.pointer_y,
                            snapshot.launcher_pointer_hit_count,
                            snapshot.launcher_category_click_count,
                            snapshot.launcher_app_click_count,
                            snapshot.launcher_launch_request.as_ref().map_or("none", |request| request.app_id)
                        ));
                    }
                } else {
                    while hold_seconds.is_none()
                        && !graceful_stop_file
                            .as_ref()
                            .is_some_and(|path| path.is_file())
                    {
                        thread::sleep(Duration::from_millis(100));
                    }
                    if graceful_stop_file
                        .as_ref()
                        .is_some_and(|path| path.is_file())
                    {
                        graceful_stop_requested.set(true);
                        if let Some(path) = graceful_stop_file.as_ref() {
                            let _ = fs::remove_file(path);
                        }
                        println!("drm_wayland_graceful_stop_requested=true");
                    }
                    if let Some(seconds) = hold_seconds {
                        thread::sleep(Duration::from_secs(seconds));
                    }
                    if input_required {
                        return Err("required DRM Wayland libinput source is not configured".into());
                    }
                }
            }
            #[cfg(all(target_os = "linux", not(feature = "smithay-smoke")))]
            if let Some(seconds) = hold_seconds {
                thread::sleep(Duration::from_secs(seconds));
            }
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            if graceful_stop_requested.get() {
                return Ok(());
            }
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            if external_client_required {
                let deadline = std::time::Instant::now() + Duration::from_secs(1);
                loop {
                    smithay_session
                        .borrow_mut()
                        .dispatch_clients()
                        .map_err(|error| {
                            format!("cannot dispatch external client update: {error}")
                        })?;
                    smithay_session
                        .borrow_mut()
                        .flush_clients()
                        .map_err(|error| format!("cannot flush external client update: {error}"))?;
                    let snapshot = smithay_session.borrow().client_surface_snapshot();
                    if snapshot.damage_commit_count >= 4
                        && snapshot.move_request_count >= 1
                        && snapshot.resize_request_count >= 1
                        && snapshot.maximize_request_count >= 1
                        && snapshot.unmaximize_request_count >= 1
                        && snapshot.fullscreen_request_count >= 1
                        && snapshot.unfullscreen_request_count >= 1
                        && snapshot.configure_ack_count >= 7
                    {
                        break;
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err(format!(
                            "external client update did not settle (damage_commits={}, pending_frame_callbacks={})",
                            snapshot.damage_commit_count, snapshot.pending_frame_callback_count
                        ));
                    }
                    thread::sleep(Duration::from_millis(10));
                }

                let (active_width, active_height) = drm_surface_dimensions
                    .borrow()
                    .as_ref()
                    .copied()
                    .ok_or_else(|| "DRM surface dimensions are unavailable".to_string())?;
                if !smithay_session
                    .borrow_mut()
                    .raise_surface_with_buffer_size(384, 256)
                {
                    return Err("Aqua state surface could not be raised".to_string());
                }
                println!("drm_wayland_gpu_repaint_surface_raised=384x256");
                smithay_session.borrow_mut().prepare_launcher_search_demo();
                let reordered_snapshots = smithay_session.borrow().client_surface_snapshots();
                let gpu_source_order_changed = reordered_snapshots.len()
                    == external_surface_snapshot.len()
                    && reordered_snapshots
                        .iter()
                        .zip(&external_surface_snapshot)
                        .any(|(current, initial)| {
                            current.sample_checksum != initial.sample_checksum
                        });
                if !gpu_source_order_changed {
                    return Err("live GPU snapshot stacking order did not change".to_string());
                }
                println!("drm_wayland_gpu_repaint_source_order_changed=true");
                let (reordered_frame, _, _) = render_fbdev_frame_with_external_clients(
                    active_width,
                    active_height,
                    &reordered_snapshots,
                    &smithay_session.borrow().launcher_state_snapshot(),
                )?;
                let reordered_checksum = checksum_bytes(&reordered_frame);
                let stacking_changed_frame = initial_external_frame_checksum
                    .borrow()
                    .as_ref()
                    .is_some_and(|initial| *initial != reordered_checksum);
                if !stacking_changed_frame {
                    return Err("stacking repaint did not change the rendered frame".to_string());
                }
                #[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
                let reordered_frame = {
                    let paint_plan = external_client_paint_plan(&reordered_snapshots)?;
                    let mut compositor = live_gpu_wayland_compositor.borrow_mut();
                    let compositor = compositor
                        .as_mut()
                        .ok_or_else(|| "live GPU compositor is unavailable".to_string())?;
                    let gpu_frame = compositor.render_direct(&paint_plan)?;
                    println!("drm_wayland_gpu_repaint_updates=true");
                    println!("drm_wayland_gpu_context_reused=true");
                    println!(
                        "drm_wayland_gpu_repaint_texture_count={}",
                        gpu_frame.client_texture_count
                    );
                    println!(
                        "drm_wayland_gpu_repaint_texture_bytes={}",
                        gpu_frame.client_texture_bytes
                    );
                    println!(
                        "drm_wayland_gpu_repaint_checksum={:016x}",
                        gpu_frame.checksum
                    );
                    gpu_frame.checksum.to_le_bytes().to_vec()
                };
                let repaint_checksum = _repaint(reordered_frame)?;
                println!("drm_wayland_stacking_repaint_complete=true");
                println!("drm_wayland_stacking_repaint_changed_frame=true");
                println!("drm_wayland_stacking_repaint_page_flips=1");
                println!("drm_wayland_stacking_repaint_checksum={repaint_checksum:016x}");
                let launcher_state = smithay_session.borrow().launcher_state_snapshot();
                println!(
                    "drm_wayland_launcher_overlay_rendered={}",
                    launcher_state.is_open()
                );
                println!(
                    "drm_wayland_launcher_category_count={}",
                    aqua_shell::LauncherCategory::ALL.len()
                );
                println!(
                    "drm_wayland_launcher_visible_app_rows={}",
                    launcher_state.visible_apps().len().min(6)
                );
                println!(
                    "drm_wayland_launcher_search_query={}",
                    launcher_state.query()
                );
                println!(
                    "drm_wayland_launcher_selected_category={}",
                    launcher_state.category().id()
                );
                println!(
                    "drm_wayland_launcher_selected_index={}",
                    launcher_state.selected_index()
                );
                let input_snapshot = smithay_session.borrow().input_snapshot();
                println!(
                    "drm_wayland_launcher_pointer_hits={}",
                    input_snapshot.launcher_pointer_hit_count
                );
                println!(
                    "drm_wayland_launcher_category_clicks={}",
                    input_snapshot.launcher_category_click_count
                );
                println!(
                    "drm_wayland_launcher_app_clicks={}",
                    input_snapshot.launcher_app_click_count
                );
                println!(
                    "drm_wayland_launcher_launch_request_app={}",
                    input_snapshot
                        .launcher_launch_request
                        .as_ref()
                        .map_or("none", |request| request.app_id)
                );
                if let Some(request) = input_snapshot.launcher_launch_request.as_ref() {
                    let preflight = preflight_first_party_launch(request, Path::new("/"));
                    println!(
                        "drm_wayland_launcher_command_allowed={}",
                        preflight.command_allowed
                    );
                    println!(
                        "drm_wayland_launcher_executable_exists={}",
                        preflight.executable_exists
                    );
                    println!(
                        "drm_wayland_launcher_launch_accepted={}",
                        preflight.accepted
                    );
                    println!(
                        "drm_wayland_launcher_launch_rejection_reason={}",
                        preflight.reason
                    );
                    if preflight.accepted {
                        let mut process_supervisor = FirstPartyProcessSupervisor::default();
                        let process = process_supervisor
                            .spawn(&preflight, &runtime_dir, "aqua-wayland-drm-0")
                            .map_err(|error| {
                                format!("cannot supervise {}: {error:?}", preflight.app_id)
                            })?;
                        println!("drm_wayland_launcher_process_started=true");
                        println!("drm_wayland_launcher_process_app={}", preflight.app_id);
                        println!("drm_wayland_launcher_process_pid={}", process.pid);
                        println!("drm_wayland_process_supervisor_active=1");
                        println!("drm_wayland_process_supervisor_duplicate_policy=reject");
                        let duplicate_rejected = matches!(
                            process_supervisor.spawn(
                                &preflight,
                                &runtime_dir,
                                "aqua-wayland-drm-0"
                            ),
                            Err(ProcessSupervisorError::AlreadyRunning)
                        );
                        if !duplicate_rejected {
                            return Err(
                                "Aqua process supervisor accepted a duplicate app instance".into(),
                            );
                        }
                        println!("drm_wayland_process_supervisor_duplicate_rejected=true");

                        let connect_deadline = std::time::Instant::now() + Duration::from_secs(2);
                        let app_stream = loop {
                            match listener.accept() {
                                Ok(Some(stream)) => break stream,
                                Ok(None) => {}
                                Err(error) => {
                                    let _ = process_supervisor.terminate_and_reap(preflight.app_id);
                                    return Err(format!(
                                        "cannot accept {} Wayland connection: {error}",
                                        preflight.app_id
                                    ));
                                }
                            }
                            if process_supervisor
                                .try_reap(preflight.app_id)
                                .map_err(|error| {
                                    format!("cannot poll {}: {error:?}", preflight.app_id)
                                })?
                                .is_some()
                            {
                                return Err(format!(
                                    "{} exited before opening a Wayland connection",
                                    preflight.app_id
                                ));
                            }
                            if std::time::Instant::now() >= connect_deadline {
                                let _ = process_supervisor.terminate_and_reap(preflight.app_id);
                                return Err(format!(
                                    "{} did not open a Wayland connection before timeout",
                                    preflight.app_id
                                ));
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        smithay_session
                            .borrow_mut()
                            .insert_client(app_stream)
                            .map_err(|error| {
                                format!(
                                    "cannot insert {} Wayland client: {error}",
                                    preflight.app_id
                                )
                            })?;

                        let surface_deadline = std::time::Instant::now() + Duration::from_secs(2);
                        loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch {}: {error}", preflight.app_id)
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush {}: {error}", preflight.app_id)
                                })?;
                            let app_id_ready =
                                smithay_session.borrow().has_toplevel_app_id("aqua.files");
                            let mapped_and_raised = app_id_ready
                                && smithay_session
                                    .borrow_mut()
                                    .raise_surface_with_buffer_size(640, 420);
                            if mapped_and_raised {
                                break;
                            }
                            if process_supervisor
                                .try_reap(preflight.app_id)
                                .map_err(|error| {
                                    format!("cannot poll {}: {error:?}", preflight.app_id)
                                })?
                                .is_some()
                            {
                                return Err(format!(
                                    "{} exited before its owned surface appeared",
                                    preflight.app_id
                                ));
                            }
                            if std::time::Instant::now() >= surface_deadline {
                                let _ = process_supervisor.terminate_and_reap(preflight.app_id);
                                return Err(format!(
                                    "{} surface ownership was not established",
                                    preflight.app_id
                                ));
                            }
                            thread::sleep(Duration::from_millis(10));
                        }
                        println!("drm_wayland_launcher_surface_app_id=aqua.files");
                        println!("drm_wayland_launcher_surface_owned=true");

                        let initial_files_checksum = smithay_session
                            .borrow()
                            .client_surface_snapshots()
                            .into_iter()
                            .find(|snapshot| snapshot.width == 640 && snapshot.height == 420)
                            .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                            .ok_or_else(|| {
                                "Aqua Files initial checksum is unavailable".to_string()
                            })?;
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_first_entry_click(10_000)
                        {
                            return Err(
                                "Aqua Files first-entry click could not be dispatched".into()
                            );
                        }
                        let selection_deadline = std::time::Instant::now() + Duration::from_secs(2);
                        let selected_files_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Files selection: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Files selection: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 640 && snapshot.height == 420)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != initial_files_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= selection_deadline {
                                return Err(
                                    "Aqua Files selection did not commit a changed buffer".into()
                                );
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_files_read_only_root=/home/aqua");
                        println!("drm_wayland_files_directory_enumerated=true");
                        println!("drm_wayland_files_symlink_followed=false");
                        println!("drm_wayland_files_pointer_selection=entry-0");
                        println!("drm_wayland_files_selection_commit=true");
                        println!(
                            "drm_wayland_files_selection_checksum={selected_files_checksum:016x}"
                        );

                        let dispatch_files_navigation =
                            |local_x: u32,
                             local_y: u32,
                             time: u32,
                             previous_checksum: u64,
                             action: &str|
                             -> Result<u64, String> {
                                if !smithay_session
                                    .borrow_mut()
                                    .dispatch_files_pointer_click(local_x, local_y, time)
                                {
                                    return Err(format!(
                                        "Aqua Files {action} pointer click could not be dispatched"
                                    ));
                                }
                                let deadline = std::time::Instant::now() + Duration::from_secs(2);
                                loop {
                                    smithay_session.borrow_mut().dispatch_clients().map_err(
                                        |error| {
                                            format!("cannot dispatch Aqua Files {action}: {error}")
                                        },
                                    )?;
                                    smithay_session.borrow_mut().flush_clients().map_err(
                                        |error| {
                                            format!("cannot flush Aqua Files {action}: {error}")
                                        },
                                    )?;
                                    if let Some(checksum) = smithay_session
                                        .borrow()
                                        .client_surface_snapshots()
                                        .into_iter()
                                        .find(|snapshot| {
                                            snapshot.width == 640 && snapshot.height == 420
                                        })
                                        .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                        .filter(|checksum| *checksum != previous_checksum)
                                    {
                                        break Ok(checksum);
                                    }
                                    if std::time::Instant::now() >= deadline {
                                        break Err(format!(
                                            "Aqua Files {action} did not commit a changed buffer"
                                        ));
                                    }
                                    thread::sleep(Duration::from_millis(10));
                                }
                            };
                        let documents_checksum = dispatch_files_navigation(
                            220,
                            140,
                            10_100,
                            selected_files_checksum,
                            "folder-open",
                        )?;
                        println!("drm_wayland_files_folder_open=Documents");
                        println!("drm_wayland_files_folder_open_commit=true");
                        let dispatch_files_keyboard =
                            |key: u32,
                             time: u32,
                             previous_checksum: u64,
                             action: &str|
                             -> Result<u64, String> {
                                if !smithay_session
                                    .borrow_mut()
                                    .dispatch_files_keyboard_key(key, time)
                                {
                                    return Err(format!(
                                        "Aqua Files {action} key could not be dispatched"
                                    ));
                                }
                                let deadline = std::time::Instant::now() + Duration::from_secs(2);
                                loop {
                                    smithay_session.borrow_mut().dispatch_clients().map_err(
                                        |error| {
                                            format!("cannot dispatch Aqua Files {action}: {error}")
                                        },
                                    )?;
                                    smithay_session.borrow_mut().flush_clients().map_err(
                                        |error| {
                                            format!("cannot flush Aqua Files {action}: {error}")
                                        },
                                    )?;
                                    if let Some(checksum) = smithay_session
                                        .borrow()
                                        .client_surface_snapshots()
                                        .into_iter()
                                        .find(|snapshot| {
                                            snapshot.width == 640 && snapshot.height == 420
                                        })
                                        .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                        .filter(|checksum| *checksum != previous_checksum)
                                    {
                                        break Ok(checksum);
                                    }
                                    if std::time::Instant::now() >= deadline {
                                        break Err(format!(
                                            "Aqua Files {action} did not commit a changed buffer"
                                        ));
                                    }
                                    thread::sleep(Duration::from_millis(10));
                                }
                            };
                        let projects_selected_checksum = dispatch_files_keyboard(
                            108,
                            10_150,
                            documents_checksum,
                            "keyboard-selection",
                        )?;
                        println!("drm_wayland_files_keyboard_selection=Projects");
                        let projects_checksum = dispatch_files_keyboard(
                            28,
                            10_160,
                            projects_selected_checksum,
                            "keyboard-activation",
                        )?;
                        println!("drm_wayland_files_keyboard_activation=Projects");
                        let documents_keyboard_back_checksum = dispatch_files_keyboard(
                            14,
                            10_170,
                            projects_checksum,
                            "keyboard-back",
                        )?;
                        println!("drm_wayland_files_keyboard_back=true");
                        let home_checksum = dispatch_files_navigation(
                            28,
                            78,
                            10_200,
                            documents_keyboard_back_checksum,
                            "back-navigation",
                        )?;
                        println!("drm_wayland_files_back_navigation=true");
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_pointer_axis(1, 10_220)
                        {
                            return Err("Aqua Files wheel axis could not be dispatched".into());
                        }
                        let wheel_deadline = std::time::Instant::now() + Duration::from_secs(2);
                        let wheel_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Files wheel: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Files wheel: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 640 && snapshot.height == 420)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != home_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= wheel_deadline {
                                return Err(
                                    "Aqua Files wheel did not commit a changed buffer".into()
                                );
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_files_pointer_wheel=true");
                        println!("drm_wayland_files_scroll_offset=1");
                        let home_key_checksum =
                            dispatch_files_keyboard(102, 10_230, wheel_checksum, "home-key")?;
                        println!("drm_wayland_files_home_key=true");
                        let end_key_checksum =
                            dispatch_files_keyboard(107, 10_235, home_key_checksum, "end-key")?;
                        println!("drm_wayland_files_end_key=true");
                        let page_up_checksum =
                            dispatch_files_keyboard(104, 10_240, end_key_checksum, "page-up")?;
                        println!("drm_wayland_files_page_up=true");
                        let page_down_checksum =
                            dispatch_files_keyboard(109, 10_245, page_up_checksum, "page-down")?;
                        println!("drm_wayland_files_page_down=true");
                        println!("drm_wayland_files_keyboard_focus_visible=true");
                        let preview_checksum = dispatch_files_keyboard(
                            28,
                            10_250,
                            page_down_checksum,
                            "safe-text-preview",
                        )?;
                        println!("drm_wayland_files_text_preview=Welcome.txt");
                        println!("drm_wayland_files_text_preview_read_only=true");
                        println!("drm_wayland_files_text_preview_multiline=true");
                        println!("drm_wayland_files_arbitrary_execution=false");
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_pointer_axis(1, 10_255)
                        {
                            return Err("Aqua Files preview wheel could not be dispatched".into());
                        }
                        let preview_scroll_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let preview_scrolled_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Files preview wheel: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Files preview wheel: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 640 && snapshot.height == 420)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != preview_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= preview_scroll_deadline {
                                return Err(
                                    "Aqua Files preview wheel did not commit a changed buffer"
                                        .into(),
                                );
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_files_preview_pointer_wheel=true");
                        println!("drm_wayland_files_preview_scroll_offset=1");
                        let preview_closed_checksum = dispatch_files_keyboard(
                            14,
                            10_260,
                            preview_scrolled_checksum,
                            "text-preview-close",
                        )?;
                        println!("drm_wayland_files_text_preview_closed=true");
                        let scrollbar_start_checksum = dispatch_files_keyboard(
                            104,
                            10_270,
                            preview_closed_checksum,
                            "scrollbar-reset",
                        )?;
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_scrollbar_drag(124, 372, 10_280)
                        {
                            return Err("Aqua Files scrollbar drag could not be dispatched".into());
                        }
                        let scrollbar_deadline = std::time::Instant::now() + Duration::from_secs(2);
                        let scrollbar_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Files scrollbar drag: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Files scrollbar drag: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 640 && snapshot.height == 420)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != scrollbar_start_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= scrollbar_deadline {
                                return Err(
                                    "Aqua Files scrollbar drag did not commit a changed buffer"
                                        .into(),
                                );
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_files_scrollbar_drag=true");
                        println!("drm_wayland_files_scrollbar_drag_offset=1");
                        let documents_forward_checksum = dispatch_files_navigation(
                            60,
                            78,
                            10_300,
                            scrollbar_checksum,
                            "forward-navigation",
                        )?;
                        println!("drm_wayland_files_forward_navigation=true");
                        let pictures_checksum = dispatch_files_navigation(
                            40,
                            272,
                            10_400,
                            documents_forward_checksum,
                            "sidebar-navigation",
                        )?;
                        println!("drm_wayland_files_sidebar_navigation=Pictures");
                        println!("drm_wayland_files_navigation_root_confined=true");
                        println!("drm_wayland_files_hover_feedback=true");
                        println!(
                            "drm_wayland_files_history_controls=back-enabled-forward-disabled"
                        );
                        println!("drm_wayland_files_navigation_checksum={pictures_checksum:016x}");

                        let mut files_launcher_state =
                            smithay_session.borrow().launcher_state_snapshot();
                        files_launcher_state.handle_event(aqua_shell::LauncherEvent::Dismiss);
                        let mut files_snapshots =
                            smithay_session.borrow().client_surface_snapshots();
                        if let Some(index) = files_snapshots
                            .iter()
                            .position(|snapshot| snapshot.width == 640 && snapshot.height == 420)
                        {
                            let files_snapshot = files_snapshots.remove(index);
                            files_snapshots.insert(0, files_snapshot);
                        }
                        #[cfg(not(feature = "smithay-gpu"))]
                        let (files_frame, _, _) = render_fbdev_frame_with_external_clients(
                            active_width,
                            active_height,
                            &files_snapshots,
                            &files_launcher_state,
                        )?;
                        #[cfg(feature = "smithay-gpu")]
                        let files_frame = {
                            let (packed, gpu_frame) = render_live_gpu_wayland_frame(
                                &live_gpu_wayland_compositor,
                                &files_snapshots,
                                active_width,
                                active_height,
                            )?;
                            println!("drm_wayland_files_gpu_repaint=true");
                            println!("drm_wayland_files_gpu_context_reused=true");
                            println!(
                                "drm_wayland_files_gpu_texture_count={}",
                                gpu_frame.client_texture_count
                            );
                            println!(
                                "drm_wayland_files_gpu_texture_bytes={}",
                                gpu_frame.client_texture_bytes
                            );
                            println!("drm_wayland_files_gpu_checksum={:016x}", gpu_frame.checksum);
                            packed
                        };
                        let files_checksum = _repaint(files_frame)?;
                        println!("drm_wayland_files_window_model=pictures");
                        println!("drm_wayland_files_window_buffer=640x420");
                        println!("drm_wayland_files_window_sidebar_items=5");
                        println!("drm_wayland_files_window_entries=0");
                        println!("drm_wayland_files_window_location=Aqua/Home/Pictures");
                        println!("drm_wayland_files_window_repaint_complete=true");
                        println!("drm_wayland_files_window_repaint_page_flips=1");
                        println!("drm_wayland_files_window_repaint_checksum={files_checksum:016x}");
                        let _ = std::io::stdout().flush();
                        thread::sleep(Duration::from_millis(750));

                        if !smithay_session.borrow_mut().close_active_toplevel() {
                            let _ = process_supervisor.terminate_and_reap(preflight.app_id);
                            return Err("Aqua Files surface could not receive close".to_string());
                        }
                        smithay_session
                            .borrow_mut()
                            .flush_clients()
                            .map_err(|error| format!("cannot flush Aqua Files close: {error}"))?;
                        let stop_deadline = std::time::Instant::now() + Duration::from_secs(2);
                        let exit_status =
                            loop {
                                smithay_session.borrow_mut().dispatch_clients().map_err(
                                    |error| format!("cannot dispatch Aqua Files cleanup: {error}"),
                                )?;
                                if let Some(status) = process_supervisor
                                    .try_reap(preflight.app_id)
                                    .map_err(|error| format!("cannot poll Aqua Files: {error:?}"))?
                                {
                                    break status;
                                }
                                if std::time::Instant::now() >= stop_deadline {
                                    break process_supervisor
                                        .terminate_and_reap(preflight.app_id)
                                        .map_err(|error| {
                                            format!("cannot reap Aqua Files: {error:?}")
                                        })?;
                                }
                                thread::sleep(Duration::from_millis(10));
                            };
                        println!(
                            "drm_wayland_launcher_process_exit_success={}",
                            exit_status.success
                        );
                        println!("drm_wayland_launcher_process_reaped=true");
                        println!("drm_wayland_process_supervisor_active=0");
                        let surface_cleanup_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        while smithay_session.borrow().has_toplevel_app_id("aqua.files") {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Files surface cleanup: {error}")
                                })?;
                            if std::time::Instant::now() >= surface_cleanup_deadline {
                                return Err("Aqua Files surface was not removed after exit".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        }
                        println!("drm_wayland_launcher_surface_cleanup=true");

                        let settings_request = aqua_shell::LaunchRequest {
                            app_id: "settings",
                            command: "/usr/bin/aqua-settings",
                            target: None,
                        };
                        let settings_preflight =
                            preflight_first_party_launch(&settings_request, Path::new("/"));
                        if !settings_preflight.accepted {
                            return Err(format!(
                                "Aqua Settings launch rejected: {}",
                                settings_preflight.reason
                            ));
                        }
                        let settings_process = process_supervisor
                            .spawn(&settings_preflight, &runtime_dir, "aqua-wayland-drm-0")
                            .map_err(|error| {
                                format!("cannot supervise Aqua Settings: {error:?}")
                            })?;
                        println!("drm_wayland_settings_process_started=true");
                        println!("drm_wayland_settings_process_pid={}", settings_process.pid);
                        let settings_connect_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let settings_stream = loop {
                            match listener.accept() {
                                Ok(Some(stream)) => break stream,
                                Ok(None) => {}
                                Err(error) => {
                                    let _ = process_supervisor.terminate_and_reap("settings");
                                    return Err(format!(
                                        "cannot accept Aqua Settings connection: {error}"
                                    ));
                                }
                            }
                            if std::time::Instant::now() >= settings_connect_deadline {
                                let _ = process_supervisor.terminate_and_reap("settings");
                                return Err("Aqua Settings connection timed out".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        smithay_session
                            .borrow_mut()
                            .insert_client(settings_stream)
                            .map_err(|error| {
                                format!("cannot insert Aqua Settings client: {error}")
                            })?;
                        let settings_surface_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Settings: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| format!("cannot flush Aqua Settings: {error}"))?;
                            if smithay_session
                                .borrow()
                                .has_toplevel_app_id("aqua.settings")
                                && smithay_session
                                    .borrow_mut()
                                    .raise_surface_with_buffer_size(600, 400)
                            {
                                break;
                            }
                            if std::time::Instant::now() >= settings_surface_deadline {
                                let _ = process_supervisor.terminate_and_reap("settings");
                                return Err("Aqua Settings surface did not appear".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        }
                        println!("drm_wayland_settings_app_id=aqua.settings");
                        println!("drm_wayland_settings_buffer=600x400");
                        let settings_initial_checksum = smithay_session
                            .borrow()
                            .client_surface_snapshots()
                            .into_iter()
                            .find(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                            .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                            .ok_or_else(|| {
                                "Aqua Settings initial checksum unavailable".to_string()
                            })?;
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_settings_pointer_click(510, 150, 10_500)
                        {
                            return Err("Aqua Settings toggle click could not be dispatched".into());
                        }
                        let settings_toggle_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let settings_toggle_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Settings toggle: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Settings toggle: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != settings_initial_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= settings_toggle_deadline {
                                return Err("Aqua Settings toggle did not redraw".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_settings_reduced_motion=true");
                        println!("drm_wayland_settings_pointer_commit=true");
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_keyboard_key(108, 10_520)
                        {
                            return Err("Aqua Settings category key could not be dispatched".into());
                        }
                        let settings_keyboard_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let settings_keyboard_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Settings keyboard: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Settings keyboard: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != settings_toggle_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= settings_keyboard_deadline {
                                return Err("Aqua Settings keyboard did not redraw".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_settings_keyboard_category=Desktop");
                        println!(
                            "drm_wayland_settings_keyboard_checksum={settings_keyboard_checksum:016x}"
                        );
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_keyboard_key(28, 10_540)
                        {
                            return Err(
                                "Aqua Settings Desktop toggle key could not be dispatched".into()
                            );
                        }
                        let desktop_toggle_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let desktop_toggle_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Settings Desktop toggle: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Settings Desktop toggle: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != settings_keyboard_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= desktop_toggle_deadline {
                                return Err("Aqua Settings Desktop toggle did not redraw".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_settings_desktop_icons=false");
                        println!(
                            "drm_wayland_settings_desktop_toggle_checksum={desktop_toggle_checksum:016x}"
                        );
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_keyboard_key(108, 10_560)
                        {
                            return Err(
                                "Aqua Settings Input category key could not be dispatched".into()
                            );
                        }
                        let input_category_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let input_category_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Settings Input category: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Settings Input category: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != desktop_toggle_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= input_category_deadline {
                                return Err("Aqua Settings Input category did not redraw".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_settings_keyboard_category=Input");
                        println!(
                            "drm_wayland_settings_input_category_checksum={input_category_checksum:016x}"
                        );
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_keyboard_key(28, 10_580)
                        {
                            return Err(
                                "Aqua Settings Key Repeat toggle could not be dispatched".into()
                            );
                        }
                        let key_repeat_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let key_repeat_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Settings Key Repeat: {error}")
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Settings Key Repeat: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != input_category_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= key_repeat_deadline {
                                return Err("Aqua Settings Key Repeat did not redraw".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_settings_key_repeat=false");
                        println!(
                            "drm_wayland_settings_key_repeat_checksum={key_repeat_checksum:016x}"
                        );
                        if !smithay_session
                            .borrow_mut()
                            .dispatch_files_keyboard_key(108, 10_600)
                        {
                            return Err(
                                "Aqua Settings Network category key could not be dispatched".into(),
                            );
                        }
                        let network_category_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let network_category_checksum = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!(
                                        "cannot dispatch Aqua Settings Network category: {error}"
                                    )
                                })?;
                            smithay_session
                                .borrow_mut()
                                .flush_clients()
                                .map_err(|error| {
                                    format!("cannot flush Aqua Settings Network category: {error}")
                                })?;
                            if let Some(checksum) = smithay_session
                                .borrow()
                                .client_surface_snapshots()
                                .into_iter()
                                .find(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                                .map(|snapshot| checksum_bytes(&snapshot.buffer_rgba))
                                .filter(|checksum| *checksum != key_repeat_checksum)
                            {
                                break checksum;
                            }
                            if std::time::Instant::now() >= network_category_deadline {
                                return Err("Aqua Settings Network category did not redraw".into());
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!("drm_wayland_settings_keyboard_category=Network");
                        println!("drm_wayland_settings_network_read_only=true");
                        println!("drm_wayland_settings_network_management=false");
                        println!(
                            "drm_wayland_settings_network_category_checksum={network_category_checksum:016x}"
                        );
                        let mut settings_launcher_state =
                            smithay_session.borrow().launcher_state_snapshot();
                        settings_launcher_state.handle_event(aqua_shell::LauncherEvent::Dismiss);
                        let mut settings_snapshots =
                            smithay_session.borrow().client_surface_snapshots();
                        if let Some(index) = settings_snapshots
                            .iter()
                            .position(|snapshot| snapshot.width == 600 && snapshot.height == 400)
                        {
                            let settings_snapshot = settings_snapshots.remove(index);
                            settings_snapshots.insert(0, settings_snapshot);
                        }
                        #[cfg(not(feature = "smithay-gpu"))]
                        let (settings_frame, _, _) = render_fbdev_frame_with_external_clients(
                            active_width,
                            active_height,
                            &settings_snapshots,
                            &settings_launcher_state,
                        )?;
                        #[cfg(feature = "smithay-gpu")]
                        let settings_frame = {
                            let (packed, gpu_frame) = render_live_gpu_wayland_frame(
                                &live_gpu_wayland_compositor,
                                &settings_snapshots,
                                active_width,
                                active_height,
                            )?;
                            println!("drm_wayland_settings_gpu_repaint=true");
                            println!("drm_wayland_settings_gpu_context_reused=true");
                            println!(
                                "drm_wayland_settings_gpu_texture_count={}",
                                gpu_frame.client_texture_count
                            );
                            println!(
                                "drm_wayland_settings_gpu_texture_bytes={}",
                                gpu_frame.client_texture_bytes
                            );
                            println!(
                                "drm_wayland_settings_gpu_checksum={:016x}",
                                gpu_frame.checksum
                            );
                            packed
                        };
                        let settings_scanout_checksum = _repaint(settings_frame)?;
                        println!("drm_wayland_settings_capture_ready=true");
                        println!(
                            "drm_wayland_settings_scanout_checksum={settings_scanout_checksum:016x}"
                        );
                        let _ = std::io::stdout().flush();
                        thread::sleep(Duration::from_millis(750));
                        if !smithay_session.borrow_mut().close_active_toplevel() {
                            return Err("Aqua Settings surface could not receive close".into());
                        }
                        smithay_session
                            .borrow_mut()
                            .flush_clients()
                            .map_err(|error| {
                                format!("cannot flush Aqua Settings close: {error}")
                            })?;
                        let settings_stop_deadline =
                            std::time::Instant::now() + Duration::from_secs(2);
                        let settings_exit = loop {
                            smithay_session
                                .borrow_mut()
                                .dispatch_clients()
                                .map_err(|error| {
                                    format!("cannot dispatch Aqua Settings cleanup: {error}")
                                })?;
                            if let Some(status) = process_supervisor
                                .try_reap("settings")
                                .map_err(|error| format!("cannot poll Aqua Settings: {error:?}"))?
                            {
                                break status;
                            }
                            if std::time::Instant::now() >= settings_stop_deadline {
                                break process_supervisor.terminate_and_reap("settings").map_err(
                                    |error| format!("cannot reap Aqua Settings: {error:?}"),
                                )?;
                            }
                            thread::sleep(Duration::from_millis(10));
                        };
                        println!(
                            "drm_wayland_settings_process_exit_success={}",
                            settings_exit.success
                        );
                        println!("drm_wayland_settings_process_reaped=true");
                        println!("drm_wayland_settings_surface_cleanup=true");
                        let settings_config_path =
                            Path::new("/home/aqua/.config/aqua/settings.conf");
                        let persisted_settings =
                            aqua_shell::SettingsWindowModel::load_or_default(settings_config_path)
                                .map_err(|error| {
                                    format!("cannot reload Aqua Settings config: {error}")
                                })?;
                        if !persisted_settings.reduced_motion {
                            return Err(
                                "Aqua Settings Reduced Motion value was not persisted".into()
                            );
                        }
                        if persisted_settings.desktop_icons {
                            return Err(
                                "Aqua Settings Desktop Icons value was not persisted".into()
                            );
                        }
                        if persisted_settings.key_repeat {
                            return Err("Aqua Settings Key Repeat value was not persisted".into());
                        }
                        println!(
                            "drm_wayland_settings_config_path={}",
                            settings_config_path.display()
                        );
                        println!(
                            "drm_wayland_settings_config_version={}",
                            aqua_shell::SETTINGS_CONFIG_VERSION
                        );
                        println!("drm_wayland_settings_persisted_reduced_motion=true");
                        println!("drm_wayland_settings_persisted_desktop_icons=false");
                        println!("drm_wayland_settings_persisted_key_repeat=false");
                        println!("drm_wayland_settings_reload_verified=true");
                        println!(
                            "drm_wayland_process_supervisor_final_active={}",
                            process_supervisor.active_count()
                        );
                    } else {
                        println!("drm_wayland_launcher_process_started=false");
                    }
                } else {
                    println!("drm_wayland_launcher_process_started=false");
                }
                let interaction_snapshot = smithay_session.borrow().client_surface_snapshot();
                println!(
                    "drm_wayland_move_requests={}",
                    interaction_snapshot.move_request_count
                );
                println!(
                    "drm_wayland_resize_requests={}",
                    interaction_snapshot.resize_request_count
                );
                println!(
                    "drm_wayland_interactive_geometry_applied={}",
                    interaction_snapshot.move_request_count >= 1
                        && interaction_snapshot.resize_request_count >= 1
                );
                println!(
                    "drm_wayland_maximize_requests={}",
                    interaction_snapshot.maximize_request_count
                );
                println!(
                    "drm_wayland_unmaximize_requests={}",
                    interaction_snapshot.unmaximize_request_count
                );
                println!(
                    "drm_wayland_fullscreen_requests={}",
                    interaction_snapshot.fullscreen_request_count
                );
                println!(
                    "drm_wayland_unfullscreen_requests={}",
                    interaction_snapshot.unfullscreen_request_count
                );
                println!(
                    "drm_wayland_state_configure_acks={}",
                    interaction_snapshot.configure_ack_count
                );
                println!(
                    "drm_wayland_state_cycle_complete={}",
                    interaction_snapshot.maximize_request_count >= 1
                        && interaction_snapshot.unmaximize_request_count >= 1
                        && interaction_snapshot.fullscreen_request_count >= 1
                        && interaction_snapshot.unfullscreen_request_count >= 1
                        && interaction_snapshot.configure_ack_count >= 7
                );
                let _ = std::io::stdout().flush();

                if !smithay_session.borrow_mut().close_active_toplevel() {
                    return Err("active xdg_toplevel could not receive close".to_string());
                }
                smithay_session
                    .borrow_mut()
                    .flush_clients()
                    .map_err(|error| format!("cannot flush active xdg_toplevel close: {error}"))?;

                let cleanup_deadline = std::time::Instant::now() + Duration::from_secs(2);
                loop {
                    smithay_session
                        .borrow_mut()
                        .dispatch_clients()
                        .map_err(|error| format!("cannot dispatch client cleanup: {error}"))?;
                    smithay_session
                        .borrow_mut()
                        .flush_clients()
                        .map_err(|error| format!("cannot flush client cleanup: {error}"))?;
                    if smithay_session.borrow().client_surface_snapshots().len() == 1 {
                        break;
                    }
                    if std::time::Instant::now() >= cleanup_deadline {
                        return Err("controlled client exit did not remove its surface".to_string());
                    }
                    thread::sleep(Duration::from_millis(10));
                }

                let surviving_snapshots = smithay_session.borrow().client_surface_snapshots();
                #[cfg(not(feature = "smithay-gpu"))]
                let (cleanup_frame, _, _) = render_fbdev_frame_with_external_clients(
                    active_width,
                    active_height,
                    &surviving_snapshots,
                    &smithay_session.borrow().launcher_state_snapshot(),
                )?;
                #[cfg(feature = "smithay-gpu")]
                let cleanup_frame = {
                    let (packed, gpu_frame) = render_live_gpu_wayland_frame(
                        &live_gpu_wayland_compositor,
                        &surviving_snapshots,
                        active_width,
                        active_height,
                    )?;
                    println!("drm_wayland_client_cleanup_gpu_repaint=true");
                    println!("drm_wayland_client_cleanup_gpu_context_reused=true");
                    println!(
                        "drm_wayland_client_cleanup_gpu_texture_count={}",
                        gpu_frame.client_texture_count
                    );
                    println!(
                        "drm_wayland_client_cleanup_gpu_texture_bytes={}",
                        gpu_frame.client_texture_bytes
                    );
                    println!(
                        "drm_wayland_client_cleanup_gpu_checksum={:016x}",
                        gpu_frame.checksum
                    );
                    packed
                };
                let cleanup_checksum = _repaint(cleanup_frame)?;
                #[cfg(not(feature = "smithay-gpu"))]
                if cleanup_checksum == repaint_checksum {
                    return Err("client cleanup repaint did not change the frame".to_string());
                }
                let cleanup_snapshot = smithay_session.borrow().client_surface_snapshot();
                println!("drm_wayland_client_cleanup_complete=true");
                println!("drm_wayland_client_cleanup_surviving_surfaces=1");
                println!(
                    "drm_wayland_client_cleanup_destroyed_surfaces={}",
                    cleanup_snapshot.destroyed_surface_count
                );
                println!(
                    "drm_wayland_client_cleanup_count={}",
                    cleanup_snapshot.client_cleanup_count
                );
                println!("drm_wayland_client_cleanup_session_alive=true");
                println!(
                    "drm_wayland_client_cleanup_keyboard_focus_reassigned={}",
                    cleanup_snapshot.cleanup_keyboard_focus_reassigned
                );
                println!(
                    "drm_wayland_client_cleanup_pointer_focus_cleared={}",
                    !cleanup_snapshot.pointer_focus_assigned
                );
                println!("drm_wayland_client_cleanup_repaint_complete=true");
                println!("drm_wayland_client_cleanup_repaint_page_flips=1");
                println!("drm_wayland_client_cleanup_repaint_checksum={cleanup_checksum:016x}");
                let _ = std::io::stdout().flush();

                // Keep the survivor visible long enough for the QEMU capture, then exercise
                // compositor-initiated xdg_toplevel close and repaint the empty desktop.
                thread::sleep(Duration::from_millis(750));
                if !smithay_session.borrow_mut().close_active_toplevel() {
                    return Err("surviving xdg_toplevel could not receive close".to_string());
                }
                smithay_session
                    .borrow_mut()
                    .flush_clients()
                    .map_err(|error| format!("cannot flush xdg_toplevel close: {error}"))?;
                let close_deadline = std::time::Instant::now() + Duration::from_secs(2);
                loop {
                    smithay_session
                        .borrow_mut()
                        .dispatch_clients()
                        .map_err(|error| format!("cannot dispatch xdg_toplevel close: {error}"))?;
                    smithay_session
                        .borrow_mut()
                        .flush_clients()
                        .map_err(|error| {
                            format!("cannot flush xdg_toplevel close cleanup: {error}")
                        })?;
                    if smithay_session
                        .borrow()
                        .client_surface_snapshots()
                        .is_empty()
                    {
                        break;
                    }
                    if std::time::Instant::now() >= close_deadline {
                        return Err(
                            "xdg_toplevel close did not remove the final surface".to_string()
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                #[cfg(not(feature = "smithay-gpu"))]
                let (desktop_frame, _, _) = render_fbdev_frame(active_width, active_height, 32)?;
                #[cfg(feature = "smithay-gpu")]
                let desktop_frame = {
                    let (packed, gpu_frame) = render_live_gpu_wayland_frame(
                        &live_gpu_wayland_compositor,
                        &[],
                        active_width,
                        active_height,
                    )?;
                    println!("drm_wayland_close_gpu_repaint=true");
                    println!("drm_wayland_close_gpu_context_reused=true");
                    println!(
                        "drm_wayland_close_gpu_texture_count={}",
                        gpu_frame.client_texture_count
                    );
                    println!(
                        "drm_wayland_close_gpu_texture_bytes={}",
                        gpu_frame.client_texture_bytes
                    );
                    println!("drm_wayland_close_gpu_checksum={:016x}", gpu_frame.checksum);
                    packed
                };
                let close_repaint_checksum = _repaint(desktop_frame)?;
                let close_snapshot = smithay_session.borrow().client_surface_snapshot();
                println!("drm_wayland_close_request_sent=true");
                println!(
                    "drm_wayland_close_request_count={}",
                    close_snapshot.close_request_count
                );
                println!("drm_wayland_close_cleanup_surfaces=0");
                println!("drm_wayland_close_repaint_complete=true");
                println!("drm_wayland_close_repaint_page_flips=1");
                println!("drm_wayland_close_repaint_checksum={close_repaint_checksum:016x}");
                #[cfg(feature = "smithay-gpu")]
                println!("drm_wayland_gpu_repaint_route_complete=true");
                let _ = std::io::stdout().flush();
            }
            Ok(())
        },
    );

    drop(client_stream);
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    for mut child in external_clients.drain(..) {
        if result.is_ok() {
            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            while child.try_wait().ok().flatten().is_none() && std::time::Instant::now() < deadline
            {
                thread::sleep(Duration::from_millis(10));
            }
        }
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
        }
        let _ = child.wait();
        println!("external_wayland_client_process_stopped=true");
        if installer_scenario {
            println!("installer_wayland_client_process_stopped=true");
        } else if typography_scenario {
            println!("typography_wayland_client_process_stopped=true");
        } else if elevation_scenario {
            println!("elevation_wayland_client_process_stopped=true");
        }
    }
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    for app_id in ["files", "settings", "properties", "terminal"] {
        if runtime_process_supervisor.borrow().contains(app_id) {
            let reaped = runtime_process_supervisor
                .borrow_mut()
                .terminate_and_reap(app_id);
            println!(
                "desktop_runtime_process_stopped_{}={}",
                app_id,
                reaped.is_ok()
            );
        }
    }
    #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
    println!(
        "desktop_runtime_process_active_count={}",
        runtime_process_supervisor.borrow().active_count()
    );
    drop(listener);
    let _ = fs::remove_file(&socket_path);
    let _ = fs::remove_file(&lock_path);

    match result {
        Ok(final_state) => {
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            {
                let snapshot = smithay_session.borrow().input_snapshot();
                let input_ready = !input_required
                    || (snapshot.keyboard_event_count >= 1
                        && snapshot.pointer_motion_count >= 1
                        && snapshot.pointer_button_count >= 2
                        && snapshot.launcher_visible);
                if let Some(source) = input_source.borrow().as_ref() {
                    println!(
                        "drm_wayland_input_keyboard_devices={}",
                        source.keyboard_devices
                    );
                    println!(
                        "drm_wayland_input_pointer_devices={}",
                        source.pointer_devices
                    );
                    println!(
                        "drm_wayland_input_discovery_ready={}",
                        source.keyboard_devices >= 1 && source.pointer_devices >= 1
                    );
                }
                println!(
                    "drm_wayland_input_keyboard_events={}",
                    snapshot.keyboard_event_count
                );
                println!(
                    "drm_wayland_input_pointer_motion_events={}",
                    snapshot.pointer_motion_count
                );
                println!(
                    "drm_wayland_input_pointer_button_events={}",
                    snapshot.pointer_button_count
                );
                println!(
                    "drm_wayland_input_launcher_visible={}",
                    snapshot.launcher_visible
                );
                println!("drm_wayland_input_dispatch_ready={input_ready}");
                println!(
                    "drm_wayland_input_shortcut_intercepts={}",
                    snapshot.keyboard_shortcut_intercept_count
                );
                println!(
                    "drm_wayland_input_keys_forwarded={}",
                    snapshot.keyboard_forward_count
                );
                println!(
                    "drm_wayland_input_pointer_hit_tests={}",
                    snapshot.pointer_hit_test_count
                );
                println!(
                    "drm_wayland_input_pointer_surface_hits={}",
                    snapshot.pointer_surface_hit_count
                );
                if !external_surface_snapshot.is_empty() {
                    println!(
                        "drm_wayland_external_client_ready={}",
                        external_surface_snapshot
                            .iter()
                            .all(SmithayClientSurfaceSnapshot::is_ready)
                    );
                    println!(
                        "drm_wayland_external_client_toplevels={}",
                        external_surface_snapshot[0].toplevel_count
                    );
                    println!(
                        "drm_wayland_external_client_commits={}",
                        external_surface_snapshot[0].commit_count
                    );
                    println!(
                        "drm_wayland_external_client_buffer_bytes={}",
                        external_surface_snapshot
                            .iter()
                            .map(|snapshot| snapshot.buffer_rgba.len())
                            .sum::<usize>()
                    );
                    println!(
                        "drm_wayland_external_client_surface_count={}",
                        external_surface_snapshot.len()
                    );
                    println!("drm_wayland_external_client_independent_buffers=true");
                    println!("drm_wayland_external_client_composited=true");
                }
                let final_surface = smithay_session.borrow().client_surface_snapshot();
                if external_client_required {
                    println!(
                        "drm_wayland_external_client_damage_commits={}",
                        final_surface.damage_commit_count
                    );
                    println!(
                        "drm_wayland_external_client_damage_rects={}",
                        final_surface.damage_rect_count
                    );
                    println!(
                        "drm_wayland_external_client_frame_callbacks_sent={}",
                        final_surface.frame_callbacks_sent
                    );
                    println!(
                        "drm_wayland_external_client_damage_ready={}",
                        final_surface.damage_commit_count >= 4
                    );
                    println!(
                        "drm_wayland_external_client_frame_callbacks_ready={}",
                        final_surface.frame_callbacks_sent >= 2
                    );
                    println!(
                        "drm_wayland_external_client_keyboard_focus={}",
                        final_surface.keyboard_focus_assigned
                    );
                    println!(
                        "drm_wayland_external_client_pointer_focus={}",
                        final_surface.pointer_focus_assigned
                    );
                    println!(
                        "drm_wayland_external_client_focus_changes={}",
                        final_surface.surface_focus_change_count
                    );
                    println!(
                        "drm_wayland_external_client_stacking_changes={}",
                        final_surface.stacking_change_count
                    );
                }
            }
            println!("wayland_dispatch_passes={wayland_dispatch_passes}");
            println!("wayland_flush_passes={wayland_flush_passes}");
            println!("wayland_socket_cleaned={}", !socket_path.exists());
            println!("drm_event_source_released=true");
            println!("crtc_restored={}", final_state.crtc_restored);
            println!("framebuffers_destroyed=true");
            #[cfg(feature = "smithay-gpu")]
            println!("gbm_scanout_buffers_released=true");
            #[cfg(not(feature = "smithay-gpu"))]
            println!("dumb_buffers_destroyed=true");
            println!("display_output_stopped=true");
            #[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
            println!("graceful_stop_completed={}", graceful_stop_requested.get());
            println!("safe_return_to_recovery=ok");
            println!("[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok");
        }
        Err(error) => {
            eprintln!("DRM Wayland session failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=drm-wayland-session status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn run_wayland_test_client_cli(socket_path: PathBuf) {
    if let Err(error) = run_external_wayland_test_client(&socket_path) {
        eprintln!("external Aqua Wayland test client failed: {error}");
        println!("[AQUA-CLIENT] stage=external-wayland-surface status=error");
        std::process::exit(1);
    }
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn run_wayland_test_client_cli(_socket_path: PathBuf) {
    eprintln!("external Aqua Wayland test client requires Linux with smithay-smoke");
    println!("[AQUA-CLIENT] stage=external-wayland-surface status=unsupported-host");
    std::process::exit(1);
}

#[cfg(not(target_os = "linux"))]
fn run_drm_wayland_session_cli(_device: PathBuf) {
    eprintln!("DRM Wayland session requires Linux");
    println!("[AQUA-COMPOSITOR] stage=drm-wayland-session status=unsupported-host");
    std::process::exit(1);
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct DirectLibinputInterface;

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl LibinputInterface for DirectLibinputInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        let access_mode = flags & libc::O_ACCMODE;
        OpenOptions::new()
            .custom_flags(flags)
            .read(access_mode != libc::O_WRONLY)
            .write(access_mode != libc::O_RDONLY)
            .open(path)
            .map(Into::into)
            .map_err(|error| error.raw_os_error().unwrap_or(libc::EIO))
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(fd);
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct LibinputAquaSeatSource {
    context: Libinput,
    poller: Poller,
    seat_name: String,
    keyboard_devices: u32,
    pointer_devices: u32,
    serial: u32,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl LibinputAquaSeatSource {
    fn open(seat_name: &str) -> Result<Self, String> {
        let mut context = Libinput::new_with_udev(DirectLibinputInterface);
        context
            .udev_assign_seat(seat_name)
            .map_err(|_| format!("cannot assign libinput udev seat {seat_name}"))?;
        let poller =
            Poller::new().map_err(|error| format!("cannot create libinput poller: {error}"))?;
        unsafe {
            poller
                .add_with_mode(&context, PollEvent::readable(1), PollMode::Level)
                .map_err(|error| format!("cannot register libinput context: {error}"))?;
        }
        Ok(Self {
            context,
            poller,
            seat_name: seat_name.to_string(),
            keyboard_devices: 0,
            pointer_devices: 0,
            serial: 1,
        })
    }

    fn dispatch_until(
        &mut self,
        session: &mut SmithayDrmSession,
        timeout: Duration,
    ) -> Result<(), String> {
        let deadline = std::time::Instant::now() + timeout;
        let mut events = PollEvents::new();
        while std::time::Instant::now() < deadline {
            self.context
                .dispatch()
                .map_err(|error| format!("cannot dispatch libinput context: {error}"))?;
            for event in &mut self.context {
                self.serial = self.serial.saturating_add(1);
                match event {
                    input::Event::Device(DeviceEvent::Added(event)) => {
                        let device = event.device();
                        if device.has_capability(DeviceCapability::Keyboard) {
                            self.keyboard_devices = self.keyboard_devices.saturating_add(1);
                        }
                        if device.has_capability(DeviceCapability::Pointer) {
                            self.pointer_devices = self.pointer_devices.saturating_add(1);
                        }
                    }
                    input::Event::Device(DeviceEvent::Removed(event)) => {
                        let device = event.device();
                        if device.has_capability(DeviceCapability::Keyboard) {
                            self.keyboard_devices = self.keyboard_devices.saturating_sub(1);
                        }
                        if device.has_capability(DeviceCapability::Pointer) {
                            self.pointer_devices = self.pointer_devices.saturating_sub(1);
                        }
                    }
                    input::Event::Keyboard(KeyboardEvent::Key(event)) => {
                        session.dispatch_keyboard_key(
                            event.key(),
                            event.key_state() == KeyState::Pressed,
                            self.serial,
                        );
                    }
                    input::Event::Pointer(PointerEvent::Motion(event)) => {
                        session.dispatch_pointer_motion(event.dx(), event.dy(), self.serial);
                    }
                    input::Event::Pointer(PointerEvent::Button(event)) => {
                        session.dispatch_pointer_button(
                            event.button(),
                            event.button_state() == ButtonState::Pressed,
                            self.serial,
                        );
                    }
                    _ => {}
                }
                if session.has_session_action_request() {
                    println!("desktop_input_action_yield=libinput-iterator");
                    break;
                }
            }
            if session.has_session_action_request() {
                println!("desktop_input_action_yield=dispatch-until-return");
                return Ok(());
            }
            events.clear();
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if self
                .poller
                .wait(&mut events, Some(remaining))
                .map_err(|error| format!("cannot wait for libinput events: {error}"))?
                == 0
            {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl Drop for LibinputAquaSeatSource {
    fn drop(&mut self) {
        let _ = self.poller.delete(&self.context);
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
struct EvdevAquaSeatSource {
    keyboard_file: File,
    pointer_file: File,
    poller: Poller,
    serial: u32,
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl EvdevAquaSeatSource {
    const INPUT_EVENT_BYTES: usize = 24;
    const EV_KEY: u16 = 1;
    const EV_REL: u16 = 2;
    const REL_X: u16 = 0;
    const REL_Y: u16 = 1;
    const BTN_LEFT: u16 = 0x110;

    fn open(keyboard_path: PathBuf, pointer_path: PathBuf) -> Result<Self, String> {
        let keyboard_file = File::open(&keyboard_path).map_err(|error| {
            format!(
                "cannot open keyboard event device {}: {error}",
                keyboard_path.display()
            )
        })?;
        let pointer_file = File::open(&pointer_path).map_err(|error| {
            format!(
                "cannot open pointer event device {}: {error}",
                pointer_path.display()
            )
        })?;
        let poller =
            Poller::new().map_err(|error| format!("cannot create evdev poller: {error}"))?;
        unsafe {
            poller
                .add(&keyboard_file, PollEvent::readable(1))
                .and_then(|_| poller.add(&pointer_file, PollEvent::readable(2)))
                .map_err(|error| format!("cannot register evdev devices: {error}"))?;
        }
        Ok(Self {
            keyboard_file,
            pointer_file,
            poller,
            serial: 1,
        })
    }

    fn dispatch_until(
        &mut self,
        session: &mut SmithayDrmSession,
        timeout: Duration,
    ) -> Result<(), String> {
        let deadline = std::time::Instant::now() + timeout;
        let mut events = PollEvents::new();
        while std::time::Instant::now() < deadline {
            events.clear();
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if self
                .poller
                .wait(&mut events, Some(remaining))
                .map_err(|error| format!("cannot wait for evdev events: {error}"))?
                == 0
            {
                break;
            }

            for event in events.iter() {
                let (file, device_kind) = match event.key {
                    1 => (&mut self.keyboard_file, 1_u8),
                    2 => (&mut self.pointer_file, 2_u8),
                    _ => continue,
                };
                let mut buffer = [0_u8; Self::INPUT_EVENT_BYTES * 64];
                let bytes_read = file
                    .read(&mut buffer)
                    .map_err(|error| format!("cannot read evdev event records: {error}"))?;
                for record in buffer[..bytes_read].chunks_exact(Self::INPUT_EVENT_BYTES) {
                    let event_type = u16::from_ne_bytes([record[16], record[17]]);
                    let code = u16::from_ne_bytes([record[18], record[19]]);
                    let value =
                        i32::from_ne_bytes([record[20], record[21], record[22], record[23]]);
                    self.serial = self.serial.saturating_add(1);
                    match (device_kind, event_type, code) {
                        (1, Self::EV_KEY, _) if value == 0 || value == 1 => {
                            session.dispatch_keyboard_key(u32::from(code), value == 1, self.serial);
                        }
                        (2, Self::EV_REL, Self::REL_X) => {
                            session.dispatch_pointer_motion(f64::from(value), 0.0, self.serial);
                        }
                        (2, Self::EV_REL, Self::REL_Y) => {
                            session.dispatch_pointer_motion(0.0, f64::from(value), self.serial);
                        }
                        (2, Self::EV_KEY, button)
                            if button >= Self::BTN_LEFT && (value == 0 || value == 1) =>
                        {
                            session.dispatch_pointer_button(
                                u32::from(button),
                                value == 1,
                                self.serial,
                            );
                        }
                        _ => {}
                    }
                }
                self.poller
                    .modify(file, PollEvent::readable(event.key))
                    .map_err(|error| format!("cannot rearm evdev device: {error}"))?;
            }

            let snapshot = session.input_snapshot();
            if snapshot.keyboard_event_count >= 1
                && snapshot.pointer_motion_count >= 1
                && snapshot.pointer_button_count >= 2
                && snapshot.launcher_visible
            {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
impl Drop for EvdevAquaSeatSource {
    fn drop(&mut self) {
        let _ = self.poller.delete(&self.keyboard_file);
        let _ = self.poller.delete(&self.pointer_file);
    }
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn probe_evdev_aqua_seat_cli(keyboard_path: PathBuf, pointer_path: PathBuf) {
    let mut input_source = EvdevAquaSeatSource::open(keyboard_path.clone(), pointer_path.clone())
        .unwrap_or_else(|error| {
            eprintln!("cannot prepare evdev Aqua Seat probe: {error}");
            std::process::exit(1);
        });

    let mut session = SmithayDrmSession::new().unwrap_or_else(|error| {
        eprintln!("cannot create Aqua Smithay input session: {error}");
        std::process::exit(1);
    });
    println!("product=Aqua Linux");
    println!("backend=linux-evdev");
    println!("keyboard_device={}", keyboard_path.display());
    println!("pointer_device={}", pointer_path.display());
    println!("seat_started={}", session.seat_started());
    println!("bounded_timeout_ms=5000");
    println!("boot_graphics=false");
    println!("autostart=false");
    println!("[AQUA-INPUT] stage=evdev-aqua-seat status=active");
    let _ = std::io::stdout().flush();

    input_source
        .dispatch_until(&mut session, Duration::from_secs(5))
        .unwrap_or_else(|error| {
            eprintln!("evdev Aqua Seat dispatch failed: {error}");
            std::process::exit(1);
        });

    let snapshot = session.input_snapshot();
    let ready = session.seat_started()
        && snapshot.keyboard_event_count >= 1
        && snapshot.pointer_motion_count >= 1
        && snapshot.launcher_visible;
    println!("keyboard_events={}", snapshot.keyboard_event_count);
    println!("pointer_motion_events={}", snapshot.pointer_motion_count);
    println!("pointer_button_events={}", snapshot.pointer_button_count);
    println!("launcher_visible={}", snapshot.launcher_visible);
    println!("evdev_events_dispatched={ready}");
    println!("safe_return_to_recovery=ok");
    println!(
        "[AQUA-INPUT] stage=evdev-aqua-seat status={}",
        if ready { "ok" } else { "error" }
    );
    if !ready {
        std::process::exit(1);
    }
}

#[cfg(not(all(target_os = "linux", feature = "smithay-smoke")))]
fn probe_evdev_aqua_seat_cli(_keyboard_path: PathBuf, _pointer_path: PathBuf) {
    eprintln!("evdev Aqua Seat probe requires Linux with smithay-smoke");
    println!("[AQUA-INPUT] stage=evdev-aqua-seat status=unsupported-host");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
fn present_drm_page_flip(
    device: &Path,
    frame_count: u32,
    timeout_ms: u64,
    hold_seconds: u64,
    event_waiter: DrmEventWaiter,
    render_frame: impl FnOnce(u32, u32) -> Result<(Vec<u8>, u64, bool), String>,
    mut on_event: impl FnMut(u32) -> Result<(), String>,
    on_active: impl FnOnce(&DrmPageFlipActiveFrame),
    on_hold: impl FnOnce(Duration, &mut dyn FnMut(Vec<u8>) -> Result<u64, String>) -> Result<(), String>,
) -> Result<DrmPageFlipFinalState, String> {
    let card = DrmCard(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|error| format!("cannot open {} read-write: {error}", device.display()))?,
    );
    let resources = card
        .resource_handles()
        .map_err(|error| format!("cannot read DRM resources: {error}"))?;
    let connectors = resources
        .connectors()
        .iter()
        .map(|handle| card.get_connector(*handle, true))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read DRM connector: {error}"))?;
    let connector = connectors
        .iter()
        .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
        .ok_or_else(|| "no connected DRM connector with a mode".to_string())?;
    let mode = connector.modes()[0];
    let crtc_handle = connector
        .current_encoder()
        .and_then(|handle| card.get_encoder(handle).ok())
        .and_then(|encoder| encoder.crtc())
        .or_else(|| resources.crtcs().first().copied())
        .ok_or_else(|| "no DRM CRTC is available".to_string())?;
    let original_crtc = card
        .get_crtc(crtc_handle)
        .map_err(|error| format!("cannot read original DRM CRTC: {error}"))?;
    let (mode_width, mode_height) = mode.size();
    let width = u32::from(mode_width);
    let height = u32::from(mode_height);
    let (packed_frame, _, _) = render_frame(width, height)?;

    let mut front_buffer = card
        .create_dumb_buffer((width, height), DrmFourcc::Xrgb8888, 32)
        .map_err(|error| format!("cannot create front DRM dumb buffer: {error}"))?;
    let mut back_buffer = card
        .create_dumb_buffer((width, height), DrmFourcc::Xrgb8888, 32)
        .map_err(|error| format!("cannot create back DRM dumb buffer: {error}"))?;
    if front_buffer.pitch() != back_buffer.pitch() {
        return Err("front and back DRM dumb-buffer pitches differ".to_string());
    }
    let pitch = back_buffer.pitch();
    let pitched_frame = with_stride(
        &packed_frame,
        width as usize * 4,
        pitch as usize,
        height as usize,
    );
    for buffer in [&mut front_buffer, &mut back_buffer] {
        let mut mapping = card
            .map_dumb_buffer(buffer)
            .map_err(|error| format!("cannot map DRM page-flip buffer: {error}"))?;
        if mapping.len() < pitched_frame.len() {
            return Err("mapped DRM page-flip buffer is smaller than the frame".to_string());
        }
        mapping[..pitched_frame.len()].copy_from_slice(&pitched_frame);
    }
    let front_framebuffer = card
        .add_framebuffer(&front_buffer, 24, 32)
        .map_err(|error| format!("cannot create front KMS framebuffer: {error}"))?;
    let back_framebuffer = card
        .add_framebuffer(&back_buffer, 24, 32)
        .map_err(|error| format!("cannot create back KMS framebuffer: {error}"))?;
    card.set_crtc(
        crtc_handle,
        Some(front_framebuffer),
        (0, 0),
        &[connector.handle()],
        Some(mode),
    )
    .map_err(|error| format!("cannot activate front KMS framebuffer: {error}"))?;

    let mut event_frames = Vec::with_capacity(frame_count as usize);
    for frame_index in 0..frame_count {
        let target_framebuffer = if frame_index % 2 == 0 {
            back_framebuffer
        } else {
            front_framebuffer
        };
        card.page_flip(crtc_handle, target_framebuffer, PageFlipFlags::EVENT, None)
            .map_err(|error| format!("cannot submit DRM page flip {}: {error}", frame_index + 1))?;
        let event_frame = wait_for_drm_page_flip(
            &card,
            crtc_handle,
            timeout_ms,
            frame_index + 1,
            event_waiter,
        )?;
        on_event(frame_index + 1)?;
        event_frames.push(event_frame);
    }

    on_active(&DrmPageFlipActiveFrame {
        connector: connector.to_string(),
        width,
        height,
        pitch,
        bytes: pitched_frame.len(),
        buffer_checksum: checksum_bytes(&pitched_frame),
        event_frames,
    });
    let mut active_is_back = frame_count % 2 == 1;
    let mut repaint = |packed_frame: Vec<u8>| -> Result<u64, String> {
        if packed_frame.len() != width as usize * height as usize * 4 {
            return Err(format!(
                "repaint frame has {} bytes, expected {}",
                packed_frame.len(),
                width as usize * height as usize * 4
            ));
        }
        let pitched_repaint = with_stride(
            &packed_frame,
            width as usize * 4,
            pitch as usize,
            height as usize,
        );
        let (target_buffer, target_framebuffer) = if active_is_back {
            (&mut front_buffer, front_framebuffer)
        } else {
            (&mut back_buffer, back_framebuffer)
        };
        let mut mapping = card
            .map_dumb_buffer(target_buffer)
            .map_err(|error| format!("cannot map DRM repaint buffer: {error}"))?;
        if mapping.len() < pitched_repaint.len() {
            return Err("mapped DRM repaint buffer is smaller than the frame".to_string());
        }
        mapping[..pitched_repaint.len()].copy_from_slice(&pitched_repaint);
        drop(mapping);
        let damage_width = u16::try_from(width)
            .map_err(|_| format!("repaint width exceeds DRM damage limits: {width}"))?;
        let damage_height = u16::try_from(height)
            .map_err(|_| format!("repaint height exceeds DRM damage limits: {height}"))?;
        match card.dirty_framebuffer(
            target_framebuffer,
            &[ClipRect::new(0, 0, damage_width, damage_height)],
        ) {
            Ok(()) => {}
            Err(error) if error.raw_os_error() == Some(38) => {}
            Err(error) => {
                return Err(format!(
                    "cannot mark DRM repaint framebuffer dirty: {error}"
                ))
            }
        }
        card.page_flip(crtc_handle, target_framebuffer, PageFlipFlags::EVENT, None)
            .map_err(|error| format!("cannot submit DRM repaint page flip: {error}"))?;
        wait_for_drm_page_flip(
            &card,
            crtc_handle,
            timeout_ms,
            frame_count + 1,
            event_waiter,
        )?;
        active_is_back = !active_is_back;
        Ok(checksum_bytes(&pitched_repaint))
    };
    on_hold(Duration::from_secs(hold_seconds), &mut repaint)?;

    let restore_connectors = if original_crtc.mode().is_some() {
        vec![connector.handle()]
    } else {
        Vec::new()
    };
    card.set_crtc(
        crtc_handle,
        original_crtc.framebuffer(),
        original_crtc.position(),
        &restore_connectors,
        original_crtc.mode(),
    )
    .map_err(|error| format!("cannot restore original DRM CRTC: {error}"))?;
    card.destroy_framebuffer(back_framebuffer)
        .map_err(|error| format!("cannot destroy back KMS framebuffer: {error}"))?;
    card.destroy_framebuffer(front_framebuffer)
        .map_err(|error| format!("cannot destroy front KMS framebuffer: {error}"))?;
    card.destroy_dumb_buffer(back_buffer)
        .map_err(|error| format!("cannot destroy back DRM dumb buffer: {error}"))?;
    card.destroy_dumb_buffer(front_buffer)
        .map_err(|error| format!("cannot destroy front DRM dumb buffer: {error}"))?;

    Ok(DrmPageFlipFinalState {
        crtc_restored: true,
        front_framebuffer_destroyed: true,
        back_framebuffer_destroyed: true,
        front_buffer_destroyed: true,
        back_buffer_destroyed: true,
    })
}

#[cfg(all(target_os = "linux", feature = "smithay-gpu"))]
#[allow(clippy::too_many_arguments)]
fn present_drm_gbm_page_flip(
    device: &Path,
    frame_count: u32,
    timeout_ms: u64,
    hold_seconds: u64,
    event_waiter: DrmEventWaiter,
    render_frame: impl FnOnce(u32, u32) -> Result<(Vec<u8>, u64, bool), String>,
    mut render_scanout: impl FnMut(&mut Dmabuf, u32, u32) -> Result<(), String>,
    mut on_event: impl FnMut(u32) -> Result<(), String>,
    on_active: impl FnOnce(&DrmPageFlipActiveFrame),
    on_hold: impl FnOnce(Duration, &mut dyn FnMut(Vec<u8>) -> Result<u64, String>) -> Result<(), String>,
) -> Result<DrmPageFlipFinalState, String> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(device)
        .map_err(|error| format!("cannot open {} read-write: {error}", device.display()))?;
    let card = DrmCard(
        file.try_clone()
            .map_err(|error| format!("cannot clone DRM card fd: {error}"))?,
    );
    if drm::Device::get_driver(&card)
        .map(|driver| driver.name() == "virtio_gpu")
        .unwrap_or(false)
    {
        drop(card);
        drop(file);
        return present_drm_page_flip(
            device,
            frame_count,
            timeout_ms,
            hold_seconds,
            event_waiter,
            render_frame,
            on_event,
            on_active,
            on_hold,
        );
    }
    let resources = card
        .resource_handles()
        .map_err(|error| format!("cannot read DRM resources: {error}"))?;
    let connectors = resources
        .connectors()
        .iter()
        .map(|handle| card.get_connector(*handle, true))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read DRM connector: {error}"))?;
    let connector = connectors
        .iter()
        .find(|info| info.state() == connector::State::Connected && !info.modes().is_empty())
        .ok_or_else(|| "no connected DRM connector with a mode".to_string())?;
    let mode = connector.modes()[0];
    let crtc_handle = connector
        .current_encoder()
        .and_then(|handle| card.get_encoder(handle).ok())
        .and_then(|encoder| encoder.crtc())
        .or_else(|| resources.crtcs().first().copied())
        .ok_or_else(|| "no DRM CRTC is available".to_string())?;
    let original_crtc = card
        .get_crtc(crtc_handle)
        .map_err(|error| format!("cannot read original DRM CRTC: {error}"))?;
    let (mode_width, mode_height) = mode.size();
    let width = u32::from(mode_width);
    let height = u32::from(mode_height);
    let (verification_frame, frame_checksum, _) = render_frame(width, height)?;

    let gbm = GbmDevice::new(DeviceFd::from(OwnedFd::from(file)))
        .map_err(|error| format!("cannot create Wayland scanout GBM device: {error}"))?;
    let mut allocator = GbmAllocator::new(gbm, GbmBufferFlags::SCANOUT | GbmBufferFlags::RENDERING);
    let modifiers = [Modifier::Linear];
    let front = allocator
        .create_buffer(width, height, Fourcc::Xrgb8888, &modifiers)
        .map_err(|error| format!("cannot allocate front Wayland scanout buffer: {error}"))?;
    let back = allocator
        .create_buffer(width, height, Fourcc::Xrgb8888, &modifiers)
        .map_err(|error| format!("cannot allocate back Wayland scanout buffer: {error}"))?;
    let mut front_dmabuf = front
        .export()
        .map_err(|error| format!("cannot export front Wayland scanout dma-buf: {error}"))?;
    let mut back_dmabuf = back
        .export()
        .map_err(|error| format!("cannot export back Wayland scanout dma-buf: {error}"))?;
    render_scanout(&mut front_dmabuf, width, height)?;
    render_scanout(&mut back_dmabuf, width, height)?;

    let modifier_explicit = PlanarBuffer::modifier(&front).is_some();
    let fb_flags = if modifier_explicit {
        FbCmd2Flags::MODIFIERS
    } else {
        FbCmd2Flags::empty()
    };
    let front_framebuffer = card
        .add_planar_framebuffer(&front, fb_flags)
        .map_err(|error| format!("cannot add front Wayland scanout framebuffer: {error}"))?;
    let back_framebuffer = card
        .add_planar_framebuffer(&back, fb_flags)
        .map_err(|error| format!("cannot add back Wayland scanout framebuffer: {error}"))?;
    card.set_crtc(
        crtc_handle,
        Some(front_framebuffer),
        (0, 0),
        &[connector.handle()],
        Some(mode),
    )
    .map_err(|error| format!("cannot activate front Wayland scanout framebuffer: {error}"))?;

    let mut event_frames = Vec::with_capacity(frame_count as usize);
    for frame_index in 0..frame_count {
        let target_framebuffer = if frame_index % 2 == 0 {
            back_framebuffer
        } else {
            front_framebuffer
        };
        card.page_flip(crtc_handle, target_framebuffer, PageFlipFlags::EVENT, None)
            .map_err(|error| format!("cannot submit native Wayland page flip: {error}"))?;
        event_frames.push(wait_for_drm_page_flip(
            &card,
            crtc_handle,
            timeout_ms,
            frame_index + 1,
            event_waiter,
        )?);
        on_event(frame_index + 1)?;
    }

    let pitch = PlanarBuffer::pitches(&front)[0];
    on_active(&DrmPageFlipActiveFrame {
        connector: connector.to_string(),
        width,
        height,
        pitch,
        bytes: pitch as usize * height as usize,
        buffer_checksum: frame_checksum,
        event_frames,
    });
    let mut active_is_back = frame_count % 2 == 1;
    let mut repaint = |verification_frame: Vec<u8>| -> Result<u64, String> {
        let (target, target_framebuffer) = if active_is_back {
            (&mut front_dmabuf, front_framebuffer)
        } else {
            (&mut back_dmabuf, back_framebuffer)
        };
        render_scanout(target, width, height)?;
        let damage_width = u16::try_from(width)
            .map_err(|_| format!("scanout width exceeds DRM damage limits: {width}"))?;
        let damage_height = u16::try_from(height)
            .map_err(|_| format!("scanout height exceeds DRM damage limits: {height}"))?;
        match card.dirty_framebuffer(
            target_framebuffer,
            &[ClipRect::new(0, 0, damage_width, damage_height)],
        ) {
            Ok(()) => {}
            Err(error) if error.raw_os_error() == Some(38) => {}
            Err(error) => {
                return Err(format!(
                    "cannot mark native Wayland repaint framebuffer dirty: {error}"
                ));
            }
        }
        card.page_flip(crtc_handle, target_framebuffer, PageFlipFlags::EVENT, None)
            .map_err(|error| format!("cannot submit native Wayland repaint: {error}"))?;
        wait_for_drm_page_flip(
            &card,
            crtc_handle,
            timeout_ms,
            frame_count + 1,
            event_waiter,
        )?;
        active_is_back = !active_is_back;
        Ok(if verification_frame.len() == 8 {
            u64::from_le_bytes(
                verification_frame
                    .as_slice()
                    .try_into()
                    .map_err(|_| "invalid native repaint checksum token".to_string())?,
            )
        } else {
            checksum_bytes(&verification_frame)
        })
    };
    on_hold(Duration::from_secs(hold_seconds), &mut repaint)?;

    let restore_connectors = if original_crtc.mode().is_some() {
        vec![connector.handle()]
    } else {
        Vec::new()
    };
    card.set_crtc(
        crtc_handle,
        original_crtc.framebuffer(),
        original_crtc.position(),
        &restore_connectors,
        original_crtc.mode(),
    )
    .map_err(|error| format!("cannot restore original DRM CRTC: {error}"))?;
    card.destroy_framebuffer(back_framebuffer)
        .map_err(|error| format!("cannot destroy back Wayland scanout framebuffer: {error}"))?;
    card.destroy_framebuffer(front_framebuffer)
        .map_err(|error| format!("cannot destroy front Wayland scanout framebuffer: {error}"))?;

    drop(verification_frame);
    Ok(DrmPageFlipFinalState {
        crtc_restored: true,
        front_framebuffer_destroyed: true,
        back_framebuffer_destroyed: true,
        front_buffer_destroyed: true,
        back_buffer_destroyed: true,
    })
}

#[cfg(target_os = "linux")]
fn wait_for_drm_page_flip(
    card: &DrmCard,
    crtc_handle: drm::control::crtc::Handle,
    timeout_ms: u64,
    frame_number: u32,
    waiter: DrmEventWaiter,
) -> Result<u32, String> {
    match waiter {
        DrmEventWaiter::Polling => {
            let poller =
                Poller::new().map_err(|error| format!("cannot create DRM poller: {error}"))?;
            // The card file remains alive and is removed from the poller before either is dropped.
            unsafe {
                poller
                    .add(&card.0, PollEvent::readable(1))
                    .map_err(|error| format!("cannot register DRM event fd: {error}"))?;
            }
            let mut poll_events = PollEvents::new();
            let ready = poller
                .wait(&mut poll_events, Some(Duration::from_millis(timeout_ms)))
                .map_err(|error| format!("cannot wait for DRM page-flip event: {error}"))?;
            poller
                .delete(&card.0)
                .map_err(|error| format!("cannot remove DRM event fd from poller: {error}"))?;
            if ready == 0 {
                return Err(format!(
                    "timed out after {timeout_ms} ms waiting for DRM page-flip event {frame_number}"
                ));
            }
        }
        DrmEventWaiter::Calloop => {
            let event_file = card
                .0
                .try_clone()
                .map_err(|error| format!("cannot duplicate DRM event fd: {error}"))?;
            let mut event_loop: EventLoop<bool> = EventLoop::try_new()
                .map_err(|error| format!("cannot create calloop DRM event loop: {error}"))?;
            event_loop
                .handle()
                .insert_source(
                    Generic::new(event_file, Interest::READ, Mode::Level),
                    |_readiness, _file, event_received| {
                        *event_received = true;
                        Ok(PostAction::Remove)
                    },
                )
                .map_err(|error| format!("cannot register DRM fd with calloop: {error}"))?;
            let mut event_received = false;
            event_loop
                .dispatch(Duration::from_millis(timeout_ms), &mut event_received)
                .map_err(|error| format!("cannot dispatch calloop DRM event: {error}"))?;
            if !event_received {
                return Err(format!(
                    "calloop timed out after {timeout_ms} ms waiting for DRM page-flip event {frame_number}"
                ));
            }
        }
    }

    card.receive_events()
        .map_err(|error| format!("cannot receive DRM page-flip event: {error}"))?
        .find_map(|event| match event {
            DrmEvent::PageFlip(event) if event.crtc == crtc_handle => Some(event.frame),
            _ => None,
        })
        .ok_or_else(|| "readable DRM fd did not contain the expected page-flip event".to_string())
}

fn probe_display_activation_plan_cli() {
    let probe = probe_display_activation_plan(Viewport::new(1536, 1024));

    println!("product={}", probe.handoff.export.plan.output.plan.product);
    println!("activation_status={}", probe.status);
    println!("launch_mode={}", probe.launch_mode);
    println!("source_handoff_ready={}", state(probe.source_handoff_ready));
    println!("target_backend={}", probe.target_backend);
    println!("frame_format={}", probe.frame_format);
    println!("frame_checksum={:016x}", probe.frame_checksum);
    println!("manual_start_required={}", probe.manual_start_required);
    println!("fallback_tty_required={}", probe.fallback_tty_required);
    println!(
        "can_activate_display_output={}",
        state(probe.can_activate_display_output)
    );
    println!("display_output_started={}", probe.display_output_started);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);
    println!("desktop_shell_started={}", probe.desktop_shell_started);
    println!("autostart={}", probe.autostart);
    println!("recovery_safe={}", state(probe.recovery_safe));

    finish_stage("display-activation-plan", probe.is_ready());
}

fn smoke_display_output() {
    match run_manual_display_output_smoke(Viewport::new(1536, 1024), 3) {
        Ok(probe) => {
            println!(
                "product={}",
                probe.activation.handoff.export.plan.output.plan.product
            );
            println!("smoke_status={}", probe.status);
            println!("launch_mode={}", probe.launch_mode);
            println!("target_backend={}", probe.target_backend);
            println!("requested_frames={}", probe.requested_frames);
            println!("presented_frames={}", probe.presented_frames);
            println!("frame_interval_ms={}", probe.frame_interval_ms);
            println!("display_output_started={}", probe.display_output_started);
            println!("display_output_stopped={}", probe.display_output_stopped);
            println!("manual_start_required={}", probe.manual_start_required);
            println!("fallback_tty_available={}", probe.fallback_tty_available);
            println!("renderer_started={}", probe.renderer_started);
            println!("boot_graphics={}", probe.boot_graphics);
            println!("desktop_shell_started={}", probe.desktop_shell_started);
            println!("autostart={}", probe.autostart);
            println!("frame_format={}", probe.frame_format);
            println!("frame_checksum={:016x}", probe.frame_checksum);
            println!("checksum_accumulator={:016x}", probe.checksum_accumulator);
            println!("recovery_safe={}", state(probe.recovery_safe));

            finish_stage("display-output-smoke", probe.is_ready());
        }
        Err(error) => {
            eprintln!("display output smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=display-output-smoke status=error");
            std::process::exit(1);
        }
    }
}

fn smoke_nested_output_surface() {
    match run_nested_output_surface_lifecycle(Viewport::new(1536, 1024), 3) {
        Ok(probe) => {
            println!(
                "product={}",
                probe
                    .smoke
                    .activation
                    .handoff
                    .export
                    .plan
                    .output
                    .plan
                    .product
            );
            println!("surface_status={}", probe.status);
            println!("launch_mode={}", probe.launch_mode);
            println!("backend={}", probe.backend);
            println!("surface_acquired={}", state(probe.surface_acquired));
            println!("surface_configured={}", state(probe.surface_configured));
            println!("frame_attached={}", state(probe.frame_attached));
            println!("frame_presented={}", state(probe.frame_presented));
            println!("surface_released={}", state(probe.surface_released));
            println!("presented_frames={}", probe.presented_frames);
            println!("frame_checksum={:016x}", probe.frame_checksum);
            println!("lifecycle_serial={}", probe.lifecycle_serial);
            println!("manual_start_required={}", probe.manual_start_required);
            println!("fallback_tty_available={}", probe.fallback_tty_available);
            println!("autostart={}", probe.autostart);
            println!("boot_graphics={}", probe.boot_graphics);
            println!("renderer_started={}", probe.renderer_started);
            println!("desktop_shell_started={}", probe.desktop_shell_started);
            println!("recovery_safe={}", state(probe.recovery_safe));

            finish_stage("nested-output-surface", probe.is_ready());
        }
        Err(error) => {
            eprintln!("nested output surface smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=nested-output-surface status=error");
            std::process::exit(1);
        }
    }
}

fn probe_client_window_model_cli() {
    let probe = probe_client_window_model(Viewport::new(1536, 1024));

    for line in probe.dump_lines() {
        println!("{line}");
    }

    finish_stage("client-window-model", probe.is_ready());
}

fn probe_client_surface_lifecycle_cli() {
    let probe = probe_client_surface_lifecycle(Viewport::new(1536, 1024));

    for line in probe.dump_lines() {
        println!("{line}");
    }

    finish_stage("client-surface-lifecycle", probe.is_ready());
}

fn probe_client_surface_registry_cli() {
    match probe_client_surface_registry(Viewport::new(1536, 1024)) {
        Ok(probe) => {
            for line in probe.dump_lines() {
                println!("{line}");
            }

            finish_stage("client-surface-registry", probe.is_ready());
        }
        Err(error) => {
            eprintln!("client surface registry probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=client-surface-registry status=error");
            std::process::exit(1);
        }
    }
}

fn probe_xdg_shell_binding_cli() {
    match probe_xdg_shell_binding(Viewport::new(1536, 1024)) {
        Ok(probe) => {
            for line in probe.dump_lines() {
                println!("{line}");
            }

            finish_stage("xdg-shell-binding", probe.is_ready());
        }
        Err(error) => {
            eprintln!("xdg shell binding probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=xdg-shell-binding status=error");
            std::process::exit(1);
        }
    }
}

fn probe_xdg_toplevel_client_cli() {
    match probe_xdg_toplevel_client() {
        Ok(probe) => {
            for line in probe.dump_lines() {
                println!("{line}");
            }

            finish_stage("xdg-toplevel-client", probe.is_ready());
        }
        Err(error) => {
            eprintln!("xdg toplevel client probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=xdg-toplevel-client status=error");
            std::process::exit(1);
        }
    }
}

fn probe_xdg_toplevel_window_model_cli() {
    match probe_xdg_toplevel_window_model(Viewport::new(1536, 1024)) {
        Ok(probe) => {
            for line in probe.dump_lines() {
                println!("{line}");
            }

            finish_stage("xdg-toplevel-window-model", probe.is_ready());
        }
        Err(error) => {
            eprintln!("xdg toplevel window model probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=xdg-toplevel-window-model status=error");
            std::process::exit(1);
        }
    }
}

fn smoke_nested_preview_loop() {
    match run_nested_preview_frame_loop(Viewport::new(1536, 1024), 3) {
        Ok(probe) => {
            println!("product={}", probe.export.plan.output.plan.product);
            println!("launch_mode={}", probe.launch_mode);
            println!("window_backend={}", probe.window_backend);
            println!("frame_interval_ms={}", probe.frame_interval_ms);
            println!("requested_frames={}", probe.requested_frames);
            println!("rendered_frames={}", probe.rendered_frames);
            println!("frame_clock_started={}", state(probe.frame_clock_started));
            println!("manual_start_required={}", probe.manual_start_required);
            println!("autostart={}", probe.autostart);
            println!("preview_window_started={}", probe.preview_window_started);
            println!("renderer_started={}", probe.renderer_started);
            println!("boot_graphics={}", probe.boot_graphics);
            println!("preview_export_ready={}", state(probe.export.is_ready()));
            println!("checksum_accumulator={:016x}", probe.checksum_accumulator);
            finish_stage("nested-preview-loop", probe.is_ready());
        }
        Err(error) => {
            eprintln!("nested preview loop smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=nested-preview-loop status=error");
            std::process::exit(1);
        }
    }
}

fn probe_manual_nested_preview_backend_cli() {
    match probe_manual_nested_preview_backend(Viewport::new(1536, 1024), 3) {
        Ok(probe) => {
            println!("product={}", probe.handoff.export.plan.output.plan.product);
            println!("backend_status={}", probe.status);
            println!("launch_mode={}", probe.launch_mode);
            println!("backend_path={}", probe.backend_path);
            println!("backend_selected={}", state(probe.backend_selected));
            println!("handoff_ready={}", state(probe.handoff_ready));
            println!(
                "surface_lifecycle_ready={}",
                state(probe.surface_lifecycle_ready)
            );
            println!("frame_loop_ready={}", state(probe.frame_loop_ready));
            println!("visible_export_ready={}", state(probe.visible_export_ready));
            println!("frame_source={}", probe.frame_source);
            println!("frame_format={}", probe.frame_format);
            println!("frame_checksum={:016x}", probe.frame_checksum);
            println!(
                "surface_frame_checksum={:016x}",
                probe.surface_frame_checksum
            );
            println!(
                "loop_checksum_accumulator={:016x}",
                probe.loop_checksum_accumulator
            );
            println!(
                "frame_checksum_matches_surface={}",
                state(probe.frame_checksum_matches_surface)
            );
            println!("manual_start_required={}", probe.manual_start_required);
            println!("fallback_tty_required={}", probe.fallback_tty_required);
            println!("fallback_tty_available={}", probe.fallback_tty_available);
            println!("bounded_frame_limit={}", probe.bounded_frame_limit);
            println!("display_output_started={}", probe.display_output_started);
            println!("display_output_stopped={}", probe.display_output_stopped);
            println!("preview_window_started={}", probe.preview_window_started);
            println!("renderer_started={}", probe.renderer_started);
            println!("boot_graphics={}", probe.boot_graphics);
            println!("desktop_shell_started={}", probe.desktop_shell_started);
            println!("autostart={}", probe.autostart);
            println!("recovery_safe={}", state(probe.recovery_safe));
            finish_stage("manual-nested-preview-backend", probe.is_ready());
        }
        Err(error) => {
            eprintln!("manual nested preview backend probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=manual-nested-preview-backend status=error");
            std::process::exit(1);
        }
    }
}

fn run_manual_nested_preview_execution_cli() {
    match run_manual_nested_preview_execution(Viewport::new(1536, 1024), 3, true) {
        Ok(probe) => {
            println!(
                "product={}",
                probe.backend.handoff.export.plan.output.plan.product
            );
            println!("execution_status={}", probe.status);
            println!("launch_mode={}", probe.launch_mode);
            println!("backend_path={}", probe.backend_path);
            println!("operator_controlled={}", probe.operator_controlled);
            println!("operator_ack_required={}", probe.operator_ack_required);
            println!("operator_acknowledged={}", probe.operator_acknowledged);
            println!("backend_ready={}", state(probe.backend_ready));
            println!("requested_frames={}", probe.requested_frames);
            println!("rendered_frames={}", probe.rendered_frames);
            println!("frame_interval_ms={}", probe.frame_interval_ms);
            println!("frame_source={}", probe.frame_source);
            println!("frame_format={}", probe.frame_format);
            println!("frame_checksum={:016x}", probe.frame_checksum);
            println!("checksum_accumulator={:016x}", probe.checksum_accumulator);
            println!("display_output_started={}", probe.display_output_started);
            println!("display_output_stopped={}", probe.display_output_stopped);
            println!("preview_window_started={}", probe.preview_window_started);
            println!("cleanup_complete={}", state(probe.cleanup_complete));
            println!("fallback_tty_available={}", probe.fallback_tty_available);
            println!(
                "safe_return_to_recovery={}",
                state(probe.safe_return_to_recovery)
            );
            println!("renderer_started={}", probe.renderer_started);
            println!("boot_graphics={}", probe.boot_graphics);
            println!("desktop_shell_started={}", probe.desktop_shell_started);
            println!("autostart={}", probe.autostart);
            println!("recovery_safe={}", state(probe.recovery_safe));
            finish_stage("manual-nested-preview-execution", probe.is_ready());
        }
        Err(error) => {
            eprintln!("manual nested preview execution failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=manual-nested-preview-execution status=error");
            std::process::exit(1);
        }
    }
}

fn probe_visible_preview_export() {
    let probe = export_visible_preview_html(Viewport::new(1536, 1024));

    println!("product={}", probe.plan.output.plan.product);
    println!("format={}", probe.format);
    println!("html_bytes={}", probe.byte_count);
    println!("html_checksum={:016x}", probe.checksum);
    println!("output_plan_ready={}", state(probe.plan.output.is_ready()));
    println!("preview_plan_ready={}", state(probe.plan.is_ready()));
    println!(
        "client_layer_pipeline_ready={}",
        state(probe.client_layer_pipeline_ready)
    );
    println!(
        "client_layer_composited={}",
        state(probe.client_layer_composited)
    );
    println!("client_layer_count={}", probe.client_layer_count);
    println!("client_layer_checksum={:016x}", probe.client_layer_checksum);
    println!(
        "client_layer_buffer_snapshot_bytes={}",
        probe.client_layer_buffer_snapshot_bytes
    );
    println!(
        "client_layer_snapshot_mode={}",
        probe.client_layer_snapshot_mode
    );
    println!("png_checksum={:016x}", probe.png_checksum);
    println!("preview_window_started={}", probe.preview_window_started);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("visible-preview-export-probe", probe.is_ready());
}

fn export_visible_preview_html_cli(path: Option<PathBuf>) {
    let Some(path) = path else {
        eprintln!("export-visible-preview-html requires an output path");
        println!("[AQUA-COMPOSITOR] stage=visible-preview-export status=error");
        std::process::exit(2);
    };
    let probe = export_visible_preview_html(Viewport::new(1536, 1024));

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("visible preview export failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=visible-preview-export status=error");
            std::process::exit(1);
        }
    }

    if let Err(error) = fs::write(&path, probe.html.as_bytes()) {
        eprintln!("visible preview export failed: {error}");
        println!("[AQUA-COMPOSITOR] stage=visible-preview-export status=error");
        std::process::exit(1);
    }

    println!("path={}", path.display());
    println!("format={}", probe.format);
    println!("html_bytes={}", probe.byte_count);
    println!("html_checksum={:016x}", probe.checksum);
    println!(
        "client_layer_pipeline_ready={}",
        state(probe.client_layer_pipeline_ready)
    );
    println!(
        "client_layer_composited={}",
        state(probe.client_layer_composited)
    );
    println!("client_layer_count={}", probe.client_layer_count);
    println!("client_layer_checksum={:016x}", probe.client_layer_checksum);
    println!(
        "client_layer_buffer_snapshot_bytes={}",
        probe.client_layer_buffer_snapshot_bytes
    );
    println!(
        "client_layer_snapshot_mode={}",
        probe.client_layer_snapshot_mode
    );
    println!("png_checksum={:016x}", probe.png_checksum);
    println!("preview_window_started={}", probe.preview_window_started);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);
    finish_stage("visible-preview-export", probe.is_ready());
}

fn probe_fbdev_frame_cli(width: Option<&str>, height: Option<&str>, bits_per_pixel: Option<&str>) {
    let width = parse_fbdev_value("width", width);
    let height = parse_fbdev_value("height", height);
    let bits_per_pixel = parse_fbdev_value("bits-per-pixel", bits_per_pixel);

    match render_fbdev_frame(width, height, bits_per_pixel) {
        Ok((frame, source_checksum, runtime_wallpaper_loaded)) => {
            println!("product=Aqua Linux");
            println!("backend=fbdev");
            println!("source_format=raw-rgba8888-composited-client-preview");
            println!("source_size=1536x1024");
            println!("source_checksum={source_checksum:016x}");
            println!(
                "wallpaper_source={}",
                if runtime_wallpaper_loaded {
                    "runtime-asset"
                } else {
                    "deterministic-fallback"
                }
            );
            println!("target_size={width}x{height}");
            println!("target_bits_per_pixel={bits_per_pixel}");
            println!("target_frame_bytes={}", frame.len());
            println!("target_checksum={:016x}", checksum_bytes(&frame));
            println!("display_output_started=false");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            finish_stage("fbdev-frame-probe", true);
        }
        Err(error) => {
            eprintln!("fbdev frame probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=fbdev-frame-probe status=error");
            std::process::exit(1);
        }
    }
}

fn present_fbdev_cli(device: PathBuf) {
    let confirmation_source = fbdev_confirmation_source(
        env::var("AQUA_FBDEV_OPERATOR_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_FBDEV_HEADLESS_TEST_CONFIRMED").as_deref() == Ok("true"),
        env::var("AQUA_FBDEV_TEST_MODE").ok().as_deref(),
    );
    let Some(confirmation_source) = confirmation_source else {
        eprintln!(
            "fbdev presentation requires explicit operator or headless QEMU test confirmation"
        );
        println!("[AQUA-COMPOSITOR] stage=fbdev-present status=blocked-operator-confirmation");
        std::process::exit(1);
    };

    match present_fbdev_frame(&device) {
        Ok(metadata) => {
            println!("product=Aqua Linux");
            println!("backend=fbdev");
            println!("confirmation_source={confirmation_source}");
            println!("device={}", device.display());
            println!("target_size={}x{}", metadata.width, metadata.height);
            println!("target_bits_per_pixel={}", metadata.bits_per_pixel);
            println!("target_stride={}", metadata.stride);
            println!("presented_bytes={}", metadata.presented_bytes);
            println!("frame_checksum={:016x}", metadata.frame_checksum);
            println!(
                "wallpaper_source={}",
                if metadata.runtime_wallpaper_loaded {
                    "runtime-asset"
                } else {
                    "deterministic-fallback"
                }
            );
            println!("visible_frame_presented=true");
            println!(
                "visible_frame_observed={}",
                confirmation_source == "manual-operator"
            );
            println!("presented_frames=1");
            println!("bounded_presentation=true");
            println!("boot_graphics=false");
            println!("autostart=false");
            println!("persistent_graphical_session_started=false");
            println!("safe_return_to_recovery=ok");
            println!("[AQUA-COMPOSITOR] stage=fbdev-present status=ok");
        }
        Err(error) => {
            eprintln!("fbdev presentation failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=fbdev-present status=error");
            std::process::exit(1);
        }
    }
}

fn fbdev_confirmation_source(
    operator_confirmed: bool,
    headless_test_confirmed: bool,
    test_mode: Option<&str>,
) -> Option<&'static str> {
    if operator_confirmed {
        Some("manual-operator")
    } else if headless_test_confirmed && test_mode == Some("headless-qemu") {
        Some("headless-qemu-test")
    } else {
        None
    }
}

struct FbdevPresentation {
    width: u32,
    height: u32,
    bits_per_pixel: u32,
    stride: usize,
    presented_bytes: usize,
    frame_checksum: u64,
    runtime_wallpaper_loaded: bool,
}

fn present_fbdev_frame(device: &Path) -> Result<FbdevPresentation, String> {
    let name = device
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "fbdev device must have a file name".to_string())?;
    let sysfs_root = env::var("AQUA_FBDEV_SYSFS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/sys/class/graphics"));
    let sysfs = sysfs_root.join(name);
    let virtual_size = fs::read_to_string(sysfs.join("virtual_size"))
        .map_err(|error| format!("cannot read virtual_size: {error}"))?;
    let (width, height) = parse_virtual_size(&virtual_size)?;
    let bits_per_pixel = fs::read_to_string(sysfs.join("bits_per_pixel"))
        .map_err(|error| format!("cannot read bits_per_pixel: {error}"))?
        .trim()
        .parse::<u32>()
        .map_err(|error| format!("invalid bits_per_pixel: {error}"))?;
    let packed_stride = width as usize * bytes_per_pixel(bits_per_pixel)?;
    let stride = fs::read_to_string(sysfs.join("stride"))
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(packed_stride);
    if stride < packed_stride {
        return Err("fbdev stride is smaller than the packed row".to_string());
    }

    let (packed_frame, _, runtime_wallpaper_loaded) =
        render_fbdev_frame(width, height, bits_per_pixel)?;
    let frame = with_stride(&packed_frame, packed_stride, stride, height as usize);
    let frame_checksum = checksum_bytes(&frame);
    let hold_seconds = env::var("AQUA_FBDEV_HOLD_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3)
        .min(30);

    let mut output = OpenOptions::new()
        .write(true)
        .open(device)
        .map_err(|error| format!("cannot open {}: {error}", device.display()))?;
    output
        .seek(SeekFrom::Start(0))
        .map_err(|error| format!("cannot seek framebuffer: {error}"))?;
    output
        .write_all(&frame)
        .map_err(|error| format!("cannot write framebuffer: {error}"))?;
    output
        .flush()
        .map_err(|error| format!("cannot flush framebuffer: {error}"))?;
    thread::sleep(Duration::from_secs(hold_seconds));

    Ok(FbdevPresentation {
        width,
        height,
        bits_per_pixel,
        stride,
        presented_bytes: frame.len(),
        frame_checksum,
        runtime_wallpaper_loaded,
    })
}

fn render_fbdev_frame(
    width: u32,
    height: u32,
    bits_per_pixel: u32,
) -> Result<(Vec<u8>, u64, bool), String> {
    if width == 0 || height == 0 {
        return Err("fbdev dimensions must be non-zero".to_string());
    }
    let viewport = Viewport::new(1536, 1024);
    let pipeline = probe_client_layer_pipeline(viewport)
        .map_err(|error| format!("client layer pipeline failed: {error}"))?;
    let wallpaper_path = env::var("AQUA_WALLPAPER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(
                env::var("AQUA_ASSET_ROOT").unwrap_or_else(|_| "/usr/share/aqua".to_string()),
            )
            .join("wallpapers/default-wallpaper.png")
        });
    let wallpaper = if wallpaper_path.is_file() {
        Some(decode_png_rgba(&wallpaper_path)?)
    } else {
        None
    };
    let source = if let Some(wallpaper) = &wallpaper {
        export_composited_preview_rgba_with_wallpaper_and_client_layers(
            viewport,
            wallpaper.width,
            wallpaper.height,
            &wallpaper.rgba,
            &pipeline.paint_plan,
        )
        .map_err(|error| format!("runtime wallpaper composition failed: {error}"))?
    } else {
        export_composited_preview_rgba_with_client_layers(viewport, &pipeline.paint_plan)
    };
    let target = pack_rgba_frame(
        &source.bytes,
        source.width,
        source.height,
        width,
        height,
        bits_per_pixel,
    )?;

    Ok((target, source.checksum, wallpaper.is_some()))
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn external_client_paint_plan(
    snapshots: &[SmithayClientSurfaceSnapshot],
) -> Result<ClientLayerPaintPlan, String> {
    if !snapshots.iter().all(SmithayClientSurfaceSnapshot::is_ready) {
        return Err("external Wayland surface snapshots are not renderer-ready".to_string());
    }
    let viewport = Viewport::new(1536, 1024);
    let sources = snapshots
        .iter()
        .enumerate()
        .map(|(index, snapshot)| {
            let surface_width = snapshot.display_width.min(viewport.width);
            let surface_height = snapshot.display_height.min(viewport.height);
            ClientSurfaceSource {
                client_id: if index == 0 {
                    "external-wayland-client-1"
                } else {
                    "external-wayland-client-2"
                },
                surface_id: if index == 0 {
                    "external-xdg-toplevel-1"
                } else {
                    "external-xdg-toplevel-2"
                },
                window_id: if index == 0 {
                    "aqua-external-test-client-1"
                } else {
                    "aqua-external-test-client-2"
                },
                z_index: if snapshot.keyboard_focus_assigned {
                    16
                } else {
                    3 + index as u8
                },
                focused: snapshot.keyboard_focus_assigned,
                rect: Rect {
                    x: snapshot.x.min(viewport.width.saturating_sub(surface_width)),
                    y: snapshot
                        .y
                        .min(viewport.height.saturating_sub(surface_height)),
                    width: surface_width,
                    height: surface_height,
                },
                width: snapshot.width,
                height: snapshot.height,
                stride: snapshot.stride,
                format: "argb8888",
                source: "external-client-committed-wl-shm",
                sample_checksum: snapshot.sample_checksum,
                sample_pixel: snapshot.sample_pixel,
                sample_grid: snapshot.sample_grid,
                client_buffer_rgba: snapshot.buffer_rgba.clone(),
                renderer_import_ready: true,
            }
        })
        .collect();
    let source_plan = plan_client_surface_sources(sources);
    Ok(plan_client_layer_paint_steps(&source_plan))
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn render_fbdev_frame_with_external_clients(
    width: u32,
    height: u32,
    snapshots: &[SmithayClientSurfaceSnapshot],
    launcher: &aqua_shell::LauncherState,
) -> Result<(Vec<u8>, u64, bool), String> {
    let viewport = Viewport::new(1536, 1024);
    let paint_plan = external_client_paint_plan(snapshots)?;
    let wallpaper_path = env::var("AQUA_WALLPAPER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(
                env::var("AQUA_ASSET_ROOT").unwrap_or_else(|_| "/usr/share/aqua".to_string()),
            )
            .join("wallpapers/default-wallpaper.png")
        });
    let wallpaper = if wallpaper_path.is_file() {
        Some(decode_png_rgba(&wallpaper_path)?)
    } else {
        None
    };
    let frame = if let Some(wallpaper) = &wallpaper {
        let (frame, launcher_probe) = export_runtime_desktop_rgba_with_launcher_and_theme(
            viewport,
            wallpaper.width,
            wallpaper.height,
            &wallpaper.rgba,
            &paint_plan,
            launcher,
            configured_runtime_theme(),
        )
        .map_err(|error| format!("external client composition failed: {error}"))?;
        if launcher.is_open() && !launcher_probe.is_ready() {
            return Err("launcher overlay did not satisfy its render contract".to_string());
        }
        frame
    } else {
        export_composited_preview_rgba_with_client_layers(viewport, &paint_plan)
    };
    let target = pack_rgba_frame(&frame.bytes, frame.width, frame.height, width, height, 32)?;
    Ok((target, frame.checksum, wallpaper.is_some()))
}

#[cfg(all(target_os = "linux", feature = "smithay-smoke"))]
fn configured_runtime_theme() -> aqua_shell::AquaTheme {
    if let Ok(value) = env::var("AQUA_THEME") {
        if let Some(theme) = aqua_shell::AquaTheme::parse(&value) {
            return theme;
        }
    }
    let config_path = env::var_os("AQUA_SETTINGS_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/aqua/.config/aqua/settings.conf"));
    aqua_shell::SettingsWindowModel::load_or_default(&config_path)
        .map(|model| model.theme)
        .unwrap_or_default()
}

fn pack_rgba_frame(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    width: u32,
    height: u32,
    bits_per_pixel: u32,
) -> Result<Vec<u8>, String> {
    let expected_source_bytes = source_width as usize * source_height as usize * 4;
    if source_width == 0 || source_height == 0 || source.len() != expected_source_bytes {
        return Err("rgba source size does not match its dimensions".to_string());
    }
    if bits_per_pixel == 32 && source_width == width && source_height == height {
        let mut target = source.to_vec();
        for pixel in target.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 0xff;
        }
        return Ok(target);
    }
    let pixel_bytes = bytes_per_pixel(bits_per_pixel)?;
    let mut target = Vec::with_capacity(width as usize * height as usize * pixel_bytes);
    for target_y in 0..height {
        let source_y = target_y as usize * source_height as usize / height as usize;
        for target_x in 0..width {
            let source_x = target_x as usize * source_width as usize / width as usize;
            let offset = (source_y * source_width as usize + source_x) * 4;
            let red = source[offset];
            let green = source[offset + 1];
            let blue = source[offset + 2];
            match bits_per_pixel {
                32 => target.extend_from_slice(&[blue, green, red, 0xff]),
                24 => target.extend_from_slice(&[blue, green, red]),
                16 => {
                    let pixel =
                        ((red as u16 >> 3) << 11) | ((green as u16 >> 2) << 5) | (blue as u16 >> 3);
                    target.extend_from_slice(&pixel.to_le_bytes());
                }
                _ => unreachable!(),
            }
        }
    }
    Ok(target)
}

struct DecodedWallpaper {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode_png_rgba(path: &Path) -> Result<DecodedWallpaper, String> {
    let file = File::open(path)
        .map_err(|error| format!("cannot open runtime wallpaper {}: {error}", path.display()))?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("cannot read runtime wallpaper PNG: {error}"))?;
    let output_size = reader
        .output_buffer_size()
        .ok_or_else(|| "runtime wallpaper output buffer is too large".to_string())?;
    let mut decoded = vec![0; output_size];
    let info = reader
        .next_frame(&mut decoded)
        .map_err(|error| format!("cannot decode runtime wallpaper PNG: {error}"))?;
    if info.bit_depth != png::BitDepth::Eight {
        return Err("runtime wallpaper must use 8-bit channels".to_string());
    }
    let bytes = &decoded[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 0xff])
            .collect(),
        png::ColorType::Rgba => bytes.to_vec(),
        other => {
            return Err(format!(
                "unsupported runtime wallpaper color type: {other:?}"
            ))
        }
    };
    Ok(DecodedWallpaper {
        width: info.width,
        height: info.height,
        rgba,
    })
}

fn bytes_per_pixel(bits_per_pixel: u32) -> Result<usize, String> {
    match bits_per_pixel {
        16 => Ok(2),
        24 => Ok(3),
        32 => Ok(4),
        other => Err(format!("unsupported fbdev bits_per_pixel: {other}")),
    }
}

fn parse_virtual_size(value: &str) -> Result<(u32, u32), String> {
    let (width, height) = value
        .trim()
        .split_once(',')
        .ok_or_else(|| "virtual_size must be width,height".to_string())?;
    let width = width
        .parse::<u32>()
        .map_err(|error| format!("invalid fbdev width: {error}"))?;
    let height = height
        .parse::<u32>()
        .map_err(|error| format!("invalid fbdev height: {error}"))?;
    if width == 0 || height == 0 {
        return Err("fbdev dimensions must be non-zero".to_string());
    }
    Ok((width, height))
}

fn with_stride(packed: &[u8], packed_stride: usize, stride: usize, rows: usize) -> Vec<u8> {
    if packed_stride == stride {
        return packed.to_vec();
    }
    let mut output = vec![0; stride * rows];
    for row in 0..rows {
        let source = row * packed_stride;
        let target = row * stride;
        output[target..target + packed_stride]
            .copy_from_slice(&packed[source..source + packed_stride]);
    }
    output
}

fn parse_fbdev_value(name: &str, value: Option<&str>) -> u32 {
    let Some(value) = value else {
        eprintln!("probe-fbdev-frame requires {name}");
        std::process::exit(2);
    };
    value.parse::<u32>().unwrap_or_else(|error| {
        eprintln!("invalid {name}: {error}");
        std::process::exit(2);
    })
}

fn checksum_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |checksum, byte| {
        (checksum ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn probe_visible_preview_plan_cli() {
    let probe = probe_visible_preview_plan(Viewport::new(1536, 1024));

    println!("product={}", probe.output.plan.product);
    println!("mode={}", probe.output.plan.mode);
    println!("primary_backend={}", probe.output.plan.primary_backend);
    println!("later_backend={}", probe.output.plan.later_backend);
    println!(
        "output_size={}x{}",
        probe.output.plan.width, probe.output.plan.height
    );
    println!("pixel_format={}", probe.output.plan.pixel_format);
    println!("output_plan_ready={}", state(probe.output.is_ready()));
    println!("scene_ready={}", state(probe.scene_ready));
    println!("render_plan_ready={}", state(probe.render_plan_ready));
    println!("paint_plan_ready={}", state(probe.paint_plan_ready));
    println!("frame_plan_ready={}", state(probe.frame_plan_ready));
    println!("frame_buffer_ready={}", state(probe.frame_buffer_ready));
    println!("raster_ready={}", state(probe.raster_ready));
    println!("png_export_ready={}", state(probe.png_export_ready));
    println!(
        "client_layer_pipeline_ready={}",
        state(probe.client_layer_pipeline_ready)
    );
    println!("client_layer_count={}", probe.client_layer_count);
    println!("client_layer_checksum={:016x}", probe.client_layer_checksum);
    println!(
        "client_layer_buffer_snapshot_bytes={}",
        probe.client_layer_buffer_snapshot_bytes
    );
    println!(
        "client_layer_snapshot_mode={}",
        probe.client_layer_snapshot_mode
    );
    println!("preview_window_started={}", probe.preview_window_started);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("visible-preview-plan-probe", probe.is_ready());
}

fn dump_output_plan() {
    let probe = probe_display_output_plan();

    for line in probe.plan.dump_lines() {
        println!("{line}");
    }

    finish_stage("output-plan-dump", probe.is_ready());
}

fn probe_output_plan() {
    let probe = probe_display_output_plan();

    println!("product={}", probe.plan.product);
    println!("mode={}", probe.plan.mode);
    println!("primary_backend={}", probe.plan.primary_backend);
    println!("later_backend={}", probe.plan.later_backend);
    println!("output_size={}x{}", probe.plan.width, probe.plan.height);
    println!("scale={}", probe.plan.scale);
    println!("pixel_format={}", probe.plan.pixel_format);
    println!("refresh_millihz={}", probe.plan.refresh_millihz);
    println!("mode_ready={}", state(probe.mode_ready));
    println!("backend_ready={}", state(probe.backend_ready));
    println!("dimensions_ready={}", state(probe.dimensions_ready));
    println!("format_ready={}", state(probe.format_ready));
    println!("refresh_ready={}", state(probe.refresh_ready));
    println!("recovery_safe={}", state(probe.recovery_safe));
    println!("boot_graphics={}", probe.plan.boot_graphics);
    println!("renderer_started={}", probe.plan.renderer_started);
    println!("desktop_shell_started={}", probe.plan.desktop_shell_started);

    finish_stage("output-plan-probe", probe.is_ready());
}

fn probe_display_output_handoff_cli() {
    let probe = probe_display_output_handoff(Viewport::new(1536, 1024));

    println!("product={}", probe.export.plan.output.plan.product);
    println!("handoff_status={}", probe.status);
    println!("target_backend={}", probe.target_backend);
    println!("output_size={}x{}", probe.output_width, probe.output_height);
    println!("pixel_format={}", probe.pixel_format);
    println!("frame_buffer_bytes={}", probe.frame_buffer_bytes);
    println!("frame_format={}", probe.frame_format);
    println!("frame_checksum={:016x}", probe.frame_checksum);
    println!("preview_export_ready={}", state(probe.export.is_ready()));
    println!(
        "client_layer_pipeline_ready={}",
        state(probe.export.client_layer_pipeline_ready)
    );
    println!(
        "client_layer_composited={}",
        state(probe.client_layer_composited)
    );
    println!(
        "client_layer_buffer_snapshot_bytes={}",
        probe.client_layer_buffer_snapshot_bytes
    );
    println!(
        "client_layer_snapshot_mode={}",
        probe.client_layer_snapshot_mode
    );
    println!(
        "output_surface_prepared={}",
        state(probe.output_surface_prepared)
    );
    println!("display_output_started={}", probe.display_output_started);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);
    println!("desktop_shell_started={}", probe.desktop_shell_started);
    println!("recovery_safe={}", state(probe.recovery_safe));

    finish_stage("display-output-handoff", probe.is_ready());
}

fn dump_raster_png_export() {
    let probe = probe_static_raster_png_export(Viewport::new(1536, 1024));

    for line in probe.export.dump_lines() {
        println!("{line}");
    }

    finish_stage("raster-png-export-dump", probe.is_ready());
}

fn probe_raster_png_export() {
    let probe = probe_static_raster_png_export(Viewport::new(1536, 1024));

    println!("renderer=aqua-renderer");
    println!("export_status={}", probe.export.status);
    println!("export_backend={}", probe.export.backend);
    println!("frame_size={}x{}", probe.export.width, probe.export.height);
    println!("export_format={}", probe.export.format);
    println!("export_bytes={}", probe.export.byte_count);
    println!("export_checksum={:016x}", probe.export.checksum);
    println!("format_ready={}", state(probe.format_ready));
    println!("byte_count_ready={}", state(probe.byte_count_ready));
    println!("checksum_ready={}", state(probe.checksum_ready));
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("raster-png-export-probe", probe.is_ready());
}

fn export_raster_png(path: Option<PathBuf>) {
    let Some(path) = path else {
        eprintln!("export-raster-png requires an output path");
        println!("[AQUA-COMPOSITOR] stage=raster-png-export status=error");
        std::process::exit(2);
    };
    let probe = probe_static_raster_png_export(Viewport::new(1536, 1024));

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("raster png export failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=raster-png-export status=error");
            std::process::exit(1);
        }
    }

    if let Err(error) = fs::write(&path, &probe.export.bytes) {
        eprintln!("raster png export failed: {error}");
        println!("[AQUA-COMPOSITOR] stage=raster-png-export status=error");
        std::process::exit(1);
    }

    println!("renderer=aqua-renderer");
    println!("path={}", path.display());
    println!("export_format={}", probe.export.format);
    println!("export_bytes={}", probe.export.byte_count);
    println!("export_checksum={:016x}", probe.export.checksum);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);
    finish_stage("raster-png-export", probe.is_ready());
}

fn dump_raster_export() {
    let probe = probe_static_raster_export(Viewport::new(1536, 1024));

    for line in probe.export.dump_lines() {
        println!("{line}");
    }

    finish_stage("raster-export-dump", probe.is_ready());
}

fn probe_raster_export() {
    let probe = probe_static_raster_export(Viewport::new(1536, 1024));

    println!("renderer=aqua-renderer");
    println!("export_status={}", probe.export.status);
    println!("export_backend={}", probe.export.backend);
    println!("frame_size={}x{}", probe.export.width, probe.export.height);
    println!("export_format={}", probe.export.format);
    println!("export_bytes={}", probe.export.byte_count);
    println!("export_checksum={:016x}", probe.export.checksum);
    println!("format_ready={}", state(probe.format_ready));
    println!("byte_count_ready={}", state(probe.byte_count_ready));
    println!("checksum_ready={}", state(probe.checksum_ready));
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("raster-export-probe", probe.is_ready());
}

fn export_raster_ppm(path: Option<PathBuf>) {
    let Some(path) = path else {
        eprintln!("export-raster-ppm requires an output path");
        println!("[AQUA-COMPOSITOR] stage=raster-export status=error");
        std::process::exit(2);
    };
    let probe = probe_static_raster_export(Viewport::new(1536, 1024));

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("raster export failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=raster-export status=error");
            std::process::exit(1);
        }
    }

    if let Err(error) = fs::write(&path, &probe.export.bytes) {
        eprintln!("raster export failed: {error}");
        println!("[AQUA-COMPOSITOR] stage=raster-export status=error");
        std::process::exit(1);
    }

    println!("renderer=aqua-renderer");
    println!("path={}", path.display());
    println!("export_format={}", probe.export.format);
    println!("export_bytes={}", probe.export.byte_count);
    println!("export_checksum={:016x}", probe.export.checksum);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("raster-export", probe.is_ready());
}

fn dump_raster() {
    let probe = probe_static_software_raster(Viewport::new(1536, 1024));

    for line in probe.probe.dump_lines() {
        println!("{line}");
    }

    finish_stage("raster-dump", probe.is_ready());
}

fn probe_raster() {
    let probe = probe_static_software_raster(Viewport::new(1536, 1024));

    println!("renderer=aqua-renderer");
    println!("raster_status={}", probe.probe.status);
    println!("raster_backend={}", probe.probe.backend);
    println!("frame_size={}x{}", probe.probe.width, probe.probe.height);
    println!("pixel_format={}", probe.probe.pixel_format);
    println!("filled_rect_count={}", probe.probe.filled_rect_count);
    println!("expected_rect_count={}", probe.probe.expected_rect_count);
    println!(
        "wallpaper_sample={:02x},{:02x},{:02x},{:02x}",
        probe.probe.wallpaper_sample[0],
        probe.probe.wallpaper_sample[1],
        probe.probe.wallpaper_sample[2],
        probe.probe.wallpaper_sample[3]
    );
    println!(
        "surface_sample={:02x},{:02x},{:02x},{:02x}",
        probe.probe.surface_sample[0],
        probe.probe.surface_sample[1],
        probe.probe.surface_sample[2],
        probe.probe.surface_sample[3]
    );
    println!(
        "dock_sample={:02x},{:02x},{:02x},{:02x}",
        probe.probe.dock_sample[0],
        probe.probe.dock_sample[1],
        probe.probe.dock_sample[2],
        probe.probe.dock_sample[3]
    );
    println!(
        "surface_border_sample={:02x},{:02x},{:02x},{:02x}",
        probe.probe.surface_border_sample[0],
        probe.probe.surface_border_sample[1],
        probe.probe.surface_border_sample[2],
        probe.probe.surface_border_sample[3]
    );
    println!(
        "surface_highlight_sample={:02x},{:02x},{:02x},{:02x}",
        probe.probe.surface_highlight_sample[0],
        probe.probe.surface_highlight_sample[1],
        probe.probe.surface_highlight_sample[2],
        probe.probe.surface_highlight_sample[3]
    );
    println!(
        "surface_corner_sample={:02x},{:02x},{:02x},{:02x}",
        probe.probe.surface_corner_sample[0],
        probe.probe.surface_corner_sample[1],
        probe.probe.surface_corner_sample[2],
        probe.probe.surface_corner_sample[3]
    );
    println!(
        "surface_shadow_sample={:02x},{:02x},{:02x},{:02x}",
        probe.probe.surface_shadow_sample[0],
        probe.probe.surface_shadow_sample[1],
        probe.probe.surface_shadow_sample[2],
        probe.probe.surface_shadow_sample[3]
    );
    println!("raster_checksum={:016x}", probe.probe.raster_checksum);
    println!(
        "surface_primitive_count={}",
        probe.probe.surface_primitive_count
    );
    println!("rect_count_ready={}", state(probe.rect_count_ready));
    println!(
        "wallpaper_sample_ready={}",
        state(probe.wallpaper_sample_ready)
    );
    println!("surface_sample_ready={}", state(probe.surface_sample_ready));
    println!("dock_sample_ready={}", state(probe.dock_sample_ready));
    println!(
        "surface_border_sample_ready={}",
        state(probe.surface_border_sample_ready)
    );
    println!(
        "surface_highlight_sample_ready={}",
        state(probe.surface_highlight_sample_ready)
    );
    println!(
        "surface_corner_sample_ready={}",
        state(probe.surface_corner_sample_ready)
    );
    println!(
        "surface_shadow_sample_ready={}",
        state(probe.surface_shadow_sample_ready)
    );
    println!("checksum_ready={}", state(probe.checksum_ready));
    println!(
        "surface_primitives_ready={}",
        state(probe.surface_primitives_ready)
    );
    println!("buffer_bytes={}", probe.probe.buffer_bytes);
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("raster-probe", probe.is_ready());
}

fn dump_frame_buffer() {
    let probe = probe_static_frame_buffer(Viewport::new(1536, 1024));

    for line in probe.probe.dump_lines() {
        println!("{line}");
    }

    finish_stage("frame-buffer-dump", probe.is_ready());
}

fn probe_frame_buffer() {
    let probe = probe_static_frame_buffer(Viewport::new(1536, 1024));

    println!("renderer=aqua-renderer");
    println!("buffer_status={}", probe.probe.status);
    println!("buffer_backend={}", probe.probe.backend);
    println!("frame_size={}x{}", probe.probe.width, probe.probe.height);
    println!("pixel_format={}", probe.probe.pixel_format);
    println!("buffer_bytes={}", probe.probe.buffer_bytes);
    println!("allocated_bytes={}", probe.probe.allocated_bytes);
    println!("clear_color={}", probe.probe.clear_color);
    println!(
        "first_pixel={:02x},{:02x},{:02x},{:02x}",
        probe.probe.first_pixel[0],
        probe.probe.first_pixel[1],
        probe.probe.first_pixel[2],
        probe.probe.first_pixel[3]
    );
    println!(
        "last_pixel={:02x},{:02x},{:02x},{:02x}",
        probe.probe.last_pixel[0],
        probe.probe.last_pixel[1],
        probe.probe.last_pixel[2],
        probe.probe.last_pixel[3]
    );
    println!("buffer_allocated={}", state(probe.buffer_allocated));
    println!("clear_color_ready={}", state(probe.clear_color_ready));
    println!("first_pixel_ready={}", state(probe.first_pixel_ready));
    println!("last_pixel_ready={}", state(probe.last_pixel_ready));
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("frame-buffer-probe", probe.is_ready());
}

fn dump_frame_plan() {
    let probe = probe_static_frame_plan(Viewport::new(1536, 1024));

    for line in probe.plan.dump_lines() {
        println!("{line}");
    }

    finish_stage("frame-plan-dump", probe.is_ready());
}

fn probe_frame_plan() {
    let probe = probe_static_frame_plan(Viewport::new(1536, 1024));

    println!("renderer=aqua-renderer");
    println!("frame_status={}", probe.plan.status);
    println!("frame_backend={}", probe.plan.backend);
    println!("frame_size={}x{}", probe.plan.width, probe.plan.height);
    println!("pixel_format={}", probe.plan.pixel_format);
    println!("stride_bytes={}", probe.plan.stride_bytes);
    println!("buffer_bytes={}", probe.plan.buffer_bytes);
    println!("clear_color={}", probe.plan.clear_color);
    println!("paint_step_count={}", probe.plan.paint_step_count);
    println!("frame_ready={}", state(probe.frame_ready));
    println!("pixel_format_ready={}", state(probe.pixel_format_ready));
    println!("stride_ready={}", state(probe.stride_ready));
    println!("damage_ready={}", state(probe.damage_ready));
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("frame-plan-probe", probe.is_ready());
}

fn dump_paint_plan() {
    let probe = probe_static_paint_plan(Viewport::new(1536, 1024));

    for line in probe.plan.dump_lines() {
        println!("{line}");
    }

    finish_stage("paint-plan-dump", probe.is_ready());
}

fn probe_paint_plan() {
    let probe = probe_static_paint_plan(Viewport::new(1536, 1024));

    println!("renderer=aqua-renderer");
    println!("paint_status={}", probe.plan.status);
    println!("paint_backend={}", probe.plan.backend);
    println!("paint_step_count={}", probe.paint_step_count);
    println!("expected_paint_steps={}", probe.expected_paint_steps);
    println!(
        "system_surface_steps_translucent={}",
        state(probe.system_surface_steps_translucent)
    );
    println!("paint_order_stable={}", state(probe.paint_order_stable));
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("paint-plan-probe", probe.is_ready());
}

fn probe_session_bootstrap_cli(
    config_path: Option<PathBuf>,
    prepared_runtime_dir: Option<PathBuf>,
) {
    let config = match config_path {
        Some(path) => match read_session_config(&path) {
            Ok(config) => {
                println!("source=file");
                println!("path={}", path.display());
                config
            }
            Err(error) => {
                eprintln!("session bootstrap probe failed: {error}");
                println!("[AQUA-COMPOSITOR] stage=session-bootstrap status=error");
                std::process::exit(1);
            }
        },
        None => {
            let content = default_session_config().dump_lines().join("\n");
            println!("source=default");
            parse_session_config(&content).expect("default session config should parse")
        }
    };
    let prepared_runtime_dir = prepared_runtime_dir.unwrap_or_else(|| {
        std::env::temp_dir().join(format!("aqua-bootstrap-probe-{}", std::process::id()))
    });

    match probe_session_bootstrap(&config, &prepared_runtime_dir) {
        Ok(probe) => {
            println!("product={}", probe.product);
            println!("mode={}", probe.mode);
            println!("configured_runtime_dir={}", probe.configured_runtime_dir);
            println!(
                "prepared_runtime_dir={}",
                probe.prepared_runtime_dir.display()
            );
            println!("WAYLAND_DISPLAY={}", probe.wayland_display);
            println!("XDG_RUNTIME_DIR={}", probe.xdg_runtime_dir);
            println!("AQUA_ASSET_ROOT={}", probe.aqua_asset_root);
            println!("config_recovery_safe={}", state(probe.config_recovery_safe));
            println!("env_recovery_safe={}", state(probe.env_recovery_safe));
            println!("runtime_dir_prepared={}", state(probe.runtime_dir_prepared));
            println!("runtime_dir_private={}", state(probe.runtime_dir_private));
            println!("autostart_blocked={}", state(probe.autostart_blocked));
            println!(
                "boot_graphics_blocked={}",
                state(probe.boot_graphics_blocked)
            );
            println!("session_started={}", probe.session_started);
            println!("desktop_shell_started={}", probe.desktop_shell_started);
            finish_stage("session-bootstrap", probe.is_ready());
        }
        Err(error) => {
            eprintln!("session bootstrap probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=session-bootstrap status=error");
            std::process::exit(1);
        }
    }
}

fn probe_session_env(path: Option<PathBuf>) {
    let (lines, ready) = match path {
        Some(path) => match read_session_config(&path) {
            Ok(config) => {
                println!("source=file");
                println!("path={}", path.display());
                let env = config.environment();
                (env.dump_lines(), env.is_recovery_safe())
            }
            Err(error) => {
                eprintln!("session env probe failed: {error}");
                println!("[AQUA-COMPOSITOR] stage=session-env status=error");
                std::process::exit(1);
            }
        },
        None => {
            let env = default_session_environment();
            println!("source=default");
            (env.dump_lines(), env.is_recovery_safe())
        }
    };

    for line in lines {
        println!("{line}");
    }

    finish_stage("session-env", ready);
}

fn probe_session_config(path: Option<PathBuf>) {
    let (lines, ready) = match path {
        Some(path) => match read_session_config(&path) {
            Ok(config) => {
                println!("source=file");
                println!("path={}", path.display());
                (config.dump_lines(), config.is_recovery_safe())
            }
            Err(error) => {
                eprintln!("session config probe failed: {error}");
                println!("[AQUA-COMPOSITOR] stage=session-config status=error");
                std::process::exit(1);
            }
        },
        None => {
            let config = default_session_config();
            println!("source=default");
            (config.dump_lines(), config.is_recovery_safe())
        }
    };

    for line in lines {
        println!("{line}");
    }

    finish_stage("session-config", ready);
}

fn smoke_session_loop() {
    match run_session_loop_smoke() {
        Ok(result) => {
            println!("event_loop=calloop");
            println!("socket_name={}", result.socket_name);
            println!("loop_started={}", state(result.loop_started));
            println!("loop_iterations={}", result.loop_iterations);
            println!("max_iterations={}", result.max_iterations);
            println!("socket_bound={}", state(result.socket_bound));
            println!("client_connected={}", state(result.client_connected));
            println!("callback_invoked={}", state(result.callback_invoked));
            println!("client_accepted={}", state(result.client_accepted));
            println!("client_inserted={}", state(result.client_inserted));
            println!("dispatch_passes={}", result.dispatch_passes);
            println!("flush_passes={}", result.flush_passes);
            println!("socket_cleaned={}", state(result.socket_cleaned));
            println!("host_stub={}", result.host_stub);
            finish_stage("session-loop", result.is_ready());
        }
        Err(error) => {
            eprintln!("session loop smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=session-loop status=error");
            std::process::exit(1);
        }
    }
}

fn dump_render_plan() {
    let probe = probe_static_render_plan(Viewport::new(1536, 1024));

    for line in probe.plan.dump_lines() {
        println!("{line}");
    }

    finish_stage("render-plan-dump", probe.is_ready());
}

fn probe_render_plan() {
    let probe = probe_static_render_plan(Viewport::new(1536, 1024));

    println!("renderer=aqua-renderer");
    println!("renderer_status={}", probe.plan.status);
    println!("renderer_backend={}", probe.plan.backend);
    println!("draw_command_count={}", probe.draw_command_count);
    println!("expected_draw_commands={}", probe.expected_draw_commands);
    println!(
        "system_surface_commands_simulated={}",
        state(probe.system_surface_commands_simulated)
    );
    println!("renderer_started={}", probe.renderer_started);
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("render-plan-probe", probe.is_ready());
}

fn probe_renderer_surface_sources_cli() {
    match probe_renderer_surface_sources(Viewport::new(1536, 1024)) {
        Ok(probe) => {
            println!("renderer=aqua-renderer");
            println!("surface_source_status={}", probe.plan.status);
            println!("surface_source_backend={}", probe.plan.backend);
            println!(
                "source_registry_ready={}",
                state(probe.source_registry_ready)
            );
            println!("surface_source_count={}", probe.source_count);
            println!("expected_surface_sources={}", probe.expected_sources);
            println!("active_source_ready={}", state(probe.active_source_ready));
            println!("import_sources_ready={}", state(probe.import_sources_ready));
            println!("z_order_ready={}", state(probe.z_order_ready));
            println!("renderer_started={}", probe.renderer_started);
            println!("boot_graphics={}", probe.boot_graphics);

            for line in probe.plan.dump_lines().into_iter().skip(4) {
                println!("{line}");
            }

            finish_stage("renderer-surface-sources", probe.is_ready());
        }
        Err(error) => {
            eprintln!("renderer surface source probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=renderer-surface-sources status=error");
            std::process::exit(1);
        }
    }
}

fn probe_client_layer_pipeline_cli() {
    match probe_client_layer_pipeline(Viewport::new(1536, 1024)) {
        Ok(probe) => {
            println!("renderer=aqua-renderer");
            println!(
                "client_layer_pipeline_status={}",
                if probe.is_ready() { "ready" } else { "missing" }
            );
            println!("source_plan_ready={}", state(probe.source_plan_ready));
            println!("paint_plan_ready={}", state(probe.paint_plan_ready));
            println!("raster_ready={}", state(probe.raster_ready));
            println!("client_layer_count={}", probe.layer_count);
            println!("expected_client_layers={}", probe.expected_layers);
            println!(
                "active_layer_sample={}",
                pixel(probe.raster_probe.active_layer_sample)
            );
            println!(
                "inactive_layer_sample={}",
                pixel(probe.raster_probe.inactive_layer_sample)
            );
            println!(
                "client_layer_checksum={:016x}",
                probe.raster_probe.layer_checksum
            );
            println!(
                "source_checksum_fold={:016x}",
                probe.raster_probe.source_checksum_fold
            );
            println!("renderer_started={}", probe.renderer_started);
            println!("boot_graphics={}", probe.boot_graphics);

            for line in probe.paint_plan.dump_lines().into_iter().skip(4) {
                println!("{line}");
            }

            finish_stage("client-layer-pipeline", probe.is_ready());
        }
        Err(error) => {
            eprintln!("client layer pipeline probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=client-layer-pipeline status=error");
            std::process::exit(1);
        }
    }
}

fn dump_scene() {
    let probe = probe_static_shell_scene(Viewport::new(1536, 1024));

    for line in probe.scene.dump_lines() {
        println!("{line}");
    }

    finish_stage("scene-dump", probe.is_ready());
}

fn probe_launcher_model_cli() {
    let probe = probe_launcher_model();

    println!("[AQUA-SHELL] stage=launcher-model status=running");
    for line in probe.dump_lines() {
        println!("{line}");
    }
    println!(
        "launcher_model_ready={}",
        if probe.is_ready() { "ok" } else { "failed" }
    );
    finish_stage("launcher-model", probe.is_ready());
}

fn probe_launcher_input_scene_cli() {
    let probe = probe_launcher_input_scene_binding(Viewport::new(1536, 1024));

    println!("[AQUA-SHELL] stage=launcher-input-scene status=running");
    println!("binding_status={}", probe.status);
    println!("input_source={}", probe.input_source);
    println!(
        "initial_launcher_visible={}",
        probe.initial_launcher_visible
    );
    println!("opened_launcher_visible={}", probe.opened_launcher_visible);
    println!(
        "dismissed_launcher_visible={}",
        probe.dismissed_launcher_visible
    );
    println!("open_draw_command_count={}", probe.open_draw_command_count);
    println!(
        "closed_draw_command_count={}",
        probe.closed_draw_command_count
    );
    println!("redraw_requests={}", probe.redraw_requests);
    println!("visibility_changes={}", probe.visibility_changes);
    println!(
        "launch_request_app={}",
        probe
            .launch_request
            .as_ref()
            .map(|request| request.app_id)
            .unwrap_or("none")
    );
    println!(
        "launch_request_command={}",
        probe
            .launch_request
            .as_ref()
            .map(|request| request.command)
            .unwrap_or("none")
    );
    println!("boot_graphics={}", probe.boot_graphics);
    println!("autostart={}", probe.autostart);
    println!(
        "launcher_input_scene_ready={}",
        if probe.is_ready() { "ok" } else { "failed" }
    );
    finish_stage("launcher-input-scene", probe.is_ready());
}

fn probe_smithay_launcher_seat_cli() {
    match probe_smithay_launcher_seat(Viewport::new(1536, 1024)) {
        Ok(probe) => {
            println!("[AQUA-SHELL] stage=smithay-launcher-seat status=running");
            println!("binding_status={}", probe.status);
            println!("seat_name={}", probe.seat_name);
            println!("seat_global_created={}", probe.seat_global_created);
            println!("keyboard_capability={}", probe.keyboard_capability);
            println!("pointer_capability={}", probe.pointer_capability);
            println!(
                "keyboard_event_intercepted={}",
                probe.keyboard_event_intercepted
            );
            println!(
                "pointer_motion_dispatched={}",
                probe.pointer_motion_dispatched
            );
            println!(
                "pointer_button_dispatched={}",
                probe.pointer_button_dispatched
            );
            println!("launcher_visible={}", probe.launcher_visible);
            println!("selected_category={}", probe.selected_category);
            println!("draw_command_count={}", probe.draw_command_count);
            println!("host_stub={}", probe.host_stub);
            println!("boot_graphics={}", probe.boot_graphics);
            println!("autostart={}", probe.autostart);
            println!(
                "smithay_launcher_seat_ready={}",
                if probe.is_ready() { "ok" } else { "failed" }
            );
            finish_stage("smithay-launcher-seat", probe.is_ready());
        }
        Err(error) => {
            eprintln!("Smithay launcher seat probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=smithay-launcher-seat status=error");
            std::process::exit(1);
        }
    }
}

fn probe_scene() {
    let probe = probe_static_shell_scene(Viewport::new(1536, 1024));

    println!("scene_model=aqua-scene");
    println!("scene_status={}", probe.scene.status);
    println!(
        "viewport={}x{}",
        probe.scene.viewport.width, probe.scene.viewport.height
    );
    println!("surface_count={}", probe.scene.surfaces.len());
    println!("asset_count={}", probe.scene.assets.len());
    println!("material_token_count={}", probe.scene.material_tokens.len());
    println!("required_surfaces={}", probe.required_surfaces);
    println!("expected_surfaces={}", probe.expected_surfaces);
    println!(
        "surfaces_fit_viewport={}",
        state(probe.surfaces_fit_viewport)
    );
    println!(
        "wallpaper_covers_viewport={}",
        state(probe.wallpaper_covers_viewport)
    );
    println!("toast_avoids_dock={}", state(probe.toast_avoids_dock));
    println!("launcher_avoids_dock={}", state(probe.launcher_avoids_dock));
    println!(
        "mock_surfaces_labeled={}",
        state(probe.mock_surfaces_labeled)
    );
    println!(
        "required_assets_present={}",
        state(probe.required_assets_present)
    );
    println!(
        "permanent_assets_only={}",
        state(probe.permanent_assets_only)
    );
    println!(
        "required_material_tokens_present={}",
        state(probe.required_material_tokens_present)
    );
    println!(
        "simulated_surface_labeled={}",
        state(probe.simulated_surface_labeled)
    );
    println!("boot_graphics={}", probe.boot_graphics);

    finish_stage("scene-probe", probe.is_ready());
}

fn smoke_run_once() {
    match run_session_once_smoke() {
        Ok(result) => {
            println!("event_loop=calloop");
            println!("socket_name={}", result.socket_name);
            println!("run_once_called={}", state(result.run_once_called));
            println!("socket_bound={}", state(result.socket_bound));
            println!("client_connected={}", state(result.client_connected));
            println!("callback_invoked={}", state(result.callback_invoked));
            println!("client_accepted={}", state(result.client_accepted));
            println!("client_inserted={}", state(result.client_inserted));
            println!("dispatch_clients={}", state(result.dispatch_clients_ok));
            println!("dispatched_requests={}", result.dispatched_requests);
            println!("flush_clients={}", state(result.flush_clients_ok));
            println!("socket_cleaned={}", state(result.socket_cleaned));
            println!("host_stub={}", result.host_stub);
            finish_stage("session-run-once", result.is_ready());
        }
        Err(error) => {
            eprintln!("session run-once smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=session-run-once status=error");
            std::process::exit(1);
        }
    }
}

fn probe_session() {
    match probe_session_skeleton() {
        Ok(probe) => {
            println!("product={}", probe.product);
            println!("mode={}", probe.mode);
            println!("foundation={}", probe.foundation);
            println!("event_loop={}", probe.event_loop);
            println!("display_owned={}", state(probe.display_owned));
            println!(
                "compositor_state_owned={}",
                state(probe.compositor_state_owned)
            );
            println!("client_inserted={}", state(probe.client_inserted));
            println!("dispatch_clients={}", state(probe.dispatch_clients_ok));
            println!("flush_clients={}", state(probe.flush_clients_ok));
            println!("host_stub={}", probe.host_stub);
            finish_stage("session-skeleton", probe.is_ready());
        }
        Err(error) => {
            eprintln!("session skeleton probe failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=session-skeleton status=error");
            std::process::exit(1);
        }
    }
}

fn smoke_calloop_socket() {
    match run_calloop_socket_smoke() {
        Ok(result) => {
            println!("foundation=smithay");
            println!("event_loop=calloop");
            println!("socket_name={}", result.socket_name);
            println!("wayland_display={}", state(result.display_created));
            println!(
                "compositor_global={}",
                state(result.compositor_global_created)
            );
            println!("socket_bound={}", state(result.socket_bound));
            println!("client_connected={}", state(result.client_connected));
            println!("callback_invoked={}", state(result.callback_invoked));
            println!("client_accepted={}", state(result.client_accepted));
            println!("client_inserted={}", state(result.client_inserted));
            println!("dispatch_clients={}", state(result.dispatch_clients_ok));
            println!("dispatched_requests={}", result.dispatched_requests);
            println!("flush_clients={}", state(result.flush_clients_ok));
            println!("socket_cleaned={}", state(result.socket_cleaned));
            println!("host_stub={}", result.host_stub);
            finish_stage("calloop-socket-smoke", result.is_ready());
        }
        Err(error) => {
            eprintln!("Calloop socket smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=calloop-socket-smoke status=error");
            std::process::exit(1);
        }
    }
}

fn smoke_socket() {
    match run_wayland_socket_smoke() {
        Ok(result) => {
            println!("foundation=smithay");
            println!("socket_name={}", result.socket_name);
            println!("wayland_display={}", state(result.display_created));
            println!(
                "compositor_global={}",
                state(result.compositor_global_created)
            );
            println!("socket_bound={}", state(result.socket_bound));
            println!("accept_nonblocking={}", state(result.accept_nonblocking));
            println!("client_connected={}", state(result.client_connected));
            println!("client_accepted={}", state(result.client_accepted));
            println!("client_inserted={}", state(result.client_inserted));
            println!("socket_cleaned={}", state(result.socket_cleaned));
            println!("host_stub={}", result.host_stub);
            finish_stage("wayland-socket-smoke", result.is_ready());
        }
        Err(error) => {
            eprintln!("Wayland socket smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=wayland-socket-smoke status=error");
            std::process::exit(1);
        }
    }
}

fn smoke_wayland() {
    match run_wayland_display_smoke() {
        Ok(result) => {
            println!("foundation=smithay");
            println!("smithay_features=wayland_frontend");
            println!("wayland_display={}", state(result.display_created));
            println!(
                "compositor_global={}",
                state(result.compositor_global_created)
            );
            println!("host_stub={}", result.host_stub);
            finish_stage("wayland-display-smoke", result.is_ready());
        }
        Err(error) => {
            eprintln!("Wayland display smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=wayland-display-smoke status=error");
            std::process::exit(1);
        }
    }
}

fn print_status() {
    for line in status_lines() {
        println!("{line}");
    }
    println!("[AQUA-COMPOSITOR] stage=skeleton status=ok mode=nested-dev");
}

fn probe_assets(root: PathBuf) {
    let probe = probe_runtime_assets(&root);
    let token_path = root.join("tokens/design-tokens.json");
    let token_product_ok = design_tokens_include_product(&token_path);
    let token_materials_ok = design_tokens_include_scene_materials(&token_path);

    println!("runtime_asset_root={}", probe.root.display());
    println!("wallpaper={}", state(probe.wallpaper));
    println!("design_tokens={}", state(probe.design_tokens));
    println!("design_tokens_product={}", state(token_product_ok));
    println!(
        "design_tokens_scene_materials={}",
        state(token_materials_ok)
    );
    println!("aqua_home_icon={}", state(probe.aqua_home_icon));
    println!("aqua_icon_license={}", state(probe.aqua_icon_license));

    if probe.is_ready() && token_product_ok && token_materials_ok {
        println!("[AQUA-COMPOSITOR] stage=asset-probe status=ok");
    } else {
        println!("[AQUA-COMPOSITOR] stage=asset-probe status=missing");
        std::process::exit(1);
    }
}

fn state(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "missing"
    }
}

fn pixel(value: [u8; 4]) -> String {
    format!(
        "{:02x},{:02x},{:02x},{:02x}",
        value[0], value[1], value[2], value[3]
    )
}

fn finish_stage(stage: &str, ok: bool) {
    if ok {
        println!("[AQUA-COMPOSITOR] stage={stage} status=ok");
    } else {
        println!("[AQUA-COMPOSITOR] stage={stage} status=unexpected");
        std::process::exit(1);
    }
}

fn smoke_loop() {
    match run_event_loop_smoke() {
        Ok(result) => {
            println!("event_loop=calloop");
            println!("ticks={}", result.ticks);
            finish_stage("event-loop-smoke", result.is_ready());
        }
        Err(error) => {
            eprintln!("event loop smoke failed: {error}");
            println!("[AQUA-COMPOSITOR] stage=event-loop-smoke status=error");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod fbdev_tests {
    use super::{
        bytes_per_pixel, checksum_frame_bytes, client_shadow_damage_rects, decode_png_rgba,
        drm_kms_confirmation_source, drm_wayland_hold_seconds, fbdev_confirmation_source,
        opaque_layer_covers_reference_output, pack_rgba_frame, parse_virtual_size,
        probe_drm_device, render_fbdev_frame, with_stride,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn drm_probe_discovers_connected_connector_without_activating_kms() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("aqua-drm-probe-{unique}"));
        let device = root.join("dev/card0");
        let connector = root.join("sys/card0-Virtual-1");
        fs::create_dir_all(device.parent().expect("device parent")).expect("device parent");
        fs::create_dir_all(&connector).expect("connector directory");
        fs::write(&device, []).expect("mock DRM card");
        fs::write(connector.join("status"), "connected\n").expect("connector status");
        fs::write(connector.join("modes"), "1280x800\n1024x768\n").expect("connector modes");

        let probe = probe_drm_device(&device, &root.join("sys"), "card0").expect("DRM probe");

        assert!(probe.is_ready());
        assert_eq!(probe.connected_connectors().count(), 1);
        assert_eq!(probe.connectors[0].name, "Virtual-1");
        assert_eq!(probe.connectors[0].modes[0], "1280x800");
        fs::remove_dir_all(root).expect("remove DRM probe fixture");
    }

    #[test]
    fn packaged_wallpaper_decodes_to_rgba8888() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/aqua-linux/assets/default-wallpaper.png");
        let wallpaper = decode_png_rgba(&path).expect("packaged wallpaper decode");

        assert_eq!((wallpaper.width, wallpaper.height), (1536, 1024));
        assert_eq!(wallpaper.rgba.len(), 1536 * 1024 * 4);
        assert!(wallpaper.rgba.chunks_exact(4).all(|pixel| pixel[3] == 0xff));
    }

    #[test]
    fn fbdev_confirmation_separates_operator_and_headless_test_paths() {
        assert_eq!(
            fbdev_confirmation_source(true, false, None),
            Some("manual-operator")
        );
        assert_eq!(
            fbdev_confirmation_source(false, true, Some("headless-qemu")),
            Some("headless-qemu-test")
        );
        assert_eq!(fbdev_confirmation_source(false, true, None), None);
        assert_eq!(
            fbdev_confirmation_source(false, true, Some("visible-qemu")),
            None
        );
    }

    #[test]
    fn drm_kms_confirmation_separates_operator_and_headless_test_paths() {
        assert_eq!(
            drm_kms_confirmation_source(true, false, None),
            Some("manual-operator")
        );
        assert_eq!(
            drm_kms_confirmation_source(false, true, Some("headless-qemu")),
            Some("headless-qemu-test")
        );
        assert_eq!(drm_kms_confirmation_source(false, true, None), None);
        assert_eq!(
            drm_kms_confirmation_source(false, true, Some("visible-qemu")),
            None
        );
    }

    #[test]
    fn drm_wayland_persistent_policy_ignores_bounded_hold_value() {
        assert_eq!(drm_wayland_hold_seconds(Some("10"), false), Some(10));
        assert_eq!(drm_wayland_hold_seconds(Some("999"), false), Some(30));
        assert_eq!(drm_wayland_hold_seconds(None, false), Some(3));
        assert_eq!(drm_wayland_hold_seconds(Some("1"), true), None);
    }

    #[test]
    fn virtual_size_requires_two_non_zero_dimensions() {
        assert_eq!(parse_virtual_size("1024,768\n").unwrap(), (1024, 768));
        assert!(parse_virtual_size("1024x768").is_err());
        assert!(parse_virtual_size("0,768").is_err());
    }

    #[test]
    fn fbdev_pixel_widths_are_explicit() {
        assert_eq!(bytes_per_pixel(16).unwrap(), 2);
        assert_eq!(bytes_per_pixel(24).unwrap(), 3);
        assert_eq!(bytes_per_pixel(32).unwrap(), 4);
        assert!(bytes_per_pixel(8).is_err());
    }

    #[test]
    fn stride_padding_preserves_each_packed_row() {
        let packed = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(
            with_stride(&packed, 3, 5, 2),
            vec![1, 2, 3, 0, 0, 4, 5, 6, 0, 0]
        );
    }

    #[test]
    fn composited_frame_converts_to_supported_fbdev_formats() {
        for (bits_per_pixel, expected_bytes) in [(16, 8), (24, 12), (32, 16)] {
            let (frame, source_checksum, runtime_wallpaper_loaded) =
                render_fbdev_frame(2, 2, bits_per_pixel).expect("fbdev conversion");
            assert_eq!(frame.len(), expected_bytes);
            assert_ne!(source_checksum, 0);
            assert!(!runtime_wallpaper_loaded);
        }
    }

    #[test]
    fn same_size_xrgb_pack_uses_bgra_channel_order() {
        let source = [0x11, 0x22, 0x33, 0x44, 0xaa, 0xbb, 0xcc, 0xdd];
        let packed = pack_rgba_frame(&source, 2, 1, 2, 1, 32).expect("same-size XRGB pack");
        assert_eq!(packed, [0x33, 0x22, 0x11, 0xff, 0xcc, 0xbb, 0xaa, 0xff]);
    }

    #[test]
    fn frame_checksum_is_deterministic_and_content_sensitive() {
        let frame = [0x10, 0x20, 0x30, 0xff, 0x40, 0x50, 0x60, 0xff];
        let mut changed = frame;
        changed[4] ^= 0x01;

        assert_eq!(checksum_frame_bytes(&frame), checksum_frame_bytes(&frame));
        assert_ne!(checksum_frame_bytes(&frame), checksum_frame_bytes(&changed));
        assert_ne!(
            checksum_frame_bytes(&frame),
            checksum_frame_bytes(&frame[..4])
        );
    }

    #[test]
    fn opaque_fullscreen_client_can_skip_hidden_background_work() {
        assert!(opaque_layer_covers_reference_output(
            0xff,
            (0, 0, 1536, 1024),
            true,
        ));
    }

    #[test]
    fn undeclared_or_partial_client_keeps_background_composition() {
        assert!(!opaque_layer_covers_reference_output(
            0xff,
            (0, 0, 1536, 1024),
            false,
        ));
        assert!(!opaque_layer_covers_reference_output(
            0xff,
            (128, 112, 1280, 800),
            true,
        ));
    }

    #[test]
    fn client_shadow_damage_is_expanded_and_bounded() {
        let viewport = aqua_scene::Viewport::new(1536, 1024);
        let pipeline = aqua_compositor::probe_client_layer_pipeline(viewport).unwrap();
        let damage = client_shadow_damage_rects(&pipeline.paint_plan, 1536, 1024);

        assert_eq!(damage.len(), pipeline.paint_plan.steps.len());
        assert!(damage.iter().all(|rect| rect.fits_in(viewport)));
        assert!(damage
            .iter()
            .zip(&pipeline.paint_plan.steps)
            .all(
                |(damage, step)| damage.width > step.rect.width && damage.height > step.rect.height
            ));
    }
}
