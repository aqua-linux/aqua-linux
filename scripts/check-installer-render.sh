#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
OUTPUT="${INSTALLER_RENDER_OUTPUT:-${ROOT_DIR}/build/installer-welcome.png}"
LOG="${INSTALLER_RENDER_LOG:-${ROOT_DIR}/build/installer-welcome.log}"
LANGUAGE_OUTPUT="${INSTALLER_LANGUAGE_RENDER_OUTPUT:-${ROOT_DIR}/build/installer-language.png}"
LANGUAGE_LOG="${INSTALLER_LANGUAGE_RENDER_LOG:-${ROOT_DIR}/build/installer-language.log}"
KEYBOARD_OUTPUT="${INSTALLER_KEYBOARD_RENDER_OUTPUT:-${ROOT_DIR}/build/installer-keyboard.png}"
KEYBOARD_LOG="${INSTALLER_KEYBOARD_RENDER_LOG:-${ROOT_DIR}/build/installer-keyboard.log}"
PARTITIONS_OUTPUT="${INSTALLER_PARTITIONS_RENDER_OUTPUT:-${ROOT_DIR}/build/installer-partitions.png}"
PARTITIONS_LOG="${INSTALLER_PARTITIONS_RENDER_LOG:-${ROOT_DIR}/build/installer-partitions.log}"
TIMEZONE_OUTPUT="${INSTALLER_TIMEZONE_RENDER_OUTPUT:-${ROOT_DIR}/build/installer-timezone.png}"
TIMEZONE_LOG="${INSTALLER_TIMEZONE_RENDER_LOG:-${ROOT_DIR}/build/installer-timezone.log}"
USER_OUTPUT="${INSTALLER_USER_RENDER_OUTPUT:-${ROOT_DIR}/build/installer-user.png}"
USER_LOG="${INSTALLER_USER_RENDER_LOG:-${ROOT_DIR}/build/installer-user.log}"
SUMMARY_OUTPUT="${INSTALLER_SUMMARY_RENDER_OUTPUT:-${ROOT_DIR}/build/installer-summary.png}"
SUMMARY_LOG="${INSTALLER_SUMMARY_RENDER_LOG:-${ROOT_DIR}/build/installer-summary.log}"

mkdir -p "$(dirname "${OUTPUT}")" "$(dirname "${LOG}")" \
    "$(dirname "${LANGUAGE_OUTPUT}")" "$(dirname "${LANGUAGE_LOG}")" \
    "$(dirname "${KEYBOARD_OUTPUT}")" "$(dirname "${KEYBOARD_LOG}")" \
    "$(dirname "${PARTITIONS_OUTPUT}")" "$(dirname "${PARTITIONS_LOG}")" \
    "$(dirname "${TIMEZONE_OUTPUT}")" "$(dirname "${TIMEZONE_LOG}")" \
    "$(dirname "${USER_OUTPUT}")" "$(dirname "${USER_LOG}")" \
    "$(dirname "${SUMMARY_OUTPUT}")" "$(dirname "${SUMMARY_LOG}")"
cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p aqua-renderer \
    --example export-installer -- "${OUTPUT}" welcome > "${LOG}"
cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p aqua-renderer \
    --example export-installer -- "${LANGUAGE_OUTPUT}" language > "${LANGUAGE_LOG}"
cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p aqua-renderer \
    --example export-installer -- "${KEYBOARD_OUTPUT}" keyboard > "${KEYBOARD_LOG}"
cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p aqua-renderer \
    --example export-installer -- "${PARTITIONS_OUTPUT}" partitions > "${PARTITIONS_LOG}"
cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p aqua-renderer \
    --example export-installer -- "${TIMEZONE_OUTPUT}" time-zone > "${TIMEZONE_LOG}"
cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p aqua-renderer \
    --example export-installer -- "${USER_OUTPUT}" user-information > "${USER_LOG}"
cargo run --manifest-path "${ROOT_DIR}/Cargo.toml" -p aqua-renderer \
    --example export-installer -- "${SUMMARY_OUTPUT}" summary > "${SUMMARY_LOG}"

grep -Fq 'installer_rendered=true' "${LOG}"
grep -Fq 'installer_layout_valid=true' "${LOG}"
grep -Fq 'installer_step=welcome' "${LOG}"
grep -Fq 'installer_focus=language-control' "${LOG}"
grep -Fq 'installer_logo_rendered=true' "${LOG}"
grep -Eq '^installer_checksum=[0-9a-f]{16}$' "${LOG}"
file "${OUTPUT}" | grep -Fq 'PNG image data, 1280 x 800, 8-bit/color RGBA'
test "$(wc -c < "${OUTPUT}" | tr -d ' ')" -gt 100000

