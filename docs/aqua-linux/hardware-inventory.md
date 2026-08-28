# MSI Sword 17 Hardware Inventory Contract

Status: collector implemented; physical inventory not collected.

`/usr/bin/aqua-hardware-inventory` creates the first bounded machine record
required before MSI Sword 17 validation starts. It reads only Linux `sysfs` and
`procfs`; it does not open device nodes, load drivers, alter firmware, mount
storage, configure networking, or authorize installation.

## Record Scope

The line-oriented `schema=1` record includes:

- DMI vendor, product, board, and firmware model strings;
- CPU model and whether the kernel exposed UEFI runtime data;
- bounded PCI vendor/device/class identifiers and bound driver names;
- bounded USB vendor/product identifiers and product names;
- DRM connector status and first advertised mode;
- input event names;
- whole-block-device model, size, read-only, and removable state; and
- power-supply type, status, and capacity.

Serial numbers, MAC addresses, hostname, filesystem UUIDs, user data, and file
contents are deliberately excluded. The output always declares
`support_claim=false` and `installation_authorized=false`.

## Collection

From an Aqua recovery shell, write the record only to removable media or copy
the terminal output through an operator-controlled channel:

```sh
aqua-hardware-inventory > /path/on/removable-media/msi-sword-17.txt
```

The tool prints to standard output only. The repository fixture verifies that
all source files have identical checksums before and after collection.

## Review Gate

The raw record is evidence intake, not a compatibility result. Before any
hardware status changes:

1. Confirm `collection_complete=true` and the expected MSI product identity.
2. Review the record for accidental unique or private identifiers before
   publishing it.
3. Map each PCI and USB identifier to a kernel driver and any required firmware.
4. Keep every support row as **Not tested** until its dedicated procedure passes.
5. Keep physical-disk installation prohibited until a separate destructive-test
   plan is explicitly approved.

The first real record should be stored under a dated hardware-validation
evidence directory only after that review. No fabricated or reference inventory
is committed in its place.
