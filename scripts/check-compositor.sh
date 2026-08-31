#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
TEMP_ROOT="$(mktemp -d)"
TEMP_EXPORT="${TEMP_ROOT}/aqua-raster.ppm"
TEMP_PNG_EXPORT="${TEMP_ROOT}/aqua-raster.png"
TEMP_VISIBLE_PREVIEW_EXPORT="${TEMP_ROOT}/aqua-visible-preview.html"
TEMP_COMMAND_CACHE="${TEMP_ROOT}/command-cache"

cd "${ROOT_DIR}"

PRESENTATION_MODEL="crates/aqua-compositor/src/presentation.rs"
COMPOSITOR_MAIN="crates/aqua-compositor/src/main.rs"
test -f "${PRESENTATION_MODEL}"
grep -Fq 'pub struct R2PresentationReport' "${PRESENTATION_MODEL}"
grep -Fq 'pub struct PresentationTelemetry' "${PRESENTATION_MODEL}"
grep -Fq 'MAX_PRESENTATION_EVENTS' "${PRESENTATION_MODEL}"
grep -Fq 'PresentationPath::ProductionGbmKms' "${PRESENTATION_MODEL}"
grep -Fq 'PresentationPath::LegacyCpuCopy' "${PRESENTATION_MODEL}"
grep -Fq 'full_frame_readbacks == 0' "${PRESENTATION_MODEL}"
grep -Fq 'cpu_framebuffer_copies == 0' "${PRESENTATION_MODEL}"
grep -Fq 'page_flip_events == self.frames_presented' "${PRESENTATION_MODEL}"
grep -Fq 'repeating_repaint_timer_after_settle' "${PRESENTATION_MODEL}"
grep -Fq 'diagnostic_readback_isolated' "${PRESENTATION_MODEL}"
grep -Fq 'supports_release_claim' "${PRESENTATION_MODEL}"
grep -Fq 'AQUA_R2_PRESENTATION_TELEMETRY' "${COMPOSITOR_MAIN}"
grep -Fq 'env::var("AQUA_R2_PRESENTATION_TELEMETRY")' "${COMPOSITOR_MAIN}"
grep -Fq 'DrmPresentationEvent::FrameRequested' "${COMPOSITOR_MAIN}"
grep -Fq 'DrmPresentationEvent::PageFlipPresented' "${COMPOSITOR_MAIN}"
grep -Fq 'elapsed_micros_bounded(flip_started_at)' "${COMPOSITOR_MAIN}"
grep -Fq 'record_wayland_presentation_counters' "${COMPOSITOR_MAIN}"
grep -Fq 'record_live_idle_observation' "${COMPOSITOR_MAIN}"
grep -Fq 'input_to_present_latency_us' "${COMPOSITOR_MAIN}"
grep -Fq 'event.time_usec()' "${COMPOSITOR_MAIN}"
grep -Fq 'CLOCK_PROCESS_CPUTIME_ID' "${COMPOSITOR_MAIN}"
grep -Fq 'process_resident_memory_kib' "${COMPOSITOR_MAIN}"
grep -Fq 'record_resource_observation' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_frame_callbacks_sent=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_damage_commits=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_settled_idle_observations=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_settled_idle_repaints=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_repeating_repaint_timer_after_settle=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_input_to_present_samples=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_max_input_to_present_us=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_observation_window_ms=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_cpu_time_us=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_memory_growth_kib=' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_acceptance_complete=false' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_record_begin=v1' "${COMPOSITOR_MAIN}"
grep -Fq 'r2_presentation_record_end=v1' "${COMPOSITOR_MAIN}"

mkdir -p "${TEMP_ROOT}/etc/init.d" "${TEMP_ROOT}/usr/bin" "${TEMP_COMMAND_CACHE}"
touch "${TEMP_ROOT}/etc/init.d/rcS" "${TEMP_ROOT}/usr/bin/aqua-recovery"

br2-external/aqua/board/aqua/x86_64/post-build.sh "${TEMP_ROOT}"

check_output_contains() {
    expected="$1"
    shift
    cache_key="$(printf '%s\n' "$@" | cksum | awk '{print $1}')"
    cache_file="${TEMP_COMMAND_CACHE}/${cache_key}.txt"
    if [ ! -f "${cache_file}" ]; then
        "$@" > "${cache_file}"
    fi
    output="$(cat "${cache_file}")"
    printf '%s\n' "${output}" | grep -Fq "${expected}"
}

