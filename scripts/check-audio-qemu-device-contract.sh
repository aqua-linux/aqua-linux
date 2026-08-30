#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
DEFAULT_CONFIG="${ROOT_DIR}/br2-external/aqua/configs/aqua_x86_64_defconfig"
AUDIO_CONFIG="${ROOT_DIR}/br2-external/aqua/configs/aqua_x86_64_audio_rehearsal_defconfig"
FRAGMENT="${ROOT_DIR}/br2-external/aqua/board/aqua/x86_64/linux-audio-qemu.config"
RUNNER="${ROOT_DIR}/scripts/check-audio-qemu.sh"
INPUT_RUNNER="${ROOT_DIR}/scripts/check-audio-input-qemu.sh"
SIGNAL_INPUT_RUNNER="${ROOT_DIR}/scripts/check-audio-signal-input-qemu.sh"
DISCONNECT_INPUT_RUNNER="${ROOT_DIR}/scripts/check-audio-input-disconnect-qemu.sh"
SERVICE_LOSS_RUNNER="${ROOT_DIR}/scripts/check-audio-active-service-loss-qemu.sh"
RESTART_EXHAUSTION_RUNNER="${ROOT_DIR}/scripts/check-audio-restart-exhaustion-qemu.sh"
MULTI_ROUTE_RUNNER="${ROOT_DIR}/scripts/check-audio-multi-route-qemu.sh"
HOTPLUG_RUNNER="${ROOT_DIR}/scripts/check-audio-hotplug-qemu.sh"
QMP_DELETE="${ROOT_DIR}/scripts/qmp-device-delete.py"
EXPECT_SCRIPT="${ROOT_DIR}/scripts/check-audio-qemu.exp"
PROBE_CONFIG="${ROOT_DIR}/br2-external/aqua/package/aqua-audio-probe/Config.in"
PROBE_MAKE="${ROOT_DIR}/br2-external/aqua/package/aqua-audio-probe/aqua-audio-probe.mk"
DBUS_INPUT_SOURCE="${ROOT_DIR}/scripts/qemu-dbus-audio-input.c"

for assignment in \
    CONFIG_SOUND=y \
    CONFIG_SND=y \
    CONFIG_SND_HDA_INTEL=y \
    CONFIG_SND_HDA_GENERIC=y \
    CONFIG_HOTPLUG_PCI=y \
    CONFIG_HOTPLUG_PCI_ACPI=y
do
    grep -Fxq "${assignment}" "${FRAGMENT}"
done

test -x "${RESTART_EXHAUSTION_RUNNER}"
grep -Fq 'AQUA_AUDIO_QEMU_CONTRACT=restart-exhaustion' "${RESTART_EXHAUSTION_RUNNER}"
for contract in \
    'status=degraded reason=restart-limit failed_service=pipewire attempts=4 restarts=3' \
    'restart_limit=true services_stopped=true playback_blocked=true recovery_shell=true' \
    'max_restarts=3' \
    'media-service-supervisor.pid' \
    'pipewire-0'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}" \
        "${RESTART_EXHAUSTION_RUNNER}"
done

test -x "${SERVICE_LOSS_RUNNER}"
grep -Fq 'AQUA_AUDIO_QEMU_CONTRACT=service-loss' "${SERVICE_LOSS_RUNNER}"
for contract in \
    'aqua-audio-probe playback-expect-interruption' \
    'status=active direction=playback frames=480' \
    'status=interrupted direction=playback reason=pcm-io' \
    'failed_service=pipewire active_stream_aborted=true false_success=false' \
    'restart_recovery=true playback_after=true recovery_shell=true'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}" \
        "${SERVICE_LOSS_RUNNER}"
done

test -x "${DISCONNECT_INPUT_RUNNER}"
grep -Fq 'AQUA_AUDIO_QEMU_CONTRACT=input-disconnect' "${DISCONNECT_INPUT_RUNNER}"
for contract in \
    'AUDIO_INPUT_DISCONNECT_BYTES' \
    'status=disconnected' \
    'reason=injected-read-failure' \
    'expected_failure=true false_success=false services_running=true recovery_shell=true' \
    'bytes_before_failure=9600 host_microphone=false'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}" \
        "${DISCONNECT_INPUT_RUNNER}" "${DBUS_INPUT_SOURCE}"
done

test -x "${SIGNAL_INPUT_RUNNER}"
grep -Fq 'AQUA_AUDIO_QEMU_CONTRACT="${AQUA_AUDIO_QEMU_CONTRACT:-input-signal}"' "${SIGNAL_INPUT_RUNNER}"
for contract in \
    'dbus,id=aqua-audio,nsamples=480' \
    'dbus,p2p=on,audiodev=aqua-audio' \
    'RegisterInListener' \
    'org.qemu.Display1.AudioInListener' \
    'aqua-audio-probe capture-signal' \
    'pattern=bipolar-injected' \
    'backend=dbus capture=true frames=4800 host_microphone=false'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}" \
        "${SIGNAL_INPUT_RUNNER}" "${DBUS_INPUT_SOURCE}"
