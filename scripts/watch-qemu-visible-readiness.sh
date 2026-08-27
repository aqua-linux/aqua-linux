#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SERIAL_LOG="${SERIAL_LOG:-${AQUA_QEMU_VISIBLE_SERIAL_LOG:-${ROOT_DIR}/build/qemu-visible-manual-serial.log}}"
TIMEOUT="${AQUA_QEMU_VISIBLE_WATCH_TIMEOUT:-120}"
INTERVAL="${AQUA_QEMU_VISIBLE_WATCH_INTERVAL:-2}"
PRINT_ONLY="${AQUA_QEMU_VISIBLE_WATCH_PRINT_ONLY:-false}"
READY_MARKER='[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh'
SESSION_MARKER='[AQUA-BOOT] stage=session-check status=ok no_graphics=true'

echo "Aqua Linux QEMU visible readiness watcher"
echo "product=Aqua Linux"
echo "mode=host-qemu-visible-readiness-watch"
echo "target=QEMU x86_64"
echo "serial_log=${SERIAL_LOG}"
echo "timeout_seconds=${TIMEOUT}"
echo "interval_seconds=${INTERVAL}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"
echo "ready_marker=${READY_MARKER}"
echo "session_marker=${SESSION_MARKER}"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "readiness_watch_ready=true"
    echo "watch_command=SERIAL_LOG=${SERIAL_LOG} scripts/watch-qemu-visible-readiness.sh"
    echo "next_capture_command=scripts/capture-qemu-visible-manual.sh"
    echo "[AQUA-HOST] stage=qemu-visible-readiness-watch status=print-only"
    exit 0
fi

case "${TIMEOUT}" in
    ''|*[!0-9]*)
        echo "Invalid timeout_seconds=${TIMEOUT}" >&2
        exit 1
        ;;
esac

case "${INTERVAL}" in
    ''|*[!0-9]*)
        echo "Invalid interval_seconds=${INTERVAL}" >&2
        exit 1
        ;;
esac

elapsed=0
while [ "${elapsed}" -le "${TIMEOUT}" ]; do
    if [ -f "${SERIAL_LOG}" ] && grep -Fq "${READY_MARKER}" "${SERIAL_LOG}"; then
        session_check_seen=false
        if grep -Fq "${SESSION_MARKER}" "${SERIAL_LOG}"; then
            session_check_seen=true
        fi

        echo "serial_log_status=ready"
        echo "session_check_seen=${session_check_seen}"
        echo "qemu_visible_serial_ready=true"
        echo "waited_seconds=${elapsed}"
        echo "next_capture_command=scripts/capture-qemu-visible-manual.sh"
        echo "next_flow_command=scripts/run-qemu-visible-evidence-flow.sh"
        echo "[AQUA-HOST] stage=qemu-visible-readiness-watch status=ok"
        exit 0
    fi

    if [ "${elapsed}" -eq "${TIMEOUT}" ]; then
        break
    fi

    sleep "${INTERVAL}"
    elapsed=$((elapsed + INTERVAL))
done

echo "serial_log_status=not-ready"
echo "qemu_visible_serial_ready=false"
echo "waited_seconds=${elapsed}"
echo "missing_marker=${READY_MARKER}"
echo "[AQUA-HOST] stage=qemu-visible-readiness-watch status=timeout"
exit 1
