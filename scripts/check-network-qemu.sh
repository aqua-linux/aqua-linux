#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-network-check.log}"
MEMORY="${MEMORY:-512M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"

for tool in expect qemu-system-x86_64; do
    command -v "${tool}" >/dev/null 2>&1 || {
        echo "Missing required tool: ${tool}" >&2
        exit 1
    }
done

for artifact in "${KERNEL}" "${ROOTFS}"; do
    test -f "${artifact}" || {
        echo "Missing QEMU artifact: ${artifact}" >&2
        echo "Run scripts/build-image-docker-volume.sh first." >&2
        exit 1
    }
done

mkdir -p "$(dirname "${SERIAL_LOG}")"
rm -f "${SERIAL_LOG}"

export ROOT_DIR KERNEL ROOTFS SERIAL_LOG MEMORY CPUS TIMEOUT_SECONDS
expect "${ROOT_DIR}/scripts/check-network-qemu.exp"

grep -Fq '[AQUA-BOOT] stage=network-service-activation status=started mode=supervised-root target=qemu interface=eth0' "${SERIAL_LOG}"
grep -Fq '[AQUA-NETWORK] stage=qemu-acceptance status=ok dhcp=true default_route=true dns_lookup=true renewal=true route_recovery=true service_recovery=true recovery_shell=true' "${SERIAL_LOG}"

echo 'Aqua Linux opt-in network QEMU acceptance passed.'
echo "Serial log: ${SERIAL_LOG}"