done

test -x "${HOTPLUG_RUNNER}"
test -x "${QMP_DELETE}"
grep -Fq 'AQUA_AUDIO_QEMU_CONTRACT=hotplug' "${HOTPLUG_RUNNER}"
for contract in \
    'addr=04.0' \
    'addr=05.0' \
    'qmp-device-delete.py' \
    '"execute": "device_del"' \
    'DEVICE_DELETED' \
    'aqua-audio-probe routes-secondary' \
    'aqua-audio-probe fallback' \
    'stage=hotplug-probe status=ok outputs=1' \
    'stage=qemu-hotplug status=ok removed_default=true'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}" "${QMP_DELETE}"
done

grep -Fxq \
    'BR2_LINUX_KERNEL_CONFIG_FRAGMENT_FILES="$(BR2_EXTERNAL_AQUA_PATH)/board/aqua/x86_64/linux-audio-qemu.config"' \
    "${AUDIO_CONFIG}"
grep -Fxq 'BR2_PACKAGE_AQUA_AUDIO_PROBE=y' "${AUDIO_CONFIG}"
grep -Fq 'depends on BR2_PACKAGE_ALSA_LIB' "${PROBE_CONFIG}"
grep -Fq 'depends on BR2_PACKAGE_AQUA_AUDIO_NATIVE' "${PROBE_CONFIG}"
grep -Fq 'AQUA_AUDIO_PROBE_DEPENDENCIES = alsa-lib aqua-audio-native' "${PROBE_MAKE}"

if grep -Eq 'BR2_PACKAGE_(ALSA_LIB|PIPEWIRE|WIREPLUMBER|AQUA_AUDIO_PROBE)=y|LINUX_KERNEL_CONFIG_FRAGMENT_FILES=.*audio' \
    "${DEFAULT_CONFIG}"; then
    echo 'Default image unexpectedly enables the audio profile.' >&2
    exit 1
fi

for contract in \
    'wav,id=aqua-audio' \
    'ich9-intel-hda' \
    'hda-duplex,audiodev=aqua-audio' \
    'stage=qemu-session status=ok base_session=true additional_group_members=true' \
    'stage=control-probe status=ok backend=aqua-audio-native' \
    'direction=playback frames=48000 rate=48000 channels=2 format=s16le' \
    'input_stream=false' \
    'restart_recovery=true'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}"
done

test -x "${INPUT_RUNNER}"
grep -Fq 'AQUA_AUDIO_QEMU_CONTRACT=input' "${INPUT_RUNNER}"
for contract in \
    '-audiodev "none,id=aqua-audio' \
    'aqua-audio-probe capture-silence' \
    'stage=qemu-capture status=ok pattern=zero-pcm' \
    'direction=capture frames=4800 rate=48000 channels=2 format=s16le peak_abs=0 pattern=silence' \
    'backend=none capture=true frames=4800 controlled_pattern=zero-pcm'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}"
done

test -x "${MULTI_ROUTE_RUNNER}"
grep -Fq 'AQUA_AUDIO_QEMU_CONTRACT=multi-route' "${MULTI_ROUTE_RUNNER}"
for contract in \
    'wav,id=aqua-route-primary' \
    'wav,id=aqua-route-secondary' \
    'ich9-intel-hda,id=aqua-hda-primary' \
    'ich9-intel-hda,id=aqua-hda-secondary' \
    'hda-output,bus=aqua-hda-primary.0,audiodev=aqua-route-primary' \
    'hda-output,bus=aqua-hda-secondary.0,audiodev=aqua-route-secondary' \
    'aqua-audio-probe routes' \
    'stage=route-probe status=ok outputs=2' \
    'stage=qemu-route-switch status=ok outputs=2 default_changed=true' \
    'stage=qemu-multi-route status=ok controllers=2 codecs=2 backends=2'
do
    grep -Fq -- "${contract}" "${RUNNER}" "${EXPECT_SCRIPT}"
done

echo '[AQUA-AUDIO] stage=qemu-device-contract status=ok device=intel-hda codecs=hda-duplex,hda-output output_backends=wav,multi-wav input_backends=none,dbus controlled_inputs=zero-pcm,bipolar-signal,input-disconnect active_service_loss=true restart_exhaustion=true multi_route=true selected_device_unplug=true fallback=true host_microphone=false default_image_audio=false'
