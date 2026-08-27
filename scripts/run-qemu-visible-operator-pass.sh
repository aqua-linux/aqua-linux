#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
STATUS_FILE="${AQUA_QEMU_VISIBLE_STATUS_FILE:-${ROOT_DIR}/build/qemu-visible-status.txt}"
STATUS_JSON="${AQUA_QEMU_VISIBLE_STATUS_JSON:-${ROOT_DIR}/build/qemu-visible-status.json}"
PLAN_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE:-${ROOT_DIR}/build/qemu-visible-operator-plan.txt}"
PLAN_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON:-${ROOT_DIR}/build/qemu-visible-operator-plan.json}"
PACKET_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE:-${ROOT_DIR}/build/qemu-visible-operator-packet.txt}"
PACKET_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON:-${ROOT_DIR}/build/qemu-visible-operator-packet.json}"
CHECKLIST="${AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST:-${ROOT_DIR}/build/qemu-visible-operator-checklist.md}"
PASS_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PASS_FILE:-${ROOT_DIR}/build/qemu-visible-operator-pass.txt}"
PASS_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PASS_JSON:-${ROOT_DIR}/build/qemu-visible-operator-pass.json}"
FIRST_GRAPHICS_SESSION_STATUS="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE:-${ROOT_DIR}/build/first-graphics-session-status.txt}"
FIRST_GRAPHICS_SESSION_STATUS_JSON="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON:-${ROOT_DIR}/build/first-graphics-session-status.json}"
PREFLIGHT_SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"
SERIAL_LOG="${AQUA_QEMU_VISIBLE_SERIAL_LOG:-${ROOT_DIR}/build/qemu-visible-manual-serial.log}"
CAPTURE_DIR_DEFAULT="${ROOT_DIR}/build/qemu-visible-captures"
CAPTURE_DIR="${AQUA_QEMU_CAPTURE_DIR:-${CAPTURE_DIR_DEFAULT}}"
PRINT_ONLY="${AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY:-false}"
NO_LAUNCH="${AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH:-false}"
CONFIRM_LAUNCH="${AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH:-false}"
PASS_STOP_RULE="Do not mark VM display observed unless a visible QEMU window has been confirmed and evidence bundle apply is run with AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true."

echo "Aqua Linux QEMU visible operator pass"
echo "product=Aqua Linux"
echo "base=Buildroot"
echo "mode=host-qemu-visible-operator-pass"
echo "target=QEMU x86_64"
echo "status_file=${STATUS_FILE}"
echo "status_json=${STATUS_JSON}"
echo "operator_plan=${PLAN_FILE}"
echo "operator_plan_json=${PLAN_JSON}"
echo "operator_packet=${PACKET_FILE}"
echo "operator_packet_json=${PACKET_JSON}"
echo "operator_checklist=${CHECKLIST}"
echo "operator_pass_file=${PASS_FILE}"
echo "operator_pass_json=${PASS_JSON}"
echo "first_graphics_session_status_file=${FIRST_GRAPHICS_SESSION_STATUS}"
echo "first_graphics_session_status_json=${FIRST_GRAPHICS_SESSION_STATUS_JSON}"
echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
echo "serial_log=${SERIAL_LOG}"
echo "capture_dir=${CAPTURE_DIR}"
echo "boot_graphics=false"
echo "autostart=false"
echo "persistent_graphical_session_started=false"
echo "operator_confirmation_required=true"
echo "launch_confirmation_required=true"
echo "launch_confirmed=${CONFIRM_LAUNCH}"
echo "no_launch=${NO_LAUNCH}"
echo "no_positive_observation_without_evidence=true"
echo "no_unverified_bundle_acceptance=true"
echo "capture_hash_verification_required=true"
echo "safe_return_to_recovery=ok"
echo "operator_pass_stop_rule=${PASS_STOP_RULE}"
echo "no_launch_rehearsal_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh"
echo "confirmed_launch_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh"
echo "next_launch_command=scripts/run-qemu-visible-manual.sh"
echo "next_vm_fbdev_command=AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present"
echo "next_capture_flow_command=scripts/run-qemu-visible-ready-capture-flow.sh"
echo "next_capture_verify_command=scripts/verify-qemu-visible-capture.sh"
echo "next_evidence_flow_command=scripts/run-qemu-visible-evidence-flow.sh"
echo "next_vm_apply_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "operator_pass_ready=true"
    echo "operator_pass_launch_armed=false"
    echo "[AQUA-HOST] stage=qemu-visible-operator-pass status=print-only"
    exit 0
fi

AQUA_QEMU_VISIBLE_STATUS_FILE="${STATUS_FILE}" \
    AQUA_QEMU_VISIBLE_STATUS_JSON="${STATUS_JSON}" \
    "${ROOT_DIR}/scripts/qemu-visible-status.sh" >/dev/null

