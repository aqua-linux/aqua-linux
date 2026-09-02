#!/usr/bin/env python3

import os
import json
import re
import socket
import sys
import time


MODES = {
    "basic": (
        "sendkey meta_l 100", "sendkey a 100", "mouse_move -192 1",
        "mouse_button 1", "sendkey b 100", "mouse_button 0",
    ),
    "launcher": (
        "mouse_move 1 1", "mouse_button 1", "mouse_button 0",
        "sendkey a 100", "sendkey b 100", "sendkey meta_l 100",
        "mouse_move -3000 -3000", "mouse_move 200 100",
        "mouse_button 1", "mouse_button 0",
    ),
    "launcher-after-dock": (
        "sendkey f 100", "sendkey i 100", "sendkey l 100",
        "sendkey e 100", "sendkey s 100", "sendkey ret 100",
    ),
    "bottom-applications-activate": (
        "mouse_move -3000 -3000", "mouse_move 146 370",
        "mouse_button 1", "mouse_button 0",
        "mouse_button 1", "mouse_button 0",
    ),
    "desktop-context": (),
    "notification-promote": (
        "mouse_move -2000 -2000", "mouse_move 26 49",
        "mouse_button 2", "mouse_button 0",
        "mouse_move 102 55", "mouse_button 1", "mouse_button 0",
        "mouse_move 3000 3000", "mouse_move -25 -87",
        "mouse_button 1", "mouse_button 0",
    ),
    "trash-empty": (
        "mouse_move -3000 -3000", "mouse_move 26 163",
        "mouse_button 2", "mouse_button 0",
        "mouse_move -3000 -3000", "mouse_move 82 189",
        "mouse_button 1", "mouse_button 0",
        "mouse_button 1", "mouse_button 0", "sendkey meta_l 100",
        "sendkey s 100", "sendkey e 100", "sendkey t 100", "sendkey t 100",
        "sendkey i 100", "sendkey n 100", "sendkey g 100", "sendkey s 100",
        "sendkey ret 100",
    ),
    "settings": (),
    "files-launch-fast": (
        "sendkey meta_l 100", "sendkey f 100", "sendkey i 100",
        "sendkey l 100", "sendkey e 100", "sendkey s 100",
        "sendkey ret 100",
    ),
    "settings-launch-fast": (
        "sendkey meta_l 100", "sendkey s 100", "sendkey e 100",
        "sendkey t 100", "sendkey t 100", "sendkey i 100",
        "sendkey n 100", "sendkey g 100", "sendkey s 100",
        "sendkey ret 100",
    ),
    "public-applications": ("sendkey meta_l 100",),
    "public-search": ("sendkey s 100", "sendkey e 100", "sendkey t 100"),
    "public-launcher-dismiss": ("sendkey esc 100",),
    "workspace-move-right": ("sendkey ctrl-alt-shift-right 250",),
    "workspace-switch-right": ("sendkey ctrl-alt-right 250",),
    "settings-interaction": (
        "sendkey down 100", "sendkey down 100", "sendkey down 100",
        "sendkey down 100",
    ),
    "settings-about": ("sendkey end 100",),
    "input-burst": (
        ("sendkey a 80",) * 12
        + ("mouse_move 350 0",)
        + ("mouse_move 1 0", "mouse_move -1 0") * 8
    ),
    "settings-reset": (
        "mouse_move -3000 -3000", "mouse_move 300 180",
        "mouse_button 1", "mouse_button 0", "sendkey home 100",
    ),
    "close-settings": ("sendkey alt-f4 250",),
    "properties-refresh-pointer": (
        "mouse_move -3000 -3000", "mouse_move 480 315",
        "mouse_button 1", "mouse_button 0",
    ),
    "properties-nonprimary-pointer": (
        "mouse_move -3000 -3000", "mouse_move 480 315",
        "mouse_button 2", "mouse_button 0",
    ),
    "properties-pointer-cancel": (
        "mouse_move -3000 -3000", "mouse_move 480 315",
        "mouse_button 1", "mouse_move 0 -80", "mouse_button 0",
    ),
    "properties-focused-space-action": ("sendkey spc 100",),
    "properties-pointer-blur-space": (
        "mouse_move 0 -80", "mouse_button 1", "mouse_button 0",
        "sendkey spc 100",
    ),
    "properties-keyboard-action": ("sendkey tab 100", "sendkey ret 100"),
    "close-properties": ("sendkey alt-f4 250",),
    "terminal-launch": (
        "sendkey meta_l 100", "sendkey t 100", "sendkey e 100",
        "sendkey r 100", "sendkey m 100", "sendkey i 100",
        "sendkey n 100", "sendkey a 100", "sendkey l 100",
        "sendkey ret 100",
    ),
    "terminal-launch-fast": (
        "sendkey meta_l 100", "sendkey down 100", "sendkey down 100",
        "sendkey ret 100",
    ),
    "terminal-command": (
        "sendkey e 1", "sendkey c 1", "sendkey h 1", "sendkey o 1",
        "sendkey spc 1", "sendkey a 1", "sendkey q 1", "sendkey u 1",
        "sendkey a 1", "sendkey t 1", "sendkey e 1", "sendkey r 1",
        "sendkey m 1", "sendkey i 1", "sendkey n 1", "sendkey a 1",
        "sendkey l 1", "sendkey o 1", "sendkey k 1", "sendkey ret 1",
    ),
    "terminal-resize": ("sendkey alt-f8 250",),
    "close-terminal": ("sendkey alt-f4 250",),
    "installer-welcome-language-keyboard": (
        "sendkey end 100", "sendkey ret 100",
        "sendkey ret 100", "sendkey end 100", "sendkey ret 100",
        "sendkey ret 100",
    ),
    "installer-pointer-welcome-forward": (
        "mouse_move 368 230", "mouse_button 1", "mouse_button 0",
    ),
    "installer-pointer-language-row": (
        "mouse_move -200 -226", "mouse_button 1", "mouse_button 0",
    ),
    "installer-language-keyboard": (
        "sendkey home 100", "sendkey end 100", "sendkey ret 100",
        "sendkey ret 100",
    ),
    "installer-keyboard-partitions": (
        "sendkey end 100", "sendkey ret 100", "sendkey ret 100",
    ),
    "installer-partitions-timezone": (
        "sendkey end 100", "sendkey ret 100", "sendkey ret 100",
    ),
    "installer-timezone-user": (
        "sendkey end 100", "sendkey ret 100",
        "sendkey a 50", "sendkey q 50", "sendkey u 50", "sendkey a 50",
        "sendkey down 100",
        "sendkey u 50", "sendkey s 50", "sendkey e 50", "sendkey r 50",
        "sendkey down 100", "sendkey spc 100", "sendkey ret 100",
        "sendkey end 100", "sendkey ret 100",
    ),
    "installer-user-summary-confirmation": (
        "sendkey ret 100",
        "sendkey shift-e 50", "sendkey shift-r 50", "sendkey shift-a 50",
        "sendkey shift-s 50", "sendkey shift-e 50", "sendkey spc 50",
        "sendkey slash 50", "sendkey d 50", "sendkey e 50", "sendkey v 50",
        "sendkey slash 50", "sendkey v 50", "sendkey d 50", "sendkey b 50",
        "sendkey ret 100",
    ),
    "installer-summary-begin": ("sendkey end 100", "sendkey ret 100"),
    "installer-progress-next": ("sendkey ret 100",),
    "session-recovery": (
        "sendkey f10 100", "sendkey down 100", "sendkey down 100",
        "sendkey down 100", "sendkey ret 100", "sendkey ret 100",
    ),
    "session-open": ("sendkey f10 100",),
    "session-recovery-finish": (
        "sendkey down 100", "sendkey down 100", "sendkey down 100",
        "sendkey ret 100", "sendkey ret 100",
    ),
    "session-recovery-arm": (
        "sendkey down 100", "sendkey down 100", "sendkey down 100",
        "sendkey ret 100",
    ),
    "session-recovery-confirm": ("sendkey ret 100",),
}


