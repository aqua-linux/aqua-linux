#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
RUNNER="${ROOT_DIR}/scripts/check-r2-presentation-qemu.sh"
VALIDATOR="${ROOT_DIR}/scripts/check-r2-presentation-log.py"
EVIDENCE_DIR="${EVIDENCE_DIR:-${ROOT_DIR}/build/qemu-r2-presentation-runs}"
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
    echo "R2 evidence directory already exists; choose a new EVIDENCE_DIR." >&2
    exit 1
fi

mkdir -p "$(dirname "${EVIDENCE_DIR}")"
mkdir "${EVIDENCE_DIR}"

run=1
while test "${run}" -le "${RUNS}"; do
    run_id="$(printf '%02d' "${run}")"
    SERIAL_LOG="${EVIDENCE_DIR}/run-${run_id}.log" \
    MONITOR_SOCKET="${EVIDENCE_DIR}/run-${run_id}.monitor.sock" \
    REPORT="${EVIDENCE_DIR}/run-${run_id}.txt" \
        "${RUNNER}"
    run=$((run + 1))
done

PYTHONDONTWRITEBYTECODE=1 python3 "${VALIDATOR}" --summarize-repeated \
    "${EVIDENCE_DIR}"/run-*.log >"${EVIDENCE_DIR}/review.txt"

grep -Fq "r2_review_qemu_runs=${RUNS}" "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_review_minimum_runs_met=true' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_review_diagnostic_isolation_recorded=true' "${EVIDENCE_DIR}/review.txt"
grep -Fq 'r2_review_budget_selected=false' "${EVIDENCE_DIR}/review.txt"

echo "Aqua Linux repeated R2 presentation collection passed."
echo "Evidence directory: ${EVIDENCE_DIR}"
echo "Review summary: ${EVIDENCE_DIR}/review.txt"
