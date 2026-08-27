#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BOOT_SUMMARY="${BOOT_SUMMARY:-${ROOT_DIR}/build/aqua-boot-summary.txt}"
BOOT_SUMMARY_JSON="${BOOT_SUMMARY_JSON:-${ROOT_DIR}/build/aqua-boot-summary.json}"
BOOT_STAGE_FILE="${BOOT_STAGE_FILE:-${ROOT_DIR}/scripts/aqua-boot-stages.txt}"

if [ ! -f "${BOOT_SUMMARY}" ]; then
    echo "Missing boot summary: ${BOOT_SUMMARY}" >&2
    echo "Run scripts/write-boot-summary.sh first." >&2
    exit 1
fi

if [ ! -f "${BOOT_SUMMARY_JSON}" ]; then
    echo "Missing JSON boot summary: ${BOOT_SUMMARY_JSON}" >&2
    echo "Run scripts/write-boot-summary.sh first." >&2
    exit 1
fi

python3 - "${BOOT_SUMMARY}" "${BOOT_SUMMARY_JSON}" "${BOOT_STAGE_FILE}" <<'PY'
import json
import sys

summary_path = sys.argv[1]
json_path = sys.argv[2]
stage_path = sys.argv[3]

with open(json_path, "r", encoding="utf-8") as handle:
    summary = json.load(handle)

with open(stage_path, "r", encoding="utf-8") as handle:
    expected_stages = [
        line.strip()
        for line in handle
        if line.strip() and not line.lstrip().startswith("#")
    ]

errors = []
if summary.get("product") != "Aqua Linux":
    errors.append("product must be Aqua Linux")
if summary.get("base") != "Buildroot":
    errors.append("base must be Buildroot")
if summary.get("dev_target") != "qemu-x86_64":
    errors.append("dev_target must be qemu-x86_64")
if summary.get("status") != "ok":
    errors.append("summary status must be ok")
if summary.get("expected_stages") != expected_stages:
    errors.append("expected_stages order changed")
if summary.get("missing_stages") != []:
    errors.append("missing_stages must be empty")
if summary.get("failed_stages") != []:
    errors.append("failed_stages must be empty")

generated_at = summary.get("generated_at_utc", "")
if not generated_at.endswith("Z") or "T" not in generated_at:
    errors.append("generated_at_utc must be UTC ISO-like timestamp ending in Z")

markers = summary.get("markers")
if not isinstance(markers, dict):
    errors.append("markers must be an object")
else:
    previous_line = 0
    for stage in expected_stages:
        marker = markers.get(stage)
        if not isinstance(marker, dict):
            errors.append(f"missing marker object for {stage}")
            continue
        if marker.get("stage") != stage:
            errors.append(f"marker {stage} has wrong stage field")
        status = marker.get("status")
        if status in {"missing", "failed", "skipped", None}:
            errors.append(f"marker {stage} has bad status {status!r}")
        line = marker.get("line")
        if not isinstance(line, int) or line <= 0:
            errors.append(f"marker {stage} must include a positive serial line")
        elif line < previous_line:
            errors.append(f"marker {stage} appears out of boot order")
        previous_line = line if isinstance(line, int) else previous_line

with open(summary_path, "r", encoding="utf-8") as handle:
    text_summary = handle.read()

for needle in [
    "product=Aqua Linux",
    "base=Buildroot",
    "dev_target=qemu-x86_64",
    "status=ok",
    "recovery-ready=ok",
]:
    if needle not in text_summary:
        errors.append(f"text summary missing {needle!r}")

if errors:
    for error in errors:
        print(f"boot summary error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux boot summary checks passed.")
PY
