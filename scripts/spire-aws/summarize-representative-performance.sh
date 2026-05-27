#!/usr/bin/env bash
# Summarize representative SPIRE AWS latency/recall and pooling A/B suite output.

set -euo pipefail

artifact_dir="${1:?usage: summarize-representative-performance.sh <artifact-dir> [output-dir]}"
output_dir="${2:-$artifact_dir}"

representative_results="${artifact_dir}/suite-results-representative.jsonl"
pooling_results="${artifact_dir}/suite-results-representative-pooling.jsonl"

mkdir -p "$output_dir"

require_results() {
  local path="$1"
  if [[ ! -s "$path" ]]; then
    printf 'ERROR: expected non-empty suite results at %s\n' "$path" >&2
    exit 2
  fi
}

require_results "$representative_results"
require_results "$pooling_results"

latency_recall_out="${output_dir}/representative-latency-recall-summary.tsv"
profile_out="${output_dir}/representative-production-profile-summary.tsv"
pooling_out="${output_dir}/representative-pooling-comparison.tsv"

{
  printf 'source\tstep\tkind\tnprobe\tqueries_or_count\tlatency_p50\tlatency_p95\tlatency_p99\trecall_at_k\n'
  jq -r '
    select(.kind == "latency" and .metric == "latency")
    | [
        "latency",
        .step,
        .kind,
        (.values.nprobe // ""),
        (.values.count // ""),
        (.values.p50 // ""),
        (.values.p95 // ""),
        (.values.p99 // ""),
        ""
      ] | @tsv
  ' "$representative_results"
  jq -r '
    select(.kind == "recall" and .metric == "recall")
    | [
        "recall",
        .step,
        .kind,
        (.values.nprobe // ""),
        (.values.queries // ""),
        "",
        "",
        "",
        (.values["recall@k"] // "")
      ] | @tsv
  ' "$representative_results"
  jq -r '
    select(.kind == "spire-pipeline" and .metric == "spire-pipeline" and (.values.latency_p95? != null))
    | [
        "spire-pipeline",
        .step,
        .kind,
        (.values.nprobe // ""),
        (.values.queries // ""),
        (.values.latency_p50 // ""),
        (.values.latency_p95 // ""),
        (.values.latency_p99 // ""),
        (.values["recall@k"] // "")
      ] | @tsv
  ' "$representative_results"
} > "$latency_recall_out"

{
  printf 'step\tnprobe\tprofiles\tstatus\tresult_source\tdispatch_sum\tsocket_open_sum\tconnect_p50\tconnect_p95\tcandidate_p50\tcandidate_p95\theap_p50\theap_p95\tmerge_p50\tmerge_p95\ttotal_p50\ttotal_p95\ttimeout_sum\tcancel_sum\tdegraded_skip_sum\treturned_sum\n'
  jq -r '
    select(.kind == "spire-pipeline" and .metric == "spire-pipeline" and (.values.socket_open_sum? != null))
    | [
        .step,
        (.values.nprobe // ""),
        (.values.profiles // ""),
        (.values.status // ""),
        (.values.result_source // ""),
        (.values.dispatch_sum // ""),
        (.values.socket_open_sum // ""),
        (.values.connect_p50 // ""),
        (.values.connect_p95 // ""),
        (.values.candidate_p50 // ""),
        (.values.candidate_p95 // ""),
        (.values.heap_p50 // ""),
        (.values.heap_p95 // ""),
        (.values.merge_p50 // ""),
        (.values.merge_p95 // ""),
        (.values.total_p50 // ""),
        (.values.total_p95 // ""),
        (.values.timeout_sum // ""),
        (.values.cancel_sum // ""),
        (.values.degraded_skip_sum // ""),
        (.values.returned_sum // "")
      ] | @tsv
  ' "$representative_results"
} > "$profile_out"

{
  printf 'mode\tstep\tnprobe\tprofiles\tstatus\tresult_source\tdispatch_sum\tsocket_open_sum\tconnect_p50\tconnect_p95\ttotal_p50\ttotal_p95\tlatency_p50\tlatency_p95\tlatency_p99\trecall_at_k\n'
  jq -r '
    select(.kind == "spire-pipeline" and .metric == "spire-pipeline")
    | (
        if (.step | test("pooling-disabled")) then "disabled"
        elif (.step | test("pooling-enabled")) then "enabled"
        else "unknown"
        end
      ) as $mode
    | select($mode != "unknown")
    | select((.values.socket_open_sum? != null) or (.values.latency_p95? != null))
    | [
        $mode,
        .step,
        (.values.nprobe // ""),
        (.values.profiles // .values.queries // ""),
        (.values.status // ""),
        (.values.result_source // ""),
        (.values.dispatch_sum // ""),
        (.values.socket_open_sum // ""),
        (.values.connect_p50 // ""),
        (.values.connect_p95 // ""),
        (.values.total_p50 // ""),
        (.values.total_p95 // ""),
        (.values.latency_p50 // ""),
        (.values.latency_p95 // ""),
        (.values.latency_p99 // ""),
        (.values["recall@k"] // "")
      ] | @tsv
  ' "$pooling_results"
} > "$pooling_out"

printf 'wrote %s\n' "$latency_recall_out"
printf 'wrote %s\n' "$profile_out"
printf 'wrote %s\n' "$pooling_out"
