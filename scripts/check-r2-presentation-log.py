#!/usr/bin/env python3
"""Validate four bounded R2 presentation records from a QEMU serial log."""

from __future__ import annotations

import pathlib
import sys


WORKLOADS = {"idle", "window-interaction", "animation", "multi-client"}
MIN_REVIEW_RUNS = 3
MAX_REVIEW_RUNS = 10
QEMU_BUDGET_PROFILE = "qemu-tcg-bochs-v1"
QEMU_BUDGET = {
    "max_frame_time_us": 50_000,
    "max_input_to_present_us": 60_000_000,
    "max_cpu_time_us": 180_000_000,
    "max_memory_growth_kib": 163_840,
}
QEMU_SOAK_BUDGET_PROFILE = "qemu-tcg-bochs-soak-v1"
QEMU_SOAK_MIN_OBSERVATION_WINDOW_MS = 300_000
QEMU_SOAK_MIN_INPUT_SAMPLES = 5
QEMU_SOAK_BUDGET = {
    "max_frame_time_us": 50_000,
    "max_input_to_present_us": 60_000_000,
    "max_cpu_time_us": 720_000_000,
    "max_memory_growth_kib": 163_840,
}
QEMU_BUDGET_FIELDS = {
    "max_frame_time_us": "max_frame_time_us",
    "max_input_to_present_us": "max_input_to_present_us",
    "cpu_time_us": "max_cpu_time_us",
    "memory_growth_kib": "max_memory_growth_kib",
}
INTEGER_FIELDS = {
    "frames_requested",
    "frames_presented",
    "page_flip_events",
    "frame_callbacks_sent",
    "damage_commits",
    "full_frame_readbacks",
    "settled_idle_observations",
    "settled_idle_repaints",
    "input_to_present_samples",
    "max_input_to_present_us",
    "cpu_framebuffer_copies",
    "max_frame_time_us",
    "observation_window_ms",
    "cpu_time_us",
    "memory_growth_kib",
}
BOOLEAN_FIELDS = {
    "live_events",
    "repeating_repaint_timer_after_settle",
    "acceptance_complete",
}
TEXT_FIELDS = {"target", "path", "workload"}
REQUIRED_FIELDS = INTEGER_FIELDS | BOOLEAN_FIELDS | TEXT_FIELDS
DIAGNOSTIC_INTEGER_FIELDS = {
    "captured_frames",
    "full_frame_readbacks",
    "production_frames_read_back",
    "production_frames_blocked",
}
DIAGNOSTIC_BOOLEAN_FIELDS = {
    "kms_activated",
    "display_output_started",
    "acceptance_complete",
}
DIAGNOSTIC_TEXT_FIELDS = {"target", "path"}
DIAGNOSTIC_REQUIRED_FIELDS = (
    DIAGNOSTIC_INTEGER_FIELDS | DIAGNOSTIC_BOOLEAN_FIELDS | DIAGNOSTIC_TEXT_FIELDS
)


def parse_records(text: str) -> list[dict[str, str]]:
    records: list[dict[str, str]] = []
    current: dict[str, str] | None = None
    for raw_line in text.replace("\r", "").splitlines():
        line = raw_line.strip()
        if line == "r2_presentation_record_begin=v1":
            if current is not None:
                raise ValueError("nested R2 presentation record")
            current = {}
            continue
        if line == "r2_presentation_record_end=v1":
            if current is None:
                raise ValueError("R2 presentation record ended without a start")
            records.append(current)
            current = None
            continue
        if not line.startswith("r2_presentation_"):
            continue
        if current is None:
            raise ValueError("R2 presentation field appeared outside a record")
        key, separator, value = line.removeprefix("r2_presentation_").partition("=")
        if not separator or not key or not value:
            raise ValueError("malformed R2 presentation field")
        if key in current:
            raise ValueError(f"duplicate R2 presentation field: {key}")
        current[key] = value
    if current is not None:
        raise ValueError("unterminated R2 presentation record")
    return records


