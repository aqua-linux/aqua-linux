#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PACKAGE="$ROOT_DIR/br2-external/aqua/package/aqua-wifi-native"
HEADER="$PACKAGE/src/aqua_wifi_native.h"
IMPLEMENTATION="$PACKAGE/src/aqua_wifi_native.c"
RUST_BINDING="$ROOT_DIR/crates/aqua-service-adapters/src/wifi_native.rs"
BROKER="$ROOT_DIR/crates/aqua-service-adapters/src/bin/aqua-network-broker.rs"
PROTOCOL="$ROOT_DIR/crates/aqua-service-adapters/src/network_broker.rs"
DEFAULT_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_defconfig"
REHEARSAL_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_wifi_rehearsal_defconfig"

for file in \
    "$PACKAGE/Config.in" \
    "$PACKAGE/aqua-wifi-native.mk" \
    "$PACKAGE/src/Makefile" \
    "$PACKAGE/src/LICENSE" \
    "$HEADER" \
    "$IMPLEMENTATION" \
    "$RUST_BINDING"
do
    test -s "$file"
done

grep -Fq 'package/aqua-wifi-native/Config.in' "$ROOT_DIR/br2-external/aqua/Config.in"
grep -Fq 'AQUA_WIFI_NATIVE_DEPENDENCIES = wpa_supplicant openssl' \
    "$PACKAGE/aqua-wifi-native.mk"
grep -Fq 'AQUA_WIFI_NATIVE_ABI_VERSION 1U' "$HEADER"
grep -Fq 'AQUA_WIFI_NATIVE_CONTROL_PATH "/run/wpa_supplicant/wlan0"' "$HEADER"
grep -Fq 'AQUA_WIFI_NATIVE_MAX_COMMAND_BYTES 192U' "$HEADER"
grep -Fq 'AQUA_WIFI_NATIVE_MAX_RESPONSE_BYTES 4096U' "$HEADER"
grep -Fq 'PKCS5_PBKDF2_HMAC_SHA1' "$IMPLEMENTATION"
grep -Fq 'wpa_ctrl_open(AQUA_WIFI_NATIVE_CONTROL_PATH)' "$IMPLEMENTATION"
grep -Fq 'wpa_ctrl_request' "$IMPLEMENTATION"
grep -Fq '#[link(name = "aqua-wifi-native")]' "$RUST_BINDING"
grep -Fq 'pub struct WifiNativeControl' "$RUST_BINDING"
grep -Fq 'pub fn derive_wpa2_psk' "$RUST_BINDING"

grep -Fq 'parse_authenticated_request' "$PROTOCOL"
grep -Fq 'WIFI_CONNECT' "$PROTOCOL"
grep -Fq 'WIFI_RECONNECT' "$PROTOCOL"
grep -Fq 'WIFI_SCAN' "$PROTOCOL"
grep -Fq 'WIFI_FORGET' "$PROTOCOL"
grep -Fq 'request_wifi_connect' "$PROTOCOL"
grep -Fq 'wpa3-personal' "$PROTOCOL"
grep -Fq 'wipe_bytes(&mut passphrase_bytes)' "$PROTOCOL"
grep -Fq 'peer_credentials(&stream)' "$BROKER"
grep -Fq 'wipe_bytes(&mut bytes)' "$BROKER"
grep -Fq 'persist_wifi_credential' "$BROKER"
grep -Fq 'SetWpa3Personal' "$BROKER"
grep -Fq 'SetSaePassword' "$BROKER"
grep -Fq 'load_wifi_credential' "$BROKER"
grep -Fq 'remove_wifi_credential' "$BROKER"
grep -Fq 'CredentialTransaction' "$BROKER"

grep -Fxq 'BR2_PACKAGE_AQUA_WIFI_NATIVE=y' "$REHEARSAL_CONFIG"
! grep -Fxq 'BR2_PACKAGE_AQUA_WIFI_NATIVE=y' "$DEFAULT_CONFIG"
test -x "$ROOT_DIR/scripts/build-wifi-native-probe-linux-docker.sh"
test -x "$ROOT_DIR/scripts/fake-wpa-control-server.py"
grep -Fq 'libwpa_client' "$ROOT_DIR/scripts/build-wifi-native-probe-linux-docker.sh"
grep -Fq 'chroot --userspec=1000:1000' \
    "$ROOT_DIR/scripts/build-wifi-native-probe-linux-docker.sh"

if grep -Eqi 'popen[[:space:]]*\(|system[[:space:]]*\(|exec[lvpe]*[[:space:]]*\(' \
    "$IMPLEMENTATION" "$RUST_BINDING"; then
    echo 'The native Wi-Fi binding must not invoke helper commands.' >&2
    exit 1
fi

echo 'Aqua Linux native Wi-Fi binding checks passed.'
