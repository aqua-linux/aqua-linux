#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-hardware-inventory"
FIXTURE="$(mktemp -d)"
trap 'rm -rf "${FIXTURE}"' EXIT HUP INT TERM

SYS_ROOT="${FIXTURE}/sys"
PROC_ROOT="${FIXTURE}/proc"
OUTPUT="${FIXTURE}/inventory.txt"
BEFORE="${FIXTURE}/before.txt"
AFTER="${FIXTURE}/after.txt"

mkdir -p \
    "${SYS_ROOT}/class/dmi/id" \
    "${SYS_ROOT}/firmware/efi" \
    "${SYS_ROOT}/bus/pci/devices/0000:00:02.0" \
    "${SYS_ROOT}/bus/pci/devices/0000:01:00.0" \
    "${SYS_ROOT}/bus/usb/devices/1-2" \
    "${SYS_ROOT}/class/drm/card0-eDP-1" \
    "${SYS_ROOT}/class/input/event0/device" \
    "${SYS_ROOT}/class/block/nvme0n1/device" \
    "${SYS_ROOT}/class/power_supply/BAT0" \
    "${PROC_ROOT}"

printf '%s\n' 'Micro-Star International Co., Ltd.' > "${SYS_ROOT}/class/dmi/id/sys_vendor"
printf '%s\n' 'Sword 17 HX B14V' > "${SYS_ROOT}/class/dmi/id/product_name"
printf '%s\n' 'REV:1.0' > "${SYS_ROOT}/class/dmi/id/product_version"
printf '%s\n' 'MS-17S2' > "${SYS_ROOT}/class/dmi/id/board_name"
printf '%s\n' 'American Megatrends International, LLC.' > "${SYS_ROOT}/class/dmi/id/bios_vendor"
printf '%s\n' 'E17S2IMS.100' > "${SYS_ROOT}/class/dmi/id/bios_version"
printf '%s\n' 'model name : Fixture CPU' > "${PROC_ROOT}/cpuinfo"

printf '%s\n' '0x8086' > "${SYS_ROOT}/bus/pci/devices/0000:00:02.0/vendor"
printf '%s\n' '0x1234' > "${SYS_ROOT}/bus/pci/devices/0000:00:02.0/device"
printf '%s\n' '0x030000' > "${SYS_ROOT}/bus/pci/devices/0000:00:02.0/class"
mkdir -p "${FIXTURE}/drivers/i915"
ln -s "${FIXTURE}/drivers/i915" "${SYS_ROOT}/bus/pci/devices/0000:00:02.0/driver"
printf '%s\n' '0x10de' > "${SYS_ROOT}/bus/pci/devices/0000:01:00.0/vendor"
printf '%s\n' '0xabcd' > "${SYS_ROOT}/bus/pci/devices/0000:01:00.0/device"
printf '%s\n' '0x030200' > "${SYS_ROOT}/bus/pci/devices/0000:01:00.0/class"

printf '%s\n' '0bda' > "${SYS_ROOT}/bus/usb/devices/1-2/idVendor"
printf '%s\n' '0129' > "${SYS_ROOT}/bus/usb/devices/1-2/idProduct"
printf '%s\n' 'Bluetooth|Radio=Fixture' > "${SYS_ROOT}/bus/usb/devices/1-2/product"
printf '%s\n' 'connected' > "${SYS_ROOT}/class/drm/card0-eDP-1/status"
printf '%s\n' '1920x1080' > "${SYS_ROOT}/class/drm/card0-eDP-1/modes"
printf '%s\n' 'AT Translated Set 2 keyboard' > "${SYS_ROOT}/class/input/event0/device/name"
printf '%s\n' 'Fixture NVMe' > "${SYS_ROOT}/class/block/nvme0n1/device/model"
printf '%s\n' '1000000' > "${SYS_ROOT}/class/block/nvme0n1/size"
printf '%s\n' '0' > "${SYS_ROOT}/class/block/nvme0n1/ro"
printf '%s\n' '0' > "${SYS_ROOT}/class/block/nvme0n1/removable"
printf '%s\n' 'Battery' > "${SYS_ROOT}/class/power_supply/BAT0/type"
printf '%s\n' 'Charging' > "${SYS_ROOT}/class/power_supply/BAT0/status"
printf '%s\n' '72' > "${SYS_ROOT}/class/power_supply/BAT0/capacity"

find "${SYS_ROOT}" "${PROC_ROOT}" -type f -exec cksum {} \; | sort > "${BEFORE}"
AQUA_INVENTORY_SYS_ROOT="${SYS_ROOT}" \
AQUA_INVENTORY_PROC_ROOT="${PROC_ROOT}" \
    "${TOOL}" > "${OUTPUT}"
find "${SYS_ROOT}" "${PROC_ROOT}" -type f -exec cksum {} \; | sort > "${AFTER}"
cmp -s "${BEFORE}" "${AFTER}"

grep -Fxq 'schema=1' "${OUTPUT}"
grep -Fxq 'collection_mode=read-only' "${OUTPUT}"
grep -Fxq 'support_claim=false' "${OUTPUT}"
grep -Fxq 'installation_authorized=false' "${OUTPUT}"
grep -Fxq 'identifiers_excluded=serial,mac,hostname,uuid' "${OUTPUT}"
grep -Fxq 'dmi|field=sys_vendor|value=Micro-Star International Co., Ltd.' "${OUTPUT}"
grep -Fxq 'dmi|field=product_name|value=Sword 17 HX B14V' "${OUTPUT}"
grep -Fxq 'cpu|model=Fixture CPU' "${OUTPUT}"
grep -Fxq 'firmware|uefi=true' "${OUTPUT}"
grep -Fxq 'pci|address=0000:00:02.0|vendor=0x8086|device=0x1234|class=0x030000|driver=i915' "${OUTPUT}"
grep -Fxq 'pci|address=0000:01:00.0|vendor=0x10de|device=0xabcd|class=0x030200|driver=none' "${OUTPUT}"
grep -Fxq 'pci_count=2' "${OUTPUT}"
grep -Fxq 'usb|node=1-2|vendor=0bda|product_id=0129|product=Bluetooth Radio Fixture' "${OUTPUT}"
grep -Fxq 'drm|connector=card0-eDP-1|status=connected|first_mode=1920x1080' "${OUTPUT}"
grep -Fxq 'input|event=event0|name=AT Translated Set 2 keyboard' "${OUTPUT}"
grep -Fxq 'block|name=nvme0n1|model=Fixture NVMe|size_sectors=1000000|read_only=0|removable=0' "${OUTPUT}"
grep -Fxq 'power|name=BAT0|type=Battery|status=Charging|capacity=72' "${OUTPUT}"
grep -Fxq 'collection_complete=true' "${OUTPUT}"

AQUA_INVENTORY_SYS_ROOT="${SYS_ROOT}" \
AQUA_INVENTORY_PROC_ROOT="${PROC_ROOT}" \
AQUA_INVENTORY_MAX_ENTRIES=1 \
    "${TOOL}" > "${FIXTURE}/bounded-inventory.txt"
grep -Fxq 'pci_count=1' "${FIXTURE}/bounded-inventory.txt"
test "$(grep -c '^pci|' "${FIXTURE}/bounded-inventory.txt")" = "1"

if grep -Eiq '(^|[|_])(serial|mac|hostname|uuid)=' "${OUTPUT}"; then
    echo "Hardware inventory exposed an excluded identifier" >&2
    exit 1
fi

echo "Aqua Linux read-only hardware inventory checks passed."
