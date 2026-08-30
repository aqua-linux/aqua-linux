#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-r2-presentation.log}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-r2-presentation-monitor.sock}"
REPORT="${REPORT:-${ROOT_DIR}/build/qemu-r2-presentation-report.txt}"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
VALIDATOR="${ROOT_DIR}/scripts/check-r2-presentation-log.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-420}"

cleanup() {
    rm -f "${MONITOR_SOCKET}"
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
test "${ROOTFS}" -nt "${ROOT_DIR}/crates/aqua-compositor/src/main.rs" || {
    echo "R2 QEMU evidence requires a rootfs rebuilt after the current compositor source." >&2
    echo "Run scripts/build-image.sh before collecting evidence." >&2
    exit 1
}

mkdir -p "$(dirname "${SERIAL_LOG}")"
rm -f "${SERIAL_LOG}" "${MONITOR_SOCKET}" "${REPORT}"

export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET INPUT_HELPER MEMORY CPUS TIMEOUT_SECONDS
expect "${ROOT_DIR}/scripts/check-r2-presentation-qemu.exp" >/dev/null
PYTHONDONTWRITEBYTECODE=1 python3 "${VALIDATOR}" "${SERIAL_LOG}" >"${REPORT}"

grep -Fq 'r2_qemu_workload_records=4' "${REPORT}"
grep -Fq 'r2_budget_selected=false' "${REPORT}"
grep -Fq 'r2_diagnostic_isolation_recorded=false' "${REPORT}"
test "$(grep -Fc 'r2_presentation_record_begin=v1' "${SERIAL_LOG}")" -eq 4
test "$(grep -Fc 'r2_presentation_record_end=v1' "${SERIAL_LOG}")" -eq 4

echo "Aqua Linux packaged R2 presentation workload collection passed."
echo "Serial log: ${SERIAL_LOG}"
echo "Observed maxima: ${REPORT}"
