#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-aqua-linux-buildroot:dev}"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-work}"

cd "${ROOT_DIR}"

docker build -f Dockerfile.buildroot -t "${IMAGE_NAME}" .
docker volume create "${VOLUME_NAME}" >/dev/null

docker run --rm \
    -e FORCE_UNSAFE_CONFIGURE=1 \
    -v "${ROOT_DIR}:/src" \
    -v "${VOLUME_NAME}:/work" \
    -w /work \
    "${IMAGE_NAME}" \
    /bin/bash -lc '
        set -euo pipefail
        rsync -a --delete \
            --exclude build \
            --exclude target \
            --exclude .git \
            /src/ /work/
        if [ -f /src/target/x86_64-unknown-linux-musl/release/aqua-compositor ]; then
            mkdir -p /work/target/x86_64-unknown-linux-musl/release
            cp /src/target/x86_64-unknown-linux-musl/release/aqua-compositor \
                /work/target/x86_64-unknown-linux-musl/release/aqua-compositor
        fi
        if [ -f /src/target/x86_64-unknown-linux-musl/release/aqua-files ]; then
            mkdir -p /work/target/x86_64-unknown-linux-musl/release
            cp /src/target/x86_64-unknown-linux-musl/release/aqua-files \
                /work/target/x86_64-unknown-linux-musl/release/aqua-files
        fi
        if [ -f /src/target/x86_64-unknown-linux-musl/release/aqua-settings ]; then
            mkdir -p /work/target/x86_64-unknown-linux-musl/release
            cp /src/target/x86_64-unknown-linux-musl/release/aqua-settings \
                /work/target/x86_64-unknown-linux-musl/release/aqua-settings
        fi
        if [ -f /src/target/x86_64-unknown-linux-musl/release/aqua-properties ]; then
            mkdir -p /work/target/x86_64-unknown-linux-musl/release
            cp /src/target/x86_64-unknown-linux-musl/release/aqua-properties \
                /work/target/x86_64-unknown-linux-musl/release/aqua-properties
        fi
        if [ -f /src/target/x86_64-unknown-linux-musl/release/aqua-terminal ]; then
            mkdir -p /work/target/x86_64-unknown-linux-musl/release
            cp /src/target/x86_64-unknown-linux-musl/release/aqua-terminal \
                /work/target/x86_64-unknown-linux-musl/release/aqua-terminal
        fi
        if [ -f /src/target/x86_64-unknown-linux-musl/release/aqua-installer ]; then
            mkdir -p /work/target/x86_64-unknown-linux-musl/release
            cp /src/target/x86_64-unknown-linux-musl/release/aqua-installer \
                /work/target/x86_64-unknown-linux-musl/release/aqua-installer
        fi
        if [ -f /src/target/x86_64-unknown-linux-musl/release/aqua-installer-probe ]; then
            mkdir -p /work/target/x86_64-unknown-linux-musl/release
            cp /src/target/x86_64-unknown-linux-musl/release/aqua-installer-probe \
                /work/target/x86_64-unknown-linux-musl/release/aqua-installer-probe
        fi
        scripts/build-image.sh "$@"
        mkdir -p /src/build/buildroot-output/images
        cp -a /work/build/buildroot-output/images/. /src/build/buildroot-output/images/
        cp -a /work/build/buildroot-output/.config /src/build/buildroot-output/.config
    ' bash "$@"
