#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-aqua-linux-buildroot:dev}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-2025-02-work}"
BUILDROOT_VERSION="2025.02.17"
OUTPUT_DIR="/work/build/audio-rehearsal-output"
EVIDENCE_DIR="${ROOT_DIR}/build/audio-rootfs-contract"

cd "${ROOT_DIR}"
docker build -f Dockerfile.buildroot -t "${IMAGE_NAME}" .
docker volume inspect "${VOLUME_NAME}" >/dev/null
mkdir -p "${EVIDENCE_DIR}"

docker run --rm \
    -e FORCE_UNSAFE_CONFIGURE=1 \
    -v "${ROOT_DIR}:/src" \
    -v "${VOLUME_NAME}:/work" \
    -w /work \
    "${IMAGE_NAME}" \
    /bin/bash -lc '
        set -euo pipefail
        buildroot_dir="/work/build/buildroot-'"${BUILDROOT_VERSION}"'"
        output_dir="'"${OUTPUT_DIR}"'"
        external_dir=/work/br2-external/aqua
        evidence_dir=/src/build/audio-rootfs-contract

        test -s "${buildroot_dir}/Makefile"
        rsync -a --delete \
            --exclude build \
            --exclude target \
            --exclude .git \
            /src/ /work/
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" \
            aqua_x86_64_audio_rehearsal_defconfig
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}"

        final_rootfs="$(mktemp -d /work/build/audio-rootfs-contract-final.XXXXXX)"
        trap '\''rm -rf "${final_rootfs}"'\'' EXIT
        tar -xf "${output_dir}/images/rootfs.tar" -C "${final_rootfs}"
        rootfs="${final_rootfs}"
        test -x "${rootfs}/usr/bin/aqua-audio-rootfs-check"
        AQUA_AUDIO_ROOT="${rootfs}" \
            "${rootfs}/usr/bin/aqua-audio-rootfs-check" | tee \
            "${evidence_dir}/rootfs-contract.txt"
        grep -Fxq "enabled=true" "${rootfs}/etc/aqua/media-services.conf"
        grep -Fxq "automatic_root_service=false" \
            "${rootfs}/etc/aqua/media-services.conf"
        grep -Fxq \
            '\''BR2_ROOTFS_OVERLAY="$(BR2_EXTERNAL_AQUA_PATH)/rootfs-overlay $(BR2_EXTERNAL_AQUA_PATH)/audio-rootfs-overlay"'\'' \
            "${output_dir}/.config"
        cp "${output_dir}/.config" "${evidence_dir}/audio-rootfs.config"
        sha256sum \
            "${rootfs}/etc/aqua/audio-stack.conf" \
            "${rootfs}/etc/aqua/media-services.conf" \
            "${rootfs}/usr/bin/pipewire" \
            "${rootfs}/usr/bin/wireplumber" \
            "${rootfs}/usr/lib/libaqua-audio-native.so.1" > \
            "${evidence_dir}/rootfs-contract.sha256"
    '

echo 'Aqua Linux audio rootfs contract rehearsal passed.'
echo "Evidence: ${EVIDENCE_DIR}/rootfs-contract.txt"