def parse_diagnostic_records(text: str) -> list[dict[str, str]]:
    records: list[dict[str, str]] = []
    current: dict[str, str] | None = None
    for raw_line in text.replace("\r", "").splitlines():
        line = raw_line.strip()
        if line == "r2_diagnostic_record_begin=v1":
            if current is not None:
                raise ValueError("nested R2 diagnostic record")
            current = {}
            continue
        if line == "r2_diagnostic_record_end=v1":
            if current is None:
                raise ValueError("R2 diagnostic record ended without a start")
            records.append(current)
            current = None
            continue
        if not line.startswith("r2_diagnostic_"):
            continue
        if current is None:
            raise ValueError("R2 diagnostic field appeared outside a record")
        key, separator, value = line.removeprefix("r2_diagnostic_").partition("=")
        if not separator or not key or not value:
            raise ValueError("malformed R2 diagnostic field")
        if key in current:
            raise ValueError(f"duplicate R2 diagnostic field: {key}")
        current[key] = value
    if current is not None:
        raise ValueError("unterminated R2 diagnostic record")
    return records


def parse_boolean(record: dict[str, str], field: str) -> bool:
    value = record[field]
    if value not in {"true", "false"}:
        raise ValueError(f"invalid Boolean field {field}: {value}")
    return value == "true"


def parse_integer(record: dict[str, str], field: str) -> int:
    value = record[field]
    if not value.isascii() or not value.isdecimal():
        raise ValueError(f"invalid integer field {field}: {value}")
    parsed = int(value)
    if parsed > (1 << 64) - 1:
        raise ValueError(f"unbounded integer field {field}: {value}")
    return parsed


def validate_record(
    record: dict[str, str],
    budget_profile: str = QEMU_BUDGET_PROFILE,
    budget: dict[str, int] = QEMU_BUDGET,
) -> dict[str, int | str | bool]:
    missing = REQUIRED_FIELDS - record.keys()
    extra = record.keys() - REQUIRED_FIELDS
    if missing:
        raise ValueError(f"missing R2 presentation fields: {','.join(sorted(missing))}")
    if extra:
        raise ValueError(f"unknown R2 presentation fields: {','.join(sorted(extra))}")
    parsed: dict[str, int | str | bool] = {
        field: parse_integer(record, field) for field in INTEGER_FIELDS
    }
    parsed.update({field: parse_boolean(record, field) for field in BOOLEAN_FIELDS})
    parsed.update({field: record[field] for field in TEXT_FIELDS})

    workload = parsed["workload"]
    if workload not in WORKLOADS:
        raise ValueError(f"unknown R2 presentation workload: {workload}")
    if parsed["target"] != "qemu-tcg" or parsed["path"] != "production-gbm-kms":
        raise ValueError("R2 record is not a QEMU production GBM/KMS sample")
    if parsed["live_events"] is not True or parsed["acceptance_complete"] is not False:
        raise ValueError("R2 record has invalid live or partial-acceptance markers")
    requested = int(parsed["frames_requested"])
    presented = int(parsed["frames_presented"])
    if requested == 0 or requested != presented or parsed["page_flip_events"] != presented:
        raise ValueError(f"incomplete R2 frame accounting for {workload}")
    if workload == "multi-client" and int(parsed["frame_callbacks_sent"]) == 0:
        raise ValueError("missing frame callback evidence for multi-client")
    if int(parsed["max_frame_time_us"]) == 0 or int(parsed["observation_window_ms"]) == 0:
        raise ValueError(f"missing timing evidence for {workload}")
    if int(parsed["cpu_framebuffer_copies"]) != 0:
        raise ValueError(f"CPU framebuffer copy recorded for {workload}")
    if int(parsed["full_frame_readbacks"]) != 0:
        raise ValueError(f"full-frame GPU readback recorded for {workload}")
    for field, budget_field in QEMU_BUDGET_FIELDS.items():
        limit = budget[budget_field]
        if int(parsed[field]) > limit:
            raise ValueError(
                f"{budget_profile} budget exceeded for {workload} {field}: "
                f"{parsed[field]} > {limit}"
            )

    if workload == "idle":
        if requested != 1 or int(parsed["damage_commits"]) > 1:
            raise ValueError("idle workload rendered or damaged more than its initial frame")
        if int(parsed["settled_idle_observations"]) == 0:
            raise ValueError("idle workload has no settled observation")
        if int(parsed["settled_idle_repaints"]) != 0:
            raise ValueError("idle workload repainted after settling")
        if parsed["repeating_repaint_timer_after_settle"] is not False:
            raise ValueError("idle workload retained a repeating repaint timer")
    else:
        if requested < 2:
            raise ValueError(f"active workload did not repaint for {workload}")
        if workload == "multi-client" and int(parsed["damage_commits"]) == 0:
            raise ValueError("missing damage evidence for multi-client")
        if int(parsed["input_to_present_samples"]) == 0:
            raise ValueError(f"missing input sample for {workload}")
        if int(parsed["max_input_to_present_us"]) == 0:
            raise ValueError(f"missing input latency for {workload}")
    return parsed


