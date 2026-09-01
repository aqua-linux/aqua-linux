#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

CHECK_TEMP_ROOT="$(mktemp -d)"
CHECK_COMMAND_CACHE="${CHECK_TEMP_ROOT}/command-cache"
mkdir -p "${CHECK_COMMAND_CACHE}"
trap 'rm -rf "${CHECK_TEMP_ROOT}"' EXIT HUP INT TERM

check_output_contains() {
    expected="$1"
    shift
    cache_key="$(printf '%s\n' "$@" | cksum | awk '{print $1}')"
    cache_file="${CHECK_COMMAND_CACHE}/${cache_key}.txt"
    if [ ! -f "${cache_file}" ]; then
        "$@" > "${cache_file}"
    fi
    output="$(cat "${cache_file}")"
    printf '%s\n' "${output}" | grep -Fq "${expected}"
}

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
scripts/check-typography-fixtures.sh
scripts/check-typography-layout-fixtures.sh
scripts/check-elevation-fixtures.sh
scripts/check-icon-fixtures.sh
scripts/check-motion-fixtures.sh
scripts/check-component-fixtures.sh
scripts/check-unprivileged-session.sh
scripts/check-media-service-supervisor.sh
scripts/check-network-service-supervisor.sh
scripts/check-network-service-boot.sh
scripts/check-wifi-service-supervisor.sh
scripts/check-wifi-service-boot.sh
scripts/check-graphical-session-supervisor.sh
scripts/check-default-recovery-safety.sh
scripts/check-graphical-session-stop.sh
scripts/check-contributor-workflow.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-manual-preflight status=print-only' env AQUA_QEMU_VISIBLE_PREFLIGHT_PRINT_ONLY=true scripts/preflight-qemu-visible-manual.sh
check_output_contains 'preflight_ready=true' env AQUA_QEMU_VISIBLE_PREFLIGHT_PRINT_ONLY=true scripts/preflight-qemu-visible-manual.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-readiness-watch status=print-only' env AQUA_QEMU_VISIBLE_WATCH_PRINT_ONLY=true scripts/watch-qemu-visible-readiness.sh
check_output_contains 'readiness_watch_ready=true' env AQUA_QEMU_VISIBLE_WATCH_PRINT_ONLY=true scripts/watch-qemu-visible-readiness.sh
preflight_fixture="${CHECK_TEMP_ROOT}/qemu-visible-manual-preflight.txt"
preflight_output="$(AQUA_QEMU_VISIBLE_PREFLIGHT_FILE="${preflight_fixture}" scripts/preflight-qemu-visible-manual.sh)"
printf '%s\n' "${preflight_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-manual-preflight status=ok'
printf '%s\n' "${preflight_output}" | grep -Fq 'preflight_written=ok'
printf '%s\n' "${preflight_output}" | grep -Fq 'host_scripts_ready=true'
printf '%s\n' "${preflight_output}" | grep -Fq 'summary_command='
grep -Fq 'preflight_status=ready' "${preflight_fixture}"
grep -Fq 'safe_to_launch_manual_qemu=true' "${preflight_fixture}"
grep -Fq 'host_ready_capture_flow_script=present' "${preflight_fixture}"
grep -Fq 'host_evidence_flow_script=present' "${preflight_fixture}"
preflight_summary_fixture="${CHECK_TEMP_ROOT}/qemu-visible-manual-preflight.json"
AQUA_QEMU_VISIBLE_PREFLIGHT_FILE="${preflight_fixture}" AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" scripts/write-qemu-visible-preflight-summary.sh >/dev/null
AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" scripts/check-qemu-visible-preflight-summary.sh >/dev/null
python3 - "${preflight_summary_fixture}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    summary = json.load(handle)

entry = summary["preflight_file_entry"]
assert entry["bytes"] > 0
assert len(entry["sha256"]) == 64
assert entry["mtime_utc"].endswith("Z")
PY
readiness_fixture="${CHECK_TEMP_ROOT}/qemu-visible-manual-serial.log"
cat > "${readiness_fixture}" <<'EOF'
[AQUA-BOOT] stage=session-check status=ok no_graphics=true
[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh
EOF
readiness_output="$(SERIAL_LOG="${readiness_fixture}" AQUA_QEMU_VISIBLE_WATCH_TIMEOUT=0 scripts/watch-qemu-visible-readiness.sh)"
printf '%s\n' "${readiness_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-readiness-watch status=ok'
printf '%s\n' "${readiness_output}" | grep -Fq 'qemu_visible_serial_ready=true'
printf '%s\n' "${readiness_output}" | grep -Fq 'session_check_seen=true'
check_output_contains '[AQUA-HOST] stage=qemu-visible-manual-runbook status=print-only' env AQUA_QEMU_MANUAL_PRINT_ONLY=true scripts/run-qemu-visible-manual.sh
check_output_contains 'Host entrypoint:' env AQUA_QEMU_MANUAL_PRINT_ONLY=true scripts/run-qemu-visible-manual.sh
check_output_contains 'scripts/preflight-qemu-visible-manual.sh' env AQUA_QEMU_MANUAL_PRINT_ONLY=true scripts/run-qemu-visible-manual.sh
check_output_contains 'scripts/watch-qemu-visible-readiness.sh' env AQUA_QEMU_MANUAL_PRINT_ONLY=true scripts/run-qemu-visible-manual.sh
check_output_contains 'AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=<capture-id> AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=<capture-file> aqua-qemu-visible-evidence-record' env AQUA_QEMU_MANUAL_PRINT_ONLY=true scripts/run-qemu-visible-manual.sh
check_output_contains 'AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker' env AQUA_QEMU_MANUAL_PRINT_ONLY=true scripts/run-qemu-visible-manual.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-manual-capture status=print-only' env AQUA_QEMU_CAPTURE_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/capture-qemu-visible-manual.sh
check_output_contains 'capture_command_ready=true' env AQUA_QEMU_CAPTURE_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/capture-qemu-visible-manual.sh
check_output_contains 'evidence_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture' env AQUA_QEMU_CAPTURE_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/capture-qemu-visible-manual.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-capture-verify status=print-only' env AQUA_QEMU_CAPTURE_VERIFY_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/verify-qemu-visible-capture.sh
check_output_contains 'capture_verify_ready=true' env AQUA_QEMU_CAPTURE_VERIFY_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/verify-qemu-visible-capture.sh
capture_verify_fixture="${CHECK_TEMP_ROOT}/qemu-visible-capture.png"
capture_verify_meta="${CHECK_TEMP_ROOT}/qemu-visible-capture.env"
printf 'aqua-capture-fixture\n' > "${capture_verify_fixture}"
if command -v shasum >/dev/null 2>&1; then
    capture_verify_sha="$(shasum -a 256 "${capture_verify_fixture}" | awk '{print $1}')"
else
    capture_verify_sha="$(sha256sum "${capture_verify_fixture}" | awk '{print $1}')"
fi
{
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${capture_verify_fixture}"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256=${capture_verify_sha}"
} > "${capture_verify_meta}"
capture_verify_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${capture_verify_meta}" scripts/verify-qemu-visible-capture.sh)"
printf '%s\n' "${capture_verify_output}" | grep -Fq 'capture_hash_verified=true'
{
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${capture_verify_fixture}"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256=0000000000000000000000000000000000000000000000000000000000000000"
} > "${capture_verify_meta}.bad"
set +e
capture_verify_bad_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${capture_verify_meta}.bad" scripts/verify-qemu-visible-capture.sh 2>&1)"
capture_verify_bad_status="$?"
set -e
test "${capture_verify_bad_status}" -ne 0
printf '%s\n' "${capture_verify_bad_output}" | grep -Fq 'capture_hash_status=mismatch'
check_output_contains '[AQUA-HOST] stage=qemu-visible-evidence-bundle status=print-only' env AQUA_QEMU_EVIDENCE_BUNDLE_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/write-qemu-visible-evidence-bundle.sh
check_output_contains 'evidence_bundle_ready=true' env AQUA_QEMU_EVIDENCE_BUNDLE_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/write-qemu-visible-evidence-bundle.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-evidence-apply-prep status=print-only' env AQUA_QEMU_EVIDENCE_APPLY_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/prepare-qemu-visible-evidence-apply.sh
check_output_contains 'apply_prep_ready=true' env AQUA_QEMU_EVIDENCE_APPLY_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/prepare-qemu-visible-evidence-apply.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-evidence-flow status=print-only' env AQUA_QEMU_EVIDENCE_FLOW_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/run-qemu-visible-evidence-flow.sh
check_output_contains 'flow_step_4=scripts/prepare-qemu-visible-evidence-apply.sh' env AQUA_QEMU_EVIDENCE_FLOW_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/run-qemu-visible-evidence-flow.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-ready-capture-flow status=print-only' env AQUA_QEMU_READY_CAPTURE_FLOW_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/run-qemu-visible-ready-capture-flow.sh
check_output_contains 'ready_capture_flow_ready=true' env AQUA_QEMU_READY_CAPTURE_FLOW_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/run-qemu-visible-ready-capture-flow.sh
check_output_contains 'capture_hash_verification_required=true' env AQUA_QEMU_READY_CAPTURE_FLOW_PRINT_ONLY=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=check-capture scripts/run-qemu-visible-ready-capture-flow.sh
check_output_contains '[AQUA-HOST] stage=qemu-visible-operator-pass status=print-only' env AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY=true scripts/run-qemu-visible-operator-pass.sh
check_output_contains 'operator_pass_ready=true' env AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY=true scripts/run-qemu-visible-operator-pass.sh
check_output_contains 'operator_pass_launch_armed=false' env AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY=true scripts/run-qemu-visible-operator-pass.sh
check_output_contains 'next_launch_command=scripts/run-qemu-visible-manual.sh' env AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY=true scripts/run-qemu-visible-operator-pass.sh
check_output_contains 'launch_confirmation_required=true' env AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY=true scripts/run-qemu-visible-operator-pass.sh
check_output_contains 'confirmed_launch_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh' env AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY=true scripts/run-qemu-visible-operator-pass.sh
check_output_contains 'operator_pass_stop_rule=Do not mark VM display observed' env AQUA_QEMU_VISIBLE_OPERATOR_PASS_PRINT_ONLY=true scripts/run-qemu-visible-operator-pass.sh
qemu_visible_operator_pass_rehearsal_output="$(AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" AQUA_QEMU_VISIBLE_STATUS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass-status.txt" AQUA_QEMU_VISIBLE_STATUS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass-status.json" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass-plan.txt" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass-plan.json" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass-packet.txt" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass-packet.json" AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST="${CHECK_TEMP_ROOT}/qemu-visible-pass-checklist.md" AQUA_QEMU_VISIBLE_OPERATOR_PASS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass.txt" AQUA_QEMU_VISIBLE_OPERATOR_PASS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass.json" AQUA_BOOT_SUMMARY_JSON="${ROOT_DIR:-$(pwd)}/build/aqua-boot-summary.json" AQUA_IMAGE_MANIFEST_JSON="${ROOT_DIR:-$(pwd)}/build/aqua-image-manifest.json" AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh)"
printf '%s\n' "${qemu_visible_operator_pass_rehearsal_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-pass status=no-launch-ready'
printf '%s\n' "${qemu_visible_operator_pass_rehearsal_output}" | grep -Fq 'operator_pass_launch_armed=true'
printf '%s\n' "${qemu_visible_operator_pass_rehearsal_output}" | grep -Fq 'operator_pass_launch_skipped=true'
printf '%s\n' "${qemu_visible_operator_pass_rehearsal_output}" | grep -Fq 'operator_pass_file='
printf '%s\n' "${qemu_visible_operator_pass_rehearsal_output}" | grep -Fq 'operator_pass_json='
grep -Fq 'no_positive_observation_without_evidence=true' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'capture_hash_verification_required=true' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'bundle_capture_hash_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'bundle_positive_capture_hash_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'bundle_missing_capture_hash_rejected_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'manual_runbook_pass_report_required_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'pass_report_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'pass_report_evidence_recorded_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'confirmed_launch_command=AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH=true scripts/run-qemu-visible-operator-pass.sh' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'operator_pass_stop_rule=Do not mark VM display observed' "${CHECK_TEMP_ROOT}/qemu-visible-pass.txt"
grep -Fq 'Status: `ready-for-operator-pass`' "${CHECK_TEMP_ROOT}/qemu-visible-pass-checklist.md"
AQUA_QEMU_VISIBLE_OPERATOR_PASS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass.txt" AQUA_QEMU_VISIBLE_OPERATOR_PASS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass.json" scripts/check-qemu-visible-operator-pass.sh >/dev/null
set +e
qemu_visible_operator_pass_blocked_output="$(AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" AQUA_QEMU_VISIBLE_STATUS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked-status.txt" AQUA_QEMU_VISIBLE_STATUS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked-status.json" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked-plan.txt" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked-plan.json" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked-packet.txt" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked-packet.json" AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked-checklist.md" AQUA_QEMU_VISIBLE_OPERATOR_PASS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt" AQUA_QEMU_VISIBLE_OPERATOR_PASS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.json" AQUA_BOOT_SUMMARY_JSON="${ROOT_DIR:-$(pwd)}/build/aqua-boot-summary.json" AQUA_IMAGE_MANIFEST_JSON="${ROOT_DIR:-$(pwd)}/build/aqua-image-manifest.json" scripts/run-qemu-visible-operator-pass.sh 2>&1)"
qemu_visible_operator_pass_blocked_status="$?"
set -e
test "${qemu_visible_operator_pass_blocked_status}" -ne 0
printf '%s\n' "${qemu_visible_operator_pass_blocked_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-pass status=blocked-launch-confirmation'
printf '%s\n' "${qemu_visible_operator_pass_blocked_output}" | grep -Fq 'operator_pass_blocked_reason=missing-explicit-launch-confirmation'
grep -Fq 'status=blocked-launch-confirmation' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'capture_hash_verification_required=true' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'bundle_capture_hash_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'bundle_positive_capture_hash_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'bundle_missing_capture_hash_rejected_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'manual_runbook_pass_report_required_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'pass_report_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'pass_report_evidence_recorded_status=ok' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
grep -Fq 'operator_pass_stop_rule=Do not mark VM display observed' "${CHECK_TEMP_ROOT}/qemu-visible-pass-blocked.txt"
qemu_visible_status_output="$(AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" AQUA_QEMU_VISIBLE_STATUS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-status.txt" AQUA_QEMU_VISIBLE_STATUS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-status.json" scripts/qemu-visible-status.sh)"
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-status status=ok'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'qemu_visible_manual_status=ready-for-operator-pass'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'next_launch_command=scripts/run-qemu-visible-manual.sh'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'preflight_source_sha256='
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'bundle_capture_hash_status=ok'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'bundle_missing_capture_hash_rejected_status=ok'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'manual_runbook_pass_report_required_status=ok'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'pass_report_status=ok'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'next_vm_report_command=aqua-qemu-visible-pass-report'
printf '%s\n' "${qemu_visible_status_output}" | grep -Fq 'capture_hash_verification_required=true'
grep -Fq 'qemu_visible_manual_status=ready-for-operator-pass' "${CHECK_TEMP_ROOT}/qemu-visible-status.txt"
AQUA_QEMU_VISIBLE_STATUS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-status.txt" AQUA_QEMU_VISIBLE_STATUS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-status.json" scripts/check-qemu-visible-status.sh >/dev/null
first_graphics_session_output="$(AQUA_QEMU_VISIBLE_STATUS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-status.json" AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE="${CHECK_TEMP_ROOT}/first-graphics-session-status.txt" AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON="${CHECK_TEMP_ROOT}/first-graphics-session-status.json" scripts/first-graphics-session-status.sh)"
printf '%s\n' "${first_graphics_session_output}" | grep -Fq 'status=ready-for-controlled-visible-attempt'
printf '%s\n' "${first_graphics_session_output}" | grep -Fq 'check_recovery_ready=ok'
printf '%s\n' "${first_graphics_session_output}" | grep -Fq 'check_bounded_visible_runner=ok'
printf '%s\n' "${first_graphics_session_output}" | grep -Fq 'check_operator_pass=ok'
printf '%s\n' "${first_graphics_session_output}" | grep -Fq 'visible_qemu_launched=false'
AQUA_FIRST_GRAPHICS_SESSION_STATUS_FILE="${CHECK_TEMP_ROOT}/first-graphics-session-status.txt" AQUA_FIRST_GRAPHICS_SESSION_STATUS_JSON="${CHECK_TEMP_ROOT}/first-graphics-session-status.json" scripts/check-first-graphics-session-status.sh >/dev/null
qemu_visible_operator_plan_output="$(AQUA_QEMU_VISIBLE_STATUS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-status.txt" AQUA_QEMU_VISIBLE_STATUS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-status.json" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${CHECK_TEMP_ROOT}/qemu-visible-operator-plan.txt" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${CHECK_TEMP_ROOT}/qemu-visible-operator-plan.json" scripts/write-qemu-visible-operator-plan.sh)"
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-plan status=ok'
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq 'next_required_action=operator-run-manual-qemu-pass'
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq 'step_3_command=scripts/run-qemu-visible-manual.sh'
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq 'step_4_command=AQUA_FBDEV_OPERATOR_CONFIRMED=true aqua-graphics-fbdev-present'
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq 'step_6_command=scripts/run-qemu-visible-ready-capture-flow.sh'
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq 'step_8_command=AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply'
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq 'step_9_command=aqua-qemu-visible-pass-report'
printf '%s\n' "${qemu_visible_operator_plan_output}" | grep -Fq 'capture_hash_verification_required=true'
AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${CHECK_TEMP_ROOT}/qemu-visible-operator-plan.txt" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${CHECK_TEMP_ROOT}/qemu-visible-operator-plan.json" scripts/check-qemu-visible-operator-plan.sh >/dev/null
qemu_visible_operator_packet_output="$(AQUA_QEMU_VISIBLE_STATUS_FILE="${CHECK_TEMP_ROOT}/qemu-visible-status.txt" AQUA_QEMU_VISIBLE_STATUS_JSON="${CHECK_TEMP_ROOT}/qemu-visible-status.json" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_FILE="${CHECK_TEMP_ROOT}/qemu-visible-operator-plan.txt" AQUA_QEMU_VISIBLE_OPERATOR_PLAN_JSON="${CHECK_TEMP_ROOT}/qemu-visible-operator-plan.json" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${CHECK_TEMP_ROOT}/qemu-visible-operator-packet.txt" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${CHECK_TEMP_ROOT}/qemu-visible-operator-packet.json" scripts/write-qemu-visible-operator-packet.sh)"
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-packet status=ok'
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq 'packet_status=ready'
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq 'artifact_operator_plan_json_status=ready'
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq 'bundle_capture_hash_status=ok'
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq 'bundle_missing_capture_hash_rejected_status=ok'
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq 'manual_runbook_pass_report_required_status=ok'
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq 'pass_report_status=ok'
printf '%s\n' "${qemu_visible_operator_packet_output}" | grep -Fq 'step_count=9'
AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${CHECK_TEMP_ROOT}/qemu-visible-operator-packet.txt" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${CHECK_TEMP_ROOT}/qemu-visible-operator-packet.json" scripts/check-qemu-visible-operator-packet.sh >/dev/null
qemu_visible_operator_checklist_output="$(AQUA_QEMU_VISIBLE_OPERATOR_PACKET_FILE="${CHECK_TEMP_ROOT}/qemu-visible-operator-packet.txt" AQUA_QEMU_VISIBLE_OPERATOR_PACKET_JSON="${CHECK_TEMP_ROOT}/qemu-visible-operator-packet.json" AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST="${CHECK_TEMP_ROOT}/qemu-visible-operator-checklist.md" scripts/write-qemu-visible-operator-checklist.sh)"
printf '%s\n' "${qemu_visible_operator_checklist_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-operator-checklist status=ok'
AQUA_QEMU_VISIBLE_OPERATOR_CHECKLIST="${CHECK_TEMP_ROOT}/qemu-visible-operator-checklist.md" scripts/check-qemu-visible-operator-checklist.sh >/dev/null
grep -Fq 'scripts/run-qemu-visible-manual.sh' "${CHECK_TEMP_ROOT}/qemu-visible-operator-checklist.md"
grep -Fq 'Capture hash verification required: `true`' "${CHECK_TEMP_ROOT}/qemu-visible-operator-checklist.md"
grep -Fq 'Missing capture hash rejection status: `ok`' "${CHECK_TEMP_ROOT}/qemu-visible-operator-checklist.md"
grep -Fq 'Manual runbook pass report required status: `ok`' "${CHECK_TEMP_ROOT}/qemu-visible-operator-checklist.md"
grep -Fq 'SHA-256:' "${CHECK_TEMP_ROOT}/qemu-visible-operator-checklist.md"
verify_capture_fixture="${CHECK_TEMP_ROOT}/qemu-visible-capture.png"
printf 'aqua-qemu-visible-capture\n' > "${verify_capture_fixture}"
if command -v shasum >/dev/null 2>&1; then
    verify_capture_sha="$(shasum -a 256 "${verify_capture_fixture}" | awk '{print $1}')"
