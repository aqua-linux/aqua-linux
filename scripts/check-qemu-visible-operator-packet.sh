#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PACKET_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE:-${ROOT_DIR}/build/qemu-visible-operator-packet.txt}"
PACKET_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON:-${ROOT_DIR}/build/qemu-visible-operator-packet.json}"

if [ ! -f "${PACKET_FILE}" ]; then
    echo "Missing QEMU visible operator packet text: ${PACKET_FILE}" >&2
    echo "Run scripts/write-qemu-visible-operator-packet.sh first." >&2
    exit 1
fi

if [ ! -f "${PACKET_JSON}" ]; then
    echo "Missing QEMU visible operator packet JSON: ${PACKET_JSON}" >&2
    echo "Run scripts/write-qemu-visible-operator-packet.sh first." >&2
    exit 1
fi

grep -Fq 'mode=host-qemu-visible-operator-packet' "${PACKET_FILE}"
grep -Fq 'packet_status=ready' "${PACKET_FILE}"
grep -Fq 'next_required_action=operator-run-manual-qemu-pass' "${PACKET_FILE}"
grep -Fq 'boot_graphics=false' "${PACKET_FILE}"
grep -Fq 'autostart=false' "${PACKET_FILE}"
grep -Fq 'operator_confirmation_required=true' "${PACKET_FILE}"
grep -Fq 'no_positive_observation_without_evidence=true' "${PACKET_FILE}"
grep -Fq 'capture_hash_verification_required=true' "${PACKET_FILE}"
grep -Fq 'first_graphics_session_status=ready-for-controlled-visible-attempt' "${PACKET_FILE}"
grep -Fq 'first_graphics_session_failed_check_count=0' "${PACKET_FILE}"
grep -Fq 'bundle_capture_hash_status=ok' "${PACKET_FILE}"
grep -Fq 'bundle_positive_capture_hash_status=ok' "${PACKET_FILE}"
grep -Fq 'bundle_missing_capture_hash_rejected_status=ok' "${PACKET_FILE}"
grep -Fq 'manual_runbook_pass_report_required_status=ok' "${PACKET_FILE}"
grep -Fq 'pass_report_status=ok' "${PACKET_FILE}"
grep -Fq 'pass_report_evidence_recorded_status=ok' "${PACKET_FILE}"
grep -Fq 'pass_report_evidence_rule_status=ok' "${PACKET_FILE}"
grep -Fq 'preflight_source_sha256=' "${PACKET_FILE}"
grep -Fq 'artifact_operator_plan_json_status=ready' "${PACKET_FILE}"
grep -Fq 'artifact_status_json_status=ready' "${PACKET_FILE}"
grep -Fq 'artifact_first_graphics_session_status_text_status=ready' "${PACKET_FILE}"
grep -Fq 'artifact_first_graphics_session_status_json_status=ready' "${PACKET_FILE}"
grep -Fq 'step_count=9' "${PACKET_FILE}"
grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-packet status=ok' "${PACKET_FILE}"

python3 - "${PACKET_JSON}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    packet = json.load(handle)

errors = []
expected = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-operator-packet",
    "target": "QEMU x86_64",
    "status": "ready-for-operator-pass",
    "packet_status": "ready",
    "next_required_action": "operator-run-manual-qemu-pass",
}

for key, value in expected.items():
    if packet.get(key) != value:
        errors.append(f"{key} must be {value!r}")

if packet.get("missing_artifacts") != []:
    errors.append("missing_artifacts must be empty")
if packet.get("first_graphics_session_status") != "ready-for-controlled-visible-attempt":
    errors.append("first_graphics_session_status must be ready-for-controlled-visible-attempt")
if packet.get("first_graphics_session_failed_checks") != []:
    errors.append("first_graphics_session_failed_checks must be empty")

safe = packet.get("safe_defaults")
if not isinstance(safe, dict):
    errors.append("safe_defaults must be an object")
