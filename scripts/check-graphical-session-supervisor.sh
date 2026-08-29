#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SUPERVISOR="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphical-session-supervisor"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT HUP INT TERM

cat > "${TMP_DIR}/session.conf" <<'EOF'
boot_graphics=true
recovery_tty_required=true
EOF
mkdir -p "${TMP_DIR}/session-runtime"
cat > "${TMP_DIR}/session.env" <<EOF
export XDG_RUNTIME_DIR=${TMP_DIR}/session-runtime
EOF

cat > "${TMP_DIR}/compositor" <<'EOF'
#!/bin/sh
count_file="${AQUA_TEST_COUNT_FILE}"
count=0
[ -f "${count_file}" ] && count="$(cat "${count_file}")"
count=$((count + 1))
echo "${count}" > "${count_file}"
[ "${count}" -ge "${AQUA_TEST_SUCCEED_ON:-99}" ]
EOF

cat > "${TMP_DIR}/recovery" <<'EOF'
#!/bin/sh
echo recovery > "${AQUA_TEST_RECOVERY_FILE}"
EOF
chmod +x "${TMP_DIR}/compositor" "${TMP_DIR}/recovery"

common_env="AQUA_COMPOSITOR_CONFIG=${TMP_DIR}/session.conf AQUA_SESSION_ENV=${TMP_DIR}/session.env AQUA_COMPOSITOR_BIN=${TMP_DIR}/compositor AQUA_RECOVERY_BIN=${TMP_DIR}/recovery AQUA_GRAPHICS_SESSION_ENABLED=true AQUA_GRAPHICS_RESTART_DELAY_SECONDS=0 AQUA_GRAPHICS_STABLE_SECONDS=30 AQUA_SESSION_RUNTIME_DIR=${TMP_DIR}/session-runtime"

success_output="$(env ${common_env} AQUA_RUNTIME_DIR="${TMP_DIR}/success-run" AQUA_TEST_COUNT_FILE="${TMP_DIR}/success-count" AQUA_TEST_RECOVERY_FILE="${TMP_DIR}/unused-recovery" AQUA_TEST_SUCCEED_ON=3 AQUA_GRAPHICS_MAX_RESTARTS=3 "${SUPERVISOR}")"
printf '%s\n' "${success_output}" | grep -Fq 'status=stopped reason=clean-exit attempts=3 restarts=2'
grep -Fq 'state=stopped' "${TMP_DIR}/success-run/graphical-session-supervisor.state"
grep -Fq 'attempts=3' "${TMP_DIR}/success-run/graphical-session-supervisor.state"
test ! -e "${TMP_DIR}/unused-recovery"

set +e
failure_output="$(env ${common_env} AQUA_RUNTIME_DIR="${TMP_DIR}/failure-run" AQUA_TEST_COUNT_FILE="${TMP_DIR}/failure-count" AQUA_TEST_RECOVERY_FILE="${TMP_DIR}/recovery-called" AQUA_GRAPHICS_MAX_RESTARTS=2 "${SUPERVISOR}" 2>&1)"
failure_status="$?"
set -e
test "${failure_status}" -ne 0
printf '%s\n' "${failure_output}" | grep -Fq 'status=fallback reason=restart-limit'
grep -Fq 'state=fallback' "${TMP_DIR}/failure-run/graphical-session-supervisor.state"
grep -Fq 'attempts=3' "${TMP_DIR}/failure-run/graphical-session-supervisor.state"
grep -Fq 'restarts=2' "${TMP_DIR}/failure-run/graphical-session-supervisor.state"
test -f "${TMP_DIR}/recovery-called"

dry_output="$(AQUA_RUNTIME_DIR="${TMP_DIR}/dry-run" AQUA_GRAPHICS_SUPERVISOR_DRY_RUN=true "${SUPERVISOR}")"
printf '%s\n' "${dry_output}" | grep -Fq 'status=ok mode=dry-run session_started=false'
grep -Fq 'state=policy-ready' "${TMP_DIR}/dry-run/graphical-session-supervisor.state"

echo "Aqua Linux graphical session supervisor checks passed."
