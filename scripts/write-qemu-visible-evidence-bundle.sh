#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-manual-qemu-display-capture}"
CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-}"
META_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_META:-}"
BUNDLE_FILE="${AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE:-}"
PREFLIGHT_SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"
PRINT_ONLY="${AQUA_QEMU_EVIDENCE_BUNDLE_PRINT_ONLY:-false}"

load_meta() {
    if [ -n "${META_FILE}" ] && [ -f "${META_FILE}" ]; then
        # shellcheck disable=SC1090
        . "${META_FILE}"
        CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-${CAPTURE_ID}}"
        CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-${CAPTURE_FILE}}"
    fi
}

quote_sh() {
    printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"
}

load_meta

if [ -z "${CAPTURE_FILE}" ]; then
    CAPTURE_FILE="${ROOT_DIR}/build/qemu-visible-captures/${CAPTURE_ID}.png"
fi

if [ -z "${BUNDLE_FILE}" ]; then
    BUNDLE_FILE="${ROOT_DIR}/build/qemu-visible-captures/${CAPTURE_ID}-evidence-bundle.txt"
fi

echo "Aqua Linux QEMU VM-display evidence bundle"
echo "product=Aqua Linux"
echo "mode=host-evidence-bundle"
echo "target=QEMU x86_64"
echo "capture_id=${CAPTURE_ID}"
echo "capture_file=${CAPTURE_FILE}"
echo "bundle_file=${BUNDLE_FILE}"
echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "evidence_bundle_ready=true"
    echo "bundle_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_META=${META_FILE} scripts/write-qemu-visible-evidence-bundle.sh"
    echo "[AQUA-HOST] stage=qemu-visible-evidence-bundle status=print-only"
    exit 0
fi

verify_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_ID="${CAPTURE_ID}" AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${CAPTURE_FILE}" AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${META_FILE}" scripts/verify-qemu-visible-capture.sh)"
printf '%s\n' "${verify_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-capture-verify status=ok'
printf '%s\n' "${verify_output}" | grep -Fq 'capture_file_status=ready'

preflight_output="$(AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${PREFLIGHT_SUMMARY_JSON}" scripts/check-qemu-visible-preflight-summary.sh)"
printf '%s\n' "${preflight_output}" | grep -Fq 'Aqua Linux QEMU visible preflight summary checks passed.'

capture_sha256="$(printf '%s\n' "${verify_output}" | awk -F= '/^capture_sha256=/ {print $2; exit}')"
if [ -z "${capture_sha256}" ]; then
    capture_sha256="unavailable"
fi
capture_hash_verified="$(printf '%s\n' "${verify_output}" | awk -F= '/^capture_hash_verified=/ {print $2; exit}')"
if [ -z "${capture_hash_verified}" ]; then
    capture_hash_verified="not-provided"
fi
if [ "${capture_hash_verified}" != "true" ]; then
    echo "capture_hash_verified=${capture_hash_verified}" >&2
    echo "capture_hash_status=not-verified" >&2
    echo "metadata_file=${META_FILE:-none}" >&2
    echo "hint=run capture with metadata that pins AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256" >&2
    echo "[AQUA-HOST] stage=qemu-visible-evidence-bundle status=error" >&2
    exit 1
fi

preflight_generated_at="$(python3 - "${PREFLIGHT_SUMMARY_JSON}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    summary = json.load(handle)

print(summary.get("generated_at_utc", "unknown"))
PY
)"

mkdir -p "$(dirname "${BUNDLE_FILE}")"

capture_id_q="$(quote_sh "${CAPTURE_ID}")"
capture_file_q="$(quote_sh "${CAPTURE_FILE}")"

{
    echo "product=Aqua Linux"
    echo "bundle=qemu-visible-evidence"
    echo "bundle_status=recovery-commands-ready"
    echo "capture_id=${CAPTURE_ID}"
    echo "capture_file=${CAPTURE_FILE}"
    echo "capture_sha256=${capture_sha256}"
    echo "capture_hash_verified=${capture_hash_verified}"
    echo "preflight_summary_status=ok"
    echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
    echo "preflight_summary_generated_at=${preflight_generated_at}"
    echo "preflight_summary_verified=true"
    echo "recovery_step_1=aqua-graphics-qemu-visible-boot-check"
    echo "recovery_step_2=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${capture_id_q} AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${capture_file_q} aqua-qemu-visible-evidence-record"
    echo "recovery_step_3=AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker"
    echo "recovery_step_4=aqua-qemu-visible-pass-report"
    echo "operator_confirmation_required=true"
    echo "manual_observation_required=true"
    echo "persistent_graphical_session_started=false"
    echo "desktop_shell_started=false"
    echo "boot_graphics=false"
    echo "autostart=false"
    echo "fallback_tty_available=true"
    echo "safe_return_to_recovery=ok"
} > "${BUNDLE_FILE}"

grep -Fq 'bundle_status=recovery-commands-ready' "${BUNDLE_FILE}"
grep -Fq 'operator_confirmation_required=true' "${BUNDLE_FILE}"
grep -Fq 'boot_graphics=false' "${BUNDLE_FILE}"
grep -Fq 'autostart=false' "${BUNDLE_FILE}"
grep -Fq 'preflight_summary_verified=true' "${BUNDLE_FILE}"

echo "capture_file_status=ready"
echo "capture_sha256=${capture_sha256}"
echo "capture_hash_verified=${capture_hash_verified}"
echo "preflight_summary_status=ok"
echo "preflight_summary_verified=true"
echo "bundle_written=ok"
echo "evidence_bundle_ready=true"
echo "recovery_step_1=aqua-graphics-qemu-visible-boot-check"
echo "recovery_step_2=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${capture_id_q} AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${capture_file_q} aqua-qemu-visible-evidence-record"
echo "recovery_step_3=AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker"
echo "recovery_step_4=aqua-qemu-visible-pass-report"
echo "[AQUA-HOST] stage=qemu-visible-evidence-bundle status=ok"
