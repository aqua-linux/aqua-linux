#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BOOT_TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-graphical-session-boot"
TMP_DIR="$(mktemp -d)"
trap 'if [ -f "${TMP_DIR}/run/graphical-session-supervisor.pid" ]; then kill "$(cat "${TMP_DIR}/run/graphical-session-supervisor.pid")" 2>/dev/null || true; fi; rm -rf "${TMP_DIR}"' EXIT HUP INT TERM

cat > "${TMP_DIR}/graphics.conf" <<'EOF'
boot_graphics=true
autostart=true
recovery_tty_required=true
EOF
cat > "${TMP_DIR}/graphics.env" <<'EOF'
export AQUA_BOOT_GRAPHICS=true
export AQUA_COMPOSITOR_AUTOSTART=true
EOF

cat > "${TMP_DIR}/supervisor" <<'EOF'
#!/bin/sh
echo "config=${AQUA_COMPOSITOR_CONFIG}" > "${AQUA_TEST_STARTED_FILE}"
echo "enabled=${AQUA_GRAPHICS_SESSION_ENABLED}" >> "${AQUA_TEST_STARTED_FILE}"
sleep 30
EOF
chmod +x "${TMP_DIR}/supervisor"

echo 'console=ttyS0' > "${TMP_DIR}/cmdline-default"
disabled_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-default" AQUA_GRAPHICS_BOOT_PROFILE="${TMP_DIR}/graphics.conf" AQUA_GRAPHICS_SESSION_ENV="${TMP_DIR}/graphics.env" AQUA_GRAPHICS_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_RUNTIME_DIR="${TMP_DIR}/disabled-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/disabled-started" "${BOOT_TOOL}")"
printf '%s\n' "${disabled_output}" | grep -Fq 'status=disabled reason=kernel-flag-absent boot_graphics=false session_started=false'
test ! -e "${TMP_DIR}/disabled-started"

echo 'console=ttyS0 aqua.boot_graphics=1' > "${TMP_DIR}/cmdline-enabled"

set +e
missing_profile_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_GRAPHICS_BOOT_PROFILE="${TMP_DIR}/missing.conf" AQUA_GRAPHICS_SESSION_ENV="${TMP_DIR}/graphics.env" AQUA_GRAPHICS_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_RUNTIME_DIR="${TMP_DIR}/missing-profile-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/missing-profile-started" "${BOOT_TOOL}" 2>&1)"
missing_profile_status="$?"
set -e
test "${missing_profile_status}" -ne 0
printf '%s\n' "${missing_profile_output}" | grep -Fq 'status=blocked reason=invalid-graphics-profile session_started=false'
test ! -e "${TMP_DIR}/missing-profile-started"

cat > "${TMP_DIR}/unsafe-graphics.conf" <<'EOF'
boot_graphics=true
autostart=true
recovery_tty_required=false
EOF
set +e
unsafe_profile_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_GRAPHICS_BOOT_PROFILE="${TMP_DIR}/unsafe-graphics.conf" AQUA_GRAPHICS_SESSION_ENV="${TMP_DIR}/graphics.env" AQUA_GRAPHICS_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_RUNTIME_DIR="${TMP_DIR}/unsafe-profile-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/unsafe-profile-started" "${BOOT_TOOL}" 2>&1)"
unsafe_profile_status="$?"
set -e
test "${unsafe_profile_status}" -ne 0
printf '%s\n' "${unsafe_profile_output}" | grep -Fq 'status=blocked reason=invalid-graphics-profile session_started=false'
test ! -e "${TMP_DIR}/unsafe-profile-started"

enabled_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_GRAPHICS_BOOT_PROFILE="${TMP_DIR}/graphics.conf" AQUA_GRAPHICS_SESSION_ENV="${TMP_DIR}/graphics.env" AQUA_GRAPHICS_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_RUNTIME_DIR="${TMP_DIR}/run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/started" "${BOOT_TOOL}")"
printf '%s\n' "${enabled_output}" | grep -Fq 'status=started mode=supervised'

i=0
while [ ! -f "${TMP_DIR}/started" ] && [ "${i}" -lt 20 ]; do
    sleep 0.05
    i=$((i + 1))
done
grep -Fq "config=${TMP_DIR}/graphics.conf" "${TMP_DIR}/started"
grep -Fq 'enabled=true' "${TMP_DIR}/started"

set +e
duplicate_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_GRAPHICS_BOOT_PROFILE="${TMP_DIR}/graphics.conf" AQUA_GRAPHICS_SESSION_ENV="${TMP_DIR}/graphics.env" AQUA_GRAPHICS_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_RUNTIME_DIR="${TMP_DIR}/run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/started" "${BOOT_TOOL}" 2>&1)"
duplicate_status="$?"
set -e
test "${duplicate_status}" -ne 0
printf '%s\n' "${duplicate_output}" | grep -Fq 'status=duplicate'

echo "Aqua Linux graphical session boot checks passed."
