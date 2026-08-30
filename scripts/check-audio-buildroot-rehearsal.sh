#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
DEFAULT_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_defconfig"
REHEARSAL_CONFIG="$ROOT_DIR/br2-external/aqua/configs/aqua_x86_64_audio_rehearsal_defconfig"
REHEARSAL_SCRIPT="$ROOT_DIR/scripts/rehearse-audio-buildroot-closure.sh"
REPORT_WRITER="$ROOT_DIR/scripts/write-audio-buildroot-closure.py"

test -f "$DEFAULT_CONFIG"
test -f "$REHEARSAL_CONFIG"
test -x "$REHEARSAL_SCRIPT"
test -x "$REPORT_WRITER"
sh -n "$REHEARSAL_SCRIPT"

for symbol in \
    BR2_PACKAGE_ALSA_LIB \
    BR2_PACKAGE_PIPEWIRE \
    BR2_PACKAGE_LUA_5_4 \
    BR2_PACKAGE_WIREPLUMBER \
    BR2_PACKAGE_AQUA_AUDIO_NATIVE \
    BR2_PACKAGE_AQUA_AUDIO_PROBE
do
    grep -Fxq "${symbol}=y" "$REHEARSAL_CONFIG"
    ! grep -Fxq "${symbol}=y" "$DEFAULT_CONFIG"
done

grep -Fxq \
    'BR2_PACKAGE_ALSA_LIB_PCM_PLUGINS="hw plug rate route softvol null ioplug"' \
    "$REHEARSAL_CONFIG"
grep -Fxq 'BR2_PACKAGE_ALSA_LIB_CTL_PLUGINS="hw ext"' "$REHEARSAL_CONFIG"

for symbol in \
    BR2_PACKAGE_DBUS \
    BR2_PACKAGE_BLUEZ5_UTILS \
    BR2_PACKAGE_JACK2 \
    BR2_PACKAGE_PULSEAUDIO \
    BR2_PACKAGE_FFMPEG \
    BR2_PACKAGE_AVAHI \
    BR2_PACKAGE_LIBCAMERA \
    BR2_PACKAGE_PIPEWIRE_GSTREAMER \
    BR2_PACKAGE_PIPEWIRE_V4L2
do
    grep -Fxq "${symbol}=n" "$REHEARSAL_CONFIG"
done

grep -Fq 'aqua_x86_64_audio_rehearsal_defconfig' "$REHEARSAL_SCRIPT"
grep -Fq 'aqua-audio-native' "$REHEARSAL_SCRIPT"
grep -Fq 'legal-info' "$REHEARSAL_SCRIPT"
grep -Fq 'show-info' "$REHEARSAL_SCRIPT"
grep -Fq 'default_image_changed' "$REPORT_WRITER"
grep -Fq 'release_cleared' "$REPORT_WRITER"
grep -Fq '"aqua-audio-native": "1"' "$REPORT_WRITER"
grep -Fq '"aqua-audio-probe": "6"' "$REPORT_WRITER"
grep -Fq 'BR2_PACKAGE_AQUA_AUDIO_NATIVE' "$REHEARSAL_SCRIPT"
grep -Fq 'BR2_PACKAGE_AQUA_AUDIO_PROBE' "$REHEARSAL_SCRIPT"
grep -Fxq \
    'BR2_ROOTFS_OVERLAY="$(BR2_EXTERNAL_AQUA_PATH)/rootfs-overlay $(BR2_EXTERNAL_AQUA_PATH)/audio-rootfs-overlay"' \
    "$REHEARSAL_CONFIG"

echo 'Aqua Linux audio Buildroot rehearsal checks passed.'