def validate_log(text: str) -> list[dict[str, int | str | bool]]:
    records = [validate_record(record) for record in parse_records(text)]
    workloads = [str(record["workload"]) for record in records]
    if len(records) != 4 or set(workloads) != WORKLOADS or len(set(workloads)) != 4:
        raise ValueError("R2 log must contain exactly one record for every required workload")
    return records


def validate_diagnostic_log(text: str) -> dict[str, int | str | bool]:
    records = parse_diagnostic_records(text)
    if len(records) != 1:
        raise ValueError("R2 log must contain exactly one diagnostic readback record")
    record = records[0]
    missing = DIAGNOSTIC_REQUIRED_FIELDS - record.keys()
    extra = record.keys() - DIAGNOSTIC_REQUIRED_FIELDS
    if missing:
        raise ValueError(f"missing R2 diagnostic fields: {','.join(sorted(missing))}")
    if extra:
        raise ValueError(f"unknown R2 diagnostic fields: {','.join(sorted(extra))}")
    parsed: dict[str, int | str | bool] = {
        field: parse_integer(record, field) for field in DIAGNOSTIC_INTEGER_FIELDS
    }
    parsed.update(
        {field: parse_boolean(record, field) for field in DIAGNOSTIC_BOOLEAN_FIELDS}
    )
    parsed.update({field: record[field] for field in DIAGNOSTIC_TEXT_FIELDS})

    if parsed["target"] != "qemu-tcg" or parsed["path"] != "diagnostic-readback":
        raise ValueError("R2 diagnostic record is not an isolated QEMU readback")
    captured = int(parsed["captured_frames"])
    readbacks = int(parsed["full_frame_readbacks"])
    if captured == 0 or readbacks == 0 or readbacks > captured:
        raise ValueError("R2 diagnostic record has invalid capture accounting")
    if (
        int(parsed["production_frames_read_back"]) != 0
        or int(parsed["production_frames_blocked"]) != 0
    ):
        raise ValueError("R2 diagnostic readback touched production presentation")
    if parsed["kms_activated"] is not False or parsed["display_output_started"] is not False:
        raise ValueError("R2 diagnostic readback activated a presentation output")
    if parsed["acceptance_complete"] is not False:
        raise ValueError("R2 diagnostic record made an unsupported acceptance claim")
    return parsed


def validate_evidence_log(
    text: str,
) -> tuple[list[dict[str, int | str | bool]], dict[str, int | str | bool]]:
    return validate_log(text), validate_diagnostic_log(text)


