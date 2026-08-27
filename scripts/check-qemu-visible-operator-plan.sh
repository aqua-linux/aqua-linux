#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PLAN_FILE="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE:-${ROOT_DIR}/build/qemu-visible-operator-plan.txt}"
PLAN_JSON="${AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON:-${ROOT_DIR}/build/qemu-visible-operator-plan.json}"

if [ ! -f "${PLAN_FILE}" ]; then
    echo "Missing QEMU visible operator plan text: ${PLAN_FILE}" >&2
    echo "Run scripts/write-qemu-visible-operator-plan.sh first." >&2
    exit 1
fi

if [ ! -f "${PLAN_JSON}" ]; then
    echo "Missing QEMU visible operator plan JSON: ${PLAN_JSON}" >&2
    echo "Run scripts/write-qemu-visible-operator-plan.sh first." >&2
    exit 1
fi

grep -Fq 'mode=host-qemu-visible-operator-plan' "${PLAN_FILE}"
grep -Fq 'status=ready-for-operator-pass' "${PLAN_FILE}"
grep -Fq 'boot_graphics=false' "${PLAN_FILE}"
grep -Fq 'autostart=false' "${PLAN_FILE}"
grep -Fq 'desktop_shell=not_started' "${PLAN_FILE}"
grep -Fq 'visual_confirmation_required=true' "${PLAN_FILE}"
grep -Fq 'no_positive_observation_without_evidence=true' "${PLAN_FILE}"
grep -Fq 'no_unverified_bundle_acceptance=true' "${PLAN_FILE}"
grep -Fq 'capture_hash_verification_required=true' "${PLAN_FILE}"
grep -Fq 'step_3_command=scripts/run-qemu-visible-manual.sh' "${PLAN_FILE}"
grep -Fq 'step_4_command=AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present' "${PLAN_FILE}"
grep -Fq 'step_6_command=scripts/run-qemu-visible-ready-capture-flow.sh' "${PLAN_FILE}"
grep -Fq 'step_8_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply' "${PLAN_FILE}"
grep -Fq 'step_9_command=aqua-qemu-visible-pass-report' "${PLAN_FILE}"
grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-plan status=ok' "${PLAN_FILE}"

python3 - "${PLAN_JSON}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    plan = json.load(handle)

errors = []
expected = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-qemu-visible-operator-plan",
    "target": "QEMU x86_64",
    "status": "ready-for-operator-pass",
    "next_required_action": "operator-run-manual-qemu-pass",
}

for key, value in expected.items():
    if plan.get(key) != value:
        errors.append(f"{key} must be {value!r}")

safe = plan.get("safe_defaults")
if not isinstance(safe, dict):
    errors.append("safe_defaults must be an object")
else:
    if safe.get("boot_graphics") is not False:
        errors.append("safe_defaults.boot_graphics must be false")
    if safe.get("autostart") is not False:
        errors.append("safe_defaults.autostart must be false")
    if safe.get("desktop_shell") != "not_started":
        errors.append("safe_defaults.desktop_shell must be not_started")
    if safe.get("persistent_graphical_session_started") is not False:
        errors.append("safe_defaults.persistent_graphical_session_started must be false")

gates = plan.get("operator_gates")
if not isinstance(gates, dict):
    errors.append("operator_gates must be an object")
else:
    for key in (
        "visual_confirmation_required",
        "manual_observation_required",
        "no_positive_observation_without_evidence",
        "no_unverified_bundle_acceptance",
        "capture_hash_verification_required",
        "operator_confirmation_required",
        "pass_report_required",
        "fbdev_frame_required",
    ):
        if gates.get(key) is not True:
            errors.append(f"operator_gates.{key} must be true")

steps = plan.get("steps")
if not isinstance(steps, list) or len(steps) != 9:
    errors.append("steps must contain nine ordered actions")
else:
    expected_ids = [
        "host-preflight",
        "host-preflight-summary",
        "host-launch-qemu",
        "vm-present-fbdev-frame",
        "operator-visible-confirmation",
        "host-ready-capture-flow",
        "vm-paste-evidence-bundle",
        "vm-apply-observed-marker",
        "vm-write-pass-report",
    ]
    actual_ids = [step.get("id") for step in steps]
    if actual_ids != expected_ids:
        errors.append("steps are not in the required operator-pass order")
    if steps[2].get("command") != "scripts/run-qemu-visible-manual.sh":
        errors.append("host-launch-qemu command changed")
    if steps[3].get("command") != "AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present":
        errors.append("vm-present-fbdev-frame command changed")
    if steps[5].get("command") != "scripts/run-qemu-visible-ready-capture-flow.sh":
        errors.append("host-ready-capture-flow command changed")
    if steps[7].get("command") != "AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply":
        errors.append("vm observed marker command changed")
    if steps[8].get("command") != "aqua-qemu-visible-pass-report":
        errors.append("vm pass report command changed")
    for step in steps[3:]:
        if step.get("requires_operator_confirmation") is not True:
            errors.append(f"{step.get('id')} must require operator confirmation")

generated_at = plan.get("generated_at_utc", "")
if not isinstance(generated_at, str) or "T" not in generated_at or not generated_at.endswith("Z"):
    errors.append("generated_at_utc must be UTC ISO-like timestamp ending in Z")

if not plan.get("source_status_json"):
    errors.append("source_status_json must be present")

if errors:
    for error in errors:
        print(f"qemu visible operator plan error: {error}", file=sys.stderr)
    sys.exit(1)

print("Aqua Linux QEMU visible operator plan checks passed.")
PY
