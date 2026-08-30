#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

AQUA_AUDIO_QEMU_CONTRACT=multi-route \
    AQUA_AUDIO_QEMU_EVIDENCE_DIR="${AQUA_AUDIO_QEMU_EVIDENCE_DIR:-${ROOT_DIR}/build/audio-multi-route-qemu}" \
    exec "${ROOT_DIR}/scripts/check-audio-qemu.sh"
