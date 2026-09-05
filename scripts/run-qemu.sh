#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/build/buildroot-output"
IMAGE_DIR="${OUTPUT_DIR}/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-serial.log}"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
QEMU_ACCELERATOR="${QEMU_ACCELERATOR:-}"
QEMU_CPU_MODEL="${QEMU_CPU_MODEL:-}"
QEMU_HOST_CURSOR="${QEMU_HOST_CURSOR:-true}"
KERNEL_APPEND="${AQUA_KERNEL_APPEND:-}"
KERNEL_COMMAND_LINE="root=/dev/vda rw console=tty1 console=ttyS0,115200n8 panic=-1 aqua.desktop_icons=1"
if [ -n "${KERNEL_APPEND}" ]; then
    KERNEL_COMMAND_LINE="${KERNEL_COMMAND_LINE} ${KERNEL_APPEND}"
fi

case "${QEMU_HOST_CURSOR}" in
    true)
        KERNEL_COMMAND_LINE="${KERNEL_COMMAND_LINE} aqua.host_cursor=1"
        QEMU_SHOW_CURSOR=on
        ;;
    false)
        QEMU_SHOW_CURSOR=off
        ;;
    *)
        echo "QEMU_HOST_CURSOR must be true or false." >&2
        exit 1
        ;;
esac

if [ ! -f "${KERNEL}" ]; then
    echo "Missing kernel: ${KERNEL}" >&2
    echo "Run scripts/build-image.sh first." >&2
    exit 1
fi

if [ ! -f "${ROOTFS}" ]; then
    echo "Missing root filesystem: ${ROOTFS}" >&2
    echo "Run scripts/build-image.sh first." >&2
    exit 1
fi

if [ -z "${QEMU_ACCELERATOR}" ]; then
    if [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
        QEMU_ACCELERATOR="kvm"
    else
        QEMU_ACCELERATOR="tcg"
    fi
fi

if [ -z "${QEMU_CPU_MODEL}" ]; then
    if [ "${QEMU_ACCELERATOR}" = "kvm" ]; then
        QEMU_CPU_MODEL="host"
    else
        QEMU_CPU_MODEL="max"
    fi
fi

mkdir -p "$(dirname "${SERIAL_LOG}")"
rm -f "${SERIAL_LOG}"

echo "Starting Aqua Linux with QEMU accelerator=${QEMU_ACCELERATOR} cpu=${QEMU_CPU_MODEL} host_cursor=${QEMU_HOST_CURSOR}"

exec qemu-system-x86_64 \
    -machine "accel=${QEMU_ACCELERATOR}" \
    -cpu "${QEMU_CPU_MODEL}" \
    -smp "${CPUS}" \
    -m "${MEMORY}" \
    -kernel "${KERNEL}" \
    -drive file="${ROOTFS}",if=virtio,format=raw \
    -append "${KERNEL_COMMAND_LINE}" \
    -serial "file:${SERIAL_LOG}" \
    -display "sdl,gl=off,show-cursor=${QEMU_SHOW_CURSOR},window-close=on" \
    -vga none \
    -device virtio-vga,xres=1280,yres=800 \
    -device virtio-keyboard-pci \
    -device virtio-tablet-pci \
    -netdev user,id=net0 \
    -device virtio-net-pci,netdev=net0 \
    -device virtio-rng-pci \
    -no-reboot
