#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
STATUS_FILE="${AQUA_QEMU_VISIBLE_STATUS_FILE:-${ROOT_DIR}/build/qemu-visible-status.txt}"
STATUS_JSON="${AQUA_QEMU_VISIBLE_STATUS_JSON:-${ROOT_DIR}/build/qemu-visible-status.json}"
PLAN_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE:-${ROOT_DIR}/build/qemu-visible-operator-plan.txt}"
PLAN_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON:-${ROOT_DIR}/build/qemu-visible-operator-plan.json}"

AQUA_QEMU_VISIBLE_STATUS_FILE="${STATUS_FILE}" \
    AQUA_QEMU_VISIBLE_STATUS_JSON="${STATUS_JSON}" \
    "${ROOT_DIR}/scripts/check-qemu-visible-status.sh" >/dev/null

mkdir -p "$(dirname "${PLAN_FILE}")" "$(dirname "${PLAN_JSON}")"

export STATUS_FILE STATUS_JSON PLAN_FILE PLAN_JSON
python3 - <<'PY'
import json
import os
from datetime import datetime, timezone

status_path = os.environ["STATUS_JSON"]
with open(status_path, "r", encoding="utf-8") as handle:
    source = json.load(handle)

steps = [
    {
        "id": "host-preflight",
        "side": "host",
        "command": source["next_commands"]["preflight"],
        "artifact": "build/qemu-visible-manual-preflight.txt",
        "requires_operator_confirmation": False,
    },
    {
        "id": "host-preflight-summary",
        "side": "host",
        "command": source["next_commands"]["preflight_summary"],
        "artifact": "build/qemu-visible-manual-preflight.json",
        "requires_operator_confirmation": False,
    },
    {
        "id": "host-launch-qemu",
        "side": "host",
        "command": source["next_commands"]["launch"],
        "artifact": "build/qemu-visible-manual-serial.log",
        "requires_operator_confirmation": False,
    },
    {
        "id": "vm-present-fbdev-frame",
        "side": "vm",
        "command": "AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present",
        "artifact": "visible QEMU framebuffer Aqua frame",
        "requires_operator_confirmation": True,
    },
    {
        "id": "operator-visible-confirmation",
        "side": "operator",
        "command": "Confirm the QEMU VM window is visible and recovery shell markers are present before capture.",
        "artifact": "operator visual observation",
        "requires_operator_confirmation": True,
    },
    {
        "id": "host-ready-capture-flow",
        "side": "host",
        "command": source["next_commands"]["capture_flow"],
        "artifact": "build/qemu-visible-evidence-bundle.txt",
        "requires_operator_confirmation": True,
    },
    {
        "id": "vm-paste-evidence-bundle",
        "side": "vm",
        "command": "Paste the heredoc printed by scripts/run-qemu-visible-ready-capture-flow.sh into the recovery shell.",
        "artifact": "/run/aqua/qemu-visible-evidence-bundle.txt",
        "requires_operator_confirmation": True,
    },
    {
        "id": "vm-apply-observed-marker",
        "side": "vm",
        "command": source["next_commands"]["vm_apply"],
        "artifact": "/run/aqua/graphics-qemu-observation-marker.txt",
        "requires_operator_confirmation": True,
    },
    {
        "id": "vm-write-pass-report",
        "side": "vm",
        "command": source["next_commands"]["vm_report"],
        "artifact": "/run/aqua/qemu-visible-pass-report.plan",
        "requires_operator_confirmation": True,
    },
]

plan = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-operator-plan",
    "target": "QEMU x86_64",
    "status": source["qemu_visible_manual_status"],
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "source_status_file": os.environ["STATUS_FILE"],
    "source_status_json": status_path,
    "plan_file": os.environ["PLAN_FILE"],
    "plan_json": os.environ["PLAN_JSON"],
    "safe_defaults": {
        "boot_graphics": source["boot_graphics"],
        "autostart": source["autostart"],
        "desktop_shell": source["desktop_shell"],
        "persistent_graphical_session_started": source["persistent_graphical_session_started"],
    },
    "operator_gates": {
        "visual_confirmation_required": True,
        "manual_observation_required": True,
        "no_positive_observation_without_evidence": True,
        "no_unverified_bundle_acceptance": True,
        "capture_hash_verification_required": True,
        "operator_confirmation_required": True,
        "pass_report_required": True,
        "fbdev_frame_required": True,
    },
    "steps": steps,
    "next_required_action": "operator-run-manual-qemu-pass",
}

with open(os.environ["PLAN_JSON"], "w", encoding="utf-8") as handle:
    json.dump(plan, handle, indent=2, sort_keys=True)
    handle.write("\n")

lines = [
    "Aqua Linux QEMU visible operator plan",
    "product=Aqua Linux",
    "base=Buildroot",
    "mode=host-qemu-visible-operator-plan",
    "target=QEMU x86_64",
    f"source_status_json={status_path}",
    f"plan_file={os.environ['PLAN_FILE']}",
    f"plan_json={os.environ['PLAN_JSON']}",
    f"status={plan['status']}",
    "boot_graphics=false",
    "autostart=false",
    "desktop_shell=not_started",
    "persistent_graphical_session_started=false",
    "visual_confirmation_required=true",
    "no_positive_observation_without_evidence=true",
    "no_unverified_bundle_acceptance=true",
    "capture_hash_verification_required=true",
    "next_required_action=operator-run-manual-qemu-pass",
    "",
]

for index, step in enumerate(steps, 1):
    lines.extend([
        f"step_{index}_id={step['id']}",
        f"step_{index}_side={step['side']}",
        f"step_{index}_command={step['command']}",
        f"step_{index}_artifact={step['artifact']}",
        f"step_{index}_operator_confirmation_required={str(step['requires_operator_confirmation']).lower()}",
    ])

lines.append("[AQUA-HOST] stage=qemu-visible-operator-plan status=ok")

with open(os.environ["PLAN_FILE"], "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
    handle.write("\n")

print("\n".join(lines))
PY
