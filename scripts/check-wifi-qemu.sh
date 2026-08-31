#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EVIDENCE_DIR="${EVIDENCE_DIR:-$ROOT_DIR/build/wifi-qemu-runtime}"
KERNEL="${KERNEL:-$EVIDENCE_DIR/bzImage}"
ROOTFS="${ROOTFS:-$EVIDENCE_DIR/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-$ROOT_DIR/build/qemu-wifi-check.log}"
MEMORY="${MEMORY:-768M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-180}"

for tool in expect qemu-system-x86_64; do
    command -v "$tool" >/dev/null 2>&1 || { echo "Missing required tool: $tool" >&2; exit 1; }
done
for artifact in "$KERNEL" "$ROOTFS"; do
    test -s "$artifact" || { echo "Missing Wi-Fi QEMU artifact: $artifact" >&2; echo 'Run scripts/build-wifi-qemu-runtime.sh first.' >&2; exit 1; }
done
mkdir -p "$(dirname "$SERIAL_LOG")"
rm -f "$SERIAL_LOG"
export ROOT_DIR KERNEL ROOTFS SERIAL_LOG MEMORY CPUS TIMEOUT_SECONDS
expect "$ROOT_DIR/scripts/check-wifi-qemu.exp"
grep -Fq '[AQUA-WIFI] stage=qemu-hwsim-acceptance status=ok radios=2 discovery=true rescan=true new_credential=true association=true dhcp=true disconnect=true reconnect=true service_recovery=true forget=true disable=true broker_auth=true default_wifi=false recovery_shell=true' "$SERIAL_LOG"

echo 'Aqua Linux opt-in mac80211_hwsim Wi-Fi QEMU acceptance passed.'
echo "Serial log: $SERIAL_LOG"
