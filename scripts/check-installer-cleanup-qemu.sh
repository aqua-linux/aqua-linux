#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
TARGET_DISK="${TARGET_DISK:-${ROOT_DIR}/build/qemu-installer-cleanup-target.qcow2}"
ARTIFACT_DISK="${ARTIFACT_DISK:-${ROOT_DIR}/build/installer-artifacts.ext4}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-installer-cleanup.log}"
HASH_REPORT="${HASH_REPORT:-${ROOT_DIR}/build/qemu-installer-cleanup.sha256}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-300}"

for tool in expect qemu-img qemu-system-x86_64; do
    command -v "${tool}" >/dev/null 2>&1 || {
        echo "Missing required tool: ${tool}" >&2
        exit 1
    }
done
for artifact in "${KERNEL}" "${ROOTFS}"; do
    test -s "${artifact}" || {
        echo "Missing QEMU artifact: ${artifact}" >&2
        exit 1
    }
done

sha256_file() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        sha256sum "$1" | awk '{print $1}'
    fi
}

rm -f "${TARGET_DISK}" "${SERIAL_LOG}" "${HASH_REPORT}"
ARTIFACT_DISK="${ARTIFACT_DISK}" "${ROOT_DIR}/scripts/write-installer-artifact-disk-docker.sh"
qemu-img create -q -f qcow2 "${TARGET_DISK}" 4G
target_before_sha256="$(sha256_file "${TARGET_DISK}")"
artifact_before_sha256="$(sha256_file "${ARTIFACT_DISK}")"

export KERNEL ROOTFS TARGET_DISK ARTIFACT_DISK SERIAL_LOG TIMEOUT_SECONDS
expect "${ROOT_DIR}/scripts/check-installer-cleanup-qemu.exp"

target_after_sha256="$(sha256_file "${TARGET_DISK}")"
artifact_after_sha256="$(sha256_file "${ARTIFACT_DISK}")"
test "${target_before_sha256}" != "${target_after_sha256}" || {
    echo "Disposable cleanup target was not changed before injection" >&2
    exit 1
}
test "${artifact_before_sha256}" = "${artifact_after_sha256}" || {
    echo "Read-only installer artifact disk changed during cleanup test" >&2
    exit 1
}

grep -Fq 'transaction_failure_injection=after-efi-mount' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_completed=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-INSTALLER-PROGRESS] state=failed phase=installing-bootloader operation=mount-efi-system-partition completed=8 total=20 percent=40' "${SERIAL_LOG}"
grep -Fq 'transaction_cleanup_attempt=/mnt/aqua-target/boot/efi' "${SERIAL_LOG}"
grep -Fq 'transaction_cleanup_completed=/mnt/aqua-target/boot/efi' "${SERIAL_LOG}"
grep -Fq 'transaction_cleanup_attempt=/mnt/aqua-target' "${SERIAL_LOG}"
grep -Fq 'transaction_cleanup_completed=/mnt/aqua-target' "${SERIAL_LOG}"
grep -Fq 'cleanup_commands=2: injected failure after EFI mount' "${SERIAL_LOG}"
grep -Fq '[AQUA-INSTALLER] stage=execution-run status=error executed=true completed=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-cleanup-unmounted status=ok efi=true root=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-cleanup-filesystems status=ok root_readable=true efi_readable=true bootloader_staged=false' "${SERIAL_LOG}"

{
    echo "target_disk=${TARGET_DISK}"
    echo "target_before_sha256=${target_before_sha256}"
    echo "target_after_sha256=${target_after_sha256}"
    echo "target_changed_before_injected_failure=true"
    echo "artifact_disk=${ARTIFACT_DISK}"
    echo "artifact_before_sha256=${artifact_before_sha256}"
    echo "artifact_after_sha256=${artifact_after_sha256}"
    echo "artifact_disk_unchanged=true"
    echo "cleanup_order=efi,root"
    echo "mounts_cleared=true"
} > "${HASH_REPORT}"

echo "Aqua installer failure cleanup QEMU check passed."
echo "Serial log: ${SERIAL_LOG}"
echo "Hash report: ${HASH_REPORT}"
