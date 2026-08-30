#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BASE_OVERLAY="$ROOT_DIR/br2-external/aqua/rootfs-overlay"
AUDIO_OVERLAY="$ROOT_DIR/br2-external/aqua/audio-rootfs-overlay"
DEFAULT_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_defconfig"
AUDIO_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_audio_rehearsal_defconfig"
CHECKER="$AUDIO_OVERLAY/usr/bin/aqua-audio-rootfs-check"
REHEARSAL="$ROOT_DIR/scripts/rehearse-audio-rootfs-contract.sh"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/aqua-audio-rootfs.XXXXXX")"
trap 'rm -rf "${FIXTURE}"' EXIT HUP INT TERM

test -x "$CHECKER"
test -x "$REHEARSAL"
sh -n "$CHECKER"
sh -n "$REHEARSAL"
grep -Fq 'images/rootfs.tar' "$REHEARSAL"
grep -Fq 'AQUA_AUDIO_ROOT="${rootfs}"' "$REHEARSAL"
grep -Fq 'rootfs-contract.sha256' "$REHEARSAL"
grep -Fxq 'enabled=false' "$BASE_OVERLAY/etc/aqua/media-services.conf"
grep -Fxq 'enabled=true' "$AUDIO_OVERLAY/etc/aqua/media-services.conf"
grep -Fxq 'automatic_root_service=false' "$AUDIO_OVERLAY/etc/aqua/media-services.conf"
grep -Fxq 'native_library=/usr/lib/libaqua-audio-native.so.1' \
    "$AUDIO_OVERLAY/etc/aqua/media-services.conf"
for assignment in \
    'buildroot_version=2025.02.17' \
    'alsa_lib_version=1.2.13' \
    'pipewire_version=1.2.8' \
    'wireplumber_version=0.5.5' \
    'eudev_version=3.2.14' \
    'lua_version=5.4.8' \
    'glib_version=2.82.5' \
    'aqua_audio_native_version=2' \
    'aqua_audio_probe_version=1'
do
    grep -Fxq "$assignment" "$AUDIO_OVERLAY/etc/aqua/audio-stack.conf"
done
grep -Fxq 'BR2_ROOTFS_OVERLAY="$(BR2_EXTERNAL_AQUA_PATH)/rootfs-overlay"' \
    "$DEFAULT_CONFIG"
grep -Fxq \
    'BR2_ROOTFS_OVERLAY="$(BR2_EXTERNAL_AQUA_PATH)/rootfs-overlay $(BR2_EXTERNAL_AQUA_PATH)/audio-rootfs-overlay"' \
    "$AUDIO_CONFIG"
! grep -Fq 'audio-rootfs-overlay' "$DEFAULT_CONFIG"
! grep -Fxq 'BR2_PACKAGE_PIPEWIRE=y' "$DEFAULT_CONFIG"
! grep -Fxq 'BR2_PACKAGE_WIREPLUMBER=y' "$DEFAULT_CONFIG"
! grep -Fxq 'BR2_PACKAGE_AQUA_AUDIO_NATIVE=y' "$DEFAULT_CONFIG"

cp -R "$BASE_OVERLAY/." "$FIXTURE/"
cp -R "$AUDIO_OVERLAY/." "$FIXTURE/"
mkdir -p \
    "$FIXTURE/usr/lib/pipewire-0.3" \
    "$FIXTURE/usr/libexec/aqua-tests" \
    "$FIXTURE/usr/lib/wireplumber-0.5" \
    "$FIXTURE/usr/share/pipewire" \
    "$FIXTURE/usr/share/wireplumber"
for executable in pipewire wireplumber wpctl aqua-audio-probe; do
    : > "$FIXTURE/usr/bin/$executable"
    chmod 755 "$FIXTURE/usr/bin/$executable"
done
: > "$FIXTURE/usr/libexec/aqua-tests/aqua-audio-adapter-probe"
chmod 755 "$FIXTURE/usr/libexec/aqua-tests/aqua-audio-adapter-probe"
for runtime_file in \
    usr/lib/libaqua-audio-native.so.1 \
    usr/lib/pipewire-0.3/libpipewire-module-protocol-native.so \
    usr/lib/wireplumber-0.5/libwireplumber-module-default-nodes-api.so \
    usr/lib/wireplumber-0.5/libwireplumber-module-mixer-api.so \
    usr/share/pipewire/pipewire.conf \
    usr/share/wireplumber/wireplumber.conf \
    etc/alsa/conf.d/99-aqua-pipewire-default.conf
do
    : > "$FIXTURE/$runtime_file"
done
mkdir -p "$FIXTURE/etc/init.d"
printf '%s\n' 'aqua:x:1000:1000:Aqua desktop session:/home/aqua:/bin/false' > "$FIXTURE/etc/passwd"
printf '%s\n' \
    'audio:x:29:aqua' \
    'video:x:44:aqua' \
    'input:x:30:aqua' > "$FIXTURE/etc/group"

AQUA_AUDIO_ROOT="$FIXTURE" "$FIXTURE/usr/bin/aqua-audio-rootfs-check" >/dev/null

mv "$FIXTURE/usr/lib/libaqua-audio-native.so.1" \
    "$FIXTURE/usr/lib/libaqua-audio-native.so.1.missing"
if AQUA_AUDIO_ROOT="$FIXTURE" "$FIXTURE/usr/bin/aqua-audio-rootfs-check" >/dev/null 2>&1; then
    echo 'Rootfs contract accepted a missing native audio library.' >&2
    exit 1
fi
mv "$FIXTURE/usr/lib/libaqua-audio-native.so.1.missing" \
    "$FIXTURE/usr/lib/libaqua-audio-native.so.1"

printf '%s\n' '#!/bin/sh' 'exec /usr/bin/pipewire' > "$FIXTURE/etc/init.d/S50pipewire"
chmod 755 "$FIXTURE/etc/init.d/S50pipewire"
if AQUA_AUDIO_ROOT="$FIXTURE" "$FIXTURE/usr/bin/aqua-audio-rootfs-check" >/dev/null 2>&1; then
    echo 'Rootfs contract accepted automatic root media-service wiring.' >&2
    exit 1
fi
rm "$FIXTURE/etc/init.d/S50pipewire"

mv "$FIXTURE/usr/bin/wireplumber" "$FIXTURE/usr/bin/wireplumber.real"
ln -s wireplumber.real "$FIXTURE/usr/bin/wireplumber"
if AQUA_AUDIO_ROOT="$FIXTURE" "$FIXTURE/usr/bin/aqua-audio-rootfs-check" >/dev/null 2>&1; then
    echo 'Rootfs contract accepted a symlinked service binary.' >&2
    exit 1
fi

echo 'Aqua Linux audio rootfs contract checks passed.'
