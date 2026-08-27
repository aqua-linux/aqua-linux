#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-serial-check.log}"
BOOT_SUMMARY="${BOOT_SUMMARY:-${ROOT_DIR}/build/aqua-boot-summary.txt}"
BOOT_SUMMARY_JSON="${BOOT_SUMMARY_JSON:-${ROOT_DIR}/build/aqua-boot-summary.json}"
BOOT_STAGE_FILE="${BOOT_STAGE_FILE:-${ROOT_DIR}/scripts/aqua-boot-stages.txt}"

if [ ! -f "${SERIAL_LOG}" ]; then
    echo "Missing QEMU serial log: ${SERIAL_LOG}" >&2
    echo "Run scripts/check-boot.sh first." >&2
    exit 1
fi

export SERIAL_LOG BOOT_SUMMARY BOOT_SUMMARY_JSON BOOT_STAGE_FILE

python3 - <<'PY'
import json
import os
import re
from datetime import datetime, timezone

serial_log = os.environ["SERIAL_LOG"]
boot_summary = os.environ["BOOT_SUMMARY"]
boot_summary_json = os.environ["BOOT_SUMMARY_JSON"]
boot_stage_file = os.environ["BOOT_STAGE_FILE"]

with open(boot_stage_file, "r", encoding="utf-8") as handle:
    stage_order = [
        line.strip()
        for line in handle
        if line.strip() and not line.lstrip().startswith("#")
    ]

marker_re = re.compile(r"\[AQUA-BOOT\]\s+stage=([^\s]+)(.*)$")
field_re = re.compile(r"([A-Za-z0-9_-]+)=((?:\"[^\"]*\")|[^\s]+)")

markers = {}
with open(serial_log, "r", encoding="utf-8", errors="replace") as handle:
    for line_number, line in enumerate(handle, start=1):
        match = marker_re.search(line)
        if not match:
            continue

        stage = match.group(1)
        fields = {}
        for key, value in field_re.findall(match.group(2)):
            fields[key] = value.strip('"')

        markers[stage] = {
            "stage": stage,
            "status": fields.get("status", "seen"),
            "fields": fields,
            "line": line_number,
        }

summary = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "dev_target": "qemu-x86_64",
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "serial_log": serial_log,
    "expected_stages": stage_order,
    "markers": {
        stage: markers.get(stage, {"stage": stage, "status": "missing", "fields": {}, "line": None})
        for stage in stage_order
    },
}

missing = [stage for stage in stage_order if stage not in markers]
failed = [
    stage
    for stage in stage_order
    if markers.get(stage, {}).get("status") in {"failed", "missing", "skipped"}
]
summary["status"] = "ok" if not missing and not failed else "failed"
summary["missing_stages"] = missing
summary["failed_stages"] = failed

os.makedirs(os.path.dirname(boot_summary), exist_ok=True)

with open(boot_summary, "w", encoding="utf-8") as handle:
    handle.write("[boot]\n")
    handle.write(f"product={summary['product']}\n")
    handle.write(f"base={summary['base']}\n")
    handle.write(f"dev_target={summary['dev_target']}\n")
    handle.write(f"status={summary['status']}\n")
    handle.write(f"serial_log={serial_log}\n")
    handle.write(f"generated_at_utc={summary['generated_at_utc']}\n")
    handle.write("\n[markers]\n")
    for stage in stage_order:
        marker = summary["markers"][stage]
        handle.write(f"{stage}={marker['status']}\n")

with open(boot_summary_json, "w", encoding="utf-8") as handle:
    json.dump(summary, handle, indent=2, sort_keys=True)
    handle.write("\n")

if summary["status"] != "ok":
    for stage in missing:
        print(f"boot summary error: missing stage {stage}", file=os.sys.stderr)
    for stage in failed:
        print(f"boot summary error: failed stage {stage}", file=os.sys.stderr)
    raise SystemExit(1)
PY

echo "Aqua Linux boot summary written: ${BOOT_SUMMARY}"
echo "Aqua Linux JSON boot summary written: ${BOOT_SUMMARY_JSON}"