check_output_contains "foundation=smithay" cargo run -p aqua-compositor -- status
check_output_contains "smithay_features=wayland_frontend" cargo run -p aqua-compositor -- status
check_output_contains "event_loop=calloop" cargo run -p aqua-compositor -- status
check_output_contains "scene_model=aqua-scene" cargo run -p aqua-compositor -- status
check_output_contains "renderer=aqua-renderer" cargo run -p aqua-compositor -- status
check_output_contains "[AQUA-COMPOSITOR] stage=event-loop-smoke status=ok" cargo run -p aqua-compositor -- smoke-loop
check_output_contains "[AQUA-COMPOSITOR] stage=wayland-display-smoke status=ok" cargo run -p aqua-compositor -- smoke-wayland
check_output_contains "[AQUA-COMPOSITOR] stage=wayland-socket-smoke status=ok" cargo run -p aqua-compositor -- smoke-socket
check_output_contains "client_inserted=ok" cargo run -p aqua-compositor -- smoke-socket
check_output_contains "[AQUA-COMPOSITOR] stage=calloop-socket-smoke status=ok" cargo run -p aqua-compositor -- smoke-calloop-socket
check_output_contains "callback_invoked=ok" cargo run -p aqua-compositor -- smoke-calloop-socket
check_output_contains "dispatch_clients=ok" cargo run -p aqua-compositor -- smoke-calloop-socket
check_output_contains "flush_clients=ok" cargo run -p aqua-compositor -- smoke-calloop-socket
check_output_contains "[AQUA-COMPOSITOR] stage=session-config status=ok" cargo run -p aqua-compositor -- probe-session-config
check_output_contains "runtime_dir=/run/user/1000" cargo run -p aqua-compositor -- probe-session-config
check_output_contains "runtime_asset_root=/usr/share/aqua" cargo run -p aqua-compositor -- probe-session-config
check_output_contains "autostart=false" cargo run -p aqua-compositor -- probe-session-config
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-session-config
check_output_contains "recovery_tty_required=true" cargo run -p aqua-compositor -- probe-session-config
check_output_contains "source=file" cargo run -p aqua-compositor -- probe-session-config "${TEMP_ROOT}/etc/aqua/compositor-session.conf"
check_output_contains "[AQUA-COMPOSITOR] stage=session-config status=ok" cargo run -p aqua-compositor -- probe-session-config "${TEMP_ROOT}/etc/aqua/compositor-session.conf"
check_output_contains "[AQUA-COMPOSITOR] stage=session-env status=ok" cargo run -p aqua-compositor -- probe-session-env
check_output_contains "WAYLAND_DISPLAY=aqua-wayland-0" cargo run -p aqua-compositor -- probe-session-env
check_output_contains "XDG_RUNTIME_DIR=/run/user/1000" cargo run -p aqua-compositor -- probe-session-env
check_output_contains "AQUA_ASSET_ROOT=/usr/share/aqua" cargo run -p aqua-compositor -- probe-session-env
check_output_contains "source=file" cargo run -p aqua-compositor -- probe-session-env "${TEMP_ROOT}/etc/aqua/compositor-session.conf"
check_output_contains "[AQUA-COMPOSITOR] stage=session-env status=ok" cargo run -p aqua-compositor -- probe-session-env "${TEMP_ROOT}/etc/aqua/compositor-session.conf"
check_output_contains "[AQUA-COMPOSITOR] stage=session-bootstrap status=ok" cargo run -p aqua-compositor -- probe-session-bootstrap "${TEMP_ROOT}/etc/aqua/compositor-session.conf" "${TEMP_ROOT}/run/aqua"
check_output_contains "configured_runtime_dir=/run/user/1000" cargo run -p aqua-compositor -- probe-session-bootstrap "${TEMP_ROOT}/etc/aqua/compositor-session.conf" "${TEMP_ROOT}/run/user/1000"
check_output_contains "runtime_dir_prepared=ok" cargo run -p aqua-compositor -- probe-session-bootstrap "${TEMP_ROOT}/etc/aqua/compositor-session.conf" "${TEMP_ROOT}/run/aqua"
check_output_contains "autostart_blocked=ok" cargo run -p aqua-compositor -- probe-session-bootstrap "${TEMP_ROOT}/etc/aqua/compositor-session.conf" "${TEMP_ROOT}/run/aqua"
check_output_contains "boot_graphics_blocked=ok" cargo run -p aqua-compositor -- probe-session-bootstrap "${TEMP_ROOT}/etc/aqua/compositor-session.conf" "${TEMP_ROOT}/run/aqua"
check_output_contains "session_started=false" cargo run -p aqua-compositor -- probe-session-bootstrap "${TEMP_ROOT}/etc/aqua/compositor-session.conf" "${TEMP_ROOT}/run/aqua"
check_output_contains "[AQUA-COMPOSITOR] stage=session-skeleton status=ok" cargo run -p aqua-compositor -- probe-session
check_output_contains "compositor_state_owned=ok" cargo run -p aqua-compositor -- probe-session
check_output_contains "[AQUA-COMPOSITOR] stage=output-plan-probe status=ok" cargo run -p aqua-compositor -- probe-output-plan
check_output_contains "primary_backend=nested-dev-window" cargo run -p aqua-compositor -- probe-output-plan
check_output_contains "later_backend=qemu-drm-kms" cargo run -p aqua-compositor -- probe-output-plan
check_output_contains "output_size=1536x1024" cargo run -p aqua-compositor -- probe-output-plan
check_output_contains "recovery_safe=ok" cargo run -p aqua-compositor -- probe-output-plan
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-output-plan
check_output_contains "[AQUA-COMPOSITOR] stage=output-plan-dump status=ok" cargo run -p aqua-compositor -- dump-output-plan
check_output_contains "[AQUA-COMPOSITOR] stage=display-output-handoff status=ok" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "handoff_status=display-output-handoff-ready" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "target_backend=nested-dev-window" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "frame_buffer_bytes=6291456" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "frame_format=raw-rgba8888-composited-client-preview" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "frame_checksum=" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "preview_export_ready=ok" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "client_layer_pipeline_ready=ok" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "client_layer_composited=ok" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "client_layer_buffer_snapshot_bytes=674816" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "client_layer_snapshot_mode=full-buffer-snapshot" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "output_surface_prepared=ok" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "display_output_started=false" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "desktop_shell_started=false" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "recovery_safe=ok" cargo run -p aqua-compositor -- probe-display-output-handoff
check_output_contains "[AQUA-COMPOSITOR] stage=display-activation-plan status=ok" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "activation_status=manual-display-activation-plan-ready" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "launch_mode=manual-dev" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "source_handoff_ready=ok" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "target_backend=nested-dev-window" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "frame_format=raw-rgba8888-composited-client-preview" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "frame_checksum=" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "manual_start_required=true" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "fallback_tty_required=true" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "can_activate_display_output=ok" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "display_output_started=false" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "desktop_shell_started=false" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "autostart=false" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "recovery_safe=ok" cargo run -p aqua-compositor -- probe-display-activation-plan
check_output_contains "[AQUA-COMPOSITOR] stage=display-output-smoke status=ok" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "smoke_status=manual-display-output-smoke-complete" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "launch_mode=manual-dev" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "target_backend=nested-dev-window" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "presented_frames=3" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "display_output_started=true" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "display_output_stopped=true" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "manual_start_required=true" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "fallback_tty_available=true" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "desktop_shell_started=false" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "autostart=false" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "frame_format=raw-rgba8888-composited-client-preview" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "frame_checksum=" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "checksum_accumulator=" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "recovery_safe=ok" cargo run -p aqua-compositor -- smoke-display-output
check_output_contains "[AQUA-COMPOSITOR] stage=nested-output-surface status=ok" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "surface_status=nested-output-surface-lifecycle-complete" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "launch_mode=manual-dev" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "backend=nested-dev-window" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "surface_acquired=ok" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "surface_configured=ok" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "frame_attached=ok" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "frame_presented=ok" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "surface_released=ok" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "presented_frames=3" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "frame_checksum=" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "lifecycle_serial=1" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "manual_start_required=true" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "fallback_tty_available=true" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "autostart=false" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "desktop_shell_started=false" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "recovery_safe=ok" cargo run -p aqua-compositor -- smoke-nested-output-surface
check_output_contains "[AQUA-COMPOSITOR] stage=visible-preview-plan-probe status=ok" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "output_plan_ready=ok" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "frame_buffer_ready=ok" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "raster_ready=ok" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "png_export_ready=ok" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "client_layer_pipeline_ready=ok" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "client_layer_count=2" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "client_layer_checksum=" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "client_layer_buffer_snapshot_bytes=" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "client_layer_snapshot_mode=full-buffer-snapshot" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "preview_window_started=false" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-visible-preview-plan
check_output_contains "[AQUA-COMPOSITOR] stage=visible-preview-export-probe status=ok" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "format=html-data-uri-png-preview" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "html_bytes=" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "html_checksum=" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "client_layer_pipeline_ready=ok" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "client_layer_composited=ok" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "client_layer_count=2" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "client_layer_checksum=" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "client_layer_buffer_snapshot_bytes=" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "client_layer_snapshot_mode=full-buffer-snapshot" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "png_checksum=" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "preview_window_started=false" cargo run -p aqua-compositor -- probe-visible-preview-export
check_output_contains "[AQUA-COMPOSITOR] stage=visible-preview-export status=ok" cargo run -p aqua-compositor -- export-visible-preview-html "${TEMP_VISIBLE_PREVIEW_EXPORT}"
test -f "${TEMP_VISIBLE_PREVIEW_EXPORT}"
test "$(wc -c < "${TEMP_VISIBLE_PREVIEW_EXPORT}" | tr -d ' ')" -gt 0
check_output_contains "[AQUA-COMPOSITOR] stage=nested-preview-loop status=ok" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "launch_mode=manual-dev" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "window_backend=nested-dev-window" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "rendered_frames=3" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "frame_clock_started=ok" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "manual_start_required=true" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "autostart=false" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "preview_window_started=false" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- smoke-nested-preview-loop
check_output_contains "[AQUA-COMPOSITOR] stage=manual-nested-preview-backend status=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "backend_status=manual-nested-preview-backend-ready" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "launch_mode=manual-recovery" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "backend_path=nested-dev-window" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "backend_selected=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "handoff_ready=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "surface_lifecycle_ready=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "frame_loop_ready=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "visible_export_ready=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "frame_source=display-output-handoff-composited-client-frame" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "frame_format=raw-rgba8888-composited-client-preview" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "frame_checksum=" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "surface_frame_checksum=" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "loop_checksum_accumulator=" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "frame_checksum_matches_surface=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "fallback_tty_available=true" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "bounded_frame_limit=3" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "display_output_started=false" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "display_output_stopped=true" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "preview_window_started=false" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "desktop_shell_started=false" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "autostart=false" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "recovery_safe=ok" cargo run -p aqua-compositor -- probe-manual-nested-preview-backend
check_output_contains "[AQUA-COMPOSITOR] stage=manual-nested-preview-execution status=ok" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "execution_status=manual-nested-preview-execution-complete" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "launch_mode=manual-recovery" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "backend_path=nested-dev-window" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "operator_controlled=true" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "operator_ack_required=true" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "operator_acknowledged=true" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "backend_ready=ok" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "rendered_frames=3" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "frame_source=manual-nested-preview-backend-frame" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "frame_format=raw-rgba8888-composited-client-preview" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "display_output_started=true" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "display_output_stopped=true" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "preview_window_started=false" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "cleanup_complete=ok" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "fallback_tty_available=true" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "safe_return_to_recovery=ok" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "desktop_shell_started=false" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "autostart=false" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "recovery_safe=ok" cargo run -p aqua-compositor -- run-manual-nested-preview-execution
check_output_contains "[AQUA-COMPOSITOR] stage=client-window-model status=ok" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "model_status=client-window-model" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "window_count=2" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "focus_ready=ok" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "move_ready=ok" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "resize_ready=ok" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "close_ready=ok" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "stacking_ready=ok" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "chrome_ready=ok" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "real_wayland_client_started=false" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-client-window-model
check_output_contains "[AQUA-COMPOSITOR] stage=client-surface-lifecycle status=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "lifecycle_status=client-surface-lifecycle" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "surface_id=terminal-demo-surface" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "role=xdg-toplevel" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "step_count=7" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "configure_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "commit_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "map_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "focus_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "unmap_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "destroy_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "focus_bound_to_window=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "window_geometry_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "real_wayland_client_started=false" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-client-surface-lifecycle
check_output_contains "[AQUA-COMPOSITOR] stage=client-surface-registry status=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "registry_status=client-surface-registry" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "source_window_model_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "record_count=2" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "active_client_id=wayland-client-1" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "active_surface_id=xdg-toplevel-1" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "active_window_id=wayland-test-client" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "configure_serial_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "lifecycle_state_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "two_client_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "focus_index_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "stacking_order_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "close_request_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "buffer_metadata_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "buffer_import_plan_ready=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "no_renderer_binding=ok" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "record client=wayland-client-1 surface=xdg-toplevel-1 window=wayland-test-client title=Aqua Test Client role=xdg-toplevel lifecycle=mapped-focused z_index=2 configure_serial=1 configured=true committed=true mapped=true focused=true close_supported=true buffer_attached=true buffer_committed=true buffer=384x256 stride=1536 format=argb8888 source=client-committed-wl-shm import_required=true import_planned=true imported_for_sampling=true sample_checksum=" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "sample_pixel=" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "sample_grid=" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "buffer_snapshot_bytes=" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "record client=wayland-client-2 surface=xdg-toplevel-2 window=aqua-settings-client title=Aqua Settings role=xdg-toplevel lifecycle=mapped-unfocused z_index=1 configure_serial=2 configured=true committed=true mapped=true focused=false close_supported=true buffer_attached=true buffer_committed=true buffer=320x220 stride=1280 format=argb8888 source=client-committed-wl-shm import_required=true import_planned=true imported_for_sampling=true sample_checksum=" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-client-surface-registry
check_output_contains "[AQUA-COMPOSITOR] stage=renderer-surface-sources status=ok" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "surface_source_status=client-surface-sources-ready" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "source_registry_ready=ok" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "surface_source_count=2" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "expected_surface_sources=2" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "active_source_ready=ok" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "import_sources_ready=ok" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "z_order_ready=ok" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "source client=wayland-client-1 surface=xdg-toplevel-1 window=wayland-test-client z_index=2 focused=true buffer=384x256 stride=1536 format=argb8888 source=client-committed-wl-shm sample_checksum=" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "sample_pixel=" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "sample_grid=" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "buffer_snapshot_bytes=" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "source client=wayland-client-2 surface=xdg-toplevel-2 window=aqua-settings-client z_index=1 focused=false buffer=320x220 stride=1280 format=argb8888 source=client-committed-wl-shm sample_checksum=" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "renderer_import_ready=true" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-renderer-surface-sources
check_output_contains "[AQUA-COMPOSITOR] stage=client-layer-pipeline status=ok" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "client_layer_pipeline_status=ready" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "source_plan_ready=ok" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "paint_plan_ready=ok" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "raster_ready=ok" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "client_layer_count=2" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "expected_client_layers=2" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "active_layer_sample=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "inactive_layer_sample=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "client_layer_checksum=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "source_checksum_fold=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "client-layer-paint order=0 client=wayland-client-1 surface=xdg-toplevel-1 window=wayland-test-client rect=416,220,704,436 source_buffer=384x256 opacity=255 blend=source-over effect=sampled-wl-shm-client-buffer sample_checksum=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "sample_pixel=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "sample_grid=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "buffer_snapshot_bytes=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "client-layer-paint order=1 client=wayland-client-2 surface=xdg-toplevel-2 window=aqua-settings-client rect=464,248,704,436 source_buffer=320x220 opacity=255 blend=source-over effect=sampled-wl-shm-client-buffer sample_checksum=" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-client-layer-pipeline
check_output_contains "[AQUA-COMPOSITOR] stage=xdg-shell-binding status=ok" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "binding_status=xdg-shell-binding" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "foundation=smithay" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "protocol=xdg_wm_base" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "handler_bound=ok" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "global_created=ok" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "toplevel_callbacks_bound=ok" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "popup_callbacks_bound=ok" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "lifecycle_probe_ready=ok" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "real_wayland_client_started=false" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-xdg-shell-binding
check_output_contains "[AQUA-COMPOSITOR] stage=xdg-toplevel-client status=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "client_status=xdg-toplevel-client" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "client_connected=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "client_inserted=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "registry_bound=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "compositor_global_seen=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "shm_global_created=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "shm_global_seen=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "shm_buffer_created=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "client_buffer_attached=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "xdg_wm_base_global_seen=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "surface_created=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "toplevel_requested=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "surface_committed=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "server_buffer_attached=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "server_shm_buffer_imported=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "server_shm_buffer_sampled=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "shm_sample_checksum=" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "shm_sample_pixel=" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "shm_sample_grid=" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "shm_buffer_snapshot_bytes=" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "server_toplevel_created=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "server_configure_sent=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "client_configure_ack_sent=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "server_configure_ack_received=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "server_close_sent=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "client_close_event_received=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "test_wayland_client_started=true" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "test_wayland_client_count=2" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-xdg-toplevel-client
check_output_contains "[AQUA-COMPOSITOR] stage=selection-ownership status=ok" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "client_count=2" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "unfocused_clipboard_rejected=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "unfocused_primary_rejected=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "clipboard_offer_reaches_new_focus=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "primary_offer_reaches_new_focus=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "clipboard_mime_negotiated=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "primary_mime_negotiated=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "unsupported_mime_not_requested=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "clipboard_payload_transferred=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "primary_payload_transferred=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "compositor_buffers_payload=false" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "owner_disconnect_clears_clipboard=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "owner_disconnect_clears_primary=true" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "data_control_global_exposed=false" cargo run -p aqua-compositor -- probe-selection-ownership
check_output_contains "[AQUA-COMPOSITOR] stage=drag-and-drop status=ok" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "start_without_implicit_grab_rejected=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "enter_reaches_pointer_focus_only=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "keyboard_focus_unchanged=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "mime_negotiated=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "copy_action_negotiated=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "payload_transferred=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "drop_delivered_to_target=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "source_finished=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "rejected_drop_cancelled=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "rejected_drop_not_delivered=true" cargo run -p aqua-compositor -- probe-drag-and-drop
check_output_contains "[AQUA-COMPOSITOR] stage=text-input status=ok" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "input_method_hidden_from_normal_clients=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "input_method_visible_to_authorized_client=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "unfocused_enable_rejected=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "turkish_preedit_delivered=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "turkish_commit_delivered=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "serial_synchronized=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "focus_handoff_deactivates_input_method=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "popup_repositioned=true" cargo run -p aqua-compositor -- probe-text-input
check_output_contains "[AQUA-COMPOSITOR] stage=keyboard-locale-matrix status=ok" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "locale_count=3" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "keyboard_layout_count=3" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "supported_combination_count=9" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "keymaps_delivered_to_all_clients=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "keymaps_compile_for_all_layouts=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "representative_utf8_matches=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "compose_key_available_for_all_layouts=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "compose_case_count=9" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "compose_utf8_matches_for_all_clients=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "dead_key_layout_count=2" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "dead_key_case_count=6" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "dead_key_utf8_matches_for_all_clients=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "cancelled_compose_rejected_for_all_locales=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "repeat_info_matches=true" cargo run -p aqua-compositor -- probe-keyboard-locale-matrix
check_output_contains "[AQUA-COMPOSITOR] stage=independent-application-matrix status=ok" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "application_matrix_status=independent-application-matrix" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "client_count=2" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "simple_shm_app_id_seen=true" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "simple_damage_app_id_seen=true" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "damage_sequence_progressed=true" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "frame_callback_sequence_progressed=true" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "remaining_surface_count=0" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "host_stub=true" cargo run -p aqua-compositor -- probe-independent-application-matrix
check_output_contains "[AQUA-COMPOSITOR] stage=privileged-protocol-boundary status=ok" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "baseline_globals_visible_to_all_clients=true" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "input_method_hidden_from_normal_clients=true" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "input_method_visible_to_authorized_client=true" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "privileged_global_count=16" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "screenshot_global_exposed=false" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "screencopy_global_exposed=false" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "activation_global_exposed=false" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "authorized_scope_is_narrow=true" cargo run -p aqua-compositor -- probe-privileged-protocol-boundary
check_output_contains "[AQUA-COMPOSITOR] stage=v1-client-buffer-contract status=ok" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "application_model=first-party-wl-shm-v1" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "required_buffer_protocol=wl_shm" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "argb8888_visible_to_all_clients=true" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "linux_dmabuf_advertised=false" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "drm_syncobj_advertised=false" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "explicit_sync_advertised=false" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "accelerated_clients_supported=false" cargo run -p aqua-compositor -- probe-v1-client-buffer-contract
check_output_contains "[AQUA-COMPOSITOR] stage=wayland-output-matrix status=ok" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "output_count=4" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "declared_scale_count=4" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "declared_transform_count=4" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "outputs_visible_to_both_clients=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "modes_match_supported_matrix=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "logical_coordinates_match=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "fractional_scale_120ths=150" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "fractional_scales_match=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "transforms_match=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "viewport_destination_applied=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "hotplug_add_reaches_both_clients=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "hotplug_remove_reaches_both_clients=true" cargo run -p aqua-compositor -- probe-wayland-output-matrix
check_output_contains "[AQUA-COMPOSITOR] stage=xdg-toplevel-window-model status=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "window_model_status=xdg-toplevel-window-model" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "source_client_ready=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "server_surface_bound=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "window_model_bound=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "window_count=2" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "two_window_model_ready=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "stacking_ready=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "window_id=wayland-test-client" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "surface_id=xdg-toplevel-1" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "title=Aqua Test Client" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "role=xdg-toplevel" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "mapped=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "focused=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "geometry_ready=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "chrome_ready=ok" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "window id=aqua-settings-client title=Aqua Settings rect=464,248,704,436 z_index=1 focused=false closed=false chrome=aqua-window" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-xdg-toplevel-window-model
check_output_contains "[AQUA-COMPOSITOR] stage=launcher-model status=ok" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "launcher_status=interactive-launcher-model" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "design_era=bright-aqua-desktop" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "material=aqua-light-surface" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "category_count=9" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "app_count=6" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "search_result_count=1" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "selected_app_id=settings" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "launch_command=/usr/bin/aqua-settings" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "keyboard_wrap_ready=true" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "closed_activation_blocked=true" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "launcher_model_ready=ok" cargo run -p aqua-compositor -- probe-launcher-model
check_output_contains "[AQUA-COMPOSITOR] stage=launcher-input-scene status=ok" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "binding_status=launcher-input-scene-binding" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "input_source=compositor-seat-adapter-contract" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "initial_launcher_visible=false" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "opened_launcher_visible=true" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "dismissed_launcher_visible=false" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "open_draw_command_count=7" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "closed_draw_command_count=6" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "redraw_requests=3" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "visibility_changes=2" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "launch_request_app=settings" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "launcher_input_scene_ready=ok" cargo run -p aqua-compositor -- probe-launcher-input-scene
check_output_contains "[AQUA-COMPOSITOR] stage=smithay-launcher-seat status=ok" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "binding_status=smithay-launcher-seat-binding" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "seat_name=Aqua Seat" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "launcher_visible=true" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "selected_category=settings" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "draw_command_count=7" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "host_stub=true" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "smithay_launcher_seat_ready=ok" cargo run -p aqua-compositor -- probe-smithay-launcher-seat
check_output_contains "[AQUA-COMPOSITOR] stage=scene-probe status=ok" cargo run -p aqua-compositor -- probe-scene
check_output_contains "required_surfaces=7" cargo run -p aqua-compositor -- probe-scene
check_output_contains "required_assets_present=ok" cargo run -p aqua-compositor -- probe-scene
check_output_contains "permanent_assets_only=ok" cargo run -p aqua-compositor -- probe-scene
check_output_contains "required_material_tokens_present=ok" cargo run -p aqua-compositor -- probe-scene
check_output_contains "simulated_surface_labeled=ok" cargo run -p aqua-compositor -- probe-scene
check_output_contains "boot_graphics=false" cargo run -p aqua-compositor -- probe-scene
check_output_contains "[AQUA-COMPOSITOR] stage=scene-dump status=ok" cargo run -p aqua-compositor -- dump-scene
check_output_contains "surface id=launcher kind=launcher material=system-surface" cargo run -p aqua-compositor -- dump-scene
check_output_contains "surface id=notification-toast kind=notification-toast material=system-surface" cargo run -p aqua-compositor -- dump-scene
check_output_contains "asset surface=wallpaper role=background path=/usr/share/aqua/wallpapers/default-wallpaper.png temporary=false" cargo run -p aqua-compositor -- dump-scene
check_output_contains "asset surface=dock role=browser-icon path=/usr/share/aqua/icons/aqua/browser.svg temporary=false" cargo run -p aqua-compositor -- dump-scene
check_output_contains "material surface=launcher role=fill token=surface.panelFill" cargo run -p aqua-compositor -- dump-scene
check_output_contains "material surface=notification-toast role=effect token=surface.layeredSurface" cargo run -p aqua-compositor -- dump-scene
check_output_contains "[AQUA-COMPOSITOR] stage=render-plan-probe status=ok" cargo run -p aqua-compositor -- probe-render-plan
check_output_contains "draw_command_count=7" cargo run -p aqua-compositor -- probe-render-plan
check_output_contains "system_surface_commands_simulated=ok" cargo run -p aqua-compositor -- probe-render-plan
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-render-plan
check_output_contains "[AQUA-COMPOSITOR] stage=render-plan-dump status=ok" cargo run -p aqua-compositor -- dump-render-plan
check_output_contains "draw surface=launcher kind=system-surface-panel" cargo run -p aqua-compositor -- dump-render-plan
check_output_contains "[AQUA-COMPOSITOR] stage=paint-plan-probe status=ok" cargo run -p aqua-compositor -- probe-paint-plan
check_output_contains "paint_step_count=7" cargo run -p aqua-compositor -- probe-paint-plan
check_output_contains "system_surface_steps_translucent=ok" cargo run -p aqua-compositor -- probe-paint-plan
check_output_contains "paint_order_stable=ok" cargo run -p aqua-compositor -- probe-paint-plan
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-paint-plan
check_output_contains "[AQUA-COMPOSITOR] stage=paint-plan-dump status=ok" cargo run -p aqua-compositor -- dump-paint-plan
check_output_contains "paint order=4 surface=launcher kind=system-surface-panel" cargo run -p aqua-compositor -- dump-paint-plan
check_output_contains "[AQUA-COMPOSITOR] stage=frame-plan-probe status=ok" cargo run -p aqua-compositor -- probe-frame-plan
check_output_contains "frame_size=1536x1024" cargo run -p aqua-compositor -- probe-frame-plan
check_output_contains "pixel_format=rgba8888" cargo run -p aqua-compositor -- probe-frame-plan
check_output_contains "stride_ready=ok" cargo run -p aqua-compositor -- probe-frame-plan
check_output_contains "damage_ready=ok" cargo run -p aqua-compositor -- probe-frame-plan
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-frame-plan
check_output_contains "[AQUA-COMPOSITOR] stage=frame-plan-dump status=ok" cargo run -p aqua-compositor -- dump-frame-plan
check_output_contains "damage_rect=0,0,1536,1024" cargo run -p aqua-compositor -- dump-frame-plan
check_output_contains "[AQUA-COMPOSITOR] stage=frame-buffer-probe status=ok" cargo run -p aqua-compositor -- probe-frame-buffer
check_output_contains "buffer_status=allocated" cargo run -p aqua-compositor -- probe-frame-buffer
check_output_contains "buffer_bytes=6291456" cargo run -p aqua-compositor -- probe-frame-buffer
check_output_contains "allocated_bytes=6291456" cargo run -p aqua-compositor -- probe-frame-buffer
check_output_contains "first_pixel=00,17,25,ff" cargo run -p aqua-compositor -- probe-frame-buffer
check_output_contains "last_pixel=00,17,25,ff" cargo run -p aqua-compositor -- probe-frame-buffer
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-frame-buffer
check_output_contains "[AQUA-COMPOSITOR] stage=frame-buffer-dump status=ok" cargo run -p aqua-compositor -- dump-frame-buffer
check_output_contains "buffer_backend=headless-command-plan" cargo run -p aqua-compositor -- dump-frame-buffer
check_output_contains "[AQUA-COMPOSITOR] stage=raster-probe status=ok" cargo run -p aqua-compositor -- probe-raster
check_output_contains "raster_status=software-rasterized" cargo run -p aqua-compositor -- probe-raster
check_output_contains "filled_rect_count=7" cargo run -p aqua-compositor -- probe-raster
check_output_contains "wallpaper_sample=04,3b,5c,ff" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_sample=51,ac,d2,ff" cargo run -p aqua-compositor -- probe-raster
check_output_contains "dock_sample=51,ac,d2,ff" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_border_sample=3d,72,8c,ff" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_highlight_sample=a3,d3,e7,ff" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_corner_sample=2a,6c,8c,ff" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_shadow_sample=33,86,aa,ff" cargo run -p aqua-compositor -- probe-raster
check_output_contains "raster_checksum=701558d1539521df" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_primitive_count=15" cargo run -p aqua-compositor -- probe-raster
check_output_contains "checksum_ready=ok" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_corner_sample_ready=ok" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_shadow_sample_ready=ok" cargo run -p aqua-compositor -- probe-raster
check_output_contains "surface_primitives_ready=ok" cargo run -p aqua-compositor -- probe-raster
check_output_contains "renderer_started=false" cargo run -p aqua-compositor -- probe-raster
check_output_contains "[AQUA-COMPOSITOR] stage=raster-dump status=ok" cargo run -p aqua-compositor -- dump-raster
check_output_contains "raster_backend=headless-command-plan" cargo run -p aqua-compositor -- dump-raster
check_output_contains "[AQUA-COMPOSITOR] stage=raster-export-probe status=ok" cargo run -p aqua-compositor -- probe-raster-export
check_output_contains "export_format=ppm-p6-rgb888" cargo run -p aqua-compositor -- probe-raster-export
check_output_contains "export_bytes=4718609" cargo run -p aqua-compositor -- probe-raster-export
check_output_contains "export_checksum=efdcba78578c2cd5" cargo run -p aqua-compositor -- probe-raster-export
check_output_contains "[AQUA-COMPOSITOR] stage=raster-export-dump status=ok" cargo run -p aqua-compositor -- dump-raster-export
check_output_contains "[AQUA-COMPOSITOR] stage=raster-export status=ok" cargo run -p aqua-compositor -- export-raster-ppm "${TEMP_EXPORT}"
test -f "${TEMP_EXPORT}"
test "$(wc -c < "${TEMP_EXPORT}" | tr -d ' ')" = "4718609"
check_output_contains "[AQUA-COMPOSITOR] stage=raster-png-export-probe status=ok" cargo run -p aqua-compositor -- probe-raster-png-export
check_output_contains "export_format=png-rgba8888" cargo run -p aqua-compositor -- probe-raster-png-export
check_output_contains "export_bytes=6293028" cargo run -p aqua-compositor -- probe-raster-png-export
check_output_contains "export_checksum=2cdb1d86a1ba9300" cargo run -p aqua-compositor -- probe-raster-png-export
check_output_contains "[AQUA-COMPOSITOR] stage=raster-png-export-dump status=ok" cargo run -p aqua-compositor -- dump-raster-png-export
check_output_contains "[AQUA-COMPOSITOR] stage=raster-png-export status=ok" cargo run -p aqua-compositor -- export-raster-png "${TEMP_PNG_EXPORT}"
test -f "${TEMP_PNG_EXPORT}"
test "$(wc -c < "${TEMP_PNG_EXPORT}" | tr -d ' ')" = "6293028"
check_output_contains "[AQUA-COMPOSITOR] stage=session-run-once status=ok" cargo run -p aqua-compositor -- smoke-run-once
check_output_contains "run_once_called=ok" cargo run -p aqua-compositor -- smoke-run-once
check_output_contains "[AQUA-COMPOSITOR] stage=session-loop status=ok" cargo run -p aqua-compositor -- smoke-session-loop
check_output_contains "loop_iterations=3" cargo run -p aqua-compositor -- smoke-session-loop
check_output_contains "dispatch_passes=3" cargo run -p aqua-compositor -- smoke-session-loop
check_output_contains "flush_passes=3" cargo run -p aqua-compositor -- smoke-session-loop
check_output_contains "design_tokens_scene_materials=ok" cargo run -p aqua-compositor -- probe-assets "${TEMP_ROOT}/usr/share/aqua"
cargo run -p aqua-compositor -- probe-assets "${TEMP_ROOT}/usr/share/aqua"

if rustup target list --installed | grep -Fxq "x86_64-unknown-linux-musl"; then
    cargo check -p aqua-compositor --target x86_64-unknown-linux-musl
    echo "Smithay/libinput musl feature check uses scripts/build-compositor-linux-docker.sh with the Buildroot sysroot."
else
    echo "Skipping Linux target compositor check; install x86_64-unknown-linux-musl to enable it."
fi

echo "Aqua Linux compositor skeleton checks passed."
echo "Temporary probe root: ${TEMP_ROOT}"
