#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EVIDENCE_DIR="${AQUA_AUDIO_QEMU_EVIDENCE_DIR:-${ROOT_DIR}/build/audio-qemu}"
KERNEL="${KERNEL:-${ROOT_DIR}/build/audio-rootfs-contract/bzImage}"
ROOTFS="${ROOTFS:-${ROOT_DIR}/build/audio-rootfs-contract/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${EVIDENCE_DIR}/serial.log}"
WAV_CAPTURE="${WAV_CAPTURE:-${EVIDENCE_DIR}/playback.wav}"
WAV_ROUTE_PRIMARY="${WAV_ROUTE_PRIMARY:-${EVIDENCE_DIR}/route-primary.wav}"
WAV_ROUTE_SECONDARY="${WAV_ROUTE_SECONDARY:-${EVIDENCE_DIR}/route-secondary.wav}"
QEMU_PID_FILE="${QEMU_PID_FILE:-${EVIDENCE_DIR}/qemu.pid}"
QMP_SOCKET="${QMP_SOCKET:-${EVIDENCE_DIR}/qmp.sock}"
QMP_DEVICE_DELETE="${QMP_DEVICE_DELETE:-${ROOT_DIR}/scripts/qmp-device-delete.py}"
AUDIO_INPUT_INJECTOR="${AUDIO_INPUT_INJECTOR:-${EVIDENCE_DIR}/qemu-dbus-audio-input}"
AUDIO_INPUT_READY="${AUDIO_INPUT_READY:-${EVIDENCE_DIR}/input-injector.ready}"
AUDIO_INPUT_RESULT="${AUDIO_INPUT_RESULT:-${EVIDENCE_DIR}/input-injector.result}"
AUDIO_INPUT_LOG="${AUDIO_INPUT_LOG:-${EVIDENCE_DIR}/input-injector.log}"
AUDIO_INPUT_DISCONNECT_BYTES="${AUDIO_INPUT_DISCONNECT_BYTES:-9600}"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-180}"
AQUA_AUDIO_QEMU_CONTRACT="${AQUA_AUDIO_QEMU_CONTRACT:-output}"

case "${AQUA_AUDIO_QEMU_CONTRACT}" in
    output|input|input-signal|input-disconnect|service-loss|capture-service-loss|control-service-loss|restart-exhaustion|multi-route|hotplug|nondefault-unplug|active-nondefault-unplug|active-default-unplug) ;;
    *)
        echo "Unsupported audio QEMU contract: ${AQUA_AUDIO_QEMU_CONTRACT}" >&2
        exit 2
        ;;
esac

for tool in expect python3 qemu-system-x86_64; do
    command -v "${tool}" >/dev/null 2>&1 || {
        echo "Missing required tool: ${tool}" >&2
        exit 1
    }
done
for artifact in "${KERNEL}" "${ROOTFS}"; do
    test -s "${artifact}" || {
        echo "Missing audio QEMU artifact: ${artifact}" >&2
        echo 'Run scripts/rehearse-audio-rootfs-contract.sh first.' >&2
        exit 1
    }
done

mkdir -p "${EVIDENCE_DIR}"
rm -f "${SERIAL_LOG}" "${WAV_CAPTURE}" "${WAV_ROUTE_PRIMARY}" \
    "${WAV_ROUTE_SECONDARY}" "${QEMU_PID_FILE}" "${QMP_SOCKET}" \
    "${AUDIO_INPUT_READY}" "${AUDIO_INPUT_RESULT}" "${AUDIO_INPUT_LOG}"
cleanup() {
    if test -f "${QEMU_PID_FILE}"; then
        qemu_pid="$(cat "${QEMU_PID_FILE}" 2>/dev/null || true)"
        case "${qemu_pid}" in
            *[!0-9]*|'') ;;
            *) kill "${qemu_pid}" 2>/dev/null || true ;;
        esac
        rm -f "${QEMU_PID_FILE}"
    fi
    rm -f "${QMP_SOCKET}"
}
trap cleanup EXIT INT TERM
export ROOT_DIR KERNEL ROOTFS SERIAL_LOG WAV_CAPTURE QEMU_PID_FILE MEMORY CPUS TIMEOUT_SECONDS
export WAV_ROUTE_PRIMARY WAV_ROUTE_SECONDARY
export QMP_SOCKET QMP_DEVICE_DELETE
export AUDIO_INPUT_INJECTOR AUDIO_INPUT_READY AUDIO_INPUT_RESULT AUDIO_INPUT_LOG
export AUDIO_INPUT_DISCONNECT_BYTES
export AQUA_AUDIO_QEMU_CONTRACT
expect "${ROOT_DIR}/scripts/check-audio-qemu.exp"

