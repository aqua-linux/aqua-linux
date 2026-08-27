#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PASS_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PASS_FILE:-${ROOT_DIR}/build/qemu-visible-operator-pass.txt}"
PASS_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PASS_JSON:-${ROOT_DIR}/build/qemu-visible-operator-pass.json}"

if [ ! -f "${PASS_FILE}" ]; then
    echo "Missing QEMU visible operator pass text: ${PASS_FILE}" >&2
    echo "Run AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh first." >&2
    exit 1
fi

if [ ! -f "${PASS_JSON}" ]; then
    echo "Missing QEMU visible operator pass JSON: ${PASS_JSON}" >&2
    echo "Run AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh first." >&2
    exit 1
fi

grep -Fq 'mode=host-qemu-visible-operator-pass' "${PASS_FILE}"
grep -Fq 'status=no-launch-ready' "${PASS_FILE}"
grep -Fq 'operator_pass_launch_armed=true' "${PASS_FILE}"
grep -Fq 'operator_pass_launch_skipped=true' "${PASS_FILE}"
grep -Fq 'boot_graphics=false' "${PASS_FILE}"
grep -Fq 'autostart=false' "${PASS_FILE}"
grep -Fq 'persistent_graphical_session_started=false' "${PASS_FILE}"
grep -Fq 'launch_confirmation_required=true' "${PASS_FILE}"
grep -Fq 'launch_confirmed=false' "${PASS_FILE}"
grep -Fq 'no_positive_observation_without_evidence=true' "${PASS_FILE}"
grep -Fq 'no_unverified_bundle_acceptance=true' "${PASS_FILE}"
grep -Fq 'capture_hash_verification_required=true' "${PASS_FILE}"
grep -Fq 'first_graphics_session_status=ready-for-controlled-visible-attempt' "${PASS_FILE}"
grep -Fq 'first_graphics_session_status_file=' "${PASS_FILE}"
grep -Fq 'first_graphics_session_status_json=' "${PASS_FILE}"
grep -Fq 'bundle_capture_hash_status=ok' "${PASS_FILE}"
grep -Fq 'bundle_positive_capture_hash_status=ok' "${PASS_FILE}"
grep -Fq 'bundle_missing_capture_hash_rejected_status=ok' "${PASS_FILE}"
grep -Fq 'manual_runbook_pass_report_required_status=ok' "${PASS_FILE}"
grep -Fq 'pass_report_status=ok' "${PASS_FILE}"
grep -Fq 'pass_report_evidence_recorded_status=ok' "${PASS_FILE}"
grep -Fq 'pass_report_evidence_rule_status=ok' "${PASS_FILE}"
grep -Fq 'safe_return_to_recovery=ok' "${PASS_FILE}"
grep -Fq 'preflight_summary_json=' "${PASS_FILE}"
grep -Fq 'preflight_source_bytes=' "${PASS_FILE}"
grep -Fq 'preflight_source_sha256=' "${PASS_FILE}"
grep -Fq 'preflight_source_mtime_utc=' "${PASS_FILE}"
grep -Fq 'serial_log=' "${PASS_FILE}"
grep -Fq 'capture_dir=' "${PASS_FILE}"
grep -Fq 'operator_pass_stop_rule=Do not mark VM display observed' "${PASS_FILE}"
grep -Fq 'no_launch_rehearsal_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh' "${PASS_FILE}"
grep -Fq 'confirmed_launch_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh' "${PASS_FILE}"
grep -Fq 'next_launch_command=scripts/run-qemu-visible-manual.sh' "${PASS_FILE}"
grep -Fq 'next_vm_fbdev_command=AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present' "${PASS_FILE}"
grep -Fq 'next_capture_verify_command=scripts/verify-qemu-visible-capture.sh' "${PASS_FILE}"
grep -Fq 'next_evidence_flow_command=scripts/run-qemu-visible-evidence-flow.sh' "${PASS_FILE}"
grep -Fq 'next_vm_report_command=aqua-qemu-visible-pass-report' "${PASS_FILE}"
grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-pass-artifact status=no-launch-ready' "${PASS_FILE}"

python3 - "${PASS_JSON}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    status = json.load(handle)

errors = []
expected = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-operator-pass",
    "target": "QEMU x86_64",
    "status": "no-launch-ready",
}

for key, value in expected.items():
    if status.get(key) != value:
        errors.append(f"{key} must be {value!r}")

if status.get("operator_pass_launch_armed") is not True:
    errors.append("operator_pass_launch_armed must be true")
if status.get("operator_pass_launch_skipped") is not True:
    errors.append("operator_pass_launch_skipped must be true")
if status.get("boot_graphics") is not False:
    errors.append("boot_graphics must be false")
if status.get("autostart") is not False:
    errors.append("autostart must be false")
if status.get("persistent_graphical_session_started") is not False:
    errors.append("persistent_graphical_session_started must be false")
