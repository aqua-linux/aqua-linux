#!/usr/bin/env python3

import os
import socket
import struct
import sys
import time
import zlib


def receive_until_prompt(connection: socket.socket) -> bytes:
    response = bytearray()
    while b"(qemu)" not in response:
        chunk = connection.recv(4096)
        if not chunk:
            break
        response.extend(chunk)
    return bytes(response)


def request_input_daemon(control_socket: str, command: str) -> bytes:
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    connection.connect(control_socket)
    with connection:
        connection.settimeout(30)
        connection.sendall(f"raw:{command}\n".encode("utf-8"))
        response = bytearray()
        while chunk := connection.recv(4096):
            response.extend(chunk)
    return bytes(response)


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    checksum = zlib.crc32(kind)
    checksum = zlib.crc32(payload, checksum)
    return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", checksum)


def convert_ppm_to_png(ppm_path: str, png_path: str) -> tuple[int, int]:
    with open(ppm_path, "rb") as source:
        if source.readline().strip() != b"P6":
            raise ValueError("QEMU screendump is not a binary PPM")
        dimensions = source.readline().split()
        while dimensions and dimensions[0].startswith(b"#"):
            dimensions = source.readline().split()
        if len(dimensions) != 2 or source.readline().strip() != b"255":
            raise ValueError("unsupported QEMU PPM header")
        width, height = (int(value) for value in dimensions)
        pixels = source.read()

    row_bytes = width * 3
    if len(pixels) != row_bytes * height:
        raise ValueError("QEMU PPM pixel payload has an unexpected size")
    scanlines = b"".join(
        b"\x00" + pixels[offset : offset + row_bytes]
        for offset in range(0, len(pixels), row_bytes)
    )
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", header)
        + png_chunk(b"IDAT", zlib.compress(scanlines, level=9))
        + png_chunk(b"IEND", b"")
    )
    with open(png_path, "wb") as target:
        target.write(png)
    return width, height


def main() -> int:
    if len(sys.argv) != 4:
        print(
            "usage: capture-qemu-monitor-screendump.py <monitor-socket> <output.ppm> <output.png>",
            file=sys.stderr,
        )
        return 2

    monitor_socket, output_path, png_path = sys.argv[1:]
    control_socket = os.environ.get("AQUA_QEMU_INPUT_CONTROL_SOCKET")
    if control_socket:
        response = request_input_daemon(control_socket, f"screendump {output_path}")
    else:
        deadline = time.monotonic() + 10
        connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        while True:
            try:
                connection.connect(monitor_socket)
                break
            except (FileNotFoundError, ConnectionRefusedError):
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.1)

        with connection:
            connection.settimeout(10)
            receive_until_prompt(connection)
            connection.sendall(f"screendump {output_path}\n".encode("utf-8"))
            response = receive_until_prompt(connection)

    if b"Error" in response or b"error:" in response or not os.path.isfile(output_path):
        print(response.decode("utf-8", errors="replace"), file=sys.stderr)
        return 1
    width, height = convert_ppm_to_png(output_path, png_path)
    if (width, height) != (1280, 800):
        print(f"unexpected QEMU screendump size: {width}x{height}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