AQUA_QEMU_VISIBLE_STATUS_FILE="${STATUS_FILE}" \
    AQUA_QEMU_VISIBLE_STATUS_JSON="${STATUS_JSON}" \
    "${ROOT_DIR}/scripts/check-qemu-visible-status.sh" >/dev/null

AQUA_QEMU_VISIBLE_STATUS_FILE="${STATUS_FILE}" \
    AQUA_QEMU_VISIBLE_STATUS_JSON="${STATUS_JSON}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${PLAN_FILE}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${PLAN_JSON}" \
    "${ROOT_DIR}/scripts/write-qemu-visible-operator-plan.sh" >/dev/null

AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${PLAN_FILE}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${PLAN_JSON}" \
    "${ROOT_DIR}/scripts/check-qemu-visible-operator-plan.sh" >/dev/null

AQUA_QEMU_VISIBLE_STATUS_FILE="${STATUS_FILE}" \
    AQUA_QEMU_VISIBLE_STATUS_JSON="${STATUS_JSON}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${PLAN_FILE}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${PLAN_JSON}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${PACKET_FILE}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${PACKET_JSON}" \
    AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE="${FIRST_GRAPHICS_SESSION_STATUS}" \
    AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON="${FIRST_GRAPHICS_SESSION_STATUS_JSON}" \
    "${ROOT_DIR}/scripts/write-qemu-visible-operator-packet.sh" >/dev/null

AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${PACKET_FILE}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${PACKET_JSON}" \
    "${ROOT_DIR}/scripts/check-qemu-visible-operator-packet.sh" >/dev/null

AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${PACKET_FILE}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${PACKET_JSON}" \
    AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST="${CHECKLIST}" \
    "${ROOT_DIR}/scripts/write-qemu-visible-operator-checklist.sh" >/dev/null

AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST="${CHECKLIST}" \
    "${ROOT_DIR}/scripts/check-qemu-visible-operator-checklist.sh" >/dev/null

mkdir -p "$(dirname "${PASS_FILE}")" "$(dirname "${PASS_JSON}")"

packet_value() {
    key="$1"
    python3 - "${PACKET_JSON}" "${key}" <<'PY'
import json
import sys

path, key = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as handle:
    packet = json.load(handle)

value = packet
for part in key.split("."):
    if not isinstance(value, dict) or part not in value:
        print("unknown")
        raise SystemExit(0)
    value = value[part]

print(value)
PY
}

PREFLIGHT_SOURCE_BYTES="$(packet_value source_status.preflight_source_bytes)"
PREFLIGHT_SOURCE_SHA256="$(packet_value source_status.preflight_source_sha256)"
PREFLIGHT_SOURCE_MTIME_UTC="$(packet_value source_status.preflight_source_mtime_utc)"
BUNDLE_CAPTURE_HASH_STATUS="$(packet_value source_status.bundle_capture_hash_status)"
BUNDLE_POSITIVE_CAPTURE_HASH_STATUS="$(packet_value source_status.bundle_positive_capture_hash_status)"
BUNDLE_MISSING_CAPTURE_HASH_REJECTED_STATUS="$(packet_value source_status.bundle_missing_capture_hash_rejected_status)"
MANUAL_RUNBOOK_PASS_REPORT_REQUIRED_STATUS="$(packet_value source_status.manual_runbook_pass_report_required_status)"
PASS_REPORT_STATUS="$(packet_value source_status.pass_report_status)"
PASS_REPORT_EVIDENCE_RECORDED_STATUS="$(packet_value source_status.pass_report_evidence_recorded_status)"
PASS_REPORT_EVIDENCE_RULE_STATUS="$(packet_value source_status.pass_report_evidence_rule_status)"
FIRST_GRAPHICS_SESSION_READY_STATUS="$(packet_value first_graphics_session_status)"