def validate_soak_log(
    text: str,
) -> tuple[dict[str, int | str | bool], dict[str, int | str | bool]]:
    records = parse_records(text)
    if len(records) != 1:
        raise ValueError("R2 soak log must contain exactly one presentation record")
    record = validate_record(
        records[0],
        budget_profile=QEMU_SOAK_BUDGET_PROFILE,
        budget=QEMU_SOAK_BUDGET,
    )
    if record["workload"] != "multi-client":
        raise ValueError("R2 soak must use the multi-client workload")
    if int(record["observation_window_ms"]) < QEMU_SOAK_MIN_OBSERVATION_WINDOW_MS:
        raise ValueError("R2 soak observation window is shorter than five minutes")
    if int(record["input_to_present_samples"]) < QEMU_SOAK_MIN_INPUT_SAMPLES:
        raise ValueError("R2 soak has too few periodic input-to-present samples")
    minimum_frames = QEMU_SOAK_MIN_INPUT_SAMPLES + 3
    if int(record["frames_presented"]) < minimum_frames:
        raise ValueError("R2 soak has too few presented frames")
    required_markers = (
        "desktop_launch_surface_app_id=aqua.files",
        "desktop_launch_surface_app_id=aqua.settings",
        "drm_wayland_input_dispatch_ready=true",
        "drm_wayland_graceful_stop_requested=true",
        "desktop_runtime_process_active_count=0",
        "[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok",
    )
    for marker in required_markers:
        if marker not in text:
            raise ValueError(f"R2 soak log is missing lifecycle marker: {marker}")
    if "[AQUA-COMPOSITOR] stage=drm-wayland-session status=error" in text:
        raise ValueError("R2 soak compositor reported an error")
    input_event_prefix = "drm_wayland_input_keyboard_events="
    input_event_values = [
        line.strip().removeprefix(input_event_prefix)
        for line in text.replace("\r", "").splitlines()
        if line.strip().startswith(input_event_prefix)
    ]
    if len(input_event_values) != 1:
        raise ValueError("R2 soak log must contain one keyboard-event total")
    if not input_event_values[0].isascii() or not input_event_values[0].isdecimal():
        raise ValueError("R2 soak keyboard-event total is malformed")
    if int(input_event_values[0]) < 40:
        raise ValueError("R2 soak did not dispatch enough periodic keyboard events")
    return record, validate_diagnostic_log(text)


def soak_report_lines(text: str) -> list[str]:
    record, _ = validate_soak_log(text)
    keyboard_events = next(
        int(line.strip().partition("=")[2])
        for line in text.replace("\r", "").splitlines()
        if line.strip().startswith("drm_wayland_input_keyboard_events=")
    )
    return [
        "r2_soak_record_begin=v1",
        "r2_soak_target=qemu-tcg",
        f"r2_soak_budget_profile={QEMU_SOAK_BUDGET_PROFILE}",
        f"r2_soak_min_observation_window_ms={QEMU_SOAK_MIN_OBSERVATION_WINDOW_MS}",
        f"r2_soak_observation_window_ms={record['observation_window_ms']}",
        f"r2_soak_min_input_to_present_samples={QEMU_SOAK_MIN_INPUT_SAMPLES}",
        f"r2_soak_input_to_present_samples={record['input_to_present_samples']}",
        f"r2_soak_keyboard_events={keyboard_events}",
        f"r2_soak_frames_presented={record['frames_presented']}",
        f"r2_soak_max_frame_time_us={record['max_frame_time_us']}",
        f"r2_soak_max_input_to_present_us={record['max_input_to_present_us']}",
        f"r2_soak_cpu_time_us={record['cpu_time_us']}",
        f"r2_soak_memory_growth_kib={record['memory_growth_kib']}",
        f"r2_soak_budget_max_frame_time_us={QEMU_SOAK_BUDGET['max_frame_time_us']}",
        f"r2_soak_budget_max_input_to_present_us={QEMU_SOAK_BUDGET['max_input_to_present_us']}",
        f"r2_soak_budget_max_cpu_time_us={QEMU_SOAK_BUDGET['max_cpu_time_us']}",
        f"r2_soak_budget_max_memory_growth_kib={QEMU_SOAK_BUDGET['max_memory_growth_kib']}",
        "r2_soak_budget_max_dropped_frames=0",
        "r2_soak_crash_budget=0",
        "r2_soak_crashes=0",
        "r2_soak_client_lifecycle_complete=true",
        "r2_soak_diagnostic_isolation_recorded=true",
        "r2_soak_physical_evidence=false",
        "r2_soak_record_end=v1",
    ]


