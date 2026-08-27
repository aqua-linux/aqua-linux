#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-manual-qemu-display-capture}"
CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-}"
META_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_META:-}"
EXPECTED_CAPTURE_SHA256="${AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256:-}"
PRINT_ONLY="${AQUA_QEMU_CAPTURE_VERIFY_PRINT_ONLY:-false}"

load_meta() {
    if [ -n "${META_FILE}" ] && [ -f "${META_FILE}" ]; then
        # shellcheck disable=SC1090
        . "${META_FILE}"
        CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-${CAPTURE_ID}}"
        CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-${CAPTURE_FILE}}"
        EXPECTED_CAPTURE_SHA256="${AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256:-${EXPECTED_CAPTURE_SHA256}}"
    fi
}

load_meta

if [ -z "${CAPTURE_FILE}" ]; then
    CAPTURE_FILE="${ROOT_DIR}/build/qemu-visible-captures/${CAPTURE_ID}.png"
fi

echo "Aqua Linux QEMU VM-display capture verifier"
echo "product=Aqua Linux"
echo "mode=host-manual-capture-verify"
echo "target=QEMU x86_64"
echo "capture_id=${CAPTURE_ID}"
echo "capture_file=${CAPTURE_FILE}"
echo "expected_capture_sha256=${EXPECTED_CAPTURE_SHA256:-none}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "capture_verify_ready=true"
    echo "evidence_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${CAPTURE_ID} AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${CAPTURE_FILE} aqua-qemu-visible-evidence-record"
    echo "[AQUA-HOST] stage=qemu-visible-capture-verify status=print-only"
    exit 0
fi

if [ ! -s "${CAPTURE_FILE}" ]; then
    echo "capture_file_status=missing-or-empty" >&2
    exit 1
fi

capture_sha256="unavailable"
if command -v shasum >/dev/null 2>&1; then
    capture_sha256="$(shasum -a 256 "${CAPTURE_FILE}" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
    capture_sha256="$(sha256sum "${CAPTURE_FILE}" | awk '{print $1}')"
fi

capture_hash_verified="not-provided"
if [ -n "${EXPECTED_CAPTURE_SHA256}" ] && [ "${EXPECTED_CAPTURE_SHA256}" != "unavailable" ]; then
    if [ "${capture_sha256}" = "unavailable" ]; then
        echo "capture_hash_status=unverifiable" >&2
        exit 1
    fi
    if [ "${capture_sha256}" != "${EXPECTED_CAPTURE_SHA256}" ]; then
        echo "capture_hash_status=mismatch" >&2
        echo "expected_capture_sha256=${EXPECTED_CAPTURE_SHA256}" >&2
        echo "actual_capture_sha256=${capture_sha256}" >&2
        exit 1
    fi
    capture_hash_verified="true"
fi

echo "capture_file_status=ready"
echo "capture_sha256=${capture_sha256}"
echo "expected_capture_sha256=${EXPECTED_CAPTURE_SHA256:-none}"
echo "capture_hash_verified=${capture_hash_verified}"
echo "capture_verify_ready=true"
echo "evidence_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${CAPTURE_ID} AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${CAPTURE_FILE} aqua-qemu-visible-evidence-record"
echo "[AQUA-HOST] stage=qemu-visible-capture-verify status=ok"
