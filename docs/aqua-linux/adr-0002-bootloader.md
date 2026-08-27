# ADR 0002: Installed-System Bootloader

## Status

Accepted on 2026-08-16.

## Context

Aqua Linux uses Buildroot with BusyBox init and needs an installer-owned boot path for QEMU x86_64 and later UEFI hardware. The existing QEMU direct-kernel path is valuable for deterministic development and recovery tests, but it is not an installed-disk boot architecture.

## Decision

- Use GRUB2 for the first Aqua Linux v1 installed-system boot path.
- Support x86_64 UEFI first. Legacy BIOS installation is deferred.
- Use Buildroot's generated `images/efi-part/EFI/BOOT/bootx64.efi` artifact.
- Install it at `EFI/BOOT/BOOTX64.EFI`, the architecture-standard removable-media fallback path, so the first implementation does not depend on mutable firmware NVRAM entries.
- Keep `grub.cfg` beside the EFI executable, locate its files by the `AQUA_ROOT` filesystem label, and pass the matching GPT `PARTLABEL=AQUA_ROOT` to the kernel so boot does not require an initramfs or a device-name assumption.
- Keep QEMU direct-kernel boot as a separate development and recovery-validation path.
- Do not use systemd-boot. Aqua Linux currently uses BusyBox init, and GRUB2 already has a supported Buildroot integration for the required filesystems and partition layout.

The EFI image embeds `boot`, `linux`, `ext2`, `fat`, `part_gpt`, `normal`, `efi_gop`, `search`, and `search_label`. GRUB's `ext2` module also reads the ext4 root filesystem used by Aqua Linux.

## Consequences

- The installer copies a prebuilt EFI application instead of running `grub-install` on the target.
- The ESP is FAT32 labeled `AQUA_EFI`; the root filesystem is ext4 labeled `AQUA_ROOT`.
- The planned kernel command line retains both graphical console and serial recovery output.
- Secure Boot signing, firmware NVRAM registration, legacy BIOS, and multi-boot discovery remain later hardening work.
- Selecting a strategy does not enable destructive installation. The executor remains disabled until tool prerequisites, target revalidation, rollback behavior, and disposable-disk QEMU validation are complete.
