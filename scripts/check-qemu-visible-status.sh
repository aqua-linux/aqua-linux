#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
STATUS_FILE="${AQUA_QEMU_VISIBLE_STATUS_FILE:-${ROOT_DIR}/build/qemu-visible-status.txt}"
STATUS_JSON="${AQUA_QEMU_VISIBLE_STATUS_JSON:-${ROOT_DIR}/build/qemu-visible-status.json}"

if [ ! -f "${STATUS_FILE}" ]; then
    echo "Missing QEMU visible status text: ${STATUS_FILE}" >&2
    echo "Run scripts/qemu-visible-status.sh first." >&2
    exit 1
fi

if [ ! -f "${STATUS_JSON}" ]; then
    echo "Missing QEMU visible status JSON: ${STATUS_JSON}" >&2
    echo "Run scripts/qemu-visible-status.sh first." >&2
    exit 1
fi

grep -Fq 'qemu_visible_manual_status=ready-for-operator-pass' "${STATUS_FILE}"
grep -Fq 'preflight_source_bytes=' "${STATUS_FILE}"
grep -Fq 'preflight_source_sha256=' "${STATUS_FILE}"
grep -Fq 'preflight_source_mtime_utc=' "${STATUS_FILE}"
grep -Fq 'bundle_capture_hash_status=ok' "${STATUS_FILE}"
grep -Fq 'bundle_positive_capture_hash_status=ok' "${STATUS_FILE}"
grep -Fq 'bundle_missing_capture_hash_rejected_status=ok' "${STATUS_FILE}"
grep -Fq 'manual_runbook_pass_report_required_status=ok' "${STATUS_FILE}"
grep -Fq 'pass_report_status=ok' "${STATUS_FILE}"
grep -Fq 'pass_report_evidence_recorded_status=ok' "${STATUS_FILE}"
grep -Fq 'pass_report_evidence_rule_status=ok' "${STATUS_FILE}"
grep -Fq 'capture_hash_verification_required=true' "${STATUS_FILE}"
grep -Fq 'next_vm_report_command=aqua-qemu-visible-pass-report' "${STATUS_FILE}"
grep -Fq '[AQUA-HOST] stage=qemu-visible-status status=ok' "${STATUS_FILE}"

python3 - "${STATUS_JSON}" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as handle:
    status = json.load(handle)

errors = []
expected = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-status",
    "target": "QEMU x86_64",
    "preflight_summary_status": "ok",
    "manual_runbook_status": "ok",
    "ready_capture_flow_status": "ok",
    "manual_runbook_pass_report_required_status": "ok",
    "pass_report_status": "ok",
    "pass_report_observed_status": "ok",
    "pass_report_attempt_completed_status": "ok",
    "pass_report_evidence_recorded_status": "ok",
    "pass_report_evidence_rule_status": "ok",
    "bundle_apply_status": "ok",
    "bundle_positive_status": "ok",
    "bundle_rejected_status": "ok",
    "bundle_preflight_status": "ok",
    "bundle_capture_hash_status": "ok",
    "bundle_positive_preflight_status": "ok",
    "bundle_positive_capture_hash_status": "ok",
    "bundle_missing_unverified_status": "ok",
    "bundle_missing_capture_hash_rejected_status": "ok",
    "desktop_shell": "not_started",
    "qemu_visible_manual_status": "ready-for-operator-pass",
    "host_status": "ok",
}

for key, value in expected.items():
    if status.get(key) != value:
        errors.append(f"{key} must be {value!r}")

if status.get("boot_graphics") is not False:
    errors.append("boot_graphics must be false")
if status.get("autostart") is not False:
    errors.append("autostart must be false")
if status.get("operator_confirmation_required") is not True:
    errors.append("operator_confirmation_required must be true")
if status.get("manual_observation_required") is not True:
    errors.append("manual_observation_required must be true")
if status.get("persistent_graphical_session_started") is not False:
    errors.append("persistent_graphical_session_started must be false")
if status.get("capture_hash_verification_required") is not True:
    errors.append("capture_hash_verification_required must be true")

if str(status.get("preflight_source_bytes", "")).strip() in ("", "unknown", "0"):
    errors.append("preflight_source_bytes must be populated")
sha256 = status.get("preflight_source_sha256")
if not isinstance(sha256, str) or len(sha256) != 64:
    errors.append("preflight_source_sha256 must be a sha256 hex digest")
mtime = status.get("preflight_source_mtime_utc", "")
if not isinstance(mtime, str) or "T" not in mtime or not mtime.endswith("Z"):
    errors.append("preflight_source_mtime_utc must be UTC ISO-like timestamp ending in Z")

generated_at = status.get("generated_at_utc", "")
if not isinstance(generated_at, str) or "T" not in generated_at or not generated_at.endswith("Z"):
    errors.append("generated_at_utc must be UTC ISO-like timestamp ending in Z")

next_commands = status.get("next_commands")
if not isinstance(next_commands, dict):
    errors.append("next_commands must be an object")
else:
    if next_commands.get("launch") != "scripts/run-qemu-visible-manual.sh":
        errors.append("next_commands.launch must point to manual QEMU launch")
    if next_commands.get("capture_flow") != "scripts/run-qemu-visible-ready-capture-flow.sh":
        errors.append("next_commands.capture_flow must point to ready capture flow")

artifacts = status.get("artifacts")
if not isinstance(artifacts, dict):
    errors.append("artifacts must be an object")
else:
    for key in ("runbook_file", "bundle_apply_file", "bundle_positive_file", "bundle_rejected_file", "bundle_hash_rejected_file", "pass_report_file"):
        if not artifacts.get(key):
            errors.append(f"artifacts.{key} must be present")

if errors:
    for error in errors:
        print(f"qemu visible status error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux QEMU visible status checks passed.")
PY
