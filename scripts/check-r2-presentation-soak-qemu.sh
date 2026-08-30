#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
EVIDENCE_DIR="${EVIDENCE_DIR:-${ROOT_DIR}/build/qemu-r2-presentation-soak}"
SERIAL_LOG="${EVIDENCE_DIR}/serial.log"
MONITOR_SOCKET="${EVIDENCE_DIR}/monitor.sock"
REPORT="${EVIDENCE_DIR}/report.txt"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
VALIDATOR="${ROOT_DIR}/scripts/check-r2-presentation-log.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
SOAK_SECONDS="${SOAK_SECONDS:-300}"
INPUT_CYCLES="${INPUT_CYCLES:-10}"
INPUT_INTERVAL_MS="${INPUT_INTERVAL_MS:-24000}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-1320}"
DISPLAY_DEVICE="${DISPLAY_DEVICE:-bochs-display}"

case "${SOAK_SECONDS}" in
    ''|*[!0-9]*)
        echo "SOAK_SECONDS must be an integer from 300 through 900." >&2
        exit 1
        ;;
esac
if test "${SOAK_SECONDS}" -lt 300 || test "${SOAK_SECONDS}" -gt 900; then
    echo "SOAK_SECONDS must be an integer from 300 through 900." >&2
    exit 1
fi
if test -e "${EVIDENCE_DIR}"; then
    echo "R2 soak evidence directory already exists; choose a new EVIDENCE_DIR." >&2
    exit 1
fi
MONITOR_SOCKET_BYTES="$(printf %s "${MONITOR_SOCKET}" | wc -c | tr -d ' ')"
if test "${MONITOR_SOCKET_BYTES}" -ge 104; then
    echo "R2 soak monitor socket path must be shorter than 104 bytes." >&2
    exit 1
fi

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
    echo "R2 QEMU soak evidence requires a current compositor rootfs." >&2
    echo "Run scripts/build-image.sh before collecting evidence." >&2
    exit 1
}

mkdir -p "$(dirname "${EVIDENCE_DIR}")"
mkdir "${EVIDENCE_DIR}"

export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET INPUT_HELPER MEMORY CPUS
export SOAK_SECONDS INPUT_CYCLES INPUT_INTERVAL_MS TIMEOUT_SECONDS DISPLAY_DEVICE
expect "${ROOT_DIR}/scripts/check-r2-presentation-soak-qemu.exp" >/dev/null
PYTHONDONTWRITEBYTECODE=1 python3 "${VALIDATOR}" --summarize-soak \
    "${SERIAL_LOG}" >"${REPORT}"

grep -Fq 'r2_soak_budget_profile=qemu-tcg-bochs-soak-v1' "${REPORT}"
grep -Fq 'r2_soak_min_observation_window_ms=300000' "${REPORT}"
grep -Fq 'r2_soak_min_input_to_present_samples=5' "${REPORT}"
grep -Fq 'r2_soak_keyboard_events=' "${REPORT}"
grep -Fq 'r2_soak_crash_budget=0' "${REPORT}"
grep -Fq 'r2_soak_crashes=0' "${REPORT}"
grep -Fq 'r2_soak_client_lifecycle_complete=true' "${REPORT}"
grep -Fq 'r2_soak_diagnostic_isolation_recorded=true' "${REPORT}"
grep -Fq 'r2_soak_physical_evidence=false' "${REPORT}"

echo "Aqua Linux packaged R2 presentation soak passed."
echo "Evidence directory: ${EVIDENCE_DIR}"
echo "Soak report: ${REPORT}"
