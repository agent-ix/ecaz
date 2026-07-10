#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

spec_files=(
  spec/functional/index/distann/FR-075-ec-distann-access-method-surface.md
  spec/functional/index/distann/FR-076-distann-graph-node-record-format.md
  spec/functional/index/distann/FR-077-distann-sharded-build-and-stitch.md
  spec/functional/index/distann/FR-078-distann-hash-placement.md
  spec/functional/index/distann/FR-079-distann-remote-expansion-protocol.md
  spec/functional/index/distann/FR-080-distann-coordinator-head-index.md
  spec/functional/index/distann/FR-081-distann-query-orchestration.md
  spec/functional/index/distann/FR-082-distann-epoch-lifecycle.md
  spec/functional/index/distann/FR-083-distann-dml-path.md
  spec/non-functional/NFR-014-spire-transport-security-and-operations.md
  spec/non-functional/NFR-016-on-disk-format-evolution-discipline.md
  spec/non-functional/NFR-017-distann-latency-recall-gate.md
  spec/non-functional/NFR-018-distann-space-amplification.md
  spec/non-functional/NFR-019-distann-per-query-touch-bound.md
  spec/non-functional/NFR-020-distann-fault-behavior.md
)

spec_errors="$(mktemp)"
matrix_errors="$(mktemp)"
summary_ids="$(mktemp)"
cleanup() {
  rm -f "${spec_errors}" "${matrix_errors}" "${summary_ids}"
}
trap cleanup EXIT

rg -o --no-filename 'EC_[A-Z0-9_]+' "${spec_files[@]}" | sort -u >"${spec_errors}"
rg -o --no-filename 'EC_[A-Z0-9_]+' spec/tests.md | sort -u >"${matrix_errors}"
missing_count="$(comm -23 "${spec_errors}" "${matrix_errors}" | wc -l | tr -d ' ')"

awk '
  /^## Test Case Summary$/ { in_summary = 1; next }
  in_summary && /^## / { exit }
  in_summary && /^\| TC-[0-9][0-9][0-9] / {
    split($0, columns, "|")
    gsub(/[[:space:]]/, "", columns[2])
    print columns[2]
  }
' spec/tests.md >"${summary_ids}"
duplicate_count="$(sort "${summary_ids}" | uniq -d | wc -l | tr -d ' ')"

require_pattern() {
  local pattern="$1"
  local file="$2"
  local label="$3"
  local value="$4"
  if rg -q "${pattern}" "${file}"; then
    printf '%s=%s\n' "${label}" "${value}"
  else
    printf '%s=fail\n' "${label}"
    return 1
  fi
}

printf 'head=%s\n' "$(git rev-parse HEAD)"
printf 'timestamp=%s\n' "$(date --iso-8601=seconds)"
printf 'stable_error_categories_missing_from_matrix=%s\n' "${missing_count}"
if [[ "${missing_count}" != "0" ]]; then
  printf 'missing_error_categories=%s\n' "$(comm -23 "${spec_errors}" "${matrix_errors}" | paste -sd, -)"
fi
printf 'duplicate_test_summary_ids=%s\n' "${duplicate_count}"
require_pattern '^\| TC-020 SPIRE \|' spec/tests.md tc_020_owner SPIRE
require_pattern '^\| TC-049 \|.*bench suite' spec/tests.md tc_049_owner benchmark_suite
require_pattern 'TC-045\.\.TC-048.*reserved|TC-045\.\.TC-048.*Reserved' spec/tests.md tc_045_through_tc_048_owner Task_173_reserved
require_pattern '^\| TC-050 \| DistANN persisted/wire-format discipline' spec/tests.md tc_050_owner DistANN_format_discipline

if [[ -f plan/tasks/179-ec-distann-physical-hash-shard-generations.md ]] \
  && rg -q '179-ec-distann-physical-hash-shard-generations\.md' plan/tasks/README.md; then
  printf 'task_179_plan_link=pass\n'
else
  printf 'task_179_plan_link=fail\n'
  exit 1
fi

if git diff --check -- spec plan; then
  printf 'git_diff_check=pass\n'
else
  printf 'git_diff_check=fail\n'
  exit 1
fi

printf 'spec_matrix_status=PARTIAL\n'
printf 'reason=physical_hash_shard_implementation_and_runtime_evidence_do_not_exist_yet\n'

if [[ "${missing_count}" != "0" || "${duplicate_count}" != "0" ]]; then
  exit 1
fi
