#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PACKET_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE:-${ROOT_DIR}/build/qemu-visible-operator-packet.txt}"
PACKET_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON:-${ROOT_DIR}/build/qemu-visible-operator-packet.json}"
CHECKLIST="${AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST:-${ROOT_DIR}/build/qemu-visible-operator-checklist.md}"

AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${PACKET_FILE}" \
    AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${PACKET_JSON}" \
    "${ROOT_DIR}/scripts/check-qemu-visible-operator-packet.sh" >/dev/null

mkdir -p "$(dirname "${CHECKLIST}")"

export PACKET_JSON CHECKLIST
python3 - <<'PY'
import json
import os
from datetime import datetime, timezone

with open(os.environ["PACKET_JSON"], "r", encoding="utf-8") as handle:
    packet = json.load(handle)

lines = [
    "# Aqua Linux QEMU Visible Operator Checklist",
    "",
    f"- Product: {packet['product']}",
    f"- Base: {packet['base']}",
    f"- Target: {packet['target']}",
    f"- Status: `{packet['status']}`",
    f"- Packet status: `{packet['packet_status']}`",
    f"- Generated: `{datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace('+00:00', 'Z')}`",
    "",
    "## Safety Gates",
    "",
    "- [ ] Confirm `boot_graphics=false`.",
    "- [ ] Confirm `autostart=false`.",
    "- [ ] Confirm `persistent_graphical_session_started=false`.",
    "- [ ] Do not mark the VM display observed before a real visible QEMU window is confirmed.",
    "- [ ] Do not apply an evidence bundle unless it was produced from the checked preflight summary.",
    "- [ ] Confirm `capture_hash_verification_required=true`.",
    "- [ ] Confirm `bundle_capture_hash_status=ok`.",
    "- [ ] Confirm `bundle_missing_capture_hash_rejected_status=ok`.",
    "- [ ] Confirm `manual_runbook_pass_report_required_status=ok`.",
    "- [ ] Confirm `pass_report_status=ok`.",
    "",
    "## Hash Gates",
    "",
    f"- Capture hash verification required: `{str(packet['operator_gates']['capture_hash_verification_required']).lower()}`",
    f"- Bundle capture hash status: `{packet['source_status']['bundle_capture_hash_status']}`",
    f"- Positive bundle capture hash status: `{packet['source_status']['bundle_positive_capture_hash_status']}`",
    f"- Missing capture hash rejection status: `{packet['source_status']['bundle_missing_capture_hash_rejected_status']}`",
    "",
    "## Pass Report Gates",
    "",
    f"- Manual runbook pass report required status: `{packet['source_status']['manual_runbook_pass_report_required_status']}`",
    f"- Pass report status: `{packet['source_status']['pass_report_status']}`",
    f"- Pass report observed status: `{packet['source_status']['pass_report_observed_status']}`",
    f"- Pass report attempt completed status: `{packet['source_status']['pass_report_attempt_completed_status']}`",
    f"- Pass report evidence recorded status: `{packet['source_status']['pass_report_evidence_recorded_status']}`",
    f"- Pass report evidence rule status: `{packet['source_status']['pass_report_evidence_rule_status']}`",
    "",
    "## Host Steps",
    "",
]

for index, step in enumerate(packet["steps"], 1):
    command = step["command"]
    side = step["side"]
    artifact = step["artifact"]
    requires_confirmation = str(step["requires_operator_confirmation"]).lower()
    lines.extend(
        [
            f"{index}. [{side}] `{step['id']}`",
            f"   - Command: `{command}`",
            f"   - Artifact: `{artifact}`",
            f"   - Operator confirmation required: `{requires_confirmation}`",
            "",
        ]
    )

lines.extend(
    [
        "## Artifact Fingerprints",
        "",
    ]
)

for name in sorted(packet["artifacts"]):
    artifact = packet["artifacts"][name]
    lines.extend(
        [
            f"- `{name}`",
            f"  - Status: `{artifact['status']}`",
            f"  - Bytes: `{artifact['bytes']}`",
            f"  - SHA-256: `{artifact['sha256']}`",
            f"  - Path: `{artifact['path']}`",
        ]
    )

lines.extend(
    [
        "",
        "## Stop Rule",
        "",
        packet["stop_rule"],
        "",
        "`[AQUA-HOST] stage=qemu-visible-operator-checklist status=ok`",
        "",
    ]
)

with open(os.environ["CHECKLIST"], "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))

print(f"Aqua Linux QEMU visible operator checklist written: {os.environ['CHECKLIST']}")
print("[AQUA-HOST] stage=qemu-visible-operator-checklist status=ok")
PY
