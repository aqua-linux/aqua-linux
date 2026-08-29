#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
PREPARE_TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-session-runtime-prepare"
LAUNCH_TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-session-user-launch"
USERS_TABLE="${ROOT_DIR}/br2-external/aqua/users.txt"
DEFCONFIG="${ROOT_DIR}/br2-external/aqua/configs/aqua_x86_64_defconfig"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT HUP INT TERM

test -x "${PREPARE_TOOL}"
test -x "${LAUNCH_TOOL}"
grep -Fxq 'aqua 1000 aqua 1000 * /home/aqua /bin/false video,audio,input Aqua desktop session' "${USERS_TABLE}"
grep -Fxq 'BR2_ROOTFS_USERS_TABLES="$(BR2_EXTERNAL_AQUA_PATH)/users.txt"' "${DEFCONFIG}"

session_user="$(id -un)"
session_uid="$(id -u)"
session_gid="$(id -g)"
runtime_dir="${TMP_DIR}/run/user/${session_uid}"
control_dir="${TMP_DIR}/run/aqua"

prepare_output="$(
    AQUA_SESSION_USER="${session_user}" \
    AQUA_SESSION_UID="${session_uid}" \
    AQUA_SESSION_GID="${session_gid}" \
    AQUA_SESSION_RUNTIME_DIR="${runtime_dir}" \
    AQUA_SESSION_CONTROL_DIR="${control_dir}" \
    "${PREPARE_TOOL}"
)"
printf '%s\n' "${prepare_output}" | grep -Fq 'status=ok'
test "$(ls -nd "${runtime_dir}" | awk '{print $3 ":" $4}')" = "${session_uid}:${session_gid}"
test "$(LC_ALL=C ls -ld "${runtime_dir}" | awk '{print substr($1, 1, 10)}')" = 'drwx------'

cat > "${TMP_DIR}/command" <<'EOF'
#!/bin/sh
exit 0
EOF
chmod +x "${TMP_DIR}/command"
launch_output="$(
    AQUA_SESSION_USER="${session_user}" \
    AQUA_SESSION_UID="${session_uid}" \
    AQUA_SESSION_GID="${session_gid}" \
    AQUA_SESSION_RUNTIME_DIR="${runtime_dir}" \
    AQUA_SESSION_ALLOWED_COMMAND="${TMP_DIR}/command" \
    AQUA_SESSION_USER_LAUNCH_DRY_RUN=true \
    "${LAUNCH_TOOL}" "${TMP_DIR}/command"
)"
printf '%s\n' "${launch_output}" | grep -Fq "status=ok mode=dry-run user=${session_user} uid=${session_uid} gid=${session_gid}"

ln -s "${runtime_dir}" "${TMP_DIR}/unsafe-runtime"
set +e
unsafe_output="$(
    AQUA_SESSION_USER="${session_user}" \
    AQUA_SESSION_UID="${session_uid}" \
    AQUA_SESSION_GID="${session_gid}" \
    AQUA_SESSION_RUNTIME_DIR="${TMP_DIR}/unsafe-runtime" \
    AQUA_SESSION_ALLOWED_COMMAND="${TMP_DIR}/command" \
    AQUA_SESSION_USER_LAUNCH_DRY_RUN=true \
    "${LAUNCH_TOOL}" "${TMP_DIR}/command" 2>&1
)"
unsafe_status="$?"
set -e
test "${unsafe_status}" -ne 0
printf '%s\n' "${unsafe_output}" | grep -Fq 'status=failed reason=unsafe-runtime-directory'

echo 'Aqua Linux unprivileged session checks passed.'
