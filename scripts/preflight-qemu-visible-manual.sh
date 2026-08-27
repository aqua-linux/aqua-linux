#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
KERNEL="${KERNEL:-${ROOT_DIR}/build/buildroot-output/images/bzImage}"
ROOTFS="${ROOTFS:-${ROOT_DIR}/build/buildroot-output/images/rootfs.ext2}"
ROOTFS_TAR="${ROOTFS_TAR:-${ROOT_DIR}/build/buildroot-output/images/rootfs.tar}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-visible-manual-serial.log}"
PREFLIGHT_FILE="${AQUA_QEMU_VISIBLE_PREFLIGHT_FILE:-${ROOT_DIR}/build/qemu-visible-manual-preflight.txt}"
PREFLIGHT_SUMMARY_JSON="${AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON:-${ROOT_DIR}/build/qemu-visible-manual-preflight.json}"
PRINT_ONLY="${AQUA_QEMU_VISIBLE_PREFLIGHT_PRINT_ONLY:-false}"

echo "Aqua Linux QEMU visible manual preflight"
echo "product=Aqua Linux"
echo "mode=host-qemu-visible-preflight"
echo "target=QEMU x86_64"
echo "kernel=${KERNEL}"
echo "rootfs=${ROOTFS}"
echo "rootfs_tar=${ROOTFS_TAR}"
echo "serial_log=${SERIAL_LOG}"
echo "preflight_file=${PREFLIGHT_FILE}"
echo "preflight_summary_json=${PREFLIGHT_SUMMARY_JSON}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "preflight_ready=true"
    echo "preflight_command=scripts/preflight-qemu-visible-manual.sh"
    echo "[AQUA-HOST] stage=qemu-visible-manual-preflight status=print-only"
    exit 0
fi

check_file() {
    label="$1"
    path="$2"

    if [ -s "${path}" ]; then
        echo "${label}=ready"
    else
        echo "${label}=missing-or-empty" >&2
        exit 1
    fi
}

check_command() {
    label="$1"
    command_name="$2"

    if command -v "${command_name}" >/dev/null 2>&1; then
        echo "${label}=ready"
    else
        echo "${label}=missing" >&2
        exit 1
    fi
}

check_rootfs_entry() {
    label="$1"
    entry="$2"

    if tar -tf "${ROOTFS_TAR}" "${entry}" >/dev/null 2>&1; then
        echo "${label}=present"
    else
        echo "${label}=missing" >&2
        exit 1
    fi
}

check_host_script() {
    label="$1"
    path="$2"

    if [ -x "${path}" ]; then
        echo "${label}=present"
    else
        echo "${label}=missing" >&2
        exit 1
    fi
}

check_file "kernel_status" "${KERNEL}"
check_file "rootfs_status" "${ROOTFS}"
check_file "rootfs_tar_status" "${ROOTFS_TAR}"
check_command "qemu_status" qemu-system-x86_64
check_host_script "host_run_script" "${ROOT_DIR}/scripts/run-qemu-visible-manual.sh"
check_host_script "host_readiness_watch_script" "${ROOT_DIR}/scripts/watch-qemu-visible-readiness.sh"
check_host_script "host_capture_script" "${ROOT_DIR}/scripts/capture-qemu-visible-manual.sh"
check_host_script "host_ready_capture_flow_script" "${ROOT_DIR}/scripts/run-qemu-visible-ready-capture-flow.sh"
check_host_script "host_evidence_flow_script" "${ROOT_DIR}/scripts/run-qemu-visible-evidence-flow.sh"
capture_tool=""
for candidate in screencapture grim gnome-screenshot spectacle import; do
    if command -v "${candidate}" >/dev/null 2>&1; then
        capture_tool="${candidate}"
        break
    fi
done
if [ -z "${capture_tool}" ]; then
    echo "capture_tool=missing" >&2
    exit 1
fi
echo "capture_tool=${capture_tool}"
echo "capture_tool_ready=true"
check_rootfs_entry "recovery_apply_tool" ./usr/bin/aqua-qemu-visible-evidence-bundle-apply
check_rootfs_entry "visible_boot_check_tool" ./usr/bin/aqua-graphics-qemu-visible-boot-check
check_rootfs_entry "evidence_record_tool" ./usr/bin/aqua-qemu-visible-evidence-record
check_rootfs_entry "observation_marker_tool" ./usr/bin/aqua-graphics-qemu-observation-marker

mkdir -p "$(dirname "${SERIAL_LOG}")"
mkdir -p "$(dirname "${PREFLIGHT_FILE}")"

{
    echo "product=Aqua Linux"
    echo "preflight=qemu-visible-manual"
    echo "preflight_status=ready"
    echo "target=QEMU x86_64"
    echo "kernel=${KERNEL}"
    echo "rootfs=${ROOTFS}"
    echo "rootfs_tar=${ROOTFS_TAR}"
    echo "serial_log=${SERIAL_LOG}"
    echo "capture_tool=${capture_tool}"
    echo "qemu_status=ready"
    echo "host_run_script=present"
    echo "host_readiness_watch_script=present"
    echo "host_capture_script=present"
    echo "host_ready_capture_flow_script=present"
    echo "host_evidence_flow_script=present"
    echo "kernel_status=ready"
    echo "rootfs_status=ready"
    echo "rootfs_tar_status=ready"
    echo "capture_tool_ready=true"
    echo "recovery_apply_tool=present"
    echo "visible_boot_check_tool=present"
    echo "evidence_record_tool=present"
    echo "observation_marker_tool=present"
    echo "operator_controlled=true"
    echo "autostart=false"
    echo "boot_graphics=false"
    echo "safe_to_launch_manual_qemu=true"
} > "${PREFLIGHT_FILE}"

grep -Fq 'preflight_status=ready' "${PREFLIGHT_FILE}"
grep -Fq 'safe_to_launch_manual_qemu=true' "${PREFLIGHT_FILE}"
grep -Fq 'host_ready_capture_flow_script=present' "${PREFLIGHT_FILE}"

echo "preflight_ready=true"
echo "preflight_written=ok"
echo "host_scripts_ready=true"
echo "summary_command=AQUA_QEMU_VISIBLE_PREFLIGHT_FILE=${PREFLIGHT_FILE} AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON=${PREFLIGHT_SUMMARY_JSON} scripts/write-qemu-visible-preflight-summary.sh"
echo "next_host_command=scripts/run-qemu-visible-manual.sh"
echo "next_ready_capture_flow_command=scripts/run-qemu-visible-ready-capture-flow.sh"
echo "next_capture_command=scripts/capture-qemu-visible-manual.sh"
echo "next_flow_command=scripts/run-qemu-visible-evidence-flow.sh"
echo "next_vm_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply"
echo "[AQUA-HOST] stage=qemu-visible-manual-preflight status=ok"
