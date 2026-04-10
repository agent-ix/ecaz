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
#       --prefix tqhnsw_real_10k --m 8 --m 16 \
#       --ef-search 40,64,100,128,160,200
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"

export PGHOST="${PGHOST:-/tmp/tqvector_pgrx_home}"
export PGPORT="${PGPORT:-28817}"
export PGDATABASE="${PGDATABASE:-postgres}"
export TQV_PSQL_BIN="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"

exec bash "${repo_root}/scripts/bench_sql_latency.sh" "$@"
