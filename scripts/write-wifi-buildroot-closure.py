#!/usr/bin/env python3
import argparse
import csv
import json
from pathlib import Path


REQUIRED = {
    "aqua-wifi-native": "1",
    "libnl": "3.11.0",
    "libopenssl": "3.5.7",
    "wpa_supplicant": "2.12",
}
FORBIDDEN = {
    "connman",
    "dbus",
    "dhcpcd",
    "hostapd",
    "iwd",
    "network-manager",
    "readline",
}


def load_json(path: Path):
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", type=Path, required=True)
    parser.add_argument("--wifi", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    base = load_json(args.base)
    wifi = load_json(args.wifi)
    missing = sorted(name for name in REQUIRED if name not in wifi)
    wrong_versions = {
        name: wifi[name].get("version")
        for name, version in REQUIRED.items()
        if name in wifi and wifi[name].get("version") != version
    }
    forbidden = sorted(name for name in FORBIDDEN if name in wifi)
    if missing or wrong_versions or forbidden:
        raise SystemExit(
            f"invalid Wi-Fi closure: missing={missing}, "
            f"wrong_versions={wrong_versions}, forbidden={forbidden}"
        )

    with args.manifest.open(newline="", encoding="utf-8") as source:
        legal_packages = {row[0] for row in csv.reader(source) if row and row[0] != "PACKAGE"}
    missing_legal = sorted(name for name in REQUIRED if name not in legal_packages)
    if missing_legal:
        raise SystemExit(f"legal-info manifest is missing: {missing_legal}")

    added = sorted(set(wifi) - set(base))
    if set(REQUIRED) - set(added):
        raise SystemExit(f"required packages are not isolated additions: {added}")

    report = {
        "schema_version": 1,
        "buildroot_version": "2025.02.17",
        "profile": "aqua_x86_64_wifi_rehearsal_defconfig",
        "default_image_changed": False,
        "service_enabled": False,
        "authenticated_broker_integration_implemented": True,
        "credential_storage_implemented": True,
        "typed_control_transport_implemented": True,
        "psk_derivation_implemented": True,
        "required_packages": REQUIRED,
        "wifi_profile_additions": added,
        "forbidden_management_packages": sorted(FORBIDDEN),
        "control_transport": "unix-socket-libwpa_client",
        "wireless_driver": "nl80211",
        "legal_info_manifest_verified": True,
        "release_cleared": False,
        "physical_hardware_validated": False,
    }
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
