#!/usr/bin/env bash
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/build/buildroot-output"
IMAGE_DIR="${OUTPUT_DIR}/images"
SERIAL_LOG="${ROOT_DIR}/build/qemu-serial-check.log"
FBDEV_QEMU_LOG="${FBDEV_QEMU_LOG:-${ROOT_DIR}/build/qemu-fbdev-present-check.log}"
GRAPHICAL_BOOT_QEMU_LOG="${GRAPHICAL_BOOT_QEMU_LOG:-${ROOT_DIR}/build/qemu-graphical-boot-check.log}"
LIVE_THEME_QEMU_LOG="${LIVE_THEME_QEMU_LOG:-${ROOT_DIR}/build/qemu-live-theme-check.log}"
FBDEV_QEMU_CAPTURE="${FBDEV_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-fbdev-present-capture.txt}"
FBDEV_QEMU_PPM="${FBDEV_QEMU_PPM:-${ROOT_DIR}/build/qemu-fbdev-present.ppm}"
FBDEV_QEMU_PNG="${FBDEV_QEMU_PNG:-${ROOT_DIR}/build/qemu-fbdev-present.png}"
FBDEV_QEMU_SHA256="${FBDEV_QEMU_SHA256:-${ROOT_DIR}/build/qemu-fbdev-present.sha256}"
KMS_QEMU_CAPTURE="${KMS_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-drm-kms-present-capture.txt}"
KMS_QEMU_PPM="${KMS_QEMU_PPM:-${ROOT_DIR}/build/qemu-drm-kms-present.ppm}"
KMS_QEMU_PNG="${KMS_QEMU_PNG:-${ROOT_DIR}/build/qemu-drm-kms-present.png}"
KMS_QEMU_SHA256="${KMS_QEMU_SHA256:-${ROOT_DIR}/build/qemu-drm-kms-present.sha256}"
GPU_SURFACE_QEMU_CAPTURE="${GPU_SURFACE_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-drm-gpu-surface-capture.txt}"
GPU_SURFACE_QEMU_PPM="${GPU_SURFACE_QEMU_PPM:-${ROOT_DIR}/build/qemu-drm-gpu-surface.ppm}"
GPU_SURFACE_QEMU_PNG="${GPU_SURFACE_QEMU_PNG:-${ROOT_DIR}/build/qemu-drm-gpu-surface.png}"
GBM_SCANOUT_QEMU_CAPTURE="${GBM_SCANOUT_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-drm-gbm-scanout-capture.txt}"
GBM_SCANOUT_QEMU_PPM="${GBM_SCANOUT_QEMU_PPM:-${ROOT_DIR}/build/qemu-drm-gbm-scanout.ppm}"
GBM_SCANOUT_QEMU_PNG="${GBM_SCANOUT_QEMU_PNG:-${ROOT_DIR}/build/qemu-drm-gbm-scanout.png}"
PAGE_FLIP_QEMU_CAPTURE="${PAGE_FLIP_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-drm-page-flip-present-capture.txt}"
PAGE_FLIP_QEMU_PPM="${PAGE_FLIP_QEMU_PPM:-${ROOT_DIR}/build/qemu-drm-page-flip-present.ppm}"
PAGE_FLIP_QEMU_PNG="${PAGE_FLIP_QEMU_PNG:-${ROOT_DIR}/build/qemu-drm-page-flip-present.png}"
PAGE_FLIP_QEMU_SHA256="${PAGE_FLIP_QEMU_SHA256:-${ROOT_DIR}/build/qemu-drm-page-flip-present.sha256}"
FRAME_LOOP_QEMU_CAPTURE="${FRAME_LOOP_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-drm-frame-loop-capture.txt}"
FRAME_LOOP_QEMU_PPM="${FRAME_LOOP_QEMU_PPM:-${ROOT_DIR}/build/qemu-drm-frame-loop.ppm}"
FRAME_LOOP_QEMU_PNG="${FRAME_LOOP_QEMU_PNG:-${ROOT_DIR}/build/qemu-drm-frame-loop.png}"
FRAME_LOOP_QEMU_SHA256="${FRAME_LOOP_QEMU_SHA256:-${ROOT_DIR}/build/qemu-drm-frame-loop.sha256}"
SESSION_LOOP_QEMU_CAPTURE="${SESSION_LOOP_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-drm-session-loop-capture.txt}"
SESSION_LOOP_QEMU_PPM="${SESSION_LOOP_QEMU_PPM:-${ROOT_DIR}/build/qemu-drm-session-loop.ppm}"
SESSION_LOOP_QEMU_PNG="${SESSION_LOOP_QEMU_PNG:-${ROOT_DIR}/build/qemu-drm-session-loop.png}"
SESSION_LOOP_QEMU_SHA256="${SESSION_LOOP_QEMU_SHA256:-${ROOT_DIR}/build/qemu-drm-session-loop.sha256}"
WAYLAND_SESSION_QEMU_CAPTURE="${WAYLAND_SESSION_QEMU_CAPTURE:-${ROOT_DIR}/build/qemu-drm-wayland-session-capture.txt}"
WAYLAND_SESSION_QEMU_PPM="${WAYLAND_SESSION_QEMU_PPM:-${ROOT_DIR}/build/qemu-drm-wayland-session.ppm}"
WAYLAND_SESSION_QEMU_PNG="${WAYLAND_SESSION_QEMU_PNG:-${ROOT_DIR}/build/qemu-drm-wayland-session.png}"
WAYLAND_SESSION_QEMU_SHA256="${WAYLAND_SESSION_QEMU_SHA256:-${ROOT_DIR}/build/qemu-drm-wayland-session.sha256}"
MANIFEST="${MANIFEST:-${ROOT_DIR}/build/aqua-image-manifest.txt}"
MANIFEST_JSON="${MANIFEST_JSON:-${ROOT_DIR}/build/aqua-image-manifest.json}"
BOOT_SUMMARY="${BOOT_SUMMARY:-${ROOT_DIR}/build/aqua-boot-summary.txt}"
BOOT_SUMMARY_JSON="${BOOT_SUMMARY_JSON:-${ROOT_DIR}/build/aqua-boot-summary.json}"
ROOTFS_TAR="${ROOTFS_TAR:-${IMAGE_DIR}/rootfs.tar}"
CONTRACT_DIR="${CONTRACT_DIR:-${ROOT_DIR}/build/rootfs-compositor-contract}"
QEMU_VISIBLE_OPERATOR_PASS="${QEMU_VISIBLE_OPERATOR_PASS:-${ROOT_DIR}/build/qemu-visible-operator-pass.txt}"
QEMU_VISIBLE_OPERATOR_PASS_JSON="${QEMU_VISIBLE_OPERATOR_PASS_JSON:-${ROOT_DIR}/build/qemu-visible-operator-pass.json}"

size_or_missing() {
    path="$1"
    if [ -f "${path}" ]; then
        wc -c < "${path}" | tr -d ' '
    else
        printf 'missing'
    fi
}

status_from_file() {
    path="$1"
    if [ -f "${path}" ]; then
        printf 'ready'
    else
        printf 'missing'
    fi
}

