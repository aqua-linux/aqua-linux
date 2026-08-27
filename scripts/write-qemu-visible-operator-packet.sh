#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
STATUS_FILE="${AQUA_QEMU_VISIBLE_STATUS_FILE:-${ROOT_DIR}/build/qemu-visible-status.txt}"
STATUS_JSON="${AQUA_QEMU_VISIBLE_STATUS_JSON:-${ROOT_DIR}/build/qemu-visible-status.json}"
PLAN_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE:-${ROOT_DIR}/build/qemu-visible-operator-plan.txt}"
PLAN_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON:-${ROOT_DIR}/build/qemu-visible-operator-plan.json}"
PACKET_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE:-${ROOT_DIR}/build/qemu-visible-operator-packet.txt}"
PACKET_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON:-${ROOT_DIR}/build/qemu-visible-operator-packet.json}"
BOOT_SUMMARY_JSON="${AQUA_BOOT_SUMMARY_JSON:-${ROOT_DIR}/build/aqua-boot-summary.json}"
IMAGE_MANIFEST_JSON="${AQUA_IMAGE_MANIFEST_JSON:-${ROOT_DIR}/build/aqua-image-manifest.json}"
FIRST_GRAPHICS_SESSION_STATUS="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE:-${ROOT_DIR}/build/first-graphics-session-status.txt}"
FIRST_GRAPHICS_SESSION_STATUS_JSON="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON:-${ROOT_DIR}/build/first-graphics-session-status.json}"

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

AQUA_BOOT_SUMMARY_JSON="${BOOT_SUMMARY_JSON}" \
    AQUA_IMAGE_MANIFEST_JSON="${IMAGE_MANIFEST_JSON}" \
    AQUA_QEMU_VISIBLE_STATUS_JSON="${STATUS_JSON}" \
    AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE="${FIRST_GRAPHICS_SESSION_STATUS}" \
    AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON="${FIRST_GRAPHICS_SESSION_STATUS_JSON}" \
    "${ROOT_DIR}/scripts/first-graphics-session-status.sh" >/dev/null

AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE="${FIRST_GRAPHICS_SESSION_STATUS}" \
    AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON="${FIRST_GRAPHICS_SESSION_STATUS_JSON}" \
    "${ROOT_DIR}/scripts/check-first-graphics-session-status.sh" >/dev/null

mkdir -p "$(dirname "${PACKET_FILE}")" "$(dirname "${PACKET_JSON}")"

export STATUS_FILE STATUS_JSON PLAN_FILE PLAN_JSON PACKET_FILE PACKET_JSON BOOT_SUMMARY_JSON IMAGE_MANIFEST_JSON
export FIRST_GRAPHICS_SESSION_STATUS FIRST_GRAPHICS_SESSION_STATUS_JSON
python3 - <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone

def file_entry(path):
    if not os.path.isfile(path):
        return {
            "path": path,
            "status": "missing",
            "bytes": 0,
            "sha256": None,
        }
    digest = hashlib.sha256()
    size = 0
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            size += len(chunk)
            digest.update(chunk)
    return {
        "path": path,
        "status": "ready",
        "bytes": size,
        "sha256": digest.hexdigest(),
    }

with open(os.environ["PLAN_JSON"], "r", encoding="utf-8") as handle:
    plan = json.load(handle)

with open(os.environ["STATUS_JSON"], "r", encoding="utf-8") as handle:
    status = json.load(handle)

with open(os.environ["FIRST_GRAPHICS_SESSION_STATUS_JSON"], "r", encoding="utf-8") as handle:
    first_graphics = json.load(handle)

artifact_paths = {
    "status_text": os.environ["STATUS_FILE"],
    "status_json": os.environ["STATUS_JSON"],
    "operator_plan_text": os.environ["PLAN_FILE"],
    "operator_plan_json": os.environ["PLAN_JSON"],
    "boot_summary_json": os.environ["BOOT_SUMMARY_JSON"],
    "image_manifest_json": os.environ["IMAGE_MANIFEST_JSON"],
    "first_graphics_session_status_text": os.environ["FIRST_GRAPHICS_SESSION_STATUS"],
    "first_graphics_session_status_json": os.environ["FIRST_GRAPHICS_SESSION_STATUS_JSON"],
}

artifacts = {name: file_entry(path) for name, path in artifact_paths.items()}

