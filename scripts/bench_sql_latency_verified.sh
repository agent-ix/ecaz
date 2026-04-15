#!/usr/bin/env bash
# Verified real-corpus latency launcher for tqvector HNSW scans.
#
# This is a guarded wrapper around scripts/bench_sql_latency.sh. It exports the
# exact tqhnsw index implied by --prefix/--m and the delegate script verifies,
# for every (m, ef_search) cell, that the measured query still plans on that
# index. This prevents silent Seq Scan + Sort measurements and also catches
# cases where the planner picks a different tqhnsw index than the one the
# operator intended to benchmark.
#
# The verified launcher is intentionally real-corpus-only and intentionally
# requires at most one effective m per invocation. Run it once per m value.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
delegate_script="${repo_root}/scripts/bench_sql_latency.sh"

print_help() {
  cat <<'USAGE'
Usage:
  bash scripts/bench_sql_latency_verified.sh --prefix <prefix> [--m N] \
      [--corpus-table NAME] [--query-table NAME] [--index-name NAME] \
      [bench_sql_latency.sh real-corpus args...]

Behavior:
  - real-corpus only
  - requires at most one effective m per invocation (defaults to 8)
  - verifies every measured (m, ef_search) cell uses <prefix>_m{N}_idx
    or the explicit --index-name override
  - aborts before timing if the plan falls back to Seq Scan + Sort or picks
    a different index than expected

Example:
  bash scripts/bench_sql_latency_verified.sh \
      --prefix tqhnsw_real_10k \
      --m 8 \
      --ef-search 40,64,100,128,160,200 \
      --cache-state cold \
      --output /tmp/nfr1_real_10k_m8.summary > /tmp/nfr1_real_10k_m8.stdout
USAGE
}

prefix=""
declared_m=""
declared_index_name=""
m_count=0
forwarded_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      prefix="$2"
      forwarded_args+=("$1" "$2")
      shift 2
      ;;
    --corpus-table|--query-table)
      forwarded_args+=("$1" "$2")
      shift 2
      ;;
    --index-name)
      declared_index_name="$2"
      forwarded_args+=("$1" "$2")
      shift 2
      ;;
    --m)
      declared_m="$2"
      m_count=$((m_count + 1))
      forwarded_args+=("$1" "$2")
      shift 2
      ;;
    -h|--help)
      print_help
      exit 0
      ;;
    *)
      forwarded_args+=("$1")
      shift
      ;;
  esac
done

if [[ -z "$prefix" ]]; then
  echo "missing required --prefix argument" >&2
  print_help >&2
  exit 2
fi
if [[ ! "$prefix" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
  echo "invalid prefix: $prefix" >&2
  exit 2
fi
if (( m_count > 1 )); then
  echo "bench_sql_latency_verified.sh accepts at most one --m per run; invoke it separately for each index" >&2
  exit 2
fi
if [[ -z "$declared_m" ]]; then
  declared_m="8"
fi
if [[ ! "$declared_m" =~ ^[0-9]+$ ]]; then
  echo "invalid m value: $declared_m" >&2
  exit 2
fi
if [[ -n "$declared_index_name" && ! "$declared_index_name" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
  echo "invalid index name: $declared_index_name" >&2
  exit 2
fi

index_name="${declared_index_name:-${prefix}_m${declared_m}_idx}"
export TQV_REQUIRE_INDEX_NAME="${index_name}"
echo "[verified] requiring planner use ${index_name} for every measured cell" >&2
exec bash "${delegate_script}" "${forwarded_args[@]}"
