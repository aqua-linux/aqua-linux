#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
ROOTFS_TAR="${ROOTFS_TAR:-${ROOT_DIR}/build/buildroot-output/images/rootfs.tar}"
IMAGE_NAME="${AQUA_COMPOSITOR_ROOTFS_CHECK_IMAGE:-ubuntu:24.04}"
HOST_CONTRACT_DIR="${CONTRACT_DIR:-${ROOT_DIR}/build/rootfs-compositor-contract}"
CONTAINER_CONTRACT_DIR="/work/build/rootfs-compositor-contract"

if [ ! -f "${ROOTFS_TAR}" ]; then
    echo "Missing rootfs tar: ${ROOTFS_TAR}" >&2
    echo "Run scripts/build-image-docker-volume.sh first." >&2
    exit 1
fi

mkdir -p "${HOST_CONTRACT_DIR}"

docker run --rm \
    --platform linux/amd64 \
    -v "${ROOT_DIR}:/work" \
    -w /work \
    -e CONTRACT_DIR="${CONTAINER_CONTRACT_DIR}" \
    "${IMAGE_NAME}" \
    sh -eu -c '
        tmp_dir="$(mktemp -d)"
        trap "rm -rf \"${tmp_dir}\"" EXIT
        mkdir -p "${tmp_dir}/rootfs" "${tmp_dir}/run"
        tar -xf build/buildroot-output/images/rootfs.tar -C "${tmp_dir}/rootfs"
        tar -xOf build/buildroot-output/images/rootfs.tar ./usr/bin/aqua-compositor > "${tmp_dir}/aqua-compositor.real"
        tar -xOf build/buildroot-output/images/rootfs.tar ./etc/aqua/compositor-session.conf > "${tmp_dir}/compositor-session.conf"
        chmod +x "${tmp_dir}/aqua-compositor.real"
        printf "%s\n" \
            "#!/bin/sh" \
            "export XKB_CONFIG_ROOT=\"${tmp_dir}/rootfs/usr/share/X11/xkb\"" \
            "exec \"${tmp_dir}/rootfs/lib/ld-musl-x86_64.so.1\" --library-path \"${tmp_dir}/rootfs/lib:${tmp_dir}/rootfs/usr/lib\" \"${tmp_dir}/aqua-compositor.real\" \"\$@\"" \
            > "${tmp_dir}/aqua-compositor"
        chmod +x "${tmp_dir}/aqua-compositor"
        "${tmp_dir}/aqua-compositor" status > "${CONTRACT_DIR}/status.txt"
        "${tmp_dir}/aqua-compositor" probe-session-config "${tmp_dir}/compositor-session.conf" > "${CONTRACT_DIR}/session-config.txt"
        "${tmp_dir}/aqua-compositor" probe-session-env "${tmp_dir}/compositor-session.conf" > "${CONTRACT_DIR}/session-env.txt"
        "${tmp_dir}/aqua-compositor" probe-session-bootstrap "${tmp_dir}/compositor-session.conf" "${tmp_dir}/run/aqua" > "${CONTRACT_DIR}/session-bootstrap.txt"
        "${tmp_dir}/aqua-compositor" probe-output-plan > "${CONTRACT_DIR}/output-plan-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-output-plan > "${CONTRACT_DIR}/output-plan-dump.txt"
        "${tmp_dir}/aqua-compositor" probe-display-output-handoff > "${CONTRACT_DIR}/display-output-handoff-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-display-activation-plan > "${CONTRACT_DIR}/display-activation-plan-probe.txt"
        AQUA_GPU_LIBRARY_ROOT="${tmp_dir}/rootfs/usr/lib" \
        "${tmp_dir}/aqua-compositor" probe-renderer-backend auto > "${CONTRACT_DIR}/renderer-backend-probe.txt"
        mkdir -p "${tmp_dir}/dev" "${tmp_dir}/sys/class/drm/card0-Virtual-1"
        : > "${tmp_dir}/dev/card0"
        printf "%s\n" connected > "${tmp_dir}/sys/class/drm/card0-Virtual-1/status"
        printf "%s\n" 1280x800 1024x768 > "${tmp_dir}/sys/class/drm/card0-Virtual-1/modes"
        AQUA_DRM_SYSFS_ROOT="${tmp_dir}/sys/class/drm" \
        AQUA_DRM_CARD_NAME=card0 \
        "${tmp_dir}/aqua-compositor" probe-drm-device "${tmp_dir}/dev/card0" > "${CONTRACT_DIR}/drm-device-probe.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_SESSION_ENV="${tmp_dir}/rootfs/etc/aqua/session.env" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        "${tmp_dir}/rootfs/usr/bin/aqua-compositor-manual-launch" > "${CONTRACT_DIR}/manual-launch-plan.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_SESSION_ENV="${tmp_dir}/rootfs/etc/aqua/session.env" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_MANUAL_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-manual-launch" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        "${tmp_dir}/rootfs/usr/bin/aqua-compositor-guarded-run" > "${CONTRACT_DIR}/guarded-run.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua-supervisor" \
        AQUA_GRAPHICS_SUPERVISOR_STATE_FILE="${tmp_dir}/run/aqua-supervisor/graphical-session-supervisor.state" \
        AQUA_GRAPHICS_SUPERVISOR_DRY_RUN=true \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphical-session-supervisor" > "${CONTRACT_DIR}/graphical-session-supervisor.txt"
        printf "%s\n" "console=ttyS0" > "${tmp_dir}/cmdline-default"
        AQUA_CMDLINE_PATH="${tmp_dir}/cmdline-default" \
        AQUA_GRAPHICS_BOOT_PROFILE="${tmp_dir}/rootfs/etc/aqua/compositor-session-graphics.conf" \
        AQUA_GRAPHICS_SUPERVISOR_BIN="${tmp_dir}/rootfs/usr/bin/aqua-graphical-session-supervisor" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua-boot" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphical-session-boot" > "${CONTRACT_DIR}/graphical-session-boot.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_SESSION_ENV="${tmp_dir}/rootfs/etc/aqua/session.env" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_MANUAL_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-manual-launch" \
        AQUA_GUARDED_RUN_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-guarded-run" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        "${tmp_dir}/rootfs/usr/bin/aqua-compositor-handoff-gate" > "${CONTRACT_DIR}/handoff-gate.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_SESSION_ENV="${tmp_dir}/rootfs/etc/aqua/session.env" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_HANDOFF_GATE_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-handoff-gate" \
        AQUA_GUARDED_RUN_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-guarded-run" \
        AQUA_MANUAL_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-manual-launch" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        "${tmp_dir}/rootfs/usr/bin/aqua-compositor-preview-exec" > "${CONTRACT_DIR}/manual-nested-preview-execution.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_SESSION_ENV="${tmp_dir}/rootfs/etc/aqua/session.env" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_PREVIEW_EXEC_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-preview-exec" \
        AQUA_HANDOFF_GATE_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-handoff-gate" \
        AQUA_GUARDED_RUN_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-guarded-run" \
        AQUA_MANUAL_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-manual-launch" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_VISIBLE_PREVIEW_REQUEST_FILE="${tmp_dir}/run/aqua/visible-preview.request" \
        "${tmp_dir}/rootfs/usr/bin/aqua-visible-preview-request" > "${CONTRACT_DIR}/visible-preview-request.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_SESSION_ENV="${tmp_dir}/rootfs/etc/aqua/session.env" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_PREVIEW_EXEC_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-preview-exec" \
        AQUA_VISIBLE_PREVIEW_REQUEST_BIN="${tmp_dir}/rootfs/usr/bin/aqua-visible-preview-request" \
        AQUA_HANDOFF_GATE_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-handoff-gate" \
        AQUA_GUARDED_RUN_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-guarded-run" \
        AQUA_MANUAL_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-manual-launch" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_VISIBLE_PREVIEW_REQUEST_FILE="${tmp_dir}/run/aqua/visible-preview.request" \
        AQUA_VISIBLE_PREVIEW_LAUNCH_FILE="${tmp_dir}/run/aqua/visible-preview-launch.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-visible-preview-launch" > "${CONTRACT_DIR}/visible-preview-launch.txt"
        "${tmp_dir}/rootfs/usr/bin/aqua-recovery-help" > "${CONTRACT_DIR}/recovery-help.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_OPERATOR_TRANSCRIPT_FILE="${tmp_dir}/run/aqua/operator-transcript.txt" \
        AQUA_RECOVERY_HELP_BIN="${tmp_dir}/rootfs/usr/bin/aqua-recovery-help" \
        AQUA_VISIBLE_PREVIEW_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-visible-preview-launch" \
        "${tmp_dir}/rootfs/usr/bin/aqua-operator-transcript" > "${CONTRACT_DIR}/operator-transcript.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_ENABLE_GATE_FILE="${tmp_dir}/run/aqua/graphics-enable-gate.plan" \
        AQUA_OPERATOR_TRANSCRIPT_BIN="${tmp_dir}/rootfs/usr/bin/aqua-operator-transcript" \
        AQUA_VISIBLE_PREVIEW_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-visible-preview-launch" \
        AQUA_HANDOFF_GATE_LOG="${tmp_dir}/run/aqua/preview-handoff-gate.log" \
        AQUA_MANUAL_EXECUTION_LOG="${tmp_dir}/run/aqua/preview-execution.log" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-enable-gate" > "${CONTRACT_DIR}/graphics-enable-gate.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_ENABLE_GATE_FILE="${tmp_dir}/run/aqua/graphics-enable-gate-positive.plan" \
        AQUA_OPERATOR_TRANSCRIPT_BIN="${tmp_dir}/rootfs/usr/bin/aqua-operator-transcript" \
        AQUA_VISIBLE_PREVIEW_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-visible-preview-launch" \
        AQUA_HANDOFF_GATE_LOG="${tmp_dir}/run/aqua/preview-handoff-gate.log" \
        AQUA_MANUAL_EXECUTION_LOG="${tmp_dir}/run/aqua/preview-execution.log" \
        AQUA_GRAPHICS_ENABLE_POSITIVE_DRY_RUN=true \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-enable-gate" > "${CONTRACT_DIR}/graphics-enable-gate-positive.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_ENABLE_GATE_FILE="${tmp_dir}/run/aqua/graphics-enable-gate-positive.plan" \
        AQUA_GRAPHICS_LAUNCH_CANDIDATE_FILE="${tmp_dir}/run/aqua/graphics-launch-candidate.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-launch-candidate" > "${CONTRACT_DIR}/graphics-launch-candidate.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_LAUNCH_CANDIDATE_FILE="${tmp_dir}/run/aqua/graphics-launch-candidate.plan" \
        AQUA_GRAPHICS_ROLLBACK_DRILL_FILE="${tmp_dir}/run/aqua/graphics-rollback-drill.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-rollback-drill" > "${CONTRACT_DIR}/graphics-rollback-drill.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_ROLLBACK_DRILL_FILE="${tmp_dir}/run/aqua/graphics-rollback-drill.plan" \
        AQUA_GRAPHICS_STARTUP_PREFLIGHT_FILE="${tmp_dir}/run/aqua/graphics-startup-preflight.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-startup-preflight" > "${CONTRACT_DIR}/graphics-startup-preflight.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_STARTUP_PREFLIGHT_FILE="${tmp_dir}/run/aqua/graphics-startup-preflight.plan" \
        AQUA_GUARDED_SMOKE_LOG="${tmp_dir}/run/aqua/guarded-display-output-smoke.log" \
        AQUA_GRAPHICS_STARTUP_REHEARSAL_FILE="${tmp_dir}/run/aqua/graphics-startup-rehearsal.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-startup-rehearsal" > "${CONTRACT_DIR}/graphics-startup-rehearsal.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_STARTUP_REHEARSAL_FILE="${tmp_dir}/run/aqua/graphics-startup-rehearsal.plan" \
        AQUA_GRAPHICS_QEMU_DISPLAY_GATE_FILE="${tmp_dir}/run/aqua/graphics-qemu-display-gate.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-qemu-display-gate" > "${CONTRACT_DIR}/graphics-qemu-display-gate.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_DISPLAY_GATE_FILE="${tmp_dir}/run/aqua/graphics-qemu-display-gate.plan" \
        AQUA_GRAPHICS_VISIBLE_QEMU_ATTEMPT_FILE="${tmp_dir}/run/aqua/graphics-visible-qemu-attempt.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-visible-qemu-attempt" > "${CONTRACT_DIR}/graphics-visible-qemu-attempt.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_VISIBLE_QEMU_ATTEMPT_FILE="${tmp_dir}/run/aqua/graphics-visible-qemu-attempt.plan" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_TRANSCRIPT_FILE="${tmp_dir}/run/aqua/graphics-visible-attempt-transcript.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-visible-attempt-transcript" > "${CONTRACT_DIR}/graphics-visible-attempt-transcript.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_TRANSCRIPT_FILE="${tmp_dir}/run/aqua/graphics-visible-attempt-transcript.plan" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RESULT_FILE="${tmp_dir}/run/aqua/graphics-visible-attempt-result.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-visible-attempt-result" > "${CONTRACT_DIR}/graphics-visible-attempt-result.txt"
        AQUA_COMPOSITOR_CONFIG="${tmp_dir}/compositor-session.conf" \
        AQUA_SESSION_ENV="${tmp_dir}/rootfs/etc/aqua/session.env" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_MANUAL_LAUNCH_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-manual-launch" \
        AQUA_GUARDED_RUN_BIN="${tmp_dir}/rootfs/usr/bin/aqua-compositor-guarded-run" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RESULT_BIN="${tmp_dir}/rootfs/usr/bin/aqua-graphics-visible-attempt-result" \
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_TRANSCRIPT_FILE="${tmp_dir}/run/aqua/graphics-visible-attempt-transcript.plan" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RUN_LOG="${tmp_dir}/run/aqua/graphics-visible-attempt-run.log" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RESULT_FILE="${tmp_dir}/run/aqua/graphics-visible-attempt-runner-result.plan" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RESULT_LOG="${tmp_dir}/run/aqua/graphics-visible-attempt-runner-result.log" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-visible-attempt-runner" > "${CONTRACT_DIR}/graphics-visible-attempt-runner.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RESULT_LOG="${tmp_dir}/run/aqua/graphics-visible-attempt-runner-result.log" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RESULT_FILE="${tmp_dir}/run/aqua/graphics-visible-attempt-runner-result.plan" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-qemu-visible-boot-check" > "${CONTRACT_DIR}/graphics-qemu-visible-boot-check.txt"
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        AQUA_ASSET_ROOT="${tmp_dir}/rootfs/usr/share/aqua" \
        AQUA_FBDEV_DRY_RUN=true \
        AQUA_FBDEV_PROBE_WIDTH=1024 \
        AQUA_FBDEV_PROBE_HEIGHT=768 \
        AQUA_FBDEV_PROBE_BPP=32 \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-fbdev-present" > "${CONTRACT_DIR}/graphics-fbdev-present.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_GRAPHICS_QEMU_OBSERVATION_FILE="${tmp_dir}/run/aqua/graphics-qemu-observation.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-qemu-observation-marker" > "${CONTRACT_DIR}/graphics-qemu-observation-marker.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_QEMU_VM_DISPLAY_EVIDENCE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence.plan" \
        AQUA_QEMU_VM_DISPLAY_CAPTURE_ID="docker-contract-qemu-visible-capture" \
        AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE="manual-qemu-display-capture-required.png" \
        "${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-evidence-record" > "${CONTRACT_DIR}/qemu-visible-evidence-record.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_GRAPHICS_QEMU_OBSERVATION_FILE="${tmp_dir}/run/aqua/graphics-qemu-observation-positive.plan" \
        AQUA_QEMU_VM_DISPLAY_EVIDENCE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence.plan" \
        AQUA_QEMU_VM_DISPLAY_OBSERVED=true \
        "${tmp_dir}/rootfs/usr/bin/aqua-graphics-qemu-observation-marker" > "${CONTRACT_DIR}/graphics-qemu-observation-positive.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_GRAPHICS_VISIBLE_ATTEMPT_RESULT_FILE="${tmp_dir}/run/aqua/graphics-visible-attempt-runner-result.plan" \
        AQUA_GRAPHICS_QEMU_OBSERVATION_FILE="${tmp_dir}/run/aqua/graphics-qemu-observation-positive.plan" \
        AQUA_QEMU_VM_DISPLAY_EVIDENCE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence.plan" \
        AQUA_QEMU_VISIBLE_PASS_REPORT_FILE="${tmp_dir}/run/aqua/qemu-visible-pass-report.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-pass-report" > "${CONTRACT_DIR}/qemu-visible-pass-report.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_QEMU_VISIBLE_MANUAL_RUNBOOK_FILE="${tmp_dir}/run/aqua/qemu-visible-manual-runbook.plan" \
        AQUA_ROOTFS_USR_BIN="${tmp_dir}/rootfs/usr/bin" \
        "${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-manual-runbook" > "${CONTRACT_DIR}/qemu-visible-manual-runbook.txt"
        cat > "${tmp_dir}/run/aqua/qemu-visible-evidence-bundle.txt" <<EOF
product=Aqua Linux
bundle=qemu-visible-evidence
bundle_status=recovery-commands-ready
capture_id=docker-contract-qemu-visible-capture
capture_file=manual-qemu-display-capture-required.png
capture_sha256=docker-contract-capture-sha256
capture_hash_verified=true
preflight_summary_status=ok
preflight_summary_json=docker-contract-qemu-visible-preflight.json
preflight_summary_generated_at=docker-contract-generated-at
preflight_summary_verified=true
recovery_step_1=aqua-graphics-qemu-visible-boot-check
recovery_step_2=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=docker-contract-qemu-visible-capture AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=manual-qemu-display-capture-required.png aqua-qemu-visible-evidence-record
recovery_step_3=AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker
recovery_step_4=aqua-qemu-visible-pass-report
operator_confirmation_required=true
manual_observation_required=true
persistent_graphical_session_started=false
desktop_shell_started=false
boot_graphics=false
autostart=false
fallback_tty_available=true
safe_return_to_recovery=ok
EOF
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle.txt" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_APPLY_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-apply.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-evidence-bundle-apply" > "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply.txt"
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle.txt" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_APPLY_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-apply-positive.plan" \
        AQUA_QEMU_VM_DISPLAY_EVIDENCE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-positive-evidence.plan" \
        AQUA_GRAPHICS_QEMU_OBSERVATION_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-positive-observation.plan" \
        AQUA_QEMU_VM_DISPLAY_OPERATOR_CONFIRMED=true \
        AQUA_QEMU_VISIBLE_EVIDENCE_BIN="${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-evidence-record" \
        AQUA_GRAPHICS_QEMU_OBSERVATION_BIN="${tmp_dir}/rootfs/usr/bin/aqua-graphics-qemu-observation-marker" \
        "${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-evidence-bundle-apply" > "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-positive.txt"
        cat > "${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-missing-preflight.txt" <<EOF
product=Aqua Linux
bundle=qemu-visible-evidence
bundle_status=recovery-commands-ready
capture_id=docker-contract-qemu-visible-capture
capture_file=manual-qemu-display-capture-required.png
capture_sha256=docker-contract-capture-sha256
capture_hash_verified=true
recovery_step_1=aqua-graphics-qemu-visible-boot-check
recovery_step_2=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=docker-contract-qemu-visible-capture AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=manual-qemu-display-capture-required.png aqua-qemu-visible-evidence-record
recovery_step_3=AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker
recovery_step_4=aqua-qemu-visible-pass-report
operator_confirmation_required=true
manual_observation_required=true
persistent_graphical_session_started=false
desktop_shell_started=false
boot_graphics=false
autostart=false
fallback_tty_available=true
safe_return_to_recovery=ok
EOF
        set +e
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-missing-preflight.txt" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_APPLY_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-missing-preflight.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-evidence-bundle-apply" > "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt"
        missing_preflight_status="$?"
        set -e
        printf "expected_failure_exit_code=%s\n" "${missing_preflight_status}" >> "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-preflight.txt"
        if [ "${missing_preflight_status}" -eq 0 ]; then
            echo "missing-preflight bundle unexpectedly passed" >&2
            exit 1
        fi
        cat > "${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-missing-capture-hash.txt" <<EOF
product=Aqua Linux
bundle=qemu-visible-evidence
bundle_status=recovery-commands-ready
capture_id=docker-contract-qemu-visible-capture
capture_file=manual-qemu-display-capture-required.png
capture_sha256=docker-contract-capture-sha256
preflight_summary_status=ok
preflight_summary_json=docker-contract-qemu-visible-preflight.json
preflight_summary_generated_at=docker-contract-generated-at
preflight_summary_verified=true
recovery_step_1=aqua-graphics-qemu-visible-boot-check
recovery_step_2=AQUA_QEMU_VM_DISPLAY_CAPTURE_ID=docker-contract-qemu-visible-capture AQUA_QEMU_VM_DISPLAY_CAPTURE_FILE=manual-qemu-display-capture-required.png aqua-qemu-visible-evidence-record
recovery_step_3=AQUA_QEMU_VM_DISPLAY_OBSERVED=true aqua-graphics-qemu-observation-marker
recovery_step_4=aqua-qemu-visible-pass-report
operator_confirmation_required=true
manual_observation_required=true
persistent_graphical_session_started=false
desktop_shell_started=false
boot_graphics=false
autostart=false
fallback_tty_available=true
safe_return_to_recovery=ok
EOF
        set +e
        AQUA_RUNTIME_DIR="${tmp_dir}/run/aqua" \
        AQUA_GRAPHICS_QEMU_VISIBLE_BOOT_CHECK_FILE="${tmp_dir}/run/aqua/graphics-qemu-visible-boot-check.plan" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-missing-capture-hash.txt" \
        AQUA_QEMU_VISIBLE_EVIDENCE_BUNDLE_APPLY_FILE="${tmp_dir}/run/aqua/qemu-visible-evidence-bundle-missing-capture-hash.plan" \
        "${tmp_dir}/rootfs/usr/bin/aqua-qemu-visible-evidence-bundle-apply" > "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt"
        missing_capture_hash_status="$?"
        set -e
        printf "expected_failure_exit_code=%s\n" "${missing_capture_hash_status}" >> "${CONTRACT_DIR}/qemu-visible-evidence-bundle-apply-missing-capture-hash.txt"
        if [ "${missing_capture_hash_status}" -eq 0 ]; then
            echo "missing-capture-hash bundle unexpectedly passed" >&2
            exit 1
        fi
        "${tmp_dir}/aqua-compositor" smoke-display-output > "${CONTRACT_DIR}/display-output-smoke.txt"
        "${tmp_dir}/aqua-compositor" smoke-nested-output-surface > "${CONTRACT_DIR}/nested-output-surface.txt"
        "${tmp_dir}/aqua-compositor" probe-visible-preview-plan > "${CONTRACT_DIR}/visible-preview-plan-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-visible-preview-export > "${CONTRACT_DIR}/visible-preview-export-probe.txt"
        "${tmp_dir}/aqua-compositor" export-visible-preview-html "${CONTRACT_DIR}/aqua-visible-preview.html" > "${CONTRACT_DIR}/visible-preview-export.txt"
        "${tmp_dir}/aqua-compositor" smoke-nested-preview-loop > "${CONTRACT_DIR}/nested-preview-loop.txt"
        "${tmp_dir}/aqua-compositor" probe-manual-nested-preview-backend > "${CONTRACT_DIR}/manual-nested-preview-backend.txt"
        "${tmp_dir}/aqua-compositor" run-manual-nested-preview-execution > "${CONTRACT_DIR}/manual-nested-preview-execution-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-client-window-model > "${CONTRACT_DIR}/client-window-model-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-client-surface-lifecycle > "${CONTRACT_DIR}/client-surface-lifecycle-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-client-surface-registry > "${CONTRACT_DIR}/client-surface-registry-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-renderer-surface-sources > "${CONTRACT_DIR}/renderer-surface-sources-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-client-layer-pipeline > "${CONTRACT_DIR}/client-layer-pipeline-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-xdg-shell-binding > "${CONTRACT_DIR}/xdg-shell-binding-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-xdg-toplevel-client > "${CONTRACT_DIR}/xdg-toplevel-client-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-xdg-toplevel-window-model > "${CONTRACT_DIR}/xdg-toplevel-window-model-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-launcher-model > "${CONTRACT_DIR}/launcher-model-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-launcher-input-scene > "${CONTRACT_DIR}/launcher-input-scene-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-smithay-launcher-seat > "${CONTRACT_DIR}/smithay-launcher-seat-probe.txt"
        "${tmp_dir}/aqua-compositor" probe-scene > "${CONTRACT_DIR}/scene-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-scene > "${CONTRACT_DIR}/scene-dump.txt"
        "${tmp_dir}/aqua-compositor" probe-render-plan > "${CONTRACT_DIR}/render-plan-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-render-plan > "${CONTRACT_DIR}/render-plan-dump.txt"
        "${tmp_dir}/aqua-compositor" probe-paint-plan > "${CONTRACT_DIR}/paint-plan-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-paint-plan > "${CONTRACT_DIR}/paint-plan-dump.txt"
        "${tmp_dir}/aqua-compositor" probe-frame-plan > "${CONTRACT_DIR}/frame-plan-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-frame-plan > "${CONTRACT_DIR}/frame-plan-dump.txt"
        "${tmp_dir}/aqua-compositor" probe-frame-buffer > "${CONTRACT_DIR}/frame-buffer-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-frame-buffer > "${CONTRACT_DIR}/frame-buffer-dump.txt"
        "${tmp_dir}/aqua-compositor" probe-raster > "${CONTRACT_DIR}/raster-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-raster > "${CONTRACT_DIR}/raster-dump.txt"
        "${tmp_dir}/aqua-compositor" probe-raster-export > "${CONTRACT_DIR}/raster-export-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-raster-export > "${CONTRACT_DIR}/raster-export-dump.txt"
        "${tmp_dir}/aqua-compositor" export-raster-ppm "${CONTRACT_DIR}/aqua-raster.ppm" > "${CONTRACT_DIR}/raster-export.txt"
        "${tmp_dir}/aqua-compositor" probe-raster-png-export > "${CONTRACT_DIR}/raster-png-export-probe.txt"
        "${tmp_dir}/aqua-compositor" dump-raster-png-export > "${CONTRACT_DIR}/raster-png-export-dump.txt"
        "${tmp_dir}/aqua-compositor" export-raster-png "${CONTRACT_DIR}/aqua-raster.png" > "${CONTRACT_DIR}/raster-png-export.txt"
        "${tmp_dir}/aqua-compositor" smoke-session-loop > "${CONTRACT_DIR}/session-loop.txt"
        mkdir -p "${tmp_dir}/run/user/1000" "${tmp_dir}/run/aqua"
        chown 1000:1000 "${tmp_dir}/run/user/1000" "${tmp_dir}/run/aqua"
        chmod 700 "${tmp_dir}/run/user/1000" "${tmp_dir}/run/aqua"
        AQUA_SESSION_ROOT="${tmp_dir}/rootfs" \
        AQUA_SESSION_RUN_DIR="${tmp_dir}/run" \
        AQUA_COMPOSITOR_BIN="${tmp_dir}/aqua-compositor" \
        "${tmp_dir}/rootfs/usr/bin/aqua-session-check" > "${CONTRACT_DIR}/session-check.txt"
    '

echo "Aqua Linux rootfs compositor contract exported: ${HOST_CONTRACT_DIR}"
