#!/bin/sh
set -eu

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ADR="$REPO_ROOT/docs/aqua-linux/adr-0004-audio-service-stack.md"
DEFCONFIG="$REPO_ROOT/br2-external/aqua/configs/aqua_x86_64_defconfig"
SUPERVISOR="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-media-service-supervisor"
STOP_TOOL="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-media-service-stop"
MEDIA_CONFIG="$REPO_ROOT/br2-external/aqua/rootfs-overlay/etc/aqua/media-services.conf"
GRAPHICS_SUPERVISOR="$REPO_ROOT/br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphical-session-supervisor"
ADAPTER="$REPO_ROOT/crates/aqua-service-adapters/src/lib.rs"
PIPEWIRE_TRANSPORT="$REPO_ROOT/crates/aqua-service-adapters/src/pipewire.rs"
NATIVE_BINDING="$REPO_ROOT/crates/aqua-service-adapters/src/wireplumber_native.rs"
SHELL_MODEL="$REPO_ROOT/crates/aqua-shell/src/lib.rs"
SETTINGS_CLIENT="$REPO_ROOT/crates/aqua-compositor/src/lib.rs"

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

test -f "$ADAPTER"
grep -Fq '"crates/aqua-service-adapters"' "$REPO_ROOT/Cargo.toml"
grep -Fq 'pub trait AudioBackend' "$ADAPTER"
grep -Fq 'pub struct AudioServiceAdapter' "$ADAPTER"
grep -Fq 'MAX_AUDIO_DEVICES' "$ADAPTER"
grep -Fq 'StaleGeneration' "$ADAPTER"
grep -Fq 'ConflictingGeneration' "$ADAPTER"
grep -Fq 'request_confirmed' "$ADAPTER"
test -f "$PIPEWIRE_TRANSPORT"
grep -Fq 'pub trait PipeWireApi' "$PIPEWIRE_TRANSPORT"
grep -Fq 'pub struct PipeWireApiTransport' "$PIPEWIRE_TRANSPORT"
grep -Fq 'fn synchronized_snapshot' "$PIPEWIRE_TRANSPORT"
grep -Fq 'DefaultsBeforeGraphSync' "$PIPEWIRE_TRANSPORT"
grep -Fq 'GenerationMismatch' "$PIPEWIRE_TRANSPORT"
grep -Fq 'set_configured_default_output' "$PIPEWIRE_TRANSPORT"
test -f "$NATIVE_BINDING"
grep -Fq 'pub struct WirePlumberNativeApi' "$NATIVE_BINDING"
grep -Fq 'impl PipeWireApi for WirePlumberNativeApi' "$NATIVE_BINDING"
grep -Fq 'AudioServiceAdapter' "$SHELL_MODEL"
grep -Fq 'authoritative_volume_percent' "$SHELL_MODEL"
grep -Fq 'aqua_settings_audio_service_health=' "$SETTINGS_CLIENT"
grep -Fq 'aqua_settings_audio_backend_applied={}' "$SETTINGS_CLIENT"
if grep -Fq 'AQUA_AUDIO_DEV_SND' "$SETTINGS_CLIENT"; then
    echo 'Settings must not treat /dev/snd as authoritative service readiness.' >&2
    exit 1
fi
if grep -Eq 'wpctl|Command::new|process::Command' "$ADAPTER"; then
    echo 'The audio adapter contract must not parse commands or spawn helper tools.' >&2
    exit 1
fi
if grep -Eq 'wpctl|Command::new|process::Command' "$PIPEWIRE_TRANSPORT"; then
    echo 'The PipeWire transport must use typed native API calls, not helper commands.' >&2
    exit 1
fi
grep -Fq '| Audio | Not tested |' \
    "$REPO_ROOT/docs/aqua-linux/hardware-support.md"

echo 'Aqua Linux audio service architecture checks passed.'
