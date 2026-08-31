#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
DEFAULT_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_defconfig"
REHEARSAL_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_wifi_rehearsal_defconfig"
REHEARSAL_SCRIPT="$ROOT_DIR/scripts/rehearse-wifi-buildroot-closure.sh"
REPORT_WRITER="$ROOT_DIR/scripts/write-wifi-buildroot-closure.py"

test -f "$DEFAULT_CONFIG"
test -f "$REHEARSAL_CONFIG"
test -x "$REHEARSAL_SCRIPT"
test -x "$REPORT_WRITER"
sh -n "$REHEARSAL_SCRIPT"
python3 - "$REPORT_WRITER" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
compile(path.read_text(encoding="utf-8"), str(path), "exec")
PY

for symbol in \
    BR2_PACKAGE_WPA_SUPPLICANT \
    BR2_PACKAGE_WPA_SUPPLICANT_NL80211 \
    BR2_PACKAGE_WPA_SUPPLICANT_AUTOSCAN \
    BR2_PACKAGE_WPA_SUPPLICANT_WPA3 \
    BR2_PACKAGE_WPA_SUPPLICANT_WPA_CLIENT_SO \
    BR2_PACKAGE_WPA_SUPPLICANT_CTRL_IFACE
do
    grep -Fxq "${symbol}=y" "$REHEARSAL_CONFIG"
    ! grep -Fxq "${symbol}=y" "$DEFAULT_CONFIG"
done

for symbol in \
    BR2_PACKAGE_WPA_SUPPLICANT_WEXT \
    BR2_PACKAGE_WPA_SUPPLICANT_WIRED \
    BR2_PACKAGE_WPA_SUPPLICANT_AP_SUPPORT \
    BR2_PACKAGE_WPA_SUPPLICANT_MESH_NETWORKING \
    BR2_PACKAGE_WPA_SUPPLICANT_EAP \
    BR2_PACKAGE_WPA_SUPPLICANT_HOTSPOT \
    BR2_PACKAGE_WPA_SUPPLICANT_WPS \
    BR2_PACKAGE_WPA_SUPPLICANT_CLI \
    BR2_PACKAGE_WPA_SUPPLICANT_PASSPHRASE \
    BR2_PACKAGE_WPA_SUPPLICANT_DBUS \
    BR2_PACKAGE_DBUS \
    BR2_PACKAGE_IWD \
    BR2_PACKAGE_CONNMAN \
    BR2_PACKAGE_NETWORK_MANAGER \
    BR2_PACKAGE_DHCPCD
do
    grep -Fxq "${symbol}=n" "$REHEARSAL_CONFIG"
done

grep -Fq 'aqua_x86_64_wifi_rehearsal_defconfig' "$REHEARSAL_SCRIPT"
grep -Fq -- '--exclude website' "$REHEARSAL_SCRIPT"
grep -Fq 'wpa_supplicant' "$REHEARSAL_SCRIPT"
grep -Fq 'libwpa_client.so' "$REHEARSAL_SCRIPT"
grep -Fq 'legal-info' "$REHEARSAL_SCRIPT"
grep -Fq 'show-info' "$REHEARSAL_SCRIPT"
grep -Fq 'default_image_changed' "$REPORT_WRITER"
grep -Fq 'typed_control_transport_implemented' "$REPORT_WRITER"
grep -Fq 'physical_hardware_validated' "$REPORT_WRITER"
grep -Fq 'release_cleared' "$REPORT_WRITER"
grep -Fq '"wpa_supplicant": "2.12"' "$REPORT_WRITER"
grep -Fq '"libnl": "3.11.0"' "$REPORT_WRITER"
grep -Fq '"libopenssl": "3.5.7"' "$REPORT_WRITER"

echo 'Aqua Linux Wi-Fi Buildroot rehearsal checks passed.'
