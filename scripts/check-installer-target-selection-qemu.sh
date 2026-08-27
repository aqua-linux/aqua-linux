#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
TARGET_DISK="${TARGET_DISK:-${ROOT_DIR}/build/qemu-installer-target.qcow2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-installer-target-selection.log}"
HASH_REPORT="${HASH_REPORT:-${ROOT_DIR}/build/qemu-installer-target-selection.sha256}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"

for tool in expect qemu-img qemu-system-x86_64; do
    command -v "${tool}" >/dev/null 2>&1 || {
        echo "Missing required tool: ${tool}" >&2
        exit 1
    }
done
for artifact in "${KERNEL}" "${ROOTFS}"; do
    test -f "${artifact}" || {
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
qemu-img create -q -f qcow2 "${TARGET_DISK}" 4G
before_sha256="$(sha256_file "${TARGET_DISK}")"

export KERNEL ROOTFS TARGET_DISK SERIAL_LOG TIMEOUT_SECONDS
expect "${ROOT_DIR}/scripts/check-installer-target-selection-qemu.exp"

after_sha256="$(sha256_file "${TARGET_DISK}")"
test "${before_sha256}" = "${after_sha256}" || {
    echo "Disposable installer target changed during read-only probe" >&2
    exit 1
}
echo '[AQUA-HOST] stage=installer-target-disk-hash status=ok disk_unchanged=true' >> "${SERIAL_LOG}"

grep -Fq 'storage_candidate_count=2' "${SERIAL_LOG}"
grep -Fq 'storage_eligible_count=1' "${SERIAL_LOG}"
grep -Fq 'storage.00=device:/dev/vda' "${SERIAL_LOG}"
grep -Fq 'eligible:false blocked:running-system-disk' "${SERIAL_LOG}"
grep -Fq 'storage.01=device:/dev/vdb' "${SERIAL_LOG}"
grep -Fq 'eligible:true blocked:none' "${SERIAL_LOG}"
grep -Fq 'readiness_target_source=storage-probe' "${SERIAL_LOG}"
grep -Fq 'readiness_target_device=/dev/vdb' "${SERIAL_LOG}"
grep -Fq 'readiness_target_bound=true' "${SERIAL_LOG}"
grep -Fq 'readiness_target_selected_for_install=false' "${SERIAL_LOG}"
grep -Fq 'install_execution_armed=false' "${SERIAL_LOG}"
grep -Fq 'plan_target_device=/dev/vdb' "${SERIAL_LOG}"
grep -Fq 'disk_commands_executed=false' "${SERIAL_LOG}"
grep -Fq 'filesystem_writes_executed=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-target-selection-qemu status=ok target=/dev/vdb bound=true execution=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-HOST] stage=installer-target-disk-hash status=ok disk_unchanged=true' "${SERIAL_LOG}"

{
    echo "target_disk=${TARGET_DISK}"
    echo "before_sha256=${before_sha256}"
    echo "after_sha256=${after_sha256}"
    echo "disk_unchanged=true"
} > "${HASH_REPORT}"

echo "Aqua installer target selection QEMU check passed."
echo "Target hash: ${before_sha256}"
echo "Serial log: ${SERIAL_LOG}"
echo "Hash report: ${HASH_REPORT}"
