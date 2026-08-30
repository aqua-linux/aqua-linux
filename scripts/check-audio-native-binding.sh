#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PACKAGE_DIR="$ROOT_DIR/br2-external/aqua/package/aqua-audio-native"
HEADER="$PACKAGE_DIR/src/aqua_audio_native.h"
IMPLEMENTATION="$PACKAGE_DIR/src/aqua_audio_native.c"
RUST_BINDING="$ROOT_DIR/crates/aqua-service-adapters/src/wireplumber_native.rs"
DEFAULT_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_defconfig"
REHEARSAL_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_audio_rehearsal_defconfig"
EXTERNAL_CONFIG="$ROOT_DIR/br2-external/aqua/Config.in"
EXTERNAL_MAKEFILE="$ROOT_DIR/br2-external/aqua/external.mk"

test -f "$PACKAGE_DIR/Config.in"
test -f "$PACKAGE_DIR/aqua-audio-native.mk"
test -f "$PACKAGE_DIR/src/Makefile"
test -f "$HEADER"
test -f "$IMPLEMENTATION"
test -f "$RUST_BINDING"

grep -Fq 'package/aqua-audio-native/Config.in' "$EXTERNAL_CONFIG"
grep -Fq 'package/*/*.mk' "$EXTERNAL_MAKEFILE"
grep -Fq 'depends on BR2_PACKAGE_WIREPLUMBER' "$PACKAGE_DIR/Config.in"
grep -Fq 'AQUA_AUDIO_NATIVE_DEPENDENCIES = host-pkgconf wireplumber' \
    "$PACKAGE_DIR/aqua-audio-native.mk"
grep -Fq 'AQUA_AUDIO_NATIVE_INSTALL_WIREPLUMBER_STAGING' \
    "$PACKAGE_DIR/aqua-audio-native.mk"
grep -Fq '$(WIREPLUMBER_DIR)/buildroot-build install' \
    "$PACKAGE_DIR/aqua-audio-native.mk"
grep -Fq 'DESTDIR=$(STAGING_DIR) install-devel' \
    "$PACKAGE_DIR/aqua-audio-native.mk"
grep -Fq 'DESTDIR=$(TARGET_DIR) install-runtime' \
    "$PACKAGE_DIR/aqua-audio-native.mk"
grep -Fq 'AQUA_AUDIO_NATIVE_ABI_VERSION 1U' "$HEADER"
grep -Fq 'AQUA_AUDIO_NATIVE_MAX_NODES 32U' "$HEADER"
grep -Fq 'aqua_audio_native_snapshot' "$HEADER"
grep -Fq 'libwireplumber-module-default-nodes-api' "$IMPLEMENTATION"
grep -Fq 'libwireplumber-module-mixer-api' "$IMPLEMENTATION"
grep -Fq 'WP_PLUGIN_FEATURE_ENABLED' "$IMPLEMENTATION"
grep -Fq 'on_plugin_activated' "$IMPLEMENTATION"
grep -Fq 'set-default-configured-node-name' "$IMPLEMENTATION"
grep -Fq 'wp_core_sync' "$IMPLEMENTATION"
grep -Fq 'operation_timeout' "$IMPLEMENTATION"
grep -Fq 'g_cancellable_cancel' "$IMPLEMENTATION"
grep -Fq 'handle_ref(handle)' "$IMPLEMENTATION"
grep -Fq '#[link(name = "aqua-audio-native")]' "$RUST_BINDING"
grep -Fq 'impl PipeWireApi for WirePlumberNativeApi' "$RUST_BINDING"
grep -Fq 'decode_snapshot' "$RUST_BINDING"
grep -Fxq 'BR2_PACKAGE_AQUA_AUDIO_NATIVE=y' "$REHEARSAL_CONFIG"
! grep -Fxq 'BR2_PACKAGE_AQUA_AUDIO_NATIVE=y' "$DEFAULT_CONFIG"

if grep -Eqi 'wpctl|popen[[:space:]]*\(|system[[:space:]]*\(|exec[lvpe]*[[:space:]]*\(' \
    "$IMPLEMENTATION" "$RUST_BINDING"; then
    echo 'The native audio binding must not invoke or parse helper commands.' >&2
    exit 1
fi

echo 'Aqua Linux native audio binding checks passed.'
