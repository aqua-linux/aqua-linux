# Contributor Issue Workflow

This workflow turns public reports into bounded work without overstating Aqua
Linux maturity or weakening its recovery, installer, hardware, and licensing
boundaries.

## Intake

Use one of the repository issue forms:

1. **Bug report** for reproducible behavior on a named revision and test
   environment.
2. **Feature proposal** for a problem with explicit acceptance criteria and
   architecture impact.
3. **Physical hardware observation** for sanitized, read-only evidence. This
   form never authorizes installation or establishes support.

Security vulnerabilities use GitHub private vulnerability reporting and must
not begin as public issues. Blank issues are disabled so every report includes
the minimum evidence and safety confirmations.

## Triage

Every new public issue starts with `status:needs-triage`. A maintainer reviews:

- reproducibility and revision;
- ownership area and issue type;
- recovery, installer, hardware, security, and licensing risk;
- acceptance criteria and required tests;
- secrets, personal data, and unique hardware identifiers.

Accepted work receives exactly one `area:*`, one `type:*`, one `priority:*`,
and `status:ready`. Use `status:blocked` only with a named dependency. Apply
`risk:destructive` or `risk:hardware-only` in addition to the standard labels
when those boundaries are relevant.

## Label Taxonomy

The canonical machine-readable definitions are in `.github/labels.yml`.
Changes on `main` are applied by the bounded `Sync issue labels` workflow. It
creates or updates canonical labels and never deletes repository-defined
labels outside the taxonomy.

| Prefix | Purpose | Required at ready state |
| --- | --- | --- |
| `type:*` | Bug, enhancement, or maintainer task | Exactly one |
| `status:*` | Needs triage, ready, or blocked | Exactly one |
| `priority:*` | P0 through P3 urgency | Exactly one |
| `area:*` | Primary ownership boundary | Exactly one |
| `risk:*` | Additional destructive or hardware-only review | When applicable |
| `good-first-issue` | Maintainer-scoped entry work | Optional |

P0 is reserved for data-loss, security, or recovery-path emergencies. P1 is a
release blocker in a supported development path. P2 is planned important work.
P3 is deferred improvement or cleanup.

## Pull Requests

Pull requests must reference an accepted issue unless they are narrow typo or
test-maintenance fixes. The pull request template requires the changed
contract, validation, safety impact, and evidence. A screenshot can support a
change but cannot replace tests, serial markers, or a provenance manifest.

Maintainers merge only when the relevant checks pass and the public claim is
no broader than the evidence.
