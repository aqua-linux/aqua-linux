#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-aqua-linux-buildroot:dev}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-2025-02-work}"
BUILDROOT_VERSION="2025.02.17"
OUTPUT_DIR="/work/build/audio-rehearsal-output"
EVIDENCE_DIR="${ROOT_DIR}/build/audio-rehearsal"

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
        evidence_dir=/src/build/audio-rehearsal

        test -s "${buildroot_dir}/Makefile"
        rsync -a --delete \
            --exclude build \
            --exclude target \
            --exclude .git \
            /src/ /work/
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" \
            aqua_x86_64_audio_rehearsal_defconfig

        for symbol in \
            BR2_PACKAGE_ALSA_LIB \
            BR2_PACKAGE_PIPEWIRE \
            BR2_PACKAGE_LUA_5_4 \
            BR2_PACKAGE_WIREPLUMBER \
            BR2_PACKAGE_LIBGLIB2 \
            BR2_PACKAGE_AQUA_AUDIO_NATIVE
        do
            grep -Fxq "${symbol}=y" "${output_dir}/.config"
        done
        grep -Fxq \
            "BR2_PACKAGE_ALSA_LIB_PCM_PLUGINS=\"hw plug rate route softvol null ioplug\"" \
            "${output_dir}/.config"
        grep -Fxq "BR2_PACKAGE_ALSA_LIB_CTL_PLUGINS=\"hw ext\"" \
            "${output_dir}/.config"
        for symbol in \
            BR2_PACKAGE_DBUS \
            BR2_PACKAGE_BLUEZ5_UTILS \
            BR2_PACKAGE_JACK2 \
            BR2_PACKAGE_PULSEAUDIO \
            BR2_PACKAGE_PIPEWIRE_GSTREAMER \
            BR2_PACKAGE_PIPEWIRE_V4L2
        do
            ! grep -Fxq "${symbol}=y" "${output_dir}/.config"
        done

        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" aqua-audio-native

        make -s -C "${buildroot_dir}" O=/work/build/buildroot-output \
            BR2_EXTERNAL="${external_dir}" show-info > "${evidence_dir}/base-show-info.json"
        make -s -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" show-info > "${evidence_dir}/audio-show-info.json"
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" legal-info

        cp "${output_dir}/.config" "${evidence_dir}/audio-rehearsal.config"
        cp "${output_dir}/legal-info/manifest.csv" "${evidence_dir}/manifest.csv"
        python3 /work/scripts/write-audio-buildroot-closure.py \
            --base "${evidence_dir}/base-show-info.json" \
            --audio "${evidence_dir}/audio-show-info.json" \
            --manifest "${evidence_dir}/manifest.csv" \
            --output "${evidence_dir}/closure.json"
    '

echo "Aqua Linux audio Buildroot closure rehearsal passed."
echo "Evidence: ${EVIDENCE_DIR}/closure.json"
