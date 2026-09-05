#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-icon-wayland.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-icon-wayland-monitor.sock}"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"

SCREENSHOT_light="${SCREENSHOT_light:-${ROOT_DIR}/build/qemu-icon-light.ppm}"
SCREENSHOT_light_PNG="${SCREENSHOT_light_PNG:-${ROOT_DIR}/build/qemu-icon-light.png}"
SCREENSHOT_dark="${SCREENSHOT_dark:-${ROOT_DIR}/build/qemu-icon-dark.ppm}"
SCREENSHOT_dark_PNG="${SCREENSHOT_dark_PNG:-${ROOT_DIR}/build/qemu-icon-dark.png}"

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
    "${SCREENSHOT_light}" "${SCREENSHOT_light_PNG}" \
    "${SCREENSHOT_dark}" "${SCREENSHOT_dark_PNG}"


export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET CAPTURE_HELPER MEMORY CPUS TIMEOUT_SECONDS
export SCREENSHOT_light SCREENSHOT_light_PNG
export SCREENSHOT_dark SCREENSHOT_dark_PNG
"${ROOT_DIR}/scripts/check-icon-wayland-qemu.exp" >/dev/null

need_marker() {
    grep -Fq "$1" "${SERIAL_LOG}" || {
        echo "Missing icon QEMU marker: $1" >&2
        tail -n 120 "${SERIAL_LOG}" >&2 || true
        exit 1
    }
}

need_marker '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh'
need_marker 'icon_wayland_raster_cache_ready=true'
need_marker 'icon_wayland_raster_roles=7'
need_marker 'icon_wayland_raster_surfaces=3'
need_marker 'desktop_icon_rasters_ready=true surface=top-bar roles=3'
need_marker 'desktop_icon_rasters_ready=true surface=desktop roles=3'
need_marker 'desktop_icon_rasters_ready=true surface=dock roles=0'
need_marker 'desktop_icon_rasters_ready=true surface=notification roles=1'
need_marker 'desktop_icon_raster_cache_entries=7'
need_marker 'desktop_icon_raster_cache_hits=0'
need_marker 'desktop_icon_raster_cache_misses=7'
need_marker 'desktop_icon_raster_cache_parsed_sources=7'
need_marker 'desktop_icon_raster_cache_evictions=0'
need_marker 'session_scenario=icon-acceptance'
need_marker 'external_fixture_clients_started=false'
need_marker 'icon_wayland_scenario_started=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok'

python3 - "${SERIAL_LOG}" <<'PY'
import pathlib
import sys

text = pathlib.Path(sys.argv[1]).read_text(errors="replace").replace("\r", "")
for marker, expected in (
    ("icon_wayland_raster_cache_ready=true", 2),
    ("icon_wayland_raster_roles=7", 2),
    ("icon_wayland_raster_surfaces=3", 2),
    ("desktop_icon_rasters_ready=true surface=top-bar roles=3", 2),
    ("desktop_icon_rasters_ready=true surface=desktop roles=3", 2),
    ("desktop_icon_rasters_ready=true surface=dock roles=0", 4),
    ("desktop_icon_rasters_ready=true surface=notification roles=1", 2),
    ("desktop_icon_raster_cache_hits=0", 2),
    ("desktop_icon_raster_cache_misses=7", 4),
    ("desktop_icon_raster_cache_parsed_sources=7", 4),
    ("session_scenario=icon-acceptance", 2),
    ("[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok", 2),
):
    actual = text.count(marker)
    if actual != expected:
        raise SystemExit(f"expected {expected} occurrences of {marker!r}, got {actual}")
PY

for png in "${SCREENSHOT_light_PNG}" "${SCREENSHOT_dark_PNG}"; do
    file "${png}" | grep -Fq 'PNG image data, 1280 x 800'
done

python3 - \
    "${SCREENSHOT_light}" "${SCREENSHOT_dark}" <<'PY'
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
    if len(pixels) != 1280 * 800 * 3:
        raise SystemExit(f"unexpected QEMU capture size: {path}")
    if len(set(pixels[::4096])) < 8:
        raise SystemExit(f"icon screenshot appears blank: {path}")
    desktop_crop = bytearray()
    for y in range(40, 300):
        start = (y * 1280 + 12) * 3
        desktop_crop.extend(pixels[start:start + 190 * 3])
    if len(set(desktop_crop[::97])) < 12:
        raise SystemExit(f"desktop icon crop lacks visual detail: {path}")
    digests.add(hashlib.sha256(pixels).digest())
if len(digests) != 2:
    raise SystemExit("icon theme captures are not visually distinct")
PY

echo "Aqua packaged icon Wayland QEMU check passed."
echo "Screenshots: ${SCREENSHOT_light_PNG} ${SCREENSHOT_dark_PNG}"
