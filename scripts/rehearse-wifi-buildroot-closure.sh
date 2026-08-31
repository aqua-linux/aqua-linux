#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-aqua-linux-buildroot:dev}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-2025-02-work}"
BUILDROOT_VERSION="2025.02.17"
OUTPUT_DIR="/work/build/wifi-rehearsal-output"
EVIDENCE_DIR="${ROOT_DIR}/build/wifi-rehearsal"

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
        evidence_dir=/src/build/wifi-rehearsal

        test -s "${buildroot_dir}/Makefile"
        rsync -a --delete \
            --exclude build \
            --exclude target \
            --exclude website \
            --exclude .git \
            /src/ /work/
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" \
            aqua_x86_64_wifi_rehearsal_defconfig

        for symbol in \
            BR2_PACKAGE_WPA_SUPPLICANT \
            BR2_PACKAGE_WPA_SUPPLICANT_NL80211 \
            BR2_PACKAGE_WPA_SUPPLICANT_AUTOSCAN \
            BR2_PACKAGE_WPA_SUPPLICANT_WPA3 \
            BR2_PACKAGE_WPA_SUPPLICANT_WPA_CLIENT_SO \
            BR2_PACKAGE_WPA_SUPPLICANT_CTRL_IFACE \
            BR2_PACKAGE_LIBNL \
            BR2_PACKAGE_LIBOPENSSL
        do
            grep -Fxq "${symbol}=y" "${output_dir}/.config"
        done
        for symbol in \
            BR2_PACKAGE_WPA_SUPPLICANT_WEXT \
            BR2_PACKAGE_WPA_SUPPLICANT_WIRED \
            BR2_PACKAGE_WPA_SUPPLICANT_AP_SUPPORT \
            BR2_PACKAGE_WPA_SUPPLICANT_MESH_NETWORKING \
            BR2_PACKAGE_WPA_SUPPLICANT_EAP \
            BR2_PACKAGE_WPA_SUPPLICANT_HOTSPOT \
            BR2_PACKAGE_WPA_SUPPLICANT_WPS \
            BR2_PACKAGE_WPA_SUPPLICANT_CLI \
            BR2_PACKAGE_WPA_SUPPLICANT_PASSPHRASE \
            BR2_PACKAGE_WPA_SUPPLICANT_DBUS \
            BR2_PACKAGE_DBUS \
            BR2_PACKAGE_IWD \
            BR2_PACKAGE_CONNMAN \
            BR2_PACKAGE_NETWORK_MANAGER \
            BR2_PACKAGE_DHCPCD
        do
            ! grep -Fxq "${symbol}=y" "${output_dir}/.config"
        done

        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" wpa_supplicant
        test -x "${output_dir}/target/usr/sbin/wpa_supplicant"
        test -s "${output_dir}/target/usr/lib/libwpa_client.so"
        test ! -e "${output_dir}/target/usr/sbin/wpa_cli"
        test ! -e "${output_dir}/target/usr/bin/wpa_passphrase"
        ! grep -Eq "^[[:space:]]*(network=\\{|psk=)" \
            "${output_dir}/target/etc/wpa_supplicant.conf"

        make -s -C "${buildroot_dir}" O=/work/build/buildroot-output \
            BR2_EXTERNAL="${external_dir}" show-info > "${evidence_dir}/base-show-info.json"
        make -s -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" show-info > "${evidence_dir}/wifi-show-info.json"
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL="${external_dir}" legal-info

        cp "${output_dir}/.config" "${evidence_dir}/wifi-rehearsal.config"
        cp "${output_dir}/legal-info/manifest.csv" "${evidence_dir}/manifest.csv"
        python3 /work/scripts/write-wifi-buildroot-closure.py \
            --base "${evidence_dir}/base-show-info.json" \
            --wifi "${evidence_dir}/wifi-show-info.json" \
            --manifest "${evidence_dir}/manifest.csv" \
            --output "${evidence_dir}/closure.json"
    '

echo "Aqua Linux Wi-Fi Buildroot closure rehearsal passed."
echo "Evidence: ${EVIDENCE_DIR}/closure.json"