grep -Fq 'installer_rendered=true' "${LANGUAGE_LOG}"
grep -Fq 'installer_layout_valid=true' "${LANGUAGE_LOG}"
grep -Fq 'installer_step=language' "${LANGUAGE_LOG}"
grep -Fq 'installer_logo_rendered=false' "${LANGUAGE_LOG}"
grep -Eq '^installer_checksum=[0-9a-f]{16}$' "${LANGUAGE_LOG}"
file "${LANGUAGE_OUTPUT}" | grep -Fq 'PNG image data, 1280 x 800, 8-bit/color RGBA'
test "$(wc -c < "${LANGUAGE_OUTPUT}" | tr -d ' ')" -gt 100000

grep -Fq 'installer_rendered=true' "${KEYBOARD_LOG}"
grep -Fq 'installer_layout_valid=true' "${KEYBOARD_LOG}"
grep -Fq 'installer_step=keyboard' "${KEYBOARD_LOG}"
grep -Fq 'installer_logo_rendered=false' "${KEYBOARD_LOG}"
grep -Eq '^installer_checksum=[0-9a-f]{16}$' "${KEYBOARD_LOG}"
file "${KEYBOARD_OUTPUT}" | grep -Fq 'PNG image data, 1280 x 800, 8-bit/color RGBA'
test "$(wc -c < "${KEYBOARD_OUTPUT}" | tr -d ' ')" -gt 100000

grep -Fq 'installer_rendered=true' "${PARTITIONS_LOG}"
grep -Fq 'installer_layout_valid=true' "${PARTITIONS_LOG}"
grep -Fq 'installer_step=partitions' "${PARTITIONS_LOG}"
grep -Fq 'installer_logo_rendered=false' "${PARTITIONS_LOG}"
grep -Eq '^installer_checksum=[0-9a-f]{16}$' "${PARTITIONS_LOG}"
file "${PARTITIONS_OUTPUT}" | grep -Fq 'PNG image data, 1280 x 800, 8-bit/color RGBA'
test "$(wc -c < "${PARTITIONS_OUTPUT}" | tr -d ' ')" -gt 100000

grep -Fq 'installer_rendered=true' "${TIMEZONE_LOG}"
grep -Fq 'installer_layout_valid=true' "${TIMEZONE_LOG}"
grep -Fq 'installer_step=time-zone' "${TIMEZONE_LOG}"
grep -Fq 'installer_logo_rendered=false' "${TIMEZONE_LOG}"
grep -Eq '^installer_checksum=[0-9a-f]{16}$' "${TIMEZONE_LOG}"
file "${TIMEZONE_OUTPUT}" | grep -Fq 'PNG image data, 1280 x 800, 8-bit/color RGBA'
test "$(wc -c < "${TIMEZONE_OUTPUT}" | tr -d ' ')" -gt 100000

grep -Fq 'installer_rendered=true' "${USER_LOG}"
grep -Fq 'installer_layout_valid=true' "${USER_LOG}"
grep -Fq 'installer_step=user-information' "${USER_LOG}"
grep -Fq 'installer_logo_rendered=false' "${USER_LOG}"
grep -Eq '^installer_checksum=[0-9a-f]{16}$' "${USER_LOG}"
file "${USER_OUTPUT}" | grep -Fq 'PNG image data, 1280 x 800, 8-bit/color RGBA'
test "$(wc -c < "${USER_OUTPUT}" | tr -d ' ')" -gt 100000

grep -Fq 'installer_rendered=true' "${SUMMARY_LOG}"
grep -Fq 'installer_layout_valid=true' "${SUMMARY_LOG}"
grep -Fq 'installer_step=summary' "${SUMMARY_LOG}"
grep -Fq 'installer_logo_rendered=false' "${SUMMARY_LOG}"
grep -Eq '^installer_checksum=[0-9a-f]{16}$' "${SUMMARY_LOG}"
file "${SUMMARY_OUTPUT}" | grep -Fq 'PNG image data, 1280 x 800, 8-bit/color RGBA'
test "$(wc -c < "${SUMMARY_OUTPUT}" | tr -d ' ')" -gt 100000

echo "Aqua installer raster render check passed."
echo "PNGs: ${OUTPUT} ${LANGUAGE_OUTPUT} ${KEYBOARD_OUTPUT} ${PARTITIONS_OUTPUT} ${TIMEZONE_OUTPUT} ${USER_OUTPUT} ${SUMMARY_OUTPUT}"
