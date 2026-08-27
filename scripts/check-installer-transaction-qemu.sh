#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
TARGET_DISK="${TARGET_DISK:-${ROOT_DIR}/build/qemu-installer-transaction-target.qcow2}"
ARTIFACT_DISK="${ARTIFACT_DISK:-${ROOT_DIR}/build/installer-artifacts.ext4}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-installer-transaction.log}"
HASH_REPORT="${HASH_REPORT:-${ROOT_DIR}/build/qemu-installer-transaction.sha256}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-300}"

if [ -z "${FIRMWARE:-}" ]; then
    for candidate in \
        /opt/homebrew/share/qemu/edk2-x86_64-code.fd \
        /usr/local/share/qemu/edk2-x86_64-code.fd \
        /usr/share/qemu/edk2-x86_64-code.fd
    do
        if [ -s "${candidate}" ]; then
            FIRMWARE="${candidate}"
            break
        fi
    done
fi
test -n "${FIRMWARE:-}" && test -s "${FIRMWARE}" || {
    echo "Missing QEMU EDK2 x86_64 firmware" >&2
    exit 1
}

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

export KERNEL ROOTFS TARGET_DISK ARTIFACT_DISK SERIAL_LOG TIMEOUT_SECONDS FIRMWARE
expect "${ROOT_DIR}/scripts/check-installer-transaction-qemu.exp"

target_after_sha256="$(sha256_file "${TARGET_DISK}")"
artifact_after_sha256="$(sha256_file "${ARTIFACT_DISK}")"
test "${target_before_sha256}" != "${target_after_sha256}" || {
    echo "Disposable installer target was not changed by execution" >&2
    exit 1
}
test "${artifact_before_sha256}" = "${artifact_after_sha256}" || {
    echo "Read-only installer artifact disk changed during execution" >&2
    exit 1
}

grep -Fq '[AQUA-TEST] stage=installer-transaction-missing-enable status=ok rejected=true' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_target=/dev/vdb' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_started=true' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_completed=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-INSTALLER-PROGRESS] state=completed phase=completed operation=complete completed=20 total=20 percent=100' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_steps=20' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_commands=8' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_internal_actions=11' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_cleanup_commands=0' "${SERIAL_LOG}"
grep -Fq '[AQUA-INSTALLER] stage=execution-run status=ok executed=true target=/dev/vdb' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-installed-content status=ok root_label=AQUA_ROOT efi_label=AQUA_EFI partlabel=AQUA_ROOT kernel=true bootloader=true configuration=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-installed-root-boot status=ok root=/dev/vda2 recovery=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-installed-uefi-boot status=ok firmware=edk2 bootloader=grub root=PARTLABEL=AQUA_ROOT recovery=true' "${SERIAL_LOG}"

{
    echo "target_disk=${TARGET_DISK}"
    echo "target_before_sha256=${target_before_sha256}"
    echo "target_after_sha256=${target_after_sha256}"
    echo "target_changed=true"
    echo "artifact_disk=${ARTIFACT_DISK}"
    echo "artifact_before_sha256=${artifact_before_sha256}"
    echo "artifact_after_sha256=${artifact_after_sha256}"
    echo "artifact_disk_unchanged=true"
    echo "installed_root_boot=true"
    echo "installed_uefi_boot=true"
    echo "uefi_firmware=${FIRMWARE}"
} > "${HASH_REPORT}"

echo "Aqua installer transaction QEMU check passed."
echo "Installed target hash: ${target_after_sha256}"
echo "Serial log: ${SERIAL_LOG}"
echo "Hash report: ${HASH_REPORT}"
