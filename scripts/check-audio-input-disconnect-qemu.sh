#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

AQUA_AUDIO_QEMU_CONTRACT=input-disconnect \
    AQUA_AUDIO_QEMU_EVIDENCE_DIR="${AQUA_AUDIO_QEMU_EVIDENCE_DIR:-${ROOT_DIR}/build/audio-input-disconnect-qemu}" \
    AUDIO_INPUT_DISCONNECT_BYTES="${AUDIO_INPUT_DISCONNECT_BYTES:-9600}" \
    exec "${ROOT_DIR}/scripts/check-audio-signal-input-qemu.sh"
