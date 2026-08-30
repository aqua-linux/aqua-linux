#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
EVIDENCE_DIR="${AQUA_AUDIO_QEMU_EVIDENCE_DIR:-${ROOT_DIR}/build/audio-qemu}"
KERNEL="${KERNEL:-${ROOT_DIR}/build/audio-rootfs-contract/bzImage}"
ROOTFS="${ROOTFS:-${ROOT_DIR}/build/audio-rootfs-contract/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${EVIDENCE_DIR}/serial.log}"
WAV_CAPTURE="${WAV_CAPTURE:-${EVIDENCE_DIR}/playback.wav}"
QEMU_PID_FILE="${QEMU_PID_FILE:-${EVIDENCE_DIR}/qemu.pid}"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-180}"
AQUA_AUDIO_QEMU_CONTRACT="${AQUA_AUDIO_QEMU_CONTRACT:-output}"

case "${AQUA_AUDIO_QEMU_CONTRACT}" in
    output|input) ;;
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
rm -f "${SERIAL_LOG}" "${WAV_CAPTURE}" "${QEMU_PID_FILE}"
cleanup() {
    if test -f "${QEMU_PID_FILE}"; then
        qemu_pid="$(cat "${QEMU_PID_FILE}" 2>/dev/null || true)"
        case "${qemu_pid}" in
            *[!0-9]*|'') ;;
            *) kill "${qemu_pid}" 2>/dev/null || true ;;
        esac
        rm -f "${QEMU_PID_FILE}"
    fi
}
trap cleanup EXIT INT TERM
export ROOT_DIR KERNEL ROOTFS SERIAL_LOG WAV_CAPTURE QEMU_PID_FILE MEMORY CPUS TIMEOUT_SECONDS
export AQUA_AUDIO_QEMU_CONTRACT
expect "${ROOT_DIR}/scripts/check-audio-qemu.exp"

for marker in \
    '[AQUA-AUDIO] stage=qemu-session status=ok base_session=true additional_group_members=true' \
    '[AQUA-AUDIO] stage=qemu-device status=ok driver=snd_hda_intel codec=hda-duplex playback=true capture_node=true' \
    '[AQUA-AUDIO] stage=qemu-service status=ok owner_uid=1000 pipewire=true wireplumber=true sink=true source=true'
do
    grep -Fq "${marker}" "${SERIAL_LOG}"
done

if test "${AQUA_AUDIO_QEMU_CONTRACT}" = output; then
    for marker in \
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
else
    for marker in \
        '[AQUA-AUDIO] stage=media-probe status=ok direction=capture frames=4800 rate=48000 channels=2 format=s16le peak_abs=0 pattern=silence' \
        '[AQUA-AUDIO] stage=qemu-capture status=ok pattern=zero-pcm' \
        '[AQUA-AUDIO] stage=qemu-input status=ok declared_device=intel-hda codec=hda-duplex backend=none capture=true frames=4800 controlled_pattern=zero-pcm'
    do
        grep -Fq "${marker}" "${SERIAL_LOG}"
    done
    echo 'Aqua Linux declared-device QEMU audio input check passed.'
fi
echo "Serial log: ${SERIAL_LOG}"
