#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-manual-qemu-display-capture}"
META_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_META:-}"
CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-}"
PREFLIGHT_SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"
PRINT_ONLY="${AQUA_QEMU_EVIDENCE_FLOW_PRINT_ONLY:-false}"

load_meta() {
    if [ -n "${META_FILE}" ] && [ -f "${META_FILE}" ]; then
        # shellcheck disable=SC1090
        . "${META_FILE}"
        CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-${CAPTURE_ID}}"
        CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-${CAPTURE_FILE}}"
    fi
}

load_meta

if [ -z "${CAPTURE_FILE}" ]; then
    CAPTURE_FILE="${ROOT_DIR}/build/qemu-visible-captures/${CAPTURE_ID}.png"
fi

BUNDLE_FILE="${AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE:-${ROOT_DIR}/build/qemu-visible-captures/${CAPTURE_ID}-evidence-bundle.txt}"

echo "Aqua Linux QEMU VM-display evidence flow"
echo "product=Aqua Linux"
echo "mode=host-evidence-flow"
echo "target=QEMU x86_64"
echo "capture_id=${CAPTURE_ID}"
echo "capture_file=${CAPTURE_FILE}"
echo "metadata_file=${META_FILE}"
echo "bundle_file=${BUNDLE_FILE}"
echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "evidence_flow_ready=true"
    echo "flow_step_1=scripts/capture-qemu-visible-manual.sh"
    echo "flow_step_2=scripts/verify-qemu-visible-capture.sh"
    echo "flow_step_3=scripts/write-qemu-visible-evidence-bundle.sh"
    echo "flow_step_4=scripts/prepare-qemu-visible-evidence-apply.sh"
    echo "[AQUA-HOST] stage=qemu-visible-evidence-flow status=print-only"
    exit 0
fi

if [ ! -s "${CAPTURE_FILE}" ]; then
    echo "capture_file_status=missing-or-empty" >&2
    echo "Run scripts/capture-qemu-visible-manual.sh after the QEMU window is visible." >&2
    exit 1
fi

verify_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_ID="${CAPTURE_ID}" AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${CAPTURE_FILE}" AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${META_FILE}" scripts/verify-qemu-visible-capture.sh)"
printf '%s\n' "${verify_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-capture-verify status=ok'

preflight_output="$(AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${PREFLIGHT_SUMMARY_JSON}" scripts/check-qemu-visible-preflight-summary.sh)"
printf '%s\n' "${preflight_output}" | grep -Fq 'Aqua Linux QEMU visible preflight summary checks passed.'

bundle_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_ID="${CAPTURE_ID}" AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${CAPTURE_FILE}" AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${META_FILE}" AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${BUNDLE_FILE}" AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${PREFLIGHT_SUMMARY_JSON}" scripts/write-qemu-visible-evidence-bundle.sh)"
printf '%s\n' "${bundle_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-evidence-bundle status=ok'
printf '%s\n' "${bundle_output}" | grep -Fq 'bundle_written=ok'
printf '%s\n' "${bundle_output}" | grep -Fq 'preflight_summary_verified=true'
printf '%s\n' "${bundle_output}" | grep -Fq 'capture_hash_verified=true'

apply_output="$(AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${BUNDLE_FILE}" scripts/prepare-qemu-visible-evidence-apply.sh)"
printf '%s\n' "${apply_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-evidence-apply-prep status=ok'
printf '%s\n' "${apply_output}" | grep -Fq 'capture_hash_verified=true'

echo "capture_file_status=ready"
echo "capture_verify_ready=true"
echo "capture_hash_verified=true"
echo "preflight_summary_verified=true"
echo "bundle_written=ok"
echo "apply_prep_ready=true"
echo "evidence_flow_ready=true"
echo
printf '%s\n' "${apply_output}"
echo "[AQUA-HOST] stage=qemu-visible-evidence-flow status=ok"