marker_status() {
    marker="$1"
    if [ -f "${SERIAL_LOG}" ] && grep -Fq "${marker}" "${SERIAL_LOG}"; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

rootfs_entry_status() {
    entry="$1"
    if [ -f "${ROOTFS_TAR}" ] && tar -tf "${ROOTFS_TAR}" "${entry}" >/dev/null 2>&1; then
        printf 'present'
    else
        printf 'missing'
    fi
}

compositor_packaged_status() {
    if [ -f "${ROOTFS_TAR}" ] && \
       tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/compositor-binary.txt 2>/dev/null | grep -Fq "aqua-compositor packaged=true"; then
        printf 'true'
    else
        printf 'false'
    fi
}

rootfs_text_contains() {
    entry="$1"
    needle="$2"
    if [ -f "${ROOTFS_TAR}" ] && \
       tar -xOf "${ROOTFS_TAR}" "${entry}" 2>/dev/null | grep -Fq "${needle}"; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

rootfs_session_config_status() {
    if [ -f "${ROOTFS_TAR}" ] && \
       tar -xOf "${ROOTFS_TAR}" ./etc/aqua/compositor-session.conf 2>/dev/null | grep -Fq "recovery_tty_required=true"; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

rootfs_session_env_status() {
    if [ -f "${ROOTFS_TAR}" ] && \
       tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session.env 2>/dev/null | grep -Fq "export WAYLAND_DISPLAY=aqua-wayland-0"; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

contract_file_contains() {
    file="$1"
    needle="$2"
    if [ -f "${file}" ] && grep -Fq "${needle}" "${file}"; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

contract_file_numeric_at_least() {
    file="$1"
    prefix="$2"
    minimum="$3"
    value="$(grep -F "${prefix}" "${file}" 2>/dev/null | tail -n 1 | sed "s/.*${prefix}//" | tr -cd '0-9')"
    if [ -n "${value}" ] && [ "${value}" -ge "${minimum}" ]; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

capture_checksum_status() {
    artifact="$1"
    metadata_key="$2"
    metadata="${3:-${FBDEV_QEMU_CAPTURE}}"
    if [ ! -f "${artifact}" ] || [ ! -f "${metadata}" ]; then
        printf 'missing'
        return
    fi

    recorded="$(awk -F= -v key="${metadata_key}" '$1 == key { print $2; exit }' "${metadata}")"
    actual="$(shasum -a 256 "${artifact}" | awk '{print $1}')"
    if [ -n "${recorded}" ] && [ "${recorded}" = "${actual}" ]; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

boot_summary_status() {
    if [ -f "${BOOT_SUMMARY}" ] && grep -Fq "status=ok" "${BOOT_SUMMARY}"; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

boot_summary_stage_status() {
    stage="$1"
    if [ -f "${BOOT_SUMMARY}" ] && grep -Fq "${stage}=ok" "${BOOT_SUMMARY}"; then
        printf 'ok'
    else
        printf 'missing'
    fi
}

mkdir -p "$(dirname "${MANIFEST}")"
mkdir -p "$(dirname "${MANIFEST_JSON}")"
GENERATED_AT_UTC="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

cat > "${MANIFEST}" <<EOF
product=Aqua Linux
base=Buildroot
dev_target=QEMU x86_64
graphics_target=custom Wayland compositor
generated_at_utc=${GENERATED_AT_UTC}

[artifacts]
bzImage.status=$(status_from_file "${IMAGE_DIR}/bzImage")
bzImage.bytes=$(size_or_missing "${IMAGE_DIR}/bzImage")
rootfs_ext2.status=$(status_from_file "${IMAGE_DIR}/rootfs.ext2")
rootfs_ext2.bytes=$(size_or_missing "${IMAGE_DIR}/rootfs.ext2")
disk_img.status=$(status_from_file "${IMAGE_DIR}/disk.img")
disk_img.bytes=$(size_or_missing "${IMAGE_DIR}/disk.img")
rootfs_tar.status=$(status_from_file "${ROOTFS_TAR}")
rootfs_tar.bytes=$(size_or_missing "${ROOTFS_TAR}")
build_config.status=$(status_from_file "${OUTPUT_DIR}/.config")
serial_log.status=$(status_from_file "${SERIAL_LOG}")
fbdev_qemu_present_log.status=$(status_from_file "${FBDEV_QEMU_LOG}")
graphical_boot_qemu_log.status=$(status_from_file "${GRAPHICAL_BOOT_QEMU_LOG}")
fbdev_qemu_capture.status=$(status_from_file "${FBDEV_QEMU_CAPTURE}")
fbdev_qemu_ppm.status=$(status_from_file "${FBDEV_QEMU_PPM}")
fbdev_qemu_ppm.bytes=$(size_or_missing "${FBDEV_QEMU_PPM}")
fbdev_qemu_png.status=$(status_from_file "${FBDEV_QEMU_PNG}")
fbdev_qemu_png.bytes=$(size_or_missing "${FBDEV_QEMU_PNG}")
fbdev_qemu_sha256.status=$(status_from_file "${FBDEV_QEMU_SHA256}")
kms_qemu_capture.status=$(status_from_file "${KMS_QEMU_CAPTURE}")
kms_qemu_ppm.status=$(status_from_file "${KMS_QEMU_PPM}")
kms_qemu_ppm.bytes=$(size_or_missing "${KMS_QEMU_PPM}")
kms_qemu_png.status=$(status_from_file "${KMS_QEMU_PNG}")
kms_qemu_png.bytes=$(size_or_missing "${KMS_QEMU_PNG}")
kms_qemu_sha256.status=$(status_from_file "${KMS_QEMU_SHA256}")
page_flip_qemu_capture.status=$(status_from_file "${PAGE_FLIP_QEMU_CAPTURE}")
page_flip_qemu_ppm.status=$(status_from_file "${PAGE_FLIP_QEMU_PPM}")
page_flip_qemu_ppm.bytes=$(size_or_missing "${PAGE_FLIP_QEMU_PPM}")
page_flip_qemu_png.status=$(status_from_file "${PAGE_FLIP_QEMU_PNG}")
page_flip_qemu_png.bytes=$(size_or_missing "${PAGE_FLIP_QEMU_PNG}")
page_flip_qemu_sha256.status=$(status_from_file "${PAGE_FLIP_QEMU_SHA256}")
frame_loop_qemu_capture.status=$(status_from_file "${FRAME_LOOP_QEMU_CAPTURE}")
frame_loop_qemu_ppm.status=$(status_from_file "${FRAME_LOOP_QEMU_PPM}")
frame_loop_qemu_ppm.bytes=$(size_or_missing "${FRAME_LOOP_QEMU_PPM}")
frame_loop_qemu_png.status=$(status_from_file "${FRAME_LOOP_QEMU_PNG}")
frame_loop_qemu_png.bytes=$(size_or_missing "${FRAME_LOOP_QEMU_PNG}")
frame_loop_qemu_sha256.status=$(status_from_file "${FRAME_LOOP_QEMU_SHA256}")
session_loop_qemu_capture.status=$(status_from_file "${SESSION_LOOP_QEMU_CAPTURE}")
session_loop_qemu_ppm.status=$(status_from_file "${SESSION_LOOP_QEMU_PPM}")
session_loop_qemu_ppm.bytes=$(size_or_missing "${SESSION_LOOP_QEMU_PPM}")
session_loop_qemu_png.status=$(status_from_file "${SESSION_LOOP_QEMU_PNG}")
session_loop_qemu_png.bytes=$(size_or_missing "${SESSION_LOOP_QEMU_PNG}")
session_loop_qemu_sha256.status=$(status_from_file "${SESSION_LOOP_QEMU_SHA256}")
wayland_session_qemu_capture.status=$(status_from_file "${WAYLAND_SESSION_QEMU_CAPTURE}")
wayland_session_qemu_ppm.status=$(status_from_file "${WAYLAND_SESSION_QEMU_PPM}")
wayland_session_qemu_ppm.bytes=$(size_or_missing "${WAYLAND_SESSION_QEMU_PPM}")
wayland_session_qemu_png.status=$(status_from_file "${WAYLAND_SESSION_QEMU_PNG}")
wayland_session_qemu_png.bytes=$(size_or_missing "${WAYLAND_SESSION_QEMU_PNG}")
wayland_session_qemu_sha256.status=$(status_from_file "${WAYLAND_SESSION_QEMU_SHA256}")
drm_device_probe.status=$(status_from_file "${CONTRACT_DIR}/drm-device-probe.txt")
boot_summary.status=$(status_from_file "${BOOT_SUMMARY}")
boot_summary_json.status=$(status_from_file "${BOOT_SUMMARY_JSON}")
compositor_status.status=$(status_from_file "${CONTRACT_DIR}/status.txt")
session_config_probe.status=$(status_from_file "${CONTRACT_DIR}/session-config.txt")
session_env_probe.status=$(status_from_file "${CONTRACT_DIR}/session-env.txt")
session_bootstrap_probe.status=$(status_from_file "${CONTRACT_DIR}/session-bootstrap.txt")
session_check_probe.status=$(status_from_file "${CONTRACT_DIR}/session-check.txt")
manual_launch_plan.status=$(status_from_file "${CONTRACT_DIR}/manual-launch-plan.txt")
guarded_run.status=$(status_from_file "${CONTRACT_DIR}/guarded-run.txt")
graphical_session_supervisor.status=$(status_from_file "${CONTRACT_DIR}/graphical-session-supervisor.txt")
media_service_supervisor.status=$(status_from_file "${CONTRACT_DIR}/media-service-supervisor.txt")
graphical_session_boot.status=$(status_from_file "${CONTRACT_DIR}/graphical-session-boot.txt")
handoff_gate.status=$(status_from_file "${CONTRACT_DIR}/handoff-gate.txt")
output_plan_probe.status=$(status_from_file "${CONTRACT_DIR}/output-plan-probe.txt")
display_output_handoff_probe.status=$(status_from_file "${CONTRACT_DIR}/display-output-handoff-probe.txt")
visible_preview_plan_probe.status=$(status_from_file "${CONTRACT_DIR}/visible-preview-plan-probe.txt")
visible_preview_export_probe.status=$(status_from_file "${CONTRACT_DIR}/visible-preview-export-probe.txt")
visible_preview_export.status=$(status_from_file "${CONTRACT_DIR}/aqua-visible-preview.html")
visible_preview_export.bytes=$(size_or_missing "${CONTRACT_DIR}/aqua-visible-preview.html")
nested_preview_loop.status=$(status_from_file "${CONTRACT_DIR}/nested-preview-loop.txt")
manual_nested_preview_backend.status=$(status_from_file "${CONTRACT_DIR}/manual-nested-preview-backend.txt")
manual_nested_preview_execution.status=$(status_from_file "${CONTRACT_DIR}/manual-nested-preview-execution.txt")
manual_nested_preview_execution_probe.status=$(status_from_file "${CONTRACT_DIR}/manual-nested-preview-execution-probe.txt")
visible_preview_request.status=$(status_from_file "${CONTRACT_DIR}/visible-preview-request.txt")
visible_preview_launch.status=$(status_from_file "${CONTRACT_DIR}/visible-preview-launch.txt")
recovery_help.status=$(status_from_file "${CONTRACT_DIR}/recovery-help.txt")
operator_transcript.status=$(status_from_file "${CONTRACT_DIR}/operator-transcript.txt")
graphics_enable_gate.status=$(status_from_file "${CONTRACT_DIR}/graphics-enable-gate.txt")
graphics_enable_gate_positive.status=$(status_from_file "${CONTRACT_DIR}/graphics-enable-gate-positive.txt")
graphics_launch_candidate.status=$(status_from_file "${CONTRACT_DIR}/graphics-launch-candidate.txt")
graphics_rollback_drill.status=$(status_from_file "${CONTRACT_DIR}/graphics-rollback-drill.txt")
graphics_startup_preflight.status=$(status_from_file "${CONTRACT_DIR}/graphics-startup-preflight.txt")
graphics_startup_rehearsal.status=$(status_from_file "${CONTRACT_DIR}/graphics-startup-rehearsal.txt")
graphics_qemu_display_gate.status=$(status_from_file "${CONTRACT_DIR}/graphics-qemu-display-gate.txt")
graphics_visible_qemu_attempt.status=$(status_from_file "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt")
graphics_visible_attempt_transcript.status=$(status_from_file "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt")
graphics_visible_attempt_result.status=$(status_from_file "${CONTRACT_DIR}/graphics-visible-attempt-result.txt")
graphics_visible_attempt_runner.status=$(status_from_file "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt")
graphics_qemu_visible_boot_check.status=$(status_from_file "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt")
graphics_fbdev_present.status=$(status_from_file "${CONTRACT_DIR}/graphics-fbdev-present.txt")
graphics_qemu_observation_marker.status=$(status_from_file "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt")
qemu_visible_evidence_record.status=$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-record.txt")
graphics_qemu_observation_positive.status=$(status_from_file "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt")
qemu_visible_pass_report.status=$(status_from_file "${CONTRACT_DIR}/qemu-visible-pass-report.txt")
qemu_visible_manual_runbook.status=$(status_from_file "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt")
qemu_visible_operator_pass.status=$(status_from_file "${QEMU_VISIBLE_OPERATOR_PASS}")
qemu_visible_operator_pass_json.status=$(status_from_file "${QEMU_VISIBLE_OPERATOR_PASS_JSON}")
qemu_visible_evidence_bundle_apply.status=$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt")
qemu_visible_evidence_bundle_apply_positive.status=$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt")
qemu_visible_evidence_bundle_apply_missing_preflight.status=$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt")
qemu_visible_evidence_bundle_apply_missing_capture_hash.status=$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt")
client_window_model_probe.status=$(status_from_file "${CONTRACT_DIR}/client-window-model-probe.txt")
client_surface_lifecycle_probe.status=$(status_from_file "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt")
client_surface_registry_probe.status=$(status_from_file "${CONTRACT_DIR}/client-surface-registry-probe.txt")
renderer_surface_sources_probe.status=$(status_from_file "${CONTRACT_DIR}/renderer-surface-sources-probe.txt")
client_layer_pipeline_probe.status=$(status_from_file "${CONTRACT_DIR}/client-layer-pipeline-probe.txt")
xdg_shell_binding_probe.status=$(status_from_file "${CONTRACT_DIR}/xdg-shell-binding-probe.txt")
xdg_toplevel_client_probe.status=$(status_from_file "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt")
xdg_toplevel_window_model_probe.status=$(status_from_file "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt")
smithay_launcher_seat_probe.status=$(status_from_file "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt")
scene_probe.status=$(status_from_file "${CONTRACT_DIR}/scene-probe.txt")
scene_dump.status=$(status_from_file "${CONTRACT_DIR}/scene-dump.txt")
render_plan_probe.status=$(status_from_file "${CONTRACT_DIR}/render-plan-probe.txt")
render_plan_dump.status=$(status_from_file "${CONTRACT_DIR}/render-plan-dump.txt")
paint_plan_probe.status=$(status_from_file "${CONTRACT_DIR}/paint-plan-probe.txt")
paint_plan_dump.status=$(status_from_file "${CONTRACT_DIR}/paint-plan-dump.txt")
frame_plan_probe.status=$(status_from_file "${CONTRACT_DIR}/frame-plan-probe.txt")
frame_plan_dump.status=$(status_from_file "${CONTRACT_DIR}/frame-plan-dump.txt")
frame_buffer_probe.status=$(status_from_file "${CONTRACT_DIR}/frame-buffer-probe.txt")
frame_buffer_dump.status=$(status_from_file "${CONTRACT_DIR}/frame-buffer-dump.txt")
raster_probe.status=$(status_from_file "${CONTRACT_DIR}/raster-probe.txt")
raster_dump.status=$(status_from_file "${CONTRACT_DIR}/raster-dump.txt")
raster_export_probe.status=$(status_from_file "${CONTRACT_DIR}/raster-export-probe.txt")
raster_export.status=$(status_from_file "${CONTRACT_DIR}/aqua-raster.ppm")
raster_export.bytes=$(size_or_missing "${CONTRACT_DIR}/aqua-raster.ppm")
raster_png_export_probe.status=$(status_from_file "${CONTRACT_DIR}/raster-png-export-probe.txt")
raster_png_export.status=$(status_from_file "${CONTRACT_DIR}/aqua-raster.png")
raster_png_export.bytes=$(size_or_missing "${CONTRACT_DIR}/aqua-raster.png")
session_loop.status=$(status_from_file "${CONTRACT_DIR}/session-loop.txt")

[rootfs]
session_config=$(rootfs_entry_status ./etc/aqua/compositor-session.conf)
session_config_recovery_safe=$(rootfs_session_config_status)
session_env=$(rootfs_entry_status ./etc/aqua/session.env)
session_env_recovery_safe=$(rootfs_session_env_status)
runtime_assets=$(rootfs_entry_status ./usr/share/aqua/tokens/design-tokens.json)
design_tokens_product=$(rootfs_text_contains ./usr/share/aqua/tokens/design-tokens.json '"product": "Aqua Linux"')
design_tokens_scene_materials=$(rootfs_text_contains ./usr/share/aqua/tokens/design-tokens.json '"blurRequired"')
compositor_binary=$(rootfs_entry_status ./usr/bin/aqua-compositor)
compositor_packaged=$(compositor_packaged_status)
autostart=false
boot_graphics=false

[scene_contract]
scene_model=$(contract_file_contains "${CONTRACT_DIR}/status.txt" "scene_model=aqua-scene")
graphics_drm_rootfs_probe=$(contract_file_contains "${CONTRACT_DIR}/drm-device-probe.txt" "[AQUA-COMPOSITOR] stage=drm-device-probe status=ok")
graphics_drm_qemu_probe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-device-probe status=ok")
graphics_drm_qemu_device=$(contract_file_contains "${FBDEV_QEMU_LOG}" "device=/dev/dri/card0")
graphics_drm_qemu_connector=$(contract_file_contains "${FBDEV_QEMU_LOG}" "connector.Virtual-1.status=connected")
graphics_drm_qemu_mode=$(contract_file_contains "${FBDEV_QEMU_LOG}" "connector.Virtual-1.first_mode=1280x800")
graphics_drm_read_only=$(contract_file_contains "${FBDEV_QEMU_LOG}" "device_open_mode=read-only")
graphics_drm_no_master=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_master_acquired=false")
graphics_drm_no_modeset=$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_activated=false")
graphics_drm_dumb_buffer=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-dumb-buffer-probe status=ok")
graphics_drm_dumb_buffer_mode=$(contract_file_contains "${FBDEV_QEMU_LOG}" "selected_mode=1280x800")
graphics_drm_dumb_buffer_pitch=$(contract_file_contains "${FBDEV_QEMU_LOG}" "buffer_pitch=5120")
graphics_drm_dumb_buffer_bytes=$(contract_file_contains "${FBDEV_QEMU_LOG}" "buffer_bytes=4096000")
graphics_drm_dumb_buffer_checksum=$(contract_file_contains "${FBDEV_QEMU_LOG}" "buffer_checksum=c85dbfbfc17843af")
graphics_drm_dumb_buffer_mapped=$(contract_file_contains "${FBDEV_QEMU_LOG}" "dumb_buffer_mapped=true")
graphics_drm_dumb_buffer_destroyed=$(contract_file_contains "${FBDEV_QEMU_LOG}" "dumb_buffer_destroyed=true")
graphics_drm_dumb_buffer_no_framebuffer=$(contract_file_contains "${FBDEV_QEMU_LOG}" "framebuffer_created=false")
graphics_drm_dumb_buffer_no_page_flip=$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_submitted=false")
graphics_drm_dumb_buffer_recovery_safe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")
graphics_drm_gbm_scanout_buffer=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gbm-scanout-buffer-probe status=ok")
graphics_drm_gbm_scanout_usage=$(contract_file_contains "${FBDEV_QEMU_LOG}" "gbm_usage_rendering=true")
graphics_drm_gbm_scanout_pitch=$(contract_file_contains "${FBDEV_QEMU_LOG}" "gbm_front_pitch=5120")
graphics_drm_gbm_dmabuf_export=$(contract_file_contains "${FBDEV_QEMU_LOG}" "dmabuf_exported=true")
graphics_drm_gbm_addfb2=$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_addfb2_back=true")
graphics_drm_gbm_framebuffer_cleanup=$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_framebuffers_destroyed=true")
graphics_drm_gbm_direct_scanout=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=ok")
graphics_drm_gbm_direct_rendered=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_gbm_scanout_back_bound=true")
graphics_drm_gbm_direct_no_cpu_copy=$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "scanout_cpu_copy=false")
graphics_drm_gbm_direct_page_flip=$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "page_flip_event_received=true")
graphics_drm_gbm_direct_crtc_restored=$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "crtc_restored=true")
graphics_drm_gbm_direct_capture=$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "status=ok")
graphics_drm_gbm_direct_capture_dimensions=$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_drm_gbm_direct_ppm_checksum=$(capture_checksum_status "${GBM_SCANOUT_QEMU_PPM}" "ppm_sha256" "${GBM_SCANOUT_QEMU_CAPTURE}")
graphics_drm_gbm_direct_png_checksum=$(capture_checksum_status "${GBM_SCANOUT_QEMU_PNG}" "png_sha256" "${GBM_SCANOUT_QEMU_CAPTURE}")
graphics_drm_kms_present=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-kms-present status=ok")
graphics_drm_kms_active=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-kms-present status=active")
graphics_drm_kms_framebuffer=$(contract_file_contains "${FBDEV_QEMU_LOG}" "framebuffer_created=true")
graphics_drm_kms_activated=$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_activated=true")
graphics_drm_kms_no_page_flip=$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_submitted=false")
graphics_drm_kms_crtc_restored=$(contract_file_contains "${FBDEV_QEMU_LOG}" "crtc_restored=true")
graphics_drm_kms_framebuffer_destroyed=$(contract_file_contains "${FBDEV_QEMU_LOG}" "framebuffer_destroyed=true")
graphics_drm_kms_recovery_safe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "display_output_stopped=true")
graphics_drm_kms_capture=$(contract_file_contains "${KMS_QEMU_CAPTURE}" "status=ok")
graphics_drm_kms_capture_dimensions=$(contract_file_contains "${KMS_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_drm_kms_ppm_checksum=$(capture_checksum_status "${KMS_QEMU_PPM}" "ppm_sha256" "${KMS_QEMU_CAPTURE}")
graphics_drm_kms_png_checksum=$(capture_checksum_status "${KMS_QEMU_PNG}" "png_sha256" "${KMS_QEMU_CAPTURE}")
graphics_drm_gpu_surface=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gpu-surface status=ok")
graphics_drm_gpu_surface_active=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gpu-surface status=active")
graphics_drm_gpu_surface_composition=$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_backend=smithay-gles2-gbm")
graphics_drm_gpu_surface_shader=$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_shader=aqua-surface-compositor-v1")
graphics_drm_gpu_surface_blur=$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_blur_passes=2")
graphics_drm_gpu_surface_page_flip=$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_event_received=true")
graphics_drm_gpu_surface_bridge=$(contract_file_contains "${FBDEV_QEMU_LOG}" "scanout_bridge=cpu-readback-copy")
graphics_drm_gpu_surface_no_direct_scanout=$(contract_file_contains "${FBDEV_QEMU_LOG}" "direct_dmabuf_scanout=false")
graphics_drm_gpu_surface_client_source=$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_client_texture_source=sampled-wl-shm-contract")
graphics_drm_gpu_surface_client_count=$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_client_texture_count=2")
graphics_drm_gpu_surface_client_composited=$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_client_textures_composited=true")
graphics_drm_gpu_surface_client_not_live=$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_live_wayland_session=false")
graphics_drm_gpu_surface_capture=$(contract_file_contains "${GPU_SURFACE_QEMU_CAPTURE}" "status=ok")
graphics_drm_gpu_surface_capture_dimensions=$(contract_file_contains "${GPU_SURFACE_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_drm_gpu_surface_ppm_checksum=$(capture_checksum_status "${GPU_SURFACE_QEMU_PPM}" "ppm_sha256" "${GPU_SURFACE_QEMU_CAPTURE}")
graphics_drm_gpu_surface_png_checksum=$(capture_checksum_status "${GPU_SURFACE_QEMU_PNG}" "png_sha256" "${GPU_SURFACE_QEMU_CAPTURE}")
graphics_drm_gpu_surface_crtc_restored=$(contract_file_contains "${GPU_SURFACE_QEMU_CAPTURE}" "crtc_restored=true")
graphics_drm_page_flip=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-page-flip status=ok")
graphics_drm_page_flip_active=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-page-flip status=active")
graphics_drm_page_flip_submitted=$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_submitted=true")
graphics_drm_page_flip_event=$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_event_received=true")
graphics_drm_page_flip_front_destroyed=$(contract_file_contains "${FBDEV_QEMU_LOG}" "front_framebuffer_destroyed=true")
graphics_drm_page_flip_back_destroyed=$(contract_file_contains "${FBDEV_QEMU_LOG}" "back_framebuffer_destroyed=true")
graphics_drm_page_flip_crtc_restored=$(contract_file_contains "${FBDEV_QEMU_LOG}" "crtc_restored=true")
graphics_drm_page_flip_recovery_safe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")
graphics_drm_page_flip_capture=$(contract_file_contains "${PAGE_FLIP_QEMU_CAPTURE}" "status=ok")
graphics_drm_page_flip_capture_dimensions=$(contract_file_contains "${PAGE_FLIP_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_drm_page_flip_ppm_checksum=$(capture_checksum_status "${PAGE_FLIP_QEMU_PPM}" "ppm_sha256" "${PAGE_FLIP_QEMU_CAPTURE}")
graphics_drm_page_flip_png_checksum=$(capture_checksum_status "${PAGE_FLIP_QEMU_PNG}" "png_sha256" "${PAGE_FLIP_QEMU_CAPTURE}")
graphics_drm_frame_loop=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-frame-loop status=ok")
graphics_drm_frame_loop_active=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-frame-loop status=active")
graphics_drm_frame_loop_submitted=$(contract_file_contains "${FBDEV_QEMU_LOG}" "submitted_page_flips=3")
graphics_drm_frame_loop_received=$(contract_file_contains "${FBDEV_QEMU_LOG}" "received_page_flip_events=3")
graphics_drm_frame_loop_order=$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_event_order_complete=true")
graphics_drm_frame_loop_alternation=$(contract_file_contains "${FBDEV_QEMU_LOG}" "front_back_buffer_alternation=true")
graphics_drm_frame_loop_crtc_restored=$(contract_file_contains "${FBDEV_QEMU_LOG}" "crtc_restored=true")
graphics_drm_frame_loop_recovery_safe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")
graphics_drm_frame_loop_capture=$(contract_file_contains "${FRAME_LOOP_QEMU_CAPTURE}" "status=ok")
graphics_drm_frame_loop_capture_dimensions=$(contract_file_contains "${FRAME_LOOP_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_drm_frame_loop_ppm_checksum=$(capture_checksum_status "${FRAME_LOOP_QEMU_PPM}" "ppm_sha256" "${FRAME_LOOP_QEMU_CAPTURE}")
graphics_drm_frame_loop_png_checksum=$(capture_checksum_status "${FRAME_LOOP_QEMU_PNG}" "png_sha256" "${FRAME_LOOP_QEMU_CAPTURE}")
graphics_drm_session_loop=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-session-loop status=ok")
graphics_drm_session_loop_active=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-session-loop status=active")
graphics_drm_session_loop_owner=$(contract_file_contains "${FBDEV_QEMU_LOG}" "session_owner=aqua-compositor")
graphics_drm_session_loop_calloop=$(contract_file_contains "${FBDEV_QEMU_LOG}" "event_loop=calloop")
graphics_drm_session_loop_source_owned=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_event_source_owned=true")
graphics_drm_session_loop_dispatch=$(contract_file_contains "${FBDEV_QEMU_LOG}" "calloop_dispatch_passes=3")
graphics_drm_session_loop_source_released=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_event_source_released=true")
graphics_drm_session_loop_wayland_stopped=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_display_started=false")
graphics_drm_session_loop_recovery_safe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")
graphics_drm_session_loop_capture=$(contract_file_contains "${SESSION_LOOP_QEMU_CAPTURE}" "status=ok")
graphics_drm_session_loop_capture_dimensions=$(contract_file_contains "${SESSION_LOOP_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_drm_session_loop_ppm_checksum=$(capture_checksum_status "${SESSION_LOOP_QEMU_PPM}" "ppm_sha256" "${SESSION_LOOP_QEMU_CAPTURE}")
graphics_drm_session_loop_png_checksum=$(capture_checksum_status "${SESSION_LOOP_QEMU_PNG}" "png_sha256" "${SESSION_LOOP_QEMU_CAPTURE}")
graphics_drm_wayland_session=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok")
graphics_drm_wayland_session_active=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-wayland-session status=active")
graphics_drm_wayland_gpu_composition=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_composition_backend=smithay-gles2-readback-dumb-buffer")
graphics_drm_wayland_gpu_render_node=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_render_device=/dev/dri/card0")
graphics_drm_wayland_gpu_same_kms_node=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_render_node_separate=false")
graphics_drm_wayland_virtio_scanout_compat=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_direct_dmabuf_scanout=false")
graphics_drm_wayland_cpu_scanout_copy=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_scanout_cpu_copy=true")
graphics_drm_wayland_gbm_cleanup=$(contract_file_contains "${FBDEV_QEMU_LOG}" "gbm_scanout_buffers_released=true")
graphics_drm_wayland_frame_readback=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_frame_readback=true")
graphics_drm_wayland_frame_checksum=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_checksum_source=frame-readback")
graphics_drm_wayland_gpu_live_source=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_texture_source=live-smithay-wl-shm-snapshot")
graphics_drm_wayland_gpu_live_count=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_texture_count=2")
graphics_drm_wayland_gpu_live_bytes=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_texture_bytes=643216")
graphics_drm_wayland_gpu_live_uploaded=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_textures_uploaded=true")
graphics_drm_wayland_gpu_live_composited=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_textures_composited=true")
graphics_drm_wayland_gpu_live_session=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_live_session=true")
graphics_drm_wayland_gpu_initial_frame=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_initial_frame_checksum=")
graphics_drm_wayland_gpu_session_context=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_context_lifecycle=session-owned")
graphics_drm_wayland_gpu_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_updates=true")
graphics_drm_wayland_gpu_context_reused=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_context_reused=true")
graphics_drm_wayland_gpu_repaint_source_order=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_source_order_changed=true")
graphics_drm_wayland_gpu_repaint_count=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_texture_count=2")
graphics_drm_wayland_gpu_repaint_bytes=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_texture_bytes=643216")
graphics_drm_wayland_gpu_repaint_checksum=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_checksum=")
graphics_drm_wayland_gpu_files_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_gpu_repaint=true")
graphics_drm_wayland_gpu_settings_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_gpu_repaint=true")
graphics_drm_wayland_gpu_cleanup_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_gpu_repaint=true")
graphics_drm_wayland_gpu_close_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_gpu_repaint=true")
graphics_drm_wayland_gpu_full_repaint_route=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_route_complete=true")
graphics_drm_wayland_shared_lifecycle=$(contract_file_contains "${FBDEV_QEMU_LOG}" "shared_session_lifecycle=true")
graphics_drm_wayland_display=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_display_created=true")
graphics_drm_wayland_socket=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_socket_bound=true")
graphics_drm_wayland_client=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_client_inserted=true")
graphics_drm_wayland_dispatch=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_dispatch_passes=3")
graphics_drm_wayland_flush=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_flush_passes=3")
graphics_drm_wayland_drm_dispatch=$(contract_file_contains "${FBDEV_QEMU_LOG}" "calloop_drm_dispatch_passes=3")
graphics_drm_wayland_smithay_globals=$(contract_file_contains "${FBDEV_QEMU_LOG}" "smithay_protocol_globals_started=true")
graphics_drm_wayland_compositor_global=$(contract_file_contains "${FBDEV_QEMU_LOG}" "compositor_global_started=true")
graphics_drm_wayland_shm_global=$(contract_file_contains "${FBDEV_QEMU_LOG}" "shm_global_started=true")
graphics_drm_wayland_xdg_shell_global=$(contract_file_contains "${FBDEV_QEMU_LOG}" "xdg_shell_global_started=true")
graphics_drm_wayland_seat=$(contract_file_contains "${FBDEV_QEMU_LOG}" "seat_started=true")
graphics_drm_wayland_socket_cleaned=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_socket_cleaned=true")
graphics_drm_wayland_recovery_safe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")
graphics_drm_wayland_input_source=$(contract_file_contains "${FBDEV_QEMU_LOG}" "input_source=libinput-udev")
graphics_drm_wayland_input_discovery=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_discovery_ready=true")
graphics_drm_wayland_input_keyboard=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_dispatch_ready=true")
graphics_drm_wayland_input_selective_forward=$(contract_file_contains "${FBDEV_QEMU_LOG}" "external_client_keyboard_event_received=true")
graphics_drm_wayland_input_pointer_hit_test=$(contract_file_contains "${FBDEV_QEMU_LOG}" "external_client_pointer_event_received=true")
graphics_drm_wayland_input_pointer_motion=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_pointer_motion_events=11")
graphics_drm_wayland_input_pointer_button=$(contract_file_numeric_at_least "${FBDEV_QEMU_LOG}" "drm_wayland_input_pointer_button_events=" 18)
graphics_drm_wayland_input_launcher=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_launcher_visible=true")
graphics_drm_wayland_launcher_overlay=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_overlay_rendered=true")
graphics_drm_wayland_launcher_pointer=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_launch_request_app=files")
graphics_drm_wayland_launcher_preflight=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_launch_rejection_reason=accepted")
graphics_drm_wayland_launcher_process=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_process_reaped=true")
graphics_drm_wayland_process_supervisor=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_process_supervisor_duplicate_rejected=true")
graphics_session_supervisor_qemu=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-TEST] stage=graphical-session-supervisor-qemu status=ok")
graphics_session_supervisor_qemu_failures=$(contract_file_contains "${FBDEV_QEMU_LOG}" "real_compositor_failures=3")
graphics_session_supervisor_qemu_restarts=$(contract_file_contains "${FBDEV_QEMU_LOG}" "bounded_restarts=2")
graphics_session_supervisor_qemu_recovery=$(contract_file_contains "${FBDEV_QEMU_LOG}" "recovery_return=ok")
graphics_boot_qemu=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-boot-qemu status=ok")
graphics_boot_qemu_activation=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "activation=supervised")
graphics_boot_qemu_wayland=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "drm_wayland=active")
graphics_boot_qemu_persistent=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "persistent=true")
graphics_boot_qemu_desktop_event_loop=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "scenario=desktop-event-loop fixtures=false recovery_tty=available")
graphics_boot_qemu_fixture_free=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "persistent=true scenario=desktop-event-loop fixtures=false")
graphics_desktop_runtime_launch_qemu=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-launch-qemu status=ok")
graphics_desktop_runtime_launch_files=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "app=files surface=aqua.files")
graphics_desktop_runtime_launch_repaint=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "repaint=true supervised=true")
graphics_desktop_runtime_settings=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-settings-qemu status=ok app=settings surface=aqua.settings clients=2 launcher_closed=true")
audio_adapter_qemu_safe_default=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=audio-adapter-qemu status=ok health=unavailable controls=false backend_applied=false packages=absent")
graphics_desktop_live_theme=$(contract_file_contains "${LIVE_THEME_QEMU_LOG}" "[AQUA-TEST] stage=desktop-live-theme-qemu status=ok from=Light to=Dark shell=true apps=files,settings restart=false frame_delta=true")
graphics_desktop_runtime_damage=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-damage-qemu status=ok app=settings interaction=keyboard-category-selected repaint=incremented revision=changed")
graphics_desktop_runtime_close=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-close-qemu status=ok app=settings close=alt-f4 exit=clean stale_surface=removed restart=never clients=1")
graphics_desktop_runtime_unexpected_exit=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-unexpected-exit-qemu status=ok app=files exit=forced stale_surface=removed restart=never active_count=0 clients=0")
graphics_desktop_runtime_cleanup=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-cleanup-qemu status=ok apps=files,settings lifecycle_clean=true active_count=0 stale_surfaces=0")
graphics_desktop_session_menu=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-session-menu-qemu status=ok actions=logout,restart,shutdown,recovery confirmation=true selected=recovery execution=return-to-recovery")
graphics_desktop_session_menu_overlay=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "desktop_session_menu_overlay_texture_ready=true")
graphics_boot_qemu_recovery_tty=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "recovery_tty=available")
graphics_stop_qemu=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-stop-qemu status=ok")
graphics_stop_qemu_clients=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "clients_stopped=true")
graphics_stop_qemu_kms=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "kms_restored=true")
graphics_stop_qemu_gbm=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "gbm_released=true")
graphics_stop_qemu_pid=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "pid_cleaned=true")
graphics_stop_qemu_recovery=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "recovery_return=ok")
graphics_restart_qemu=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-restart-qemu status=ok")
graphics_session_cycle_qemu=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-session-cycle-qemu status=ok")
graphics_session_cycle_qemu_sockets=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "sockets_clean=true")
graphics_session_cycle_qemu_pids=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "pids_clean=true")
graphics_session_cycle_qemu_clients=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "clients_clean=true")
graphics_session_cycle_qemu_drm=$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "drm_clean=true")
graphics_drm_wayland_settings=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_process_reaped=true")
graphics_drm_wayland_settings_interaction=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_keyboard_category=Desktop")
graphics_drm_wayland_settings_persistence=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_reload_verified=true")
graphics_drm_wayland_settings_desktop=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_persisted_desktop_icons=false")
graphics_drm_wayland_settings_input=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_persisted_key_repeat=false")
graphics_drm_wayland_settings_network=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_network_management=false")
graphics_drm_wayland_real_font=$(contract_file_contains "${FBDEV_QEMU_LOG}" "aqua_settings_font_ready=true")
graphics_drm_wayland_launcher_surface_owner=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_surface_owned=true")
graphics_drm_wayland_files_window=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_window_repaint_complete=true")
graphics_drm_wayland_files_read_only=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_directory_enumerated=true")
graphics_drm_wayland_files_pointer=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_selection_commit=true")
graphics_drm_wayland_files_navigation=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_sidebar_navigation=Pictures")
graphics_drm_wayland_files_keyboard=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_keyboard_activation=Projects")
graphics_drm_wayland_files_hover=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_hover_feedback=true")
graphics_drm_wayland_files_scroll=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_scroll_offset=1")
graphics_drm_wayland_files_safe_preview=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_arbitrary_execution=false")
graphics_drm_wayland_files_wheel=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_pointer_wheel=true")
graphics_drm_wayland_files_page_keys=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_page_up=true")
graphics_drm_wayland_files_edge_keys=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_end_key=true")
graphics_drm_wayland_files_scrollbar_drag=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_scrollbar_drag=true")
graphics_drm_wayland_files_preview_scroll=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_preview_scroll_offset=1")
graphics_drm_wayland_input_dispatch=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_dispatch_ready=true")
graphics_drm_wayland_external_client=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_ready=true")
graphics_drm_wayland_external_client_buffer=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_buffer_bytes=")
graphics_drm_wayland_third_party_client=$(contract_file_contains "${FBDEV_QEMU_LOG}" "third_party_wayland_client=weston-simple-shm")
graphics_drm_wayland_no_weston_compositor=$(contract_file_contains "${FBDEV_QEMU_LOG}" "weston_compositor_started=false")
graphics_drm_wayland_external_client_multi_surface=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_surface_count=2")
graphics_drm_wayland_external_client_independent_buffers=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_independent_buffers=true")
graphics_drm_wayland_external_client_composited=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_composited=true")
graphics_drm_wayland_external_client_frame_callback=$(contract_file_contains "${FBDEV_QEMU_LOG}" "external_client_frame_callback_received=true")
graphics_drm_wayland_external_client_damage=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_damage_ready=true")
graphics_drm_wayland_external_client_focus=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_keyboard_focus=true")
graphics_drm_wayland_external_client_focus_change=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_focus_changes=8")
graphics_drm_wayland_external_client_stacking=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_stacking_changes=8")
graphics_drm_wayland_stacking_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_stacking_repaint_complete=true")
graphics_drm_wayland_stacking_repaint_changed=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_stacking_repaint_changed_frame=true")
graphics_drm_wayland_client_cleanup=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_complete=true")
graphics_drm_wayland_client_cleanup_focus=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_keyboard_focus_reassigned=true")
graphics_drm_wayland_client_cleanup_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_repaint_complete=true")
graphics_drm_wayland_interactive_geometry=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_interactive_geometry_applied=true")
graphics_drm_wayland_state_cycle=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_state_cycle_complete=true")
graphics_drm_wayland_state_configure=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_state_configure_acks=9")
graphics_drm_wayland_close=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_request_sent=true")
graphics_drm_wayland_close_cleanup=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_cleanup_surfaces=0")
graphics_drm_wayland_close_repaint=$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_repaint_complete=true")
graphics_evdev_aqua_seat=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-INPUT] stage=evdev-aqua-seat status=ok")
graphics_evdev_keyboard=$(contract_file_contains "${FBDEV_QEMU_LOG}" "keyboard_events=1")
graphics_evdev_pointer_motion=$(contract_file_contains "${FBDEV_QEMU_LOG}" "pointer_motion_events=2")
graphics_evdev_pointer_button=$(contract_file_contains "${FBDEV_QEMU_LOG}" "pointer_button_events=2")
graphics_evdev_launcher=$(contract_file_contains "${FBDEV_QEMU_LOG}" "launcher_visible=true")
graphics_drm_wayland_capture=$(contract_file_contains "${WAYLAND_SESSION_QEMU_CAPTURE}" "status=ok")
graphics_drm_wayland_capture_dimensions=$(contract_file_contains "${WAYLAND_SESSION_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_drm_wayland_ppm_checksum=$(capture_checksum_status "${WAYLAND_SESSION_QEMU_PPM}" "ppm_sha256" "${WAYLAND_SESSION_QEMU_CAPTURE}")
graphics_drm_wayland_png_checksum=$(capture_checksum_status "${WAYLAND_SESSION_QEMU_PNG}" "png_sha256" "${WAYLAND_SESSION_QEMU_CAPTURE}")
session_config_probe=$(contract_file_contains "${CONTRACT_DIR}/session-config.txt" "[AQUA-COMPOSITOR] stage=session-config status=ok")
session_config_runtime_dir=$(contract_file_contains "${CONTRACT_DIR}/session-config.txt" "runtime_dir=/run/user/1000")
session_env_probe=$(contract_file_contains "${CONTRACT_DIR}/session-env.txt" "[AQUA-COMPOSITOR] stage=session-env status=ok")
session_env_wayland=$(contract_file_contains "${CONTRACT_DIR}/session-env.txt" "WAYLAND_DISPLAY=aqua-wayland-0")
session_bootstrap_probe=$(contract_file_contains "${CONTRACT_DIR}/session-bootstrap.txt" "[AQUA-COMPOSITOR] stage=session-bootstrap status=ok")
session_bootstrap_runtime=$(contract_file_contains "${CONTRACT_DIR}/session-bootstrap.txt" "runtime_dir_prepared=ok")
session_bootstrap_no_graphics=$(contract_file_contains "${CONTRACT_DIR}/session-bootstrap.txt" "session_started=false")
session_check_probe=$(contract_file_contains "${CONTRACT_DIR}/session-check.txt" "[AQUA-SESSION] stage=session-check status=ok no_graphics=true")
manual_launch_plan=$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "[AQUA-MANUAL] stage=compositor-launch-plan status=ok")
manual_launch_safe=$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "safe_to_run_from_recovery=ok")
manual_launch_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "autostart=false")
manual_launch_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "boot_graphics=false")
manual_launch_recovery_tty=$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "fallback_tty_required=true")
manual_launch_no_display_start=$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "starts_display_output=false")
manual_launch_no_shell_start=$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "starts_desktop_shell=false")
guarded_run=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "[AQUA-GUARDED] stage=compositor-bounded-run status=ok")
guarded_run_complete=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "run_status=qemu-safe-guarded-compositor-run-complete")
guarded_run_launch_plan=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "launch_plan_ready=ok")
guarded_run_bounded=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "bounded_run_complete=ok")
guarded_run_frames=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "bounded_run_frames=3")
guarded_run_started=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "display_output_started=true")
guarded_run_stopped=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "display_output_stopped=true")
guarded_run_fallback_tty=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "fallback_tty_available=true")
guarded_run_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "autostart=false")
guarded_run_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "boot_graphics=false")
guarded_run_no_shell=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "desktop_shell_started=false")
guarded_run_return=$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "safe_return_to_recovery=ok")
graphical_session_supervisor=$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "[AQUA-SESSION] stage=graphical-session-supervisor status=ok")
graphical_session_supervisor_bounded=$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "policy=bounded-restart-with-recovery-fallback")
graphical_session_supervisor_recovery=$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "recovery_fallback=armed")
graphical_session_supervisor_safe_default=$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "session_started=false")
media_service_supervisor=$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "[AQUA-MEDIA] stage=media-service-supervisor status=ok")
media_service_supervisor_safe_default=$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "enabled=false")
media_service_supervisor_ordered_start=$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "ordered_start=pipewire,wireplumber")
media_service_supervisor_ordered_stop=$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "ordered_stop=wireplumber,pipewire")
graphical_session_boot=$(contract_file_contains "${CONTRACT_DIR}/graphical-session-boot.txt" "[AQUA-BOOT] stage=graphical-session-activation status=disabled")
graphical_session_boot_kernel_gate=$(contract_file_contains "${CONTRACT_DIR}/graphical-session-boot.txt" "reason=kernel-flag-absent")
graphical_session_boot_safe_default=$(contract_file_contains "${CONTRACT_DIR}/graphical-session-boot.txt" "boot_graphics=false")
handoff_gate=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "[AQUA-GATE] stage=nested-preview-handoff status=ok")
handoff_gate_ready=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "gate_status=qemu-safe-nested-preview-handoff-gate-ready")
handoff_gate_guarded=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "guarded_run_ready=ok")
handoff_gate_handoff=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "handoff_ready=ok")
handoff_gate_preview=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "visible_preview_ready=ok")
handoff_gate_loop=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "preview_loop_ready=ok")
handoff_gate_backend=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_nested_backend_gate=ok")
handoff_gate_backend_ready=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_backend_ready=ok")
handoff_gate_backend_no_start=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_backend_no_display_start=ok")
handoff_gate_candidate=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "candidate_path=manual-nested-preview")
handoff_gate_manual=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_operator_required=true")
handoff_gate_no_auto=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "automatic_promotion=false")
handoff_gate_recovery=$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "safe_to_remain_in_recovery=ok")
output_plan=$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "[AQUA-COMPOSITOR] stage=output-plan-probe status=ok")
output_plan_backend=$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "primary_backend=nested-dev-window")
output_plan_later_backend=$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "later_backend=qemu-drm-kms")
output_plan_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "recovery_safe=ok")
display_output_handoff=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "[AQUA-COMPOSITOR] stage=display-output-handoff status=ok")
display_output_handoff_ready=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "handoff_status=display-output-handoff-ready")
display_output_handoff_backend=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "target_backend=nested-dev-window")
display_output_handoff_framebuffer=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "frame_buffer_bytes=6291456")
display_output_handoff_frame_format=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "frame_format=raw-rgba8888-composited-client-preview")
display_output_handoff_frame_checksum=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "frame_checksum=")
display_output_handoff_client_snapshot=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "client_layer_buffer_snapshot_bytes=")
display_output_handoff_snapshot_mode=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "client_layer_snapshot_mode=full-buffer-snapshot")
display_output_handoff_no_start=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "display_output_started=false")
display_output_handoff_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "recovery_safe=ok")
display_activation_plan=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "[AQUA-COMPOSITOR] stage=display-activation-plan status=ok")
display_activation_plan_ready=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "activation_status=manual-display-activation-plan-ready")
display_activation_plan_manual=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "launch_mode=manual-dev")
display_activation_plan_handoff=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "source_handoff_ready=ok")
display_activation_plan_frame=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "frame_format=raw-rgba8888-composited-client-preview")
display_activation_plan_fallback_tty=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "fallback_tty_required=true")
display_activation_plan_can_activate=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "can_activate_display_output=ok")
display_activation_plan_no_start=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "display_output_started=false")
display_activation_plan_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "autostart=false")
display_activation_plan_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "recovery_safe=ok")
display_output_smoke=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "[AQUA-COMPOSITOR] stage=display-output-smoke status=ok")
display_output_smoke_complete=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "smoke_status=manual-display-output-smoke-complete")
display_output_smoke_started=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "display_output_started=true")
display_output_smoke_stopped=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "display_output_stopped=true")
display_output_smoke_frames=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "presented_frames=3")
display_output_smoke_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "autostart=false")
display_output_smoke_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "boot_graphics=false")
display_output_smoke_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "recovery_safe=ok")
nested_output_surface=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "[AQUA-COMPOSITOR] stage=nested-output-surface status=ok")
nested_output_surface_complete=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_status=nested-output-surface-lifecycle-complete")
nested_output_surface_acquired=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_acquired=ok")
nested_output_surface_configured=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_configured=ok")
nested_output_surface_frame_attached=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "frame_attached=ok")
nested_output_surface_frame_presented=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "frame_presented=ok")
nested_output_surface_released=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_released=ok")
nested_output_surface_frames=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "presented_frames=3")
nested_output_surface_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "autostart=false")
nested_output_surface_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "boot_graphics=false")
nested_output_surface_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "recovery_safe=ok")
visible_preview_plan=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "[AQUA-COMPOSITOR] stage=visible-preview-plan-probe status=ok")
visible_preview_output=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "output_plan_ready=ok")
visible_preview_frame_buffer=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "frame_buffer_ready=ok")
visible_preview_raster=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "raster_ready=ok")
visible_preview_png_export=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "png_export_ready=ok")
visible_preview_client_layer_pipeline=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_pipeline_ready=ok")
visible_preview_client_layer_count=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_count=2")
visible_preview_client_layer_checksum=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_checksum=")
visible_preview_client_layer_buffer_snapshot=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_buffer_snapshot_bytes=")
visible_preview_client_layer_snapshot_mode=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_snapshot_mode=full-buffer-snapshot")
visible_preview_window_not_started=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "preview_window_started=false")
visible_preview_export=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "[AQUA-COMPOSITOR] stage=visible-preview-export-probe status=ok")
visible_preview_export_format=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "format=html-data-uri-png-preview")
visible_preview_export_bytes=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "html_bytes=")
visible_preview_export_checksum=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "html_checksum=")
visible_preview_export_client_layers=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_pipeline_ready=ok")
visible_preview_export_client_layers_composited=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_composited=ok")
visible_preview_export_client_layer_buffer_snapshot=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_buffer_snapshot_bytes=")
visible_preview_export_client_layer_snapshot_mode=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_snapshot_mode=full-buffer-snapshot")
visible_preview_export_png_checksum=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "png_checksum=")
nested_preview_loop=$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "[AQUA-COMPOSITOR] stage=nested-preview-loop status=ok")
nested_preview_frame_clock=$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "frame_clock_started=ok")
nested_preview_frames=$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "rendered_frames=3")
nested_preview_manual=$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "manual_start_required=true")
nested_preview_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "autostart=false")
manual_nested_preview_backend=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "[AQUA-COMPOSITOR] stage=manual-nested-preview-backend status=ok")
manual_nested_preview_backend_ready=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "backend_status=manual-nested-preview-backend-ready")
manual_nested_preview_backend_path=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "backend_path=nested-dev-window")
manual_nested_preview_backend_selected=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "backend_selected=ok")
manual_nested_preview_backend_handoff=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "handoff_ready=ok")
manual_nested_preview_backend_surface=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "surface_lifecycle_ready=ok")
manual_nested_preview_backend_loop=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "frame_loop_ready=ok")
manual_nested_preview_backend_export=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "visible_export_ready=ok")
manual_nested_preview_backend_frame=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "frame_format=raw-rgba8888-composited-client-preview")
manual_nested_preview_backend_checksum_match=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "frame_checksum_matches_surface=ok")
manual_nested_preview_backend_fallback_tty=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "fallback_tty_available=true")
manual_nested_preview_backend_no_start=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "display_output_started=false")
manual_nested_preview_backend_stopped=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "display_output_stopped=true")
manual_nested_preview_backend_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "autostart=false")
manual_nested_preview_backend_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "recovery_safe=ok")
manual_nested_preview_execution=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "[AQUA-PREVIEW] stage=manual-nested-preview-execution status=ok")
manual_nested_preview_execution_ready=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "execution_status=qemu-safe-manual-nested-preview-execution-ready")
manual_nested_preview_execution_gate=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "handoff_gate_ready=ok")
manual_nested_preview_execution_backend=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "handoff_gate_manual_backend=ok")
manual_nested_preview_execution_operator=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "operator_acknowledged=true")
manual_nested_preview_execution_frames=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "bounded_frames=3")
manual_nested_preview_execution_started=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "display_output_started=true")
manual_nested_preview_execution_stopped=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "display_output_stopped=true")
manual_nested_preview_execution_cleanup=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "execution_cleanup=ok")
manual_nested_preview_execution_return=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "safe_return_to_recovery=ok")
manual_nested_preview_execution_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "autostart=false")
manual_nested_preview_execution_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "boot_graphics=false")
manual_nested_preview_execution_probe=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution-probe.txt" "[AQUA-COMPOSITOR] stage=manual-nested-preview-execution status=ok")
manual_nested_preview_execution_probe_complete=$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution-probe.txt" "execution_status=manual-nested-preview-execution-complete")
visible_preview_request=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "[AQUA-PREVIEW] stage=visible-nested-preview-request status=ok")
visible_preview_request_ready=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_status=qemu-safe-visible-nested-preview-request-ready")
visible_preview_request_target=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_target=host-visible-nested-window")
visible_preview_request_backend=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "window_backend=minifb")
visible_preview_request_feature_gate=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "feature_gate=host-window-preview")
visible_preview_request_manual_execution=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "manual_execution_ready=ok")
visible_preview_request_file=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_file_written=ok")
visible_preview_request_host_tool_not_packaged=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "host_tool_packaged=false")
visible_preview_request_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_autostart=false")
visible_preview_request_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_boot_graphics=false")
visible_preview_request_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "safe_return_to_recovery=ok")
visible_preview_launch=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "[AQUA-PREVIEW] stage=visible-nested-preview-launch status=ok")
visible_preview_launch_ready=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_status=qemu-safe-visible-nested-preview-launch-ready")
visible_preview_launch_request=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "request_command_ready=ok")
visible_preview_launch_plan=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_plan_written=ok")
visible_preview_launch_backend=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_window_backend=minifb")
visible_preview_launch_feature_gate=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_feature_gate=host-window-preview")
visible_preview_launch_host_tool_not_packaged=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_host_tool_packaged=false")
visible_preview_launch_no_qemu_window=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_qemu_window_started=false")
visible_preview_launch_no_preview_window=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_preview_window_started=false")
visible_preview_launch_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_autostart=false")
visible_preview_launch_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_boot_graphics=false")
visible_preview_launch_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "safe_return_to_recovery=ok")
recovery_help=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "[AQUA-RECOVERY] stage=operator-help status=ok")
recovery_help_text_mode=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "mode=text-recovery")
recovery_help_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "autostart=false")
recovery_help_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "boot_graphics=false")
recovery_help_visible_launcher=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "aqua-visible-preview-launch")
recovery_help_host_tool_external=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "aqua-host-tools --features host-window-preview -- smoke-manual-execution-window")
recovery_help_operator_pass_host=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "scripts/run-qemu-visible-operator-pass.sh")
recovery_help_operator_pass_no_launch=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh")
recovery_help_operator_checklist=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "build/qemu-visible-operator-checklist.md")
recovery_help_operator_pass_artifact=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "build/qemu-visible-operator-pass.txt")
recovery_help_operator_pass_external=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "Host QEMU operator-pass tooling is not packaged into the Buildroot rootfs.")
recovery_help_visible_pass_report=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "aqua-qemu-visible-pass-report")
recovery_help_pass_report_required=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "The QEMU visible manual runbook requires pass_report_required=true.")
recovery_help_pass_report_after_apply=$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "Run aqua-qemu-visible-pass-report after confirmed evidence bundle apply.")
qemu_visible_operator_pass_no_launch=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "status=no-launch-ready")
qemu_visible_operator_pass_launch_required=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "launch_confirmation_required=true")
qemu_visible_operator_pass_not_confirmed=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "launch_confirmed=false")
qemu_visible_operator_pass_no_positive_without_evidence=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "no_positive_observation_without_evidence=true")
qemu_visible_operator_pass_no_unverified_bundle=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "no_unverified_bundle_acceptance=true")
qemu_visible_operator_pass_recovery_safe=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "safe_return_to_recovery=ok")
qemu_visible_operator_pass_stop_rule=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "operator_pass_stop_rule=Do not mark VM display observed")
qemu_visible_operator_pass_no_launch_command=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "no_launch_rehearsal_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh")
qemu_visible_operator_pass_confirmed_command=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "confirmed_launch_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh")
qemu_visible_operator_pass_capture_flow=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_capture_flow_command=scripts/run-qemu-visible-ready-capture-flow.sh")
qemu_visible_operator_pass_capture_verify=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_capture_verify_command=scripts/verify-qemu-visible-capture.sh")
qemu_visible_operator_pass_evidence_flow=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_evidence_flow_command=scripts/run-qemu-visible-evidence-flow.sh")
qemu_visible_operator_pass_vm_apply=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_vm_apply_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply")
qemu_visible_operator_pass_capture_hash_gate=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "capture_hash_verification_required=true")
qemu_visible_operator_pass_capture_hash_status=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "bundle_capture_hash_status=ok")
qemu_visible_operator_pass_positive_capture_hash_status=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "bundle_positive_capture_hash_status=ok")
qemu_visible_operator_pass_missing_capture_hash_rejected=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "bundle_missing_capture_hash_rejected_status=ok")
qemu_visible_operator_pass_manual_runbook_pass_report_required=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "manual_runbook_pass_report_required_status=ok")
qemu_visible_operator_pass_preflight_source=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "preflight_source_sha256=")
qemu_visible_operator_pass_preflight_mtime=$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "preflight_source_mtime_utc=")
operator_transcript=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "[AQUA-RECOVERY] stage=operator-transcript status=ok")
operator_transcript_ready=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_status=qemu-safe-operator-transcript-ready")
operator_transcript_dry_run=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_dry_run=true")
operator_transcript_qemu_steps=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_qemu_steps=9")
operator_transcript_host_steps=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_host_steps=2")
operator_transcript_qemu_command=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "next_qemu_command=/usr/bin/aqua-graphics-enable-gate")
operator_transcript_host_command=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "next_host_command=aqua-host-tools --features host-window-preview -- smoke-manual-execution-window")
operator_transcript_host_tool_not_packaged=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "host_tool_packaged=false")
operator_transcript_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "autostart=false")
operator_transcript_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "boot_graphics=false")
operator_transcript_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "safe_return_to_recovery=ok")
graphics_enable_gate=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "[AQUA-GATE] stage=graphics-enable-gate status=ok")
graphics_enable_gate_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "gate_status=qemu-safe-graphics-enable-gate-ready")
graphics_enable_gate_preflight=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "preflight_status=evaluated")
graphics_enable_gate_allow_handoff=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_handoff_gate_ok=true")
graphics_enable_gate_allow_manual_execution=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_manual_execution_ok=true")
graphics_enable_gate_allow_fallback_tty=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_fallback_tty_supervised=true")
graphics_enable_gate_allow_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_boot_graphics_explicitly_enabled=true")
graphics_enable_gate_check_handoff=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_handoff_gate=ok")
graphics_enable_gate_check_manual_execution=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_manual_execution=ok")
graphics_enable_gate_check_fallback_tty=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_fallback_tty=ok")
graphics_enable_gate_check_cleanup=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_manual_cleanup=ok")
graphics_enable_gate_check_stopped=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_manual_stopped=ok")
graphics_enable_gate_currently_blocked=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "currently_allowable=false")
graphics_enable_gate_refused=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "enable_decision=refuse")
graphics_enable_gate_reason=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "refuse_reason=boot-graphics-disabled-until-fail-safe-compositor")
graphics_enable_gate_blocked_criteria=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "blocked_criteria=boot_graphics=false")
graphics_enable_gate_plan=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "gate_plan_written=ok")
graphics_enable_gate_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "autostart=false")
graphics_enable_gate_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "boot_graphics=false")
graphics_enable_gate_no_display_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "display_output_started=false")
graphics_enable_gate_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "safe_return_to_recovery=ok")
graphics_enable_gate_positive=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "[AQUA-GATE] stage=graphics-enable-gate status=ok")
graphics_enable_gate_positive_preflight=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "preflight_status=evaluated")
graphics_enable_gate_positive_dry_run=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "positive_dry_run=true")
graphics_enable_gate_positive_handoff=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "check_handoff_gate=ok")
graphics_enable_gate_positive_manual_execution=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "check_manual_execution=ok")
graphics_enable_gate_positive_allowable=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "currently_allowable=true")
graphics_enable_gate_positive_decision=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "enable_decision=allow-dry-run")
graphics_enable_gate_positive_no_actual_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "actual_graphics_started=false")
graphics_enable_gate_positive_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "boot_graphics=false")
graphics_enable_gate_positive_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "safe_return_to_recovery=ok")
graphics_launch_candidate=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "[AQUA-GATE] stage=graphics-launch-candidate status=ok")
graphics_launch_candidate_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_status=qemu-safe-graphics-launch-candidate-ready")
graphics_launch_candidate_allowable=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_allowable=true")
graphics_launch_candidate_selected=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_selected=true")
graphics_launch_candidate_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_started=false")
graphics_launch_candidate_no_actual_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "actual_graphics_started=false")
graphics_launch_candidate_no_display_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "display_output_started=false")
graphics_launch_candidate_rollback=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "rollback_strategy=return-to-text-recovery")
graphics_launch_candidate_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "safe_return_to_recovery=ok")
graphics_rollback_drill=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "[AQUA-GATE] stage=graphics-rollback-drill status=ok")
graphics_rollback_drill_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "rollback_drill_status=qemu-safe-rollback-drill-ready")
graphics_rollback_drill_cancel_path=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "operator_cancel_simulated=true")
graphics_rollback_drill_failure_path=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "startup_failure_simulated=true")
graphics_rollback_drill_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "candidate_started=false")
graphics_rollback_drill_no_actual_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "actual_graphics_started=false")
graphics_rollback_drill_no_display_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "display_output_started=false")
graphics_rollback_drill_verified=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "rollback_verified=true")
graphics_rollback_drill_command=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "rollback_command=/usr/bin/aqua-recovery")
graphics_rollback_drill_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "safe_return_to_recovery=ok")
graphics_startup_preflight=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "[AQUA-GATE] stage=graphics-startup-preflight status=ok")
graphics_startup_preflight_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "startup_preflight_status=qemu-safe-guarded-startup-preflight-ready")
graphics_startup_preflight_bounded=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "bounded_startup_candidate=true")
graphics_startup_preflight_operator_ack=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "operator_ack_required=true")
graphics_startup_preflight_fallback_tty=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "fallback_tty_available=true")
graphics_startup_preflight_rollback=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "rollback_verified=true")
graphics_startup_preflight_decision=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "startup_preflight_decision=allow-bounded-manual-preflight-only")
graphics_startup_preflight_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "candidate_started=false")
graphics_startup_preflight_no_actual_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "actual_graphics_started=false")
graphics_startup_preflight_no_display_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "display_output_started=false")
graphics_startup_preflight_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "safe_return_to_recovery=ok")
graphics_startup_rehearsal=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "[AQUA-GATE] stage=graphics-startup-rehearsal status=ok")
graphics_startup_rehearsal_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "startup_rehearsal_status=qemu-safe-guarded-startup-rehearsal-complete")
graphics_startup_rehearsal_bounded=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "bounded_startup_rehearsal=true")
graphics_startup_rehearsal_frames=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "bounded_run_frames=3")
graphics_startup_rehearsal_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "display_output_started=true")
graphics_startup_rehearsal_stopped=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "display_output_stopped=true")
graphics_startup_rehearsal_no_actual_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "actual_graphics_started=false")
graphics_startup_rehearsal_no_desktop_shell=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "desktop_shell_started=false")
graphics_startup_rehearsal_decision=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "startup_rehearsal_decision=allow-next-manual-qemu-display-step")
graphics_startup_rehearsal_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "safe_return_to_recovery=ok")
graphics_qemu_display_gate=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "[AQUA-GATE] stage=graphics-qemu-display-gate status=ok")
graphics_qemu_display_gate_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_gate_status=qemu-safe-manual-display-step-gate-ready")
graphics_qemu_display_gate_candidate=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_step_candidate=true")
graphics_qemu_display_gate_passed=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_gate_passed=true")
graphics_qemu_display_gate_decision=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_gate_decision=allow-manual-qemu-display-step")
graphics_qemu_display_gate_manual=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "manual_start_required=true")
graphics_qemu_display_gate_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "visible_qemu_step_started=false")
graphics_qemu_display_gate_no_actual_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "actual_graphics_started=false")
graphics_qemu_display_gate_no_desktop_shell=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "desktop_shell_started=false")
graphics_qemu_display_gate_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "safe_return_to_recovery=ok")
graphics_visible_qemu_attempt=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "[AQUA-GATE] stage=graphics-visible-qemu-attempt status=ok")
graphics_visible_qemu_attempt_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "visible_qemu_attempt_status=qemu-safe-visible-attempt-plan-ready")
graphics_visible_qemu_attempt_plan=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "attempt_plan_written=true")
graphics_visible_qemu_attempt_allowed=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "visible_qemu_attempt_allowed=true")
graphics_visible_qemu_attempt_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "visible_qemu_attempt_started=false")
graphics_visible_qemu_attempt_manual=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "manual_start_required=true")
graphics_visible_qemu_attempt_fallback_tty=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "fallback_tty_available=true")
graphics_visible_qemu_attempt_no_desktop_shell=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "desktop_shell_started=false")
graphics_visible_qemu_attempt_command=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "qemu_attempt_command=/usr/bin/aqua-compositor-guarded-run")
graphics_visible_qemu_attempt_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "safe_return_to_recovery=ok")
graphics_visible_attempt_transcript=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "[AQUA-GATE] stage=graphics-visible-attempt-transcript status=ok")
graphics_visible_attempt_transcript_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "visible_attempt_transcript_status=qemu-safe-visible-attempt-transcript-ready")
graphics_visible_attempt_transcript_sequence=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "operator_sequence_ready=true")
graphics_visible_attempt_transcript_step_attempt=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "operator_step_2=/usr/bin/aqua-graphics-visible-qemu-attempt")
graphics_visible_attempt_transcript_step_run=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "operator_step_3=/usr/bin/aqua-compositor-guarded-run")
graphics_visible_attempt_transcript_expected_return=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "expected_return=safe-return-to-recovery")
graphics_visible_attempt_transcript_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "visible_qemu_attempt_started=false")
graphics_visible_attempt_transcript_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "persistent_graphical_session_started=false")
graphics_visible_attempt_transcript_no_desktop_shell=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "desktop_shell_started=false")
graphics_visible_attempt_transcript_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "safe_return_to_recovery=ok")
graphics_visible_attempt_result=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "[AQUA-GATE] stage=graphics-visible-attempt-result status=ok")
graphics_visible_attempt_result_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "visible_attempt_result_status=qemu-safe-visible-attempt-result-ready")
graphics_visible_attempt_result_source=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "source_transcript_ready=ok")
graphics_visible_attempt_result_manual_not_run=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "attempt_result=manual-not-run")
graphics_visible_attempt_result_collected=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "attempt_result_collected=true")
graphics_visible_attempt_result_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "visible_qemu_attempt_started=false")
graphics_visible_attempt_result_not_completed=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "visible_qemu_attempt_completed=false")
graphics_visible_attempt_result_no_display_start=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "display_output_started=false")
graphics_visible_attempt_result_no_display_stop=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "display_output_stopped=false")
graphics_visible_attempt_result_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "persistent_graphical_session_started=false")
graphics_visible_attempt_result_no_desktop_shell=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "desktop_shell_started=false")
graphics_visible_attempt_result_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "boot_graphics=false")
graphics_visible_attempt_result_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "autostart=false")
graphics_visible_attempt_result_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "safe_return_to_recovery=ok")
graphics_visible_attempt_runner=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "[AQUA-GATE] stage=graphics-visible-attempt-runner status=ok")
graphics_visible_attempt_runner_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "visible_attempt_runner_status=qemu-safe-visible-attempt-runner-complete")
graphics_visible_attempt_runner_guarded=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_guarded_run=ok")
graphics_visible_attempt_runner_bounded=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_bounded_run=ok")
graphics_visible_attempt_runner_result=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_result=completed-bounded-run")
graphics_visible_attempt_runner_collector=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_result_collector=ok")
graphics_visible_attempt_runner_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "visible_qemu_attempt_started=true")
graphics_visible_attempt_runner_completed=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "visible_qemu_attempt_completed=true")
graphics_visible_attempt_runner_frames=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "bounded_run_frames=3")
graphics_visible_attempt_runner_display_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "display_output_started=true")
graphics_visible_attempt_runner_display_stopped=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "display_output_stopped=true")
graphics_visible_attempt_runner_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "persistent_graphical_session_started=false")
graphics_visible_attempt_runner_no_desktop_shell=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "desktop_shell_started=false")
graphics_visible_attempt_runner_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "boot_graphics=false")
graphics_visible_attempt_runner_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "autostart=false")
graphics_visible_attempt_runner_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "safe_return_to_recovery=ok")
graphics_qemu_visible_boot_check=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "[AQUA-GATE] stage=graphics-qemu-visible-boot-check status=ok")
graphics_qemu_visible_boot_check_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "qemu_visible_boot_check_status=qemu-visible-boot-path-check-ready")
graphics_qemu_visible_boot_check_runner=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "runner_result_completed=ok")
graphics_fbdev_present=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "[AQUA-GATE] stage=graphics-fbdev-present status=dry-run-ok")
graphics_fbdev_present_probe=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "fbdev_probe=ok")
graphics_fbdev_present_frame=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "target_frame_bytes=3145728")
graphics_fbdev_present_not_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "visible_frame_presented=false")
graphics_fbdev_present_bounded=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "bounded_frames=1")
graphics_fbdev_present_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "boot_graphics=false")
graphics_fbdev_present_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "autostart=false")
graphics_fbdev_present_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "safe_return_to_recovery=ok")
graphics_fbdev_headless_qemu_write=$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-TEST] stage=fbdev-present-qemu status=ok framebuffer_write=true visible_observation=false safe_return_to_recovery=ok")
graphics_fbdev_headless_qemu_checksum=$(contract_file_contains "${FBDEV_QEMU_LOG}" "frame_checksum=c85dbfbfc17843af")
graphics_fbdev_headless_qemu_wallpaper=$(contract_file_contains "${FBDEV_QEMU_LOG}" "wallpaper_source=runtime-asset")
graphics_fbdev_headless_qemu_mode=$(contract_file_contains "${FBDEV_QEMU_LOG}" "target_size=1280x800")
graphics_fbdev_headless_qemu_recovery_safe=$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")
graphics_fbdev_headless_qemu_capture=$(contract_file_contains "${FBDEV_QEMU_CAPTURE}" "status=ok")
graphics_fbdev_headless_qemu_capture_dimensions=$(contract_file_contains "${FBDEV_QEMU_CAPTURE}" "dimensions=1280x800")
graphics_fbdev_headless_qemu_ppm_checksum=$(capture_checksum_status "${FBDEV_QEMU_PPM}" "ppm_sha256")
graphics_fbdev_headless_qemu_png_checksum=$(capture_checksum_status "${FBDEV_QEMU_PNG}" "png_sha256")
graphics_fbdev_headless_qemu_capture_unobserved=$(contract_file_contains "${FBDEV_QEMU_CAPTURE}" "visible_observation=false")
graphics_qemu_visible_boot_path_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "qemu_visible_boot_path_ready=true")
graphics_qemu_visible_boot_observed_false=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "qemu_vm_display_observed=false")
graphics_qemu_visible_boot_manual_observation=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "manual_observation_required=true")
graphics_qemu_visible_boot_bounded=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "bounded_attempt_completed=true")
graphics_qemu_visible_boot_frames=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "bounded_run_frames=3")
graphics_qemu_visible_boot_display_started=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "display_output_started=true")
graphics_qemu_visible_boot_display_stopped=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "display_output_stopped=true")
graphics_qemu_visible_boot_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "persistent_graphical_session_started=false")
graphics_qemu_visible_boot_no_desktop_shell=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "desktop_shell_started=false")
graphics_qemu_visible_boot_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "boot_graphics=false")
graphics_qemu_visible_boot_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "autostart=false")
graphics_qemu_visible_boot_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "safe_return_to_recovery=ok")
graphics_qemu_observation_marker=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "[AQUA-GATE] stage=graphics-qemu-observation-marker status=ok")
graphics_qemu_observation_marker_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "qemu_observation_marker_status=qemu-visible-observation-marker-ready")
graphics_qemu_observation_marker_source=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "source_boot_check_ready=ok")
graphics_qemu_observation_marker_path_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "qemu_visible_boot_path_ready=true")
graphics_qemu_observation_marker_not_observed=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "qemu_vm_display_observed=false")
graphics_qemu_observation_marker_status_not_observed=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "observation_status=not-observed")
graphics_qemu_observation_marker_recorded=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "manual_observation_recorded=true")
graphics_qemu_observation_marker_operator=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "operator_confirmation_required=true")
graphics_qemu_observation_marker_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "persistent_graphical_session_started=false")
graphics_qemu_observation_marker_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "boot_graphics=false")
graphics_qemu_observation_marker_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "autostart=false")
graphics_qemu_observation_marker_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "safe_return_to_recovery=ok")
qemu_visible_evidence_record=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "[AQUA-GATE] stage=qemu-visible-evidence-record status=ok")
qemu_visible_evidence_record_ready=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "evidence_record_status=qemu-vm-display-evidence-ready")
qemu_visible_evidence_record_status=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "evidence_status=operator-capture-recorded")
qemu_visible_evidence_record_capture=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "capture_file=manual-qemu-display-capture-required.png")
qemu_visible_evidence_record_manual=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "qemu_visible_manual_evidence=true")
qemu_visible_evidence_record_allows_observation=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "observation_marker_may_be_positive=true")
qemu_visible_evidence_record_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "persistent_graphical_session_started=false")
qemu_visible_evidence_record_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "boot_graphics=false")
qemu_visible_evidence_record_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "autostart=false")
qemu_visible_evidence_record_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "safe_return_to_recovery=ok")
graphics_qemu_observation_positive=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "[AQUA-GATE] stage=graphics-qemu-observation-marker status=ok")
graphics_qemu_observation_positive_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "qemu_observation_marker_status=qemu-visible-observation-marker-ready")
graphics_qemu_observation_positive_source=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "source_boot_check_ready=ok")
graphics_qemu_observation_positive_path_ready=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "qemu_visible_boot_path_ready=true")
graphics_qemu_observation_positive_observed=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "qemu_vm_display_observed=true")
graphics_qemu_observation_positive_status_observed=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "observation_status=observed")
graphics_qemu_observation_positive_evidence_required=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "evidence_required=true")
graphics_qemu_observation_positive_evidence_recorded=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "evidence_status=operator-capture-recorded")
graphics_qemu_observation_positive_recorded=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "manual_observation_recorded=true")
graphics_qemu_observation_positive_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "persistent_graphical_session_started=false")
graphics_qemu_observation_positive_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "boot_graphics=false")
graphics_qemu_observation_positive_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "autostart=false")
graphics_qemu_observation_positive_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "safe_return_to_recovery=ok")
qemu_visible_pass_report=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "[AQUA-GATE] stage=qemu-visible-pass-report status=ok")
qemu_visible_pass_report_ready=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "pass_report_status=qemu-visible-pass-report-ready")
qemu_visible_pass_report_source_attempt=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "source_attempt_result_collected=ok")
qemu_visible_pass_report_source_observation=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "source_observation_recorded=ok")
qemu_visible_pass_report_observed=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "qemu_vm_display_observed=true")
qemu_visible_pass_report_attempt_completed=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "visible_qemu_attempt_completed=true")
qemu_visible_pass_report_evidence_required=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "evidence_required=true")
qemu_visible_pass_report_evidence_recorded=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "evidence_status=operator-capture-recorded")
qemu_visible_pass_report_evidence_rule=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "positive_observation_requires_evidence=true")
qemu_visible_pass_report_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "persistent_graphical_session_started=false")
qemu_visible_pass_report_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "boot_graphics=false")
qemu_visible_pass_report_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "autostart=false")
qemu_visible_pass_report_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "safe_return_to_recovery=ok")
qemu_visible_manual_runbook=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "[AQUA-GATE] stage=qemu-visible-manual-runbook status=ok")
qemu_visible_manual_runbook_ready=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "manual_runbook_status=qemu-vm-display-manual-runbook-ready")
qemu_visible_manual_runbook_host=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "host_entrypoint=scripts/run-qemu-visible-manual.sh")
qemu_visible_manual_runbook_ready_capture_flow=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "host_ready_capture_flow_supported=true")
qemu_visible_manual_runbook_script=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "host_display_script_ready=true")
qemu_visible_manual_runbook_no_docker=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "docker_required=false")
qemu_visible_manual_runbook_manual_observation=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "manual_observation_required=true")
qemu_visible_manual_runbook_evidence_required=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "evidence_record_required=true")
qemu_visible_manual_runbook_observation_rule=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "observation_rule=run-step-13-only-after-operator-confirms-vm-display-and-records-evidence-then-run-step-14")
qemu_visible_manual_runbook_pass_report_required=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "pass_report_required=true")
qemu_visible_manual_runbook_bounded=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "bounded_attempt_required=true")
qemu_visible_manual_runbook_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "persistent_graphical_session_started=false")
qemu_visible_manual_runbook_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "boot_graphics=false")
qemu_visible_manual_runbook_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "autostart=false")
qemu_visible_manual_runbook_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "safe_return_to_recovery=ok")
qemu_visible_evidence_bundle_apply=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=ok")
qemu_visible_evidence_bundle_apply_ready=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "bundle_apply_status=qemu-visible-evidence-bundle-apply-ready")
qemu_visible_evidence_bundle_apply_waiting=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "apply_status=waiting-for-operator-confirmation")
qemu_visible_evidence_bundle_apply_not_observed=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "qemu_vm_display_observed=false")
qemu_visible_evidence_bundle_apply_no_evidence=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "evidence_record_written=false")
qemu_visible_evidence_bundle_apply_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "boot_graphics=false")
qemu_visible_evidence_bundle_apply_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "autostart=false")
qemu_visible_evidence_bundle_apply_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "safe_return_to_recovery=ok")
qemu_visible_evidence_bundle_apply_preflight_verified=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "preflight_summary_verified=true")
qemu_visible_evidence_bundle_apply_capture_hash_verified=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "capture_hash_verified=true")
qemu_visible_evidence_bundle_apply_positive=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=ok")
qemu_visible_evidence_bundle_apply_positive_applied=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "apply_status=applied-positive-observation")
qemu_visible_evidence_bundle_apply_positive_evidence=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "evidence_record_written=true")
qemu_visible_evidence_bundle_apply_positive_observed=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "qemu_vm_display_observed=true")
qemu_visible_evidence_bundle_apply_positive_no_persistent=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "persistent_graphical_session_started=false")
qemu_visible_evidence_bundle_apply_positive_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "boot_graphics=false")
qemu_visible_evidence_bundle_apply_positive_no_autostart=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "autostart=false")
qemu_visible_evidence_bundle_apply_positive_recovery_safe=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "safe_return_to_recovery=ok")
qemu_visible_evidence_bundle_apply_positive_preflight_verified=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "preflight_summary_verified=true")
qemu_visible_evidence_bundle_apply_positive_capture_hash_verified=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "capture_hash_verified=true")
qemu_visible_evidence_bundle_apply_missing_preflight_rejected=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=error")
qemu_visible_evidence_bundle_apply_missing_preflight_status=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "bundle_preflight_summary_status=failed")
qemu_visible_evidence_bundle_apply_missing_preflight_verified=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "bundle_preflight_summary_verified=failed")
qemu_visible_evidence_bundle_apply_missing_preflight_exit=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "expected_failure_exit_code=1")
qemu_visible_evidence_bundle_apply_missing_preflight_not_observed=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "operator_confirmed=false")
qemu_visible_evidence_bundle_apply_missing_preflight_unverified=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "preflight_summary_verified=false")
qemu_visible_evidence_bundle_apply_missing_capture_hash_rejected=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=error")
qemu_visible_evidence_bundle_apply_missing_capture_hash_status=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "bundle_capture_hash_verified=failed")
qemu_visible_evidence_bundle_apply_missing_capture_hash_value=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "capture_hash_verified=missing")
qemu_visible_evidence_bundle_apply_missing_capture_hash_preflight_verified=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "preflight_summary_verified=true")
qemu_visible_evidence_bundle_apply_missing_capture_hash_exit=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "expected_failure_exit_code=1")
qemu_visible_evidence_bundle_apply_missing_capture_hash_not_observed=$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "operator_confirmed=false")
client_window_model=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "[AQUA-COMPOSITOR] stage=client-window-model status=ok")
client_window_focus=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "focus_ready=ok")
client_window_move=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "move_ready=ok")
client_window_resize=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "resize_ready=ok")
client_window_close=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "close_ready=ok")
client_window_stacking=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "stacking_ready=ok")
client_window_chrome=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "chrome_ready=ok")
client_window_no_real_client=$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "real_wayland_client_started=false")
client_surface_lifecycle=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "[AQUA-COMPOSITOR] stage=client-surface-lifecycle status=ok")
client_surface_configure=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "configure_ready=ok")
client_surface_commit=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "commit_ready=ok")
client_surface_map=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "map_ready=ok")
client_surface_focus=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "focus_ready=ok")
client_surface_unmap=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "unmap_ready=ok")
client_surface_destroy=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "destroy_ready=ok")
client_surface_geometry=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "window_geometry_ready=ok")
client_surface_no_real_client=$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "real_wayland_client_started=false")
client_surface_registry=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "[AQUA-COMPOSITOR] stage=client-surface-registry status=ok")
client_surface_registry_source=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "source_window_model_ready=ok")
client_surface_registry_record=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "record_count=2")
client_surface_registry_active=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "active_client_id=wayland-client-1")
client_surface_registry_configure=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "configure_serial_ready=ok")
client_surface_registry_lifecycle=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "lifecycle_state_ready=ok")
client_surface_registry_two_client=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "two_client_ready=ok")
client_surface_registry_focus=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "focus_index_ready=ok")
client_surface_registry_stacking=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "stacking_order_ready=ok")
client_surface_registry_close=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "close_request_ready=ok")
client_surface_registry_buffer_metadata=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "buffer_metadata_ready=ok")
client_surface_registry_buffer_import_plan=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "buffer_import_plan_ready=ok")
client_surface_registry_sample_pixel=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "sample_pixel=")
client_surface_registry_sample_grid=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "sample_grid=")
client_surface_registry_buffer_snapshot=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "buffer_snapshot_bytes=")
client_surface_registry_no_render=$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "no_renderer_binding=ok")
renderer_surface_sources=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "[AQUA-COMPOSITOR] stage=renderer-surface-sources status=ok")
renderer_surface_sources_registry=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "source_registry_ready=ok")
renderer_surface_sources_count=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "surface_source_count=2")
renderer_surface_sources_active=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "active_source_ready=ok")
renderer_surface_sources_import=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "import_sources_ready=ok")
renderer_surface_sources_z_order=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "z_order_ready=ok")
renderer_surface_sources_sample_pixel=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "sample_pixel=")
renderer_surface_sources_sample_grid=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "sample_grid=")
renderer_surface_sources_buffer_snapshot=$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "buffer_snapshot_bytes=")
client_layer_pipeline=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "[AQUA-COMPOSITOR] stage=client-layer-pipeline status=ok")
client_layer_source_plan=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "source_plan_ready=ok")
client_layer_paint_plan=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "paint_plan_ready=ok")
client_layer_raster=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "raster_ready=ok")
client_layer_count=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "client_layer_count=2")
client_layer_checksum=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "client_layer_checksum=")
client_layer_sample_pixel=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "sample_pixel=")
client_layer_sample_grid=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "sample_grid=")
client_layer_buffer_snapshot=$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "buffer_snapshot_bytes=")
xdg_shell_binding=$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "[AQUA-COMPOSITOR] stage=xdg-shell-binding status=ok")
xdg_shell_protocol=$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "protocol=xdg_wm_base")
xdg_shell_handler=$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "handler_bound=ok")
xdg_shell_global=$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "global_created=ok")
xdg_shell_toplevel_callbacks=$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "toplevel_callbacks_bound=ok")
xdg_shell_popup_callbacks=$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "popup_callbacks_bound=ok")
xdg_shell_no_real_client=$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "real_wayland_client_started=false")
xdg_toplevel_client=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "[AQUA-COMPOSITOR] stage=xdg-toplevel-client status=ok")
xdg_toplevel_client_connected=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_connected=ok")
xdg_toplevel_client_inserted=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_inserted=ok")
xdg_toplevel_client_registry=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "registry_bound=ok")
xdg_toplevel_client_globals=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "xdg_wm_base_global_seen=ok")
xdg_toplevel_shm_global=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_global_seen=ok")
xdg_toplevel_shm_buffer=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_buffer_created=ok")
xdg_toplevel_client_buffer_attach=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_buffer_attached=ok")
xdg_toplevel_surface=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "surface_created=ok")
xdg_toplevel_request=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "toplevel_requested=ok")
xdg_toplevel_server_buffer_attach=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_buffer_attached=ok")
xdg_toplevel_shm_import=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_shm_buffer_imported=ok")
xdg_toplevel_shm_sample=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_shm_buffer_sampled=ok")
xdg_toplevel_shm_sample_pixel=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_sample_pixel=")
xdg_toplevel_shm_sample_grid=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_sample_grid=")
xdg_toplevel_shm_buffer_snapshot=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_buffer_snapshot_bytes=")
xdg_toplevel_server_created=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_toplevel_created=ok")
xdg_toplevel_no_boot_graphics=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "boot_graphics=false")
xdg_toplevel_configure_ack=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_configure_ack_received=ok")
xdg_toplevel_close_event=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_close_event_received=ok")
xdg_toplevel_client_count=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "test_wayland_client_count=2")
xdg_toplevel_window_model=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "[AQUA-COMPOSITOR] stage=xdg-toplevel-window-model status=ok")
xdg_toplevel_window_source=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "source_client_ready=ok")
xdg_toplevel_window_surface_bound=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "server_surface_bound=ok")
xdg_toplevel_window_model_bound=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "window_model_bound=ok")
xdg_toplevel_window_count=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "window_count=2")
xdg_toplevel_window_two_model=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "two_window_model_ready=ok")
xdg_toplevel_window_stacking=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "stacking_ready=ok")
xdg_toplevel_window_chrome=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "chrome_ready=ok")
xdg_toplevel_window_no_render=$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "renderer_started=false")
smithay_launcher_seat=$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "[AQUA-COMPOSITOR] stage=smithay-launcher-seat status=ok")
smithay_launcher_seat_global=$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "seat_global_created=true")
smithay_launcher_seat_input=$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "pointer_button_dispatched=true")
smithay_launcher_seat_rootfs=$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "host_stub=false")
scene_status=$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "scene_status=static-shell-model")
required_surfaces=$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "required_surfaces=7")
runtime_asset_bindings=$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "required_assets_present=ok")
system_surface_token_bindings=$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "required_material_tokens_present=ok")
simulated_surface_labeled=$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "simulated_surface_labeled=ok")
scene_dump=$(contract_file_contains "${CONTRACT_DIR}/scene-dump.txt" "[AQUA-COMPOSITOR] stage=scene-dump status=ok")
renderer=$(contract_file_contains "${CONTRACT_DIR}/status.txt" "renderer=aqua-renderer")
render_plan=$(contract_file_contains "${CONTRACT_DIR}/render-plan-probe.txt" "[AQUA-COMPOSITOR] stage=render-plan-probe status=ok")
draw_commands=$(contract_file_contains "${CONTRACT_DIR}/render-plan-probe.txt" "draw_command_count=7")
renderer_started=$(contract_file_contains "${CONTRACT_DIR}/render-plan-probe.txt" "renderer_started=false")
paint_plan=$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "[AQUA-COMPOSITOR] stage=paint-plan-probe status=ok")
paint_steps=$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "paint_step_count=7")
paint_order=$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "paint_order_stable=ok")
paint_surface=$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "system_surface_steps_translucent=ok")
frame_plan=$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "[AQUA-COMPOSITOR] stage=frame-plan-probe status=ok")
frame_format=$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "pixel_format=rgba8888")
frame_stride=$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "stride_ready=ok")
frame_damage=$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "damage_ready=ok")
frame_buffer=$(contract_file_contains "${CONTRACT_DIR}/frame-buffer-probe.txt" "[AQUA-COMPOSITOR] stage=frame-buffer-probe status=ok")
frame_buffer_bytes=$(contract_file_contains "${CONTRACT_DIR}/frame-buffer-probe.txt" "allocated_bytes=6291456")
frame_buffer_clear=$(contract_file_contains "${CONTRACT_DIR}/frame-buffer-probe.txt" "first_pixel=00,17,25,ff")
raster=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "[AQUA-COMPOSITOR] stage=raster-probe status=ok")
raster_rects=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "filled_rect_count=7")
raster_wallpaper_sample=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "wallpaper_sample=04,3b,5c,ff")
raster_surface_sample=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_sample=84,e0,ff,ff")
raster_surface_border_sample=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_border_sample=3d,72,8c,ff")
raster_surface_highlight_sample=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_highlight_sample=be,ef,ff,ff")
raster_surface_corner_sample=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_corner_sample=2a,6c,8c,ff")
raster_surface_shadow_sample=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_shadow_sample=52,a6,c6,ff")
surface_primitives=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_primitive_count=15")
raster_checksum=$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "raster_checksum=717b7e2c50c329f1")
raster_export=$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "[AQUA-COMPOSITOR] stage=raster-export-probe status=ok")
raster_export_format=$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "export_format=ppm-p6-rgb888")
raster_export_bytes=$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "export_bytes=4718609")
raster_export_checksum=$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "export_checksum=553f5b2626c15af1")
raster_png_export=$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "[AQUA-COMPOSITOR] stage=raster-png-export-probe status=ok")
raster_png_export_format=$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "export_format=png-rgba8888")
raster_png_export_bytes=$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "export_bytes=6293028")
raster_png_export_checksum=$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "export_checksum=1554b44a4319fe02")
session_loop=$(contract_file_contains "${CONTRACT_DIR}/session-loop.txt" "[AQUA-COMPOSITOR] stage=session-loop status=ok")
session_loop_iterations=$(contract_file_contains "${CONTRACT_DIR}/session-loop.txt" "loop_iterations=3")
session_loop_dispatch=$(contract_file_contains "${CONTRACT_DIR}/session-loop.txt" "dispatch_passes=3")
session_loop_flush=$(contract_file_contains "${CONTRACT_DIR}/session-loop.txt" "flush_passes=3")
desktop_shell=not_started

