#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PREFLIGHT_FILE="${AQUA_QEMU_VISIBLE_PREFLIGHT_FILE:-${ROOT_DIR}/build/qemu-visible-manual-preflight.txt}"
SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"

if [ ! -f "${PREFLIGHT_FILE}" ]; then
    echo "Missing QEMU visible preflight file: ${PREFLIGHT_FILE}" >&2
    echo "Run scripts/preflight-qemu-visible-manual.sh first." >&2
    exit 1
fi

export PREFLIGHT_FILE SUMMARY_JSON

python3 - <<'PY'
import hashlib
import json
import os
from datetime import datetime, timezone

preflight_file = os.environ["PREFLIGHT_FILE"]
summary_json = os.environ["SUMMARY_JSON"]

def file_entry(path):
    stat = os.stat(path)
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return {
        "path": path,
        "bytes": stat.st_size,
        "mtime_utc": datetime.fromtimestamp(stat.st_mtime, timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "sha256": digest.hexdigest(),
    }

fields = {}
with open(preflight_file, "r", encoding="utf-8", errors="replace") as handle:
    for raw_line in handle:
        line = raw_line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        fields[key] = value

required_ready = [
    "kernel_status",
    "rootfs_status",
    "rootfs_tar_status",
    "qemu_status",
]
required_present = [
    "host_run_script",
    "host_readiness_watch_script",
    "host_capture_script",
    "host_ready_capture_flow_script",
    "host_evidence_flow_script",
    "recovery_apply_tool",
    "visible_boot_check_tool",
    "evidence_record_tool",
    "observation_marker_tool",
]
required_true = [
    "capture_tool_ready",
    "safe_to_launch_manual_qemu",
]
required_false = [
    "autostart",
    "boot_graphics",
]

checks = {}
for key in required_ready:
    checks[key] = fields.get(key) == "ready"
for key in required_present:
    checks[key] = fields.get(key) == "present"
for key in required_true:
    checks[key] = fields.get(key) == "true"
for key in required_false:
    checks[key] = fields.get(key) == "false"

status = "ok" if all(checks.values()) and fields.get("product") == "Aqua Linux" else "failed"

summary = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "dev_target": "qemu-x86_64",
    "mode": "host-qemu-visible-preflight-summary",
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "preflight_file": preflight_file,
    "preflight_file_entry": file_entry(preflight_file),
    "status": status,
    "fields": fields,
    "checks": checks,
    "next_host_command": "scripts/run-qemu-visible-manual.sh",
    "next_ready_capture_flow_command": "scripts/run-qemu-visible-ready-capture-flow.sh",
    "next_vm_command": "AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply",
}

os.makedirs(os.path.dirname(summary_json), exist_ok=True)
with open(summary_json, "w", encoding="utf-8") as handle:
    json.dump(summary, handle, indent=2, sort_keys=True)
    handle.write("\n")

if status != "ok":
    for key, passed in sorted(checks.items()):
        if not passed:
            print(f"preflight summary error: {key} failed", file=os.sys.stderr)
    raise SystemExit(1)
PY

echo "Aqua Linux QEMU visible preflight summary written: ${SUMMARY_JSON}"
