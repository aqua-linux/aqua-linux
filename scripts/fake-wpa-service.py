#!/usr/bin/env python3
import argparse
import os
import signal
import socket


def main() -> None:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("-i", required=True)
    parser.add_argument("-D", required=True)
    parser.add_argument("-c", required=True)
    parser.add_argument("-C", required=True)
    args = parser.parse_args()
    if args.i != "wlan0" or args.D != "nl80211":
        raise SystemExit("unexpected Wi-Fi service fixture arguments")
    if not os.path.isfile(args.c):
        raise SystemExit("missing Wi-Fi service fixture config")
    os.makedirs(args.C, mode=0o755, exist_ok=True)
    path = os.path.join(args.C, args.i)
    if os.path.lexists(path):
        os.unlink(path)
    control = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
    control.bind(path)
    running = True

    def stop(_signal: int, _frame: object) -> None:
        nonlocal running
        running = False

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    try:
        while running:
            signal.pause()
    finally:
        control.close()
        if os.path.lexists(path):
            os.unlink(path)


if __name__ == "__main__":
    main()
