#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
STAGING_ROOT="${STAGING_ROOT:-${ROOT_DIR}/build/installer-artifacts-root}"
ARTIFACT_DISK="${ARTIFACT_DISK:-${ROOT_DIR}/build/installer-artifacts.ext4}"
IMAGE_NAME="${IMAGE_NAME:-aqua-linux-buildroot:dev}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-2025-02-work}"
ARTIFACT_SIZE="${ARTIFACT_SIZE:-192M}"
ARTIFACT_DIRECTORY="$(dirname "${ARTIFACT_DISK}")"
ARTIFACT_FILENAME="$(basename "${ARTIFACT_DISK}")"

ROOTFS_ARCHIVE="${IMAGE_DIR}/rootfs.tar"
KERNEL_IMAGE="${IMAGE_DIR}/bzImage"
BOOTLOADER_IMAGE="${IMAGE_DIR}/efi-part/EFI/BOOT/bootx64.efi"

for artifact in "${ROOTFS_ARCHIVE}" "${KERNEL_IMAGE}" "${BOOTLOADER_IMAGE}"; do
    test -s "${artifact}" || {
        echo "Missing non-empty installer artifact: ${artifact}" >&2
        exit 1
    }
done
command -v docker >/dev/null 2>&1 || {
    echo "Missing required tool: docker" >&2
    exit 1
}

sha256_file() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        sha256sum "$1" | awk '{print $1}'
    fi
}

rm -rf "${STAGING_ROOT}"
mkdir -p "${STAGING_ROOT}"
cp "${ROOTFS_ARCHIVE}" "${STAGING_ROOT}/rootfs.tar"
cp "${KERNEL_IMAGE}" "${STAGING_ROOT}/bzImage"
cp "${BOOTLOADER_IMAGE}" "${STAGING_ROOT}/bootx64.efi"
{
    echo "$(sha256_file "${ROOTFS_ARCHIVE}")  rootfs.tar"
    echo "$(sha256_file "${KERNEL_IMAGE}")  bzImage"
    echo "$(sha256_file "${BOOTLOADER_IMAGE}")  bootx64.efi"
} > "${STAGING_ROOT}/manifest.sha256"

rm -f "${ARTIFACT_DISK}"
mkdir -p "${ARTIFACT_DIRECTORY}"
docker run --rm \
    -v "${STAGING_ROOT}:/staging:ro" \
    -v "${ARTIFACT_DIRECTORY}:/out" \
    -v "${VOLUME_NAME}:/work" \
    "${IMAGE_NAME}" \
    /work/build/buildroot-output/host/sbin/mkfs.ext4 \
        -q -d /staging \
        -r 1 -m 0 -L AQUA_INSTALL \
        "/out/${ARTIFACT_FILENAME}" "${ARTIFACT_SIZE}"

test -s "${ARTIFACT_DISK}"
echo "installer_artifact_disk=${ARTIFACT_DISK}"
echo "installer_artifact_disk_sha256=$(sha256_file "${ARTIFACT_DISK}")"
echo "installer_artifact_manifest=${STAGING_ROOT}/manifest.sha256"
echo "[AQUA-HOST] stage=installer-artifact-disk status=ok artifacts=3 filesystem=ext4 read_only_at_guest=true"
