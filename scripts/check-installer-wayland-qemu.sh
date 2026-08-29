#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-installer-wayland.log}"
SCREENSHOT="${SCREENSHOT:-${ROOT_DIR}/build/qemu-installer-welcome.ppm}"
SCREENSHOT_PNG="${SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-installer-welcome.png}"
NAV_SCREENSHOT="${NAV_SCREENSHOT:-${ROOT_DIR}/build/qemu-installer-keyboard.ppm}"
NAV_SCREENSHOT_PNG="${NAV_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-installer-keyboard.png}"
PARTITIONS_SCREENSHOT="${PARTITIONS_SCREENSHOT:-${ROOT_DIR}/build/qemu-installer-partitions.ppm}"
PARTITIONS_SCREENSHOT_PNG="${PARTITIONS_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-installer-partitions.png}"
TIMEZONE_SCREENSHOT="${TIMEZONE_SCREENSHOT:-${ROOT_DIR}/build/qemu-installer-timezone.ppm}"
TIMEZONE_SCREENSHOT_PNG="${TIMEZONE_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-installer-timezone.png}"
SUMMARY_SCREENSHOT="${SUMMARY_SCREENSHOT:-${ROOT_DIR}/build/qemu-installer-summary.ppm}"
SUMMARY_SCREENSHOT_PNG="${SUMMARY_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-installer-summary.png}"
PROGRESS_SCREENSHOT="${PROGRESS_SCREENSHOT:-${ROOT_DIR}/build/qemu-installer-progress.ppm}"
PROGRESS_SCREENSHOT_PNG="${PROGRESS_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-installer-progress.png}"
COMPLETED_SCREENSHOT="${COMPLETED_SCREENSHOT:-${ROOT_DIR}/build/qemu-installer-completed.ppm}"
COMPLETED_SCREENSHOT_PNG="${COMPLETED_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-installer-completed.png}"
TARGET_DISK="${TARGET_DISK:-${ROOT_DIR}/build/qemu-installer-target.qcow2}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-installer-wayland-monitor.sock}"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-360}"

for tool in expect file python3 qemu-img qemu-system-x86_64; do
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
rm -f "${SERIAL_LOG}" "${SCREENSHOT}" "${SCREENSHOT_PNG}" \
    "${NAV_SCREENSHOT}" "${NAV_SCREENSHOT_PNG}" \
    "${PARTITIONS_SCREENSHOT}" "${PARTITIONS_SCREENSHOT_PNG}" \
    "${TIMEZONE_SCREENSHOT}" "${TIMEZONE_SCREENSHOT_PNG}" \
    "${SUMMARY_SCREENSHOT}" "${SUMMARY_SCREENSHOT_PNG}" \
    "${PROGRESS_SCREENSHOT}" "${PROGRESS_SCREENSHOT_PNG}" \
    "${COMPLETED_SCREENSHOT}" "${COMPLETED_SCREENSHOT_PNG}" \
    "${TARGET_DISK}" "${MONITOR_SOCKET}"
qemu-img create -q -f qcow2 "${TARGET_DISK}" 4G

export KERNEL ROOTFS SERIAL_LOG SCREENSHOT SCREENSHOT_PNG NAV_SCREENSHOT NAV_SCREENSHOT_PNG \
    PARTITIONS_SCREENSHOT PARTITIONS_SCREENSHOT_PNG TIMEZONE_SCREENSHOT \
    TIMEZONE_SCREENSHOT_PNG TARGET_DISK MONITOR_SOCKET \
    SUMMARY_SCREENSHOT SUMMARY_SCREENSHOT_PNG \
    PROGRESS_SCREENSHOT PROGRESS_SCREENSHOT_PNG \
    COMPLETED_SCREENSHOT COMPLETED_SCREENSHOT_PNG \
    CAPTURE_HELPER INPUT_HELPER MEMORY CPUS TIMEOUT_SECONDS
"${ROOT_DIR}/scripts/check-installer-wayland-qemu.exp" >/dev/null

need_marker() {
    if ! grep -Fq "$1" "${SERIAL_LOG}"; then
        echo "Missing installer QEMU marker: $1" >&2
        tail -n 120 "${SERIAL_LOG}" >&2 || true
        exit 1
    fi
}

need_regex() {
    if ! grep -Eq "$1" "${SERIAL_LOG}"; then
        echo "Missing installer QEMU marker regex: $1" >&2
        tail -n 120 "${SERIAL_LOG}" >&2 || true
        exit 1
    fi
}

