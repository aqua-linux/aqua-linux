# Security Policy

## Supported Versions

Aqua Linux is a pre-release development project. No version is currently
supported for production or daily-use security updates.

## Reporting A Vulnerability

Do not publish exploitable installer, privilege, boot, filesystem, or session
issues in a public issue before maintainers have a chance to assess them.

Use GitHub's private vulnerability reporting feature for the repository. Include
the affected commit, reproduction steps, impact, and whether the issue can
affect a host disk or only a disposable QEMU target.

Do not test destructive installer paths against physical disks. Aqua's real
installer execution path is intended for disposable QEMU targets until hardware
validation is explicitly opened.

## Current Security Limitations

- The project is not hardened or independently audited.
- The default development account and recovery paths are not production policy.
- Secure Boot, measured boot, encrypted storage, sandboxing, signed updates,
  and production authentication are not complete.
- QEMU validation does not establish real-hardware security.

These limitations must remain visible in release notes until resolved.
