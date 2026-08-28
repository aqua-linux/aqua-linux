#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
IMAGE="${AQUA_SMITHAY_CHECK_IMAGE:-rust:1.85-bookworm}"

docker run --rm --platform linux/amd64 \
    -v "${ROOT_DIR}:/work" \
    -w /work \
    "${IMAGE}" \
    sh -eu -c '
        apt-get update >/dev/null
        apt-get install -y --no-install-recommends \
            pkg-config libinput-dev libudev-dev libxkbcommon-dev >/dev/null
        output="$(cargo run --quiet -p aqua-compositor --features smithay-smoke -- probe-smithay-launcher-seat)"
        printf "%s\n" "$output"
        printf "%s\n" "$output" | grep -Fq "seat_global_created=true"
        printf "%s\n" "$output" | grep -Fq "keyboard_capability=true"
        printf "%s\n" "$output" | grep -Fq "pointer_capability=true"
        printf "%s\n" "$output" | grep -Fq "keyboard_event_intercepted=true"
        printf "%s\n" "$output" | grep -Fq "pointer_motion_dispatched=true"
        printf "%s\n" "$output" | grep -Fq "pointer_button_dispatched=true"
        printf "%s\n" "$output" | grep -Fq "launcher_visible=true"
        printf "%s\n" "$output" | grep -Fq "selected_category=settings"
        printf "%s\n" "$output" | grep -Fq "host_stub=false"
        printf "%s\n" "$output" | grep -Fq "[AQUA-COMPOSITOR] stage=smithay-launcher-seat status=ok"
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_launcher_keyboard_is_compositor_owned
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_notification_close_promotes_queue_and_timeout_hides_toast
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_first_party_surfaces_raise_and_move_between_workspaces
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            first_party_runtime_theme_transition_is_idempotent
    '
