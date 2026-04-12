#!/usr/bin/env bash
# Verified real-corpus latency launcher for tqvector HNSW scans.
#
# This is a guarded wrapper around scripts/bench_sql_latency.sh. Before it
# starts a long real-corpus run, it checks a representative EXPLAIN plan and
# refuses to proceed unless the planner selects the exact tqhnsw index implied
# by --prefix/--m. This prevents silent Seq Scan + Sort measurements and also
# catches cases where the planner picks a different tqhnsw index than the one
# the operator intended to benchmark.
#
# The verified launcher is intentionally real-corpus-only and intentionally
# requires at most one effective m per invocation. Run it once per m value.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
delegate_script="${repo_root}/scripts/bench_sql_latency.sh"
PSQL_BIN="${TQV_PSQL_BIN:-psql}"

print_help() {
  cat <<'USAGE'
Usage:
  bash scripts/bench_sql_latency_verified.sh --prefix <prefix> [--m N] \
      [bench_sql_latency.sh real-corpus args...]

Behavior:
  - real-corpus only
  - requires at most one effective m per invocation (defaults to 8)
  - verifies a representative EXPLAIN plan uses <prefix>_m{N}_idx
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
m_count=0
forwarded_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      prefix="$2"
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

corpus_table="${prefix}_corpus"
query_table="${prefix}_queries"
index_name="${prefix}_m${declared_m}_idx"
k="${K:-10}"

probe_query="$("$PSQL_BIN" -X -A -t -q -c "SELECT source FROM ${query_table} ORDER BY id LIMIT 1;")"
if [[ -z "$probe_query" ]]; then
  echo "no probe query found in ${query_table}; did the loader run?" >&2
  exit 1
fi
if [[ "$probe_query" == *"'"* ]]; then
  echo "unexpected single quote in probe query literal output from ${query_table}" >&2
  exit 2
fi

plan_text="$("$PSQL_BIN" -X -A -t -q <<SQL
EXPLAIN
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${probe_query}'::real[]
LIMIT ${k};
SQL
)"

if ! grep -Fq "${index_name}" <<<"$plan_text"; then
  echo "planner verification failed for ${index_name}" >&2
  echo "expected the representative plan to use ${index_name}, but it did not." >&2
  echo "aborting before timing so this launcher does not record Seq Scan + Sort" >&2
  echo "or the wrong tqhnsw index for the requested m value." >&2
  echo >&2
  echo "Representative EXPLAIN plan:" >&2
  echo "${plan_text}" >&2
  exit 1
fi

echo "[verified] representative EXPLAIN uses ${index_name}" >&2
exec bash "${delegate_script}" "${forwarded_args[@]}"
