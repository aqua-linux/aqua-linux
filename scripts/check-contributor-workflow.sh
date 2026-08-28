#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "${ROOT_DIR}"

need_file() {
    test -s "$1" || {
        echo "Missing contributor workflow file: $1" >&2
        exit 1
    }
}

need_text() {
    grep -Fq "$2" "$1" || {
        echo "Missing contributor workflow contract in $1: $2" >&2
        exit 1
    }
}

need_file .github/ISSUE_TEMPLATE/config.yml
need_file .github/ISSUE_TEMPLATE/bug_report.yml
need_file .github/ISSUE_TEMPLATE/feature_request.yml
need_file .github/ISSUE_TEMPLATE/hardware_observation.yml
need_file .github/PULL_REQUEST_TEMPLATE.md
need_file .github/labels.yml
need_file .github/workflows/sync-labels.yml
need_file docs/aqua-linux/contributor-workflow.md

need_text .github/ISSUE_TEMPLATE/config.yml "blank_issues_enabled: false"
need_text .github/ISSUE_TEMPLATE/config.yml "/security/advisories/new"
need_text .github/ISSUE_TEMPLATE/bug_report.yml "status:needs-triage"
need_text .github/ISSUE_TEMPLATE/bug_report.yml "I did not run a destructive installer path against a physical disk."
need_text .github/ISSUE_TEMPLATE/feature_request.yml "custom Smithay compositor"
need_text .github/ISSUE_TEMPLATE/hardware_observation.yml "does not authorize installation or establish hardware support"
need_text .github/PULL_REQUEST_TEMPLATE.md "Recovery behavior remains available."
need_text .github/workflows/sync-labels.yml "issues: write"
need_text .github/workflows/sync-labels.yml "without deleting repository-defined labels"
need_text docs/aqua-linux/contributor-workflow.md 'exactly one `area:*`'

for label in \
    type:bug type:enhancement type:task \
    status:needs-triage status:ready status:blocked \
    priority:p0 priority:p1 priority:p2 priority:p3 \
    area:boot area:compositor area:shell area:installer area:hardware area:build area:docs area:assets \
    risk:destructive risk:hardware-only good-first-issue; do
    count="$(grep -Fxc -- "- name: ${label}" .github/labels.yml)"
    test "${count}" -eq 1 || {
        echo "Label must be defined exactly once: ${label}" >&2
        exit 1
    }
done

if grep -RIniE 'serial( number)?[[:space:]]*[:=][[:space:]]*[^<{]' .github/ISSUE_TEMPLATE; then
    echo "Issue template appears to request a hardware serial number" >&2
    exit 1
fi

echo "Aqua Linux contributor workflow checks passed."
