#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
CONTROL="$ROOT_DIR/crates/aqua-service-adapters/src/wifi_control.rs"
DEFAULT_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_defconfig"
REHEARSAL_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_wifi_rehearsal_defconfig"
SUPERVISOR="$ROOT_DIR/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-supervisor"
ADR="$ROOT_DIR/docs/aqua-linux/adr-0005-network-service-stack.md"

test -f "$CONTROL"
grep -Fq 'pub const WIFI_CONTROL_PROTOCOL: &str = "AQUA-WIFI-CONTROL/1"' "$CONTROL"
grep -Fq 'pub enum WifiControlRequest' "$CONTROL"
grep -Fq 'SetSsid' "$CONTROL"
grep -Fq 'SetPsk' "$CONTROL"
grep -Fq 'SetWpa2Personal' "$CONTROL"
grep -Fq 'MAX_WIFI_CONTROL_COMMAND_BYTES' "$CONTROL"
grep -Fq 'MAX_WIFI_CONTROL_RESPONSE_BYTES' "$CONTROL"
grep -Fq 'WifiControlStatus' "$CONTROL"
grep -Fq 'authoritative_association' "$CONTROL"

grep -Fq 'pub struct WifiPassphrase' "$CONTROL"
grep -Fq 'WifiPsk([redacted])' "$CONTROL"
grep -Fq 'pub struct WifiCredentialPayload' "$CONTROL"
grep -Fq 'Wpa2Personal' "$CONTROL"
grep -Fq 'WIFI_CREDENTIAL_DIRECTORY_MODE: u32 = 0o700' "$CONTROL"
grep -Fq 'WIFI_CREDENTIAL_FILE_MODE: u32 = 0o600' "$CONTROL"
grep -Fq 'WIFI_CREDENTIAL_TEMP_PATH' "$CONTROL"
grep -Fq 'validate_credential_metadata' "$CONTROL"
grep -Fq 'CredentialRecordTooLarge' "$CONTROL"

! grep -Fxq 'BR2_PACKAGE_WPA_SUPPLICANT=y' "$DEFAULT_CONFIG"
grep -Fxq 'BR2_PACKAGE_WPA_SUPPLICANT=y' "$REHEARSAL_CONFIG"
grep -Fq 'wifi_packaged=false' "$SUPERVISOR"
grep -Fq 'The default image still contains neither' "$ADR"
grep -Fq 'Settings exposes no Wi-Fi control' "$ADR"

echo 'Aqua Linux Wi-Fi control and credential contract checks passed.'