else:
    if safe.get("boot_graphics") is not False:
        errors.append("safe_defaults.boot_graphics must be false")
    if safe.get("autostart") is not False:
        errors.append("safe_defaults.autostart must be false")
    if safe.get("persistent_graphical_session_started") is not False:
        errors.append("safe_defaults.persistent_graphical_session_started must be false")

gates = packet.get("operator_gates")
if not isinstance(gates, dict):
    errors.append("operator_gates must be an object")
else:
    for key in (
        "visual_confirmation_required",
        "manual_observation_required",
        "no_positive_observation_without_evidence",
        "no_unverified_bundle_acceptance",
        "capture_hash_verification_required",
        "operator_confirmation_required",
        "pass_report_required",
        "fbdev_frame_required",
    ):
        if gates.get(key) is not True:
            errors.append(f"operator_gates.{key} must be true")

source_status = packet.get("source_status")
if not isinstance(source_status, dict):
    errors.append("source_status must be an object")
else:
    if str(source_status.get("preflight_source_bytes", "")).strip() in ("", "unknown", "0"):
        errors.append("source_status.preflight_source_bytes must be populated")
    for key in (
        "bundle_capture_hash_status",
        "bundle_positive_capture_hash_status",
        "bundle_missing_capture_hash_rejected_status",
        "manual_runbook_pass_report_required_status",
        "pass_report_status",
        "pass_report_observed_status",
        "pass_report_attempt_completed_status",
        "pass_report_evidence_recorded_status",
        "pass_report_evidence_rule_status",
    ):
        if source_status.get(key) != "ok":
            errors.append(f"source_status.{key} must be ok")
    sha256 = source_status.get("preflight_source_sha256")
    if not isinstance(sha256, str) or len(sha256) != 64:
        errors.append("source_status.preflight_source_sha256 must be a sha256 hex digest")
    mtime = source_status.get("preflight_source_mtime_utc", "")
    if not isinstance(mtime, str) or "T" not in mtime or not mtime.endswith("Z"):
        errors.append("source_status.preflight_source_mtime_utc must be UTC ISO-like timestamp ending in Z")

artifacts = packet.get("artifacts")
if not isinstance(artifacts, dict):
    errors.append("artifacts must be an object")
else:
    for name in (
        "status_text",
        "status_json",
        "operator_plan_text",
        "operator_plan_json",
        "boot_summary_json",
        "image_manifest_json",
        "first_graphics_session_status_text",
        "first_graphics_session_status_json",
    ):
        artifact = artifacts.get(name)
        if not isinstance(artifact, dict):
            errors.append(f"artifacts.{name} must be an object")
            continue
        if artifact.get("status") != "ready":
            errors.append(f"artifacts.{name}.status must be ready")
        if not isinstance(artifact.get("sha256"), str) or len(artifact["sha256"]) != 64:
            errors.append(f"artifacts.{name}.sha256 must be a sha256 hex digest")
        if not artifact.get("bytes"):
            errors.append(f"artifacts.{name}.bytes must be non-zero")

steps = packet.get("steps")
if not isinstance(steps, list) or len(steps) != 9:
    errors.append("steps must contain nine ordered actions")
else:
    if steps[2].get("command") != "scripts/run-qemu-visible-manual.sh":
        errors.append("launch step must run scripts/run-qemu-visible-manual.sh")
    if steps[3].get("command") != "AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present":
        errors.append("fbdev presentation command changed")
    if steps[7].get("command") != "AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply":
        errors.append("final VM apply command changed")
    if steps[8].get("command") != "aqua-qemu-visible-pass-report":
        errors.append("final VM pass report command changed")

stop_rule = packet.get("stop_rule", "")
if "Do not mark VM display observed" not in stop_rule:
    errors.append("stop_rule must protect the observed marker")

generated_at = packet.get("generated_at_utc", "")
if not isinstance(generated_at, str) or "T" not in generated_at or not generated_at.endswith("Z"):
    errors.append("generated_at_utc must be UTC ISO-like timestamp ending in Z")

if errors:
    for error in errors:
        print(f"qemu visible operator packet error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux QEMU visible operator packet checks passed.")
PY
