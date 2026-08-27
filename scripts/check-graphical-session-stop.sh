#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
STOP_TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphical-session-stop"
TMP_DIR="$(mktemp -d)"
trap 'if [ -f "${TMP_DIR}/run/graphical-session-supervisor.pid" ]; then kill "$(cat "${TMP_DIR}/run/graphical-session-supervisor.pid")" 2>/dev/null || true; fi; rm -rf "${TMP_DIR}"' EXIT HUP INT TERM
mkdir -p "${TMP_DIR}/run"

(
    while [ ! -f "${TMP_DIR}/run/graphical-session.stop" ]; do sleep 0.05; done
    cat > "${TMP_DIR}/run/graphical-session-supervisor.state" <<'EOF'
state=stopped
EOF
    cat > "${TMP_DIR}/run/graphical-session-supervisor.log" <<'EOF'
external_wayland_client_process_stopped=true
crtc_restored=true
gbm_scanout_buffers_released=true
graceful_stop_completed=true
EOF
    rm -f "${TMP_DIR}/run/graphical-session-supervisor.pid"
) &
fake_pid="$!"
echo "${fake_pid}" > "${TMP_DIR}/run/graphical-session-supervisor.pid"

output="$(AQUA_RUNTIME_DIR="${TMP_DIR}/run" AQUA_GRAPHICS_STOP_TIMEOUT_SECONDS=5 "${STOP_TOOL}")"
printf '%s\n' "${output}" | grep -Fq 'status=requested'
printf '%s\n' "${output}" | grep -Fq 'status=ok supervisor_stopped=true pid_cleaned=true kms_restored=true clients_stopped=true recovery_return=ok'
test ! -e "${TMP_DIR}/run/graphical-session-supervisor.pid"
test ! -e "${TMP_DIR}/run/graphical-session.stop"

idle_output="$(AQUA_RUNTIME_DIR="${TMP_DIR}/idle" "${STOP_TOOL}")"
printf '%s\n' "${idle_output}" | grep -Fq 'status=idle reason=supervisor-not-running'

echo "Aqua Linux graphical session stop checks passed."
