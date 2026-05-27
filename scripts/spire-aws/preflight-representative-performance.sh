#!/usr/bin/env bash
# Local-only readiness checks for the Phase 13e representative performance pass.

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"
priority_suite="${SPIRE_AWS_REPRESENTATIVE_PRIORITY_SUITE:-$script_dir/suite-representative-priority.json}"
pooling_suite="${SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE:-$script_dir/suite-representative-pooling.json}"
makefile="${SPIRE_AWS_REPRESENTATIVE_MAKEFILE:-$repo_root/infra/spire-aws/Makefile}"
summarizer="$script_dir/summarize-representative-performance.sh"
verifier="$script_dir/verify-representative-performance-summary.sh"

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

require_file() {
  local path="$1"
  [[ -s "$path" ]] || die "expected non-empty file: $path"
}

require_executable() {
  local path="$1"
  [[ -x "$path" ]] || die "expected executable script: $path"
}

require_jq() {
  local description="$1"
  local file="$2"
  local filter="$3"

  jq -e "$filter" "$file" >/dev/null ||
    die "$description failed in $file"
}

require_make_target_contains() {
  local target="$1"
  local expected="$2"
  awk -v target="$target" -v expected="$expected" '
    $0 ~ "^[A-Za-z0-9_.-]+:" {
      in_target = ($0 ~ "^" target ":")
    }
    in_target && index($0, expected) { found = 1 }
    END { exit(found ? 0 : 1) }
  ' "$makefile" ||
    die "Makefile target $target must contain $expected"
}

require_make_target_absent() {
  local target="$1"
  local forbidden="$2"
  awk -v target="$target" -v forbidden="$forbidden" '
    $0 ~ "^[A-Za-z0-9_.-]+:" {
      in_target = ($0 ~ "^" target ":")
    }
    in_target && index($0, forbidden) { found = 1 }
    END { exit(found ? 1 : 0) }
  ' "$makefile" ||
    die "Makefile target $target must not contain $forbidden"
}

require_make_target_order() {
  local target="$1"
  local before="$2"
  local after="$3"
  awk -v target="$target" -v before="$before" -v after="$after" '
    $0 ~ "^[A-Za-z0-9_.-]+:" {
      in_target = ($0 ~ "^" target ":")
    }
    in_target && index($0, before) && before_line == 0 {
      before_line = NR
    }
    in_target && index($0, after) && after_line == 0 {
      after_line = NR
    }
    END { exit(before_line > 0 && after_line > 0 && before_line < after_line ? 0 : 1) }
  ' "$makefile" ||
    die "Makefile target $target must run $before before $after"
}

require_file "$priority_suite"
require_file "$pooling_suite"
require_file "$makefile"
require_executable "$summarizer"
require_executable "$verifier"

jq empty "$priority_suite" "$pooling_suite" >/dev/null
bash -n "$summarizer" "$verifier"

require_jq "representative priority suite recall coverage" "$priority_suite" '
  [.steps[] | select(.kind == "recall" and .k == 10 and (.queries_limit // 0) >= 1000)] | length >= 1
'
require_jq "representative priority suite latency coverage" "$priority_suite" '
  [.steps[] | select(.kind == "latency" and .k == 10 and (.concurrency // 0) >= 1 and (.iterations // 0) >= 1000)] | length >= 1
'
require_jq "representative priority suite production profile coverage" "$priority_suite" '
  [.steps[]
   | select(.kind == "spire-pipeline"
     and .top_k == 10
     and .include_remote == true
     and .require_remote_placements == true
     and .include_query_metrics == true
     and .include_recall == true
     and .include_production_read_profile == true
     and .production_read_only == true
     and (.queries_limit // 0) >= 1000)] | length >= 1
'
require_jq "representative pooling suite disabled profile" "$pooling_suite" '
  [.steps[]
   | select(.kind == "spire-pipeline"
     and (.pgoptions | test("ec_spire.remote_search_connection_pool_size=0"))
     and .include_remote == true
     and .require_remote_placements == true
     and .include_query_metrics == true
     and .include_recall == true
     and .include_production_read_profile == true
     and .production_read_only == true
     and (.queries_limit // 0) >= 1000)] | length >= 1
'
require_jq "representative pooling suite enabled profile" "$pooling_suite" '
  [.steps[]
   | select(.kind == "spire-pipeline"
     and (.pgoptions | test("ec_spire.remote_search_connection_pool_size=[1-9][0-9]*"))
     and .include_remote == true
     and .require_remote_placements == true
     and .include_query_metrics == true
     and .include_recall == true
     and .include_production_read_profile == true
     and .production_read_only == true
     and (.queries_limit // 0) >= 1000)] | length >= 1
'

require_make_target_contains "verify-representative-performance-tunneled" "bench-representative-priority"
require_make_target_contains "verify-representative-performance-tunneled" "bench-representative-pooling"
require_make_target_contains "verify-representative-performance-tunneled" "summarize-representative-performance"
require_make_target_contains "verify-representative-performance-tunneled" "verify-representative-performance-summary"
require_make_target_absent "verify-representative-performance-tunneled" "fault-"
require_make_target_order "pass-representative-performance-body" "preflight-representative-performance" "provision"

printf 'SPIRE representative performance preflight passed: priority=%s pooling=%s\n' \
  "$priority_suite" \
  "$pooling_suite"
