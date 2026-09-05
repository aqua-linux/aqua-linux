#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SUPERVISOR="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-supervisor"
STOP_TOOL="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-network-service-stop"
HOOK="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/usr/bin/aqua-udhcpc-hook"
CONFIG="${ROOT_DIR}/br2-external/aqua/rootfs-overlay/etc/aqua/network-services.conf"
TMP_DIR="$(mktemp -d)"
supervisor_pid=""
trap 'if [ -n "${supervisor_pid}" ]; then kill "${supervisor_pid}" 2>/dev/null || true; fi; rm -rf "${TMP_DIR}"' EXIT HUP INT TERM

test -x "${SUPERVISOR}"
test -x "${STOP_TOOL}"
test -x "${HOOK}"
grep -Fxq 'enabled=false' "${CONFIG}"
grep -Fxq 'legacy_owner_disabled=false' "${CONFIG}"
grep -Fxq 'interface=eth0' "${CONFIG}"

control_dir="${TMP_DIR}/run/aqua-network"
interface_root="${TMP_DIR}/sys/class/net"
mkdir -p "${control_dir}" "${TMP_DIR}/bin" "${interface_root}/eth0"
chmod 755 "${control_dir}"

dry_output="$(
    AQUA_NETWORK_CONTROL_DIR="${control_dir}" \
    AQUA_NETWORK_SUPERVISOR_DRY_RUN=true \
    "${SUPERVISOR}"
)"
printf '%s\n' "${dry_output}" | grep -Fq 'status=ok mode=dry-run enabled=false interface=eth0 legacy_owner_disabled=false'
grep -Fq 'state=policy-ready' "${control_dir}/network-service-supervisor.state"
grep -Fq 'settings_management=false' "${control_dir}/network-service-supervisor.state"
grep -Fq 'wifi_packaged=false' "${control_dir}/network-service-supervisor.state"

disabled_output="$(
    AQUA_NETWORK_CONTROL_DIR="${control_dir}" \
    AQUA_NETWORK_SERVICES_ENABLED=false \
    "${SUPERVISOR}"
)"
printf '%s\n' "${disabled_output}" | grep -Fq 'status=disabled reason=transition-not-enabled boot_owner=none'
grep -Fq 'state=disabled' "${control_dir}/network-service-supervisor.state"

set +e
conflict_output="$(
    AQUA_NETWORK_CONTROL_DIR="${control_dir}" \
    AQUA_NETWORK_SERVICES_ENABLED=true \
    AQUA_NETWORK_SUPERVISOR_TEST_MODE=true \
    "${SUPERVISOR}" 2>&1
)"
conflict_status="$?"
set -e
test "${conflict_status}" -ne 0
printf '%s\n' "${conflict_output}" | grep -Fq 'status=blocked reason=legacy-owner-not-declared-disabled owner=buildroot-s40network'

if [ "$(id -u)" -ne 0 ]; then
    set +e
    ownership_output="$(
        AQUA_NETWORK_CONTROL_DIR="${control_dir}" \
        AQUA_NETWORK_SERVICES_ENABLED=true \
        AQUA_NETWORK_LEGACY_OWNER_DISABLED=true \
        "${SUPERVISOR}" 2>&1
    )"
    ownership_status="$?"
    set -e
    test "${ownership_status}" -ne 0
    printf '%s\n' "${ownership_output}" | grep -Fq 'status=blocked reason=unsafe-control-directory'
fi

cat > "${TMP_DIR}/bin/default.script" <<'EOF'
#!/bin/sh
set -eu
printf '%s %s\n' "$1" "${interface}" >> "${AQUA_TEST_EVENT_FILE}"
case "$1" in
    bound|renew)
        printf '%s\n' \
            'Iface Destination Gateway Flags RefCnt Use Metric Mask MTU Window IRTT' \
            'eth0 00000000 0202000A 0003 0 0 0 00000000 0 0 0' > "${AQUA_TEST_ROUTE_FILE}"
        printf '%s\n' 'nameserver 10.0.2.3 # eth0' > "${AQUA_TEST_RESOLVER_FILE}"
        ;;
    *)
        : > "${AQUA_TEST_ROUTE_FILE}"
        : > "${AQUA_TEST_RESOLVER_FILE}"
        ;;
esac
EOF
cat > "${TMP_DIR}/bin/udhcpc" <<'EOF'
#!/bin/sh
set -eu
count=0
[ ! -f "${AQUA_TEST_ATTEMPT_FILE}" ] || count="$(cat "${AQUA_TEST_ATTEMPT_FILE}")"
count=$((count + 1))
printf '%s\n' "${count}" > "${AQUA_TEST_ATTEMPT_FILE}"
hook=""
interface=""
while [ "$#" -gt 0 ]; do
    case "$1" in
        -s) hook="$2"; shift 2 ;;
        -i) interface="$2"; shift 2 ;;
        *) shift ;;
    esac
