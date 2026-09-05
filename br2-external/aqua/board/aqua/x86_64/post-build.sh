#!/usr/bin/env sh
set -eu

TARGET_DIR="${1:-${TARGET_DIR:-}}"
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
AQUA_EXTERNAL_DIR="$(CDPATH= cd -- "${SCRIPT_DIR}/../../.." && pwd)"
REPO_DIR="$(CDPATH= cd -- "${AQUA_EXTERNAL_DIR}/../.." && pwd)"

if [ -z "${TARGET_DIR}" ]; then
    echo "TARGET_DIR is required" >&2
    exit 1
fi

mkdir -p \
    "${TARGET_DIR}/home/aqua/.config/aqua" \
    "${TARGET_DIR}/home/aqua/Documents/Projects" \
    "${TARGET_DIR}/home/aqua/Downloads" \
    "${TARGET_DIR}/home/aqua/Pictures"
cat > "${TARGET_DIR}/home/aqua/Welcome.txt" <<'EOF'
Aqua Linux
Buildroot base
Custom Wayland compositor
Read-only preview
Safe recovery remains available
Pointer and keyboard navigation
Viewport scrolling
Small text preview
No file execution
Aqua Files
EOF
cat > "${TARGET_DIR}/home/aqua/Notes.txt" <<'EOF'
Aqua Linux read-only notes
EOF
cat > "${TARGET_DIR}/home/aqua/.config/aqua/settings.conf" <<'EOF'
version=1
reduced_motion=false
desktop_icons=true
key_repeat=true
theme=Light
EOF
chmod 600 "${TARGET_DIR}/home/aqua/.config/aqua/settings.conf"
if grep -q '^aqua:[^:]*:1000:1000:' "${TARGET_DIR}/etc/passwd" 2>/dev/null; then
    chown -R 1000:1000 "${TARGET_DIR}/home/aqua"
fi

