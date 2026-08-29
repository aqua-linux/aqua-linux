#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-elevation-wayland.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-elevation-wayland-monitor.sock}"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"

SCREENSHOT_lightwhite="${SCREENSHOT_lightwhite:-${ROOT_DIR}/build/qemu-elevation-lightwhite.ppm}"
SCREENSHOT_lightwhite_PNG="${SCREENSHOT_lightwhite_PNG:-${ROOT_DIR}/build/qemu-elevation-lightwhite.png}"
SCREENSHOT_softtouch="${SCREENSHOT_softtouch:-${ROOT_DIR}/build/qemu-elevation-softtouch.ppm}"
SCREENSHOT_softtouch_PNG="${SCREENSHOT_softtouch_PNG:-${ROOT_DIR}/build/qemu-elevation-softtouch.png}"
SCREENSHOT_deepside="${SCREENSHOT_deepside:-${ROOT_DIR}/build/qemu-elevation-deepside.ppm}"
SCREENSHOT_deepside_PNG="${SCREENSHOT_deepside_PNG:-${ROOT_DIR}/build/qemu-elevation-deepside.png}"
SCREENSHOT_nightmare="${SCREENSHOT_nightmare:-${ROOT_DIR}/build/qemu-elevation-nightmare.ppm}"
SCREENSHOT_nightmare_PNG="${SCREENSHOT_nightmare_PNG:-${ROOT_DIR}/build/qemu-elevation-nightmare.png}"

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
    "${SCREENSHOT_lightwhite}" "${SCREENSHOT_lightwhite_PNG}" \
    "${SCREENSHOT_softtouch}" "${SCREENSHOT_softtouch_PNG}" \
    "${SCREENSHOT_deepside}" "${SCREENSHOT_deepside_PNG}" \
    "${SCREENSHOT_nightmare}" "${SCREENSHOT_nightmare_PNG}"

export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET CAPTURE_HELPER MEMORY CPUS TIMEOUT_SECONDS
export SCREENSHOT_lightwhite SCREENSHOT_lightwhite_PNG
export SCREENSHOT_softtouch SCREENSHOT_softtouch_PNG
export SCREENSHOT_deepside SCREENSHOT_deepside_PNG
export SCREENSHOT_nightmare SCREENSHOT_nightmare_PNG
"${ROOT_DIR}/scripts/check-elevation-wayland-qemu.exp" >/dev/null

need_marker() {
    grep -Fq "$1" "${SERIAL_LOG}" || {
        echo "Missing elevation QEMU marker: $1" >&2
        tail -n 120 "${SERIAL_LOG}" >&2 || true
        exit 1
    }
}

need_marker '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh'
need_marker 'elevation_wayland_surface_ready=true'
need_marker 'elevation_wayland_surface_count=2'
need_marker 'elevation_wayland_focused_surface_count=1'
need_marker 'elevation_wayland_inactive_surface_count=1'
need_marker 'gpu_shadow_mask_cache_misses=2'
need_marker 'gpu_shadow_damage_rects=2'
need_marker 'drm_wayland_gpu_client_texture_count=2'
need_marker 'session_scenario=elevation-acceptance'
need_marker 'external_fixture_clients_started=true'
need_marker 'elevation_wayland_client_started=true'
need_marker 'elevation_wayland_client_process_stopped=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok'

python3 - "${SERIAL_LOG}" <<'PY'
import pathlib
import sys

text = pathlib.Path(sys.argv[1]).read_text(errors="replace").replace("\r", "")
for marker, expected in (
    ("elevation_wayland_surface_ready=true", 4),
    ("elevation_wayland_focused_surface_count=1", 4),
    ("elevation_wayland_inactive_surface_count=1", 4),
    ("gpu_shadow_mask_cache_misses=2", 4),
    ("gpu_shadow_damage_rects=2", 4),
    ("elevation_wayland_client_process_stopped=true", 8),
    ("[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok", 4),
):
    actual = text.count(marker)
    if actual != expected:
        raise SystemExit(f"expected {expected} occurrences of {marker!r}, got {actual}")
PY

for png in "${SCREENSHOT_lightwhite_PNG}" "${SCREENSHOT_softtouch_PNG}" \
    "${SCREENSHOT_deepside_PNG}" "${SCREENSHOT_nightmare_PNG}"; do
    file "${png}" | grep -Fq 'PNG image data, 1280 x 800'
done

python3 - \
    "${SCREENSHOT_lightwhite}" "${SCREENSHOT_softtouch}" \
    "${SCREENSHOT_deepside}" "${SCREENSHOT_nightmare}" <<'PY'
import hashlib
import pathlib
import sys

digests = set()
for raw in sys.argv[1:]:
    path = pathlib.Path(raw)
    data = path.read_bytes()
    header_end = data.find(b"\n255\n")
    if header_end < 0:
        raise SystemExit(f"invalid QEMU PPM: {path}")
    pixels = data[header_end + 5:]
    if len(set(pixels[::4096])) < 8:
        raise SystemExit(f"elevation screenshot appears blank: {path}")
    digests.add(hashlib.sha256(pixels).digest())
if len(digests) != 4:
    raise SystemExit("elevation theme captures are not visually distinct")
PY

echo "Aqua packaged elevation Wayland QEMU check passed."
echo "Screenshots: ${SCREENSHOT_lightwhite_PNG} ${SCREENSHOT_softtouch_PNG} ${SCREENSHOT_deepside_PNG} ${SCREENSHOT_nightmare_PNG}"
