#!/usr/bin/env python3
"""Probe bounded Intel HDA output restoration and roll back partial state."""

import json
import re
import socket
import sys
import time


IDENTIFIER = re.compile(r"[A-Za-z0-9_.-]{1,128}\Z")
PCI_ADDRESS = re.compile(r"(?:0[0-9]|[12][0-9]|3[01])\.[0-7]\Z")


def fail(reason: str) -> None:
    raise SystemExit(f"QMP audio output add failed: {reason}")


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


def command_response(stream, command: dict, command_id: str, deadline: float) -> dict:
    send_command(stream, {**command, "id": command_id})
    while True:
        message = receive_message(stream, deadline)
        if message.get("id") == command_id:
            return message


def require_response(stream, command: dict, command_id: str, deadline: float) -> None:
    message = command_response(stream, command, command_id, deadline)
    if "error" in message:
        fail(f"{command_id} rejected: {message['error']}")


def delete_and_wait(stream, device_id: str, deadline: float) -> None:
    send_command(
        stream,
        {
            "execute": "device_del",
            "arguments": {"id": device_id},
            "id": "controller-rollback",
        },
    )
    accepted = False
    while True:
        message = receive_message(stream, deadline)
        if message.get("id") == "controller-rollback":
            if "error" in message:
                fail(f"controller rollback rejected: {message['error']}")
            accepted = True
        if message.get("event") == "DEVICE_DELETED":
            deleted = message.get("data", {}).get("device")
            if deleted == device_id:
                if not accepted:
                    fail("controller rollback event arrived before acknowledgement")
                return


def main() -> None:
    if len(sys.argv) != 6:
        fail(
            "usage: qmp-audio-output-add.py SOCKET CONTROLLER_ID "
            "CODEC_ID AUDIODEV_ID PCI_ADDRESS"
        )
    socket_path, controller_id, codec_id, audiodev_id, pci_address = sys.argv[1:]
    for label, value in (
        ("controller id", controller_id),
        ("codec id", codec_id),
        ("audiodev id", audiodev_id),
    ):
        if not IDENTIFIER.fullmatch(value):
            fail(f"invalid {label}")
    if not PCI_ADDRESS.fullmatch(pci_address):
        fail("invalid PCI address")

    deadline = time.monotonic() + 30
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    connection.settimeout(30)
    try:
        connection.connect(socket_path)
        stream = connection.makefile("rwb", buffering=0)
        greeting = receive_message(stream, deadline)
        if "QMP" not in greeting:
            fail("missing QMP greeting")
        require_response(
            stream,
            {"execute": "qmp_capabilities"},
            "capabilities",
            deadline,
        )
        require_response(
            stream,
            {
                "execute": "device_add",
                "arguments": {
                    "driver": "ich9-intel-hda",
                    "id": controller_id,
                    "addr": pci_address,
                },
            },
            "controller-add",
            deadline,
        )
        codec_response = command_response(
            stream,
            {
                "execute": "device_add",
                "arguments": {
                    "driver": "hda-output",
                    "id": codec_id,
                    "bus": f"{controller_id}.0",
                    "audiodev": audiodev_id,
                },
            },
            "codec-add",
            deadline,
        )
        if "error" not in codec_response:
            fail("QEMU unexpectedly accepted runtime HDA codec insertion")
        error = codec_response["error"]
        if not isinstance(error, dict) or error.get("class") != "GenericError":
            fail(f"unexpected codec-add rejection: {error}")
        description = error.get("desc")
        if not isinstance(description, str) or "does not support hotplugging" not in description:
            fail(f"unexpected codec-add rejection: {error}")
        delete_and_wait(stream, controller_id, deadline)
        print(
            "[AQUA-AUDIO] stage=qmp-audio-output-add status=blocked "
            "reason=hda-codec-bus-not-hotpluggable "
            f"controller={controller_id} codec={codec_id} "
            f"audiodev={audiodev_id} slot={pci_address} "
            "controller_rollback=true"
        )
    except (OSError, socket.timeout) as error:
        fail(str(error))
    finally:
        connection.close()


if __name__ == "__main__":
    main()
