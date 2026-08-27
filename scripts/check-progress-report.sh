#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PROGRESS_JSON="${PROGRESS_JSON:-${ROOT_DIR}/docs/aqua-linux/progress.json}"
PROGRESS_HTML="${PROGRESS_HTML:-${ROOT_DIR}/docs/aqua-linux/progress.html}"

if [ ! -f "${PROGRESS_JSON}" ]; then
    echo "Missing progress JSON: ${PROGRESS_JSON}" >&2
    exit 1
fi

if [ ! -f "${PROGRESS_HTML}" ]; then
    echo "Missing progress HTML: ${PROGRESS_HTML}" >&2
    echo "Run scripts/write-progress-report.sh first." >&2
    exit 1
fi

python3 - "${PROGRESS_JSON}" "${PROGRESS_HTML}" <<'PY'
import json
import sys

json_path = sys.argv[1]
html_path = sys.argv[2]

with open(json_path, "r", encoding="utf-8") as handle:
    data = json.load(handle)

errors = []
if data.get("product") != "Aqua Linux":
    errors.append("product must be Aqua Linux")
if data.get("base") != "Buildroot":
    errors.append("base must be Buildroot")
if data.get("graphicsTarget") != "custom Wayland compositor":
    errors.append("graphicsTarget must stay custom Wayland compositor")
if not isinstance(data.get("overallPercent"), int):
    errors.append("overallPercent must be an integer")
elif not 0 <= data["overallPercent"] <= 100:
    errors.append("overallPercent must be between 0 and 100")

phases = data.get("phases")
if not isinstance(phases, list) or len(phases) != 12:
    errors.append("progress report must track 12 v1 phases")
else:
    expected_ids = [f"m{i}" for i in range(12)]
    actual_ids = [phase.get("id") for phase in phases]
    if actual_ids != expected_ids:
        errors.append("phase ids must be m0 through m11 in order")
    for phase in phases:
        percent = phase.get("percent")
        if not isinstance(percent, int) or not 0 <= percent <= 100:
            errors.append(f"{phase.get('id', 'unknown')} percent must be 0..100 integer")
        if not phase.get("name") or not phase.get("summary"):
            errors.append(f"{phase.get('id', 'unknown')} must include name and summary")
        updated = phase.get("updated")
        if not isinstance(updated, str) or len(updated) != 10 or updated[4] != "-" or updated[7] != "-":
            errors.append(f"{phase.get('id', 'unknown')} must include updated date as YYYY-MM-DD")

with open(html_path, "r", encoding="utf-8") as handle:
    html = handle.read()

for needle in [
    "Aqua Linux",
    "v1.0 progress report",
    f"{data.get('overallPercent')}%",
    "<table>",
    "Updated",
    "Progress",
    "Repository and Build Skeleton",
    "Buildroot Boot to Text Recovery",
    "Boot Aqua Compositor in QEMU",
    "Generated from docs/aqua-linux/progress.json",
]:
    if needle not in html:
        errors.append(f"progress HTML missing {needle!r}")

for removed in [
    "splash-card",
    "splash-loader",
]:
    if removed in html:
        errors.append(f"progress HTML should not include removed splash asset {removed!r}")

if errors:
    for error in errors:
        print(f"progress report error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux progress report checks passed.")
PY
