#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
ROOTFS_TAR="${ROOTFS_TAR:-${ROOT_DIR}/build/buildroot-output/images/rootfs.tar}"

need_tar() {
    if [ ! -f "${ROOTFS_TAR}" ]; then
        echo "Missing rootfs tar: ${ROOTFS_TAR}" >&2
        echo "Run scripts/build-image-docker-volume.sh first." >&2
        exit 1
    fi
}

need_entry() {
    entry="$1"
    if ! tar -tf "${ROOTFS_TAR}" "${entry}" >/dev/null 2>&1; then
        echo "Missing runtime asset in rootfs tar: ${entry}" >&2
        exit 1
    fi
}

need_tar

need_entry "./usr/share/aqua/wallpapers/default-wallpaper.png"
need_entry "./usr/share/aqua/wallpapers/wallpaper-pale-waves.png"
need_entry "./usr/share/aqua/wallpapers/wallpaper-surf.png"
need_entry "./usr/share/aqua/wallpapers/wallpaper-reef.png"
need_entry "./usr/share/aqua/wallpapers/wallpaper-sunlit-water.png"
need_entry "./usr/share/aqua/wallpapers/wallpaper-moonlit-lagoon.png"
need_entry "./usr/share/aqua/brand/aqua-symbol-primary.png"
need_entry "./usr/share/aqua/brand/aqua-symbol-inverse.png"
need_entry "./usr/share/aqua/brand/aqua-symbol-accent.png"
need_entry "./usr/share/aqua/brand/aqua-wordmark-primary.png"
need_entry "./usr/share/aqua/brand/aqua-logo-primary.png"
need_entry "./usr/share/aqua/tokens/design-tokens.json"
need_entry "./usr/share/aqua/icons/aqua/LICENSE"
need_entry "./usr/share/aqua/icons/aqua/home.svg"
need_entry "./usr/share/aqua/icons/aqua/files.svg"
need_entry "./usr/share/aqua/icons/aqua/aqua-drive.svg"
need_entry "./usr/share/aqua/icons/aqua/trash.svg"
need_entry "./usr/share/aqua/icons/aqua/browser.svg"
need_entry "./usr/share/aqua/icons/aqua/terminal.svg"
need_entry "./usr/share/aqua/icons/aqua/settings.svg"
need_entry "./usr/share/aqua/icons/aqua/software.svg"
need_entry "./usr/share/aqua/icons/aqua/wifi.svg"
need_entry "./usr/share/aqua/icons/aqua/volume.svg"
need_entry "./usr/share/aqua/icons/aqua/battery.svg"
need_entry "./usr/share/aqua/icons/aqua/notification.svg"
need_entry "./usr/share/aqua/icons/aqua/updates.svg"
need_entry "./usr/share/aqua/fonts/NotoSans-Regular.ttf"
need_entry "./usr/share/aqua/fonts/OFL.txt"
need_entry "./usr/share/doc/aqua/asset-manifest.md"
need_entry "./usr/share/doc/aqua/runtime-assets.md"
need_entry "./usr/share/doc/aqua/aqua-icons.md"
need_entry "./usr/share/doc/aqua/third-party-licenses.md"
need_entry "./usr/libexec/aqua-tests/weston-simple-shm"
need_entry "./usr/share/doc/aqua/wayland-compat-client.txt"
need_entry "./usr/bin/aqua-session-check"
need_entry "./etc/aqua/compositor-session.conf"
need_entry "./etc/aqua/session.env"

tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"fill"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"secondaryFill"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"border"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"separator"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"shadow"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"optionalBlurRadius"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"blurRequired"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"defaultTheme": "LightWhite"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"Softtouch"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"Deepside"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/tokens/design-tokens.json | grep -Fq '"Nightmare"'
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/fonts/OFL.txt | grep -Fq 'SIL OPEN FONT LICENSE Version 1.1'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/compositor-session.conf | grep -Fq 'wayland_socket=aqua-wayland-0'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/compositor-session.conf | grep -Fq 'runtime_dir=/run/aqua'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/compositor-session.conf | grep -Fq 'runtime_asset_root=/usr/share/aqua'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/compositor-session.conf | grep -Fq 'autostart=false'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/compositor-session.conf | grep -Fq 'boot_graphics=false'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/compositor-session.conf | grep -Fq 'recovery_tty_required=true'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session.env | grep -Fq 'export WAYLAND_DISPLAY=aqua-wayland-0'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session.env | grep -Fq 'export XDG_RUNTIME_DIR=/run/aqua'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session.env | grep -Fq 'export AQUA_ASSET_ROOT=/usr/share/aqua'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session.env | grep -Fq 'export AQUA_COMPOSITOR_AUTOSTART=false'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session.env | grep -Fq 'export AQUA_BOOT_GRAPHICS=false'
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq 'weston_compositor_packaged=false'

if tar -tf "${ROOTFS_TAR}" | grep -Eq '^\./usr/(bin/weston($|-)|lib/libweston|libexec/weston-|share/libweston|share/wayland-sessions/weston\.desktop)'; then
    echo "Weston compositor runtime leaked into the Aqua rootfs." >&2
    exit 1
fi

echo "Aqua Linux runtime asset checks passed."