if status.get("operator_confirmation_required") is not True:
    errors.append("operator_confirmation_required must be true")
if status.get("launch_confirmation_required") is not True:
    errors.append("launch_confirmation_required must be true")
if status.get("launch_confirmed") is not False:
    errors.append("launch_confirmed must be false for no-launch pass")
if status.get("no_positive_observation_without_evidence") is not True:
    errors.append("no_positive_observation_without_evidence must be true")
if status.get("no_unverified_bundle_acceptance") is not True:
    errors.append("no_unverified_bundle_acceptance must be true")
if status.get("capture_hash_verification_required") is not True:
    errors.append("capture_hash_verification_required must be true")
if status.get("first_graphics_session_status") != "ready-for-controlled-visible-attempt":
    errors.append("first_graphics_session_status must be ready-for-controlled-visible-attempt")
for key in ("first_graphics_session_status_file", "first_graphics_session_status_json"):
    if not status.get(key):
        errors.append(f"{key} must be present")
for key in (
    "bundle_capture_hash_status",
    "bundle_positive_capture_hash_status",
    "bundle_missing_capture_hash_rejected_status",
    "manual_runbook_pass_report_required_status",
    "pass_report_status",
    "pass_report_evidence_recorded_status",
    "pass_report_evidence_rule_status",
):
    if status.get(key) != "ok":
        errors.append(f"{key} must be ok")
if status.get("safe_return_to_recovery") != "ok":
    errors.append("safe_return_to_recovery must be ok")

for key in ("preflight_summary_json", "serial_log", "capture_dir"):
    if not status.get(key):
        errors.append(f"{key} must be present")

if str(status.get("preflight_source_bytes", "")).strip() in ("", "unknown", "0"):
    errors.append("preflight_source_bytes must be populated")
sha256 = status.get("preflight_source_sha256")
if not isinstance(sha256, str) or len(sha256) != 64:
    errors.append("preflight_source_sha256 must be a sha256 hex digest")
mtime = status.get("preflight_source_mtime_utc", "")
if not isinstance(mtime, str) or "T" not in mtime or not mtime.endswith("Z"):
    errors.append("preflight_source_mtime_utc must be UTC ISO-like timestamp ending in Z")

stop_rule = status.get("stop_rule", "")
if "Do not mark VM display observed" not in stop_rule:
    errors.append("stop_rule must protect the observed marker")

next_commands = status.get("next_commands")
if not isinstance(next_commands, dict):
    errors.append("next_commands must be an object")
else:
    if next_commands.get("no_launch_rehearsal") != "AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh":
        errors.append("next_commands.no_launch_rehearsal changed")
    if next_commands.get("confirmed_launch") != "AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh":
        errors.append("next_commands.confirmed_launch changed")
    if next_commands.get("launch") != "scripts/run-qemu-visible-manual.sh":
        errors.append("next_commands.launch changed")
    if next_commands.get("vm_fbdev") != "AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present":
        errors.append("next_commands.vm_fbdev changed")
    if next_commands.get("capture_flow") != "scripts/run-qemu-visible-ready-capture-flow.sh":
        errors.append("next_commands.capture_flow changed")
    if next_commands.get("vm_report") != "aqua-qemu-visible-pass-report":
        errors.append("next_commands.vm_report changed")
    if next_commands.get("capture_verify") != "scripts/verify-qemu-visible-capture.sh":
        errors.append("next_commands.capture_verify changed")
    if next_commands.get("evidence_flow") != "scripts/run-qemu-visible-evidence-flow.sh":
        errors.append("next_commands.evidence_flow changed")
    if next_commands.get("vm_apply") != "AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply":
        errors.append("next_commands.vm_apply changed")

required_sequence = status.get("required_operator_sequence")
if not isinstance(required_sequence, list) or len(required_sequence) != 7:
    errors.append("required_operator_sequence must contain seven operator steps")
else:
    if "no-launch rehearsal" not in required_sequence[0]:
        errors.append("required_operator_sequence must start with no-launch rehearsal")
    if "AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true" not in required_sequence[1]:
        errors.append("required_operator_sequence must include explicit launch confirmation")
    if "AQUA_FBDEV_OPERATOR_CONFIRMED=true" not in required_sequence[2]:
        errors.append("required_operator_sequence must include fbdev presentation")
    if "aqua-qemu-visible-pass-report" not in required_sequence[6]:
        errors.append("required_operator_sequence must end with pass report")

generated_at = status.get("generated_at_utc", "")
if not isinstance(generated_at, str) or "T" not in generated_at or not generated_at.endswith("Z"):
    errors.append("generated_at_utc must be UTC ISO-like timestamp ending in Z")

if not status.get("operator_checklist"):
    errors.append("operator_checklist must be present")
if not status.get("operator_packet"):
    errors.append("operator_packet must be present")

if errors:
    for error in errors:
        print(f"qemu visible operator pass error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux QEMU visible operator pass checks passed.")
PY
