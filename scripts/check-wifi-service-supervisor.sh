#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SUPERVISOR="$ROOT_DIR/br2-external/aqua/wifi-rootfs-overlay/usr/bin/aqua-wifi-service-supervisor"
STOP_TOOL="$ROOT_DIR/br2-external/aqua/wifi-rootfs-overlay/usr/bin/aqua-wifi-service-stop"
HOOK="$ROOT_DIR/br2-external/aqua/rootfs-overlay/usr/bin/aqua-udhcpc-hook"
FAKE_WPA="$ROOT_DIR/scripts/fake-wpa-service.py"
TMP_DIR="$(mktemp -d)"
supervisor_pid=""
trap 'if [ -n "$supervisor_pid" ]; then kill "$supervisor_pid" 2>/dev/null || true; fi; rm -rf "$TMP_DIR"' EXIT HUP INT TERM

for tool in "$SUPERVISOR" "$STOP_TOOL" "$HOOK" "$FAKE_WPA"; do
    test -x "$tool"
done

mkdir -p "$TMP_DIR/bin" "$TMP_DIR/run/aqua-wifi" \
    "$TMP_DIR/run/aqua-network" "$TMP_DIR/run/wpa_supplicant"
chmod 755 "$TMP_DIR/run/aqua-wifi" "$TMP_DIR/run/aqua-network" \
    "$TMP_DIR/run/wpa_supplicant"
printf '%s\n' 'ctrl_interface=/run/wpa_supplicant' > "$TMP_DIR/wpa.conf"
printf '%s\n' \
    'Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT' > "$TMP_DIR/route"
: > "$TMP_DIR/resolv.conf"
printf '%s\n' 1 > "$TMP_DIR/carrier"

cat > "$TMP_DIR/bin/default.script" <<'EOF'
#!/bin/sh
set -eu
case "$1" in
    bound|renew)
        printf '%s\n' \
            'Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT' \
            'wlan0 00000000 01002A0A 0003 0 0 0 00000000 0 0 0' > "$AQUA_TEST_ROUTE_FILE"
        printf '%s\n' 'nameserver 10.42.0.1 # wlan0' > "$AQUA_TEST_RESOLVER_FILE"
        ;;
    *)
        printf '%s\n' 'Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT' > "$AQUA_TEST_ROUTE_FILE"
        : > "$AQUA_TEST_RESOLVER_FILE"
        ;;
esac
EOF
cat > "$TMP_DIR/bin/udhcpc" <<'EOF'
#!/bin/sh
set -eu
hook=""
interface=""
while [ "$#" -gt 0 ]; do
    case "$1" in
        -s) hook="$2"; shift 2 ;;
        -i) interface="$2"; shift 2 ;;
        *) shift ;;
    esac
done
export interface
"$hook" bound
cleanup() {
    "$hook" deconfig
    exit 0
}
trap cleanup HUP INT TERM
while :; do sleep 1; done
EOF
chmod +x "$TMP_DIR/bin/default.script" "$TMP_DIR/bin/udhcpc"

association="$TMP_DIR/run/aqua-network/wifi.associated"
state="$TMP_DIR/run/aqua-wifi/wifi-service.state"
common_environment="AQUA_TEST_ROUTE_FILE=$TMP_DIR/route AQUA_TEST_RESOLVER_FILE=$TMP_DIR/resolv.conf"

env $common_environment \
    AQUA_WIFI_ENABLED=true \
    AQUA_WIFI_TEST_MODE=true \
    AQUA_WIFI_CONTROL_DIR="$TMP_DIR/run/aqua-wifi" \
    AQUA_NETWORK_CONTROL_DIR="$TMP_DIR/run/aqua-network" \
    AQUA_WIFI_WPA_CONTROL_DIR="$TMP_DIR/run/wpa_supplicant" \
    AQUA_WIFI_ASSOCIATION_FILE="$association" \
    AQUA_WIFI_CARRIER_FILE="$TMP_DIR/carrier" \
    AQUA_WIFI_ROUTE_TABLE="$TMP_DIR/route" \
    AQUA_WIFI_RESOLVER_FILE="$TMP_DIR/resolv.conf" \
    AQUA_WIFI_WPA_BIN="$FAKE_WPA" \
    AQUA_WIFI_WPA_CONFIG="$TMP_DIR/wpa.conf" \
    AQUA_WIFI_UDHCPC_BIN="$TMP_DIR/bin/udhcpc" \
    AQUA_WIFI_UDHCPC_HOOK="$HOOK" \
    AQUA_UDHCPC_DEFAULT_SCRIPT="$TMP_DIR/bin/default.script" \
    AQUA_WIFI_MAX_RESTARTS=3 \
    AQUA_WIFI_RESTART_DELAY_SECONDS=0 \
    AQUA_WIFI_READY_TIMEOUT_SECONDS=4 \
    AQUA_WIFI_MONITOR_INTERVAL_SECONDS=1 \
    AQUA_WIFI_STOP_TIMEOUT_SECONDS=4 \
    "$SUPERVISOR" > "$TMP_DIR/supervisor.log" 2>&1 &
supervisor_pid="$!"

wait_for_state() {
    expected_state="$1"
    expected_attempt="$2"
    waited=0
    while [ "$waited" -lt 20 ]; do
        if grep -Fq "state=$expected_state" "$state" 2>/dev/null &&
           grep -Fq "attempts=$expected_attempt" "$state" 2>/dev/null; then
            return 0
        fi
        sleep 1
        waited=$((waited + 1))
    done
    cat "$state" "$TMP_DIR/supervisor.log" 2>/dev/null || true
    return 1
}

write_association() {
    printf '%s\n' \
        'product=Aqua Linux' \
        'interface=wlan0' \
        'network_id=0' \
        'authoritative=true' > "$association"
}

wait_for_state ready 1
grep -Fq 'wpa_ready=true' "$state"
grep -Fq 'default_wifi=false' "$state"
write_association
wait_for_state running 1
grep -Fq 'associated=true' "$state"
grep -Fq 'carrier_ready=true' "$state"
grep -Fq 'lease_ready=true' "$state"
grep -Fq 'route_ready=true' "$state"
grep -Fq 'dns_ready=true' "$state"

rm -f "$association"
wait_for_state ready 1
grep -Fq 'associated=false' "$state"
grep -Fxq 'dhcp_pid=' "$state"

write_association
wait_for_state running 1
wpa_pid="$(sed -n 's/^wpa_pid=//p' "$state")"
kill -KILL "$wpa_pid"
wait_for_state ready 2
grep -Fq 'restarts=1' "$state"
test ! -e "$association"
write_association
wait_for_state running 2

AQUA_WIFI_CONTROL_DIR="$TMP_DIR/run/aqua-wifi" \
AQUA_WIFI_STOP_TIMEOUT_SECONDS=8 \
    "$STOP_TOOL" > "$TMP_DIR/stop.log"
wait "$supervisor_pid"
supervisor_pid=""
grep -Fq 'state=stopped' "$state"
grep -Fq 'wpa_supplicant_stopped=true dhcp_stopped=true' "$TMP_DIR/stop.log"
test ! -e "$TMP_DIR/run/aqua-wifi/wifi-service.pid"
test ! -e "$TMP_DIR/run/wpa_supplicant/wlan0"

echo 'Aqua Linux Wi-Fi service supervisor checks passed.'
