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

require_file "$latency_recall"
require_file "$production_profile"
require_file "$pooling_comparison"
require_file "$pooling_delta"
require_file "$priority_suite"
require_file "$pooling_suite"
require_jq

expected_nprobes="$(
  jq -r '
    [
      .steps[]
      | select(
          (.kind == "latency" and .k == 10)
          or (.kind == "recall" and .k == 10)
          or (.kind == "spire-pipeline" and .top_k == 10 and .include_recall == true)
        )
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

[[ -n "$expected_nprobes" ]] || fail "expected nprobe sweep is empty in $artifact_dir/$priority_suite"
[[ "$expected_nprobes" == "$pooling_nprobes" ]] ||
  fail "representative priority and pooling nprobe sweeps differ: priority=[$expected_nprobes] pooling=[$pooling_nprobes]"

require_awk "representative latency p50/p95/p99 rows for all priority nprobe values" "$latency_recall" '
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

require_awk "representative recall@k rows for all priority nprobe values" "$latency_recall" '
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

require_awk "production SPIRE pipeline latency and recall rows for all priority nprobe values" "$latency_recall" '
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

require_awk "production read profile rows for all priority nprobe values" "$production_profile" '
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

printf 'representative performance summary verified: %s\n' "$artifact_dir"