write_pass_artifacts() {
    pass_status="$1"
    launch_armed="$2"
    launch_skipped="$3"

    {
        echo "Aqua Linux QEMU visible operator pass status"
        echo "product=Aqua Linux"
        echo "base=Buildroot"
        echo "mode=host-qemu-visible-operator-pass"
        echo "target=QEMU x86_64"
        echo "status=${pass_status}"
        echo "operator_pass_launch_armed=${launch_armed}"
        echo "operator_pass_launch_skipped=${launch_skipped}"
        echo "boot_graphics=false"
        echo "autostart=false"
        echo "persistent_graphical_session_started=false"
        echo "operator_confirmation_required=true"
        echo "launch_confirmation_required=true"
        echo "launch_confirmed=${CONFIRM_LAUNCH}"
        echo "no_positive_observation_without_evidence=true"
        echo "no_unverified_bundle_acceptance=true"
        echo "capture_hash_verification_required=true"
        echo "first_graphics_session_status=${FIRST_GRAPHICS_SESSION_READY_STATUS}"
        echo "first_graphics_session_status_file=${FIRST_GRAPHICS_SESSION_STATUS}"
        echo "first_graphics_session_status_json=${FIRST_GRAPHICS_SESSION_STATUS_JSON}"
        echo "bundle_capture_hash_status=${BUNDLE_CAPTURE_HASH_STATUS}"
        echo "bundle_positive_capture_hash_status=${BUNDLE_POSITIVE_CAPTURE_HASH_STATUS}"
        echo "bundle_missing_capture_hash_rejected_status=${BUNDLE_MISSING_CAPTURE_HASH_REJECTED_STATUS}"
        echo "manual_runbook_pass_report_required_status=${MANUAL_RUNBOOK_PASS_REPORT_REQUIRED_STATUS}"
        echo "pass_report_status=${PASS_REPORT_STATUS}"
        echo "pass_report_evidence_recorded_status=${PASS_REPORT_EVIDENCE_RECORDED_STATUS}"
        echo "pass_report_evidence_rule_status=${PASS_REPORT_EVIDENCE_RULE_STATUS}"
        echo "safe_return_to_recovery=ok"
        echo "operator_checklist=${CHECKLIST}"
        echo "operator_packet=${PACKET_FILE}"
        echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
        echo "preflight_source_bytes=${PREFLIGHT_SOURCE_BYTES}"
        echo "preflight_source_sha256=${PREFLIGHT_SOURCE_SHA256}"
        echo "preflight_source_mtime_utc=${PREFLIGHT_SOURCE_MTIME_UTC}"
        echo "serial_log=${SERIAL_LOG}"
        echo "capture_dir=${CAPTURE_DIR}"
        echo "operator_pass_stop_rule=${PASS_STOP_RULE}"
        echo "no_launch_rehearsal_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh"
        echo "confirmed_launch_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh"
        echo "next_launch_command=scripts/run-qemu-visible-manual.sh"
        echo "next_vm_fbdev_command=AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present"
        echo "next_capture_flow_command=scripts/run-qemu-visible-ready-capture-flow.sh"
        echo "next_capture_verify_command=scripts/verify-qemu-visible-capture.sh"
        echo "next_evidence_flow_command=scripts/run-qemu-visible-evidence-flow.sh"
        echo "next_vm_apply_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply"
        echo "next_vm_report_command=aqua-qemu-visible-pass-report"
        echo "[AQUA-HOST] stage=qemu-visible-operator-pass-artifact status=${pass_status}"
    } > "${PASS_FILE}"

    export PASS_JSON PASS_FILE CHECKLIST PACKET_FILE PREFLIGHT_SUMMARY_JSON PREFLIGHT_SOURCE_BYTES PREFLIGHT_SOURCE_SHA256 PREFLIGHT_SOURCE_MTIME_UTC
    export BUNDLE_CAPTURE_HASH_STATUS BUNDLE_POSITIVE_CAPTURE_HASH_STATUS BUNDLE_MISSING_CAPTURE_HASH_REJECTED_STATUS
    export MANUAL_RUNBOOK_PASS_REPORT_REQUIRED_STATUS PASS_REPORT_STATUS PASS_REPORT_EVIDENCE_RECORDED_STATUS PASS_REPORT_EVIDENCE_RULE_STATUS
    export FIRST_GRAPHICS_SESSION_READY_STATUS FIRST_GRAPHICS_SESSION_STATUS FIRST_GRAPHICS_SESSION_STATUS_JSON
    export SERIAL_LOG CAPTURE_DIR PASS_STOP_RULE pass_status launch_armed launch_skipped CONFIRM_LAUNCH
    python3 - <<'PY'
import json
import os
from datetime import datetime, timezone

status = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-operator-pass",
    "target": "QEMU x86_64",
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "status": os.environ["pass_status"],
    "operator_pass_launch_armed": os.environ["launch_armed"] == "true",
    "operator_pass_launch_skipped": os.environ["launch_skipped"] == "true",
    "boot_graphics": False,
    "autostart": False,
    "persistent_graphical_session_started": False,
    "operator_confirmation_required": True,
    "launch_confirmation_required": True,
    "launch_confirmed": os.environ["CONFIRM_LAUNCH"] == "true",
    "operator_checklist": os.environ["CHECKLIST"],
    "operator_packet": os.environ["PACKET_FILE"],
    "operator_pass_file": os.environ["PASS_FILE"],
    "operator_pass_json": os.environ["PASS_JSON"],
    "preflight_summary_json": os.environ["PREFLIGHT_SUMMARY_JSON"],
    "preflight_source_bytes": os.environ["PREFLIGHT_SOURCE_BYTES"],
    "preflight_source_sha256": os.environ["PREFLIGHT_SOURCE_SHA256"],
    "preflight_source_mtime_utc": os.environ["PREFLIGHT_SOURCE_MTIME_UTC"],
    "serial_log": os.environ["SERIAL_LOG"],
    "capture_dir": os.environ["CAPTURE_DIR"],
    "no_positive_observation_without_evidence": True,
    "no_unverified_bundle_acceptance": True,
    "capture_hash_verification_required": True,
    "first_graphics_session_status": os.environ["FIRST_GRAPHICS_SESSION_READY_STATUS"],
    "first_graphics_session_status_file": os.environ["FIRST_GRAPHICS_SESSION_STATUS"],
    "first_graphics_session_status_json": os.environ["FIRST_GRAPHICS_SESSION_STATUS_JSON"],
    "bundle_capture_hash_status": os.environ["BUNDLE_CAPTURE_HASH_STATUS"],
    "bundle_positive_capture_hash_status": os.environ["BUNDLE_POSITIVE_CAPTURE_HASH_STATUS"],
    "bundle_missing_capture_hash_rejected_status": os.environ["BUNDLE_MISSING_CAPTURE_HASH_REJECTED_STATUS"],
    "manual_runbook_pass_report_required_status": os.environ["MANUAL_RUNBOOK_PASS_REPORT_REQUIRED_STATUS"],
    "pass_report_status": os.environ["PASS_REPORT_STATUS"],
    "pass_report_evidence_recorded_status": os.environ["PASS_REPORT_EVIDENCE_RECORDED_STATUS"],
    "pass_report_evidence_rule_status": os.environ["PASS_REPORT_EVIDENCE_RULE_STATUS"],
    "safe_return_to_recovery": "ok",
    "stop_rule": os.environ["PASS_STOP_RULE"],
    "next_commands": {
        "no_launch_rehearsal": "AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh",
        "confirmed_launch": "AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh",
        "launch": "scripts/run-qemu-visible-manual.sh",
        "vm_fbdev": "AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present",
        "capture_flow": "scripts/run-qemu-visible-ready-capture-flow.sh",
        "capture_verify": "scripts/verify-qemu-visible-capture.sh",
        "evidence_flow": "scripts/run-qemu-visible-evidence-flow.sh",
        "vm_apply": "AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply",
        "vm_report": "aqua-qemu-visible-pass-report",
    },
    "required_operator_sequence": [
        "Run the no-launch rehearsal and inspect this pass artifact.",
        "Set AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true only when ready to open QEMU.",
        "Run AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present in the VM recovery shell.",
        "Confirm the visible QEMU window manually.",
        "Run the ready capture flow and paste the generated evidence bundle into the VM recovery shell.",
        "Apply the evidence bundle with AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true only after visual confirmation.",
        "Run aqua-qemu-visible-pass-report in the VM recovery shell to summarize the bounded attempt, observation, and evidence.",
    ],
}

