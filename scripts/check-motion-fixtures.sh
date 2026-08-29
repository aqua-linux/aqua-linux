#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EXPECTED="${ROOT_DIR}/docs/aqua-linux/motion-fixtures.txt"
ACTUAL="$(mktemp)"
trap 'rm -f "${ACTUAL}"' EXIT HUP INT TERM

cargo run --quiet -p aqua-shell --example export-motion-fixtures > "${ACTUAL}"
diff -u "${EXPECTED}" "${ACTUAL}"
grep -Fq 'continuous=true' "${ACTUAL}"
grep -Fq 'reduced active=false launcher_opacity=1.0 launcher_offset_y=0 menu_offset_y=0 notification_offset_x=0 repeating_allowed=false state_feedback=true' "${ACTUAL}"

echo "Aqua Linux motion fixture checks passed."
