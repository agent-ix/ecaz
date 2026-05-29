#!/usr/bin/env bash
# Fail closed unless the representative SPIRE performance summaries contain
# the latency, recall, production profile, and pooling A/B evidence required
# before an AWS performance packet is accepted.

set -euo pipefail

artifact_dir="${1:?usage: verify-representative-performance-summary.sh <artifact-dir>}"

fail() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

require_file() {
  local path="$artifact_dir/$1"
  if [[ ! -s "$path" ]]; then
    fail "expected non-empty summary file at $path"
  fi
}

require_jq() {
  command -v jq >/dev/null 2>&1 || fail "jq is required"
}

require_awk() {
  local description="$1"
  local file="$2"
  local program="$3"

  if ! awk -F '\t' -v expected="$expected_nprobes" "$program" "$artifact_dir/$file"; then
    fail "$description missing or incomplete in $artifact_dir/$file"
  fi
}

latency_recall="representative-latency-recall-summary.tsv"
production_profile="representative-production-profile-summary.tsv"
pooling_comparison="representative-pooling-comparison.tsv"
pooling_delta="representative-pooling-delta-summary.tsv"
priority_suite="suite-representative-priority.json"
pooling_suite="suite-representative-pooling.json"
recall_floor_nprobe="${SPIRE_AWS_REPRESENTATIVE_RECALL_FLOOR_NPROBE:-64}"
recall_floor="${SPIRE_AWS_REPRESENTATIVE_RECALL_FLOOR:-0.95}"

require_file "$latency_recall"
require_file "$production_profile"
require_file "$pooling_comparison"
require_file "$pooling_delta"
require_file "$priority_suite"
require_file "$pooling_suite"
require_jq

jq -e --arg nprobe "$recall_floor_nprobe" --argjson floor "$recall_floor" '
  any(.thresholds[];
    .step == "13a3a-recall-k10"
    and .metric == "recall"
    and .field == "recall@k"
    and .op == "gte"
    and .value >= $floor
    and .filters.nprobe == $nprobe
  )
  and any(.thresholds[];
    .step == "13e3-production-read-profile-k10"
    and .metric == "spire-pipeline"
    and .field == "recall@k"
    and .op == "gte"
    and .value >= $floor
    and .filters.nprobe == $nprobe
  )
' "$artifact_dir/$priority_suite" >/dev/null ||
  fail "representative priority suite must gate recall@k >= $recall_floor at nprobe=$recall_floor_nprobe"

jq -e --arg nprobe "$recall_floor_nprobe" --argjson floor "$recall_floor" '
  any(.thresholds[];
    .step == "13e4-pooling-disabled-profile-k10"
    and .metric == "spire-pipeline"
    and .field == "recall@k"
    and .op == "gte"
    and .value >= $floor
    and .filters.nprobe == $nprobe
  )
  and any(.thresholds[];
    .step == "13e4-pooling-enabled-profile-k10"
    and .metric == "spire-pipeline"
    and .field == "recall@k"
    and .op == "gte"
    and .value >= $floor
    and .filters.nprobe == $nprobe
  )
' "$artifact_dir/$pooling_suite" >/dev/null ||
  fail "representative pooling suite must gate recall@k >= $recall_floor at nprobe=$recall_floor_nprobe"

latency_nprobes="$(
  jq -r '
    [
      .steps[]
      | select(.kind == "latency" and .k == 10)
      | .sweep[]
    ]
    | unique
    | sort_by(.)
    | map(tostring)
    | join(" ")
  ' "$artifact_dir/$priority_suite"
)"

recall_nprobes="$(
  jq -r '
    [
      .steps[]
      | select(.kind == "recall" and .k == 10)
      | .sweep[]
    ]
    | unique
    | sort_by(.)
    | map(tostring)
    | join(" ")
  ' "$artifact_dir/$priority_suite"
)"

pipeline_nprobes="$(
  jq -r '
    [
      .steps[]
      | select(.kind == "spire-pipeline" and .top_k == 10 and .include_recall == true)
      | .sweep[]
    ]
    | unique
    | sort_by(.)
    | map(tostring)
    | join(" ")
  ' "$artifact_dir/$priority_suite"
)"

pooling_nprobes="$(
  jq -r '
    [
      .steps[]
      | select(.kind == "spire-pipeline" and .top_k == 10 and .include_recall == true)
      | .sweep[]
    ]
    | unique
    | sort_by(.)
    | map(tostring)
    | join(" ")
  ' "$artifact_dir/$pooling_suite"
)"

[[ -n "$latency_nprobes" ]] || fail "expected latency nprobe sweep is empty in $artifact_dir/$priority_suite"
[[ -n "$recall_nprobes" ]] || fail "expected recall nprobe sweep is empty in $artifact_dir/$priority_suite"
[[ -n "$pipeline_nprobes" ]] || fail "expected production-read nprobe sweep is empty in $artifact_dir/$priority_suite"
[[ "$pipeline_nprobes" == "$pooling_nprobes" ]] ||
  fail "representative priority and pooling production-read nprobe sweeps differ: priority=[$pipeline_nprobes] pooling=[$pooling_nprobes]"

