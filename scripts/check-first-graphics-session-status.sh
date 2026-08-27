#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
STATUS_FILE="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE:-${ROOT_DIR}/build/first-graphics-session-status.txt}"
STATUS_JSON="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON:-${ROOT_DIR}/build/first-graphics-session-status.json}"

test -f "${STATUS_FILE}"
test -f "${STATUS_JSON}"

grep -Fq 'status=ready-for-controlled-visible-attempt' "${STATUS_FILE}"
grep -Fq 'failed_check_count=0' "${STATUS_FILE}"
grep -Fq 'boot_graphics=false' "${STATUS_FILE}"
grep -Fq 'autostart=false' "${STATUS_FILE}"
grep -Fq 'desktop_shell=not_started' "${STATUS_FILE}"
grep -Fq 'persistent_graphical_session_started=false' "${STATUS_FILE}"
grep -Fq 'operator_confirmation_required=true' "${STATUS_FILE}"
grep -Fq 'visible_qemu_launched=false' "${STATUS_FILE}"
grep -Fq '[AQUA-HOST] stage=first-graphics-session-status status=ok' "${STATUS_FILE}"

python3 - "${STATUS_JSON}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    status = json.load(handle)

errors = []
expected = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-first-graphics-session-status",
    "target": "QEMU x86_64",
    "status": "ready-for-controlled-visible-attempt",
    "host_status": "ok",
    "operator_confirmation_required": True,
    "visible_qemu_launched": False,
}
for key, value in expected.items():
    if status.get(key) != value:
        errors.append(f"{key} must be {value!r}")

checks = status.get("checks")
if not isinstance(checks, dict) or not checks or not all(checks.values()):
    errors.append("all readiness checks must be true")
if status.get("failed_checks") != []:
    errors.append("failed_checks must be empty")

safe = status.get("safe_defaults", {})
if safe.get("boot_graphics") is not False:
    errors.append("boot_graphics must remain false")
if safe.get("autostart") is not False:
    errors.append("autostart must remain false")
if safe.get("desktop_shell") != "not_started":
    errors.append("desktop_shell must remain not_started")
if safe.get("persistent_graphical_session_started") is not False:
    errors.append("persistent graphical session must remain stopped")

if errors:
    for error in errors:
        print(f"first graphics session status error: {error}", file=sys.stderr)
    raise SystemExit(1)

print("Aqua Linux first graphics session status checks passed.")
PY
