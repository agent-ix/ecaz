#!/usr/bin/env bash
# Summarize representative SPIRE AWS latency/recall and pooling A/B suite output.

set -euo pipefail

artifact_dir="${1:?usage: summarize-representative-performance.sh <artifact-dir> [output-dir]}"
output_dir="${2:-$artifact_dir}"

representative_results="${artifact_dir}/suite-results-representative-priority.jsonl"
if [[ ! -s "$representative_results" ]]; then
  representative_results="${artifact_dir}/suite-results-representative.jsonl"
fi
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
pooling_delta_out="${output_dir}/representative-pooling-delta-summary.tsv"

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
  printf 'step\tnprobe\tprofiles\tstatus\tresult_source\tdispatch_sum\tsocket_open_sum\tconnect_p50\tconnect_p95\tendpoint_identity_p50\tendpoint_identity_p95\tcandidate_p50\tcandidate_p95\theap_p50\theap_p95\tmerge_p50\tmerge_p95\ttotal_p50\ttotal_p95\tcandidate_query_sum\theap_query_sum\tendpoint_identity_query_sum\tpayload_bytes_sum\ttimeout_sum\tcancel_sum\tdegraded_skip_sum\treturned_sum\n'
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
        (.values.endpoint_identity_p50 // ""),
        (.values.endpoint_identity_p95 // ""),
        (.values.candidate_p50 // ""),
        (.values.candidate_p95 // ""),
        (.values.heap_p50 // ""),
        (.values.heap_p95 // ""),
        (.values.merge_p50 // ""),
        (.values.merge_p95 // ""),
        (.values.total_p50 // ""),
        (.values.total_p95 // ""),
        (.values.candidate_query_sum // ""),
        (.values.heap_query_sum // ""),
        (.values.endpoint_identity_query_sum // ""),
        (.values.payload_bytes_sum // ""),
        (.values.timeout_sum // ""),
        (.values.cancel_sum // ""),
        (.values.degraded_skip_sum // ""),
        (.values.returned_sum // "")
      ] | @tsv
  ' "$representative_results"
} > "$profile_out"

{
  printf 'mode\tstep\tnprobe\tprofiles\tstatus\tresult_source\tdispatch_sum\tsocket_open_sum\tconnect_p50\tconnect_p95\tendpoint_identity_p50\tendpoint_identity_p95\ttotal_p50\ttotal_p95\tlatency_p50\tlatency_p95\tlatency_p99\trecall_at_k\tendpoint_identity_query_sum\n'
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
        (.values.endpoint_identity_p50 // ""),
        (.values.endpoint_identity_p95 // ""),
        (.values.total_p50 // ""),
        (.values.total_p95 // ""),
        (.values.latency_p50 // ""),
        (.values.latency_p95 // ""),
        (.values.latency_p99 // ""),
        (.values["recall@k"] // ""),
        (.values.endpoint_identity_query_sum // "")
      ] | @tsv
  ' "$pooling_results"
} > "$pooling_out"

jq -s -r '
  def mode:
    if (.step | test("pooling-disabled")) then "disabled"
    elif (.step | test("pooling-enabled")) then "enabled"
    else "unknown"
    end;
  def parse_ms($key):
    try (.values[$key] | tostring | sub(" ms$"; "") | tonumber) catch null;
  def parse_num($key):
    try (.values[$key] | tostring | tonumber) catch null;
  def row:
    {
      mode: mode,
      nprobe: (.values.nprobe // ""),
      profiles_or_queries: (.values.profiles // .values.queries // ""),
      latency_p50_ms: parse_ms("latency_p50"),
      latency_p95_ms: parse_ms("latency_p95"),
      latency_p99_ms: parse_ms("latency_p99"),
      recall_at_k: parse_num("recall@k"),
      socket_open_sum: parse_num("socket_open_sum"),
      dispatch_sum: parse_num("dispatch_sum"),
      endpoint_identity_query_sum: parse_num("endpoint_identity_query_sum"),
      connect_p50_ms: parse_ms("connect_p50"),
      connect_p95_ms: parse_ms("connect_p95"),
      endpoint_identity_p50_ms: parse_ms("endpoint_identity_p50"),
      endpoint_identity_p95_ms: parse_ms("endpoint_identity_p95"),
      total_p50_ms: parse_ms("total_p50"),
      total_p95_ms: parse_ms("total_p95")
    };
  def merge_rows:
    reduce .[] as $row ({};
      .mode = $row.mode
      | .nprobe = $row.nprobe
      | (if ($row.profiles_or_queries // "") != "" then .profiles_or_queries = $row.profiles_or_queries else . end)
      | (if $row.latency_p50_ms != null then .latency_p50_ms = $row.latency_p50_ms else . end)
      | (if $row.latency_p95_ms != null then .latency_p95_ms = $row.latency_p95_ms else . end)
      | (if $row.latency_p99_ms != null then .latency_p99_ms = $row.latency_p99_ms else . end)
      | (if $row.recall_at_k != null then .recall_at_k = $row.recall_at_k else . end)
      | (if $row.socket_open_sum != null then .socket_open_sum = $row.socket_open_sum else . end)
      | (if $row.dispatch_sum != null then .dispatch_sum = $row.dispatch_sum else . end)
      | (if $row.endpoint_identity_query_sum != null then .endpoint_identity_query_sum = $row.endpoint_identity_query_sum else . end)
      | (if $row.connect_p50_ms != null then .connect_p50_ms = $row.connect_p50_ms else . end)
      | (if $row.connect_p95_ms != null then .connect_p95_ms = $row.connect_p95_ms else . end)
      | (if $row.endpoint_identity_p50_ms != null then .endpoint_identity_p50_ms = $row.endpoint_identity_p50_ms else . end)
      | (if $row.endpoint_identity_p95_ms != null then .endpoint_identity_p95_ms = $row.endpoint_identity_p95_ms else . end)
      | (if $row.total_p50_ms != null then .total_p50_ms = $row.total_p50_ms else . end)
      | (if $row.total_p95_ms != null then .total_p95_ms = $row.total_p95_ms else . end)
    );
  def fmt:
    if . == null then "" else tostring end;
  def delta($disabled; $enabled; $key):
    if ($disabled[$key] != null and $enabled[$key] != null)
    then ($disabled[$key] - $enabled[$key])
    else null
    end;
  def pct($disabled; $enabled; $key):
    if ($disabled[$key] != null and $enabled[$key] != null and $disabled[$key] != 0)
    then ((($disabled[$key] - $enabled[$key]) / $disabled[$key]) * 100)
    else null
    end;

  ([
    "nprobe",
    "disabled_profiles_or_queries",
    "enabled_profiles_or_queries",
    "disabled_socket_open_sum",
    "enabled_socket_open_sum",
    "socket_open_delta",
    "socket_open_reduction_pct",
    "disabled_endpoint_identity_query_sum",
    "enabled_endpoint_identity_query_sum",
    "endpoint_identity_query_delta",
    "disabled_endpoint_identity_p95_ms",
    "enabled_endpoint_identity_p95_ms",
    "disabled_connect_p95_ms",
    "enabled_connect_p95_ms",
    "connect_p95_delta_ms",
    "disabled_total_p95_ms",
    "enabled_total_p95_ms",
    "total_p95_delta_ms",
    "disabled_latency_p95_ms",
    "enabled_latency_p95_ms",
    "latency_p95_delta_ms",
    "latency_p95_reduction_pct",
    "disabled_recall_at_k",
    "enabled_recall_at_k",
    "recall_delta"
  ] | @tsv),
  (
    map(select(.kind == "spire-pipeline" and .metric == "spire-pipeline"))
    | map(row)
    | map(select(.mode != "unknown" and .nprobe != ""))
    | sort_by(.mode, .nprobe)
    | group_by(.mode, .nprobe)
    | map(merge_rows)
    | sort_by((.nprobe | tonumber), .mode)
    | group_by(.nprobe)
    | map({
        nprobe: .[0].nprobe,
        disabled: (map(select(.mode == "disabled")) | first),
        enabled: (map(select(.mode == "enabled")) | first)
      })
    | map(select(.disabled != null and .enabled != null))
    | sort_by(.nprobe | tonumber)
    | .[]
    | [
        .nprobe,
        (.disabled.profiles_or_queries // ""),
        (.enabled.profiles_or_queries // ""),
        (.disabled.socket_open_sum | fmt),
        (.enabled.socket_open_sum | fmt),
        (delta(.disabled; .enabled; "socket_open_sum") | fmt),
        (pct(.disabled; .enabled; "socket_open_sum") | fmt),
        (.disabled.endpoint_identity_query_sum | fmt),
        (.enabled.endpoint_identity_query_sum | fmt),
        (delta(.disabled; .enabled; "endpoint_identity_query_sum") | fmt),
        (.disabled.endpoint_identity_p95_ms | fmt),
        (.enabled.endpoint_identity_p95_ms | fmt),
        (.disabled.connect_p95_ms | fmt),
        (.enabled.connect_p95_ms | fmt),
        (delta(.disabled; .enabled; "connect_p95_ms") | fmt),
        (.disabled.total_p95_ms | fmt),
        (.enabled.total_p95_ms | fmt),
        (delta(.disabled; .enabled; "total_p95_ms") | fmt),
        (.disabled.latency_p95_ms | fmt),
        (.enabled.latency_p95_ms | fmt),
        (delta(.disabled; .enabled; "latency_p95_ms") | fmt),
        (pct(.disabled; .enabled; "latency_p95_ms") | fmt),
        (.disabled.recall_at_k | fmt),
        (.enabled.recall_at_k | fmt),
        (delta(.disabled; .enabled; "recall_at_k") | fmt)
      ] | @tsv
  )
' "$pooling_results" > "$pooling_delta_out"

printf 'wrote %s\n' "$latency_recall_out"
printf 'wrote %s\n' "$profile_out"
printf 'wrote %s\n' "$pooling_out"
printf 'wrote %s\n' "$pooling_delta_out"
