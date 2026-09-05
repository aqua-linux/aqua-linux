#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-component-wayland.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-component-wayland-monitor.sock}"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-240}"

SCREENSHOT_light="${SCREENSHOT_light:-${ROOT_DIR}/build/qemu-component-light.ppm}"
SCREENSHOT_light_PNG="${SCREENSHOT_light_PNG:-${ROOT_DIR}/build/qemu-component-light.png}"
SCREENSHOT_dark="${SCREENSHOT_dark:-${ROOT_DIR}/build/qemu-component-dark.ppm}"
SCREENSHOT_dark_PNG="${SCREENSHOT_dark_PNG:-${ROOT_DIR}/build/qemu-component-dark.png}"

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
"${ROOT_DIR}/scripts/check-component-wayland-qemu.exp" >/dev/null

need_marker() {
    grep -Fq "$1" "${SERIAL_LOG}" || {
        echo "Missing component QEMU marker: $1" >&2
        tail -n 120 "${SERIAL_LOG}" >&2 || true
        exit 1
    }
}

need_marker '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh'
need_marker 'aqua_component_acceptance_fixture_revision=aqua-component-fixtures-20'
need_marker 'aqua_component_acceptance_catalog=22'
need_marker 'aqua_component_acceptance_shared=22'
need_marker 'aqua_component_acceptance_ready=true'
need_marker 'component_wayland_surface_ready=true'
need_marker 'component_wayland_surface_size=1280x800'
need_marker 'component_wayland_catalog_count=22'
need_marker 'component_wayland_shared_primitive_count=22'
need_marker 'component_wayland_shell_chrome_visible=false'
need_marker 'session_scenario=component-acceptance'
need_marker 'component_wayland_client_started=true'
need_marker 'component_wayland_client_process_stopped=true'
need_marker '[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok'

for theme in Light Dark; do
    need_marker "aqua_component_acceptance_theme=${theme}"
done

python3 - "${SERIAL_LOG}" <<'PY'
import pathlib
import sys

text = pathlib.Path(sys.argv[1]).read_text(errors="replace").replace("\r", "")
for marker in (
    "aqua_component_acceptance_connected=true",
    "aqua_component_acceptance_ready=true",
    "component_wayland_surface_ready=true",
    "[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok",
):
    if text.count(marker) != 2:
        raise SystemExit(f"expected two occurrences of {marker!r}, got {text.count(marker)}")
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
    sample = pixels[::4096]
    if len(set(sample)) < 8:
        raise SystemExit(f"component screenshot appears blank: {path}")
    digests.add(hashlib.sha256(pixels).digest())
if len(digests) != 2:
    raise SystemExit("component theme captures are not visually distinct")
PY

echo "Aqua packaged component Wayland QEMU check passed."
echo "Screenshots: ${SCREENSHOT_light_PNG} ${SCREENSHOT_dark_PNG}"
