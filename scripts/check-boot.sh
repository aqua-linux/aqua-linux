#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/build/buildroot-output"
IMAGE_DIR="${OUTPUT_DIR}/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-serial-check.log}"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"

need_file() {
    if [ ! -f "$1" ]; then
        echo "Missing $2: $1" >&2
        echo "Run scripts/build-image.sh or scripts/build-image-docker-volume.sh first." >&2
        exit 1
    fi
}

need_marker() {
    if ! grep -Fq "$1" "${SERIAL_LOG}"; then
        echo "Missing boot marker: $1" >&2
        echo "Serial log: ${SERIAL_LOG}" >&2
        echo "Last serial lines:" >&2
        tail -n 80 "${SERIAL_LOG}" >&2 || true
        exit 1
    fi
}

need_file "${KERNEL}" "kernel"
need_file "${ROOTFS}" "root filesystem"

mkdir -p "$(dirname "${SERIAL_LOG}")"
: > "${SERIAL_LOG}"

qemu-system-x86_64 \
    -machine accel=tcg \
    -cpu max \
    -smp "${CPUS}" \
    -m "${MEMORY}" \
    -kernel "${KERNEL}" \
    -drive file="${ROOTFS}",if=virtio,format=raw \
    -append "root=/dev/vda rw console=tty1 console=ttyS0,115200n8 panic=-1" \
    -serial "file:${SERIAL_LOG}" \
    -display none \
    -vga none \
    -device virtio-vga \
    -netdev user,id=net0 \
    -device virtio-net-pci,netdev=net0 \
    -device virtio-rng-pci \
    -no-reboot &

QEMU_PID="$!"
cleanup() {
    if kill -0 "${QEMU_PID}" 2>/dev/null; then
        kill "${QEMU_PID}" 2>/dev/null || true
        wait "${QEMU_PID}" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

i=0
while [ "${i}" -lt "${TIMEOUT_SECONDS}" ]; do
    if grep -Fq '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh' "${SERIAL_LOG}"; then
        break
    fi
    if ! kill -0 "${QEMU_PID}" 2>/dev/null; then
        echo "QEMU exited before recovery-ready marker." >&2
        echo "Serial log: ${SERIAL_LOG}" >&2
        echo "Last serial lines:" >&2
        tail -n 80 "${SERIAL_LOG}" >&2 || true
        exit 1
    fi
    i=$((i + 1))
    sleep 1
done

if [ "${i}" -ge "${TIMEOUT_SECONDS}" ]; then
    echo "Timed out waiting for Aqua Linux recovery marker after ${TIMEOUT_SECONDS}s." >&2
    echo "Serial log: ${SERIAL_LOG}" >&2
    echo "Last serial lines:" >&2
    tail -n 80 "${SERIAL_LOG}" >&2 || true
    exit 1
fi

need_marker '[AQUA-BOOT] stage=rcS-start product="Aqua Linux"'
need_marker '[AQUA-BOOT] stage=filesystems-mounted status=ok'
need_marker '[AQUA-BOOT] stage=udev-ready status=ok seat=seat0'
need_marker '[AQUA-BOOT] stage=fbdev-device status=ok device=/dev/fb0 mode='
need_marker '[AQUA-BOOT] stage=os-release id=aqua pretty="Aqua Linux Milestone 1"'
need_marker '[AQUA-BOOT] stage=session-config status=ok autostart=false boot_graphics=false recovery_tty=true'
need_marker '[AQUA-BOOT] stage=session-runtime status=ok user=aqua uid=1000 runtime_dir=/run/user/1000 control_dir=/run/aqua mode=0700'
need_marker '[AQUA-BOOT] stage=session-env status=ok wayland=aqua-wayland-0 xdg=/run/user/1000 assets=/usr/share/aqua'
need_marker '[AQUA-BOOT] stage=runtime-assets-ready milestone=2 status=ok'
need_marker '[AQUA-BOOT] stage=compositor-binary status=packaged autostart=false boot_graphics=false'
need_marker '[AQUA-BOOT] stage=compositor-status status=ok mode=nested-dev'
need_marker '[AQUA-BOOT] stage=session-bootstrap status=ok runtime_dir=/run/user/1000 autostart=false boot_graphics=false session_started=false'
need_marker '[AQUA-BOOT] stage=compositor-assets status=ok root=/usr/share/aqua'
need_marker '[AQUA-BOOT] stage=output-plan status=ok backend=nested-dev-window boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=visible-preview-plan status=ok preview_window_started=false boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=scene-contract status=ok surfaces=7 boot_graphics=false'
need_marker '[AQUA-BOOT] stage=render-plan status=ok commands=7 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=paint-plan status=ok steps=7 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=frame-plan status=ok format=rgba8888 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=frame-buffer status=ok bytes=6291456 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=raster status=ok rects=7 surface_layers=15 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=surface-primitives status=ok layers=15 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=raster-export status=ok bytes=4718609 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=raster-png-export status=ok bytes=6293028 boot_graphics=false renderer_started=false'
need_marker '[AQUA-BOOT] stage=session-check status=ok no_graphics=true'
need_marker '[AQUA-BOOT] stage=graphical-session-activation status=disabled boot_graphics=false session_started=false'
need_marker '[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh'

echo "Aqua Linux boot markers verified."
echo "Serial log: ${SERIAL_LOG}"