def receive_until_prompt(connection: socket.socket) -> bytes:
    response = bytearray()
    while b"(qemu)" not in response:
        chunk = connection.recv(4096)
        if not chunk:
            raise ConnectionError("QEMU monitor closed before returning a prompt")
        response.extend(chunk)
    return bytes(response)


def connect_unix(path: str, timeout: float) -> socket.socket:
    deadline = time.monotonic() + timeout
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    while True:
        try:
            connection.connect(path)
            return connection
        except (FileNotFoundError, ConnectionRefusedError):
            if time.monotonic() >= deadline:
                connection.close()
                raise
            time.sleep(0.1)


def execute_commands(
    connection: socket.socket, commands: tuple[str, ...], delay: float
) -> None:
    for command in commands:
        connection.sendall(f"{command}\n".encode("utf-8"))
        response = receive_until_prompt(connection)
        if b"unknown command" in response or b"Error" in response:
            raise RuntimeError(response.decode("utf-8", errors="replace"))
        if delay:
            time.sleep(delay)


def select_virtio_pointer(connection: socket.socket) -> None:
    connection.sendall(b"info mice\n")
    response = receive_until_prompt(connection)
    match = re.search(rb"Mouse #(\d+): QEMU Virtio Mouse", response)
    if match is None:
        raise RuntimeError("QEMU Virtio Mouse was not listed by the monitor")
    execute_commands(
        connection,
        (f"mouse_set {match.group(1).decode('ascii')}",),
        delay=0,
    )


