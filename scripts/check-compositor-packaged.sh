#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
ROOTFS_TAR="${ROOTFS_TAR:-${ROOT_DIR}/build/buildroot-output/images/rootfs.tar}"

if [ ! -f "${ROOTFS_TAR}" ]; then
    echo "Missing rootfs tar: ${ROOTFS_TAR}" >&2
    echo "Run scripts/build-image-docker-volume.sh first." >&2
    exit 1
fi

tar -tf "${ROOTFS_TAR}" ./usr/bin/aqua-compositor >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/lib/libxkbcommon.so.0 >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/X11/xkb/rules/evdev >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/aqua/compose/Compose >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/aqua/compose/Compose | grep -Fq '<Multi_key> <apostrophe> <e> : "é" U00E9'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session.env | grep -Fq 'export XCOMPOSEFILE=/usr/share/aqua/compose/Compose'
tar -xOf "${ROOTFS_TAR}" ./etc/aqua/session-graphics.env | grep -Fq 'export XCOMPOSEFILE=/usr/share/aqua/compose/Compose'
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/compositor-binary.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/compositor-binary.txt | grep -Fq "aqua-compositor packaged=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/compositor-binary.txt | grep -Fq "autostart=false"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/compositor-binary.txt | grep -Fq "boot_graphics=false"
tar -tf "${ROOTFS_TAR}" ./usr/libexec/aqua-tests/weston-simple-shm >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/libexec/aqua-tests/weston-simple-damage >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/libexec/aqua-tests/weston-simple-touch >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/libexec/aqua-tests/weston-terminal >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/libexec/aqua-tests/aqua-glfw-wayland-probe >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/lib/libglfw.so.3 >/dev/null
for frame_asset in icon_window.png sign_close.png sign_maximize.png sign_minimize.png; do
    tar -tf "${ROOTFS_TAR}" "./usr/share/weston/${frame_asset}" >/dev/null
done
weston_fixture_asset_count="$(
    tar -tf "${ROOTFS_TAR}" \
        | grep -E '^\./usr/share/weston/[^/]+$' \
        | wc -l \
        | tr -d '[:space:]'
)"
if [ "${weston_fixture_asset_count}" != "4" ]; then
    echo "Unexpected Weston fixture asset count: ${weston_fixture_asset_count}" >&2
    exit 1
fi
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "source=upstream-weston-14.0.1-simple-clients"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "fixtures=weston-simple-shm,weston-simple-damage,weston-simple-touch,weston-terminal"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "matrix_fixtures=weston-simple-shm,weston-simple-damage,weston-simple-touch,weston-terminal,aqua-glfw-wayland-probe"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "fixture_count=5"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "independent_toolkit=glfw-3.4-wayland"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "glfw_probe_path=/usr/libexec/aqua-tests/aqua-glfw-wayland-probe"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "glfw_client_api=none"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "glfw_render_path=wl_shm-argb8888"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "frame_assets=icon_window.png,sign_close.png,sign_maximize.png,sign_minimize.png"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/wayland-compat-client.txt | grep -Fq "weston_compositor_packaged=false"
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/application-compatibility.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/application-compatibility.txt | grep -Fq "application_model=native-wayland"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/application-compatibility.txt | grep -Fq "x11_applications_supported=false"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/application-compatibility.txt | grep -Fq "independently_tested_toolkits=weston-client-toolkit,glfw-3.4-wayland"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/application-compatibility.txt | grep -Fq "broader_toolkit_coverage=bounded-not-general"
tar -tf "${ROOTFS_TAR}" ./usr/bin/aqua-files >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/files-binary.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/files-binary.txt | grep -Fq "aqua-files packaged=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/files-binary.txt | grep -Fq "app_id=aqua.files"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/files-binary.txt | grep -Fq "autostart=false"
tar -tf "${ROOTFS_TAR}" ./usr/bin/aqua-settings >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/settings-binary.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/settings-binary.txt | grep -Fq "aqua-settings packaged=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/settings-binary.txt | grep -Fq "app_id=aqua.settings"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/settings-binary.txt | grep -Fq "autostart=false"
tar -tf "${ROOTFS_TAR}" ./usr/bin/aqua-properties >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/properties-binary.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/properties-binary.txt | grep -Fq "aqua-properties packaged=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/properties-binary.txt | grep -Fq "app_id=aqua.properties"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/properties-binary.txt | grep -Fq "targets=files,settings,trash"
tar -tf "${ROOTFS_TAR}" ./usr/bin/aqua-terminal >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/terminal-binary.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/terminal-binary.txt | grep -Fq "aqua-terminal packaged=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/terminal-binary.txt | grep -Fq "app_id=aqua.terminal"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/terminal-binary.txt | grep -Fq "pty=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/terminal-binary.txt | grep -Fq "emulator=vt100"
tar -tf "${ROOTFS_TAR}" ./usr/bin/aqua-installer-probe >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-probe-binary.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-probe-binary.txt | grep -Fq "aqua-installer-probe packaged=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-probe-binary.txt | grep -Fq "mode=readiness"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-probe-binary.txt | grep -Fq "autostart=false"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-probe-binary.txt | grep -Fq "execution_allowed=false"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-probe-binary.txt | grep -Fq "disk_commands_executed=false"
tar -tf "${ROOTFS_TAR}" ./usr/bin/aqua-installer >/dev/null
tar -tf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt >/dev/null
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "aqua-installer packaged=true"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "app_id=aqua.installer"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "surface=wl_shm-xdg-toplevel"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "initial_step=welcome"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "live_input=keyboard-navigation"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "input_scope=welcome-language-keyboard-partitions-timezone-user"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "autostart=false"
tar -xOf "${ROOTFS_TAR}" ./usr/share/doc/aqua/installer-binary.txt | grep -Fq "execution_allowed=false"
tar -tf "${ROOTFS_TAR}" ./home/aqua/.config/aqua/settings.conf >/dev/null
test "$(tar -xOf "${ROOTFS_TAR}" ./home/aqua/.config/aqua/settings.conf)" = "version=1
reduced_motion=false
desktop_icons=true
key_repeat=true
theme=Light"
test "$(tar -tvf "${ROOTFS_TAR}" ./home/aqua/.config/aqua/settings.conf | cut -c1-10)" = "-rw-------"

echo "Aqua Linux compositor binary packaging checks passed."
