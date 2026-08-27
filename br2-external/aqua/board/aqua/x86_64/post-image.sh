#!/usr/bin/env sh
set -eu

for installer_tool in \
    /sbin/sfdisk \
    /sbin/mkfs.fat \
    /sbin/mkfs.ext4 \
    /bin/tar \
    /bin/mount \
    /bin/umount
do
    if [ ! -x "${TARGET_DIR}${installer_tool}" ]; then
        echo "Missing executable installer prerequisite: ${installer_tool}" >&2
        exit 1
    fi
done

if [ ! -f "${BINARIES_DIR}/efi-part/EFI/BOOT/bootx64.efi" ]; then
    echo "Missing GRUB2 x86_64 EFI image" >&2
    exit 1
fi

install -D -m 0644 \
    "${BR2_EXTERNAL_AQUA_PATH}/board/aqua/x86_64/grub.cfg" \
    "${BINARIES_DIR}/efi-part/EFI/BOOT/grub.cfg"
