#!/bin/sh
set -eu

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ADR="$REPO_ROOT/docs/aqua-linux/adr-0005-network-service-stack.md"
DEFCONFIG="$REPO_ROOT/br2-external/aqua/configs/aqua_x86_64_defconfig"
ADAPTER="$REPO_ROOT/crates/aqua-service-adapters/src/network.rs"
BROKER_PROTOCOL="$REPO_ROOT/crates/aqua-service-adapters/src/network_broker.rs"
BROKER="$REPO_ROOT/crates/aqua-service-adapters/src/bin/aqua-network-broker.rs"
WIFI_CONTROL="$REPO_ROOT/crates/aqua-service-adapters/src/wifi_control.rs"
SHELL_MODEL="$REPO_ROOT/crates/aqua-shell/src/lib.rs"
SETTINGS_CLIENT="$REPO_ROOT/crates/aqua-compositor/src/lib.rs"
SUPERVISOR="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-supervisor"
BOOT_TOOL="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-boot"
STOP_TOOL="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-stop"
UDHCPC_HOOK="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-udhcpc-hook"
UDHCPC_CLIENT="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-udhcpc-client"
NETWORK_CONFIG="$REPO_ROOT/br2-external/aqua/rootfs-overlay/etc/aqua/network-services.conf"
QEMU_NETWORK_CONFIG="$REPO_ROOT/br2-external/aqua/rootfs-overlay/etc/aqua/network-services-qemu.conf"
WIFI_REHEARSAL_CONFIG="$REPO_ROOT/br2-external/aqua/configs/aqua_x86_64_wifi_rehearsal_defconfig"
RCS="$REPO_ROOT/br2-external/aqua/rootfs-overlay/etc/init.d/rcS"

need_adr() {
    grep -Fq "$1" "$ADR" || {
        echo "Missing network architecture contract: $1" >&2
        exit 1
    }
}

test -f "$ADR"
need_adr 'Accepted on 2026-08-31.'
need_adr 'BusyBox `udhcpc` remains the initial IPv4 DHCP client'
need_adr '`wpa_supplicant` is selected for Wi-Fi association'
need_adr 'will not add NetworkManager, ConnMan, or a second DHCP client'
need_adr 'narrow authenticated privilege broker'
need_adr '**Satisfied on 2026-08-31:** the privilege broker authenticates'
need_adr 'Physical Ethernet and Wi-Fi support remain `Not tested`'

grep -Fxq 'BR2_SYSTEM_DHCP="eth0"' "$DEFCONFIG"
if grep -Eq '^BR2_PACKAGE_(WPA_SUPPLICANT|IWD|CONNMAN|NETWORK_MANAGER|DHCPCD)=y$' "$DEFCONFIG"; then
    echo 'Network management packages must remain disabled until ADR 0005 gates pass.' >&2
    exit 1
fi
test -f "$WIFI_REHEARSAL_CONFIG"
grep -Fxq 'BR2_PACKAGE_WPA_SUPPLICANT=y' "$WIFI_REHEARSAL_CONFIG"
grep -Fxq 'BR2_PACKAGE_WPA_SUPPLICANT_NL80211=y' "$WIFI_REHEARSAL_CONFIG"
grep -Fxq 'BR2_PACKAGE_WPA_SUPPLICANT_WPA3=y' "$WIFI_REHEARSAL_CONFIG"
grep -Fxq 'BR2_PACKAGE_WPA_SUPPLICANT_WPA_CLIENT_SO=y' "$WIFI_REHEARSAL_CONFIG"
grep -Fxq 'BR2_PACKAGE_WPA_SUPPLICANT_DBUS=n' "$WIFI_REHEARSAL_CONFIG"
test -x "$REPO_ROOT/scripts/check-wifi-buildroot-rehearsal.sh"
test -x "$REPO_ROOT/scripts/rehearse-wifi-buildroot-closure.sh"
test -x "$REPO_ROOT/scripts/check-wifi-control-contract.sh"
test -x "$REPO_ROOT/scripts/check-wifi-native-binding.sh"
grep -Fq 'pub enum WifiControlRequest' "$WIFI_CONTROL"
grep -Fq 'pub struct WifiCredentialRecord' "$WIFI_CONTROL"
grep -Fq 'WIFI_CREDENTIAL_FILE_MODE: u32 = 0o600' "$WIFI_CONTROL"
grep -Fq 'parse_authenticated_request' "$BROKER_PROTOCOL"
grep -Fq 'persist_wifi_credential' "$BROKER"
grep -Fq 'request_wifi_scan' "$BROKER_PROTOCOL"
grep -Fq 'request_wifi_connect' "$BROKER_PROTOCOL"
grep -Fq 'WifiScanSecurity::Wpa3Personal' "$BROKER_PROTOCOL"
grep -Fq 'WIFI_FORGET' "$BROKER_PROTOCOL"
grep -Fq 'WifiSecretInput' "$SHELL_MODEL"
grep -Fq 'MAX_WIFI_CONNECT_ATTEMPTS: u8 = 2' "$SHELL_MODEL"
grep -Fq 'WifiScanRequested' "$SHELL_MODEL"
grep -Fq 'WifiForgetRequested' "$SHELL_MODEL"

