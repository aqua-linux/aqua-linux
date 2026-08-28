#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
TEMP_ROOT="$(mktemp -d)"
trap 'rm -rf "${TEMP_ROOT}"' EXIT HUP INT TERM

POST_BUILD="${ROOT_DIR}/br2-external/aqua/board/aqua/x86_64/post-build.sh"
ROOTFS_OVERLAY="${ROOT_DIR}/br2-external/aqua/rootfs-overlay"
DEFAULT_CONFIG="${TEMP_ROOT}/etc/aqua/compositor-session.conf"
DEFAULT_ENV="${TEMP_ROOT}/etc/aqua/session.env"
INITTAB="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/etc/inittab"
GRUB_CONFIG="${ROOT_DIR}/br2-external/aqua/board/aqua/x86_64/grub.cfg"
QEMU_RUNNER="${ROOT_DIR}/scripts/run-qemu.sh"

cp -R "${ROOTFS_OVERLAY}/." "${TEMP_ROOT}/"
"${POST_BUILD}" "${TEMP_ROOT}"

need_exact_once() {
    file="$1"
    line="$2"
    count="$(grep -Fxc "${line}" "${file}")"
    test "${count}" = "1" || {
        echo "Expected exactly one '${line}' in ${file}, found ${count}" >&2
        exit 1
    }
}

need_exact_once "${DEFAULT_CONFIG}" 'autostart=false'
need_exact_once "${DEFAULT_CONFIG}" 'boot_graphics=false'
need_exact_once "${DEFAULT_CONFIG}" 'recovery_tty_required=true'
need_exact_once "${DEFAULT_ENV}" 'export AQUA_COMPOSITOR_AUTOSTART=false'
need_exact_once "${DEFAULT_ENV}" 'export AQUA_BOOT_GRAPHICS=false'

grep -Fq 'tty1::respawn:/sbin/getty -L tty1 0 vt100' "${INITTAB}"
grep -Fq 'ttyS0::respawn:/sbin/getty -L ttyS0 115200 vt100' "${INITTAB}"

if grep -Fq 'aqua.boot_graphics=1' "${GRUB_CONFIG}"; then
    echo "Default GRUB entry must not enable graphical boot" >&2
    exit 1
fi
if grep -Fq 'aqua.boot_graphics=1' "${QEMU_RUNNER}"; then
    echo "Default QEMU runner must not enable graphical boot" >&2
    exit 1
fi

"${ROOT_DIR}/scripts/check-graphical-session-boot.sh"

echo "Aqua Linux default recovery safety checks passed."
