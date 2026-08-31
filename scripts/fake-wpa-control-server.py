#!/usr/bin/env python3
import argparse
import os
import socket
import stat


EXPECTED = [
    (b"STATUS", b"wpa_state=DISCONNECTED\n"),
    (b"ADD_NETWORK", b"7\n"),
    (b"SET_NETWORK 7 ssid 49454545", b"OK\n"),
    (b"SET_NETWORK 7 key_mgmt WPA-PSK", b"OK\n"),
    (
        b"SET_NETWORK 7 psk "
        b"f42c6fc52df0ebef9ebb4b90b38a5f902e83fe1b135a70e23aed762e9710a12e",
        b"OK\n",
    ),
    (b"ENABLE_NETWORK 7", b"OK\n"),
    (b"SELECT_NETWORK 7", b"OK\n"),
    (b"STATUS", b"id=7\nwpa_state=COMPLETED\n"),
]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--socket", required=True)
    parser.add_argument("--client-root", required=True)
    args = parser.parse_args()
    parent = os.path.dirname(args.socket)
    os.makedirs(parent, mode=0o755, exist_ok=True)
    if os.path.lexists(args.socket):
        metadata = os.lstat(args.socket)
        if not stat.S_ISSOCK(metadata.st_mode):
            raise SystemExit("refusing to replace a non-socket fixture path")
        os.unlink(args.socket)

    server = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
    server.settimeout(20)
    server.bind(args.socket)
    try:
        for expected, response in EXPECTED:
            request, client = server.recvfrom(4097)
            if request != expected:
                raise SystemExit("unexpected typed wpa_supplicant fixture request")
            if not client or (isinstance(client, bytes) and client.startswith(b"\0")):
                raise SystemExit("unexpected wpa_supplicant fixture client address")
            if isinstance(client, bytes):
                rooted_client = os.fsencode(args.client_root) + client
            else:
                rooted_client = args.client_root + client
            server.sendto(response, rooted_client)
    finally:
        server.close()
        if os.path.lexists(args.socket):
            os.unlink(args.socket)


if __name__ == "__main__":
    main()
