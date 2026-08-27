#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"

if [ ! -f "${SUMMARY_JSON}" ]; then
    echo "Missing QEMU visible preflight summary: ${SUMMARY_JSON}" >&2
    echo "Run scripts/write-qemu-visible-preflight-summary.sh first." >&2
    exit 1
fi

python3 - "${SUMMARY_JSON}" <<'PY'
import json
import sys

summary_path = sys.argv[1]
with open(summary_path, "r", encoding="utf-8") as handle:
    summary = json.load(handle)

errors = []
if summary.get("product") != "Aqua Linux":
    errors.append("product must be Aqua Linux")
if summary.get("base") != "Buildroot":
    errors.append("base must be Buildroot")
if summary.get("dev_target") != "qemu-x86_64":
    errors.append("dev_target must be qemu-x86_64")
if summary.get("mode") != "host-qemu-visible-preflight-summary":
    errors.append("mode must be host-qemu-visible-preflight-summary")
if summary.get("status") != "ok":
    errors.append("status must be ok")

preflight_entry = summary.get("preflight_file_entry")
if not isinstance(preflight_entry, dict):
    errors.append("preflight_file_entry must be an object")
else:
    if preflight_entry.get("path") != summary.get("preflight_file"):
        errors.append("preflight_file_entry.path must match preflight_file")
    if not isinstance(preflight_entry.get("bytes"), int) or preflight_entry["bytes"] <= 0:
        errors.append("preflight_file_entry.bytes must be positive")
    sha256 = preflight_entry.get("sha256")
    if not isinstance(sha256, str) or len(sha256) != 64:
        errors.append("preflight_file_entry.sha256 must be a sha256 hex digest")
    mtime = preflight_entry.get("mtime_utc", "")
    if not isinstance(mtime, str) or not mtime.endswith("Z") or "T" not in mtime:
        errors.append("preflight_file_entry.mtime_utc must be UTC ISO-like timestamp ending in Z")

checks = summary.get("checks")
if not isinstance(checks, dict) or not checks:
    errors.append("checks must be a non-empty object")
else:
    for key, value in checks.items():
        if value is not True:
            errors.append(f"check {key} must be true")

fields = summary.get("fields")
if not isinstance(fields, dict):
    errors.append("fields must be an object")
else:
    for key, value in {
        "product": "Aqua Linux",
        "preflight_status": "ready",
        "safe_to_launch_manual_qemu": "true",
        "host_ready_capture_flow_script": "present",
        "host_evidence_flow_script": "present",
        "boot_graphics": "false",
        "autostart": "false",
    }.items():
        if fields.get(key) != value:
            errors.append(f"field {key} must be {value!r}")

generated_at = summary.get("generated_at_utc", "")
if not isinstance(generated_at, str) or not generated_at.endswith("Z") or "T" not in generated_at:
    errors.append("generated_at_utc must be UTC ISO-like timestamp ending in Z")

if summary.get("next_ready_capture_flow_command") != "scripts/run-qemu-visible-ready-capture-flow.sh":
    errors.append("next_ready_capture_flow_command must point to ready capture flow")

if errors:
    for error in errors:
        print(f"preflight summary error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux QEMU visible preflight summary checks passed.")
PY
