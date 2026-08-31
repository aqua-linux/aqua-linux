#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BOOT_TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-boot"
TMP_DIR="$(mktemp -d)"
trap 'if [ -f "${TMP_DIR}/run/network-service-supervisor.pid" ]; then kill "$(cat "${TMP_DIR}/run/network-service-supervisor.pid")" 2>/dev/null || true; fi; rm -rf "${TMP_DIR}"' EXIT HUP INT TERM

cat > "${TMP_DIR}/qemu.conf" <<'EOF'
enabled=true
interface=eth0
udhcpc_binary=/usr/bin/aqua-udhcpc-client
udhcpc_default_script=/usr/share/udhcpc/default.script
legacy_owner_disabled=true
max_restarts=3
restart_delay_seconds=2
readiness_timeout_seconds=20
lease_loss_grace_seconds=5
monitor_interval_seconds=1
stop_timeout_seconds=5
profile_scope=qemu-only
EOF

cat > "${TMP_DIR}/supervisor" <<'EOF'
#!/bin/sh
echo "$$" > "${AQUA_NETWORK_SUPERVISOR_PID_FILE}"
echo "config=${AQUA_NETWORK_SERVICES_CONFIG}" > "${AQUA_TEST_STARTED_FILE}"
echo "control_dir=${AQUA_NETWORK_CONTROL_DIR}" >> "${AQUA_TEST_STARTED_FILE}"
sleep 30
EOF
chmod +x "${TMP_DIR}/supervisor"

echo 'console=ttyS0' > "${TMP_DIR}/cmdline-default"
disabled_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-default" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/qemu.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/disabled-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/disabled-started" "${BOOT_TOOL}")"
printf '%s\n' "${disabled_output}" | grep -Fq 'status=disabled reason=kernel-flag-absent boot_network=false network_started=false'
test ! -e "${TMP_DIR}/disabled-started"

echo 'console=ttyS0 aqua.boot_network=10' > "${TMP_DIR}/cmdline-near-match"
near_match_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-near-match" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/qemu.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/near-match-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/near-match-started" "${BOOT_TOOL}")"
printf '%s\n' "${near_match_output}" | grep -Fq 'status=disabled reason=kernel-flag-absent'
test ! -e "${TMP_DIR}/near-match-started"

echo 'console=ttyS0 aqua.boot_network=1' > "${TMP_DIR}/cmdline-enabled"
set +e
missing_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/missing.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/missing-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/missing-started" "${BOOT_TOOL}" 2>&1)"
missing_status="$?"
set -e
test "${missing_status}" -ne 0
printf '%s\n' "${missing_output}" | grep -Fq 'status=blocked reason=invalid-qemu-network-profile network_started=false'
test ! -e "${TMP_DIR}/missing-started"

cat > "${TMP_DIR}/unsafe.conf" <<'EOF'
enabled=true
interface=eth0
udhcpc_binary=/usr/bin/aqua-udhcpc-client
udhcpc_default_script=/usr/share/udhcpc/default.script
legacy_owner_disabled=false
max_restarts=3
restart_delay_seconds=2
readiness_timeout_seconds=20
lease_loss_grace_seconds=5
monitor_interval_seconds=1
stop_timeout_seconds=5
profile_scope=qemu-only
EOF
set +e
unsafe_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/unsafe.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/unsafe-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/unsafe-started" "${BOOT_TOOL}" 2>&1)"
unsafe_status="$?"
set -e
test "${unsafe_status}" -ne 0
printf '%s\n' "${unsafe_output}" | grep -Fq 'status=blocked reason=invalid-qemu-network-profile network_started=false'
test ! -e "${TMP_DIR}/unsafe-started"

cp "${TMP_DIR}/qemu.conf" "${TMP_DIR}/duplicate-key.conf"
printf '%s\n' 'udhcpc_binary=/tmp/untrusted-client' >> "${TMP_DIR}/duplicate-key.conf"
set +e
duplicate_key_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/duplicate-key.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/duplicate-key-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/duplicate-key-started" "${BOOT_TOOL}" 2>&1)"
duplicate_key_status="$?"
set -e
test "${duplicate_key_status}" -ne 0
printf '%s\n' "${duplicate_key_output}" | grep -Fq 'status=blocked reason=invalid-qemu-network-profile network_started=false'
test ! -e "${TMP_DIR}/duplicate-key-started"

ln -s "${TMP_DIR}/supervisor" "${TMP_DIR}/supervisor-link"
set +e
symlink_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/qemu.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor-link" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/symlink-run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/symlink-started" "${BOOT_TOOL}" 2>&1)"
symlink_status="$?"
set -e
test "${symlink_status}" -ne 0
printf '%s\n' "${symlink_output}" | grep -Fq 'status=blocked reason=network-supervisor-missing network_started=false'
test ! -e "${TMP_DIR}/symlink-started"

enabled_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/qemu.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/started" "${BOOT_TOOL}")"
printf '%s\n' "${enabled_output}" | grep -Fq 'status=started mode=supervised-root target=qemu interface=eth0'

i=0
while [ ! -f "${TMP_DIR}/started" ] && [ "${i}" -lt 20 ]; do
    sleep 0.05
    i=$((i + 1))
done
grep -Fq "config=${TMP_DIR}/qemu.conf" "${TMP_DIR}/started"
grep -Fq "control_dir=${TMP_DIR}/run" "${TMP_DIR}/started"

set +e
duplicate_output="$(AQUA_CMDLINE_PATH="${TMP_DIR}/cmdline-enabled" AQUA_NETWORK_BOOT_PROFILE="${TMP_DIR}/qemu.conf" AQUA_NETWORK_SUPERVISOR_BIN="${TMP_DIR}/supervisor" AQUA_NETWORK_CONTROL_DIR="${TMP_DIR}/run" AQUA_TEST_STARTED_FILE="${TMP_DIR}/started" "${BOOT_TOOL}" 2>&1)"
duplicate_status="$?"
set -e
test "${duplicate_status}" -ne 0
printf '%s\n' "${duplicate_output}" | grep -Fq 'status=duplicate'

echo "Aqua Linux network service boot checks passed."
