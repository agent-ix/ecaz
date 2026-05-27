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

run_summary_gate_self_check() {
  local parent_dir="${SPIRE_AWS_REPRESENTATIVE_PREFLIGHT_WORKDIR:-$repo_root/target}"
  local work_dir sample_input sample_output bad_summary

  mkdir -p "$parent_dir"
  work_dir="$(mktemp -d "$parent_dir/spire-representative-preflight.XXXXXX")"
  trap 'rm -rf "$work_dir"; trap - RETURN' RETURN

  sample_input="$work_dir/sample-input"
  sample_output="$work_dir/sample-output"
  bad_summary="$work_dir/bad-summary"
  mkdir -p "$sample_input" "$sample_output" "$bad_summary"

  cat > "$sample_input/suite-results-representative-priority.jsonl" <<'JSONL'
{"kind":"latency","metric":"latency","step":"13a3a-latency-k10-c1","values":{"nprobe":8,"count":1000,"p50":"10.0 ms","p95":"20.0 ms","p99":"30.0 ms"}}
{"kind":"latency","metric":"latency","step":"13a3a-latency-k10-c1","values":{"nprobe":16,"count":1000,"p50":"10.0 ms","p95":"20.0 ms","p99":"30.0 ms"}}
{"kind":"latency","metric":"latency","step":"13a3a-latency-k10-c1","values":{"nprobe":24,"count":1000,"p50":"10.0 ms","p95":"20.0 ms","p99":"30.0 ms"}}
{"kind":"latency","metric":"latency","step":"13a3a-latency-k10-c1","values":{"nprobe":32,"count":1000,"p50":"10.0 ms","p95":"20.0 ms","p99":"30.0 ms"}}
{"kind":"recall","metric":"recall","step":"13a3a-recall-k10","values":{"nprobe":8,"queries":1000,"recall@k":0.99}}
{"kind":"recall","metric":"recall","step":"13a3a-recall-k10","values":{"nprobe":16,"queries":1000,"recall@k":0.99}}
{"kind":"recall","metric":"recall","step":"13a3a-recall-k10","values":{"nprobe":24,"queries":1000,"recall@k":0.99}}
{"kind":"recall","metric":"recall","step":"13a3a-recall-k10","values":{"nprobe":32,"queries":1000,"recall@k":0.99}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e3-production-read-profile-k10","values":{"nprobe":8,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"22.0 ms","latency_p99":"33.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":10,"connect_p50":"0.1 ms","connect_p95":"0.3 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","candidate_p50":"5.0 ms","candidate_p95":"8.0 ms","heap_p50":"1.0 ms","heap_p95":"2.0 ms","merge_p50":"0.5 ms","merge_p95":"1.0 ms","total_p50":"11.0 ms","total_p95":"22.0 ms","candidate_query_sum":1000,"heap_query_sum":1000,"endpoint_identity_query_sum":1000,"payload_bytes_sum":4096,"timeout_sum":0,"cancel_sum":0,"degraded_skip_sum":0,"returned_sum":10000}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e3-production-read-profile-k10","values":{"nprobe":16,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"22.0 ms","latency_p99":"33.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":10,"connect_p50":"0.1 ms","connect_p95":"0.3 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","candidate_p50":"5.0 ms","candidate_p95":"8.0 ms","heap_p50":"1.0 ms","heap_p95":"2.0 ms","merge_p50":"0.5 ms","merge_p95":"1.0 ms","total_p50":"11.0 ms","total_p95":"22.0 ms","candidate_query_sum":1000,"heap_query_sum":1000,"endpoint_identity_query_sum":1000,"payload_bytes_sum":4096,"timeout_sum":0,"cancel_sum":0,"degraded_skip_sum":0,"returned_sum":10000}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e3-production-read-profile-k10","values":{"nprobe":24,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"22.0 ms","latency_p99":"33.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":10,"connect_p50":"0.1 ms","connect_p95":"0.3 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","candidate_p50":"5.0 ms","candidate_p95":"8.0 ms","heap_p50":"1.0 ms","heap_p95":"2.0 ms","merge_p50":"0.5 ms","merge_p95":"1.0 ms","total_p50":"11.0 ms","total_p95":"22.0 ms","candidate_query_sum":1000,"heap_query_sum":1000,"endpoint_identity_query_sum":1000,"payload_bytes_sum":4096,"timeout_sum":0,"cancel_sum":0,"degraded_skip_sum":0,"returned_sum":10000}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e3-production-read-profile-k10","values":{"nprobe":32,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"22.0 ms","latency_p99":"33.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":10,"connect_p50":"0.1 ms","connect_p95":"0.3 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","candidate_p50":"5.0 ms","candidate_p95":"8.0 ms","heap_p50":"1.0 ms","heap_p95":"2.0 ms","merge_p50":"0.5 ms","merge_p95":"1.0 ms","total_p50":"11.0 ms","total_p95":"22.0 ms","candidate_query_sum":1000,"heap_query_sum":1000,"endpoint_identity_query_sum":1000,"payload_bytes_sum":4096,"timeout_sum":0,"cancel_sum":0,"degraded_skip_sum":0,"returned_sum":10000}}
JSONL

  cat > "$sample_input/suite-results-representative-pooling.jsonl" <<'JSONL'
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-disabled-profile-k10","values":{"nprobe":8,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"12.0 ms","latency_p95":"24.0 ms","latency_p99":"36.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":1000,"connect_p50":"0.2 ms","connect_p95":"1.0 ms","endpoint_identity_p50":"0.2 ms","endpoint_identity_p95":"0.5 ms","endpoint_identity_query_sum":1000,"total_p50":"12.0 ms","total_p95":"24.0 ms"}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-enabled-profile-k10","values":{"nprobe":8,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"20.0 ms","latency_p99":"30.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":4,"connect_p50":"0.1 ms","connect_p95":"0.2 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","endpoint_identity_query_sum":4,"total_p50":"11.0 ms","total_p95":"20.0 ms"}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-disabled-profile-k10","values":{"nprobe":16,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"12.0 ms","latency_p95":"24.0 ms","latency_p99":"36.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":1000,"connect_p50":"0.2 ms","connect_p95":"1.0 ms","endpoint_identity_p50":"0.2 ms","endpoint_identity_p95":"0.5 ms","endpoint_identity_query_sum":1000,"total_p50":"12.0 ms","total_p95":"24.0 ms"}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-enabled-profile-k10","values":{"nprobe":16,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"20.0 ms","latency_p99":"30.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":4,"connect_p50":"0.1 ms","connect_p95":"0.2 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","endpoint_identity_query_sum":4,"total_p50":"11.0 ms","total_p95":"20.0 ms"}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-disabled-profile-k10","values":{"nprobe":24,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"12.0 ms","latency_p95":"24.0 ms","latency_p99":"36.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":1000,"connect_p50":"0.2 ms","connect_p95":"1.0 ms","endpoint_identity_p50":"0.2 ms","endpoint_identity_p95":"0.5 ms","endpoint_identity_query_sum":1000,"total_p50":"12.0 ms","total_p95":"24.0 ms"}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-enabled-profile-k10","values":{"nprobe":24,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"20.0 ms","latency_p99":"30.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":4,"connect_p50":"0.1 ms","connect_p95":"0.2 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","endpoint_identity_query_sum":4,"total_p50":"11.0 ms","total_p95":"20.0 ms"}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-disabled-profile-k10","values":{"nprobe":32,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"12.0 ms","latency_p95":"24.0 ms","latency_p99":"36.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":1000,"connect_p50":"0.2 ms","connect_p95":"1.0 ms","endpoint_identity_p50":"0.2 ms","endpoint_identity_p95":"0.5 ms","endpoint_identity_query_sum":1000,"total_p50":"12.0 ms","total_p95":"24.0 ms"}}
{"kind":"spire-pipeline","metric":"spire-pipeline","step":"13e4-pooling-enabled-profile-k10","values":{"nprobe":32,"queries":1000,"profiles":1000,"status":"ok","result_source":"remote","latency_p50":"11.0 ms","latency_p95":"20.0 ms","latency_p99":"30.0 ms","recall@k":0.99,"dispatch_sum":1000,"socket_open_sum":4,"connect_p50":"0.1 ms","connect_p95":"0.2 ms","endpoint_identity_p50":"0.1 ms","endpoint_identity_p95":"0.2 ms","endpoint_identity_query_sum":4,"total_p50":"11.0 ms","total_p95":"20.0 ms"}}
JSONL

  "$summarizer" "$sample_input" "$sample_output" >/dev/null
  "$verifier" "$sample_output" >/dev/null

  cp "$sample_output"/representative-*.tsv "$bad_summary"/
  awk 'BEGIN{FS=OFS="\t"} NR==1 {print; next} {$29=0; print}' \
    "$sample_output/representative-pooling-delta-summary.tsv" \
    > "$bad_summary/representative-pooling-delta-summary.tsv"

  if "$verifier" "$bad_summary" >/dev/null 2>&1; then
    die "representative summary verifier accepted missing p99 pooling latency improvement"
  fi
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
run_summary_gate_self_check

printf 'SPIRE representative performance preflight passed: priority=%s pooling=%s\n' \
  "$priority_suite" \
  "$pooling_suite"
