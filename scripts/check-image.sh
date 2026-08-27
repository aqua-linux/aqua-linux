#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

cd "${ROOT_DIR}"

scripts/check-runtime-assets.sh
scripts/check-compositor-packaged.sh
scripts/check-compositor-rootfs-docker.sh
scripts/check-boot.sh
scripts/check-fbdev-presenter-qemu.sh
scripts/write-boot-summary.sh
scripts/check-boot-summary.sh
scripts/write-image-manifest.sh
scripts/preflight-qemu-visible-manual.sh >/dev/null
scripts/write-qemu-visible-preflight-summary.sh >/dev/null
scripts/check-qemu-visible-preflight-summary.sh
scripts/qemu-visible-status.sh >/dev/null
scripts/check-qemu-visible-status.sh
scripts/write-qemu-visible-operator-plan.sh >/dev/null
scripts/check-qemu-visible-operator-plan.sh
scripts/write-qemu-visible-operator-packet.sh >/dev/null
scripts/check-qemu-visible-operator-packet.sh
scripts/write-qemu-visible-operator-checklist.sh >/dev/null
scripts/check-qemu-visible-operator-checklist.sh
AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh >/dev/null
scripts/check-qemu-visible-operator-pass.sh
scripts/write-image-manifest.sh
scripts/check-image-manifest.sh
scripts/report-artifacts.sh

echo "Aqua Linux image checks passed."