need_marker '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh'
need_marker 'installer_wayland_client_process_started=true'
need_marker 'aqua_installer_connected=true'
need_marker 'aqua_installer_app_id=aqua.installer'
need_marker 'aqua_installer_step=welcome'
need_marker 'aqua_installer_live_input=true'
need_marker 'aqua_installer_execution_allowed=false'
need_marker 'aqua_installer_buffer=1280x800'
need_marker 'installer_wayland_surface_ready=true'
need_marker 'installer_wayland_surface_size=1280x800'
need_marker 'installer_wayland_surface_execution_allowed=false'
need_marker 'installer_wayland_shell_chrome_visible=false'
need_marker 'desktop_system_overview_visible=false'
need_marker 'aqua_installer_step=language'
need_regex 'aqua_installer_pointer x=[0-9]+ y=[0-9]+ action=AdvanceRequested'
need_marker 'aqua_installer_pointer_form_update=SelectionChanged { step: Language, index: 0, value: "tr_TR.UTF-8" }'
need_regex 'aqua_installer_pointer x=[0-9]+ y=[0-9]+ action=FocusChanged\(StepContent\) content=true'
need_marker 'aqua_installer_locale=tr_TR.UTF-8'
need_marker 'aqua_installer_step=keyboard'
need_marker 'aqua_installer_keyboard_layout=trq'
need_marker 'aqua_installer_storage_candidate_count=2'
need_marker 'aqua_installer_storage_eligible_count=1'
need_marker 'aqua_installer_storage_candidate=/dev/vda eligible=false blocked_reasons=running-system-disk'
need_marker 'aqua_installer_storage_candidate=/dev/vdb eligible=true blocked_reasons=none'
need_marker 'aqua_installer_step=partitions'
need_marker 'aqua_installer_target_device=/dev/vdb'
need_marker 'aqua_installer_step=time-zone'
need_marker 'aqua_installer_timezone=Europe/Istanbul'
need_marker 'aqua_installer_step=user-information'
need_marker 'aqua_installer_user_profile username=aqua display_name=user password_configured=true'
need_marker 'aqua_installer_step=summary'
need_marker 'aqua_installer_summary_destructive_acknowledgement=true'
need_marker 'aqua_installer_summary_confirmation_applied=true'
need_marker 'aqua_installer_summary_ready=true'
need_marker 'aqua_installer_summary_target_device=/dev/vdb'
need_marker 'aqua_installer_step=installation'
need_marker 'aqua_installer_progress_presentation_rehearsal=true'
need_marker '[AQUA-INSTALLER-PROGRESS] state=running'
need_marker 'completed=8 total=20 percent=40'
need_marker 'completed=13 total=20 percent=65'
need_marker 'completed=19 total=20 percent=95'
need_marker '[AQUA-INSTALLER-PROGRESS] state=completed phase=completed operation=complete completed=20 total=20 percent=100'
need_marker 'aqua_installer_step=completed'
need_marker 'aqua_installer_transaction_executed=false'
need_marker 'aqua_installer_presentation_rehearsal_completed=true'
need_marker 'aqua_installer_redraw_count=35'
need_marker 'installer_wayland_input_sequence_complete=true'
need_marker 'installer_wayland_repaint=true'
need_marker 'gpu_native_opaque_direct_bridge=true'
need_marker 'session_scenario=installer-welcome'
need_marker '[AQUA-COMPOSITOR] stage=drm-wayland-session status=active'
need_marker 'installer_wayland_client_process_stopped=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok'

if grep -Fq 'password=' "${SERIAL_LOG}"; then
    echo "Installer serial log unexpectedly contains password content" >&2
    exit 1
fi

python3 - "${SERIAL_LOG}" <<'PY'
import pathlib
import sys

lines = pathlib.Path(sys.argv[1]).read_text(errors="replace").replace("\r", "").splitlines()
direct_totals = []
direct_frame = False
for line in lines:
    if line == "gpu_native_opaque_direct_bridge=true":
        direct_frame = True
    elif direct_frame and line.startswith("gpu_native_frame_total_ms="):
        direct_totals.append(int(line.split("=", 1)[1]))
        direct_frame = False

if len(direct_totals) < 10:
    raise SystemExit(f"insufficient opaque direct-bridge frames: {len(direct_totals)}")
if max(direct_totals) > 500:
    raise SystemExit(f"opaque direct-bridge frame exceeded 500 ms: {max(direct_totals)}")
PY

file "${SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'
file "${NAV_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'
file "${PARTITIONS_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'
file "${TIMEZONE_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'
file "${SUMMARY_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'
file "${PROGRESS_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'
file "${COMPLETED_SCREENSHOT_PNG}" | grep -Fq 'PNG image data, 1280 x 800'
if cmp -s "${SCREENSHOT_PNG}" "${NAV_SCREENSHOT_PNG}" || \
    cmp -s "${NAV_SCREENSHOT_PNG}" "${PARTITIONS_SCREENSHOT_PNG}" || \
    cmp -s "${PARTITIONS_SCREENSHOT_PNG}" "${TIMEZONE_SCREENSHOT_PNG}" || \
    cmp -s "${TIMEZONE_SCREENSHOT_PNG}" "${SUMMARY_SCREENSHOT_PNG}" || \
    cmp -s "${SUMMARY_SCREENSHOT_PNG}" "${PROGRESS_SCREENSHOT_PNG}" || \
    cmp -s "${PROGRESS_SCREENSHOT_PNG}" "${COMPLETED_SCREENSHOT_PNG}"; then
    echo "Installer navigation captures are unexpectedly identical" >&2
    exit 1
fi
python3 - "${SCREENSHOT}" "${NAV_SCREENSHOT}" "${PARTITIONS_SCREENSHOT}" \
    "${TIMEZONE_SCREENSHOT}" "${SUMMARY_SCREENSHOT}" \
    "${PROGRESS_SCREENSHOT}" "${COMPLETED_SCREENSHOT}" <<'PY'
import pathlib
import sys

for path in sys.argv[1:]:
    data = pathlib.Path(path).read_bytes()
    header_end = data.find(b"\n255\n")
    if header_end < 0:
        raise SystemExit(f"invalid QEMU PPM: {path}")
    pixels = data[header_end + 5:]
    sample = pixels[::4096]
    if len(set(sample)) < 8:
        raise SystemExit(f"installer screenshot appears blank: {path}")
    if sum(sample) / len(sample) < 80:
        raise SystemExit(f"installer screenshot appears to show the dark recovery TTY: {path}")
PY

echo "Aqua Installer packaged Wayland QEMU check passed."
echo "Screenshots: ${SCREENSHOT_PNG} ${NAV_SCREENSHOT_PNG} ${PARTITIONS_SCREENSHOT_PNG} ${TIMEZONE_SCREENSHOT_PNG} ${SUMMARY_SCREENSHOT_PNG} ${PROGRESS_SCREENSHOT_PNG} ${COMPLETED_SCREENSHOT_PNG}"
