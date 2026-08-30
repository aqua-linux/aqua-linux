#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${AQUA_COMPOSITOR_BUILDER_IMAGE:-rust:trixie}"
TARGET_TRIPLE="${AQUA_COMPOSITOR_TARGET:-x86_64-unknown-linux-musl}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-2025-02-work}"
AUDIO_BUILDROOT_OUTPUT="/work/build/audio-rehearsal-output"
case "$(docker info --format '{{.Architecture}}')" in
    aarch64 | arm64) DOCKER_PLATFORM="linux/arm64" ;;
    x86_64 | amd64) DOCKER_PLATFORM="linux/amd64" ;;
    *) echo "Unsupported Docker architecture" >&2; exit 1 ;;
esac

docker volume inspect "${VOLUME_NAME}" >/dev/null
docker run --rm \
    --platform "${DOCKER_PLATFORM}" \
    -v "${ROOT_DIR}:/src" \
    -v "${VOLUME_NAME}:/work" \
    -w /src \
    "${IMAGE_NAME}" \
    sh -eu -c "
        linker=${AUDIO_BUILDROOT_OUTPUT}/host/bin/x86_64-buildroot-linux-musl-gcc
        sysroot=${AUDIO_BUILDROOT_OUTPUT}/host/x86_64-buildroot-linux-musl/sysroot
        test -x \"\${linker}\"
        test -f \"\${sysroot}/usr/lib/libaqua-audio-native.so\"
        rustup target add ${TARGET_TRIPLE}
        export PATH=${AUDIO_BUILDROOT_OUTPUT}/host/bin:\${PATH}
        export AQUA_AUDIO_BUILDROOT_LINKER=\"\${linker}\"
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=/src/scripts/audio-buildroot-linker.sh \
        RUSTFLAGS=\"-C target-feature=-crt-static -L native=\${sysroot}/usr/lib\" \
            cargo build -p aqua-service-adapters --release \
                --target ${TARGET_TRIPLE} --features wireplumber-native \
                --bin aqua-audio-adapter-probe
    "

echo "Aqua audio adapter probe Linux binary:"
echo "  ${ROOT_DIR}/target/${TARGET_TRIPLE}/release/aqua-audio-adapter-probe"
