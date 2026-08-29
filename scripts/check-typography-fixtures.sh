#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EXPECTED="${ROOT_DIR}/docs/aqua-linux/typography-fixtures.txt"
ACTUAL="$(mktemp)"
trap 'rm -f "${ACTUAL}"' EXIT HUP INT TERM

cargo run --quiet --manifest-path "${ROOT_DIR}/Cargo.toml" \
    -p aqua-text --example export-typography-fixtures > "${ACTUAL}"
cmp "${EXPECTED}" "${ACTUAL}"

echo "Aqua Linux typography fixture checks passed."