grep -Fq \
    '[AQUA-AUDIO] stage=qemu-session status=ok base_session=true additional_group_members=true' \
    "${SERIAL_LOG}"

if test "${AQUA_AUDIO_QEMU_CONTRACT}" = output; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-AUDIO] stage=control-probe status=ok backend=aqua-audio-native default_sink=true volume=35 mute_cycle=true' \
        '[AQUA-AUDIO] stage=qemu-route status=ok default_sink=true requested_volume=0.35 mute_cycle=true' \
        '[AQUA-AUDIO] stage=media-probe status=ok direction=playback frames=48000 rate=48000 channels=2 format=s16le' \
        '[AQUA-AUDIO] stage=qemu-recovery status=ok failed_service=wireplumber restart_recovery=true owner_uid=1000' \
        '[AQUA-AUDIO] stage=qemu-media status=ok declared_device=intel-hda codec=hda-duplex backend=wav playback=true input_node=true input_stream=false restart_recovery=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_CAPTURE}"
    echo 'Aqua Linux declared-device QEMU audio output check passed.'
    echo "Playback capture: ${WAV_CAPTURE}"
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = input; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-AUDIO] stage=media-probe status=ok direction=capture frames=4800 rate=48000 channels=2 format=s16le peak_abs=0 pattern=silence' \
        '[AQUA-AUDIO] stage=qemu-capture status=ok pattern=zero-pcm' \
        '[AQUA-AUDIO] stage=qemu-input status=ok declared_device=intel-hda codec=hda-duplex backend=none capture=true frames=4800 controlled_pattern=zero-pcm'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    echo 'Aqua Linux declared-device QEMU audio input check passed.'
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = input-signal; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-AUDIO] stage=media-probe status=ok direction=capture frames=4800 rate=48000 channels=2 format=s16le' \
        'pattern=bipolar-injected' \
        '[AQUA-AUDIO] stage=qemu-capture status=ok pattern=bipolar-injected' \
        '[AQUA-AUDIO] stage=qemu-input-signal status=ok declared_device=intel-hda codec=hda-duplex backend=dbus capture=true frames=4800 host_microphone=false'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    for result in \
        'status=ok' \
        'format=s16le' \
        'rate=48000' \
        'channels=2' \
        'amplitude=4096' \
        'pattern=square-1khz'
    do
        grep -Fxq "${result}" "${AUDIO_INPUT_RESULT}"
    done
    served_bytes="$(sed -n 's/^bytes_served=//p' "${AUDIO_INPUT_RESULT}")"
    case "${served_bytes}" in
        *[!0-9]*|'') echo 'Invalid D-Bus input byte count.' >&2; exit 1 ;;
    esac
    test "${served_bytes}" -ge 19200
    echo 'Aqua Linux deterministic non-silent QEMU audio input check passed.'
    echo "Injector result: ${AUDIO_INPUT_RESULT}"
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = input-disconnect; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-AUDIO] stage=media-probe status=failed reason=invalid-injected-signal' \
        '[AQUA-AUDIO] stage=qemu-input-failure status=ok backend=dbus expected_failure=true false_success=false services_running=true recovery_shell=true' \
        '[AQUA-AUDIO] stage=qemu-input-disconnect status=ok declared_device=intel-hda codec=hda-duplex backend=dbus bytes_before_failure=9600 host_microphone=false'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    grep -Fxq 'status=disconnected' "${AUDIO_INPUT_RESULT}"
    grep -Fxq 'reason=injected-read-failure' "${AUDIO_INPUT_RESULT}"
    grep -Fxq 'bytes_served=9600' "${AUDIO_INPUT_RESULT}"
    echo 'Aqua Linux QEMU audio input disconnect check passed.'
    echo "Injector result: ${AUDIO_INPUT_RESULT}"
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = service-loss; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-AUDIO] stage=media-probe status=active direction=playback frames=480' \
        '[AQUA-AUDIO] stage=media-probe status=interrupted direction=playback reason=pcm-io' \
        '[AQUA-AUDIO] stage=qemu-service-loss status=ok failed_service=pipewire active_stream_aborted=true false_success=false restart_recovery=true playback_after=true recovery_shell=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=ok direction=playback frames=48000 rate=48000 channels=2 format=s16le' "${SERIAL_LOG}")" -eq 1
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_CAPTURE}"
    echo 'Aqua Linux active audio service-loss recovery check passed.'
    echo "Playback capture: ${WAV_CAPTURE}"
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = capture-service-loss; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-AUDIO] stage=media-probe status=active direction=capture frames=480' \
        '[AQUA-AUDIO] stage=media-probe status=interrupted direction=capture reason=pcm-io' \
        '[AQUA-AUDIO] stage=qemu-capture-service-loss status=ok failed_service=pipewire active_stream_aborted=true false_success=false restart_recovery=true capture_after=true controlled_pattern=zero-pcm host_microphone=false recovery_shell=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=ok direction=capture frames=4800 rate=48000 channels=2 format=s16le peak_abs=0 pattern=silence' "${SERIAL_LOG}")" -eq 1
    echo 'Aqua Linux active audio capture service-loss recovery check passed.'
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = control-service-loss; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-AUDIO] stage=control-probe status=failed operation=open' \
        '[AQUA-AUDIO] stage=qemu-control-service-loss status=ok failed_service=pipewire control_rejected=true false_acknowledgement=false restart_recovery=true control_after=true recovery_shell=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=control-probe status=ok backend=aqua-audio-native default_sink=true volume=35 mute_cycle=true' "${SERIAL_LOG}")" -eq 2
    test "$(grep -Fc '[AQUA-AUDIO] stage=control-probe status=failed' "${SERIAL_LOG}")" -eq 1
    echo 'Aqua Linux audio control service-loss acknowledgement check passed.'
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = restart-exhaustion; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true' \
        '[AQUA-MEDIA] stage=media-service-supervisor status=degraded reason=restart-limit failed_service=pipewire attempts=4 restarts=3' \
        '[AQUA-AUDIO] stage=qemu-restart-exhaustion status=ok failed_service=pipewire attempts=4 restarts=3 restart_limit=true services_stopped=true playback_blocked=true recovery_shell=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-MEDIA] stage=media-service-supervisor status=restarting failed_service=pipewire' "${SERIAL_LOG}")" -eq 3
    echo 'Aqua Linux QEMU audio restart-exhaustion check passed.'
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = multi-route; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-output outputs=2 capture_node=false' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sinks=true route_profile=true' \
        '[AQUA-AUDIO] stage=route-probe status=ok outputs=2 previous_default=true requested_node=true default_changed=true' \
        '[AQUA-AUDIO] stage=qemu-route-switch status=ok outputs=2 default_changed=true playback_before=true playback_after=true' \
        '[AQUA-AUDIO] stage=qemu-multi-route status=ok controllers=2 codecs=2 backends=2 default_changed=true captures=2'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=ok direction=playback frames=48000 rate=48000 channels=2 format=s16le' "${SERIAL_LOG}")" -eq 2
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_ROUTE_PRIMARY}"
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_ROUTE_SECONDARY}"
    echo 'Aqua Linux multi-device QEMU audio route check passed.'
    echo "Primary route capture: ${WAV_ROUTE_PRIMARY}"
    echo "Secondary route capture: ${WAV_ROUTE_SECONDARY}"
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = hotplug; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-output outputs=2 capture_node=false' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sinks=true route_profile=true' \
        '[AQUA-AUDIO] stage=route-probe status=ok outputs=2 previous_default=true requested_node=true default_changed=true requested_slot=05.0' \
        '[AQUA-AUDIO] stage=qemu-device-unplug status=ok device=aqua-hda-secondary event=DEVICE_DELETED alsa_outputs=1' \
        '[AQUA-AUDIO] stage=hotplug-probe status=ok outputs=1 default_output=true graph_ready=true' \
        '[AQUA-AUDIO] stage=qemu-hotplug status=ok removed_default=true fallback_output=true playback_after=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=ok direction=playback frames=48000 rate=48000 channels=2 format=s16le' "${SERIAL_LOG}")" -eq 3
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_ROUTE_PRIMARY}"
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_ROUTE_SECONDARY}"
    echo 'Aqua Linux QEMU audio default-device unplug fallback check passed.'
    echo "Remaining route capture: ${WAV_ROUTE_PRIMARY}"
    echo "Removed route capture: ${WAV_ROUTE_SECONDARY}"
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = nondefault-unplug; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-output outputs=2 capture_node=false' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sinks=true route_profile=true' \
        '[AQUA-AUDIO] stage=topology-probe status=ok outputs=2 default_slot=04.0 graph_ready=true' \
        '[AQUA-AUDIO] stage=qemu-device-unplug status=ok device=aqua-hda-secondary event=DEVICE_DELETED alsa_outputs=1' \
        '[AQUA-AUDIO] stage=topology-probe status=ok outputs=1 default_slot=04.0 graph_ready=true' \
        '[AQUA-AUDIO] stage=qemu-nondefault-unplug status=ok removed_default=false default_unchanged=true playback_before=true playback_after=true recovery_shell=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=ok direction=playback frames=48000 rate=48000 channels=2 format=s16le' "${SERIAL_LOG}")" -eq 2
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_ROUTE_PRIMARY}"
    echo 'Aqua Linux QEMU non-default audio device unplug check passed.'
    echo "Stable default route capture: ${WAV_ROUTE_PRIMARY}"
