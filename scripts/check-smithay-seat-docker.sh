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
        selection_output="$(cargo run --quiet -p aqua-compositor --features smithay-smoke -- probe-selection-ownership)"
        printf "%s\n" "$selection_output"
        printf "%s\n" "$selection_output" | grep -Fq "client_count=2"
        printf "%s\n" "$selection_output" | grep -Fq "globals_visible_to_both_clients=true"
        printf "%s\n" "$selection_output" | grep -Fq "unfocused_clipboard_rejected=true"
        printf "%s\n" "$selection_output" | grep -Fq "unfocused_primary_rejected=true"
        printf "%s\n" "$selection_output" | grep -Fq "focused_clipboard_accepted=true"
        printf "%s\n" "$selection_output" | grep -Fq "focused_primary_accepted=true"
        printf "%s\n" "$selection_output" | grep -Fq "clipboard_offer_reaches_new_focus=true"
        printf "%s\n" "$selection_output" | grep -Fq "primary_offer_reaches_new_focus=true"
        printf "%s\n" "$selection_output" | grep -Fq "clipboard_mime_negotiated=true"
        printf "%s\n" "$selection_output" | grep -Fq "primary_mime_negotiated=true"
        printf "%s\n" "$selection_output" | grep -Fq "unsupported_mime_not_requested=true"
        printf "%s\n" "$selection_output" | grep -Fq "clipboard_payload_transferred=true"
        printf "%s\n" "$selection_output" | grep -Fq "primary_payload_transferred=true"
        printf "%s\n" "$selection_output" | grep -Fq "clipboard_payload_bytes=24"
        printf "%s\n" "$selection_output" | grep -Fq "primary_payload_bytes=32"
        printf "%s\n" "$selection_output" | grep -Fq "transfer_limit_bytes=4096"
        printf "%s\n" "$selection_output" | grep -Fq "compositor_buffers_payload=false"
        printf "%s\n" "$selection_output" | grep -Fq "owner_disconnect_clears_clipboard=true"
        printf "%s\n" "$selection_output" | grep -Fq "owner_disconnect_clears_primary=true"
        printf "%s\n" "$selection_output" | grep -Fq "ownership_handoff_accepted=true"
        printf "%s\n" "$selection_output" | grep -Fq "data_control_global_exposed=false"
        printf "%s\n" "$selection_output" | grep -Fq "host_stub=false"
        printf "%s\n" "$selection_output" | grep -Fq "[AQUA-COMPOSITOR] stage=selection-ownership status=ok"
        dnd_output="$(cargo run --quiet -p aqua-compositor --features smithay-smoke -- probe-drag-and-drop)" || {
            printf "%s\n" "$dnd_output"
            exit 1
        }
        printf "%s\n" "$dnd_output"
        printf "%s\n" "$dnd_output" | grep -Fq "client_count=2"
        printf "%s\n" "$dnd_output" | grep -Fq "start_without_implicit_grab_rejected=true"
        printf "%s\n" "$dnd_output" | grep -Fq "pointer_grab_started=true"
        printf "%s\n" "$dnd_output" | grep -Fq "source_client_owns_drag=true"
        printf "%s\n" "$dnd_output" | grep -Fq "enter_reaches_pointer_focus_only=true"
        printf "%s\n" "$dnd_output" | grep -Fq "keyboard_focus_unchanged=true"
        printf "%s\n" "$dnd_output" | grep -Fq "mime_negotiated=true"
        printf "%s\n" "$dnd_output" | grep -Fq "unsupported_mime_not_accepted=true"
        printf "%s\n" "$dnd_output" | grep -Fq "copy_action_negotiated=true"
        printf "%s\n" "$dnd_output" | grep -Fq "payload_transferred=true"
        printf "%s\n" "$dnd_output" | grep -Fq "payload_bytes=28"
        printf "%s\n" "$dnd_output" | grep -Fq "transfer_limit_bytes=4096"
        printf "%s\n" "$dnd_output" | grep -Fq "compositor_buffers_payload=false"
        printf "%s\n" "$dnd_output" | grep -Fq "drop_delivered_to_target=true"
        printf "%s\n" "$dnd_output" | grep -Fq "source_drop_performed=true"
        printf "%s\n" "$dnd_output" | grep -Fq "source_finished=true"
        printf "%s\n" "$dnd_output" | grep -Fq "rejected_drop_cancelled=true"
        printf "%s\n" "$dnd_output" | grep -Fq "rejected_drop_not_delivered=true"
        printf "%s\n" "$dnd_output" | grep -Fq "data_control_global_exposed=false"
        printf "%s\n" "$dnd_output" | grep -Fq "host_stub=false"
        printf "%s\n" "$dnd_output" | grep -Fq "[AQUA-COMPOSITOR] stage=drag-and-drop status=ok"
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_launcher_keyboard_is_compositor_owned
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_notification_close_promotes_queue_and_timeout_hides_toast
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_first_party_surfaces_raise_and_move_between_workspaces
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            first_party_runtime_theme_transition_is_idempotent
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_selection_ownership_is_keyboard_focus_bound
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_drag_and_drop_is_focus_safe_and_bounded
    '
