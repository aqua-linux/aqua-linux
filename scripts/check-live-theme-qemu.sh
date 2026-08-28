#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-live-theme-check.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-live-theme-monitor.sock}"
INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET:-${ROOT_DIR}/build/qemu-live-theme-input.sock}"
INPUT_DAEMON_LOG="${INPUT_DAEMON_LOG:-${ROOT_DIR}/build/qemu-live-theme-input.log}"
LIGHT_SCREENSHOT="${LIGHT_SCREENSHOT:-${ROOT_DIR}/build/qemu-theme-lightwhite.ppm}"
LIGHT_SCREENSHOT_PNG="${LIGHT_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-theme-lightwhite.png}"
DEEP_SCREENSHOT="${DEEP_SCREENSHOT:-${ROOT_DIR}/build/qemu-theme-deepside.ppm}"
DEEP_SCREENSHOT_PNG="${DEEP_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-theme-deepside.png}"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
THEME_FRAME_CHECK="${ROOT_DIR}/scripts/check-qemu-theme-frame-delta.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"
INPUT_DAEMON_PID=""

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

mkdir -p "$(dirname "${SERIAL_LOG}")"
rm -f "${SERIAL_LOG}" "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}" \
    "${INPUT_DAEMON_LOG}" "${LIGHT_SCREENSHOT}" "${LIGHT_SCREENSHOT_PNG}" \
    "${DEEP_SCREENSHOT}" "${DEEP_SCREENSHOT_PNG}"

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
export INPUT_HELPER CAPTURE_HELPER THEME_FRAME_CHECK
export LIGHT_SCREENSHOT LIGHT_SCREENSHOT_PNG DEEP_SCREENSHOT DEEP_SCREENSHOT_PNG
export AQUA_QEMU_INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET}"
expect "${ROOT_DIR}/scripts/check-live-theme-qemu.exp"

grep -Fq '[AQUA-TEST] stage=desktop-live-theme-qemu status=ok from=LightWhite to=Deepside shell=true apps=files,settings restart=false frame_delta=true' "${SERIAL_LOG}"
test -s "${LIGHT_SCREENSHOT_PNG}"
test -s "${DEEP_SCREENSHOT_PNG}"

echo "Aqua Linux live theme QEMU check passed."
echo "Serial log: ${SERIAL_LOG}"
echo "LightWhite screenshot: ${LIGHT_SCREENSHOT_PNG}"
echo "Deepside screenshot: ${DEEP_SCREENSHOT_PNG}"