def repeated_review_lines(logs: list[str]) -> list[str]:
    if not MIN_REVIEW_RUNS <= len(logs) <= MAX_REVIEW_RUNS:
        raise ValueError(
            f"R2 review requires {MIN_REVIEW_RUNS}-{MAX_REVIEW_RUNS} independent runs"
        )
    runs = [validate_evidence_log(log) for log in logs]
    records = [record for run_records, _ in runs for record in run_records]
    lines = [
        "r2_review_record_begin=v1",
        f"r2_review_qemu_runs={len(runs)}",
        f"r2_review_workload_records={len(records)}",
        f"r2_review_diagnostic_records={len(runs)}",
    ]
    for workload in sorted(WORKLOADS):
        workload_records = [
            record for record in records if record["workload"] == workload
        ]
        for field in (
            "max_frame_time_us",
            "max_input_to_present_us",
            "cpu_time_us",
            "memory_growth_kib",
        ):
            maximum = max(int(record[field]) for record in workload_records)
            lines.append(f"r2_review_{workload}_{field}={maximum}")
    lines.extend(
        (
            "r2_review_minimum_runs_met=true",
            "r2_review_diagnostic_isolation_recorded=true",
            f"r2_review_budget_profile={QEMU_BUDGET_PROFILE}",
            f"r2_review_budget_max_frame_time_us={QEMU_BUDGET['max_frame_time_us']}",
            f"r2_review_budget_max_input_to_present_us={QEMU_BUDGET['max_input_to_present_us']}",
            f"r2_review_budget_max_cpu_time_us={QEMU_BUDGET['max_cpu_time_us']}",
            f"r2_review_budget_max_memory_growth_kib={QEMU_BUDGET['max_memory_growth_kib']}",
            "r2_review_budget_max_dropped_frames=0",
            "r2_review_budget_selected=true",
            "r2_review_physical_budget_selected=false",
            "r2_review_record_end=v1",
        )
    )
    return lines


def fixture_record(workload: str) -> str:
    idle = workload == "idle"
    values = {
        "live_events": "true",
        "target": "qemu-tcg",
        "path": "production-gbm-kms",
        "workload": workload,
        "frames_requested": "1" if idle else "3",
        "frames_presented": "1" if idle else "3",
        "page_flip_events": "1" if idle else "3",
        "frame_callbacks_sent": "0" if idle else "1",
        "damage_commits": "0" if idle else "2",
        "full_frame_readbacks": "0",
        "settled_idle_observations": "5" if idle else "0",
        "settled_idle_repaints": "0",
        "repeating_repaint_timer_after_settle": "false",
        "input_to_present_samples": "0" if idle else "1",
        "max_input_to_present_us": "0" if idle else "25000",
        "cpu_framebuffer_copies": "0",
        "max_frame_time_us": "17000",
        "observation_window_ms": "1000",
        "cpu_time_us": "50000",
        "memory_growth_kib": "128",
        "acceptance_complete": "false",
    }
    fields = "\n".join(f"r2_presentation_{key}={value}" for key, value in values.items())
    return f"r2_presentation_record_begin=v1\n{fields}\nr2_presentation_record_end=v1"


def diagnostic_fixture() -> str:
    return "\n".join(
        (
            "r2_diagnostic_record_begin=v1",
            "r2_diagnostic_target=qemu-tcg",
            "r2_diagnostic_path=diagnostic-readback",
            "r2_diagnostic_captured_frames=2",
            "r2_diagnostic_full_frame_readbacks=2",
            "r2_diagnostic_production_frames_read_back=0",
            "r2_diagnostic_production_frames_blocked=0",
            "r2_diagnostic_kms_activated=false",
            "r2_diagnostic_display_output_started=false",
            "r2_diagnostic_acceptance_complete=false",
            "r2_diagnostic_record_end=v1",
        )
    )


