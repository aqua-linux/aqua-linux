#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BUILDROOT_VERSION="${BUILDROOT_VERSION:-2025.02.17}"
BUILDROOT_SHA256="${BUILDROOT_SHA256:-13618704563ad0b928a4564aaa73e2db97e12e8df0ed5ae874744a83964a023a}"
BUILDROOT_ARCHIVE="buildroot-${BUILDROOT_VERSION}.tar.xz"
BUILDROOT_URL="https://buildroot.org/downloads/${BUILDROOT_ARCHIVE}"
BUILD_DIR="${ROOT_DIR}/build"
DOWNLOAD_DIR="${BUILD_DIR}/downloads"
BUILDROOT_DIR="${BUILD_DIR}/buildroot-${BUILDROOT_VERSION}"
OUTPUT_DIR="${BUILD_DIR}/buildroot-output"
EXTERNAL_DIR="${ROOT_DIR}/br2-external/aqua"

unset LD_LIBRARY_PATH

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

verify_buildroot_archive() {
    actual_sha256="$(sha256_file "$1")"
    if [ "${actual_sha256}" != "${BUILDROOT_SHA256}" ]; then
        echo "Buildroot archive checksum mismatch: $1" >&2
        echo "Expected: ${BUILDROOT_SHA256}" >&2
        echo "Actual:   ${actual_sha256}" >&2
        exit 1
    fi
}

buildroot_archive_is_valid() {
    [ "$(sha256_file "$1")" = "${BUILDROOT_SHA256}" ]
}

validate_buildroot_source() {
    [ -s "$1/Makefile" ] &&
        [ -s "$1/Config.in" ] &&
        [ -s "$1/support/scripts/br2-external" ]
}

extract_buildroot_source() {
    extract_dir="${BUILD_DIR}/.buildroot-extract.$$"
    extracted_source="${extract_dir}/buildroot-${BUILDROOT_VERSION}"

    rm -rf "${extract_dir}"
    mkdir -p "${extract_dir}"
    trap 'rm -rf "${extract_dir}"' EXIT INT TERM

    echo "Extracting Buildroot ${BUILDROOT_VERSION}..."
    tar -C "${extract_dir}" -xf "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}"
    if ! validate_buildroot_source "${extracted_source}"; then
        echo "Extracted Buildroot source failed integrity checks." >&2
        exit 1
    fi

    mv "${extracted_source}" "${BUILDROOT_DIR}"
    rm -rf "${extract_dir}"
    trap - EXIT INT TERM
}

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
    echo "Apple clang's gcc-compatible shim does not pass Buildroot 2025.02 host checks." >&2
    echo "Use a Linux build host/container, or install GNU gcc and set HOSTCC/HOSTCXX." >&2
    exit 1
fi

mkdir -p "${DOWNLOAD_DIR}"

if [ -f "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}" ] &&
    ! buildroot_archive_is_valid "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}"
then
    archive_quarantine_suffix="$(date -u +%Y%m%dT%H%M%SZ).$$"
    archive_quarantine="${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}.invalid.${archive_quarantine_suffix}"
    echo "Quarantining invalid Buildroot archive at ${archive_quarantine}."
    mv "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}" "${archive_quarantine}"
fi

if [ ! -f "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}" ]; then
    echo "Downloading Buildroot ${BUILDROOT_VERSION}..."
    DOWNLOAD_TEMP="${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}.tmp.$$"
    trap 'rm -f "${DOWNLOAD_TEMP}"' EXIT INT TERM
    curl -fL --retry 3 "${BUILDROOT_URL}" -o "${DOWNLOAD_TEMP}"
    verify_buildroot_archive "${DOWNLOAD_TEMP}"
    mv "${DOWNLOAD_TEMP}" "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}"
    trap - EXIT INT TERM
fi

verify_buildroot_archive "${DOWNLOAD_DIR}/${BUILDROOT_ARCHIVE}"

if ! validate_buildroot_source "${BUILDROOT_DIR}"; then
    quarantine_suffix="$(date -u +%Y%m%dT%H%M%SZ).$$"

    if [ -e "${BUILDROOT_DIR}" ]; then
        quarantine_source="${BUILDROOT_DIR}.invalid.${quarantine_suffix}"
        echo "Quarantining incomplete Buildroot source at ${quarantine_source}."
        mv "${BUILDROOT_DIR}" "${quarantine_source}"
    fi

    if [ -e "${OUTPUT_DIR}" ]; then
        quarantine_output="${OUTPUT_DIR}.invalid.${quarantine_suffix}"
        echo "Quarantining incompatible Buildroot output at ${quarantine_output}."
        mv "${OUTPUT_DIR}" "${quarantine_output}"
    fi

    extract_buildroot_source
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