[boot_markers]
rcS_start=$(marker_status '[AQUA-BOOT] stage=rcS-start product="Aqua Linux"')
filesystems_mounted=$(marker_status '[AQUA-BOOT] stage=filesystems-mounted status=ok')
fbdev_device=$(marker_status '[AQUA-BOOT] stage=fbdev-device status=ok device=/dev/fb0 mode=')
os_release=$(marker_status '[AQUA-BOOT] stage=os-release id=aqua pretty="Aqua Linux Milestone 1"')
session_config=$(marker_status '[AQUA-BOOT] stage=session-config status=ok autostart=false boot_graphics=false recovery_tty=true')
session_runtime=$(marker_status '[AQUA-BOOT] stage=session-runtime status=ok user=aqua uid=1000 runtime_dir=/run/user/1000 control_dir=/run/aqua mode=0700')
session_env=$(marker_status '[AQUA-BOOT] stage=session-env status=ok wayland=aqua-wayland-0 xdg=/run/user/1000 assets=/usr/share/aqua')
runtime_assets_ready=$(marker_status '[AQUA-BOOT] stage=runtime-assets-ready milestone=2 status=ok')
compositor_binary=$(marker_status '[AQUA-BOOT] stage=compositor-binary status=packaged autostart=false boot_graphics=false')
compositor_status=$(marker_status '[AQUA-BOOT] stage=compositor-status status=ok mode=nested-dev')
session_bootstrap=$(marker_status '[AQUA-BOOT] stage=session-bootstrap status=ok runtime_dir=/run/user/1000 autostart=false boot_graphics=false session_started=false')
compositor_assets=$(marker_status '[AQUA-BOOT] stage=compositor-assets status=ok root=/usr/share/aqua')
output_plan=$(marker_status '[AQUA-BOOT] stage=output-plan status=ok backend=nested-dev-window boot_graphics=false renderer_started=false')
visible_preview_plan=$(marker_status '[AQUA-BOOT] stage=visible-preview-plan status=ok preview_window_started=false boot_graphics=false renderer_started=false')
scene_contract=$(marker_status '[AQUA-BOOT] stage=scene-contract status=ok surfaces=7 boot_graphics=false')
render_plan=$(marker_status '[AQUA-BOOT] stage=render-plan status=ok commands=7 boot_graphics=false renderer_started=false')
paint_plan=$(marker_status '[AQUA-BOOT] stage=paint-plan status=ok steps=7 boot_graphics=false renderer_started=false')
frame_plan=$(marker_status '[AQUA-BOOT] stage=frame-plan status=ok format=rgba8888 boot_graphics=false renderer_started=false')
frame_buffer=$(marker_status '[AQUA-BOOT] stage=frame-buffer status=ok bytes=6291456 boot_graphics=false renderer_started=false')
raster=$(marker_status '[AQUA-BOOT] stage=raster status=ok rects=7 surface_layers=15 boot_graphics=false renderer_started=false')
surface_primitives=$(marker_status '[AQUA-BOOT] stage=surface-primitives status=ok layers=15 boot_graphics=false renderer_started=false')
raster_export=$(marker_status '[AQUA-BOOT] stage=raster-export status=ok bytes=4718609 boot_graphics=false renderer_started=false')
raster_png_export=$(marker_status '[AQUA-BOOT] stage=raster-png-export status=ok bytes=6293028 boot_graphics=false renderer_started=false')
session_check=$(marker_status '[AQUA-BOOT] stage=session-check status=ok no_graphics=true')
recovery_ready=$(marker_status '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh')