elif test "${AQUA_AUDIO_QEMU_CONTRACT}" = active-nondefault-unplug; then
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-output outputs=2 capture_node=false' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sinks=true route_profile=true' \
        '[AQUA-AUDIO] stage=topology-probe status=ok outputs=2 default_slot=04.0 graph_ready=true' \
        '[AQUA-AUDIO] stage=media-probe status=active direction=playback frames=480' \
        '[AQUA-AUDIO] stage=qemu-device-unplug status=ok device=aqua-hda-secondary event=DEVICE_DELETED alsa_outputs=1' \
        '[AQUA-AUDIO] stage=topology-probe status=ok outputs=1 default_slot=04.0 graph_ready=true' \
        '[AQUA-AUDIO] stage=qemu-active-nondefault-unplug status=ok removed_default=false active_stream_survived=true false_interruption=true default_unchanged=true recovery_shell=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=ok direction=playback frames=48000 rate=48000 channels=2 format=s16le' "${SERIAL_LOG}")" -eq 1
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=interrupted direction=playback' "${SERIAL_LOG}")" -eq 0
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_ROUTE_PRIMARY}"
    echo 'Aqua Linux active playback non-default audio unplug check passed.'
    echo "Uninterrupted default route capture: ${WAV_ROUTE_PRIMARY}"
