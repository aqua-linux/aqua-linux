#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-aqua-linux-buildroot:dev}"

cd "${ROOT_DIR}"

docker build -f Dockerfile.buildroot -t "${IMAGE_NAME}" .
docker run --rm \
    -e FORCE_UNSAFE_CONFIGURE=1 \
    -u "$(id -u):$(id -g)" \
    -v "${ROOT_DIR}:/work" \
    -w /work \
    "${IMAGE_NAME}" \
    scripts/build-image.sh "$@"
