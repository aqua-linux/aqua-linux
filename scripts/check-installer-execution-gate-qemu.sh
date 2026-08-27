#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
TARGET_DISK="${TARGET_DISK:-${ROOT_DIR}/build/qemu-installer-gate-target.qcow2}"
ARTIFACT_DISK="${ARTIFACT_DISK:-${ROOT_DIR}/build/installer-artifacts.ext4}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-installer-execution-gate.log}"
HASH_REPORT="${HASH_REPORT:-${ROOT_DIR}/build/qemu-installer-execution-gate.sha256}"
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
ARTIFACT_DISK="${ARTIFACT_DISK}" "${ROOT_DIR}/scripts/write-installer-artifact-disk-docker.sh"
qemu-img create -q -f qcow2 "${TARGET_DISK}" 4G
before_sha256="$(sha256_file "${TARGET_DISK}")"
artifact_before_sha256="$(sha256_file "${ARTIFACT_DISK}")"

export KERNEL ROOTFS TARGET_DISK ARTIFACT_DISK SERIAL_LOG TIMEOUT_SECONDS
expect "${ROOT_DIR}/scripts/check-installer-execution-gate-qemu.exp"

after_sha256="$(sha256_file "${TARGET_DISK}")"
artifact_after_sha256="$(sha256_file "${ARTIFACT_DISK}")"
test "${before_sha256}" = "${after_sha256}" || {
    echo "Disposable installer target changed while authorizing execution gate" >&2
    exit 1
}
test "${artifact_before_sha256}" = "${artifact_after_sha256}" || {
    echo "Read-only installer artifact disk changed during gate validation" >&2
    exit 1
}
echo '[AQUA-HOST] stage=installer-execution-gate-disk-hash status=ok disk_unchanged=true' >> "${SERIAL_LOG}"

grep -Fq '[AQUA-TEST] stage=installer-execution-gate-wrong-confirmation status=ok rejected=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-execution-gate-missing-operator-enable status=ok rejected=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=installer-artifacts-qemu status=ok device=/dev/vdc readonly=true manifest=verified artifacts=3' "${SERIAL_LOG}"
grep -Fq 'execution_gate_status=authorized-no-execution' "${SERIAL_LOG}"
grep -Fq 'execution_gate_qemu_runtime=true' "${SERIAL_LOG}"
grep -Fq 'execution_gate_kernel_cmdline=true' "${SERIAL_LOG}"
grep -Fq 'execution_gate_operator_enable=true' "${SERIAL_LOG}"
grep -Fq 'execution_gate_target_revalidated=true' "${SERIAL_LOG}"
grep -Fq 'execution_gate_artifacts_staged=true' "${SERIAL_LOG}"
grep -Fq 'execution_gate_artifact_manifest_verified=true' "${SERIAL_LOG}"
grep -Fq 'execution_gate_confirmation_exact=true' "${SERIAL_LOG}"
grep -Fq 'execution_gate_expected_confirmation=ERASE /dev/vdb' "${SERIAL_LOG}"
grep -Fq 'execution_gate_target_device=/dev/vdb' "${SERIAL_LOG}"
grep -Fq 'execution_gate_transaction_steps=20' "${SERIAL_LOG}"
grep -Fq 'install_execution_armed=true' "${SERIAL_LOG}"
grep -Fq 'transaction_execution_started=false' "${SERIAL_LOG}"
grep -Fq 'disk_commands_executed=false' "${SERIAL_LOG}"
grep -Fq 'filesystem_writes_executed=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-INSTALLER] stage=execution-gate status=authorized executed=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-HOST] stage=installer-execution-gate-disk-hash status=ok disk_unchanged=true' "${SERIAL_LOG}"

{
    echo "target_disk=${TARGET_DISK}"
    echo "before_sha256=${before_sha256}"
    echo "after_sha256=${after_sha256}"
    echo "disk_unchanged=true"
    echo "artifact_disk=${ARTIFACT_DISK}"
    echo "artifact_before_sha256=${artifact_before_sha256}"
    echo "artifact_after_sha256=${artifact_after_sha256}"
    echo "artifact_disk_unchanged=true"
} > "${HASH_REPORT}"

echo "Aqua installer execution gate QEMU check passed."
echo "Target hash: ${before_sha256}"
echo "Serial log: ${SERIAL_LOG}"
echo "Hash report: ${HASH_REPORT}"
