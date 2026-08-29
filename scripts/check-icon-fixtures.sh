#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EXPECTED="${ROOT_DIR}/docs/aqua-linux/icon-fixtures.txt"
ACTUAL="$(mktemp)"
trap 'rm -f "${ACTUAL}"' EXIT HUP INT TERM

cargo run --quiet -p aqua-renderer --example export-icon-fixtures > "${ACTUAL}"
cmp "${EXPECTED}" "${ACTUAL}"
grep -Fq 'revision=aqua-icon-fixtures-1' "${ACTUAL}"
grep -Fq 'summary roles=13 cases=1456' "${ACTUAL}"
grep -Fq 'parsed_sources=13' "${ACTUAL}"
grep -Fq 'ready=true' "${ACTUAL}"

echo "Aqua Linux icon fixture checks passed."
