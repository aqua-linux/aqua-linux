#!/usr/bin/env python3

import sys
from pathlib import Path


def read_ppm(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    tokens: list[bytes] = []
    offset = 0
    while len(tokens) < 4:
        while offset < len(data) and data[offset] in b" \t\r\n":
            offset += 1
        if offset < len(data) and data[offset] == ord("#"):
            while offset < len(data) and data[offset] not in b"\r\n":
                offset += 1
            continue
        start = offset
        while offset < len(data) and data[offset] not in b" \t\r\n":
            offset += 1
        tokens.append(data[start:offset])

    if tokens[0] != b"P6" or tokens[3] != b"255":
        raise ValueError(f"unsupported PPM header: {path}")
    width, height = int(tokens[1]), int(tokens[2])
    if offset >= len(data) or data[offset] not in b" \t\r\n":
        raise ValueError(f"missing PPM header separator: {path}")
    if data[offset : offset + 2] == b"\r\n":
        offset += 2
    else:
        offset += 1
    pixels = data[offset:]
    expected = width * height * 3
    if len(pixels) != expected:
        raise ValueError(f"invalid PPM payload: {path}")
    return width, height, pixels


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: check-qemu-theme-frame-delta.py BEFORE.ppm AFTER.ppm", file=sys.stderr)
        return 2

    before_width, before_height, before = read_ppm(Path(sys.argv[1]))
    after_width, after_height, after = read_ppm(Path(sys.argv[2]))
    if (before_width, before_height) != (after_width, after_height):
        print("theme frames have different dimensions", file=sys.stderr)
        return 1

    pixel_count = before_width * before_height
    changed_pixels = 0
    absolute_delta = 0
    for offset in range(0, len(before), 3):
        pixel_delta = sum(abs(before[offset + channel] - after[offset + channel]) for channel in range(3))
        absolute_delta += pixel_delta
        if pixel_delta >= 24:
            changed_pixels += 1

    changed_percent = changed_pixels * 100.0 / pixel_count
    mean_channel_delta = absolute_delta / (pixel_count * 3)
    print(
        "qemu_theme_frame_delta="
        f"changed_pixels:{changed_pixels} "
        f"changed_percent:{changed_percent:.2f} "
        f"mean_channel_delta:{mean_channel_delta:.2f}"
    )
    if changed_percent < 5.0 or mean_channel_delta < 2.0:
        print("theme frame delta is below the visible-change threshold", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
