#!/bin/sh
set -eu

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ADR="$REPO_ROOT/docs/aqua-linux/adr-0004-audio-service-stack.md"
DEFCONFIG="$REPO_ROOT/br2-external/aqua/configs/aqua_x86_64_defconfig"
SUPERVISOR="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-media-service-supervisor"
STOP_TOOL="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-media-service-stop"
MEDIA_CONFIG="$REPO_ROOT/br2-external/aqua/rootfs-overlay/etc/aqua/media-services.conf"
GRAPHICS_SUPERVISOR="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphical-session-supervisor"

need_adr() {
    grep -Fq "$1" "$ADR" || {
        echo "Missing audio architecture contract: $1" >&2
        exit 1
    }
}

test -f "$ADR"
need_adr 'Accepted on 2026-08-29.'
need_adr 'ALSA kernel drivers and `alsa-lib`'
need_adr 'PipeWire is the per-user media server'
need_adr 'WirePlumber is the per-user PipeWire session and policy manager'
need_adr 'Aqua will not run the media'
need_adr 'graph as root or make sound devices globally writable'
need_adr 'not parse `wpctl` output'
need_adr 'supported Buildroot LTS baseline'
need_adr 'Physical hardware support remains `Not tested`'

test -x "$SUPERVISOR"
test -x "$STOP_TOOL"
grep -Fq 'ordered_start=pipewire,wireplumber' "$SUPERVISOR"
grep -Fq 'ordered_stop=wireplumber,pipewire' "$SUPERVISOR"
grep -Fq 'root_media_daemon=false' "$SUPERVISOR"
grep -Fq 'reason=restart-limit' "$SUPERVISOR"
grep -Fq 'MEDIA_SUPERVISOR_BIN=' "$GRAPHICS_SUPERVISOR"
grep -Fq 'stop_media_supervisor' "$GRAPHICS_SUPERVISOR"
grep -Fxq 'enabled=false' "$MEDIA_CONFIG"

if grep -Eq '^BR2_PACKAGE_(ALSA_LIB|ALSA_UTILS|PIPEWIRE|WIREPLUMBER)=y$' "$DEFCONFIG"; then
    echo 'Audio packages must remain disabled until ADR 0004 prerequisites pass.' >&2
    exit 1
fi

grep -Fq 'aqua_settings_audio_backend_applied=false' \
    "$REPO_ROOT/crates/aqua-compositor/src/lib.rs"
grep -Fq '| Audio | Not tested |' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"

echo 'Aqua Linux audio service architecture checks passed.'
