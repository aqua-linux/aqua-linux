#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
EVIDENCE_DIR="${EVIDENCE_DIR:-${ROOT_DIR}/build/qemu-r2-presentation-qualification}"
SERIAL_LOG="${EVIDENCE_DIR}/serial.log"
MONITOR_SOCKET="${EVIDENCE_DIR}/monitor.sock"
REPORT="${EVIDENCE_DIR}/report.txt"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
VALIDATOR="${ROOT_DIR}/scripts/check-r2-presentation-log.py"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
SOAK_SECONDS="${SOAK_SECONDS:-900}"
INPUT_CYCLES="${INPUT_CYCLES:-15}"
INPUT_INTERVAL_MS="${INPUT_INTERVAL_MS:-24000}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-1920}"
DISPLAY_DEVICE="${DISPLAY_DEVICE:-bochs-display}"

if test "${SOAK_SECONDS}" != 900; then
    echo "The qemu-tcg-bochs-qualification-v1 profile requires SOAK_SECONDS=900." >&2
    exit 1
fi
if test "${INPUT_CYCLES}" != 15 || test "${INPUT_INTERVAL_MS}" != 24000; then
    echo "The qualification profile requires 15 acknowledged input cycles at 24000 ms intervals." >&2
    exit 1
fi
if test -e "${EVIDENCE_DIR}"; then
    echo "R2 qualification evidence directory already exists; choose a new EVIDENCE_DIR." >&2
    exit 1
fi
MONITOR_SOCKET_BYTES="$(printf %s "${MONITOR_SOCKET}" | wc -c | tr -d ' ')"
if test "${MONITOR_SOCKET_BYTES}" -ge 104; then
    echo "R2 qualification monitor socket path must be shorter than 104 bytes." >&2
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
    echo "R2 QEMU qualification evidence requires a current compositor rootfs." >&2
    echo "Run scripts/build-image.sh before collecting evidence." >&2
    exit 1
}

mkdir -p "$(dirname "${EVIDENCE_DIR}")"
mkdir "${EVIDENCE_DIR}"

export KERNEL ROOTFS SERIAL_LOG MONITOR_SOCKET INPUT_HELPER MEMORY CPUS
export SOAK_SECONDS INPUT_CYCLES INPUT_INTERVAL_MS TIMEOUT_SECONDS DISPLAY_DEVICE
expect "${ROOT_DIR}/scripts/check-r2-presentation-soak-qemu.exp" >/dev/null
PYTHONDONTWRITEBYTECODE=1 python3 "${VALIDATOR}" \
    --summarize-qualification-soak "${SERIAL_LOG}" >"${REPORT}"

grep -Fq 'r2_qualification_soak_budget_profile=qemu-tcg-bochs-qualification-v1' "${REPORT}"
grep -Fq 'r2_qualification_soak_min_observation_window_ms=900000' "${REPORT}"
grep -Fq 'r2_qualification_soak_min_input_to_present_samples=15' "${REPORT}"
grep -Fq 'r2_qualification_soak_min_keyboard_events=45' "${REPORT}"
grep -Fq 'r2_qualification_soak_crash_budget=0' "${REPORT}"
grep -Fq 'r2_qualification_soak_crashes=0' "${REPORT}"
grep -Fq 'r2_qualification_soak_client_lifecycle_complete=true' "${REPORT}"
grep -Fq 'r2_qualification_soak_diagnostic_isolation_recorded=true' "${REPORT}"
grep -Fq 'r2_qualification_soak_physical_evidence=false' "${REPORT}"
grep -Fq 'r2_qualification_soak_release_ready=false' "${REPORT}"

echo "Aqua Linux packaged R2 presentation qualification soak passed."
echo "Evidence directory: ${EVIDENCE_DIR}"
echo "Qualification report: ${REPORT}"
