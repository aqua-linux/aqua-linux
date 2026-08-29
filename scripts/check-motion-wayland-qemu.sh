#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-motion-wayland.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-motion-wayland-monitor.sock}"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
SCREENSHOT_standard="${SCREENSHOT_standard:-${ROOT_DIR}/build/qemu-motion-standard.ppm}"
SCREENSHOT_standard_PNG="${SCREENSHOT_standard_PNG:-${ROOT_DIR}/build/qemu-motion-standard.png}"
SCREENSHOT_reduced="${SCREENSHOT_reduced:-${ROOT_DIR}/build/qemu-motion-reduced.ppm}"
SCREENSHOT_reduced_PNG="${SCREENSHOT_reduced_PNG:-${ROOT_DIR}/build/qemu-motion-reduced.png}"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-360}"

for tool in expect file python3 qemu-system-x86_64; do
    command -v "${tool}" >/dev/null 2>&1 || {
        echo "Missing required tool: ${tool}" >&2
        exit 1
    }
done
for artifact in "${KERNEL}" "${ROOTFS}"; do
    [ -f "${artifact}" ] || {
        echo "Missing QEMU artifact: ${artifact}" >&2
        exit 1
    }
done

mkdir -p "$(dirname "${SERIAL_LOG}")"
rm -f "${SERIAL_LOG}" "${MONITOR_SOCKET}" \
    "${SCREENSHOT_standard}" "${SCREENSHOT_standard_PNG}" \
    "${SCREENSHOT_reduced}" "${SCREENSHOT_reduced_PNG}"

export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET CAPTURE_HELPER MEMORY CPUS TIMEOUT_SECONDS
export SCREENSHOT_standard SCREENSHOT_standard_PNG SCREENSHOT_reduced SCREENSHOT_reduced_PNG
"${ROOT_DIR}/scripts/check-motion-wayland-qemu.exp" >/dev/null

for marker in \
    '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh' \
    'session_scenario=motion-acceptance' \
    'motion_wayland_scenario_started=true' \
    'motion_wayland_launcher_target=open' \
    'motion_wayland_launcher_target=closed' \
    'motion_wayland_launcher_target=reopened' \
    'motion_wayland_notification_target=visible' \
    'motion_wayland_notification_target=hidden' \
    'motion_wayland_runtime_settled=true' \
    'motion_wayland_sequence_complete=true'; do
    grep -Fq "${marker}" "${SERIAL_LOG}" || {
        echo "Missing motion QEMU marker: ${marker}" >&2
        tail -n 160 "${SERIAL_LOG}" >&2 || true
        exit 1
    }
done

python3 - "${SERIAL_LOG}" <<'PY'
import pathlib
import re
import sys

text = pathlib.Path(sys.argv[1]).read_text(errors="replace").replace("\r", "")
if text.count("session_scenario=motion-acceptance") != 2:
    raise SystemExit("motion scenario did not run exactly twice")
if text.count("motion_wayland_sequence_complete=true") != 2:
    raise SystemExit("motion sequences did not both complete")

pattern = re.compile(
    r"motion_wayland_frame time_ms=(\d+) active=(true|false) "
    r"launcher_opacity=([0-9.]+) launcher_offset_y=(-?\d+) "
    r"notification_opacity=([0-9.]+) notification_offset_x=(-?\d+) "
    r"reduced_motion=(true|false)"
)
frames = [
    (int(t), active == "true", float(lo), int(ly), float(no), int(nx), reduced == "true")
    for t, active, lo, ly, no, nx, reduced in pattern.findall(text)
]
standard = [frame for frame in frames if not frame[-1]]
reduced = [frame for frame in frames if frame[-1]]
if len(standard) < 5:
    raise SystemExit(f"expected at least 5 standard motion frames, got {len(standard)}")
if not any(active and offset > 0 for _, active, _, offset, _, _, _ in standard):
    raise SystemExit("standard launcher did not schedule spatial frames")
if not any(active and offset > 0 for _, active, _, _, _, offset, _ in standard):
    raise SystemExit("standard notification did not schedule spatial frames")
if not reduced:
    raise SystemExit("reduced-motion frames are missing")
if any(active or launcher_offset or notification_offset for _, active, _, launcher_offset, _, notification_offset, _ in reduced):
    raise SystemExit("reduced-motion path scheduled animation or spatial travel")
PY

for png in "${SCREENSHOT_standard_PNG}" "${SCREENSHOT_reduced_PNG}"; do
    file "${png}" | grep -Fq 'PNG image data, 1280 x 800'
done

echo "Aqua Linux packaged motion Wayland checks passed."
