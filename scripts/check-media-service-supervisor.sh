#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SUPERVISOR="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-media-service-supervisor"
STOP_TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-media-service-stop"
CONFIG="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/etc/aqua/media-services.conf"
TMP_DIR="$(mktemp -d)"
supervisor_pid=""
trap 'if [ -n "${supervisor_pid}" ]; then kill "${supervisor_pid}" 2>/dev/null || true; fi; rm -rf "${TMP_DIR}"' EXIT HUP INT TERM

test -x "${SUPERVISOR}"
test -x "${STOP_TOOL}"
grep -Fxq 'enabled=false' "${CONFIG}"
grep -Fxq 'pipewire_binary=/usr/bin/pipewire' "${CONFIG}"
grep -Fxq 'wireplumber_binary=/usr/bin/wireplumber' "${CONFIG}"

runtime_dir="${TMP_DIR}/run/user/$(id -u)"
control_dir="${TMP_DIR}/run/aqua"
mkdir -p "${runtime_dir}" "${control_dir}" "${TMP_DIR}/bin"
chmod 700 "${runtime_dir}" "${control_dir}"

dry_output="$(
    XDG_RUNTIME_DIR="${runtime_dir}" \
    AQUA_SESSION_RUNTIME_DIR="${runtime_dir}" \
    AQUA_SESSION_CONTROL_DIR="${control_dir}" \
    AQUA_MEDIA_SUPERVISOR_DRY_RUN=true \
    "${SUPERVISOR}"
)"
printf '%s\n' "${dry_output}" | grep -Fq 'status=ok mode=dry-run enabled=false ordered_start=pipewire,wireplumber ordered_stop=wireplumber,pipewire'
grep -Fq 'state=policy-ready' "${control_dir}/media-service-supervisor.state"

disabled_output="$(
    XDG_RUNTIME_DIR="${runtime_dir}" \
    AQUA_SESSION_RUNTIME_DIR="${runtime_dir}" \
    AQUA_SESSION_CONTROL_DIR="${control_dir}" \
    AQUA_MEDIA_SERVICES_ENABLED=false \
    "${SUPERVISOR}"
)"
printf '%s\n' "${disabled_output}" | grep -Fq 'status=disabled reason=packages-not-enabled root_media_daemon=false'
grep -Fq 'state=disabled' "${control_dir}/media-service-supervisor.state"
grep -Fq "service_owner_uid=$(id -u)" "${control_dir}/media-service-supervisor.state"

cat > "${TMP_DIR}/bin/pipewire" <<'EOF'
#!/bin/sh
set -eu
count=0
[ ! -f "${AQUA_TEST_ATTEMPT_FILE}" ] || count="$(cat "${AQUA_TEST_ATTEMPT_FILE}")"
count=$((count + 1))
printf '%s\n' "${count}" > "${AQUA_TEST_ATTEMPT_FILE}"
printf 'start pipewire %s\n' "${count}" >> "${AQUA_TEST_EVENT_FILE}"
: > "${AQUA_PIPEWIRE_READY_PATH}"
cleanup() {
    printf 'stop pipewire %s\n' "${count}" >> "${AQUA_TEST_EVENT_FILE}"
    rm -f "${AQUA_PIPEWIRE_READY_PATH}"
    exit 0
}
trap cleanup HUP INT TERM
if [ "${AQUA_TEST_ALWAYS_FAIL:-false}" = true ] || [ "${count}" -eq 1 ]; then
    sleep 2
    exit 23
fi
while :; do sleep 1; done
EOF
cat > "${TMP_DIR}/bin/wireplumber" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' 'start wireplumber' >> "${AQUA_TEST_EVENT_FILE}"
cleanup() {
    printf '%s\n' 'stop wireplumber' >> "${AQUA_TEST_EVENT_FILE}"
    exit 0
}
trap cleanup HUP INT TERM
while :; do sleep 1; done
EOF
chmod +x "${TMP_DIR}/bin/pipewire" "${TMP_DIR}/bin/wireplumber"

attempt_file="${TMP_DIR}/attempt"
event_file="${TMP_DIR}/events"
ready_file="${runtime_dir}/pipewire-0"
: > "${event_file}"
AQUA_TEST_ATTEMPT_FILE="${attempt_file}" \
AQUA_TEST_EVENT_FILE="${event_file}" \
AQUA_PIPEWIRE_READY_PATH="${ready_file}" \
XDG_RUNTIME_DIR="${runtime_dir}" \
AQUA_SESSION_RUNTIME_DIR="${runtime_dir}" \
AQUA_SESSION_CONTROL_DIR="${control_dir}" \
AQUA_MEDIA_SERVICES_ENABLED=true \
AQUA_PIPEWIRE_BIN="${TMP_DIR}/bin/pipewire" \
AQUA_WIREPLUMBER_BIN="${TMP_DIR}/bin/wireplumber" \
AQUA_PIPEWIRE_READY_KIND=file \
AQUA_MEDIA_MAX_RESTARTS=2 \
AQUA_MEDIA_RESTART_DELAY_SECONDS=0 \
AQUA_MEDIA_READY_TIMEOUT_SECONDS=4 \
AQUA_WIREPLUMBER_STABLE_SECONDS=1 \
AQUA_MEDIA_STABLE_SECONDS=30 \
AQUA_MEDIA_MONITOR_INTERVAL_SECONDS=1 \
AQUA_MEDIA_STOP_TIMEOUT_SECONDS=4 \
"${SUPERVISOR}" > "${TMP_DIR}/supervisor.log" 2>&1 &
supervisor_pid="$!"