def mode_uses_pointer(mode: str) -> bool:
    return any(command.startswith("mouse_") for command in MODES[mode])


def receive_qmp_result(connection: socket.socket) -> dict:
    while True:
        line = connection.makefile("rb", buffering=0).readline()
        if not line:
            raise ConnectionError("QEMU QMP monitor closed before returning a result")
        response = json.loads(line)
        if "return" in response or "error" in response:
            return response


def execute_qmp(connection: socket.socket, command: dict) -> None:
    connection.sendall(json.dumps(command, separators=(",", ":")).encode() + b"\n")
    response = receive_qmp_result(connection)
    if "error" in response:
        raise RuntimeError(response["error"].get("desc", str(response["error"])))


def initialize_qmp(connection: socket.socket) -> None:
    greeting = json.loads(connection.makefile("rb", buffering=0).readline())
    if "QMP" not in greeting:
        raise RuntimeError("QEMU QMP greeting was not received")
    execute_qmp(connection, {"execute": "qmp_capabilities"})


def execute_mode_with_qmp(
    hmp: socket.socket, qmp: socket.socket, mode: str, delay: float
) -> None:
    pressed_button = None
    for command in MODES[mode]:
        parts = command.split()
        if parts[0] == "mouse_move":
            events = []
            for axis, value in zip(("x", "y", "z"), parts[1:]):
                if int(value):
                    events.append(
                        {"type": "rel", "data": {"axis": axis, "value": int(value)}}
                    )
            if events:
                execute_qmp(
                    qmp,
                    {
                        "execute": "input-send-event",
                        "arguments": {"events": events},
                    },
                )
        elif parts[0] == "mouse_button":
            button_mask = int(parts[1])
            if button_mask:
                button = {1: "left", 2: "right", 4: "middle"}.get(button_mask)
                if button is None:
                    raise ValueError(f"unsupported mouse button mask: {button_mask}")
                pressed_button = button
                down = True
            else:
                button = pressed_button or "left"
                pressed_button = None
                down = False
            execute_qmp(
                qmp,
                {
                    "execute": "input-send-event",
                    "arguments": {
                        "events": [
                            {"type": "btn", "data": {"button": button, "down": down}}
                        ],
                    },
                },
            )
        elif parts[0] == "sendkey":
            key_parts = parts[1].split("-")
            modifiers = key_parts[:-1]
            qcode = key_parts[-1]
            events = [
                {
                    "type": "key",
                    "data": {
                        "down": True,
                        "key": {"type": "qcode", "data": modifier},
                    },
                }
                for modifier in modifiers
            ]
            events.extend(
                [
                    {
                        "type": "key",
                        "data": {
                            "down": True,
                            "key": {"type": "qcode", "data": qcode},
                        },
                    },
                    {
                        "type": "key",
                        "data": {
                            "down": False,
                            "key": {"type": "qcode", "data": qcode},
                        },
                    },
                ]
            )
            events.extend(
                {
                    "type": "key",
                    "data": {
                        "down": False,
                        "key": {"type": "qcode", "data": modifier},
                    },
                }
                for modifier in reversed(modifiers)
            )
            execute_qmp(
                qmp,
                {
                    "execute": "input-send-event",
                    "arguments": {"events": events},
                },
            )
        else:
            execute_commands(hmp, (command,), delay=0)
        if delay:
            time.sleep(delay)


