#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-aqua-linux-buildroot:dev}"
RUST_IMAGE="${AQUA_COMPOSITOR_BUILDER_IMAGE:-rust:trixie}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-2025-02-work}"
BUILDROOT_VERSION="2025.02.17"
OUTPUT_DIR="/work/build/wifi-qemu-output"
TARGET_TRIPLE="x86_64-unknown-linux-musl"
EVIDENCE_DIR="$ROOT_DIR/build/wifi-qemu-runtime"
case "$(docker info --format '{{.Architecture}}')" in
    aarch64|arm64) BUILD_PLATFORM=linux/arm64 ;;
    x86_64|amd64) BUILD_PLATFORM=linux/amd64 ;;
    *) echo 'Unsupported Docker architecture' >&2; exit 1 ;;
esac

docker build -f "$ROOT_DIR/Dockerfile.buildroot" -t "$IMAGE_NAME" "$ROOT_DIR"
docker volume inspect "$VOLUME_NAME" >/dev/null
mkdir -p "$EVIDENCE_DIR"

docker run --rm \
    -e FORCE_UNSAFE_CONFIGURE=1 \
    -v "$ROOT_DIR:/src" \
    -v "$VOLUME_NAME:/work" \
    -w /work \
    "$IMAGE_NAME" \
    /bin/bash -lc '
        set -euo pipefail
        buildroot_dir="/work/build/buildroot-'"$BUILDROOT_VERSION"'"
        output_dir="'"$OUTPUT_DIR"'"
        rsync -a --delete --exclude build --exclude target --exclude .git /src/ /work/
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL=/work/br2-external/aqua \
            aqua_x86_64_wifi_qemu_defconfig
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL=/work/br2-external/aqua hostapd-dirclean
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL=/work/br2-external/aqua aqua-wifi-native
    '

docker run --rm \
    --platform "$BUILD_PLATFORM" \
    -v "$ROOT_DIR:/src" \
    -v "$VOLUME_NAME:/work" \
    -w /src \
    "$RUST_IMAGE" \
    sh -eu -c '
        output="'"$OUTPUT_DIR"'"
        target="'"$TARGET_TRIPLE"'"
        linker="${output}/host/bin/x86_64-buildroot-linux-musl-gcc"
        sysroot="${output}/host/x86_64-buildroot-linux-musl/sysroot"
        test -x "${linker}"
        test -s "${sysroot}/usr/lib/libaqua-wifi-native.so"
        rustup target add "${target}"
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="${linker}" \
        RUSTFLAGS="-C target-feature=-crt-static -L native=${sysroot}/usr/lib" \
            cargo build -p aqua-service-adapters --release \
                --target "${target}" --features wifi-native \
                --bin aqua-network-broker --bin aqua-wifi-native-probe
    '

docker run --rm \
    -e FORCE_UNSAFE_CONFIGURE=1 \
    -v "$ROOT_DIR:/src" \
    -v "$VOLUME_NAME:/work" \
    -w /work \
    "$IMAGE_NAME" \
    /bin/bash -lc '
        set -euo pipefail
        buildroot_dir="/work/build/buildroot-'"$BUILDROOT_VERSION"'"
        output_dir="'"$OUTPUT_DIR"'"
        evidence=/src/build/wifi-qemu-runtime
        mkdir -p /work/target/'"$TARGET_TRIPLE"'/release
        cp /src/target/'"$TARGET_TRIPLE"'/release/aqua-network-broker \
            /work/target/'"$TARGET_TRIPLE"'/release/aqua-network-broker
        cp /src/target/'"$TARGET_TRIPLE"'/release/aqua-wifi-native-probe \
            /work/target/'"$TARGET_TRIPLE"'/release/aqua-wifi-native-probe
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL=/work/br2-external/aqua
        make -C "${buildroot_dir}" O="${output_dir}" \
            BR2_EXTERNAL=/work/br2-external/aqua legal-info
        test -s "${output_dir}/images/bzImage"
        test -s "${output_dir}/images/rootfs.ext2"
        grep -Fxq CONFIG_MAC80211_HWSIM=y "${output_dir}/build/linux-6.6.32/.config"
        grep -Fxq CONFIG_CFG80211=y "${output_dir}/build/linux-6.6.32/.config"
        grep -Fxq CONFIG_MAC80211=y "${output_dir}/build/linux-6.6.32/.config"
        test -x "${output_dir}/target/usr/sbin/wpa_supplicant"
        test -x "${output_dir}/target/usr/sbin/hostapd"
        test -x "${output_dir}/target/usr/bin/aqua-wifi-service-supervisor"
        test -x "${output_dir}/target/usr/bin/aqua-network-broker"
        test -x "${output_dir}/target/usr/libexec/aqua-tests/aqua-wifi-native-probe"
        grep -Fq "\"hostapd\",\"2.11\",\"BSD-3-Clause\"" \
            "${output_dir}/legal-info/manifest.csv"
        grep -Fq "\"wpa_supplicant\",\"2.12\",\"BSD-3-Clause\"" \
            "${output_dir}/legal-info/manifest.csv"
        cp "${output_dir}/images/bzImage" "${evidence}/bzImage"
        cp "${output_dir}/images/rootfs.ext2" "${evidence}/rootfs.ext2"
        cp "${output_dir}/.config" "${evidence}/buildroot.config"
        cp "${output_dir}/build/linux-6.6.32/.config" "${evidence}/linux.config"
        cp "${output_dir}/legal-info/manifest.csv" "${evidence}/legal-manifest.csv"
    '

echo 'Aqua Linux opt-in Wi-Fi QEMU runtime build passed.'
echo "Evidence: $EVIDENCE_DIR"