else
    verify_capture_sha="$(sha256sum "${verify_capture_fixture}" | awk '{print $1}')"
fi
verify_capture_meta="${CHECK_TEMP_ROOT}/qemu-visible-capture-verified.env"
{
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=fixture-capture"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=${verify_capture_fixture}"
    echo "AQUA_QEMU_VM_DISPLAY_CAPTURE_SHA256=${verify_capture_sha}"
} > "${verify_capture_meta}"
verify_capture_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=fixture-capture AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${verify_capture_fixture}" scripts/verify-qemu-visible-capture.sh)"
printf '%s\n' "${verify_capture_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-capture-verify status=ok'
printf '%s\n' "${verify_capture_output}" | grep -Fq 'capture_file_status=ready'
printf '%s\n' "${verify_capture_output}" | grep -Fq 'evidence_command=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=fixture-capture'
set +e
unverified_bundle_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=fixture-capture AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${verify_capture_fixture}" AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${CHECK_TEMP_ROOT}/fixture-unverified-evidence-bundle.txt" AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" scripts/write-qemu-visible-evidence-bundle.sh 2>&1)"
unverified_bundle_status="$?"
set -e
test "${unverified_bundle_status}" -ne 0
printf '%s\n' "${unverified_bundle_output}" | grep -Fq 'capture_hash_status=not-verified'
evidence_bundle_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${verify_capture_meta}" AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt" AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" scripts/write-qemu-visible-evidence-bundle.sh)"
printf '%s\n' "${evidence_bundle_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-evidence-bundle status=ok'
printf '%s\n' "${evidence_bundle_output}" | grep -Fq 'bundle_written=ok'
printf '%s\n' "${evidence_bundle_output}" | grep -Fq 'preflight_summary_verified=true'
printf '%s\n' "${evidence_bundle_output}" | grep -Fq 'capture_hash_verified=true'
printf '%s\n' "${evidence_bundle_output}" | grep -Fq 'recovery_step_1=aqua-graphics-qemu-visible-boot-check'
printf '%s\n' "${evidence_bundle_output}" | grep -Fq 'recovery_step_4=aqua-qemu-visible-pass-report'
grep -Fq 'bundle_status=recovery-commands-ready' "${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt"
grep -Fq 'preflight_summary_verified=true' "${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt"
grep -Fq 'capture_hash_verified=true' "${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt"
grep -Fq 'recovery_step_3=AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker' "${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt"
grep -Fq 'recovery_step_4=aqua-qemu-visible-pass-report' "${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt"
apply_prep_output="$(AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt" scripts/prepare-qemu-visible-evidence-apply.sh)"
printf '%s\n' "${apply_prep_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-evidence-apply-prep status=ok'
printf '%s\n' "${apply_prep_output}" | grep -Fq 'capture_hash_verified=true'
printf '%s\n' "${apply_prep_output}" | grep -Fq "cat > /run/aqua/qemu-visible-evidence-bundle.txt <<'AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE'"
printf '%s\n' "${apply_prep_output}" | grep -Fq 'aqua-qemu-visible-evidence-bundle-apply'
printf '%s\n' "${apply_prep_output}" | grep -Fq 'AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply'
printf '%s\n' "${apply_prep_output}" | grep -Fq 'aqua-qemu-visible-pass-report'
missing_hash_bundle="${CHECK_TEMP_ROOT}/fixture-evidence-bundle-missing-hash.txt"
sed '/^capture_hash_verified=/d' "${CHECK_TEMP_ROOT}/fixture-evidence-bundle.txt" > "${missing_hash_bundle}"
set +e
missing_hash_apply_output="$(AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${missing_hash_bundle}" scripts/prepare-qemu-visible-evidence-apply.sh 2>&1)"
missing_hash_apply_status="$?"
set -e
test "${missing_hash_apply_status}" -ne 0
printf '%s\n' "${missing_hash_apply_output}" | grep -Fq 'bundle_file='
evidence_flow_output="$(AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${verify_capture_meta}" AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${CHECK_TEMP_ROOT}/fixture-flow-evidence-bundle.txt" AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" scripts/run-qemu-visible-evidence-flow.sh)"
printf '%s\n' "${evidence_flow_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-evidence-flow status=ok'
printf '%s\n' "${evidence_flow_output}" | grep -Fq 'evidence_flow_ready=true'
printf '%s\n' "${evidence_flow_output}" | grep -Fq 'capture_hash_verified=true'
printf '%s\n' "${evidence_flow_output}" | grep -Fq 'preflight_summary_verified=true'
printf '%s\n' "${evidence_flow_output}" | grep -Fq 'AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true aqua-qemu-visible-evidence-bundle-apply'
ready_capture_serial="${CHECK_TEMP_ROOT}/qemu-visible-ready-capture-serial.log"
cat > "${ready_capture_serial}" <<'EOF'
[AQUA-BOOT] stage=session-check status=ok no_graphics=true
[AQUA-BOOT] stage=recovery-ready status=ok shell=/bin/sh
EOF
ready_capture_output="$(SERIAL_LOG="${ready_capture_serial}" AQUA_QEMU_VISIBLE_WATCH_TIMEOUT=0 AQUA_QEMU_READY_CAPTURE_SKIP_CAPTURE=true AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=fixture-ready-capture AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="${verify_capture_fixture}" AQUA_QEMU_VM_DISPLAY_CAPTURE_META="${CHECK_TEMP_ROOT}/fixture-ready-capture.env" AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE="${CHECK_TEMP_ROOT}/fixture-ready-capture-bundle.txt" AQUA_QEMU_VISIBLE_PREFLIGHT_SUMMARY_JSON="${preflight_summary_fixture}" scripts/run-qemu-visible-ready-capture-flow.sh)"
printf '%s\n' "${ready_capture_output}" | grep -Fq '[AQUA-HOST] stage=qemu-visible-ready-capture-flow status=ok'
printf '%s\n' "${ready_capture_output}" | grep -Fq 'ready_capture_flow_ready=true'
printf '%s\n' "${ready_capture_output}" | grep -Fq 'capture_hash_verified=true'
printf '%s\n' "${ready_capture_output}" | grep -Fq 'preflight_summary_verified=true'
printf '%s\n' "${ready_capture_output}" | grep -Fq 'capture_step=skipped-existing-file'
check_output_contains '[AQUA-HOST] stage=preview-window-probe status=ok' cargo run -p aqua-host-tools -- probe-preview-window
check_output_contains 'window_backend=minifb' cargo run -p aqua-host-tools -- probe-preview-window
check_output_contains 'handoff_ready=ok' cargo run -p aqua-host-tools -- probe-preview-window
check_output_contains 'frame_source=display-output-handoff-composited-client-frame' cargo run -p aqua-host-tools -- probe-preview-window
check_output_contains 'frame_format=raw-rgba8888-composited-client-preview' cargo run -p aqua-host-tools -- probe-preview-window
check_output_contains 'client_layer_snapshot_mode=full-buffer-snapshot' cargo run -p aqua-host-tools -- probe-preview-window
check_output_contains 'rootfs_packaged=false' cargo run -p aqua-host-tools -- probe-preview-window
check_output_contains '[AQUA-HOST] stage=nested-output-presenter-probe status=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'presenter_status=manual-nested-output-presenter-ready' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'source_handoff_ready=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'source_surface_lifecycle_ready=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'frame_source=display-output-handoff-composited-client-frame' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'frame_format=raw-rgba8888-composited-client-preview' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'surface_status=nested-output-surface-lifecycle-complete' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'surface_acquired=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'surface_configured=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'frame_attached=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'frame_presented=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'surface_released=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'surface_frame_matches_presenter_frame=ok' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains 'display_output_started=false' cargo run -p aqua-host-tools -- probe-nested-output-presenter
check_output_contains '[AQUA-HOST] stage=host-window-lifecycle-probe status=ok' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'window_status=manual-host-window-lifecycle-ready' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'window_backend=minifb' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'feature_gate=host-window-preview' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'feature_gate_required=true' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'source_presenter_ready=ok' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'source_surface_lifecycle_ready=ok' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'frame_format=raw-rgba8888-composited-client-preview' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'bounded_frame_limit=600' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'window_opened=false' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'manual_start_required=true' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'rootfs_packaged=false' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
check_output_contains 'recovery_safe=ok' cargo run -p aqua-host-tools -- probe-host-window-lifecycle
host_bridge_output="$(cargo run -p aqua-host-tools -- probe-manual-execution-window-bridge)"
printf '%s\n' "${host_bridge_output}" | grep -Fq '[AQUA-HOST] stage=manual-execution-window-bridge status=ok'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'bridge_status=manual-execution-window-bridge-ready'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'source_execution_ready=ok'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'source_display_started=true'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'source_display_stopped=true'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'source_safe_return_to_recovery=ok'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'window_backend=minifb'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'host_window_ready=ok'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'frame_checksum_matches=ok'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'visible_window_bound=ok'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'window_opened=false'
printf '%s\n' "${host_bridge_output}" | grep -Fq 'rootfs_packaged=false'
host_handoff_fixture="${CHECK_TEMP_ROOT}/visible-preview-launch.txt"
cat > "${host_handoff_fixture}" <<'EOF'
launch_status=qemu-safe-visible-nested-preview-launch-ready
launch_request_ready=ok
request_command_ready=ok
launch_plan_written=ok
launch_window_backend=minifb
launch_feature_gate=host-window-preview
launch_host_tool_packaged=false
launch_qemu_window_started=false
launch_preview_window_started=false
launch_autostart=false
launch_boot_graphics=false
fallback_tty_available=true
safe_return_to_recovery=ok
[AQUA-PREVIEW] stage=visible-nested-preview-launch status=ok
EOF
host_handoff_output="$(AQUA_VISIBLE_PREVIEW_LAUNCH_ARTIFACT="${host_handoff_fixture}" cargo run -p aqua-host-tools -- handoff-summary)"
printf '%s\n' "${host_handoff_output}" | grep -Fq '[AQUA-HOST] stage=host-dev-handoff-summary status=ok'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'handoff_status=host-dev-handoff-summary-ready'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'recovery_launcher_ready=ok'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'recovery_launcher_status=qemu-safe-visible-nested-preview-launch-ready'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'recovery_request_ready=ok'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'recovery_launch_plan_ready=ok'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'host_bridge_ready=ok'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'next_qemu_command=/usr/bin/aqua-visible-preview-launch'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'next_host_command=aqua-host-tools --features host-window-preview -- smoke-manual-execution-window'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'host_tool_packaged=false'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'rootfs_graphical_boot=false'
printf '%s\n' "${host_handoff_output}" | grep -Fq 'rootfs_autostart=false'
grep -Fq 'smoke-host-window-lifecycle' crates/aqua-host-tools/src/main.rs
grep -Fq 'HOST_WINDOW_SMOKE_FRAME_LIMIT: u32 = 3' crates/aqua-host-tools/src/main.rs
grep -Fq 'stage=host-window-lifecycle-smoke status=' crates/aqua-host-tools/src/main.rs
grep -Fq 'probe-manual-execution-window-bridge' crates/aqua-host-tools/src/main.rs
grep -Fq 'handoff-summary' crates/aqua-host-tools/src/main.rs
grep -Fq 'smoke-manual-execution-window' crates/aqua-host-tools/src/main.rs
grep -Fq 'stage=manual-execution-window-smoke status=' crates/aqua-host-tools/src/main.rs
cargo check -p aqua-host-tools --features host-window-preview
scripts/check-assets.sh
scripts/check-visual-preview.sh
scripts/check-public-repo.sh
scripts/check-installer-render.sh
scripts/write-progress-report.sh
scripts/check-progress-report.sh
scripts/check-application-compatibility.sh
scripts/check-compositor.sh

test -f br2-external/aqua/external.desc
test -f br2-external/aqua/configs/aqua_x86_64_defconfig
test -f br2-external/aqua/board/aqua/x86_64/linux.config
test -f br2-external/aqua/board/aqua/x86_64/post-build.sh
test -f br2-external/aqua/rootfs-overlay/etc/init.d/rcS
test -f br2-external/aqua/rootfs-overlay/usr/bin/aqua-recovery
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-session-check
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-compositor-manual-launch
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-compositor-guarded-run
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-compositor-handoff-gate
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-compositor-preview-exec
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-visible-preview-request
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-visible-preview-launch
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-recovery-help
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-operator-transcript
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-enable-gate
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-launch-candidate
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-rollback-drill
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-startup-preflight
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-startup-rehearsal
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-qemu-display-gate
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-visible-qemu-attempt
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-visible-attempt-transcript
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-visible-attempt-result
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-visible-attempt-runner
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-qemu-visible-boot-check
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-qemu-observation-marker
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-qemu-visible-pass-report
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-qemu-visible-evidence-bundle-apply
test -f scripts/build-image.sh
test -f scripts/run-qemu.sh
test -f br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-fbdev-present
grep -Fq 'mount -t devpts devpts /dev/pts' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq 'stage=devpts-ready status=ok' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq -- '-device virtio-vga' scripts/run-qemu.sh
grep -Fq 'CONFIG_FRAMEBUFFER_CONSOLE=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'CONFIG_DRM_FBDEV_EMULATION=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'BR2_PACKAGE_MESA3D_GALLIUM_DRIVER_VIRGL=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_MESA3D_GALLIUM_DRIVER_SWRAST=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_MESA3D_GBM=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_MESA3D_OPENGL_EGL=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_MESA3D_OPENGL_ES=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'smithay-gpu = ' crates/aqua-compositor/Cargo.toml
grep -Fq -- '--features smithay-gpu' scripts/build-compositor-linux-docker.sh
grep -Fq 'release/aqua-properties' scripts/build-compositor-linux-docker.sh
grep -Fq 'release/aqua-terminal' scripts/build-compositor-linux-docker.sh
grep -Fq 'release/aqua-installer' scripts/build-compositor-linux-docker.sh
grep -Fq 'probe-renderer-backend auto' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'probe-gpu-offscreen-frame /dev/dri/card0' scripts/check-fbdev-presenter-qemu.exp
grep -Fq "stage=gpu-offscreen-frame status=ok" scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'gpu_runtime_abi = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'OPERATOR_CONFIRMED="${AQUA_FBDEV_OPERATOR_CONFIRMED:-false}"' br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-fbdev-present
grep -Fq 'HEADLESS_TEST_CONFIRMED="${AQUA_FBDEV_HEADLESS_TEST_CONFIRMED:-false}"' br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphics-fbdev-present
grep -Fq 'graphics fbdev' scripts/report-artifacts.sh
grep -Fq 'graphics_fbdev_present' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "graphics_fbdev_present")' scripts/check-image-manifest.sh
grep -Fq 'graphics_fbdev_headless_qemu_write=' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "graphics_fbdev_headless_qemu_write")' scripts/check-image-manifest.sh
grep -Fq 'headless QEMU fbdev write' scripts/report-artifacts.sh
grep -Fq 'graphics_fbdev_headless_qemu_capture=' scripts/write-image-manifest.sh
grep -Fq 'graphics_fbdev_headless_qemu_wallpaper=' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "graphics_fbdev_headless_qemu_capture")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "graphics_fbdev_headless_qemu_wallpaper")' scripts/check-image-manifest.sh
grep -Fq '/src/target/x86_64-unknown-linux-musl/release/aqua-compositor' scripts/build-image-docker-volume.sh
grep -Fq '/src/target/x86_64-unknown-linux-musl/release/aqua-settings' scripts/build-image-docker-volume.sh
grep -Fq '/src/target/x86_64-unknown-linux-musl/release/aqua-properties' scripts/build-image-docker-volume.sh
grep -Fq '/src/target/x86_64-unknown-linux-musl/release/aqua-terminal' scripts/build-image-docker-volume.sh
grep -Fq '/src/target/x86_64-unknown-linux-musl/release/aqua-installer' scripts/build-image-docker-volume.sh
test -f scripts/check-boot.sh
test -x scripts/check-fbdev-presenter-qemu.sh
test -x scripts/check-fbdev-presenter-qemu.exp
test -x scripts/capture-qemu-monitor-screendump.py
grep -Fq 'AQUA_FBDEV_HEADLESS_TEST_CONFIRMED=true' scripts/check-fbdev-presenter-qemu.exp
grep -Fq 'visible_observation=false' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'screendump' scripts/capture-qemu-monitor-screendump.py
grep -Fq 'qemu-fbdev-present.png' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'qemu-drm-kms-present.png' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'AQUA_DRM_KMS_HEADLESS_TEST_CONFIRMED=true' scripts/check-fbdev-presenter-qemu.exp
grep -Fq 'stage=drm-kms-present status=active' scripts/check-fbdev-presenter-qemu.exp
grep -Fq 'crtc_restored=true' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'present-drm-kms' crates/aqua-compositor/src/main.rs
grep -Fq 'present-drm-page-flip' crates/aqua-compositor/src/main.rs
grep -Fq 'PageFlipFlags::EVENT' crates/aqua-compositor/src/main.rs
grep -Fq 'page_flip_event_received=true' crates/aqua-compositor/src/main.rs
grep -Fq 'qemu-drm-page-flip-present.png' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'run-drm-frame-loop' crates/aqua-compositor/src/main.rs
grep -Fq 'submitted_page_flips=' crates/aqua-compositor/src/main.rs
grep -Fq 'received_page_flip_events=' crates/aqua-compositor/src/main.rs
grep -Fq 'qemu-drm-frame-loop.png' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'run-drm-session-loop' crates/aqua-compositor/src/main.rs
grep -Fq 'DrmEventWaiter::Calloop' crates/aqua-compositor/src/main.rs
grep -Fq 'drm_event_source_owned=true' crates/aqua-compositor/src/main.rs
grep -Fq 'qemu-drm-session-loop.png' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'run-drm-wayland-session' crates/aqua-compositor/src/main.rs
grep -Fq 'SmithayDrmSession' crates/aqua-compositor/src/main.rs
grep -Fq 'probe-evdev-aqua-seat' crates/aqua-compositor/src/main.rs
grep -Fq 'CONFIG_VIRTIO_INPUT=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'CONFIG_DRM_BOCHS=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'virtio-keyboard-pci' scripts/check-fbdev-presenter-qemu.exp
grep -Fq 'virtio-mouse-pci' scripts/check-fbdev-presenter-qemu.exp
test -x scripts/send-qemu-monitor-input.py
test -x scripts/check-qemu-input-daemon.py
test -x scripts/check-r2-presentation-log.py
test -x scripts/check-r2-presentation-qemu.sh
test -x scripts/check-r2-presentation-qemu.exp
test -x scripts/check-r2-presentation-repeated-qemu.sh
test -x scripts/check-r2-presentation-soak-qemu.sh
test -x scripts/check-r2-presentation-soak-qemu.exp
test -x scripts/check-r2-presentation-qualification-qemu.sh
test -x scripts/check-r2-presentation-repeated-qualification-qemu.sh
PYTHONPYCACHEPREFIX="${CHECK_TEMP_ROOT}/python-cache" \
    python3 -m py_compile \
    scripts/send-qemu-monitor-input.py \
    scripts/capture-qemu-monitor-screendump.py \
    scripts/check-qemu-input-daemon.py \
    scripts/check-r2-presentation-log.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check-qemu-input-daemon.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check-r2-presentation-log.py --self-test