test -x "$SUPERVISOR"
test -x "$BOOT_TOOL"
test -x "$STOP_TOOL"
test -x "$UDHCPC_HOOK"
test -x "$UDHCPC_CLIENT"
grep -Fxq 'enabled=false' "$NETWORK_CONFIG"
grep -Fxq 'legacy_owner_disabled=false' "$NETWORK_CONFIG"
grep -Fxq 'enabled=true' "$QEMU_NETWORK_CONFIG"
grep -Fxq 'legacy_owner_disabled=true' "$QEMU_NETWORK_CONFIG"
grep -Fxq 'profile_scope=qemu-only' "$QEMU_NETWORK_CONFIG"
grep -Fq 'reason=root-service-required' "$SUPERVISOR"
grep -Fq 'reason=legacy-owner-not-declared-disabled' "$SUPERVISOR"
grep -Fq 'reason=interface-not-present' "$SUPERVISOR"
grep -Fq 'policy_owner=aqua-network-service-supervisor' "$SUPERVISOR"
grep -Fq 'settings_management=false' "$SUPERVISOR"
grep -Fq 'wifi_packaged=false' "$SUPERVISOR"
grep -Fq 'AQUA_UDHCPC_DEFAULT_SCRIPT' "$UDHCPC_HOOK"
grep -Fq 'exec /sbin/udhcpc -f -n -i eth0 -s /usr/bin/aqua-udhcpc-hook' "$UDHCPC_CLIENT"
grep -Fxq 'udhcpc_binary=/usr/bin/aqua-udhcpc-client' "$NETWORK_CONFIG"
grep -Fxq 'udhcpc_binary=/usr/bin/aqua-udhcpc-client' "$QEMU_NETWORK_CONFIG"
set +e
client_output="$("${UDHCPC_CLIENT}" 2>&1)"
client_status="$?"
set -e
test "${client_status}" -eq 2
printf '%s\n' "${client_output}" | grep -Fq 'status=blocked reason=invalid-arguments'
grep -Fq 'aqua.boot_network=1' "$BOOT_TOOL"
grep -Fq 'reason=invalid-qemu-network-profile' "$BOOT_TOOL"
grep -Fq '/usr/bin/aqua-network-service-boot' "$RCS"
if grep -Eq '/etc/init\.d/S40network|for .*S\?\?' "$RCS"; then
    echo 'Aqua rcS must not invoke the generated Buildroot network init script.' >&2
    exit 1
fi
grep -Fq 'network-service-supervisor.txt' "$REPO_ROOT/scripts/export-rootfs-compositor-contract-docker.sh"
grep -Fq 'network-service-supervisor.txt' "$REPO_ROOT/scripts/check-compositor-rootfs-docker.sh"
grep -Fq 'network-service-boot.txt' "$REPO_ROOT/scripts/export-rootfs-compositor-contract-docker.sh"
grep -Fq 'network-service-boot.txt' "$REPO_ROOT/scripts/check-compositor-rootfs-docker.sh"
grep -Fq 'network-broker-binary.txt' "$REPO_ROOT/scripts/export-rootfs-compositor-contract-docker.sh"
grep -Fq 'network-broker-binary.txt' "$REPO_ROOT/scripts/check-compositor-rootfs-docker.sh"

