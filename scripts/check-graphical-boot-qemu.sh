#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE_DIR="${ROOT_DIR}/build/buildroot-output/images"
KERNEL="${KERNEL:-${IMAGE_DIR}/bzImage}"
ROOTFS="${ROOTFS:-${IMAGE_DIR}/rootfs.ext2}"
SERIAL_LOG="${SERIAL_LOG:-${ROOT_DIR}/build/qemu-graphical-boot-check.log}"
MEMORY="${MEMORY:-1024M}"
CPUS="${CPUS:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-120}"
MONITOR_SOCKET="${MONITOR_SOCKET:-${ROOT_DIR}/build/qemu-graphical-boot-monitor.sock}"
QMP_SOCKET="${QMP_SOCKET:-${ROOT_DIR}/build/qemu-graphical-boot-qmp.sock}"
VNC_SOCKET="${VNC_SOCKET:-${ROOT_DIR}/build/qemu-graphical-boot-vnc.sock}"
INPUT_HELPER="${ROOT_DIR}/scripts/send-qemu-monitor-input.py"
INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET:-${ROOT_DIR}/build/qemu-graphical-boot-input.sock}"
INPUT_DAEMON_LOG="${INPUT_DAEMON_LOG:-${ROOT_DIR}/build/qemu-graphical-boot-input.log}"
INPUT_DAEMON_PID=""
CAPTURE_HELPER="${ROOT_DIR}/scripts/capture-qemu-monitor-screendump.py"
SESSION_MENU_SCREENSHOT="${SESSION_MENU_SCREENSHOT:-${ROOT_DIR}/build/qemu-session-menu.ppm}"
SESSION_MENU_SCREENSHOT_PNG="${SESSION_MENU_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-session-menu.png}"
CLEAN_DESKTOP_SCREENSHOT="${CLEAN_DESKTOP_SCREENSHOT:-${ROOT_DIR}/build/qemu-clean-desktop.ppm}"
CLEAN_DESKTOP_SCREENSHOT_PNG="${CLEAN_DESKTOP_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-clean-desktop.png}"
TERMINAL_SCREENSHOT="${TERMINAL_SCREENSHOT:-${ROOT_DIR}/build/qemu-aqua-terminal.ppm}"
TERMINAL_SCREENSHOT_PNG="${TERMINAL_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-aqua-terminal.png}"
THEME_LIGHT_SCREENSHOT="${THEME_LIGHT_SCREENSHOT:-${ROOT_DIR}/build/qemu-theme-light.ppm}"
THEME_LIGHT_SCREENSHOT_PNG="${THEME_LIGHT_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-theme-light.png}"
THEME_DEEP_SCREENSHOT="${THEME_DEEP_SCREENSHOT:-${ROOT_DIR}/build/qemu-theme-dark.ppm}"
THEME_DEEP_SCREENSHOT_PNG="${THEME_DEEP_SCREENSHOT_PNG:-${ROOT_DIR}/build/qemu-theme-dark.png}"
THEME_FRAME_CHECK="${ROOT_DIR}/scripts/check-qemu-theme-frame-delta.py"

cleanup() {
    if [ -n "${INPUT_DAEMON_PID}" ]; then
        kill "${INPUT_DAEMON_PID}" 2>/dev/null || true
        wait "${INPUT_DAEMON_PID}" 2>/dev/null || true
    fi
    rm -f "${MONITOR_SOCKET}" "${QMP_SOCKET}" "${VNC_SOCKET}" "${INPUT_CONTROL_SOCKET}"
}
trap cleanup EXIT INT TERM

for tool in expect python3 qemu-system-x86_64; do
    command -v "${tool}" >/dev/null 2>&1 || {
        echo "Missing required tool: ${tool}" >&2
        exit 1
    }
done

for artifact in "${KERNEL}" "${ROOTFS}"; do
    test -f "${artifact}" || {
        echo "Missing QEMU artifact: ${artifact}" >&2
        exit 1
    }
done

mkdir -p "$(dirname "${SERIAL_LOG}")"
rm -f "${SERIAL_LOG}" "${MONITOR_SOCKET}" "${QMP_SOCKET}" "${VNC_SOCKET}" "${INPUT_CONTROL_SOCKET}" "${INPUT_DAEMON_LOG}" \
    "${SESSION_MENU_SCREENSHOT}" "${SESSION_MENU_SCREENSHOT_PNG}" \
    "${CLEAN_DESKTOP_SCREENSHOT}" "${CLEAN_DESKTOP_SCREENSHOT_PNG}"
