#!/bin/sh
set -eu

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BUILD_SCRIPT="$REPO_ROOT/scripts/build-image.sh"
BUILD_DOC="$REPO_ROOT/docs/aqua-linux/buildroot.md"
EXPECTED_VERSION='2025.02.17'
EXPECTED_SHA256='13618704563ad0b928a4564aaa73e2db97e12e8df0ed5ae874744a83964a023a'
EXPECTED_VOLUME='aqua-linux-buildroot-2025-02-work'
DEFCONFIG="$REPO_ROOT/br2-external/aqua/configs/aqua_x86_64_defconfig"

grep -Fq "BUILDROOT_VERSION=\"\${BUILDROOT_VERSION:-${EXPECTED_VERSION}}\"" "$BUILD_SCRIPT"
grep -Fq "BUILDROOT_SHA256=\"\${BUILDROOT_SHA256:-${EXPECTED_SHA256}}\"" "$BUILD_SCRIPT"
grep -Fq 'curl -fL --retry 3' "$BUILD_SCRIPT"
grep -Fq 'verify_buildroot_archive "${DOWNLOAD_TEMP}"' "$BUILD_SCRIPT"
grep -Fq 'verify_buildroot_archive "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}"' "$BUILD_SCRIPT"
grep -Fq 'Quarantining invalid Buildroot archive' "$BUILD_SCRIPT"
grep -Fq 'validate_buildroot_source "${BUILDROOT_DIR}"' "$BUILD_SCRIPT"
grep -Fq 'Quarantining incomplete Buildroot source' "$BUILD_SCRIPT"
grep -Fq 'Quarantining incompatible Buildroot output' "$BUILD_SCRIPT"
grep -Fq 'extract_buildroot_source' "$BUILD_SCRIPT"
grep -Fq "$EXPECTED_VERSION" "$BUILD_DOC"
grep -Fxq 'BR2_PACKAGE_HOST_LINUX_HEADERS_CUSTOM_6_6=y' "$DEFCONFIG"

for script in \
    scripts/build-image-docker-volume.sh \
    scripts/build-compositor-linux-docker.sh \
    scripts/write-installer-artifact-disk-docker.sh
do
    grep -Fq "$EXPECTED_VOLUME" "$REPO_ROOT/$script"
done

echo 'Aqua Linux Buildroot LTS pin checks passed.'