else
    for marker in \
        '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-output outputs=2 capture_node=false' \
        '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sinks=true route_profile=true' \
        '[AQUA-AUDIO] stage=route-probe status=ok outputs=2 previous_default=true requested_node=true default_changed=true requested_slot=05.0' \
        '[AQUA-AUDIO] stage=media-probe status=active direction=playback frames=480' \
        '[AQUA-AUDIO] stage=media-probe status=interrupted direction=playback reason=route-loss frames=480' \
        '[AQUA-AUDIO] stage=qemu-device-unplug status=ok device=aqua-hda-secondary event=DEVICE_DELETED alsa_outputs=1' \
        '[AQUA-AUDIO] stage=hotplug-probe status=ok outputs=1 default_output=true graph_ready=true' \
        '[AQUA-AUDIO] stage=qemu-active-default-unplug status=ok removed_default=true active_stream_aborted=true false_success=false fallback_output=true playback_after=true recovery_shell=true'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=ok direction=playback frames=48000 rate=48000 channels=2 format=s16le' "${SERIAL_LOG}")" -eq 1
    test "$(grep -Fc '[AQUA-AUDIO] stage=media-probe status=interrupted direction=playback' "${SERIAL_LOG}")" -eq 1
    python3 "${ROOT_DIR}/scripts/check-qemu-audio-wave.py" "${WAV_ROUTE_PRIMARY}"
    echo 'Aqua Linux active default audio device unplug fallback check passed.'
    echo "Fallback route capture: ${WAV_ROUTE_PRIMARY}"
fi
echo "Serial log: ${SERIAL_LOG}"
