#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
OUTPUT_DIR="${OUTPUT_DIR:-${ROOT_DIR}/build/qemu-public-runtime}"
SERIAL_LOG="${SERIAL_LOG:-${OUTPUT_DIR}/serial.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-public-runtime-monitor.sock}"
INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET:-${ROOT_DIR}/build/qemu-public-runtime-input.sock}"
INPUT_DAEMON_LOG="${INPUT_DAEMON_LOG:-${OUTPUT_DIR}/input.log}"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"
INPUT_DAEMON_PID=""

DESKTOP_PPM="${OUTPUT_DIR}/desktop.ppm"
DESKTOP_PNG="${OUTPUT_DIR}/desktop.png"
APPLICATIONS_PPM="${OUTPUT_DIR}/applications.ppm"
APPLICATIONS_PNG="${OUTPUT_DIR}/applications.png"
SEARCH_PPM="${OUTPUT_DIR}/search.ppm"
SEARCH_PNG="${OUTPUT_DIR}/search.png"
WINDOWS_PPM="${OUTPUT_DIR}/windows.ppm"
WINDOWS_PNG="${OUTPUT_DIR}/windows.png"

cleanup() {
    if [ -n "${INPUT_DAEMON_PID}" ]; then
        kill "${INPUT_DAEMON_PID}" 2>/dev/null || true
        wait "${INPUT_DAEMON_PID}" 2>/dev/null || true
    fi
    rm -f "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}"
}
trap cleanup EXIT INT TERM

for tool in expect python3 qemu-system-x86_64; do
    command -v "${tool}" >/dev/null 2>&1 || {
        echo "Missing required tool: ${tool}" >&2
        exit 1
    }
done
for artifact in "${KERNEL}" "${ROOTFS}"; do
    test -f "${artifact}" || {
        echo "Missing QEMU artifact: ${artifact}" >&2
        exit 1
    }
done

mkdir -p "${OUTPUT_DIR}"
rm -f "${OUTPUT_DIR}"/*.ppm "${OUTPUT_DIR}"/*.png "${SERIAL_LOG}" \
    "${INPUT_DAEMON_LOG}" "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}"

python3 "${INPUT_HELPER}" --serve "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}" >"${INPUT_DAEMON_LOG}" 2>&1 &
INPUT_DAEMON_PID=$!
i=0
while [ "${i}" -lt 100 ] && [ ! -S "${INPUT_CONTROL_SOCKET}" ]; do
    kill -0 "${INPUT_DAEMON_PID}" 2>/dev/null || {
        cat "${INPUT_DAEMON_LOG}" >&2
        exit 1
    }
    sleep 0.1
    i=$((i + 1))
done
test -S "${INPUT_CONTROL_SOCKET}"

export ROOT_DIR KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET MEMORY CPUS TIMEOUT_SECONDS
export INPUT_HELPER CAPTURE_HELPER DESKTOP_PPM DESKTOP_PNG APPLICATIONS_PPM
export APPLICATIONS_PNG SEARCH_PPM SEARCH_PNG WINDOWS_PPM WINDOWS_PNG
export AQUA_QEMU_INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET}"
expect "${ROOT_DIR}/scripts/check-public-runtime-qemu.exp"

grep -Fq '[AQUA-TEST] stage=desktop-public-runtime-qemu status=ok captures=desktop,applications,search,windows clients=files,settings' "${SERIAL_LOG}"
python3 - "${DESKTOP_PNG}" "${APPLICATIONS_PNG}" "${SEARCH_PNG}" "${WINDOWS_PNG}" <<'PY'
import struct
import sys
import zlib

def png_pixels(path):
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"not a PNG: {path}")
    offset = 8
    chunks = []
    width = height = color_type = None
    while offset < len(data):
        length = struct.unpack(">I", data[offset:offset + 4])[0]
        kind = data[offset + 4:offset + 8]
        payload = data[offset + 8:offset + 8 + length]
        offset += 12 + length
        if kind == b"IHDR":
            width, height, depth, color_type = struct.unpack(">IIBB", payload[:10])
            if depth != 8 or color_type not in (2, 6):
                raise SystemExit(f"unsupported PNG format: {path}")
        elif kind == b"IDAT":
            chunks.append(payload)
        elif kind == b"IEND":
            break
    if (width, height) != (1280, 800):
        raise SystemExit(f"unexpected screenshot size {width}x{height}: {path}")
    channels = 3 if color_type == 2 else 4
    raw = zlib.decompress(b"".join(chunks))
    stride = width * channels
    rows = []
    previous = bytearray(stride)
    cursor = 0
    for _ in range(height):
        filter_type = raw[cursor]
        row = bytearray(raw[cursor + 1:cursor + 1 + stride])
        cursor += stride + 1
        for index in range(stride):
            left = row[index - channels] if index >= channels else 0
            up = previous[index]
            upper_left = previous[index - channels] if index >= channels else 0
            if filter_type == 1:
                row[index] = (row[index] + left) & 255
            elif filter_type == 2:
                row[index] = (row[index] + up) & 255
            elif filter_type == 3:
                row[index] = (row[index] + ((left + up) // 2)) & 255
            elif filter_type == 4:
                estimate = left + up - upper_left
                pa, pb, pc = abs(estimate-left), abs(estimate-up), abs(estimate-upper_left)
                predictor = left if pa <= pb and pa <= pc else up if pb <= pc else upper_left
                row[index] = (row[index] + predictor) & 255
            elif filter_type != 0:
                raise SystemExit(f"unsupported PNG filter: {path}")
        rows.append(bytes(row))
        previous = row
    pixels = b"".join(rows)
    if len(set(pixels[::channels])) < 32:
        raise SystemExit(f"screenshot appears blank: {path}")
    return pixels

frames = [png_pixels(path) for path in sys.argv[1:]]
for left, right in zip(frames, frames[1:]):
    changed = sum(a != b for a, b in zip(left, right))
    if changed < len(left) // 100:
        raise SystemExit("adjacent public runtime captures are unexpectedly similar")
print("Public runtime PNG validation passed.")
PY

echo "Aqua Linux public runtime QEMU capture check passed."
echo "Captures: ${DESKTOP_PNG} ${APPLICATIONS_PNG} ${SEARCH_PNG} ${WINDOWS_PNG}"
