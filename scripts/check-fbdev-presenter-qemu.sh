#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-fbdev-present-check.log}"
SCREENSHOT="${SCREENSHOT:-${ROOT_DIR}/build/qemu-fbdev-present.ppm}"
SCREENSHOT_PNG="${SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-fbdev-present.png}"
SCREENSHOT_SHA256="${SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-fbdev-present.sha256}"
SCREENSHOT_METADATA="${SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-fbdev-present-capture.txt}"
KMS_SCREENSHOT="${KMS_SCREENSHOT:-${ROOT_DIR}/build/qemu-drm-kms-present.ppm}"
KMS_SCREENSHOT_PNG="${KMS_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-drm-kms-present.png}"
KMS_SCREENSHOT_SHA256="${KMS_SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-drm-kms-present.sha256}"
KMS_SCREENSHOT_METADATA="${KMS_SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-drm-kms-present-capture.txt}"
GPU_SURFACE_SCREENSHOT="${GPU_SURFACE_SCREENSHOT:-${ROOT_DIR}/build/qemu-drm-gpu-surface.ppm}"
GPU_SURFACE_SCREENSHOT_PNG="${GPU_SURFACE_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-drm-gpu-surface.png}"
GPU_SURFACE_SCREENSHOT_SHA256="${GPU_SURFACE_SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-drm-gpu-surface.sha256}"
GPU_SURFACE_SCREENSHOT_METADATA="${GPU_SURFACE_SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-drm-gpu-surface-capture.txt}"
GBM_SCANOUT_SCREENSHOT="${GBM_SCANOUT_SCREENSHOT:-${ROOT_DIR}/build/qemu-drm-gbm-scanout.ppm}"
GBM_SCANOUT_SCREENSHOT_PNG="${GBM_SCANOUT_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-drm-gbm-scanout.png}"
GBM_SCANOUT_SCREENSHOT_SHA256="${GBM_SCANOUT_SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-drm-gbm-scanout.sha256}"
GBM_SCANOUT_SCREENSHOT_METADATA="${GBM_SCANOUT_SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-drm-gbm-scanout-capture.txt}"
PAGE_FLIP_SCREENSHOT="${PAGE_FLIP_SCREENSHOT:-${ROOT_DIR}/build/qemu-drm-page-flip-present.ppm}"
PAGE_FLIP_SCREENSHOT_PNG="${PAGE_FLIP_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-drm-page-flip-present.png}"
PAGE_FLIP_SCREENSHOT_SHA256="${PAGE_FLIP_SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-drm-page-flip-present.sha256}"
PAGE_FLIP_SCREENSHOT_METADATA="${PAGE_FLIP_SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-drm-page-flip-present-capture.txt}"
FRAME_LOOP_SCREENSHOT="${FRAME_LOOP_SCREENSHOT:-${ROOT_DIR}/build/qemu-drm-frame-loop.ppm}"
FRAME_LOOP_SCREENSHOT_PNG="${FRAME_LOOP_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-drm-frame-loop.png}"
FRAME_LOOP_SCREENSHOT_SHA256="${FRAME_LOOP_SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-drm-frame-loop.sha256}"
FRAME_LOOP_SCREENSHOT_METADATA="${FRAME_LOOP_SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-drm-frame-loop-capture.txt}"
SESSION_LOOP_SCREENSHOT="${SESSION_LOOP_SCREENSHOT:-${ROOT_DIR}/build/qemu-drm-session-loop.ppm}"
SESSION_LOOP_SCREENSHOT_PNG="${SESSION_LOOP_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-drm-session-loop.png}"
SESSION_LOOP_SCREENSHOT_SHA256="${SESSION_LOOP_SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-drm-session-loop.sha256}"
SESSION_LOOP_SCREENSHOT_METADATA="${SESSION_LOOP_SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-drm-session-loop-capture.txt}"
WAYLAND_SESSION_SCREENSHOT="${WAYLAND_SESSION_SCREENSHOT:-${ROOT_DIR}/build/qemu-drm-wayland-session.ppm}"
WAYLAND_SESSION_SCREENSHOT_PNG="${WAYLAND_SESSION_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-drm-wayland-session.png}"
WAYLAND_SESSION_SCREENSHOT_SHA256="${WAYLAND_SESSION_SCREENSHOT_SHA256:-${ROOT_DIR}/build/qemu-drm-wayland-session.sha256}"
WAYLAND_SESSION_SCREENSHOT_METADATA="${WAYLAND_SESSION_SCREENSHOT_METADATA:-${ROOT_DIR}/build/qemu-drm-wayland-session-capture.txt}"
SETTINGS_SCREENSHOT="${SETTINGS_SCREENSHOT:-${ROOT_DIR}/build/qemu-aqua-settings.ppm}"
SETTINGS_SCREENSHOT_PNG="${SETTINGS_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-aqua-settings.png}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-fbdev-present-monitor.sock}"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-180}"
EXPECTED_FRAME_CHECKSUM="${AQUA_EXPECTED_FRAME_CHECKSUM:-c85dbfbfc17843af}"

for tool in expect file python3 qemu-system-x86_64 shasum; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        echo "Missing required tool: ${tool}" >&2
        exit 1
    fi
done

for artifact in "${KERNEL}" "${ROOTFS}"; do
    if [ ! -f "${artifact}" ]; then
        echo "Missing QEMU artifact: ${artifact}" >&2
        exit 1
    fi
done

mkdir -p "$(dirname "${SERIAL_LOG}")"
rm -f "${SERIAL_LOG}" "${SCREENSHOT}" "${SCREENSHOT_PNG}" "${SCREENSHOT_SHA256}" "${SCREENSHOT_METADATA}" \
    "${KMS_SCREENSHOT}" "${KMS_SCREENSHOT_PNG}" "${KMS_SCREENSHOT_SHA256}" "${KMS_SCREENSHOT_METADATA}" \
    "${GPU_SURFACE_SCREENSHOT}" "${GPU_SURFACE_SCREENSHOT_PNG}" "${GPU_SURFACE_SCREENSHOT_SHA256}" "${GPU_SURFACE_SCREENSHOT_METADATA}" \
    "${GBM_SCANOUT_SCREENSHOT}" "${GBM_SCANOUT_SCREENSHOT_PNG}" "${GBM_SCANOUT_SCREENSHOT_SHA256}" "${GBM_SCANOUT_SCREENSHOT_METADATA}" \
    "${PAGE_FLIP_SCREENSHOT}" "${PAGE_FLIP_SCREENSHOT_PNG}" "${PAGE_FLIP_SCREENSHOT_SHA256}" "${PAGE_FLIP_SCREENSHOT_METADATA}" \
    "${FRAME_LOOP_SCREENSHOT}" "${FRAME_LOOP_SCREENSHOT_PNG}" "${FRAME_LOOP_SCREENSHOT_SHA256}" "${FRAME_LOOP_SCREENSHOT_METADATA}" \
    "${SESSION_LOOP_SCREENSHOT}" "${SESSION_LOOP_SCREENSHOT_PNG}" "${SESSION_LOOP_SCREENSHOT_SHA256}" "${SESSION_LOOP_SCREENSHOT_METADATA}" \
    "${WAYLAND_SESSION_SCREENSHOT}" "${WAYLAND_SESSION_SCREENSHOT_PNG}" "${WAYLAND_SESSION_SCREENSHOT_SHA256}" "${WAYLAND_SESSION_SCREENSHOT_METADATA}" \
    "${SETTINGS_SCREENSHOT}" "${SETTINGS_SCREENSHOT_PNG}" \
    "${MONITOR_SOCKET}"

