#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
ROOTFS_TAR="${ROOTFS_TAR:-${ROOT_DIR}/build/buildroot-output/images/rootfs.tar}"

for config in \
    "${ROOT_DIR}/br2-external/aqua/configs/aqua_x86_64_defconfig" \
    "${ROOT_DIR}/br2-external/aqua/configs/aqua_x86_64_audio_rehearsal_defconfig"; do
    grep -Fxq 'BR2_PACKAGE_XORG7=n' "${config}"
    grep -Fxq 'BR2_PACKAGE_XWAYLAND=n' "${config}"
done

if [ ! -f "${ROOTFS_TAR}" ]; then
    echo "Missing rootfs tar: ${ROOTFS_TAR}" >&2
    echo "Run scripts/build-image-docker-volume.sh first." >&2
    exit 1
fi

contract=./usr/share/doc/aqua/application-compatibility.txt
tar -tf "${ROOTFS_TAR}" "${contract}" >/dev/null
for marker in \
    'application_model=native-wayland' \
    'supported_clients=first-party-and-independently-tested-wl_shm-argb8888' \
    'xwayland_packaged=false' \
    'x11_server_packaged=false' \
    'x11_applications_supported=false' \
    'display_environment_exported=false' \
    'xkb_data_scope=wayland-keyboard-layouts' \
    'broader_toolkit_coverage=unproven'; do
    tar -xOf "${ROOTFS_TAR}" "${contract}" | grep -Fxq "${marker}"
done

entries="$(tar -tf "${ROOTFS_TAR}")"
if printf '%s\n' "${entries}" | grep -Eq '^\./usr/(bin/(Xwayland|Xorg)$|libexec/Xwayland$|lib/xorg(/|$))'; then
    echo "XWayland or Xorg server runtime leaked into the Aqua rootfs." >&2
    exit 1
fi
if printf '%s\n' "${entries}" | grep -Eq '^\./tmp/\.X11-unix(/|$)'; then
    echo "An X11 socket directory leaked into the Aqua rootfs." >&2
    exit 1
fi

for env_file in ./etc/aqua/session.env ./etc/aqua/session-graphics.env; do
    if tar -xOf "${ROOTFS_TAR}" "${env_file}" | grep -Eq '^[[:space:]]*(export[[:space:]]+)?DISPLAY='; then
        echo "DISPLAY leaked into ${env_file}." >&2
        exit 1
    fi
done

# XKB data is deliberately retained for native Wayland keyboard handling.
tar -tf "${ROOTFS_TAR}" ./usr/share/X11/xkb/rules/evdev >/dev/null

echo "Aqua Linux application compatibility boundary checks passed."
