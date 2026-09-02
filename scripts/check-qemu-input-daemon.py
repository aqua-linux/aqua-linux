#!/usr/bin/env python3

import os
import socket
import subprocess
import tempfile
import threading
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
HELPER = ROOT / "scripts/send-qemu-monitor-input.py"


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="aqua-hmp-daemon-") as directory:
        root = Path(directory)
        monitor = root / "monitor.sock"
        control = root / "control.sock"
        commands = []
        accept_count = 0

        def fake_monitor() -> None:
            nonlocal accept_count
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
                server.bind(str(monitor))
                server.listen(1)
                connection, _ = server.accept()
                accept_count += 1
                with connection:
                    connection.sendall(b"QEMU monitor\n(qemu) ")
                    stream = connection.makefile("rb")
                    for line in stream:
                        command = line.decode("utf-8").strip()
                        commands.append(command)
                        if command == "info mice":
                            connection.sendall(
                                b"  Mouse #4: QEMU Virtio Mouse\n"
                                b"* Mouse #2: QEMU PS/2 Mouse\n"
                            )
                        connection.sendall(b"(qemu) ")

        monitor_thread = threading.Thread(target=fake_monitor, daemon=True)
        monitor_thread.start()
        daemon = subprocess.Popen(
            ["python3", str(HELPER), "--serve", str(monitor), str(control)]
        )
        try:
            for _ in range(100):
                if control.exists():
                    break
                if daemon.poll() is not None:
                    raise RuntimeError("input daemon exited before creating its control socket")
                time.sleep(0.02)
            if not control.exists():
                raise RuntimeError("input daemon did not create its control socket")

            environment = os.environ | {
                "AQUA_QEMU_INPUT_CONTROL_SOCKET": str(control)
            }
            for mode in (
                "basic",
                "files-keyboard-focus",
                "settings-pointer-blur",
                "settings-keyboard-refocus",
                "settings-keyboard-blur",
                "settings-refocus",
                "settings-about",
                "properties-nonprimary-pointer",
                "properties-pointer-cancel",
                "properties-refresh-pointer",
                "properties-focused-space-action",
                "properties-pointer-blur-space",
                "properties-keyboard-blur",
                "properties-blurred-space",
                "properties-keyboard-action",
                "close-properties",
            ):
                subprocess.run(
                    ["python3", str(HELPER), str(monitor), mode],
                    check=True,
                    env=environment,
                )

            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                client.connect(str(control))
                client.sendall(b"shutdown\n")
                if client.recv(32).strip() != b"ok":
                    raise RuntimeError("input daemon did not acknowledge shutdown")
            if daemon.wait(timeout=5) != 0:
                raise RuntimeError("input daemon returned a failure status")
        finally:
            if daemon.poll() is None:
                daemon.terminate()
                daemon.wait(timeout=5)

        monitor_thread.join(timeout=5)
        if monitor_thread.is_alive():
            raise RuntimeError("fake QEMU monitor did not stop")
        if accept_count != 1:
            raise RuntimeError(f"expected one monitor connection, got {accept_count}")
        expected = [
            "info mice",
            "mouse_set 4",
            "sendkey meta_l 100",
            "sendkey a 100",
            "mouse_move -192 1",
            "mouse_button 1",
            "sendkey b 100",
            "mouse_button 0",
            "sendkey left 100",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 520 235",
            "mouse_button 1",
            "mouse_button 0",
            "sendkey down 100",
            "sendkey up 100",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 220 200",
            "mouse_button 1",
            "mouse_button 0",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 520 235",
            "mouse_button 1",
            "mouse_button 0",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 520 235",
            "mouse_button 1",
            "mouse_button 0",
            "sendkey end 100",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 480 315",
            "mouse_button 2",
            "mouse_button 0",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 480 315",
            "mouse_button 1",
            "mouse_move 0 -80",
            "mouse_button 0",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 480 315",
            "mouse_button 1",
            "mouse_button 0",
            "sendkey spc 100",
            "info mice",
            "mouse_set 4",
            "mouse_move 0 -80",
            "mouse_button 1",
            "mouse_button 0",
            "sendkey spc 100",
            "info mice",
            "mouse_set 4",
            "sendkey tab 100",
            "mouse_move -3000 -3000",
            "mouse_move 220 200",
            "mouse_button 1",
            "mouse_button 0",
            "sendkey spc 100",
            "info mice",
            "mouse_set 4",
            "mouse_move -3000 -3000",
            "mouse_move 520 235",
            "mouse_button 1",
            "mouse_button 0",
            "sendkey tab 100",
            "sendkey ret 100",
            "sendkey alt-f4 250",
        ]
        if commands != expected:
            raise RuntimeError(f"unexpected monitor commands: {commands!r}")

    print("Persistent QEMU HMP input daemon checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
