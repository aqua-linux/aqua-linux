#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PROGRESS_JSON="${PROGRESS_JSON:-${ROOT_DIR}/docs/aqua-linux/progress.json}"
PROGRESS_MD="${PROGRESS_MD:-${ROOT_DIR}/docs/aqua-linux/progress.md}"

if [ ! -f "${PROGRESS_JSON}" ]; then
    echo "Missing progress JSON: ${PROGRESS_JSON}" >&2
    exit 1
fi

if [ ! -f "${PROGRESS_MD}" ]; then
    echo "Missing progress Markdown: ${PROGRESS_MD}" >&2
    echo "Run scripts/write-progress-report.sh first." >&2
    exit 1
fi

python3 - "${PROGRESS_JSON}" "${PROGRESS_MD}" <<'PY'
import json
import sys

json_path = sys.argv[1]
markdown_path = sys.argv[2]

with open(json_path, "r", encoding="utf-8") as handle:
    data = json.load(handle)

errors = []
if data.get("product") != "Aqua Linux":
    errors.append("product must be Aqua Linux")
if data.get("base") != "Buildroot":
    errors.append("base must be Buildroot")
if data.get("graphicsTarget") != "custom Wayland compositor":
    errors.append("graphicsTarget must stay custom Wayland compositor")
if not data.get("r3SelectionBoundary"):
    errors.append("r3SelectionBoundary must describe the active compatibility work")
if not isinstance(data.get("overallPercent"), int):
    errors.append("overallPercent must be an integer")
elif not 0 <= data["overallPercent"] <= 100:
    errors.append("overallPercent must be between 0 and 100")

readiness = data.get("readiness")
if not isinstance(readiness, dict):
    errors.append("readiness must be an object")
else:
    if readiness.get("classification") != "packaged-QEMU-proven prototype":
        errors.append("readiness classification must match the current prototype boundary")
    if readiness.get("evidenceLevel") != "packaged-QEMU-proven":
        errors.append("readiness evidenceLevel must be packaged-QEMU-proven")
    for key in ("dailyUseReady", "hardwareProven", "releaseReady"):
        if readiness.get(key) is not False:
            errors.append(f"readiness {key} must remain false until its mandatory gates pass")
    if not readiness.get("summary"):
        errors.append("readiness must include a summary")

phases = data.get("phases")
if not isinstance(phases, list) or len(phases) != 13:
    errors.append("progress report must track 13 v1 phases")
else:
    expected_ids = [f"m{i}" for i in range(13)]
    actual_ids = [phase.get("id") for phase in phases]
    if actual_ids != expected_ids:
        errors.append("phase ids must be m0 through m12 in order")
    for phase in phases:
        percent = phase.get("percent")
        if not isinstance(percent, int) or not 0 <= percent <= 100:
            errors.append(f"{phase.get('id', 'unknown')} percent must be 0..100 integer")
        if not phase.get("name") or not phase.get("summary"):
            errors.append(f"{phase.get('id', 'unknown')} must include name and summary")
        updated = phase.get("updated")
        if not isinstance(updated, str) or len(updated) != 10 or updated[4] != "-" or updated[7] != "-":
            errors.append(f"{phase.get('id', 'unknown')} must include updated date as YYYY-MM-DD")

with open(markdown_path, "r", encoding="utf-8") as handle:
    markdown = handle.read()

for needle in [
    "# Aqua Linux v1.0 Progress Report",
    f"**Roadmap implementation progress: {data.get('overallPercent')}%**",
    "## Product Readiness",
    "exact direct-FD payload transfer",
    "separate two-client drag-and-drop probe",
    "normal, 90, 180, and 270 degree transforms",
    "authorized input-method client",
    "weston-simple-damage",
    "| Classification | packaged-QEMU-proven prototype |",
    "| Daily-use ready | No |",
    "| Hardware-proven | No |",
    "| Release-ready | No |",
    "[v1-readiness.md](v1-readiness.md)",
    "| Updated | Phase | Status | Progress | Summary |",
    "Repository and Build Skeleton",
    "Buildroot Boot to Text Recovery",
    "Boot Aqua Compositor in QEMU",
    "Generated from `docs/aqua-linux/progress.json`",
    "## Next Developments",
]:
    if needle not in markdown:
        errors.append(f"progress Markdown missing {needle!r}")

expected_order = sorted(
    phases or [],
    key=lambda phase: (phase.get("updated", ""), phase.get("id", "")),
    reverse=True,
)
positions = [markdown.find(f"{phase['id'].upper()}: {phase['name']}") for phase in expected_order]
if any(position < 0 for position in positions) or positions != sorted(positions):
    errors.append("progress Markdown phases must be ordered by most recent update")

if "<html" in markdown.lower() or "<table" in markdown.lower():
    errors.append("progress Markdown must not contain the retired HTML report")

if errors:
    for error in errors:
        print(f"progress report error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux progress report checks passed.")
PY