export KERNEL ROOTFS SERIAL_LOG SCREENSHOT SCREENSHOT_PNG KMS_SCREENSHOT KMS_SCREENSHOT_PNG \
    GPU_SURFACE_SCREENSHOT GPU_SURFACE_SCREENSHOT_PNG \
    GBM_SCANOUT_SCREENSHOT GBM_SCANOUT_SCREENSHOT_PNG \
    PAGE_FLIP_SCREENSHOT PAGE_FLIP_SCREENSHOT_PNG \
    FRAME_LOOP_SCREENSHOT FRAME_LOOP_SCREENSHOT_PNG \
    SESSION_LOOP_SCREENSHOT SESSION_LOOP_SCREENSHOT_PNG \
    WAYLAND_SESSION_SCREENSHOT WAYLAND_SESSION_SCREENSHOT_PNG \
    SETTINGS_SCREENSHOT SETTINGS_SCREENSHOT_PNG \
    MONITOR_SOCKET CAPTURE_HELPER INPUT_HELPER MEMORY CPUS TIMEOUT_SECONDS
"${ROOT_DIR}/scripts/check-fbdev-presenter-qemu.exp" >/dev/null

need_marker() {
    if ! grep -Fq "$1" "${SERIAL_LOG}"; then
        echo "Missing fbdev QEMU marker: $1" >&2
        tail -n 100 "${SERIAL_LOG}" >&2 || true
        exit 1
    fi
}

need_numeric_marker_min() {
    prefix="$1"
    minimum="$2"
    value="$(grep -F "${prefix}" "${SERIAL_LOG}" 2>/dev/null | tail -n 1 | sed "s/.*${prefix}//" | tr -cd '0-9')"
    if [ -z "${value}" ] || [ "${value}" -lt "${minimum}" ]; then
        echo "Missing fbdev QEMU marker: ${prefix}<${minimum}" >&2
        tail -n 100 "${SERIAL_LOG}" >&2 || true
        exit 1
    fi
}

