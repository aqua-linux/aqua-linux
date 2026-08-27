#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${AQUA_COMPOSITOR_BUILDER_IMAGE:-rust:trixie}"
TARGET_TRIPLE="${AQUA_COMPOSITOR_TARGET:-x86_64-unknown-linux-musl}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-work}"
BUILDROOT_OUTPUT="/aqua-buildroot/build/buildroot-output"
if [ -z "${AQUA_COMPOSITOR_BUILDER_PLATFORM:-}" ]; then
    case "$(docker info --format '{{.Architecture}}')" in
        aarch64 | arm64) DOCKER_PLATFORM="linux/arm64" ;;
        x86_64 | amd64) DOCKER_PLATFORM="linux/amd64" ;;
        *) echo "Unsupported Docker architecture" >&2; exit 1 ;;
    esac
else
    DOCKER_PLATFORM="${AQUA_COMPOSITOR_BUILDER_PLATFORM}"
fi

docker run --rm \
    --platform "${DOCKER_PLATFORM}" \
    -v "${ROOT_DIR}:/work" \
    -v "${VOLUME_NAME}:/aqua-buildroot" \
    -w /work \
    "${IMAGE_NAME}" \
    sh -eu -c "
        linker=${BUILDROOT_OUTPUT}/host/bin/x86_64-buildroot-linux-musl-gcc
        sysroot=${BUILDROOT_OUTPUT}/host/x86_64-buildroot-linux-musl/sysroot
        test -x \"\${linker}\"
        test -f \"\${sysroot}/usr/lib/libxkbcommon.so\"
        test -f \"\${sysroot}/usr/lib/libinput.so\"
        test -f \"\${sysroot}/usr/lib/libudev.so\"
        test -f \"\${sysroot}/usr/lib/libgbm.so\"
        test -f \"\${sysroot}/usr/lib/libEGL.so\"
        test -f \"\${sysroot}/usr/lib/libGLESv2.so\"
        rustup target add ${TARGET_TRIPLE}
        export PKG_CONFIG=${BUILDROOT_OUTPUT}/host/bin/pkg-config
        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_SYSROOT_DIR=\${sysroot}
        export PKG_CONFIG_PATH=\${sysroot}/usr/lib/pkgconfig:\${sysroot}/usr/share/pkgconfig
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=\"\${linker}\" \
        RUSTFLAGS=\"-C target-feature=-crt-static -L native=\${sysroot}/usr/lib\" \
            cargo build -p aqua-compositor --release --target ${TARGET_TRIPLE} --features smithay-gpu
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=\"\${linker}\" \
        RUSTFLAGS=\"-C target-feature=-crt-static -L native=\${sysroot}/usr/lib\" \
            cargo build -p aqua-installer --release --target ${TARGET_TRIPLE} --bin aqua-installer-probe
    "

echo "Aqua compositor Linux binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-compositor"
echo "Aqua Files Linux binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-files"
echo "Aqua Settings Linux binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-settings"
echo "Aqua Properties Linux binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-properties"
echo "Aqua Terminal Linux binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-terminal"
echo "Aqua Installer probe Linux binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-installer-probe"
echo "Aqua Installer Wayland binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-installer"