waited=0
while [ "${waited}" -lt 20 ]; do
    if grep -Fq 'state=running' "${control_dir}/media-service-supervisor.state" 2>/dev/null &&
       grep -Fq 'attempts=2' "${control_dir}/media-service-supervisor.state" 2>/dev/null; then
        break
    fi
    sleep 1
    waited=$((waited + 1))
done
grep -Fq 'state=running' "${control_dir}/media-service-supervisor.state"
grep -Fq 'attempts=2' "${control_dir}/media-service-supervisor.state"
grep -Fq 'restarts=1' "${control_dir}/media-service-supervisor.state"
grep -Fq 'root_media_daemon=false' "${control_dir}/media-service-supervisor.state"
grep -Fq 'status=restarting failed_service=pipewire next_restart=1' "${TMP_DIR}/supervisor.log"

AQUA_SESSION_CONTROL_DIR="${control_dir}" \
AQUA_MEDIA_STOP_TIMEOUT_SECONDS=8 \
"${STOP_TOOL}" > "${TMP_DIR}/stop.log" 2>&1 &
stop_pid="$!"
wait "${supervisor_pid}"
supervisor_pid=""
wait "${stop_pid}"
grep -Fq 'status=ok supervisor_stopped=true services_stopped=wireplumber,pipewire ordered=true' "${TMP_DIR}/stop.log"
grep -Fq 'state=stopped' "${control_dir}/media-service-supervisor.state"
test ! -e "${control_dir}/media-service-supervisor.pid"
test ! -e "${ready_file}"

wireplumber_stop_line="$(grep -n '^stop wireplumber$' "${event_file}" | tail -n 1 | cut -d: -f1)"
pipewire_stop_line="$(grep -n '^stop pipewire 2$' "${event_file}" | tail -n 1 | cut -d: -f1)"
test "${wireplumber_stop_line}" -lt "${pipewire_stop_line}"

rm -f "${attempt_file}" "${event_file}" "${control_dir}/media-service-supervisor.state"
: > "${event_file}"
set +e
AQUA_TEST_ATTEMPT_FILE="${attempt_file}" \
AQUA_TEST_EVENT_FILE="${event_file}" \
AQUA_TEST_ALWAYS_FAIL=true \
AQUA_PIPEWIRE_READY_PATH="${ready_file}" \
XDG_RUNTIME_DIR="${runtime_dir}" \
AQUA_SESSION_RUNTIME_DIR="${runtime_dir}" \
AQUA_SESSION_CONTROL_DIR="${control_dir}" \
AQUA_MEDIA_SERVICES_ENABLED=true \
AQUA_PIPEWIRE_BIN="${TMP_DIR}/bin/pipewire" \
AQUA_WIREPLUMBER_BIN="${TMP_DIR}/bin/wireplumber" \
AQUA_PIPEWIRE_READY_KIND=file \
AQUA_MEDIA_MAX_RESTARTS=1 \
AQUA_MEDIA_RESTART_DELAY_SECONDS=0 \
AQUA_MEDIA_READY_TIMEOUT_SECONDS=4 \
AQUA_WIREPLUMBER_STABLE_SECONDS=1 \
AQUA_MEDIA_STABLE_SECONDS=30 \
AQUA_MEDIA_MONITOR_INTERVAL_SECONDS=1 \
AQUA_MEDIA_STOP_TIMEOUT_SECONDS=4 \
"${SUPERVISOR}" > "${TMP_DIR}/degraded.log" 2>&1
degraded_status="$?"
set -e
test "${degraded_status}" -ne 0
grep -Fq 'state=degraded' "${control_dir}/media-service-supervisor.state"
grep -Fq 'attempts=2' "${control_dir}/media-service-supervisor.state"
grep -Fq 'restarts=1' "${control_dir}/media-service-supervisor.state"
grep -Fq 'status=degraded reason=restart-limit failed_service=pipewire attempts=2 restarts=1' "${TMP_DIR}/degraded.log"

echo 'Aqua Linux per-user media service supervisor checks passed.'
