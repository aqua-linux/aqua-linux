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
        text_input_output="$(cargo run --quiet -p aqua-compositor --features smithay-smoke -- probe-text-input)" || {
            printf "%s\n" "$text_input_output"
            exit 1
        }
        printf "%s\n" "$text_input_output"
        printf "%s\n" "$text_input_output" | grep -Fq "client_count=3"
        printf "%s\n" "$text_input_output" | grep -Fq "text_input_visible_to_normal_clients=true"
        printf "%s\n" "$text_input_output" | grep -Fq "input_method_hidden_from_normal_clients=true"
        printf "%s\n" "$text_input_output" | grep -Fq "input_method_visible_to_authorized_client=true"
        printf "%s\n" "$text_input_output" | grep -Fq "focus_follows_keyboard=true"
        printf "%s\n" "$text_input_output" | grep -Fq "unfocused_enable_rejected=true"
        printf "%s\n" "$text_input_output" | grep -Fq "focused_enable_activates_input_method=true"
        printf "%s\n" "$text_input_output" | grep -Fq "surrounding_text_forwarded=true"
        printf "%s\n" "$text_input_output" | grep -Fq "content_type_forwarded=true"
        printf "%s\n" "$text_input_output" | grep -Fq "cursor_rectangle_forwarded=true"
        printf "%s\n" "$text_input_output" | grep -Fq "turkish_preedit_delivered=true"
        printf "%s\n" "$text_input_output" | grep -Fq "turkish_commit_delivered=true"
        printf "%s\n" "$text_input_output" | grep -Fq "delete_surrounding_delivered=true"
        printf "%s\n" "$text_input_output" | grep -Fq "serial_synchronized=true"
        printf "%s\n" "$text_input_output" | grep -Fq "focus_handoff_deactivates_input_method=true"
        printf "%s\n" "$text_input_output" | grep -Fq "focus_handoff_enters_new_client=true"
        printf "%s\n" "$text_input_output" | grep -Fq "stale_unfocused_client_blocked=true"
        printf "%s\n" "$text_input_output" | grep -Fq "popup_parent_bound=true"
        printf "%s\n" "$text_input_output" | grep -Fq "popup_repositioned=true"
        printf "%s\n" "$text_input_output" | grep -Fq "payload_limit_bytes=4000"
        printf "%s\n" "$text_input_output" | grep -Fq "host_stub=false"
        printf "%s\n" "$text_input_output" | grep -Fq "[AQUA-COMPOSITOR] stage=text-input status=ok"
        keyboard_matrix_output="$(cargo run --quiet -p aqua-compositor --features smithay-smoke -- probe-keyboard-locale-matrix)" || {
            printf "%s\n" "$keyboard_matrix_output"
            exit 1
        }
        printf "%s\n" "$keyboard_matrix_output"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "locale_count=3"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "keyboard_layout_count=3"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "supported_combination_count=9"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "client_count_per_layout=2"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "keymaps_delivered_to_all_clients=true"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "keymaps_compile_for_all_layouts=true"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "representative_utf8_matches=true"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "repeat_info_matches=true"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "host_stub=false"
        printf "%s\n" "$keyboard_matrix_output" | grep -Fq "[AQUA-COMPOSITOR] stage=keyboard-locale-matrix status=ok"
        buffer_contract_output="$(cargo run --quiet -p aqua-compositor --features smithay-smoke -- probe-v1-client-buffer-contract)" || {
            printf "%s\n" "$buffer_contract_output"
            exit 1
        }
        printf "%s\n" "$buffer_contract_output"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "application_model=first-party-wl-shm-v1"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "required_buffer_protocol=wl_shm"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "required_shm_format=argb8888"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "client_count=2"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "wl_shm_visible_to_all_clients=true"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "argb8888_visible_to_all_clients=true"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "linux_dmabuf_advertised=false"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "drm_syncobj_advertised=false"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "explicit_sync_advertised=false"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "accelerated_clients_supported=false"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "synchronization_scope=wl_buffer.release+wl_surface.frame"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "host_stub=false"
        printf "%s\n" "$buffer_contract_output" | grep -Fq "[AQUA-COMPOSITOR] stage=v1-client-buffer-contract status=ok"
        output_matrix="$(cargo run --quiet -p aqua-compositor --features smithay-smoke -- probe-wayland-output-matrix)" || {
            printf "%s\n" "$output_matrix"
            exit 1
        }
        printf "%s\n" "$output_matrix"
        printf "%s\n" "$output_matrix" | grep -Fq "client_count=2"
        printf "%s\n" "$output_matrix" | grep -Fq "outputs_visible_to_both_clients=true"
        printf "%s\n" "$output_matrix" | grep -Fq "modes_match_supported_matrix=true"
        printf "%s\n" "$output_matrix" | grep -Fq "preferred_modes_advertised=true"
        printf "%s\n" "$output_matrix" | grep -Fq "logical_coordinates_match=true"
        printf "%s\n" "$output_matrix" | grep -Fq "integer_scales_match=true"
        printf "%s\n" "$output_matrix" | grep -Fq "fractional_scale_120ths=150"
        printf "%s\n" "$output_matrix" | grep -Fq "viewport_source_applied=true"
        printf "%s\n" "$output_matrix" | grep -Fq "viewport_destination_applied=true"
        printf "%s\n" "$output_matrix" | grep -Fq "hotplug_remove_reaches_both_clients=true"
        printf "%s\n" "$output_matrix" | grep -Fq "remaining_output_usable=true"
        printf "%s\n" "$output_matrix" | grep -Fq "host_stub=false"
        printf "%s\n" "$output_matrix" | grep -Fq "[AQUA-COMPOSITOR] stage=wayland-output-matrix status=ok"
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
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_text_input_is_focus_and_authorization_safe
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_keyboard_locale_matrix_delivers_compilable_keymaps
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            v1_client_buffer_contract_excludes_accelerated_clients
        cargo test --quiet -p aqua-compositor --features smithay-smoke --lib \
            smithay_output_matrix_is_discoverable_scaled_and_hotpluggable
    '
