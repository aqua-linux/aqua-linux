# ADR 0005: Network Service Stack

## Status

Accepted on 2026-08-31. The observation boundary, disabled root-owned DHCP
supervisor, authenticated privilege broker, and opt-in QEMU runtime acceptance
are implemented. The isolated Wi-Fi package and legal closure is rehearsed.
The typed Wi-Fi contract is bound to an opt-in native control bridge and the
authenticated broker, with bounded PSK derivation and deterministic transport
evidence. Default ownership, Settings configuration controls, radio lifecycle,
and physical runtime evidence remain gated.

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
   it. An opt-in Buildroot rehearsal resolves its dependency and license
   closure together with Aqua's native control bridge, but both remain absent
   from the default image until radio lifecycle and target evidence pass.
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
7. Configuration requests cross a narrow authenticated privilege broker
   with an operation allowlist, target interface binding, bounded timeouts,
   secret redaction, and authoritative acknowledgement. Settings exposes
   broker-gated Wi-Fi discovery and explicit rescan, WPA2-Personal credential
   entry, disconnect, saved-credential reconnect, and saved-network forget;
   when the fixed broker socket is absent, the controls remain visibly
   disabled and the default image stays read-only.
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
- The initial personal-network control contract accepts only typed status,
  scan, add, SSID, derived WPA2 PSK, key-management, enable, select, remove,
  and disconnect requests. It emits no caller-supplied command name or network
  identifier above 4095. Commands are capped at 192 bytes; responses are capped
  at 4096 bytes and association is authoritative only for `COMPLETED` with a
  validated network identifier.
- SSIDs and secrets have redacted debug output. An input passphrase is limited
  to 8-63 printable ASCII bytes and must remain transient. The persisted record
  contains only the derived 32-byte WPA2 PSK, but that PSK is still treated as
  a network-equivalent secret. The opt-in native bridge performs the bounded
  WPA2 derivation and the broker wipes the request buffer after parsing.
- Native scan input is capped at 4096 bytes and 32 rows, validates BSSID,
  frequency, signal, flags, and printable SSID fields, deduplicates by SSID,
  and returns only the four strongest results. The authenticated broker emits
  one 512-byte-bounded response whose entries contain hex-encoded SSID,
  validated signal level, and either `wpa2-personal` or `unsupported` security.
  Settings never permits credential entry for an unsupported result.
- Settings keeps the passphrase in a fixed 63-byte redacted buffer, accepts
  only printable ASCII, masks rendering, rejects submission before eight
  bytes, and wipes the buffer on cancel or after every broker request. A
  failed association stays in credential entry for at most one explicit retry;
  the second failure closes the flow. The settings configuration and logs
  never persist or print the passphrase.
- The v1 credential record has one versioned, 256-byte-bounded schema at the
  fixed `/var/lib/aqua-network/wifi.psk` path. Its directory and file must be
  root-owned normal non-symlink objects with exact `0700` and `0600` modes.
  Writes must use the fixed sibling temporary path, sync, and atomic rename.
  Forget validates the same directory and record metadata before unlinking the
  record, syncs the directory, clears association state, and acknowledges only
  `credential_saved=false`.
  This permission boundary is not an encryption-at-rest claim.

## Packaging And Acceptance Gates

Network management must remain disabled until all of these are satisfied:

1. **Satisfied on 2026-08-31:** a packaged root-owned supervisor
   provides finite DHCP startup/readiness, lease-loss detection, restart,
   shutdown, public non-secret state, and deterministic recovery behavior.
   Aqua's custom `rcS` does not invoke Buildroot's generated `S40network`, so
   the default boot currently has no DHCP policy owner. The supervisor remains
   disabled by default. An exact `aqua.boot_network=1` kernel flag plus the
   separate QEMU-only profile can opt into the transition without changing the
   recovery boot. A validated regular-file launcher supplies the fixed BusyBox
   `udhcpc` invocation without weakening executable preflight.
2. **Satisfied on 2026-08-31:** the privilege broker authenticates the Aqua
   UID/GID from kernel Unix-socket peer credentials and accepts only the fixed,
   versioned `status` and `renew-dhcp` operations for `eth0`. Requests,
   responses, state input, I/O waits, and renewal acknowledgement are bounded.
   Root and other peers are rejected; no arbitrary command or path crosses the
   protocol. The broker is packaged but starts only with the opt-in network
   profile, and Settings still has no configuration control.
3. **Satisfied on 2026-08-31:** the opt-in
   `aqua_x86_64_wifi_rehearsal_defconfig` resolves `wpa_supplicant` 2.12,
   libnl 3.11.0, and OpenSSL 3.5.7 and completes Buildroot `legal-info`.
   It selects only nl80211, autoscan, WPA3, the control interface, and
   `libwpa_client`, plus the MIT-licensed `aqua-wifi-native` bridge; it excludes
   alternate network managers, D-Bus, a second DHCP client, CLI/passphrase
   tools, and unsupported association modes. The default profile and service
   ownership remain unchanged.
4. **Satisfied on 2026-08-31:** the opt-in `aqua-wifi-native` bridge executes
   only bounded typed requests through `libwpa_client` at the fixed
   `wlan0` control socket and derives WPA2 PSKs through OpenSSL's bounded PBKDF2
   API. The authenticated broker accepts only typed status, scan, connect,
   saved reconnect, disconnect, and forget requests from Aqua UID/GID 1000,
   persists an atomic root-owned `0600` PSK-only record after authoritative
   association, and rolls back a failed association. Deterministic native
   fixtures prove the known PSK vector, exact control sequence, peer
   authentication, secret redaction, and storage metadata without a radio.
   Settings renders at most the two strongest discovered rows beside explicit
   rescan and forget actions, masks transient credential input, accepts only
   WPA2-Personal selection, and permits at most two explicit association
   submissions per entry flow. The default image still
   contains neither the bridge nor `wpa_supplicant`; no daemon lifecycle is
   enabled, and WPA3 association remains outside this initial control contract.
5. **Satisfied on 2026-08-31:** deterministic fixtures prove offline,
   configuring, online, DNS loss, route loss, malformed input, bounded source
   sizes, and recovery without hanging Settings or the shell.
6. **Satisfied on 2026-08-31:** packaged QEMU proves Ethernet DHCP, default
   routing, external DNS lookup, lease renewal, forced service failure, route
   loss, bounded reconnect, and recovery-shell availability.
7. **Satisfied for the QEMU virtual-radio target on 2026-09-01:** packaged
   mac80211_hwsim proves bounded broker discovery and explicit rescan of the
   isolated WPA2 fixture, new credential association, DHCP, DNS, disconnect,
   saved reconnect, service recovery, safe saved-network forget, rejection of
   reconnect after forget, and radio disable. Physical targets still require
   their own radio, firmware, association, and failure evidence.

Physical Ethernet and Wi-Fi support remain `Not tested`; this decision and its
deterministic observation tests are not hardware evidence.

## Consequences

The existing small Buildroot architecture gains one explicit network policy
owner without enabling a large general-purpose network daemon. Aqua can present
truthful route and DNS health and can cross a narrowly authenticated control
boundary when the opt-in profile is active. The tradeoff is that advanced
network features remain deliberately unavailable until their packaging and
runtime evidence are complete.
