#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_NAME="${AQUA_COMPOSITOR_BUILDER_IMAGE:-rust:trixie}"
TARGET_TRIPLE="x86_64-unknown-linux-musl"
VOLUME_NAME="${VOLUME_NAME:-aqua-linux-buildroot-2025-02-work}"
WIFI_OUTPUT="/work/build/wifi-rehearsal-output"
case "$(docker info --format '{{.Architecture}}')" in
    aarch64 | arm64) BUILD_PLATFORM="linux/arm64" ;;
    x86_64 | amd64) BUILD_PLATFORM="linux/amd64" ;;
    *) echo 'Unsupported Docker architecture' >&2; exit 1 ;;
esac

docker volume inspect "$VOLUME_NAME" >/dev/null
docker run --rm \
    --platform "$BUILD_PLATFORM" \
    -v "$ROOT_DIR:/src" \
    -v "$VOLUME_NAME:/work" \
    -w /src \
    "$IMAGE_NAME" \
    sh -eu -c '
        output="'"$WIFI_OUTPUT"'"
        target="'"$TARGET_TRIPLE"'"
        linker="${output}/host/bin/x86_64-buildroot-linux-musl-gcc"
        sysroot="${output}/host/x86_64-buildroot-linux-musl/sysroot"
        test -x "${linker}"
        test -s "${sysroot}/usr/lib/libaqua-wifi-native.so"
        test -s "${sysroot}/usr/lib/libwpa_client.so"
        test -s "${sysroot}/usr/lib/libcrypto.so"
        rustup target add "${target}"
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="${linker}" \
        RUSTFLAGS="-C target-feature=-crt-static -L native=${sysroot}/usr/lib" \
            cargo build -p aqua-service-adapters --release \
                --target "${target}" --features wifi-native \
                --bin aqua-network-broker --bin aqua-wifi-native-probe
    '

docker run --rm \
    --platform linux/amd64 \
    -v "$ROOT_DIR:/src" \
    -v "$VOLUME_NAME:/work" \
    -w /src \
    "$IMAGE_NAME" \
    sh -eu -c '
        output="'"$WIFI_OUTPUT"'"
        target="'"$TARGET_TRIPLE"'"
        sysroot="${output}/host/x86_64-buildroot-linux-musl/sysroot"
        install -D -m 0755 "target/${target}/release/aqua-network-broker" \
            "${sysroot}/usr/bin/aqua-network-broker"
        install -D -m 0755 "target/${target}/release/aqua-wifi-native-probe" \
            "${sysroot}/usr/bin/aqua-wifi-native-probe"
        mkdir -p "${sysroot}/run/wpa_supplicant" \
            "${sysroot}/run/aqua-network" "${sysroot}/var/lib"
        chmod 755 "${sysroot}/run/wpa_supplicant" \
            "${sysroot}/run/aqua-network"
        unlink "${sysroot}/run/aqua-network/control.sock" 2>/dev/null || true
        unlink "${sysroot}/var/lib/aqua-network/wifi.psk" 2>/dev/null || true
        rmdir "${sysroot}/var/lib/aqua-network" 2>/dev/null || true

        python3 /src/scripts/fake-wpa-control-server.py \
            --socket "${sysroot}/run/wpa_supplicant/wlan0" \
            --client-root "${sysroot}" &
        fake_pid=$!
        broker_pid=""
        cleanup() {
            if [ -n "${broker_pid}" ]; then
                kill "${broker_pid}" 2>/dev/null || true
                wait "${broker_pid}" 2>/dev/null || true
            fi
            if [ -n "${fake_pid}" ]; then
                kill "${fake_pid}" 2>/dev/null || true
                wait "${fake_pid}" 2>/dev/null || true
            fi
        }
        trap cleanup EXIT HUP INT TERM
        waited=0
        while [ ! -S "${sysroot}/run/wpa_supplicant/wlan0" ]; do
            [ "${waited}" -lt 50 ] || exit 1
            sleep 0.1
            waited=$((waited + 1))
        done

        chroot "${sysroot}" /usr/bin/aqua-wifi-native-probe native
        chroot "${sysroot}" /usr/bin/aqua-network-broker serve \
            > "${sysroot}/run/aqua-network/broker.log" 2>&1 &
        broker_pid=$!
        waited=0
        while [ ! -S "${sysroot}/run/aqua-network/control.sock" ]; do
            [ "${waited}" -lt 50 ] || exit 1
            sleep 0.1
            waited=$((waited + 1))
        done
        chroot --userspec=1000:1000 "${sysroot}" \
            /usr/bin/aqua-wifi-native-probe broker
        credential="${sysroot}/var/lib/aqua-network/wifi.psk"
        test -f "${credential}"
        test ! -L "${credential}"
        test "$(stat -c %u "${credential}")" -eq 0
        test "$(stat -c %a "${credential}")" = 600
        test "$(wc -c < "${credential}")" -le 256
        ! grep -Fq password "${credential}"
        grep -Fq 'psk=f42c6fc52df0ebef9ebb4b90b38a5f902e83fe1b135a70e23aed762e9710a12e' \
            "${credential}"
        kill "${broker_pid}"
        wait "${broker_pid}" 2>/dev/null || true
        broker_pid=""
        wait "${fake_pid}"
        fake_pid=""
        ! grep -Fq password "${sysroot}/run/aqua-network/broker.log"
        grep -Fq "operations=status,renew-dhcp,wifi-status,wifi-scan,wifi-connect,wifi-reconnect,wifi-disconnect,wifi-forget" \
            "${sysroot}/run/aqua-network/broker.log"
        trap - EXIT HUP INT TERM
    '

echo 'Aqua Linux native Wi-Fi control and authenticated broker probe passed.'
