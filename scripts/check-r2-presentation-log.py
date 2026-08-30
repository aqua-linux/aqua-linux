#!/usr/bin/env python3
"""Validate four bounded R2 presentation records from a QEMU serial log."""

from __future__ import annotations

import pathlib
import sys


WORKLOADS = {"idle", "window-interaction", "animation", "multi-client"}
INTEGER_FIELDS = {
    "frames_requested",
    "frames_presented",
    "page_flip_events",
    "frame_callbacks_sent",
    "damage_commits",
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
    if workload != "idle" and int(parsed["frame_callbacks_sent"]) == 0:
        raise ValueError(f"missing frame callback evidence for {workload}")
    if int(parsed["max_frame_time_us"]) == 0 or int(parsed["observation_window_ms"]) == 0:
        raise ValueError(f"missing timing evidence for {workload}")
    if int(parsed["cpu_framebuffer_copies"]) != 0:
        raise ValueError(f"CPU framebuffer copy recorded for {workload}")

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
        if int(parsed["damage_commits"]) == 0:
            raise ValueError(f"missing damage evidence for {workload}")
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


def self_test() -> None:
    fixture = "\n".join(fixture_record(workload) for workload in sorted(WORKLOADS))
    records = validate_log(fixture)
    assert len(records) == 4
    try:
        validate_log(fixture.replace("r2_presentation_path=production-gbm-kms", "r2_presentation_path=legacy-cpu-copy", 1))
    except ValueError:
        pass
    else:
        raise AssertionError("legacy path fixture unexpectedly passed")
    try:
        validate_log(f"r2_presentation_workload=idle\n{fixture}")
    except ValueError:
        pass
    else:
        raise AssertionError("out-of-record field fixture unexpectedly passed")


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
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} SERIAL_LOG", file=sys.stderr)
        return 2
    records = validate_log(read_bounded_log(pathlib.Path(sys.argv[1])))
    print("r2_qemu_workload_records=4")
    print(f"r2_recorded_max_frame_time_us={max(int(record['max_frame_time_us']) for record in records)}")
    print(f"r2_recorded_max_input_to_present_us={max(int(record['max_input_to_present_us']) for record in records)}")
    print(f"r2_recorded_max_cpu_time_us={max(int(record['cpu_time_us']) for record in records)}")
    print(f"r2_recorded_max_memory_growth_kib={max(int(record['memory_growth_kib']) for record in records)}")
    print("r2_budget_selected=false")
    print("r2_diagnostic_isolation_recorded=false")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"R2 presentation log validation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
