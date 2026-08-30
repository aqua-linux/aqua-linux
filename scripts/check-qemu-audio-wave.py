#!/usr/bin/env python3

import array
import math
import struct
import sys


def fail(reason: str) -> None:
    raise SystemExit(f"QEMU audio WAV contract failed: {reason}")


if len(sys.argv) != 2:
    fail("expected one WAV path")

with open(sys.argv[1], "rb") as capture:
    header = capture.read(44)
    payload = capture.read()

if len(header) != 44 or header[0:4] != b"RIFF" or header[8:12] != b"WAVE":
    fail("invalid_riff_header")
if header[12:16] != b"fmt " or header[36:40] != b"data":
    fail("unsupported_chunk_layout")
format_code, channels, rate, _, block_align, bits = struct.unpack_from(
    "<HHIIHH", header, 20
)
if format_code != 1:
    fail(f"format_code={format_code}")
sample_width = bits // 8
if bits != 16:
    fail(f"bits={bits}")
if channels != 2:
    fail(f"channels={channels}")
if sample_width != 2:
    fail(f"sample_width={sample_width}")
if block_align != channels * sample_width:
    fail(f"block_align={block_align}")
if rate != 48000:
    fail(f"rate={rate}")
frames = len(payload) // block_align
if frames < 24000:
    fail(f"frames={frames}")
samples = array.array("h")
samples.frombytes(payload)
if sys.byteorder != "little":
    samples.byteswap()
rms = int(math.sqrt(sum(sample * sample for sample in samples) / len(samples)))
if rms < 100:
    fail(f"silent_or_too_quiet rms={rms}")

print(
    "[AQUA-AUDIO] stage=qemu-wave-capture status=ok "
    f"frames={frames} rate={rate} channels={channels} sample_width={sample_width} rms={rms}"
)