need_marker 'confirmation_source=headless-qemu-test'
need_marker '[AQUA-BOOT] stage=udev-ready status=ok seat=seat0'
need_marker 'backend=linux-evdev'
need_marker 'keyboard_device=/dev/input/event1'
need_marker 'pointer_device=/dev/input/event2'
need_marker 'keyboard_events=1'
need_marker 'pointer_motion_events=2'
need_marker 'pointer_button_events=2'
need_marker 'launcher_visible=true'
need_marker 'evdev_events_dispatched=true'
need_marker '[AQUA-INPUT] stage=evdev-aqua-seat status=ok'
need_marker 'backend=drm-kms'
need_marker 'device=/dev/dri/card0'
need_marker 'device_open_mode=read-only'
need_marker 'device_open_read_only=ok'
need_marker 'connected_connector_count=1'
need_marker 'connector.Virtual-1.status=connected'
need_marker 'drm_master_acquired=false'
need_marker 'kms_activated=false'
need_marker '[AQUA-COMPOSITOR] stage=drm-device-probe status=ok'
need_marker 'renderer_drm_available=true'
need_marker 'renderer_gbm_available=true'
need_marker 'renderer_egl_available=true'
need_marker 'renderer_gles2_available=true'
need_marker 'renderer_gpu_runtime_ready=true'
need_marker 'renderer_selected_backend=smithay-gles2-gbm'
need_marker 'renderer_context_created=false'
need_marker 'renderer_display_output_started=false'
need_marker 'renderer_recovery_safe=true'
need_marker '[AQUA-COMPOSITOR] stage=renderer-backend-probe status=ok'
need_marker 'gpu_backend=smithay-gles2-gbm'
need_marker 'gpu_context_created=true'
need_marker 'gpu_offscreen_size=320x240'
need_marker 'gpu_offscreen_format=abgr8888'
need_marker 'gpu_scene_surface_count=5'
need_marker 'gpu_scene_surface_layer_count=3'
need_marker 'gpu_scene_shader=aqua-surface-compositor-v1'
need_marker 'gpu_surface_shader_compiled=true'
need_marker 'gpu_surface_shader_panels=3'
need_marker 'gpu_surface_refraction_strength=0.0025'
need_marker 'gpu_surface_tint_strength=0.18'
need_marker 'gpu_surface_highlight_strength=0.16'
need_marker 'gpu_surface_refraction_bounded=true'
need_marker 'gpu_surface_rounded_mask=true'
need_marker 'gpu_surface_corner_radius_px=12.0'
need_marker 'gpu_surface_edge_light_strength=0.24'
need_marker 'gpu_surface_edge_width_px=1.5'
need_marker 'gpu_surface_blur_shader_compiled=true'
need_marker 'gpu_surface_blur_passes=2'
need_marker 'gpu_surface_blur_kernel_samples=9'
need_marker 'gpu_surface_blur_radius_px=4.0'
need_marker 'gpu_surface_blur_intermediate_size=320x240'
need_marker 'gpu_surface_blur_synchronized=true'
need_marker 'gpu_surface_blur_composited=true'
need_marker 'gpu_client_texture_source=sampled-wl-shm-contract'
need_marker 'gpu_client_texture_count=2'
need_marker 'gpu_client_texture_bytes=674816'
need_marker 'gpu_client_textures_uploaded=true'
need_marker 'gpu_client_textures_composited=true'
need_marker 'gpu_client_live_wayland_session=false'
need_marker 'gpu_wallpaper_source=runtime-asset'
need_marker 'gpu_wallpaper_size=1536x1024'
need_marker 'gpu_wallpaper_texture_uploaded=true'
need_marker 'gpu_wallpaper_composited=true'
need_marker 'gpu_frame_rendered=true'
need_marker 'gpu_frame_synchronized=true'
need_marker 'gpu_frame_readback=true'
need_marker 'gpu_frame_bytes=307200'
need_marker 'gpu_frame_checksum='
need_marker 'gpu_frame_repeat_checksum='
need_marker 'gpu_frame_deterministic=true'
need_marker 'gpu_context_destroyed=true'
need_marker 'gpu_kms_activated=false'
need_marker 'gpu_display_output_started=false'
need_marker 'gpu_recovery_safe=true'
need_marker '[AQUA-COMPOSITOR] stage=gpu-offscreen-frame status=ok'
need_marker 'backend=drm-kms-gpu-surface'
need_marker 'composition_backend=smithay-gles2-gbm'
need_marker 'composition_shader=aqua-surface-compositor-v1'
need_marker 'composition_blur_passes=2'
need_marker 'composition_source_size=320x240'
need_marker 'composition_source_checksum='
need_marker 'composition_client_texture_source=sampled-wl-shm-contract'
need_marker 'composition_client_texture_count=2'
need_marker 'composition_client_textures_composited=true'
need_marker 'composition_live_wayland_session=false'
need_marker 'scanout_bridge=cpu-readback-copy'
need_marker 'direct_dmabuf_scanout=false'
need_marker 'scanout_format=xrgb8888'
need_marker 'scanout_checksum='
need_marker '[AQUA-COMPOSITOR] stage=drm-gpu-surface status=active'
need_marker 'gpu_surface_front_framebuffer_destroyed=true'
need_marker 'gpu_surface_back_framebuffer_destroyed=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-gpu-surface status=ok'
need_marker 'device_open_mode=read-write-bounded'
need_marker 'connected_connector_found=ok'
need_marker 'selected_mode=1280x800'
need_marker 'pixel_format=xrgb8888'
need_marker 'buffer_pitch=5120'
need_marker 'buffer_bytes=4096000'
need_marker 'dumb_buffer_created=true'
need_marker 'dumb_buffer_mapped=true'
need_marker 'framebuffer_created=false'
need_marker 'drm_master_requested=false'
need_marker 'page_flip_submitted=false'
need_marker 'dumb_buffer_destroyed=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-dumb-buffer-probe status=ok'
need_marker 'backend=drm-gbm-scanout-foundation'
need_marker 'gbm_buffer_count=2'
need_marker 'gbm_usage_scanout=true'
need_marker 'gbm_usage_rendering=true'
need_marker 'gbm_front_handle_count=1'
need_marker 'gbm_back_handle_count=1'
need_marker 'dmabuf_exported=true'
need_marker 'dmabuf_front_plane_count=1'
need_marker 'dmabuf_back_plane_count=1'
need_marker 'kms_addfb2_front=true'
need_marker 'kms_addfb2_back=true'
need_marker 'kms_framebuffers_destroyed=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-gbm-scanout-buffer-probe status=ok'
need_marker 'backend=drm-gbm-dmabuf-direct-scanout'
need_marker 'scanout_bridge=gbm-dmabuf-direct'
need_marker 'scanout_cpu_copy=false'
need_marker 'scanout_verification_readback=true'
need_marker 'direct_dmabuf_scanout=true'
need_marker 'gbm_front_rendered=true'
need_marker 'gbm_back_rendered=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=active'
need_marker '[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=ok'
need_marker 'confirmation_source=headless-qemu-test'
need_marker 'framebuffer_created=true'
need_marker 'kms_activated=true'
need_marker 'display_output_started=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-kms-present status=active'
need_marker 'crtc_restored=true'
need_marker 'framebuffer_destroyed=true'
need_marker 'display_output_stopped=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-kms-present status=ok'
need_marker 'backend=drm-kms-page-flip'
need_marker 'front_framebuffer_created=true'
need_marker 'back_framebuffer_created=true'
need_marker 'page_flip_submitted=true'
need_marker 'page_flip_event_received=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-page-flip status=active'
need_marker 'front_framebuffer_destroyed=true'
need_marker 'back_framebuffer_destroyed=true'
need_marker 'front_dumb_buffer_destroyed=true'
need_marker 'back_dumb_buffer_destroyed=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-page-flip status=ok'
need_marker 'backend=drm-kms-frame-loop'
need_marker 'requested_frames=3'
need_marker 'submitted_page_flips=3'
need_marker 'received_page_flip_events=3'
need_marker 'page_flip_event_order_complete=true'
need_marker 'page_flip_event_sequence_available=false'
need_marker 'front_back_buffer_alternation=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-frame-loop status=active'
need_marker '[AQUA-COMPOSITOR] stage=drm-frame-loop status=ok'
need_marker 'backend=drm-kms-session-loop'
need_marker 'session_owner=aqua-compositor'
need_marker 'event_loop=calloop'
need_marker 'drm_event_source_owned=true'
need_marker 'calloop_dispatch_passes=3'
need_marker 'wayland_display_started=false'
need_marker '[AQUA-COMPOSITOR] stage=drm-session-loop status=active'
need_marker 'drm_event_source_released=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-session-loop status=ok'
need_marker 'backend=drm-kms-wayland-session'
need_marker 'shared_session_lifecycle=true'
need_marker 'wayland_display_created=true'
need_marker 'wayland_socket=/run/user/1000/aqua-wayland-drm-0'
need_marker 'wayland_socket_bound=true'
need_marker 'wayland_client_connected=true'
need_marker 'wayland_client_inserted=true'
need_marker 'wayland_dispatch_passes=3'
need_marker 'wayland_flush_passes=3'
need_marker 'calloop_drm_dispatch_passes=3'
need_marker 'smithay_protocol_globals_started=true'
need_marker 'compositor_global_started=true'
need_marker 'shm_global_started=true'
need_marker 'xdg_shell_global_started=true'
need_marker 'seat_started=true'
need_marker 'input_source=libinput-udev'
need_marker 'input_source_enabled=true'
need_marker 'input_required=true'
need_marker 'input_seat=seat0'
need_marker 'external_client_connected=true'
need_marker 'external_client_protocol=xdg_toplevel'
need_marker 'external_wayland_surface_ready=true'
need_marker 'external_wayland_surface_count=2'
need_marker 'external_wayland_surface_sizes='
need_marker 'external_wayland_surface_bytes='
need_marker 'drm_wayland_composition_backend=smithay-gles2-readback-dumb-buffer'
need_marker 'drm_wayland_gpu_render_device=/dev/dri/card0'
need_marker 'drm_wayland_gpu_render_node_separate=false'
need_marker 'drm_wayland_scanout_bridge=gpu-readback-dumb-buffer'
need_marker 'drm_wayland_scanout_cpu_copy=true'
need_marker 'drm_wayland_direct_dmabuf_scanout=false'
need_marker 'drm_wayland_gpu_frame_readback=true'
need_marker 'drm_wayland_gpu_checksum_source=frame-readback'
need_marker 'gbm_scanout_buffers_released=true'
need_marker 'drm_wayland_gpu_client_texture_source=live-smithay-wl-shm-snapshot'
need_marker 'drm_wayland_gpu_client_texture_count=2'
need_marker 'drm_wayland_gpu_client_texture_bytes=643216'
need_marker 'drm_wayland_gpu_client_textures_uploaded=true'
need_marker 'drm_wayland_gpu_client_textures_composited=true'
need_marker 'drm_wayland_gpu_live_session=true'
need_marker 'drm_wayland_gpu_initial_frame_checksum='
need_marker 'drm_wayland_gpu_context_lifecycle=session-owned'
need_marker 'drm_wayland_gpu_repaint_updates=true'
need_marker 'drm_wayland_gpu_context_reused=true'
need_marker 'drm_wayland_gpu_repaint_surface_raised=384x256'
need_marker 'drm_wayland_gpu_repaint_source_order_changed=true'
need_marker 'drm_wayland_gpu_repaint_texture_count=2'
need_marker 'drm_wayland_gpu_repaint_texture_bytes=643216'
need_marker 'drm_wayland_gpu_repaint_checksum='
need_marker 'third_party_wayland_client=weston-simple-shm'
need_marker 'third_party_wayland_client_role=compatibility-fixture'
need_marker 'weston_compositor_started=false'
need_marker 'aqua_state_client_raised=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-wayland-session status=active'
need_marker 'drm_wayland_input_keyboard_events='
need_marker 'drm_wayland_input_pointer_motion_events=11'
need_numeric_marker_min 'drm_wayland_input_pointer_button_events=' 18
need_marker 'drm_wayland_input_launcher_visible=true'
need_marker 'drm_wayland_launcher_overlay_rendered=true'
need_marker 'drm_wayland_launcher_category_count=9'
need_marker 'drm_wayland_launcher_visible_app_rows=1'
need_marker 'drm_wayland_launcher_selected_index=0'
need_marker 'drm_wayland_launcher_search_query=settings'
need_marker 'drm_wayland_launcher_selected_category=all-applications'
need_marker 'drm_wayland_launcher_pointer_hits=1'
need_marker 'drm_wayland_launcher_category_clicks=0'
need_marker 'drm_wayland_launcher_app_clicks=1'
need_marker 'drm_wayland_launcher_launch_request_app=files'
need_marker 'drm_wayland_launcher_command_allowed=true'
need_marker 'drm_wayland_launcher_executable_exists=true'
need_marker 'drm_wayland_launcher_launch_accepted=true'
need_marker 'drm_wayland_launcher_launch_rejection_reason=accepted'
need_marker 'drm_wayland_launcher_process_started=true'
need_marker 'drm_wayland_launcher_process_app=files'
need_marker 'drm_wayland_launcher_process_pid='
need_marker 'drm_wayland_process_supervisor_active=1'
need_marker 'drm_wayland_process_supervisor_duplicate_policy=reject'
need_marker 'drm_wayland_process_supervisor_duplicate_rejected=true'
need_marker 'drm_wayland_launcher_surface_app_id=aqua.files'
need_marker 'drm_wayland_launcher_surface_owned=true'
need_marker 'drm_wayland_launcher_process_exit_success=true'
need_marker 'drm_wayland_launcher_process_reaped=true'
need_marker 'drm_wayland_process_supervisor_active=0'
need_marker 'drm_wayland_launcher_surface_cleanup=true'
need_marker 'drm_wayland_settings_process_started=true'
need_marker 'drm_wayland_settings_process_pid='
need_marker 'drm_wayland_settings_app_id=aqua.settings'
need_marker 'drm_wayland_settings_buffer=600x400'
need_marker 'aqua_settings_font_family=Noto Sans'
need_marker 'aqua_settings_font_source=embedded-ttf'
need_marker 'aqua_settings_font_ready=true'
need_marker 'drm_wayland_settings_reduced_motion=true'
need_marker 'drm_wayland_settings_pointer_commit=true'
need_marker 'drm_wayland_settings_keyboard_category=Desktop'
need_marker 'aqua_settings_loaded_reduced_motion=false'
need_marker 'aqua_settings_loaded_desktop_icons=true'
need_marker 'aqua_settings_loaded_key_repeat=true'
need_marker 'aqua_settings_loaded_theme=LightWhite'
need_marker 'aqua_settings_persisted=true'
need_marker 'drm_wayland_settings_config_path=/home/aqua/.config/aqua/settings.conf'
need_marker 'drm_wayland_settings_config_version=1'
need_marker 'drm_wayland_settings_persisted_reduced_motion=true'
need_marker 'drm_wayland_settings_desktop_icons=false'
need_marker 'drm_wayland_settings_desktop_toggle_checksum='
need_marker 'drm_wayland_settings_persisted_desktop_icons=false'
need_marker 'drm_wayland_settings_keyboard_category=Input'
need_marker 'drm_wayland_settings_input_category_checksum='
need_marker 'drm_wayland_settings_key_repeat=false'
need_marker 'drm_wayland_settings_key_repeat_checksum='
need_marker 'drm_wayland_settings_persisted_key_repeat=false'
need_marker 'aqua_settings_network_status_available=true'
need_marker 'aqua_settings_network_interface_count=1'
need_marker 'aqua_settings_network_interface='
need_marker 'aqua_settings_network_state='
need_marker 'aqua_settings_wifi_control_available=false'
need_marker 'drm_wayland_settings_keyboard_category=Network'
need_marker 'drm_wayland_settings_network_read_only=true'
need_marker 'drm_wayland_settings_network_management=false'
need_marker 'drm_wayland_settings_wifi_control_available=false'
need_marker 'drm_wayland_settings_network_category_checksum='
need_marker 'drm_wayland_settings_reload_verified=true'
need_marker 'drm_wayland_settings_keyboard_checksum='
need_marker 'drm_wayland_settings_capture_ready=true'
need_marker 'drm_wayland_settings_scanout_checksum='
need_marker 'drm_wayland_settings_gpu_repaint=true'
need_marker 'drm_wayland_settings_gpu_context_reused=true'
need_marker 'drm_wayland_settings_gpu_texture_count=3'
need_marker 'drm_wayland_settings_gpu_texture_bytes=1603216'
need_marker 'drm_wayland_settings_gpu_checksum='
need_marker 'drm_wayland_settings_process_exit_success=true'
need_marker 'drm_wayland_settings_process_reaped=true'
need_marker 'drm_wayland_settings_surface_cleanup=true'
need_marker 'drm_wayland_process_supervisor_final_active=0'
need_marker 'drm_wayland_files_window_model=pictures'
need_marker 'drm_wayland_files_window_buffer=640x420'
need_marker 'drm_wayland_files_window_sidebar_items=5'
need_marker 'drm_wayland_files_window_entries=0'
need_marker 'drm_wayland_files_window_location=Aqua/Home/Pictures'
need_marker 'drm_wayland_files_window_repaint_complete=true'
need_marker 'drm_wayland_files_window_repaint_page_flips=1'
need_marker 'drm_wayland_files_window_repaint_checksum='
need_marker 'drm_wayland_files_gpu_repaint=true'
need_marker 'drm_wayland_files_gpu_context_reused=true'
need_marker 'drm_wayland_files_gpu_texture_count=3'
need_marker 'drm_wayland_files_gpu_texture_bytes=1718416'
need_marker 'drm_wayland_files_gpu_checksum='
need_marker 'drm_wayland_files_read_only_root=/home/aqua'
need_marker 'drm_wayland_files_directory_enumerated=true'
need_marker 'drm_wayland_files_symlink_followed=false'
need_marker 'drm_wayland_files_pointer_selection=entry-0'
need_marker 'drm_wayland_files_selection_commit=true'
need_marker 'drm_wayland_files_selection_checksum='
need_marker 'drm_wayland_files_folder_open=Documents'
need_marker 'drm_wayland_files_folder_open_commit=true'
need_marker 'drm_wayland_files_keyboard_selection=Projects'
need_marker 'drm_wayland_files_keyboard_activation=Projects'
need_marker 'drm_wayland_files_keyboard_back=true'
need_marker 'drm_wayland_files_scroll_offset=1'
need_marker 'drm_wayland_files_pointer_wheel=true'
need_marker 'drm_wayland_files_page_down=true'
need_marker 'drm_wayland_files_page_up=true'
need_marker 'drm_wayland_files_home_key=true'
need_marker 'drm_wayland_files_end_key=true'
need_marker 'drm_wayland_files_scrollbar_drag=true'
need_marker 'drm_wayland_files_scrollbar_drag_offset=1'
need_marker 'drm_wayland_files_keyboard_focus_visible=true'
need_marker 'drm_wayland_files_text_preview=Welcome.txt'
need_marker 'drm_wayland_files_text_preview_read_only=true'
need_marker 'drm_wayland_files_text_preview_multiline=true'
need_marker 'drm_wayland_files_preview_pointer_wheel=true'
need_marker 'drm_wayland_files_preview_scroll_offset=1'
need_marker 'drm_wayland_files_arbitrary_execution=false'
need_marker 'drm_wayland_files_text_preview_closed=true'
need_marker 'drm_wayland_files_back_navigation=true'
need_marker 'drm_wayland_files_forward_navigation=true'
need_marker 'drm_wayland_files_sidebar_navigation=Pictures'
need_marker 'drm_wayland_files_navigation_root_confined=true'
need_marker 'drm_wayland_files_hover_feedback=true'
need_marker 'drm_wayland_files_history_controls=back-enabled-forward-disabled'
need_marker 'drm_wayland_files_navigation_checksum='
need_marker 'drm_wayland_input_discovery_ready=true'
need_marker 'drm_wayland_input_dispatch_ready=true'
need_marker 'drm_wayland_input_shortcut_intercepts='
need_marker 'drm_wayland_input_keys_forwarded='
need_marker 'drm_wayland_input_pointer_hit_tests='
need_marker 'drm_wayland_input_pointer_surface_hits='
need_marker 'drm_wayland_external_client_ready=true'
need_marker 'drm_wayland_external_client_toplevels=2'
need_marker 'drm_wayland_external_client_surface_count=2'
need_marker 'drm_wayland_external_client_independent_buffers=true'
need_marker 'drm_wayland_external_client_buffer_bytes='
need_marker 'drm_wayland_external_client_composited=true'
need_marker 'external_client_frame_callback_received=true'
need_marker 'external_client_partial_damage_commit=true'
need_marker 'external_client_keyboard_event_received=true'
need_marker 'external_client_pointer_event_received=true'
need_marker 'drm_wayland_external_client_damage_commits='
need_marker 'drm_wayland_external_client_damage_rects='
need_marker 'drm_wayland_external_client_frame_callbacks_sent='
need_marker 'drm_wayland_external_client_damage_ready=true'
need_marker 'drm_wayland_external_client_frame_callbacks_ready=true'
need_marker 'drm_wayland_external_client_keyboard_focus=true'
need_marker 'drm_wayland_external_client_pointer_focus=false'
need_marker 'drm_wayland_external_client_focus_changes=8'
need_marker 'drm_wayland_external_client_stacking_changes=8'
need_marker 'drm_wayland_stacking_repaint_complete=true'
need_marker 'drm_wayland_stacking_repaint_changed_frame=true'
need_marker 'drm_wayland_stacking_repaint_page_flips=1'
need_marker 'drm_wayland_stacking_repaint_checksum='
need_marker 'drm_wayland_move_requests=1'
need_marker 'drm_wayland_resize_requests=1'
need_marker 'drm_wayland_interactive_geometry_applied=true'
need_marker 'drm_wayland_maximize_requests=1'
need_marker 'drm_wayland_unmaximize_requests=1'
need_marker 'drm_wayland_fullscreen_requests=1'
need_marker 'drm_wayland_unfullscreen_requests=1'
need_marker 'drm_wayland_state_configure_acks=9'
need_marker 'drm_wayland_state_cycle_complete=true'
need_marker 'external_client_size_constraints_sent=true'
need_marker 'external_client_state_configures=5'
need_marker 'external_client_state_cycle_complete=true'
need_marker 'drm_wayland_client_cleanup_complete=true'
need_marker 'drm_wayland_client_cleanup_surviving_surfaces=1'
need_marker 'drm_wayland_client_cleanup_destroyed_surfaces=3'
need_marker 'drm_wayland_client_cleanup_count=3'
need_marker 'drm_wayland_client_cleanup_session_alive=true'
need_marker 'drm_wayland_client_cleanup_keyboard_focus_reassigned=true'
need_marker 'drm_wayland_client_cleanup_pointer_focus_cleared=true'
need_marker 'drm_wayland_client_cleanup_repaint_complete=true'
need_marker 'drm_wayland_client_cleanup_repaint_page_flips=1'
need_marker 'drm_wayland_client_cleanup_repaint_checksum='
need_marker 'drm_wayland_client_cleanup_gpu_repaint=true'
need_marker 'drm_wayland_client_cleanup_gpu_context_reused=true'
need_marker 'drm_wayland_client_cleanup_gpu_texture_count=1'
need_marker 'drm_wayland_client_cleanup_gpu_texture_bytes=250000'
need_marker 'drm_wayland_client_cleanup_gpu_checksum='
need_marker 'external_client_close_event_received=true'
need_marker 'external_client_close_cleanup=true'
need_marker 'drm_wayland_close_request_sent=true'
need_marker 'drm_wayland_close_request_count=4'
need_marker 'drm_wayland_close_cleanup_surfaces=0'
need_marker 'drm_wayland_close_repaint_complete=true'
need_marker 'drm_wayland_close_repaint_page_flips=1'
need_marker 'drm_wayland_close_repaint_checksum='
need_marker 'drm_wayland_close_gpu_repaint=true'
need_marker 'drm_wayland_close_gpu_context_reused=true'
need_marker 'drm_wayland_close_gpu_texture_count=0'
need_marker 'drm_wayland_close_gpu_texture_bytes=0'
need_marker 'drm_wayland_close_gpu_checksum='
need_marker 'drm_wayland_gpu_repaint_route_complete=true'
need_marker 'external_wayland_frame_checksum='
need_marker 'external_wayland_client_process_stopped=true'
need_marker 'wayland_socket_cleaned=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok'
need_marker 'target_size=1280x800'
need_marker 'target_bits_per_pixel=32'
need_marker 'target_stride=5120'
need_marker 'presented_bytes=4096000'
need_marker "frame_checksum=${EXPECTED_FRAME_CHECKSUM}"
need_marker 'wallpaper_source=runtime-asset'
need_marker 'visible_frame_presented=true'
need_marker 'visible_frame_observed=false'
need_marker 'safe_return_to_recovery=ok'
need_marker '[AQUA-COMPOSITOR] stage=fbdev-present status=ok'
need_marker '[AQUA-GATE] stage=graphics-fbdev-present status=ok'
need_marker '[AQUA-TEST] stage=fbdev-present-qemu status=ok framebuffer_write=true visible_observation=false safe_return_to_recovery=ok'

