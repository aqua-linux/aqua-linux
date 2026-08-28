## Problem

Describe the concrete problem and its current observable behavior.

## Change

Describe the changed contract and keep unrelated work out of this pull request.

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `scripts/check.sh`
- [ ] `scripts/check-public-repo.sh`
- [ ] Relevant Buildroot or QEMU validation was run, or the reason it was not run is stated below.

## Safety And Scope

- [ ] Recovery behavior remains available.
- [ ] Installer target validation and destructive confirmation were not weakened.
- [ ] Physical hardware support is not inferred from QEMU or an unreviewed observation.
- [ ] New dependencies and assets have documented provenance and licenses.
- [ ] Logs, screenshots, and fixtures contain no secrets or unique hardware identifiers.

## Evidence

List commands, serial markers, manifests, and provenance-recorded captures. Screenshots do not replace automated evidence.
