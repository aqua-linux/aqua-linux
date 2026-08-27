#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-installer-probe-check.log}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"

for artifact in "${KERNEL}" "${ROOTFS}"; do
    test -f "${artifact}" || {
        echo "Missing QEMU artifact: ${artifact}" >&2
        exit 1
    }
done

rm -f "${SERIAL_LOG}"
export KERNEL ROOTFS SERIAL_LOG TIMEOUT_SECONDS
expect "${ROOT_DIR}/scripts/check-installer-probe-qemu.exp"

grep -Fq 'storage_candidate_count=1' "${SERIAL_LOG}"
grep -Fq 'storage_eligible_count=0' "${SERIAL_LOG}"
grep -Fq 'readiness_target_source=synthetic-readiness' "${SERIAL_LOG}"
grep -Fq 'readiness_target_selected_for_install=false' "${SERIAL_LOG}"
grep -Fq 'plan_operation_count=13' "${SERIAL_LOG}"
grep -Fq 'command_count=8' "${SERIAL_LOG}"
grep -Fq 'internal_action_count=11' "${SERIAL_LOG}"
grep -Fq 'transaction_step_count=20' "${SERIAL_LOG}"
grep -Fq 'installer_ui_status=keyboard-navigable-installer-window-contract-ready' "${SERIAL_LOG}"
grep -Fq 'installer_ui_viewport=1280x800' "${SERIAL_LOG}"
grep -Fq 'installer_ui_window=32,32,1216,736' "${SERIAL_LOG}"
grep -Fq 'installer_ui_step_count=9' "${SERIAL_LOG}"
grep -Fq 'installer_ui_keyboard_navigation=true' "${SERIAL_LOG}"
grep -Fq 'installer_form_status=validated-language-keyboard-form-controls-ready' "${SERIAL_LOG}"
grep -Fq 'installer_language_option_count=3' "${SERIAL_LOG}"
grep -Fq 'installer_keyboard_option_count=3' "${SERIAL_LOG}"
grep -Fq 'installer_timezone_form_status=validated-timezone-form-control-ready' "${SERIAL_LOG}"
grep -Fq 'installer_timezone_option_count=4' "${SERIAL_LOG}"
grep -Fq 'installer_user_form_status=password-content-free-user-form-ready' "${SERIAL_LOG}"
grep -Fq 'installer_user_password_content_stored=false' "${SERIAL_LOG}"
grep -Fq 'installer_summary_form_status=target-bound-summary-confirmation-ready' "${SERIAL_LOG}"
grep -Fq 'installer_summary_target_bound=true' "${SERIAL_LOG}"
grep -Fq 'installer_disk_form_status=eligible-storage-selection-form-ready' "${SERIAL_LOG}"
grep -Fq 'installer_disk_option_count=1' "${SERIAL_LOG}"
grep -Fq 'installer_disk_eligible_count=0' "${SERIAL_LOG}"
grep -Fq 'installer_ui_rendered=false' "${SERIAL_LOG}"
grep -Fq 'execution_allowed=false' "${SERIAL_LOG}"
grep -Fq 'disk_commands_executed=false' "${SERIAL_LOG}"
grep -Fq 'filesystem_writes_executed=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-INSTALLER] stage=readiness-probe status=ok executed=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-probe-qemu status=ok execution=false recovery_safe=true' "${SERIAL_LOG}"

echo "Aqua installer recovery-safe QEMU probe passed."
echo "Serial log: ${SERIAL_LOG}"