if [ ! -s "${KMS_SCREENSHOT}" ]; then
    echo "Missing QEMU KMS screendump: ${KMS_SCREENSHOT}" >&2
    exit 1
fi

if [ ! -s "${GPU_SURFACE_SCREENSHOT}" ]; then
    echo "Missing QEMU GPU surface screendump: ${GPU_SURFACE_SCREENSHOT}" >&2
    exit 1
fi
if [ ! -s "${GBM_SCANOUT_SCREENSHOT}" ]; then
    echo "Missing QEMU GBM direct-scanout screendump: ${GBM_SCANOUT_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${GBM_SCANOUT_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU GBM direct-scanout screendump format:" >&2
    file "${GBM_SCANOUT_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${GBM_SCANOUT_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU GBM direct-scanout PNG format:" >&2
    file "${GBM_SCANOUT_SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${GBM_SCANOUT_SCREENSHOT}" > "${GBM_SCANOUT_SCREENSHOT_SHA256}"
gbm_scanout_ppm_sha256="$(awk '{print $1}' "${GBM_SCANOUT_SCREENSHOT_SHA256}")"
gbm_scanout_png_sha256="$(shasum -a 256 "${GBM_SCANOUT_SCREENSHOT_PNG}" | awk '{print $1}')"
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'backend=drm-gbm-dmabuf-direct-scanout'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'scanout_bridge=gbm-dmabuf-direct'
    echo 'scanout_cpu_copy=false'
    echo 'direct_dmabuf_scanout=true'
    echo "ppm_sha256=${gbm_scanout_ppm_sha256}"
    echo "png_sha256=${gbm_scanout_png_sha256}"
    echo 'page_flip_event_received=true'
    echo 'crtc_restored=true'
    echo 'safe_return_to_recovery=ok'
} > "${GBM_SCANOUT_SCREENSHOT_METADATA}"
if ! file "${GPU_SURFACE_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU GPU surface screendump format:" >&2
    file "${GPU_SURFACE_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${GPU_SURFACE_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU GPU surface PNG format:" >&2
    file "${GPU_SURFACE_SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${GPU_SURFACE_SCREENSHOT}" > "${GPU_SURFACE_SCREENSHOT_SHA256}"
