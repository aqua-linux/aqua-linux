#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
CHECKLIST="${AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST:-${ROOT_DIR}/build/qemu-visible-operator-checklist.md}"

if [ ! -f "${CHECKLIST}" ]; then
    echo "Missing QEMU visible operator checklist: ${CHECKLIST}" >&2
    echo "Run scripts/write-qemu-visible-operator-checklist.sh first." >&2
    exit 1
fi

grep -Fq '# Aqua Linux QEMU Visible Operator Checklist' "${CHECKLIST}"
grep -Fq 'Status: `ready-for-operator-pass`' "${CHECKLIST}"
grep -Fq 'Packet status: `ready`' "${CHECKLIST}"
grep -Fq 'Confirm `boot_graphics=false`.' "${CHECKLIST}"
grep -Fq 'Confirm `autostart=false`.' "${CHECKLIST}"
grep -Fq 'persistent_graphical_session_started=false' "${CHECKLIST}"
grep -Fq 'Do not mark the VM display observed' "${CHECKLIST}"
grep -Fq 'Confirm `capture_hash_verification_required=true`.' "${CHECKLIST}"
grep -Fq 'Confirm `bundle_capture_hash_status=ok`.' "${CHECKLIST}"
grep -Fq 'Confirm `bundle_missing_capture_hash_rejected_status=ok`.' "${CHECKLIST}"
grep -Fq 'Confirm `manual_runbook_pass_report_required_status=ok`.' "${CHECKLIST}"
grep -Fq 'Confirm `pass_report_status=ok`.' "${CHECKLIST}"
grep -Fq '## Hash Gates' "${CHECKLIST}"
grep -Fq 'Capture hash verification required: `true`' "${CHECKLIST}"
grep -Fq 'Bundle capture hash status: `ok`' "${CHECKLIST}"
grep -Fq 'Positive bundle capture hash status: `ok`' "${CHECKLIST}"
grep -Fq 'Missing capture hash rejection status: `ok`' "${CHECKLIST}"
grep -Fq '## Pass Report Gates' "${CHECKLIST}"
grep -Fq 'Manual runbook pass report required status: `ok`' "${CHECKLIST}"
grep -Fq 'Pass report status: `ok`' "${CHECKLIST}"
grep -Fq 'Pass report evidence recorded status: `ok`' "${CHECKLIST}"
grep -Fq 'Pass report evidence rule status: `ok`' "${CHECKLIST}"
grep -Fq 'scripts/run-qemu-visible-manual.sh' "${CHECKLIST}"
grep -Fq 'scripts/run-qemu-visible-ready-capture-flow.sh' "${CHECKLIST}"
grep -Fq 'AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply' "${CHECKLIST}"
grep -Fq 'aqua-qemu-visible-pass-report' "${CHECKLIST}"
grep -Fq '## Artifact Fingerprints' "${CHECKLIST}"
grep -Fq '`operator_plan_json`' "${CHECKLIST}"
grep -Fq '`image_manifest_json`' "${CHECKLIST}"
grep -Fq 'SHA-256:' "${CHECKLIST}"
grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-checklist status=ok' "${CHECKLIST}"

echo "Aqua Linux QEMU visible operator checklist checks passed."
