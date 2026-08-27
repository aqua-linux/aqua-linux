#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-terminal-check.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-terminal-monitor.sock}"
INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET:-${ROOT_DIR}/build/qemu-terminal-input.sock}"
INPUT_DAEMON_LOG="${INPUT_DAEMON_LOG:-${ROOT_DIR}/build/qemu-terminal-input.log}"
INPUT_DAEMON_PID=""
SCREENSHOT_PPM="${SCREENSHOT_PPM:-${ROOT_DIR}/build/qemu-aqua-terminal.ppm}"
SCREENSHOT_PNG="${SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-aqua-terminal.png}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"

cleanup() {
    if [ -n "${INPUT_DAEMON_PID}" ]; then
        kill "${INPUT_DAEMON_PID}" 2>/dev/null || true
        wait "${INPUT_DAEMON_PID}" 2>/dev/null || true
    fi
    rm -f "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}"
}
trap cleanup EXIT INT TERM
rm -f "${SERIAL_LOG}" "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}" \
    "${INPUT_DAEMON_LOG}" "${SCREENSHOT_PPM}" "${SCREENSHOT_PNG}"

python3 "${INPUT_HELPER}" --serve "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}" \
    >"${INPUT_DAEMON_LOG}" 2>&1 &
INPUT_DAEMON_PID=$!
i=0
while [ "${i}" -lt 100 ] && [ ! -S "${INPUT_CONTROL_SOCKET}" ]; do
    kill -0 "${INPUT_DAEMON_PID}" 2>/dev/null || exit 1
    sleep 0.1
    i=$((i + 1))
done
test -S "${INPUT_CONTROL_SOCKET}"

export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET SCREENSHOT_PPM SCREENSHOT_PNG
export TIMEOUT_SECONDS INPUT_HELPER CAPTURE_HELPER
export AQUA_QEMU_INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET}"
expect "${ROOT_DIR}/scripts/check-terminal-qemu.exp"

grep -Fq '[AQUA-TEST] stage=terminal-qemu status=ok surface=aqua.terminal pty=true emulator=vt100 command=true resize=protocol-ready' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=terminal-resize-qemu status=ok input=alt-f8 buffer=640x478 grid=74x21 pty=true repaint=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=terminal-cleanup-qemu status=ok process=clean surface=removed restart=never' "${SERIAL_LOG}"
test -s "${SCREENSHOT_PNG}"
echo "Aqua Terminal QEMU check passed."
echo "Screenshot: ${SCREENSHOT_PNG}"