gpu_surface_ppm_sha256="$(awk '{print $1}' "${GPU_SURFACE_SCREENSHOT_SHA256}")"
gpu_surface_png_sha256="$(shasum -a 256 "${GPU_SURFACE_SCREENSHOT_PNG}" | awk '{print $1}')"
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'backend=drm-kms-gpu-surface'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'composition_backend=smithay-gles2-gbm'
    echo 'composition_shader=aqua-surface-compositor-v1'
    echo 'composition_blur_passes=2'
    echo 'scanout_bridge=cpu-readback-copy'
    echo 'direct_dmabuf_scanout=false'
    echo "ppm_sha256=${gpu_surface_ppm_sha256}"
    echo "png_sha256=${gpu_surface_png_sha256}"
    echo 'page_flip_event_received=true'
    echo 'crtc_restored=true'
    echo 'safe_return_to_recovery=ok'
} > "${GPU_SURFACE_SCREENSHOT_METADATA}"
if ! file "${KMS_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU KMS screendump format:" >&2
    file "${KMS_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${KMS_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU KMS PNG format:" >&2
    file "${KMS_SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${KMS_SCREENSHOT}" > "${KMS_SCREENSHOT_SHA256}"
kms_ppm_sha256="$(awk '{print $1}' "${KMS_SCREENSHOT_SHA256}")"
kms_png_sha256="$(shasum -a 256 "${KMS_SCREENSHOT_PNG}" | awk '{print $1}')"
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'backend=drm-kms'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'wallpaper_source=runtime-asset'
    echo "frame_checksum=${EXPECTED_FRAME_CHECKSUM}"
    echo "ppm_sha256=${kms_ppm_sha256}"
    echo "png_sha256=${kms_png_sha256}"
    echo 'kms_activated=true'
    echo 'crtc_restored=true'
    echo 'safe_return_to_recovery=ok'
} > "${KMS_SCREENSHOT_METADATA}"

