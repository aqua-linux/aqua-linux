#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
OUT_DIR="${AQUA_QEMU_CAPTURE_DIR:-${ROOT_DIR}/build/qemu-visible-captures}"
CAPTURE_ID="${AQUA_QEMU_VM_DISPLAY_CAPTURE_ID:-qemu-visible-manual-$(date -u +%Y%m%dT%H%M%SZ)}"
CAPTURE_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE:-${OUT_DIR}/${CAPTURE_ID}.png}"
META_FILE="${AQUA_QEMU_VM_DISPLAY_CAPTURE_META:-${CAPTURE_FILE}.env}"
PRINT_ONLY="${AQUA_QEMU_CAPTURE_PRINT_ONLY:-false}"

echo "Aqua Linux manual QEMU VM-display capture"
echo "product=Aqua Linux"
echo "mode=host-manual-capture"
echo "target=QEMU x86_64"
echo "capture_id=${CAPTURE_ID}"
echo "capture_file=${CAPTURE_FILE}"
echo "metadata_file=${META_FILE}"
echo "operator_controlled=true"
echo "autostart=false"
echo "boot_graphics=false"
echo "readiness_watch_command=scripts/watch-qemu-visible-readiness.sh"

if [ "${PRINT_ONLY}" = "true" ]; then
    echo "capture_command_ready=true"
    echo "evidence_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${CAPTURE_ID} AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${CAPTURE_FILE} aqua-qemu-visible-evidence-record"
    echo "[AQUA-HOST] stage=qemu-visible-manual-capture status=print-only"
    exit 0
fi

mkdir -p "$(dirname "${CAPTURE_FILE}")"

capture_tool=""
if command -v screencapture >/dev/null 2>&1; then
    capture_tool="screencapture"
    screencapture -x "${CAPTURE_FILE}"
elif command -v grim >/dev/null 2>&1; then
    capture_tool="grim"
    grim "${CAPTURE_FILE}"
elif command -v gnome-screenshot >/dev/null 2>&1; then
    capture_tool="gnome-screenshot"
    gnome-screenshot -f "${CAPTURE_FILE}"
elif command -v spectacle >/dev/null 2>&1; then
    capture_tool="spectacle"
    spectacle -b -n -o "${CAPTURE_FILE}"
elif command -v import >/dev/null 2>&1; then
    capture_tool="import"
    import -window root "${CAPTURE_FILE}"
else
    echo "capture_tool=missing" >&2
    echo "Install or provide one of: screencapture, grim, gnome-screenshot, spectacle, import." >&2
    exit 1
fi

if [ ! -s "${CAPTURE_FILE}" ]; then
    echo "capture_file_status=missing-or-empty" >&2
    exit 1
fi

capture_sha256="unavailable"
if command -v shasum >/dev/null 2>&1; then
    capture_sha256="$(shasum -a 256 "${CAPTURE_FILE}" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
    capture_sha256="$(sha256sum "${CAPTURE_FILE}" | awk '{print $1}')"
fi

{
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${CAPTURE_ID}"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${CAPTURE_FILE}"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256=${capture_sha256}"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_TOOL=${capture_tool}"
} > "${META_FILE}"

echo "capture_tool=${capture_tool}"
echo "capture_file_status=ready"
echo "capture_sha256=${capture_sha256}"
echo "metadata_written=ok"
echo "evidence_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=${CAPTURE_ID} AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${CAPTURE_FILE} aqua-qemu-visible-evidence-record"
echo "verify_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_META=${META_FILE} scripts/verify-qemu-visible-capture.sh"
echo "bundle_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_META=${META_FILE} scripts/write-qemu-visible-evidence-bundle.sh"
echo "apply_prep_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_META=${META_FILE} scripts/prepare-qemu-visible-evidence-apply.sh"
echo "flow_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_META=${META_FILE} scripts/run-qemu-visible-evidence-flow.sh"
echo "[AQUA-HOST] stage=qemu-visible-manual-capture status=ok"
