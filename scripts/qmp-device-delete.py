#!/usr/bin/env python3
"""Delete one QEMU device and require its asynchronous completion event."""

import json
import re
import socket
import sys
import time


DEVICE_ID = re.compile(r"[A-Za-z0-9_.-]{1,128}\Z")


def fail(reason: str) -> None:
    raise SystemExit(f"QMP device deletion failed: {reason}")


def receive_message(stream, deadline: float) -> dict:
    while True:
        if time.monotonic() >= deadline:
            fail("timed out waiting for QMP response")
        line = stream.readline()
        if not line:
            fail("QMP connection closed")
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            fail(f"invalid QMP response: {error}")
        if isinstance(message, dict):
            return message


def send_command(stream, command: dict) -> None:
    stream.write(json.dumps(command, separators=(",", ":")).encode() + b"\n")
    stream.flush()


def main() -> None:
    if len(sys.argv) != 3:
        fail("usage: qmp-device-delete.py SOCKET DEVICE_ID")
    socket_path, device_id = sys.argv[1:]
    if not DEVICE_ID.fullmatch(device_id):
        fail("invalid device id")

    deadline = time.monotonic() + 30
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    connection.settimeout(30)
    try:
        connection.connect(socket_path)
        stream = connection.makefile("rwb", buffering=0)
        greeting = receive_message(stream, deadline)
        if "QMP" not in greeting:
            fail("missing QMP greeting")

        send_command(stream, {"execute": "qmp_capabilities", "id": "capabilities"})
        while True:
            message = receive_message(stream, deadline)
            if message.get("id") == "capabilities":
                if "error" in message:
                    fail(f"capability negotiation rejected: {message['error']}")
                break

        send_command(
            stream,
            {
                "execute": "device_del",
                "arguments": {"id": device_id},
                "id": "device-delete",
            },
        )
        accepted = False
        while True:
            message = receive_message(stream, deadline)
            if message.get("id") == "device-delete":
                if "error" in message:
                    fail(f"device_del rejected: {message['error']}")
                accepted = True
            if message.get("event") == "DEVICE_DELETED":
                deleted = message.get("data", {}).get("device")
                if deleted == device_id:
                    if not accepted:
                        fail("completion event arrived before command acknowledgement")
                    break
        print(
            "[AQUA-AUDIO] stage=qmp-device-delete status=ok "
            f"device={device_id} event=DEVICE_DELETED"
        )
    except (OSError, socket.timeout) as error:
        fail(str(error))
    finally:
        connection.close()


if __name__ == "__main__":
    main()
