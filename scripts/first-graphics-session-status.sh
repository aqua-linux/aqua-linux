#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BOOT_SUMMARY_JSON="${AQUA_BOOT_SUMMARY_JSON:-${ROOT_DIR}/build/aqua-boot-summary.json}"
IMAGE_MANIFEST_JSON="${AQUA_IMAGE_MANIFEST_JSON:-${ROOT_DIR}/build/aqua-image-manifest.json}"
QEMU_VISIBLE_STATUS_JSON="${AQUA_QEMU_VISIBLE_STATUS_JSON:-${ROOT_DIR}/build/qemu-visible-status.json}"
STATUS_FILE="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE:-${ROOT_DIR}/build/first-graphics-session-status.txt}"
STATUS_JSON="${AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON:-${ROOT_DIR}/build/first-graphics-session-status.json}"

for required_file in "${BOOT_SUMMARY_JSON}" "${IMAGE_MANIFEST_JSON}" "${QEMU_VISIBLE_STATUS_JSON}"; do
    if [ ! -f "${required_file}" ]; then
        echo "Missing first graphics session status input: ${required_file}" >&2
        exit 1
    fi
done

mkdir -p "$(dirname "${STATUS_FILE}")" "$(dirname "${STATUS_JSON}")"

export BOOT_SUMMARY_JSON IMAGE_MANIFEST_JSON QEMU_VISIBLE_STATUS_JSON STATUS_FILE STATUS_JSON
python3 - <<'PY'
import json
import os
from datetime import datetime, timezone


def load(path):
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


boot = load(os.environ["BOOT_SUMMARY_JSON"])
manifest = load(os.environ["IMAGE_MANIFEST_JSON"])
qemu = load(os.environ["QEMU_VISIBLE_STATUS_JSON"])

checks = {
    "boot_summary": boot.get("status") == "ok",
    "recovery_ready": boot.get("markers", {}).get("recovery-ready", {}).get("status") == "ok",
    "rootfs_image": manifest.get("artifacts", {}).get("rootfs_ext2", {}).get("status") == "ready",
    "compositor_packaged": manifest.get("rootfs", {}).get("compositor_packaged") == "true",
    "display_activation_plan": manifest.get("scene_contract", {}).get("display_activation_plan_can_activate") == "ok",
    "manual_nested_execution": manifest.get("scene_contract", {}).get("manual_nested_preview_execution_ready") == "ok",
    "bounded_visible_runner": manifest.get("scene_contract", {}).get("graphics_visible_attempt_runner_ready") == "ok",
    "qemu_visible_boot_path": manifest.get("scene_contract", {}).get("graphics_qemu_visible_boot_path_ready") == "ok",
    "fbdev_presenter": manifest.get("scene_contract", {}).get("graphics_fbdev_present") == "ok",
    "fbdev_presenter_bounded": manifest.get("scene_contract", {}).get("graphics_fbdev_present_bounded") == "ok",
    "fbdev_presenter_recovery_safe": manifest.get("scene_contract", {}).get("graphics_fbdev_present_recovery_safe") == "ok",
    "operator_pass": qemu.get("qemu_visible_manual_status") == "ready-for-operator-pass",
    "capture_hash_gate": qemu.get("capture_hash_verification_required") is True,
    "pass_report_gate": qemu.get("manual_runbook_pass_report_required_status") == "ok",
    "fallback_recovery": manifest.get("scene_contract", {}).get("graphics_visible_attempt_runner_recovery_safe") == "ok",
    "boot_graphics_disabled": manifest.get("rootfs", {}).get("boot_graphics") is False and qemu.get("boot_graphics") is False,
    "autostart_disabled": manifest.get("rootfs", {}).get("autostart") is False and qemu.get("autostart") is False,
    "desktop_shell_not_started": manifest.get("scene_contract", {}).get("desktop_shell") == "not_started" and qemu.get("desktop_shell") == "not_started",
}

failed_checks = [name for name, passed in checks.items() if not passed]
ready = not failed_checks
status = "ready-for-controlled-visible-attempt" if ready else "blocked"
host_status = "ok" if ready else "blocked"

result = {
    "product": "Aqua Linux",
    "base": "Buildroot",
    "mode": "host-first-graphics-session-status",
    "target": "QEMU x86_64",
    "generated_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "status": status,
    "host_status": host_status,
    "checks": checks,
    "failed_checks": failed_checks,
    "safe_defaults": {
        "boot_graphics": False,
        "autostart": False,
        "desktop_shell": "not_started",
        "persistent_graphical_session_started": False,
    },
    "operator_confirmation_required": True,
    "visible_qemu_launched": False,
    "next_action": "AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh",
    "sources": {
        "boot_summary_json": os.environ["BOOT_SUMMARY_JSON"],
        "image_manifest_json": os.environ["IMAGE_MANIFEST_JSON"],
        "qemu_visible_status_json": os.environ["QEMU_VISIBLE_STATUS_JSON"],
    },
}

lines = [
    "Aqua Linux first graphics session status",
    "product=Aqua Linux",
    "base=Buildroot",
    "mode=host-first-graphics-session-status",
    "target=QEMU x86_64",
    f"status={status}",
]
for name, passed in checks.items():
    lines.append(f"check_{name}={'ok' if passed else 'failed'}")
lines.extend([
    f"failed_check_count={len(failed_checks)}",
    "boot_graphics=false",
    "autostart=false",
    "desktop_shell=not_started",
    "persistent_graphical_session_started=false",
    "operator_confirmation_required=true",
    "visible_qemu_launched=false",
    f"next_action={result['next_action']}",
    f"[AQUA-HOST] stage=first-graphics-session-status status={host_status}",
])

with open(os.environ["STATUS_FILE"], "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines) + "\n")
with open(os.environ["STATUS_JSON"], "w", encoding="utf-8") as handle:
    json.dump(result, handle, indent=2, sort_keys=True)
    handle.write("\n")

print("\n".join(lines))
raise SystemExit(0 if ready else 1)
PY
