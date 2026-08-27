#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PREFLIGHT_SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"
MANIFEST="${AQUA_IMAGE_MANIFEST:-${ROOT_DIR}/build/aqua-image-manifest.txt}"
CONTRACT_DIR="${AQUA_ROOTFS_CONTRACT_DIR:-${ROOT_DIR}/build/rootfs-compositor-contract}"
STATUS_FILE="${AQUA_QEMU_VISIBLE_STATUS_FILE:-${ROOT_DIR}/build/qemu-visible-status.txt}"
STATUS_JSON="${AQUA_QEMU_VISIBLE_STATUS_JSON:-${ROOT_DIR}/build/qemu-visible-status.json}"

manifest_value() {
    key="$1"

    if [ -f "${MANIFEST}" ]; then
        value="$(awk -F= -v key="${key}" '$1 == key { value = substr($0, length(key) + 2) } END { print value }' "${MANIFEST}")"
        if [ -n "${value}" ]; then
            printf '%s' "${value}"
            return
        fi
    fi

    printf 'unknown'
}

file_status() {
    path="$1"

    if [ -s "${path}" ]; then
        printf 'ready'
    elif [ -f "${path}" ]; then
        printf 'empty'
    else
        printf 'missing'
    fi
}

preflight_summary_status="missing"
preflight_source_bytes="unknown"
preflight_source_sha256="unknown"
preflight_source_mtime_utc="unknown"
if [ -f "${PREFLIGHT_SUMMARY_JSON}" ] \
    && AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${PREFLIGHT_SUMMARY_JSON}" "${ROOT_DIR}/scripts/check-qemu-visible-preflight-summary.sh" >/dev/null 2>&1; then
    preflight_summary_status="ok"
    preflight_source_values="$(python3 - "${PREFLIGHT_SUMMARY_JSON}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    summary = json.load(handle)

entry = summary.get("preflight_file_entry", {})
print(entry.get("bytes", "unknown"))
print(entry.get("sha256", "unknown"))
print(entry.get("mtime_utc", "unknown"))
PY
)"
    preflight_source_bytes="$(printf '%s\n' "${preflight_source_values}" | sed -n '1p')"
    preflight_source_sha256="$(printf '%s\n' "${preflight_source_values}" | sed -n '2p')"
    preflight_source_mtime_utc="$(printf '%s\n' "${preflight_source_values}" | sed -n '3p')"
fi

manual_runbook_status="$(manifest_value qemu_visible_manual_runbook)"
ready_capture_flow_status="$(manifest_value qemu_visible_manual_runbook_ready_capture_flow)"
manual_runbook_pass_report_required_status="$(manifest_value qemu_visible_manual_runbook_pass_report_required)"
pass_report_status="$(manifest_value qemu_visible_pass_report)"
pass_report_observed_status="$(manifest_value qemu_visible_pass_report_observed)"
pass_report_attempt_completed_status="$(manifest_value qemu_visible_pass_report_attempt_completed)"
pass_report_evidence_recorded_status="$(manifest_value qemu_visible_pass_report_evidence_recorded)"
pass_report_evidence_rule_status="$(manifest_value qemu_visible_pass_report_evidence_rule)"
bundle_apply_status="$(manifest_value qemu_visible_evidence_bundle_apply)"
bundle_positive_status="$(manifest_value qemu_visible_evidence_bundle_apply_positive)"
bundle_rejected_status="$(manifest_value qemu_visible_evidence_bundle_apply_missing_preflight_rejected)"
bundle_preflight_status="$(manifest_value qemu_visible_evidence_bundle_apply_preflight_verified)"
bundle_capture_hash_status="$(manifest_value qemu_visible_evidence_bundle_apply_capture_hash_verified)"
bundle_positive_preflight_status="$(manifest_value qemu_visible_evidence_bundle_apply_positive_preflight_verified)"
bundle_positive_capture_hash_status="$(manifest_value qemu_visible_evidence_bundle_apply_positive_capture_hash_verified)"
bundle_missing_unverified_status="$(manifest_value qemu_visible_evidence_bundle_apply_missing_preflight_unverified)"
bundle_missing_capture_hash_rejected_status="$(manifest_value qemu_visible_evidence_bundle_apply_missing_capture_hash_rejected)"
boot_graphics="$(manifest_value boot_graphics)"
autostart="$(manifest_value autostart)"
desktop_shell="$(manifest_value desktop_shell)"