with open(os.environ["PASS_JSON"], "w", encoding="utf-8") as handle:
    json.dump(status, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

echo "operator_pass_ready=true"
echo "operator_pass_launch_armed=true"
echo "operator_checklist_ready=${CHECKLIST}"

if [ "${NO_LAUNCH}" = "true" ]; then
    write_pass_artifacts "no-launch-ready" "true" "true"
    echo "operator_pass_launch_skipped=true"
    echo "operator_pass_file=${PASS_FILE}"
    echo "operator_pass_json=${PASS_JSON}"
    echo "[AQUA-HOST] stage=qemu-visible-operator-pass status=no-launch-ready"
    exit 0
fi

if [ "${CONFIRM_LAUNCH}" != "true" ]; then
    write_pass_artifacts "blocked-launch-confirmation" "false" "true"
    echo "operator_pass_launch_armed=false"
    echo "operator_pass_launch_skipped=true"
    echo "operator_pass_blocked_reason=missing-explicit-launch-confirmation"
    echo "operator_pass_file=${PASS_FILE}"
    echo "operator_pass_json=${PASS_JSON}"
    echo "Set AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true to open QEMU after the no-launch rehearsal is ready." >&2
    echo "[AQUA-HOST] stage=qemu-visible-operator-pass status=blocked-launch-confirmation"
    exit 1
fi

write_pass_artifacts "launching" "true" "false"
echo "[AQUA-HOST] stage=qemu-visible-operator-pass status=launching"

exec "${ROOT_DIR}/scripts/run-qemu-visible-manual.sh"