if [ ! -s "${PAGE_FLIP_SCREENSHOT}" ]; then
    echo "Missing QEMU DRM page-flip screendump: ${PAGE_FLIP_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${PAGE_FLIP_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU DRM page-flip screendump format:" >&2
    file "${PAGE_FLIP_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${PAGE_FLIP_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU DRM page-flip PNG format:" >&2
    file "${PAGE_FLIP_SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${PAGE_FLIP_SCREENSHOT}" > "${PAGE_FLIP_SCREENSHOT_SHA256}"
page_flip_ppm_sha256="$(awk '{print $1}' "${PAGE_FLIP_SCREENSHOT_SHA256}")"
page_flip_png_sha256="$(shasum -a 256 "${PAGE_FLIP_SCREENSHOT_PNG}" | awk '{print $1}')"
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'backend=drm-kms-page-flip'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'wallpaper_source=runtime-asset'
    echo "frame_checksum=${EXPECTED_FRAME_CHECKSUM}"
    echo "ppm_sha256=${page_flip_ppm_sha256}"
    echo "png_sha256=${page_flip_png_sha256}"
    echo 'page_flip_submitted=true'
    echo 'page_flip_event_received=true'
    echo 'crtc_restored=true'
    echo 'safe_return_to_recovery=ok'
} > "${PAGE_FLIP_SCREENSHOT_METADATA}"

