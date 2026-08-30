#!/usr/bin/env python3
"""Validate four bounded R2 presentation records from a QEMU serial log."""

from __future__ import annotations

import pathlib
import sys


WORKLOADS = {"idle", "window-interaction", "animation", "multi-client"}
MIN_REVIEW_RUNS = 3
MAX_REVIEW_RUNS = 10
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


def validate_record(record: dict[str, str]) -> dict[str, int | str | bool]:
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
            "r2_review_budget_selected=false",
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
    assert "r2_review_budget_selected=false" in review
    try:
        validate_log(fixture.replace("r2_presentation_path=production-gbm-kms", "r2_presentation_path=legacy-cpu-copy", 1))
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
    if len(sys.argv) != 2:
        print(
            f"Usage: {sys.argv[0]} SERIAL_LOG | --summarize-repeated SERIAL_LOG...",
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
    print("r2_budget_selected=false")
    print("r2_diagnostic_isolation_recorded=true")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"R2 presentation log validation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