grep -Fq 'test "${ROOTFS}" -nt "${ROOT_DIR}/crates/aqua-compositor/src/main.rs"' scripts/check-r2-presentation-qemu.sh
grep -Fq 'DISPLAY_DEVICE="${DISPLAY_DEVICE:-bochs-display}"' scripts/check-r2-presentation-qemu.sh
grep -Fq 'start_bounded_session idle 4' scripts/check-r2-presentation-qemu.exp
grep -Fq 'start_bounded_session window-interaction 6' scripts/check-r2-presentation-qemu.exp
grep -Fq 'AQUA_R2_PRESENTATION_WORKLOAD=animation' scripts/check-r2-presentation-qemu.exp
grep -Fq 'start_bounded_session multi-client 90' scripts/check-r2-presentation-qemu.exp
grep -Fq '.min(120)' crates/aqua-compositor/src/main.rs
grep -Fq 'let frame_count = if r2_idle_workload { 1 } else { 3 };' crates/aqua-compositor/src/main.rs
grep -Fq 'r2_presentation_full_frame_readbacks={}' crates/aqua-compositor/src/main.rs
grep -Fq 'if int(parsed["full_frame_readbacks"]) != 0:' scripts/check-r2-presentation-log.py
grep -Fq 'drm_wayland_gpu_frame_readback={cpu_scanout_compat}' crates/aqua-compositor/src/main.rs
grep -Fq 'render-submission-token' crates/aqua-compositor/src/main.rs
grep -Fq 'AQUA_R2_DIAGNOSTIC_READBACK_TELEMETRY=true' scripts/check-r2-presentation-qemu.exp
grep -Fq 'r2_diagnostic_record_end=v1' scripts/check-r2-presentation-qemu.exp
grep -Fq 'r2_budget_profile=qemu-tcg-bochs-v1' scripts/check-r2-presentation-qemu.sh
grep -Fq 'r2_budget_selected=true' scripts/check-r2-presentation-qemu.sh
grep -Fq 'r2_physical_budget_selected=false' scripts/check-r2-presentation-qemu.sh
grep -Fq 'r2_diagnostic_isolation_recorded=true' scripts/check-r2-presentation-qemu.sh
grep -Fq 'RUNS="${RUNS:-3}"' scripts/check-r2-presentation-repeated-qemu.sh
grep -Fq 'R2 evidence directory already exists' scripts/check-r2-presentation-repeated-qemu.sh
grep -Fq -- '--summarize-repeated' scripts/check-r2-presentation-repeated-qemu.sh
grep -Fq 'r2_review_budget_profile=qemu-tcg-bochs-v1' scripts/check-r2-presentation-repeated-qemu.sh
grep -Fq 'r2_review_budget_selected=true' scripts/check-r2-presentation-repeated-qemu.sh
grep -Fq 'r2_review_physical_budget_selected=false' scripts/check-r2-presentation-repeated-qemu.sh
grep -Fq 'pub const QEMU_TCG_BOCHS_V1_BUDGET' crates/aqua-compositor/src/presentation.rs
grep -Fq 'pub const QEMU_TCG_BOCHS_SOAK_V1_BUDGET' crates/aqua-compositor/src/presentation.rs
grep -Fq 'pub const QEMU_TCG_BOCHS_QUALIFICATION_V1_BUDGET' crates/aqua-compositor/src/presentation.rs
grep -Fq 'SOAK_SECONDS="${SOAK_SECONDS:-300}"' scripts/check-r2-presentation-soak-qemu.sh
grep -Fq 'R2 soak evidence directory already exists' scripts/check-r2-presentation-soak-qemu.sh
grep -Fq 'monitor socket path must be shorter than 104 bytes' scripts/check-r2-presentation-soak-qemu.sh
grep -Fq -- '--summarize-soak' scripts/check-r2-presentation-soak-qemu.sh
grep -Fq 'r2_soak_budget_profile=qemu-tcg-bochs-soak-v1' scripts/check-r2-presentation-soak-qemu.sh
grep -Fq 'r2_soak_physical_evidence=false' scripts/check-r2-presentation-soak-qemu.sh
grep -Fq 'AQUA_DRM_WAYLAND_SESSION_PERSISTENT=true' scripts/check-r2-presentation-soak-qemu.exp
grep -Fq 'AQUA_DRM_WAYLAND_STOP_FILE=/run/aqua/r2-presentation-soak.stop' scripts/check-r2-presentation-soak-qemu.exp
grep -Fq 'for {set cycle 1} {$cycle <= $env(INPUT_CYCLES)} {incr cycle}' scripts/check-r2-presentation-soak-qemu.exp
grep -Fq 'desktop_event_launcher_visible=true' scripts/check-r2-presentation-soak-qemu.exp
grep -Fq 'desktop_event_launcher_visible=false' scripts/check-r2-presentation-soak-qemu.exp
grep -Fq 'QEMU_SOAK_MIN_OBSERVATION_WINDOW_MS = 300_000' scripts/check-r2-presentation-log.py
grep -Fq 'QEMU_SOAK_MIN_INPUT_SAMPLES = 5' scripts/check-r2-presentation-log.py
grep -Fq 'SOAK_SECONDS="${SOAK_SECONDS:-900}"' scripts/check-r2-presentation-qualification-qemu.sh
grep -Fq 'INPUT_CYCLES="${INPUT_CYCLES:-15}"' scripts/check-r2-presentation-qualification-qemu.sh
grep -Fq 'monitor socket path must be shorter than 104 bytes' scripts/check-r2-presentation-qualification-qemu.sh
grep -Fq -- '--summarize-qualification-soak' scripts/check-r2-presentation-qualification-qemu.sh
grep -Fq 'r2_qualification_soak_budget_profile=qemu-tcg-bochs-qualification-v1' scripts/check-r2-presentation-qualification-qemu.sh
grep -Fq 'r2_qualification_soak_release_ready=false' scripts/check-r2-presentation-qualification-qemu.sh
grep -Fq 'RUNS="${RUNS:-3}"' scripts/check-r2-presentation-repeated-qualification-qemu.sh
grep -Fq 'R2 qualification evidence directory already exists' scripts/check-r2-presentation-repeated-qualification-qemu.sh
grep -Fq -- '--summarize-repeated-qualification' scripts/check-r2-presentation-repeated-qualification-qemu.sh
grep -Fq 'r2_qualification_review_budget_profile=qemu-tcg-bochs-qualification-v1' scripts/check-r2-presentation-repeated-qualification-qemu.sh
grep -Fq 'r2_qualification_review_physical_evidence=false' scripts/check-r2-presentation-repeated-qualification-qemu.sh
grep -Fq 'r2_qualification_review_release_ready=false' scripts/check-r2-presentation-repeated-qualification-qemu.sh
grep -Fq 'QEMU_QUALIFICATION_MIN_OBSERVATION_WINDOW_MS = 900_000' scripts/check-r2-presentation-log.py
grep -Fq 'QEMU_QUALIFICATION_MIN_INPUT_SAMPLES = 15' scripts/check-r2-presentation-log.py
grep -Fq -- '--serve' scripts/send-qemu-monitor-input.py
grep -Fq 'AQUA_QEMU_INPUT_CONTROL_SOCKET' scripts/check-graphical-boot-qemu.sh
grep -Fq 'request_input_daemon' scripts/capture-qemu-monitor-screendump.py
grep -Fq 'evdev_events_dispatched=true' scripts/check-fbdev-presenter-qemu.sh
grep -Fq '("scene_contract", "graphics_evdev_aqua_seat")' scripts/check-image-manifest.sh
grep -Fq 'smithay_protocol_globals_started=true' scripts/check-fbdev-presenter-qemu.sh
grep -Fq '("scene_contract", "graphics_drm_wayland_seat")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "graphics_drm_wayland_input_dispatch")' scripts/check-image-manifest.sh
grep -Fq 'wayland_display_created=true' crates/aqua-compositor/src/main.rs
grep -Fq 'wayland_client_inserted=true' crates/aqua-compositor/src/main.rs
grep -Fq 'qemu-drm-wayland-session.png' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'wallpaper_source=runtime-asset' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'scripts/check-fbdev-presenter-qemu.sh' scripts/check-image.sh
test -f scripts/report-artifacts.sh
grep -Fq 'probe-preview-window' crates/aqua-host-tools/src/main.rs
grep -Fq 'probe-nested-output-presenter' crates/aqua-host-tools/src/main.rs
grep -Fq 'probe-host-window-lifecycle' crates/aqua-host-tools/src/main.rs
grep -Fq 'probe-manual-execution-window-bridge' crates/aqua-host-tools/src/main.rs
grep -Fq 'handoff-summary' crates/aqua-host-tools/src/main.rs
grep -Fq 'smoke-host-window-lifecycle' crates/aqua-host-tools/src/main.rs
grep -Fq 'smoke-manual-execution-window' crates/aqua-host-tools/src/main.rs
grep -Fq 'run_nested_output_surface_lifecycle' crates/aqua-host-tools/src/main.rs
grep -Fq 'host-window-preview' crates/aqua-host-tools/Cargo.toml
grep -Fq 'minifb' crates/aqua-host-tools/Cargo.toml
test -x scripts/write-boot-summary.sh
test -x scripts/check-boot-summary.sh
test -f scripts/aqua-boot-stages.txt
grep -Fq 'Aqua Linux contract summary' scripts/report-artifacts.sh
grep -Fq 'Aqua Linux boot marker summary' scripts/report-artifacts.sh
grep -Fq 'session-check.txt' scripts/report-artifacts.sh
grep -Fq 'manual-launch-plan.txt' scripts/report-artifacts.sh
grep -Fq 'manual launch plan' scripts/report-artifacts.sh
grep -Fq 'guarded-run.txt' scripts/report-artifacts.sh
grep -Fq 'guarded run' scripts/report-artifacts.sh
grep -Fq 'handoff-gate.txt' scripts/report-artifacts.sh
grep -Fq 'handoff gate' scripts/report-artifacts.sh
grep -Fq 'manual-nested-preview-backend.txt' scripts/report-artifacts.sh
grep -Fq 'manual nested backend' scripts/report-artifacts.sh
grep -Fq 'manual-nested-preview-execution.txt' scripts/report-artifacts.sh
grep -Fq 'manual execution' scripts/report-artifacts.sh
grep -Fq 'execution probe' scripts/report-artifacts.sh
grep -Fq 'manual nested execution' scripts/report-artifacts.sh
grep -Fq 'visible-preview-request.txt' scripts/report-artifacts.sh
grep -Fq 'visible preview request' scripts/report-artifacts.sh
grep -Fq 'visible-preview-launch.txt' scripts/report-artifacts.sh
grep -Fq 'visible preview launcher' scripts/report-artifacts.sh
grep -Fq 'recovery-help.txt' scripts/report-artifacts.sh
grep -Fq 'recovery operator help' scripts/report-artifacts.sh
grep -Fq 'recovery help operator pass' scripts/report-artifacts.sh
grep -Fq 'recovery help no-launch' scripts/report-artifacts.sh
grep -Fq 'recovery help checklist' scripts/report-artifacts.sh
grep -Fq 'recovery help pass artifact' scripts/report-artifacts.sh
grep -Fq 'recovery help pass external' scripts/report-artifacts.sh
grep -Fq 'operator-transcript.txt' scripts/report-artifacts.sh
grep -Fq 'operator transcript' scripts/report-artifacts.sh
grep -Fq 'graphics-enable-gate.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-enable-gate-positive.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-launch-candidate.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-rollback-drill.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-startup-preflight.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-startup-rehearsal.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-qemu-display-gate.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-visible-qemu-attempt.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-visible-attempt-transcript.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-visible-attempt-result.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-visible-attempt-runner.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-qemu-visible-boot-check.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-qemu-observation-marker.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-evidence-record.txt' scripts/report-artifacts.sh
grep -Fq 'graphics-qemu-observation-positive.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-manual-runbook.txt' scripts/report-artifacts.sh
grep -Fq 'qemu visible ready capture flow' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-evidence-bundle-apply.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-evidence-bundle-apply-positive.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-evidence-bundle-apply-missing-preflight.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-evidence-bundle-apply-missing-capture-hash.txt' scripts/report-artifacts.sh
grep -Fq 'graphics enable gate' scripts/report-artifacts.sh
grep -Fq 'graphics allow dry-run' scripts/report-artifacts.sh
grep -Fq 'graphics launch candidate' scripts/report-artifacts.sh
grep -Fq 'graphics rollback drill' scripts/report-artifacts.sh
grep -Fq 'graphics startup preflight' scripts/report-artifacts.sh
grep -Fq 'graphics startup rehearsal' scripts/report-artifacts.sh
grep -Fq 'graphics qemu display gate' scripts/report-artifacts.sh
grep -Fq 'graphics visible qemu attempt' scripts/report-artifacts.sh
grep -Fq 'graphics visible attempt transcript' scripts/report-artifacts.sh
grep -Fq 'activation plan' scripts/report-artifacts.sh
grep -Fq 'aqua-boot-summary.json' scripts/report-artifacts.sh
grep -Fq 'boot json' scripts/report-artifacts.sh
grep -Fq 'aqua-image-manifest.json' scripts/report-artifacts.sh
grep -Fq 'manifest json' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-status.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-status.json' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-operator-plan.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-operator-plan.json' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-operator-packet.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-operator-packet.json' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-operator-checklist.md' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-operator-pass.txt' scripts/report-artifacts.sh
grep -Fq 'qemu-visible-operator-pass.json' scripts/report-artifacts.sh
grep -Fq 'first-graphics-session-status.txt' scripts/report-artifacts.sh
grep -Fq 'first-graphics-session-status.json' scripts/report-artifacts.sh
grep -Fq 'qemu visible status' scripts/report-artifacts.sh
grep -Fq 'qemu visible json' scripts/report-artifacts.sh
grep -Fq 'qemu operator plan' scripts/report-artifacts.sh
grep -Fq 'qemu operator json' scripts/report-artifacts.sh
grep -Fq 'qemu operator packet' scripts/report-artifacts.sh
grep -Fq 'qemu packet json' scripts/report-artifacts.sh
grep -Fq 'qemu checklist' scripts/report-artifacts.sh
grep -Fq 'qemu pass' scripts/report-artifacts.sh
grep -Fq 'qemu pass json' scripts/report-artifacts.sh
grep -Fq 'first graphics' scripts/report-artifacts.sh
grep -Fq 'first graphics json' scripts/report-artifacts.sh
grep -Fq 'qemu pass stop rule' scripts/report-artifacts.sh
grep -Fq 'qemu pass evidence flow' scripts/report-artifacts.sh
grep -Fq 'qemu pass hash gate' scripts/report-artifacts.sh
grep -Fq 'qemu pass hash status' scripts/report-artifacts.sh
grep -Fq 'qemu pass hash rejection' scripts/report-artifacts.sh
grep -Fq 'recovery help report required' scripts/report-artifacts.sh
grep -Fq 'recovery help report after apply' scripts/report-artifacts.sh
grep -Fq 'qemu pass runbook report gate' scripts/report-artifacts.sh
grep -Fq 'qemu pass preflight hash' scripts/report-artifacts.sh
grep -Fq 'qemu visible pass report' scripts/report-artifacts.sh
grep -Fq 'qemu pass report evidence' scripts/report-artifacts.sh
grep -Fq 'qemu runbook pass report required' scripts/report-artifacts.sh
grep -Fq 'qemu_visible_operator_pass_stop_rule' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_confirmed_command' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_capture_hash_gate' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_capture_hash_status' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_missing_capture_hash_rejected' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_preflight_source' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_capture_hash_gate' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_manual_runbook_pass_report_required' scripts/check-image-manifest.sh
test -f scripts/check-visual-preview.sh
test -x scripts/write-progress-report.sh
test -x scripts/check-progress-report.sh
test -x scripts/check-application-compatibility.sh
test -x scripts/write-qemu-visible-evidence-bundle.sh
test -x scripts/prepare-qemu-visible-evidence-apply.sh
test -x scripts/run-qemu-visible-evidence-flow.sh
test -x scripts/preflight-qemu-visible-manual.sh
test -x scripts/watch-qemu-visible-readiness.sh
test -x scripts/run-qemu-visible-ready-capture-flow.sh
test -x scripts/qemu-visible-status.sh
test -x scripts/check-qemu-visible-status.sh
test -x scripts/write-qemu-visible-operator-plan.sh
test -x scripts/check-qemu-visible-operator-plan.sh
test -x scripts/write-qemu-visible-operator-packet.sh
test -x scripts/check-qemu-visible-operator-packet.sh
test -x scripts/write-qemu-visible-operator-checklist.sh
test -x scripts/check-qemu-visible-operator-checklist.sh
test -x scripts/run-qemu-visible-operator-pass.sh
test -x scripts/check-qemu-visible-operator-pass.sh
test -x scripts/write-qemu-visible-preflight-summary.sh
test -x scripts/check-qemu-visible-preflight-summary.sh
test -f scripts/check-runtime-assets.sh
test -x scripts/check-image.sh
test -x scripts/check-image-manifest.sh
test -x scripts/write-image-manifest.sh
test -x scripts/export-rootfs-compositor-contract-docker.sh
test -f scripts/check-compositor.sh
test -x scripts/check-compositor-packaged.sh
test -x scripts/check-installer-probe-qemu.sh
test -x scripts/check-installer-probe-qemu.exp
test -x scripts/check-installer-target-selection-qemu.sh
test -x scripts/check-installer-target-selection-qemu.exp
test -x scripts/check-installer-execution-gate-qemu.sh
test -x scripts/check-installer-execution-gate-qemu.exp
test -x scripts/check-installer-transaction-qemu.sh
test -x scripts/check-installer-transaction-qemu.exp
test -x scripts/check-installer-cleanup-qemu.sh
test -x scripts/check-installer-cleanup-qemu.exp
test -x scripts/check-installer-wayland-qemu.sh
test -x scripts/check-installer-wayland-qemu.exp
test -x scripts/write-installer-artifact-disk-docker.sh
test -x scripts/check-compositor-rootfs-docker.sh
test -x scripts/build-compositor-linux-docker.sh
grep -Fq 'scripts/check-compositor-rootfs-docker.sh' scripts/check-image.sh
grep -Fq 'scripts/check-boot.sh' scripts/check-image.sh
grep -Fq 'scripts/write-boot-summary.sh' scripts/check-image.sh
grep -Fq 'scripts/check-boot-summary.sh' scripts/check-image.sh
grep -Fq 'scripts/write-image-manifest.sh' scripts/check-image.sh
grep -Fq 'scripts/check-image-manifest.sh' scripts/check-image.sh
grep -Fq 'scripts/report-artifacts.sh' scripts/check-image.sh
grep -Fq 'scripts/qemu-visible-status.sh' scripts/check-image.sh
grep -Fq 'scripts/check-qemu-visible-status.sh' scripts/check-image.sh
grep -Fq 'scripts/write-qemu-visible-operator-plan.sh' scripts/check-image.sh
grep -Fq 'scripts/check-qemu-visible-operator-plan.sh' scripts/check-image.sh
grep -Fq 'scripts/write-qemu-visible-operator-packet.sh' scripts/check-image.sh
grep -Fq 'scripts/check-qemu-visible-operator-packet.sh' scripts/check-image.sh
grep -Fq 'scripts/write-qemu-visible-operator-checklist.sh' scripts/check-image.sh
grep -Fq 'scripts/check-qemu-visible-operator-checklist.sh' scripts/check-image.sh
grep -Fq 'scripts/run-qemu-visible-operator-pass.sh' docs/aqua-linux/compositor.md
grep -Fq 'AQUA_QEMU_VISIBLE_OPERATOR_PASS_NO_LAUNCH' docs/aqua-linux/compositor.md
grep -Fq 'AQUA_QEMU_VISIBLE_OPERATOR_PASS_CONFIRM_LAUNCH' docs/aqua-linux/compositor.md
grep -Fq 'qemu-visible-operator-pass.txt' docs/aqua-linux/compositor.md
grep -Fq 'Aqua Linux QEMU visible preflight summary checks passed.' scripts/check-qemu-visible-preflight-summary.sh
grep -Fq 'Aqua Linux JSON boot summary written' scripts/write-boot-summary.sh
grep -Fq '"expected_stages"' scripts/write-boot-summary.sh
grep -Fq 'BOOT_STAGE_FILE' scripts/write-boot-summary.sh
grep -Fq 'surface-primitives' scripts/aqua-boot-stages.txt
grep -Fq 'output-plan' scripts/aqua-boot-stages.txt
grep -Fq 'visible-preview-plan' scripts/aqua-boot-stages.txt
grep -Fq 'raster-png-export' scripts/aqua-boot-stages.txt
grep -Fq 'recovery-ready' scripts/aqua-boot-stages.txt
grep -Fq 'Aqua Linux boot summary checks passed.' scripts/check-boot-summary.sh
grep -Fq 'expected_stages order changed' scripts/check-boot-summary.sh
grep -Fq 'BOOT_STAGE_FILE' scripts/check-boot-summary.sh
grep -Fq 'appears out of boot order' scripts/check-boot-summary.sh
grep -Fq 'Aqua Linux JSON image manifest checks passed.' scripts/check-image-manifest.sh
grep -Fq 'desktop_shell' scripts/check-image-manifest.sh
grep -Fq 'surface_primitives' scripts/check-image-manifest.sh
grep -Fq 'output_plan' scripts/check-image-manifest.sh
grep -Fq 'display_output_handoff' scripts/check-image-manifest.sh
grep -Fq 'visible_preview_plan' scripts/check-image-manifest.sh
grep -Fq 'visible_preview_export' scripts/check-image-manifest.sh
grep -Fq 'display_output_handoff_frame_format' scripts/check-image-manifest.sh
grep -Fq 'display_output_handoff_frame_checksum' scripts/check-image-manifest.sh
grep -Fq 'display_activation_plan_probe' scripts/check-image-manifest.sh
grep -Fq 'display_activation_plan_can_activate' scripts/check-image-manifest.sh
grep -Fq 'display_output_smoke' scripts/check-image-manifest.sh
grep -Fq 'display_output_smoke_started' scripts/check-image-manifest.sh
grep -Fq 'nested_output_surface' scripts/check-image-manifest.sh
grep -Fq 'nested_output_surface_frame_presented' scripts/check-image-manifest.sh
grep -Fq 'nested_preview_loop' scripts/check-image-manifest.sh
grep -Fq 'manual_nested_preview_execution' scripts/check-image-manifest.sh
grep -Fq 'manual_nested_preview_execution_no_boot_graphics' scripts/check-image-manifest.sh
grep -Fq 'visible_preview_request' scripts/check-image-manifest.sh
grep -Fq 'visible_preview_request_no_boot_graphics' scripts/check-image-manifest.sh
grep -Fq 'visible_preview_launch' scripts/check-image-manifest.sh
grep -Fq 'visible_preview_launch_no_boot_graphics' scripts/check-image-manifest.sh
grep -Fq 'recovery_help' scripts/check-image-manifest.sh
grep -Fq 'recovery_help_no_boot_graphics' scripts/check-image-manifest.sh
grep -Fq 'operator_transcript' scripts/check-image-manifest.sh
grep -Fq 'operator_transcript_no_boot_graphics' scripts/check-image-manifest.sh
grep -Fq 'graphics_enable_gate' scripts/check-image-manifest.sh
grep -Fq 'graphics_enable_gate_preflight' scripts/check-image-manifest.sh
grep -Fq 'graphics_enable_gate_check_manual_execution' scripts/check-image-manifest.sh
grep -Fq 'graphics_enable_gate_positive_allowable' scripts/check-image-manifest.sh
grep -Fq 'graphics_launch_candidate_rollback' scripts/check-image-manifest.sh
grep -Fq 'graphics_rollback_drill_verified' scripts/check-image-manifest.sh
grep -Fq 'graphics_startup_preflight_decision' scripts/check-image-manifest.sh
grep -Fq 'graphics_startup_rehearsal_decision' scripts/check-image-manifest.sh
grep -Fq 'graphics_qemu_display_gate_decision' scripts/check-image-manifest.sh
grep -Fq 'graphics_visible_qemu_attempt_command' scripts/check-image-manifest.sh
grep -Fq 'graphics_visible_attempt_transcript_expected_return' scripts/check-image-manifest.sh
grep -Fq 'graphics_visible_attempt_result_manual_not_run' scripts/check-image-manifest.sh
grep -Fq 'graphics_visible_attempt_runner_completed' scripts/check-image-manifest.sh
grep -Fq 'graphics_qemu_visible_boot_path_ready' scripts/check-image-manifest.sh
grep -Fq 'graphics_qemu_observation_marker_not_observed' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_record_capture' scripts/check-image-manifest.sh
grep -Fq 'graphics_qemu_observation_positive_observed' scripts/check-image-manifest.sh
grep -Fq 'graphics_qemu_observation_positive_evidence_recorded' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_pass_report_observed' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_pass_report_evidence_recorded' scripts/check-image-manifest.sh
grep -Fq 'recovery_help_pass_report_required' scripts/check-image-manifest.sh
grep -Fq 'recovery_help_pass_report_after_apply' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_host' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_ready_capture_flow' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_evidence_required' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_pass_report_required' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_waiting' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_positive_observed' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_preflight_verified' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_capture_hash_verified' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_positive_preflight_verified' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_positive_capture_hash_verified' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_preflight_rejected' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_preflight_verified' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_preflight_unverified' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_capture_hash_rejected' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_capture_hash_status' scripts/check-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_capture_hash_value' scripts/check-image-manifest.sh
grep -Fq 'graphics_enable_gate_no_boot_graphics' scripts/check-image-manifest.sh
grep -Fq 'client_window_model' scripts/check-image-manifest.sh
grep -Fq 'client_surface_lifecycle' scripts/check-image-manifest.sh
grep -Fq 'client_surface_registry' scripts/check-image-manifest.sh
grep -Fq 'client_surface_registry_two_client' scripts/check-image-manifest.sh
grep -Fq 'client_surface_registry_stacking' scripts/check-image-manifest.sh
grep -Fq 'xdg_shell_binding' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_client' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_shm_global' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_shm_import' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_shm_sample' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_configure_ack' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_close_event' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_client_count' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_window_model' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_window_count' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_window_two_model' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_window_stacking' scripts/check-image-manifest.sh
grep -Fq 'raster_png_export' scripts/check-image-manifest.sh
grep -Fq 'session_check' scripts/check-image-manifest.sh
grep -Fq 'scene-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'scripts/export-rootfs-compositor-contract-docker.sh' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'output-plan-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'visible-preview-plan-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'visible-preview-export-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'aqua-visible-preview.html' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'nested-preview-loop.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'manual-nested-preview-backend.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'manual-nested-preview-execution.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'manual-nested-preview-execution-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'visible-preview-request.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'visible-preview-launch.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'recovery-help.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'operator-transcript.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-enable-gate.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-enable-gate-positive.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-launch-candidate.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-rollback-drill.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-startup-preflight.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-startup-rehearsal.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-qemu-display-gate.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-visible-qemu-attempt.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-visible-attempt-transcript.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-visible-attempt-result.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-visible-attempt-runner.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-qemu-visible-boot-check.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-qemu-observation-marker.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'qemu-visible-evidence-record.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'graphics-qemu-observation-positive.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'qemu-visible-manual-runbook.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'client-window-model-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'client-surface-registry-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'xdg-toplevel-window-model-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'render-plan-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'paint-plan-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'frame-plan-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'frame-buffer-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'raster-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'raster-export-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'aqua-raster.ppm' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'raster-png-export-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'aqua-raster.png' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'session-loop.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'session-config.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'session-env.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'session-bootstrap.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'session-check.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'output-plan-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'visible-preview-plan-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'visible-preview-export-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-visible-preview.html' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'nested-preview-loop.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'probe-manual-nested-preview-backend' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'run-manual-nested-preview-execution' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-visible-preview-request' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-visible-preview-launch' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-recovery-help' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-operator-transcript' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-enable-gate' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-launch-candidate' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-rollback-drill' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-startup-preflight' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-startup-rehearsal' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-qemu-display-gate' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-visible-qemu-attempt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-visible-attempt-transcript' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-visible-attempt-result' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-visible-attempt-runner' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-qemu-visible-boot-check' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-graphics-qemu-observation-marker' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-qemu-visible-evidence-record' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-qemu-visible-pass-report' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-qemu-visible-evidence-bundle-apply' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'AQUA_QEMU_VM_DISPLAY_OBSERVED=true' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-qemu-visible-manual-runbook' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'client-window-model-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'client-surface-registry-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'xdg-toplevel-window-model-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'launcher-model-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'launcher-input-scene-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'paint-plan-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'frame-plan-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'frame-buffer-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'raster-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'raster-export-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-raster.ppm' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'raster-png-export-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'aqua-raster.png' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'session-loop.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'session-config.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'session-env.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'session-bootstrap.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'session-check.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'manual-launch-plan.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'guarded-run.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'handoff-gate.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'paint-plan-dump.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'frame-plan-dump.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'frame-buffer-dump.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'raster-dump.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'session_loop=' scripts/write-image-manifest.sh
grep -Fq 'session_config_probe=' scripts/write-image-manifest.sh
grep -Fq 'session_env_probe=' scripts/write-image-manifest.sh
grep -Fq 'session_bootstrap_probe=' scripts/write-image-manifest.sh
grep -Fq 'session_check_probe=' scripts/write-image-manifest.sh
grep -Fq 'manual_launch_plan=' scripts/write-image-manifest.sh
grep -Fq 'manual_launch_no_boot_graphics=' scripts/write-image-manifest.sh
grep -Fq 'guarded_run=' scripts/write-image-manifest.sh
grep -Fq 'guarded_run_return=' scripts/write-image-manifest.sh
grep -Fq 'handoff_gate=' scripts/write-image-manifest.sh
grep -Fq 'handoff_gate_no_auto=' scripts/write-image-manifest.sh
grep -Fq 'handoff_gate_backend=' scripts/write-image-manifest.sh
grep -Fq 'manual_nested_preview_backend=' scripts/write-image-manifest.sh
grep -Fq 'manual_nested_preview_backend_no_start=' scripts/write-image-manifest.sh
grep -Fq 'manual_nested_preview_execution=' scripts/write-image-manifest.sh
grep -Fq 'manual_nested_preview_execution_no_boot_graphics=' scripts/write-image-manifest.sh
grep -Fq 'visible_preview_request=' scripts/write-image-manifest.sh
grep -Fq 'visible_preview_request_no_boot_graphics=' scripts/write-image-manifest.sh
grep -Fq 'visible_preview_launch=' scripts/write-image-manifest.sh
grep -Fq 'visible_preview_launch_no_boot_graphics=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_no_boot_graphics=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_operator_pass_host=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_operator_pass_no_launch=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_operator_checklist=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_operator_pass_artifact=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_operator_pass_external=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_visible_pass_report=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_capture_hash_gate=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_capture_hash_status=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_positive_capture_hash_status=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_missing_capture_hash_rejected=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_operator_pass_manual_runbook_pass_report_required=' scripts/write-image-manifest.sh
grep -Fq 'operator_transcript=' scripts/write-image-manifest.sh
grep -Fq 'operator_transcript_no_boot_graphics=' scripts/write-image-manifest.sh
grep -Fq 'graphics_enable_gate=' scripts/write-image-manifest.sh
grep -Fq 'graphics_enable_gate_preflight=' scripts/write-image-manifest.sh
grep -Fq 'graphics_enable_gate_check_manual_execution=' scripts/write-image-manifest.sh
grep -Fq 'graphics_enable_gate_positive_allowable=' scripts/write-image-manifest.sh
grep -Fq 'graphics_launch_candidate_rollback=' scripts/write-image-manifest.sh
grep -Fq 'graphics_rollback_drill_verified=' scripts/write-image-manifest.sh
grep -Fq 'graphics_startup_preflight_decision=' scripts/write-image-manifest.sh
grep -Fq 'graphics_startup_rehearsal_decision=' scripts/write-image-manifest.sh
grep -Fq 'graphics_qemu_display_gate_decision=' scripts/write-image-manifest.sh
grep -Fq 'graphics_visible_qemu_attempt_command=' scripts/write-image-manifest.sh
grep -Fq 'graphics_visible_attempt_transcript_expected_return=' scripts/write-image-manifest.sh
grep -Fq 'graphics_visible_attempt_result_manual_not_run=' scripts/write-image-manifest.sh
grep -Fq 'graphics_visible_attempt_runner_completed=' scripts/write-image-manifest.sh
grep -Fq 'graphics_qemu_visible_boot_path_ready=' scripts/write-image-manifest.sh
grep -Fq 'graphics_qemu_observation_marker_not_observed=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_record_capture=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_pass_report_observed=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_pass_report_evidence_recorded=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_waiting=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_positive_observed=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_preflight_verified=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_capture_hash_verified=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_positive_preflight_verified=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_positive_capture_hash_verified=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_preflight_rejected=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_preflight_verified=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_preflight_unverified=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_capture_hash_rejected=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_capture_hash_status=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_evidence_bundle_apply_missing_capture_hash_value=' scripts/write-image-manifest.sh
grep -Fq 'graphics_qemu_observation_positive_observed=' scripts/write-image-manifest.sh
grep -Fq 'graphics_qemu_observation_positive_evidence_recorded=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_pass_report_required=' scripts/write-image-manifest.sh
grep -Fq 'recovery_help_pass_report_after_apply=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_host=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_ready_capture_flow=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_evidence_required=' scripts/write-image-manifest.sh
grep -Fq 'qemu_visible_manual_runbook_pass_report_required=' scripts/write-image-manifest.sh
grep -Fq 'graphics_enable_gate_no_boot_graphics=' scripts/write-image-manifest.sh
grep -Fq 'output_plan=' scripts/write-image-manifest.sh
grep -Fq 'display_output_handoff=' scripts/write-image-manifest.sh
grep -Fq 'visible_preview_plan=' scripts/write-image-manifest.sh
grep -Fq 'visible_preview_export=' scripts/write-image-manifest.sh
grep -Fq 'display_output_handoff_frame_format=' scripts/write-image-manifest.sh
grep -Fq 'display_output_handoff_frame_checksum=' scripts/write-image-manifest.sh
grep -Fq 'display_activation_plan=' scripts/write-image-manifest.sh
grep -Fq 'display_activation_plan_can_activate=' scripts/write-image-manifest.sh
grep -Fq 'display_output_smoke=' scripts/write-image-manifest.sh
grep -Fq 'display_output_smoke_started=' scripts/write-image-manifest.sh
grep -Fq 'nested_output_surface=' scripts/write-image-manifest.sh
grep -Fq 'nested_output_surface_frame_presented=' scripts/write-image-manifest.sh
grep -Fq 'nested_preview_loop=' scripts/write-image-manifest.sh
grep -Fq 'client_window_model=' scripts/write-image-manifest.sh
grep -Fq 'client_surface_lifecycle=' scripts/write-image-manifest.sh
grep -Fq 'client_surface_registry=' scripts/write-image-manifest.sh
grep -Fq 'client_surface_registry_two_client=' scripts/write-image-manifest.sh
grep -Fq 'client_surface_registry_stacking=' scripts/write-image-manifest.sh
grep -Fq 'xdg_shell_binding=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_client=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_shm_global=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_shm_import=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_shm_sample=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_configure_ack=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_close_event=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_client_count=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_window_model=' scripts/write-image-manifest.sh
grep -Fq 'smithay_launcher_seat=' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "smithay_launcher_seat_rootfs")' scripts/check-image-manifest.sh
grep -Fq 'xdg_toplevel_window_count=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_window_two_model=' scripts/write-image-manifest.sh
grep -Fq 'xdg_toplevel_window_stacking=' scripts/write-image-manifest.sh
grep -Fq 'paint_plan=' scripts/write-image-manifest.sh
grep -Fq 'paint_plan_probe.status=' scripts/write-image-manifest.sh
grep -Fq 'frame_plan=' scripts/write-image-manifest.sh
grep -Fq 'frame_plan_probe.status=' scripts/write-image-manifest.sh
grep -Fq 'frame_buffer=' scripts/write-image-manifest.sh
grep -Fq 'frame_buffer_probe.status=' scripts/write-image-manifest.sh
grep -Fq 'raster=' scripts/write-image-manifest.sh
grep -Fq 'raster_probe.status=' scripts/write-image-manifest.sh
grep -Fq 'raster_export=' scripts/write-image-manifest.sh
grep -Fq 'raster_export_probe.status=' scripts/write-image-manifest.sh
grep -Fq 'raster_png_export=' scripts/write-image-manifest.sh
grep -Fq 'raster_png_export_probe.status=' scripts/write-image-manifest.sh
grep -Fq 'MANIFEST_JSON=' scripts/write-image-manifest.sh
grep -Fq 'BOOT_SUMMARY=' scripts/write-image-manifest.sh
grep -Fq '"boot_summary"' scripts/write-image-manifest.sh
grep -Fq '"boot_markers"' scripts/write-image-manifest.sh
grep -Fq 'fbdev_device=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'fbdev_device=$(boot_summary_stage_status fbdev-device)' scripts/write-image-manifest.sh
grep -Fq 'Aqua Linux boot summary contract' scripts/report-artifacts.sh
grep -Fq 'output handoff' scripts/report-artifacts.sh
grep -Fq 'drm device probe' scripts/report-artifacts.sh
grep -Fq 'nested output surface' scripts/report-artifacts.sh
grep -Fq 'nested preview loop' scripts/report-artifacts.sh
grep -Fq '("boot_summary", "status")' scripts/check-image-manifest.sh
grep -Fq '("boot_markers", "fbdev_device")' scripts/check-image-manifest.sh
grep -Fq '("boot_summary", "fbdev_device")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "manual_nested_preview_execution")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "graphics_drm_qemu_probe")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "graphics_drm_no_modeset")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "manual_nested_preview_execution_no_boot_graphics")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "visible_preview_request")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "visible_preview_request_no_boot_graphics")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "visible_preview_launch")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "visible_preview_launch_no_boot_graphics")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help_no_boot_graphics")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help_operator_pass_host")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help_operator_pass_no_launch")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help_operator_checklist")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help_operator_pass_artifact")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help_operator_pass_external")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "recovery_help_visible_pass_report")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "operator_transcript")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "operator_transcript_no_boot_graphics")' scripts/check-image-manifest.sh
grep -Fq 'Aqua Linux progress report checks passed.' scripts/check-progress-report.sh
grep -Fq 'progress.json' scripts/write-progress-report.sh
grep -Fq 'progress.md' scripts/write-progress-report.sh
test -f docs/aqua-linux/compositor-foundation.toml
test -f docs/aqua-linux/progress.json
test -f docs/aqua-linux/progress.md
grep -Fq '"overallPercent":' docs/aqua-linux/progress.json
grep -Fq 'probe-drm-device' crates/aqua-compositor/src/main.rs
grep -Fq 'drm-device-probe.txt' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'device_open_mode=read-only' scripts/check-fbdev-presenter-qemu.sh
grep -Fq 'drm_device_probe = "recovery-safe read-only DRM card discovery' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'drm_dumb_buffer_probe = "real QEMU virtio-gpu allocates and maps' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'probe-drm-dumb-buffer' crates/aqua-compositor/src/main.rs
grep -Fq '("scene_contract", "graphics_drm_dumb_buffer")' scripts/check-image-manifest.sh
grep -Fq 'DRM dumb buffer checksum' scripts/report-artifacts.sh
grep -Eq '"overallPercent": [0-9]{1,3}' docs/aqua-linux/progress.json
python3 - <<'PY'
import json
from pathlib import Path