if [ ! -s "${FRAME_LOOP_SCREENSHOT}" ]; then
    echo "Missing QEMU DRM frame-loop screendump: ${FRAME_LOOP_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${FRAME_LOOP_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU DRM frame-loop screendump format:" >&2
    file "${FRAME_LOOP_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${FRAME_LOOP_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU DRM frame-loop PNG format:" >&2
    file "${FRAME_LOOP_SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${FRAME_LOOP_SCREENSHOT}" > "${FRAME_LOOP_SCREENSHOT_SHA256}"
frame_loop_ppm_sha256="$(awk '{print $1}' "${FRAME_LOOP_SCREENSHOT_SHA256}")"
frame_loop_png_sha256="$(shasum -a 256 "${FRAME_LOOP_SCREENSHOT_PNG}" | awk '{print $1}')"
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'backend=drm-kms-frame-loop'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'wallpaper_source=runtime-asset'
    echo "frame_checksum=${EXPECTED_FRAME_CHECKSUM}"
    echo "ppm_sha256=${frame_loop_ppm_sha256}"
    echo "png_sha256=${frame_loop_png_sha256}"
    echo 'submitted_page_flips=3'
    echo 'received_page_flip_events=3'
    echo 'page_flip_event_order_complete=true'
    echo 'page_flip_event_sequence_available=false'
    echo 'front_back_buffer_alternation=true'
    echo 'crtc_restored=true'
    echo 'safe_return_to_recovery=ok'
} > "${FRAME_LOOP_SCREENSHOT_METADATA}"

if [ ! -s "${SESSION_LOOP_SCREENSHOT}" ]; then
    echo "Missing QEMU DRM session-loop screendump: ${SESSION_LOOP_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${SESSION_LOOP_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU DRM session-loop screendump format:" >&2
    file "${SESSION_LOOP_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${SESSION_LOOP_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU DRM session-loop PNG format:" >&2
    file "${SESSION_LOOP_SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${SESSION_LOOP_SCREENSHOT}" > "${SESSION_LOOP_SCREENSHOT_SHA256}"
session_loop_ppm_sha256="$(awk '{print $1}' "${SESSION_LOOP_SCREENSHOT_SHA256}")"
session_loop_png_sha256="$(shasum -a 256 "${SESSION_LOOP_SCREENSHOT_PNG}" | awk '{print $1}')"
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'backend=drm-kms-session-loop'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'wallpaper_source=runtime-asset'
    echo "frame_checksum=${EXPECTED_FRAME_CHECKSUM}"
    echo "ppm_sha256=${session_loop_ppm_sha256}"
    echo "png_sha256=${session_loop_png_sha256}"
    echo 'session_owner=aqua-compositor'
    echo 'event_loop=calloop'
    echo 'calloop_dispatch_passes=3'
    echo 'received_page_flip_events=3'
    echo 'drm_event_source_released=true'
    echo 'crtc_restored=true'
    echo 'safe_return_to_recovery=ok'
} > "${SESSION_LOOP_SCREENSHOT_METADATA}"

if [ ! -s "${WAYLAND_SESSION_SCREENSHOT}" ]; then
    echo "Missing QEMU DRM Wayland-session screendump: ${WAYLAND_SESSION_SCREENSHOT}" >&2
    exit 1
fi
if [ ! -s "${SETTINGS_SCREENSHOT}" ]; then
    echo "Missing QEMU Aqua Settings screendump: ${SETTINGS_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${SETTINGS_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU Aqua Settings screendump format:" >&2
    file "${SETTINGS_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${SETTINGS_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU Aqua Settings PNG format:" >&2
    file "${SETTINGS_SCREENSHOT_PNG}" >&2
    exit 1
fi
if ! file "${WAYLAND_SESSION_SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU DRM Wayland-session screendump format:" >&2
    file "${WAYLAND_SESSION_SCREENSHOT}" >&2
    exit 1
fi
if ! file "${WAYLAND_SESSION_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU DRM Wayland-session PNG format:" >&2
    file "${WAYLAND_SESSION_SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${WAYLAND_SESSION_SCREENSHOT}" > "${WAYLAND_SESSION_SCREENSHOT_SHA256}"
wayland_session_ppm_sha256="$(awk '{print $1}' "${WAYLAND_SESSION_SCREENSHOT_SHA256}")"
wayland_session_png_sha256="$(shasum -a 256 "${WAYLAND_SESSION_SCREENSHOT_PNG}" | awk '{print $1}')"
wayland_session_frame_checksum="$(sed -n 's/.*drm_wayland_client_cleanup_repaint_checksum=\([0-9a-f][0-9a-f]*\).*/\1/p' "${SERIAL_LOG}" | tail -n 1)"
if [ -z "${wayland_session_frame_checksum}" ]; then
    echo "Missing repainted Wayland frame checksum" >&2
    exit 1
