#!/bin/sh
set -eu

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ADR="$REPO_ROOT/docs/aqua-linux/adr-0005-network-service-stack.md"
DEFCONFIG="$REPO_ROOT/br2-external/aqua/configs/aqua_x86_64_defconfig"
ADAPTER="$REPO_ROOT/crates/aqua-service-adapters/src/network.rs"
SHELL_MODEL="$REPO_ROOT/crates/aqua-shell/src/lib.rs"
SETTINGS_CLIENT="$REPO_ROOT/crates/aqua-compositor/src/lib.rs"
SUPERVISOR="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-supervisor"
BOOT_TOOL="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-boot"
STOP_TOOL="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-stop"
UDHCPC_HOOK="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-udhcpc-hook"
UDHCPC_CLIENT="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-udhcpc-client"
NETWORK_CONFIG="$REPO_ROOT/br2-external/aqua/rootfs-overlay/etc/aqua/network-services.conf"
QEMU_NETWORK_CONFIG="$REPO_ROOT/br2-external/aqua/rootfs-overlay/etc/aqua/network-services-qemu.conf"
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
need_adr 'Physical Ethernet and Wi-Fi support remain `Not tested`'

grep -Fxq 'BR2_SYSTEM_DHCP="eth0"' "$DEFCONFIG"
if grep -Eq '^BR2_PACKAGE_(WPA_SUPPLICANT|IWD|CONNMAN|NETWORK_MANAGER|DHCPCD)=y$' "$DEFCONFIG"; then
    echo 'Network management packages must remain disabled until ADR 0005 gates pass.' >&2
    exit 1
fi

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
grep -Fq 'aqua_settings_network_health={}' "$SETTINGS_CLIENT"
grep -Fq 'aqua_settings_network_default_route={}' "$SETTINGS_CLIENT"
grep -Fq 'aqua_settings_network_dns_count={}' "$SETTINGS_CLIENT"
grep -Fq '| Network adapter | Validated |' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"
grep -Fq '`scripts/check-network-qemu.sh`' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"
grep -Fq '| Wi-Fi and Bluetooth | Not tested |' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"

echo 'Aqua Linux network service architecture checks passed.'
