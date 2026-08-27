#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-manual-qemu-display-capture}"
META_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_META:-}"
BUNDLE_FILE="${AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE:-}"
VM_BUNDLE_FILE="${AQUA_QEMU_VM_VISIBLE_EVIDENCE_BUNDLE_FILE:-/run/aqua/qemu-visible-evidence-bundle.txt}"
PRINT_ONLY="${AQUA_QEMU_EVIDENCE_APPLY_PRINT_ONLY:-false}"

load_meta() {
    if [ -n "${META_FILE}" ] && [ -f "${META_FILE}" ]; then
        # shellcheck disable=SC1090
        . "${META_FILE}"
        CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-${CAPTURE_ID}}"
    fi
}

load_meta

if [ -z "${BUNDLE_FILE}" ]; then
    BUNDLE_FILE="${ROOT_DIR}/build/qemu-visible-captures/${CAPTURE_ID}-evidence-bundle.txt"
fi

echo "Aqua Linux QEMU VM-display evidence apply prep"
echo "product=Aqua Linux"
echo "mode=host-evidence-apply-prep"
echo "target=QEMU x86_64"
echo "bundle_file=${BUNDLE_FILE}"
echo "vm_bundle_file=${VM_BUNDLE_FILE}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "apply_prep_ready=true"
    echo "apply_prep_command=AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE=<bundle-file> scripts/prepare-qemu-visible-evidence-apply.sh"
    echo "[AQUA-HOST] stage=qemu-visible-evidence-apply-prep status=print-only"
    exit 0
fi

if [ ! -s "${BUNDLE_FILE}" ]; then
    echo "bundle_file_status=missing-or-empty" >&2
    exit 1
fi

grep -Fq 'bundle_status=recovery-commands-ready' "${BUNDLE_FILE}"
grep -Fq 'operator_confirmation_required=true' "${BUNDLE_FILE}"
grep -Fq 'boot_graphics=false' "${BUNDLE_FILE}"
grep -Fq 'autostart=false' "${BUNDLE_FILE}"
grep -Fq 'capture_sha256=' "${BUNDLE_FILE}"
grep -Fq 'capture_hash_verified=true' "${BUNDLE_FILE}"
grep -Fq 'preflight_summary_status=ok' "${BUNDLE_FILE}"
grep -Fq 'preflight_summary_verified=true' "${BUNDLE_FILE}"

echo "bundle_file_status=ready"
echo "capture_hash_verified=true"
echo "preflight_summary_verified=true"
echo "apply_prep_ready=true"
echo
echo "Paste into the QEMU recovery shell after the visible VM display is confirmed:"
echo "mkdir -p /run/aqua"
echo "cat > ${VM_BUNDLE_FILE} <<'AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE'"
cat "${BUNDLE_FILE}"
echo "AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE"
echo "aqua-qemu-visible-evidence-bundle-apply"
echo "AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply"
echo "aqua-qemu-visible-pass-report"
echo "[AQUA-HOST] stage=qemu-visible-evidence-apply-prep status=ok"
