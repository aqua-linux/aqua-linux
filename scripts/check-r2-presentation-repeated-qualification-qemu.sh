#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
RUNNER="${ROOT_DIR}/scripts/check-r2-presentation-qualification-qemu.sh"
VALIDATOR="${ROOT_DIR}/scripts/check-r2-presentation-log.py"
EVIDENCE_DIR="${EVIDENCE_DIR:-${ROOT_DIR}/build/qemu-r2-presentation-qualification-runs}"
RUNS="${RUNS:-3}"

case "${RUNS}" in
    ''|*[!0-9]*)
        echo "RUNS must be an integer from 3 through 10." >&2
        exit 1
        ;;
esac
if test "${RUNS}" -lt 3 || test "${RUNS}" -gt 10; then
    echo "RUNS must be an integer from 3 through 10." >&2
    exit 1
fi
if test -e "${EVIDENCE_DIR}"; then
    echo "R2 qualification evidence directory already exists; choose a new EVIDENCE_DIR." >&2
    exit 1
fi

mkdir -p "$(dirname "${EVIDENCE_DIR}")"
mkdir "${EVIDENCE_DIR}"

run=1
while test "${run}" -le "${RUNS}"; do
    run_id="$(printf '%02d' "${run}")"
    EVIDENCE_DIR="${EVIDENCE_DIR}/run-${run_id}" "${RUNNER}"
    run=$((run + 1))
done

PYTHONDONTWRITEBYTECODE=1 python3 "${VALIDATOR}" \
    --summarize-repeated-qualification \
    "${EVIDENCE_DIR}"/run-*/serial.log >"${EVIDENCE_DIR}/review.txt"

grep -Fq "r2_qualification_review_qemu_runs=${RUNS}" "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_qualification_review_minimum_runs_met=true' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_qualification_review_crashes=0' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_qualification_review_client_lifecycle_complete=true' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_qualification_review_diagnostic_isolation_recorded=true' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_qualification_review_budget_profile=qemu-tcg-bochs-qualification-v1' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_qualification_review_physical_evidence=false' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_qualification_review_release_ready=false' "${EVIDENCE_DIR}/review.txt"

echo "Aqua Linux repeated R2 presentation qualification collection passed."
echo "Evidence directory: ${EVIDENCE_DIR}"
echo "Review summary: ${EVIDENCE_DIR}/review.txt"