if [ "${preflight_summary_status}" = "ok" ] \
    && [ "${manual_runbook_status}" = "ok" ] \
    && [ "${ready_capture_flow_status}" = "ok" ] \
    && [ "${manual_runbook_pass_report_required_status}" = "ok" ] \
    && [ "${pass_report_status}" = "ok" ] \
    && [ "${pass_report_observed_status}" = "ok" ] \
    && [ "${pass_report_attempt_completed_status}" = "ok" ] \
    && [ "${pass_report_evidence_recorded_status}" = "ok" ] \
    && [ "${pass_report_evidence_rule_status}" = "ok" ] \
    && [ "${bundle_apply_status}" = "ok" ] \
    && [ "${bundle_positive_status}" = "ok" ] \
    && [ "${bundle_rejected_status}" = "ok" ] \
    && [ "${bundle_preflight_status}" = "ok" ] \
    && [ "${bundle_capture_hash_status}" = "ok" ] \
    && [ "${bundle_positive_preflight_status}" = "ok" ] \
    && [ "${bundle_positive_capture_hash_status}" = "ok" ] \
    && [ "${bundle_missing_unverified_status}" = "ok" ] \
    && [ "${bundle_missing_capture_hash_rejected_status}" = "ok" ] \
    && [ "${boot_graphics}" = "false" ] \
    && [ "${autostart}" = "false" ] \
    && [ "${desktop_shell}" = "not_started" ]; then
    qemu_visible_manual_status="ready-for-operator-pass"
    host_status="ok"
    exit_code=0
else
    qemu_visible_manual_status="incomplete"
    host_status="blocked"
    exit_code=1
fi

status_output="$(
    echo "Aqua Linux QEMU visible status"
    echo "product=Aqua Linux"
    echo "mode=host-qemu-visible-status"
    echo "target=QEMU x86_64"
    echo "status_file=${STATUS_FILE}"
    echo "status_json=${STATUS_JSON}"
    echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
    echo "preflight_summary_status=${preflight_summary_status}"
    echo "preflight_source_bytes=${preflight_source_bytes}"
    echo "preflight_source_sha256=${preflight_source_sha256}"
    echo "preflight_source_mtime_utc=${preflight_source_mtime_utc}"
    echo "image_manifest=${MANIFEST}"
    echo "image_manifest_status=$(file_status "${MANIFEST}")"
    echo "manual_runbook_status=${manual_runbook_status}"
    echo "ready_capture_flow_status=${ready_capture_flow_status}"
    echo "manual_runbook_pass_report_required_status=${manual_runbook_pass_report_required_status}"
    echo "pass_report_status=${pass_report_status}"
    echo "pass_report_observed_status=${pass_report_observed_status}"
    echo "pass_report_attempt_completed_status=${pass_report_attempt_completed_status}"
    echo "pass_report_evidence_recorded_status=${pass_report_evidence_recorded_status}"
    echo "pass_report_evidence_rule_status=${pass_report_evidence_rule_status}"
    echo "bundle_apply_status=${bundle_apply_status}"
    echo "bundle_positive_status=${bundle_positive_status}"
    echo "bundle_rejected_status=${bundle_rejected_status}"
    echo "bundle_preflight_status=${bundle_preflight_status}"
    echo "bundle_capture_hash_status=${bundle_capture_hash_status}"
    echo "bundle_positive_preflight_status=${bundle_positive_preflight_status}"
    echo "bundle_positive_capture_hash_status=${bundle_positive_capture_hash_status}"
    echo "bundle_missing_unverified_status=${bundle_missing_unverified_status}"
    echo "bundle_missing_capture_hash_rejected_status=${bundle_missing_capture_hash_rejected_status}"
    echo "boot_graphics=${boot_graphics}"
    echo "autostart=${autostart}"
    echo "desktop_shell=${desktop_shell}"
    echo "runbook_file=${CONTRACT_DIR}/qemu-visible-manual-runbook.txt"
    echo "bundle_apply_file=${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt"
    echo "bundle_positive_file=${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt"
    echo "bundle_rejected_file=${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt"
    echo "bundle_hash_rejected_file=${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt"
    echo "pass_report_file=${CONTRACT_DIR}/qemu-visible-pass-report.txt"
    echo "capture_hash_verification_required=true"
    echo "operator_confirmation_required=true"
    echo "manual_observation_required=true"
    echo "persistent_graphical_session_started=false"
    echo "next_preflight_command=scripts/preflight-qemu-visible-manual.sh"
    echo "next_preflight_summary_command=scripts/write-qemu-visible-preflight-summary.sh"
    echo "next_launch_command=scripts/run-qemu-visible-manual.sh"
    echo "next_capture_flow_command=scripts/run-qemu-visible-ready-capture-flow.sh"
    echo "next_vm_apply_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply"
    echo "next_vm_report_command=aqua-qemu-visible-pass-report"
    echo "qemu_visible_manual_status=${qemu_visible_manual_status}"
    echo "[AQUA-HOST] stage=qemu-visible-status status=${host_status}"
)"

mkdir -p "$(dirname "${STATUS_FILE}")"
printf '%s\n' "${status_output}" > "${STATUS_FILE}"

