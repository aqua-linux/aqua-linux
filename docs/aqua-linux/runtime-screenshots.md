# Aqua Linux Runtime Screenshots

These images are captures of the current Aqua Linux runtime in QEMU x86_64
TCG. They are not design references, browser mockups, or claims about hardware
support. The capture path boots the Buildroot image, starts the custom Smithay
Wayland compositor through the explicit graphics gate, exercises real shell
input and first-party Wayland clients, then requires a clean session stop.

Regenerate and publish the set with:

```sh
scripts/check-public-runtime-qemu.sh
scripts/publish-public-runtime-screenshots.sh
```

The committed [manifest](assets/runtime/manifest.json) records the source
revision, QEMU environment, dimensions, validation marker, and SHA-256 digest
for each image.

## Clean Desktop

![Aqua Linux clean desktop captured in QEMU](assets/runtime/qemu-desktop.png)

## Applications

![Aqua Linux Applications captured in QEMU](assets/runtime/qemu-applications.png)

## Global Search

![Aqua Linux Global Search captured in QEMU](assets/runtime/qemu-search.png)

## First-Party Windows

![Aqua Files and Aqua Settings captured in QEMU](assets/runtime/qemu-first-party-windows.png)