data = json.loads(Path("docs/aqua-linux/progress.json").read_text())
phases = {phase["id"]: phase for phase in data["phases"]}
assert phases["m2"]["percent"] == 100
assert phases["m2"]["status"] == "complete"
assert phases["m4"]["percent"] == 100
assert phases["m7"]["percent"] == 100
assert phases["m8"]["percent"] == 100
assert phases["m8"]["status"] == "complete"
assert phases["m9"]["percent"] == 100
assert phases["m9"]["status"] == "complete"
assert phases["m11"]["percent"] == 100
assert phases["m11"]["status"] == "complete"
assert phases["m12"]["percent"] == 100
assert phases["m12"]["status"] == "complete"
PY
test -f crates/aqua-installer/Cargo.toml
test -f crates/aqua-installer/src/lib.rs
test -f crates/aqua-installer/src/main.rs
test -f docs/aqua-linux/installer.md
test -x scripts/check-installer-render.sh
grep -Fq 'export-installer' scripts/check-installer-render.sh
grep -Fq 'installer_logo_rendered=true' scripts/check-installer-render.sh
grep -Fq 'installer_step=language' scripts/check-installer-render.sh
grep -Fq 'installer_step=keyboard' scripts/check-installer-render.sh
grep -Fq 'installer_step=partitions' scripts/check-installer-render.sh
grep -Fq 'installer_step=time-zone' scripts/check-installer-render.sh
grep -Fq 'installer_step=user-information' scripts/check-installer-render.sh
grep -Fq 'installer_step=summary' scripts/check-installer-render.sh
grep -Fq '"crates/aqua-installer"' Cargo.toml
grep -Fq 'pub enum InstallerStep' crates/aqua-installer/src/lib.rs
grep -Fq 'ERASE {}' crates/aqua-installer/src/lib.rs
grep -Fq 'pub fn probe_storage' crates/aqua-installer/src/lib.rs
grep -Fq 'RunningSystemDisk' crates/aqua-installer/src/lib.rs
grep -Fq 'pub fn build_dry_run_plan' crates/aqua-installer/src/lib.rs
grep -Fq 'execution_allowed=false' crates/aqua-installer/src/lib.rs
grep -Fq 'pub enum BootloaderStrategy' crates/aqua-installer/src/lib.rs
grep -Fq 'Grub2X86_64Efi' crates/aqua-installer/src/lib.rs
grep -Fq 'BOOTX64.EFI' crates/aqua-installer/src/lib.rs
grep -Fq 'pub fn validate_install_prerequisites' crates/aqua-installer/src/lib.rs
grep -Fq 'pub fn revalidate_install_target' crates/aqua-installer/src/lib.rs
grep -Fq 'pub fn compile_install_commands' crates/aqua-installer/src/lib.rs
grep -Fq 'pub struct NonExecutingInstallCommandRunner' crates/aqua-installer/src/lib.rs
grep -Fq 'pub const fn executed(&self) -> bool' crates/aqua-installer/src/lib.rs
grep -Fq 'pub fn compile_internal_install_actions' crates/aqua-installer/src/lib.rs
grep -Fq 'pub struct NonExecutingInternalInstallRunner' crates/aqua-installer/src/lib.rs
grep -Fq 'pub fn build_install_transaction_graph' crates/aqua-installer/src/lib.rs
grep -Fq 'pub struct NonExecutingInstallTransactionRunner' crates/aqua-installer/src/lib.rs
grep -Fq 'InstallCleanupRequirement::EfiMounted' crates/aqua-installer/src/lib.rs
grep -Fq 'pub struct FixtureInstallRoot' crates/aqua-installer/src/lib.rs
grep -Fq 'pub struct FixtureInternalInstallExecutor' crates/aqua-installer/src/lib.rs
grep -Fq 'stage=readiness-probe status=ok executed=false' crates/aqua-installer/src/main.rs
grep -Fq 'readiness_target_source=synthetic-readiness' crates/aqua-installer/src/main.rs
grep -Fq 'QEMU_DISPOSABLE_TARGET_ONLY' crates/aqua-installer/src/main.rs
grep -Fq 'transaction_execution_started=false' crates/aqua-installer/src/main.rs
grep -Fq 'execution_gate_artifact_manifest_verified=true' crates/aqua-installer/src/main.rs
grep -Fq 'pub const fn disk_commands_executed(&self) -> bool' crates/aqua-installer/src/lib.rs
grep -Fq 'pub struct FixtureToolShimRoot' crates/aqua-installer/src/lib.rs
grep -Fq 'pub struct FixtureToolShimRunner' crates/aqua-installer/src/lib.rs
grep -Fq 'pub const fn real_disk_tools_executed(&self) -> bool' crates/aqua-installer/src/lib.rs
grep -Fq 'pub const FIXTURE_TOOL_SHIM_TIMEOUT: Duration = Duration::from_secs(2)' crates/aqua-installer/src/lib.rs
grep -Fq 'BR2_PACKAGE_UTIL_LINUX_BINARIES=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_DOSFSTOOLS_MKFS_FAT=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_E2FSPROGS=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_TAR=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_TARGET_GRUB2_X86_64_EFI=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'CONFIG_EFI=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'root=PARTLABEL=AQUA_ROOT' br2-external/aqua/board/aqua/x86_64/grub.cfg
test -x br2-external/aqua/board/aqua/x86_64/post-image.sh
grep -Fq 'Missing executable installer prerequisite' br2-external/aqua/board/aqua/x86_64/post-image.sh
test -f docs/aqua-linux/adr-0002-bootloader.md
grep -Fq 'installer_storage_probe = "bounded read-only Linux inventory' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_dry_run_plan = "deterministic structured plan' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_prerequisites = "Buildroot packages sfdisk' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_target_revalidation = "a fresh bounded storage probe' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_command_rehearsal = "the canonical plan compiles without shell interpolation' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_internal_rehearsal = "eleven Rust-owned actions prepare target mountpoints' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_transaction_rehearsal = "a fingerprint-bound 20-step graph' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_fixture_executor = "an empty real directory under the system temporary root' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_tool_shim_runner = "only executable non-symlink programs below a temporary capability root' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'Changing the target or install mode invalidates prior confirmation.' docs/aqua-linux/installer.md
grep -Fq 'Applications and Global Search are separate centered modes' docs/aqua-linux/progress.json
grep -Fq 'pub enum LauncherMode' crates/aqua-shell/src/lib.rs
grep -Fq 'LauncherMode::Applications' crates/aqua-renderer/src/lib.rs
grep -Fq 'LauncherMode::Search' crates/aqua-renderer/src/lib.rs
grep -Fq 'pub enum BottomShellTarget' crates/aqua-shell/src/lib.rs
grep -Fq 'desktop_bottom_shell_group_count' crates/aqua-compositor/src/main.rs
grep -Fq 'bottom-applications-activate' scripts/send-qemu-monitor-input.py
grep -Fq 'pub const LIGHTWHITE_WINDOW_CHROME' crates/aqua-renderer/src/lib.rs
grep -Fq 'fn draw_bright_window_titlebar' crates/aqua-renderer/src/lib.rs
grep -Fq 'first_party_bright_window_chrome' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'Notifications, real system overview' docs/aqua-linux/progress.json
grep -Fq 'notification_center = "bounded FIFO notification state' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'desktop-notification-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'system_overview = "persistent shell model reads bounded Aqua Linux identity' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'desktop-system-overview-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'desktop_icons = "persistent Files, Settings, and Trash icon state' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'desktop-icons-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'desktop_properties = "packaged Aqua Properties is a supervised 480x300 xdg-toplevel' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'desktop-properties-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'desktop-properties-refresh-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'desktop-input-burst-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'aqua_shell::SESSION_MENU_RUNTIME_WIDTH' crates/aqua-compositor/src/main.rs
grep -Fq 'aqua_shell::SESSION_MENU_RUNTIME_HEIGHT' crates/aqua-compositor/src/main.rs
grep -Fq 'session_menu_visual = "the current session menu uploads a native 512x293' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'desktop-properties-close-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'aqua-properties' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-terminal' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'portable-pty = "0.9.0"' crates/aqua-compositor/Cargo.toml
grep -Fq 'vt100 = "0.16.2"' crates/aqua-compositor/Cargo.toml
test -x scripts/check-terminal-qemu.sh
test -x scripts/check-terminal-qemu.exp
grep -Fq 'stage=pty-probe status=ok' crates/aqua-compositor/src/lib.rs
grep -Fq '"id": "m5"' docs/aqua-linux/progress.json
grep -Fq 'upstream Weston simple-shm C reference client' docs/aqua-linux/progress.json
grep -Fq 'weston-simple-shm' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq '"${TARGET_DIR}/usr/lib/libweston-"*' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'rounded arithmetic mean of the 13 M0-M12 phase percentages' docs/aqua-linux/progress.json
test -f docs/aqua-linux/adr-0003-aqua-ui-framework.md
grep -Fq 'Status' docs/aqua-linux/adr-0003-aqua-ui-framework.md
grep -Fq 'Accepted on 2026-08-29' docs/aqua-linux/adr-0003-aqua-ui-framework.md
grep -Fq 'aqua-text' docs/aqua-linux/adr-0003-aqua-ui-framework.md
grep -Fq 'aqua-components' docs/aqua-linux/adr-0003-aqua-ui-framework.md
grep -Fq 'aqua-renderer' docs/aqua-linux/adr-0003-aqua-ui-framework.md
grep -Fq 'Post-M12: Aqua UI Framework Consolidation' docs/aqua-linux/milestones.md
grep -Fq 'ADR 0003' docs/aqua-linux/ui-contract.md
test -f docs/aqua-linux/adr-0004-audio-service-stack.md
grep -Fq 'Accepted on 2026-08-29' docs/aqua-linux/adr-0004-audio-service-stack.md
grep -Fq 'ADR 0004' docs/aqua-linux/v1-readiness.md
test -x scripts/check-audio-service-architecture.sh
scripts/check-audio-service-architecture.sh
test -f docs/aqua-linux/adr-0005-network-service-stack.md
grep -Fq 'Accepted on 2026-08-31' docs/aqua-linux/adr-0005-network-service-stack.md
grep -Fq 'ADR 0005' docs/aqua-linux/v1-readiness.md
test -x scripts/check-network-service-architecture.sh
scripts/check-network-service-architecture.sh
test -x scripts/check-network-qemu.sh
test -x scripts/check-network-qemu.exp
grep -Fq 'aqua.boot_network=1' scripts/check-network-qemu.exp
grep -Fq 'stage=qemu-acceptance status=ok' scripts/check-network-qemu.sh
scripts/check-audio-buildroot-rehearsal.sh
scripts/check-wifi-buildroot-rehearsal.sh
scripts/check-wifi-control-contract.sh
scripts/check-wifi-native-binding.sh
scripts/check-wifi-service-architecture.sh
scripts/check-audio-native-binding.sh
scripts/check-audio-rootfs-contract.sh
scripts/check-audio-qemu-device-contract.sh
test -x scripts/build-audio-adapter-probe-linux-docker.sh
test -x scripts/audio-buildroot-linker.sh
test -x scripts/check-audio-control-submission-budget-qemu.sh
test -x scripts/check-audio-control-route-loss-qemu.sh
test -x scripts/check-audio-mute-route-loss-qemu.sh
grep -Fq 'control-submission-budget' scripts/check-audio-qemu.sh
grep -Fq 'control-route-loss' scripts/check-audio-qemu.sh
grep -Fq 'mute-route-loss' scripts/check-audio-qemu.sh
grep -Fq 'stage=qemu-control-submission-budget status=ok' scripts/check-audio-qemu.exp
grep -Fq 'stage=qemu-control-route-loss status=ok' scripts/check-audio-qemu.exp
grep -Fq 'stage=qemu-mute-route-loss status=ok' scripts/check-audio-qemu.exp
grep -Fq 'target_output' crates/aqua-service-adapters/src/lib.rs
grep -Fq 'has_snapshot' br2-external/aqua/package/aqua-audio-native/src/aqua_audio_native.c
grep -Fq 'snapshot_payload_equal' br2-external/aqua/package/aqua-audio-native/src/aqua_audio_native.c
grep -Fq 'aqua-audio-adapter-probe' br2-external/aqua/audio-rootfs-overlay/usr/bin/aqua-audio-rootfs-check
test -x scripts/check-buildroot-lts.sh
scripts/check-buildroot-lts.sh
grep -Fq 'Aqua Linux v1.0 readiness is governed separately by the mandatory gates in docs/aqua-linux/v1-readiness.md.' docs/aqua-linux/progress.json
grep -Fq '"dailyUseReady": false' docs/aqua-linux/progress.json
grep -Fq '"hardwareProven": false' docs/aqua-linux/progress.json
grep -Fq '"releaseReady": false' docs/aqua-linux/progress.json
test -f docs/aqua-linux/v1-readiness.md
test -f docs/aqua-linux/ui-contract.md
grep -Fxq '/docs/aqua-linux/local-references/' .gitignore
test -z "$(find docs/aqua-linux/assets -maxdepth 1 -name 'reference-*.png' -print)"
grep -Fq 'The bottom area is split into launcher/search controls, a centered running-app dock, and workspace thumbnails.' docs/aqua-linux/visual-reference.md
grep -Fq 'Theme selection for LightWhite, Softtouch, Deepside, or Nightmare.' docs/aqua-linux/visual-reference.md
grep -Fq 'runningAppDock": "bottom-center"' docs/aqua-linux/design-tokens.json
grep -Fq 'Third-party icons shown in references are composition examples only.' docs/aqua-linux/interface-style.md
grep -Fq '## Elevation And Shadows' docs/aqua-linux/interface-style.md
grep -Fq '## Scalable Iconography' docs/aqua-linux/interface-style.md
grep -Fq '## Motion' docs/aqua-linux/interface-style.md
grep -Fq 'unicodeShapingRequired' docs/aqua-linux/design-tokens.json
grep -Fq 'scaleNativeRasterization' docs/aqua-linux/design-tokens.json
grep -Fq 'frameCallbackDriven' docs/aqua-linux/design-tokens.json
grep -Fq 'stableLayoutAcrossStates' docs/aqua-linux/design-tokens.json
grep -Fq '## Visual Fidelity Acceptance' docs/aqua-linux/ui-contract.md
grep -Fq '## Milestone 12: Visual Fidelity And Component System' docs/aqua-linux/milestones.md
grep -Fq 'pub enum AquaTheme' crates/aqua-shell/src/lib.rs
grep -Fq 'pub const NIGHTMARE_WINDOW_CHROME' crates/aqua-renderer/src/lib.rs
grep -Fq 'render_files_window_rgba_with_theme' crates/aqua-renderer/src/lib.rs
grep -Fq 'runtime_theme_palettes = "AquaTheme bounds LightWhite, Softtouch, Deepside, and Nightmare' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'runtime_shell_theme_palettes = "the compositor applies one shared ShellPalette' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'pub struct ShellPalette' crates/aqua-renderer/src/lib.rs
grep -Fq 'render_launcher_overlay_rgba_with_theme' crates/aqua-renderer/src/lib.rs
grep -Fq 'render_installer_window_rgba_with_theme' crates/aqua-renderer/src/lib.rs
grep -Fq 'aqua_installer_theme=' crates/aqua-compositor/src/lib.rs
grep -Fq 'installer_theme_palettes = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'runtime_live_theme_broadcast = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'runtime_live_theme_qemu = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'desktop_runtime_theme_broadcast=' crates/aqua-compositor/src/main.rs
grep -Fq 'aqua_runtime_theme_changed=' crates/aqua-compositor/src/lib.rs
grep -Fq 'desktop-live-theme-qemu status=ok' scripts/check-graphical-boot-qemu.sh
grep -Fq 'desktop-live-theme-qemu status=ok' scripts/check-live-theme-qemu.sh
grep -Fq 'check-qemu-theme-frame-delta.py' scripts/check-graphical-boot-qemu.sh
test -x scripts/check-live-theme-qemu.sh
test -x scripts/check-live-theme-qemu.exp
test -x scripts/check-qemu-theme-frame-delta.py
test -x scripts/check-workspaces-qemu.sh
test -x scripts/check-workspaces-qemu.exp
grep -Fq 'desktop-workspaces-qemu status=ok' scripts/check-workspaces-qemu.sh
grep -Fq 'workspace_window_assignment = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'workspace_input = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'workspace_qemu = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'workspace-move-right' scripts/send-qemu-monitor-input.py
grep -Fq 'workspace-switch-right' scripts/send-qemu-monitor-input.py
test -x scripts/check-public-runtime-qemu.sh
test -x scripts/check-public-runtime-qemu.exp
test -x scripts/publish-public-runtime-screenshots.sh
test -x scripts/check-public-runtime-screenshots.sh
grep -Fq 'desktop-public-runtime-qemu status=ok' scripts/check-public-runtime-qemu.sh
grep -Fq 'public_runtime_screenshots = ' docs/aqua-linux/compositor-foundation.toml
if grep -E '!\[[^]]*\]\([^)]*\)' README.md |
    grep -Ev 'actions/workflows/ci\.yml/badge\.svg|img\.shields\.io/badge/'; then
    echo 'README may embed approved status badges only; runtime screenshots are not allowed.' >&2
    exit 1
