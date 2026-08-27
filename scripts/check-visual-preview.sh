#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PREVIEW_DIR="${ROOT_DIR}/docs/aqua-linux/preview"

need_file() {
    if [ ! -f "$1" ]; then
        echo "Missing preview file: $1" >&2
        exit 1
    fi
}

need_text() {
    if ! grep -Fq -- "$2" "$1"; then
        echo "Missing expected text in $1: $2" >&2
        exit 1
    fi
}

need_file "${PREVIEW_DIR}/index.html"
need_file "${PREVIEW_DIR}/styles.css"
need_file "${ROOT_DIR}/docs/aqua-linux/assets/default-wallpaper.png"
need_file "${ROOT_DIR}/docs/aqua-linux/assets/aqua-logo-primary.png"
need_file "${ROOT_DIR}/docs/aqua-linux/assets/aqua-symbol-primary.png"
need_file "${ROOT_DIR}/docs/aqua-linux/assets/temp-icons/lucide/home.svg"
need_file "${ROOT_DIR}/docs/aqua-linux/assets/temp-icons/lucide/wifi.svg"

need_text "${PREVIEW_DIR}/index.html" "Mock system status"
need_text "${PREVIEW_DIR}/index.html" "Applications preview"
need_text "${PREVIEW_DIR}/index.html" "aqua-symbol-primary.png"
need_text "${PREVIEW_DIR}/styles.css" "grid-template-columns: 1fr auto 1fr"
need_text "${PREVIEW_DIR}/styles.css" "--surface"

echo "Aqua Linux visual preview checks passed."
