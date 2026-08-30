#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EVIDENCE_DIR="${AQUA_AUDIO_QEMU_EVIDENCE_DIR:-${ROOT_DIR}/build/audio-signal-input-qemu}"
INJECTOR="${EVIDENCE_DIR}/qemu-dbus-audio-input"
SOURCE="${ROOT_DIR}/scripts/qemu-dbus-audio-input.c"
CC_BIN="${CC:-cc}"

mkdir -p "${EVIDENCE_DIR}"
if command -v pkg-config >/dev/null 2>&1; then
    # pkg-config intentionally expands into individual compiler arguments.
    # shellcheck disable=SC2046
    "${CC_BIN}" -std=c11 -Wall -Wextra -Werror -Wpedantic \
        "${SOURCE}" $(pkg-config --cflags --libs gio-unix-2.0) -o "${INJECTOR}"
elif command -v brew >/dev/null 2>&1; then
    GLIB_PREFIX="$(brew --prefix glib)"
    "${CC_BIN}" -std=c11 -Wall -Wextra -Werror -Wpedantic \
        -I"${GLIB_PREFIX}/include/glib-2.0" \
        -I"${GLIB_PREFIX}/lib/glib-2.0/include" \
        "${SOURCE}" -L"${GLIB_PREFIX}/lib" \
        -Wl,-rpath,"${GLIB_PREFIX}/lib" -lgio-2.0 -lgobject-2.0 -lglib-2.0 \
        -o "${INJECTOR}"
else
    echo 'Building the QEMU D-Bus input injector requires gio-unix-2.0 development files.' >&2
    exit 1
fi

AQUA_AUDIO_QEMU_CONTRACT="${AQUA_AUDIO_QEMU_CONTRACT:-input-signal}" \
    AQUA_AUDIO_QEMU_EVIDENCE_DIR="${EVIDENCE_DIR}" \
    AUDIO_INPUT_INJECTOR="${INJECTOR}" \
    exec "${ROOT_DIR}/scripts/check-audio-qemu.sh"