test -f "$BROKER_PROTOCOL"
test -f "$BROKER"
grep -Fq 'pub const MAX_REQUEST_BYTES: usize = 256;' "$BROKER_PROTOCOL"
grep -Fq 'pub const MAX_RESPONSE_BYTES: usize = 512;' "$BROKER_PROTOCOL"
grep -Fq 'pub const PROTOCOL_VERSION: &str = "AQUA-NETWORK/1";' "$BROKER_PROTOCOL"
grep -Fq 'NetworkBrokerOperation::Status' "$BROKER_PROTOCOL"
grep -Fq 'NetworkBrokerOperation::RenewDhcp' "$BROKER_PROTOCOL"
grep -Fq 'libc::SO_PEERCRED' "$BROKER"
grep -Fq 'peer.uid != AQUA_UID || peer.gid != AQUA_GID' "$BROKER"
grep -Fq 'ERROR unauthorized-peer' "$BROKER"
grep -Fq 'pub const NETWORK_BROKER_SOCKET_PATH: &str = "/run/aqua-network/control.sock";' "$BROKER_PROTOCOL"
grep -Fq 'const SOCKET_PATH: &str = aqua_service_adapters::network_broker::NETWORK_BROKER_SOCKET_PATH;' "$BROKER"
grep -Fq 'arbitrary_commands=false arbitrary_paths=false' "$BROKER"
grep -Fq 'aqua-network-broker' "$REPO_ROOT/scripts/build-compositor-linux-docker.sh"
grep -Fq 'aqua-network-broker' "$REPO_ROOT/scripts/build-image-docker-volume.sh"
grep -Fq 'aqua-network-broker' "$REPO_ROOT/br2-external/aqua/board/aqua/x86_64/post-build.sh"
grep -Fq 'broker_auth=true root_rejected=true typed_renewal=true' "$REPO_ROOT/scripts/check-network-qemu.sh"

test -f "$ADAPTER"
grep -Fq 'pub enum NetworkServiceHealth' "$ADAPTER"
grep -Fq 'pub struct NetworkAuthoritativeState' "$ADAPTER"
grep -Fq 'pub fn read_network_snapshot' "$ADAPTER"
grep -Fq 'MAX_NETWORK_INTERFACES' "$ADAPTER"
grep -Fq 'MAX_DNS_SERVERS' "$ADAPTER"
grep -Fq 'MAX_ROUTE_BYTES' "$ADAPTER"
grep -Fq 'MAX_RESOLVER_BYTES' "$ADAPTER"
if grep -Eq 'Command::new|process::Command|udhcpc|wpa_cli' "$ADAPTER"; then
    echo 'The unprivileged network adapter must not spawn management commands.' >&2
    exit 1
fi

grep -Fq 'NetworkAuthoritativeState' "$SHELL_MODEL"
grep -Fq 'read_network_snapshot' "$SHELL_MODEL"
grep -Fq 'WifiControlRequested' "$SHELL_MODEL"
grep -Fq 'request_wifi_broker' "$SHELL_MODEL"
grep -Fq 'aqua_settings_network_health={}' "$SETTINGS_CLIENT"
grep -Fq 'aqua_settings_network_default_route={}' "$SETTINGS_CLIENT"
grep -Fq 'aqua_settings_network_dns_count={}' "$SETTINGS_CLIENT"
grep -Fq 'aqua_settings_wifi_control requested={}' "$SETTINGS_CLIENT"
grep -Fq '| Network adapter | Validated |' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"
grep -Fq '`scripts/check-network-qemu.sh`' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"
grep -Fq '| Wi-Fi | Validated |' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"
grep -Fq '| Bluetooth | Not tested |' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"

echo 'Aqua Linux network service architecture checks passed.'
