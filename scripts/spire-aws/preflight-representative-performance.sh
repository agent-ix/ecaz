#!/usr/bin/env bash
# Local-only readiness checks for the Phase 13e representative performance pass.

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"
priority_suite="${SPIRE_AWS_REPRESENTATIVE_PRIORITY_SUITE:-$script_dir/suite-representative-priority.json}"
pooling_suite="${SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE:-$script_dir/suite-representative-pooling.json}"
makefile="${SPIRE_AWS_REPRESENTATIVE_MAKEFILE:-$repo_root/infra/spire-aws/Makefile}"
watchdog="${SPIRE_AWS_REPRESENTATIVE_WATCHDOG:-$script_dir/run-pass-with-watchdog.sh}"
summarizer="$script_dir/summarize-representative-performance.sh"
verifier="$script_dir/verify-representative-performance-summary.sh"
representative_pass_entrypoint="$script_dir/run-representative-performance-pass.sh"

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

require_representative_artifact_dir() {
  local artifact_dir="$1"
  local absolute_artifact_dir existing_artifact

  [[ -n "$artifact_dir" ]] || die "ARTIFACT_DIR is empty"
  case "$artifact_dir" in
    /*)
      absolute_artifact_dir="$artifact_dir"
      ;;
    *)
      absolute_artifact_dir="$repo_root/$artifact_dir"
      ;;
  esac
  absolute_artifact_dir="$(realpath -m -- "$absolute_artifact_dir")"

  case "$absolute_artifact_dir" in
    "$repo_root"/reviews/task-30/*/artifacts)
      ;;
    *)
      die "ARTIFACT_DIR must be packet-local under reviews/task-30/<packet>/artifacts: $artifact_dir"
      ;;
  esac

  if [[ "$absolute_artifact_dir" == "$repo_root/reviews/task-30/957-spire-aws-verification/artifacts" ]]; then
    die "ARTIFACT_DIR must not use the legacy default packet: $absolute_artifact_dir"
  fi

  if [[ "${SPIRE_AWS_REUSE_ARTIFACT_DIR:-0}" == "1" ]]; then
    return
  fi

  if [[ -d "$absolute_artifact_dir" ]]; then
    existing_artifact="$(
      find "$absolute_artifact_dir" -maxdepth 1 \
        \( \
          -name 'aws-topology*.json' -o \
          -name 'suite-results-representative*.jsonl' -o \
          -name 'suite-manifest-representative*.json' -o \
          -name 'suite-representative*.json' -o \
          -name 'representative-*.tsv' -o \
          -name '.representative-performance-pass.started' \
        \) \
        -print -quit
    )"
    [[ -z "$existing_artifact" ]] ||
      die "ARTIFACT_DIR already contains representative run output: $existing_artifact"
  fi
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

require_make_target_sequence() {
  local target="$1"
  shift
  local expected

  expected="$(printf '%s\n' "$@")"
  awk -v target="$target" -v expected="$expected" '
    BEGIN {
      expected_count = split(expected, wanted, "\n")
    }
    $0 ~ "^[A-Za-z0-9_.-]+:" {
      if (in_target && $0 !~ "^" target ":") {
        in_target = 0
      }
      if ($0 ~ "^" target ":") {
        in_target = 1
      }
    }
    in_target {
      body = body "\n" $0
    }
    END {
      cursor = 1
      for (i = 1; i <= expected_count; i++) {
        if (wanted[i] == "") {
          continue
        }
        found = index(substr(body, cursor), wanted[i])
        if (!found) {
          exit 1
        }
        cursor += found + length(wanted[i]) - 1
      }
    }
  ' "$makefile" ||
    die "Makefile target $target must run sequence: $*"
}

require_watchdog_timeout() {
  local target="$1"
  local minimum_seconds="$2"

  awk -v target="$target" -v minimum="$minimum_seconds" '
    $0 ~ "^[[:space:]]*" target "\\)" {
      in_target = 1
      next
    }
    in_target && /^[[:space:]]*[A-Za-z0-9_-]+[-A-Za-z0-9_]*\)/ {
      in_target = 0
    }
    in_target && /default_timeout=/ {
      split($0, parts, "=")
      value = parts[2] + 0
      if (value >= minimum) {
        found = 1
      }
    }
    END { exit(found ? 0 : 1) }
  ' "$watchdog" ||
    die "watchdog target $target must default to at least ${minimum_seconds}s"
}

