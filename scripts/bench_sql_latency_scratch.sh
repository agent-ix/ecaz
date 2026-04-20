#!/usr/bin/env bash
# Run bench_sql_latency.sh against the repo-local pgrx scratch cluster.
#
# This is the bench-time twin of scripts/load_real_corpus_scratch.sh: it
# pins the same socket, port, database, and psql binary the loader uses, so
# a one-shot "load then bench" against the scratch pg17 cluster does not
# need any per-run env setup.
#
# All arguments are forwarded verbatim to scripts/bench_sql_latency.sh, so
# this wrapper is the canonical entry point for both the legacy synthetic
# fixture mode and the new --prefix real-corpus mode.
#
# Example:
#   scripts/bench_sql_latency_scratch.sh \
#       --prefix ec_hnsw_real_10k --m 8 --m 16 \
#       --ef-search 40,64,100,128,160,200 \
#       --cache-state cold \
#       --output /tmp/nfr1_real_10k.summary > /tmp/nfr1_real_10k.stdout
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"
socket_dir=""
port=""
database="${PGDATABASE:-postgres}"
forwarded_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --db)
      database="$2"
      shift 2
      ;;
    --socket-dir)
      socket_dir="$2"
      shift 2
      ;;
    --port)
      port="$2"
      shift 2
      ;;
    *)
      forwarded_args+=("$1")
      shift
      ;;
  esac
done

if [[ -n "$socket_dir" ]]; then
  export PGHOST="$socket_dir"
elif [[ -z "${PGHOST:-}" ]]; then
  export PGHOST="$("${script_dir}/resolve_scratch_socket_dir.sh")"
fi
if [[ -n "$port" ]]; then
  export PGPORT="$port"
else
  export PGPORT="${PGPORT:-28817}"
fi
export PGDATABASE="$database"
export TQV_PSQL_BIN="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"

exec bash "${repo_root}/scripts/bench_sql_latency.sh" "${forwarded_args[@]}"
