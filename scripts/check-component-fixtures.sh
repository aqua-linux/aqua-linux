#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EXPECTED="${ROOT_DIR}/docs/aqua-linux/component-fixtures.txt"
ACTUAL="$(mktemp)"
trap 'rm -f "${ACTUAL}"' EXIT HUP INT TERM

cargo run --quiet --manifest-path "${ROOT_DIR}/Cargo.toml" \
    -p aqua-renderer --example export-component-fixtures > "${ACTUAL}"
cmp "${EXPECTED}" "${ACTUAL}"

echo "Aqua Linux component fixture checks passed."