export STATUS_JSON STATUS_FILE PREFLIGHT_SUMMARY_JSON MANIFEST CONTRACT_DIR
export preflight_summary_status preflight_source_bytes preflight_source_sha256 preflight_source_mtime_utc manual_runbook_status ready_capture_flow_status manual_runbook_pass_report_required_status
export pass_report_status pass_report_observed_status pass_report_attempt_completed_status pass_report_evidence_recorded_status pass_report_evidence_rule_status
export bundle_apply_status bundle_positive_status bundle_rejected_status
export bundle_preflight_status bundle_capture_hash_status bundle_positive_preflight_status bundle_positive_capture_hash_status
export bundle_missing_unverified_status bundle_missing_capture_hash_rejected_status
export boot_graphics autostart desktop_shell qemu_visible_manual_status host_status

mkdir -p "$(dirname "${STATUS_JSON}")"
python3 - <<'PY'
import json
import os
from datetime import datetime, timezone

status = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-status",
    "target": "QEMU x86_64",
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "status_file": os.environ["STATUS_FILE"],
    "status_json": os.environ["STATUS_JSON"],
    "preflight_summary_json": os.environ["PREFLIGHT_SUMMARY_JSON"],
    "preflight_summary_status": os.environ["preflight_summary_status"],
    "preflight_source_bytes": os.environ["preflight_source_bytes"],
    "preflight_source_sha256": os.environ["preflight_source_sha256"],
    "preflight_source_mtime_utc": os.environ["preflight_source_mtime_utc"],
    "image_manifest": os.environ["MANIFEST"],
    "manual_runbook_status": os.environ["manual_runbook_status"],
    "ready_capture_flow_status": os.environ["ready_capture_flow_status"],
    "manual_runbook_pass_report_required_status": os.environ["manual_runbook_pass_report_required_status"],
    "pass_report_status": os.environ["pass_report_status"],
    "pass_report_observed_status": os.environ["pass_report_observed_status"],
    "pass_report_attempt_completed_status": os.environ["pass_report_attempt_completed_status"],
    "pass_report_evidence_recorded_status": os.environ["pass_report_evidence_recorded_status"],
    "pass_report_evidence_rule_status": os.environ["pass_report_evidence_rule_status"],
    "bundle_apply_status": os.environ["bundle_apply_status"],
    "bundle_positive_status": os.environ["bundle_positive_status"],
    "bundle_rejected_status": os.environ["bundle_rejected_status"],
    "bundle_preflight_status": os.environ["bundle_preflight_status"],
    "bundle_capture_hash_status": os.environ["bundle_capture_hash_status"],
    "bundle_positive_preflight_status": os.environ["bundle_positive_preflight_status"],
    "bundle_positive_capture_hash_status": os.environ["bundle_positive_capture_hash_status"],
    "bundle_missing_unverified_status": os.environ["bundle_missing_unverified_status"],
    "bundle_missing_capture_hash_rejected_status": os.environ["bundle_missing_capture_hash_rejected_status"],
    "boot_graphics": os.environ["boot_graphics"] == "true",
    "autostart": os.environ["autostart"] == "true",
    "desktop_shell": os.environ["desktop_shell"],
    "operator_confirmation_required": True,
    "manual_observation_required": True,
    "persistent_graphical_session_started": False,
    "qemu_visible_manual_status": os.environ["qemu_visible_manual_status"],
    "host_status": os.environ["host_status"],
    "artifacts": {
        "runbook_file": os.path.join(os.environ["CONTRACT_DIR"], "qemu-visible-manual-runbook.txt"),
        "bundle_apply_file": os.path.join(os.environ["CONTRACT_DIR"], "qemu-visible-evidence-bundle-apply.txt"),
        "bundle_positive_file": os.path.join(os.environ["CONTRACT_DIR"], "qemu-visible-evidence-bundle-apply-positive.txt"),
        "bundle_rejected_file": os.path.join(os.environ["CONTRACT_DIR"], "qemu-visible-evidence-bundle-apply-missing-preflight.txt"),
        "bundle_hash_rejected_file": os.path.join(os.environ["CONTRACT_DIR"], "qemu-visible-evidence-bundle-apply-missing-capture-hash.txt"),
        "pass_report_file": os.path.join(os.environ["CONTRACT_DIR"], "qemu-visible-pass-report.txt"),
    },
    "capture_hash_verification_required": True,
    "next_commands": {
        "preflight": "scripts/preflight-qemu-visible-manual.sh",
        "preflight_summary": "scripts/write-qemu-visible-preflight-summary.sh",
        "launch": "scripts/run-qemu-visible-manual.sh",
        "capture_flow": "scripts/run-qemu-visible-ready-capture-flow.sh",
        "vm_apply": "AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply",
        "vm_report": "aqua-qemu-visible-pass-report",
    },
}

with open(os.environ["STATUS_JSON"], "w", encoding="utf-8") as handle:
    json.dump(status, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY

printf '%s\n' "${status_output}"
exit "${exit_code}"
