#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${TARGET_DIR:-${ROOT_DIR}/docs/aqua-linux/assets/runtime}"
MANIFEST="${TARGET_DIR}/manifest.json"

test -f "${MANIFEST}"
python3 - "${MANIFEST}" <<'PY'
import hashlib
import json
import os
import struct
import sys

manifest_path = sys.argv[1]
root = os.path.dirname(manifest_path)
data = json.load(open(manifest_path, encoding="utf-8"))
expected = {
    "qemu-desktop.png",
    "qemu-applications.png",
    "qemu-search.png",
    "qemu-first-party-windows.png",
}
images = data.get("images", [])
if {image.get("file") for image in images} != expected:
    raise SystemExit("public runtime manifest has an unexpected image set")
if data.get("environment") != "QEMU x86_64 TCG":
    raise SystemExit("public runtime captures must identify QEMU")
if data.get("captureScript") != "scripts/check-public-runtime-qemu.sh":
    raise SystemExit("public runtime capture provenance is missing")
for image in images:
    filename = image["file"]
    if "reference" in filename.lower():
        raise SystemExit("design reference cannot be published as runtime evidence")
    path = os.path.join(root, filename)
    payload = open(path, "rb").read()
    if payload[:8] != b"\x89PNG\r\n\x1a\n" or payload[12:16] != b"IHDR":
        raise SystemExit(f"invalid PNG: {filename}")
    width, height = struct.unpack(">II", payload[16:24])
    if (width, height) != (1280, 800):
        raise SystemExit(f"unexpected capture dimensions: {filename}")
    if image.get("width") != width or image.get("height") != height:
        raise SystemExit(f"manifest dimensions changed: {filename}")
    digest = hashlib.sha256(payload).hexdigest()
    if image.get("sha256") != digest:
        raise SystemExit(f"capture hash changed: {filename}")
print("Public QEMU runtime screenshot checks passed.")
PY