chmod +x "${TARGET_DIR}/etc/init.d/rcS"
chmod +x "${TARGET_DIR}/usr/bin/aqua-recovery"
[ -f "${TARGET_DIR}/usr/bin/aqua-session-check" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-session-check"
[ -f "${TARGET_DIR}/usr/bin/aqua-session-runtime-prepare" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-session-runtime-prepare"
[ -f "${TARGET_DIR}/usr/bin/aqua-session-user-launch" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-session-user-launch"
[ -f "${TARGET_DIR}/usr/bin/aqua-media-service-supervisor" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-media-service-supervisor"
[ -f "${TARGET_DIR}/usr/bin/aqua-media-service-stop" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-media-service-stop"
[ -f "${TARGET_DIR}/usr/bin/aqua-network-service-supervisor" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-network-service-supervisor"
[ -f "${TARGET_DIR}/usr/bin/aqua-network-service-boot" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-network-service-boot"
[ -f "${TARGET_DIR}/usr/bin/aqua-network-service-stop" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-network-service-stop"
[ -f "${TARGET_DIR}/usr/bin/aqua-wifi-service-supervisor" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-wifi-service-supervisor"
[ -f "${TARGET_DIR}/usr/bin/aqua-wifi-service-boot" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-wifi-service-boot"
[ -f "${TARGET_DIR}/usr/bin/aqua-wifi-service-stop" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-wifi-service-stop"
[ -f "${TARGET_DIR}/usr/bin/aqua-wifi-udhcpc-client" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-wifi-udhcpc-client"
[ -f "${TARGET_DIR}/etc/aqua/wpa_supplicant-aqua.conf" ] && chmod 600 "${TARGET_DIR}/etc/aqua/wpa_supplicant-aqua.conf"
[ -f "${TARGET_DIR}/usr/bin/aqua-udhcpc-hook" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-udhcpc-hook"
[ -f "${TARGET_DIR}/usr/bin/aqua-compositor-manual-launch" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-compositor-manual-launch"
[ -f "${TARGET_DIR}/usr/bin/aqua-compositor-guarded-run" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-compositor-guarded-run"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphical-session-supervisor" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphical-session-supervisor"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphical-session-boot" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphical-session-boot"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphical-session-stop" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphical-session-stop"
[ -f "${TARGET_DIR}/usr/bin/aqua-compositor-handoff-gate" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-compositor-handoff-gate"
[ -f "${TARGET_DIR}/usr/bin/aqua-compositor-preview-exec" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-compositor-preview-exec"
[ -f "${TARGET_DIR}/usr/bin/aqua-visible-preview-request" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-visible-preview-request"
[ -f "${TARGET_DIR}/usr/bin/aqua-visible-preview-launch" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-visible-preview-launch"
[ -f "${TARGET_DIR}/usr/bin/aqua-recovery-help" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-recovery-help"
[ -f "${TARGET_DIR}/usr/bin/aqua-operator-transcript" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-operator-transcript"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-enable-gate" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-enable-gate"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-launch-candidate" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-launch-candidate"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-rollback-drill" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-rollback-drill"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-startup-preflight" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-startup-preflight"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-startup-rehearsal" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-startup-rehearsal"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-qemu-display-gate" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-qemu-display-gate"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-visible-qemu-attempt" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-visible-qemu-attempt"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-visible-attempt-transcript" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-visible-attempt-transcript"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-visible-attempt-result" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-visible-attempt-result"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-visible-attempt-runner" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-visible-attempt-runner"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-qemu-visible-boot-check" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-qemu-visible-boot-check"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-fbdev-present" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-fbdev-present"
[ -f "${TARGET_DIR}/usr/bin/aqua-graphics-qemu-observation-marker" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-graphics-qemu-observation-marker"
[ -f "${TARGET_DIR}/usr/bin/aqua-qemu-visible-pass-report" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-qemu-visible-pass-report"
[ -f "${TARGET_DIR}/usr/bin/aqua-qemu-visible-evidence-bundle-apply" ] && chmod +x "${TARGET_DIR}/usr/bin/aqua-qemu-visible-evidence-bundle-apply"

AQUA_COMPOSITOR_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-compositor"
AQUA_FILES_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-files"
AQUA_SETTINGS_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-settings"
AQUA_PROPERTIES_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-properties"
AQUA_TERMINAL_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-terminal"
AQUA_INSTALLER_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-installer"
AQUA_TYPOGRAPHY_ACCEPTANCE_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-typography-acceptance"
AQUA_COMPONENT_ACCEPTANCE_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-component-acceptance"
AQUA_INSTALLER_PROBE_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-installer-probe"
AQUA_AUDIO_ADAPTER_PROBE_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-audio-adapter-probe"
AQUA_NETWORK_BROKER_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-network-broker"
AQUA_WIFI_NATIVE_PROBE_BINARY="${REPO_DIR}/target/x86_64-unknown-linux-musl/release/aqua-wifi-native-probe"
mkdir -p "${TARGET_DIR}/usr/bin" "${TARGET_DIR}/usr/libexec/aqua-tests" "${TARGET_DIR}/usr/share/doc/aqua"
if [ -f "${AQUA_COMPOSITOR_BINARY}" ]; then
    cp "${AQUA_COMPOSITOR_BINARY}" "${TARGET_DIR}/usr/bin/aqua-compositor"
    chmod +x "${TARGET_DIR}/usr/bin/aqua-compositor"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/compositor-binary.txt" <<'EOF'
aqua-compositor packaged=true
path=/usr/bin/aqua-compositor
autostart=false
boot_graphics=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/compositor-binary.txt" <<'EOF'
aqua-compositor packaged=false
path=/usr/bin/aqua-compositor
autostart=false
boot_graphics=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_AUDIO_ADAPTER_PROBE_BINARY}" ] &&
   [ -f "${TARGET_DIR}/etc/aqua/audio-stack.conf" ]; then
    cp "${AQUA_AUDIO_ADAPTER_PROBE_BINARY}" \
        "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-audio-adapter-probe"
    chmod +x "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-audio-adapter-probe"
fi

if [ -f "${AQUA_NETWORK_BROKER_BINARY}" ]; then
    cp "${AQUA_NETWORK_BROKER_BINARY}" "${TARGET_DIR}/usr/bin/aqua-network-broker"
    chmod 755 "${TARGET_DIR}/usr/bin/aqua-network-broker"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/network-broker-binary.txt" <<'EOF'
aqua-network-broker packaged=true
path=/usr/bin/aqua-network-broker
owner_uid=0
client_uid=1000
protocol=AQUA-NETWORK/1
operations=status,renew-dhcp
autostart=false
default_network=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/network-broker-binary.txt" <<'EOF'
aqua-network-broker packaged=false
path=/usr/bin/aqua-network-broker
owner_uid=0
client_uid=1000
protocol=AQUA-NETWORK/1
operations=status,renew-dhcp
autostart=false
default_network=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_WIFI_NATIVE_PROBE_BINARY}" ] &&
   [ -f "${TARGET_DIR}/etc/aqua/wifi-services.conf" ]; then
    cp "${AQUA_WIFI_NATIVE_PROBE_BINARY}" \
        "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-wifi-native-probe"
    chmod 755 "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-wifi-native-probe"
fi

if [ -f "${AQUA_INSTALLER_BINARY}" ]; then
    cp "${AQUA_INSTALLER_BINARY}" "${TARGET_DIR}/usr/bin/aqua-installer"
    chmod +x "${TARGET_DIR}/usr/bin/aqua-installer"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/installer-binary.txt" <<'EOF'
aqua-installer packaged=true
path=/usr/bin/aqua-installer
app_id=aqua.installer
surface=wl_shm-xdg-toplevel
initial_step=welcome
live_input=keyboard-navigation
input_scope=welcome-language-keyboard-partitions-timezone-user
autostart=false
execution_allowed=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/installer-binary.txt" <<'EOF'
aqua-installer packaged=false
path=/usr/bin/aqua-installer
app_id=aqua.installer
surface=wl_shm-xdg-toplevel
initial_step=welcome
live_input=false
autostart=false
execution_allowed=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_TYPOGRAPHY_ACCEPTANCE_BINARY}" ]; then
    cp "${AQUA_TYPOGRAPHY_ACCEPTANCE_BINARY}" \
        "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-typography-acceptance"
    chmod +x "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-typography-acceptance"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/typography-acceptance-binary.txt" <<'EOF'
aqua-typography-acceptance packaged=true
path=/usr/libexec/aqua-tests/aqua-typography-acceptance
app_id=aqua.typography-acceptance
surface=wl_shm-xdg-toplevel
locales=tr-TR,ar
autostart=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/typography-acceptance-binary.txt" <<'EOF'
aqua-typography-acceptance packaged=false
path=/usr/libexec/aqua-tests/aqua-typography-acceptance
app_id=aqua.typography-acceptance
surface=wl_shm-xdg-toplevel
locales=tr-TR,ar
autostart=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_COMPONENT_ACCEPTANCE_BINARY}" ]; then
    cp "${AQUA_COMPONENT_ACCEPTANCE_BINARY}" \
        "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-component-acceptance"
    chmod +x "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-component-acceptance"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/component-acceptance-binary.txt" <<'EOF'
aqua-component-acceptance packaged=true
path=/usr/libexec/aqua-tests/aqua-component-acceptance
app_id=aqua.component-acceptance
surface=wl_shm-xdg-toplevel
fixture_revision=aqua-component-fixtures-20
catalog=22
shared=22
autostart=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/component-acceptance-binary.txt" <<'EOF'
aqua-component-acceptance packaged=false
path=/usr/libexec/aqua-tests/aqua-component-acceptance
app_id=aqua.component-acceptance
surface=wl_shm-xdg-toplevel
fixture_revision=aqua-component-fixtures-20
catalog=22
shared=22
autostart=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

# Keep installer readiness inspectable from recovery without enabling writes.
if [ -f "${AQUA_INSTALLER_PROBE_BINARY}" ]; then
    cp "${AQUA_INSTALLER_PROBE_BINARY}" "${TARGET_DIR}/usr/bin/aqua-installer-probe"
    chmod +x "${TARGET_DIR}/usr/bin/aqua-installer-probe"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/installer-probe-binary.txt" <<'EOF'
aqua-installer-probe packaged=true
path=/usr/bin/aqua-installer-probe
mode=readiness
autostart=false
execution_allowed=false
disk_commands_executed=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/installer-probe-binary.txt" <<'EOF'
aqua-installer-probe packaged=false
path=/usr/bin/aqua-installer-probe
mode=readiness
autostart=false
execution_allowed=false
disk_commands_executed=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_TERMINAL_BINARY}" ]; then
    cp "${AQUA_TERMINAL_BINARY}" "${TARGET_DIR}/usr/bin/aqua-terminal"
    chmod +x "${TARGET_DIR}/usr/bin/aqua-terminal"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/terminal-binary.txt" <<'EOF'
aqua-terminal packaged=true
path=/usr/bin/aqua-terminal
app_id=aqua.terminal
pty=true
emulator=vt100
autostart=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/terminal-binary.txt" <<'EOF'
aqua-terminal packaged=false
path=/usr/bin/aqua-terminal
app_id=aqua.terminal
pty=true
emulator=vt100
autostart=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_PROPERTIES_BINARY}" ]; then
    cp "${AQUA_PROPERTIES_BINARY}" "${TARGET_DIR}/usr/bin/aqua-properties"
    chmod +x "${TARGET_DIR}/usr/bin/aqua-properties"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/properties-binary.txt" <<'EOF'
aqua-properties packaged=true
path=/usr/bin/aqua-properties
app_id=aqua.properties
autostart=false
targets=files,settings,trash
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/properties-binary.txt" <<'EOF'
aqua-properties packaged=false
path=/usr/bin/aqua-properties
app_id=aqua.properties
autostart=false
targets=files,settings,trash
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_SETTINGS_BINARY}" ]; then
    cp "${AQUA_SETTINGS_BINARY}" "${TARGET_DIR}/usr/bin/aqua-settings"
    chmod +x "${TARGET_DIR}/usr/bin/aqua-settings"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/settings-binary.txt" <<'EOF'
aqua-settings packaged=true
path=/usr/bin/aqua-settings
app_id=aqua.settings
autostart=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/settings-binary.txt" <<'EOF'
aqua-settings packaged=false
path=/usr/bin/aqua-settings
app_id=aqua.settings
autostart=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

if [ -f "${AQUA_FILES_BINARY}" ]; then
    cp "${AQUA_FILES_BINARY}" "${TARGET_DIR}/usr/bin/aqua-files"
    chmod +x "${TARGET_DIR}/usr/bin/aqua-files"
    cat > "${TARGET_DIR}/usr/share/doc/aqua/files-binary.txt" <<'EOF'
aqua-files packaged=true
path=/usr/bin/aqua-files
app_id=aqua.files
autostart=false
EOF
else
    cat > "${TARGET_DIR}/usr/share/doc/aqua/files-binary.txt" <<'EOF'
aqua-files packaged=false
path=/usr/bin/aqua-files
app_id=aqua.files
autostart=false
build_hint=scripts/build-compositor-linux-docker.sh
EOF
fi

# Keep the upstream C reference clients as compatibility fixtures without
# shipping Weston compositor, shells, backends, launchers, or desktop runtime.
if [ -x "${TARGET_DIR}/usr/bin/weston-simple-shm" ] || \
    [ -x "${TARGET_DIR}/usr/libexec/aqua-tests/weston-simple-shm" ]; then
    mkdir -p "${TARGET_DIR}/usr/libexec/aqua-tests"
    for fixture in \
        weston-simple-shm \
        weston-simple-damage \
        weston-simple-touch \
        weston-terminal; do
        if [ -x "${TARGET_DIR}/usr/bin/${fixture}" ]; then
            mv "${TARGET_DIR}/usr/bin/${fixture}" \
                "${TARGET_DIR}/usr/libexec/aqua-tests/${fixture}"
        fi
        # Buildroot finalization is re-entrant: a prior pass has already moved
        # installed fixtures out of /usr/bin. Recover a newly added fixture
        # from the one active Weston build directory without pinning its path.
        if [ ! -x "${TARGET_DIR}/usr/libexec/aqua-tests/${fixture}" ]; then
            fixture_build_path=""
            for candidate in \
                "$(dirname "${TARGET_DIR}")"/build/weston-*/buildroot-build/clients/"${fixture}"; do
                [ -x "${candidate}" ] || continue
                if [ -n "${fixture_build_path}" ]; then
                    echo "multiple upstream Wayland fixture builds found: ${fixture}" >&2
                    exit 1
                fi
                fixture_build_path="${candidate}"
            done
            if [ -n "${fixture_build_path}" ]; then
                cp "${fixture_build_path}" \
                    "${TARGET_DIR}/usr/libexec/aqua-tests/${fixture}"
                chmod +x "${TARGET_DIR}/usr/libexec/aqua-tests/${fixture}"
            fi
        fi
        if [ ! -x "${TARGET_DIR}/usr/libexec/aqua-tests/${fixture}" ]; then
            echo "missing required upstream Wayland fixture: ${fixture}" >&2
            exit 1
        fi
    done
    frame_asset_stage="${TARGET_DIR}/usr/libexec/aqua-tests/weston-frame-assets"
    rm -rf "${frame_asset_stage}"
    mkdir -p "${frame_asset_stage}"
    for asset in icon_window.png sign_close.png sign_maximize.png sign_minimize.png; do
        if [ -f "${TARGET_DIR}/usr/share/weston/${asset}" ]; then
            cp "${TARGET_DIR}/usr/share/weston/${asset}" \
                "${frame_asset_stage}/${asset}"
            continue
        fi
        asset_build_path=""
        for candidate in \
            "$(dirname "${TARGET_DIR}")"/build/weston-*/data/"${asset}"; do
            [ -f "${candidate}" ] || continue
            if [ -n "${asset_build_path}" ]; then
                echo "multiple upstream Weston frame assets found: ${asset}" >&2
                exit 1
            fi
            asset_build_path="${candidate}"
        done
        if [ -z "${asset_build_path}" ]; then
            echo "missing required upstream Weston frame asset: ${asset}" >&2
            exit 1
        fi
        cp "${asset_build_path}" "${frame_asset_stage}/${asset}"
    done
    rm -f "${TARGET_DIR}/usr/bin/weston" "${TARGET_DIR}/usr/bin/weston-"*
    rm -rf \
        "${TARGET_DIR}/usr/lib/libweston-"* \
        "${TARGET_DIR}/usr/lib/weston" \
        "${TARGET_DIR}/usr/libexec/weston-"* \
        "${TARGET_DIR}/usr/share/libweston-"* \
        "${TARGET_DIR}/usr/share/wayland-sessions/weston.desktop" \
        "${TARGET_DIR}/usr/share/weston"
    mkdir -p "${TARGET_DIR}/usr/share/weston"
    mv "${frame_asset_stage}"/* "${TARGET_DIR}/usr/share/weston/"
    rmdir "${frame_asset_stage}"
    if [ ! -x "${TARGET_DIR}/usr/libexec/aqua-tests/aqua-glfw-wayland-probe" ] || \
        [ ! -e "${TARGET_DIR}/usr/lib/libglfw.so.3" ]; then
        echo "missing required GLFW Wayland compatibility fixture" >&2
        exit 1
    fi
    cat > "${TARGET_DIR}/usr/share/doc/aqua/wayland-compat-client.txt" <<'EOF'
source=upstream-weston-14.0.1-simple-clients
role=third-party-wayland-compatibility-fixtures
fixtures=weston-simple-shm,weston-simple-damage,weston-simple-touch,weston-terminal
matrix_fixtures=weston-simple-shm,weston-simple-damage,weston-simple-touch,weston-terminal,aqua-glfw-wayland-probe
fixture_count=5
simple_shm_path=/usr/libexec/aqua-tests/weston-simple-shm
simple_damage_path=/usr/libexec/aqua-tests/weston-simple-damage
simple_touch_path=/usr/libexec/aqua-tests/weston-simple-touch
weston_terminal_path=/usr/libexec/aqua-tests/weston-terminal
independent_toolkit=glfw-3.4-wayland
glfw_probe_path=/usr/libexec/aqua-tests/aqua-glfw-wayland-probe
glfw_client_api=none
glfw_render_path=wl_shm-argb8888
frame_assets=icon_window.png,sign_close.png,sign_maximize.png,sign_minimize.png
protocol_scope=xdg-shell+wl_shm+wl_surface.damage_buffer+wl_touch+wl_keyboard+pty+glfw-window-lifecycle
weston_compositor_packaged=false
autostart=false
EOF
fi

# Aqua v1 intentionally exposes a native Wayland-only application session.
# Fail closed if a future package change introduces an X11 display server.
for forbidden in \
    usr/bin/Xwayland \
    usr/bin/Xorg \
    usr/libexec/Xwayland; do
    if [ -e "${TARGET_DIR}/${forbidden}" ]; then
        echo "unsupported X11 display server leaked into rootfs: /${forbidden}" >&2
        exit 1
    fi
done
if [ -d "${TARGET_DIR}/usr/lib/xorg" ]; then
    echo "unsupported Xorg server modules leaked into rootfs: /usr/lib/xorg" >&2
    exit 1
fi
cat > "${TARGET_DIR}/usr/share/doc/aqua/application-compatibility.txt" <<'EOF'
application_model=native-wayland
supported_clients=first-party-and-independently-tested-wl_shm-argb8888
xwayland_packaged=false
x11_server_packaged=false
x11_applications_supported=false
display_environment_exported=false
xkb_data_scope=wayland-keyboard-layouts
independently_tested_toolkits=weston-client-toolkit,glfw-3.4-wayland
broader_toolkit_coverage=bounded-not-general
EOF

mkdir -p "${TARGET_DIR}/etc/aqua"
cat > "${TARGET_DIR}/etc/aqua/milestone" <<'EOF'
product=Aqua Linux
milestone=1
base=Buildroot
dev_target=QEMU x86_64
graphics_target=custom Wayland compositor
runtime=recovery-shell
EOF

cat > "${TARGET_DIR}/etc/aqua/release" <<'EOF'
PRODUCT="Aqua Linux"
MILESTONE="1"
BASE="Buildroot"
FIRST_DEV_TARGET="QEMU x86_64"
GRAPHICS_TARGET="custom Wayland compositor"
RUNTIME="recovery-shell"
EOF

cat > "${TARGET_DIR}/etc/aqua/compositor-session.conf" <<'EOF'
product=Aqua Linux
mode=nested-dev
wayland_socket=aqua-wayland-0
runtime_dir=/run/user/1000
runtime_asset_root=/usr/share/aqua
autostart=false
boot_graphics=false
recovery_tty_required=true
supervisor=bounded-restart
supervisor_max_restarts=3
supervisor_restart_delay_seconds=2
supervisor_stable_seconds=30
EOF

cat > "${TARGET_DIR}/etc/aqua/session.env" <<'EOF'
export WAYLAND_DISPLAY=aqua-wayland-0
export XDG_RUNTIME_DIR=/run/user/1000
export AQUA_ASSET_ROOT=/usr/share/aqua
export XCOMPOSEFILE=/usr/share/aqua/compose/Compose
export AQUA_SESSION_MODE=nested-dev
export AQUA_COMPOSITOR_AUTOSTART=false
export AQUA_BOOT_GRAPHICS=false
EOF

cat > "${TARGET_DIR}/etc/aqua/compositor-session-graphics.conf" <<'EOF'
product=Aqua Linux
mode=drm-wayland
wayland_socket=aqua-wayland-drm-0
runtime_dir=/run/user/1000
runtime_asset_root=/usr/share/aqua
autostart=true
boot_graphics=true
recovery_tty_required=true
supervisor=bounded-restart
supervisor_max_restarts=3
supervisor_restart_delay_seconds=2
supervisor_stable_seconds=30
EOF

cat > "${TARGET_DIR}/etc/aqua/session-graphics.env" <<'EOF'
export WAYLAND_DISPLAY=aqua-wayland-drm-0
export XDG_RUNTIME_DIR=/run/user/1000
export AQUA_ASSET_ROOT=/usr/share/aqua
export XCOMPOSEFILE=/usr/share/aqua/compose/Compose
export AQUA_SESSION_MODE=drm-wayland
export AQUA_COMPOSITOR_AUTOSTART=true
export AQUA_BOOT_GRAPHICS=true
export AQUA_DRM_WAYLAND_SESSION_OPERATOR_CONFIRMED=true
export AQUA_DRM_WAYLAND_INPUT_REQUIRED=true
export AQUA_DRM_WAYLAND_EXTERNAL_CLIENT_REQUIRED=false
export AQUA_DRM_WAYLAND_SCENARIO=desktop-event-loop
export AQUA_DRM_WAYLAND_SESSION_HOLD_SECONDS=30
export AQUA_DRM_WAYLAND_SESSION_PERSISTENT=true
EOF

mkdir -p "${TARGET_DIR}/usr/lib"
cat > "${TARGET_DIR}/usr/lib/os-release" <<'EOF'
NAME="Aqua Linux"
ID=aqua
ID_LIKE=buildroot
VERSION="Milestone 1"
VERSION_ID="m1"
PRETTY_NAME="Aqua Linux Milestone 1"
EOF

ln -sf ../usr/lib/os-release "${TARGET_DIR}/etc/os-release"

ASSET_SOURCE_DIR="${REPO_DIR}/docs/aqua-linux/assets"
TOKEN_SOURCE="${REPO_DIR}/docs/aqua-linux/design-tokens.json"

if [ -d "${ASSET_SOURCE_DIR}" ] && [ -f "${TOKEN_SOURCE}" ]; then
    mkdir -p \
        "${TARGET_DIR}/usr/share/aqua/wallpapers" \
        "${TARGET_DIR}/usr/share/aqua/brand" \
        "${TARGET_DIR}/usr/share/aqua/icons/aqua" \
        "${TARGET_DIR}/usr/share/aqua/fonts" \
        "${TARGET_DIR}/usr/share/aqua/tokens" \
        "${TARGET_DIR}/usr/share/doc/aqua"

    cp "${ASSET_SOURCE_DIR}/default-wallpaper.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/default-wallpaper.png"
    cp "${ASSET_SOURCE_DIR}/wallpaper-light.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/wallpaper-light.png"
    cp "${ASSET_SOURCE_DIR}/wallpaper-dark.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/wallpaper-dark.png"
    cp "${ASSET_SOURCE_DIR}/wallpaper-pale-waves.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/wallpaper-pale-waves.png"
    cp "${ASSET_SOURCE_DIR}/wallpaper-surf.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/wallpaper-surf.png"
    cp "${ASSET_SOURCE_DIR}/wallpaper-reef.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/wallpaper-reef.png"
    cp "${ASSET_SOURCE_DIR}/wallpaper-sunlit-water.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/wallpaper-sunlit-water.png"
    cp "${ASSET_SOURCE_DIR}/wallpaper-moonlit-lagoon.png" "${TARGET_DIR}/usr/share/aqua/wallpapers/wallpaper-moonlit-lagoon.png"
    cp "${ASSET_SOURCE_DIR}/aqua-symbol-primary.png" "${TARGET_DIR}/usr/share/aqua/brand/aqua-symbol-primary.png"
    cp "${ASSET_SOURCE_DIR}/aqua-symbol-inverse.png" "${TARGET_DIR}/usr/share/aqua/brand/aqua-symbol-inverse.png"
    cp "${ASSET_SOURCE_DIR}/aqua-symbol-accent.png" "${TARGET_DIR}/usr/share/aqua/brand/aqua-symbol-accent.png"
    cp "${ASSET_SOURCE_DIR}/aqua-wordmark-primary.png" "${TARGET_DIR}/usr/share/aqua/brand/aqua-wordmark-primary.png"
    cp "${ASSET_SOURCE_DIR}/aqua-logo-primary.png" "${TARGET_DIR}/usr/share/aqua/brand/aqua-logo-primary.png"
    cp "${TOKEN_SOURCE}" "${TARGET_DIR}/usr/share/aqua/tokens/design-tokens.json"
    cp "${ASSET_SOURCE_DIR}/manifest.md" "${TARGET_DIR}/usr/share/doc/aqua/asset-manifest.md"
    cp "${REPO_DIR}/docs/aqua-linux/runtime-assets.md" "${TARGET_DIR}/usr/share/doc/aqua/runtime-assets.md"
    cp "${ASSET_SOURCE_DIR}/icons/aqua/README.md" "${TARGET_DIR}/usr/share/doc/aqua/aqua-icons.md"
    cp "${REPO_DIR}/THIRD_PARTY_LICENSES.md" "${TARGET_DIR}/usr/share/doc/aqua/third-party-licenses.md"

    cp "${ASSET_SOURCE_DIR}/icons/aqua/"*.svg "${TARGET_DIR}/usr/share/aqua/icons/aqua/"
    cp "${ASSET_SOURCE_DIR}/icons/aqua/LICENSE" "${TARGET_DIR}/usr/share/aqua/icons/aqua/LICENSE"
    cp "${ASSET_SOURCE_DIR}/fonts/NotoSans-Regular.ttf" "${TARGET_DIR}/usr/share/aqua/fonts/NotoSans-Regular.ttf"
    cp "${ASSET_SOURCE_DIR}/fonts/NotoSansArabic-Regular.ttf" "${TARGET_DIR}/usr/share/aqua/fonts/NotoSansArabic-Regular.ttf"
    cp "${ASSET_SOURCE_DIR}/fonts/OFL.txt" "${TARGET_DIR}/usr/share/aqua/fonts/OFL.txt"
fi