packet = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-operator-packet",
    "target": "QEMU x86_64",
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "status": plan["status"],
    "next_required_action": plan["next_required_action"],
    "safe_defaults": plan["safe_defaults"],
    "operator_gates": plan["operator_gates"],
    "source_status": {
        "status": status["qemu_visible_manual_status"],
        "host_status": status["host_status"],
        "preflight_summary_status": status["preflight_summary_status"],
        "preflight_source_bytes": status["preflight_source_bytes"],
        "preflight_source_sha256": status["preflight_source_sha256"],
        "preflight_source_mtime_utc": status["preflight_source_mtime_utc"],
        "manual_runbook_pass_report_required_status": status["manual_runbook_pass_report_required_status"],
        "pass_report_status": status["pass_report_status"],
        "pass_report_observed_status": status["pass_report_observed_status"],
        "pass_report_attempt_completed_status": status["pass_report_attempt_completed_status"],
        "pass_report_evidence_recorded_status": status["pass_report_evidence_recorded_status"],
        "pass_report_evidence_rule_status": status["pass_report_evidence_rule_status"],
        "bundle_capture_hash_status": status["bundle_capture_hash_status"],
        "bundle_positive_capture_hash_status": status["bundle_positive_capture_hash_status"],
        "bundle_missing_unverified_status": status["bundle_missing_unverified_status"],
        "bundle_missing_capture_hash_rejected_status": status["bundle_missing_capture_hash_rejected_status"],
    },
    "first_graphics_session_status": first_graphics["status"],
    "first_graphics_session_failed_checks": first_graphics["failed_checks"],
    "artifacts": artifacts,
    "steps": plan["steps"],
    "stop_rule": "Do not mark VM display observed unless a visible QEMU window has been confirmed and evidence bundle apply is run with AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true.",
}

missing = [name for name, artifact in artifacts.items() if artifact["status"] != "ready"]
packet["packet_status"] = "ready" if not missing else "incomplete"
packet["missing_artifacts"] = missing

with open(os.environ["PACKET_JSON"], "w", encoding="utf-8") as handle:
    json.dump(packet, handle, indent=2, sort_keys=True)
    handle.write("\n")

lines = [
    "Aqua Linux QEMU visible operator packet",
    "product=Aqua Linux",
    "base=Buildroot",
    "mode=host-qemu-visible-operator-packet",
    "target=QEMU x86_64",
    f"packet_file={os.environ['PACKET_FILE']}",
    f"packet_json={os.environ['PACKET_JSON']}",
    f"status={packet['status']}",
    f"packet_status={packet['packet_status']}",
    f"next_required_action={packet['next_required_action']}",
    "boot_graphics=false",
    "autostart=false",
    "persistent_graphical_session_started=false",
    "operator_confirmation_required=true",
    "no_positive_observation_without_evidence=true",
    "no_unverified_bundle_acceptance=true",
    "capture_hash_verification_required=true",
    f"first_graphics_session_status={packet['first_graphics_session_status']}",
    f"first_graphics_session_failed_check_count={len(packet['first_graphics_session_failed_checks'])}",
    f"manual_runbook_pass_report_required_status={packet['source_status']['manual_runbook_pass_report_required_status']}",
    f"bundle_capture_hash_status={packet['source_status']['bundle_capture_hash_status']}",
    f"bundle_positive_capture_hash_status={packet['source_status']['bundle_positive_capture_hash_status']}",
    f"bundle_missing_capture_hash_rejected_status={packet['source_status']['bundle_missing_capture_hash_rejected_status']}",
    f"pass_report_status={packet['source_status']['pass_report_status']}",
    f"pass_report_evidence_recorded_status={packet['source_status']['pass_report_evidence_recorded_status']}",
    f"pass_report_evidence_rule_status={packet['source_status']['pass_report_evidence_rule_status']}",
    f"preflight_source_bytes={packet['source_status']['preflight_source_bytes']}",
    f"preflight_source_sha256={packet['source_status']['preflight_source_sha256']}",
    f"preflight_source_mtime_utc={packet['source_status']['preflight_source_mtime_utc']}",
    "",
]

for name in sorted(artifacts):
    artifact = artifacts[name]
    lines.append(f"artifact_{name}_status={artifact['status']}")
    lines.append(f"artifact_{name}_bytes={artifact['bytes']}")
    lines.append(f"artifact_{name}_sha256={artifact['sha256']}")
    lines.append(f"artifact_{name}_path={artifact['path']}")

lines.extend([
    "",
    f"step_count={len(packet['steps'])}",
    f"stop_rule={packet['stop_rule']}",
    "[AQUA-HOST] stage=qemu-visible-operator-packet status={}".format("ok" if packet["packet_status"] == "ready" else "blocked"),
])

with open(os.environ["PACKET_FILE"], "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
    handle.write("\n")

print("\n".join(lines))
PY
