#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
STATUS_FILE="${ROOT_DIR}/docs/aqua-linux/hardware-support.md"

test -s "${STATUS_FILE}"

for heading in \
    '## Status Meanings' \
    '## QEMU x86_64' \
    '## MSI Sword 17' \
    '## Claim Rules'; do
    grep -Fq "${heading}" "${STATUS_FILE}"
done

grep -Fq 'QEMU x86_64 is the only validated machine' "${STATUS_FILE}"
grep -Fq 'MSI Sword 17 is the planned physical validation target' "${STATUS_FILE}"
grep -Fq '| Network adapter | Present, unvalidated |' "${STATUS_FILE}"
grep -Fq '| Audio | Not tested |' "${STATUS_FILE}"
grep -Fq '| Suspend and resume | Deferred |' "${STATUS_FILE}"
grep -Fq 'No MSI Sword 17 hardware validation has started.' "${STATUS_FILE}"
grep -Fq 'installation and daily-use claims remain unsupported.' "${STATUS_FILE}"
grep -Fq 'Milestone 10 remains at 0%' "${STATUS_FILE}"

for evidence in \
    scripts/check-boot.sh \
    scripts/check-graphical-boot-qemu.sh \
    scripts/check-fbdev-presenter-qemu.sh \
    scripts/check-installer-transaction-qemu.sh \
    scripts/check-public-runtime-qemu.sh \
    scripts/check-terminal-qemu.sh \
    br2-external/aqua/board/aqua/x86_64/linux.config; do
    test -s "${ROOT_DIR}/${evidence}"
    grep -Fq "${evidence}" "${STATUS_FILE}"
done

test "$(jq -r '.phases[] | select(.id == "m10") | .percent' \
    "${ROOT_DIR}/docs/aqua-linux/progress.json")" = "0"
test "$(jq -r '.phases[] | select(.id == "m10") | .status' \
    "${ROOT_DIR}/docs/aqua-linux/progress.json")" = "not-started"

echo "Aqua Linux hardware support status checks passed."