fi
grep -Fq 'These images are captures of the current Aqua Linux runtime in QEMU' docs/aqua-linux/runtime-screenshots.md
scripts/check-public-runtime-screenshots.sh
test -x scripts/check-hardware-support-status.sh
scripts/check-hardware-support-status.sh
test -x br2-external/aqua/rootfs-overlay/usr/bin/aqua-hardware-inventory
test -x scripts/check-hardware-inventory.sh
scripts/check-hardware-inventory.sh
grep -Fq 'desktop_shell_theme=' crates/aqua-compositor/src/main.rs
grep -Fq 'theme=LightWhite' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'run-manual-nested-preview-execution' docs/aqua-linux/compositor.md
grep -Fq 'aqua-visible-preview-request' docs/aqua-linux/compositor.md
grep -Fq 'aqua-visible-preview-launch' docs/aqua-linux/compositor.md
grep -Fq 'aqua-recovery-help' docs/aqua-linux/compositor.md
grep -Fq 'handoff-summary' docs/aqua-linux/compositor.md
grep -Fq 'aqua-operator-transcript' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-enable-gate' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-rollback-drill' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-startup-preflight' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-startup-rehearsal' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-qemu-display-gate' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-visible-qemu-attempt' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-visible-attempt-transcript' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-visible-attempt-result' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-visible-attempt-runner' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-qemu-visible-boot-check' docs/aqua-linux/compositor.md
grep -Fq 'aqua-graphics-qemu-observation-marker' docs/aqua-linux/compositor.md
grep -Fq 'AQUA_QEMU_VM_DISPLAY_OBSERVED=true' docs/aqua-linux/compositor.md
grep -Fq 'aqua-qemu-visible-evidence-record' docs/aqua-linux/compositor.md
grep -Fq 'aqua-qemu-visible-pass-report' docs/aqua-linux/compositor.md
grep -Fq 'scripts/run-qemu-visible-manual.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/preflight-qemu-visible-manual.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/watch-qemu-visible-readiness.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/run-qemu-visible-ready-capture-flow.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/qemu-visible-status.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/write-qemu-visible-operator-plan.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/write-qemu-visible-operator-packet.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/write-qemu-visible-operator-checklist.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/run-qemu-visible-operator-pass.sh' docs/aqua-linux/compositor.md
grep -Fq 'no-launch rehearsal command' docs/aqua-linux/compositor.md
grep -Fq 'evidence flow' docs/aqua-linux/compositor.md
grep -Fq 'qemu-visible-manual-preflight.json' docs/aqua-linux/compositor.md
grep -Fq 'qemu-visible-manual-preflight.txt' docs/aqua-linux/compositor.md
grep -Fq 'scripts/capture-qemu-visible-manual.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/verify-qemu-visible-capture.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/write-qemu-visible-evidence-bundle.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/prepare-qemu-visible-evidence-apply.sh' docs/aqua-linux/compositor.md
grep -Fq 'scripts/run-qemu-visible-evidence-flow.sh' docs/aqua-linux/compositor.md
grep -Fq 'preflight_summary_verified=true' docs/aqua-linux/compositor.md
grep -Fq 'aqua-qemu-visible-manual-runbook' docs/aqua-linux/compositor.md
grep -Fq 'probe-manual-execution-window-bridge' docs/aqua-linux/compositor.md
grep -Fq 'smoke-manual-execution-window' docs/aqua-linux/compositor.md
grep -Fq 'manual_nested_preview_execution = "rootfs-verified operator-controlled manual nested preview execution' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'manual_execution_window_bridge = "feature-gated aqua-host-tools bridge' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'visible_preview_request = "QEMU-safe recovery command records a manual host-visible preview request' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'visible_preview_launch = "QEMU-safe recovery launcher turns the manual host-visible preview request into a bounded launch plan' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_dev_handoff_summary = "host-side aqua-host-tools summary pairs the rootfs-exported visible-preview launcher artifact' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'operator_transcript = "QEMU recovery dry-run transcript writes the ordered manual recovery command sequence' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_launch_candidate = "QEMU recovery supervised no-start launch candidate' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_rollback_drill = "QEMU recovery supervised rollback drill' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_startup_preflight = "QEMU recovery guarded startup preflight' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_startup_rehearsal = "QEMU recovery guarded startup rehearsal' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_qemu_display_gate = "QEMU recovery manual QEMU display-step gate' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_visible_qemu_attempt = "QEMU recovery first visible QEMU compositor attempt plan' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_visible_attempt_transcript = "QEMU recovery manual visible-attempt execution transcript' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_visible_attempt_result = "QEMU recovery visible-attempt result collector' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_visible_attempt_runner = "QEMU recovery explicit visible-attempt runner' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_qemu_visible_boot_check = "QEMU recovery first QEMU-visible boot path check' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_qemu_observation_marker = "QEMU recovery manual VM-display observation marker' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'graphics_qemu_observation_positive = "QEMU recovery positive VM-display observation dry-run' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'qemu_visible_evidence_record = "QEMU visible evidence record' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_capture = "Host-side scripts/capture-qemu-visible-manual.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_preflight = "Host-side scripts/preflight-qemu-visible-manual.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_readiness_watch = "Host-side scripts/watch-qemu-visible-readiness.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_ready_capture_flow = "Host-side scripts/run-qemu-visible-ready-capture-flow.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_capture_verify = "Host-side scripts/verify-qemu-visible-capture.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_evidence_bundle = "Host-side scripts/write-qemu-visible-evidence-bundle.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_evidence_apply_prep = "Host-side scripts/prepare-qemu-visible-evidence-apply.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_qemu_visible_evidence_flow = "Host-side scripts/run-qemu-visible-evidence-flow.sh' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'qemu_visible_manual_runbook = "QEMU manual VM-display runbook' docs/aqua-linux/compositor-foundation.toml
test -f docs/aqua-linux/adr-0001-compositor-foundation.md