run_summary_gate_self_check() {
  local parent_dir="${SPIRE_AWS_REPRESENTATIVE_PREFLIGHT_WORKDIR:-$repo_root/target}"
  local work_dir sample_input sample_output bad_summary bad_recall_summary

  mkdir -p "$parent_dir"
  work_dir="$(mktemp -d "$parent_dir/spire-representative-preflight.XXXXXX")"
  trap 'rm -rf "$work_dir"; trap - RETURN' RETURN

  sample_input="$work_dir/sample-input"
  sample_output="$work_dir/sample-output"
  bad_summary="$work_dir/bad-summary"
  bad_recall_summary="$work_dir/bad-recall-summary"
  mkdir -p "$sample_input" "$sample_output" "$bad_summary" "$bad_recall_summary"

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

  cp "$priority_suite" "$sample_input/suite-representative-priority.json"
  cp "$pooling_suite" "$sample_input/suite-representative-pooling.json"
  "$summarizer" "$sample_input" "$sample_output" >/dev/null
  cp "$sample_input/suite-representative-priority.json" "$sample_output/"
  cp "$sample_input/suite-representative-pooling.json" "$sample_output/"
  "$verifier" "$sample_output" >/dev/null

  cp "$sample_output"/representative-*.tsv "$bad_summary"/
  cp "$sample_output"/suite-representative-*.json "$bad_summary"/
  awk 'BEGIN{FS=OFS="\t"} NR==1 {print; next} {$29=0; print}' \
    "$sample_output/representative-pooling-delta-summary.tsv" \
    > "$bad_summary/representative-pooling-delta-summary.tsv"

  if "$verifier" "$bad_summary" >/dev/null 2>&1; then
    die "representative summary verifier accepted missing p99 pooling latency improvement"
  fi

  cp "$sample_output"/representative-*.tsv "$bad_recall_summary"/
  cp "$sample_output"/suite-representative-*.json "$bad_recall_summary"/
  awk 'BEGIN{FS=OFS="\t"} NR==1 {print; next} ($1 == "recall" || $1 == "spire-pipeline") && $4 == 32 {$9=0.5} {print}' \
    "$sample_output/representative-latency-recall-summary.tsv" \
    > "$bad_recall_summary/representative-latency-recall-summary.tsv"

  if "$verifier" "$bad_recall_summary" >/dev/null 2>&1; then
    die "representative summary verifier accepted recall below the representative floor"
  fi
}

run_watchdog_gate_self_check() {
  local parent_dir="${SPIRE_AWS_REPRESENTATIVE_PREFLIGHT_WORKDIR:-$repo_root/target}"
  local work_dir short_watchdog

  mkdir -p "$parent_dir"
  work_dir="$(mktemp -d "$parent_dir/spire-representative-watchdog.XXXXXX")"
  trap 'rm -rf "$work_dir"; trap - RETURN' RETURN

  short_watchdog="$work_dir/run-pass-with-watchdog-short.sh"
  cat > "$short_watchdog" <<'SH'
case "$target" in
  pass-representative-performance-body)
    default_timeout=60
    ;;
esac
SH

  if SPIRE_AWS_REPRESENTATIVE_WATCHDOG="$short_watchdog" "$0" --watchdog-timeout-self-check >/dev/null 2>&1; then
    die "representative preflight accepted watchdog timeout below representative-tier minimum"
  fi
}

if [[ "${1:-}" == "--watchdog-timeout-self-check" ]]; then
  require_watchdog_timeout "pass-representative-performance-body" 14400
  exit 0
fi

require_file "$priority_suite"
require_file "$pooling_suite"
require_file "$makefile"
require_file "$watchdog"
require_executable "$summarizer"
require_executable "$verifier"
require_executable "$watchdog"
require_executable "$representative_pass_entrypoint"
if [[ -n "${ARTIFACT_DIR:-}" ]]; then
  require_representative_artifact_dir "$ARTIFACT_DIR"
fi

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
require_jq "representative priority suite recall thresholds" "$priority_suite" '
  any(.thresholds[];
    .step == "13a3a-recall-k10"
    and .metric == "recall"
    and .field == "recall@k"
    and .op == "gte"
    and .filters.nprobe == "32"
    and .value >= 0.95
  )
  and any(.thresholds[];
    .step == "13e3-production-read-profile-k10"
    and .metric == "spire-pipeline"
    and .field == "recall@k"
    and .op == "gte"
    and .filters.nprobe == "32"
    and .value >= 0.95
  )
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
require_jq "representative pooling suite recall thresholds" "$pooling_suite" '
  any(.thresholds[];
    .step == "13e4-pooling-disabled-profile-k10"
    and .metric == "spire-pipeline"
    and .field == "recall@k"
    and .op == "gte"
    and .filters.nprobe == "32"
    and .value >= 0.95
  )
  and any(.thresholds[];
    .step == "13e4-pooling-enabled-profile-k10"
    and .metric == "spire-pipeline"
    and .field == "recall@k"
    and .op == "gte"
    and .filters.nprobe == "32"
    and .value >= 0.95
  )
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
require_make_target_contains "pass-representative-performance" "run-pass-with-watchdog.sh pass-representative-performance-body"
require_make_target_order "pass-representative-performance-body" "preflight-representative-performance" "provision"
require_make_target_sequence \
  "pass-representative-performance-body" \
  "preflight-representative-performance" \
  "provision" \
  "install-extension" \
  "verify-representative-performance-tunneled"
require_make_target_sequence \
  "verify-representative-performance-tunneled" \
  "with-ssm-port-forwards.sh" \
  "load-representative" \
  "register-representative" \
  "smoke-representative" \
  "bench-representative-priority" \
  "bench-representative-pooling" \
  "summarize-representative-performance" \
  "verify-representative-performance-summary"
require_watchdog_timeout "pass-representative-performance-body" 14400
run_summary_gate_self_check
run_watchdog_gate_self_check

printf 'SPIRE representative performance preflight passed: priority=%s pooling=%s\n' \
  "$priority_suite" \
  "$pooling_suite"
