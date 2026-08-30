#!/usr/bin/env python3
import argparse
import csv
import json
from pathlib import Path


REQUIRED = {
    "aqua-audio-native": "1",
    "aqua-audio-probe": "8",
    "alsa-lib": "1.2.13",
    "eudev": "3.2.14",
    "libglib2": "2.82.5",
    "lua": "5.4.8",
    "pipewire": "1.2.8",
    "wireplumber": "0.5.5",
}
FORBIDDEN = {
    "avahi",
    "bluez5_utils",
    "dbus",
    "ffmpeg",
    "gst1-plugins-base",
    "gstreamer1",
    "jack2",
    "libcamera",
    "pulseaudio",
}


def load_json(path: Path):
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", type=Path, required=True)
    parser.add_argument("--audio", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    base = load_json(args.base)
    audio = load_json(args.audio)
    missing = sorted(name for name in REQUIRED if name not in audio)
    wrong_versions = {
        name: audio[name].get("version")
        for name, version in REQUIRED.items()
        if name in audio and audio[name].get("version") != version
    }
    forbidden = sorted(name for name in FORBIDDEN if name in audio)
    if missing or wrong_versions or forbidden:
        raise SystemExit(
            f"invalid audio closure: missing={missing}, "
            f"wrong_versions={wrong_versions}, forbidden={forbidden}"
        )

    with args.manifest.open(newline="", encoding="utf-8") as source:
        legal_packages = {row[0] for row in csv.reader(source) if row and row[0] != "PACKAGE"}
    missing_legal = sorted(name for name in REQUIRED if name not in legal_packages)
    if missing_legal:
        raise SystemExit(f"legal-info manifest is missing: {missing_legal}")

    added = sorted(set(audio) - set(base))
    report = {
        "schema_version": 1,
        "buildroot_version": "2025.02.17",
        "profile": "aqua_x86_64_audio_rehearsal_defconfig",
        "default_image_changed": False,
        "required_packages": REQUIRED,
        "audio_profile_additions": added,
        "forbidden_compatibility_packages": sorted(FORBIDDEN),
        "legal_info_manifest_verified": True,
        "release_cleared": False,
    }
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