grep -Fq 'NAME="Aqua Linux"' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'ID=aqua' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq '/usr/share/aqua/wallpapers' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq '/usr/share/aqua/icons/aqua' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq '/usr/share/doc/aqua/third-party-licenses.md' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'target/x86_64-unknown-linux-musl/release/aqua-compositor' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-compositor-preview-exec' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-visible-preview-request' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-visible-preview-launch' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-recovery-help' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-operator-transcript' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-enable-gate' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-launch-candidate' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-rollback-drill' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-startup-preflight' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-startup-rehearsal' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-qemu-display-gate' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-visible-qemu-attempt' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-visible-attempt-transcript' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-visible-attempt-result' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-visible-attempt-runner' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-qemu-visible-boot-check' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-graphics-qemu-observation-marker' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-qemu-visible-pass-report' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-qemu-visible-evidence-bundle-apply' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq '/etc/aqua/compositor-session.conf' br2-external/aqua/rootfs-overlay/usr/bin/aqua-recovery
grep -Fq 'wayland_socket=aqua-wayland-0' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'runtime_dir=/run/user/1000' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'runtime_asset_root=/usr/share/aqua' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'autostart=false' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'boot_graphics=false' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'recovery_tty_required=true' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'export WAYLAND_DISPLAY=aqua-wayland-0' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'export XDG_RUNTIME_DIR=/run/user/1000' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'export AQUA_ASSET_ROOT=/usr/share/aqua' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'export XCOMPOSEFILE=/usr/share/aqua/compose/Compose' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq '<dead_acute> <e> : "é" U00E9' br2-external/aqua/rootfs-overlay/usr/share/aqua/compose/Compose
grep -Fq '/etc/aqua/session.env' br2-external/aqua/rootfs-overlay/etc/profile
grep -Fq '/usr/bin/aqua-compositor status' br2-external/aqua/rootfs-overlay/usr/bin/aqua-recovery
grep -Fq '/usr/bin/aqua-session-check' br2-external/aqua/rootfs-overlay/usr/bin/aqua-recovery
grep -Fq "report session-check ok 'no_graphics=true'" br2-external/aqua/rootfs-overlay/usr/bin/aqua-session-check
grep -Fq 'AQUA_SESSION_ROOT' br2-external/aqua/rootfs-overlay/usr/bin/aqua-session-check
grep -Fq 'AQUA_SESSION_RUN_DIR' br2-external/aqua/rootfs-overlay/usr/bin/aqua-session-check
grep -Fq '[AQUA-BOOT] stage=session-config status=ok autostart=false boot_graphics=false recovery_tty=true' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=session-runtime status=ok user=aqua uid=1000 runtime_dir=/run/user/1000 control_dir=/run/aqua mode=0700' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=session-env status=ok wayland=aqua-wayland-0 xdg=/run/user/1000 assets=/usr/share/aqua' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=compositor-binary status=packaged autostart=false boot_graphics=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=compositor-status status=ok mode=nested-dev' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=session-bootstrap status=ok runtime_dir=/run/user/1000 autostart=false boot_graphics=false session_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=compositor-assets status=ok root=/usr/share/aqua' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=output-plan status=ok backend=nested-dev-window boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=visible-preview-plan status=ok preview_window_started=false boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=scene-contract status=ok surfaces=7 boot_graphics=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=render-plan status=ok commands=7 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=paint-plan status=ok steps=7 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=frame-plan status=ok format=rgba8888 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=frame-buffer status=ok bytes=6291456 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=raster status=ok rects=7 surface_layers=15 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=surface-primitives status=ok layers=15 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=raster-export status=ok bytes=4718609 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=raster-png-export status=ok bytes=6293028 boot_graphics=false renderer_started=false' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=session-check status=ok no_graphics=true' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=session-env status=ok wayland=aqua-wayland-0 xdg=/run/user/1000 assets=/usr/share/aqua' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=compositor-binary status=packaged autostart=false boot_graphics=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=compositor-status status=ok mode=nested-dev' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=session-bootstrap status=ok runtime_dir=/run/user/1000 autostart=false boot_graphics=false session_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=compositor-assets status=ok root=/usr/share/aqua' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=output-plan status=ok backend=nested-dev-window boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=visible-preview-plan status=ok preview_window_started=false boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=scene-contract status=ok surfaces=7 boot_graphics=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=render-plan status=ok commands=7 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=paint-plan status=ok steps=7 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=frame-plan status=ok format=rgba8888 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=frame-buffer status=ok bytes=6291456 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=raster status=ok rects=7 surface_layers=15 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=surface-primitives status=ok layers=15 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=raster-export status=ok bytes=4718609 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=raster-png-export status=ok bytes=6293028 boot_graphics=false renderer_started=false' scripts/check-boot.sh
grep -Fq '[AQUA-BOOT] stage=session-check status=ok no_graphics=true' scripts/check-boot.sh
grep -Fq 'session_config_recovery_safe=' scripts/write-image-manifest.sh
grep -Fq 'session_env_recovery_safe=' scripts/write-image-manifest.sh
grep -Fq 'session_runtime=' scripts/write-image-manifest.sh
grep -Fq 'compositor_status=' scripts/write-image-manifest.sh
grep -Fq 'session_bootstrap=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'session_check_probe=$(contract_file_contains' scripts/write-image-manifest.sh
grep -Fq 'compositor_assets=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'output_plan=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'visible_preview_plan=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'scene_contract=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'render_plan=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'paint_plan=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'frame_plan=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'frame_buffer=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'raster=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'raster_export=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'raster_png_export=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'session_check=$(marker_status' scripts/write-image-manifest.sh
grep -Fq 'boot_graphics=false' scripts/write-image-manifest.sh
grep -Fq '[scene_contract]' scripts/write-image-manifest.sh
grep -Fq 'design_tokens_scene_materials=' scripts/write-image-manifest.sh
grep -Fq 'contract_file_contains' scripts/write-image-manifest.sh
grep -Fq 'render_plan=' scripts/write-image-manifest.sh
grep -Fq 'paint_plan=' scripts/write-image-manifest.sh
grep -Fq 'frame_plan=' scripts/write-image-manifest.sh
grep -Fq 'frame_buffer=' scripts/write-image-manifest.sh
grep -Fq 'raster=' scripts/write-image-manifest.sh
grep -Fq 'raster_checksum=' scripts/write-image-manifest.sh
grep -Fq 'surface_primitives=' scripts/write-image-manifest.sh
grep -Fq 'raster_surface_border_sample=' scripts/write-image-manifest.sh
grep -Fq 'raster_surface_highlight_sample=' scripts/write-image-manifest.sh
grep -Fq 'raster_surface_corner_sample=' scripts/write-image-manifest.sh
grep -Fq 'raster_surface_shadow_sample=' scripts/write-image-manifest.sh
grep -Fq 'raster_export=' scripts/write-image-manifest.sh
grep -Fq 'raster_png_export=' scripts/write-image-manifest.sh
grep -Fq 'renderer=aqua-renderer' scripts/check-compositor.sh
grep -Fq 'probe-session-env' scripts/check-compositor.sh
grep -Fq 'probe-session-bootstrap' scripts/check-compositor.sh
grep -Fq 'probe-output-plan' scripts/check-compositor.sh
grep -Fq 'probe-visible-preview-plan' scripts/check-compositor.sh
grep -Fq 'probe-visible-preview-export' scripts/check-compositor.sh
grep -Fq 'smoke-nested-preview-loop' scripts/check-compositor.sh
grep -Fq 'probe-manual-nested-preview-backend' scripts/check-compositor.sh
grep -Fq 'probe-client-window-model' scripts/check-compositor.sh
grep -Fq 'probe-client-surface-lifecycle' scripts/check-compositor.sh
grep -Fq 'probe-client-surface-registry' scripts/check-compositor.sh
grep -Fq 'buffer_metadata_ready=ok' scripts/check-compositor.sh
grep -Fq 'buffer_import_plan_ready=ok' scripts/check-compositor.sh
grep -Fq 'probe-renderer-surface-sources' scripts/check-compositor.sh
grep -Fq 'probe-client-layer-pipeline' scripts/check-compositor.sh
grep -Fq 'sample_pixel=' scripts/check-compositor.sh
grep -Fq 'renderer_surface_sources=' scripts/write-image-manifest.sh
grep -Fq 'client_layer_pipeline=' scripts/write-image-manifest.sh
grep -Fq 'renderer_gpu_client_textures_composited' scripts/write-image-manifest.sh
grep -Fq 'graphics_drm_gpu_surface_client_not_live' scripts/write-image-manifest.sh
grep -Fq 'graphics_drm_wayland_gpu_live_composited' scripts/write-image-manifest.sh
grep -Fq 'graphics_drm_wayland_gpu_context_reused' scripts/write-image-manifest.sh
grep -Fq 'graphics_drm_wayland_gpu_repaint_checksum' scripts/write-image-manifest.sh
grep -Fq 'graphics_drm_wayland_gpu_full_repaint_route' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "graphics_drm_wayland_gpu_full_repaint_route")' scripts/check-image-manifest.sh
grep -Fq 'probe-drm-gbm-scanout-buffer' crates/aqua-compositor/src/main.rs
grep -Fq 'graphics_drm_gbm_addfb2' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "graphics_drm_gbm_addfb2")' scripts/check-image-manifest.sh
grep -Fq 'present-drm-gbm-scanout' crates/aqua-compositor/src/main.rs
grep -Fq 'graphics_drm_gbm_direct_scanout' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "graphics_drm_gbm_direct_scanout")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "renderer_surface_sources")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "client_layer_pipeline")' scripts/check-image-manifest.sh
grep -Fq 'client_surface_registry_buffer_metadata=' scripts/write-image-manifest.sh
grep -Fq 'client_surface_registry_buffer_import_plan=' scripts/write-image-manifest.sh
grep -Fq '("scene_contract", "client_surface_registry_buffer_metadata")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "client_surface_registry_buffer_import_plan")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "manual_launch_plan")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "manual_launch_no_display_start")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "guarded_run")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "guarded_run_return")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "handoff_gate")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "handoff_gate_no_auto")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "handoff_gate_backend")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "manual_nested_preview_backend")' scripts/check-image-manifest.sh
grep -Fq '("scene_contract", "manual_nested_preview_backend_no_start")' scripts/check-image-manifest.sh
grep -Fq 'probe-xdg-shell-binding' scripts/check-compositor.sh
grep -Fq 'probe-xdg-toplevel-client' scripts/check-compositor.sh
grep -Fq 'probe-selection-ownership' scripts/check-compositor.sh
grep -Fq 'probe-drag-and-drop' scripts/check-compositor.sh
grep -Fq 'probe-text-input' scripts/check-compositor.sh
grep -Fq 'probe-keyboard-locale-matrix' scripts/check-compositor.sh
grep -Fq 'probe-independent-application-matrix' scripts/check-compositor.sh
grep -Fq 'probe-privileged-protocol-boundary' scripts/check-compositor.sh
grep -Fq 'probe-v1-client-buffer-contract' scripts/check-compositor.sh
grep -Fq 'probe-wayland-output-matrix' scripts/check-compositor.sh
grep -Fq 'text-input-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'keyboard-locale-matrix-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'independent-application-matrix-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'weston-simple-damage' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'weston-simple-damage' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'weston-simple-touch' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'weston-simple-touch' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'weston-terminal' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'weston-terminal' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-glfw-wayland-probe' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'BR2_PACKAGE_AQUA_GLFW_WAYLAND_PROBE=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'GLFW_PLATFORM_WAYLAND' br2-external/aqua/package/aqua-glfw-wayland-probe/src/aqua_glfw_wayland_probe.c
grep -Fq 'metadata.offset' crates/aqua-compositor/src/lib.rs
grep -Fq 'sign_close.png' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'compose_key_available_for_all_layouts=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'dead_key_utf8_matches_for_all_clients=true' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'cancelled_compose_rejected_for_all_locales=true' scripts/check-compositor.sh
grep -Fq 'privileged-protocol-boundary-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'v1-client-buffer-contract-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'wayland-output-matrix-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'popup-subsurface-matrix-probe.txt' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'declared_transform_count=4' scripts/check-compositor-rootfs-docker.sh
grep -Fq 'hotplug_add_reaches_both_clients=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'smithay_text_input_is_focus_and_authorization_safe' scripts/check-smithay-seat-docker.sh
grep -Fq 'smithay_keyboard_locale_matrix_delivers_compose_and_dead_keys' scripts/check-smithay-seat-docker.sh
grep -Fq 'smithay_privileged_protocol_boundary_is_narrow_and_unadvertised' scripts/check-smithay-seat-docker.sh
grep -Fq 'v1_client_buffer_contract_excludes_accelerated_clients' scripts/check-smithay-seat-docker.sh
grep -Fq 'smithay_output_matrix_is_discoverable_scaled_and_hotpluggable' scripts/check-smithay-seat-docker.sh
grep -Fq 'smithay_popup_and_subsurface_lifecycles_are_independent' scripts/check-smithay-seat-docker.sh
grep -Fq 'unfocused_clipboard_rejected=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'unfocused_primary_rejected=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'data_control_global_exposed=false' scripts/check-smithay-seat-docker.sh
grep -Fq 'privileged_wayland_boundary = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'keyboard_locale_matrix = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'independent_application_matrix = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'wayland_output_matrix = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'popup_subsurface_matrix = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'clipboard_payload_transferred=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'primary_payload_transferred=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'owner_disconnect_clears_clipboard=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'owner_disconnect_clears_primary=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'start_without_implicit_grab_rejected=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'enter_reaches_pointer_focus_only=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'copy_action_negotiated=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'rejected_drop_cancelled=true' scripts/check-smithay-seat-docker.sh
grep -Fq 'server_shm_buffer_sampled=ok' scripts/check-compositor.sh
grep -Fq 'probe-xdg-toplevel-window-model' scripts/check-compositor.sh
grep -Fq 'probe-launcher-model' scripts/check-compositor.sh
grep -Fq 'probe-launcher-input-scene' scripts/check-compositor.sh
grep -Fq 'probe-smithay-launcher-seat' scripts/check-compositor.sh
test -x scripts/check-smithay-seat-docker.sh
grep -Fq 'libxkbcommon-dev' scripts/check-smithay-seat-docker.sh
grep -Fq 'host_stub=false' scripts/check-smithay-seat-docker.sh
grep -Fq 'smithay_launcher_seat = "real Linux and packaged musl Buildroot probes' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor"' scripts/export-rootfs-compositor-contract-docker.sh
grep -Fq 'AQUA_COMPOSITOR_BIN:-$(root_path /usr/bin/aqua-compositor)' br2-external/aqua/rootfs-overlay/usr/bin/aqua-session-check
grep -Fq './usr/lib/libxkbcommon.so.0' scripts/check-compositor-packaged.sh
grep -Fq './usr/share/X11/xkb/rules/evdev' scripts/check-compositor-packaged.sh
grep -Fq './usr/bin/aqua-installer-probe' scripts/check-compositor-packaged.sh
grep -Fq 'execution_allowed=false' scripts/check-compositor-packaged.sh
grep -Fq 'storage_eligible_count=0' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_ui_status=keyboard-navigable-installer-window-contract-ready' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_ui_keyboard_navigation=true' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_form_status=validated-language-keyboard-form-controls-ready' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_language_option_count=3' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_keyboard_option_count=3' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_timezone_form_status=validated-timezone-form-control-ready' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_timezone_option_count=4' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_user_form_status=password-content-free-user-form-ready' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_user_password_content_stored=false' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_summary_form_status=target-bound-summary-confirmation-ready' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_summary_target_bound=true' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_disk_form_status=eligible-storage-selection-form-ready' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_disk_option_count=1' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_disk_eligible_count=0' scripts/check-installer-probe-qemu.sh
grep -Fq 'installer_ui_rendered=false' scripts/check-installer-probe-qemu.sh
grep -Fq 'disk_commands_executed=false' scripts/check-installer-probe-qemu.sh
grep -Fq 'filesystem_writes_executed=false' scripts/check-installer-probe-qemu.sh
grep -Fq 'before_sha256=' scripts/check-installer-target-selection-qemu.sh
grep -Fq 'after_sha256=' scripts/check-installer-target-selection-qemu.sh
grep -Fq 'readiness_target_device=/dev/vdb' scripts/check-installer-target-selection-qemu.sh
grep -Fq 'plan_target_device=/dev/vdb' scripts/check-installer-target-selection-qemu.sh
grep -Fq 'before_sha256=' scripts/check-installer-execution-gate-qemu.sh
grep -Fq 'after_sha256=' scripts/check-installer-execution-gate-qemu.sh
grep -Fq 'execution_gate_target_device=/dev/vdb' scripts/check-installer-execution-gate-qemu.sh
grep -Fq 'transaction_execution_started=false' scripts/check-installer-execution-gate-qemu.sh
grep -Fq 'artifact_before_sha256=' scripts/check-installer-execution-gate-qemu.sh
grep -Fq 'artifact_after_sha256=' scripts/check-installer-execution-gate-qemu.sh
grep -Fq 'AQUA_INSTALLER_QEMU_TRANSACTION_EXECUTE=ERASE_DISPOSABLE_VDB_NOW' scripts/check-installer-transaction-qemu.exp
grep -Fq 'transaction_execution_completed=true' scripts/check-installer-transaction-qemu.sh
grep -Fq 'state=completed phase=completed operation=complete completed=20 total=20 percent=100' scripts/check-installer-transaction-qemu.sh
grep -Fq 'target_changed=true' scripts/check-installer-transaction-qemu.sh
grep -Fq 'artifact_disk_unchanged=true' scripts/check-installer-transaction-qemu.sh
grep -Fq 'installer-installed-content status=ok' scripts/check-installer-transaction-qemu.sh
grep -Fq 'installer-installed-root-boot status=ok root=/dev/vda2' scripts/check-installer-transaction-qemu.sh
grep -Fq 'installed_uefi_boot=true' scripts/check-installer-transaction-qemu.sh
grep -Fq 'installer-installed-uefi-boot status=ok firmware=edk2 bootloader=grub root=PARTLABEL=AQUA_ROOT' scripts/check-installer-transaction-qemu.sh
grep -Fq 'if=pflash,format=raw,unit=0,readonly=on' scripts/check-installer-transaction-qemu.exp
grep -Fq 'root=PARTLABEL=AQUA_ROOT' scripts/check-installer-transaction-qemu.exp
grep -Fq 'AQUA_INSTALLER_QEMU_FAILURE_INJECT=AFTER_EFI_MOUNT' scripts/check-installer-cleanup-qemu.exp
grep -Fq 'state=failed phase=installing-bootloader operation=mount-efi-system-partition completed=8 total=20 percent=40' scripts/check-installer-cleanup-qemu.sh
grep -Fq 'transaction_cleanup_completed=/mnt/aqua-target/boot/efi' scripts/check-installer-cleanup-qemu.sh
grep -Fq 'transaction_cleanup_completed=/mnt/aqua-target' scripts/check-installer-cleanup-qemu.sh
grep -Fq 'installer-cleanup-unmounted status=ok efi=true root=true' scripts/check-installer-cleanup-qemu.sh
grep -Fq 'cleanup_order=efi,root' scripts/check-installer-cleanup-qemu.sh
grep -Fq 'AQUA_DRM_WAYLAND_SCENARIO=installer-welcome' scripts/check-installer-wayland-qemu.exp
grep -Fq 'installer_wayland_surface_ready=true' scripts/check-installer-wayland-qemu.sh
grep -Fq 'installer_wayland_shell_chrome_visible=false' scripts/check-installer-wayland-qemu.sh
grep -Fq 'qemu-installer-welcome.png' scripts/check-installer-wayland-qemu.sh
grep -Fq 'qemu-installer-keyboard.png' scripts/check-installer-wayland-qemu.sh
grep -Fq 'qemu-installer-partitions.png' scripts/check-installer-wayland-qemu.sh
grep -Fq 'qemu-installer-timezone.png' scripts/check-installer-wayland-qemu.sh
grep -Fq 'qemu-installer-summary.png' scripts/check-installer-wayland-qemu.sh
grep -Fq 'installer-welcome-language-keyboard' scripts/send-qemu-monitor-input.py
grep -Fq 'installer-keyboard-partitions' scripts/send-qemu-monitor-input.py
grep -Fq 'installer-partitions-timezone' scripts/send-qemu-monitor-input.py
grep -Fq 'installer-timezone-user' scripts/send-qemu-monitor-input.py
grep -Fq 'installer-user-summary-confirmation' scripts/send-qemu-monitor-input.py
grep -Fq '"sendkey ret 100"' scripts/send-qemu-monitor-input.py
grep -Fq 'aqua_installer_redraw_count=35' scripts/check-installer-wayland-qemu.sh
grep -Fq 'aqua_installer_keyboard_layout=trq' scripts/check-installer-wayland-qemu.sh
grep -Fq 'aqua_installer_target_device=/dev/vdb' scripts/check-installer-wayland-qemu.sh
grep -Fq 'aqua_installer_timezone=Europe/Istanbul' scripts/check-installer-wayland-qemu.sh
grep -Fq 'aqua_installer_user_profile username=aqua display_name=user password_configured=true' scripts/check-installer-wayland-qemu.sh
grep -Fq 'aqua_installer_summary_destructive_acknowledgement=true' scripts/check-installer-wayland-qemu.sh
grep -Fq 'aqua_installer_summary_confirmation_applied=true' scripts/check-installer-wayland-qemu.sh
grep -Fq 'aqua_installer_summary_target_device=/dev/vdb' scripts/check-installer-wayland-qemu.sh
grep -Fq 'installer-welcome' crates/aqua-compositor/src/main.rs
grep -Fq 'snapshot.keyboard_event_count >= 106' crates/aqua-compositor/src/main.rs
grep -Fq '.any(|surface| surface.commit_count >= 38)' crates/aqua-compositor/src/main.rs
grep -Fq 'DestructiveAcknowledgementRequired' crates/aqua-installer/src/lib.rs
grep -Fq 'CONFIG_VFAT_FS=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'installer_qemu_transaction = "a second exact opt-in executes' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'installer_failure_cleanup = "an exact QEMU-only AFTER_EFI_MOUNT injection' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'rootfs.tar' scripts/write-installer-artifact-disk-docker.sh
grep -Fq 'bootx64.efi' scripts/write-installer-artifact-disk-docker.sh
grep -Fq 'manifest.sha256' scripts/write-installer-artifact-disk-docker.sh
grep -Fq 'name = "aqua-shell"' crates/aqua-shell/Cargo.toml
test -f crates/aqua-text/Cargo.toml
test -f crates/aqua-text/src/lib.rs
test -x scripts/check-typography-fixtures.sh
test -x scripts/check-typography-layout-fixtures.sh
test -x scripts/check-typography-wayland-qemu.sh
test -x scripts/check-typography-wayland-qemu.exp
test -x scripts/check-component-wayland-qemu.sh
test -x scripts/check-component-wayland-qemu.exp
test -x scripts/check-elevation-wayland-qemu.sh
test -x scripts/check-elevation-wayland-qemu.exp
test -x scripts/check-icon-fixtures.sh
test -x scripts/check-component-fixtures.sh
test -x scripts/check-icon-wayland-qemu.sh
test -x scripts/check-icon-wayland-qemu.exp
test -f docs/aqua-linux/icon-fixtures.txt
test -f docs/aqua-linux/component-fixtures.txt
test -f docs/aqua-linux/typography-fixtures.txt
test -f docs/aqua-linux/typography-layout-fixtures.txt
grep -Fq '"crates/aqua-text"' Cargo.toml
grep -Fq 'rustybuzz = "0.20.1"' crates/aqua-text/Cargo.toml
grep -Fq 'pub struct TextService' crates/aqua-text/src/lib.rs
grep -Fq 'pub fn shape_line' crates/aqua-text/src/lib.rs
grep -Fq 'pub fn typography_fixture_report' crates/aqua-text/src/lib.rs
grep -Fq 'noto-sans-arabic-regular-2.009' docs/aqua-linux/typography-fixtures.txt
grep -Fq 'pub fn typography_layout_acceptance_report' crates/aqua-renderer/src/lib.rs
grep -Fq 'aqua-typography-layout-fixtures-1' docs/aqua-linux/typography-layout-fixtures.txt
grep -Fq 'aqua-component-fixtures-19' docs/aqua-linux/component-fixtures.txt
test -f docs/aqua-linux/component-catalog.md
grep -Fq 'Window frame and title bar | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Top system bar | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Menu | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Section group | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Metadata row | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Grid cell | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Application overview | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Global search | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Running-app dock | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Workspace switcher | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Notification | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Confirmation dialog | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Standard button | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Icon button | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Search field | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Checkbox | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Switch | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Segmented control | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Toolbar | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'List row | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
grep -Fq 'Sidebar navigation | Shared packaged-QEMU-proven primitive' docs/aqua-linux/component-catalog.md
test -f crates/aqua-components/Cargo.toml
grep -Fq 'pub enum SharedComponentKind' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct WindowFrame' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct TopSystemBar' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct Menu' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct SectionGroup' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct MetadataRow' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct IconButton' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct SearchField' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct GlobalSearch' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct RunningAppDock' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct WorkspaceSwitcher' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct NotificationToast' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct ConfirmationDialog' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct Checkbox' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct SwitchControl' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct SegmentedControl' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct Slider' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct Toolbar' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct SidebarNavigation' crates/aqua-components/src/lib.rs
grep -Fq 'pub struct ListNavigation' crates/aqua-components/src/lib.rs
grep -Fq 'pub enum ListNavigationKey' crates/aqua-components/src/lib.rs
grep -Fq 'aqua-components = { path = "../aqua-components" }' crates/aqua-shell/Cargo.toml
grep -Fq 'aqua-components = { path = "../aqua-components" }' crates/aqua-renderer/Cargo.toml
grep -Fq 'aqua-components = { path = "../aqua-components" }' crates/aqua-compositor/Cargo.toml
grep -Fq 'aqua-components = { path = "../aqua-components" }' crates/aqua-installer/Cargo.toml
grep -Fq 'pub fn first_party_window_action(' crates/aqua-compositor/src/lib.rs
grep -Fq 'WindowFrame::new(' crates/aqua-renderer/src/lib.rs
grep -Fq 'StandardButton::new(' crates/aqua-renderer/src/lib.rs
grep -Fq 'ListRow::new(' crates/aqua-renderer/src/lib.rs
grep -Fq 'launcher.search_field(' crates/aqua-renderer/src/lib.rs
grep -Fq 'files_toolbar_layout(width, height)' crates/aqua-renderer/src/lib.rs
grep -Fq 'desktop_context_menu_with_selection(index, selected_row).map' crates/aqua-renderer/src/lib.rs
grep -Fq 'desktop_context_menu_with_selection(icon_index, self.context_menu_selected_row)?' crates/aqua-shell/src/lib.rs
grep -Fq 'ListNavigation::new(self.window.entries.len(), visible_rows)' crates/aqua-shell/src/lib.rs
grep -Fq '.theme_segmented_control()' crates/aqua-shell/src/lib.rs
grep -Fq '.keyboard_target(navigation_key)' crates/aqua-shell/src/lib.rs
grep -Fq 'control.keyboard_toggles(ActivationKey::Enter)' crates/aqua-shell/src/lib.rs
grep -Fq 'section.row_rect(row_index).height.min(36)' crates/aqua-shell/src/lib.rs
grep -Fq 'button.keyboard_activates(ActivationKey::Enter)' crates/aqua-shell/src/lib.rs
grep -Fq 'cell.keyboard_activates(ActivationKey::Enter)' crates/aqua-shell/src/lib.rs
grep -Fq 'row.keyboard_activates(ActivationKey::Enter)' crates/aqua-shell/src/lib.rs
grep -Fq 'navigate_selection_in_viewport(' crates/aqua-shell/src/lib.rs
grep -Fq 'handle_event_in_viewport(' crates/aqua-shell/src/lib.rs
grep -Fq 'self.launcher_state.handle_event_in_viewport(' crates/aqua-compositor/src/lib.rs
grep -Fq 'launcher.handle_event_in_viewport(LauncherEvent::Activate, viewport.width, viewport.height)' crates/aqua-compositor/src/lib.rs
grep -Fq 'navigator.handle_key_in_viewport(files_width, files_height, files_key)' crates/aqua-compositor/src/lib.rs
grep -Fq 'files_sidebar_navigation(height).keyboard_target(' crates/aqua-shell/src/lib.rs
grep -Fq 'files_key_for_code(key)' crates/aqua-compositor/src/lib.rs
grep -Fq 'self.window.selected_sidebar = previously_selected_sidebar' crates/aqua-shell/src/lib.rs
grep -Fq 'sidebar_row(height, index)' crates/aqua-shell/src/lib.rs
grep -Fq 'self.sidebar_at(height, x, y)' crates/aqua-shell/src/lib.rs
grep -Fq 'files_visible_rows_in_viewport(width, height)' crates/aqua-shell/src/lib.rs
grep -Fq 'files_preview_visible_lines_in_viewport(width, height)' crates/aqua-shell/src/lib.rs
grep -Fq 'self.window.content_focus_rect(width, height)' crates/aqua-shell/src/lib.rs
grep -Fq 'preview_scrollbar_in_viewport(width, height)' crates/aqua-renderer/src/lib.rs
grep -Fq 'model.content_focus_rect(width, height)' crates/aqua-renderer/src/lib.rs
grep -Fq 'files_empty_state_layout(width, height)' crates/aqua-renderer/src/lib.rs
grep -Fq 'navigator.handle_pointer_in_viewport(' crates/aqua-compositor/src/lib.rs
grep -Fq 'navigator.handle_scroll_in_viewport(' crates/aqua-compositor/src/lib.rs
grep -Fq 'state.buffer_width.max(1)' crates/aqua-compositor/src/lib.rs
grep -Fq 'details_section_group(width, height)' crates/aqua-renderer/src/lib.rs
grep -Fq 'model.section_group()' crates/aqua-renderer/src/lib.rs
grep -Fq 'details_metadata_row(width, height' crates/aqua-renderer/src/lib.rs
grep -Fq 'MetadataRow::new(' crates/aqua-renderer/src/lib.rs
grep -Fq 'top_system_bar(width, height)' crates/aqua-renderer/src/lib.rs
grep -Fq 'top_system_bar_session_hit(' crates/aqua-compositor/src/lib.rs
grep -Fq 'menu.menu_layout(width, height)' crates/aqua-renderer/src/lib.rs
grep -Fq 'toolbar.back.with_state(' crates/aqua-renderer/src/lib.rs
grep -Fq 'model.active_switch()' crates/aqua-renderer/src/lib.rs
grep -Fq 'model.theme_segmented_control()' crates/aqua-renderer/src/lib.rs
grep -Fq 'model.audio_slider()' crates/aqua-renderer/src/lib.rs
grep -Fq 'pub struct AudioVolumeModel' crates/aqua-shell/src/lib.rs
grep -Fq 'aqua_settings_audio_backend_applied={}' crates/aqua-compositor/src/lib.rs
grep -Fq 'pub enum AudioControlStatus' crates/aqua-shell/src/lib.rs
grep -Fq 'aqua_settings_audio_control_status={}' crates/aqua-compositor/src/lib.rs
grep -Fq 'aqua_settings_audio_controls_enabled={}' crates/aqua-compositor/src/lib.rs
grep -Fq 'AQUA_DRM_WAYLAND_SCENARIO=typography-acceptance' scripts/check-typography-wayland-qemu.exp
grep -Fq 'typography_wayland_surface_ready=true' scripts/check-typography-wayland-qemu.sh
grep -Fq 'AQUA_DRM_WAYLAND_SCENARIO=component-acceptance' scripts/check-component-wayland-qemu.exp
grep -Fq 'aqua_component_acceptance_fixture_revision=aqua-component-fixtures-19' scripts/check-component-wayland-qemu.sh
grep -Fq 'component_wayland_shared_primitive_count=22' scripts/check-component-wayland-qemu.sh
grep -Fq 'AQUA_DRM_WAYLAND_SCENARIO=elevation-acceptance' scripts/check-elevation-wayland-qemu.exp
grep -Fq 'elevation_wayland_focused_surface_count=1' scripts/check-elevation-wayland-qemu.sh
grep -Fq 'gpu_shadow_damage_rects=2' scripts/check-elevation-wayland-qemu.sh
grep -Fq 'AQUA_DRM_WAYLAND_SCENARIO=icon-acceptance' scripts/check-icon-wayland-qemu.exp
grep -Fq 'icon_wayland_raster_cache_ready=true' scripts/check-icon-wayland-qemu.sh
grep -Fq 'desktop_icon_raster_cache_hits=3' scripts/check-icon-wayland-qemu.sh
grep -Fq 'icon-acceptance' crates/aqua-compositor/src/main.rs
grep -Fq 'elevation-acceptance' crates/aqua-compositor/src/main.rs
grep -Fq 'aqua-typography-acceptance' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'aqua-component-acceptance' br2-external/aqua/board/aqua/x86_64/post-build.sh
grep -Fq 'component-acceptance' crates/aqua-compositor/src/main.rs
grep -Fq 'typography_qemu = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'elevation_qemu = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'icon_qemu = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'component_qemu = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'icon_rasterization = ' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'resvg = { version = "0.45.1", default-features = false }' crates/aqua-renderer/Cargo.toml
grep -Fq 'pub struct IconRasterCache' crates/aqua-renderer/src/icons.rs
grep -Fq 'aqua-text = { path = "../aqua-text" }' crates/aqua-renderer/Cargo.toml
grep -Fq 'export-visible-preview-html' docs/aqua-linux/compositor.md
grep -Fq 'smoke-nested-preview-loop' docs/aqua-linux/compositor.md
grep -Fq 'probe-manual-nested-preview-backend' docs/aqua-linux/compositor.md
grep -Fq 'probe-preview-window' docs/aqua-linux/compositor.md
grep -Fq 'probe-nested-output-presenter' docs/aqua-linux/compositor.md
grep -Fq 'probe-host-window-lifecycle' docs/aqua-linux/compositor.md
grep -Fq 'probe-manual-execution-window-bridge' docs/aqua-linux/compositor.md
grep -Fq 'smoke-host-window-lifecycle' docs/aqua-linux/compositor.md
grep -Fq 'smoke-manual-execution-window' docs/aqua-linux/compositor.md
grep -Fq 'host-window-preview' docs/aqua-linux/compositor.md
grep -Fq 'probe-client-window-model' docs/aqua-linux/compositor.md
grep -Fq 'probe-client-surface-lifecycle' docs/aqua-linux/compositor.md
grep -Fq 'probe-client-surface-registry' docs/aqua-linux/compositor.md
grep -Fq 'probe-renderer-surface-sources' docs/aqua-linux/compositor.md
grep -Fq 'probe-client-layer-pipeline' docs/aqua-linux/compositor.md
grep -Fq 'probe-xdg-shell-binding' docs/aqua-linux/compositor.md
grep -Fq 'probe-xdg-toplevel-client' docs/aqua-linux/compositor.md
grep -Fq 'wl_data_device_manager' docs/aqua-linux/compositor.md
grep -Fq 'selection_ownership' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'probe-xdg-toplevel-window-model' docs/aqua-linux/compositor.md
grep -Fq 'probe-launcher-model' docs/aqua-linux/compositor.md
grep -Fq 'probe-launcher-input-scene' docs/aqua-linux/compositor.md
grep -Fq 'probe-smithay-launcher-seat' docs/aqua-linux/compositor.md
grep -Fq 'scripts/check-smithay-seat-docker.sh' docs/aqua-linux/compositor.md
grep -Fq 'probe-paint-plan' scripts/check-compositor.sh
grep -Fq 'probe-frame-plan' scripts/check-compositor.sh
grep -Fq 'probe-frame-buffer' scripts/check-compositor.sh
grep -Fq 'probe-raster' scripts/check-compositor.sh
grep -Fq 'probe-raster-export' scripts/check-compositor.sh
grep -Fq 'export-raster-ppm' scripts/check-compositor.sh
grep -Fq 'probe-raster-png-export' scripts/check-compositor.sh
grep -Fq 'export-raster-png' scripts/check-compositor.sh
grep -Fq 'design_tokens_scene_materials=ok' scripts/check-compositor.sh
grep -Fq '"blurRequired"' scripts/check-runtime-assets.sh
grep -Fq 'export WAYLAND_DISPLAY=aqua-wayland-0' scripts/check-runtime-assets.sh
grep -Fq 'BR2_PACKAGE_XORG7=n' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_XWAYLAND=n' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'application_compatibility_boundary = "Aqua v1 is native Wayland-only' docs/aqua-linux/compositor-foundation.toml
grep -Fq '[AQUA-BOOT] stage=os-release id=aqua pretty="Aqua Linux Milestone 1"' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq '[AQUA-BOOT] stage=runtime-assets-ready milestone=2 status=ok' br2-external/aqua/rootfs-overlay/etc/init.d/rcS
grep -Fq 'selected = "smithay"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'BR2_PACKAGE_LIBXKBCOMMON=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'BR2_PACKAGE_XKEYBOARD_CONFIG=y' br2-external/aqua/configs/aqua_x86_64_defconfig
grep -Fq 'CONFIG_NETDEVICES=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'CONFIG_ETHERNET=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'CONFIG_VIRTIO_NET=y' br2-external/aqua/board/aqua/x86_64/linux.config
grep -Fq 'aqua_x86_64_defconfig' scripts/build-image.sh
if grep -Fq 'olddefconfig' scripts/build-image.sh; then
    echo "build-image.sh must derive .config from the versioned Aqua defconfig" >&2
    exit 1
