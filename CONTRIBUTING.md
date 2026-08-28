# Contributing To Aqua Linux

Aqua Linux is an early operating-system project. Contributions must preserve
its architecture and recovery guarantees.

## Project Boundaries

- Keep Buildroot as the OS base.
- Keep the custom Smithay Wayland compositor as the graphics architecture.
- Do not add a full desktop environment or turn Aqua into a theme pack.
- Keep QEMU x86_64 as the first reproducible development target.
- Preserve the text recovery shell and explicit graphical-session gates.
- Treat private interface boards as local references, not public or runtime
  asset sources.

Substantial architecture changes should begin with an ADR under
`docs/aqua-linux/`.

## Issues And Triage

Use the structured GitHub forms for bugs, feature proposals, and physical
hardware observations. Blank issues are disabled. Security vulnerabilities
must use private vulnerability reporting instead of a public issue.

The complete intake, triage, priority, area, risk, and ready-state rules are in
[contributor-workflow.md](docs/aqua-linux/contributor-workflow.md). Label names
and descriptions are canonical in `.github/labels.yml`.

## Development Setup

Install Rust stable, Docker or a Linux Buildroot toolchain, QEMU x86_64,
Python 3, `expect`, and `jq`.

Run source checks before submitting:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
scripts/check.sh
scripts/check-public-repo.sh
scripts/check-contributor-workflow.sh
```

Buildroot and QEMU changes should also run the narrowest relevant image check.
State clearly when a test was not run and why.

## Change Scope

- Prefer focused pull requests.
- Add or update tests with behavior changes.
- Do not commit `build/`, `target/`, disk images, serial logs, screenshots,
  caches, local environment files, or the separately maintained `website/`
  working tree.
- Do not bypass installer target validation or destructive confirmation.
- Do not remove recovery behavior to make a graphical demo appear successful.
- Keep runtime values real or explicitly label them as mock in design-only
  output.

## Assets And Licensing

Every new dependency or asset must include its source/provenance, license,
required notice, and runtime or test-only role.

Do not submit copied desktop assets, extracted reference-board icons, or files
with uncertain redistribution rights. Update `THIRD_PARTY_LICENSES.md` and the
asset manifest when applicable.

## Commit And Pull Request Notes

Use imperative commit subjects and explain the problem, changed contract,
validation performed, and recovery/installer/hardware/licensing impact.
Screenshots may support a change but cannot replace automated or serial-log
evidence.

Create focused branches from current `main`. Pull requests should reference an
accepted issue except for narrow typo or test-maintenance fixes, complete the
repository pull request checklist, and avoid combining unrelated milestones.
