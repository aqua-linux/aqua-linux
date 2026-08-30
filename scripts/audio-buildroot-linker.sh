#!/usr/bin/env sh
set -eu

: "${AQUA_AUDIO_BUILDROOT_LINKER:?AQUA_AUDIO_BUILDROOT_LINKER is required}"
if [ ! -x "${AQUA_AUDIO_BUILDROOT_LINKER}" ]; then
    ls -ld /work /work/build /work/build/audio-rehearsal-output/host/bin >&2 || true
    ls -l "${AQUA_AUDIO_BUILDROOT_LINKER}" >&2 || true
    exit 127
fi
exec "${AQUA_AUDIO_BUILDROOT_LINKER}" "$@"