done
export interface
if [ "${AQUA_TEST_ALWAYS_FAIL:-false}" = true ]; then
    exit 23
fi
"${hook}" bound
cleanup() {
    "${hook}" deconfig
    exit 0
}
trap cleanup HUP INT TERM
if [ "${count}" -eq 1 ]; then
    waited=0
    while ! grep -Fq 'state=running' "${AQUA_NETWORK_CONTROL_DIR}/network-service-supervisor.state"; do
        [ "${waited}" -lt 10 ] || exit 24
        sleep 1
        waited=$((waited + 1))
    done
    # Model a client crash without racing a separate lease/DNS loss event.
    exit 23
fi
while :; do sleep 1; done
EOF
chmod +x "${TMP_DIR}/bin/default.script" "${TMP_DIR}/bin/udhcpc"

event_file="${TMP_DIR}/events"
attempt_file="${TMP_DIR}/attempts"
route_file="${TMP_DIR}/route"
resolver_file="${TMP_DIR}/resolv.conf"
: > "${event_file}"
AQUA_TEST_EVENT_FILE="${event_file}" \
AQUA_TEST_ATTEMPT_FILE="${attempt_file}" \
AQUA_TEST_ROUTE_FILE="${route_file}" \
AQUA_TEST_RESOLVER_FILE="${resolver_file}" \
AQUA_NETWORK_CONTROL_DIR="${control_dir}" \
AQUA_NETWORK_SERVICES_ENABLED=true \
AQUA_NETWORK_LEGACY_OWNER_DISABLED=true \
AQUA_NETWORK_SUPERVISOR_TEST_MODE=true \
AQUA_NETWORK_SYS_CLASS_NET="${interface_root}" \
AQUA_NETWORK_ROUTE_TABLE="${route_file}" \
AQUA_NETWORK_RESOLVER_FILE="${resolver_file}" \
AQUA_UDHCPC_BIN="${TMP_DIR}/bin/udhcpc" \
AQUA_UDHCPC_HOOK="${HOOK}" \
AQUA_UDHCPC_DEFAULT_SCRIPT="${TMP_DIR}/bin/default.script" \
AQUA_NETWORK_MAX_RESTARTS=3 \
AQUA_NETWORK_RESTART_DELAY_SECONDS=0 \
AQUA_NETWORK_READY_TIMEOUT_SECONDS=4 \
AQUA_NETWORK_LEASE_LOSS_GRACE_SECONDS=1 \
AQUA_NETWORK_MONITOR_INTERVAL_SECONDS=1 \
AQUA_NETWORK_STOP_TIMEOUT_SECONDS=4 \
"${SUPERVISOR}" > "${TMP_DIR}/supervisor.log" 2>&1 &
supervisor_pid="$!"

waited=0
while [ "${waited}" -lt 20 ]; do
    if grep -Fq 'state=running' "${control_dir}/network-service-supervisor.state" 2>/dev/null &&
       grep -Fq 'attempts=2' "${control_dir}/network-service-supervisor.state" 2>/dev/null; then
        break
    fi
    sleep 1
    waited=$((waited + 1))
done
grep -Fq 'state=running' "${control_dir}/network-service-supervisor.state"
grep -Fq 'attempts=2' "${control_dir}/network-service-supervisor.state"
grep -Fq 'restarts=1' "${control_dir}/network-service-supervisor.state"
grep -Fq 'lease_ready=true' "${control_dir}/network-service-supervisor.state"
grep -Fq 'route_ready=true' "${control_dir}/network-service-supervisor.state"
grep -Fq 'dns_ready=true' "${control_dir}/network-service-supervisor.state"
grep -Fxq 'nameserver 10.0.2.3' "${resolver_file}"
grep -Fq 'policy_owner=aqua-network-service-supervisor' "${control_dir}/network-service-supervisor.state"
grep -Fq 'status=restarting failure=dhcp-client next_restart=1' "${TMP_DIR}/supervisor.log"

: > "${route_file}"
waited=0
while [ "${waited}" -lt 20 ]; do
    if grep -Fq 'state=running' "${control_dir}/network-service-supervisor.state" 2>/dev/null &&
       grep -Fq 'attempts=3' "${control_dir}/network-service-supervisor.state" 2>/dev/null; then
        break
    fi
    sleep 1
    waited=$((waited + 1))
done
grep -Fq 'attempts=3' "${control_dir}/network-service-supervisor.state"
grep -Fq 'restarts=2' "${control_dir}/network-service-supervisor.state"
grep -Fq 'status=restarting failure=route-lost next_restart=2' "${TMP_DIR}/supervisor.log"

