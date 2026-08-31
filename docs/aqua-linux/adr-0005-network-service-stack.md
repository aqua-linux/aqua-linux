# ADR 0005: Network Service Stack

## Status

Accepted on 2026-08-31. The observation boundary is implemented; privileged
configuration, Wi-Fi packaging, and end-to-end runtime evidence remain gated.

## Context

The default Buildroot image uses eudev and BusyBox init, sets
`BR2_SYSTEM_DHCP="eth0"`, and obtains its initial Ethernet configuration through
BusyBox `udhcpc`. Aqua Settings previously read only bounded `operstate` values
from `/sys/class/net`. It could not distinguish an offline link from a link
waiting for configuration, a usable route, or a route without DNS. It also had
no selected architecture for Wi-Fi or privileged network changes.

R4 requires link state, Wi-Fi association where supported, DHCP, DNS,
reconnection, visible offline/error behavior, and Settings integration. The
graphical session runs as the unprivileged `aqua` user, so it must not own raw
network devices, rewrite resolver state, or launch privileged helper commands.

## Decision

Aqua Linux will use this network stack:

1. Linux networking and eudev form the device and link-discovery boundary.
2. BusyBox `udhcpc` remains the initial IPv4 DHCP client. Its lease lifecycle,
   retry budget, route updates, resolver updates, logs, and degraded state will
   move under a root-owned Aqua network supervisor before network management is
   enabled in Settings.
3. `wpa_supplicant` is selected for Wi-Fi association on hardware that requires
   it. It remains absent from the default image until an opt-in Buildroot
   rehearsal resolves its dependency and license closure and a typed control
   transport, credential-storage contract, and radio evidence pass.
4. Aqua will not add NetworkManager, ConnMan, or a second DHCP client to the v1
   image. A single policy owner avoids competing route, lease, and resolver
   writers on the BusyBox system.
5. The unprivileged `aqua-service-adapters` boundary observes bounded typed
   interface state from `/sys/class/net`, the kernel IPv4 default-route table
   from `/proc/net/route`, and validated IP nameservers from `/etc/resolv.conf`.
   It does not parse diagnostic CLI output, spawn commands, or report saved
   intent as applied state.
6. Settings maps the authoritative observation to `unavailable`, `offline`,
   `configuring`, `online`, or `degraded`. `online` requires an up interface,
   an up default route bound to that interface, and at least one valid DNS
   server. Missing route data or a route without usable DNS fails visibly.
7. Future configuration requests cross a narrow authenticated privilege broker
   with an operation allowlist, target interface binding, bounded timeouts,
   secret redaction, and authoritative acknowledgement. The current Settings
   surface remains read-only and exposes no configuration request path.
8. The initial resolver is libc plus the resolver file managed by the selected
   DHCP lifecycle. A caching or validating resolver is not added without a
   separate operational and security decision.
9. IPv6 configuration, captive portals, VPNs, hotspot mode, enterprise Wi-Fi,
   and Bluetooth networking are outside this first boundary and require their
   own contracts and evidence.

## Failure And Security Contract

- Interface discovery is capped at eight valid Linux interface names and does
  not expose loopback. Directory traversal is bounded independently.
- Route and resolver inputs have fixed byte limits. Nameservers must parse as
  IPv4 or IPv6 addresses, duplicates are removed, and at most three are
  exposed.
- No up link reports `offline`; an up link without a default route reports
  `configuring`; unreadable route state, a route bound to a non-up interface,
  or a routed link without valid DNS reports `degraded`.
- Failure to read the interface source reports `unavailable` to Settings. The
  shell stays responsive and network controls remain absent.
- SSIDs, passphrases, DHCP options, and resolver search domains are not logged
  or exposed by this observation boundary.

## Packaging And Acceptance Gates

Network management must remain disabled until all of these are satisfied:

1. A root-owned supervisor provides finite DHCP and association startup,
   reconnect, restart, shutdown, state publication, and recovery behavior.
2. The privilege broker authenticates the Aqua session and accepts only typed,
   allowlisted operations without accepting arbitrary commands or paths.
3. An opt-in Buildroot profile resolves `wpa_supplicant` and its exact legal
   closure while the default profile remains unchanged.
4. Deterministic fixtures prove offline, configuring, online, DNS loss, route
   loss, malformed input, bounded source sizes, and recovery without hanging
   Settings or the shell.
5. QEMU proves Ethernet DHCP, default routing, DNS lookup, lease renewal,
   forced service failure, bounded reconnect, and recovery-shell availability.
6. Wi-Fi is reported only on a target with an applicable radio. Association,
   secret handling, DHCP, DNS, reconnect, radio disable, and failure recovery
   require evidence for that target.

Physical Ethernet and Wi-Fi support remain `Not tested`; this decision and its
deterministic observation tests are not hardware evidence.

## Consequences

The existing small Buildroot architecture gains one explicit network policy
owner without enabling a large general-purpose network daemon. Aqua can now
present truthful route and DNS health before privileged management exists. The
tradeoff is that advanced network features remain deliberately unavailable
until their broker, packaging, and runtime evidence are complete.
