#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-session-menu-check.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-session-menu-monitor.sock}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"

trap 'rm -f "${MONITOR_SOCKET}"' EXIT HUP INT TERM
rm -f "${SERIAL_LOG}" "${MONITOR_SOCKET}"
export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET TIMEOUT_SECONDS INPUT_HELPER
expect "${ROOT_DIR}/scripts/check-session-menu-qemu.exp"
grep -Fq '[AQUA-TEST] stage=desktop-session-menu-focused-qemu status=ok execution=return-to-recovery cleanup=clean' "${SERIAL_LOG}"
echo "Aqua session menu focused QEMU check passed."