def self_test() -> None:
    fixture = "\n".join(
        [*(fixture_record(workload) for workload in sorted(WORKLOADS)), diagnostic_fixture()]
    )
    records = validate_log(fixture)
    diagnostic = validate_diagnostic_log(fixture)
    assert len(records) == 4
    assert diagnostic["full_frame_readbacks"] == 2
    review = repeated_review_lines([fixture] * MIN_REVIEW_RUNS)
    assert f"r2_review_qemu_runs={MIN_REVIEW_RUNS}" in review
    assert f"r2_review_budget_profile={QEMU_BUDGET_PROFILE}" in review
    assert "r2_review_budget_selected=true" in review
    assert "r2_review_physical_budget_selected=false" in review
    try:
        validate_log(
            fixture.replace(
                "r2_presentation_path=production-gbm-kms",
                "r2_presentation_path=legacy-cpu-copy",
                1,
            )
        )
    except ValueError:
        pass
    else:
        raise AssertionError("legacy path fixture unexpectedly passed")
    try:
        validate_log(
            fixture.replace(
                "r2_presentation_full_frame_readbacks=0",
                "r2_presentation_full_frame_readbacks=1",
                1,
            )
        )
    except ValueError:
        pass
    else:
        raise AssertionError("production readback fixture unexpectedly passed")
    over_budget_cases = {
        "max_frame_time_us": (17_000, 50_001),
        "max_input_to_present_us": (25_000, 60_000_001),
        "cpu_time_us": (50_000, 180_000_001),
        "memory_growth_kib": (128, 163_841),
    }
    for field, (accepted, rejected) in over_budget_cases.items():
        try:
            validate_log(
                fixture.replace(
                    f"r2_presentation_{field}={accepted}",
                    f"r2_presentation_{field}={rejected}",
                    1,
                )
            )
        except ValueError:
            pass
        else:
            raise AssertionError(f"over-budget {field} fixture unexpectedly passed")
    try:
        validate_log(f"r2_presentation_workload=idle\n{fixture}")
    except ValueError:
        pass
    else:
        raise AssertionError("out-of-record field fixture unexpectedly passed")
    try:
        validate_diagnostic_log(
            fixture.replace("r2_diagnostic_production_frames_blocked=0", "r2_diagnostic_production_frames_blocked=1")
        )
    except ValueError:
        pass
    else:
        raise AssertionError("production-blocking diagnostic fixture unexpectedly passed")
    try:
        repeated_review_lines([fixture] * (MIN_REVIEW_RUNS - 1))
    except ValueError:
        pass
    else:
        raise AssertionError("undersized repeated-run review unexpectedly passed")

    soak_fixture = fixture_record("multi-client")
    soak_overrides = {
        "frames_requested": (3, 13),
        "frames_presented": (3, 13),
        "page_flip_events": (3, 13),
        "input_to_present_samples": (1, QEMU_SOAK_MIN_INPUT_SAMPLES),
        "observation_window_ms": (1_000, QEMU_SOAK_MIN_OBSERVATION_WINDOW_MS),
        "cpu_time_us": (50_000, 400_000_000),
    }
    for field, (original, replacement) in soak_overrides.items():
        soak_fixture = soak_fixture.replace(
            f"r2_presentation_{field}={original}",
            f"r2_presentation_{field}={replacement}",
        )
    soak_fixture = "\n".join(
        (
            soak_fixture,
            "desktop_launch_surface_app_id=aqua.files",
            "desktop_launch_surface_app_id=aqua.settings",
            "drm_wayland_input_dispatch_ready=true",
            "drm_wayland_input_keyboard_events=40",
            "drm_wayland_graceful_stop_requested=true",
            "desktop_runtime_process_active_count=0",
            "[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok",
            diagnostic_fixture(),
        )
    )
    soak_report = soak_report_lines(soak_fixture)
    assert f"r2_soak_budget_profile={QEMU_SOAK_BUDGET_PROFILE}" in soak_report
    assert "r2_soak_crashes=0" in soak_report
    assert "r2_soak_physical_evidence=false" in soak_report
    try:
        soak_report_lines(
            soak_fixture.replace(
                f"r2_presentation_observation_window_ms={QEMU_SOAK_MIN_OBSERVATION_WINDOW_MS}",
                f"r2_presentation_observation_window_ms={QEMU_SOAK_MIN_OBSERVATION_WINDOW_MS - 1}",
            )
        )
    except ValueError:
        pass
    else:
        raise AssertionError("undersized R2 soak fixture unexpectedly passed")
    try:
        soak_report_lines(
            soak_fixture.replace(
                "[AQUA-COMPOSITOR] stage=drm-wayland-session status=ok",
                "[AQUA-COMPOSITOR] stage=drm-wayland-session status=error",
            )
        )
    except ValueError:
        pass
    else:
        raise AssertionError("crashed R2 soak fixture unexpectedly passed")
    soak_failure_cases = (
        (
            "r2_presentation_input_to_present_samples=5",
            "r2_presentation_input_to_present_samples=4",
        ),
        (
            "r2_presentation_memory_growth_kib=128",
            "r2_presentation_memory_growth_kib=163841",
        ),
        (
            "desktop_launch_surface_app_id=aqua.files",
            "desktop_launch_surface_app_id=aqua.unknown",
        ),
        (
            "drm_wayland_input_keyboard_events=40",
            "drm_wayland_input_keyboard_events=39",
        ),
    )
    for accepted, rejected in soak_failure_cases:
        try:
            soak_report_lines(soak_fixture.replace(accepted, rejected))
        except ValueError:
            pass
        else:
            raise AssertionError(f"invalid R2 soak mutation unexpectedly passed: {rejected}")