[boot_summary]
boot_summary_status=$(boot_summary_status)
fbdev_device=$(boot_summary_stage_status fbdev-device)
session_config=$(boot_summary_stage_status session-config)
session_runtime=$(boot_summary_stage_status session-runtime)
session_env=$(boot_summary_stage_status session-env)
session_bootstrap=$(boot_summary_stage_status session-bootstrap)
compositor_assets=$(boot_summary_stage_status compositor-assets)
output_plan=$(boot_summary_stage_status output-plan)
visible_preview_plan=$(boot_summary_stage_status visible-preview-plan)
scene_contract=$(boot_summary_stage_status scene-contract)
render_plan=$(boot_summary_stage_status render-plan)
paint_plan=$(boot_summary_stage_status paint-plan)
frame_plan=$(boot_summary_stage_status frame-plan)
frame_buffer=$(boot_summary_stage_status frame-buffer)
raster=$(boot_summary_stage_status raster)
surface_primitives=$(boot_summary_stage_status surface-primitives)
raster_export=$(boot_summary_stage_status raster-export)
raster_png_export=$(boot_summary_stage_status raster-png-export)
session_check=$(boot_summary_stage_status session-check)
recovery_ready=$(boot_summary_stage_status recovery-ready)
EOF

echo "Aqua Linux image manifest written: ${MANIFEST}"