: > "${resolver_file}"
waited=0
while [ "${waited}" -lt 20 ]; do
    if grep -Fq 'state=running' "${control_dir}/network-service-supervisor.state" 2>/dev/null &&
       grep -Fq 'attempts=4' "${control_dir}/network-service-supervisor.state" 2>/dev/null; then
        break
    fi
    sleep 1
    waited=$((waited + 1))
done
grep -Fq 'attempts=4' "${control_dir}/network-service-supervisor.state"
grep -Fq 'restarts=3' "${control_dir}/network-service-supervisor.state"
grep -Fq 'status=restarting failure=dns-lost next_restart=3' "${TMP_DIR}/supervisor.log"

AQUA_NETWORK_CONTROL_DIR="${control_dir}" \
AQUA_NETWORK_STOP_TIMEOUT_SECONDS=8 \
"${STOP_TOOL}" > "${TMP_DIR}/stop.log" 2>&1 &
stop_pid="$!"
wait "${supervisor_pid}"
supervisor_pid=""
wait "${stop_pid}"
grep -Fq 'status=ok supervisor_stopped=true service_stopped=udhcpc' "${TMP_DIR}/stop.log"
grep -Fq 'state=stopped' "${control_dir}/network-service-supervisor.state"
test ! -e "${control_dir}/network-service-supervisor.pid"
test ! -e "${control_dir}/lease.ready"
grep -Fq 'bound eth0' "${event_file}"
grep -Fq 'deconfig eth0' "${event_file}"

rm -f "${attempt_file}" "${control_dir}/network-service-supervisor.state"
set +e
AQUA_TEST_EVENT_FILE="${event_file}" \
AQUA_TEST_ATTEMPT_FILE="${attempt_file}" \
AQUA_TEST_ROUTE_FILE="${route_file}" \
AQUA_TEST_RESOLVER_FILE="${resolver_file}" \
AQUA_TEST_ALWAYS_FAIL=true \
AQUA_NETWORK_CONTROL_DIR="${control_dir}" \
AQUA_NETWORK_SERVICES_ENABLED=true \
AQUA_NETWORK_LEGACY_OWNER_DISABLED=true \
AQUA_NETWORK_SUPERVISOR_TEST_MODE=true \
AQUA_NETWORK_SYS_CLASS_NET="${interface_root}" \
AQUA_NETWORK_ROUTE_TABLE="${route_file}" \
AQUA_NETWORK_RESOLVER_FILE="${resolver_file}" \
AQUA_UDHCPC_BIN="${TMP_DIR}/bin/udhcpc" \
AQUA_UDHCPC_HOOK="${HOOK}" \
AQUA_UDHCPC_DEFAULT_SCRIPT="${TMP_DIR}/bin/default.script" \
AQUA_NETWORK_MAX_RESTARTS=1 \
AQUA_NETWORK_RESTART_DELAY_SECONDS=0 \
AQUA_NETWORK_READY_TIMEOUT_SECONDS=2 \
AQUA_NETWORK_LEASE_LOSS_GRACE_SECONDS=1 \
AQUA_NETWORK_MONITOR_INTERVAL_SECONDS=1 \
AQUA_NETWORK_STOP_TIMEOUT_SECONDS=2 \
"${SUPERVISOR}" > "${TMP_DIR}/degraded.log" 2>&1
degraded_status="$?"
set -e
test "${degraded_status}" -ne 0
grep -Fq 'state=degraded' "${control_dir}/network-service-supervisor.state"
grep -Fq 'attempts=2' "${control_dir}/network-service-supervisor.state"
grep -Fq 'restarts=1' "${control_dir}/network-service-supervisor.state"
grep -Fq 'failure=dhcp-client' "${control_dir}/network-service-supervisor.state"
grep -Fq 'status=degraded reason=restart-limit failure=dhcp-client attempts=2 restarts=1' "${TMP_DIR}/degraded.log"

set +e
interface=wlan0 \
AQUA_NETWORK_INTERFACE=eth0 \
AQUA_NETWORK_READY_FILE="${control_dir}/lease.ready" \
AQUA_UDHCPC_DEFAULT_SCRIPT="${TMP_DIR}/bin/default.script" \
AQUA_TEST_EVENT_FILE="${event_file}" \
"${HOOK}" bound > "${TMP_DIR}/mismatch.log" 2>&1
mismatch_status="$?"
set -e
test "${mismatch_status}" -ne 0
grep -Fq 'status=blocked event=bound interface=wlan0' "${TMP_DIR}/mismatch.log"

echo 'Aqua Linux root-owned network service supervisor checks passed.'
