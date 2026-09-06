#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SOURCE_DIR="${SOURCE_DIR:-${ROOT_DIR}/build/qemu-public-runtime}"
TARGET_DIR="${TARGET_DIR:-${ROOT_DIR}/docs/aqua-linux/assets/runtime}"
SERIAL_LOG="${SERIAL_LOG:-${SOURCE_DIR}/serial.log}"
MARKER='[AQUA-TEST] stage=desktop-public-runtime-qemu status=ok captures=desktop,applications,search,search-empty,applications-dark,search-dark,search-empty-dark,windows themes=Light,Dark clients=files,settings'

grep -Fq "${MARKER}" "${SERIAL_LOG}" || {
    echo "Missing successful public runtime QEMU marker: ${SERIAL_LOG}" >&2
    exit 1
}

mkdir -p "${TARGET_DIR}"
for name in desktop applications search search-empty applications-dark search-dark search-empty-dark windows; do
    test -s "${SOURCE_DIR}/${name}.png" || {
        echo "Missing validated QEMU capture: ${SOURCE_DIR}/${name}.png" >&2
        exit 1
    }
done

cp "${SOURCE_DIR}/desktop.png" "${TARGET_DIR}/qemu-desktop.png"
cp "${SOURCE_DIR}/applications.png" "${TARGET_DIR}/qemu-applications.png"
cp "${SOURCE_DIR}/search.png" "${TARGET_DIR}/qemu-search.png"
cp "${SOURCE_DIR}/search-empty.png" "${TARGET_DIR}/qemu-search-empty.png"
cp "${SOURCE_DIR}/applications-dark.png" "${TARGET_DIR}/qemu-applications-dark.png"
cp "${SOURCE_DIR}/search-dark.png" "${TARGET_DIR}/qemu-search-dark.png"
cp "${SOURCE_DIR}/search-empty-dark.png" "${TARGET_DIR}/qemu-search-empty-dark.png"
cp "${SOURCE_DIR}/windows.png" "${TARGET_DIR}/qemu-first-party-windows.png"

SOURCE_REVISION="$(git -C "${ROOT_DIR}" rev-parse HEAD)"
export TARGET_DIR SOURCE_REVISION MARKER
python3 - <<'PY'
import hashlib
import json
import os
import struct

target = os.environ["TARGET_DIR"]
views = [
    ("qemu-desktop.png", "Clean desktop"),
    ("qemu-applications.png", "Applications"),
    ("qemu-search.png", "Global Search query: set"),
    ("qemu-search-empty.png", "Global Search empty state: setxyz"),
    ("qemu-applications-dark.png", "Applications, Dark theme"),
    ("qemu-search-dark.png", "Global Search query: set, Dark theme"),
    ("qemu-search-empty-dark.png", "Global Search empty state: setxyz, Dark theme"),
    ("qemu-first-party-windows.png", "Aqua Files and Aqua Settings"),
]
images = []
for filename, view in views:
    path = os.path.join(target, filename)
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise SystemExit(f"invalid PNG: {path}")
    width, height = struct.unpack(">II", data[16:24])
    images.append({
        "file": filename,
        "view": view,
        "width": width,
        "height": height,
        "sha256": hashlib.sha256(data).hexdigest(),
    })

manifest = {
    "schemaVersion": 1,
    "product": "Aqua Linux",
    "environment": "QEMU x86_64 TCG",
    "captureScript": "scripts/check-public-runtime-qemu.sh",
    "publishScript": "scripts/publish-public-runtime-screenshots.sh",
    "validationMarker": os.environ["MARKER"],
    "runtimeSourceRevision": os.environ["SOURCE_REVISION"],
    "images": images,
}
with open(os.path.join(target, "manifest.json"), "w", encoding="utf-8") as handle:
    json.dump(manifest, handle, indent=2)
    handle.write("\n")
PY

"${ROOT_DIR}/scripts/check-public-runtime-screenshots.sh"
echo "Public QEMU runtime screenshots published: ${TARGET_DIR}"