cat > "${MANIFEST_JSON}" <<EOF
{
  "product": "Aqua Linux",
  "base": "Buildroot",
  "dev_target": "QEMU x86_64",
  "graphics_target": "custom Wayland compositor",
  "generated_at_utc": "${GENERATED_AT_UTC}",
  "artifacts": {
    "bzImage": {
      "status": "$(status_from_file "${IMAGE_DIR}/bzImage")",
      "bytes": "$(size_or_missing "${IMAGE_DIR}/bzImage")"
    },
    "rootfs_ext2": {
      "status": "$(status_from_file "${IMAGE_DIR}/rootfs.ext2")",
      "bytes": "$(size_or_missing "${IMAGE_DIR}/rootfs.ext2")"
    },
    "disk_img": {
      "status": "$(status_from_file "${IMAGE_DIR}/disk.img")",
      "bytes": "$(size_or_missing "${IMAGE_DIR}/disk.img")"
    },
    "rootfs_tar": {
      "status": "$(status_from_file "${ROOTFS_TAR}")",
      "bytes": "$(size_or_missing "${ROOTFS_TAR}")"
    },
    "build_config": {
      "status": "$(status_from_file "${OUTPUT_DIR}/.config")"
    },
    "serial_log": {
      "status": "$(status_from_file "${SERIAL_LOG}")"
    },
    "fbdev_qemu_present_log": {
      "status": "$(status_from_file "${FBDEV_QEMU_LOG}")"
    },
    "graphical_boot_qemu_log": {
      "status": "$(status_from_file "${GRAPHICAL_BOOT_QEMU_LOG}")"
    },
    "fbdev_qemu_capture": {
      "status": "$(status_from_file "${FBDEV_QEMU_CAPTURE}")"
    },
    "fbdev_qemu_ppm": {
      "status": "$(status_from_file "${FBDEV_QEMU_PPM}")",
      "bytes": "$(size_or_missing "${FBDEV_QEMU_PPM}")"
    },
    "fbdev_qemu_png": {
      "status": "$(status_from_file "${FBDEV_QEMU_PNG}")",
      "bytes": "$(size_or_missing "${FBDEV_QEMU_PNG}")"
    },
    "fbdev_qemu_sha256": {
      "status": "$(status_from_file "${FBDEV_QEMU_SHA256}")"
    },
    "kms_qemu_capture": {
      "status": "$(status_from_file "${KMS_QEMU_CAPTURE}")"
    },
    "kms_qemu_ppm": {
      "status": "$(status_from_file "${KMS_QEMU_PPM}")",
      "bytes": "$(size_or_missing "${KMS_QEMU_PPM}")"
    },
    "kms_qemu_png": {
      "status": "$(status_from_file "${KMS_QEMU_PNG}")",
      "bytes": "$(size_or_missing "${KMS_QEMU_PNG}")"
    },
    "kms_qemu_sha256": {
      "status": "$(status_from_file "${KMS_QEMU_SHA256}")"
    },
    "page_flip_qemu_capture": {
      "status": "$(status_from_file "${PAGE_FLIP_QEMU_CAPTURE}")"
    },
    "page_flip_qemu_ppm": {
      "status": "$(status_from_file "${PAGE_FLIP_QEMU_PPM}")",
      "bytes": "$(size_or_missing "${PAGE_FLIP_QEMU_PPM}")"
    },
    "page_flip_qemu_png": {
      "status": "$(status_from_file "${PAGE_FLIP_QEMU_PNG}")",
      "bytes": "$(size_or_missing "${PAGE_FLIP_QEMU_PNG}")"
    },
    "page_flip_qemu_sha256": {
      "status": "$(status_from_file "${PAGE_FLIP_QEMU_SHA256}")"
    },
    "frame_loop_qemu_capture": {
      "status": "$(status_from_file "${FRAME_LOOP_QEMU_CAPTURE}")"
    },
    "frame_loop_qemu_ppm": {
      "status": "$(status_from_file "${FRAME_LOOP_QEMU_PPM}")",
      "bytes": "$(size_or_missing "${FRAME_LOOP_QEMU_PPM}")"
    },
    "frame_loop_qemu_png": {
      "status": "$(status_from_file "${FRAME_LOOP_QEMU_PNG}")",
      "bytes": "$(size_or_missing "${FRAME_LOOP_QEMU_PNG}")"
    },
    "frame_loop_qemu_sha256": {
      "status": "$(status_from_file "${FRAME_LOOP_QEMU_SHA256}")"
    },
    "session_loop_qemu_capture": {
      "status": "$(status_from_file "${SESSION_LOOP_QEMU_CAPTURE}")"
    },
    "session_loop_qemu_ppm": {
      "status": "$(status_from_file "${SESSION_LOOP_QEMU_PPM}")",
      "bytes": "$(size_or_missing "${SESSION_LOOP_QEMU_PPM}")"
    },
    "session_loop_qemu_png": {
      "status": "$(status_from_file "${SESSION_LOOP_QEMU_PNG}")",
      "bytes": "$(size_or_missing "${SESSION_LOOP_QEMU_PNG}")"
    },
    "session_loop_qemu_sha256": {
      "status": "$(status_from_file "${SESSION_LOOP_QEMU_SHA256}")"
    },
    "wayland_session_qemu_capture": {
      "status": "$(status_from_file "${WAYLAND_SESSION_QEMU_CAPTURE}")"
    },
    "wayland_session_qemu_ppm": {
      "status": "$(status_from_file "${WAYLAND_SESSION_QEMU_PPM}")",
      "bytes": "$(size_or_missing "${WAYLAND_SESSION_QEMU_PPM}")"
    },
    "wayland_session_qemu_png": {
      "status": "$(status_from_file "${WAYLAND_SESSION_QEMU_PNG}")",
      "bytes": "$(size_or_missing "${WAYLAND_SESSION_QEMU_PNG}")"
    },
    "wayland_session_qemu_sha256": {
      "status": "$(status_from_file "${WAYLAND_SESSION_QEMU_SHA256}")"
    },
    "drm_device_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/drm-device-probe.txt")"
    },
    "boot_summary": {
      "status": "$(status_from_file "${BOOT_SUMMARY}")"
    },
    "boot_summary_json": {
      "status": "$(status_from_file "${BOOT_SUMMARY_JSON}")"
    },
    "session_check_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/session-check.txt")"
    },
    "manual_launch_plan": {
      "status": "$(status_from_file "${CONTRACT_DIR}/manual-launch-plan.txt")"
    },
    "guarded_run": {
      "status": "$(status_from_file "${CONTRACT_DIR}/guarded-run.txt")"
    },
    "graphical_session_supervisor": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphical-session-supervisor.txt")"
    },
    "media_service_supervisor": {
      "status": "$(status_from_file "${CONTRACT_DIR}/media-service-supervisor.txt")"
    },
    "graphical_session_boot": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphical-session-boot.txt")"
    },
    "handoff_gate": {
      "status": "$(status_from_file "${CONTRACT_DIR}/handoff-gate.txt")"
    },
    "output_plan_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/output-plan-probe.txt")"
    },
    "display_output_handoff_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/display-output-handoff-probe.txt")"
    },
    "display_activation_plan_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/display-activation-plan-probe.txt")"
    },
    "display_output_smoke": {
      "status": "$(status_from_file "${CONTRACT_DIR}/display-output-smoke.txt")"
    },
    "nested_output_surface": {
      "status": "$(status_from_file "${CONTRACT_DIR}/nested-output-surface.txt")"
    },
    "visible_preview_plan_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/visible-preview-plan-probe.txt")"
    },
    "visible_preview_export_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/visible-preview-export-probe.txt")"
    },
    "visible_preview_export": {
      "status": "$(status_from_file "${CONTRACT_DIR}/aqua-visible-preview.html")",
      "bytes": "$(size_or_missing "${CONTRACT_DIR}/aqua-visible-preview.html")"
    },
    "nested_preview_loop": {
      "status": "$(status_from_file "${CONTRACT_DIR}/nested-preview-loop.txt")"
    },
    "manual_nested_preview_backend": {
      "status": "$(status_from_file "${CONTRACT_DIR}/manual-nested-preview-backend.txt")"
    },
    "manual_nested_preview_execution": {
      "status": "$(status_from_file "${CONTRACT_DIR}/manual-nested-preview-execution.txt")"
    },
    "manual_nested_preview_execution_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/manual-nested-preview-execution-probe.txt")"
    },
    "visible_preview_request": {
      "status": "$(status_from_file "${CONTRACT_DIR}/visible-preview-request.txt")"
    },
    "visible_preview_launch": {
      "status": "$(status_from_file "${CONTRACT_DIR}/visible-preview-launch.txt")"
    },
    "recovery_help": {
      "status": "$(status_from_file "${CONTRACT_DIR}/recovery-help.txt")"
    },
    "operator_transcript": {
      "status": "$(status_from_file "${CONTRACT_DIR}/operator-transcript.txt")"
    },
    "graphics_enable_gate": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-enable-gate.txt")"
    },
    "graphics_enable_gate_positive": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-enable-gate-positive.txt")"
    },
    "graphics_launch_candidate": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-launch-candidate.txt")"
    },
    "graphics_rollback_drill": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-rollback-drill.txt")"
    },
    "graphics_startup_preflight": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-startup-preflight.txt")"
    },
    "graphics_startup_rehearsal": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-startup-rehearsal.txt")"
    },
    "graphics_qemu_display_gate": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-qemu-display-gate.txt")"
    },
    "graphics_visible_qemu_attempt": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt")"
    },
    "graphics_visible_attempt_transcript": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt")"
    },
    "graphics_visible_attempt_result": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-visible-attempt-result.txt")"
    },
    "graphics_visible_attempt_runner": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt")"
    },
    "graphics_qemu_visible_boot_check": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt")"
    },
    "graphics_fbdev_present": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-fbdev-present.txt")"
    },
    "graphics_qemu_observation_marker": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt")"
    },
    "qemu_visible_evidence_record": {
      "status": "$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-record.txt")"
    },
    "graphics_qemu_observation_positive": {
      "status": "$(status_from_file "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt")"
    },
    "qemu_visible_pass_report": {
      "status": "$(status_from_file "${CONTRACT_DIR}/qemu-visible-pass-report.txt")"
    },
    "qemu_visible_manual_runbook": {
      "status": "$(status_from_file "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt")"
    },
    "qemu_visible_operator_pass": {
      "status": "$(status_from_file "${QEMU_VISIBLE_OPERATOR_PASS}")"
    },
    "qemu_visible_operator_pass_json": {
      "status": "$(status_from_file "${QEMU_VISIBLE_OPERATOR_PASS_JSON}")"
    },
    "qemu_visible_evidence_bundle_apply": {
      "status": "$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt")"
    },
    "qemu_visible_evidence_bundle_apply_positive": {
      "status": "$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt")"
    },
    "qemu_visible_evidence_bundle_apply_missing_preflight": {
      "status": "$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt")"
    },
    "qemu_visible_evidence_bundle_apply_missing_capture_hash": {
      "status": "$(status_from_file "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt")"
    },
    "client_window_model_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/client-window-model-probe.txt")"
    },
    "client_surface_lifecycle_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt")"
    },
    "client_surface_registry_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/client-surface-registry-probe.txt")"
    },
    "renderer_surface_sources_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/renderer-surface-sources-probe.txt")"
    },
    "client_layer_pipeline_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/client-layer-pipeline-probe.txt")"
    },
    "xdg_shell_binding_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/xdg-shell-binding-probe.txt")"
    },
    "xdg_toplevel_client_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt")"
    },
    "xdg_toplevel_window_model_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt")"
    },
    "smithay_launcher_seat_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt")"
    },
    "paint_plan_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/paint-plan-probe.txt")"
    },
    "frame_plan_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/frame-plan-probe.txt")"
    },
    "frame_buffer_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/frame-buffer-probe.txt")"
    },
    "raster_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/raster-probe.txt")"
    },
    "raster_export_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/raster-export-probe.txt")"
    },
    "raster_export": {
      "status": "$(status_from_file "${CONTRACT_DIR}/aqua-raster.ppm")",
      "bytes": "$(size_or_missing "${CONTRACT_DIR}/aqua-raster.ppm")"
    },
    "raster_png_export_probe": {
      "status": "$(status_from_file "${CONTRACT_DIR}/raster-png-export-probe.txt")"
    },
    "raster_png_export": {
      "status": "$(status_from_file "${CONTRACT_DIR}/aqua-raster.png")",
      "bytes": "$(size_or_missing "${CONTRACT_DIR}/aqua-raster.png")"
    }
  },
  "rootfs": {
    "session_config": "$(rootfs_entry_status ./etc/aqua/compositor-session.conf)",
    "session_config_recovery_safe": "$(rootfs_session_config_status)",
    "session_env": "$(rootfs_entry_status ./etc/aqua/session.env)",
    "session_env_recovery_safe": "$(rootfs_session_env_status)",
    "runtime_assets": "$(rootfs_entry_status ./usr/share/aqua/tokens/design-tokens.json)",
    "design_tokens_product": "$(rootfs_text_contains ./usr/share/aqua/tokens/design-tokens.json '"product": "Aqua Linux"')",
    "design_tokens_scene_materials": "$(rootfs_text_contains ./usr/share/aqua/tokens/design-tokens.json '"blurRequired"')",
    "compositor_binary": "$(rootfs_entry_status ./usr/bin/aqua-compositor)",
    "compositor_packaged": "$(compositor_packaged_status)",
    "autostart": false,
    "boot_graphics": false
  },
  "scene_contract": {
    "scene_model": "$(contract_file_contains "${CONTRACT_DIR}/status.txt" "scene_model=aqua-scene")",
    "graphics_drm_rootfs_probe": "$(contract_file_contains "${CONTRACT_DIR}/drm-device-probe.txt" "[AQUA-COMPOSITOR] stage=drm-device-probe status=ok")",
    "graphics_drm_qemu_probe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-device-probe status=ok")",
    "graphics_drm_qemu_device": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "device=/dev/dri/card0")",
    "graphics_drm_qemu_connector": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "connector.Virtual-1.status=connected")",
    "graphics_drm_qemu_mode": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "connector.Virtual-1.first_mode=1280x800")",
    "graphics_drm_read_only": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "device_open_mode=read-only")",
    "graphics_drm_no_master": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_master_acquired=false")",
    "graphics_drm_no_modeset": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_activated=false")",
    "graphics_drm_dumb_buffer": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-dumb-buffer-probe status=ok")",
    "graphics_drm_dumb_buffer_mode": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "selected_mode=1280x800")",
    "graphics_drm_dumb_buffer_pitch": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "buffer_pitch=5120")",
    "graphics_drm_dumb_buffer_bytes": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "buffer_bytes=4096000")",
    "graphics_drm_dumb_buffer_checksum": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "buffer_checksum=c85dbfbfc17843af")",
    "graphics_drm_dumb_buffer_mapped": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "dumb_buffer_mapped=true")",
    "graphics_drm_dumb_buffer_destroyed": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "dumb_buffer_destroyed=true")",
    "graphics_drm_dumb_buffer_no_framebuffer": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "framebuffer_created=false")",
    "graphics_drm_dumb_buffer_no_page_flip": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_submitted=false")",
    "graphics_drm_dumb_buffer_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")",
    "graphics_drm_gbm_scanout_buffer": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gbm-scanout-buffer-probe status=ok")",
    "graphics_drm_gbm_scanout_usage": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gbm_usage_rendering=true")",
    "graphics_drm_gbm_scanout_pitch": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gbm_front_pitch=5120")",
    "graphics_drm_gbm_dmabuf_export": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "dmabuf_exported=true")",
    "graphics_drm_gbm_addfb2": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_addfb2_back=true")",
    "graphics_drm_gbm_framebuffer_cleanup": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_framebuffers_destroyed=true")",
    "graphics_drm_gbm_direct_scanout": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gbm-scanout status=ok")",
    "graphics_drm_gbm_direct_rendered": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_gbm_scanout_back_bound=true")",
    "graphics_drm_gbm_direct_no_cpu_copy": "$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "scanout_cpu_copy=false")",
    "graphics_drm_gbm_direct_page_flip": "$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "page_flip_event_received=true")",
    "graphics_drm_gbm_direct_crtc_restored": "$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "crtc_restored=true")",
    "graphics_drm_gbm_direct_capture": "$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "status=ok")",
    "graphics_drm_gbm_direct_capture_dimensions": "$(contract_file_contains "${GBM_SCANOUT_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_drm_gbm_direct_ppm_checksum": "$(capture_checksum_status "${GBM_SCANOUT_QEMU_PPM}" "ppm_sha256" "${GBM_SCANOUT_QEMU_CAPTURE}")",
    "graphics_drm_gbm_direct_png_checksum": "$(capture_checksum_status "${GBM_SCANOUT_QEMU_PNG}" "png_sha256" "${GBM_SCANOUT_QEMU_CAPTURE}")",
    "graphics_drm_kms_present": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-kms-present status=ok")",
    "graphics_drm_kms_active": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-kms-present status=active")",
    "graphics_drm_kms_framebuffer": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "framebuffer_created=true")",
    "graphics_drm_kms_activated": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "kms_activated=true")",
    "graphics_drm_kms_no_page_flip": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_submitted=false")",
    "graphics_drm_kms_crtc_restored": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "crtc_restored=true")",
    "graphics_drm_kms_framebuffer_destroyed": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "framebuffer_destroyed=true")",
    "graphics_drm_kms_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "display_output_stopped=true")",
    "graphics_drm_kms_capture": "$(contract_file_contains "${KMS_QEMU_CAPTURE}" "status=ok")",
    "graphics_drm_kms_capture_dimensions": "$(contract_file_contains "${KMS_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_drm_kms_ppm_checksum": "$(capture_checksum_status "${KMS_QEMU_PPM}" "ppm_sha256" "${KMS_QEMU_CAPTURE}")",
    "graphics_drm_kms_png_checksum": "$(capture_checksum_status "${KMS_QEMU_PNG}" "png_sha256" "${KMS_QEMU_CAPTURE}")",
    "graphics_drm_gpu_surface": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gpu-surface status=ok")",
    "graphics_drm_gpu_surface_active": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-gpu-surface status=active")",
    "graphics_drm_gpu_surface_composition": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_backend=smithay-gles2-gbm")",
    "graphics_drm_gpu_surface_shader": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_shader=aqua-surface-compositor-v1")",
    "graphics_drm_gpu_surface_blur": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_blur_passes=2")",
    "graphics_drm_gpu_surface_page_flip": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_event_received=true")",
    "graphics_drm_gpu_surface_bridge": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "scanout_bridge=cpu-readback-copy")",
    "graphics_drm_gpu_surface_no_direct_scanout": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "direct_dmabuf_scanout=false")",
    "graphics_drm_gpu_surface_client_source": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_client_texture_source=sampled-wl-shm-contract")",
    "graphics_drm_gpu_surface_client_count": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_client_texture_count=2")",
    "graphics_drm_gpu_surface_client_composited": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_client_textures_composited=true")",
    "graphics_drm_gpu_surface_client_not_live": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "composition_live_wayland_session=false")",
    "graphics_drm_gpu_surface_capture": "$(contract_file_contains "${GPU_SURFACE_QEMU_CAPTURE}" "status=ok")",
    "graphics_drm_gpu_surface_capture_dimensions": "$(contract_file_contains "${GPU_SURFACE_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_drm_gpu_surface_ppm_checksum": "$(capture_checksum_status "${GPU_SURFACE_QEMU_PPM}" "ppm_sha256" "${GPU_SURFACE_QEMU_CAPTURE}")",
    "graphics_drm_gpu_surface_png_checksum": "$(capture_checksum_status "${GPU_SURFACE_QEMU_PNG}" "png_sha256" "${GPU_SURFACE_QEMU_CAPTURE}")",
    "graphics_drm_gpu_surface_crtc_restored": "$(contract_file_contains "${GPU_SURFACE_QEMU_CAPTURE}" "crtc_restored=true")",
    "graphics_drm_page_flip": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-page-flip status=ok")",
    "graphics_drm_page_flip_active": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-page-flip status=active")",
    "graphics_drm_page_flip_submitted": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_submitted=true")",
    "graphics_drm_page_flip_event": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_event_received=true")",
    "graphics_drm_page_flip_front_destroyed": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "front_framebuffer_destroyed=true")",
    "graphics_drm_page_flip_back_destroyed": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "back_framebuffer_destroyed=true")",
    "graphics_drm_page_flip_crtc_restored": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "crtc_restored=true")",
    "graphics_drm_page_flip_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")",
    "graphics_drm_page_flip_capture": "$(contract_file_contains "${PAGE_FLIP_QEMU_CAPTURE}" "status=ok")",
    "graphics_drm_page_flip_capture_dimensions": "$(contract_file_contains "${PAGE_FLIP_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_drm_page_flip_ppm_checksum": "$(capture_checksum_status "${PAGE_FLIP_QEMU_PPM}" "ppm_sha256" "${PAGE_FLIP_QEMU_CAPTURE}")",
    "graphics_drm_page_flip_png_checksum": "$(capture_checksum_status "${PAGE_FLIP_QEMU_PNG}" "png_sha256" "${PAGE_FLIP_QEMU_CAPTURE}")",
    "graphics_drm_frame_loop": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-frame-loop status=ok")",
    "graphics_drm_frame_loop_active": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-frame-loop status=active")",
    "graphics_drm_frame_loop_submitted": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "submitted_page_flips=3")",
    "graphics_drm_frame_loop_received": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "received_page_flip_events=3")",
    "graphics_drm_frame_loop_order": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "page_flip_event_order_complete=true")",
    "graphics_drm_frame_loop_alternation": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "front_back_buffer_alternation=true")",
    "graphics_drm_frame_loop_crtc_restored": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "crtc_restored=true")",
    "graphics_drm_frame_loop_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")",
    "graphics_drm_frame_loop_capture": "$(contract_file_contains "${FRAME_LOOP_QEMU_CAPTURE}" "status=ok")",
    "graphics_drm_frame_loop_capture_dimensions": "$(contract_file_contains "${FRAME_LOOP_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_drm_frame_loop_ppm_checksum": "$(capture_checksum_status "${FRAME_LOOP_QEMU_PPM}" "ppm_sha256" "${FRAME_LOOP_QEMU_CAPTURE}")",
    "graphics_drm_frame_loop_png_checksum": "$(capture_checksum_status "${FRAME_LOOP_QEMU_PNG}" "png_sha256" "${FRAME_LOOP_QEMU_CAPTURE}")",
    "graphics_drm_session_loop": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-session-loop status=ok")",
    "graphics_drm_session_loop_active": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-session-loop status=active")",
    "graphics_drm_session_loop_owner": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "session_owner=aqua-compositor")",
    "graphics_drm_session_loop_calloop": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "event_loop=calloop")",
    "graphics_drm_session_loop_source_owned": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_event_source_owned=true")",
    "graphics_drm_session_loop_dispatch": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "calloop_dispatch_passes=3")",
    "graphics_drm_session_loop_source_released": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_event_source_released=true")",
    "graphics_drm_session_loop_wayland_stopped": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_display_started=false")",
    "graphics_drm_session_loop_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")",
    "graphics_drm_session_loop_capture": "$(contract_file_contains "${SESSION_LOOP_QEMU_CAPTURE}" "status=ok")",
    "graphics_drm_session_loop_capture_dimensions": "$(contract_file_contains "${SESSION_LOOP_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_drm_session_loop_ppm_checksum": "$(capture_checksum_status "${SESSION_LOOP_QEMU_PPM}" "ppm_sha256" "${SESSION_LOOP_QEMU_CAPTURE}")",
    "graphics_drm_session_loop_png_checksum": "$(capture_checksum_status "${SESSION_LOOP_QEMU_PNG}" "png_sha256" "${SESSION_LOOP_QEMU_CAPTURE}")",
    "graphics_drm_wayland_session": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok")",
    "graphics_drm_wayland_session_active": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=drm-wayland-session status=active")",
    "graphics_drm_wayland_gpu_composition": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_composition_backend=smithay-gles2-readback-dumb-buffer")",
    "graphics_drm_wayland_gpu_render_node": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_render_device=/dev/dri/card0")",
    "graphics_drm_wayland_gpu_same_kms_node": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_render_node_separate=false")",
    "graphics_drm_wayland_virtio_scanout_compat": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_direct_dmabuf_scanout=false")",
    "graphics_drm_wayland_cpu_scanout_copy": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_scanout_cpu_copy=true")",
    "graphics_drm_wayland_gbm_cleanup": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gbm_scanout_buffers_released=true")",
    "graphics_drm_wayland_frame_readback": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_frame_readback=true")",
    "graphics_drm_wayland_frame_checksum": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_checksum_source=frame-readback")",
    "graphics_drm_wayland_gpu_live_source": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_texture_source=live-smithay-wl-shm-snapshot")",
    "graphics_drm_wayland_gpu_live_count": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_texture_count=2")",
    "graphics_drm_wayland_gpu_live_bytes": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_texture_bytes=643216")",
    "graphics_drm_wayland_gpu_live_uploaded": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_textures_uploaded=true")",
    "graphics_drm_wayland_gpu_live_composited": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_client_textures_composited=true")",
    "graphics_drm_wayland_gpu_live_session": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_live_session=true")",
    "graphics_drm_wayland_gpu_initial_frame": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_initial_frame_checksum=")",
    "graphics_drm_wayland_gpu_session_context": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_context_lifecycle=session-owned")",
    "graphics_drm_wayland_gpu_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_updates=true")",
    "graphics_drm_wayland_gpu_context_reused": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_context_reused=true")",
    "graphics_drm_wayland_gpu_repaint_source_order": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_source_order_changed=true")",
    "graphics_drm_wayland_gpu_repaint_count": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_texture_count=2")",
    "graphics_drm_wayland_gpu_repaint_bytes": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_texture_bytes=643216")",
    "graphics_drm_wayland_gpu_repaint_checksum": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_checksum=")",
    "graphics_drm_wayland_gpu_files_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_gpu_repaint=true")",
    "graphics_drm_wayland_gpu_settings_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_gpu_repaint=true")",
    "graphics_drm_wayland_gpu_cleanup_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_gpu_repaint=true")",
    "graphics_drm_wayland_gpu_close_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_gpu_repaint=true")",
    "graphics_drm_wayland_gpu_full_repaint_route": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_gpu_repaint_route_complete=true")",
    "graphics_drm_wayland_shared_lifecycle": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "shared_session_lifecycle=true")",
    "graphics_drm_wayland_display": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_display_created=true")",
    "graphics_drm_wayland_socket": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_socket_bound=true")",
    "graphics_drm_wayland_client": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_client_inserted=true")",
    "graphics_drm_wayland_dispatch": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_dispatch_passes=3")",
    "graphics_drm_wayland_flush": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_flush_passes=3")",
    "graphics_drm_wayland_drm_dispatch": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "calloop_drm_dispatch_passes=3")",
    "graphics_drm_wayland_smithay_globals": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "smithay_protocol_globals_started=true")",
    "graphics_drm_wayland_compositor_global": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "compositor_global_started=true")",
    "graphics_drm_wayland_shm_global": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "shm_global_started=true")",
    "graphics_drm_wayland_xdg_shell_global": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "xdg_shell_global_started=true")",
    "graphics_drm_wayland_seat": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "seat_started=true")",
    "graphics_drm_wayland_socket_cleaned": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wayland_socket_cleaned=true")",
    "graphics_drm_wayland_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")",
    "graphics_drm_wayland_input_source": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "input_source=libinput-udev")",
    "graphics_drm_wayland_input_discovery": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_discovery_ready=true")",
    "graphics_drm_wayland_input_keyboard": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_dispatch_ready=true")",
    "graphics_drm_wayland_input_selective_forward": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "external_client_keyboard_event_received=true")",
    "graphics_drm_wayland_input_pointer_hit_test": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "external_client_pointer_event_received=true")",
    "graphics_drm_wayland_input_pointer_motion": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_pointer_motion_events=11")",
    "graphics_drm_wayland_input_pointer_button": "$(contract_file_numeric_at_least "${FBDEV_QEMU_LOG}" "drm_wayland_input_pointer_button_events=" 18)",
    "graphics_drm_wayland_input_launcher": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_launcher_visible=true")",
    "graphics_drm_wayland_launcher_overlay": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_overlay_rendered=true")",
    "graphics_drm_wayland_launcher_pointer": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_launch_request_app=files")",
    "graphics_drm_wayland_launcher_preflight": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_launch_rejection_reason=accepted")",
    "graphics_drm_wayland_launcher_process": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_process_reaped=true")",
    "graphics_drm_wayland_process_supervisor": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_process_supervisor_duplicate_rejected=true")",
    "graphics_session_supervisor_qemu": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-TEST] stage=graphical-session-supervisor-qemu status=ok")",
    "graphics_session_supervisor_qemu_failures": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "real_compositor_failures=3")",
    "graphics_session_supervisor_qemu_restarts": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "bounded_restarts=2")",
    "graphics_session_supervisor_qemu_recovery": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "recovery_return=ok")",
    "graphics_boot_qemu": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-boot-qemu status=ok")",
    "graphics_boot_qemu_activation": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "activation=supervised")",
    "graphics_boot_qemu_wayland": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "drm_wayland=active")",
    "graphics_boot_qemu_persistent": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "persistent=true")",
    "graphics_boot_qemu_desktop_event_loop": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "scenario=desktop-event-loop fixtures=false recovery_tty=available")",
    "graphics_boot_qemu_fixture_free": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "persistent=true scenario=desktop-event-loop fixtures=false")",
    "graphics_desktop_runtime_launch_qemu": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-launch-qemu status=ok")",
    "graphics_desktop_runtime_launch_files": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "app=files surface=aqua.files")",
    "graphics_desktop_runtime_launch_repaint": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "repaint=true supervised=true")",
    "graphics_desktop_runtime_settings": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-settings-qemu status=ok app=settings surface=aqua.settings clients=2 launcher_closed=true")",
    "audio_adapter_qemu_safe_default": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=audio-adapter-qemu status=ok health=unavailable controls=false backend_applied=false packages=absent")",
    "graphics_desktop_live_theme": "$(contract_file_contains "${LIVE_THEME_QEMU_LOG}" "[AQUA-TEST] stage=desktop-live-theme-qemu status=ok from=Light to=Dark shell=true apps=files,settings restart=false frame_delta=true")",
    "graphics_desktop_runtime_damage": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-damage-qemu status=ok app=settings interaction=keyboard-category-selected repaint=incremented revision=changed")",
    "graphics_desktop_runtime_close": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-close-qemu status=ok app=settings close=alt-f4 exit=clean stale_surface=removed restart=never clients=1")",
    "graphics_desktop_runtime_unexpected_exit": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-unexpected-exit-qemu status=ok app=files exit=forced stale_surface=removed restart=never active_count=0 clients=0")",
    "graphics_desktop_runtime_cleanup": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-runtime-cleanup-qemu status=ok apps=files,settings lifecycle_clean=true active_count=0 stale_surfaces=0")",
    "graphics_desktop_session_menu": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=desktop-session-menu-qemu status=ok actions=logout,restart,shutdown,recovery confirmation=true selected=recovery execution=return-to-recovery")",
    "graphics_desktop_session_menu_overlay": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "desktop_session_menu_overlay_texture_ready=true")",
    "graphics_boot_qemu_recovery_tty": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "recovery_tty=available")",
    "graphics_stop_qemu": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-stop-qemu status=ok")",
    "graphics_stop_qemu_clients": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "clients_stopped=true")",
    "graphics_stop_qemu_kms": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "kms_restored=true")",
    "graphics_stop_qemu_gbm": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "gbm_released=true")",
    "graphics_stop_qemu_pid": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "pid_cleaned=true")",
    "graphics_stop_qemu_recovery": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "recovery_return=ok")",
    "graphics_restart_qemu": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-restart-qemu status=ok")",
    "graphics_session_cycle_qemu": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "[AQUA-TEST] stage=graphical-session-cycle-qemu status=ok")",
    "graphics_session_cycle_qemu_sockets": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "sockets_clean=true")",
    "graphics_session_cycle_qemu_pids": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "pids_clean=true")",
    "graphics_session_cycle_qemu_clients": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "clients_clean=true")",
    "graphics_session_cycle_qemu_drm": "$(contract_file_contains "${GRAPHICAL_BOOT_QEMU_LOG}" "drm_clean=true")",
    "graphics_drm_wayland_settings": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_process_reaped=true")",
    "graphics_drm_wayland_settings_interaction": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_keyboard_category=Desktop")",
    "graphics_drm_wayland_settings_persistence": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_reload_verified=true")",
    "graphics_drm_wayland_settings_desktop": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_persisted_desktop_icons=false")",
    "graphics_drm_wayland_settings_input": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_persisted_key_repeat=false")",
    "graphics_drm_wayland_settings_network": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_settings_network_management=false")",
    "graphics_drm_wayland_real_font": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "aqua_settings_font_ready=true")",
    "graphics_drm_wayland_launcher_surface_owner": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_launcher_surface_owned=true")",
    "graphics_drm_wayland_files_window": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_window_repaint_complete=true")",
    "graphics_drm_wayland_files_read_only": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_directory_enumerated=true")",
    "graphics_drm_wayland_files_pointer": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_selection_commit=true")",
    "graphics_drm_wayland_files_navigation": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_sidebar_navigation=Pictures")",
    "graphics_drm_wayland_files_keyboard": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_keyboard_activation=Projects")",
    "graphics_drm_wayland_files_hover": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_hover_feedback=true")",
    "graphics_drm_wayland_files_scroll": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_scroll_offset=1")",
    "graphics_drm_wayland_files_safe_preview": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_arbitrary_execution=false")",
    "graphics_drm_wayland_files_wheel": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_pointer_wheel=true")",
    "graphics_drm_wayland_files_page_keys": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_page_up=true")",
    "graphics_drm_wayland_files_edge_keys": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_end_key=true")",
    "graphics_drm_wayland_files_scrollbar_drag": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_scrollbar_drag=true")",
    "graphics_drm_wayland_files_preview_scroll": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_files_preview_scroll_offset=1")",
    "graphics_drm_wayland_input_dispatch": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_input_dispatch_ready=true")",
    "graphics_drm_wayland_external_client": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_ready=true")",
    "graphics_drm_wayland_external_client_buffer": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_buffer_bytes=")",
    "graphics_drm_wayland_third_party_client": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "third_party_wayland_client=weston-simple-shm")",
    "graphics_drm_wayland_no_weston_compositor": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "weston_compositor_started=false")",
    "graphics_drm_wayland_external_client_multi_surface": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_surface_count=2")",
    "graphics_drm_wayland_external_client_independent_buffers": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_independent_buffers=true")",
    "graphics_drm_wayland_external_client_composited": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_composited=true")",
    "graphics_drm_wayland_external_client_frame_callback": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "external_client_frame_callback_received=true")",
    "graphics_drm_wayland_external_client_damage": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_damage_ready=true")",
    "graphics_drm_wayland_external_client_focus": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_keyboard_focus=true")",
    "graphics_drm_wayland_external_client_focus_change": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_focus_changes=8")",
    "graphics_drm_wayland_external_client_stacking": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_external_client_stacking_changes=8")",
    "graphics_drm_wayland_stacking_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_stacking_repaint_complete=true")",
    "graphics_drm_wayland_stacking_repaint_changed": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_stacking_repaint_changed_frame=true")",
    "graphics_drm_wayland_client_cleanup": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_complete=true")",
    "graphics_drm_wayland_client_cleanup_focus": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_keyboard_focus_reassigned=true")",
    "graphics_drm_wayland_client_cleanup_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_client_cleanup_repaint_complete=true")",
    "graphics_drm_wayland_interactive_geometry": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_interactive_geometry_applied=true")",
    "graphics_drm_wayland_state_cycle": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_state_cycle_complete=true")",
    "graphics_drm_wayland_state_configure": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_state_configure_acks=9")",
    "graphics_drm_wayland_close": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_request_sent=true")",
    "graphics_drm_wayland_close_cleanup": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_cleanup_surfaces=0")",
    "graphics_drm_wayland_close_repaint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "drm_wayland_close_repaint_complete=true")",
    "graphics_evdev_aqua_seat": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-INPUT] stage=evdev-aqua-seat status=ok")",
    "graphics_evdev_keyboard": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "keyboard_events=1")",
    "graphics_evdev_pointer_motion": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "pointer_motion_events=2")",
    "graphics_evdev_pointer_button": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "pointer_button_events=2")",
    "graphics_evdev_launcher": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "launcher_visible=true")",
    "graphics_drm_wayland_capture": "$(contract_file_contains "${WAYLAND_SESSION_QEMU_CAPTURE}" "status=ok")",
    "graphics_drm_wayland_capture_dimensions": "$(contract_file_contains "${WAYLAND_SESSION_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_drm_wayland_ppm_checksum": "$(capture_checksum_status "${WAYLAND_SESSION_QEMU_PPM}" "ppm_sha256" "${WAYLAND_SESSION_QEMU_CAPTURE}")",
    "graphics_drm_wayland_png_checksum": "$(capture_checksum_status "${WAYLAND_SESSION_QEMU_PNG}" "png_sha256" "${WAYLAND_SESSION_QEMU_CAPTURE}")",
    "session_config_probe": "$(contract_file_contains "${CONTRACT_DIR}/session-config.txt" "[AQUA-COMPOSITOR] stage=session-config status=ok")",
    "session_env_probe": "$(contract_file_contains "${CONTRACT_DIR}/session-env.txt" "[AQUA-COMPOSITOR] stage=session-env status=ok")",
    "session_bootstrap_probe": "$(contract_file_contains "${CONTRACT_DIR}/session-bootstrap.txt" "[AQUA-COMPOSITOR] stage=session-bootstrap status=ok")",
    "session_check_probe": "$(contract_file_contains "${CONTRACT_DIR}/session-check.txt" "[AQUA-SESSION] stage=session-check status=ok no_graphics=true")",
    "manual_launch_plan": "$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "[AQUA-MANUAL] stage=compositor-launch-plan status=ok")",
    "manual_launch_safe": "$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "safe_to_run_from_recovery=ok")",
    "manual_launch_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "autostart=false")",
    "manual_launch_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "boot_graphics=false")",
    "manual_launch_recovery_tty": "$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "fallback_tty_required=true")",
    "manual_launch_no_display_start": "$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "starts_display_output=false")",
    "manual_launch_no_shell_start": "$(contract_file_contains "${CONTRACT_DIR}/manual-launch-plan.txt" "starts_desktop_shell=false")",
    "guarded_run": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "[AQUA-GUARDED] stage=compositor-bounded-run status=ok")",
    "guarded_run_complete": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "run_status=qemu-safe-guarded-compositor-run-complete")",
    "guarded_run_launch_plan": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "launch_plan_ready=ok")",
    "guarded_run_bounded": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "bounded_run_complete=ok")",
    "guarded_run_frames": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "bounded_run_frames=3")",
    "guarded_run_started": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "display_output_started=true")",
    "guarded_run_stopped": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "display_output_stopped=true")",
    "guarded_run_fallback_tty": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "fallback_tty_available=true")",
    "guarded_run_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "autostart=false")",
    "guarded_run_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "boot_graphics=false")",
    "guarded_run_no_shell": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "desktop_shell_started=false")",
    "guarded_run_return": "$(contract_file_contains "${CONTRACT_DIR}/guarded-run.txt" "safe_return_to_recovery=ok")",
    "graphical_session_supervisor": "$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "[AQUA-SESSION] stage=graphical-session-supervisor status=ok")",
    "graphical_session_supervisor_bounded": "$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "policy=bounded-restart-with-recovery-fallback")",
    "graphical_session_supervisor_recovery": "$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "recovery_fallback=armed")",
    "graphical_session_supervisor_safe_default": "$(contract_file_contains "${CONTRACT_DIR}/graphical-session-supervisor.txt" "session_started=false")",
    "media_service_supervisor": "$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "[AQUA-MEDIA] stage=media-service-supervisor status=ok")",
    "media_service_supervisor_safe_default": "$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "enabled=false")",
    "media_service_supervisor_ordered_start": "$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "ordered_start=pipewire,wireplumber")",
    "media_service_supervisor_ordered_stop": "$(contract_file_contains "${CONTRACT_DIR}/media-service-supervisor.txt" "ordered_stop=wireplumber,pipewire")",
    "graphical_session_boot": "$(contract_file_contains "${CONTRACT_DIR}/graphical-session-boot.txt" "[AQUA-BOOT] stage=graphical-session-activation status=disabled")",
    "graphical_session_boot_kernel_gate": "$(contract_file_contains "${CONTRACT_DIR}/graphical-session-boot.txt" "reason=kernel-flag-absent")",
    "graphical_session_boot_safe_default": "$(contract_file_contains "${CONTRACT_DIR}/graphical-session-boot.txt" "boot_graphics=false")",
    "handoff_gate": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "[AQUA-GATE] stage=nested-preview-handoff status=ok")",
    "handoff_gate_ready": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "gate_status=qemu-safe-nested-preview-handoff-gate-ready")",
    "handoff_gate_guarded": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "guarded_run_ready=ok")",
    "handoff_gate_handoff": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "handoff_ready=ok")",
    "handoff_gate_preview": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "visible_preview_ready=ok")",
    "handoff_gate_loop": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "preview_loop_ready=ok")",
    "handoff_gate_backend": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_nested_backend_gate=ok")",
    "handoff_gate_backend_ready": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_backend_ready=ok")",
    "handoff_gate_backend_no_start": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_backend_no_display_start=ok")",
    "handoff_gate_candidate": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "candidate_path=manual-nested-preview")",
    "handoff_gate_manual": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "manual_operator_required=true")",
    "handoff_gate_no_auto": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "automatic_promotion=false")",
    "handoff_gate_recovery": "$(contract_file_contains "${CONTRACT_DIR}/handoff-gate.txt" "safe_to_remain_in_recovery=ok")",
    "output_plan": "$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "[AQUA-COMPOSITOR] stage=output-plan-probe status=ok")",
    "output_plan_backend": "$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "primary_backend=nested-dev-window")",
    "output_plan_later_backend": "$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "later_backend=qemu-drm-kms")",
    "output_plan_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/output-plan-probe.txt" "recovery_safe=ok")",
    "display_output_handoff": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "[AQUA-COMPOSITOR] stage=display-output-handoff status=ok")",
    "display_output_handoff_ready": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "handoff_status=display-output-handoff-ready")",
    "display_output_handoff_backend": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "target_backend=nested-dev-window")",
    "display_output_handoff_framebuffer": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "frame_buffer_bytes=6291456")",
    "display_output_handoff_frame_format": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "frame_format=raw-rgba8888-composited-client-preview")",
    "display_output_handoff_frame_checksum": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "frame_checksum=")",
    "display_output_handoff_client_snapshot": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "client_layer_buffer_snapshot_bytes=")",
    "display_output_handoff_snapshot_mode": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "client_layer_snapshot_mode=full-buffer-snapshot")",
    "display_output_handoff_no_start": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "display_output_started=false")",
    "display_output_handoff_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/display-output-handoff-probe.txt" "recovery_safe=ok")",
    "display_activation_plan": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "[AQUA-COMPOSITOR] stage=display-activation-plan status=ok")",
    "display_activation_plan_ready": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "activation_status=manual-display-activation-plan-ready")",
    "display_activation_plan_manual": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "launch_mode=manual-dev")",
    "display_activation_plan_handoff": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "source_handoff_ready=ok")",
    "display_activation_plan_frame": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "frame_format=raw-rgba8888-composited-client-preview")",
    "display_activation_plan_fallback_tty": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "fallback_tty_required=true")",
    "display_activation_plan_can_activate": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "can_activate_display_output=ok")",
    "display_activation_plan_no_start": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "display_output_started=false")",
    "display_activation_plan_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "autostart=false")",
    "display_activation_plan_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/display-activation-plan-probe.txt" "recovery_safe=ok")",
    "display_output_smoke": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "[AQUA-COMPOSITOR] stage=display-output-smoke status=ok")",
    "display_output_smoke_complete": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "smoke_status=manual-display-output-smoke-complete")",
    "display_output_smoke_started": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "display_output_started=true")",
    "display_output_smoke_stopped": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "display_output_stopped=true")",
    "display_output_smoke_frames": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "presented_frames=3")",
    "display_output_smoke_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "autostart=false")",
    "display_output_smoke_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "boot_graphics=false")",
    "display_output_smoke_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/display-output-smoke.txt" "recovery_safe=ok")",
    "nested_output_surface": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "[AQUA-COMPOSITOR] stage=nested-output-surface status=ok")",
    "nested_output_surface_complete": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_status=nested-output-surface-lifecycle-complete")",
    "nested_output_surface_acquired": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_acquired=ok")",
    "nested_output_surface_configured": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_configured=ok")",
    "nested_output_surface_frame_attached": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "frame_attached=ok")",
    "nested_output_surface_frame_presented": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "frame_presented=ok")",
    "nested_output_surface_released": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "surface_released=ok")",
    "nested_output_surface_frames": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "presented_frames=3")",
    "nested_output_surface_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "autostart=false")",
    "nested_output_surface_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "boot_graphics=false")",
    "nested_output_surface_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/nested-output-surface.txt" "recovery_safe=ok")",
    "visible_preview_plan": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "[AQUA-COMPOSITOR] stage=visible-preview-plan-probe status=ok")",
    "visible_preview_output": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "output_plan_ready=ok")",
    "visible_preview_frame_buffer": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "frame_buffer_ready=ok")",
    "visible_preview_raster": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "raster_ready=ok")",
    "visible_preview_png_export": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "png_export_ready=ok")",
    "visible_preview_client_layer_pipeline": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_pipeline_ready=ok")",
    "visible_preview_client_layer_count": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_count=2")",
    "visible_preview_client_layer_checksum": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_checksum=")",
    "visible_preview_client_layer_buffer_snapshot": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_buffer_snapshot_bytes=")",
    "visible_preview_client_layer_snapshot_mode": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "client_layer_snapshot_mode=full-buffer-snapshot")",
    "visible_preview_window_not_started": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-plan-probe.txt" "preview_window_started=false")",
    "visible_preview_export": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "[AQUA-COMPOSITOR] stage=visible-preview-export-probe status=ok")",
    "visible_preview_export_format": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "format=html-data-uri-png-preview")",
    "visible_preview_export_bytes": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "html_bytes=")",
    "visible_preview_export_checksum": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "html_checksum=")",
    "visible_preview_export_client_layers": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_pipeline_ready=ok")",
    "visible_preview_export_client_layers_composited": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_composited=ok")",
    "visible_preview_export_client_layer_buffer_snapshot": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_buffer_snapshot_bytes=")",
    "visible_preview_export_client_layer_snapshot_mode": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "client_layer_snapshot_mode=full-buffer-snapshot")",
    "visible_preview_export_png_checksum": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-export-probe.txt" "png_checksum=")",
    "nested_preview_loop": "$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "[AQUA-COMPOSITOR] stage=nested-preview-loop status=ok")",
    "nested_preview_frame_clock": "$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "frame_clock_started=ok")",
    "nested_preview_frames": "$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "rendered_frames=3")",
    "nested_preview_manual": "$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "manual_start_required=true")",
    "nested_preview_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/nested-preview-loop.txt" "autostart=false")",
    "manual_nested_preview_backend": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "[AQUA-COMPOSITOR] stage=manual-nested-preview-backend status=ok")",
    "manual_nested_preview_backend_ready": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "backend_status=manual-nested-preview-backend-ready")",
    "manual_nested_preview_backend_path": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "backend_path=nested-dev-window")",
    "manual_nested_preview_backend_selected": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "backend_selected=ok")",
    "manual_nested_preview_backend_handoff": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "handoff_ready=ok")",
    "manual_nested_preview_backend_surface": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "surface_lifecycle_ready=ok")",
    "manual_nested_preview_backend_loop": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "frame_loop_ready=ok")",
    "manual_nested_preview_backend_export": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "visible_export_ready=ok")",
    "manual_nested_preview_backend_frame": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "frame_format=raw-rgba8888-composited-client-preview")",
    "manual_nested_preview_backend_checksum_match": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "frame_checksum_matches_surface=ok")",
    "manual_nested_preview_backend_fallback_tty": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "fallback_tty_available=true")",
    "manual_nested_preview_backend_no_start": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "display_output_started=false")",
    "manual_nested_preview_backend_stopped": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "display_output_stopped=true")",
    "manual_nested_preview_backend_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "autostart=false")",
    "manual_nested_preview_backend_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-backend.txt" "recovery_safe=ok")",
    "manual_nested_preview_execution": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "[AQUA-PREVIEW] stage=manual-nested-preview-execution status=ok")",
    "manual_nested_preview_execution_ready": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "execution_status=qemu-safe-manual-nested-preview-execution-ready")",
    "manual_nested_preview_execution_gate": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "handoff_gate_ready=ok")",
    "manual_nested_preview_execution_backend": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "handoff_gate_manual_backend=ok")",
    "manual_nested_preview_execution_operator": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "operator_acknowledged=true")",
    "manual_nested_preview_execution_frames": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "bounded_frames=3")",
    "manual_nested_preview_execution_started": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "display_output_started=true")",
    "manual_nested_preview_execution_stopped": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "display_output_stopped=true")",
    "manual_nested_preview_execution_cleanup": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "execution_cleanup=ok")",
    "manual_nested_preview_execution_return": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "safe_return_to_recovery=ok")",
    "manual_nested_preview_execution_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "autostart=false")",
    "manual_nested_preview_execution_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution.txt" "boot_graphics=false")",
    "manual_nested_preview_execution_probe": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution-probe.txt" "[AQUA-COMPOSITOR] stage=manual-nested-preview-execution status=ok")",
    "manual_nested_preview_execution_probe_complete": "$(contract_file_contains "${CONTRACT_DIR}/manual-nested-preview-execution-probe.txt" "execution_status=manual-nested-preview-execution-complete")",
    "visible_preview_request": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "[AQUA-PREVIEW] stage=visible-nested-preview-request status=ok")",
    "visible_preview_request_ready": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_status=qemu-safe-visible-nested-preview-request-ready")",
    "visible_preview_request_target": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_target=host-visible-nested-window")",
    "visible_preview_request_backend": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "window_backend=minifb")",
    "visible_preview_request_feature_gate": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "feature_gate=host-window-preview")",
    "visible_preview_request_manual_execution": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "manual_execution_ready=ok")",
    "visible_preview_request_file": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_file_written=ok")",
    "visible_preview_request_host_tool_not_packaged": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "host_tool_packaged=false")",
    "visible_preview_request_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_autostart=false")",
    "visible_preview_request_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "request_boot_graphics=false")",
    "visible_preview_request_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-request.txt" "safe_return_to_recovery=ok")",
    "visible_preview_launch": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "[AQUA-PREVIEW] stage=visible-nested-preview-launch status=ok")",
    "visible_preview_launch_ready": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_status=qemu-safe-visible-nested-preview-launch-ready")",
    "visible_preview_launch_request": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "request_command_ready=ok")",
    "visible_preview_launch_plan": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_plan_written=ok")",
    "visible_preview_launch_backend": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_window_backend=minifb")",
    "visible_preview_launch_feature_gate": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_feature_gate=host-window-preview")",
    "visible_preview_launch_host_tool_not_packaged": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_host_tool_packaged=false")",
    "visible_preview_launch_no_qemu_window": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_qemu_window_started=false")",
    "visible_preview_launch_no_preview_window": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_preview_window_started=false")",
    "visible_preview_launch_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_autostart=false")",
    "visible_preview_launch_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "launch_boot_graphics=false")",
    "visible_preview_launch_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/visible-preview-launch.txt" "safe_return_to_recovery=ok")",
    "recovery_help": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "[AQUA-RECOVERY] stage=operator-help status=ok")",
    "recovery_help_text_mode": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "mode=text-recovery")",
    "recovery_help_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "autostart=false")",
    "recovery_help_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "boot_graphics=false")",
    "recovery_help_visible_launcher": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "aqua-visible-preview-launch")",
    "recovery_help_host_tool_external": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "aqua-host-tools --features host-window-preview -- smoke-manual-execution-window")",
    "recovery_help_operator_pass_host": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "scripts/run-qemu-visible-operator-pass.sh")",
    "recovery_help_operator_pass_no_launch": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh")",
    "recovery_help_operator_checklist": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "build/qemu-visible-operator-checklist.md")",
    "recovery_help_operator_pass_artifact": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "build/qemu-visible-operator-pass.txt")",
    "recovery_help_operator_pass_external": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "Host QEMU operator-pass tooling is not packaged into the Buildroot rootfs.")",
    "recovery_help_visible_pass_report": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "aqua-qemu-visible-pass-report")",
    "recovery_help_pass_report_required": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "The QEMU visible manual runbook requires pass_report_required=true.")",
    "recovery_help_pass_report_after_apply": "$(contract_file_contains "${CONTRACT_DIR}/recovery-help.txt" "Run aqua-qemu-visible-pass-report after confirmed evidence bundle apply.")",
    "qemu_visible_operator_pass_no_launch": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "status=no-launch-ready")",
    "qemu_visible_operator_pass_launch_required": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "launch_confirmation_required=true")",
    "qemu_visible_operator_pass_not_confirmed": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "launch_confirmed=false")",
    "qemu_visible_operator_pass_no_positive_without_evidence": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "no_positive_observation_without_evidence=true")",
    "qemu_visible_operator_pass_no_unverified_bundle": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "no_unverified_bundle_acceptance=true")",
    "qemu_visible_operator_pass_recovery_safe": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "safe_return_to_recovery=ok")",
    "qemu_visible_operator_pass_stop_rule": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "operator_pass_stop_rule=Do not mark VM display observed")",
    "qemu_visible_operator_pass_no_launch_command": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "no_launch_rehearsal_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh")",
    "qemu_visible_operator_pass_confirmed_command": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "confirmed_launch_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh")",
    "qemu_visible_operator_pass_capture_flow": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_capture_flow_command=scripts/run-qemu-visible-ready-capture-flow.sh")",
    "qemu_visible_operator_pass_capture_verify": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_capture_verify_command=scripts/verify-qemu-visible-capture.sh")",
    "qemu_visible_operator_pass_evidence_flow": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_evidence_flow_command=scripts/run-qemu-visible-evidence-flow.sh")",
    "qemu_visible_operator_pass_vm_apply": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "next_vm_apply_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply")",
    "qemu_visible_operator_pass_capture_hash_gate": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "capture_hash_verification_required=true")",
    "qemu_visible_operator_pass_capture_hash_status": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "bundle_capture_hash_status=ok")",
    "qemu_visible_operator_pass_positive_capture_hash_status": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "bundle_positive_capture_hash_status=ok")",
    "qemu_visible_operator_pass_missing_capture_hash_rejected": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "bundle_missing_capture_hash_rejected_status=ok")",
    "qemu_visible_operator_pass_manual_runbook_pass_report_required": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "manual_runbook_pass_report_required_status=ok")",
    "qemu_visible_operator_pass_preflight_source": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "preflight_source_sha256=")",
    "qemu_visible_operator_pass_preflight_mtime": "$(contract_file_contains "${QEMU_VISIBLE_OPERATOR_PASS}" "preflight_source_mtime_utc=")",
    "operator_transcript": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "[AQUA-RECOVERY] stage=operator-transcript status=ok")",
    "operator_transcript_ready": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_status=qemu-safe-operator-transcript-ready")",
    "operator_transcript_dry_run": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_dry_run=true")",
    "operator_transcript_qemu_steps": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_qemu_steps=9")",
    "operator_transcript_host_steps": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "transcript_host_steps=2")",
    "operator_transcript_qemu_command": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "next_qemu_command=/usr/bin/aqua-graphics-enable-gate")",
    "operator_transcript_host_command": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "next_host_command=aqua-host-tools --features host-window-preview -- smoke-manual-execution-window")",
    "operator_transcript_host_tool_not_packaged": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "host_tool_packaged=false")",
    "operator_transcript_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "autostart=false")",
    "operator_transcript_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "boot_graphics=false")",
    "operator_transcript_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/operator-transcript.txt" "safe_return_to_recovery=ok")",
    "graphics_enable_gate": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "[AQUA-GATE] stage=graphics-enable-gate status=ok")",
    "graphics_enable_gate_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "gate_status=qemu-safe-graphics-enable-gate-ready")",
    "graphics_enable_gate_preflight": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "preflight_status=evaluated")",
    "graphics_enable_gate_allow_handoff": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_handoff_gate_ok=true")",
    "graphics_enable_gate_allow_manual_execution": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_manual_execution_ok=true")",
    "graphics_enable_gate_allow_fallback_tty": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_fallback_tty_supervised=true")",
    "graphics_enable_gate_allow_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "allow_when_boot_graphics_explicitly_enabled=true")",
    "graphics_enable_gate_check_handoff": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_handoff_gate=ok")",
    "graphics_enable_gate_check_manual_execution": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_manual_execution=ok")",
    "graphics_enable_gate_check_fallback_tty": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_fallback_tty=ok")",
    "graphics_enable_gate_check_cleanup": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_manual_cleanup=ok")",
    "graphics_enable_gate_check_stopped": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "check_manual_stopped=ok")",
    "graphics_enable_gate_currently_blocked": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "currently_allowable=false")",
    "graphics_enable_gate_refused": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "enable_decision=refuse")",
    "graphics_enable_gate_reason": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "refuse_reason=boot-graphics-disabled-until-fail-safe-compositor")",
    "graphics_enable_gate_blocked_criteria": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "blocked_criteria=boot_graphics=false")",
    "graphics_enable_gate_plan": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "gate_plan_written=ok")",
    "graphics_enable_gate_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "autostart=false")",
    "graphics_enable_gate_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "boot_graphics=false")",
    "graphics_enable_gate_no_display_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "display_output_started=false")",
    "graphics_enable_gate_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate.txt" "safe_return_to_recovery=ok")",
    "graphics_enable_gate_positive": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "[AQUA-GATE] stage=graphics-enable-gate status=ok")",
    "graphics_enable_gate_positive_preflight": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "preflight_status=evaluated")",
    "graphics_enable_gate_positive_dry_run": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "positive_dry_run=true")",
    "graphics_enable_gate_positive_handoff": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "check_handoff_gate=ok")",
    "graphics_enable_gate_positive_manual_execution": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "check_manual_execution=ok")",
    "graphics_enable_gate_positive_allowable": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "currently_allowable=true")",
    "graphics_enable_gate_positive_decision": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "enable_decision=allow-dry-run")",
    "graphics_enable_gate_positive_no_actual_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "actual_graphics_started=false")",
    "graphics_enable_gate_positive_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "boot_graphics=false")",
    "graphics_enable_gate_positive_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-enable-gate-positive.txt" "safe_return_to_recovery=ok")",
    "graphics_launch_candidate": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "[AQUA-GATE] stage=graphics-launch-candidate status=ok")",
    "graphics_launch_candidate_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_status=qemu-safe-graphics-launch-candidate-ready")",
    "graphics_launch_candidate_allowable": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_allowable=true")",
    "graphics_launch_candidate_selected": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_selected=true")",
    "graphics_launch_candidate_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "candidate_started=false")",
    "graphics_launch_candidate_no_actual_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "actual_graphics_started=false")",
    "graphics_launch_candidate_no_display_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "display_output_started=false")",
    "graphics_launch_candidate_rollback": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "rollback_strategy=return-to-text-recovery")",
    "graphics_launch_candidate_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-launch-candidate.txt" "safe_return_to_recovery=ok")",
    "graphics_rollback_drill": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "[AQUA-GATE] stage=graphics-rollback-drill status=ok")",
    "graphics_rollback_drill_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "rollback_drill_status=qemu-safe-rollback-drill-ready")",
    "graphics_rollback_drill_cancel_path": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "operator_cancel_simulated=true")",
    "graphics_rollback_drill_failure_path": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "startup_failure_simulated=true")",
    "graphics_rollback_drill_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "candidate_started=false")",
    "graphics_rollback_drill_no_actual_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "actual_graphics_started=false")",
    "graphics_rollback_drill_no_display_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "display_output_started=false")",
    "graphics_rollback_drill_verified": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "rollback_verified=true")",
    "graphics_rollback_drill_command": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "rollback_command=/usr/bin/aqua-recovery")",
    "graphics_rollback_drill_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-rollback-drill.txt" "safe_return_to_recovery=ok")",
    "graphics_startup_preflight": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "[AQUA-GATE] stage=graphics-startup-preflight status=ok")",
    "graphics_startup_preflight_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "startup_preflight_status=qemu-safe-guarded-startup-preflight-ready")",
    "graphics_startup_preflight_bounded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "bounded_startup_candidate=true")",
    "graphics_startup_preflight_operator_ack": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "operator_ack_required=true")",
    "graphics_startup_preflight_fallback_tty": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "fallback_tty_available=true")",
    "graphics_startup_preflight_rollback": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "rollback_verified=true")",
    "graphics_startup_preflight_decision": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "startup_preflight_decision=allow-bounded-manual-preflight-only")",
    "graphics_startup_preflight_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "candidate_started=false")",
    "graphics_startup_preflight_no_actual_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "actual_graphics_started=false")",
    "graphics_startup_preflight_no_display_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "display_output_started=false")",
    "graphics_startup_preflight_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-preflight.txt" "safe_return_to_recovery=ok")",
    "graphics_startup_rehearsal": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "[AQUA-GATE] stage=graphics-startup-rehearsal status=ok")",
    "graphics_startup_rehearsal_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "startup_rehearsal_status=qemu-safe-guarded-startup-rehearsal-complete")",
    "graphics_startup_rehearsal_bounded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "bounded_startup_rehearsal=true")",
    "graphics_startup_rehearsal_frames": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "bounded_run_frames=3")",
    "graphics_startup_rehearsal_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "display_output_started=true")",
    "graphics_startup_rehearsal_stopped": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "display_output_stopped=true")",
    "graphics_startup_rehearsal_no_actual_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "actual_graphics_started=false")",
    "graphics_startup_rehearsal_no_desktop_shell": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "desktop_shell_started=false")",
    "graphics_startup_rehearsal_decision": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "startup_rehearsal_decision=allow-next-manual-qemu-display-step")",
    "graphics_startup_rehearsal_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-startup-rehearsal.txt" "safe_return_to_recovery=ok")",
    "graphics_qemu_display_gate": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "[AQUA-GATE] stage=graphics-qemu-display-gate status=ok")",
    "graphics_qemu_display_gate_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_gate_status=qemu-safe-manual-display-step-gate-ready")",
    "graphics_qemu_display_gate_candidate": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_step_candidate=true")",
    "graphics_qemu_display_gate_passed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_gate_passed=true")",
    "graphics_qemu_display_gate_decision": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "qemu_display_gate_decision=allow-manual-qemu-display-step")",
    "graphics_qemu_display_gate_manual": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "manual_start_required=true")",
    "graphics_qemu_display_gate_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "visible_qemu_step_started=false")",
    "graphics_qemu_display_gate_no_actual_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "actual_graphics_started=false")",
    "graphics_qemu_display_gate_no_desktop_shell": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "desktop_shell_started=false")",
    "graphics_qemu_display_gate_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-display-gate.txt" "safe_return_to_recovery=ok")",
    "graphics_visible_qemu_attempt": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "[AQUA-GATE] stage=graphics-visible-qemu-attempt status=ok")",
    "graphics_visible_qemu_attempt_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "visible_qemu_attempt_status=qemu-safe-visible-attempt-plan-ready")",
    "graphics_visible_qemu_attempt_plan": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "attempt_plan_written=true")",
    "graphics_visible_qemu_attempt_allowed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "visible_qemu_attempt_allowed=true")",
    "graphics_visible_qemu_attempt_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "visible_qemu_attempt_started=false")",
    "graphics_visible_qemu_attempt_manual": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "manual_start_required=true")",
    "graphics_visible_qemu_attempt_fallback_tty": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "fallback_tty_available=true")",
    "graphics_visible_qemu_attempt_no_desktop_shell": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "desktop_shell_started=false")",
    "graphics_visible_qemu_attempt_command": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "qemu_attempt_command=/usr/bin/aqua-compositor-guarded-run")",
    "graphics_visible_qemu_attempt_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt" "safe_return_to_recovery=ok")",
    "graphics_visible_attempt_transcript": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "[AQUA-GATE] stage=graphics-visible-attempt-transcript status=ok")",
    "graphics_visible_attempt_transcript_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "visible_attempt_transcript_status=qemu-safe-visible-attempt-transcript-ready")",
    "graphics_visible_attempt_transcript_sequence": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "operator_sequence_ready=true")",
    "graphics_visible_attempt_transcript_step_attempt": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "operator_step_2=/usr/bin/aqua-graphics-visible-qemu-attempt")",
    "graphics_visible_attempt_transcript_step_run": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "operator_step_3=/usr/bin/aqua-compositor-guarded-run")",
    "graphics_visible_attempt_transcript_expected_return": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "expected_return=safe-return-to-recovery")",
    "graphics_visible_attempt_transcript_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "visible_qemu_attempt_started=false")",
    "graphics_visible_attempt_transcript_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "persistent_graphical_session_started=false")",
    "graphics_visible_attempt_transcript_no_desktop_shell": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "desktop_shell_started=false")",
    "graphics_visible_attempt_transcript_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt" "safe_return_to_recovery=ok")",
    "graphics_visible_attempt_result": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "[AQUA-GATE] stage=graphics-visible-attempt-result status=ok")",
    "graphics_visible_attempt_result_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "visible_attempt_result_status=qemu-safe-visible-attempt-result-ready")",
    "graphics_visible_attempt_result_source": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "source_transcript_ready=ok")",
    "graphics_visible_attempt_result_manual_not_run": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "attempt_result=manual-not-run")",
    "graphics_visible_attempt_result_collected": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "attempt_result_collected=true")",
    "graphics_visible_attempt_result_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "visible_qemu_attempt_started=false")",
    "graphics_visible_attempt_result_not_completed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "visible_qemu_attempt_completed=false")",
    "graphics_visible_attempt_result_no_display_start": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "display_output_started=false")",
    "graphics_visible_attempt_result_no_display_stop": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "display_output_stopped=false")",
    "graphics_visible_attempt_result_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "persistent_graphical_session_started=false")",
    "graphics_visible_attempt_result_no_desktop_shell": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "desktop_shell_started=false")",
    "graphics_visible_attempt_result_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "boot_graphics=false")",
    "graphics_visible_attempt_result_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "autostart=false")",
    "graphics_visible_attempt_result_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-result.txt" "safe_return_to_recovery=ok")",
    "graphics_visible_attempt_runner": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "[AQUA-GATE] stage=graphics-visible-attempt-runner status=ok")",
    "graphics_visible_attempt_runner_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "visible_attempt_runner_status=qemu-safe-visible-attempt-runner-complete")",
    "graphics_visible_attempt_runner_guarded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_guarded_run=ok")",
    "graphics_visible_attempt_runner_bounded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_bounded_run=ok")",
    "graphics_visible_attempt_runner_result": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_result=completed-bounded-run")",
    "graphics_visible_attempt_runner_collector": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "attempt_result_collector=ok")",
    "graphics_visible_attempt_runner_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "visible_qemu_attempt_started=true")",
    "graphics_visible_attempt_runner_completed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "visible_qemu_attempt_completed=true")",
    "graphics_visible_attempt_runner_frames": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "bounded_run_frames=3")",
    "graphics_visible_attempt_runner_display_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "display_output_started=true")",
    "graphics_visible_attempt_runner_display_stopped": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "display_output_stopped=true")",
    "graphics_visible_attempt_runner_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "persistent_graphical_session_started=false")",
    "graphics_visible_attempt_runner_no_desktop_shell": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "desktop_shell_started=false")",
    "graphics_visible_attempt_runner_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "boot_graphics=false")",
    "graphics_visible_attempt_runner_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "autostart=false")",
    "graphics_visible_attempt_runner_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt" "safe_return_to_recovery=ok")",
    "graphics_qemu_visible_boot_check": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "[AQUA-GATE] stage=graphics-qemu-visible-boot-check status=ok")",
    "graphics_qemu_visible_boot_check_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "qemu_visible_boot_check_status=qemu-visible-boot-path-check-ready")",
    "graphics_qemu_visible_boot_check_runner": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "runner_result_completed=ok")",
    "graphics_fbdev_present": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "[AQUA-GATE] stage=graphics-fbdev-present status=dry-run-ok")",
    "graphics_fbdev_present_probe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "fbdev_probe=ok")",
    "graphics_fbdev_present_frame": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "target_frame_bytes=3145728")",
    "graphics_fbdev_present_not_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "visible_frame_presented=false")",
    "graphics_fbdev_present_bounded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "bounded_frames=1")",
    "graphics_fbdev_present_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "boot_graphics=false")",
    "graphics_fbdev_present_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "autostart=false")",
    "graphics_fbdev_present_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-fbdev-present.txt" "safe_return_to_recovery=ok")",
    "graphics_fbdev_headless_qemu_write": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-TEST] stage=fbdev-present-qemu status=ok framebuffer_write=true visible_observation=false safe_return_to_recovery=ok")",
    "graphics_fbdev_headless_qemu_checksum": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "frame_checksum=c85dbfbfc17843af")",
    "graphics_fbdev_headless_qemu_wallpaper": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "wallpaper_source=runtime-asset")",
    "graphics_fbdev_headless_qemu_mode": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "target_size=1280x800")",
    "graphics_fbdev_headless_qemu_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "safe_return_to_recovery=ok")",
    "graphics_fbdev_headless_qemu_capture": "$(contract_file_contains "${FBDEV_QEMU_CAPTURE}" "status=ok")",
    "graphics_fbdev_headless_qemu_capture_dimensions": "$(contract_file_contains "${FBDEV_QEMU_CAPTURE}" "dimensions=1280x800")",
    "graphics_fbdev_headless_qemu_ppm_checksum": "$(capture_checksum_status "${FBDEV_QEMU_PPM}" "ppm_sha256")",
    "graphics_fbdev_headless_qemu_png_checksum": "$(capture_checksum_status "${FBDEV_QEMU_PNG}" "png_sha256")",
    "graphics_fbdev_headless_qemu_capture_unobserved": "$(contract_file_contains "${FBDEV_QEMU_CAPTURE}" "visible_observation=false")",
    "graphics_qemu_visible_boot_path_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "qemu_visible_boot_path_ready=true")",
    "graphics_qemu_visible_boot_observed_false": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "qemu_vm_display_observed=false")",
    "graphics_qemu_visible_boot_manual_observation": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "manual_observation_required=true")",
    "graphics_qemu_visible_boot_bounded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "bounded_attempt_completed=true")",
    "graphics_qemu_visible_boot_frames": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "bounded_run_frames=3")",
    "graphics_qemu_visible_boot_display_started": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "display_output_started=true")",
    "graphics_qemu_visible_boot_display_stopped": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "display_output_stopped=true")",
    "graphics_qemu_visible_boot_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "persistent_graphical_session_started=false")",
    "graphics_qemu_visible_boot_no_desktop_shell": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "desktop_shell_started=false")",
    "graphics_qemu_visible_boot_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "boot_graphics=false")",
    "graphics_qemu_visible_boot_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "autostart=false")",
    "graphics_qemu_visible_boot_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt" "safe_return_to_recovery=ok")",
    "graphics_qemu_observation_marker": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "[AQUA-GATE] stage=graphics-qemu-observation-marker status=ok")",
    "graphics_qemu_observation_marker_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "qemu_observation_marker_status=qemu-visible-observation-marker-ready")",
    "graphics_qemu_observation_marker_source": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "source_boot_check_ready=ok")",
    "graphics_qemu_observation_marker_path_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "qemu_visible_boot_path_ready=true")",
    "graphics_qemu_observation_marker_not_observed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "qemu_vm_display_observed=false")",
    "graphics_qemu_observation_marker_status_not_observed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "observation_status=not-observed")",
    "graphics_qemu_observation_marker_recorded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "manual_observation_recorded=true")",
    "graphics_qemu_observation_marker_operator": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "operator_confirmation_required=true")",
    "graphics_qemu_observation_marker_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "persistent_graphical_session_started=false")",
    "graphics_qemu_observation_marker_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "boot_graphics=false")",
    "graphics_qemu_observation_marker_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "autostart=false")",
    "graphics_qemu_observation_marker_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt" "safe_return_to_recovery=ok")",
    "qemu_visible_evidence_record": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "[AQUA-GATE] stage=qemu-visible-evidence-record status=ok")",
    "qemu_visible_evidence_record_ready": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "evidence_record_status=qemu-vm-display-evidence-ready")",
    "qemu_visible_evidence_record_status": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "evidence_status=operator-capture-recorded")",
    "qemu_visible_evidence_record_capture": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "capture_file=manual-qemu-display-capture-required.png")",
    "qemu_visible_evidence_record_manual": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "qemu_visible_manual_evidence=true")",
    "qemu_visible_evidence_record_allows_observation": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "observation_marker_may_be_positive=true")",
    "qemu_visible_evidence_record_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "persistent_graphical_session_started=false")",
    "qemu_visible_evidence_record_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "boot_graphics=false")",
    "qemu_visible_evidence_record_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "autostart=false")",
    "qemu_visible_evidence_record_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-record.txt" "safe_return_to_recovery=ok")",
    "graphics_qemu_observation_positive": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "[AQUA-GATE] stage=graphics-qemu-observation-marker status=ok")",
    "graphics_qemu_observation_positive_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "qemu_observation_marker_status=qemu-visible-observation-marker-ready")",
    "graphics_qemu_observation_positive_source": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "source_boot_check_ready=ok")",
    "graphics_qemu_observation_positive_path_ready": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "qemu_visible_boot_path_ready=true")",
    "graphics_qemu_observation_positive_observed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "qemu_vm_display_observed=true")",
    "graphics_qemu_observation_positive_status_observed": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "observation_status=observed")",
    "graphics_qemu_observation_positive_evidence_required": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "evidence_required=true")",
    "graphics_qemu_observation_positive_evidence_recorded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "evidence_status=operator-capture-recorded")",
    "graphics_qemu_observation_positive_recorded": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "manual_observation_recorded=true")",
    "graphics_qemu_observation_positive_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "persistent_graphical_session_started=false")",
    "graphics_qemu_observation_positive_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "boot_graphics=false")",
    "graphics_qemu_observation_positive_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "autostart=false")",
    "graphics_qemu_observation_positive_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt" "safe_return_to_recovery=ok")",
    "qemu_visible_pass_report": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "[AQUA-GATE] stage=qemu-visible-pass-report status=ok")",
    "qemu_visible_pass_report_ready": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "pass_report_status=qemu-visible-pass-report-ready")",
    "qemu_visible_pass_report_source_attempt": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "source_attempt_result_collected=ok")",
    "qemu_visible_pass_report_source_observation": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "source_observation_recorded=ok")",
    "qemu_visible_pass_report_observed": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "qemu_vm_display_observed=true")",
    "qemu_visible_pass_report_attempt_completed": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "visible_qemu_attempt_completed=true")",
    "qemu_visible_pass_report_evidence_required": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "evidence_required=true")",
    "qemu_visible_pass_report_evidence_recorded": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "evidence_status=operator-capture-recorded")",
    "qemu_visible_pass_report_evidence_rule": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "positive_observation_requires_evidence=true")",
    "qemu_visible_pass_report_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "persistent_graphical_session_started=false")",
    "qemu_visible_pass_report_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "boot_graphics=false")",
    "qemu_visible_pass_report_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "autostart=false")",
    "qemu_visible_pass_report_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-pass-report.txt" "safe_return_to_recovery=ok")",
    "qemu_visible_manual_runbook": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "[AQUA-GATE] stage=qemu-visible-manual-runbook status=ok")",
    "qemu_visible_manual_runbook_ready": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "manual_runbook_status=qemu-vm-display-manual-runbook-ready")",
    "qemu_visible_manual_runbook_host": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "host_entrypoint=scripts/run-qemu-visible-manual.sh")",
    "qemu_visible_manual_runbook_ready_capture_flow": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "host_ready_capture_flow_supported=true")",
    "qemu_visible_manual_runbook_script": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "host_display_script_ready=true")",
    "qemu_visible_manual_runbook_no_docker": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "docker_required=false")",
    "qemu_visible_manual_runbook_manual_observation": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "manual_observation_required=true")",
    "qemu_visible_manual_runbook_evidence_required": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "evidence_record_required=true")",
    "qemu_visible_manual_runbook_observation_rule": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "observation_rule=run-step-13-only-after-operator-confirms-vm-display-and-records-evidence-then-run-step-14")",
    "qemu_visible_manual_runbook_pass_report_required": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "pass_report_required=true")",
    "qemu_visible_manual_runbook_bounded": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "bounded_attempt_required=true")",
    "qemu_visible_manual_runbook_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "persistent_graphical_session_started=false")",
    "qemu_visible_manual_runbook_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "boot_graphics=false")",
    "qemu_visible_manual_runbook_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "autostart=false")",
    "qemu_visible_manual_runbook_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt" "safe_return_to_recovery=ok")",
    "qemu_visible_evidence_bundle_apply": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=ok")",
    "qemu_visible_evidence_bundle_apply_ready": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "bundle_apply_status=qemu-visible-evidence-bundle-apply-ready")",
    "qemu_visible_evidence_bundle_apply_waiting": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "apply_status=waiting-for-operator-confirmation")",
    "qemu_visible_evidence_bundle_apply_not_observed": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "qemu_vm_display_observed=false")",
    "qemu_visible_evidence_bundle_apply_no_evidence": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "evidence_record_written=false")",
    "qemu_visible_evidence_bundle_apply_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "boot_graphics=false")",
    "qemu_visible_evidence_bundle_apply_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "autostart=false")",
    "qemu_visible_evidence_bundle_apply_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "safe_return_to_recovery=ok")",
    "qemu_visible_evidence_bundle_apply_preflight_verified": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "preflight_summary_verified=true")",
    "qemu_visible_evidence_bundle_apply_capture_hash_verified": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt" "capture_hash_verified=true")",
    "qemu_visible_evidence_bundle_apply_positive": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=ok")",
    "qemu_visible_evidence_bundle_apply_positive_applied": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "apply_status=applied-positive-observation")",
    "qemu_visible_evidence_bundle_apply_positive_evidence": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "evidence_record_written=true")",
    "qemu_visible_evidence_bundle_apply_positive_observed": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "qemu_vm_display_observed=true")",
    "qemu_visible_evidence_bundle_apply_positive_no_persistent": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "persistent_graphical_session_started=false")",
    "qemu_visible_evidence_bundle_apply_positive_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "boot_graphics=false")",
    "qemu_visible_evidence_bundle_apply_positive_no_autostart": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "autostart=false")",
    "qemu_visible_evidence_bundle_apply_positive_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "safe_return_to_recovery=ok")",
    "qemu_visible_evidence_bundle_apply_positive_preflight_verified": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "preflight_summary_verified=true")",
    "qemu_visible_evidence_bundle_apply_positive_capture_hash_verified": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt" "capture_hash_verified=true")",
    "qemu_visible_evidence_bundle_apply_missing_preflight_rejected": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=error")",
    "qemu_visible_evidence_bundle_apply_missing_preflight_status": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "bundle_preflight_summary_status=failed")",
    "qemu_visible_evidence_bundle_apply_missing_preflight_verified": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "bundle_preflight_summary_verified=failed")",
    "qemu_visible_evidence_bundle_apply_missing_preflight_exit": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "expected_failure_exit_code=1")",
    "qemu_visible_evidence_bundle_apply_missing_preflight_not_observed": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "operator_confirmed=false")",
    "qemu_visible_evidence_bundle_apply_missing_preflight_unverified": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt" "preflight_summary_verified=false")",
    "qemu_visible_evidence_bundle_apply_missing_capture_hash_rejected": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "[AQUA-GATE] stage=qemu-visible-evidence-bundle-apply status=error")",
    "qemu_visible_evidence_bundle_apply_missing_capture_hash_status": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "bundle_capture_hash_verified=failed")",
    "qemu_visible_evidence_bundle_apply_missing_capture_hash_value": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "capture_hash_verified=missing")",
    "qemu_visible_evidence_bundle_apply_missing_capture_hash_preflight_verified": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "preflight_summary_verified=true")",
    "qemu_visible_evidence_bundle_apply_missing_capture_hash_exit": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "expected_failure_exit_code=1")",
    "qemu_visible_evidence_bundle_apply_missing_capture_hash_not_observed": "$(contract_file_contains "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt" "operator_confirmed=false")",
    "client_window_model": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "[AQUA-COMPOSITOR] stage=client-window-model status=ok")",
    "client_window_focus": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "focus_ready=ok")",
    "client_window_move": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "move_ready=ok")",
    "client_window_resize": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "resize_ready=ok")",
    "client_window_close": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "close_ready=ok")",
    "client_window_stacking": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "stacking_ready=ok")",
    "client_window_chrome": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "chrome_ready=ok")",
    "client_window_no_real_client": "$(contract_file_contains "${CONTRACT_DIR}/client-window-model-probe.txt" "real_wayland_client_started=false")",
    "client_surface_lifecycle": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "[AQUA-COMPOSITOR] stage=client-surface-lifecycle status=ok")",
    "client_surface_configure": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "configure_ready=ok")",
    "client_surface_commit": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "commit_ready=ok")",
    "client_surface_map": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "map_ready=ok")",
    "client_surface_focus": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "focus_ready=ok")",
    "client_surface_unmap": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "unmap_ready=ok")",
    "client_surface_destroy": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "destroy_ready=ok")",
    "client_surface_geometry": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "window_geometry_ready=ok")",
    "client_surface_no_real_client": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt" "real_wayland_client_started=false")",
    "client_surface_registry": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "[AQUA-COMPOSITOR] stage=client-surface-registry status=ok")",
    "client_surface_registry_source": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "source_window_model_ready=ok")",
    "client_surface_registry_record": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "record_count=2")",
    "client_surface_registry_active": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "active_client_id=wayland-client-1")",
    "client_surface_registry_configure": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "configure_serial_ready=ok")",
    "client_surface_registry_lifecycle": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "lifecycle_state_ready=ok")",
    "client_surface_registry_two_client": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "two_client_ready=ok")",
    "client_surface_registry_focus": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "focus_index_ready=ok")",
    "client_surface_registry_stacking": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "stacking_order_ready=ok")",
    "client_surface_registry_close": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "close_request_ready=ok")",
    "client_surface_registry_buffer_metadata": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "buffer_metadata_ready=ok")",
    "client_surface_registry_buffer_import_plan": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "buffer_import_plan_ready=ok")",
    "client_surface_registry_sample_pixel": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "sample_pixel=")",
    "client_surface_registry_sample_grid": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "sample_grid=")",
    "client_surface_registry_buffer_snapshot": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "buffer_snapshot_bytes=")",
    "client_surface_registry_no_render": "$(contract_file_contains "${CONTRACT_DIR}/client-surface-registry-probe.txt" "no_renderer_binding=ok")",
    "renderer_surface_sources": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "[AQUA-COMPOSITOR] stage=renderer-surface-sources status=ok")",
    "renderer_surface_sources_registry": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "source_registry_ready=ok")",
    "renderer_surface_sources_count": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "surface_source_count=2")",
    "renderer_surface_sources_active": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "active_source_ready=ok")",
    "renderer_surface_sources_import": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "import_sources_ready=ok")",
    "renderer_surface_sources_z_order": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "z_order_ready=ok")",
    "renderer_surface_sources_sample_pixel": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "sample_pixel=")",
    "renderer_surface_sources_sample_grid": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "sample_grid=")",
    "renderer_surface_sources_buffer_snapshot": "$(contract_file_contains "${CONTRACT_DIR}/renderer-surface-sources-probe.txt" "buffer_snapshot_bytes=")",
    "renderer_gpu_runtime": "$(contract_file_contains "${CONTRACT_DIR}/renderer-backend-probe.txt" "gpu_runtime_ready=true")",
    "renderer_gpu_backend_selected": "$(contract_file_contains "${CONTRACT_DIR}/renderer-backend-probe.txt" "renderer_selected_backend=smithay-gles2-gbm")",
    "renderer_gpu_software_fallback_safe": "$(contract_file_contains "${CONTRACT_DIR}/renderer-backend-probe.txt" "renderer_software_fallback=false")",
    "renderer_gpu_no_context": "$(contract_file_contains "${CONTRACT_DIR}/renderer-backend-probe.txt" "renderer_context_created=false")",
    "renderer_gpu_no_display_start": "$(contract_file_contains "${CONTRACT_DIR}/renderer-backend-probe.txt" "renderer_display_output_started=false")",
    "renderer_gpu_recovery_safe": "$(contract_file_contains "${CONTRACT_DIR}/renderer-backend-probe.txt" "renderer_recovery_safe=true")",
    "renderer_gpu_offscreen_frame": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "[AQUA-COMPOSITOR] stage=gpu-offscreen-frame status=ok")",
    "renderer_gpu_context_created": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_context_created=true")",
    "renderer_gpu_frame_rendered": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_frame_rendered=true")",
    "renderer_gpu_frame_synchronized": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_frame_synchronized=true")",
    "renderer_gpu_scene_surfaces": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_scene_surface_count=5")",
    "renderer_gpu_scene_surface_layers": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_scene_surface_layer_count=3")",
    "renderer_gpu_scene_shader": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_scene_shader=aqua-surface-compositor-v1")",
    "renderer_gpu_surface_shader_compiled": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_shader_compiled=true")",
    "renderer_gpu_surface_shader_panels": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_shader_panels=3")",
    "renderer_gpu_surface_refraction": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_refraction_strength=0.0025")",
    "renderer_gpu_surface_tint": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_tint_strength=0.18")",
    "renderer_gpu_surface_highlight": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_highlight_strength=0.16")",
    "renderer_gpu_surface_bounded": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_refraction_bounded=true")",
    "renderer_gpu_surface_rounded_mask": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_rounded_mask=true")",
    "renderer_gpu_surface_corner_radius": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_corner_radius_px=12.0")",
    "renderer_gpu_surface_edge_light": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_edge_light_strength=0.24")",
    "renderer_gpu_surface_edge_width": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_edge_width_px=1.5")",
    "renderer_gpu_surface_blur_shader": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_blur_shader_compiled=true")",
    "renderer_gpu_surface_blur_passes": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_blur_passes=2")",
    "renderer_gpu_surface_blur_kernel": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_blur_kernel_samples=9")",
    "renderer_gpu_surface_blur_radius": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_blur_radius_px=4.0")",
    "renderer_gpu_surface_blur_intermediate": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_blur_intermediate_size=320x240")",
    "renderer_gpu_surface_blur_synchronized": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_blur_synchronized=true")",
    "renderer_gpu_surface_blur_composited": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_surface_blur_composited=true")",
    "renderer_gpu_wallpaper_runtime": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_wallpaper_source=runtime-asset")",
    "renderer_gpu_wallpaper_size": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_wallpaper_size=1536x1024")",
    "renderer_gpu_wallpaper_uploaded": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_wallpaper_texture_uploaded=true")",
    "renderer_gpu_wallpaper_composited": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_wallpaper_composited=true")",
    "renderer_gpu_client_texture_source": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_client_texture_source=sampled-wl-shm-contract")",
    "renderer_gpu_client_texture_count": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_client_texture_count=2")",
    "renderer_gpu_client_texture_bytes": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_client_texture_bytes=674816")",
    "renderer_gpu_client_textures_uploaded": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_client_textures_uploaded=true")",
    "renderer_gpu_client_textures_composited": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_client_textures_composited=true")",
    "renderer_gpu_client_not_live": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_client_live_wayland_session=false")",
    "renderer_gpu_frame_readback": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_frame_readback=true")",
    "renderer_gpu_frame_bytes": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_frame_bytes=307200")",
    "renderer_gpu_frame_checksum": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_frame_checksum=")",
    "renderer_gpu_frame_deterministic": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_frame_deterministic=true")",
    "renderer_gpu_context_destroyed": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_context_destroyed=true")",
    "renderer_gpu_offscreen_no_kms": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_kms_activated=false")",
    "renderer_gpu_offscreen_no_display": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_display_output_started=false")",
    "renderer_gpu_offscreen_recovery_safe": "$(contract_file_contains "${FBDEV_QEMU_LOG}" "gpu_recovery_safe=true")",
    "client_layer_pipeline": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "[AQUA-COMPOSITOR] stage=client-layer-pipeline status=ok")",
    "client_layer_source_plan": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "source_plan_ready=ok")",
    "client_layer_paint_plan": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "paint_plan_ready=ok")",
    "client_layer_raster": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "raster_ready=ok")",
    "client_layer_count": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "client_layer_count=2")",
    "client_layer_checksum": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "client_layer_checksum=")",
    "client_layer_sample_pixel": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "sample_pixel=")",
    "client_layer_sample_grid": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "sample_grid=")",
    "client_layer_buffer_snapshot": "$(contract_file_contains "${CONTRACT_DIR}/client-layer-pipeline-probe.txt" "buffer_snapshot_bytes=")",
    "xdg_shell_binding": "$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "[AQUA-COMPOSITOR] stage=xdg-shell-binding status=ok")",
    "xdg_shell_protocol": "$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "protocol=xdg_wm_base")",
    "xdg_shell_handler": "$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "handler_bound=ok")",
    "xdg_shell_global": "$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "global_created=ok")",
    "xdg_shell_toplevel_callbacks": "$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "toplevel_callbacks_bound=ok")",
    "xdg_shell_popup_callbacks": "$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "popup_callbacks_bound=ok")",
    "xdg_shell_no_real_client": "$(contract_file_contains "${CONTRACT_DIR}/xdg-shell-binding-probe.txt" "real_wayland_client_started=false")",
    "xdg_toplevel_client": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "[AQUA-COMPOSITOR] stage=xdg-toplevel-client status=ok")",
    "xdg_toplevel_client_connected": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_connected=ok")",
    "xdg_toplevel_client_inserted": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_inserted=ok")",
    "xdg_toplevel_client_registry": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "registry_bound=ok")",
    "xdg_toplevel_client_globals": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "xdg_wm_base_global_seen=ok")",
    "xdg_toplevel_shm_global": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_global_seen=ok")",
    "xdg_toplevel_shm_buffer": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_buffer_created=ok")",
    "xdg_toplevel_client_buffer_attach": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_buffer_attached=ok")",
    "xdg_toplevel_surface": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "surface_created=ok")",
    "xdg_toplevel_request": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "toplevel_requested=ok")",
    "xdg_toplevel_server_buffer_attach": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_buffer_attached=ok")",
    "xdg_toplevel_shm_import": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_shm_buffer_imported=ok")",
    "xdg_toplevel_shm_sample": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_shm_buffer_sampled=ok")",
    "xdg_toplevel_shm_sample_pixel": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_sample_pixel=")",
    "xdg_toplevel_shm_sample_grid": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_sample_grid=")",
    "xdg_toplevel_shm_buffer_snapshot": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "shm_buffer_snapshot_bytes=")",
    "xdg_toplevel_server_created": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_toplevel_created=ok")",
    "xdg_toplevel_no_boot_graphics": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "boot_graphics=false")",
    "xdg_toplevel_configure_ack": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "server_configure_ack_received=ok")",
    "xdg_toplevel_close_event": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "client_close_event_received=ok")",
    "xdg_toplevel_client_count": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt" "test_wayland_client_count=2")",
    "xdg_toplevel_window_model": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "[AQUA-COMPOSITOR] stage=xdg-toplevel-window-model status=ok")",
    "xdg_toplevel_window_source": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "source_client_ready=ok")",
    "xdg_toplevel_window_surface_bound": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "server_surface_bound=ok")",
    "xdg_toplevel_window_model_bound": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "window_model_bound=ok")",
    "xdg_toplevel_window_count": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "window_count=2")",
    "xdg_toplevel_window_two_model": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "two_window_model_ready=ok")",
    "xdg_toplevel_window_stacking": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "stacking_ready=ok")",
    "xdg_toplevel_window_chrome": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "chrome_ready=ok")",
    "xdg_toplevel_window_no_render": "$(contract_file_contains "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt" "renderer_started=false")",
    "smithay_launcher_seat": "$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "[AQUA-COMPOSITOR] stage=smithay-launcher-seat status=ok")",
    "smithay_launcher_seat_global": "$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "seat_global_created=true")",
    "smithay_launcher_seat_input": "$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "pointer_button_dispatched=true")",
    "smithay_launcher_seat_rootfs": "$(contract_file_contains "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt" "host_stub=false")",
    "scene_status": "$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "scene_status=static-shell-model")",
    "required_surfaces": "$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "required_surfaces=7")",
    "runtime_asset_bindings": "$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "required_assets_present=ok")",
    "system_surface_token_bindings": "$(contract_file_contains "${CONTRACT_DIR}/scene-probe.txt" "required_material_tokens_present=ok")",
    "renderer": "$(contract_file_contains "${CONTRACT_DIR}/status.txt" "renderer=aqua-renderer")",
    "render_plan": "$(contract_file_contains "${CONTRACT_DIR}/render-plan-probe.txt" "[AQUA-COMPOSITOR] stage=render-plan-probe status=ok")",
    "renderer_started": "$(contract_file_contains "${CONTRACT_DIR}/render-plan-probe.txt" "renderer_started=false")",
    "paint_plan": "$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "[AQUA-COMPOSITOR] stage=paint-plan-probe status=ok")",
    "paint_steps": "$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "paint_step_count=7")",
    "paint_order": "$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "paint_order_stable=ok")",
    "paint_surface": "$(contract_file_contains "${CONTRACT_DIR}/paint-plan-probe.txt" "system_surface_steps_translucent=ok")",
    "frame_plan": "$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "[AQUA-COMPOSITOR] stage=frame-plan-probe status=ok")",
    "frame_format": "$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "pixel_format=rgba8888")",
    "frame_stride": "$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "stride_ready=ok")",
    "frame_damage": "$(contract_file_contains "${CONTRACT_DIR}/frame-plan-probe.txt" "damage_ready=ok")",
    "frame_buffer": "$(contract_file_contains "${CONTRACT_DIR}/frame-buffer-probe.txt" "[AQUA-COMPOSITOR] stage=frame-buffer-probe status=ok")",
    "frame_buffer_bytes": "$(contract_file_contains "${CONTRACT_DIR}/frame-buffer-probe.txt" "allocated_bytes=6291456")",
    "frame_buffer_clear": "$(contract_file_contains "${CONTRACT_DIR}/frame-buffer-probe.txt" "first_pixel=00,17,25,ff")",
    "raster": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "[AQUA-COMPOSITOR] stage=raster-probe status=ok")",
    "raster_rects": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "filled_rect_count=7")",
    "raster_wallpaper_sample": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "wallpaper_sample=04,3b,5c,ff")",
    "raster_surface_sample": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_sample=84,e0,ff,ff")",
    "raster_surface_border_sample": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_border_sample=3d,72,8c,ff")",
    "raster_surface_highlight_sample": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_highlight_sample=be,ef,ff,ff")",
    "raster_surface_corner_sample": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_corner_sample=2a,6c,8c,ff")",
    "raster_surface_shadow_sample": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_shadow_sample=52,a6,c6,ff")",
    "surface_primitives": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "surface_primitive_count=15")",
    "raster_checksum": "$(contract_file_contains "${CONTRACT_DIR}/raster-probe.txt" "raster_checksum=717b7e2c50c329f1")",
    "raster_export": "$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "[AQUA-COMPOSITOR] stage=raster-export-probe status=ok")",
    "raster_export_format": "$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "export_format=ppm-p6-rgb888")",
    "raster_export_bytes": "$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "export_bytes=4718609")",
    "raster_export_checksum": "$(contract_file_contains "${CONTRACT_DIR}/raster-export-probe.txt" "export_checksum=553f5b2626c15af1")",
    "raster_png_export": "$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "[AQUA-COMPOSITOR] stage=raster-png-export-probe status=ok")",
    "raster_png_export_format": "$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "export_format=png-rgba8888")",
    "raster_png_export_bytes": "$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "export_bytes=6293028")",
    "raster_png_export_checksum": "$(contract_file_contains "${CONTRACT_DIR}/raster-png-export-probe.txt" "export_checksum=1554b44a4319fe02")",
    "session_loop": "$(contract_file_contains "${CONTRACT_DIR}/session-loop.txt" "[AQUA-COMPOSITOR] stage=session-loop status=ok")",
    "desktop_shell": "not_started"
  },
  "boot_markers": {
    "rcS_start": "$(marker_status '[AQUA-BOOT] stage=rcS-start product="Aqua Linux"')",
    "filesystems_mounted": "$(marker_status '[AQUA-BOOT] stage=filesystems-mounted status=ok')",
    "fbdev_device": "$(marker_status '[AQUA-BOOT] stage=fbdev-device status=ok device=/dev/fb0 mode=')",
    "os_release": "$(marker_status '[AQUA-BOOT] stage=os-release id=aqua pretty="Aqua Linux Milestone 1"')",
    "session_config": "$(marker_status '[AQUA-BOOT] stage=session-config status=ok autostart=false boot_graphics=false recovery_tty=true')",
    "session_runtime": "$(marker_status '[AQUA-BOOT] stage=session-runtime status=ok user=aqua uid=1000 runtime_dir=/run/user/1000 control_dir=/run/aqua mode=0700')",
    "session_env": "$(marker_status '[AQUA-BOOT] stage=session-env status=ok wayland=aqua-wayland-0 xdg=/run/user/1000 assets=/usr/share/aqua')",
    "runtime_assets_ready": "$(marker_status '[AQUA-BOOT] stage=runtime-assets-ready milestone=2 status=ok')",
    "compositor_binary": "$(marker_status '[AQUA-BOOT] stage=compositor-binary status=packaged autostart=false boot_graphics=false')",
    "compositor_status": "$(marker_status '[AQUA-BOOT] stage=compositor-status status=ok mode=nested-dev')",
    "session_bootstrap": "$(marker_status '[AQUA-BOOT] stage=session-bootstrap status=ok runtime_dir=/run/user/1000 autostart=false boot_graphics=false session_started=false')",
    "compositor_assets": "$(marker_status '[AQUA-BOOT] stage=compositor-assets status=ok root=/usr/share/aqua')",
    "output_plan": "$(marker_status '[AQUA-BOOT] stage=output-plan status=ok backend=nested-dev-window boot_graphics=false renderer_started=false')",
    "visible_preview_plan": "$(marker_status '[AQUA-BOOT] stage=visible-preview-plan status=ok preview_window_started=false boot_graphics=false renderer_started=false')",
    "scene_contract": "$(marker_status '[AQUA-BOOT] stage=scene-contract status=ok surfaces=7 boot_graphics=false')",
    "render_plan": "$(marker_status '[AQUA-BOOT] stage=render-plan status=ok commands=7 boot_graphics=false renderer_started=false')",
    "paint_plan": "$(marker_status '[AQUA-BOOT] stage=paint-plan status=ok steps=7 boot_graphics=false renderer_started=false')",
    "frame_plan": "$(marker_status '[AQUA-BOOT] stage=frame-plan status=ok format=rgba8888 boot_graphics=false renderer_started=false')",
    "frame_buffer": "$(marker_status '[AQUA-BOOT] stage=frame-buffer status=ok bytes=6291456 boot_graphics=false renderer_started=false')",
    "raster": "$(marker_status '[AQUA-BOOT] stage=raster status=ok rects=7 surface_layers=15 boot_graphics=false renderer_started=false')",
    "surface_primitives": "$(marker_status '[AQUA-BOOT] stage=surface-primitives status=ok layers=15 boot_graphics=false renderer_started=false')",
    "raster_export": "$(marker_status '[AQUA-BOOT] stage=raster-export status=ok bytes=4718609 boot_graphics=false renderer_started=false')",
    "raster_png_export": "$(marker_status '[AQUA-BOOT] stage=raster-png-export status=ok bytes=6293028 boot_graphics=false renderer_started=false')",
    "session_check": "$(marker_status '[AQUA-BOOT] stage=session-check status=ok no_graphics=true')",
    "recovery_ready": "$(marker_status '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh')"
  },
  "boot_summary": {
    "status": "$(boot_summary_status)",
    "fbdev_device": "$(boot_summary_stage_status fbdev-device)",
    "session_config": "$(boot_summary_stage_status session-config)",
    "session_runtime": "$(boot_summary_stage_status session-runtime)",
    "session_env": "$(boot_summary_stage_status session-env)",
    "session_bootstrap": "$(boot_summary_stage_status session-bootstrap)",
    "compositor_assets": "$(boot_summary_stage_status compositor-assets)",
    "output_plan": "$(boot_summary_stage_status output-plan)",
    "visible_preview_plan": "$(boot_summary_stage_status visible-preview-plan)",
    "scene_contract": "$(boot_summary_stage_status scene-contract)",
    "render_plan": "$(boot_summary_stage_status render-plan)",
    "paint_plan": "$(boot_summary_stage_status paint-plan)",
    "frame_plan": "$(boot_summary_stage_status frame-plan)",
    "frame_buffer": "$(boot_summary_stage_status frame-buffer)",
    "raster": "$(boot_summary_stage_status raster)",
    "surface_primitives": "$(boot_summary_stage_status surface-primitives)",
    "raster_export": "$(boot_summary_stage_status raster-export)",
    "raster_png_export": "$(boot_summary_stage_status raster-png-export)",
    "session_check": "$(boot_summary_stage_status session-check)",
    "recovery_ready": "$(boot_summary_stage_status recovery-ready)"
  }
}
EOF

echo "Aqua Linux JSON image manifest written: ${MANIFEST_JSON}"
