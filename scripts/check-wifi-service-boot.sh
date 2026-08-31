#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BOOT_TOOL="$ROOT_DIR/br2-external/aqua/wifi-rootfs-overlay/usr/bin/aqua-wifi-service-boot"
TMP_DIR="$(mktemp -d)"
trap 'for file in "$TMP_DIR/run/wifi-service.pid" "$TMP_DIR/run/network-privilege-broker.pid"; do if [ -f "$file" ]; then kill "$(cat "$file")" 2>/dev/null || true; fi; done; rm -rf "$TMP_DIR"' EXIT HUP INT TERM

cat > "$TMP_DIR/profile.conf" <<'EOF'
enabled=true
interface=wlan0
wpa_binary=/usr/sbin/wpa_supplicant
wpa_config=/etc/aqua/wpa_supplicant-aqua.conf
udhcpc_binary=/usr/bin/aqua-wifi-udhcpc-client
udhcpc_default_script=/usr/share/udhcpc/default.script
max_restarts=3
restart_delay_seconds=2
readiness_timeout_seconds=20
monitor_interval_seconds=1
stop_timeout_seconds=5
profile_scope=qemu-hwsim-only
EOF
cat > "$TMP_DIR/supervisor" <<'EOF'
#!/bin/sh
exec python3 -c 'import os, socket, signal; p=os.environ["AQUA_TEST_WPA_SOCKET"]; os.makedirs(os.path.dirname(p), exist_ok=True); s=socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM); s.bind(p); open(os.environ["AQUA_WIFI_PID_FILE"], "w").write(str(os.getpid())); signal.pause()'
EOF
cat > "$TMP_DIR/broker" <<'EOF'
#!/bin/sh
exec python3 -c 'import os, socket; p=os.environ["AQUA_NETWORK_BROKER_SOCKET"]; s=socket.socket(socket.AF_UNIX); s.bind(p); s.listen(1); s.accept()'
EOF
chmod +x "$TMP_DIR/supervisor" "$TMP_DIR/broker"
printf '%s\n' 'console=ttyS0' > "$TMP_DIR/cmdline-default"

disabled_output="$(
    AQUA_CMDLINE_PATH="$TMP_DIR/cmdline-default" \
    AQUA_WIFI_BOOT_PROFILE="$TMP_DIR/profile.conf" \
    "$BOOT_TOOL"
)"
printf '%s\n' "$disabled_output" | grep -Fq 'status=disabled reason=kernel-flag-absent'

printf '%s\n' 'console=ttyS0 aqua.boot_wifi=1' > "$TMP_DIR/cmdline-enabled"
cp "$TMP_DIR/profile.conf" "$TMP_DIR/invalid.conf"
printf '%s\n' 'interface=wlan1' >> "$TMP_DIR/invalid.conf"
set +e
invalid_output="$(
    AQUA_CMDLINE_PATH="$TMP_DIR/cmdline-enabled" \
    AQUA_WIFI_BOOT_PROFILE="$TMP_DIR/invalid.conf" \
    "$BOOT_TOOL" 2>&1
)"
invalid_status="$?"
set -e
test "$invalid_status" -ne 0
printf '%s\n' "$invalid_output" | grep -Fq 'status=blocked reason=invalid-qemu-wifi-profile'

mkdir -p "$TMP_DIR/run/wpa" "$TMP_DIR/run/network"
enabled_output="$(
    AQUA_CMDLINE_PATH="$TMP_DIR/cmdline-enabled" \
    AQUA_WIFI_BOOT_PROFILE="$TMP_DIR/profile.conf" \
    AQUA_WIFI_SUPERVISOR_BIN="$TMP_DIR/supervisor" \
    AQUA_NETWORK_BROKER_BIN="$TMP_DIR/broker" \
    AQUA_WIFI_CONTROL_DIR="$TMP_DIR/run" \
    AQUA_NETWORK_CONTROL_DIR="$TMP_DIR/run/network" \
    AQUA_WIFI_WPA_CONTROL_SOCKET="$TMP_DIR/run/wpa/wlan0" \
    AQUA_TEST_WPA_SOCKET="$TMP_DIR/run/wpa/wlan0" \
    AQUA_NETWORK_BROKER_SOCKET="$TMP_DIR/run/network/control.sock" \
    "$BOOT_TOOL"
)"
printf '%s\n' "$enabled_output" | grep -Fq 'status=started mode=supervised-root target=qemu-hwsim interface=wlan0'
grep -Eq '^[0-9]+$' "$TMP_DIR/run/wifi-service.pid"
grep -Eq '^[0-9]+$' "$TMP_DIR/run/network-privilege-broker.pid"
test -S "$TMP_DIR/run/wpa/wlan0"
test -S "$TMP_DIR/run/network/control.sock"

set +e
duplicate_output="$(
    AQUA_CMDLINE_PATH="$TMP_DIR/cmdline-enabled" \
    AQUA_WIFI_BOOT_PROFILE="$TMP_DIR/profile.conf" \
    AQUA_WIFI_SUPERVISOR_BIN="$TMP_DIR/supervisor" \
    AQUA_NETWORK_BROKER_BIN="$TMP_DIR/broker" \
    AQUA_WIFI_CONTROL_DIR="$TMP_DIR/run" \
    AQUA_NETWORK_CONTROL_DIR="$TMP_DIR/run/network" \
    AQUA_WIFI_WPA_CONTROL_SOCKET="$TMP_DIR/run/wpa/wlan0" \
    AQUA_TEST_WPA_SOCKET="$TMP_DIR/run/wpa/wlan0" \
    AQUA_NETWORK_BROKER_SOCKET="$TMP_DIR/run/network/control.sock" \
    "$BOOT_TOOL" 2>&1
)"
duplicate_status="$?"
set -e
test "$duplicate_status" -ne 0
printf '%s\n' "$duplicate_output" | grep -Fq 'status=duplicate'

echo 'Aqua Linux Wi-Fi service boot checks passed.'
