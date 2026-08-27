#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PROGRESS_JSON="${PROGRESS_JSON:-${ROOT_DIR}/docs/aqua-linux/progress.json}"
PROGRESS_MD="${PROGRESS_MD:-${ROOT_DIR}/docs/aqua-linux/progress.md}"

if [ ! -f "${PROGRESS_JSON}" ]; then
    echo "Missing progress JSON: ${PROGRESS_JSON}" >&2
    exit 1
fi

export PROGRESS_JSON PROGRESS_MD

python3 - <<'PY'
import json
import os

source = os.environ["PROGRESS_JSON"]
target = os.environ["PROGRESS_MD"]

with open(source, "r", encoding="utf-8") as handle:
    data = json.load(handle)


def cell(value):
    return str(value).replace("|", "\\|").replace("\n", " ").strip()


def status_label(status):
    return status.replace("-", " ").title()


def phase_sort_key(phase):
    return (phase.get("updated", "0000-00-00"), phase.get("id", ""))


lines = [
    f"# {data['product']} {data['release']} Progress Report",
    "",
    "> Generated from `docs/aqua-linux/progress.json`. Update the changed phase date, then run `scripts/write-progress-report.sh`.",
    "",
    f"**Overall progress: {int(data['overallPercent'])}%**",
    "",
    "| Field | Value |",
    "| --- | --- |",
    f"| Updated | {cell(data['updated'])} |",
    f"| OS base | {cell(data['base'])} |",
    f"| Graphics target | {cell(data['graphicsTarget'])} |",
    f"| Development target | {cell(data['devTarget'])} |",
    f"| Hardware target | {cell(data['hardwareTarget'])} |",
    "",
    "## Current Stage",
    "",
    cell(data["currentStage"]),
    "",
    "## Phases",
    "",
    "Phases are ordered by their most recent update.",
    "",
    "| Updated | Phase | Status | Progress | Summary |",
    "| --- | --- | --- | ---: | --- |",
]

for phase in sorted(data["phases"], key=phase_sort_key, reverse=True):
    lines.append(
        "| {updated} | {phase_id}: {name} | {status} | {percent}% | {summary} |".format(
            updated=cell(phase["updated"]),
            phase_id=cell(phase["id"].upper()),
            name=cell(phase["name"]),
            status=cell(status_label(phase["status"])),
            percent=int(phase["percent"]),
            summary=cell(phase["summary"]),
        )
    )

lines.extend(["", "## Completion Rules", ""])
lines.extend(f"- {cell(rule)}" for rule in data["rules"])
lines.extend(["", "## Next Developments", ""])
lines.extend(f"{index}. {cell(step)}" for index, step in enumerate(data["nextSteps"], 1))
lines.append("")

with open(target, "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))

print(f"Aqua Linux progress report written: {target}")
PY
