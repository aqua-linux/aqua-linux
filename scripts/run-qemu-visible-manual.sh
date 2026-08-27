#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
RUN_QEMU="${ROOT_DIR}/scripts/run-qemu.sh"
KERNEL="${KERNEL:-${ROOT_DIR}/build/buildroot-output/images/bzImage}"
ROOTFS="${ROOTFS:-${ROOT_DIR}/build/buildroot-output/images/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-visible-manual-serial.log}"
PRINT_ONLY="${AQUA_QEMU_MANUAL_PRINT_ONLY:-false}"
SKIP_PREFLIGHT="${AQUA_QEMU_VISIBLE_SKIP_PREFLIGHT:-false}"

echo "Aqua Linux manual QEMU VM-display procedure"
echo "product=Aqua Linux"
echo "mode=host-manual"
echo "target=QEMU x86_64"
echo "docker_required=false"
echo "qemu_display_required=true"
echo "autostart=false"
echo "boot_graphics=false"
echo
echo "Host entrypoint:"
echo "  scripts/preflight-qemu-visible-manual.sh"
echo "  scripts/run-qemu-visible-manual.sh"
echo "  scripts/watch-qemu-visible-readiness.sh"
echo "  scripts/capture-qemu-visible-manual.sh"
echo "  scripts/run-qemu-visible-ready-capture-flow.sh"
echo
echo "Recovery shell sequence:"
echo "  aqua-recovery-help"
echo "  aqua-graphics-enable-gate"
echo "  aqua-graphics-launch-candidate"
echo "  aqua-graphics-rollback-drill"
echo "  aqua-graphics-startup-preflight"
echo "  aqua-graphics-startup-rehearsal"
echo "  aqua-graphics-qemu-display-gate"
echo "  aqua-graphics-visible-qemu-attempt"
echo "  aqua-graphics-visible-attempt-transcript"
echo "  aqua-graphics-visible-attempt-runner"
echo "  aqua-graphics-qemu-visible-boot-check"
echo "  AQUA_FBDEV_DRY_RUN=true aqua-graphics-fbdev-present"
echo "  AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present"
echo "  # On the host, wait until the serial log reaches recovery-ready:"
echo "  SERIAL_LOG=${SERIAL_LOG} scripts/watch-qemu-visible-readiness.sh"
echo "  # On the host, capture the visible QEMU display:"
echo "  scripts/capture-qemu-visible-manual.sh"
echo "  scripts/verify-qemu-visible-capture.sh"
echo "  scripts/write-qemu-visible-evidence-bundle.sh"
echo "  scripts/prepare-qemu-visible-evidence-apply.sh"
echo "  scripts/run-qemu-visible-evidence-flow.sh"
echo "  scripts/run-qemu-visible-ready-capture-flow.sh"
echo "  AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=<capture-id> AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=<capture-file> aqua-qemu-visible-evidence-record"
echo "  AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker"
echo
echo "Observation rule:"
echo "  Record evidence first; run the final observation marker only after the VM display is visually confirmed."
echo

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "[AQUA-HOST] stage=qemu-visible-manual-runbook status=print-only"
    exit 0
fi

if [ ! -x "${RUN_QEMU}" ]; then
    echo "Missing QEMU runner: ${RUN_QEMU}" >&2
    exit 1
fi

if [ ! -f "${KERNEL}" ]; then
    echo "Missing kernel: ${KERNEL}" >&2
    echo "Run scripts/build-image-docker-volume.sh first." >&2
    exit 1
fi

if [ ! -f "${ROOTFS}" ]; then
    echo "Missing root filesystem: ${ROOTFS}" >&2
    echo "Run scripts/build-image-docker-volume.sh first." >&2
    exit 1
fi

if [ "${SKIP_PREFLIGHT}" != "true" ]; then
    KERNEL="${KERNEL}" ROOTFS="${ROOTFS}" ROOTFS_TAR="${ROOTFS_TAR:-${ROOT_DIR}/build/buildroot-output/images/rootfs.tar}" SERIAL_LOG="${SERIAL_LOG}" \
        "${ROOT_DIR}/scripts/preflight-qemu-visible-manual.sh" >/dev/null
fi

echo "[AQUA-HOST] stage=qemu-visible-manual-runbook status=launching"
KERNEL="${KERNEL}" ROOTFS="${ROOTFS}" SERIAL_LOG="${SERIAL_LOG}" exec "${RUN_QEMU}"
