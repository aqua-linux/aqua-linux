# Aqua v1 Application Compatibility

Aqua v1 is a native Wayland system. Its declared application model is limited
to first-party applications and independently tested Wayland clients using
`wl_shm` ARGB8888 buffers. The current external compatibility fixtures are
Weston 14.0.1 `weston-simple-shm`, `weston-simple-damage`, and
`weston-simple-touch`, plus the fuller `weston-terminal` client. The touch
fixture covers protocol-level `wl_touch`
down, motion, up, frame delivery and exact client-painted buffer changes; it
does not constitute physical touchscreen evidence. The terminal fixture opens
a real PTY backed by the packaged shell and proves keyboard-driven client
redrawing, but does not broaden the claim beyond the Weston client toolkit.

XWayland and an Xorg server are not packaged, the graphical session does not
export `DISPLAY`, and Aqua does not claim support for X11-only applications.
The `/usr/share/X11/xkb` tree remains in the image solely as keyboard-layout
data consumed by libxkbcommon for Wayland clients; its presence does not imply
an X11 server or X11 application compatibility.

Applications that require Linux dma-buf client buffers, explicit GPU
synchronization, untested Wayland protocols, or an unverified toolkit are
outside the v1 support boundary until they receive their own implementation
and packaged-runtime evidence. Compositor-owned GBM dma-buf scanout is an
output detail and does not expand the client contract.

The packaged contract is recorded at
`/usr/share/doc/aqua/application-compatibility.txt`. Image validation rejects
XWayland, Xorg server binaries or modules, an X11 socket directory, and any
`DISPLAY` assignment in Aqua session environment files.
