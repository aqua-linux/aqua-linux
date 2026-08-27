#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
ICON_DIR="${ROOT_DIR}/docs/aqua-linux/assets/temp-icons/lucide"
BASE_URL="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons"

mkdir -p "${ICON_DIR}"

fetch_icon() {
    source_name="$1"
    target_name="$2"
    url="${BASE_URL}/${source_name}.svg"
    target="${ICON_DIR}/${target_name}.svg"

    curl -fsSL "${url}" -o "${target}"
    printf 'ready: %s\n' "${target}"
}

fetch_icon house home
fetch_icon folder files
fetch_icon hard-drive aqua-drive
fetch_icon trash-2 trash
fetch_icon globe browser
fetch_icon terminal terminal
fetch_icon settings settings
fetch_icon package software
fetch_icon wifi wifi
fetch_icon volume-2 volume
fetch_icon battery battery
fetch_icon bell notification
fetch_icon refresh-cw updates

curl -fsSL "https://raw.githubusercontent.com/lucide-icons/lucide/main/LICENSE" -o "${ICON_DIR}/LICENSE"

echo "Aqua Linux temporary Lucide icons fetched."