def read_bounded_log(path: pathlib.Path) -> str:
    max_bytes = 4 * 1024 * 1024
    if path.stat().st_size > max_bytes:
        raise ValueError("QEMU serial log exceeds the R2 evidence limit")
    return path.read_text(errors="replace")


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        self_test()
        print("Aqua Linux R2 presentation log validator self-test passed.")
        return 0
    if len(sys.argv) >= 2 and sys.argv[1] == "--summarize-repeated":
        paths = [pathlib.Path(argument) for argument in sys.argv[2:]]
        resolved_paths = [path.resolve() for path in paths]
        if len(set(resolved_paths)) != len(resolved_paths):
            raise ValueError("R2 repeated review requires distinct serial log paths")
        logs = [read_bounded_log(path) for path in paths]
        print("\n".join(repeated_review_lines(logs)))
        return 0
    if len(sys.argv) == 3 and sys.argv[1] == "--summarize-soak":
        log = read_bounded_log(pathlib.Path(sys.argv[2]))
        print("\n".join(soak_report_lines(log)))
        return 0
    if len(sys.argv) != 2:
        print(
            f"Usage: {sys.argv[0]} SERIAL_LOG | --summarize-repeated SERIAL_LOG... | --summarize-soak SERIAL_LOG",
            file=sys.stderr,
        )
        return 2
    log = read_bounded_log(pathlib.Path(sys.argv[1]))
    records, _ = validate_evidence_log(log)
    print("r2_qemu_workload_records=4")
    print(f"r2_recorded_max_frame_time_us={max(int(record['max_frame_time_us']) for record in records)}")
    print(f"r2_recorded_max_input_to_present_us={max(int(record['max_input_to_present_us']) for record in records)}")
    print(f"r2_recorded_max_cpu_time_us={max(int(record['cpu_time_us']) for record in records)}")
    print(f"r2_recorded_max_memory_growth_kib={max(int(record['memory_growth_kib']) for record in records)}")
    print(f"r2_budget_profile={QEMU_BUDGET_PROFILE}")
    print(f"r2_budget_max_frame_time_us={QEMU_BUDGET['max_frame_time_us']}")
    print(
        "r2_budget_max_input_to_present_us="
        f"{QEMU_BUDGET['max_input_to_present_us']}"
    )
    print(f"r2_budget_max_cpu_time_us={QEMU_BUDGET['max_cpu_time_us']}")
    print(
        "r2_budget_max_memory_growth_kib="
        f"{QEMU_BUDGET['max_memory_growth_kib']}"
    )
    print("r2_budget_max_dropped_frames=0")
    print("r2_budget_selected=true")
    print("r2_physical_budget_selected=false")
    print("r2_diagnostic_isolation_recorded=true")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"R2 presentation log validation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
