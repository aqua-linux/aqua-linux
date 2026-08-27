#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BUILDROOT_VERSION="${BUILDROOT_VERSION:-2024.02.12}"
BUILDROOT_ARCHIVE="buildroot-${BUILDROOT_VERSION}.tar.xz"
BUILDROOT_URL="https://buildroot.org/downloads/${BUILDROOT_ARCHIVE}"
BUILD_DIR="${ROOT_DIR}/build"
DOWNLOAD_DIR="${BUILD_DIR}/downloads"
BUILDROOT_DIR="${BUILD_DIR}/buildroot-${BUILDROOT_VERSION}"
OUTPUT_DIR="${BUILD_DIR}/buildroot-output"
EXTERNAL_DIR="${ROOT_DIR}/br2-external/aqua"

unset LD_LIBRARY_PATH

find_gnu_tool() {
    for tool in "$@"; do
        if command -v "${tool}" >/dev/null 2>&1 && "${tool}" -v 2>&1 | grep -q '^gcc version'; then
            command -v "${tool}"
            return 0
        fi
    done

    return 1
}

if [ -z "${HOSTCC:-}" ]; then
    HOSTCC="$(find_gnu_tool gcc gcc-15 gcc-14 gcc-13 gcc-12 gcc-11 || true)"
fi

if [ -z "${HOSTCXX:-}" ]; then
    HOSTCXX="$(find_gnu_tool g++ g++-15 g++-14 g++-13 g++-12 g++-11 || true)"
fi

if [ -z "${HOSTCC}" ] || [ -z "${HOSTCXX}" ]; then
    echo "Buildroot needs GNU gcc/g++ as host compilers." >&2
    echo "Apple clang's gcc-compatible shim does not pass Buildroot 2024.02 host checks." >&2
    echo "Use a Linux build host/container, or install GNU gcc and set HOSTCC/HOSTCXX." >&2
    exit 1
fi

mkdir -p "${DOWNLOAD_DIR}"

if [ ! -d "${BUILDROOT_DIR}" ]; then
    if [ ! -f "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}" ]; then
        echo "Downloading Buildroot ${BUILDROOT_VERSION}..."
        curl -L "${BUILDROOT_URL}" -o "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}"
    fi

    echo "Extracting Buildroot ${BUILDROOT_VERSION}..."
    tar -C "${BUILD_DIR}" -xf "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}"
fi

make -C "${BUILDROOT_DIR}" O="${OUTPUT_DIR}" BR2_EXTERNAL="${EXTERNAL_DIR}" HOSTCC="${HOSTCC}" HOSTCXX="${HOSTCXX}" aqua_x86_64_defconfig

make -C "${BUILDROOT_DIR}" O="${OUTPUT_DIR}" BR2_EXTERNAL="${EXTERNAL_DIR}" HOSTCC="${HOSTCC}" HOSTCXX="${HOSTCXX}" "$@"

if [ -f "${OUTPUT_DIR}/images/rootfs.ext2" ]; then
    cp "${OUTPUT_DIR}/images/rootfs.ext2" "${OUTPUT_DIR}/images/disk.img"
fi

echo "Aqua Linux image artifact paths:"
for artifact in \
    "${OUTPUT_DIR}/images/bzImage" \
    "${OUTPUT_DIR}/images/rootfs.ext2" \
    "${OUTPUT_DIR}/images/disk.img"
do
    if [ -f "${artifact}" ]; then
        echo "  ready: ${artifact}"
    else
        echo "  pending: ${artifact}"
    fi
done