rm -f "${TERMINAL_SCREENSHOT}" "${TERMINAL_SCREENSHOT_PNG}"
rm -f "${THEME_LIGHT_SCREENSHOT}" "${THEME_LIGHT_SCREENSHOT_PNG}" \
    "${THEME_DEEP_SCREENSHOT}" "${THEME_DEEP_SCREENSHOT_PNG}"

AQUA_QEMU_QMP_SOCKET="${QMP_SOCKET}" python3 "${INPUT_HELPER}" --serve "${MONITOR_SOCKET}" "${INPUT_CONTROL_SOCKET}" >"${INPUT_DAEMON_LOG}" 2>&1 &
INPUT_DAEMON_PID=$!
i=0
while [ "${i}" -lt 100 ] && [ ! -S "${INPUT_CONTROL_SOCKET}" ]; do
    kill -0 "${INPUT_DAEMON_PID}" 2>/dev/null || {
        cat "${INPUT_DAEMON_LOG}" >&2
        exit 1
    }
    sleep 0.1
    i=$((i + 1))
done
test -S "${INPUT_CONTROL_SOCKET}" || {
    echo "QEMU input daemon control socket was not created" >&2
    exit 1
}

export ROOT_DIR KERNEL ROOTFS SERIAL_LOG MEMORY CPUS TIMEOUT_SECONDS MONITOR_SOCKET QMP_SOCKET VNC_SOCKET INPUT_HELPER
export CAPTURE_HELPER SESSION_MENU_SCREENSHOT SESSION_MENU_SCREENSHOT_PNG
export CLEAN_DESKTOP_SCREENSHOT CLEAN_DESKTOP_SCREENSHOT_PNG
export TERMINAL_SCREENSHOT TERMINAL_SCREENSHOT_PNG
export THEME_LIGHT_SCREENSHOT THEME_LIGHT_SCREENSHOT_PNG
export THEME_DEEP_SCREENSHOT THEME_DEEP_SCREENSHOT_PNG THEME_FRAME_CHECK
export AQUA_QEMU_INPUT_CONTROL_SOCKET="${INPUT_CONTROL_SOCKET}"
expect "${ROOT_DIR}/scripts/check-graphical-boot-qemu.exp"

