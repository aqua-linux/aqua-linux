#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
ASSET_DIR="${ROOT_DIR}/docs/aqua-linux/assets"
AQUA_ICON_DIR="${ASSET_DIR}/icons/aqua"

need_file() {
    if [ ! -f "$1" ]; then
        echo "Missing asset file: $1" >&2
        exit 1
    fi
}

need_text() {
    if ! grep -Fq "$2" "$1"; then
        echo "Missing expected text in $1: $2" >&2
        exit 1
    fi
}

need_file "${ASSET_DIR}/default-wallpaper.png"
need_file "${ASSET_DIR}/wallpaper-light.png"
need_file "${ASSET_DIR}/wallpaper-dark.png"
need_file "${ASSET_DIR}/wallpaper-pale-waves.png"
need_file "${ASSET_DIR}/wallpaper-surf.png"
need_file "${ASSET_DIR}/wallpaper-reef.png"
need_file "${ASSET_DIR}/wallpaper-sunlit-water.png"
need_file "${ASSET_DIR}/wallpaper-moonlit-lagoon.png"
need_file "${ASSET_DIR}/aqua-symbol-primary.png"
need_file "${ASSET_DIR}/aqua-symbol-inverse.png"
need_file "${ASSET_DIR}/aqua-symbol-accent.png"
need_file "${ASSET_DIR}/aqua-wordmark-primary.png"
need_file "${ASSET_DIR}/aqua-logo-primary.png"
need_file "${ASSET_DIR}/manifest.md"
need_file "${AQUA_ICON_DIR}/README.md"
need_file "${ASSET_DIR}/fonts/NotoSans-Regular.ttf"
need_file "${ASSET_DIR}/fonts/NotoSansArabic-Regular.ttf"
need_file "${ASSET_DIR}/fonts/OFL.txt"

need_file "${AQUA_ICON_DIR}/LICENSE"
need_file "${AQUA_ICON_DIR}/home.svg"
need_file "${AQUA_ICON_DIR}/files.svg"
need_file "${AQUA_ICON_DIR}/aqua-drive.svg"
need_file "${AQUA_ICON_DIR}/trash.svg"
need_file "${AQUA_ICON_DIR}/browser.svg"
need_file "${AQUA_ICON_DIR}/terminal.svg"
need_file "${AQUA_ICON_DIR}/settings.svg"
need_file "${AQUA_ICON_DIR}/software.svg"
need_file "${AQUA_ICON_DIR}/wifi.svg"
need_file "${AQUA_ICON_DIR}/volume.svg"
need_file "${AQUA_ICON_DIR}/battery.svg"
need_file "${AQUA_ICON_DIR}/notification.svg"
need_file "${AQUA_ICON_DIR}/updates.svg"

need_file "${ROOT_DIR}/docs/aqua-linux/design-tokens.json"
need_file "${ROOT_DIR}/docs/aqua-linux/icon-production.md"
need_file "${ROOT_DIR}/docs/aqua-linux/runtime-assets.md"
need_file "${ROOT_DIR}/THIRD_PARTY_LICENSES.md"

need_text "${ASSET_DIR}/manifest.md" "Approved Aqua Brand Exports"
need_text "${ASSET_DIR}/manifest.md" "Wi-Fi"
need_text "${ASSET_DIR}/manifest.md" "Aqua Core Icons"
need_text "${ASSET_DIR}/manifest.md" "NotoSansArabic-Regular.ttf"
need_text "${ASSET_DIR}/manifest.md" "wallpaper-sunlit-water.png"
need_text "${ASSET_DIR}/manifest.md" "wallpaper-pale-waves.png"
need_text "${ASSET_DIR}/manifest.md" "wallpaper-light.png"
need_text "${ASSET_DIR}/manifest.md" "wallpaper-dark.png"
need_text "${ASSET_DIR}/manifest.md" "wallpaper-moonlit-lagoon.png"
need_text "${ASSET_DIR}/manifest.md" "Private Interface References"
need_text "${ASSET_DIR}/manifest.md" "Do not commit or package private boards"
need_text "${ROOT_DIR}/docs/aqua-linux/design-tokens.json" "\"product\": \"Aqua Linux\""
need_text "${ROOT_DIR}/docs/aqua-linux/design-tokens.json" "\"defaultTheme\": \"Light\""
need_text "${ROOT_DIR}/docs/aqua-linux/design-tokens.json" "\"Light\""
need_text "${ROOT_DIR}/docs/aqua-linux/design-tokens.json" "\"Dark\""
need_text "${ROOT_DIR}/docs/aqua-linux/design-tokens.json" "\"blurRequired\": false"
need_text "${ROOT_DIR}/docs/aqua-linux/icon-production.md" "## Delivery 1: Shell And Window Essentials"
need_text "${ROOT_DIR}/docs/aqua-linux/icon-production.md" '`applications.svg`'
need_text "${ROOT_DIR}/docs/aqua-linux/icon-production.md" '`window-close.svg`'
need_text "${ROOT_DIR}/docs/aqua-linux/icon-production.md" "Aqua does not derive, trace, recolor, or redistribute icons"
need_text "${ROOT_DIR}/docs/aqua-linux/license-audit.md" "elementary Icons is not a source dependency or runtime asset"
need_text "${ROOT_DIR}/docs/aqua-linux/runtime-assets.md" "/usr/share/aqua/"
need_text "${AQUA_ICON_DIR}/README.md" "project-authored"

echo "Aqua Linux asset checks passed."