def mode_delay(mode: str) -> float:
    if mode == "basic":
        return 0
    if mode == "terminal-command":
        return 0.005
    if mode == "input-burst":
        return 0.12
    return 0.35


def request_daemon(control_socket: str, request: str) -> int:
    with connect_unix(control_socket, 10) as connection:
        connection.settimeout(30)
        connection.sendall(f"{request}\n".encode("utf-8"))
        response = bytearray()
        while chunk := connection.recv(4096):
            response.extend(chunk)
    message = response.decode("utf-8", errors="replace").strip()
    if message != "ok":
        print(message or "QEMU input daemon returned no status", file=sys.stderr)
        return 1
    return 0


def serve(monitor_socket: str, control_socket: str) -> int:
    try:
        os.unlink(control_socket)
    except FileNotFoundError:
        pass
    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(control_socket)
    server.listen(4)
    try:
        with connect_unix(monitor_socket, 120) as monitor:
            monitor.settimeout(30)
            receive_until_prompt(monitor)
            qmp_socket = os.environ.get("AQUA_QEMU_QMP_SOCKET")
            qmp = connect_unix(qmp_socket, 120) if qmp_socket else None
            if qmp is not None:
                qmp.settimeout(30)
                initialize_qmp(qmp)
            while True:
                client, _ = server.accept()
                with client:
                    request = client.recv(8192).decode("utf-8").strip()
                    if request == "shutdown":
                        client.sendall(b"ok\n")
                        return 0
                    try:
                        if request.startswith("raw:"):
                            execute_commands(
                                monitor, (request.removeprefix("raw:"),), delay=0
                            )
                        elif request in MODES:
                            if qmp is not None:
                                if mode_uses_pointer(request):
                                    select_virtio_pointer(monitor)
                                execute_mode_with_qmp(
                                    monitor, qmp, request, delay=mode_delay(request)
                                )
                            else:
                                if mode_uses_pointer(request):
                                    select_virtio_pointer(monitor)
                                execute_commands(
                                    monitor, MODES[request], delay=mode_delay(request)
                                )
                        else:
                            raise ValueError(f"unsupported input mode: {request}")
                    except Exception as error:
                        client.sendall(f"error: {error}\n".encode("utf-8"))
                    else:
                        client.sendall(b"ok\n")
            if qmp is not None:
                qmp.close()
    finally:
        server.close()
        try:
            os.unlink(control_socket)
        except FileNotFoundError:
            pass


def print_usage() -> None:
    modes = "|".join(MODES)
    print(
        "usage: send-qemu-monitor-input.py <monitor-socket> "
        f"[{modes}]\n"
        "       send-qemu-monitor-input.py --serve <monitor-socket> "
        "<control-socket>",
        file=sys.stderr,
    )


def main() -> int:
    if len(sys.argv) == 4 and sys.argv[1] == "--serve":
        return serve(sys.argv[2], sys.argv[3])
    if len(sys.argv) not in (2, 3):
        print_usage()
        return 2

    monitor_socket = sys.argv[1]
    mode = sys.argv[2] if len(sys.argv) == 3 else "basic"
    if mode not in MODES:
        print(f"unsupported input mode: {mode}", file=sys.stderr)
        return 2

    control_socket = os.environ.get("AQUA_QEMU_INPUT_CONTROL_SOCKET")
    if control_socket:
        return request_daemon(control_socket, mode)

    with connect_unix(monitor_socket, 10) as connection:
        connection.settimeout(10)
        receive_until_prompt(connection)
        if mode_uses_pointer(mode):
            select_virtio_pointer(connection)
        execute_commands(connection, MODES[mode], delay=mode_delay(mode))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