expected_nprobes="$latency_nprobes"
require_awk "representative latency p50/p95/p99 rows for all priority latency nprobe values" "$latency_recall" '
  function present(v) { return v != "" && v != "null" }
  BEGIN {
    split(expected, wanted, /[[:space:]]+/)
    for (i in wanted) {
      if (wanted[i] != "") {
        needed[wanted[i]] = 1
      }
    }
  }
  NR > 1 && $1 == "latency" && present($6) && present($7) && present($8) {
    seen[$4] = 1
  }
  END {
    for (n in needed) {
      if (!seen[n]) {
        exit 1
      }
    }
  }
'

expected_nprobes="$recall_nprobes"
require_awk "representative recall@k rows for all priority recall nprobe values" "$latency_recall" '
  function present(v) { return v != "" && v != "null" }
  BEGIN {
    split(expected, wanted, /[[:space:]]+/)
    for (i in wanted) {
      if (wanted[i] != "") {
        needed[wanted[i]] = 1
      }
    }
  }
  NR > 1 && $1 == "recall" && present($9) {
    seen[$4] = 1
  }
  END {
    for (n in needed) {
      if (!seen[n]) {
        exit 1
      }
    }
  }
'

awk -F '\t' -v nprobe="$recall_floor_nprobe" -v floor="$recall_floor" '
  function numeric(v) { return v ~ /^-?[0-9]+([.][0-9]+)?$/ }
  NR > 1 && $1 == "recall" && $4 == nprobe && numeric($9) && ($9 + 0) >= floor {
    recall = 1
  }
  NR > 1 && $1 == "spire-pipeline" && $4 == nprobe && numeric($9) && ($9 + 0) >= floor {
    pipeline = 1
  }
  END { exit(recall && pipeline ? 0 : 1) }
' "$artifact_dir/$latency_recall" ||
  fail "representative recall floor not met at nprobe=$recall_floor_nprobe floor=$recall_floor"

expected_nprobes="$pipeline_nprobes"
require_awk "production SPIRE pipeline latency and recall rows for all priority production-read nprobe values" "$latency_recall" '
  function present(v) { return v != "" && v != "null" }
  BEGIN {
    split(expected, wanted, /[[:space:]]+/)
    for (i in wanted) {
      if (wanted[i] != "") {
        needed[wanted[i]] = 1
      }
    }
  }
  NR > 1 && $1 == "spire-pipeline" && present($6) && present($7) && present($8) && present($9) {
    seen[$4] = 1
  }
  END {
    for (n in needed) {
      if (!seen[n]) {
        exit 1
      }
    }
  }
'

require_awk "production read profile rows for all priority production-read nprobe values" "$production_profile" '
  function present(v) { return v != "" && v != "null" }
  BEGIN {
    split(expected, wanted, /[[:space:]]+/)
    for (i in wanted) {
      if (wanted[i] != "") {
        needed[wanted[i]] = 1
      }
    }
  }
  NR > 1 && present($4) && present($5) && present($6) && present($7) && present($9) && present($11) && present($19) && present($22) {
    seen[$2] = 1
  }
  END {
    for (n in needed) {
      if (!seen[n]) {
        exit 1
      }
    }
  }
'

require_awk "pooled and unpooled comparison rows" "$pooling_comparison" '
  function present(v) { return v != "" && v != "null" }
  NR > 1 && ($1 == "disabled" || $1 == "enabled") {
    if (!(present($8) && present($15) && present($16) && present($17) && present($18) && present($19))) {
      bad = 1
    }
    if ($1 == "disabled") {
      disabled = 1
    } else if ($1 == "enabled") {
      enabled = 1
    }
  }
  END { exit(disabled && enabled && !bad ? 0 : 1) }
'

expected_nprobes="$pooling_nprobes"
require_awk "pooling delta improvement row" "$pooling_delta" '
  function present(v) { return v != "" && v != "null" }
  function numeric(v) { return v ~ /^-?[0-9]+([.][0-9]+)?$/ }
  function abs(v) { return v < 0 ? -v : v }
  BEGIN {
    split(expected, wanted, /[[:space:]]+/)
    for (i in wanted) {
      if (wanted[i] != "") {
        needed[wanted[i]] = 1
      }
    }
  }
  NR > 1 &&
    present($6) && present($8) && present($9) &&
    present($21) && present($25) && present($29) && present($33) &&
    numeric($6) && numeric($8) && numeric($9) &&
    numeric($21) && numeric($25) && numeric($29) && numeric($33) {
    if (($6 + 0) > 0 &&
        ($8 + 0) > 0 &&
        ($9 + 0) > 0 &&
        ($21 + 0) > 0 &&
        ($25 + 0) > 0 &&
        ($29 + 0) > 0 &&
        abs($33 + 0) <= 0.000001) {
      seen[$1] = 1
    }
  }
  END {
    for (n in needed) {
      if (!seen[n]) {
        exit 1
      }
    }
  }
'

awk -F '\t' -v nprobe="$recall_floor_nprobe" -v floor="$recall_floor" '
  function numeric(v) { return v ~ /^-?[0-9]+([.][0-9]+)?$/ }
  NR > 1 && $1 == nprobe &&
    numeric($31) && numeric($32) &&
    ($31 + 0) >= floor && ($32 + 0) >= floor {
    passed = 1
  }
  END { exit(passed ? 0 : 1) }
' "$artifact_dir/$pooling_delta" ||
  fail "representative pooling recall floor not met at nprobe=$recall_floor_nprobe floor=$recall_floor"

printf 'representative performance summary verified: %s nprobes=[%s]\n' \
  "$artifact_dir" \
  "latency:$latency_nprobes recall:$recall_nprobes production:$pipeline_nprobes pooling:$pooling_nprobes"