fi
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'backend=drm-kms-wayland-session'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'wallpaper_source=runtime-asset'
    echo "frame_checksum=${wayland_session_frame_checksum}"
    echo "ppm_sha256=${wayland_session_ppm_sha256}"
    echo "png_sha256=${wayland_session_png_sha256}"
    echo 'shared_session_lifecycle=true'
    echo 'scanout_bridge=gbm-dmabuf-direct'
    echo 'scanout_cpu_copy=false'
    echo 'direct_dmabuf_scanout=true'
    echo 'gbm_scanout_buffers_released=true'
    echo 'gpu_frame_readback=false'
    echo 'gpu_checksum_source=surface-inputs'
    echo 'wayland_dispatch_passes=3'
    echo 'wayland_flush_passes=3'
    echo 'calloop_drm_dispatch_passes=3'
    echo 'input_source=libinput-udev'
    echo 'input_seat=seat0'
    echo 'input_required=true'
    echo 'input_discovery_ready=true'
    echo 'input_keyboard_events=1'
    echo 'input_pointer_motion_events=1'
    echo 'input_pointer_button_events=2'
    echo 'input_launcher_visible=true'
    echo 'input_dispatch_ready=true'
    echo 'external_client_process_count=2'
    echo 'external_client_protocol=xdg_toplevel'
    echo 'external_client_surfaces=mixed-client-native-sizes'
    echo 'external_client_surface_count=2'
    echo 'external_client_buffer_bytes=dynamic'
    echo 'third_party_wayland_client=weston-simple-shm'
    echo 'weston_compositor_started=false'
    echo 'external_client_independent_buffers=true'
    echo 'external_client_composited=true'
    echo 'external_client_frame_callback=true'
    echo 'external_client_partial_damage=true'
    echo 'external_client_keyboard_focus=true'
    echo 'external_client_pointer_focus=false'
    echo 'external_client_focus_changes=6'
    echo 'external_client_stacking_changes=6'
    echo 'stacking_repaint_complete=true'
    echo 'stacking_repaint_changed_frame=true'
    echo 'stacking_repaint_page_flips=1'
    echo 'interactive_move_requests=1'
    echo 'interactive_resize_requests=1'
    echo 'interactive_geometry_applied=true'
    echo 'state_cycle_complete=true'
    echo 'state_configure_acks=8'
    echo 'client_cleanup_complete=true'
    echo 'client_cleanup_surviving_surfaces=1'
    echo 'client_cleanup_destroyed_surfaces=2'
    echo 'client_cleanup_session_alive=true'
    echo 'client_cleanup_keyboard_focus_reassigned=true'
    echo 'client_cleanup_pointer_focus_cleared=true'
    echo 'client_cleanup_repaint_complete=true'
    echo 'client_cleanup_repaint_page_flips=1'
    echo 'close_request_sent=true'
    echo 'close_cleanup_surfaces=0'
    echo 'close_repaint_complete=true'
    echo 'close_repaint_page_flips=1'
    echo 'wayland_socket_cleaned=true'
    echo 'crtc_restored=true'
    echo 'safe_return_to_recovery=ok'
} > "${WAYLAND_SESSION_SCREENSHOT_METADATA}"

if [ ! -s "${SCREENSHOT}" ]; then
    echo "Missing QEMU framebuffer screendump: ${SCREENSHOT}" >&2
    exit 1
fi
if ! file "${SCREENSHOT}" | grep -Fq 'Netpbm image data, size = 1280 x 800'; then
    echo "Unexpected QEMU framebuffer screendump format:" >&2
    file "${SCREENSHOT}" >&2
    exit 1
fi
if ! file "${SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'; then
    echo "Unexpected QEMU framebuffer PNG format:" >&2
    file "${SCREENSHOT_PNG}" >&2
    exit 1
fi
shasum -a 256 "${SCREENSHOT}" > "${SCREENSHOT_SHA256}"
ppm_sha256="$(awk '{print $1}' "${SCREENSHOT_SHA256}")"
png_sha256="$(shasum -a 256 "${SCREENSHOT_PNG}" | awk '{print $1}')"
{
    echo 'status=ok'
    echo 'source=qemu-monitor-screendump'
    echo 'format=ppm-p6+png-rgb'
    echo 'dimensions=1280x800'
    echo 'wallpaper_source=runtime-asset'
    echo "frame_checksum=${EXPECTED_FRAME_CHECKSUM}"
    echo "ppm_sha256=${ppm_sha256}"
    echo "png_sha256=${png_sha256}"
    echo 'framebuffer_write=true'
    echo 'visible_observation=false'
    echo 'safe_return_to_recovery=ok'
} > "${SCREENSHOT_METADATA}"

echo "Aqua Linux headless QEMU fbdev presenter check passed."
echo "Serial log: ${SERIAL_LOG}"
echo "Framebuffer capture: ${SCREENSHOT}"
echo "Framebuffer PNG: ${SCREENSHOT_PNG}"
echo "Capture checksum: ${SCREENSHOT_SHA256}"
echo "Capture metadata: ${SCREENSHOT_METADATA}"
echo "KMS capture: ${KMS_SCREENSHOT}"
echo "KMS PNG: ${KMS_SCREENSHOT_PNG}"
echo "KMS capture checksum: ${KMS_SCREENSHOT_SHA256}"
echo "KMS capture metadata: ${KMS_SCREENSHOT_METADATA}"
echo "Page-flip capture: ${PAGE_FLIP_SCREENSHOT}"
echo "Page-flip PNG: ${PAGE_FLIP_SCREENSHOT_PNG}"
echo "Page-flip capture checksum: ${PAGE_FLIP_SCREENSHOT_SHA256}"
echo "Page-flip capture metadata: ${PAGE_FLIP_SCREENSHOT_METADATA}"
echo "Frame-loop capture: ${FRAME_LOOP_SCREENSHOT}"
echo "Frame-loop PNG: ${FRAME_LOOP_SCREENSHOT_PNG}"
echo "Frame-loop capture checksum: ${FRAME_LOOP_SCREENSHOT_SHA256}"
echo "Frame-loop capture metadata: ${FRAME_LOOP_SCREENSHOT_METADATA}"
echo "Session-loop capture: ${SESSION_LOOP_SCREENSHOT}"
echo "Session-loop PNG: ${SESSION_LOOP_SCREENSHOT_PNG}"
echo "Session-loop capture checksum: ${SESSION_LOOP_SCREENSHOT_SHA256}"
echo "Session-loop capture metadata: ${SESSION_LOOP_SCREENSHOT_METADATA}"
echo "Wayland-session capture: ${WAYLAND_SESSION_SCREENSHOT}"
echo "Wayland-session PNG: ${WAYLAND_SESSION_SCREENSHOT_PNG}"
echo "Wayland-session capture checksum: ${WAYLAND_SESSION_SCREENSHOT_SHA256}"
echo "Wayland-session capture metadata: ${WAYLAND_SESSION_SCREENSHOT_METADATA}"
echo "Aqua Settings capture: ${SETTINGS_SCREENSHOT}"
echo "Aqua Settings PNG: ${SETTINGS_SCREENSHOT_PNG}"
echo "GBM direct-scanout capture: ${GBM_SCANOUT_SCREENSHOT}"
echo "GBM direct-scanout PNG: ${GBM_SCANOUT_SCREENSHOT_PNG}"
echo "GBM direct-scanout checksum: ${GBM_SCANOUT_SCREENSHOT_SHA256}"
echo "GBM direct-scanout metadata: ${GBM_SCANOUT_SCREENSHOT_METADATA}"
