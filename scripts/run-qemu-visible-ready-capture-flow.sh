#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SERIAL_LOG="${SERIAL_LOG:-${AQUA_QEMU_VISIBLE_SERIAL_LOG:-${ROOT_DIR}/build/qemu-visible-manual-serial.log}}"
CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-qemu-visible-ready-$(date -u +%Y%m%dT%H%M%SZ)}"
CAPTURE_DIR="${AQUA_QEMU_CAPTURE_DIR:-${ROOT_DIR}/build/qemu-visible-captures}"
CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-${CAPTURE_DIR}/${CAPTURE_ID}.png}"
META_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_META:-${CAPTURE_FILE}.env}"
BUNDLE_FILE="${AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE:-${CAPTURE_DIR}/${CAPTURE_ID}-evidence-bundle.txt}"
PREFLIGHT_SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"
PRINT_ONLY="${AQUA_QEMU_READY_CAPTURE_FLOW_PRINT_ONLY:-false}"
SKIP_CAPTURE="${AQUA_QEMU_READY_CAPTURE_SKIP_CAPTURE:-false}"

echo "Aqua Linux QEMU visible ready capture flow"
echo "product=Aqua Linux"
echo "mode=host-qemu-visible-ready-capture-flow"
echo "target=QEMU x86_64"
echo "serial_log=${SERIAL_LOG}"
echo "capture_id=${CAPTURE_ID}"
echo "capture_file=${CAPTURE_FILE}"
echo "metadata_file=${META_FILE}"
echo "bundle_file=${BUNDLE_FILE}"
echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "ready_capture_flow_ready=true"
    echo "capture_hash_verification_required=true"
    echo "flow_step_1=scripts/watch-qemu-visible-readiness.sh"
    echo "flow_step_2=scripts/capture-qemu-visible-manual.sh"
    echo "flow_step_3=scripts/run-qemu-visible-evidence-flow.sh"
    echo "next_vm_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply"
    echo "[AQUA-HOST] stage=qemu-visible-ready-capture-flow status=print-only"
    exit 0
fi

watch_output="$(SERIAL_LOG="${SERIAL_LOG}" scripts/watch-qemu-visible-readiness.sh)"
printf '%s\n' "${watch_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-readiness-watch status=ok'
printf '%s\n' "${watch_output}" | grep -Fq 'qemu_visible_serial_ready=true'

if [ "${SKIP_CAPTURE}" = "true" ]; then
    if [ ! -s "${CAPTURE_FILE}" ]; then
        echo "capture_file_status=missing-or-empty" >&2
        echo "AQUA_QEMU_READY_CAPTURE_SKIP_CAPTURE=true requires an existing capture file." >&2
        exit 1
    fi
    capture_sha256="unavailable"
    if command -v shasum >/dev/null 2>&1; then
        capture_sha256="$(shasum -a 256 "${CAPTURE_FILE}" | awk '{print $1}')"
    elif command -v sha256sum >/dev/null 2>&1; then
        capture_sha256="$(sha256sum "${CAPTURE_FILE}" | awk '{print $1}')"
    fi
    {
        echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${CAPTURE_ID}"
        echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${CAPTURE_FILE}"
        echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256=${capture_sha256}"
        echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_TOOL=existing-file"
    } > "${META_FILE}"
    echo "capture_step=skipped-existing-file"
else
    capture_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_ID="${CAPTURE_ID}" AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${CAPTURE_FILE}" AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${META_FILE}" scripts/capture-qemu-visible-manual.sh)"
    printf '%s\n' "${capture_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-manual-capture status=ok'
    printf '%s\n' "${capture_output}" | grep -Fq 'capture_file_status=ready'
    echo "capture_step=host-screenshot"
fi

flow_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_ID="${CAPTURE_ID}" AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${CAPTURE_FILE}" AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${META_FILE}" AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${BUNDLE_FILE}" AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${PREFLIGHT_SUMMARY_JSON}" scripts/run-qemu-visible-evidence-flow.sh)"
printf '%s\n' "${flow_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-evidence-flow status=ok'
printf '%s\n' "${flow_output}" | grep -Fq 'evidence_flow_ready=true'
printf '%s\n' "${flow_output}" | grep -Fq 'capture_hash_verified=true'
printf '%s\n' "${flow_output}" | grep -Fq 'preflight_summary_verified=true'

echo "readiness_watch_ready=true"
echo "capture_file_status=ready"
echo "capture_hash_verified=true"
echo "preflight_summary_verified=true"
echo "evidence_flow_ready=true"
echo "ready_capture_flow_ready=true"
echo
printf '%s\n' "${flow_output}"
echo "[AQUA-HOST] stage=qemu-visible-ready-capture-flow status=ok"