grep -Fq '[AQUA-BOOT] stage=graphical-session-activation status=started mode=supervised boot_graphics=true recovery_tty=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=unprivileged-session-qemu status=ok user=aqua uid=1000 gid=1000 groups=video,audio,input runtime=/run/user/1000 mode=0700 compositor_uid=1000' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=media-supervisor-qemu status=ok user=aqua uid=1000 state=disabled packages=absent root_daemon=false graphics_continues=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=graphical-boot-qemu status=ok activation=supervised drm_wayland=active persistent=true scenario=desktop-event-loop fixtures=false recovery_tty=available' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-system-overview-qemu status=ok data=clock,kernel,uptime,load,memory gpu_texture=true visible=false' "${SERIAL_LOG}"
test -s "${CLEAN_DESKTOP_SCREENSHOT_PNG}"
grep -Fq '[AQUA-TEST] stage=desktop-icons-qemu status=ok selection=files gpu_texture=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-trash-qemu status=ok confirmation=true removed=2 remaining=0 root_confined=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-runtime-launch-qemu status=ok app=files surface=aqua.files repaint=true supervised=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-files-keyboard-focus-qemu status=ok input=left focus=sidebar-0 clients=1' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-files-keyboard-blur-qemu status=ok input=surface-transfer focus=none focused_sidebar=none repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-files-pointer-leave-qemu status=ok hover=cleared drag_cancelled=false repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-notification-qemu status=ok source=launcher toast=visible gpu_texture=true active_id=1' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-notification-promotion-qemu status=ok dismissed=1 promoted=2 queued=0 pointer=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-qemu status=ok target=files surface=aqua.properties kind=Folder location=/home/aqua clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-primary-button-qemu status=ok input=non-primary shared_button=true ignored=true focus=none activation=false generation=0 repaint=hover-only clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-pointer-cancel-qemu status=ok shared_button=true pressed=true focus=primary-action dragged_out=true release_activation=false generation=0 repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-refresh-qemu status=ok input=pointer shared_button=true hover=true pressed=true focus=primary-action release_activation=true action=refresh-contents generation=1 repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-pointer-focus-qemu status=ok input=pointer-space shared_button=true focus=primary-action handoff=true action=refresh-contents generation=2 repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-pointer-blur-qemu status=ok input=background-click-space shared_button=true focus=none activation=false generation=2 repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-pointer-leave-qemu status=ok hover=cleared press_cancelled=false repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-keyboard-blur-qemu status=ok input=surface-transfer shared_button=true focus=none activation=false generation=2 repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-keyboard-qemu status=ok input=tab-enter shared_button=true focus=primary-action action=refresh-contents generation=3 repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-properties-close-qemu status=ok close=alt-f4 exit=clean stale_surface=removed restart=never clients=1' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-runtime-settings-qemu status=ok app=settings surface=aqua.settings clients=2 launcher_closed=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=audio-adapter-qemu status=ok health=unavailable controls=false backend_applied=false packages=absent' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-live-theme-qemu status=ok from=Light to=Dark shell=true apps=files,settings restart=false frame_delta=true' "${SERIAL_LOG}"
test -s "${THEME_LIGHT_SCREENSHOT_PNG}"
test -s "${THEME_DEEP_SCREENSHOT_PNG}"
grep -Fq '[AQUA-TEST] stage=desktop-runtime-settings-focus-qemu status=ok app=settings input=home category=0' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-runtime-damage-qemu status=ok app=settings interaction=keyboard-category-selected repaint=incremented revision=changed' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-settings-primary-button-qemu status=ok input=non-primary ignored=true focus=keyboard category=4 activation=false repaint=pointer-motion-only clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-settings-pointer-blur-qemu status=ok input=background-click focus=none category=4 activation=false repaint=true clients=2' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-settings-keyboard-blur-qemu status=ok input=surface-transfer focus=none category=4 repaint=true refocus=true clients=2' "${SERIAL_LOG}"
grep -Eq '\[AQUA-TEST\] stage=desktop-input-burst-qemu status=ok keyboard_events=[2-9][0-9]+ pointer_commands=17 pointer_motion_events=[1-9][0-9]* pointer_coalescing=allowed category=4 monitor_connection=persistent' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-settings-about-qemu status=ok category=5 metadata_rows=3 read_only=true prototype=true repaint=true' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-runtime-close-qemu status=ok app=settings close=alt-f4 exit=clean stale_surface=removed restart=never clients=1' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-runtime-unexpected-exit-qemu status=ok app=files exit=forced stale_surface=removed restart=never active_count=0 clients=0' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-terminal-qemu status=ok app=terminal surface=aqua.terminal pty=true emulator=vt100 command=true resize=protocol-ready' "${SERIAL_LOG}"
test -s "${TERMINAL_SCREENSHOT_PNG}"
grep -Fq '[AQUA-TEST] stage=desktop-runtime-cleanup-qemu status=ok apps=files,settings lifecycle_clean=true active_count=0 stale_surfaces=0' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=desktop-session-menu-qemu status=ok actions=logout,restart,shutdown,recovery confirmation=true selected=recovery execution=return-to-recovery' "${SERIAL_LOG}"
grep -Fq 'desktop_session_menu_overlay_texture_ready=true' "${SERIAL_LOG}"
test -s "${SESSION_MENU_SCREENSHOT_PNG}"
grep -Fq '[AQUA-TEST] stage=graphical-stop-qemu status=ok clients_stopped=true kms_restored=true gbm_released=true pid_cleaned=true recovery_return=ok' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=graphical-restart-qemu status=ok second_session=active stale_pid=false stale_drm=false stale_socket=false' "${SERIAL_LOG}"
grep -Fq '[AQUA-TEST] stage=graphical-session-cycle-qemu status=ok starts=2 stops=2 sockets_clean=true pids_clean=true clients_clean=true drm_clean=true recovery_return=ok' "${SERIAL_LOG}"

echo "Aqua Linux opt-in graphical boot QEMU check passed."
echo "Serial log: ${SERIAL_LOG}"
echo "Clean desktop screenshot: ${CLEAN_DESKTOP_SCREENSHOT_PNG}"
echo "Session menu screenshot: ${SESSION_MENU_SCREENSHOT_PNG}"
echo "Terminal screenshot: ${TERMINAL_SCREENSHOT_PNG}"
echo "Light theme screenshot: ${THEME_LIGHT_SCREENSHOT_PNG}"
echo "Dark theme screenshot: ${THEME_DEEP_SCREENSHOT_PNG}"