fi
grep -Fq -- '--features smithay-gpu' scripts/build-compositor-linux-docker.sh
grep -Fq 'x86_64-buildroot-linux-musl-gcc' scripts/build-compositor-linux-docker.sh
grep -Fq 'version = "0.7.0"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'status = "selected-scene-model-spike"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'scene_model = "aqua-scene"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'component_system = "the independent aqua-components crate owns renderer-neutral component inventory' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'smithay_features = ["wayland_frontend"]' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'smithay_feature_gate = "smithay-smoke"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'socket_smoke = "bind_absolute lifecycle with nonblocking accept, local client insert, and cleanup"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'calloop_socket_smoke = "Generic<ListeningSocket> dispatch with local client insert, dispatch_clients, and flush_clients"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'session_skeleton = "AquaCompositorSession owns Display and compositor state for insert, dispatch, and flush"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'session_config = "recovery-safe nested-dev defaults for socket, runtime dir, assets, autostart, boot graphics, and fallback TTY"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'session_env = "derived environment contract for WAYLAND_DISPLAY, XDG_RUNTIME_DIR, AQUA_ASSET_ROOT, autostart, and boot graphics"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'session_bootstrap = "config-driven runtime directory preparation without compositor autostart, boot graphics, or desktop shell"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'session_run_once = "AquaCompositorSession run-once smoke accepts one local client, dispatches, flushes, and cleans up"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'session_loop = "AquaCompositorSession bounded loop smoke runs three dispatch/flush passes after accepting one local client"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'static_shell_scene = "wallpaper, top panel, desktop icon column, dock, launcher, system overview, and notification toast geometry"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'scene_asset_bindings = "runtime wallpaper, brand, and permanent Aqua Core Icon paths are part of the scene dump contract"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'scene_material_tokens = "bright Aqua surfaces reference shared surface, border, shadow, color, and layout token paths; blurRequired=false"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'renderer_plan = "aqua-renderer produces headless draw command plans without drawing or boot graphics"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'frame_plan = "aqua-renderer defines rgba8888 frame size, stride, buffer byte count, clear color, and full damage rect without allocating display output"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'frame_buffer = "aqua-renderer allocates and clears an in-memory rgba8888 software framebuffer without DRM, KMS, Wayland output, or boot graphics"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'software_raster = "aqua-renderer fills the static paint plan into the software framebuffer and verifies wallpaper/surface/interior/edge/highlight sample pixels plus a full-buffer checksum without display output"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'surface_primitives = "legacy-named aqua-renderer helpers draw deterministic rounded edge, highlight, and inset-shadow primitives in software without requiring blur, DRM, KMS, Wayland output, or boot graphics"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'raster_export = "aqua-renderer exports the deterministic software raster to a PPM artifact for inspection without boot graphics or display output"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'raster_png_export = "aqua-renderer exports the deterministic software raster to a dependency-free PNG artifact for visual inspection without boot graphics or display output"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'display_output_plan = "recovery-safe nested-dev output contract for 1536x1024 rgba8888, with qemu-drm-kms reserved for QEMU hardware validation and no renderer or boot graphics started"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'display_output_handoff = "recovery-safe handoff contract proving the composited raw rgba8888 preview frame format/checksum, full client-buffer snapshots, and framebuffer metadata are ready for the nested-dev output path without starting display output, renderer, desktop shell, or boot graphics"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'gpu_visible_kms_frame = "explicitly confirmed QEMU paths compose the layered system-surface scene plus wl_shm textures' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'gbm_direct_scanout = "explicitly confirmed bounded QEMU presenter' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'drm_wayland_session = "explicitly confirmed real QEMU path keeps Smithay compositor' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'display_activation_plan = "manual-dev activation gate proving the display-output handoff can be promoted toward controlled startup while fallback TTY remains required and display output, renderer, desktop shell, boot graphics, and autostart remain stopped"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'display_output_smoke = "manual-dev bounded display-output smoke starts and stops three nested-dev frames from the activation gate while renderer, desktop shell, boot graphics, and autostart remain disabled"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'nested_output_surface = "manual-dev nested output surface lifecycle smoke acquires, configures, attaches, presents, and releases the bounded display-output frame while renderer, desktop shell, boot graphics, and autostart remain disabled"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'visible_preview_plan = "readiness contract that joins output, scene, render, paint, frame, framebuffer, raster, PNG export, and client-layer pipeline probes before opening any nested preview window"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'visible_preview_export = "single-file HTML data-uri preview artifact generated from the deterministic PNG raster with client-layer paint steps composited into the image, without opening a window, starting the renderer, or enabling boot graphics"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'nested_preview_loop = "manual-dev bounded frame clock over the visible preview export path' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'manual_nested_preview_backend = "rootfs-verified manual nested preview backend path' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_nested_output_presenter = "manual aqua-host-tools presenter probe consumes the compositor display-output handoff and nested output surface lifecycle' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'host_preview_window = "feature-gated aqua-host-tools minifb window lifecycle probe and manual 3-frame smoke' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'manual_compositor_launch = "QEMU-safe recovery shell launch plan validates session config, session env, runtime dir, display activation readiness, fallback TTY, autostart=false, and boot_graphics=false without starting display output or the desktop shell"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'guarded_compositor_run = "QEMU-safe recovery shell bounded run validates the manual launch plan, starts and stops three nested-dev display-output smoke frames, preserves fallback TTY, and keeps autostart=false, boot_graphics=false, renderer_started=false, and desktop_shell_started=false"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'nested_preview_handoff_gate = "QEMU-safe recovery shell gate validates guarded run, display-output handoff, visible preview readiness, nested preview loop, and manual nested backend path before allowing manual operator promotion; automatic promotion remains false"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'client_window_model = "deterministic pre-client contract' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'client_surface_lifecycle = "deterministic pre-client xdg-toplevel lifecycle contract' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'client_surface_registry = "two-client xdg-toplevel registry contract tracks' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'xdg_shell_binding = "Smithay XdgShellState global and handler contract' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'xdg_toplevel_client = "two minimal in-process xdg_wm_base test clients' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'xdg_toplevel_window_model = "two recorded xdg-toplevel client surfaces are bound' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'build_script = "scripts/build-compositor-linux-docker.sh"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'autostart = false' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'boot_graphics = false' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'event_loop = "calloop"' docs/aqua-linux/compositor-foundation.toml
grep -Fq 'Calloop is linked' docs/aqua-linux/adr-0001-compositor-foundation.md
grep -Fq 'bounded session-loop smoke' docs/aqua-linux/adr-0001-compositor-foundation.md
grep -Fq 'manual-dev nested preview frame loop' docs/aqua-linux/adr-0001-compositor-foundation.md

echo "Aqua Linux local checks passed."
