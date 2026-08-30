#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
AQUA_AUDIO_QEMU_CONTRACT=nondefault-unplug \
    AQUA_AUDIO_QEMU_EVIDENCE_DIR="${AQUA_AUDIO_QEMU_EVIDENCE_DIR:-${ROOT_DIR}/build/audio-nondefault-unplug-qemu}" \
    exec "${ROOT_DIR}/scripts/check-audio-qemu.sh" "$@"
